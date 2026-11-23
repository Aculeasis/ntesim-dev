use super::*;
use serde_json::json;

pub(super) fn adler_incantation_packet(
    request: &GameScenarioRequest,
) -> crate::simulation::metadata::PacketMetadata {
    crate::simulation::metadata::resolve_requested_action_packet(
        &request.content_id,
        ntesim_data::skill_levels::frozen::tables(),
        "character:1033",
        "GA_Adler_Melee",
        "GE_Player_Adler_Melee1_Damage",
        None,
        &request.model_assumptions,
    )
    .unwrap()
    .packet
}

#[test]
fn adler_r6_deactivates_after_a4_absorbs_the_break_overflow() {
    let mut req = crate::simulation::tests::adler_actions::request();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(6);
    req.actors[0].static_stats.base_def = Some(1000);
    req.actors[0].static_stats.def = 1000;
    let mut state = crate::simulation::state::GameSimulatorState::new(&req);
    super::adler_shields::settle_adler_callback(&req, &mut state, 0, 0, 1940);
    let packet = adler_incantation_packet(&req);
    assert!(crate::characters::adler::skill::r6_incantation_bonus_active(&req, &state, 0, &packet));
    let (receipt, _) = super::adler_shields::settle_incoming_event(&req, &mut state, 0, 0, 2828);
    assert_eq!(receipt.absorbed_by_effects[0].amount, 500);
    assert_eq!(receipt.actual_hp_loss, 0);
    assert!(
        !crate::characters::adler::skill::r6_incantation_bonus_active(&req, &state, 0, &packet)
    );
}

#[test]
fn selecting_only_adler_effect_six_does_not_activate_resonance_six() {
    let mut req = crate::simulation::tests::adler_actions::request();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([6]).unwrap();
    let mut state = crate::simulation::state::GameSimulatorState::new(&req);
    super::adler_shields::settle_adler_callback(&req, &mut state, 0, 0, 1800);
    let packet = adler_incantation_packet(&req);
    assert!(
        !crate::characters::adler::skill::r6_incantation_bonus_active(&req, &state, 0, &packet)
    );
}

#[test]
fn adler_r6_bonus_uses_one_additive_damage_bonus_zone_and_reports_hypothesis() {
    let mut baseline = crate::simulation::tests::adler_actions::request();
    baseline.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(6);
    baseline.actors[0].static_stats.base_def = Some(1000);
    baseline.actors[0].static_stats.def = 1000;
    baseline.actors[0].static_stats.damage_bonus = Ratio {
        numerator: 1,
        denominator: 2,
    };
    baseline.actors[0].static_stats.crit_rate = Ratio::zero();
    baseline.targets[0].defense = 0;
    baseline.targets[0].resistance = Ratio::zero();
    baseline.relative_duration_us = 2_000;
    baseline.actions = vec![crate::simulation::tests::adler_actions::make_action(
        0,
        0,
        "GA_Adler_Melee",
        "GE_Player_Adler_Melee1_Damage",
        0,
        0,
    )];
    let without_shield = run_game_scenario(&baseline, &ExecutionLimits::default()).unwrap();
    assert_eq!(without_shield["execution_completed"], true);
    assert_eq!(
        without_shield["hit_receipts"][0]["non_crit_damage"],
        json!("651/2")
    );
    assert!(
        without_shield["resolved_model_assumptions"]
            .get(crate::characters::adler::skill::R6_HYPOTHESIS_ID.as_str())
            .is_none()
    );

    let mut protected = baseline.clone();
    protected.actions = vec![
        crate::simulation::tests::adler_actions::def_action(
            0,
            "GA_Adler_Skill",
            "GE_Player_Adler_Skill2_Damage",
            Ratio {
                numerator: 1,
                denominator: 5,
            },
        ),
        crate::simulation::tests::adler_actions::make_action(
            1_000,
            0,
            "GA_Adler_Melee",
            "GE_Player_Adler_Melee1_Damage",
            0,
            0,
        ),
    ];
    let with_shield = run_game_scenario(&protected, &ExecutionLimits::default()).unwrap();
    assert_eq!(with_shield["execution_completed"], true);
    assert_eq!(with_shield["hits_settled"], 4);
    let baseline_damage = &without_shield["hit_receipts"][0]["non_crit_damage"];
    let protected_hit = with_shield["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|hit| hit["ability_id"] == "GA_Adler_Melee")
        .unwrap();
    let boosted_damage = &protected_hit["non_crit_damage"];
    assert_eq!(baseline_damage, &json!("651/2"));
    assert_eq!(boosted_damage, &json!("1736/5"));
    assert_eq!(
        support::receipt_assumption(
            &with_shield,
            protected_hit,
            crate::characters::adler::skill::R6_HYPOTHESIS_ID.as_str()
        ),
        Some(crate::characters::adler::skill::R6_HYPOTHESIS_ID.description())
    );

    let mut no_existing_bonus = protected;
    no_existing_bonus.actors[0].static_stats.damage_bonus = Ratio::zero();
    let no_bonus_report =
        run_game_scenario(&no_existing_bonus, &ExecutionLimits::default()).unwrap();
    let no_bonus_hit = no_bonus_report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|hit| hit["ability_id"] == "GA_Adler_Melee")
        .unwrap();
    assert_eq!(no_bonus_hit["non_crit_damage"], json!("2387/10"));
}

#[test]
fn adler_r6_protection_tracks_source_shield_lifecycle_and_field_independently() {
    let mut req = crate::simulation::tests::adler_actions::request();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(6);
    let mut recipient = req.actors[0].clone();
    recipient.slot = 1;
    recipient.character_id = "character:1019".into();
    recipient.weapon_id.clear();
    recipient.gear_procs.clear();
    recipient.set_id = None;
    req.actors.push(recipient);
    let packet = adler_incantation_packet(&req);
    let mut state = crate::simulation::state::GameSimulatorState::new(&req);
    super::adler_shields::settle_adler_callback(&req, &mut state, 0, 1, 1800);
    assert!(crate::characters::adler::skill::r6_incantation_bonus_active(&req, &state, 1, &packet));
    state.active_actor_index = 0;
    assert!(crate::characters::adler::skill::r6_incantation_bonus_active(&req, &state, 1, &packet));
    state.actors[1].shields[0].remaining_hp = 0;
    assert!(
        !crate::characters::adler::skill::r6_incantation_bonus_active(&req, &state, 1, &packet)
    );
    state.actors[1].shields[0].remaining_hp = 1;
    assert!(
        crate::characters::adler::skill::r6_incantation_bonus_active(
            &req, &state, 11_999_999, &packet
        )
    );
    assert!(
        !crate::characters::adler::skill::r6_incantation_bonus_active(
            &req, &state, 12_000_000, &packet
        )
    );
    state.actors[1].hp = 0;
    assert!(
        !crate::characters::adler::skill::r6_incantation_bonus_active(&req, &state, 1, &packet)
    );
    state.actors[1].hp = 1000;
    state.actors[0].hp = 0;
    assert!(
        !crate::characters::adler::skill::r6_incantation_bonus_active(&req, &state, 1, &packet)
    );
    state.actors[0].hp = 1000;
    let mut other_element = packet.clone();
    other_element.element = None;
    assert!(
        !crate::characters::adler::skill::r6_incantation_bonus_active(
            &req,
            &state,
            1,
            &other_element
        )
    );
}

#[test]
fn adler_r6_unprotected_packets_do_not_report_or_apply_bonus() {
    let mut req = crate::simulation::tests::adler_actions::request();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(6);
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    req.targets[0].defense = 0;
    req.targets[0].resistance = Ratio::zero();
    req.relative_duration_us = 2_000;
    req.actions = vec![crate::simulation::tests::adler_actions::make_action(
        0,
        0,
        "GA_Adler_Melee",
        "GE_Player_Adler_Melee1_Damage",
        0,
        0,
    )];
    let baseline = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(baseline["execution_completed"], true);
    assert_eq!(baseline["hits_settled"], 1);
    assert!(
        baseline["resolved_model_assumptions"]
            .get(crate::characters::adler::skill::R6_HYPOTHESIS_ID.as_str())
            .is_none()
    );
}

#[test]
fn adler_r6_hypothesis_validation_and_recording() {
    let mut req = crate::simulation::tests::adler_actions::request();
    req.model_assumptions.insert(
        crate::characters::adler::skill::BLESSING_HYPOTHESIS_ID.into(),
        crate::characters::adler::skill::BLESSING_HYPOTHESIS_ID
            .description()
            .into(),
    );
    req.model_assumptions.insert(
        crate::characters::adler::skill::R6_HYPOTHESIS_ID.into(),
        crate::characters::adler::skill::R6_HYPOTHESIS_ID
            .description()
            .into(),
    );
    assert!(crate::simulation::validation::validate_model_assumptions(&req).is_ok());
    let mut execution_assumptions = std::collections::BTreeMap::new();
    crate::runtime_api::record_hypothesis(
        &req,
        &mut execution_assumptions,
        crate::characters::adler::skill::R6_HYPOTHESIS_ID,
        crate::characters::adler::skill::R6_HYPOTHESIS_ID.description(),
    )
    .unwrap();
    assert_eq!(
        execution_assumptions[crate::characters::adler::skill::R6_HYPOTHESIS_ID.as_str()],
        crate::characters::adler::skill::R6_HYPOTHESIS_ID.description()
    );
}

#[test]
fn adler_r6_applies_to_team_sakiri_packets_after_switch_and_off_field_delivery() {
    let run = |awakening: u32, off_field: bool| {
        let mut req = crate::simulation::tests::adler_actions::request();
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.actors[0].static_stats.base_def = Some(1000);
        req.actors[0].static_stats.def = 1000;
        req.actors[0].static_stats.damage_bonus = Ratio {
            numerator: 1,
            denominator: 2,
        };
        req.actors[0].static_stats.crit_rate = Ratio::zero();
        let mut sakiri = req.actors[0].clone();
        sakiri.slot = 1;
        sakiri.character_id = "character:1003".into();
        sakiri.awakening = ntesim_data::AwakeningSelection::prefix(0);
        sakiri.weapon_id.clear();
        sakiri.gear_procs.clear();
        sakiri.set_id = None;
        req.actors.push(sakiri);
        req.targets[0].hp_current = 100_000;
        req.targets[0].hp_max = 100_000;
        req.targets[0].defense = 0;
        req.targets[0].resistance = Ratio::zero();
        req.relative_duration_us = 702_500;
        req.actions = vec![
            crate::simulation::tests::adler_actions::def_action(
                0,
                "GA_Adler_Skill",
                "GE_Player_Adler_Skill2_Damage",
                Ratio {
                    numerator: 1,
                    denominator: 5,
                },
            ),
            GameActionCommand::SwitchActor {
                timestamp_us: 1_000,
                target_actor_index: 1,
                cooldown_us: 0,
            },
            GameActionCommand::InvokeAbility {
                timestamp_us: 702_000,
                actor_index: 1,
                ability_id: "GA_Sagiri_Melee".into(),
                damage_effect_id: "GE_Player_Sagiri_Melee1_Damage".into(),
                array_index: 0,
                atk_rate: Ratio::one(),
                scaling_stat: crate::simulation::NativeScalingStat::Atk,
                energy_cost: 0,
                recovery_us: 0,
                cooldown_us: 0,
                hit_delay_us: 100,
            },
        ];
        if off_field {
            req.actions.push(GameActionCommand::SwitchActor {
                timestamp_us: 702_050,
                target_actor_index: 0,
                cooldown_us: 0,
            });
        }
        run_game_scenario(&req, &ExecutionLimits::default()).unwrap()
    };

    for off_field in [false, true] {
        let baseline = run(0, off_field);
        let protected = run(6, off_field);
        assert_eq!(baseline["execution_completed"], true);
        assert_eq!(protected["execution_completed"], true);
        let baseline_hit = baseline["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .find(|hit| hit["actor_index"] == 1 && hit["ability_id"] == "GA_Sagiri_Melee")
            .unwrap();
        let protected_hit = protected["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .find(|hit| hit["actor_index"] == 1 && hit["ability_id"] == "GA_Sagiri_Melee")
            .unwrap();
        let protected_packet = &support::hit_metadata(&protected, protected_hit)["packet"];
        assert_eq!(
            protected_packet["element"], "incantation",
            "Sakiri packet metadata: {}",
            protected_packet
        );
        assert_eq!(protected_hit["actor_index"], baseline_hit["actor_index"]);
        if off_field {
            assert_eq!(protected_hit["time_us"], 702_100);
            assert_eq!(protected_hit["actor_index"], 1);
        }
        let baseline_damage: crate::simulation::ExactFraction =
            serde_json::from_value(baseline_hit["non_crit_damage"].clone()).unwrap();
        let protected_damage: crate::simulation::ExactFraction =
            serde_json::from_value(protected_hit["non_crit_damage"].clone()).unwrap();
        assert_eq!(
            protected_damage.numerator * 15 * baseline_damage.denominator,
            baseline_damage.numerator * 16 * protected_damage.denominator
        );
        assert_eq!(
            support::receipt_assumption(
                &protected,
                protected_hit,
                crate::characters::adler::skill::R6_HYPOTHESIS_ID.as_str()
            ),
            Some(crate::characters::adler::skill::R6_HYPOTHESIS_ID.description())
        );
    }
}

#[test]
fn adler_r6_leaves_protected_nature_packets_unchanged() {
    let run = |awakening: u32| {
        let mut req = crate::simulation::tests::adler_actions::request();
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.actors[0].static_stats.base_def = Some(1000);
        req.actors[0].static_stats.def = 1000;
        req.actors[0].static_stats.crit_rate = Ratio::zero();
        let mut mint = req.actors[0].clone();
        mint.slot = 1;
        mint.character_id = "character:1019".into();
        mint.awakening = ntesim_data::AwakeningSelection::prefix(0);
        mint.weapon_id.clear();
        mint.gear_procs.clear();
        mint.set_id = None;
        req.actors.push(mint);
        req.targets[0].hp_current = 100_000;
        req.targets[0].hp_max = 100_000;
        req.targets[0].defense = 0;
        req.targets[0].resistance = Ratio::zero();
        req.relative_duration_us = 2_500;
        req.actions = vec![
            crate::simulation::tests::adler_actions::def_action(
                0,
                "GA_Adler_Skill",
                "GE_Player_Adler_Skill2_Damage",
                Ratio {
                    numerator: 1,
                    denominator: 5,
                },
            ),
            GameActionCommand::SwitchActor {
                timestamp_us: 1_000,
                target_actor_index: 1,
                cooldown_us: 0,
            },
            GameActionCommand::InvokeAbility {
                timestamp_us: 2_000,
                actor_index: 1,
                ability_id: "GA_Mint019_Skill".into(),
                damage_effect_id: "GE_Player_Mint_Skill1_Damage_New".into(),
                array_index: 0,
                atk_rate: Ratio::one(),
                scaling_stat: crate::simulation::NativeScalingStat::Atk,
                energy_cost: 0,
                recovery_us: 0,
                cooldown_us: 0,
                hit_delay_us: 100,
            },
        ];
        run_game_scenario(&req, &ExecutionLimits::default()).unwrap()
    };
    let baseline = run(0);
    let protected = run(6);
    assert_eq!(baseline["execution_completed"], true);
    assert_eq!(protected["execution_completed"], true);
    let baseline_hit = baseline["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|hit| hit["actor_index"] == 1)
        .unwrap();
    let protected_hit = protected["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|hit| hit["actor_index"] == 1)
        .unwrap();
    assert_eq!(
        support::hit_metadata(&protected, protected_hit)["packet"]["element"],
        "nature"
    );
    assert_eq!(
        protected_hit["non_crit_damage"],
        baseline_hit["non_crit_damage"]
    );
    assert!(!support::has_assumption(
        protected_hit,
        crate::characters::adler::skill::R6_HYPOTHESIS_ID.as_str()
    ));
}
