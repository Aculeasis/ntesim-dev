use super::*;
use crate::simulation::tests::support::formula_damage_total;
use serde_json::{Value, json};

pub(super) fn request(targets: usize) -> GameScenarioRequest {
    let mut req = super::daffodil_actions::request();
    let model = crate::characters::daffodil::clock::MODEL;
    req.model_assumptions
        .insert(model.into(), model.description().into());
    req.actors[0].initial_energy = 0;
    req.actors[0].talent_levels = Some([8; 3]);
    req.relative_duration_us = 40_000_000;
    req.actions.clear();
    for index in 1..targets {
        let mut target = req.targets[0].clone();
        target.target_index = index;
        req.targets.push(target);
    }
    for target in &mut req.targets {
        target.hp_current = 100_000_000;
        target.hp_max = 100_000_000;
    }
    req
}
pub(super) fn action(time: u64, ability: &str, packet: &str, cost: u64) -> GameActionCommand {
    super::daffodil_actions::make_action(time, 0, ability, packet, cost, 0)
}
pub(super) fn normal(time: u64) -> GameActionCommand {
    action(
        time,
        "GA_Daffodill_Melee",
        "GE_Player_Daffodill_Melee1_Damage",
        0,
    )
}
pub(super) fn skill(time: u64) -> GameActionCommand {
    action(
        time,
        "GA_Daffodill_Skill",
        "GE_Player_Daffodill_Skill1_Damage",
        0,
    )
}
pub(super) fn burst(time: u64) -> GameActionCommand {
    action(
        time,
        "GA_Daffodill_UltraSkill",
        "GE_Player_Daffodill_UltraSkill1_Damage",
        120,
    )
}
pub(super) fn chain(req: &mut GameScenarioRequest, mut time: u64) -> u64 {
    for duration in [1_003_012, 715_347, 1_665_359, 1_379_811, 1_238_135] {
        req.actions.push(normal(time));
        time += duration;
    }
    time
}
pub(super) fn run(req: &GameScenarioRequest) -> Value {
    let result = run_game_scenario(req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        result["execution_completed"], true,
        "{} {}",
        result["halt_diagnostic"], result["rejected_action"]
    );
    result
}
#[test]
fn daffodil_clock_two_real_chains_keep_exact_native_contacts_and_single_aoe_energy() {
    for targets in [1, 3] {
        let mut req = request(targets);
        assert_eq!(chain(&mut req, 0), 6_001_664);
        assert_eq!(chain(&mut req, 6_001_664), 12_003_328);
        let report = run(&req);
        assert_eq!(report["hits_settled"], 38 * targets);
        let hits: Vec<_> = report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|h| h["target_index"] == 0)
            .collect();
        let expected = [
            185_308, 1_127_632, 1_348_465, 1_904_485, 1_946_781, 2_073_716, 2_116_091, 2_287_601,
            2_626_073, 3_636_055, 3_873_857, 4_022_633, 5_013_682, 5_063_536, 5_188_473, 5_247_092,
            5_297_935, 5_360_781, 5_563_916,
        ];
        for (index, hit) in hits.iter().enumerate() {
            assert_eq!(
                hit["time_us"],
                expected[index % 19] + if index >= 19 { 6_001_664 } else { 0 }
            );
        }
        let actor = &report["final_actor_states"][0];
        assert_eq!(actor["energy"], 39);
        assert_eq!(actor["cycle"]["current"], 66);
        assert_eq!(actor["cycle"]["carry"], json!("49/125"));
        assert_eq!(actor["cycle"]["conserved"], true);
        assert_eq!(actor["charge_remainder"], json!("201/250"));
        let mut early = req.clone();
        early.actions.push(normal(12_003_327));
        let rejected = run_game_scenario(&early, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            rejected["rejected_action"]["reason"]["until_us"],
            12_003_328
        );
    }
}
#[test]
fn daffodil_skill_starts_cooldown_at_notify_and_delivers_distinct_late_requests() {
    let mut req = request(2);
    req.actions = vec![skill(0)];
    let report = run(&req);
    let hits: Vec<_> = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|h| h["target_index"] == 0)
        .map(|h| h["time_us"].as_u64().unwrap())
        .collect();
    assert_eq!(
        hits,
        [
            196_675, 331_903, 670_029, 749_890, 832_625, 916_519, 1_000_193, 1_105_589
        ]
    );
    req.actions.push(skill(16_182_289));
    let rejected = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        rejected["rejected_action"]["reason"]["until_us"],
        16_182_290
    );
    *req.actions.last_mut().unwrap() = skill(16_182_290);
    assert_eq!(run(&req)["hits_settled"], 32);
}
#[test]
fn daffodil_finale_applies_eyes_at_commit_but_hp_begins_at_the_real_late_notify() {
    let mut req = request(1);
    req.actors[0].initial_energy = 120;
    req.actions = vec![burst(0)];
    let report = run(&req);
    let hits = report["hit_receipts"].as_array().unwrap();
    assert_eq!(hits.len(), 14);
    assert_eq!(hits[0]["time_us"], 2_849_375);
    assert_eq!(hits[13]["time_us"], 4_657_557);
    assert_eq!(
        serde_json::json!(formula_damage_total(&report)),
        json!("37708/5")
    );
    req.absolute_limit_us = 2_849_374;
    let truncated = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(truncated["hits_settled"], 0);
    assert_eq!(truncated["execution_completed"], false);
    req.absolute_limit_us = 3_600_000_000;
    req.relative_duration_us = 6_000_000;
    req.actions.push(normal(5_088_977));
    let rejected = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(rejected["rejected_action"]["reason"]["until_us"], 5_088_978);
}

#[test]
fn daffodil_finale_jump_does_not_clear_its_distinct_switch_lock() {
    let mut req = super::daffodil_cancel::request(1);
    req.actors[0].initial_energy = 120;
    let mut partner = req.actors[0].clone();
    partner.slot = req.actors.len();
    partner.character_id = "character:1042".into();
    partner.initial_energy = 0;
    partner.awakening = Default::default();
    req.actors.push(partner);
    req.actions = vec![
        burst(0),
        GameActionCommand::PlayerInput {
            timestamp_us: 5_086_241,
            actor_index: 0,
            input: PlayerInput::Jump,
            critical_success: false,
        },
        GameActionCommand::SwitchActor {
            timestamp_us: 5_087_547,
            target_actor_index: 1,
            cooldown_us: 0,
        },
    ];
    let rejected = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(rejected["rejected_action"]["reason"]["until_us"], 5_087_548);
    *req.actions.last_mut().unwrap() = GameActionCommand::SwitchActor {
        timestamp_us: 5_087_548,
        target_actor_index: 1,
        cooldown_us: 0,
    };
    assert_eq!(run(&req)["final_buff_state"]["active_actor_index"], 1);
}
#[test]
fn daffodil_zero_damage_requests_do_not_pause_or_create_energy() {
    let mut req = request(1);
    req.targets[0].resistance = Ratio::one();
    let mut now = 0;
    for duration in [1_003_012, 705_447, 1_607_759, 1_379_811, 1_161_335] {
        req.actions.push(normal(now));
        now += duration;
    }
    assert_eq!(now, 5_857_364);
    req.actions.push(normal(now));
    let report = run(&req);
    assert_eq!(report["hits_settled"], 20);
    assert_eq!(report["final_actor_states"][0]["energy"], 0);
    assert!(report["final_actor_states"][0]["cycle"].is_null());
}

#[test]
fn daffodil_cycle_efficiency_overflow_and_first_living_recipient_preserve_charge_independently() {
    // Native chain:33.196Cycle,19.902Charge. Two chains at Cycle efficiency2
    // yield132.784:100 retained,32 overflow and.784 carry, with39.804Charge.
    for targets in [1, 3] {
        let mut req = request(targets);
        req.actors[0].cycle_gain_efficiency = Ratio {
            numerator: 2,
            denominator: 1,
        };
        if targets > 1 {
            req.targets[0].hp_current = 0;
        }
        chain(&mut req, 0);
        chain(&mut req, 6_001_664);
        let report = run(&req);
        let actor = &report["final_actor_states"][0];
        assert_eq!(actor["cycle"]["current"], 100);
        assert_eq!(actor["cycle"]["overflow"], 32);
        assert_eq!(actor["cycle"]["carry"], json!("98/125"));
        assert_eq!(actor["cycle"]["conserved"], true);
        assert_eq!(actor["energy"], 39);
        assert_eq!(actor["charge_remainder"], json!("201/250"));
    }
    let mut req = request(1);
    req.actors[0].initial_energy = 120;
    req.actions = vec![burst(0)];
    let report = run(&req);
    assert!(report["final_actor_states"][0]["cycle"].is_null());
    assert_eq!(report["final_actor_states"][0]["energy"], 0);
}
