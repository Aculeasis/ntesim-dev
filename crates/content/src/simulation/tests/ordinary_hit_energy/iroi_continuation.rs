use super::owner_rotations::{assert_hit_oracle, assert_one_burst_spend, push_packets};
use super::*;

fn iroi_timed_action(
    timestamp_us: u64,
    ability: &str,
    effect: &str,
    energy_cost: u64,
    recovery_us: u64,
    cooldown_us: u64,
    hit_delay_us: u64,
) -> GameActionCommand {
    let mut command = mint_timed_action(
        timestamp_us,
        ability,
        effect,
        energy_cost,
        recovery_us,
        cooldown_us,
    );
    if let GameActionCommand::InvokeAbility {
        hit_delay_us: delay,
        ..
    } = &mut command
    {
        *delay = hit_delay_us;
    }
    command
}

fn iroi_skill_burst_energy_request(case: &Case) -> GameScenarioRequest {
    let mut req = request(case, false);
    req.actors[0].talent_levels = Some([1; 3]);
    req.actors[0].static_stats.base_atk = 1000;
    req.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    req.actors[0].static_stats.damage_bonus = Ratio::zero();
    req.actors[0].static_stats.elemental_bonus = Ratio::zero();
    req.actors[0].static_stats.elemental_bonuses.clear();
    req.actors[0].static_stats.crit_rate = Ratio::zero();
    req.actors[0].initial_energy = 0;
    req.actors[0].energy_capacity = 100;
    req.actors[0].initial_hp = 10_000;
    req.actors[0].initial_hp_max = 20_000;
    req.actors[1].initial_hp = 10_000;
    req.actors[1].initial_hp_max = 20_000;
    req.actors[2].initial_hp = 20_000;
    req.actors[2].initial_hp_max = 20_000;
    req.targets[0].hp_max = 100_000_000;
    req.targets[0].hp_current = 100_000_000;
    req.targets[0].defense = 1080;
    req.targets[0].resistance = Ratio::zero();
    req.relative_duration_us = 32_000_000;

    let mut actions = Vec::new();
    for input in 0..25 {
        actions.push(iroi_timed_action(
            input * 1_000_000,
            case.melee.0,
            case.melee.1,
            0,
            1_000_000,
            0,
            100,
        ));
    }
    let (skill_ability, skill_effect, _) = case.skill.expect("Iroi Skill packet train");
    actions.push(iroi_timed_action(
        25_000_000,
        skill_ability,
        skill_effect,
        0,
        1_000_000,
        6_000_000,
        500_000,
    ));
    actions.push(iroi_timed_action(
        26_000_000,
        "GA_Oneiroi_UltraSkill",
        "GE_Player_Oneiroi_UltraSkill_Damage",
        100,
        1_000_000,
        20_000_000,
        100,
    ));
    for input in 0..5 {
        actions.push(iroi_timed_action(
            27_000_000 + input * 1_000_000,
            case.melee.0,
            case.melee.1,
            0,
            1_000_000,
            0,
            100,
        ));
    }
    req.actions = actions;
    req
}

fn assert_iroi_heals(
    trace: &[serde_json::Value],
    event_kind: &str,
    time_us: u64,
    atk_ratio: &str,
    gross: u64,
) {
    let heals = trace
        .iter()
        .filter(|event| event["event_kind"] == event_kind)
        .collect::<Vec<_>>();
    assert_eq!(heals.len(), 3);
    for (recipient, event) in heals.iter().enumerate() {
        assert_eq!(event["time_us"], time_us);
        assert_eq!(event["to_actor"], recipient);
        assert_eq!(event["data"]["exact_raw"], "1000/1");
        assert_eq!(event["data"]["atk_ratio"], atk_ratio);
        assert_eq!(event["data"]["gross"], gross);
        let applied = if recipient == 2 { 0 } else { gross };
        assert_eq!(event["data"]["applied"], applied);
        assert_eq!(event["data"]["overflow"], gross - applied);
    }
}

#[test]
fn iroi_zero_start_skill_burst_rotation_reconciles_damage_healing_and_energy() {
    const NORMAL_UNBUFFED: [(&str, u64, u64); 16] = [
        ("GE_Player_Oneiroi_Melee1_Damage", 380, 1),
        ("GE_Player_Oneiroi_Melee2_Damage", 159, 2),
        ("GE_Player_Oneiroi_Melee2_Damage", 159, 2),
        ("GE_Player_Oneiroi_Melee2_Damage", 159, 2),
        ("GE_Player_Oneiroi_Melee3_Damage", 103, 2),
        ("GE_Player_Oneiroi_Melee3_Damage", 103, 2),
        ("GE_Player_Oneiroi_Melee3_Damage", 103, 2),
        ("GE_Player_Oneiroi_Melee3_Damage", 103, 2),
        ("GE_Player_Oneiroi_Melee3_Damage", 103, 2),
        ("GE_Player_Oneiroi_Melee3_Damage", 103, 2),
        ("GE_Player_Oneiroi_Melee3_Damage", 103, 2),
        ("GE_Player_Oneiroi_Melee3_Damage", 103, 2),
        ("GE_Player_Oneiroi_Melee4_Damage", 351, 1),
        ("GE_Player_Oneiroi_Melee5_Damage", 129, 1),
        ("GE_Player_Oneiroi_Melee5_Damage", 129, 1),
        ("GE_Player_Oneiroi_Melee5_Damage", 129, 1),
    ];
    const NORMAL_BUFFED: [(&str, u64, u64); 16] = [
        ("GE_Player_Oneiroi_Melee1_Damage", 15200, 39),
        ("GE_Player_Oneiroi_Melee2_Damage", 1060, 13),
        ("GE_Player_Oneiroi_Melee2_Damage", 1060, 13),
        ("GE_Player_Oneiroi_Melee2_Damage", 1060, 13),
        ("GE_Player_Oneiroi_Melee3_Damage", 2060, 39),
        ("GE_Player_Oneiroi_Melee3_Damage", 2060, 39),
        ("GE_Player_Oneiroi_Melee3_Damage", 2060, 39),
        ("GE_Player_Oneiroi_Melee3_Damage", 2060, 39),
        ("GE_Player_Oneiroi_Melee3_Damage", 2060, 39),
        ("GE_Player_Oneiroi_Melee3_Damage", 2060, 39),
        ("GE_Player_Oneiroi_Melee3_Damage", 2060, 39),
        ("GE_Player_Oneiroi_Melee3_Damage", 2060, 39),
        ("GE_Player_Oneiroi_Melee4_Damage", 360, 1),
        ("GE_Player_Oneiroi_Melee5_Damage", 1720, 13),
        ("GE_Player_Oneiroi_Melee5_Damage", 1720, 13),
        ("GE_Player_Oneiroi_Melee5_Damage", 1720, 13),
    ];
    const SKILL: [(&str, u64, u64); 9] = [
        ("GE_Player_Oneiroi_Skill_Gun_Damage", 10000, 13),
        ("GE_Player_Oneiroi_Skill_Syringe_Damage", 3580, 39),
        ("GE_Player_Oneiroi_Skill_Syringe_Damage", 3580, 39),
        ("GE_Player_Oneiroi_Skill_Syringe_Damage", 3580, 39),
        ("GE_Player_Oneiroi_Skill_Syringe_Damage", 3580, 39),
        ("GE_Player_Oneiroi_Skill_Syringe_Damage", 3580, 39),
        ("GE_Player_Oneiroi_Skill_Syringe_Damage", 3580, 39),
        ("GE_Player_Oneiroi_Skill_Syringe_Damage", 3580, 39),
        ("GE_Player_Oneiroi_Skill_Trumpet_Damage", 25000, 39),
    ];
    const BURST: [(&str, u64, u64); 1] = [("GE_Player_Oneiroi_UltraSkill_Damage", 20000, 13)];
    const NORMAL_CHARGE: [u64; 16] = [
        2431, 996, 996, 996, 642, 642, 642, 642, 642, 642, 642, 642, 5525, 1989, 1989, 1989,
    ];
    const SKILL_CHARGE: [u64; 9] = [300, 614, 614, 614, 614, 614, 614, 614, 200];

    let case = CASES
        .iter()
        .find(|case| case.owner == "character:1075")
        .unwrap();
    let req = iroi_skill_burst_energy_request(case);
    let mut expected_hits = Vec::new();
    for _ in 0..5 {
        push_packets(&mut expected_hits, "GA_Oneiroi_Melee", &NORMAL_UNBUFFED);
    }
    push_packets(&mut expected_hits, "GA_Oneiroi_Skill", &SKILL);
    push_packets(&mut expected_hits, "GA_Oneiroi_UltraSkill", &BURST);
    push_packets(&mut expected_hits, "GA_Oneiroi_Melee", &NORMAL_BUFFED);

    let mut charge = Vec::new();
    for _ in 0..5 {
        charge.extend(NORMAL_CHARGE);
    }
    charge.extend(SKILL_CHARGE);
    charge.extend(NORMAL_CHARGE);
    assert_eq!(charge.len(), 105);
    let report = check_flow_with_spend(case, &req, &charge, &[(1, 1); 3], Some((89, 100, 100)));

    assert_hit_oracle(&report, &expected_hits, (1_111_315, 78));
    assert_eq!(report["hit_receipts"].as_array().unwrap().len(), 106);
    assert_eq!(
        report["hit_receipts"][80]["time_us"], 25_500_000,
        "Skill impacts must land after its cast-time DEF-ignore grant"
    );
    assert_eq!(report["hit_receipts"][89]["time_us"], 26_000_100);
    assert_eq!(report["hit_receipts"][90]["time_us"], 27_000_100);

    let trace = report["trace"].as_array().unwrap();
    assert_iroi_heals(trace, "iroi_skill_heal", 25_000_000, "943/1000", 943);
    assert_iroi_heals(trace, "iroi_burst_heal", 26_000_100, "1413/1000", 1413);
    let burst_hit_index = trace
        .iter()
        .position(|event| event["event_kind"] == "hit_damage" && event["time_us"] == 26_000_100)
        .unwrap();
    let first_burst_heal_index = trace
        .iter()
        .position(|event| event["event_kind"] == "iroi_burst_heal")
        .unwrap();
    assert!(burst_hit_index < first_burst_heal_index);
    for (actor, expected_hp) in [12_356, 12_356, 20_000].into_iter().enumerate() {
        assert_eq!(report["final_actor_states"][actor]["hp"], expected_hp);
        assert_eq!(report["final_actor_states"][actor]["hp_max"], 20_000);
    }

    assert_one_burst_spend(&report, 26_000_000, 100);
    assert_eq!(report["final_actor_states"][0]["energy"], 22);
    assert_eq!(report["final_actor_states"][0]["granted_energy"], 137);
    assert_eq!(report["final_actor_states"][0]["overflow_energy"], 15);
    assert_eq!(report["final_actor_states"][0]["spent_energy"], 100);
    assert_eq!(
        report["final_actor_states"][0]["charge_remainder"],
        fraction(2, 25)
    );
    for recipient in 1..3 {
        assert_eq!(report["final_actor_states"][recipient]["energy"], 82);
        assert_eq!(
            report["final_actor_states"][recipient]["granted_energy"],
            82
        );
        assert_eq!(
            report["final_actor_states"][recipient]["charge_remainder"],
            fraction(31, 125)
        );
    }
}
