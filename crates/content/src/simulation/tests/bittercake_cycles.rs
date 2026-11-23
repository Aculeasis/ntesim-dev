use super::*;

#[test]
fn bittercake_native_def_damage_and_shields_keep_hp_loss_icd_and_expiry_through_three_casts() {
    for (tier, defense, shield) in [
        (1, 1700, 2360),
        (2, 1750, 2400),
        (3, 1800, 2440),
        (4, 1850, 2480),
        (5, 1900, 2520),
    ] {
        let source = include_str!("../../../../../examples/bittercake-defense-cycles.nte")
            .replace("refine=1", &format!("refine={tier}"));
        let mut req = crate::dsl::compiler::compile_scenario(&source, Default::default())
            .unwrap()
            .request;
        let actor = &mut req.actors[0];
        actor.initial_hp = 10000;
        actor.initial_hp_max = 10000;
        let stats = &mut actor.static_stats;
        stats.base_def = Some(1000);
        stats.def = 1240;
        stats.def_percent = Ratio::zero();
        stats.def_flat = Ratio {
            numerator: 240,
            denominator: 1,
        };
        stats.crit_rate = Ratio::zero();
        stats.damage_bonus = Ratio::zero();
        stats.elemental_bonus = Ratio::zero();
        stats.elemental_bonuses.clear();
        for (event, amount) in req
            .incoming_damage_events
            .iter_mut()
            .zip([1, 1, shield, 1, 1, shield, 2152])
        {
            event.delivered_damage = amount;
        }
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{}",
            report["halt_diagnostic"]
        );
        let grants: Vec<_> = report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "bittercake_defense")
            .map(|event| event["time_us"].as_u64().unwrap())
            .collect();
        assert_eq!(grants, [0, 20_000_000]);
        let receipts = report["incoming_damage_receipts"].as_array().unwrap();
        for (index, expected) in [(1, 1), (2, shield - 1), (5, shield), (6, 2152)] {
            let absorbed: u64 = receipts[index]["absorbed_by_shields"]
                .as_array()
                .unwrap()
                .iter()
                .map(|part| part["amount"].as_u64().unwrap())
                .sum();
            assert_eq!(absorbed, expected, "tier {tier}, incoming {index}");
        }
        let loss: Vec<_> = receipts
            .iter()
            .map(|receipt| receipt["actual_hp_loss"].as_u64().unwrap())
            .collect();
        assert_eq!(loss, [1, 0, 1, 1, 1, 0, 0]);
        let initial: Vec<_> = report["hit_receipts"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|hit| hit["damage_effect_id"] == "GE_Player_Adler_Skill2_Damage")
            .map(|hit| hit["rounded_damage"].as_u64().unwrap())
            .collect();
        // .7 times DEF (1000 + 240 flat + 200 Righteous Heart + the weapon grant).
        assert_eq!(initial, [defense * 7 / 10, defense * 7 / 10, 1008]);
        assert_eq!(report["final_actor_states"][0]["hp"], 9996);
        assert_eq!(report["final_buff_state"]["active_actor_index"], 1);
    }
}
