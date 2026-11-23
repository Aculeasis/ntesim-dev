use super::nanally_actions::{make_action, request};
use super::*;
use crate::simulation::tests::support::formula_damage_total;
use serde_json::{Value, json};

fn scenario(effects: &[u8]) -> GameScenarioRequest {
    let mut req = request();
    req.actors[0].awakening =
        ntesim_data::AwakeningSelection::selected(effects.iter().copied()).unwrap();
    req.actors[0].talent_levels = Some([1, 1, 8]);
    req.actors[0].initial_energy = 100;
    req.targets[0].hp_current = 10_000_000;
    req.targets[0].hp_max = 10_000_000;
    req.targets[0].defense = 1080;
    req.relative_duration_us = 20_000_000;
    req.actions = vec![make_action(
        0,
        0,
        "GA_Nanally_UltraSkill",
        "GE_Player_Nanally_UltraSkill1_Damage",
        100,
        0,
    )];
    req
}

// Shift authored relative Normal inputs to legal post-Burst recovery.
const AFTER_BURST: u64 = 3_482_140;
fn normal(at: u64) -> GameActionCommand {
    make_action(
        AFTER_BURST + at,
        0,
        "GA_Nanally_Melee",
        "GE_Player_Nanally_Melee1_Damage",
        0,
        0,
    )
}

fn children(report: &Value) -> Vec<&Value> {
    report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| {
            hit["damage_effect_id"]
                .as_str()
                .is_some_and(crate::characters::nanally::underboss::is_coordinated_packet)
        })
        .collect()
}

fn air_attacks(report: &Value) -> Vec<&Value> {
    report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| hit["damage_effect_id"] == "GE_Player_Nanally_Cat_AirAttack_Damage")
        .collect()
}

#[test]
fn burst_underboss_has_seven_direct_hits_and_no_summon_or_burst_coordination_damage() {
    for effects in [&[][..], &[4][..]] {
        let report = run_game_scenario(&scenario(effects), &ExecutionLimits::default()).unwrap();
        assert_eq!(report["hits_settled"], 7);
        assert!(air_attacks(&report).is_empty());
        // Level8 direct Q sum17.138, ATK1000, mitigation1/2; A4 affects Cat only.
        assert_eq!(
            serde_json::json!(formula_damage_total(&report)),
            json!("8569/1")
        );
        assert_eq!(report["final_actor_states"][0]["spent_energy"], 100);
        assert_eq!(report["final_actor_states"][0]["granted_energy"], 0);
    }
}

#[test]
fn burst_underboss_direct_damage_does_not_create_cat_at_expiry_or_from_a3() {
    let mut req = scenario(&[]);
    if let GameActionCommand::InvokeAbility { hit_delay_us, .. } = &mut req.actions[0] {
        *hit_delay_us = 6_000_000;
    }
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["hits_settled"], 7);
    assert!(air_attacks(&report).is_empty());
    req.targets[0].hp_current = 1;
    let lethal = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(lethal["hits_settled"], 1);
    assert!(air_attacks(&lethal).is_empty());

    let mut req = scenario(&[3]);
    req.actions.insert(
        0,
        make_action(
            0,
            0,
            "GA_Nanally_Skill",
            "GE_Player_Nanally_Skill1_Damage",
            0,
            0,
        ),
    );
    if let GameActionCommand::InvokeAbility { timestamp_us, .. } = &mut req.actions[1] {
        *timestamp_us = 2_000_000;
    }
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["hits_settled"], 14); // 5E +7Q +one A3 for each initiator.
    assert!(air_attacks(&report).is_empty());
}

#[test]
fn burst_underboss_creates_no_cat_damage_on_zero_loss_or_repeated_summoning() {
    let mut req = scenario(&[]);
    req.targets[0].defense = 1_000_000_000;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["hits_settled"], 7);
    assert!(air_attacks(&report).is_empty());
    let mut req = scenario(&[]);
    req.actions.push(make_action(
        19_584_608,
        0,
        "GA_Nanally_UltraSkill",
        "GE_Player_Nanally_UltraSkill1_Damage",
        0,
        0,
    ));
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["hits_settled"], 14);
    assert!(air_attacks(&report).is_empty());
}

#[test]
fn underboss_time_budget_does_not_impose_an_inferred_eight_group_counter() {
    let mut req = scenario(&[2]);
    for at in [
        0, 611_861, 1_482_102, 2_693_850, 4_025_363, 5_337_783, 5_949_644, 6_819_885, 8_031_633,
    ] {
        req.actions.push(normal(100_000 + at));
    }
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    // Nine legal Normal inputs in the A2 nine-second window:26 direct,16 Cat,7 Q.
    assert_eq!(children(&report).len(), 16);
    assert_eq!(report["hits_settled"], 49);
    assert!(air_attacks(&report).is_empty());
}

#[test]
fn underboss_full_normal_chain_and_skill_capture_burst_talent_without_team_charge() {
    let mut req = scenario(&[]);
    for stage in 0..5 {
        req.actions.push(normal(
            100_000 + super::nanally_clock::STARTS[stage as usize],
        ));
    }
    req.actions.push(make_action(
        8_920_000,
        0,
        "GA_Nanally_Skill",
        "GE_Player_Nanally_Skill1_Damage",
        0,
        0,
    ));
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    let hits = children(&report);
    // Independently transcribed level-8 published Underboss ratio groups:
    // .428; .386; .728+.686; .543*3; .771+1.028; .411*5.
    let coefficients = [
        428, 386, 728, 686, 543, 543, 543, 771, 1028, 411, 411, 411, 411, 411,
    ];
    assert_eq!(hits.len(), coefficients.len());
    for (hit, coefficient) in hits.iter().zip(coefficients) {
        assert_eq!(
            hit["non_crit_damage"],
            serde_json::to_value(ExactFraction::new(coefficient, 2).unwrap()).unwrap()
        );
        let meta = super::support::hit_metadata(&report, hit);
        assert_eq!(meta["binding"]["kind"], "owner_child");
        assert_eq!(meta["binding"]["packet_ga_name"], "GA_Nanally_UltraSkill");
        assert_eq!(
            meta["packet"]["native_damage_source_category"],
            "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_A"
        );
    }
    assert!(
        hits[..9]
            .iter()
            .all(|hit| hit["ability_id"] == "GA_Nanally_Melee")
    );
    assert!(
        hits[9..]
            .iter()
            .all(|hit| hit["ability_id"] == "GA_Nanally_Skill")
    );
    assert_eq!(report["hits_settled"], 40); // Burst7, then14 Normal +5 Skill +14 later Cat packets.
    assert_eq!(report["final_actor_states"][0]["spent_energy"], 100);
    // Native ordinary Normal chain 16.599 plus ordinary Skill 6.3; Cat ChargeAdd=0.
    assert_eq!(report["final_actor_states"][0]["granted_energy"], 22);
    assert_eq!(
        report["final_actor_states"][0]["charge_remainder"],
        json!("899/1000")
    );
    assert_eq!(
        run_game_scenario(&req, &ExecutionLimits::default()).unwrap(),
        report
    );
}

#[test]
fn underboss_a4_child_zone_a1_icd_carry_and_a5_group_stacks() {
    let mut base = scenario(&[1, 2, 5]);
    base.actors[0]
        .gear_procs
        .push(GearProcConfig::ChargeEfficiency {
            multiplier: Ratio {
                numerator: 7,
                denominator: 5,
            },
        });
    base.actors[0].static_stats.damage_bonus = Ratio {
        numerator: 1,
        denominator: 2,
    };
    base.actions.extend([
        normal(1_000_000),
        normal(2_000_000),
        normal(3_000_000),
        normal(4_500_000),
        normal(6_000_000),
    ]);
    let baseline = run_game_scenario(&base, &ExecutionLimits::default()).unwrap();
    let mut selected = base.clone();
    // One selected A4 leaves the captured Burst coordinate at level 8.
    selected.actors[0].awakening = ntesim_data::AwakeningSelection::selected([4]).unwrap();
    let a4 = run_game_scenario(&selected, &ExecutionLimits::default()).unwrap();
    assert_eq!(children(&a4)[0]["non_crit_damage"], json!("535/1")); // 1000*.428*.5*(1+.5+1).
    let grants = baseline["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["event_kind"] == "nanally_a1_energy_grant")
        .collect::<Vec<_>>();
    assert_eq!(
        grants
            .iter()
            .map(|event| event["time_us"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        [4_673_194, 5_691_163, 6_715_066, 8_331_618, 9_731_502]
    );
    // 16.599 ordinary +5*2.5 self-only, all at wearer efficiency1.4 =40.7386.
    assert_eq!(baseline["final_actor_states"][0]["granted_energy"], 40);
    assert_eq!(
        baseline["final_actor_states"][0]["charge_remainder"],
        json!("3693/5000")
    );
    assert_eq!(
        baseline["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "nanally_a5_atk_buff")
            .count(),
        5
    ); // Five performed Normal follow-up groups; no summon callback.
}

#[test]
fn underboss_half_open_expiry_selected_a2_and_native_markers_ignore_caller_delay() {
    for (effects, at, expected) in [
        (&[][..], 5_911_414, 0),
        (&[2][..], 5_911_414, 1),
        (&[2][..], 8_911_414, 0),
    ] {
        let mut req = scenario(effects);
        req.actions.push(normal(at));
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(children(&report).len(), expected, "at={at} {report}");
    }
    let mut req = scenario(&[]);
    let mut delayed = normal(1_000_000);
    if let GameActionCommand::InvokeAbility { hit_delay_us, .. } = &mut delayed {
        *hit_delay_us = 2_000_000;
    }
    req.actions.push(delayed);
    req.actions.push(make_action(
        19_584_608,
        0,
        "GA_Nanally_UltraSkill",
        "GE_Player_Nanally_UltraSkill1_Damage",
        0,
        0,
    ));
    req.actions.push(normal(19_782_140));
    req.relative_duration_us = 30_000_000;
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(children(&report).len(), 2);
    assert_eq!(children(&report)[0]["time_us"], 4_673_194);
    assert_eq!(children(&report)[1]["time_us"], 23_455_334);
}

#[test]
fn underboss_launch_precedes_legal_switch_and_death_cancels_unissued_groups() {
    for defeated in [false, true] {
        let mut req = scenario(&[1, 5]);
        let mut ally = req.actors[0].clone();
        ally.slot = 1;
        ally.character_id = "character:1019".into();
        ally.awakening = ntesim_data::AwakeningSelection::default();
        req.actors.push(ally);
        req.actions.push(normal(1_000_000));
        if !defeated {
            req.actions.push(GameActionCommand::SwitchActor {
                timestamp_us: 5_182_140,
                target_actor_index: 1,
                cooldown_us: 0,
            });
        }
        if defeated {
            req.incoming_damage_events.push(GameActorDamageEvent {
                timestamp_us: 900_050,
                actor_index: 0,
                delivered_damage: req.actors[0].initial_hp,
            });
            req.model_assumptions.insert(
                super::super::events::INCOMING_DAMAGE_HYPOTHESIS_ID.into(),
                super::super::events::INCOMING_DAMAGE_HYPOTHESIS_ID
                    .description()
                    .into(),
            );
        }
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(children(&report).len(), usize::from(!defeated));
        assert_eq!(report["final_actor_states"][1]["energy"], 100); // A1 never redirects to the active ally.
        let has_a5 = crate::simulation::tests::support::final_buffs(&report)
            .iter()
            .any(|buff| buff["id"] == "Buff_Nanally010_Level5");
        assert!(!has_a5); // field exit/death clear the owned buff; death cancels pending groups.
    }
    let mut req = scenario(&[1, 5]);
    req.targets[0].hp_current = 1;
    req.actions.push(normal(1_000_000));
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert!(children(&report).is_empty());
    assert_eq!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "nanally_a5_atk_buff")
            .count(),
        0
    ); // Pending off-field group cannot grant A5; summon creation grants none.
}

#[test]
fn underboss_energy_finances_a_second_burst_with_efficiency_overflow_and_ally_carry() {
    let mut req = scenario(&[1]);
    req.actors[0]
        .gear_procs
        .push(GearProcConfig::ChargeEfficiency {
            multiplier: Ratio {
                numerator: 2,
                denominator: 1,
            },
        });
    let mut ally = req.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1019".into();
    ally.awakening = ntesim_data::AwakeningSelection::default();
    ally.gear_procs.clear();
    ally.initial_energy = 0;
    req.actors.push(ally);
    req.relative_duration_us = 25_000_000;
    // Five A1-eligible launches before the first six-second summon expires.
    req.actions
        .extend([100_000, 1_100_000, 2_100_000, 3_400_000, 4_800_000].map(normal));
    for offset in super::nanally_clock::STARTS.into_iter().take(5) {
        req.actions.push(normal(6_200_000 + offset));
    }
    req.actions.push(make_action(
        15_100_000,
        0,
        "GA_Nanally_Skill",
        "GE_Player_Nanally_Skill1_Damage",
        0,
        0,
    ));
    req.actions.push(make_action(
        20_584_608,
        0,
        "GA_Nanally_UltraSkill",
        "GE_Player_Nanally_UltraSkill1_Damage",
        100,
        0,
    ));
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{}",
        report["halt_diagnostic"]
    );
    let actor = &report["final_actor_states"][0];
    // (2*16.599 +6.3)*2 +five eligible groups*2.5*2 =103.996.
    // Cap100, overflow3, carry.996; second Burst spends100 with no free refund.
    assert_eq!(actor["spent_energy"], 200);
    assert_eq!(actor["energy"], 0);
    assert_eq!(actor["overflow_energy"], 3);
    assert_eq!(actor["charge_remainder"], json!("249/250"));
    // Ally receives only ordinary native team distribution: 39.498*.6 = 23.6988.
    assert_eq!(report["final_actor_states"][1]["energy"], 23);
    assert_eq!(
        report["final_actor_states"][1]["charge_remainder"],
        json!("1747/2500")
    );
}

#[test]
fn underboss_a5_expiry_restarts_the_group_stack_pool_for_the_next_summon() {
    // First group at4.673194 expires at19.673194; the next stage1 marker straddles that deadline.
    for (input, prior_stacks) in [(19_584_607, 1u128), (19_584_608, 0), (20_584_608, 0)] {
        let mut req = scenario(&[5]);
        req.relative_duration_us = 25_000_000;
        req.actions.extend([
            normal(1_000_000),
            make_action(
                18_584_608,
                0,
                "GA_Nanally_UltraSkill",
                "GE_Player_Nanally_UltraSkill1_Damage",
                0,
                0,
            ),
            normal(input),
            normal(input + 1_000_000),
        ]);
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        let stacks = report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|event| event["event_kind"] == "nanally_a5_atk_buff")
            .map(|event| event["data"]["stacks"].as_u64().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        assert_eq!(stacks.len(), 3);
        assert_eq!(stacks[0], 1);
        assert_eq!(stacks[1], u64::try_from(prior_stacks + 1).unwrap());
        assert_eq!(stacks[2], u64::try_from(prior_stacks + 2).unwrap());
        let hits = children(&report);
        assert_eq!(hits.len(), 3);
        assert_eq!(hits[0]["non_crit_damage"], json!("214/1"));
        // Current group samples preceding live ATK, then grants one new stack.
        assert_eq!(
            hits[1]["non_crit_damage"],
            serde_json::to_value(ExactFraction::new(214 * (50 + prior_stacks), 50).unwrap())
                .unwrap()
        );
        assert_eq!(
            hits[2]["non_crit_damage"],
            serde_json::to_value(ExactFraction::new(193 * (51 + prior_stacks), 50).unwrap())
                .unwrap()
        );
    }
}
