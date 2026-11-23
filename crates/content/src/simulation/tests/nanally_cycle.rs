use super::nanally_actions::make_action;
use super::*;

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
fn chains(count: u64, targets: usize) -> GameScenarioRequest {
    let mut req = super::nanally_actions::request();
    req.actors[0].initial_cycle = 0;
    req.actors[0].initial_energy = 0;
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    req.stochastic_crit = None;
    req.targets[0].defense = 1080;
    req.targets[0].hp_current = 100_000_000;
    req.targets[0].hp_max = 100_000_000;
    for index in 1..targets {
        let mut target = req.targets[0].clone();
        target.target_index = index;
        req.targets.push(target);
    }
    req.relative_duration_us = count * 6_000_000;
    req.actions = (0..count)
        .flat_map(|cycle| {
            super::nanally_clock::STARTS[..5]
                .iter()
                .map(move |time| normal(cycle * 6_000_000 + time))
        })
        .collect();
    req
}
fn run(req: &GameScenarioRequest) -> GameScenarioReport {
    run_game_scenario_report(req, &Default::default()).unwrap()
}

#[test]
fn nanally_cycle_native_normal_packets_reach_cap_once_per_group_without_resetting_carry() {
    for targets in [1, 3] {
        let report = run(&chains(4, targets));
        assert!(report.success && report.execution_completed);
        let cycle = report.final_actor_states[0].cycle.as_ref().unwrap();
        // 1.9+2.49+.784+1.344+8.151+7*.446+4.948+4.962=27.701 per chain.
        assert_eq!(
            (cycle.current, cycle.granted, cycle.spent, cycle.overflow),
            (100, 110, 0, 10)
        );
        assert_eq!(cycle.carry, ExactFraction::new(201, 250).unwrap());
        assert!(cycle.conserved);
        assert_eq!(report.final_actor_states[0].granted_energy, 66);
    }
}

#[test]
fn nanally_cycle_gain_efficiency_dead_primary_and_zero_loss_stay_separate_from_charge() {
    let mut req = chains(1, 2);
    req.targets[0].hp_current = 0;
    req.actors[0].cycle_gain_efficiency = Ratio {
        numerator: 3,
        denominator: 2,
    };
    let report = run(&req);
    assert!(report.success && report.execution_completed);
    let cycle = report.final_actor_states[0].cycle.as_ref().unwrap();
    assert_eq!(cycle.current, 41);
    assert_eq!(cycle.carry, ExactFraction::new(1103, 2000).unwrap());
    assert_eq!(report.final_actor_states[0].granted_energy, 16);
    req.actors[0].cycle_gain_efficiency = Ratio::zero();
    let report = run(&req);
    assert_eq!(
        report.final_actor_states[0].cycle.as_ref().unwrap().current,
        0
    );
    assert_eq!(report.final_actor_states[0].granted_energy, 16);
    req.targets[1].resistance = Ratio {
        numerator: 1_000_000_000,
        denominator: 1,
    };
    assert!(run(&req).final_actor_states[0].cycle.is_none());
}

#[test]
fn nanally_cycle_heavy_and_landing_source_periods_use_native_unscaled_channels() {
    for (source, whole, numerator, denominator) in [
        (
            include_str!("../../../../../presets/native/nanally-heavy-control.nte"),
            47,
            201,
            500,
        ),
        (
            include_str!("../../../../../presets/native/nanally-plunge-control.nte"),
            8,
            2,
            5,
        ),
    ] {
        let result = PreparedSourceScenario::new(source, Default::default(), &Default::default())
            .unwrap()
            .run(42)
            .unwrap();
        assert!(result.report.success && result.report.execution_completed);
        let cycle = result.report.final_actor_states[0].cycle.as_ref().unwrap();
        assert_eq!(cycle.current, whole);
        assert_eq!(
            cycle.carry,
            ExactFraction::new(numerator, denominator).unwrap()
        );
        super::assert_native_replay(&result);
    }
}

#[test]
fn nanally_cycle_skill_aim_riposte_and_zero_channels_keep_their_own_native_amounts() {
    for (rotation, whole, numerator, denominator) in [
        ("nanally skill;", 10, 2, 5),
        ("nanally special; nanally attack:2;", 2, 1, 5),
        ("nanally dash[critical=true]; nanally attack;", 6, 9, 10),
    ] {
        let source = format!(
            "options duration=8 crit=random stamina_capacity=60 initial_stamina=60
            stamina_use_efficiency=1 stamina_recovery_rate=0;
            nanally char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=0;
            target hp=100000000 def=1080 resist=0; active nanally; {rotation}"
        );
        let result = PreparedSourceScenario::new(&source, Default::default(), &Default::default())
            .unwrap()
            .run(42)
            .unwrap();
        assert!(result.report.success && result.report.execution_completed);
        let cycle = result.report.final_actor_states[0].cycle.as_ref().unwrap();
        assert_eq!(cycle.current, whole);
        assert_eq!(
            cycle.carry,
            ExactFraction::new(numerator, denominator).unwrap()
        );
    }
    let mut req = chains(1, 1);
    req.actors[0].initial_energy = 100;
    req.actions = vec![make_action(
        0,
        0,
        "GA_Nanally_UltraSkill",
        "GE_Player_Nanally_UltraSkill1_Damage",
        100,
        0,
    )];
    assert!(run(&req).final_actor_states[0].cycle.is_none());
    req.actions.push(normal(4_000_000));
    let report = run(&req);
    assert_eq!(report.hits_settled, 9);
    let cycle = report.final_actor_states[0].cycle.as_ref().unwrap();
    assert_eq!(cycle.current, 1);
    assert_eq!(cycle.carry, ExactFraction::new(9, 10).unwrap()); // No extra Cat Cycle.
}

#[test]
fn nanally_cycle_interrupted_before_contact_grants_nothing() {
    let mut req = chains(1, 1);
    req.actions.truncate(1);
    let model = crate::characters::nanally::clock::MODEL;
    req.model_assumptions
        .insert(model.into(), model.description().into());
    req.interrupt_events.push(GameActorInterruptEvent {
        timestamp_us: 100_000,
        actor_index: 0,
        armor_damage: None,
    });
    let report = run(&req);
    assert!(report.success && report.execution_completed);
    assert!(report.final_actor_states[0].cycle.is_none());
}

#[test]
fn nanally_cycle_a3_and_all_heavy_cat_callbacks_do_not_create_cycle() {
    let source = "options duration=12 crit=random;
        nanally char lvl=80/80 awakening=3 talent=1,1,1 qte=1 start_energy=100;
        target hp=100000000 def=1080 resist=0; active nanally;
        nanally skill; nanally burst; nanally attack[hold=3s];";
    let result = PreparedSourceScenario::new(source, Default::default(), &Default::default())
        .unwrap()
        .run(42)
        .unwrap();
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}",
        result.diagnostic
    );
    assert!(
        result
            .report
            .hit_receipts
            .iter()
            .any(|hit| hit.damage_effect_id == "GE_Nanally010_Lv3_Damage")
    );
    assert_eq!(
        result
            .report
            .hit_receipts
            .iter()
            .filter(|hit| hit
                .damage_effect_id
                .starts_with("GE_Player_Nanally_Cat_Branch"))
            .count(),
        7
    );
    let cycle = result.report.final_actor_states[0].cycle.as_ref().unwrap();
    assert_eq!(cycle.current, 34); // FiveSkill10.4 +Normal1/heavy23.701; no callback grant.
    assert_eq!(cycle.carry, ExactFraction::new(101, 1000).unwrap());
}

#[test]
fn nanally_cycle_natural_qualified_spend_keeps_overflow_carry_and_new_generation() {
    let result = PreparedSourceScenario::new(
        include_str!("../../../../../presets/native/nanally-cycle-control.nte"),
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
    assert_eq!(result.report.hits_settled, 80); // 56 Normals +2 ZeroSupport +1 NanoSupport +1 Normal +20 pistils.
    let cycle = result.report.final_actor_states[0].cycle.as_ref().unwrap();
    assert_eq!(
        (
            cycle.initial,
            cycle.current,
            cycle.granted,
            cycle.spent,
            cycle.overflow
        ),
        (0, 2, 112, 100, 10)
    );
    assert_eq!(cycle.carry, ExactFraction::new(88, 125).unwrap());
    assert!(cycle.conserved);
    assert_eq!(
        result
            .report
            .hit_receipts
            .iter()
            .filter(|hit| hit.damage_effect_id == "ntesim.reaction.blossom")
            .count(),
        20
    );
    super::assert_native_replay(&result);
}
