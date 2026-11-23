use super::*;
use crate::runtime_api::{CastContext, DamageContext, HitModifiers};
use crate::simulation::metadata::{ActionMetadata, InputActionRole, MetadataProvenance};
use crate::simulation::state::GameSimulatorState;
use crate::simulation::timer_queue::EventQueue as BinaryHeap;
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};

fn t1_proc() -> GearProcConfig {
    GearProcConfig::TigerTallyAtk {
        atk_bonus: Ratio {
            numerator: 3,
            denominator: 20,
        },
        normal_damage_bonus: Ratio {
            numerator: 3,
            denominator: 20,
        },
        duration_us: 15_000_000,
        talisman_window_us: 15_000_000,
        boss_damage_bonus: Ratio {
            numerator: 1,
            denominator: 10,
        },
        boss_damage_duration_us: 10_000_000,
    }
}

fn projection_tables() -> BTreeMap<String, serde_json::Value> {
    super::support::weapon_projection_tables()
}

fn tier_curve(tier: u32) -> (u64, u64) {
    match tier {
        1 => (3, 20),
        2 => (3, 16),
        3 => (9, 40),
        4 => (21, 80),
        5 => (3, 10),
        _ => unreachable!(),
    }
}

#[test]
fn tiger_tally_projection_t1_to_t5() {
    let tables = projection_tables();
    let req = research_request();
    let mut actor = req.actors[0].clone();
    actor.character_id = "character:1025".into();
    actor.weapon_id = "weapon:fork_TigerTally".into();
    actor.gear_procs.clear();
    for tier in 1..=5u32 {
        actor.weapon_tier = tier;
        let report = crate::weapons::tiger_tally::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut actor,
            if tier == 1 { &[] } else { &[0] },
        )
        .unwrap();
        let proc = &report["compiled_proc"];
        let (num, den) = tier_curve(tier);
        let expected = || ExactFraction::new(num.into(), den.into()).unwrap();
        let ratio_of = |key: &str| {
            ExactFraction::from_ratio(serde_json::from_value(proc[key].clone()).unwrap()).unwrap()
        };
        assert_eq!(ratio_of("atk_bonus"), expected(), "tier {tier} atk curve");
        assert_eq!(
            ratio_of("normal_damage_bonus"),
            expected(),
            "tier {tier} normal curve"
        );
        let expected_boss = ExactFraction::from_ratio(Ratio {
            numerator: u64::from(tier + 3),
            denominator: 40,
        })
        .unwrap();
        assert_eq!(
            ratio_of("boss_damage_bonus"),
            expected_boss,
            "tier {tier} Boss damage curve"
        );
        assert_eq!(
            proc["duration_us"],
            json!(15_000_000),
            "tier {tier} duration"
        );
        assert_eq!(proc["talisman_window_us"], json!(15_000_000));
        assert_eq!(proc["boss_damage_duration_us"], json!(10_000_000));
        assert_eq!(actor.gear_procs.len(), 1);
        assert_eq!(report["replaced_proc"].is_null(), tier == 1);
    }
}

#[test]
fn tiger_tally_projection_rejects_group_and_unit_changes() {
    let tables = projection_tables();
    let req = research_request();
    let mut actor = req.actors[0].clone();
    actor.character_id = "character:1019".into();
    actor.weapon_id = "weapon:fork_TigerTally".into();
    actor.weapon_tier = 1;
    actor.gear_procs.clear();
    assert!(
        crate::weapons::tiger_tally::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut actor,
            &[]
        )
        .unwrap_err()
        .contains("incompatible")
    );

    let mut mutated = tables.clone();
    let row = &mut mutated.get_mut("DT_ForkUpgradeStarDataTable").unwrap()["rows"]["upgradestar_pack_fork_TigerTally_5"]
        ["DataList"];
    row[0]["bIsPercent"] = json!(false);
    let mut legal = req.actors[0].clone();
    legal.character_id = "character:1025".into();
    legal.weapon_id = "weapon:fork_TigerTally".into();
    legal.weapon_tier = 5;
    legal.gear_procs.clear();
    assert!(
        crate::weapons::tiger_tally::projection::project(
            &mutated,
            1,
            "ntesim-current",
            &mut legal,
            &[]
        )
        .unwrap_err()
        .contains("unit hint")
    );
}

#[test]
fn tiger_tally_projection_binds_talisman_curves() {
    let tables = projection_tables();
    let req = research_request();
    let mut actor = req.actors[0].clone();
    actor.character_id = "character:1025".into();
    actor.weapon_id = "weapon:fork_TigerTally".into();
    actor.weapon_tier = 1;
    actor.gear_procs.clear();
    let report = crate::weapons::tiger_tally::projection::project(
        &tables,
        1,
        "ntesim-current",
        &mut actor,
        &[],
    )
    .unwrap();
    let proc = &report["compiled_proc"];
    assert!(proc.get("kill_heal").is_none());
    assert_eq!(
        ExactFraction::from_ratio(
            serde_json::from_value(proc["boss_damage_bonus"].clone()).unwrap()
        )
        .unwrap(),
        ExactFraction::new(1, 10).unwrap()
    );
    assert_eq!(proc["talisman_window_us"], json!(15_000_000));
    assert_eq!(proc["boss_damage_duration_us"], json!(10_000_000));
    for excluded in [
        "buff_TigerTally_CD4",
        "buff_TigerTally_Qup",
        "buff_TigerTally_CD3",
    ] {
        assert!(
            report["native_parameter_report"]["parameters"]
                .as_array()
                .unwrap()
                .iter()
                .any(|p| p["name_id"] == excluded),
            "{excluded} stays in the frozen parameter report"
        );
    }
    assert_eq!(
        report["excluded_native_clauses"].as_array().unwrap().len(),
        0
    );
}

#[test]
fn tiger_tally_config_rejects_invalid() {
    let limits = ExecutionLimits::default();
    let mut req = research_request();
    req.actors[0].weapon_id = "weapon:fork_TigerTally".into();
    let actor = &req.actors[0];
    let ctx = crate::gear::validation::ConfigContext {
        actor,
        actor_index: 0,
        limits: &limits,
    };
    let validation = crate::weapons::tiger_tally::validate_config(&t1_proc(), &ctx).unwrap();
    assert_eq!(validation.kind, "tiger_tally");
    assert_eq!(validation.duration_us, 15_000_000);

    let mut wrong_weapon = req.actors[0].clone();
    wrong_weapon.weapon_id = "weapon:fork_Kite".into();
    let ctx = crate::gear::validation::ConfigContext {
        actor: &wrong_weapon,
        actor_index: 0,
        limits: &limits,
    };
    assert!(
        crate::weapons::tiger_tally::validate_config(&t1_proc(), &ctx)
            .err()
            .expect("expected weapon mismatch rejection")
            .contains("weapon:fork_TigerTally")
    );

    let zero_duration = GearProcConfig::TigerTallyAtk {
        atk_bonus: Ratio {
            numerator: 3,
            denominator: 20,
        },
        normal_damage_bonus: Ratio {
            numerator: 3,
            denominator: 20,
        },
        duration_us: 0,
        talisman_window_us: 15_000_000,
        boss_damage_bonus: Ratio {
            numerator: 1,
            denominator: 10,
        },
        boss_damage_duration_us: 10_000_000,
    };
    let ctx = crate::gear::validation::ConfigContext {
        actor,
        actor_index: 0,
        limits: &limits,
    };
    assert!(
        crate::weapons::tiger_tally::validate_config(&zero_duration, &ctx)
            .err()
            .expect("expected zero duration rejection")
            .contains("positive durations")
    );
    let bad_ratio = GearProcConfig::TigerTallyAtk {
        atk_bonus: Ratio {
            numerator: 1,
            denominator: 0,
        },
        normal_damage_bonus: Ratio {
            numerator: 3,
            denominator: 20,
        },
        duration_us: 15_000_000,
        talisman_window_us: 15_000_000,
        boss_damage_bonus: Ratio {
            numerator: 1,
            denominator: 10,
        },
        boss_damage_duration_us: 10_000_000,
    };
    assert!(crate::weapons::tiger_tally::validate_config(&bad_ratio, &ctx).is_err());
}

#[test]
fn tiger_tally_v2_hypothesis_records_and_validates_conflicts() {
    let mut req = cast_request();
    req.model_assumptions.clear();
    let mut resolved = std::collections::BTreeMap::new();
    crate::runtime_api::record_hypothesis(
        &req,
        &mut resolved,
        crate::weapons::tiger_tally::HYPOTHESIS_ID,
        crate::weapons::tiger_tally::HYPOTHESIS_ID.description(),
    )
    .unwrap();
    assert_eq!(
        resolved[crate::weapons::tiger_tally::HYPOTHESIS_ID.as_str()],
        crate::weapons::tiger_tally::HYPOTHESIS_ID.description()
    );

    req.model_assumptions.insert(
        crate::weapons::tiger_tally::HYPOTHESIS_ID.into(),
        "The earlier Normal-only model does not include the talisman and Arc clauses.".into(),
    );
    let mismatch = crate::runtime_api::record_hypothesis(
        &req,
        &mut resolved,
        crate::weapons::tiger_tally::HYPOTHESIS_ID,
        crate::weapons::tiger_tally::HYPOTHESIS_ID.description(),
    )
    .unwrap_err();
    assert!(mismatch.contains("conflicting reached model hypothesis"));
}

fn atk_request() -> GameScenarioRequest {
    let mut req = research_request();
    req.stochastic_crit = None;
    let actor = &mut req.actors[0];
    actor.character_id = "character:1025".into();
    actor.weapon_id = "weapon:fork_TigerTally".into();
    actor.weapon_tier = 1;
    actor.gear_procs = vec![t1_proc()];
    actor.static_stats.base_atk = 1000;
    actor.static_stats.atk = Ratio {
        numerator: 2000,
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

#[test]
fn tiger_tally_static_atk_is_base_scaled_not_static_multiplied() {
    let req = atk_request();
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(
        report["callback_model_assumptions"][crate::weapons::tiger_tally::HYPOTHESIS_ID.as_str()],
        crate::weapons::tiger_tally::HYPOTHESIS_ID.description()
    );
    // 2000 static ATK + 1000 base ATK * 3/20 = 2150; static * 1.15 would be 2300.
    assert_eq!(
        report["hit_receipts"][0]["non_crit_damage"],
        json!("2150/1")
    );
    assert_eq!(report["hit_receipts"][0]["rounded_damage"], 2150);

    let mut bare = req.clone();
    bare.actors[0].gear_procs.clear();
    let baseline = run_game_scenario(&bare, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        baseline["hit_receipts"][0]["non_crit_damage"],
        json!("2000/1")
    );
}

fn action_metadata_with_role(role: Option<InputActionRole>) -> ActionMetadata {
    ActionMetadata {
        categories: BTreeSet::new(),
        gameplay_tags: vec![],
        ability_id: "caller-test".into(),
        native_ability_index: None,
        input_role: role,
        input_role_provenance: None,
        primary_slot: None,
        source_evidence: None,
        identity_evidence: None,
        provenance: MetadataProvenance::AcceptedModelHypothesis {
            id: "test.tiger_tally_cast".into(),
            description: "Test-owned cast metadata.".into(),
        },
    }
}

fn grant_on_cast(
    role: Option<InputActionRole>,
    req: &GameScenarioRequest,
    state: &mut GameSimulatorState,
    time_us: u64,
) {
    let metadata = action_metadata_with_role(role);
    let mut assumptions = BTreeMap::new();
    let mut queue = BinaryHeap::new();
    let mut sequence = 0u64;
    let mut ctx = CastContext {
        origins: &mut state.origins,
        command_index: 0,
        native_array_index: None,
        execution_assumptions: &mut assumptions,
        config: &req.actors[0],
        request: req,
        actor_index: 0,
        active_actor_index: 0,
        target_index: 0,
        ability_id: "caller-test",
        action_metadata: &metadata,
        packet_metadata: None,
        hit_time_us: time_us,
        world_time_us: time_us,
        time_us,
        buffs: &mut state.buffs,
        actor: &mut state.actors[0],
        stamina: state.combat_stamina.as_mut(),
        team_hp_snapshot: &[],
        targets: &[],
        queue: &mut queue,
        sequence: &mut sequence,
    };
    crate::gear::on_cast(&mut ctx).unwrap();
}

fn buff_expiries(state: &GameSimulatorState) -> Vec<u64> {
    state
        .buffs
        .instances()
        .iter()
        .filter(|buff| buff.id == crate::weapons::tiger_tally::BUFF_ID)
        .map(|buff| match buff.duration {
            ntesim_core::buffs::dispatch::BuffDuration::Timed { expires_at_us, .. } => {
                expires_at_us
            }
            ntesim_core::buffs::dispatch::BuffDuration::Permanent => u64::MAX,
        })
        .collect()
}

fn cast_request() -> GameScenarioRequest {
    let mut req = research_request();
    req.stochastic_crit = None;
    req.actors[0].weapon_id = "weapon:fork_TigerTally".into();
    req.actors[0].gear_procs = vec![t1_proc()];
    req
}

#[test]
fn tiger_tally_buff_grants_on_redirect_and_ultimate_only() {
    let req = cast_request();
    let mut state = GameSimulatorState::new(&req);
    grant_on_cast(Some(InputActionRole::RedirectSkill), &req, &mut state, 0);
    assert_eq!(buff_expiries(&state).len(), 1);

    grant_on_cast(Some(InputActionRole::Ultimate), &req, &mut state, 1_000_000);
    assert_eq!(buff_expiries(&state).len(), 2);

    grant_on_cast(Some(InputActionRole::Normal), &req, &mut state, 2_000_000);
    grant_on_cast(Some(InputActionRole::Qte), &req, &mut state, 3_000_000);
    grant_on_cast(None, &req, &mut state, 4_000_000);
    assert_eq!(buff_expiries(&state).len(), 2);
}

#[test]
fn tiger_tally_buff_caps_at_two_stacks_and_expires() {
    let req = cast_request();
    let mut state = GameSimulatorState::new(&req);
    grant_on_cast(Some(InputActionRole::RedirectSkill), &req, &mut state, 0);
    grant_on_cast(Some(InputActionRole::Ultimate), &req, &mut state, 1_000_000);
    grant_on_cast(
        Some(InputActionRole::RedirectSkill),
        &req,
        &mut state,
        2_000_000,
    );
    let expiries = buff_expiries(&state);
    assert_eq!(expiries.len(), 2, "third grant at the cap is dropped");
    assert_eq!(
        expiries,
        vec![15_000_000, 16_000_000],
        "independent per-instance expiry"
    );

    let live = |time_us: u64| {
        state
            .buffs
            .get_stat_modifiers_for_actor(0, 0, time_us)
            .unwrap()
            .get(crate::weapons::tiger_tally::NORMAL_DAMAGE_BONUS_STAT)
            .cloned()
    };
    let ratio_value = |ratio: ntesim_core::formula::Ratio| {
        ExactFraction::new(ratio.numerator.into(), ratio.denominator.into()).unwrap()
    };
    assert_eq!(
        live(14_999_999).map(ratio_value),
        Some(ExactFraction::new(3, 10).unwrap()),
        "both instances live before 15s"
    );
    assert_eq!(
        live(15_000_000).map(ratio_value),
        Some(ExactFraction::new(3, 20).unwrap()),
        "first instance expires at exactly 15s, second survives"
    );
    assert!(
        live(16_000_000).is_none(),
        "second instance expires at exactly 16s"
    );
}

fn arc_buff_expiries(state: &GameSimulatorState) -> Vec<u64> {
    state
        .buffs
        .instances()
        .iter()
        .filter(|buff| buff.id == crate::weapons::tiger_tally::ARC_BUFF_ID)
        .map(|buff| match buff.duration {
            ntesim_core::buffs::dispatch::BuffDuration::Timed { expires_at_us, .. } => {
                expires_at_us
            }
            ntesim_core::buffs::dispatch::BuffDuration::Permanent => u64::MAX,
        })
        .collect()
}

#[test]
fn tiger_tally_opposite_talismans_activate_arc_in_either_order_and_at_strict_boundary() {
    let req = cast_request();
    let mut left_then_right = GameSimulatorState::new(&req);
    grant_on_cast(
        Some(InputActionRole::RedirectSkill),
        &req,
        &mut left_then_right,
        0,
    );
    assert_eq!(
        left_then_right.actors[0]
            .equipment
            .tiger_tally
            .left_talisman_until_us,
        Some(15_000_000)
    );
    grant_on_cast(
        Some(InputActionRole::Ultimate),
        &req,
        &mut left_then_right,
        14_999_999,
    );
    assert_eq!(arc_buff_expiries(&left_then_right), vec![24_999_999]);
    assert_eq!(
        left_then_right.actors[0]
            .equipment
            .tiger_tally
            .left_talisman_until_us,
        None
    );
    assert_eq!(
        left_then_right.actors[0]
            .equipment
            .tiger_tally
            .right_talisman_until_us,
        None,
        "the paired talismans are consumed by Arc"
    );

    let mut right_then_left = GameSimulatorState::new(&req);
    grant_on_cast(
        Some(InputActionRole::Ultimate),
        &req,
        &mut right_then_left,
        0,
    );
    grant_on_cast(
        Some(InputActionRole::RedirectSkill),
        &req,
        &mut right_then_left,
        14_999_999,
    );
    assert_eq!(arc_buff_expiries(&right_then_left), vec![24_999_999]);

    let mut same_side_refresh = GameSimulatorState::new(&req);
    grant_on_cast(
        Some(InputActionRole::RedirectSkill),
        &req,
        &mut same_side_refresh,
        0,
    );
    grant_on_cast(
        Some(InputActionRole::RedirectSkill),
        &req,
        &mut same_side_refresh,
        10_000_000,
    );
    assert_eq!(
        same_side_refresh.actors[0]
            .equipment
            .tiger_tally
            .left_talisman_until_us,
        Some(25_000_000),
        "same-side grants refresh their own window"
    );
    grant_on_cast(
        Some(InputActionRole::Ultimate),
        &req,
        &mut same_side_refresh,
        24_999_999,
    );
    assert_eq!(arc_buff_expiries(&same_side_refresh), vec![34_999_999]);

    let mut boundary = GameSimulatorState::new(&req);
    grant_on_cast(Some(InputActionRole::RedirectSkill), &req, &mut boundary, 0);
    grant_on_cast(
        Some(InputActionRole::Ultimate),
        &req,
        &mut boundary,
        15_000_000,
    );
    assert!(
        arc_buff_expiries(&boundary).is_empty(),
        "a talisman expiring at cast time cannot trigger Arc"
    );
    assert_eq!(
        boundary.actors[0]
            .equipment
            .tiger_tally
            .right_talisman_until_us,
        Some(30_000_000),
        "the new right talisman starts a fresh window"
    );
}

fn damage_modifiers(
    req: &GameScenarioRequest,
    state: &GameSimulatorState,
    role: Option<InputActionRole>,
    time_us: u64,
) -> HitModifiers {
    damage_modifiers_for_target(req, state, role, time_us, 0)
}

fn damage_modifiers_for_target(
    req: &GameScenarioRequest,
    state: &GameSimulatorState,
    role: Option<InputActionRole>,
    time_us: u64,
    target_index: usize,
) -> HitModifiers {
    let category = match role {
        Some(InputActionRole::Normal) => "A",
        Some(InputActionRole::CriticalRiposte) => "Z",
        Some(InputActionRole::RedirectSkill) => "E",
        Some(InputActionRole::Ultimate) => "Q",
        _ => "H",
    };
    damage_modifiers_for_packet(req, state, role, time_us, target_index, Some(category)).unwrap()
}

fn damage_modifiers_for_packet(
    req: &GameScenarioRequest,
    state: &GameSimulatorState,
    role: Option<InputActionRole>,
    time_us: u64,
    target_index: usize,
    category: Option<&str>,
) -> Result<HitModifiers, String> {
    let action_metadata = action_metadata_with_role(role);
    let packet = crate::simulation::metadata::PacketMetadata {
        damage_effect_id: "GE_Test_Damage".into(),
        native_ga_name: None,
        native_damage_type_ex: "EExecutionDamageTypeEX::DAMAGE_TYPE_NATURE".into(),
        native_damage_source_category: category
            .map(|value| format!("EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_{value}")),
        damage_kind: ntesim_core::combat::DamageKind::Elemental,
        element: None,
        fixed_crit_rate: None,
        unbal_value: None,
        fixed_crit_rate_provenance: None,
        unbal_value_provenance: None,
        semantic_tags: BTreeSet::new(),
        semantic_provenance: None,
        source_evidence: None,
        provenance: MetadataProvenance::AcceptedModelHypothesis {
            id: "test".into(),
            description: "Test-owned packet metadata.".into(),
        },
        packet_kind: Default::default(),
    };
    let owned_status_counts = BTreeMap::new();
    let buff_stats = state
        .buffs
        .get_stat_modifiers_for_actor(0, 0, time_us)
        .unwrap();
    let ctx = DamageContext {
        foreign_dot_stacks: 0,
        packet_group_id: None,
        command_index: 0,
        config: &req.actors[0],
        start_fons: None,
        ability_id: "caller-test",
        action_metadata: &action_metadata,
        target: &state.targets[target_index],
        actor: &state.actors[0],
        world_time_us: time_us,
        time_us,
        debuff_count: 0,
        damage_effect_id: "GE_Test_Damage",
        actor_index: 0,
        active_actor_index: 0,
        target_count: 1,
        non_boss_target_count: 1,
        target_index,
        buffs: &state.buffs,
        buff_stats: (&buff_stats).into(),
        owned_status_counts: &owned_status_counts,
        team_has_three_elements: false,
    };
    crate::gear::damage_bonus(
        &ctx,
        &packet,
        HitModifiers::new(ExactFraction::new(0, 1).unwrap()).unwrap(),
    )
}

#[test]
fn tiger_tally_classifies_delivered_packets_independently_of_the_input_role() {
    let req = cast_request();
    let mut state = GameSimulatorState::new(&req);
    grant_on_cast(Some(InputActionRole::RedirectSkill), &req, &mut state, 0);
    for role in [
        Some(InputActionRole::Normal),
        Some(InputActionRole::Ultimate),
        None,
    ] {
        for category in ["A", "Z", "B", "E", "F", "H", "P", "Q", "R", "T", "X"] {
            let modifiers =
                damage_modifiers_for_packet(&req, &state, role, 1, 0, Some(category)).unwrap();
            assert_eq!(
                modifiers.damage_bonus,
                ExactFraction::new(u128::from(matches!(category, "A" | "Z")) * 3, 20).unwrap(),
                "{role:?} child category {category}",
            );
        }
        for category in [None, Some("UNKNOWN")] {
            assert!(
                damage_modifiers_for_packet(&req, &state, role, 1, 0, category)
                    .unwrap_err()
                    .contains("damage source classification")
            );
        }
    }
    // Classification is irrelevant after the last category-restricted stack expires.
    assert!(
        damage_modifiers_for_packet(&req, &state, None, 15_000_000, 0, None)
            .unwrap()
            .damage_bonus
            .is_zero()
    );
}

fn apply_boss_status(
    buffs: &mut ntesim_core::buffs::dispatch::BuffManager,
    target_index: usize,
    duration_us: u64,
) {
    crate::simulation::state::apply_target_tagged_status(
        buffs,
        crate::simulation::state::TargetStatusDeclaration {
            source_actor: 0,
            source: "test.boss".into(),
            target_index,
            id: "test.boss_status".into(),
            now_us: 0,
            duration_us,
            reductions: BTreeMap::new(),
        },
        BTreeSet::from(["ntesim.status.boss".into()]),
        1,
        ntesim_core::buffs::dispatch::RefreshPolicy::Reset,
        ntesim_core::buffs::dispatch::DecayPolicy::AllAtOnce,
    )
    .unwrap();
}

#[test]
fn tiger_tally_arc_bonus_is_additive_boss_gated_refreshes_and_expires() {
    let mut req = cast_request();
    req.targets.push(req.targets[0].clone());
    req.targets[1].target_index = 1;
    req.targets[1].is_boss = true;
    let mut state = GameSimulatorState::new(&req);
    grant_on_cast(Some(InputActionRole::RedirectSkill), &req, &mut state, 0);
    grant_on_cast(Some(InputActionRole::Ultimate), &req, &mut state, 1_000_000);
    grant_on_cast(
        Some(InputActionRole::RedirectSkill),
        &req,
        &mut state,
        4_000_000,
    );
    grant_on_cast(Some(InputActionRole::Ultimate), &req, &mut state, 5_000_000);
    assert_eq!(
        arc_buff_expiries(&state),
        vec![15_000_000],
        "a new Arc activation resets one existing buff"
    );
    // A legacy synthetic tag cannot turn the non-Boss target into a Boss.
    apply_boss_status(&mut state.buffs, 0, 30_000_000);

    let non_boss =
        damage_modifiers_for_target(&req, &state, Some(InputActionRole::Normal), 6_000_000, 0);
    assert_eq!(non_boss.damage_bonus, ExactFraction::new(3, 10).unwrap());

    let boss_normal =
        damage_modifiers_for_target(&req, &state, Some(InputActionRole::Normal), 6_000_000, 1);
    assert_eq!(
        boss_normal.damage_bonus,
        ExactFraction::new(2, 5).unwrap(),
        "two Normal stacks and Arc add in M_bonus"
    );

    let boss_ultimate =
        damage_modifiers_for_target(&req, &state, Some(InputActionRole::Ultimate), 6_000_000, 1);
    assert_eq!(
        boss_ultimate.damage_bonus,
        ExactFraction::new(1, 10).unwrap(),
        "Arc applies to the wearer's non-Normal damage against Bosses"
    );

    let expired =
        damage_modifiers_for_target(&req, &state, Some(InputActionRole::Ultimate), 15_000_000, 1);
    assert!(
        expired.damage_bonus.is_zero(),
        "the refreshed 10s Arc bonus is expired at its exact deadline"
    );
}

#[test]
fn tiger_tally_arc_zone_math_hand_derived() {
    fn gcd(mut a: u128, mut b: u128) -> u128 {
        while b != 0 {
            let rest = a % b;
            a = b;
            b = rest;
        }
        a
    }
    fn reduced(n: u128, d: u128) -> (u128, u128) {
        let g = gcd(n, d);
        (n / g, d / g)
    }
    // Frozen buff_TigerTally_Qup values T1..T5: 0.10, 0.125, 0.15, 0.175, 0.20.
    // Hand-derived exact fractions: (tier + 3) / 40.
    let boss: Vec<(u128, u128)> = (1u128..=5).map(|t| reduced(t + 3, 40)).collect();
    assert_eq!(boss, [(1, 10), (1, 8), (3, 20), (7, 40), (1, 5)]);
    // Misreadings rejected: T2 is 1/8, not 1/10; T4 is 7/40, not 3/16.
    assert_ne!(boss[1], (1, 10));
    assert_ne!(boss[3], (3, 16));

    // T1: two Normal stacks (3/20 each) plus the T1 Arc bonus (1/10) join the
    // single additive damage-bonus zone. In fortieths: 6/40 + 6/40 + 4/40 =
    // 16/40 = 2/5, so M_bonus = 1 + 2/5 = 7/5 = 1.4.
    assert_eq!(reduced(6 + 6 + 4, 40), (2, 5));
    assert_eq!(reduced(40 + 16, 40), (7, 5));
    // A multiplicative reading (1 + 3/10) * (1 + 1/10) = 143/100 is rejected.
    assert_ne!(reduced(52 * 44, 40 * 40), reduced(7, 5));

    // Without the Boss flag the same two Normal stacks give bonus_sum 3/10 and
    // M_bonus 13/10; the Arc contribution is exactly zero, not merely small.
    assert_eq!(reduced(6 + 6, 40), (3, 10));
    assert_eq!(reduced(40 + 12, 40), (13, 10));
}

#[test]
fn tiger_tally_equipped_native_casts_use_scenario_boss_flag_at_all_refinements() {
    // With independent 25% DMG and two weapon stacks, Boss/non-Boss normal-hit
    // ratios are (1.25 + 2*normal + arc)/(1.25 + 2*normal). These constants are
    // hand-reduced from the five published refinement rows, not runtime output.
    for (tier, numerator, denominator) in [
        (1, 33, 31),
        (2, 14, 13),
        (3, 37, 34),
        (4, 78, 71),
        (5, 41, 37),
    ] {
        let run = |boss: bool| {
            let source = format!(
                "options iteration=1 duration=30 crit=fixed;\n\
                 hathor char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=100;\n\
                 hathor add weapon=fork_TigerTally refine={tier};\n\
                 hathor add stats dmg%=0.25;\n\
                 target hp=100000000 def=0 resist=0 boss={boss};\n\
                 active hathor; hathor skill[hold=0]; let arc=.time; hathor burst; wait(1); hathor attack; while .time<arc+10 {{wait(arc+10-.time);}} hathor attack;"
            );
            let executed =
                PreparedSourceScenario::new(&source, Default::default(), &Default::default())
                    .unwrap()
                    .run(42)
                    .unwrap();
            assert!(
                executed.report.success && executed.report.execution_completed,
                "{:?}",
                executed.diagnostic
            );
            let report = serde_json::to_value(executed.report).unwrap();
            assert_eq!(
                report["execution_completed"], true,
                "{} {}",
                report["halt_diagnostic"], report["rejected_action"]
            );
            assert_eq!(report["final_target_states"][0]["is_boss"], boss);
            report["hit_receipts"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|hit| hit["damage_effect_id"] == "GE_Player_Hathor_Melee1_Damage")
                .map(|hit| {
                    serde_json::from_value::<ExactFraction>(hit["non_crit_damage"].clone()).unwrap()
                })
                .collect::<Vec<_>>()
        };
        let plain = run(false);
        let boss = run(true);
        assert_eq!(boss.len(), 2);
        assert_eq!(
            boss[0],
            plain[0]
                .mul(ExactFraction::new(numerator, denominator).unwrap())
                .unwrap(),
            "T{tier} Arc joins M_bonus"
        );
        assert_eq!(
            boss[1], plain[1],
            "T{tier} Arc expires while Normal stacks survive"
        );
    }
}

#[test]
fn tiger_tally_damage_bonus_applies_to_normal_packets_only() {
    let req = cast_request();
    let mut state = GameSimulatorState::new(&req);
    grant_on_cast(Some(InputActionRole::RedirectSkill), &req, &mut state, 0);

    let normal = damage_modifiers(&req, &state, Some(InputActionRole::Normal), 1_000_000);
    assert_eq!(normal.atk_percent, ExactFraction::new(3, 20).unwrap());
    assert_eq!(
        normal.damage_bonus,
        ExactFraction::new(3, 20).unwrap(),
        "one stack adds 3/20 to the additive damage zone"
    );

    let riposte = damage_modifiers(
        &req,
        &state,
        Some(InputActionRole::CriticalRiposte),
        1_000_000,
    );
    assert_eq!(riposte.atk_percent, ExactFraction::new(3, 20).unwrap());
    assert_eq!(
        riposte.damage_bonus,
        ExactFraction::new(3, 20).unwrap(),
        "one stack adds 3/20 for Critical Riposte as well"
    );

    let redirect = damage_modifiers(
        &req,
        &state,
        Some(InputActionRole::RedirectSkill),
        1_000_000,
    );
    assert_eq!(redirect.atk_percent, ExactFraction::new(3, 20).unwrap());
    assert!(
        redirect.damage_bonus.is_zero(),
        "non-Normal packets get no buff damage"
    );

    grant_on_cast(Some(InputActionRole::Ultimate), &req, &mut state, 2_000_000);
    let stacked = damage_modifiers(&req, &state, Some(InputActionRole::Normal), 3_000_000);
    assert_eq!(
        stacked.damage_bonus,
        ExactFraction::new(3, 10).unwrap(),
        "two stacks sum to 3/10 in the additive zone"
    );
    let stacked_riposte = damage_modifiers(
        &req,
        &state,
        Some(InputActionRole::CriticalRiposte),
        3_000_000,
    );
    assert_eq!(
        stacked_riposte.damage_bonus,
        ExactFraction::new(3, 10).unwrap(),
        "two stacks sum to 3/10 for Critical Riposte in the additive zone"
    );

    let no_buff = GameSimulatorState::new(&req);
    let plain = damage_modifiers(&req, &no_buff, Some(InputActionRole::Normal), 1_000_000);
    assert_eq!(plain.atk_percent, ExactFraction::new(3, 20).unwrap());
    assert!(plain.damage_bonus.is_zero(), "no bonus without the buff");
}
