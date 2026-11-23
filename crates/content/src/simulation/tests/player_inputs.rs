use super::*;

fn request(character: &str) -> GameScenarioRequest {
    let mut request = sample_s1_request();
    let actor = &mut request.actors[0];
    actor.character_id = character.into();
    actor.weapon_id.clear();
    actor.gear_procs.clear();
    actor.set_id = None;
    actor.awakening = ntesim_data::AwakeningSelection::prefix(0);
    actor.talent_levels = Some([1; 3]);
    actor.character_max_level = Some(80);
    actor.character_level = 80;
    actor.static_stats.base_atk = 1000;
    actor.static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    actor.static_stats.elemental_bonus = Ratio::zero();
    actor.static_stats.crit_rate = Ratio::zero();
    request.targets[0].defense = 0;
    request.targets[0].resistance = Ratio::zero();
    request.relative_duration_us = 30_000_000;
    request.actions.clear();
    request
}

fn input(time: u64, input: PlayerInput) -> GameActionCommand {
    GameActionCommand::PlayerInput {
        timestamp_us: time,
        actor_index: 0,
        input,
        critical_success: false,
    }
}

fn ability(time: u64, name: &str, packet: &str) -> GameActionCommand {
    GameActionCommand::InvokeAbility {
        timestamp_us: time,
        actor_index: 0,
        ability_id: name.into(),
        array_index: 0,
        damage_effect_id: packet.into(),
        atk_rate: Ratio::one(),
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost: 0,
        recovery_us: 0,
        cooldown_us: 0,
        hit_delay_us: 0,
    }
}

#[test]
fn player_inputs_lacrimosa_both_forms_switch_damage_and_truck() {
    let mut req = request("character:1004");
    req.actions.push(input(0, PlayerInput::Special));
    for index in 1..=5 {
        req.actions.push(ability(
            index * 800_000,
            "GA_Lacrimosa_Melee",
            "GE_Player_Lacrimosa_Melee1_Damage",
        ));
    }
    req.actions.push(ability(
        4_800_000,
        "GA_Lacrimosa_UltraSkill",
        "GE_Player_Lacrimosa_UltraSkill1_Damage",
    ));
    req.actions.push(input(5_600_000, PlayerInput::Special));
    for index in 0..5 {
        req.actions.push(ability(
            6_400_000 + index * 800_000,
            "GA_Lacrimosa_Melee",
            "GE_Player_Lacrimosa_Melee1_Damage",
        ));
    }
    let report = run_game_scenario_report(&req, &ExecutionLimits::default()).unwrap();
    assert!(report.execution_completed);
    assert_eq!(report.hit_receipts.len(), 38);
    assert_eq!(
        report.total_damage_exact,
        ExactFraction::new(22_297, 1).unwrap()
    );
    assert_eq!(report.final_actor_states[0].attack_mode, "ranged");
    assert_eq!(
        report.hit_receipts[0].damage_effect_id,
        "GE_Player_Lacrimosa_SwitchModB_Damage"
    );
    assert_eq!(
        report.hit_receipts[1].damage_effect_id,
        "GE_Player_Lacrimosa_B_Melee1_Damage"
    );
    assert_eq!(
        report.hit_receipts[16].damage_effect_id,
        "GE_Player_Lacrimosa_UltraSkillModB_Damage"
    );
    assert!(
        report
            .resolved_model_assumptions
            .contains_key(crate::characters::lacrimosa::skill::FORM_HYPOTHESIS_ID.as_str())
    );
}

#[test]
fn player_inputs_dash_resets_chain_without_resetting_lacrimosa_form() {
    let mut req = request("character:1004");
    req.actions = vec![
        input(0, PlayerInput::Special),
        ability(
            800_000,
            "GA_Lacrimosa_Melee",
            "GE_Player_Lacrimosa_Melee1_Damage",
        ),
        input(1_600_000, PlayerInput::Dash),
        ability(
            2_100_000,
            "GA_Lacrimosa_Melee",
            "GE_Player_Lacrimosa_Melee1_Damage",
        ),
    ];
    let report = run_game_scenario_report(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report.final_actor_states[0].attack_mode, "melee");
    assert_eq!(
        report.total_damage_exact,
        ExactFraction::new(2_538, 1).unwrap()
    );
    assert_eq!(report.hit_receipts.len(), 5);
    assert_eq!(
        report.hit_receipts[3].damage_effect_id,
        "GE_Player_Lacrimosa_B_Melee1_Damage"
    );
}

#[test]
fn player_inputs_aim_selects_native_shot_and_dash_exits_for_each_owner() {
    for (character, ability_id, primary, shot, coefficient) in [
        (
            "character:1033",
            "GA_Adler_Melee",
            "GE_Player_Adler_Melee1_Damage",
            "GE_Player_Adler_Aim_Damage",
            104,
        ),
        (
            "character:1010",
            "GA_Nanally_Melee",
            "GE_Player_Nanally_Melee1_Damage",
            "GE_Player_Nanally_BranchPraojectile_Damage",
            130,
        ),
        (
            "character:1055",
            "GA_Kuhara_Melee",
            "GE_Player_Kuhara_Melee1_Damage",
            "GE_Player_Kuhara_AimAttack_Damage",
            160,
        ),
    ] {
        let mut req = request(character);
        req.actions = vec![
            input(0, PlayerInput::Special),
            ability(800_000, ability_id, primary),
            input(1_600_000, PlayerInput::Dash),
            ability(2_100_000, ability_id, primary),
        ];
        let report = run_game_scenario_report(&req, &ExecutionLimits::default()).unwrap();
        assert!(report.execution_completed);
        assert_eq!(report.hit_receipts[0].damage_effect_id, shot);
        assert_eq!(
            report.hit_receipts[0].non_crit_damage,
            ExactFraction::new(coefficient, 1).unwrap()
        );
        assert_eq!(report.hit_receipts[1].damage_effect_id, primary);
        assert_eq!(report.final_actor_states[0].attack_mode, "normal");
    }
}

#[test]
fn player_inputs_failed_admission_preserves_aim_and_reports_input_index() {
    let mut req = request("character:1033");
    req.actions = vec![
        input(0, PlayerInput::Special),
        input(100_000, PlayerInput::Dash),
    ];
    let report = run_game_scenario_report(&req, &ExecutionLimits::default()).unwrap();
    assert!(!report.execution_completed);
    assert_eq!(report.final_actor_states[0].attack_mode, "aim");
    assert!(report.hit_receipts.is_empty());
    let rejected = report.rejected_action.unwrap();
    assert_eq!(rejected.command_index, 1);
    assert!(matches!(
        rejected.reason,
        GameAdmissionReason::Recovery { until_us: 800_000 }
    ));
}

#[test]
fn player_inputs_only_non_damage_inputs_keep_reached_assumptions() {
    let mut req = request("character:1033");
    req.actions = vec![
        input(0, PlayerInput::Special),
        input(800_000, PlayerInput::Special),
    ];
    let report = run_game_scenario_report(&req, &ExecutionLimits::default()).unwrap();
    assert!(report.hit_receipts.is_empty());
    assert_eq!(report.final_actor_states[0].attack_mode, "normal");
    assert!(
        report
            .resolved_model_assumptions
            .contains_key(super::super::inputs::TIMING_ID.as_str())
    );
    assert!(
        report
            .resolved_model_assumptions
            .contains_key(crate::characters::player_inputs::AIM_HYPOTHESIS_ID.as_str())
    );
    req.actors[0].character_id = "character:1019".into();
    assert!(
        validate_game_request(&req, &ExecutionLimits::default())
            .unwrap_err()
            .contains("Unsupported Special")
    );
}

#[test]
fn player_inputs_skill_and_burst_exit_aim_only_when_admitted() {
    for (name, packet) in [
        ("GA_Nanally_Skill", "GE_Player_Nanally_Skill1_Damage"),
        (
            "GA_Nanally_UltraSkill",
            "GE_Player_Nanally_UltraSkill1_Damage",
        ),
    ] {
        let mut req = request("character:1010");
        req.actions = vec![
            input(0, PlayerInput::Special),
            ability(800_000, name, packet),
            ability(
                5_000_000,
                "GA_Nanally_Melee",
                "GE_Player_Nanally_Melee1_Damage",
            ),
        ];
        let report = run_game_scenario_report(&req, &ExecutionLimits::default()).unwrap();
        assert!(report.execution_completed, "{:?}", report.halt_diagnostic);
        assert_eq!(report.final_actor_states[0].attack_mode, "normal");
        assert_eq!(
            report
                .hit_receipts
                .iter()
                .find(|hit| hit.command_index == Some(2))
                .unwrap()
                .damage_effect_id,
            "GE_Player_Nanally_Melee1_Damage"
        );
        if let GameActionCommand::InvokeAbility { energy_cost, .. } = &mut req.actions[1] {
            *energy_cost = 1000;
        }
        let report = run_game_scenario_report(&req, &ExecutionLimits::default()).unwrap();
        assert!(!report.execution_completed);
        assert_eq!(report.final_actor_states[0].attack_mode, "aim");
    }
}

#[test]
fn player_inputs_form_change_preserves_already_scheduled_packet() {
    let mut req = request("character:1004");
    let mut delayed = ability(0, "GA_Lacrimosa_Melee", "GE_Player_Lacrimosa_Melee1_Damage");
    if let GameActionCommand::InvokeAbility { hit_delay_us, .. } = &mut delayed {
        *hit_delay_us = 2_000_000;
    }
    req.actions = vec![
        delayed,
        input(800_000, PlayerInput::Special),
        ability(
            1_600_000,
            "GA_Lacrimosa_Melee",
            "GE_Player_Lacrimosa_Melee1_Damage",
        ),
    ];
    let report = run_game_scenario_report(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report.hit_receipts.last().unwrap().damage_effect_id,
        "GE_Player_Lacrimosa_Melee1_Damage"
    );
    assert_eq!(report.hit_receipts.last().unwrap().time_us, 2_000_000);
    assert_eq!(
        report.total_damage_exact,
        ExactFraction::new(2035, 1).unwrap()
    );
    assert_eq!(report.final_actor_states[0].attack_mode, "melee");
}

#[test]
fn player_inputs_special_uses_normal_talent_and_resonance() {
    for (level, awakening, damage) in [(1, 0, 790), (8, 0, 1354), (8, 3, 1462)] {
        let mut req = request("character:1004");
        req.actors[0].talent_levels = Some([level, 1, 1]);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.actions = vec![input(0, PlayerInput::Special)];
        let report = run_game_scenario_report(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report.total_damage_exact,
            ExactFraction::new(damage, 1).unwrap()
        );
    }
}

#[test]
fn player_input_report_receipt_defaults_critical_success_for_legacy_json() {
    let report =
        run_game_scenario_report(&request("character:1004"), &ExecutionLimits::default()).unwrap();
    let mut legacy = serde_json::to_value(report).unwrap();
    legacy["player_input_receipts"] = serde_json::json!([{
        "command_index": 0,
        "actor_index": 0,
        "time_us": 0,
        "input": "dash",
        "attack_mode_before": "normal",
        "attack_mode_after": "normal",
        "execution_model_assumption_ids": []
    }]);

    let legacy_report: GameScenarioReport = serde_json::from_value(legacy).unwrap();
    let receipt = &legacy_report.player_input_receipts[0];
    assert_eq!(receipt.input, PlayerInput::Dash);
    assert!(!receipt.critical_success);
}
