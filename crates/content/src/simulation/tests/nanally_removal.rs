use super::nanally_actions::make_action;
use super::*;

fn request() -> GameScenarioRequest {
    let mut req = super::nanally_actions::request();
    req.relative_duration_us = 35_000_000;
    req.actors[0].initial_energy = 0;
    req.actors[0].initial_cycle = 0;
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    req.targets[0].hp_current = 100_000_000;
    req.targets[0].hp_max = 100_000_000;
    req.actions = [0, 2_429_917, 16_425_571, 18_855_488].map(skill).to_vec();
    req
}
fn skill(time: u64) -> GameActionCommand {
    make_action(
        time,
        0,
        "GA_Nanally_Skill",
        "GE_Player_Nanally_Skill1_Damage",
        0,
        16_000_000,
    )
}
fn run(req: &GameScenarioRequest) -> GameScenarioReport {
    run_game_scenario_report(req, &ExecutionLimits::default()).unwrap()
}

#[test]
fn nanally_removal_two_legal_cycles_preserve_cooldowns_energy_and_actual_admission_roles() {
    let req = request();
    let report = run(&req);
    assert!(
        report.success && report.execution_completed,
        "{:?}",
        report.halt_diagnostic
    );
    assert_eq!(report.hits_settled, 10);
    assert_eq!(report.final_actor_states[0].granted_energy, 12);
    assert_eq!(
        report.final_actor_states[0].charge_remainder,
        Some(ExactFraction::new(3, 5).unwrap())
    );
    assert_eq!(report.action_metadata_ledger.len(), 2);
    let mut roles_by_command: Vec<_> = report
        .action_metadata_ledger
        .iter()
        .flat_map(|entry| {
            let role = report.action_metadata(entry).unwrap().action.input_role;
            entry.command_indices.iter().map(move |&idx| (idx, role))
        })
        .collect();
    roles_by_command.sort_by_key(|(idx, _)| *idx);
    let roles: Vec<_> = roles_by_command.into_iter().map(|(_, role)| role).collect();
    use super::super::metadata::InputActionRole::{Other, RedirectSkill};
    assert_eq!(
        roles,
        [
            Some(RedirectSkill),
            Some(Other),
            Some(RedirectSkill),
            Some(Other)
        ]
    );
    assert_eq!(
        report
            .trace
            .iter()
            .filter(|event| event.event_kind == "utility_ability")
            .count(),
        2
    );
    let mut denied = req.clone();
    denied.actions.push(skill(20_000_000));
    let rejected = run(&denied);
    assert!(!rejected.execution_completed);
    assert!(
        rejected
            .halt_diagnostic
            .unwrap()
            .contains("on cooldown until 32851142")
    );
    assert_eq!(rejected.hits_settled, 10);
}

#[test]
fn nanally_removal_gate_is_half_open_and_failed_input_preserves_the_accepted_prefix() {
    let mut req = request();
    req.actions.truncate(2);
    req.actions[1] = skill(2_429_916);
    let rejected = run(&req);
    assert!(!rejected.execution_completed);
    assert_eq!(rejected.hits_settled, 5);
    assert_eq!(rejected.action_metadata_ledger.len(), 1);
    assert!(
        rejected
            .halt_diagnostic
            .unwrap()
            .contains("on cooldown until 2429917")
    );
    req.actions[1] = skill(2_429_917);
    assert!(run(&req).execution_completed);
}

#[test]
fn nanally_removal_source_queries_read_the_selected_gate_and_native_replay_preserves_it() {
    let source = include_str!("../../../../../presets/native/nanally-authority-control.nte");
    let result = PreparedSourceScenario::new(source, Default::default(), &Default::default())
        .unwrap()
        .run(42)
        .unwrap();
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}; {:?}",
        result.diagnostic,
        result.report.halt_diagnostic
    );
    assert_eq!(result.report.hits_settled, 10);
    assert_eq!(
        result
            .request
            .actions
            .iter()
            .filter(|action| matches!(action, GameActionCommand::InvokeAbility { .. }))
            .count(),
        4
    );
    super::assert_native_replay(&result);
}

#[test]
fn nanally_removal_preserves_a5_and_underboss_without_admitting_an_extra_cat_group() {
    let mut req = request();
    req.relative_duration_us = 10_000_000;
    req.actors[0].initial_energy = 100;
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([5]).unwrap();
    req.actions = vec![
        skill(0),
        make_action(
            1_200_000,
            0,
            "GA_Nanally_UltraSkill",
            "GE_Player_Nanally_UltraSkill1_Damage",
            100,
            15_000_000,
        ),
        make_action(
            5_000_000,
            0,
            "GA_Nanally_Melee",
            "GE_Player_Nanally_Melee1_Damage",
            0,
            0,
        ),
        skill(9_582_140),
        make_action(
            10_582_140,
            0,
            "GA_Nanally_Melee",
            "GE_Player_Nanally_Melee1_Damage",
            0,
            0,
        ),
    ];
    let report = run(&req);
    assert!(report.success && report.execution_completed);
    assert_eq!(report.hits_settled, 16); //5E+7Q+2Normals+2Cat; removal contributes none.
    let after = report
        .hit_receipts
        .iter()
        .find(|hit| {
            hit.time_us == 10_773_194 && hit.damage_effect_id == "GE_Player_Nanally_Melee1_Damage"
        })
        .unwrap();
    assert_eq!(after.non_crit_damage, ExactFraction::new(3774, 25).unwrap()); //148*(1+.02).
    let stacks: Vec<_> = report
        .trace
        .iter()
        .filter(|event| event.event_kind == "nanally_a5_atk_buff")
        .collect();
    assert_eq!(stacks.len(), 2);
    assert_eq!(stacks[1].data.as_ref().and_then(|d| d.stacks), Some(2));
}

#[test]
fn nanally_removal_field_exit_restores_the_original_cooldown_query_without_resetting_it() {
    let source = "options duration=20 crit=random;
        nanally char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
        target hp=100000000 def=1080 resist=0; active nanally;
        nanally skill; wait_until(1); mint swap; wait_until(3); nanally swap;
        if .nanally.authority.active || .nanally.skill.ready || .nanally.skill.cd+.time!=16.425571 {let assertion_failure=1/0;}
        wait_until(16.425571); nanally skill;";
    let result = PreparedSourceScenario::new(source, Default::default(), &Default::default())
        .unwrap()
        .run(42)
        .unwrap();
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}; {:?}",
        result.diagnostic,
        result.report.halt_diagnostic
    );
    assert_eq!(
        result
            .report
            .trace
            .iter()
            .filter(|event| event.event_kind == "utility_ability")
            .count(),
        0
    );
    super::assert_native_replay(&result);
}
