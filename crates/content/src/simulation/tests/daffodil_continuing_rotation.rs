use super::daffodil_actions::{make_action, request};
use super::*;
use crate::simulation::tests::support::formula_damage_total;
use serde_json::{Value, json};

fn add_receipt_damage(
    report: &Value,
    command_range: std::ops::RangeInclusive<usize>,
) -> ExactFraction {
    report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|receipt| {
            let command_index = receipt["command_index"]
                .as_u64()
                .expect("receipt has a command index") as usize;
            command_range.contains(&command_index)
        })
        .try_fold(ExactFraction::new(0, 1).unwrap(), |sum, receipt| {
            sum.add(
                serde_json::from_value::<ExactFraction>(receipt["damage"].clone())
                    .map_err(|err| err.to_string())?,
            )
        })
        .unwrap()
}

fn normal_action(timestamp_us: u64) -> GameActionCommand {
    let mut action = make_action(
        timestamp_us,
        0,
        "GA_Daffodill_Melee",
        "GE_Player_Daffodill_Melee1_Damage",
        0,
        0,
    );
    if let GameActionCommand::InvokeAbility {
        recovery_us,
        hit_delay_us,
        ..
    } = &mut action
    {
        *recovery_us = 800_000;
        *hit_delay_us = 100;
    }
    action
}

#[test]
fn daffodil_zero_start_skill_burst_and_normal_continuation() {
    let mut req = request();
    req.relative_duration_us = 36_000_000;
    req.actors[0].talent_levels = Some([8, 8, 8]);
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(0);
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
    req.actors[0].energy_capacity = 120;

    for (slot, character_id) in [(1, "character:1019"), (2, "character:1072")] {
        let mut ally = req.actors[0].clone();
        ally.slot = slot;
        ally.character_id = character_id.into();
        ally.weapon_id.clear();
        ally.weapon_tier = 1;
        ally.gear_procs.clear();
        ally.set_id = None;
        ally.initial_energy = 0;
        ally.energy_capacity = 120;
        req.actors.push(ally);
    }

    req.targets[0].hp_current = 10_000_000;
    req.targets[0].defense = 1080;
    req.targets[0].resistance = Ratio::zero();
    req.actions = (0..30)
        .map(|index| normal_action(index * 800_000))
        .collect();
    let mut skill = make_action(
        24_000_000,
        0,
        "GA_Daffodill_Skill",
        "GE_Player_Daffodill_Skill1_Damage",
        0,
        16_000_000,
    );
    if let GameActionCommand::InvokeAbility {
        recovery_us,
        hit_delay_us,
        ..
    } = &mut skill
    {
        *recovery_us = 1_200_000;
        *hit_delay_us = 100;
    }
    let mut burst = make_action(
        26_000_000,
        0,
        "GA_Daffodill_UltraSkill",
        "GE_Player_Daffodill_UltraSkill1_Damage",
        120,
        20_000_000,
    );
    if let GameActionCommand::InvokeAbility {
        recovery_us,
        hit_delay_us,
        ..
    } = &mut burst
    {
        *recovery_us = 2_000_000;
        *hit_delay_us = 100;
    }
    req.actions.push(skill);
    req.actions.push(burst);
    req.actions
        .extend((0..5).map(|index| normal_action(32_000_000 + index * 800_000)));

    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["relative_duration_us"], 36_000_000);
    assert_eq!(report["hits_settled"], 155);
    assert_eq!(
        add_receipt_damage(&report, 0..=29),
        ExactFraction::new(16_389, 1).unwrap()
    );
    assert_eq!(
        add_receipt_damage(&report, 30..=30),
        ExactFraction::new(5_145, 2).unwrap()
    );
    assert_eq!(
        add_receipt_damage(&report, 31..=31),
        ExactFraction::new(37_708, 5).unwrap()
    );
    assert_eq!(
        add_receipt_damage(&report, 32..=36),
        ExactFraction::new(60_093, 20).unwrap()
    );
    assert_eq!(
        serde_json::json!(formula_damage_total(&report)),
        json!("118031/4")
    );
    assert_eq!(
        ExactFraction::new(118_031, 144).unwrap(),
        ExactFraction::new(118_031, 4)
            .unwrap()
            .div(ExactFraction::new(36, 1).unwrap())
            .unwrap()
    );

    let receipts = report["hit_receipts"].as_array().unwrap();
    for (command_index, time_us) in [
        (0, 100),
        (29, 23_200_100),
        (30, 24_000_100),
        (31, 26_000_100),
        (32, 32_000_100),
        (36, 35_200_100),
    ] {
        let first_hit = receipts
            .iter()
            .find(|receipt| receipt["command_index"] == command_index)
            .unwrap();
        assert_eq!(first_hit["time_us"], time_us);
    }

    assert_eq!(report["final_actor_states"][0]["initial_energy"], 0);
    assert_eq!(report["final_actor_states"][0]["granted_energy"], 145);
    assert_eq!(report["final_actor_states"][0]["spent_energy"], 120);
    assert_eq!(report["final_actor_states"][0]["energy"], 20);
    assert_eq!(report["final_actor_states"][0]["overflow_energy"], 5);
    assert_eq!(
        report["final_actor_states"][0]["charge_remainder"],
        json!("413/1000")
    );
    for slot in [1, 2] {
        assert_eq!(report["final_actor_states"][slot]["granted_energy"], 87);
        assert_eq!(report["final_actor_states"][slot]["energy"], 87);
        assert_eq!(
            report["final_actor_states"][slot]["charge_remainder"],
            json!("1239/5000")
        );
    }
    assert!(report["resolved_model_assumptions"]["daffodil.normal_chain_v1"].is_string());
    assert!(report["resolved_model_assumptions"]["daffodil.skill_burst_packets_v1"].is_string());
    assert!(report["resolved_model_assumptions"]["daffodil.ordinary_hit_energy_v1"].is_string());
}
