use super::*;

fn chain(req: &mut GameScenarioRequest, start: u64) {
    let mut time = start;
    for recovery in [923724, 840277, 1218687, 1706341, 820279] {
        req.actions.push(normal(time));
        time += recovery;
    }
}

#[test]
fn cycle_is_per_owner_packet_and_carries_exactly_through_caps() {
    for targets in [1, 3] {
        let mut req = phase2_request();
        req.actions.clear();
        req.actors[0].initial_energy = 0;
        req.actors[0].energy_capacity = 1000;
        for index in 1..targets {
            let mut target = req.targets[0].clone();
            target.target_index = index;
            req.targets.push(target);
        }
        for index in 0..3 {
            chain(&mut req, index * 5509308);
        }
        let report = run(&req);
        let actor = &report["final_actor_states"][0];
        assert_eq!(
            actor["cycle"],
            json!({"initial":0,"current":100,"capacity":100,"granted":112,"spent":0,"overflow":12,"carry":"29/250","conserved":true})
        );
        // Each chain has 37.372 Cycle and 22.372 Ultimate, regardless of recipients.
        assert_eq!(actor["granted_energy"], 67);
        assert_eq!(actor["charge_remainder"], json!("29/250"));
    }
}

#[test]
fn cycle_gain_efficiency_is_independent_and_a_dead_primary_does_not_lose_the_group() {
    let mut req = phase2_request();
    req.actions.clear();
    req.actors[0].cycle_gain_efficiency = Ratio {
        numerator: 3,
        denominator: 2,
    };
    req.actors[0].initial_energy = 0;
    req.actors[0].energy_capacity = 1000;
    req.targets.push(req.targets[0].clone());
    req.targets[1].target_index = 1;
    req.targets[0].hp_current = 0;
    chain(&mut req, 0);
    let report = run(&req);
    let actor = &report["final_actor_states"][0];
    assert_eq!(actor["cycle"]["current"], 56);
    assert_eq!(actor["cycle"]["carry"], json!("29/500"));
    assert_eq!(actor["granted_energy"], 22);
    assert_eq!(actor["charge_remainder"], json!("93/250"));
    assert_eq!(report["hits_settled"], 11);
}

#[test]
fn zero_damage_and_zero_efficiency_do_not_create_free_cycle() {
    let mut req = phase2_request();
    req.actions = vec![normal(0)];
    req.actors[0].cycle_gain_efficiency = Ratio::zero();
    let report = run(&req);
    assert_eq!(report["final_actor_states"][0]["cycle"]["current"], 0);
    assert_eq!(
        report["final_actor_states"][0]["cycle"]["carry"],
        json!("0/1")
    );
    req.actors[0].static_stats.atk = Ratio::zero();
    req.actors[0].static_stats.base_atk = 0;
    assert!(run(&req)["final_actor_states"][0].get("cycle").is_none());
}

#[test]
fn qualified_support_spends_gained_cycle_and_preserves_fractional_carry_on_return() {
    let mut req = phase2_request();
    req.actions.clear();
    req.actors[0].initial_cycle = 0;
    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1004".into();
    ally.initial_cycle = 37;
    req.actors.push(ally);
    for index in 0..3 {
        chain(&mut req, index * 5509308);
    }
    req.actions.extend([
        GameActionCommand::SwitchActor {
            timestamp_us: 17_000_000,
            target_actor_index: 1,
            cooldown_us: 250000,
        },
        GameActionCommand::SwitchActor {
            timestamp_us: 20_000_000,
            target_actor_index: 0,
            cooldown_us: 250000,
        },
        normal(20_250_000),
    ]);
    let report = run(&req);
    let cycle = &report["final_actor_states"][0]["cycle"];
    assert_eq!(cycle["spent"], 100);
    assert_eq!(cycle["current"], 3);
    assert_eq!(cycle["overflow"], 12);
    assert_eq!(cycle["carry"], json!("887/1000"));
    assert_eq!(cycle["conserved"], true);
}

#[test]
fn invalid_cycle_efficiency_rejects_before_execution() {
    let mut req = phase2_request();
    req.actors[0].cycle_gain_efficiency = Ratio {
        numerator: 1,
        denominator: 0,
    };
    assert!(run_game_scenario(&req, &ExecutionLimits::default()).is_err());
}
