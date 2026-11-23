use super::*;
use serde_json::{Value, json};

fn request() -> GameScenarioRequest {
    let mut req = crate::dsl::compiler::compile_scenario(
        include_str!("../../../../../examples/zero-support-cycles.nte"),
        Default::default(),
    )
    .unwrap()
    .request;
    let actor = &mut req.actors[1];
    actor.gear_procs.clear();
    actor.static_stats.base_atk = 1000;
    actor.static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    actor.static_stats.crit_rate = Ratio::zero();
    actor.static_stats.damage_bonus = Ratio::zero();
    actor.static_stats.elemental_bonus = Ratio::zero();
    actor.static_stats.elemental_bonuses.clear();
    req
}

fn hits(report: &Value) -> Vec<&Value> {
    report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["ability_id"] == "GA_Female046_QTE")
        .collect()
}

#[test]
fn zero_support_keeps_two_packet_talent_coordinates_energy_and_continuing_cooldowns() {
    for (talent, awakening, first, second, atk_tenths) in [
        (1, 0, 753, 1247, 10),
        (8, 0, 1291, 2137, 10),
        (8, 3, 1291, 2137, 10),
        (8, 4, 1291, 2137, 10),
        (8, 5, 1291, 2137, 11),
        (10, 0, 1505, 2493, 10),
    ] {
        let mut req = request();
        req.actors[1].purchased_qte_level = Some(talent);
        req.actors[1].talent_levels = Some([talent; 3]);
        req.actors[1].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{}",
            report["rejected_action"]
        );
        let hits = hits(&report);
        assert_eq!(hits.len(), 12);
        for (index, hit) in hits.iter().enumerate() {
            let cycle = index / 4;
            let packet = index % 4 / 2;
            assert_eq!(
                hit["time_us"],
                cycle as u64 * 15_000_000 + [100_000, 350_000][packet]
            );
            assert_eq!(hit["actor_index"], 1);
            assert_eq!(hit["target_index"], index % 2);
            assert_eq!(hit["command_index"], [0, 3, 6][cycle]);
            assert_eq!(
                hit["switch_origin"],
                json!({"kind":"support","incoming_actor_index":1})
            );
            let expected = ExactFraction::new([first, second][packet] * atk_tenths, 10).unwrap();
            assert_eq!(
                hit["non_crit_damage"],
                serde_json::to_value(expected).unwrap()
            );
        }
        // Three casts * (2.899 + 4.801), once per packet, not per AoE recipient.
        assert_eq!(report["final_actor_states"][1]["energy"], 23);
        assert_eq!(
            report["final_actor_states"][1]["charge_remainder"],
            json!("1/10")
        );
        assert_eq!(report["final_actor_states"][0]["energy"], 13);
        assert_eq!(
            report["final_actor_states"][0]["charge_remainder"],
            json!("43/50")
        );
    }
}

#[test]
fn zero_support_energy_uses_the_first_positive_recipient_and_owner_death_cancels_followup() {
    let mut req = request();
    req.targets[0].hp_current = 0;
    req.aim_targets = [(0, 1), (3, 1), (6, 1)].into();
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(hits(&report).len(), 6);
    assert_eq!(report["final_actor_states"][1]["energy"], 23);
    req.actions.truncate(1);
    req.aim_targets.clear();
    req.relative_duration_us = 1_000_000;
    req.targets[0].hp_current = req.targets[0].hp_max;
    req.incoming_damage_events
        .push(crate::simulation::GameActorDamageEvent {
            timestamp_us: 350_000,
            actor_index: 1,
            delivered_damage: req.actors[1].initial_hp,
        });
    let id = crate::HypothesisId::CombatIncomingDamageFixtureV1;
    req.model_assumptions
        .insert(id.into(), id.description().into());
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    assert_eq!(hits(&report).len(), 2);
    assert_eq!(report["final_actor_states"][1]["energy"], 2);
    assert_eq!(
        report["final_actor_states"][1]["charge_remainder"],
        json!("899/1000")
    );
}

#[test]
fn zero_support_implicit_entry_preserves_recovery_failure() {
    let req = crate::dsl::compiler::compile_scenario(
        "options duration=2; mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
         zero char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
         target hp=100000000 def=0 resist=0; active mint; zero attack;",
        Default::default(),
    )
    .unwrap()
    .request;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["execution_completed"], false);
    assert_eq!(
        report["rejected_action"]["reason"],
        json!({"kind":"recovery","until_us":600_000})
    );
    assert_eq!(report["final_buff_state"]["active_actor_index"], 1);
    assert_eq!(
        report["action_metadata_ledger"][0]["switch_origin"],
        json!({"kind":"support","incoming_actor_index":1})
    );
}

#[test]
fn zero_support_launched_followup_keeps_live_off_field_owner_and_native_coordinate() {
    let mut req = request();
    req.active_actor_index = 1;
    req.actions = vec![
        GameActionCommand::InvokeAbility {
            timestamp_us: 0,
            actor_index: 1,
            ability_id: "GA_Female046_QTE".into(),
            damage_effect_id: "GE_Player_Female046_QTE1_Damage".into(),
            array_index: 0,
            atk_rate: Ratio {
                numerator: 753,
                denominator: 1000,
            },
            scaling_stat: NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 0,
            cooldown_us: 0,
            hit_delay_us: 100_000,
        },
        GameActionCommand::SwitchActor {
            timestamp_us: 200_000,
            target_actor_index: 0,
            cooldown_us: 0,
        },
    ];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let hits = hits(&report);
    assert_eq!(hits.len(), 4);
    assert_eq!(hits[2]["time_us"], 350_000);
    assert_eq!(hits[2]["actor_index"], 1);
    assert_eq!(hits[2]["command_index"], 0);
    assert_eq!(hits[2]["non_crit_damage"], json!("1247/1"));
    assert_eq!(report["final_buff_state"]["active_actor_index"], 0);
    assert_eq!(report["final_actor_states"][1]["energy"], 7);
}
