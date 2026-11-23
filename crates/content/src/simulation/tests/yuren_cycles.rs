use super::*;

fn request(tier: u32, percent: u128) -> GameScenarioRequest {
    let source = include_str!("../../../../../examples/rainman-shield-cycles.nte")
        .replace("refine=1", &format!("refine={tier}"));
    let mut req = crate::dsl::compiler::compile_scenario(&source, Default::default())
        .unwrap()
        .request;
    let stats = &req.actors[0].static_stats;
    // Independent native-build checks: passive HP is added once to base HP;
    // the 52.5% DEF secondary is separate, with one nearest/ties-up settlement.
    assert_eq!(
        ExactFraction::from_ratio(stats.hp_percent).unwrap(),
        ExactFraction::new(percent, 100).unwrap()
    );
    assert_eq!(
        u128::from(stats.hp),
        (u128::from(stats.base_hp.unwrap()) * (100 + percent) + 50) / 100
    );
    assert_eq!(
        ExactFraction::from_ratio(stats.def_percent).unwrap(),
        ExactFraction::new(21, 40).unwrap()
    );
    assert_eq!(
        u128::from(stats.def),
        (u128::from(stats.base_def.unwrap()) * 61 + 20) / 40
    );
    for actor in &mut req.actors {
        actor.static_stats.base_atk = 1000;
        actor.static_stats.atk = Ratio {
            numerator: 1000,
            denominator: 1,
        };
        actor.static_stats.base_hp = Some(1000);
        actor.static_stats.hp = 1000;
        actor.static_stats.hp_percent = Ratio::zero();
        actor.static_stats.hp_flat = Ratio::zero();
        actor.initial_hp_max = 1000;
        actor.initial_hp = 1000;
    }
    req.actors[0].initial_hp = 501;
    let stats = &mut req.actors[0].static_stats;
    stats.base_def = Some(1000);
    stats.def = 1000;
    stats.def_percent = Ratio::zero();
    stats.def_flat = Ratio::zero();
    stats.shield_bonus = Ratio::zero();
    req
}

#[test]
fn rainman_native_shields_keep_hp_thresholds_absorption_healing_and_refinements_through_four_casts()
{
    for (tier, percent, strengthened) in [
        (1, 10, 2156),
        (2, 12, 2195),
        (3, 14, 2234),
        (4, 16, 2274),
        (5, 18, 2313),
    ] {
        let mut req = request(tier, percent);
        for (event, amount) in
            req.incoming_damage_events
                .iter_mut()
                .zip([strengthened + 1, 1961, 1960, strengthened])
        {
            event.delivered_damage = amount;
        }
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{}",
            report["halt_diagnostic"]
        );
        let receipts = report["incoming_damage_receipts"].as_array().unwrap();
        assert_eq!(receipts.len(), 4);
        // Native Skill at DEF1200 grants 1960. HP501 qualifies; equality500
        // and below499 do not. Iroi's ordinary live-ATK heal later fills1000,
        // restoring eligibility at the fourth cast. Each shield is a snapshot.
        for (index, capacity) in [strengthened, 1960, 1960, strengthened]
            .into_iter()
            .enumerate()
        {
            let absorbed: u64 = receipts[index]["absorbed_by_shields"]
                .as_array()
                .unwrap()
                .iter()
                .map(|part| part["amount"].as_u64().unwrap())
                .sum();
            assert_eq!(absorbed, capacity, "tier {tier}, cast {index}");
            assert_eq!(receipts[index]["actor_index"], 0);
            assert_eq!(receipts[index]["actual_hp_loss"], [1, 1, 0, 0][index]);
            assert_eq!(receipts[index]["hp_after"], [500, 499, 499, 1000][index]);
        }
        assert_eq!(
            report["trace"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|event| event["event_kind"] == "adler_blessing_shield")
                .count(),
            4
        );
        assert_eq!(report["final_actor_states"][0]["hp"], 1000);
        assert_eq!(report["final_buff_state"]["active_actor_index"], 1);
    }
}

#[test]
fn rainman_joins_existing_shield_efficiency_in_one_additive_grant_zone() {
    let mut req = request(1, 10);
    req.actors[0].static_stats.shield_bonus = Ratio {
        numerator: 1,
        denominator: 5,
    };
    for (event, amount) in req
        .incoming_damage_events
        .iter_mut()
        .zip([2549, 2353, 2352, 2548])
    {
        event.delivered_damage = amount;
    }
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let absorbed: Vec<u64> = report["incoming_damage_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|receipt| {
            receipt["absorbed_by_shields"]
                .as_array()
                .unwrap()
                .iter()
                .map(|part| part["amount"].as_u64().unwrap())
                .sum()
        })
        .collect();
    // 1960*(1+.20+.10)=2548. Equality and below remove only Rainman's .10.
    assert_eq!(absorbed, [2548, 2352, 2352, 2548]);
    assert_eq!(report["final_actor_states"][0]["hp"], 1000);
}
