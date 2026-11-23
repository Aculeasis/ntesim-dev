use super::nanally_actions::make_action;
use super::*;
use crate::simulation::tests::support::formula_damage_total;
pub(super) const STARTS: [u64; 6] = [0, 611_861, 1_482_102, 2_693_850, 4_025_363, 5_337_783];
fn normal(time: u64) -> GameActionCommand {
    make_action(
        time,
        0,
        "GA_Nanally_Melee",
        "GE_Player_Nanally_Melee1_Damage",
        0,
        0,
    )
}
fn request() -> GameScenarioRequest {
    let mut req = super::nanally_actions::request();
    req.relative_duration_us = 6_500_000;
    req.actors[0].initial_energy = 0;
    req.actors[0].initial_cycle = 0;
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    req.stochastic_crit = None;
    req.targets[0].hp_current = 100_000_000;
    req.targets[0].hp_max = 100_000_000;
    req.targets[0].defense = 1080;
    req.actions = STARTS.map(normal).to_vec();
    let model = crate::characters::nanally::clock::MODEL;
    req.model_assumptions
        .insert(model.into(), model.description().into());
    req
}
fn run(req: &GameScenarioRequest) -> GameScenarioReport {
    run_game_scenario_report(req, &ExecutionLimits::default()).unwrap()
}

#[test]
fn nanally_clock_full_chain_projects_source_markers_hitstop_and_the_next_legal_input() {
    let req = request();
    let report = run(&req);
    assert!(
        report.success && report.execution_completed,
        "{:?}",
        report.halt_diagnostic
    );
    let times: Vec<_> = report.hit_receipts.iter().map(|hit| hit.time_us).collect();
    assert_eq!(
        times,
        [
            191054, 820884, 1715028, 1798917, 1990241, 3043328, 3093314, 3202582, 3324718, 3410975,
            3464352, 3527671, 4274725, 4886227, 5528837
        ]
    );
    // Six inputs: native first-five4.313 plus repeated stage1 .296;1000ATK, DEF1/2.
    assert_eq!(
        formula_damage_total(&report),
        ExactFraction::new(4609, 2).unwrap()
    );
    let mut early = req;
    early.actions[5] = normal(5_337_782);
    let rejected = run(&early);
    assert!(!rejected.execution_completed);
    assert!(
        rejected
            .halt_diagnostic
            .unwrap()
            .contains("locked until 5337783")
    );
}

#[test]
fn nanally_clock_aoe_does_not_multiply_hitstop_or_packet_energy() {
    let mut req = request();
    let single = run(&req);
    let mut target = req.targets[0].clone();
    target.target_index = 1;
    req.targets.push(target);
    let aoe = run(&req);
    assert!(aoe.success && aoe.execution_completed);
    assert_eq!(aoe.hits_settled, 30);
    assert_eq!(
        formula_damage_total(&aoe),
        ExactFraction::new(4609, 1).unwrap()
    );
    assert_eq!(
        aoe.final_actor_states[0].granted_energy,
        single.final_actor_states[0].granted_energy
    );
    assert_eq!(
        aoe.final_actor_states[0].charge_remainder,
        single.final_actor_states[0].charge_remainder
    );
    for target in 0..=1 {
        assert_eq!(
            aoe.hit_receipts
                .iter()
                .filter(|hit| hit.target_index == target)
                .map(|hit| hit.time_us)
                .collect::<Vec<_>>(),
            single
                .hit_receipts
                .iter()
                .map(|hit| hit.time_us)
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn nanally_clock_zero_hp_loss_does_not_invent_contact_pause() {
    let mut req = request();
    req.actions = [
        normal(0),
        normal(1_000_000),
        normal(2_000_000),
        normal(3_083_048),
    ]
    .to_vec();
    let hit = run(&req);
    assert!(!hit.execution_completed);
    assert!(
        hit.halt_diagnostic
            .unwrap()
            .contains("locked until 3211748")
    );
    req.targets[0].resistance = Ratio {
        numerator: 1_000_000_000,
        denominator: 1,
    };
    let miss = run(&req);
    assert!(miss.success && miss.execution_completed);
    assert!(miss.hit_receipts.iter().all(|hit| hit.applied_hp_loss == 0));
    assert_eq!(miss.final_actor_states[0].granted_energy, 0);
}

#[test]
fn nanally_clock_jump_cancels_only_the_unissued_normal4_tail_at_its_own_gate() {
    let mut req = request();
    req.actions.truncate(4);
    req.actions.push(GameActionCommand::PlayerInput {
        timestamp_us: 3_191_919,
        actor_index: 0,
        input: PlayerInput::Jump,
        critical_success: false,
    });
    let report = run(&req);
    assert!(
        report.success && report.execution_completed,
        "{:?}",
        report.halt_diagnostic
    );
    let tail: Vec<_> = report
        .hit_receipts
        .iter()
        .filter(|hit| hit.command_index == Some(3))
        .collect();
    assert_eq!(tail.len(), 2);
    assert_eq!(
        tail.iter().map(|hit| hit.time_us).collect::<Vec<_>>(),
        [3_043_328, 3_093_314]
    );
    let mut denied = req;
    denied.actions[4] = GameActionCommand::PlayerInput {
        timestamp_us: 3_191_918,
        actor_index: 0,
        input: PlayerInput::Jump,
        critical_success: false,
    };
    assert!(!run(&denied).execution_completed);
}

#[test]
fn nanally_clock_two_cycles_keep_energy_carry_and_replay_identity() {
    let mut req = request();
    req.relative_duration_us = 18_000_000;
    req.actions
        .extend(STARTS.map(|time| normal(time + 10_000_000)));
    let report = run(&req);
    assert!(report.success && report.execution_completed);
    assert_eq!(report.hits_settled, 30);
    assert_eq!(
        formula_damage_total(&report),
        ExactFraction::new(4609, 1).unwrap()
    );
    assert_eq!(report.final_actor_states[0].granted_energy, 35);
    assert_eq!(
        report.final_actor_states[0].charge_remainder,
        Some(ExactFraction::new(237, 500).unwrap())
    );
    assert_eq!(
        serde_json::to_value(run(&req)).unwrap(),
        serde_json::to_value(report).unwrap()
    );
}

#[test]
fn nanally_clock_interruption_before_contact_cancels_normal_and_unlaunched_cat() {
    let mut req = request();
    req.actors[0].initial_energy = 100;
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Nanally_UltraSkill",
            "GE_Player_Nanally_UltraSkill1_Damage",
            100,
            0,
        ),
        normal(3_600_000),
    ];
    req.interrupt_events.push(GameActorInterruptEvent {
        timestamp_us: 3_700_000,
        actor_index: 0,
        armor_damage: None,
    });
    let report = run(&req);
    assert!(report.success && report.execution_completed);
    assert_eq!(report.hits_settled, 7);
    assert!(
        report
            .hit_receipts
            .iter()
            .all(|hit| hit.ability_id == "GA_Nanally_UltraSkill")
    );
    assert_eq!(report.final_actor_states[0].granted_energy, 0);
}

#[test]
fn nanally_clock_aim_uses_one_recipient_and_can_cancel_before_its_marker() {
    let mut req = request();
    let mut target = req.targets[0].clone();
    target.target_index = 1;
    req.targets.push(target);
    let special = |time| GameActionCommand::PlayerInput {
        timestamp_us: time,
        actor_index: 0,
        input: PlayerInput::Special,
        critical_success: false,
    };
    req.actions = vec![special(0), normal(1_000_000), normal(1_133_047)];
    let report = run(&req);
    assert!(
        report.success && report.execution_completed,
        "{:?}",
        report.halt_diagnostic
    );
    assert_eq!(report.hits_settled, 2);
    assert_eq!(
        report
            .hit_receipts
            .iter()
            .map(|hit| hit.time_us)
            .collect::<Vec<_>>(),
        [1_016_958, 1_150_005]
    );
    assert!(report.hit_receipts.iter().all(|hit| hit.target_index == 0
        && hit.damage_effect_id == "GE_Player_Nanally_BranchPraojectile_Damage"));
    // Two positive single-recipient Aim packets grant .7 each, without a Cat group.
    assert_eq!(report.final_actor_states[0].granted_energy, 1);
    assert_eq!(
        report.final_actor_states[0].charge_remainder,
        Some(ExactFraction::new(2, 5).unwrap())
    );
    req.actions.truncate(2);
    req.actions.push(special(1_010_000));
    let cancelled = run(&req);
    assert!(cancelled.success && cancelled.execution_completed);
    assert_eq!(cancelled.hits_settled, 0);
    assert_eq!(cancelled.final_actor_states[0].granted_energy, 0);
}

#[test]
fn nanally_clock_aoe_grants_packet_energy_when_the_primary_dummy_is_already_dead() {
    let mut req = request();
    req.actions = vec![normal(0)];
    let mut target = req.targets[0].clone();
    target.target_index = 1;
    req.targets.push(target);
    req.targets[0].hp_current = 0;
    let report = run(&req);
    assert!(report.success && report.execution_completed);
    assert_eq!(report.hits_settled, 1);
    assert_eq!(report.hit_receipts[0].target_index, 1);
    assert_eq!(report.final_actor_states[0].granted_energy, 1);
    assert_eq!(
        report.final_actor_states[0].charge_remainder,
        Some(ExactFraction::new(69, 500).unwrap())
    );
}

#[test]
fn nanally_clock_source_two_chains_use_actual_recovery_and_roundtrip_native_replay() {
    let result = PreparedSourceScenario::new(
        include_str!("../../../../../presets/native/nanally-normal-control.nte"),
        Default::default(),
        &Default::default(),
    )
    .unwrap()
    .run(42)
    .unwrap();
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}",
        result.diagnostic
    );
    assert_eq!(result.report.hits_settled, 28);
    super::assert_native_replay(&result);
}
