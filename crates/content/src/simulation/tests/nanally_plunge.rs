use super::nanally_actions::make_action;
use super::*;
use crate::simulation::tests::support::formula_damage_total;

const PACKET: &str = "GE_Player_Nanally_AirAttack1_Damage";
fn request(level: u32, bonus: Ratio) -> GameScenarioRequest {
    let mut req = super::nanally_actions::request();
    req.actors[0].talent_levels = Some([level, 1, 1]);
    req.actors[0].initial_energy = 0;
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    req.stochastic_crit = None;
    req.targets[0].defense = 1080;
    req.targets[0].hp_current = 100_000_000;
    req.targets[0].hp_max = 100_000_000;
    req.relative_duration_us = 3_000_000;
    req.actions = vec![make_action(0, 0, "GA_Nanally_Melee", PACKET, 0, 0)];
    req.plunge_inputs.insert(
        0,
        crate::simulation::GamePlungeInput {
            landing_delay_us: 800_000,
            height_bonus: bonus,
        },
    );
    let model = crate::characters::nanally::clock::MODEL;
    req.model_assumptions
        .insert(model.into(), model.description().into());
    req
}
fn run(req: &GameScenarioRequest) -> GameScenarioReport {
    run_game_scenario_report(req, &Default::default()).unwrap()
}
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

#[test]
fn nanally_plunge_requires_its_own_positive_airborne_profile() {
    let mut req = request(1, Ratio::zero());
    req.plunge_inputs.clear();
    assert!(
        run_game_scenario_report(&req, &Default::default())
            .unwrap_err()
            .contains("explicit airborne landing/height profile")
    );
    req.plunge_inputs.insert(
        0,
        crate::simulation::GamePlungeInput {
            landing_delay_us: 0,
            height_bonus: Ratio::zero(),
        },
    );
    assert!(
        run_game_scenario_report(&req, &Default::default())
            .unwrap_err()
            .contains("positive landing time")
    );
    req.plunge_inputs.get_mut(&0).unwrap().landing_delay_us = 800_000;
    req.actions[0] = normal(0);
    assert!(
        run_game_scenario_report(&req, &Default::default())
            .unwrap_err()
            .contains("native landing packet")
    );
}

#[test]
fn nanally_plunge_native_landing_coefficients_scale_height_without_scaling_charge() {
    for (level, bonus, numerator, denominator) in [
        (1, Ratio::zero(), 250, 1),
        (
            8,
            Ratio {
                numerator: 1,
                denominator: 2,
            },
            2571,
            4,
        ),
        (9, Ratio::one(), 925, 1),
    ] {
        let req = request(level, bonus);
        let report = run(&req);
        assert!(
            report.success && report.execution_completed,
            "{:?}",
            report.halt_diagnostic
        );
        assert_eq!(report.hits_settled, 1);
        assert_eq!(report.hit_receipts[0].time_us, 850_170);
        assert_eq!(
            formula_damage_total(&report),
            ExactFraction::new(numerator, denominator).unwrap()
        );
        assert_eq!(report.final_actor_states[0].granted_energy, 2);
        assert_eq!(
            report.final_actor_states[0].charge_remainder,
            Some(ExactFraction::new(1, 2).unwrap())
        );
        assert_eq!(
            report
                .hit_metadata(&report.hit_receipts[0])
                .unwrap()
                .packet
                .native_damage_source_category
                .as_deref(),
            Some("EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_X")
        );
    }
}

#[test]
fn nanally_plunge_aoe_shares_contact_and_charge_and_uses_separate_cancel_gates() {
    for (input, gate) in [
        (None, 1_114_541),
        (Some(PlayerInput::Jump), 982_958),
        (Some(PlayerInput::Special), 1_608_561),
    ] {
        let mut req = request(1, Ratio::zero());
        let mut target = req.targets[0].clone();
        target.target_index = 1;
        req.targets.push(target);
        req.actions.push(input.map_or_else(
            || normal(gate),
            |input| GameActionCommand::PlayerInput {
                timestamp_us: gate,
                actor_index: 0,
                input,
                critical_success: false,
            },
        ));
        let report = run(&req);
        assert!(
            report.success && report.execution_completed,
            "{:?}",
            report.halt_diagnostic
        );
        let landings: Vec<_> = report
            .hit_receipts
            .iter()
            .filter(|hit| hit.damage_effect_id == PACKET)
            .collect();
        assert_eq!(landings.len(), 2);
        assert!(landings.iter().all(|hit| hit.time_us == 850_170));
        assert_eq!(landings[0].damage, landings[1].damage);
        assert_eq!(
            report.final_actor_states[0].granted_energy,
            if input.is_none() { 3 } else { 2 }
        );
        match &mut req.actions[1] {
            GameActionCommand::InvokeAbility { timestamp_us, .. }
            | GameActionCommand::PlayerInput { timestamp_us, .. } => *timestamp_us -= 1,
            _ => unreachable!(),
        }
        assert!(!run(&req).execution_completed);
    }
}

#[test]
fn nanally_plunge_before_landing_interruption_cancels_damage_cat_and_charge() {
    let mut req = request(1, Ratio::zero());
    req.actors[0].initial_energy = 100;
    req.actions.insert(
        0,
        make_action(
            0,
            0,
            "GA_Nanally_UltraSkill",
            "GE_Player_Nanally_UltraSkill1_Damage",
            100,
            0,
        ),
    );
    if let GameActionCommand::InvokeAbility { timestamp_us, .. } = &mut req.actions[1] {
        *timestamp_us = 4_000_000;
    }
    let input = req.plunge_inputs.remove(&0).unwrap();
    req.plunge_inputs.insert(1, input);
    req.interrupt_events.push(GameActorInterruptEvent {
        timestamp_us: 4_800_000,
        actor_index: 0,
        armor_damage: None,
    });
    req.relative_duration_us = 6_000_000;
    let cancelled = run(&req);
    assert!(cancelled.success && cancelled.execution_completed);
    assert_eq!(cancelled.hits_settled, 7);
    assert_eq!(cancelled.final_actor_states[0].granted_energy, 0);
    req.interrupt_events.clear();
    let reached = run(&req);
    assert!(reached.success && reached.execution_completed);
    assert_eq!(reached.hits_settled, 9);
    assert_eq!(reached.hit_receipts[7].time_us, 4_850_170);
    assert_eq!(reached.hit_receipts[8].time_us, 4_850_170);
    assert_eq!(
        reached.hit_receipts[8].damage_effect_id,
        "GE_Player_Nanally_Cat_AirAttack_Damage"
    );
    assert_eq!(reached.final_actor_states[0].granted_energy, 2);
}

#[test]
fn nanally_plunge_zero_loss_has_no_contact_pause_or_resources() {
    let mut req = request(1, Ratio::zero());
    req.targets[0].resistance = Ratio {
        numerator: 1_000_000_000,
        denominator: 1,
    };
    req.actions.push(normal(1_055_141));
    let report = run(&req);
    assert!(
        report.success && report.execution_completed,
        "{:?}",
        report.halt_diagnostic
    );
    assert_eq!(report.final_actor_states[0].granted_energy, 0);
}

#[test]
fn nanally_plunge_source_two_continuing_landings_preserve_real_flight_and_replay() {
    let result = PreparedSourceScenario::new(
        include_str!("../../../../../presets/native/nanally-plunge-control.nte"),
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
    assert_eq!(result.report.hits_settled, 2);
    assert_eq!(
        result
            .report
            .hit_receipts
            .iter()
            .map(|hit| hit.time_us)
            .collect::<Vec<_>>(),
        [850_170, 1_964_711]
    );
    assert_eq!(result.report.final_actor_states[0].granted_energy, 5);
    assert_eq!(result.report.final_actor_states[0].charge_remainder, None);
    super::assert_native_replay(&result);
}

#[test]
fn nanally_plunge_cat_uses_burst_talent_without_height_and_counts_one_group_for_aoe() {
    let mut req = request(8, Ratio::one());
    req.actors[0].initial_energy = 100;
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([1, 5]).unwrap();
    let mut target = req.targets[0].clone();
    target.target_index = 1;
    req.targets.push(target);
    req.actions.insert(
        0,
        make_action(
            0,
            0,
            "GA_Nanally_UltraSkill",
            "GE_Player_Nanally_UltraSkill1_Damage",
            100,
            0,
        ),
    );
    if let GameActionCommand::InvokeAbility { timestamp_us, .. } = &mut req.actions[1] {
        *timestamp_us = 4_000_000;
    }
    let input = req.plunge_inputs.remove(&0).unwrap();
    req.plunge_inputs.insert(1, input);
    req.relative_duration_us = 6_000_000;
    let report = run(&req);
    assert!(report.success && report.execution_completed);
    let cat: Vec<_> = report
        .hit_receipts
        .iter()
        .filter(|hit| hit.damage_effect_id == "GE_Player_Nanally_Cat_AirAttack_Damage")
        .collect();
    assert_eq!(cat.len(), 2);
    assert!(cat.iter().all(|hit| hit.time_us == 4_850_170
        && hit.non_crit_damage == ExactFraction::new(250, 1).unwrap()));
    assert_eq!(report.final_actor_states[0].granted_energy, 5); // 2.5 landing + 2.5 A1, once per group.
    assert_eq!(
        report
            .trace
            .iter()
            .filter(|event| event.event_kind == "nanally_a5_atk_buff")
            .count(),
        1
    );
    if let GameActionCommand::InvokeAbility { timestamp_us, .. } = &mut req.actions[1] {
        *timestamp_us = 8_734_438; // Landing marker equals the half-open9.415870 deadline.
    }
    req.relative_duration_us = 11_000_000;
    let expired = run(&req);
    assert!(expired.success && expired.execution_completed);
    assert!(
        !expired
            .hit_receipts
            .iter()
            .any(|hit| hit.damage_effect_id == "GE_Player_Nanally_Cat_AirAttack_Damage")
    );
    assert_eq!(expired.final_actor_states[0].granted_energy, 2);
}

#[test]
fn nanally_aim_positive_packet_distributes_native_charge_to_living_party_once() {
    let compiled = crate::dsl::compiler::compile_scenario(
        "options duration=3 crit=random; nanally char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=0;
         mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=0;
         target hp=100000000 def=1080 resist=0; active nanally;
         nanally special; nanally attack:2;", Default::default()).unwrap();
    let report = run(&compiled.request);
    assert!(report.success && report.execution_completed);
    assert_eq!(report.hits_settled, 2);
    assert_eq!(report.final_actor_states[0].granted_energy, 1);
    assert_eq!(
        report.final_actor_states[0].charge_remainder,
        Some(ExactFraction::new(2, 5).unwrap())
    );
    assert_eq!(report.final_actor_states[1].granted_energy, 0);
    assert_eq!(
        report.final_actor_states[1].charge_remainder,
        Some(ExactFraction::new(21, 25).unwrap())
    );
}
