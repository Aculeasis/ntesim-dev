use super::*;
use serde_json::json;

fn invoke(time: u64, actor: usize, ability: &str, packet: &str, delay: u64) -> GameActionCommand {
    GameActionCommand::InvokeAbility {
        timestamp_us: time,
        actor_index: actor,
        ability_id: ability.into(),
        damage_effect_id: packet.into(),
        array_index: 0,
        atk_rate: Ratio::one(),
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost: 0,
        recovery_us: 0,
        cooldown_us: 0,
        hit_delay_us: delay,
    }
}
fn switch(time: u64, actor: usize) -> GameActionCommand {
    GameActionCommand::SwitchActor {
        timestamp_us: time,
        target_actor_index: actor,
        cooldown_us: 0,
    }
}

#[test]
fn typed_admitted_costs_bind_commands_and_leave_rejected_actions_unspent() {
    let mut request = research_request();
    request.stochastic_crit = None;
    request.actions.truncate(3);
    request.action_metadata.retain(|index, _| *index < 3);
    for (action, cost) in request.actions.iter_mut().zip([40, 60, 1]) {
        if let GameActionCommand::InvokeAbility { energy_cost, .. } = action {
            *energy_cost = cost;
        }
    }
    let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["success"], false);
    let costs = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|event| event.get("energy_change"))
        .collect::<Vec<_>>();
    assert_eq!(costs.len(), 2);
    for (index, (before, spent, after)) in [(100, 40, 60), (60, 60, 0)].into_iter().enumerate() {
        assert_eq!(
            costs[index],
            &json!({"source":"ability_cost","source_actor_index":0,"recipient_actor_index":0,"command_index":index,"ability_id":"caller-test","energy_before":before,"spent":spent,"energy_after":after})
        );
        let typed: GameEnergyReceipt = serde_json::from_value(costs[index].clone()).unwrap();
        assert!(matches!(typed, GameEnergyReceipt::Spend(_)));
    }
    assert_eq!(report["final_actor_states"][0]["spent_energy"], 100);
    assert_eq!(report["final_actor_states"][0]["energy"], 0);
    if let GameActionCommand::InvokeAbility { energy_cost, .. } = &mut request.actions[0] {
        *energy_cost = 101;
    }
    let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event.get("energy_change").is_some())
            .count(),
        0
    );
    assert_eq!(report["final_actor_states"][0]["spent_energy"], 0);
    // Zero-cost inputs still identify an actual admission without inventing a grant.
    request = research_request();
    let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["trace"][0]["energy_change"]["spent"], 0);
    assert_eq!(report["trace"][0]["energy_change"]["energy_before"], 100);
    assert_eq!(report["trace"][0]["energy_change"]["energy_after"], 100);
}

fn continuing_energy_request() -> GameScenarioRequest {
    let mut request = sample_s1_request();
    request.action_metadata.clear();
    let base = request.actors[0].clone();
    request.actors = [
        ("character:1046", 2, 98),
        ("character:1010", 1, 99),
        ("character:1003", 5, 0),
    ]
    .into_iter()
    .enumerate()
    .map(|(slot, (id, awakening, initial_energy))| {
        let mut actor = base.clone();
        actor.slot = slot;
        actor.character_id = id.into();
        actor.awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        actor.initial_energy = initial_energy;
        actor.weapon_id.clear();
        actor.set_id = None;
        actor.gear_procs.clear();
        actor
    })
    .collect();
    request.actors[1]
        .gear_procs
        .push(GearProcConfig::ChargeEfficiency {
            multiplier: Ratio {
                numerator: 28,
                denominator: 25,
            },
        });
    request.targets[0].airborne_until_us = 1000;
    request.relative_duration_us = 1000;
    request.actions = vec![
        invoke(
            0,
            0,
            "GA_Female046_Skill",
            "GE_Player_Female046_Skill1_Damage",
            200,
        ),
        switch(0, 1),
        invoke(
            10,
            1,
            "GA_Nanally_UltraSkill",
            "GE_Player_Nanally_Cat_AirAttack_Damage",
            200,
        ),
        switch(10, 2),
        invoke(
            20,
            2,
            "GA_Sagiri_Skill",
            "GE_Player_Sagiri_Skill1_Damage",
            100,
        ),
        switch(30, 0),
        switch(40, 1),
        invoke(
            50,
            1,
            "GA_Nanally_UltraSkill",
            "GE_Player_Nanally_Cat_AirAttack_Damage",
            200,
        ),
        switch(50, 0),
        switch(60, 1),
        invoke(
            70,
            1,
            "GA_Nanally_UltraSkill",
            "GE_Player_Nanally_Cat_AirAttack_Damage",
            200,
        ),
        switch(70, 0),
        invoke(
            80,
            0,
            "GA_Female046_UltraSkill",
            "GE_Player_Female046_UltraSkill1_Damage",
            0,
        ),
    ];
    // Space out the timed handoffs and preserve the two delayed deliveries.
    for action in &mut request.actions {
        match action {
            GameActionCommand::InvokeAbility {
                timestamp_us,
                hit_delay_us,
                ..
            } => {
                if *timestamp_us <= 10 {
                    *hit_delay_us += 600_000;
                } else {
                    *timestamp_us += 600_000;
                }
            }
            GameActionCommand::SwitchActor { timestamp_us, .. } if *timestamp_us > 10 => {
                *timestamp_us += 600_000;
            }
            _ => {}
        }
    }
    request.targets[0].airborne_until_us += 600_000;
    request.relative_duration_us += 600_000;
    // Earlier launched packets keep their independent delivery times.
    for action in &mut request.actions {
        let timestamp_us = match action {
            GameActionCommand::InvokeAbility { timestamp_us, .. }
            | GameActionCommand::SwitchActor { timestamp_us, .. } => timestamp_us,
            _ => continue,
        };
        if *timestamp_us > 600_030 {
            *timestamp_us += 600_000;
        }
    }
    request.targets[0].airborne_until_us += 600_000;
    request.relative_duration_us += 600_000;
    // The first Nanally entry now admits a 1.31s Support recovery. Already
    // launched Zero packets retain their delivery times; authored later inputs wait.
    for action in &mut request.actions {
        let timestamp_us = match action {
            GameActionCommand::InvokeAbility { timestamp_us, .. }
            | GameActionCommand::SwitchActor { timestamp_us, .. } => timestamp_us,
            _ => continue,
        };
        if *timestamp_us > 0 {
            *timestamp_us += 1_310_000;
        }
    }
    request.targets[0].airborne_until_us += 1_310_000;
    request.relative_duration_us += 1_310_000;
    if let GameActionCommand::InvokeAbility { energy_cost, .. } =
        request.actions.last_mut().unwrap()
    {
        *energy_cost = 100;
    }
    // Zero's paid Skill->Anima handoff also schedules a real Blossom tail.
    request.relative_duration_us = 12_000_000;
    request
}

fn assert_packet_energy_ledger(grants: &[&serde_json::Value]) {
    // Independent exact-carry oracle for the actual delivery order. Each row is
    // one native packet, with gross whole units [Zero, Nanally, Sakiri]. Nanally
    // alone has 28/25 efficiency; off-source recipients multiply raw charge by 3/5.
    for (index, source, owner, num, den, grosses) in [
        (1, "zero_hit", 0, 587, 1000, [0, 0, 0]),
        (2, "zero_hit", 0, 587, 1000, [1, 0, 0]),
        (3, "zero_hit", 0, 3683, 1000, [3, 3, 2]),
        (4, "zero_hit", 0, 493, 125, [4, 2, 3]),
        (5, "nanally_hit", 1, 7, 1, [5, 8, 4]),
        (6, "sakiri_hit", 2, 1951, 500, [2, 3, 4]),
        (8, "sakiri_hit", 2, 2699, 500, [3, 4, 5]),
    ] {
        let fanout = &grants[index]["energy_fanout"];
        assert_eq!(fanout["source"], source);
        assert_eq!(fanout["source_actor_index"], owner);
        assert_eq!(fanout["base"], json!(format!("{}/{}", num, den)));
        let recipients = fanout["recipients"].as_array().unwrap();
        assert_eq!(recipients.len(), 3);
        for (recipient, gross) in grosses.into_iter().enumerate() {
            let entry = &recipients[recipient];
            assert_eq!(entry["recipient_actor_index"], recipient);
            assert_eq!(entry["gross"], gross);
        }
    }
    // Manual Burst/summon commands create no A1 grant; only actual follow-up groups qualify.
    assert!(grants.iter().all(|event| {
        event["energy_change"]["source"] != "nanally_a1"
            && event["energy_fanout"]["source"] != "nanally_a1"
    }));
    assert_eq!(
        grants[7]["energy_change"],
        json!({
            "source":"sakiri_a5", "source_actor_index":2, "recipient_actor_index":2,
            "energy_before":13, "gross":87, "retained":87, "overflow":0, "energy_after":100,
            "charge_remainder_before":"1913/5000",
            "charge_remainder_after":"1913/5000",
        })
    );
}

#[test]
fn typed_energy_tracks_off_field_recipients_carry_overflow_and_later_spending() {
    let request = continuing_energy_request();
    let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let grants: Vec<_> = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| {
            event.get("energy_fanout").is_some()
                || (event.get("energy_change").is_some()
                    && event["energy_change"]["source"] != "ability_cost")
        })
        .collect();
    assert_eq!(grants.len(), 9);
    for event in &grants {
        if let Some(grant) = event.get("energy_change") {
            assert_ne!(event["active_actor"], grant["recipient_actor_index"]);
            assert_eq!(grant["source_actor_index"], grant["recipient_actor_index"]);
        } else {
            let fanout = &event["energy_fanout"];
            assert!(
                ["sakiri_hit", "zero_hit", "nanally_hit"]
                    .iter()
                    .any(|source| fanout["source"] == *source)
            );
            assert_eq!(fanout["recipients"].as_array().unwrap().len(), 3);
        }
    }
    assert_eq!(
        grants[0]["energy_change"],
        json!({
            "source": "zero_a2", "source_actor_index": 0, "recipient_actor_index": 0,
            "energy_before": 98, "gross": 8, "retained": 2, "overflow": 6, "energy_after": 100,
            "charge_basis": "8/1",
        })
    );
    assert_packet_energy_ledger(&grants);
    let states = &report["final_actor_states"];
    // Nanally receives Zero and Sakiri's authored Skills as a peer,
    // plus her own paid Support, for 20.003872.
    assert_eq!(states[0]["energy"], 0);
    assert_eq!(states[0]["spent_energy"], 100);
    assert_eq!(states[1]["granted_energy"], 20);
    assert_eq!(states[1]["overflow_energy"], 19);
    assert_eq!(states[1]["charge_remainder"], json!("121/31250"));
    assert_eq!(states[2]["energy"], 100);
    assert_eq!(
        report,
        run_game_scenario(&request, &ExecutionLimits::default()).unwrap()
    );
}

#[test]
fn typed_weapon_and_mint_grants_preserve_capped_and_zero_headroom_transitions() {
    let mut request = sample_s1_request();
    request.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(6);
    request.actors[0].initial_energy = 99;
    request.actions = vec![request.actions[1].clone()];
    let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    let grants: Vec<_> = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|event| event.get("energy_change"))
        .filter(|change| change["source"] != "ability_cost")
        .collect();
    assert_eq!(grants.len(), 4);
    assert_eq!(grants[0]["source"], "mint_hit");
    assert_eq!(grants[0]["gross"], 2);
    assert_eq!(grants[0]["retained"], 1);
    assert_eq!(grants[0]["overflow"], 1);
    assert_eq!(grants[1]["source"], "mind_royale");
    assert_eq!(grants[1]["gross"], 18);
    assert_eq!(grants[1]["retained"], 0);
    assert_eq!(grants[1]["overflow"], 18);
    assert_eq!(grants[2]["source"], "mint_a6");
    assert_eq!(grants[2]["gross"], 2);
    assert_eq!(grants[2]["retained"], 0);
    assert_eq!(grants[2]["overflow"], 2);
    assert_eq!(grants[3]["source"], "mint_hit");
    assert_eq!(grants[3]["gross"], 2);
    assert_eq!(grants[3]["retained"], 0);
    assert_eq!(grants[3]["overflow"], 2);
}

#[test]
fn mind_royale_skips_pending_energy_after_same_time_fatal_damage() {
    let mut request = sample_s1_request();
    request.actions = vec![request.actions[1].clone()];
    if let GameActionCommand::InvokeAbility { timestamp_us, .. } = &mut request.actions[0] {
        *timestamp_us = 0;
    }
    request.incoming_damage_events = vec![GameActorDamageEvent {
        timestamp_us: 0,
        actor_index: 0,
        delivered_damage: request.actors[0].initial_hp,
    }];
    request.model_assumptions.insert(
        super::super::events::INCOMING_DAMAGE_HYPOTHESIS_ID.into(),
        super::super::events::INCOMING_DAMAGE_HYPOTHESIS_ID
            .description()
            .into(),
    );
    let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["final_actor_states"][0]["hp"], 0);
    assert_eq!(report["final_actor_states"][0]["granted_energy"], 0);
    assert_eq!(report["final_actor_states"][0]["overflow_energy"], 0);
    assert!(
        !report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["event_kind"] == "mind_royale_energy")
    );
}

#[test]
fn mind_royale_living_off_field_recipient_keeps_efficiency_carry_and_overflow() {
    let mut request = sample_s1_request();
    request.actors[0].initial_energy = 95;
    request.actors[0]
        .gear_procs
        .push(GearProcConfig::ChargeEfficiency {
            multiplier: Ratio {
                numerator: 3,
                denominator: 2,
            },
        });
    let mut teammate = request.actors[0].clone();
    teammate.slot = 1;
    teammate.gear_procs.clear();
    request.actors.push(teammate);
    let mut state = super::super::state::GameSimulatorState::new(&request);
    state.active_actor_index = 1;
    state.actors[0].charge_remainder = ExactFraction::new(1, 3).unwrap();
    let mut trace = Vec::new();
    crate::callbacks::apply(
        crate::callbacks::ContentEvent::MindRoyaleEnergyGrant {
            actor_index: 0,
            amount: 18,
        },
        &mut crate::runtime_api::CallbackContext {
            state: &mut state,
            request: &request,
            queue: &mut crate::simulation::timer_queue::EventQueue::new(),
            sequence: &mut 0,
            trace: &mut trace,
        },
    )
    .unwrap();
    // 18 * 3/2 + 1/3 => 27 whole, 1/3 carry; headroom 5 => overflow 22.
    assert_eq!(state.actors[0].granted_energy, 27);
    assert_eq!(state.actors[0].overflow_energy, 22);
    assert_eq!(state.actors[0].energy.current(), 100);
    assert_eq!(
        state.actors[0].charge_remainder,
        ExactFraction::new(1, 3).unwrap()
    );
    assert_eq!(state.actors[1].granted_energy, 0);
    assert_eq!(trace.len(), 1);
    state.actors[0].hp = 0;
    let frozen = state.actors[0].clone();
    crate::callbacks::apply(
        crate::callbacks::ContentEvent::MindRoyaleEnergyGrant {
            actor_index: 0,
            amount: 18,
        },
        &mut crate::runtime_api::CallbackContext {
            state: &mut state,
            request: &request,
            queue: &mut crate::simulation::timer_queue::EventQueue::new(),
            sequence: &mut 0,
            trace: &mut trace,
        },
    )
    .unwrap();
    assert_eq!(state.actors[0].granted_energy, frozen.granted_energy);
    assert_eq!(state.actors[0].overflow_energy, frozen.overflow_energy);
    assert_eq!(state.actors[0].charge_remainder, frozen.charge_remainder);
    assert_eq!(trace.len(), 1);
}
