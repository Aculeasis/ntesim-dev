use super::hathor_actions::{make_action, request};
use super::*;
use serde_json::json;

fn team_request() -> GameScenarioRequest {
    let mut req = request();
    let mut base = req.actors[0].clone();
    base.initial_energy = 0;
    base.energy_capacity = 100;
    base.weapon_id.clear();
    base.set_id = None;
    base.gear_procs.clear();
    req.actors = ["character:1025", "character:1019", "character:1003"]
        .into_iter()
        .enumerate()
        .map(|(slot, character_id)| {
            let mut actor = base.clone();
            actor.slot = slot;
            actor.character_id = character_id.into();
            actor
        })
        .collect();
    req.actions = vec![
        make_action(
            0,
            0,
            "GA_Hathor_Melee",
            "GE_Player_Hathor_Branch_Damage",
            0,
            0,
        ),
        GameActionCommand::SwitchActor {
            timestamp_us: 50,
            target_actor_index: 1,
            cooldown_us: 0,
        },
        GameActionCommand::SwitchActor {
            timestamp_us: 500,
            target_actor_index: 0,
            cooldown_us: 0,
        },
        make_action(
            1_000,
            0,
            "GA_Hathor_Melee",
            "GE_Player_Hathor_AirAttack_Damage",
            0,
            0,
        ),
    ];
    req
}

#[test]
fn hathor_direct_branch_and_airattack_energy_use_independent_team_carry_oracle() {
    let mut req = team_request();
    for (actor, (numerator, denominator)) in req.actors.iter_mut().zip([(28, 25), (3, 2), (7, 5)]) {
        actor.gear_procs.push(GearProcConfig::ChargeEfficiency {
            multiplier: Ratio {
                numerator,
                denominator,
            },
        });
    }

    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(report["hits_settled"], 2);
    assert_eq!(
        report["resolved_model_assumptions"]
            [crate::simulation::energy::HATHOR_DIRECT_PACKET_ENERGY_HYPOTHESIS_ID.as_str()],
        crate::simulation::energy::HATHOR_DIRECT_PACKET_ENERGY_HYPOTHESIS_ID.description()
    );
    assert!(
        !report["resolved_model_assumptions"]
            .as_object()
            .unwrap()
            .contains_key(crate::simulation::energy::HATHOR_HIT_ENERGY_HYPOTHESIS_ID.as_str())
    );

    // Independent literal oracle: source ChargeAdd is 12.0/6.3, other-element
    // distribution is 3/5, then recipient efficiencies are 28/25, 3/2 and 7/5.
    // Integer grants are settled after carry per recipient: [13,10,10], then [7,6,5].
    let expected = [
        json!({
            "source": "hathor_hit",
            "source_actor_index": 0,
            "base": "12/1",
            "recipients": [
                {"recipient_actor_index":0,"distribution":"1/1","energy_before":0,"gross":13,"retained":13,"overflow":0,"energy_after":13,"charge_remainder_after":"11/25","charge_basis":"12/1"},
                {"recipient_actor_index":1,"distribution":"3/5","energy_before":0,"gross":10,"retained":10,"overflow":0,"energy_after":10,"charge_remainder_after":"4/5","charge_basis":"36/5"},
                {"recipient_actor_index":2,"distribution":"3/5","energy_before":0,"gross":10,"retained":10,"overflow":0,"energy_after":10,"charge_remainder_after":"2/25","charge_basis":"36/5"}
            ]
        }),
        json!({
            "source": "hathor_hit",
            "source_actor_index": 0,
            "base": "63/10",
            "recipients": [
                {"recipient_actor_index":0,"distribution":"1/1","energy_before":13,"gross":7,"retained":7,"overflow":0,"energy_after":20,"charge_remainder_before":"11/25","charge_remainder_after":"62/125","charge_basis":"63/10"},
                {"recipient_actor_index":1,"distribution":"3/5","energy_before":10,"gross":6,"retained":6,"overflow":0,"energy_after":16,"charge_remainder_before":"4/5","charge_remainder_after":"47/100","charge_basis":"189/50"},
                {"recipient_actor_index":2,"distribution":"3/5","energy_before":10,"gross":5,"retained":5,"overflow":0,"energy_after":15,"charge_remainder_before":"2/25","charge_remainder_after":"93/250","charge_basis":"189/50"}
            ]
        }),
    ];
    let grant_events = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["event_kind"] == "hathor_direct_packet_energy_grant")
        .collect::<Vec<_>>();
    assert_eq!(grant_events.len(), 2);
    assert_eq!(grant_events[0]["active_actor"], 1);
    assert_eq!(grant_events[1]["active_actor"], 0);
    let grants = grant_events
        .iter()
        .map(|event| event["energy_fanout"].clone())
        .collect::<Vec<_>>();
    assert_eq!(grants, expected);

    for (slot, (energy, remainder)) in [
        (20, json!("62/125")),
        (16, json!("47/100")),
        (15, json!("93/250")),
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(report["final_actor_states"][slot]["energy"], energy);
        assert_eq!(report["final_actor_states"][slot]["granted_energy"], energy);
        assert_eq!(
            report["final_actor_states"][slot]["charge_remainder"],
            remainder
        );
    }
}

#[test]
fn hathor_direct_energy_requires_positive_hp_loss_and_respects_fatal_cap_and_dead_recipients() {
    let mut req = team_request();
    req.actions.truncate(1);
    req.actors[0].initial_energy = 98;
    req.actors[1].initial_energy = 95;
    req.actors[2].initial_energy = 100;
    req.actors[2].initial_hp = 0;
    req.targets[0].hp_current = 1;
    let fatal = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(fatal["final_target_states"][0]["hp_current"], 0);
    let grants = fatal["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["event_kind"] == "hathor_direct_packet_energy_grant")
        .map(|event| event["energy_fanout"].clone())
        .collect::<Vec<_>>();
    assert_eq!(
        grants,
        [json!({
            "source": "hathor_hit",
            "source_actor_index": 0,
            "base": "12/1",
            "recipients": [
                {"recipient_actor_index":0,"distribution":"1/1","energy_before":98,"gross":12,"retained":2,"overflow":10,"energy_after":100,"charge_basis":"12/1"},
                {"recipient_actor_index":1,"distribution":"3/5","energy_before":95,"gross":7,"retained":5,"overflow":2,"energy_after":100,"charge_remainder_after":"1/5","charge_basis":"36/5"}
            ]
        })]
    );

    let mut zero_loss = team_request();
    zero_loss.actors.truncate(1);
    zero_loss.actions.truncate(1);
    zero_loss.actors[0].static_stats.atk = Ratio::zero();
    let zero = run_game_scenario(&zero_loss, &ExecutionLimits::default()).unwrap();
    assert_eq!(zero["hits_settled"], 1);
    assert_eq!(zero["final_actor_states"][0]["energy"], 0);
    assert!(
        zero["trace"]
            .as_array()
            .unwrap()
            .iter()
            .all(|event| event["event_kind"] != "hathor_direct_packet_energy_grant")
    );
    assert!(
        !zero["resolved_model_assumptions"]
            .as_object()
            .unwrap()
            .contains_key(
                crate::simulation::energy::HATHOR_DIRECT_PACKET_ENERGY_HYPOTHESIS_ID.as_str()
            )
    );
}
