use super::*;
use crate::simulation::tests::support::formula_damage_total;
use serde_json::{Value, json};
pub(super) fn request(targets: usize) -> GameScenarioRequest {
    let mut req = super::edgar_actions::request();
    req.actors[0].awakening = Default::default();
    req.actors[0].initial_hp = 1000;
    req.actors[0].initial_hp_max = 10000;
    req.actors[0].static_stats.hp = 10000;
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    req.relative_duration_us = 40_000_000;
    req.actions.clear();
    let id = crate::characters::edgar::clock::MODEL;
    req.model_assumptions
        .insert(id.into(), id.description().into());
    for index in 1..targets {
        let mut t = req.targets[0].clone();
        t.target_index = index;
        req.targets.push(t);
    }
    for target in &mut req.targets {
        target.hp_current = 100_000_000;
        target.hp_max = 100_000_000;
    }
    req
}
pub(super) fn action(time: u64, ability: &str, packet: &str, cost: u64) -> GameActionCommand {
    super::edgar_actions::make_action(time, 0, ability, packet, cost, 0)
}
pub(super) fn normal(time: u64) -> GameActionCommand {
    action(time, "GA_Edgar_Melee", "GE_Player_Edgar_Melee1_Damage", 0)
}
pub(super) fn skill(time: u64) -> GameActionCommand {
    action(time, "GA_Edgar_Skill", "GE_Player_Edgar_Skill_Damage", 0)
}
pub(super) fn burst(time: u64) -> GameActionCommand {
    action(
        time,
        "GA_Edgar_UltraSkill",
        "GE_Player_Edgar_UltraSkill_Damage",
        100,
    )
}
pub(super) fn run(req: &GameScenarioRequest) -> Value {
    let r = run_game_scenario(req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        r["execution_completed"], true,
        "{} {}",
        r["halt_diagnostic"], r["rejected_action"]
    );
    r
}
pub(super) fn hold(req: &mut GameScenarioRequest, command: usize, duration_us: u64) {
    req.hold_inputs.insert(
        command,
        GameHoldInput {
            duration_us,
            moving: false,
            reserved_stocks: None,
        },
    );
}
pub(super) fn heals<'a>(report: &'a Value, kind: &str) -> Vec<&'a Value> {
    report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["event_kind"] == kind)
        .collect()
}
#[test]
fn edgar_full_normal_chain_uses_causal_delayed_hitpause_and_exact_channels() {
    for targets in [1, 3] {
        let mut req = request(targets);
        req.actors[0].talent_levels = Some([8; 3]);
        req.actors[0].initial_energy = 0;
        req.actions = [0, 1_125_906, 2_105_958, 3_392_882, 4_699_077]
            .into_iter()
            .map(normal)
            .collect();
        let r = run(&req);
        assert_eq!(r["hits_settled"], 6 * targets);
        let own: Vec<_> = r["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|h| h["target_index"] == 0)
            .collect();
        assert_eq!(
            own.iter()
                .map(|h| h["time_us"].as_u64().unwrap())
                .collect::<Vec<_>>(),
            [183333, 1409239, 2306315, 2599992, 3960388, 5433484]
        );
        assert_eq!(
            serde_json::json!(formula_damage_total(&r)),
            json!(format!("{}/{}", 5565 * targets, 2))
        );
        let a = &r["final_actor_states"][0];
        assert_eq!(a["energy"], 20);
        assert_eq!(a["charge_remainder"], json!("3/10"));
        assert_eq!(a["cycle"]["current"], 33);
        assert_eq!(a["cycle"]["carry"], json!("801/1000"));
        req.actions.push(normal(6_064_026));
        let early = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(early["execution_completed"], false);
        assert_eq!(early["rejected_action"]["reason"]["until_us"], 6_064_027);
    }
}
#[test]
fn edgar_held_skill_emits_seven_single_target_pulses_and_independent_heals() {
    let mut req = request(3);
    req.actors[0].initial_energy = 0;
    req.actions = vec![skill(0)];
    hold(&mut req, 0, 4_900_000);
    let r = run(&req);
    assert_eq!(r["hits_settled"], 7);
    assert_eq!(
        r["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .map(|h| h["time_us"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        [821980, 1421980, 2021980, 2621980, 3221980, 3821980, 4421980]
    );
    assert!(
        r["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .all(|h| h["target_index"] == 0)
    );
    assert_eq!(r["final_actor_states"][0]["hp"], 4080);
    assert_eq!(
        r["final_actor_states"][0]["resources"]["edgar.keys_of_truth"]["current"],
        1
    );
    let h = heals(&r, "edgar_skill_heal");
    assert_eq!(h.len(), 7);
    assert_eq!(h[0]["time_us"], 821711);
    req.actions.push(skill(27_868_340));
    let early = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(early["execution_completed"], false);
    assert_eq!(early["rejected_action"]["reason"]["until_us"], 27_868_341);
    req.actions[1] = skill(27_868_341);
    run(&req);
}
#[test]
fn edgar_tap_short_loop_and_release_between_heal_and_damage_do_not_create_extra_pulses() {
    for (duration, hits, heals_count) in [(1, 1, 1), (800000, 0, 0), (821800, 0, 1), (900000, 1, 1)]
    {
        let mut req = request(1);
        req.actions = vec![skill(0)];
        hold(&mut req, 0, duration);
        let r = run(&req);
        assert_eq!(r["hits_settled"], hits);
        assert_eq!(heals(&r, "edgar_skill_heal").len(), heals_count);
    }
}
#[test]
fn edgar_skill_selector_uses_absolute_hp_and_exact_a3_boundary() {
    for (hp, after) in [(300, 806), (301, 741)] {
        let mut req = request(1);
        req.actors[0].awakening = ntesim_data::AwakeningSelection::selected([3]).unwrap();
        let mut ally = req.actors[0].clone();
        ally.slot = 1;
        ally.character_id = "character:1019".into();
        ally.awakening = Default::default();
        ally.initial_hp = hp;
        ally.initial_hp_max = 1000;
        ally.static_stats.hp = 1000;
        req.actors.push(ally);
        req.actions = vec![skill(0)];
        let r = run(&req);
        assert_eq!(r["final_actor_states"][1]["hp"], after);
        assert_eq!(r["final_actor_states"][0]["hp"], 1000);
    }
}
#[test]
fn edgar_burst_consumes_keys_at_spawn_and_field_ticks_through_the_endpoint() {
    let mut req = request(1);
    req.actions = vec![skill(0), burst(2_000_000)];
    let r = run(&req);
    assert_eq!(r["hits_settled"], 2);
    assert_eq!(r["hit_receipts"][1]["time_us"], 4_750_956);
    assert_eq!(
        r["final_actor_states"][0]["resources"]["edgar.keys_of_truth"]["current"],
        0
    );
    let h = heals(&r, "edgar_field_heal");
    assert_eq!(h.len(), 11);
    assert_eq!(h[0]["time_us"], 6_402_224);
    assert_eq!(h[10]["time_us"], 16_402_224);
    assert_eq!(r["final_actor_states"][0]["hp"], 6500);
    req.interrupt_events = vec![GameActorInterruptEvent {
        timestamp_us: 3_000_000,
        actor_index: 0,
        armor_damage: None,
    }];
    let cancelled = run(&req);
    assert_eq!(heals(&cancelled, "edgar_field_heal").len(), 0);
    assert_eq!(
        cancelled["final_actor_states"][0]["resources"]["edgar.keys_of_truth"]["current"],
        1
    );
    assert_eq!(cancelled["final_actor_states"][0]["spent_energy"], 100);
}
