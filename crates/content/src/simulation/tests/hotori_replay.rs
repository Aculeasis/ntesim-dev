use super::*;
use crate::simulation::metadata::InputActionRole;

const SYNTHETIC_ACTION_HYPOTHESIS: &str = "test.synthetic_action";
const SYNTHETIC_ACTION_DESCRIPTION: &str = "Test-only explicit metadata for synthetic actions.";

const SKILL_TIME_US: u64 = 11_000_000;
const SWITCH_TO_ALLY_US: u64 = 12_500_000;
const SWITCH_TO_HOTORI_US: u64 = 14_400_000;
const RECORD_CLOSE_US: u64 = 16_167_475;
const BURST_TIME_US: u64 = 36_000_000;
const PLAYBACK_TIME_US: u64 = 40_216_431;

fn request() -> GameScenarioRequest {
    let mut req = sample_s1_request();
    req.actors[0].character_id = "character:1052".into();
    req.actors[0].weapon_id.clear();
    req.actors[0].gear_procs.clear();
    req.actors[0].set_id = None;
    req.actors[0].talent_levels = Some([1; 3]);
    req.actors[0].character_max_level = Some(80);
    req.actors[0].character_level = 80;
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.actors[0].static_stats.elemental_bonus = Ratio::zero();
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    req.actors[0].initial_energy = 120;
    req.actors[0].energy_capacity = 120;
    req.targets[0].resistance = Ratio::zero();
    req.targets[0].level = 80;
    req.targets[0].hp_current = 10_000_000;
    // World's Tide keeps its resource deadline pending until Burst + 14.183s.
    req.relative_duration_us = 52_000_000;
    req.actions.clear();
    req
}

fn ally(slot: usize) -> GameActorConfig {
    let mut ally = sample_s1_request().actors[0].clone();
    ally.slot = slot;
    ally.initial_energy = 0;
    ally.weapon_id.clear();
    ally.gear_procs.clear();
    ally.set_id = None;
    ally.static_stats.crit_rate = Ratio::zero();
    ally
}

fn mint_ally(slot: usize) -> GameActorConfig {
    let mut mint = ally(slot);
    // The Mind Royale proc is source-bound to its weapon.
    mint.weapon_id = "weapon:fork_lingganzhongjiezhe".into();
    mint.gear_procs = vec![GearProcConfig::MindRoyaleEnergy {
        energy_grant: 18,
        cooldown_us: 20_000_000,
    }];
    mint
}

fn hotori_ability(
    timestamp_us: u64,
    ability_id: &str,
    damage_effect_id: &str,
) -> GameActionCommand {
    GameActionCommand::InvokeAbility {
        timestamp_us,
        actor_index: 0,
        ability_id: ability_id.into(),
        damage_effect_id: damage_effect_id.into(),
        array_index: 0,
        atk_rate: Ratio::one(),
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost: 0,
        recovery_us: 0,
        cooldown_us: 0,
        hit_delay_us: 0,
    }
}

fn hotori_skill(timestamp_us: u64) -> GameActionCommand {
    hotori_ability(timestamp_us, "GA_Jin_Skill", "GE_Player_Jin_Skill1_Damage")
}

fn hotori_burst(timestamp_us: u64) -> GameActionCommand {
    hotori_ability(
        timestamp_us,
        "GA_Jin_UltraSkill",
        "GE_Player_Jin_UltraSkill1_Damage",
    )
}

fn mint_skill(timestamp_us: u64, actor_index: usize) -> GameActionCommand {
    GameActionCommand::InvokeAbility {
        timestamp_us,
        actor_index,
        ability_id: "GA_Mint019_Skill".into(),
        damage_effect_id: "GE_Player_Mint_Skill1_Damage_New".into(),
        array_index: 7,
        atk_rate: Ratio {
            numerator: 771,
            denominator: 500,
        },
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost: 0,
        recovery_us: 1_200_000,
        cooldown_us: 0,
        hit_delay_us: 0,
    }
}

fn switch(timestamp_us: u64, target_actor_index: usize) -> GameActionCommand {
    GameActionCommand::SwitchActor {
        timestamp_us,
        target_actor_index,
        cooldown_us: 0,
    }
}

/// Research-declared single-packet Redirect Skill for an arbitrary ally slot.
fn synthetic_redirect(req: &mut GameScenarioRequest, actor_index: usize, timestamp_us: u64) {
    let index = req.actions.len();
    let ability_id = format!("caller-ability-{index}");
    let damage_effect_id = format!("caller-packet-{index}");
    req.actions.push(GameActionCommand::InvokeAbility {
        timestamp_us,
        actor_index,
        ability_id: ability_id.clone(),
        damage_effect_id: damage_effect_id.clone(),
        array_index: 0,
        atk_rate: Ratio::one(),
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost: 0,
        recovery_us: 0,
        cooldown_us: 0,
        hit_delay_us: 0,
    });
    let mut metadata = research_request().action_metadata[&0].clone();
    metadata.ability_id = ability_id;
    metadata.damage_effect_id = damage_effect_id;
    metadata.input_action_role = Some(InputActionRole::RedirectSkill);
    metadata.action_gameplay_tags = Some(vec!["Ability.Skill".into()]);
    req.action_metadata.insert(index, metadata);
}

fn accept_synthetic_action(req: &mut GameScenarioRequest) {
    req.model_assumptions.insert(
        SYNTHETIC_ACTION_HYPOTHESIS.into(),
        SYNTHETIC_ACTION_DESCRIPTION.into(),
    );
}

fn trace_events<'a>(report: &'a serde_json::Value, kind: &str) -> Vec<&'a serde_json::Value> {
    report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["event_kind"] == kind)
        .collect()
}

fn accept(req: &mut GameScenarioRequest, id: crate::HypothesisId) {
    req.model_assumptions
        .insert(id.as_str().into(), id.description().into());
}

fn assert_hotori_playback_grants_and_hits(report: &serde_json::Value) {
    // The recorded cast granted 18 energy at 13s and set a 20s ICD. Playback
    // is past the 33s ready deadline, so the live cast grants again.
    let grants = trace_events(report, "mind_royale_energy");
    assert_eq!(grants.len(), 2);
    for (grant, time) in grants.iter().zip([13_000_000, PLAYBACK_TIME_US]) {
        assert_eq!(grant["time_us"], time);
        assert_eq!(grant["energy_change"]["source"], "mind_royale");
        assert_eq!(grant["energy_change"]["source_actor_index"], 1);
        assert_eq!(grant["energy_change"]["recipient_actor_index"], 1);
        assert_eq!(grant["energy_change"]["gross"], 18);
    }

    let hits = report["hit_receipts"].as_array().unwrap();
    // Mint's two authored packets then their two replayed copies; the replay
    // keeps the .365246s second-hit stagger.
    let ally_hits: Vec<_> = hits.iter().filter(|hit| hit["actor_index"] == 1).collect();
    assert_eq!(ally_hits.len(), 4);
    let (authored, replayed) = ally_hits.split_at(2);
    assert_eq!(authored[0]["time_us"], 13_000_000);
    assert_eq!(authored[1]["time_us"], 13_365_246);
    assert_eq!(replayed[0]["time_us"], PLAYBACK_TIME_US);
    assert_eq!(replayed[1]["time_us"], PLAYBACK_TIME_US + 365_246);
    for (recorded_hit, replay_hit) in authored.iter().zip(replayed) {
        assert_eq!(
            replay_hit["damage_effect_id"],
            "GE_Player_Mint_Skill1_Damage_New"
        );
        assert_eq!(
            replay_hit["damage_effect_id"],
            recorded_hit["damage_effect_id"]
        );
        assert_eq!(
            replay_hit["non_crit_damage"],
            recorded_hit["non_crit_damage"]
        );
        assert_eq!(replay_hit["command_index"], recorded_hit["command_index"]);
    }
    // Playback is not an admission, so it must not produce another record.
    assert_eq!(trace_events(report, "hotori_record_slot").len(), 1);
    assert!(
        report["resolved_model_assumptions"]["hotori.recorded_replay_v1"]
            .as_str()
            .is_some()
    );
}

#[test]
fn hotori_records_redirect_skill_and_replays_it_against_live_ally_state() {
    let mut req = request();
    req.actors.push(mint_ally(1));
    req.actions = vec![
        hotori_skill(SKILL_TIME_US),
        switch(12_500_000, 1),
        mint_skill(13_000_000, 1),
        switch(14_400_000, 0),
        hotori_burst(BURST_TIME_US),
    ];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{}",
        report["halt_diagnostic"]
    );
    // 7 Present Replay hits, 2 authored Mint Skill packets, 10 World's Tide
    // packets and the 2 replayed Mint Skill packets.
    assert_eq!(report["hits_settled"], 21);

    let records = trace_events(&report, "hotori_record_slot");
    assert_eq!(records.len(), 1);
    assert_eq!(records[0]["time_us"], 13_000_000);
    assert_eq!(records[0]["from_actor"], 1);
    assert_eq!(records[0]["data"]["ability_id"], "GA_Mint019_Skill");
    assert_eq!(
        records[0]["data"]["damage_effect_id"],
        "GE_Player_Mint_Skill1_Damage_New"
    );

    let clears = trace_events(&report, "hotori_replay_clear");
    assert_eq!(clears.len(), 1);
    assert_eq!(clears[0]["time_us"], PLAYBACK_TIME_US);
    assert_eq!(clears[0]["data"]["count"], 1);

    let dispatches = trace_events(&report, "hotori_replay_dispatch");
    assert_eq!(dispatches.len(), 1);
    assert_eq!(dispatches[0]["time_us"], PLAYBACK_TIME_US);
    assert_eq!(dispatches[0]["from_actor"], 1);
    assert_eq!(dispatches[0]["data"]["ability_id"], "GA_Mint019_Skill");
    assert_eq!(
        dispatches[0]["data"]["damage_effect_id"],
        "GE_Player_Mint_Skill1_Damage_New"
    );
    assert_eq!(dispatches[0]["data"]["completes_at_us"], 40_216_431);

    assert_hotori_playback_grants_and_hits(&report);
    assert_eq!(
        report,
        run_game_scenario(&req, &ExecutionLimits::default()).unwrap()
    );

    // A longer Mind Royale cooldown keeps the playback grant inside the ICD.
    let mut icd = request();
    let mut mint = mint_ally(1);
    mint.gear_procs = vec![GearProcConfig::MindRoyaleEnergy {
        energy_grant: 18,
        cooldown_us: 30_000_000,
    }];
    icd.actors.push(mint);
    icd.actions = req.actions.clone();
    let icd_report = run_game_scenario(&icd, &ExecutionLimits::default()).unwrap();
    let icd_grants = trace_events(&icd_report, "mind_royale_energy");
    assert_eq!(icd_grants.len(), 1);
    assert_eq!(icd_grants[0]["time_us"], 13_000_000);
}

#[test]
fn hotori_recording_window_is_half_open_and_needs_redirect_inputs() {
    // Hotori's own Skill recovery lasts until 12.4086605, so the earliest
    // reachable allied admission is just inside the window; the close bound is
    // half-open. The open bound is covered by the admission interior checks.
    for (timestamp, expected) in [
        (12_500_000, 1),
        (RECORD_CLOSE_US - 1, 1),
        (RECORD_CLOSE_US, 0),
        (17_000_000, 0),
    ] {
        let mut req = request();
        req.relative_duration_us = 20_000_000;
        req.actors.push(mint_ally(1));
        req.actions = vec![
            hotori_skill(SKILL_TIME_US),
            switch(12_500_000, 1),
            mint_skill(timestamp, 1),
        ];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{}",
            report["halt_diagnostic"]
        );
        let records = trace_events(&report, "hotori_record_slot");
        assert_eq!(records.len(), expected, "timestamp {timestamp}");
        if expected == 1 {
            assert_eq!(records[0]["time_us"], timestamp);
        }
    }

    let mut melee = request();
    melee.relative_duration_us = 20_000_000;
    melee.actors.push(mint_ally(1));
    melee.actions = vec![
        hotori_skill(SKILL_TIME_US),
        switch(12_500_000, 1),
        GameActionCommand::InvokeAbility {
            timestamp_us: 13_000_000,
            actor_index: 1,
            ability_id: "GA_Mint019_Melee".into(),
            damage_effect_id: "GE_Player_Mint_Melee1_Damage_New".into(),
            array_index: 7,
            atk_rate: Ratio {
                numerator: 89,
                denominator: 500,
            },
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 0,
            cooldown_us: 0,
            hit_delay_us: 0,
        },
    ];
    let report = run_game_scenario(&melee, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{}",
        report["halt_diagnostic"]
    );
    assert_eq!(trace_events(&report, "hotori_record_slot").len(), 0);
}

#[test]
fn hotori_recording_keeps_one_slot_per_owner_and_at_most_three_owners() {
    // Per-owner dedupe: the first admitted Redirect fills the owner's slot.
    let mut dedupe = request();
    dedupe.actors.push(mint_ally(1));
    dedupe.actions = vec![hotori_skill(SKILL_TIME_US), switch(SWITCH_TO_ALLY_US, 1)];
    synthetic_redirect(&mut dedupe, 1, 13_000_000);
    synthetic_redirect(&mut dedupe, 1, 13_500_000);
    dedupe.actions.push(switch(SWITCH_TO_HOTORI_US, 0));
    dedupe.actions.push(hotori_burst(BURST_TIME_US));
    accept_synthetic_action(&mut dedupe);
    let report = run_game_scenario(&dedupe, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{}",
        report["halt_diagnostic"]
    );
    let records = trace_events(&report, "hotori_record_slot");
    assert_eq!(records.len(), 1);
    assert_eq!(records[0]["time_us"], 13_000_000);
    assert_eq!(records[0]["from_actor"], 1);
    assert_eq!(records[0]["data"]["ability_id"], "caller-ability-2");
    assert_eq!(records[0]["data"]["damage_effect_id"], "caller-packet-2");
    let dispatches = trace_events(&report, "hotori_replay_dispatch");
    assert_eq!(dispatches.len(), 1);
    assert_eq!(
        trace_events(&report, "hotori_replay_clear")[0]["data"]["count"],
        1
    );
    let replay_hits: Vec<_> = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["time_us"] == PLAYBACK_TIME_US)
        .collect();
    assert_eq!(replay_hits.len(), 1);
    assert_eq!(replay_hits[0]["actor_index"], 1);

    // Three distinct allied owners fill all three slots; admission order is
    // preserved and the record order drives playback order.
    let mut capped = request();
    for slot in 1..=3 {
        capped.actors.push(ally(slot));
    }
    capped.actions = vec![hotori_skill(SKILL_TIME_US), switch(SWITCH_TO_ALLY_US, 1)];
    for owner in 1..=3u64 {
        let redirect_time = 13_000_000 + 100_000 * (owner - 1);
        if owner > 1 {
            capped
                .actions
                .push(switch(redirect_time - 50_000, owner as usize));
        }
        synthetic_redirect(&mut capped, owner as usize, redirect_time);
    }
    capped.actions.push(switch(SWITCH_TO_HOTORI_US, 0));
    capped.actions.push(hotori_burst(BURST_TIME_US));
    accept_synthetic_action(&mut capped);
    let report = run_game_scenario(&capped, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{}",
        report["halt_diagnostic"]
    );
    let records = trace_events(&report, "hotori_record_slot");
    assert_eq!(records.len(), 3);
    let owners: Vec<u64> = records
        .iter()
        .map(|record| record["from_actor"].as_u64().unwrap())
        .collect();
    assert_eq!(owners, vec![1, 2, 3]);
    assert_eq!(
        trace_events(&report, "hotori_replay_dispatch").len(),
        3,
        "one dispatch per stored owner"
    );
    let replay_hits: Vec<_> = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["time_us"] == PLAYBACK_TIME_US)
        .collect();
    assert_eq!(replay_hits.len(), 3);
    assert_eq!(
        replay_hits
            .iter()
            .map(|hit| hit["actor_index"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        vec![1, 2, 3]
    );
}

#[test]
fn hotori_playback_skips_defeated_recorded_owners() {
    let mut req = request();
    req.actors.push(mint_ally(1));
    req.actions = vec![
        hotori_skill(SKILL_TIME_US),
        switch(12_500_000, 1),
        mint_skill(13_000_000, 1),
        switch(14_400_000, 0),
        hotori_burst(BURST_TIME_US),
    ];
    req.incoming_damage_events = vec![crate::simulation::GameActorDamageEvent {
        timestamp_us: 20_000_000,
        actor_index: 1,
        delivered_damage: 20_000,
    }];
    req.model_assumptions.insert(
        crate::simulation::events::INCOMING_DAMAGE_HYPOTHESIS_ID.into(),
        crate::simulation::events::INCOMING_DAMAGE_HYPOTHESIS_ID
            .description()
            .into(),
    );
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{}",
        report["halt_diagnostic"]
    );
    assert_eq!(report["final_actor_states"][1]["hp"], 0);
    assert_eq!(trace_events(&report, "hotori_record_slot").len(), 1);
    let skipped = trace_events(&report, "hotori_replay_skipped");
    assert_eq!(skipped.len(), 1);
    assert_eq!(skipped[0]["data"]["reason"], "owner_defeated");
    assert_eq!(trace_events(&report, "mind_royale_energy").len(), 1);
    assert!(
        report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .all(|hit| !(hit["actor_index"] == 1 && hit["time_us"] == PLAYBACK_TIME_US))
    );
}

#[test]
fn hotori_new_present_replay_clears_the_previous_recording() {
    let mut req = request();
    req.relative_duration_us = 42_000_000;
    req.actors.push(mint_ally(1));
    req.actions = vec![
        hotori_skill(SKILL_TIME_US),
        switch(SWITCH_TO_ALLY_US, 1),
        mint_skill(13_000_000, 1),
        switch(SWITCH_TO_HOTORI_US, 0),
        // A second Present Replay clears the first store and opens a new
        // five-second window.
        hotori_skill(36_000_000),
        switch(37_500_000, 1),
        mint_skill(38_000_000, 1),
    ];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{}",
        report["halt_diagnostic"]
    );
    let records = trace_events(&report, "hotori_record_slot");
    assert_eq!(records.len(), 2);
    for (record, time) in records.iter().zip([13_000_000, 38_000_000]) {
        assert_eq!(record["time_us"], time);
        assert_eq!(record["data"]["slot"], 0);
    }
    assert_eq!(trace_events(&report, "hotori_replay_dispatch").len(), 0);
}

#[test]
fn hotori_records_and_replays_engine_dispatched_support_entries() {
    let mut req = request();
    req.actors[0].initial_cycle = 100;
    let mut nanally = ally(1);
    nanally.character_id = "character:1010".into();
    req.actors.push(nanally);
    req.actions = vec![
        hotori_skill(SKILL_TIME_US),
        switch(13_000_000, 1),
        switch(14_400_000, 0),
        hotori_burst(BURST_TIME_US),
    ];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{}",
        report["halt_diagnostic"]
    );
    assert_eq!(trace_events(&report, "support_dispatch").len(), 1);
    let records = trace_events(&report, "hotori_record_slot");
    assert_eq!(records.len(), 1);
    assert_eq!(records[0]["time_us"], 13_000_000);
    assert_eq!(records[0]["from_actor"], 1);
    assert_eq!(records[0]["data"]["ability_id"], "GA_Nanally_QTE");
    assert_eq!(
        records[0]["data"]["damage_effect_id"],
        "GE_Player_Nanally_QTE1_Damage"
    );
    let dispatches = trace_events(&report, "hotori_replay_dispatch");
    assert_eq!(dispatches.len(), 1);
    assert_eq!(dispatches[0]["data"]["completes_at_us"], 40_882_841);
    let replay_hits: Vec<_> = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["actor_index"] == 1 && hit["time_us"] == 40_882_841)
        .collect();
    assert_eq!(replay_hits.len(), 1);
    assert_eq!(
        replay_hits[0]["damage_effect_id"],
        "GE_Player_Nanally_QTE1_Damage"
    );
}

#[test]
fn hotori_replay_model_is_required_in_strict_mode() {
    let mut req = request();
    req.actors[0].talent_levels = None;
    req.actors.push(ally(1));
    req.actions = vec![
        hotori_skill(SKILL_TIME_US),
        switch(SWITCH_TO_ALLY_US, 1),
        mint_skill(13_000_000, 1),
        switch(SWITCH_TO_HOTORI_US, 0),
        hotori_burst(BURST_TIME_US),
    ];
    for id in [
        ntesim_data::native_action_metadata::PRIMARY_ROLE_MAPPING_HYPOTHESIS_ID,
        crate::characters::hotori::timepiece::MODEL,
        crate::characters::hotori::timepiece::CLOCK_MODEL,
        crate::characters::hotori::skill::PACKETS_HYPOTHESIS_ID,
        crate::characters::mint::skill::PACKETS_HYPOTHESIS_ID,
        crate::simulation::energy::HOTORI_HIT_ENERGY_HYPOTHESIS_ID,
        crate::simulation::energy::MINT_HIT_ENERGY_HYPOTHESIS_ID,
        crate::HypothesisId::CombatPacketCycleAccrualV1,
    ] {
        accept(&mut req, id);
    }
    accept(&mut req, crate::characters::hotori::replay::MODEL);
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{}",
        report["halt_diagnostic"]
    );
    assert_eq!(report["hits_settled"], 21);
}

#[test]
fn hotori_playback_without_records_does_not_touch_the_replay_model() {
    let mut req = request();
    req.relative_duration_us = 36_000_000;
    req.actions = vec![hotori_burst(20_000_000)];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{}",
        report["halt_diagnostic"]
    );
    assert_eq!(report["hits_settled"], 10);
    assert_eq!(trace_events(&report, "hotori_replay_clear").len(), 0);
    assert_eq!(trace_events(&report, "hotori_replay_dispatch").len(), 0);
    assert!(
        report["resolved_model_assumptions"]
            .get("hotori.recorded_replay_v1")
            .is_none()
    );
    assert_eq!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "hotori_time_stop_enter")
            .count(),
        1
    );
}
