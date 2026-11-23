use super::*;
use crate::simulation::tests::support::formula_damage_total;

#[test]
fn baicang_zero_start_normal_skill_burst_and_continuation_rotation() {
    let req = baicang_zero_start_rotation_request();
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 76);
    assert_eq!(
        serde_json::json!(formula_damage_total(&report)),
        json!("32772/1")
    );
    assert_baicang_rotation_damage(&report);
    assert_baicang_rotation_energy(&req, &report);
}

fn baicang_zero_start_rotation_request() -> GameScenarioRequest {
    let mut req = request();
    req.actors[0].talent_levels = Some([8; 3]);
    req.actors[0].initial_energy = 0;
    req.actors[0].energy_capacity = 120;
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    for (slot, character_id) in [(1, "character:1019"), (2, "character:1003")] {
        let mut ally = req.actors[0].clone();
        ally.slot = slot;
        ally.character_id = character_id.into();
        req.actors.push(ally);
    }
    assert_eq!(req.actors.len(), 3);
    req.targets[0].hp_max = 10_000_000;
    req.targets[0].hp_current = 10_000_000;
    req.relative_duration_us = 60_000_000;
    req.model_assumptions.insert(
        crate::simulation::energy::BAICANG_HIT_ENERGY_HYPOTHESIS_ID.into(),
        crate::simulation::energy::BAICANG_HIT_ENERGY_HYPOTHESIS_ID
            .description()
            .into(),
    );

    req.actions = baicang_zero_start_rotation_actions();
    req
}

fn baicang_zero_start_rotation_actions() -> Vec<GameActionCommand> {
    let mut actions = Vec::new();
    let mut now = 0_u64;
    for chain in 0..5 {
        push_baicang_normal_chain(&mut actions, &mut now);
        if chain < 3 {
            push_baicang_skill(&mut actions, &mut now);
        }
    }
    assert_eq!(now, 41_600_000);
    push_baicang_burst(&mut actions, now);
    now += 14_000_000; // Burst recovery and its 12s modeled domain interval.
    assert_eq!(now, 55_600_000);
    push_baicang_normal_chain(&mut actions, &mut now);
    assert_eq!(now, 59_600_000);
    actions
}

fn push_baicang_normal_chain(actions: &mut Vec<GameActionCommand>, now: &mut u64) {
    for _ in 0..5 {
        let mut action = make_action(
            *now,
            0,
            "GA_Cang_Melee",
            "GE_Player_Cang_Melee1_Damage",
            0,
            0,
        );
        if let GameActionCommand::InvokeAbility { recovery_us, .. } = &mut action {
            *recovery_us = 800_000;
        }
        actions.push(action);
        *now += 800_000;
    }
}

fn push_baicang_skill(actions: &mut Vec<GameActionCommand>, now: &mut u64) {
    let mut skill = make_action(
        *now,
        0,
        "GA_Cang_Skill",
        "GE_Player_Cang_Skill_Damage",
        0,
        6_000_000,
    );
    if let GameActionCommand::InvokeAbility { recovery_us, .. } = &mut skill {
        *recovery_us = 1_200_000;
    }
    actions.push(skill);
    *now += 7_200_000; // 1.2s recovery plus the authored 6s wait.
}

fn push_baicang_burst(actions: &mut Vec<GameActionCommand>, now: u64) {
    let mut burst = make_action(
        now,
        0,
        "GA_Cang_UltraSkill",
        "GE_Player_Cang_UltraSkill2_Damage",
        120,
        20_000_000,
    );
    if let GameActionCommand::InvokeAbility { recovery_us, .. } = &mut burst {
        *recovery_us = 2_000_000;
    }
    actions.push(burst);
}

fn assert_baicang_rotation_damage(report: &serde_json::Value) {
    let hits = report["hit_receipts"].as_array().unwrap();
    assert_baicang_normal_packet_damage(hits);
    assert_baicang_skill_packet_damage(hits);
    assert_baicang_burst_packet_damage(hits);
}

fn assert_baicang_normal_packet_damage(hits: &[serde_json::Value]) {
    // Independent level-8, A0, 1000 ATK, 0 RES, half-DEF packet oracle.
    // Each Normal chain contributes 4033.5 across the eleven stage packets.
    let normal_packet_damage = [
        ("GE_Player_Cang_Melee1_Damage", 407, 1),
        ("GE_Player_Cang_Melee2_Damage", 725, 2),
        ("GE_Player_Cang_Melee3_Damage", 2039, 2),
        ("GE_Player_Cang_Melee4_Damage", 215, 1),
        ("GE_Player_Cang_Melee5_Damage", 1479, 2),
    ];
    for (effect, numerator, denominator) in normal_packet_damage {
        let matching = hits
            .iter()
            .filter(|hit| hit["damage_effect_id"] == effect)
            .collect::<Vec<_>>();
        let expected_count = if effect == "GE_Player_Cang_Melee4_Damage" {
            42
        } else {
            6
        };
        assert_eq!(matching.len(), expected_count, "{effect}");
        for hit in matching {
            assert_eq!(
                hit["non_crit_damage"],
                json!(format!("{}/{}", numerator, denominator)),
                "{effect}"
            );
        }
    }
}

fn assert_baicang_skill_packet_damage(hits: &[serde_json::Value]) {
    for hit in hits
        .iter()
        .filter(|hit| hit["ability_id"] == "GA_Cang_Skill")
    {
        assert_eq!(hit["non_crit_damage"], json!("2057/2"));
    }
    assert_eq!(
        hits.iter()
            .filter(|hit| hit["ability_id"] == "GA_Cang_Skill")
            .count(),
        3
    );
}

fn assert_baicang_burst_packet_damage(hits: &[serde_json::Value]) {
    let burst_hits = hits
        .iter()
        .filter(|hit| hit["ability_id"] == "GA_Cang_UltraSkill")
        .collect::<Vec<_>>();
    assert_eq!(burst_hits.len(), 7);
    assert_eq!(burst_hits[0]["non_crit_damage"], json!("6855/2"));
    assert_eq!(
        burst_hits
            .iter()
            .map(|hit| hit["time_us"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        vec![
            41_600_100, 46_454_959, 48_454_959, 50_454_959, 52_454_959, 54_454_959, 56_454_959
        ]
    );
    for tick in &burst_hits[1..] {
        assert_eq!(tick["non_crit_damage"], json!("343/1"));
    }
}

fn assert_baicang_rotation_energy(req: &GameScenarioRequest, report: &serde_json::Value) {
    // Six Normal chains and three Skills generate 144.432 base charge.
    // The source gets 1x and both allies get 3/5 under the owner hypothesis.
    let trace = report["trace"].as_array().unwrap();
    let hit_grants = trace
        .iter()
        .filter(|event| event["energy_fanout"]["source"] == "baicang_hit")
        .collect::<Vec<_>>();
    assert_eq!(hit_grants.len(), 69);
    let expected_charge_milli = baicang_expected_rotation_charge_milli();
    for recipient in 0..req.actors.len() {
        assert_baicang_recipient_energy(
            req,
            report,
            &hit_grants,
            &expected_charge_milli,
            recipient,
        );
    }
    assert_baicang_burst_energy_cost(trace);
}

fn baicang_expected_rotation_charge_milli() -> Vec<u64> {
    let normal_charge_milli = [
        2257_u64, 2009, 5656, 1193, 1193, 1193, 1193, 1193, 1193, 1193, 4099,
    ];
    let mut expected_charge_milli = Vec::new();
    for chain in 0..5 {
        expected_charge_milli.extend(normal_charge_milli);
        if chain < 3 {
            expected_charge_milli.push(3400);
        }
    }
    expected_charge_milli.extend(normal_charge_milli);
    expected_charge_milli
}

fn assert_baicang_recipient_energy(
    req: &GameScenarioRequest,
    report: &serde_json::Value,
    hit_grants: &[&serde_json::Value],
    expected_charge_milli: &[u64],
    recipient: usize,
) {
    let (relation_n, relation_d) = if recipient == 0 {
        (1_u64, 1_u64)
    } else {
        (3, 5)
    };
    let denominator = 1000 * relation_d;
    let mut cumulative_milli = 0_u64;
    let mut generated = 0_u64;
    let mut overflow = 0_u64;
    let mut energy = req.actors[recipient].initial_energy;
    for (packet_index, base_milli) in expected_charge_milli.iter().copied().enumerate() {
        apply_baicang_burst_spend(recipient, packet_index, &mut energy);
        let fanout = &hit_grants[packet_index]["energy_fanout"];
        assert_eq!(fanout["source_actor_index"], 0);
        let recipients = fanout["recipients"].as_array().unwrap();
        assert_eq!(recipients.len(), req.actors.len());
        let entry = &recipients[recipient];
        assert_eq!(entry["recipient_actor_index"], recipient);
        assert_baicang_energy_basis(fanout, entry, base_milli, relation_n, relation_d);
        assert_eq!(entry["energy_before"], energy);
        cumulative_milli += base_milli * relation_n;
        let cumulative_generated = cumulative_milli / denominator;
        let gross = cumulative_generated - generated;
        generated = cumulative_generated;
        let retained = gross.min(req.actors[recipient].energy_capacity - energy);
        let packet_overflow = gross - retained;
        overflow += packet_overflow;
        energy += retained;
        assert_eq!(entry["gross"], gross);
        assert_eq!(entry["retained"], retained);
        assert_eq!(entry["overflow"], packet_overflow);
        assert_eq!(entry["energy_after"], energy);
    }
    assert_baicang_final_recipient_energy(
        report,
        recipient,
        generated,
        overflow,
        energy,
        cumulative_milli,
        denominator,
    );
}

fn apply_baicang_burst_spend(recipient: usize, packet_index: usize, energy: &mut u64) {
    if recipient == 0 && packet_index == 58 {
        assert_eq!(*energy, 120);
        *energy -= 120;
    }
}

fn assert_baicang_energy_basis(
    fanout: &serde_json::Value,
    entry: &serde_json::Value,
    base_milli: u64,
    relation_n: u64,
    relation_d: u64,
) {
    assert_eq!(
        fanout["base"],
        serde_json::to_value(ExactFraction::new(base_milli as u128, 1000).unwrap()).unwrap()
    );
    assert_eq!(
        entry["distribution"],
        serde_json::to_value(ExactFraction::new(relation_n as u128, relation_d as u128).unwrap())
            .unwrap()
    );
}

fn assert_baicang_final_recipient_energy(
    report: &serde_json::Value,
    recipient: usize,
    generated: u64,
    overflow: u64,
    energy: u64,
    cumulative_milli: u64,
    denominator: u64,
) {
    let state = &report["final_actor_states"][recipient];
    assert_eq!(state["granted_energy"], generated);
    assert_eq!(state["overflow_energy"], overflow);
    assert_eq!(state["energy"], energy);
    assert_eq!(state["spent_energy"], if recipient == 0 { 120 } else { 0 });
    if recipient == 0 {
        assert_eq!(generated, 144);
        assert_eq!(overflow, 2);
        assert_eq!(energy, 22);
        assert_eq!(cumulative_milli % denominator, 432);
        assert_eq!(state["charge_remainder"], json!("54/125"));
    } else {
        assert_eq!(generated, 86);
        assert_eq!(overflow, 0);
        assert_eq!(energy, 86);
        assert_eq!(cumulative_milli % denominator, 3296);
        assert_eq!(state["charge_remainder"], json!("412/625"));
    }
}

fn assert_baicang_burst_energy_cost(trace: &[serde_json::Value]) {
    let cost_events = trace
        .iter()
        .filter(|event| event["energy_change"]["source"] == "ability_cost")
        .filter(|event| event["energy_change"]["spent"].as_u64().unwrap_or(0) > 0)
        .collect::<Vec<_>>();
    assert_eq!(cost_events.len(), 1);
    let cost_event = cost_events[0];
    assert_eq!(cost_event["time_us"], 41_600_000);
    let cost = &cost_event["energy_change"];
    assert_eq!(cost["recipient_actor_index"], 0);
    assert_eq!(cost["energy_before"], 120);
    assert_eq!(cost["spent"], 120);
    assert_eq!(cost["energy_after"], 0);
}
