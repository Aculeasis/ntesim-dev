use super::*;
use serde_json::json;

struct DirectPacket {
    effect: &'static str,
    charge_milli: u32,
}

struct DirectOwner {
    character: &'static str,
    ability: &'static str,
    source: &'static str,
    event_kind: &'static str,
    energy_hypothesis_id: &'static str,
    energy_hypothesis: &'static str,
    input_hypothesis_id: &'static str,
    input_hypothesis: &'static str,
    packets: &'static [DirectPacket],
}

const DIRECT_OWNERS: &[DirectOwner] = &[
    DirectOwner {
        character: "character:1054",
        ability: "GA_Daffodill_Melee",
        source: "daffodil_hit",
        event_kind: "daffodil_direct_packet_energy_grant",
        energy_hypothesis_id:
            crate::simulation::energy::DAFFODIL_DIRECT_PACKET_ENERGY_HYPOTHESIS_ID.as_str(),
        energy_hypothesis: crate::simulation::energy::DAFFODIL_DIRECT_PACKET_ENERGY_HYPOTHESIS_ID
            .description(),
        input_hypothesis_id: crate::characters::daffodil::skill::DIRECT_PACKET_INPUTS_HYPOTHESIS_ID
            .as_str(),
        input_hypothesis: crate::characters::daffodil::skill::DIRECT_PACKET_INPUTS_HYPOTHESIS_ID
            .description(),
        packets: &[
            DirectPacket {
                effect: "GE_Player_Daffodill_Branch1_Damage",
                charge_milli: 722,
            },
            DirectPacket {
                effect: "GE_Player_Daffodill_Branch2_Damage",
                charge_milli: 594,
            },
            DirectPacket {
                effect: "GE_Player_Daffodill_Branch3_Damage",
                charge_milli: 2420,
            },
            DirectPacket {
                effect: "GE_Player_Daffodill_Branch4_Damage",
                charge_milli: 702,
            },
            DirectPacket {
                effect: "GE_Player_Daffodill_Branch5_Damage",
                charge_milli: 984,
            },
            DirectPacket {
                effect: "GE_Player_Daffodill_Branch6_Damage",
                charge_milli: 594,
            },
            DirectPacket {
                effect: "GE_Player_Daffodill_Branch7_Damage",
                charge_milli: 594,
            },
            DirectPacket {
                effect: "GE_Player_Daffodill_EntryAttack1_Damage",
                charge_milli: 579,
            },
            DirectPacket {
                effect: "GE_Player_Daffodill_EntryAttack2_Damage",
                charge_milli: 470,
            },
            DirectPacket {
                effect: "GE_Player_Daffodill_EntryAttack4_Damage",
                charge_milli: 940,
            },
        ],
    },
    DirectOwner {
        character: "character:1070",
        ability: "GA_Mitsuki_Melee",
        source: "aurelia_hit",
        event_kind: "aurelia_direct_packet_energy_grant",
        energy_hypothesis_id: crate::simulation::energy::AURELIA_DIRECT_PACKET_ENERGY_HYPOTHESIS_ID
            .as_str(),
        energy_hypothesis: crate::simulation::energy::AURELIA_DIRECT_PACKET_ENERGY_HYPOTHESIS_ID
            .description(),
        input_hypothesis_id: crate::characters::aurelia::skill::DIRECT_PACKET_INPUTS_HYPOTHESIS_ID
            .as_str(),
        input_hypothesis: crate::characters::aurelia::skill::DIRECT_PACKET_INPUTS_HYPOTHESIS_ID
            .description(),
        packets: &[
            DirectPacket {
                effect: "GE_Player_Mitsuki_AirAttack_Damage",
                charge_milli: 6500,
            },
            DirectPacket {
                effect: "GE_Player_Mitsuki_EvadeBranch_Damage",
                charge_milli: 1300,
            },
        ],
    },
    DirectOwner {
        character: "character:1003",
        ability: "GA_Sagiri_Melee",
        source: "sakiri_hit",
        event_kind: "sakiri_direct_packet_energy_grant",
        energy_hypothesis_id: crate::simulation::energy::SAKIRI_DIRECT_PACKET_ENERGY_HYPOTHESIS_ID
            .as_str(),
        energy_hypothesis: crate::simulation::energy::SAKIRI_DIRECT_PACKET_ENERGY_HYPOTHESIS_ID
            .description(),
        input_hypothesis_id: crate::characters::sakiri::skill::DIRECT_PACKET_INPUTS_HYPOTHESIS_ID
            .as_str(),
        input_hypothesis: crate::characters::sakiri::skill::DIRECT_PACKET_INPUTS_HYPOTHESIS_ID
            .description(),
        packets: &[DirectPacket {
            effect: "GE_Player_Sagiri_AirAttack1_Damage",
            charge_milli: 2500,
        }],
    },
];

fn direct_action(
    owner: &DirectOwner,
    effect: &str,
    timestamp_us: u64,
    hit_delay_us: u64,
) -> GameActionCommand {
    GameActionCommand::InvokeAbility {
        timestamp_us,
        actor_index: 0,
        ability_id: owner.ability.into(),
        damage_effect_id: effect.into(),
        array_index: 0,
        atk_rate: Ratio::one(),
        scaling_stat: NativeScalingStat::Atk,
        energy_cost: 0,
        recovery_us: 0,
        cooldown_us: 0,
        hit_delay_us,
    }
}

fn team_request(owner: &DirectOwner) -> GameScenarioRequest {
    let mut req = sample_s1_request();
    let mut base = req.actors[0].clone();
    base.character_id = owner.character.into();
    base.weapon_id.clear();
    base.set_id = None;
    base.gear_procs.clear();
    base.talent_levels = Some([1; 3]);
    base.character_max_level = Some(80);
    base.character_level = 80;
    base.initial_energy = 0;
    base.energy_capacity = 100;
    base.static_stats.base_atk = 1000;
    base.static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    base.static_stats.elemental_bonus = Ratio::zero();
    req.actors = [owner.character, "character:1019", "character:1025"]
        .into_iter()
        .enumerate()
        .map(|(slot, character_id)| {
            let mut actor = base.clone();
            actor.slot = slot;
            actor.character_id = character_id.into();
            actor
        })
        .collect();
    req.targets[0].resistance = Ratio::zero();
    req.targets[0].level = 80;
    req.targets[0].hp_current = 100_000_000;
    req.targets[0].hp_max = 100_000_000;
    req
}

fn gcd(mut left: u64, mut right: u64) -> u64 {
    while right != 0 {
        (left, right) = (right, left % right);
    }
    left
}

fn reduced_ratio(numerator: u64, denominator: u64) -> serde_json::Value {
    let divisor = gcd(numerator, denominator);
    json!(format!("{}/{}", numerator / divisor, denominator / divisor))
}

fn expected_basis(charge_milli: u32) -> serde_json::Value {
    reduced_ratio(u64::from(charge_milli), 1000)
}

fn expected_recipient_energy(charges_milli: &[u32], recipient: usize) -> (u64, serde_json::Value) {
    // Frozen relation is 3/5 off-source. The three Charge Efficiency values are
    // 28/25, 3/2 and 7/5. A 1/50,000-energy integer scale preserves all inputs.
    let mut total = 0_u64;
    for charge in charges_milli {
        total += u64::from(*charge)
            * match recipient {
                0 => 56,
                1 => 45,
                2 => 42,
                _ => unreachable!(),
            };
    }
    (total / 50_000, reduced_ratio(total % 50_000, 50_000))
}

#[test]
fn exact_direct_inputs_use_owner_scoped_chargeadd_and_shared_team_settlement() {
    for owner in DIRECT_OWNERS {
        let mut req = team_request(owner);
        for (actor, (numerator, denominator)) in
            req.actors.iter_mut().zip([(28, 25), (3, 2), (7, 5)])
        {
            actor.gear_procs.push(GearProcConfig::ChargeEfficiency {
                multiplier: Ratio {
                    numerator,
                    denominator,
                },
            });
        }
        req.actions = owner
            .packets
            .iter()
            .enumerate()
            .map(|(index, packet)| direct_action(owner, packet.effect, index as u64 * 1000, 100))
            .collect();

        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(report["execution_completed"], true, "{}", owner.character);
        assert_eq!(
            report["hits_settled"],
            owner.packets.len(),
            "{}",
            owner.character
        );
        assert_eq!(
            report["hit_receipts"].as_array().unwrap().len(),
            owner.packets.len()
        );
        assert_eq!(
            report["resolved_model_assumptions"][owner.energy_hypothesis_id],
            owner.energy_hypothesis
        );
        assert_eq!(
            report["resolved_model_assumptions"][owner.input_hypothesis_id],
            owner.input_hypothesis
        );
        assert!(
            !report["resolved_model_assumptions"]
                .as_object()
                .unwrap()
                .contains_key(match owner.character {
                    "character:1054" =>
                        crate::simulation::energy::DAFFODIL_HIT_ENERGY_HYPOTHESIS_ID.as_str(),
                    "character:1070" =>
                        crate::simulation::energy::AURELIA_HIT_ENERGY_HYPOTHESIS_ID.as_str(),
                    _ => crate::simulation::energy::SAKIRI_HIT_ENERGY_HYPOTHESIS_ID.as_str(),
                })
        );

        let expected_hits = owner
            .packets
            .iter()
            .map(|packet| packet.effect)
            .collect::<Vec<_>>();
        assert_eq!(
            report["hit_receipts"]
                .as_array()
                .unwrap()
                .iter()
                .map(|hit| hit["damage_effect_id"].as_str().unwrap())
                .collect::<Vec<_>>(),
            expected_hits
        );

        let events = report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == owner.event_kind)
            .collect::<Vec<_>>();
        assert_eq!(events.len(), owner.packets.len(), "{}", owner.character);
        let charges = owner
            .packets
            .iter()
            .map(|packet| packet.charge_milli)
            .collect::<Vec<_>>();
        let mut energy_before = [0_u64; 3];
        let mut remainder = [0_u64; 3];
        for (packet_index, packet) in owner.packets.iter().enumerate() {
            let fanout = &events[packet_index]["energy_fanout"];
            assert_eq!(fanout["source"], owner.source);
            assert_eq!(fanout["source_actor_index"], 0);
            assert_eq!(fanout["base"], expected_basis(packet.charge_milli));
            let recipients = fanout["recipients"].as_array().unwrap();
            assert_eq!(recipients.len(), 3);
            for recipient in 0..3 {
                let entry = &recipients[recipient];
                assert_eq!(entry["recipient_actor_index"], recipient);
                assert_eq!(entry["energy_before"], energy_before[recipient]);
                let increment = u64::from(packet.charge_milli)
                    * match recipient {
                        0 => 56,
                        1 => 45,
                        2 => 42,
                        _ => unreachable!(),
                    };
                let gross = (remainder[recipient] + increment) / 50_000;
                remainder[recipient] = (remainder[recipient] + increment) % 50_000;
                energy_before[recipient] += gross;
                assert_eq!(entry["gross"], gross);
                assert_eq!(entry["retained"], gross);
                assert_eq!(entry["overflow"], 0);
                assert_eq!(entry["energy_after"], energy_before[recipient]);
                assert_eq!(
                    entry["distribution"],
                    if recipient == 0 {
                        json!("1/1")
                    } else {
                        json!("3/5")
                    }
                );
            }
        }
        for recipient in 0..3 {
            let expected = expected_recipient_energy(&charges, recipient);
            assert_eq!(
                report["final_actor_states"][recipient]["energy"],
                expected.0
            );
            assert_eq!(
                report["final_actor_states"][recipient]["granted_energy"],
                expected.0
            );
            assert_eq!(
                report["final_actor_states"][recipient]["charge_remainder"],
                expected.1
            );
        }
    }
}

#[test]
fn direct_packet_energy_records_and_validates_its_own_hypothesis() {
    for owner in DIRECT_OWNERS {
        let mut req = team_request(owner);
        for actor in &mut req.actors {
            actor.talent_levels = None;
        }
        req.actors[0].purchased_skill_level = 1;
        req.actions = vec![direct_action(owner, owner.packets[0].effect, 0, 100)];
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(report["hits_settled"], 1);
        assert!(report["resolved_model_assumptions"][owner.energy_hypothesis_id].is_string());

        req.model_assumptions.insert(
            owner.energy_hypothesis_id.into(),
            "conflicting description".into(),
        );
        let conflicting = run_game_scenario(&req, &ExecutionLimits::default()).unwrap_err();
        assert!(conflicting.contains("conflicting reached model hypothesis"));
    }
}

#[test]
fn direct_packet_energy_keeps_positive_loss_delivery_and_dead_recipient_gates() {
    for (owner, packet_index) in DIRECT_OWNERS.iter().zip([2, 0, 0]) {
        let mut fatal_req = team_request(owner);
        fatal_req.actions = vec![direct_action(
            owner,
            owner.packets[packet_index].effect,
            0,
            1000,
        )];
        fatal_req.actions.push(GameActionCommand::SwitchActor {
            timestamp_us: 50,
            target_actor_index: 1,
            cooldown_us: 0,
        });
        fatal_req.relative_duration_us = 2000;
        fatal_req.actors[0].initial_energy = 98;
        fatal_req.actors[1].initial_energy = 95;
        fatal_req.actors[2].initial_hp = 0;
        fatal_req.targets[0].hp_current = 1;
        fatal_req
            .actors
            .iter_mut()
            .for_each(|actor| actor.gear_procs.clear());
        let fatal = run_game_scenario(&fatal_req, &ExecutionLimits::default()).unwrap();
        assert_eq!(fatal["final_target_states"][0]["hp_current"], 0);
        assert_eq!(fatal["hit_receipts"][0]["applied_hp_loss"], 1);
        let grants = fatal["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == owner.event_kind)
            .collect::<Vec<_>>();
        assert_eq!(grants.len(), 1);
        assert_eq!(grants[0]["active_actor"], 1);
        let fanout = &grants[0]["energy_fanout"];
        assert_eq!(fanout["source_actor_index"], 0);
        let recipients = fanout["recipients"].as_array().unwrap();
        assert_eq!(recipients.len(), 2);
        assert_eq!(recipients[0]["recipient_actor_index"], 0);
        assert_eq!(recipients[1]["recipient_actor_index"], 1);

        let mut zero_req = team_request(owner);
        zero_req.actions = vec![direct_action(
            owner,
            owner.packets[packet_index].effect,
            0,
            100,
        )];
        zero_req.actors[0].static_stats.atk = Ratio::zero();
        let zero = run_game_scenario(&zero_req, &ExecutionLimits::default()).unwrap();
        assert_eq!(zero["hits_settled"], 1);
        assert!(
            zero["trace"]
                .as_array()
                .unwrap()
                .iter()
                .all(|event| event["event_kind"] != owner.event_kind)
        );
        assert!(
            !zero["resolved_model_assumptions"]
                .as_object()
                .unwrap()
                .contains_key(owner.energy_hypothesis_id)
        );
    }
}
