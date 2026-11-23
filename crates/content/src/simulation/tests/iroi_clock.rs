use super::*;
use crate::simulation::tests::support::formula_damage_total;
use serde_json::{Value, json};
mod burst;
mod held_actions;
mod skill;

fn request(dummies: usize) -> GameScenarioRequest {
    let mut req = crate::dsl::compiler::compile_scenario(
        "options duration=12 crit=fixed; iroi char lvl=80/80 awakening=0 talent=8,8,8 qte=8 start_energy=0;
        target hp=100000000 def=1080 resist=0; active iroi;",
        Default::default(),
    )
    .unwrap()
    .request;
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.atk = Ratio::one();
    req.actors[0].static_stats.atk.numerator = 1000;
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    req.actors[0].static_stats.elemental_bonus = Ratio::zero();
    req.targets = (0..dummies)
        .map(|index| {
            let mut target = req.targets[0].clone();
            target.target_index = index;
            target
        })
        .collect();
    let model = crate::characters::iroi::clock::MODEL;
    req.model_assumptions
        .insert(model.into(), model.description().into());
    req.action_admission_order = GameActionAdmissionOrder::AfterSettledEvents;
    req
}
fn action(time: u64, ability: &str) -> GameActionCommand {
    GameActionCommand::InvokeAbility {
        timestamp_us: time,
        actor_index: 0,
        ability_id: ability.into(),
        array_index: 7,
        damage_effect_id: crate::characters::iroi::primary_packet(ability)
            .unwrap()
            .into(),
        atk_rate: Ratio::one(),
        scaling_stat: Default::default(),
        energy_cost: 0,
        recovery_us: 0,
        cooldown_us: 0,
        hit_delay_us: 0,
    }
}
fn chain(req: &mut GameScenarioRequest, start: u64) {
    for offset in [0, 506352, 1590159, 2249850, 3239721] {
        req.actions.push(action(start + offset, "GA_Oneiroi_Melee"));
    }
}
fn run(req: &GameScenarioRequest) -> Value {
    let report = run_game_scenario(req, &Default::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{}",
        report["rejected_action"]
    );
    report
}

#[test]
fn iroi_clock_two_queued_chains_preserve_sixteen_causal_packets_and_moon_across_recipients() {
    for (dummies, hits, damage) in [(1, 32, 6064), (3, 80, 13954)] {
        let mut req = request(dummies);
        chain(&mut req, 0);
        chain(&mut req, 4379897);
        let report = run(&req);
        assert_eq!(report["hits_settled"], hits);
        assert_eq!(
            serde_json::json!(formula_damage_total(&report)),
            json!(format!("{}/{}", damage, 1))
        );
        assert_eq!(report["hit_receipts"][0]["time_us"], 316667);
        assert_eq!(report, run(&req));
        // Internal selected-profile fixture: read actual native Moon without
        // adding a report field or declaring the pending public source binding.
        let script = format!(
            "options duration=12 crit=fixed;
            iroi char lvl=80/80 awakening=0 talent=8,8,8 qte=8 start_energy=0;
            {} active iroi;
            iroi attack:5; if .iroi.resources.moon!=20.01 {{let assertion_failure=1/0;}}
            iroi attack:5; if .iroi.resources.moon!=40.02 {{let assertion_failure=1/0;}}",
            "target hp=100000000 def=1080 resist=0;".repeat(dummies)
        );
        let (parsed, mut config) = crate::dsl::inspection::prepare(&script).unwrap();
        let model = crate::characters::iroi::clock::MODEL;
        config
            .model_assumptions
            .insert(model.into(), model.description().into());
        let observed = PreparedSourceScenario::from_configuration(
            &script,
            config,
            Default::default(),
            &Default::default(),
            &parsed.options[0].loc,
        )
        .unwrap()
        .run(42)
        .unwrap();
        assert!(
            observed.report.execution_completed,
            "{:?}",
            observed.diagnostic
        );
        assert_eq!(
            observed.occurrences.last().unwrap().completed_us,
            Some(8759794)
        );
    }
}

#[test]
fn iroi_clock_launched_normal_three_keeps_all_eight_world_contacts_after_a_skill_cancel() {
    let mut req = request(1);
    chain(&mut req, 0);
    req.actions.truncate(3);
    req.actions.push(action(2330586, "GA_Oneiroi_Skill"));
    let report = run(&req);
    let times: Vec<_> = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["damage_effect_id"] == "GE_Player_Oneiroi_Melee3_Damage")
        .map(|hit| hit["time_us"].as_u64().unwrap())
        .collect();
    assert_eq!(
        times,
        (0..8)
            .map(|index| 2058938 + index * 100000)
            .collect::<Vec<_>>()
    );
    req.actions[3] = action(2330585, "GA_Oneiroi_Skill");
    let failed = run_game_scenario(&req, &Default::default()).unwrap();
    assert_eq!(failed["rejected_action"]["reason"]["until_us"], 2330586);
}

#[test]
fn iroi_clock_normal_two_retargets_each_projectile_without_becoming_aoe() {
    let mut req = request(2);
    req.targets[1].hp_current = 100;
    req.targets[1].hp_max = 100;
    req.actions = vec![
        action(0, "GA_Oneiroi_Melee"),
        action(506352, "GA_Oneiroi_Melee"),
    ];
    req.aim_targets.insert(1, 1);
    let report = run(&req);
    let shots: Vec<_> = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["damage_effect_id"] == "GE_Player_Oneiroi_Melee2_Damage")
        .map(|hit| hit["target_index"].as_u64().unwrap())
        .collect();
    assert_eq!(shots, [1, 0, 0]);
    assert_eq!(report["hits_settled"], 4);
    let source = "options duration=4 crit=fixed;
        iroi char lvl=80/80 awakening=0 talent=8,8,8 qte=8 start_energy=0;
        target hp=100000000 def=1080 resist=0;
        target hp=100 def=1080 resist=0; active iroi;
        iroi attack; iroi attack[target=t1];
        if .iroi.resources.moon!=5 {let assertion_failure=1/0;}";
    let (parsed, mut config) = crate::dsl::inspection::prepare(source).unwrap();
    let model = crate::characters::iroi::clock::MODEL;
    config
        .model_assumptions
        .insert(model.into(), model.description().into());
    config.actors[0].actor.static_stats = req.actors[0].static_stats.clone();
    config.actors[0].projection.static_stats = req.actors[0].static_stats.clone();
    let observed = PreparedSourceScenario::from_configuration(
        source,
        config,
        Default::default(),
        &Default::default(),
        &parsed.options[0].loc,
    )
    .unwrap()
    .run(42)
    .unwrap();
    assert!(
        observed.report.execution_completed,
        "{:?}",
        observed.diagnostic
    );
}

#[test]
fn iroi_clock_paid_dash_uses_movement_gate_and_releases_the_old_override_atomically() {
    let mut req = request(1);
    req.combat_stamina = Some(CombatStaminaProfile {
        capacity: Ratio {
            numerator: 100,
            denominator: 1,
        },
        initial: Ratio {
            numerator: 100,
            denominator: 1,
        },
        use_efficiency: Ratio::one(),
        recovery_rate: Ratio::zero(),
    });
    req.actions = vec![
        action(0, "GA_Oneiroi_Melee"),
        GameActionCommand::PlayerInput {
            timestamp_us: 865811,
            actor_index: 0,
            input: PlayerInput::Dash,
            critical_success: false,
        },
        action(1365810, "GA_Oneiroi_Skill"),
    ];
    let failed = run_game_scenario(&req, &Default::default()).unwrap();
    assert_eq!(failed["rejected_action"]["reason"]["until_us"], 1365811);
    assert_eq!(
        failed["combat_stamina"]["spends"].as_array().unwrap().len(),
        1
    );
    req.actions[2] = action(1365811, "GA_Oneiroi_Skill");
    assert_eq!(run(&req)["combat_stamina"]["total_spent"], json!("15/1"));
}

#[test]
fn iroi_clock_death_before_actor_birth_or_after_three_ticks_cancels_only_unissued_contacts() {
    for (delay, expected) in [(100000, 0), (700000, 3)] {
        let mut req = request(1);
        chain(&mut req, 0);
        req.actions.truncate(3);
        let model = crate::HypothesisId::CombatIncomingDamageFixtureV1;
        req.model_assumptions
            .insert(model.into(), model.description().into());
        req.incoming_damage_events.push(GameActorDamageEvent {
            timestamp_us: 1590159 + delay,
            actor_index: 0,
            delivered_damage: req.actors[0].initial_hp,
        });
        let report = run(&req);
        assert_eq!(
            report["hit_receipts"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|hit| hit["damage_effect_id"] == "GE_Player_Oneiroi_Melee3_Damage")
                .count(),
            expected
        );
    }
}
