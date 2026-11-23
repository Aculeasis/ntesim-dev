use super::*;
use crate::simulation::metadata::InputActionRole;

fn fixture() -> GameScenarioRequest {
    let mut req = research_request();
    req.relative_duration_us = 35_000_000;
    req.actors[0].weapon_id = "weapon:fork_twinbirds".into();
    req.actors[0].gear_procs = vec![GearProcConfig::Twinbirds {
        cycle_intensity_bonus: Ratio {
            numerator: 72,
            denominator: 1,
        },
        psyche_bonus: Ratio {
            numerator: 1,
            denominator: 5,
        },
        team_crit_damage_bonus: Ratio {
            numerator: 4,
            denominator: 25,
        },
        self_crit_damage_bonus: Ratio {
            numerator: 8,
            denominator: 25,
        },
        duration_us: 15_000_000,
    }];
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.actors[0].static_stats.elemental_bonus = Ratio::zero();
    req.actors[0].static_stats.crit_rate = Ratio::one();
    req.actors[0].static_stats.crit_damage_bonus = Ratio::zero();
    req.targets[0].resistance = Ratio::zero();
    req.targets[0].defense = 1080;
    let metadata = req.action_metadata[&0].clone();
    req.actions.clear();
    req.action_metadata.clear();
    req.action_metadata.insert(0, metadata);
    req
}

fn nova_request(tier: u32) -> GameScenarioRequest {
    let source = format!(
        "options iteration=1 duration=15 crit=fixed;
        lacrimosa char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
        blackbird char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        blackbird add weapon=fork_twinbirds refine={tier};
        target hp=100000000 def=1080 resist=0;
        active lacrimosa; blackbird swap; wait(3); lacrimosa swap;"
    );
    crate::dsl::compiler::compile_scenario(&source, Default::default())
        .unwrap()
        .request
}

fn nova_hit(report: &serde_json::Value) -> &serde_json::Value {
    report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|hit| hit["damage_effect_id"] == "ntesim.reaction.nova")
        .unwrap()
}

#[test]
fn twinbirds_all_refinements_supply_ci_before_any_cast_and_snapshot_off_field() {
    // Nova at level 80: 45000*(1+CI/600), ignoring DEF. Frozen CI is points,
    // not percentages: T1..T5 = 72/90/108/126/144.
    for (tier, ci, damage) in [
        (1, 72, 50400),
        (2, 90, 51750),
        (3, 108, 53100),
        (4, 126, 54450),
        (5, 144, 55800),
    ] {
        let req = nova_request(tier);
        assert_eq!(req.actors[1].static_stats.cycle_intensity, Ratio::zero());
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        let hit = nova_hit(&report);
        assert_eq!(
            hit["actor_index"], 1,
            "the equipped incoming participant wins CI ownership"
        );
        assert_eq!(
            hit["non_crit_damage"],
            serde_json::json!(format!("{}/{}", damage, 1))
        );
        assert_eq!(report["final_buff_state"]["active_actor_index"], 0);
        let all_buffs = crate::simulation::tests::support::final_buffs(&report);
        let buff = all_buffs
            .iter()
            .find(|buff| buff["id"] == "twinbirds.cycle_intensity")
            .unwrap();
        assert_eq!(buff["owner_actor"], 1);
        assert_eq!(buff["stacks"], 1);
        assert_eq!(buff["recipient"]["type"], "self_actor");
        assert_eq!(
            buff["stat_modifiers"]["cycle_intensity"],
            serde_json::json!(format!("{}/{}", ci, 1))
        );
        assert_eq!(
            report["resolved_model_assumptions"][crate::weapons::twinbirds::HYPOTHESIS_ID.as_str()],
            crate::weapons::twinbirds::HYPOTHESIS_ID.description()
        );
    }
}

#[test]
fn twinbirds_ci_adds_with_authored_points_and_never_leaks_to_the_other_participant() {
    for (other_ci, wearer_ci, owner, damage) in [
        (73, 0, 0, 50475),
        (72, 0, 0, 50400),
        (73, 2, 1, 50550),
        (0, 528, 1, 90000),
    ] {
        let mut req = nova_request(1);
        req.actors[0].static_stats.cycle_intensity = Ratio {
            numerator: other_ci,
            denominator: 1,
        };
        req.actors[1].static_stats.cycle_intensity = Ratio {
            numerator: wearer_ci,
            denominator: 1,
        };
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        let hit = nova_hit(&report);
        assert_eq!(hit["actor_index"], owner);
        assert_eq!(
            hit["non_crit_damage"],
            serde_json::json!(format!("{}/{}", damage, 1))
        );
    }
}

#[test]
fn twinbirds_ci_does_not_fill_cycle_energy_or_accept_an_invalid_ratio() {
    let mut req = nova_request(1);
    req.actors[0].initial_cycle = 0;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert!(report["hit_receipts"].as_array().unwrap().is_empty());
    if let GearProcConfig::Twinbirds {
        cycle_intensity_bonus,
        ..
    } = &mut req.actors[1].gear_procs[0]
    {
        cycle_intensity_bonus.denominator = 0;
    }
    assert!(run_game_scenario(&req, &ExecutionLimits::default()).is_err());
}

#[test]
fn twinbirds_ci_persists_through_two_continuing_cycles_without_duplicate_grants() {
    let mut req = nova_request(1);
    req.actors[1].initial_cycle = 100;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let hits: Vec<_> = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["damage_effect_id"] == "ntesim.reaction.nova")
        .collect();
    // Two authored generations and Blackbird's one child of the latest one;
    // the earlier refreshed generation cannot create a competing child.
    assert_eq!(hits.len(), 3);
    for hit in hits {
        assert_eq!(hit["actor_index"], 1);
        assert_eq!(hit["non_crit_damage"], serde_json::json!("50400/1"));
    }
    let all_buffs = crate::simulation::tests::support::final_buffs(&report);
    let buffs: Vec<_> = all_buffs
        .iter()
        .filter(|buff| buff["id"] == "twinbirds.cycle_intensity")
        .collect();
    assert_eq!(buffs.len(), 1);
    assert_eq!(buffs[0]["stacks"], 1);
}

fn append(
    req: &mut GameScenarioRequest,
    actor: usize,
    time: u64,
    role: InputActionRole,
    element: &str,
    delay: u64,
) {
    let index = req.actions.len();
    req.actions.push(GameActionCommand::InvokeAbility {
        timestamp_us: time,
        actor_index: actor,
        ability_id: "caller-test".into(),
        damage_effect_id: "caller-effect".into(),
        array_index: 0,
        atk_rate: Ratio::one(),
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost: 0,
        recovery_us: 0,
        cooldown_us: 0,
        hit_delay_us: delay,
    });
    let mut metadata = req.action_metadata[&0].clone();
    metadata.input_action_role = Some(role);
    metadata.action_gameplay_tags = Some(vec!["Ability.Melee".into()]);
    metadata.damage_source_category =
        Some("EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_A".into());
    metadata.damage_type_ex = Some(format!("EExecutionDamageTypeEX::DAMAGE_TYPE_{element}"));
    req.action_metadata.insert(index, metadata);
}

#[test]
fn twinbirds_applies_psyche_bonus_and_team_and_self_crit_damage_on_ultimate() {
    let mut req = fixture();
    append(&mut req, 0, 0, InputActionRole::Normal, "PSYCHE", 0);
    append(
        &mut req,
        0,
        1_000_000,
        InputActionRole::Ultimate,
        "PSYCHE",
        0,
    );
    append(&mut req, 0, 5_000_000, InputActionRole::Normal, "PSYCHE", 0);
    append(
        &mut req,
        0,
        10_000_000,
        InputActionRole::Ultimate,
        "PSYCHE",
        0,
    );
    append(
        &mut req,
        0,
        20_000_000,
        InputActionRole::Normal,
        "PSYCHE",
        0,
    );
    append(
        &mut req,
        0,
        28_903_489,
        InputActionRole::Normal,
        "PSYCHE",
        0,
    );

    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    // Base ATK 1000, DEF mitigation 0.5. Psyche bonus 1.2 -> base damage 600.
    // Active Ultimate grants 16% team + 32% self = 48% CRIT DMG -> 600 * 1.48 = 888.
    // Ultimate recast at 10s refreshes15 World seconds, expiring at absolute28.903489s after the cinematic; that exact-time hit has expired (precedes equal-time hit).
    let hits = report["hit_receipts"].as_array().unwrap();
    let hit_values = hits
        .iter()
        .map(|hit| hit["rounded_damage"].as_u64().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(hit_values, vec![600, 888, 888, 888, 888, 600]);
    assert_eq!(
        report["resolved_model_assumptions"][crate::weapons::twinbirds::HYPOTHESIS_ID.as_str()],
        crate::weapons::twinbirds::HYPOTHESIS_ID.description()
    );
    assert_eq!(
        report["resolved_model_assumptions"][crate::gear::CAST_BUFF_HYPOTHESIS_ID.as_str()],
        crate::gear::CAST_BUFF_HYPOTHESIS_ID.description()
    );
}

#[test]
fn twinbirds_buffs_active_ally_crit_damage_and_retains_self_bonus_for_wearer() {
    let mut req = fixture();
    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1003".into();
    ally.weapon_id = String::new();
    ally.gear_procs.clear();
    ally.static_stats.crit_rate = Ratio::one();
    ally.static_stats.crit_damage_bonus = Ratio::zero();
    req.actors.push(ally);

    req.stochastic_crit
        .as_mut()
        .unwrap()
        .eligible_effects
        .insert(
            "character:1003".into(),
            BTreeSet::from(["caller-effect".into()]),
        );
    req.stochastic_crit
        .as_mut()
        .unwrap()
        .actor_sequences
        .insert("character:1003".into(), 55);

    append(
        &mut req,
        0,
        1_000_000,
        InputActionRole::Ultimate,
        "PSYCHE",
        0,
    );
    req.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 2_000_000,
        target_actor_index: 1,
        cooldown_us: 0,
    });
    append(&mut req, 1, 3_000_000, InputActionRole::Normal, "NATURE", 0);
    req.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 4_000_000,
        target_actor_index: 0,
        cooldown_us: 0,
    });
    append(&mut req, 0, 5_000_000, InputActionRole::Normal, "PSYCHE", 0);

    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let hits = report["hit_receipts"].as_array().unwrap();
    // Wearer hit under ultimate: 888.
    assert_eq!(hits[0]["rounded_damage"], 888);
    // Ally hit under team 16% CRIT DMG without Psyche bonus: 1000 * 0.5 * 1.16 = 580.
    let ally_receipt = hits
        .iter()
        .find(|h| h["actor_index"] == 1 && h["ability_id"] != "GA_Sagiri_QTE")
        .unwrap();
    assert_eq!(ally_receipt["rounded_damage"], 580);
    // Wearer hit after switch-back: 888.
    let wearer_after = hits
        .iter()
        .find(|h| h["actor_index"] == 0 && h["time_us"] == 5_000_000)
        .unwrap();
    assert_eq!(wearer_after["rounded_damage"], 888);
}

#[test]
fn twinbirds_rejects_bad_equipment_and_does_not_trigger_on_failed_admission() {
    let mut req = fixture();
    append(&mut req, 0, 0, InputActionRole::Ultimate, "PSYCHE", 0);
    let limits = ExecutionLimits::default();
    let mut bad = req.clone();
    bad.actors[0].weapon_id = "weapon:fork_nonos".into();
    assert!(
        run_game_scenario(&bad, &limits)
            .unwrap_err()
            .contains("requires equipped weapon:fork_twinbirds")
    );

    let duplicate = req.actors[0].gear_procs[0].clone();
    bad = req.clone();
    bad.actors[0].gear_procs.push(duplicate);
    assert!(
        run_game_scenario(&bad, &limits)
            .unwrap_err()
            .contains("duplicates")
    );

    req.actors[0].initial_energy = 0;
    if let GameActionCommand::InvokeAbility { energy_cost, .. } = &mut req.actions[0] {
        *energy_cost = 100;
    }
    let report = run_game_scenario(&req, &limits).unwrap();
    assert_eq!(report["execution_completed"], false);
    assert!(
        !report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["event_kind"] == "twinbirds_crit_damage")
    );
}
