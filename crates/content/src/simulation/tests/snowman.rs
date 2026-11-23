use super::*;
use crate::runtime_api::HitModifiers;
use crate::simulation::state::GameSimulatorState;
use serde_json::json;

fn t1_proc() -> GearProcConfig {
    GearProcConfig::SnowmanShieldAtk {
        atk_bonus: Ratio {
            numerator: 9,
            denominator: 50,
        },
    }
}

fn projection_tables() -> std::collections::BTreeMap<String, serde_json::Value> {
    super::support::weapon_projection_tables()
}

#[test]
fn snowman_projection_binds_t1_t5_rejects_group_and_unit_changes() {
    let tables = projection_tables();
    let req = research_request();
    let mut actor = req.actors[0].clone();
    actor.character_id = "character:1023".into();
    actor.weapon_id = "weapon:fork_snowman".into();
    actor.gear_procs.clear();
    // Frozen CT_ForkBuff buff_snowman_AtkUp keys, hardcoded from the table.
    let expected: [(u32, u128, u128); 5] = [
        (1, 9, 50),
        (2, 21, 100),
        (3, 6, 25),
        (4, 27, 100),
        (5, 3, 10),
    ];
    for (tier, numerator, denominator) in expected {
        actor.weapon_tier = tier;
        let report = crate::weapons::snowman::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut actor,
            if tier == 1 { &[] } else { &[0] },
        )
        .unwrap();
        let proc = &report["compiled_proc"];
        let bonus =
            ExactFraction::from_ratio(serde_json::from_value(proc["atk_bonus"].clone()).unwrap())
                .unwrap();
        assert_eq!(
            bonus,
            ExactFraction::new(numerator, denominator).unwrap(),
            "tier {tier} ATK curve"
        );
        assert_eq!(
            report["native_description"]["localized"],
            json!("Increases ATK by <lv>{0}</> while the wearer has a Shield.")
        );
    }
    assert_eq!(
        actor.gear_procs.len(),
        1,
        "projection replaces the tier-1 proc in place"
    );

    let mut cross_group = req.actors[0].clone();
    cross_group.character_id = "character:1019".into();
    cross_group.weapon_id = "weapon:fork_snowman".into();
    cross_group.weapon_tier = 1;
    cross_group.gear_procs.clear();
    assert!(
        crate::weapons::snowman::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut cross_group,
            &[]
        )
        .unwrap_err()
        .contains("incompatible")
    );

    let mut unit_actor = req.actors[0].clone();
    unit_actor.character_id = "character:1023".into();
    unit_actor.weapon_id = "weapon:fork_snowman".into();
    unit_actor.weapon_tier = 1;
    unit_actor.gear_procs.clear();
    let mut hint_tables = projection_tables();
    hint_tables.get_mut("DT_ForkUpgradeStarDataTable").unwrap()["rows"]["upgradestar_pack_fork_snowman_1"]
        ["DataList"][0]["bIsPercent"] = json!(false);
    assert!(
        crate::weapons::snowman::projection::project(
            &hint_tables,
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
fn snowman_config_rejects_invalid_proc() {
    let req = research_request();
    let limits = ExecutionLimits::default();
    let mut wrong_weapon = req.actors[0].clone();
    wrong_weapon.weapon_id = "weapon:fork_Whale".into();
    let bad_ctx = crate::gear::validation::ConfigContext {
        actor: &wrong_weapon,
        actor_index: 0,
        limits: &limits,
    };
    match crate::weapons::snowman::validate_config(&t1_proc(), &bad_ctx) {
        Ok(_) => panic!("snowman proc accepted on the wrong weapon"),
        Err(error) => assert!(error.contains("fork_snowman"), "unexpected error: {error}"),
    }
    let bad_ratio = GearProcConfig::SnowmanShieldAtk {
        atk_bonus: Ratio {
            numerator: 1,
            denominator: 0,
        },
    };
    let bad_ratio_ctx = crate::gear::validation::ConfigContext {
        actor: &req.actors[0],
        actor_index: 0,
        limits: &limits,
    };
    assert!(crate::weapons::snowman::validate_config(&bad_ratio, &bad_ratio_ctx).is_err());
    let mut right_weapon = req.actors[0].clone();
    right_weapon.weapon_id = "weapon:fork_SNOWMAN".into();
    let ci_ctx = crate::gear::validation::ConfigContext {
        actor: &right_weapon,
        actor_index: 0,
        limits: &limits,
    };
    let validation = crate::weapons::snowman::validate_config(&t1_proc(), &ci_ctx)
        .unwrap_or_else(|error| panic!("valid snowman proc rejected: {error}"));
    assert_eq!(validation.kind, "snowman");
}

fn apply_shield(state: &mut GameSimulatorState, actor_index: usize, expires_at_us: u64) {
    state.actors[actor_index]
        .shields
        .push(crate::simulation::state::ActorShieldRuntime {
            id: "test-shield".into(),
            source: "test".into(),
            remaining_hp: 100,
            max_hp: 100,
            expires_at_us,
        });
}

fn zone_modifiers(
    req: &crate::simulation::GameScenarioRequest,
    state: &GameSimulatorState,
    time_us: u64,
) -> HitModifiers {
    super::support::gear_zone_modifiers(req, state, time_us, 0)
}

fn shield_request() -> crate::simulation::GameScenarioRequest {
    let mut req = research_request();
    req.stochastic_crit = None;
    let actor = &mut req.actors[0];
    actor.character_id = "character:1023".into();
    actor.weapon_id = "weapon:fork_snowman".into();
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
    req
}

#[test]
fn snowman_atk_bonus_applies_only_with_shield() {
    let req = shield_request();
    let plain_state = GameSimulatorState::new(&req);
    let plain = zone_modifiers(&req, &plain_state, 1);
    assert!(
        plain.atk_percent.is_zero(),
        "no ATK bonus without a shield on the wearer"
    );

    let mut shielded = GameSimulatorState::new(&req);
    apply_shield(&mut shielded, 0, 30_000_000);
    let shielded_modifiers = zone_modifiers(&req, &shielded, 1);
    assert_eq!(
        shielded_modifiers.atk_percent,
        ExactFraction::new(9, 50).unwrap(),
        "T1 18% ATK bonus joins the ATK percent zone while shielded"
    );
    assert!(
        shielded_modifiers.damage_bonus.is_zero(),
        "shield ATK bonus must not leak into the damage-bonus zone"
    );
    assert!(
        shielded_modifiers.final_bonus.is_zero(),
        "shield ATK bonus must not leak into the final zone"
    );
}

#[test]
fn snowman_bonus_follows_shield_expiry() {
    let req = shield_request();
    let mut state = GameSimulatorState::new(&req);
    apply_shield(&mut state, 0, 10_000_000);
    let before = zone_modifiers(&req, &state, 9_999_999);
    assert_eq!(
        before.atk_percent,
        ExactFraction::new(9, 50).unwrap(),
        "bonus live just before shield expiry"
    );
    let after = zone_modifiers(&req, &state, 10_000_001);
    assert!(
        after.atk_percent.is_zero(),
        "bonus gone once the shield status expires"
    );
}

#[test]
fn snowman_requires_a_positive_live_shield_on_the_packet_owner() {
    let req = shield_request();
    let mut state = GameSimulatorState::new(&req);
    apply_shield(&mut state, 0, 10);
    state.actors[0].shields[0].remaining_hp = 0;
    assert!(zone_modifiers(&req, &state, 1).atk_percent.is_zero());
    state.actors[0].shields[0].remaining_hp = 100;
    assert!(zone_modifiers(&req, &state, 10).atk_percent.is_zero());
    state.actors[0].shields[0].expires_at_us = 11;
    state.actors[0].hp = 0;
    assert!(zone_modifiers(&req, &state, 1).atk_percent.is_zero());

    let mut team = shield_request();
    let mut teammate = team.actors[0].clone();
    teammate.slot = 1;
    teammate.character_id = "character:1019".into();
    teammate.weapon_id.clear();
    teammate.gear_procs.clear();
    teammate.set_id = None;
    team.actors.push(teammate);
    let mut teammate_shield = GameSimulatorState::new(&team);
    apply_shield(&mut teammate_shield, 1, 10);
    assert!(
        zone_modifiers(&team, &teammate_shield, 1)
            .atk_percent
            .is_zero()
    );
}

#[test]
fn snowman_scenario_without_shield_deals_unbuffed_damage() {
    let mut req = shield_request();
    req.actions.truncate(1);
    req.action_metadata
        .retain(|index, _| *index < req.actions.len());
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(
        report["hit_receipts"][0]["non_crit_damage"],
        json!("1000/1"),
        "1000 static ATK with no shield stays 1000: the bonus is not silently applied"
    );
}

#[test]
fn snowman_duplicate_procs_rejected() {
    let mut req = shield_request();
    let extra = req.actors[0].gear_procs[0].clone();
    req.actors[0].gear_procs.push(extra);
    assert!(
        run_game_scenario(&req, &ExecutionLimits::default())
            .unwrap_err()
            .contains("duplicates")
    );
}

#[test]
fn snowman_shielded_scenario_adds_base_atk_scaled_bonus() {
    use super::adler_actions::{def_action, make_action, request};
    let mut req = request();
    let actor = &mut req.actors[0];
    actor.weapon_id = "weapon:fork_snowman".into();
    actor.weapon_tier = 1;
    actor.gear_procs = vec![t1_proc()];
    actor.static_stats.base_atk = 1000;
    actor.static_stats.atk = Ratio {
        numerator: 2000,
        denominator: 1,
    };
    actor.static_stats.base_def = Some(1000);
    actor.static_stats.def = 1000;
    actor.static_stats.crit_rate = Ratio::zero();
    req.targets[0].defense = 0;
    req.targets[0].resistance = Ratio::zero();
    req.relative_duration_us = 1_000_000;
    req.actions = vec![
        def_action(
            0,
            "GA_Adler_Skill",
            "GE_Player_Adler_Skill2_Damage",
            Ratio {
                numerator: 1,
                denominator: 5,
            },
        ),
        make_action(
            1_000,
            0,
            "GA_Adler_Melee",
            "GE_Player_Adler_Melee1_Damage",
            0,
            0,
        ),
    ];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    // The scenario keeps pending skill events past the horizon; only the
    // melee hit receipt matters here.
    let melee = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|hit| hit["ability_id"] == "GA_Adler_Melee")
        .unwrap();
    // Hand-derived: live ATK = 2000 + 1000 * 9/50 = 2180, and the frozen
    // Melee1 coefficient at talent level 1 is 201/1000, so the shielded hit is
    // 2180 * 201/1000 = 21909/50. Multiplying the compiled ATK instead would
    // give 2000 * 59/50 * 201/1000 = 11859/25.
    assert_eq!(melee["non_crit_damage"], json!("21909/50"));
    assert_ne!(melee["non_crit_damage"], json!("11859/25"));
}

#[test]
fn snowman_bonus_dead_at_exact_expiry() {
    let req = shield_request();
    let mut state = GameSimulatorState::new(&req);
    apply_shield(&mut state, 0, 10_000_000);
    assert_eq!(
        zone_modifiers(&req, &state, 9_999_999).atk_percent,
        ExactFraction::new(9, 50).unwrap(),
        "bonus live just before the shield deadline"
    );
    // Half-open interval: the shield no longer counts the instant the clock
    // reaches its expiry.
    assert!(
        zone_modifiers(&req, &state, 10_000_000)
            .atk_percent
            .is_zero(),
        "time_us == expires_at_us grants no bonus"
    );
}

#[test]
fn snowman_bonus_removed_when_incoming_damage_depletes_shield() {
    let req = shield_request();
    let mut state = GameSimulatorState::new(&req);
    apply_shield(&mut state, 0, 30_000_000);
    assert_eq!(
        zone_modifiers(&req, &state, 1).atk_percent,
        ExactFraction::new(9, 50).unwrap()
    );
    let (receipt, _trace) =
        super::adler_shields::settle_incoming_event(&req, &mut state, 0, 0, 100);
    assert_eq!(
        receipt
            .absorbed_by_shields
            .iter()
            .map(|entry| entry.amount)
            .sum::<u64>(),
        100,
        "incoming damage exactly depletes the 100-capacity shield"
    );
    assert_eq!(receipt.actual_hp_loss, 0);
    assert!(
        state.actors[0]
            .shields
            .iter()
            .all(|shield| shield.remaining_hp == 0),
        "no positive shield remains after exact depletion"
    );
    assert!(
        zone_modifiers(&req, &state, 2).atk_percent.is_zero(),
        "a fully depleted shield removes the ATK bonus"
    );
}

#[test]
fn strict_recorder_accepts_snowman_hypothesis() {
    let req = research_request();
    let mut reached = BTreeMap::new();
    crate::runtime_api::record_hypothesis(
        &req,
        &mut reached,
        crate::weapons::snowman::HYPOTHESIS_ID,
        crate::weapons::snowman::HYPOTHESIS_ID.description(),
    )
    .unwrap();
    assert_eq!(
        reached[crate::weapons::snowman::HYPOTHESIS_ID.as_str()],
        crate::weapons::snowman::HYPOTHESIS_ID.description()
    );
}
