use super::nanally_actions::make_action;
use super::*;
use crate::simulation::tests::support::formula_damage_total;

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
fn hold(duration_us: u64) -> crate::simulation::GameHoldInput {
    crate::simulation::GameHoldInput {
        duration_us,
        reserved_stocks: None,
        moving: false,
    }
}
fn request(duration_us: u64) -> GameScenarioRequest {
    let mut req = super::nanally_actions::request();
    req.actors[0].talent_levels = Some([8, 8, 8]);
    req.actors[0].initial_energy = 0;
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    req.stochastic_crit = None;
    req.targets[0].defense = 1080;
    req.targets[0].hp_current = 100_000_000;
    req.targets[0].hp_max = 100_000_000;
    req.relative_duration_us = 9_000_000;
    req.actions = vec![normal(0)];
    req.hold_inputs.insert(0, hold(duration_us));
    let model = crate::characters::nanally::clock::MODEL;
    req.model_assumptions
        .insert(model.into(), model.description().into());
    req
}
fn run(req: &GameScenarioRequest) -> GameScenarioReport {
    run_game_scenario_report(req, &Default::default()).unwrap()
}

#[test]
fn nanally_held_native_sections_keep_normal_prefix_exact_heavy_damage_time_and_charge() {
    let mut req = request(3_000_000);
    req.actions.push(normal(3_706_608));
    let report = run(&req);
    assert!(
        report.success && report.execution_completed,
        "{:?}",
        report.halt_diagnostic
    );
    let first: Vec<_> = report
        .hit_receipts
        .iter()
        .filter(|hit| hit.command_index == Some(0))
        .collect();
    assert_eq!(first.len(), 7);
    assert_eq!(
        first.iter().map(|hit| hit.time_us).collect::<Vec<_>>(),
        [
            191_054, 760_520, 893_587, 1_376_795, 1_638_344, 1_867_695, 2_621_019
        ]
    );
    let damage = first
        .iter()
        .try_fold(ExactFraction::new(0, 1).unwrap(), |sum, hit| {
            sum.add(hit.damage)
        })
        .unwrap();
    assert_eq!(damage, ExactFraction::new(6329, 2).unwrap()); // .507 prefix +5.822 heavy, DEF1/2.
    assert!(first.iter().all(|hit| {
        report
            .hit_metadata(hit)
            .unwrap()
            .packet
            .native_damage_source_category
            .as_deref()
            == Some("EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_A")
    }));
    assert_eq!(report.final_actor_states[0].granted_energy, 15); // 14.239 +nextNormal1.138.
    assert_eq!(
        report.final_actor_states[0].charge_remainder,
        Some(ExactFraction::new(377, 1000).unwrap())
    );
    req.actions[1] = normal(3_706_607);
    assert!(!run(&req).execution_completed);
}

#[test]
fn nanally_held_release_is_real_time_and_half_open_at_each_dilated_section_gate() {
    for (release, count) in [
        (0, 1),
        (300_001, 1),
        (300_002, 3),
        (1_147_502, 3),
        (1_147_503, 6),
        (2_326_503, 6),
        (2_326_504, 7),
    ] {
        let report = run(&request(release));
        assert!(
            report.success && report.execution_completed,
            "release{release}: {:?}",
            report.halt_diagnostic
        );
        assert_eq!(report.hits_settled, count, "release{release}");
    }
    let mut req = request(5_000_000);
    req.actions.push(normal(4_000_000));
    assert!(!run(&req).execution_completed); // Cannot discard requested remaining hold for free.
}

#[test]
fn nanally_held_fourth_normal_enters_fifth_before_heavy_and_cancels_its_unissued_second_packet() {
    let mut req = request(4_000_000);
    req.actions = super::nanally_clock::STARTS[..4]
        .iter()
        .copied()
        .map(normal)
        .collect();
    req.hold_inputs.clear();
    req.hold_inputs.insert(3, hold(4_000_000));
    let report = run(&req);
    assert!(
        report.success && report.execution_completed,
        "{:?}",
        report.halt_diagnostic
    );
    assert_eq!(report.hits_settled, 19);
    assert!(
        report
            .hit_receipts
            .iter()
            .any(|hit| hit.damage_effect_id == "GE_Player_Nanally_Melee9_Damage")
    );
    assert!(
        !report
            .hit_receipts
            .iter()
            .any(|hit| hit.damage_effect_id == "GE_Player_Nanally_Melee10_Damage")
    );
    let first_heavy = report
        .hit_receipts
        .iter()
        .find(|hit| hit.damage_effect_id == "GE_Player_Nanally_ExtraAtk1_Damage")
        .unwrap();
    assert_eq!(first_heavy.time_us, 4_413_771);
}

#[test]
fn nanally_held_second_and_third_prefixes_keep_distinct_position_gates_and_contact_lag() {
    for (count, heavy_at, hits, damage) in [(2, 1_422_381, 8, 3497), (3, 2_671_322, 11, 4870)] {
        let mut req = request(3_000_000);
        req.actions = super::nanally_clock::STARTS[..count]
            .iter()
            .copied()
            .map(normal)
            .collect();
        req.hold_inputs.clear();
        req.hold_inputs.insert(count - 1, hold(3_000_000));
        let report = run(&req);
        assert!(
            report.success && report.execution_completed,
            "{:?}",
            report.halt_diagnostic
        );
        assert_eq!(report.hits_settled, hits);
        assert_eq!(
            formula_damage_total(&report),
            ExactFraction::new(damage, 1).unwrap()
        );
        assert_eq!(
            report
                .hit_receipts
                .iter()
                .find(|hit| hit.damage_effect_id == "GE_Player_Nanally_ExtraAtk1_Damage")
                .unwrap()
                .time_us,
            heavy_at
        );
    }
}

#[test]
fn nanally_held_interruption_cancels_only_unissued_packets_and_future_sections() {
    let mut req = request(3_000_000);
    req.interrupt_events.push(GameActorInterruptEvent {
        timestamp_us: 800_000,
        actor_index: 0,
        armor_damage: None,
    });
    let report = run(&req);
    assert!(report.success && report.execution_completed);
    assert_eq!(report.hits_settled, 2); // Normal prefix and firstHeavy1 contact already settled.
    assert_eq!(report.final_actor_states[0].granted_energy, 1);
    assert_eq!(
        report.final_actor_states[0].charge_remainder,
        Some(ExactFraction::new(369, 1000).unwrap())
    );
}

#[test]
fn nanally_held_heavy_jump_gate_can_release_the_button_before_first_heavy_contact() {
    let mut req = request(3_000_000);
    req.actions.push(GameActionCommand::PlayerInput {
        timestamp_us: 716_835,
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
    assert_eq!(report.hits_settled, 1);
    assert_eq!(report.final_actor_states[0].granted_energy, 1);
    assert_eq!(
        report.final_actor_states[0].charge_remainder,
        Some(ExactFraction::new(69, 500).unwrap())
    );
    if let GameActionCommand::PlayerInput { timestamp_us, .. } = &mut req.actions[1] {
        *timestamp_us -= 1;
    }
    assert!(!run(&req).execution_completed);
}

#[test]
fn nanally_held_cannot_enter_from_aim_or_override_movement_stocks_or_packet_stage() {
    let mut req = request(3_000_000);
    req.actions.insert(
        0,
        GameActionCommand::PlayerInput {
            timestamp_us: 0,
            actor_index: 0,
            input: PlayerInput::Special,
            critical_success: false,
        },
    );
    req.actions[1] = normal(1_000_000);
    req.hold_inputs.clear();
    req.hold_inputs.insert(1, hold(3_000_000));
    let report = run(&req);
    assert!(!report.execution_completed);
    assert!(report.halt_diagnostic.unwrap().contains("leaving Aim"));
    req.hold_inputs.get_mut(&1).unwrap().duration_us = 0;
    let tap = run(&req);
    assert!(tap.success && tap.execution_completed);
    assert_eq!(
        tap.hit_receipts[0].damage_effect_id,
        "GE_Player_Nanally_BranchPraojectile_Damage"
    );
    let mut req = request(3_000_000);
    req.hold_inputs.get_mut(&0).unwrap().moving = true;
    assert!(
        run_game_scenario_report(&req, &Default::default())
            .unwrap_err()
            .contains("stationary")
    );
    req.hold_inputs.get_mut(&0).unwrap().moving = false;
    req.hold_inputs.get_mut(&0).unwrap().reserved_stocks = Some(1);
    assert!(
        run_game_scenario_report(&req, &Default::default())
            .unwrap_err()
            .contains("reserved_stocks")
    );
    req.hold_inputs.get_mut(&0).unwrap().reserved_stocks = None;
    if let GameActionCommand::InvokeAbility {
        damage_effect_id, ..
    } = &mut req.actions[0]
    {
        *damage_effect_id = "GE_Player_Nanally_ExtraAtk1_Damage".into();
    }
    assert!(
        run_game_scenario_report(&req, &Default::default())
            .unwrap_err()
            .contains("canonical")
    );
}

#[test]
fn nanally_held_cat_heavy_groups_use_burst_curve_and_share_one_callback_across_recipients() {
    let mut req = request(3_000_000);
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
        normal(4_000_000),
    ];
    req.hold_inputs.clear();
    req.hold_inputs.insert(1, hold(3_000_000));
    let report = run(&req);
    assert!(
        report.success && report.execution_completed,
        "{:?}",
        report.halt_diagnostic
    );
    assert_eq!(report.hits_settled, 22);
    let heavy: Vec<_> = report
        .hit_receipts
        .iter()
        .filter(|hit| {
            hit.damage_effect_id
                .starts_with("GE_Player_Nanally_Cat_Branch")
        })
        .collect();
    assert_eq!(heavy.len(), 7);
    let damage = heavy
        .iter()
        .try_fold(ExactFraction::new(0, 1).unwrap(), |sum, hit| {
            sum.add(hit.damage)
        })
        .unwrap();
    assert_eq!(damage, ExactFraction::new(2438, 1).unwrap());
    assert_eq!(report.final_actor_states[0].granted_energy, 14);
    let mut a4 = req.clone();
    a4.actors[0].awakening = ntesim_data::AwakeningSelection::selected([4]).unwrap();
    let enhanced = run(&a4);
    let cat_damage = enhanced
        .hit_receipts
        .iter()
        .filter(|hit| {
            hit.damage_effect_id
                .starts_with("GE_Player_Nanally_Cat_Branch")
        })
        .try_fold(ExactFraction::new(0, 1).unwrap(), |sum, hit| {
            sum.add(hit.damage)
        })
        .unwrap();
    assert_eq!(cat_damage, ExactFraction::new(4876, 1).unwrap());
    req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([1, 5]).unwrap();
    let mut target = req.targets[0].clone();
    target.target_index = 1;
    req.targets.push(target);
    let report = run(&req);
    assert!(report.success && report.execution_completed);
    assert_eq!(report.hits_settled, 43);
    assert_eq!(report.final_actor_states[0].granted_energy, 21);
    assert_eq!(
        report.final_actor_states[0].charge_remainder,
        Some(ExactFraction::new(739, 1000).unwrap())
    );
    assert_eq!(
        report
            .trace
            .iter()
            .filter(|event| event.event_kind == "nanally_a5_atk_buff")
            .count(),
        4
    );
}

#[test]
fn nanally_held_source_two_continuing_chains_keep_exact_duration_charge_and_replay() {
    let result = PreparedSourceScenario::new(
        include_str!("../../../../../presets/native/nanally-heavy-control.nte"),
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
    assert_eq!(result.report.hits_settled, 14);
    assert_eq!(result.request.actions[1].timestamp_us(), 3_706_608);
    assert_eq!(result.request.hold_inputs.len(), 2);
    assert_eq!(result.request.hold_inputs[&1].duration_us, 3_000_000);
    assert_eq!(result.report.final_actor_states[0].granted_energy, 28);
    assert_eq!(
        result.report.final_actor_states[0].charge_remainder,
        Some(ExactFraction::new(239, 500).unwrap())
    );
    super::assert_native_replay(&result);
}

#[test]
fn nanally_held_paid_riposte_prefix_retains_one_input_identity_through_heavy_sections() {
    let source = "options duration=8 crit=random stamina_capacity=60 initial_stamina=60
        stamina_use_efficiency=1 stamina_recovery_rate=0;
        nanally char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=0;
        target hp=100000000 def=1080 resist=0; active nanally;
        nanally dash[critical=true]; nanally attack[hold=4s];";
    let result = PreparedSourceScenario::new(source, Default::default(), &Default::default())
        .unwrap()
        .run(42)
        .unwrap();
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}",
        result.diagnostic
    );
    assert_eq!(result.request.actions.len(), 2);
    assert_eq!(result.report.hits_settled, 7);
    assert!(
        result
            .report
            .hit_receipts
            .iter()
            .all(|hit| hit.command_index == Some(1))
    );
    assert_eq!(
        result.report.hit_receipts[0].damage_effect_id,
        "GE_Player_Nanally_PerfectEvadeAttack_Damage"
    );
    assert_eq!(result.report.hit_receipts[6].time_us, 4_018_019);
    let value = serde_json::to_value(&result.report).unwrap();
    assert_eq!(
        value["combat_stamina"]["total_spent"],
        serde_json::json!("15/1")
    );
    super::assert_native_replay(&result);
}

#[test]
fn nanally_held_zero_loss_uses_nominal_section_deadlines_and_creates_no_charge() {
    let mut req = request(3_000_000);
    req.targets[0].resistance = Ratio {
        numerator: 1_000_000_000,
        denominator: 1,
    };
    let report = run(&req);
    assert!(report.success && report.execution_completed);
    assert_eq!(
        report
            .hit_receipts
            .iter()
            .map(|hit| hit.time_us)
            .collect::<Vec<_>>(),
        [
            191_054, 760_520, 814_387, 1_129_295, 1_340_844, 1_520_195, 2_174_519
        ]
    );
    assert_eq!(report.final_actor_states[0].granted_energy, 0);
}

#[test]
fn nanally_held_source_profiles_follow_actual_indices_after_automatic_support() {
    let source = "options duration=10 crit=random;
        nanally char lvl=80/80 awakening=0 talent=8,8,8 qte=8 start_energy=0;
        zero char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=0 start_cycle=100;
        target hp=100000000 def=1080 resist=0; active zero;
        nanally attack[hold=3s]:2;";
    let result = PreparedSourceScenario::new(source, Default::default(), &Default::default())
        .unwrap()
        .run(42)
        .unwrap();
    assert!(
        result.report.success && result.report.execution_completed,
        "{:?}",
        result.diagnostic
    );
    assert_eq!(
        result
            .request
            .hold_inputs
            .keys()
            .copied()
            .collect::<Vec<_>>(),
        [1, 2]
    );
    assert_eq!(result.request.actions[1].timestamp_us(), 1_308_275);
    assert_eq!(result.request.actions[2].timestamp_us(), 5_014_883);
    assert_eq!(
        result
            .report
            .hit_receipts
            .iter()
            .filter(|hit| hit.ability_id.starts_with("GA_Nanally_"))
            .count(),
        15
    );
    assert_eq!(result.report.final_actor_states[0].granted_energy, 35);
    assert_eq!(
        result.report.final_actor_states[0].charge_remainder,
        Some(ExactFraction::new(239, 500).unwrap())
    );
    super::assert_native_replay(&result);
}
