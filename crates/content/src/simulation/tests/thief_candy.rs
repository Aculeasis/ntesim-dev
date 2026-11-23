use super::*;
use crate::simulation::metadata::{ActionMetadata, InputActionRole, MetadataProvenance};
use crate::simulation::state::GameSimulatorState;
use crate::simulation::tests::support::formula_damage_total;
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};

fn t1_proc() -> GearProcConfig {
    GearProcConfig::ThiefCandyDodge {
        damage_bonus: Ratio {
            numerator: 2,
            denominator: 25,
        },
        duration_us: 10_000_000,
    }
}

fn cast_request() -> GameScenarioRequest {
    let mut req = research_request();
    req.stochastic_crit = None;
    req.actors[0].weapon_id = "weapon:fork_ThiefCandy".into();
    req.actors[0].gear_procs = vec![t1_proc()];
    req
}

fn critical_dash(time_us: u64) -> GameActionCommand {
    GameActionCommand::PlayerInput {
        timestamp_us: time_us,
        actor_index: 0,
        input: PlayerInput::Dash,
        critical_success: true,
    }
}

fn action_metadata(tagged: bool) -> ActionMetadata {
    let mut tags = vec!["Ability.Melee".to_owned()];
    if tagged {
        tags.push(crate::weapons::thief_candy::CRITICAL_DODGE_TAG.into());
    }
    ActionMetadata {
        categories: BTreeSet::new(),
        gameplay_tags: tags,
        ability_id: "caller-test".into(),
        native_ability_index: None,
        input_role: Some(InputActionRole::Normal),
        input_role_provenance: None,
        primary_slot: None,
        source_evidence: None,
        identity_evidence: None,
        provenance: MetadataProvenance::AcceptedModelHypothesis {
            id: "test.thief_candy_cast".into(),
            description: "Test-owned cast metadata.".into(),
        },
    }
}

fn grant_on_cast(
    tagged: bool,
    req: &GameScenarioRequest,
    state: &mut GameSimulatorState,
    time_us: u64,
) {
    let metadata = action_metadata(tagged);
    super::support::invoke_gear_cast(req, state, time_us, &metadata);
}

fn buff_expiries(state: &GameSimulatorState) -> Vec<u64> {
    state
        .buffs
        .instances()
        .iter()
        .filter(|buff| buff.id == crate::weapons::thief_candy::BUFF_ID)
        .map(|buff| match buff.duration {
            ntesim_core::buffs::dispatch::BuffDuration::Timed { expires_at_us, .. } => {
                expires_at_us
            }
            ntesim_core::buffs::dispatch::BuffDuration::Permanent => u64::MAX,
        })
        .collect()
}

fn damage_bonus_at(state: &GameSimulatorState, time_us: u64) -> Option<ExactFraction> {
    state
        .buffs
        .get_stat_modifiers_for_actor(0, 0, time_us)
        .unwrap()
        .get("damage_bonus")
        .copied()
        .map(|ratio| ExactFraction::new(ratio.numerator.into(), ratio.denominator.into()).unwrap())
}

fn projection_tables() -> BTreeMap<String, serde_json::Value> {
    super::support::weapon_projection_tables()
}

fn projection_actor() -> GameActorConfig {
    serde_json::from_value(json!({
        "slot":0,"character_id":"character:1003","initial_hp":1,"initial_hp_max":1,"initial_energy":0,
        "energy_capacity":100,"weapon_id":"weapon:fork_ThiefCandy","weapon_tier":1,
        "static_stats":{"base_atk":1,"atk":"1/1","hp":1,"def":1,
        "crit_rate":"0/1","crit_damage_bonus":"0/1"}
    }))
    .unwrap()
}

#[test]
fn thief_candy_projection_binds_five_refinement_tiers() {
    let tables = projection_tables();
    let expected = [(2u64, 25u64), (12, 125), (14, 125), (16, 125), (18, 125)];
    let mut actor = projection_actor();
    for (tier, (numerator, denominator)) in (1..=5u32).zip(expected) {
        actor.weapon_tier = tier;
        actor.gear_procs.clear();
        let report = crate::weapons::thief_candy::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut actor,
            &[],
        )
        .unwrap();
        let proc = &report["compiled_proc"];
        assert_eq!(proc["kind"], "thief_candy_dodge");
        assert_eq!(
            proc["damage_bonus"],
            json!(format!("{}/{}", numerator, denominator)),
            "tier {tier} damage bonus"
        );
        assert_eq!(proc["duration_us"], 10_000_000);
        assert_eq!(report["native_parameter_report"]["parameter_count"], 2);
        assert_eq!(actor.gear_procs.len(), 1);
    }
}

#[test]
fn thief_candy_projection_rejects_group_mismatch_and_changed_units() {
    let tables = projection_tables();
    let mut actor = projection_actor();
    actor.character_id = "character:1052".into();
    assert!(
        crate::weapons::thief_candy::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut actor,
            &[],
        )
        .unwrap_err()
        .contains("group is incompatible")
    );
    actor.character_id = "character:1003".into();
    let mut changed = tables.clone();
    changed.get_mut("DT_ForkUpgradeStarDataTable").unwrap()["rows"]
        ["upgradestar_pack_fork_ThiefCandy_1"]["DataList"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|entry| entry["NameID"] == "buff_ThiefCandy_Up")
        .unwrap()["bIsPercent"] = json!(false);
    assert!(
        crate::weapons::thief_candy::projection::project(
            &changed,
            1,
            "ntesim-current",
            &mut actor,
            &[],
        )
        .unwrap_err()
        .contains("unit hint")
    );
}

#[test]
fn thief_candy_validate_config_rejects_mismatches() {
    let limits = ExecutionLimits::default();
    let mut actor = projection_actor();
    let valid = t1_proc();
    let ctx = crate::gear::validation::ConfigContext {
        actor: &actor,
        actor_index: 0,
        limits: &limits,
    };
    let validation = crate::weapons::thief_candy::validate_config(&valid, &ctx).unwrap();
    assert_eq!(validation.kind, "thief_candy");
    assert_eq!(validation.duration_us, 10_000_000);

    actor.weapon_id = "weapon:fork_Kite".into();
    let ctx = crate::gear::validation::ConfigContext {
        actor: &actor,
        actor_index: 0,
        limits: &limits,
    };
    let err = crate::weapons::thief_candy::validate_config(&valid, &ctx)
        .err()
        .expect("expected weapon mismatch rejection");
    assert!(err.contains("weapon:fork_ThiefCandy"));

    actor.weapon_id = "weapon:fork_ThiefCandy".into();
    let ctx = crate::gear::validation::ConfigContext {
        actor: &actor,
        actor_index: 0,
        limits: &limits,
    };
    let zero_duration = GearProcConfig::ThiefCandyDodge {
        damage_bonus: Ratio {
            numerator: 2,
            denominator: 25,
        },
        duration_us: 0,
    };
    let err = crate::weapons::thief_candy::validate_config(&zero_duration, &ctx)
        .err()
        .expect("expected zero duration rejection");
    assert!(err.contains("positive duration"));
    let bad_ratio = GearProcConfig::ThiefCandyDodge {
        damage_bonus: Ratio {
            numerator: 2,
            denominator: 0,
        },
        duration_us: 10_000_000,
    };
    assert!(crate::weapons::thief_candy::validate_config(&bad_ratio, &ctx).is_err());
}

#[test]
fn thief_candy_buff_granted_only_by_tagged_dodge() {
    let req = cast_request();
    let mut state = GameSimulatorState::new(&req);
    grant_on_cast(false, &req, &mut state, 0);
    grant_on_cast(false, &req, &mut state, 1_000_000);
    assert!(buff_expiries(&state).is_empty());
    grant_on_cast(true, &req, &mut state, 2_000_000);
    let expiries = buff_expiries(&state);
    assert_eq!(expiries.len(), 1);
    assert_eq!(expiries[0], 12_000_000);
    let buff = state
        .buffs
        .instances()
        .iter()
        .find(|buff| buff.id == crate::weapons::thief_candy::BUFF_ID)
        .unwrap();
    assert_eq!(buff.source, "weapon:fork_ThiefCandy");
    assert_eq!(buff.owner_actor, 0);
}

#[test]
fn thief_candy_stacks_cap_at_three_and_refresh_duration() {
    let req = cast_request();
    let mut state = GameSimulatorState::new(&req);
    grant_on_cast(true, &req, &mut state, 0);
    grant_on_cast(true, &req, &mut state, 2_000_000);
    grant_on_cast(true, &req, &mut state, 4_000_000);
    grant_on_cast(true, &req, &mut state, 6_000_000);
    let expiries = buff_expiries(&state);
    assert_eq!(expiries.len(), 1, "Reset policy keeps one instance");
    assert_eq!(expiries[0], 16_000_000);
    let stacks = state
        .buffs
        .instances()
        .iter()
        .find(|buff| buff.id == crate::weapons::thief_candy::BUFF_ID)
        .unwrap()
        .stacks;
    assert_eq!(stacks, 3, "fourth grant at the cap adds no stack");
    assert_eq!(
        damage_bonus_at(&state, 7_000_000),
        Some(ExactFraction::new(6, 25).unwrap())
    );
}

#[test]
fn thief_candy_retrigger_resets_expiry() {
    let req = cast_request();
    let mut state = GameSimulatorState::new(&req);
    grant_on_cast(true, &req, &mut state, 0);
    assert_eq!(buff_expiries(&state), vec![10_000_000]);
    grant_on_cast(true, &req, &mut state, 9_000_000);
    assert_eq!(buff_expiries(&state), vec![19_000_000]);
    let stacks = state
        .buffs
        .instances()
        .iter()
        .find(|buff| buff.id == crate::weapons::thief_candy::BUFF_ID)
        .unwrap()
        .stacks;
    assert_eq!(stacks, 2);
}

#[test]
fn thief_candy_buff_expires_after_duration() {
    let req = cast_request();
    let mut state = GameSimulatorState::new(&req);
    grant_on_cast(true, &req, &mut state, 0);
    assert_eq!(
        damage_bonus_at(&state, 9_999_999),
        Some(ExactFraction::new(2, 25).unwrap())
    );
    assert_eq!(damage_bonus_at(&state, 10_000_001), None);
}

#[test]
fn thief_candy_damage_bonus_joins_additive_zone() {
    let req = cast_request();
    let mut state = GameSimulatorState::new(&req);
    grant_on_cast(true, &req, &mut state, 0);
    grant_on_cast(true, &req, &mut state, 1_000_000);
    let bonus = damage_bonus_at(&state, 2_000_000).unwrap();
    let zone = ExactFraction::new(1, 1).unwrap().add(bonus).unwrap();
    assert_eq!(zone, ExactFraction::new(29, 25).unwrap());
}

#[test]
fn thief_candy_buff_surfaces_damage_bonus_through_damage_path_query() {
    let req = cast_request();
    let mut state = GameSimulatorState::new(&req);
    grant_on_cast(true, &req, &mut state, 0);
    grant_on_cast(true, &req, &mut state, 1_000_000);
    grant_on_cast(true, &req, &mut state, 2_000_000);
    // hit/calculation.rs reads live buff stats through this exact query when a
    // hit is delivered; the staged modifier must reach it as 3 x 2/25.
    let living = BTreeSet::from([0usize]);
    let stats = state
        .buffs
        .get_stat_modifiers_for_actor_with_living_sources(0, 0, 3_000_000, &living)
        .unwrap();
    assert_eq!(
        stats["damage_bonus"],
        Ratio {
            numerator: 6,
            denominator: 25
        }
    );
    // The 10s window refreshed at t=2s expires at t=12s; afterwards the
    // consumer query sees no damage_bonus at all.
    let expired = state
        .buffs
        .get_stat_modifiers_for_actor_with_living_sources(0, 0, 12_000_001, &living)
        .unwrap();
    assert!(!expired.contains_key("damage_bonus"));
}

#[test]
fn thief_candy_additive_zone_rejects_multiplicative_misreading() {
    let req = cast_request();
    let mut state = GameSimulatorState::new(&req);
    grant_on_cast(true, &req, &mut state, 0);
    grant_on_cast(true, &req, &mut state, 1_000_000);
    grant_on_cast(true, &req, &mut state, 2_000_000);
    grant_on_cast(true, &req, &mut state, 3_000_000);
    let one = ExactFraction::new(1, 1).unwrap();
    // Cap: the fourth grant adds no stack; 3 x 2/25 = 6/25, M_bonus = 31/25.
    let zone = one
        .add(damage_bonus_at(&state, 4_000_000).unwrap())
        .unwrap();
    assert_eq!(zone, ExactFraction::new(31, 25).unwrap());
    // A multiplicative misreading would give (27/25)^3 = 19683/15625.
    assert_ne!(zone, ExactFraction::new(19683, 15625).unwrap());

    let mut state = GameSimulatorState::new(&req);
    grant_on_cast(true, &req, &mut state, 0);
    grant_on_cast(true, &req, &mut state, 1_000_000);
    // Two stacks: M_bonus = 1 + 4/25 = 29/25; multiplicative 1.08^2 = 729/625 rejected.
    let zone = one
        .add(damage_bonus_at(&state, 2_000_000).unwrap())
        .unwrap();
    assert_eq!(zone, ExactFraction::new(29, 25).unwrap());
    assert_ne!(zone, ExactFraction::new(729, 625).unwrap());
}

#[test]
fn thief_candy_critical_dash_reaches_sakiri_skill_and_expires_at_the_source_boundary() {
    let mut req = cast_request();
    let actor = &mut req.actors[0];
    actor.character_id = "character:1003".into();
    actor.talent_levels = Some([8; 3]);
    actor.character_max_level = Some(80);
    actor.character_level = 80;
    actor.static_stats.base_atk = 1000;
    actor.static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    actor.static_stats.elemental_bonus = Ratio::zero();
    actor.static_stats.crit_rate = Ratio::zero();
    actor.static_stats.damage_bonus = Ratio::zero();
    req.targets[0].level = 80;
    req.targets[0].defense = 0;
    req.targets[0].resistance = Ratio::zero();
    req.relative_duration_us = 12_000_000;
    req.actions = vec![
        critical_dash(0),
        critical_dash(500_000),
        critical_dash(1_000_000),
        GameActionCommand::InvokeAbility {
            timestamp_us: 1_500_000,
            actor_index: 0,
            ability_id: "GA_Sagiri_Skill".into(),
            array_index: 7,
            damage_effect_id: "GE_Player_Sagiri_Skill1_Damage".into(),
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 1_200_000,
            cooldown_us: 6_000_000,
            hit_delay_us: 0,
        },
        GameActionCommand::InvokeAbility {
            timestamp_us: 11_000_000,
            actor_index: 0,
            ability_id: "GA_Sagiri_Melee".into(),
            array_index: 7,
            damage_effect_id: "GE_Player_Sagiri_Melee1_Damage".into(),
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 800_000,
            cooldown_us: 0,
            hit_delay_us: 0,
        },
    ];
    req.action_metadata.clear();

    let report = run_game_scenario_report(&req, &ExecutionLimits::default()).unwrap();
    assert!(report.execution_completed);
    assert_eq!(report.player_input_receipts.len(), 3);
    assert!(report.player_input_receipts.iter().all(|receipt| {
        receipt.input == PlayerInput::Dash
            && receipt.critical_success
            && receipt
                .execution_model_assumption_ids
                .iter()
                .any(|id| id == super::super::inputs::CRITICAL_DODGE_ID.as_str())
            && receipt
                .execution_model_assumption_ids
                .iter()
                .any(|id| id == crate::weapons::thief_candy::HYPOTHESIS_ID.as_str())
    }));
    assert_eq!(
        report
            .resolved_model_assumptions
            .get(super::super::inputs::CRITICAL_DODGE_ID.as_str())
            .map(String::as_str),
        Some(super::super::inputs::CRITICAL_DODGE_ID.description())
    );
    assert_eq!(
        report
            .resolved_model_assumptions
            .get(crate::weapons::thief_candy::HYPOTHESIS_ID.as_str())
            .map(String::as_str),
        Some(crate::weapons::thief_candy::HYPOTHESIS_ID.description())
    );
    assert_eq!(report.hit_receipts.len(), 3);
    // T1 gives 3 x 8% in M_bonus: (2.876 + 3.979) x 1000 x 31/25 = 8500.2.
    assert_eq!(
        formula_damage_total(&report),
        ExactFraction::new(45_956, 5).unwrap()
    );
    assert_eq!(
        report.hit_receipts[0].non_crit_damage,
        ExactFraction::new(89_156, 25).unwrap()
    );
    assert_eq!(
        report.hit_receipts[1].non_crit_damage,
        ExactFraction::new(123_349, 25).unwrap()
    );
    // Last dash is at 1s, so the 10s buff ends at 11s; the boundary is exclusive.
    assert_eq!(
        report.hit_receipts[2].non_crit_damage,
        ExactFraction::new(691, 1).unwrap()
    );
}
