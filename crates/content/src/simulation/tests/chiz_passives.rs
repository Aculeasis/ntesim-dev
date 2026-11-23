use super::*;
use crate::formula::Ratio;
use crate::simulation::{GameActionCommand, GearProcConfig};
use serde_json::json;

fn chiz_request() -> GameScenarioRequest {
    crate::dsl::compiler::compile_scenario(
        "options duration=12;
        chiz char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=0;
        target hp=100000000 def=1080 resist=0; active chiz;",
        Default::default(),
    )
    .unwrap()
    .request
}

fn charge_request() -> GameScenarioRequest {
    let mut req = super::remora_tests::request(3_000_000);
    let mut chiz = chiz_request().actors.remove(0);
    chiz.slot = 3;
    chiz.initial_energy = 0;
    req.actors.push(chiz);
    req
}

#[test]
fn chiz_hanahaki_augments_only_actual_charge_once_with_live_receiver_efficiency() {
    for extra_target in [false, true] {
        let mut req = charge_request();
        if let GameActionCommand::SwitchActor { timestamp_us, .. } = &mut req.actions[1] {
            *timestamp_us = 1_603_615;
        }
        req.actors[2]
            .gear_procs
            .retain(|proc| !matches!(proc, GearProcConfig::ChargeEfficiency { .. }));
        req.actors[2]
            .gear_procs
            .push(GearProcConfig::ChargeEfficiency {
                multiplier: Ratio {
                    numerator: 133,
                    denominator: 100,
                },
            });
        if extra_target {
            let mut target = req.targets[0].clone();
            target.target_index = 1;
            req.targets.push(target);
        }
        let report = super::remora_tests::run(&req, extra_target);
        let grants = super::remora_tests::charges(&report);
        assert_eq!(grants.len(), if extra_target { 5 } else { 3 });
        for grant in grants {
            assert_eq!(grant["energy_change"]["charge_basis"], json!("14/1"));
            assert_eq!(grant["energy_change"]["recipient_actor_index"], 2);
        }
        // The extra12s status extends eligibility, never grants per recipient.
        assert_eq!(
            report["final_actor_states"][2]["energy"],
            if extra_target { 100 } else { 74 }
        );
        assert_eq!(
            report["final_actor_states"][2]["charge_remainder"],
            if extra_target {
                json!("587/1000")
            } else {
                json!("347/1000")
            }
        );
        // The two Supports add (5.7 + 8.2) * 1.33 to Hathor
        // and (9.5 + 8.2) * .6 to the off-field Chiz.
        assert_eq!(
            report["final_actor_states"][2]["overflow_energy"],
            if extra_target { 11 } else { 0 }
        );
        assert_eq!(report["final_actor_states"][3]["energy"], 10);
        assert_eq!(
            report["final_actor_states"][3]["charge_remainder"],
            json!("31/50")
        );
    }
}

#[test]
fn chiz_hanahaki_live_provider_and_unlock_are_checked_at_the_real_pistil() {
    for dead in [false, true] {
        let mut req = charge_request();
        if dead {
            req.incoming_damage_events
                .push(crate::simulation::GameActorDamageEvent {
                    timestamp_us: 5_000_000,
                    actor_index: 3,
                    delivered_damage: req.actors[3].initial_hp,
                });
            let model = crate::HypothesisId::CombatIncomingDamageFixtureV1;
            req.model_assumptions
                .insert(model.into(), model.description().into());
        } else {
            req.actors[3].character_level = 20;
            req.actors[3].character_max_level = Some(20);
        }
        let report = super::remora_tests::run(&req, false);
        let grants = super::remora_tests::charges(&report);
        assert_eq!(
            grants[0]["energy_change"]["charge_basis"],
            json!(format!("{}/{}", if dead { 14 } else { 10 }, 1))
        );
        assert_eq!(grants[1]["energy_change"]["charge_basis"], json!("10/1"));
    }
}

#[test]
fn chiz_active_hanahaki_and_hoarding_use_one_raw_bonus_then_one_additive_efficiency() {
    let mut req = charge_request();
    // Retain two active-Chiz pistils after native Support's recovery, plus
    // one earlier pistil delivered to Hathor during that same continuing bud.
    if let GameActionCommand::SwitchActor { timestamp_us, .. } = &mut req.actions[1] {
        *timestamp_us = 1_603_615;
    }
    req.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 3_500_000,
        target_actor_index: 3,
        cooldown_us: 0,
    });
    let report = super::remora_tests::run(&req, false);
    let grants = super::remora_tests::charges(&report);
    assert_eq!(grants.len(), 3);
    assert_eq!(grants[0]["energy_change"]["recipient_actor_index"], 2);
    for grant in &grants[1..] {
        assert_eq!(grant["energy_change"]["recipient_actor_index"], 3);
        assert_eq!(grant["energy_change"]["charge_basis"], json!("14/1"));
    }
    // 10.62 Support peer energy + 2 * 14 * 1.2 active Charge = 44.22.
    assert_eq!(report["final_actor_states"][3]["energy"], 44);
    assert_eq!(
        report["final_actor_states"][3]["charge_remainder"],
        json!("11/50")
    );
}

fn invoke(time: u64, skill: bool) -> GameActionCommand {
    serde_json::from_value(json!({"kind":"invoke_ability","timestamp_us":time,"actor_index":0,
        "ability_id":if skill {"GA_Chiichan073_Skill"} else {"GA_Chiichan073_Melee"},
        "damage_effect_id":if skill {"GE_Player_Chiichan073_Skill1_1Damage"} else {"GE_Player_Chiichan073_Melee1_Damage"},
        "array_index":0,"atk_rate":"1/1","scaling_stat":"atk",
        "energy_cost":0,"recovery_us":0,"cooldown_us":0,"hit_delay_us":if skill {500_000} else {100}})).unwrap()
}

#[test]
fn chiz_hoarding_reads_live_field_state_for_afterimage_and_two_continuing_reentries() {
    for unlocked in [false, true] {
        let mut req = chiz_request();
        req.actors[0].initial_energy = 0;
        req.actors[0]
            .gear_procs
            .retain(|proc| !matches!(proc, GearProcConfig::ChargeEfficiency { .. }));
        req.actors[0]
            .gear_procs
            .push(GearProcConfig::ChargeEfficiency {
                multiplier: Ratio {
                    numerator: 133,
                    denominator: 100,
                },
            });
        if !unlocked {
            req.actors[0].character_level = 40;
            req.actors[0].character_max_level = Some(50);
        }
        let mut ally = req.actors[0].clone();
        ally.slot = 1;
        ally.character_id = "character:1019".into();
        ally.gear_procs.clear();
        req.actors.push(ally);
        req.actions.clear();
        for time in [0, 3_000_000] {
            req.actions.push(invoke(time, true));
            req.actions.push(GameActionCommand::SwitchActor {
                timestamp_us: time + 100_000,
                target_actor_index: 1,
                cooldown_us: 0,
            });
            req.actions.push(GameActionCommand::SwitchActor {
                timestamp_us: time + 2_000_000,
                target_actor_index: 0,
                cooldown_us: 0,
            });
            req.actions.push(invoke(time + 2_000_001, false));
        }
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(report["final_actor_states"][0]["energy"], 7);
        assert_eq!(
            report["final_actor_states"][0]["charge_remainder"],
            if unlocked {
                json!("42709/50000")
            } else {
                json!("8169/50000")
            }
        );
        assert_eq!(report["final_actor_states"][1]["energy"], 3);
        assert_eq!(
            report["final_actor_states"][1]["charge_remainder"],
            json!("579/2500")
        );
        assert!(super::remora_tests::charges(&report).is_empty()); // Cosmos hits are not Charge.
    }
}
