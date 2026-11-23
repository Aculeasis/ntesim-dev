use super::*;
use crate::simulation::{ExactFraction, GameActorConfig};
use serde_json::json;
use std::collections::BTreeMap;

fn t1_proc() -> GearProcConfig {
    GearProcConfig::Nakupeda {
        hp_bonus: Ratio {
            numerator: 6,
            denominator: 25,
        },
        lowest_hp_heal_ratio: Ratio {
            numerator: 1,
            denominator: 5,
        },
        shield_ratio: Ratio {
            numerator: 1,
            denominator: 5,
        },
        shield_duration_us: 15_000_000,
        team_heal_ratio: Ratio {
            numerator: 1,
            denominator: 10,
        },
        cooldown_us: 30_000_000,
    }
}

fn projection_tables() -> BTreeMap<String, serde_json::Value> {
    super::support::weapon_projection_tables()
}

fn projection_actor() -> GameActorConfig {
    serde_json::from_value(json!({
        "slot":0,"character_id":"character:1020","initial_hp":1,"initial_hp_max":1,"initial_energy":0,
        "energy_capacity":100,"weapon_id":"weapon:fork_Nakupeda","weapon_tier":1,
        "static_stats":{"base_atk":1,"atk":"1/1","hp":1,"def":1,
        "crit_rate":"0/1","crit_damage_bonus":"0/1"}
    }))
    .unwrap()
}

fn damage_request() -> crate::simulation::GameScenarioRequest {
    let mut req = research_request();
    req.stochastic_crit = None;
    let actor = &mut req.actors[0];
    actor.character_id = "character:1020".into();
    actor.weapon_id = "weapon:fork_Nakupeda".into();
    actor.weapon_tier = 1;
    actor.gear_procs = vec![t1_proc()];
    actor.static_stats.base_atk = 1000;
    actor.static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    actor.static_stats.crit_rate = Ratio::zero();
    actor.static_stats.damage_bonus = Ratio::zero();
    req.targets[0].defense = 0;
    req.targets[0].resistance = Ratio::zero();
    req.actions.truncate(1);
    req.action_metadata
        .retain(|index, _| *index < req.actions.len());
    req
}

fn ultimate_request(seed: u64) -> crate::simulation::GameScenarioRequest {
    let mut req = research_request();
    req.stochastic_crit = None;
    req.seed = seed;
    req.relative_duration_us = 1;
    req.actors[0].character_id = "character:1020".into();
    req.actors[0].weapon_id = "weapon:fork_Nakupeda".into();
    req.actors[0].gear_procs = vec![t1_proc()];
    req.actors[0].initial_hp = 500;
    req.actors[0].initial_hp_max = 1000;
    req.actors[0].initial_energy = 100;
    req.actors[0].energy_capacity = 100;
    req.actors[0].static_stats.hp = 1000;
    req.actors[0].static_stats.base_hp = Some(1000);
    req.actors.truncate(1);
    for (slot, character, hp) in [(1, "character:1019", 100), (2, "character:1003", 120)] {
        let mut ally = req.actors[0].clone();
        ally.slot = slot;
        ally.character_id = character.into();
        ally.weapon_id.clear();
        ally.gear_procs.clear();
        ally.initial_hp = hp;
        ally.initial_energy = 0;
        ally.energy_capacity = 100;
        req.actors.push(ally);
    }
    req.action_metadata.clear();
    req.actions = vec![crate::simulation::GameActionCommand::InvokeAbility {
        timestamp_us: 0,
        actor_index: 0,
        ability_id: "GA_Haniel_UltraSkill".into(),
        array_index: 0,
        damage_effect_id: "GE_Player_Haniel_Ultraskill1_Damage".into(),
        atk_rate: Ratio::one(),
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost: 100,
        recovery_us: 0,
        cooldown_us: 0,
        hit_delay_us: 0,
    }];
    req.targets[0].defense = 0;
    req.targets[0].resistance = Ratio::zero();
    req
}

#[test]
fn nakupeda_zero_rounded_shield_has_no_live_capacity() {
    let mut req = ultimate_request(0);
    req.actors[0].initial_hp = 1;
    req.actors[0].initial_hp_max = 1;
    req.actors[0].static_stats.hp = 1;
    req.actors[0].static_stats.base_hp = Some(1);
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["proc_selection_receipts"][0]["outcome_index"], 1);
    assert!(
        report["final_actor_states"][0]
            .get("active_shields")
            .is_none()
    );
}

#[test]
fn nakupeda_projection_binds_five_tiers() {
    let tables = projection_tables();
    let mut actor = projection_actor();
    // Frozen buff_Nakipeda_Hp curve: 0.24/0.28/0.32/0.36/0.40.
    let expected_tiers = [(1, 6, 25), (2, 7, 25), (3, 8, 25), (4, 9, 25), (5, 2, 5)];
    for (tier, num, den) in expected_tiers {
        actor.weapon_tier = tier;
        actor.gear_procs.clear();
        let report = crate::weapons::nakupeda::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut actor,
            &[],
        )
        .unwrap();
        let proc = &report["compiled_proc"];
        assert_eq!(proc["kind"], "nakupeda");
        let bound: Ratio = serde_json::from_value(proc["hp_bonus"].clone()).unwrap();
        assert_eq!(
            ExactFraction::from_ratio(bound).unwrap(),
            ExactFraction::new(num, den).unwrap(),
            "tier {tier}"
        );
        for (name, expected) in [
            (
                "lowest_hp_heal_ratio",
                [(1, 5), (1, 4), (3, 10), (7, 20), (2, 5)][(tier - 1) as usize],
            ),
            (
                "shield_ratio",
                [(1, 5), (1, 4), (3, 10), (7, 20), (2, 5)][(tier - 1) as usize],
            ),
            (
                "team_heal_ratio",
                [(1, 10), (1, 8), (3, 20), (7, 40), (1, 5)][(tier - 1) as usize],
            ),
        ] {
            let ratio: Ratio = serde_json::from_value(proc[name].clone()).unwrap();
            assert_eq!(
                ExactFraction::from_ratio(ratio).unwrap(),
                ExactFraction::new(expected.0, expected.1).unwrap()
            );
        }
        assert_eq!(proc["shield_duration_us"], 15_000_000);
        assert_eq!(proc["cooldown_us"], 30_000_000);
        assert_eq!(
            report["native_description"]["localized"],
            json!(
                "Increases HP by <lv>{0}</>.\nGrants one of the following random effects when the wearer casts an Ultimate:\n1. Recovers <lv>{1}</> of Max HP for the team member with the lowest HP percentage.\n2. Grants a Shield equal to <lv>{2}</> of the wearer's Max HP for <lv>{3}</>s.\n3. Recovers <lv>{4}</> HP for all team members.\nTriggers once every <lv>{5}</>s."
            )
        );
        assert_eq!(
            report["scope"],
            crate::weapons::nakupeda::HYPOTHESIS_ID.description()
        );
        assert!(report["runtime_execution_validated"].as_bool() == Some(true));
        assert_eq!(actor.gear_procs.len(), 1);
    }
}

#[test]
fn nakupeda_parameter_binds_by_name_not_datalist_position() {
    // Mutating only the HP curve proves name-based binding; the random-effect
    // curves must not leak into hp_bonus.
    let mut tables = projection_tables();
    tables.get_mut("CT_ForkBuff").unwrap()["rows"]["buff_Nakipeda_Hp"]["Keys"][0]["Value"] =
        json!(0.99);
    let mut actor = projection_actor();
    let report = crate::weapons::nakupeda::projection::project(
        &tables,
        1,
        "ntesim-current",
        &mut actor,
        &[],
    )
    .unwrap();
    let proc = &report["compiled_proc"];
    let hp: Ratio = serde_json::from_value(proc["hp_bonus"].clone()).unwrap();
    assert_eq!(
        ExactFraction::from_ratio(hp).unwrap(),
        ExactFraction::new(99, 100).unwrap()
    );
}

#[test]
fn nakupeda_projection_rejects_group_mismatch_and_changed_units() {
    let tables = projection_tables();
    let mut actor = projection_actor();
    actor.character_id = "character:1004".into();
    assert!(
        crate::weapons::nakupeda::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut actor,
            &[]
        )
        .unwrap_err()
        .contains("incompatible")
    );
    let mut changed = tables.clone();
    changed.get_mut("DT_ForkUpgradeStarDataTable").unwrap()["rows"]["upgradestar_pack_fork_Nakupeda_1"]
        ["DataList"][0]["bIsPercent"] = json!(false);
    let mut unit_actor = projection_actor();
    assert!(
        crate::weapons::nakupeda::projection::project(
            &changed,
            1,
            "ntesim-current",
            &mut unit_actor,
            &[]
        )
        .unwrap_err()
        .contains("unit hint")
    );
}

#[test]
fn nakupeda_projection_rejects_random_effect_unit_drift() {
    // The out-of-scope random-effect curves are pinned: a unit change in any
    // of them must fail loudly rather than silently project.
    let mut tables = projection_tables();
    tables.get_mut("DT_ForkUpgradeStarDataTable").unwrap()["rows"]["upgradestar_pack_fork_Nakupeda_1"]
        ["DataList"][3]["bIsPercent"] = json!(true);
    let mut actor = projection_actor();
    assert!(
        crate::weapons::nakupeda::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut actor,
            &[]
        )
        .unwrap_err()
        .contains("unit hint")
    );
}

#[test]
fn nakupeda_config_rejects_foreign_weapon() {
    let mut req = damage_request();
    req.actors[0].weapon_id = "weapon:fork_Whale".into();
    assert!(
        run_game_scenario(&req, &ExecutionLimits::default())
            .unwrap_err()
            .contains("requires equipped")
    );
    let mut incompatible = damage_request();
    incompatible.actors[0].character_id = "character:1003".into();
    assert!(
        run_game_scenario(&incompatible, &ExecutionLimits::default())
            .unwrap_err()
            .contains("incompatible")
    );
}

#[test]
fn nakupeda_damage_hook_leaves_damage_untouched() {
    let req = damage_request();
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(
        report["hit_receipts"][0]["non_crit_damage"],
        json!("1000/1")
    );
    let mut bare = req.clone();
    bare.actors[0].gear_procs.clear();
    let baseline = run_game_scenario(&bare, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        baseline["hit_receipts"][0]["non_crit_damage"],
        report["hit_receipts"][0]["non_crit_damage"]
    );
}

#[test]
fn nakupeda_ordinary_ultimate_selects_all_three_seeded_outcomes() {
    for (seed, expected_word, outcome) in [
        (4, 2_057_372_262_u32, 0),
        (0, 3_837_872_008_u32, 1),
        (2, 257_813_417_u32, 2),
    ] {
        let mut req = ultimate_request(seed);
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(report["execution_completed"], true, "seed {seed}");
        let selection = &report["proc_selection_receipts"][0];
        assert_eq!(selection["outcome_index"], outcome);
        assert_eq!(selection["sample"]["bound"], 3);
        assert_eq!(selection["sample"]["value"], outcome);
        assert_eq!(selection["sample"]["draws"][0]["value"], expected_word);
        assert_eq!(
            selection["sample"]["draws"][0]["stream"],
            "weapon:nakupeda:0"
        );
        assert_eq!(selection["cooldown_until_us"], 30_000_000);
        match outcome {
            0 => {
                assert_eq!(report["final_actor_states"][1]["hp"], 300);
                assert_eq!(report["final_actor_states"][2]["hp"], 120);
            }
            1 => {
                assert_eq!(
                    report["final_actor_states"][0]["active_shields"][0]["max_hp"],
                    200
                );
                assert_eq!(
                    report["final_actor_states"][0]["active_shields"][0]["expires_at_us"],
                    19_237_614
                );
            }
            2 => {
                assert_eq!(report["final_actor_states"][0]["hp"], 600);
                assert_eq!(report["final_actor_states"][1]["hp"], 200);
                assert_eq!(report["final_actor_states"][2]["hp"], 220);
            }
            _ => unreachable!(),
        }
        req.stochastic_crit = Some(crate::simulation::ResearchCritPolicy {
            roll_policy: crate::simulation::ResearchCritRollPolicy::IndependentPerTargetHitV1,
            hypothesis: "Test independent per-target-hit crit rolls; game policy unknown".into(),
            eligible_effects: BTreeMap::from([(
                "character:1020".into(),
                std::collections::BTreeSet::from(["GE_Player_Haniel_Ultraskill1_Damage".into()]),
            )]),
            actor_sequences: BTreeMap::from([("character:1020".into(), 54)]),
        });
        let audited = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        let mut crit_only = req.clone();
        crit_only.actors[0].gear_procs.clear();
        let crit_only_report = run_game_scenario(&crit_only, &ExecutionLimits::default()).unwrap();
        assert_eq!(audited["rng"], crit_only_report["rng"]);
        let mut colliding = req.clone();
        colliding
            .stochastic_crit
            .as_mut()
            .unwrap()
            .actor_sequences
            .insert("character:1020".into(), 1_u64 << 62);
        assert!(
            run_game_scenario(&colliding, &ExecutionLimits::default())
                .unwrap_err()
                .contains("collides with stochastic_crit")
        );
        assert_eq!(
            audited["proc_selection_receipts"],
            report["proc_selection_receipts"]
        );
        assert_eq!(audited["proc_rng"], report["proc_rng"]);
        assert_eq!(
            run_game_scenario(&req, &ExecutionLimits::default()).unwrap(),
            audited
        );
    }
}

#[test]
fn nakupeda_owner_defeat_cancels_settlement_after_audited_selection() {
    for seed in [0, 2] {
        let mut req = ultimate_request(seed);
        req.incoming_damage_events.push(GameActorDamageEvent {
            timestamp_us: 0,
            actor_index: 0,
            delivered_damage: 500,
        });
        req.model_assumptions.insert(
            crate::simulation::events::INCOMING_DAMAGE_HYPOTHESIS_ID.into(),
            crate::simulation::events::INCOMING_DAMAGE_HYPOTHESIS_ID
                .description()
                .into(),
        );
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(
            report["proc_selection_receipts"].as_array().unwrap().len(),
            1
        );
        assert_eq!(report["final_actor_states"][1]["hp"], 100);
        assert_eq!(report["final_actor_states"][2]["hp"], 120);
        assert!(
            report["trace"]
                .as_array()
                .unwrap()
                .iter()
                .any(|event| event["event_kind"] == "nakupeda_cancelled")
        );
    }
}

#[test]
fn nakupeda_lowest_hp_uses_exact_fractions_and_keeps_first_tied_slot() {
    assert_eq!(
        crate::weapons::nakupeda::lowest_hp_recipient(&[
            (200, 1000),
            (400, 2000),
            (0, 1),
            (800, 1000),
        ]),
        Some(0)
    );
    assert_eq!(
        crate::weapons::nakupeda::lowest_hp_recipient(&[(200, 1000), (99, 1000)]),
        Some(1)
    );
    assert_eq!(
        crate::weapons::nakupeda::lowest_hp_recipient(&[(0, 1000), (1, 1000), (2, 2000)]),
        Some(1)
    );
    assert_eq!(
        crate::weapons::nakupeda::lowest_hp_recipient(&[(0, 1000), (0, 1000)]),
        None
    );
}

#[test]
fn nakupeda_icd_starts_on_successful_admission_and_is_ready_at_exact_deadline() {
    let mut req = ultimate_request(4);
    for actor in &mut req.actors {
        actor.initial_hp = actor.initial_hp_max;
    }
    req.relative_duration_us = 30_000_001;
    req.actors[0].initial_energy = 300;
    req.actors[0].energy_capacity = 300;
    let first = req.actions[0].clone();
    let crate::simulation::GameActionCommand::InvokeAbility {
        actor_index,
        ability_id,
        array_index,
        damage_effect_id,
        atk_rate,
        scaling_stat,
        energy_cost,
        recovery_us,
        cooldown_us,
        hit_delay_us,
        ..
    } = first
    else {
        unreachable!();
    };
    req.actions = [0, 10_000_000, 38_475_228]
        .into_iter()
        .map(
            |timestamp_us| crate::simulation::GameActionCommand::InvokeAbility {
                timestamp_us,
                actor_index,
                ability_id: ability_id.clone(),
                array_index,
                damage_effect_id: damage_effect_id.clone(),
                atk_rate,
                scaling_stat,
                energy_cost,
                recovery_us,
                cooldown_us,
                hit_delay_us,
            },
        )
        .collect();
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    let selections = report["proc_selection_receipts"].as_array().unwrap();
    assert_eq!(selections.len(), 2);
    assert_eq!(selections[0]["time_us"], 0);
    assert_eq!(selections[1]["time_us"], 38_475_228);
    assert_eq!(selections[1]["sample"]["draws"][0]["index"], 1);
}

#[test]
fn nakupeda_wearers_keep_independent_streams_and_cooldown_deadlines() {
    let mut req = ultimate_request(0);
    req.relative_duration_us = 600_001;
    req.actors[2].weapon_id = "weapon:fork_Nakupeda".into();
    req.actors[2].character_id = "character:1046".into();
    req.actors[2].gear_procs = vec![t1_proc()];
    req.actors[2].initial_energy = 100;
    let mut other_cast = req.actions[0].clone();
    if let GameActionCommand::InvokeAbility {
        timestamp_us,
        actor_index,
        ability_id,
        damage_effect_id,
        ..
    } = &mut other_cast
    {
        *timestamp_us = 600_001;
        *actor_index = 2;
        *ability_id = "GA_Female046_UltraSkill".into();
        *damage_effect_id = crate::characters::zero::primary_packet(ability_id)
            .unwrap()
            .into();
    }
    req.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 1,
        target_actor_index: 2,
        cooldown_us: 0,
    });
    req.actions.push(other_cast);
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let selections = report["proc_selection_receipts"].as_array().unwrap();
    assert_eq!(selections.len(), 2);
    assert_eq!(selections[0]["actor_index"], 0);
    assert_eq!(selections[1]["actor_index"], 2);
    assert_eq!(selections[0]["cooldown_until_us"], 30_000_000);
    assert_eq!(selections[1]["cooldown_until_us"], 30_600_001);
    assert_ne!(selections[0]["stream"], selections[1]["stream"]);
    assert_eq!(
        report,
        run_game_scenario(&req, &ExecutionLimits::default()).unwrap()
    );
}

#[test]
fn nakupeda_failed_admission_draws_nothing_and_shields_absorb_expire_and_clear_on_death() {
    let mut rejected = ultimate_request(0);
    if let crate::simulation::GameActionCommand::InvokeAbility { energy_cost, .. } =
        &mut rejected.actions[0]
    {
        *energy_cost = 101;
    }
    let failed = run_game_scenario(&rejected, &ExecutionLimits::default()).unwrap();
    assert_eq!(failed["execution_completed"], false);
    assert!(
        failed
            .get("proc_selection_receipts")
            .and_then(serde_json::Value::as_array)
            .is_none_or(Vec::is_empty)
    );

    let mut shielded = ultimate_request(0);
    shielded.relative_duration_us = 15_000_000;
    shielded.incoming_damage_events = vec![
        crate::simulation::GameActorDamageEvent {
            timestamp_us: 14_999_999,
            actor_index: 0,
            delivered_damage: 100,
        },
        crate::simulation::GameActorDamageEvent {
            timestamp_us: 15_000_000,
            actor_index: 0,
            delivered_damage: 100,
        },
    ];
    shielded.model_assumptions.insert(
        crate::simulation::events::INCOMING_DAMAGE_HYPOTHESIS_ID.into(),
        crate::simulation::events::INCOMING_DAMAGE_HYPOTHESIS_ID
            .description()
            .into(),
    );
    let report = run_game_scenario(&shielded, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["incoming_damage_receipts"][0]["actual_hp_loss"], 0);
    assert_eq!(
        report["incoming_damage_receipts"][0]["absorbed_by_shields"][0]["amount"],
        100
    );
    assert_eq!(report["incoming_damage_receipts"][1]["actual_hp_loss"], 100);
    assert!(
        report["final_actor_states"][0]
            .get("active_shields")
            .and_then(serde_json::Value::as_array)
            .is_none_or(Vec::is_empty)
    );

    let mut fatal = ultimate_request(0);
    fatal.relative_duration_us = 1;
    fatal.incoming_damage_events = vec![crate::simulation::GameActorDamageEvent {
        timestamp_us: 1,
        actor_index: 0,
        delivered_damage: 1000,
    }];
    fatal.model_assumptions.insert(
        crate::simulation::events::INCOMING_DAMAGE_HYPOTHESIS_ID.into(),
        crate::simulation::events::INCOMING_DAMAGE_HYPOTHESIS_ID
            .description()
            .into(),
    );
    let report = run_game_scenario(&fatal, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["final_actor_states"][0]["hp"], 0);
    assert!(
        report["final_actor_states"][0]
            .get("active_shields")
            .and_then(serde_json::Value::as_array)
            .is_none_or(Vec::is_empty)
    );
}

#[test]
fn nakupeda_uses_live_max_hp_for_shields_healing_and_later_damage() {
    for (seed, delivered, expected_hp, absorbed, horizon, final_max) in [
        (0, 600, 508, 208, 2, 1400),
        (2, 1, 1003, 0, 2, 1400),
        (2, 1, 1000, 0, 20_000_000, 1000),
    ] {
        let mut req = ultimate_request(seed);
        req.relative_duration_us = horizon;
        req.actors[0].initial_hp = 1000;
        req.actors[0].set_id = Some("set:Suit10".into());
        req.actors[0].gear_procs.push(GearProcConfig::HpSuitMaxHp {
            max_hp_bonus_per_stack: Ratio {
                numerator: 1,
                denominator: 25,
            },
            duration_us: 10_000_000,
            max_stacks: 10,
        });
        if let GameActionCommand::InvokeAbility { timestamp_us, .. } = &mut req.actions[0] {
            *timestamp_us = 1;
        }
        req.incoming_damage_events = vec![
            GameActorDamageEvent {
                timestamp_us: 0,
                actor_index: 0,
                delivered_damage: 100,
            },
            GameActorDamageEvent {
                timestamp_us: 2,
                actor_index: 0,
                delivered_damage: delivered,
            },
        ];
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
        assert_eq!(report["final_actor_states"][0]["hp"], expected_hp);
        assert_eq!(report["final_actor_states"][0]["hp_max"], final_max);
        assert_eq!(report["invariants_passed"], true);
        let total_absorbed: u64 = report["incoming_damage_receipts"][1]["absorbed_by_shields"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|entry| entry["amount"].as_u64().unwrap())
            .sum();
        assert_eq!(total_absorbed, absorbed);
    }
}

#[test]
fn nakupeda_healing_separates_zones_caps_overheal_and_never_revives() {
    let mut req = ultimate_request(4);
    req.actors[0].static_stats.healing_bonus = Ratio {
        numerator: 1,
        denominator: 5,
    };
    let mut state = crate::simulation::state::GameSimulatorState::new(&req);
    state
        .apply_buff(
            0,
            crate::simulation::state::TimedBuffDeclaration::reset(
                "recipient.healing",
                "test",
                ntesim_core::buffs::dispatch::BuffRecipient::SpecificActor(1),
                100,
                ntesim_core::buffs::dispatch::BuffStackingScope::PerSource,
                BTreeMap::from([(
                    "incoming_healing_bonus".into(),
                    ExactFraction::new(1, 10).unwrap(),
                )]),
            ),
        )
        .unwrap();
    for (hp, expected, excess) in [(100, 364, 0), (990, 1000, 254), (1000, 1000, 264)] {
        state.actors[1].hp = hp;
        let mut trace = Vec::new();
        crate::weapons::nakupeda::apply(
            &mut crate::runtime_api::CallbackContext {
                state: &mut state,
                request: &req,
                queue: &mut crate::simulation::timer_queue::EventQueue::new(),
                sequence: &mut 0,
                trace: &mut trace,
            },
            0,
            0,
            &[1],
            &[ExactFraction::new(200, 1).unwrap()],
            15_000_000,
        )
        .unwrap();
        assert_eq!(state.actors[1].hp, expected);
        assert_eq!(
            trace[0].data.as_ref().and_then(|d| d.overflow),
            Some(excess)
        );
    }
    state.actors[1].hp = 0;
    crate::weapons::nakupeda::apply(
        &mut crate::runtime_api::CallbackContext {
            state: &mut state,
            request: &req,
            queue: &mut crate::simulation::timer_queue::EventQueue::new(),
            sequence: &mut 0,
            trace: &mut Vec::new(),
        },
        0,
        0,
        &[1],
        &[ExactFraction::new(200, 1).unwrap()],
        15_000_000,
    )
    .unwrap();
    assert_eq!(state.actors[1].hp, 0);
}

#[test]
fn nakupeda_shield_survives_switching_and_conserves_overkill() {
    let mut req = ultimate_request(0);
    req.relative_duration_us = 2;
    req.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 1,
        target_actor_index: 1,
        cooldown_us: 0,
    });
    req.incoming_damage_events.push(GameActorDamageEvent {
        timestamp_us: 2,
        actor_index: 0,
        delivered_damage: 800,
    });
    req.model_assumptions.insert(
        crate::simulation::events::INCOMING_DAMAGE_HYPOTHESIS_ID.into(),
        crate::simulation::events::INCOMING_DAMAGE_HYPOTHESIS_ID
            .description()
            .into(),
    );
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let incoming = &report["incoming_damage_receipts"][0];
    assert_eq!(incoming["absorbed_by_shields"][0]["amount"], 200);
    assert_eq!(incoming["actual_hp_loss"], 500);
    assert_eq!(incoming["overflow"], 100);
}

#[test]
fn nakupeda_shield_sums_static_and_live_granter_bonus_before_final_rounding() {
    let mut req = ultimate_request(0);
    req.actors[0].static_stats.shield_bonus = Ratio {
        numerator: 1,
        denominator: 5,
    };
    req.actors[1].static_stats.shield_bonus = Ratio {
        numerator: 9,
        denominator: 10,
    };
    req.relative_duration_us = 2;
    req.incoming_damage_events.push(GameActorDamageEvent {
        timestamp_us: 1,
        actor_index: 0,
        delivered_damage: 100,
    });
    req.model_assumptions.insert(
        crate::simulation::events::INCOMING_DAMAGE_HYPOTHESIS_ID.into(),
        crate::simulation::events::INCOMING_DAMAGE_HYPOTHESIS_ID
            .description()
            .into(),
    );
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["final_actor_states"][0]["active_shields"][0]["max_hp"],
        240
    );
    assert_eq!(
        report["final_actor_states"][0]["active_shields"][0]["remaining_hp"],
        140
    );
    assert_eq!(
        report["incoming_damage_receipts"][0]["absorbed_by_shields"][0]["amount"],
        100
    );

    let mut state = crate::simulation::state::GameSimulatorState::new(&req);
    state
        .apply_buff(
            0,
            crate::simulation::state::TimedBuffDeclaration::reset(
                "live.shield",
                "test",
                ntesim_core::buffs::dispatch::BuffRecipient::SpecificActor(0),
                1,
                ntesim_core::buffs::dispatch::BuffStackingScope::PerSource,
                BTreeMap::from([("shield_bonus".into(), ExactFraction::new(3, 10).unwrap())]),
            ),
        )
        .unwrap();
    let mut trace = Vec::new();
    crate::weapons::nakupeda::apply(
        &mut crate::runtime_api::CallbackContext {
            state: &mut state,
            request: &req,
            queue: &mut crate::simulation::timer_queue::EventQueue::new(),
            sequence: &mut 0,
            trace: &mut trace,
        },
        0,
        1,
        &[0],
        &[ExactFraction::new(200, 1).unwrap()],
        15_000_000,
    )
    .unwrap();
    assert_eq!(state.actors[0].shields[0].max_hp, 300);
    crate::weapons::nakupeda::apply(
        &mut crate::runtime_api::CallbackContext {
            state: &mut state,
            request: &req,
            queue: &mut crate::simulation::timer_queue::EventQueue::new(),
            sequence: &mut 0,
            trace: &mut trace,
        },
        0,
        1,
        &[0],
        &[ExactFraction::new(1003, 4).unwrap()],
        15_000_000,
    )
    .unwrap();
    assert_eq!(state.actors[0].shields[0].max_hp, 376);
    state.advance_unpaused_test_clock(1);
    let expired_bonus = crate::runtime_api::CallbackContext {
        state: &mut state,
        request: &req,
        queue: &mut crate::simulation::timer_queue::EventQueue::new(),
        sequence: &mut 0,
        trace: &mut trace,
    }
    .live_shield_bonus(0)
    .unwrap();
    assert!(expired_bonus.is_zero());
    assert_eq!(state.actors[0].shields[0].max_hp, 376);
    assert_eq!(state.actors[0].shields[0].remaining_hp, 376);
}

#[test]
fn nakupeda_idle_expiry_keeps_the_actual_deadline_in_trace() {
    let mut req = ultimate_request(0);
    req.relative_duration_us = 20_000_000;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["max_proc_depth_reached"], 1);
    let expiry = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .find(|event| event["event_kind"] == "actor_shield_expired")
        .unwrap();
    assert_eq!(expiry["time_us"], 19_237_614);
}
