use super::*;
use serde_json::json;

fn make_action(
    timestamp_us: u64,
    actor_index: usize,
    ability_id: &str,
    damage_effect_id: &str,
) -> GameActionCommand {
    GameActionCommand::InvokeAbility {
        timestamp_us,
        actor_index,
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

fn request() -> GameScenarioRequest {
    let mut req = sample_s1_request();
    req.actors[0].character_id = "character:1004".into();
    req.actors[0].weapon_id.clear();
    req.actors[0].gear_procs.clear();
    req.actors[0].set_id = None;
    req.actors[0].talent_levels = Some([1, 8, 1]);
    req.actors[0].character_max_level = Some(80);
    req.actors[0].character_level = 80;
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.actors[0].static_stats.elemental_bonus = Ratio::zero();
    req.actors[0].initial_energy = 120;
    req.actors[0].energy_capacity = 120;
    req.targets[0].resistance = Ratio::zero();
    req.targets[0].level = 80;
    req.model_assumptions.insert(
        crate::characters::lacrimosa::dot::DOT_SCHEDULING_HYPOTHESIS_ID.into(),
        crate::characters::lacrimosa::dot::DOT_SCHEDULING_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::simulation::metadata::FIXED_CRIT_PROBABILITY_HYPOTHESIS_ID
            .as_str()
            .into(),
        crate::simulation::metadata::FIXED_CRIT_PROBABILITY_ID
            .description()
            .into(),
    );
    req.actions = vec![make_action(
        0,
        0,
        "GA_Lacrimosa_Skill",
        "GE_Player_Lacrimosa_Skill1_Damage",
    )];
    req
}

fn blood_ticks(report: &serde_json::Value) -> Vec<&serde_json::Value> {
    let mut ticks: Vec<&serde_json::Value> = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["damage_effect_id"] == "GE_Player_Lacrimosa_Blood_Damage")
        .collect();
    ticks.sort_by_key(|hit| hit["time_us"].as_u64().unwrap());
    ticks
}

fn assert_tick(tick: &serde_json::Value, time_us: u64, numerator: u64, denominator: u64) {
    assert_eq!(tick["time_us"], time_us, "tick time");
    assert_eq!(
        tick["non_crit_damage"],
        json!(format!("{}/{}", numerator, denominator)),
        "tick at t={}s",
        time_us / 1_000_000,
    );
}

fn all_target_request() -> GameScenarioRequest {
    let mut req = request();
    req.relative_duration_us = 40_000_000;
    req.actors[0].talent_levels = Some([8, 1, 1]);
    req.targets[0].defense = 0;
    req.targets[0].hp_max = 10_000_000;
    req.targets[0].hp_current = 10_000_000;
    req.targets.push(req.targets[0].clone());
    req.targets[1].target_index = 1;
    let id = crate::characters::lacrimosa::dot::ALL_TARGETS_HYPOTHESIS_ID;
    req.model_assumptions
        .insert(id.into(), id.description().into());
    req
}

#[test]
fn all_target_blood_uses_independent_normal_talent_and_resonance() {
    for (levels, awakening, expected) in [
        (
            [1, 8, 1],
            0,
            Ratio {
                numerator: 3,
                denominator: 50,
            },
        ),
        (
            [8, 1, 9],
            0,
            Ratio {
                numerator: 103,
                denominator: 1000,
            },
        ),
        (
            [7, 1, 1],
            3,
            Ratio {
                numerator: 103,
                denominator: 1000,
            },
        ),
    ] {
        let mut req = all_target_request();
        req.actors[0].talent_levels = Some(levels);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        let ticks = blood_ticks(&report);
        assert_eq!(ticks.len(), 28);
        assert!(
            ticks
                .iter()
                .all(|hit| hit["scaling_rate"] == serde_json::to_value(expected).unwrap())
        );
    }
}

#[test]
fn all_target_delayed_refresh_retains_both_grids_and_old_ticks_until_impact() {
    let mut req = all_target_request();
    let mut delayed = make_action(
        500_000,
        0,
        "GA_Lacrimosa_Skill",
        "GE_Player_Lacrimosa_Skill1_Damage",
    );
    if let GameActionCommand::InvokeAbility { hit_delay_us, .. } = &mut delayed {
        *hit_delay_us = 2_000_000;
    }
    req.actions.push(delayed);
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let ticks = blood_ticks(&report);
    assert_eq!(ticks.len(), 64);
    let mut total = ExactFraction::new(0, 1).unwrap();
    for target in 0..2 {
        let owned: Vec<_> = ticks
            .iter()
            .filter(|hit| hit["target_index"] == target)
            .collect();
        assert_eq!(owned.len(), 32);
        assert_tick(owned[0], 1_000_000, 515, 1);
        assert_tick(owned[1], 2_000_000, 515, 1);
        assert_tick(owned[2], 3_000_000, 1030, 1);
        assert_tick(owned[5], 6_000_000, 927, 1);
        assert_tick(owned[31], 32_000_000, 103, 1);
    }
    for tick in ticks {
        total = total
            .add(serde_json::from_value(tick["non_crit_damage"].clone()).unwrap())
            .unwrap();
    }
    // Per target: two five-stack ticks plus 3*(10+...+1) = 175 stack-ticks.
    assert_eq!(total, ExactFraction::new(36050, 1).unwrap());
}

#[test]
fn all_target_skill_damage_precedes_living_recipient_selection() {
    let mut req = all_target_request();
    req.targets[0].hp_current = 1;
    req.targets[0].hp_max = 1;
    if let GameActionCommand::InvokeAbility { hit_delay_us, .. } = &mut req.actions[0] {
        *hit_delay_us = 1_000_000;
    }
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let skill: Vec<_> = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["damage_effect_id"] == "GE_Player_Lacrimosa_Skill1_Damage")
        .collect();
    assert_eq!(skill.len(), 2);
    let ticks = blood_ticks(&report);
    assert_eq!(ticks.len(), 14);
    assert!(ticks.iter().all(|hit| hit["target_index"] == 1));
    assert_eq!(ticks[0]["time_us"], 2_000_000);
}

#[test]
fn all_target_extension_diagnoses_a_missing_scheduling_dependency() {
    let mut req = all_target_request();
    req.model_assumptions
        .remove(crate::characters::lacrimosa::dot::DOT_SCHEDULING_HYPOTHESIS_ID.as_str());
    let error = run_game_scenario(&req, &ExecutionLimits::default()).unwrap_err();
    assert!(error.contains("requires its accepted DoT scheduling model"));
}

// Oracles use ATK=1000, DEF 1080 -> F=1/2, RES 0, skill 8 -> per-stack ratio
// 0.103. Tick damage = 1000 * 0.103 * (1 + clamp(count-1,0,20)) * 1/2
// = 51.5 * (1 + additional).

#[test]
fn lacrimosa_dot_schedules_no_ticks_without_accepted_assumption() {
    let mut req = request();
    req.model_assumptions
        .remove(crate::characters::lacrimosa::dot::DOT_SCHEDULING_HYPOTHESIS_ID.as_str());
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    // Only the Skill hit settles; the Nightmare model stays fully inert so
    // existing hit-count assertions are unaffected.
    assert_eq!(report["hits_settled"], 1);
    assert!(blood_ticks(&report).is_empty());
}

#[test]
fn lacrimosa_dot_single_cast_decay_schedule() {
    // Oracle (a)+(b): 1 cast at t=0.
    // t=1,2: 5 stacks -> 51.5*5 = 515/2.
    // t=3,4,5: expiry at t=3 removes one -> 51.5*4 = 206/1.
    // t=6: expiry at t=6 removes one -> 51.5*3 = 309/2.
    // t=13: expiries at 3,6,9,12 leave 1 stack -> 51.5 = 103/2.
    let req = request();
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let ticks = blood_ticks(&report);
    assert_eq!(ticks.len(), 14, "ticks at t=1..14, none after final expiry");
    assert_eq!(report["hits_settled"], 15);
    for (index, tick) in ticks.iter().enumerate() {
        assert_eq!(tick["time_us"], ((index as u64) + 1) * 1_000_000);
    }
    assert_tick(ticks[0], 1_000_000, 515, 2);
    assert_tick(ticks[1], 2_000_000, 515, 2);
    assert_tick(ticks[2], 3_000_000, 206, 1);
    assert_tick(ticks[3], 4_000_000, 206, 1);
    assert_tick(ticks[4], 5_000_000, 206, 1);
    assert_tick(ticks[5], 6_000_000, 309, 2);
    assert_tick(ticks[12], 13_000_000, 103, 2);
}

#[test]
fn lacrimosa_dot_four_casts_cap_stacks_at_twenty() {
    // Oracle (a): 4 casts (t=0, 0.5s, 1.5s, 2.5s) reach the 20-stack cap.
    // Tick t=4 sees 20 live stacks: 51.5*20 = 1030/1. Cast times avoid the
    // integer tick grid so no application coincides with a tick.
    let mut req = request();
    // 20 stacks with a 5.5s aggregate deadline decay one per 3s, so the last
    // tick lands at t=62s; the default 30s horizon would cut the train.
    req.relative_duration_us = 90_000_000;
    req.actions = [0, 500_000, 1_500_000, 2_500_000]
        .map(|timestamp_us| {
            make_action(
                timestamp_us,
                0,
                "GA_Lacrimosa_Skill",
                "GE_Player_Lacrimosa_Skill1_Damage",
            )
        })
        .to_vec();
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let ticks = blood_ticks(&report);
    // t=1: 10 stacks (casts at 0 and 0.5s) -> 51.5*10 = 515/1.
    assert_tick(ticks[0], 1_000_000, 515, 1);
    // t=4: 20 stacks, no expiry yet (deadline refreshed to t=5.5) -> 1030/1.
    let tick4 = ticks
        .iter()
        .find(|tick| tick["time_us"] == 4_000_000)
        .unwrap();
    assert_tick(tick4, 4_000_000, 1030, 1);
}

#[test]
fn lacrimosa_dot_reapply_refreshes_duration_and_removes_single_stack() {
    // Oracle (b): casts at t=0 and t=2.
    // t=1: 5 stacks -> 51.5*5 = 515/2. The t=2 tick never delivers: the t=2s
    // admission (priority ActionAdmission) rebuilds the train before the
    // pending tick (priority HitDelivery) would fire.
    // t=3,4: reapply refreshed the aggregate deadline, no expiry -> 10
    //   stacks, 51.5*10 = 515/1.
    // t=5,6: single-stack removal at the refreshed deadline -> 9 stacks,
    //   51.5*9 = 927/2.
    let mut req = request();
    // 10 stacks with a 5s aggregate deadline decay one per 3s; the last tick
    // lands at t=31s, past the default 30s horizon.
    req.relative_duration_us = 60_000_000;
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Lacrimosa_Skill",
            "GE_Player_Lacrimosa_Skill1_Damage",
        ),
        make_action(
            2_000_000,
            0,
            "GA_Lacrimosa_Skill",
            "GE_Player_Lacrimosa_Skill1_Damage",
        ),
    ];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let ticks = blood_ticks(&report);
    assert_tick(ticks[0], 1_000_000, 515, 2);
    assert_tick(ticks[1], 3_000_000, 515, 1);
    assert_tick(ticks[2], 4_000_000, 515, 1);
    assert_tick(ticks[3], 5_000_000, 927, 2);
    assert_tick(ticks[4], 6_000_000, 927, 2);
}

#[test]
fn lacrimosa_dot_ten_second_window_sums_to_oracle() {
    // Oracle (c): 1 cast; ticks t=1..10 sum to
    // 2*257.5 + 3*206 + 3*154.5 + 2*103 = 1802.5 = 3605/2.
    let req = request();
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let mut sum = ExactFraction::new(0, 1).unwrap();
    for tick in blood_ticks(&report) {
        if tick["time_us"].as_u64().unwrap() <= 10_000_000 {
            let dmg: ExactFraction =
                serde_json::from_value(tick["non_crit_damage"].clone()).unwrap();
            sum = sum.add(dmg).unwrap();
        }
    }
    assert_eq!(sum, ExactFraction::new(3605, 2).unwrap());
}

#[test]
fn lacrimosa_dot_ticks_persist_across_field_switch() {
    // Oracle (d): cast at t=0, switch off-field at t=2; all 14 ticks still
    // settle with Lacrimosa attribution.
    let mut req = request();
    let mut ally = req.actors[0].clone();
    ally.character_id = "character:1019".into();
    ally.slot = 1;
    req.actors.push(ally);
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Lacrimosa_Skill",
            "GE_Player_Lacrimosa_Skill1_Damage",
        ),
        GameActionCommand::SwitchActor {
            timestamp_us: 2_000_000,
            target_actor_index: 1,
            cooldown_us: 0,
        },
    ];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let ticks = blood_ticks(&report);
    assert_eq!(ticks.len(), 14);
    for tick in &ticks {
        assert_eq!(tick["actor_index"], 0);
    }
    assert_tick(ticks[4], 5_000_000, 206, 1);
    assert_tick(ticks[12], 13_000_000, 103, 2);
}

#[test]
fn lacrimosa_dot_reapply_falsifies_own_deadline_expiry() {
    // Oracle (e), corrected: casts at t=0 and t=2 under aggregate
    // RemoveSingleStackAndRefreshDuration give 9 live stacks at t=7
    // (single removal at t=5, refresh to t=8): 51.5*9 = 927/2.
    // The conflicting per-application own-deadline model would give 7 stacks
    // (app1: 5->4 at t=3, 4->3 at t=6; app2: 5->4 at t=5): 51.5*7 = 721/2.
    // The approved plan's 515/1 was misderived (it assumed no expiry at t=5,
    // contradicting oracle (b)); re-derived from the model here.
    let mut req = request();
    // Same 10-stack / 5s-deadline train as the refresh test: last tick t=31s.
    req.relative_duration_us = 60_000_000;
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Lacrimosa_Skill",
            "GE_Player_Lacrimosa_Skill1_Damage",
        ),
        make_action(
            2_000_000,
            0,
            "GA_Lacrimosa_Skill",
            "GE_Player_Lacrimosa_Skill1_Damage",
        ),
    ];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let ticks = blood_ticks(&report);
    let tick7 = ticks
        .iter()
        .find(|tick| tick["time_us"] == 7_000_000)
        .unwrap();
    assert_tick(tick7, 7_000_000, 927, 2);
}

#[test]
fn lacrimosa_dot_tick_carries_dot_tag_and_native_coefficient() {
    let req = request();
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let ticks = blood_ticks(&report);
    let tick = ticks[0];
    assert_eq!(tick["actor_index"], 0);
    assert_eq!(tick["ability_id"], "GA_Lacrimosa_Skill");
    assert_eq!(tick["time_us"], 1_000_000);
    // Native per-stack ratio at skill 8 rides on the tick; the stack
    // multiplier is added in the coefficient-addition zone at settle time.
    assert_eq!(tick["scaling_rate"], json!("103/1000"));
    // Serialized fixed crit 0.5 from the Blood row.
    assert_eq!(tick["crit_rate"], json!("1/2"));
    let tags = super::support::hit_metadata(&report, tick)["packet"]["semantic_tags"]
        .as_array()
        .unwrap();
    assert!(
        tags.iter().any(|tag| tag == "State.Damage.Dot"),
        "tick missing State.Damage.Dot: {tags:?}"
    );
}
