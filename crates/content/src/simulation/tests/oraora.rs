use super::*;
use serde_json::{Value, json};

const HYPOTHESIS_ID: &str = crate::weapons::oraora::HYPOTHESIS_ID.as_str();
const HYPOTHESIS: &str = crate::weapons::oraora::HYPOTHESIS_ID.description();

fn request() -> GameScenarioRequest {
    let mut request = sample_s1_request();
    request
        .model_assumptions
        .insert(HYPOTHESIS_ID.into(), HYPOTHESIS.into());
    request.model_assumptions.insert(
        crate::simulation::metadata::NORMAL_PHYSICAL_HYPOTHESIS_ID
            .as_str()
            .into(),
        crate::simulation::metadata::NORMAL_PHYSICAL_ID
            .description()
            .into(),
    );
    request.relative_duration_us = 25_000_000;
    request.actions.clear();
    request.action_metadata.clear();
    request.actors[0].character_id = "character:1010".into();
    request.actors[0].weapon_id = "weapon:fork_oulaquantao".into();
    request.actors[0].gear_procs = vec![GearProcConfig::OraoraBasicAttackDamage {
        damage_bonus_per_stack: Ratio {
            numerator: 1,
            denominator: 50,
        },
        duration_us: 10_000_000,
    }];
    request.actors[0].static_stats.atk = Ratio {
        numerator: 1000,
        denominator: 1,
    };
    request.actors[0].static_stats.base_atk = 1000;
    request.actors[0].static_stats.damage_bonus = Ratio::zero();
    request.actors[0].static_stats.elemental_bonus = Ratio::zero();
    request.targets[0].defense = 1080;
    request.targets[0].resistance = Ratio::zero();
    request
}

fn append(request: &mut GameScenarioRequest, time_us: u64, category: &str) {
    append_for(request, time_us, category, 0, 0);
}

fn append_for(
    request: &mut GameScenarioRequest,
    time_us: u64,
    category: &str,
    actor_index: usize,
    hit_delay_us: u64,
) {
    let index = request.actions.len();
    let ability_id = format!("oraora-test-ability-{index}");
    let damage_effect_id = format!("oraora-test-effect-{index}");
    request.actions.push(GameActionCommand::InvokeAbility {
        timestamp_us: time_us,
        actor_index,
        ability_id: ability_id.clone(),
        damage_effect_id: damage_effect_id.clone(),
        array_index: 0,
        atk_rate: Ratio::one(),
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost: 0,
        recovery_us: 0,
        cooldown_us: 0,
        hit_delay_us,
    });
    request.action_metadata.insert(
        index,
        crate::simulation::ResearchActionMetadata {
            hypothesis_id: "test.oraora_packet_v1".into(),
            ability_id,
            damage_effect_id,
            action_gameplay_tags: Some(vec!["Ability.Skill".into()]),
            input_action_role: Some(crate::simulation::metadata::InputActionRole::Other),
            damage_type_ex: Some("EExecutionDamageTypeEX::DAMAGE_TYPE_NORMAL".into()),
            damage_source_category: Some(format!(
                "EExecutionDamageSourceCategory::DAMAGE_SOURCE_CATEGORY_{category}"
            )),
            fixed_crit_rate: Some(crate::simulation::ResearchFixedCritRate::NoOverride),
            unbal_value: None,
            packet_ga_name_binding: None,
        },
    );
    request.model_assumptions.insert(
        "test.oraora_packet_v1".into(),
        "The synthetic packet isolates native delivery category, timestamps, and connected damage for weapon lifecycle checks.".into(),
    );
}

fn exact(value: &Value) -> ExactFraction {
    serde_json::from_value(value.clone()).unwrap()
}

fn run(request: &GameScenarioRequest) -> Value {
    let report = run_game_scenario(request, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{}",
        report["halt_diagnostic"]
    );
    report
}

#[test]
fn oraora_hypothesis_records_and_validates_conflicts() {
    let mut request = sample_s1_request();
    request.model_assumptions.clear();
    let mut resolved = Default::default();
    crate::runtime_api::record_hypothesis(&request, &mut resolved, HYPOTHESIS_ID, HYPOTHESIS)
        .unwrap();
    assert_eq!(resolved[HYPOTHESIS_ID], HYPOTHESIS);

    request
        .model_assumptions
        .insert(HYPOTHESIS_ID.into(), HYPOTHESIS.into());
    assert!(
        crate::runtime_api::record_hypothesis(
            &request,
            &mut resolved,
            HYPOTHESIS_ID,
            "changed lifecycle"
        )
        .unwrap_err()
        .contains("conflicting reached model hypothesis")
    );
}

#[test]
fn oraora_requires_delivered_category_a_and_expire_independently() {
    let mut equipped = request();
    append(&mut equipped, 0, "A");
    append(&mut equipped, 2_000_000, "A");
    append(&mut equipped, 9_999_999, "E");
    append(&mut equipped, 10_000_000, "A");
    append(&mut equipped, 12_000_000, "A");
    let mut baseline = equipped.clone();
    baseline.actors[0].gear_procs.clear();
    baseline.actors[0].weapon_id.clear();
    let report = run(&equipped);
    let base = run(&baseline);
    let hits = report["hit_receipts"].as_array().unwrap();
    let base_hits = base["hit_receipts"].as_array().unwrap();
    assert_eq!(hits.len(), 5);
    for (index, (hit, baseline_hit)) in hits.iter().zip(base_hits).enumerate() {
        let time = hit["time_us"].as_u64().unwrap();
        let category =
            super::support::hit_metadata(&report, hit)["packet"]["native_damage_source_category"]
                .as_str()
                .unwrap();
        let stacks = match (time, category.ends_with("_A")) {
            (0, _) => 0,
            (2_000_000, _) => 1,
            (9_999_999, _) => 2,
            (10_000_000, true) => 1,
            (12_000_000, true) => 1,
            _ => 0,
        };
        let expected = if category.ends_with("_A") {
            ExactFraction::new(50 + stacks, 50).unwrap()
        } else {
            ExactFraction::new(1, 1).unwrap()
        };
        let ratio = exact(&hit["non_crit_damage"])
            .div(exact(&baseline_hit["non_crit_damage"]))
            .unwrap();
        assert_eq!(ratio, expected, "hit {index} at {time}");
    }
    let callbacks = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|entry| entry["event_kind"] == "oraora_basic_attack_stack")
        .count();
    assert_eq!(callbacks, 4);
    assert_eq!(
        report["resolved_model_assumptions"][HYPOTHESIS_ID],
        HYPOTHESIS
    );
}

#[test]
fn oraora_caps_at_ten_and_cap_triggers_do_not_refresh_expiry() {
    let mut equipped = request();
    for _ in 0..11 {
        append(&mut equipped, 0, "A");
    }
    append(&mut equipped, 5_000_000, "A");
    append(&mut equipped, 10_000_000, "A");
    let report = run(&equipped);
    let hits = report["hit_receipts"].as_array().unwrap();
    for hit in &hits[..11] {
        assert_eq!(hit["non_crit_damage"], hits[0]["non_crit_damage"]);
    }
    let at_five = exact(&hits[11]["non_crit_damage"])
        .div(exact(&hits[0]["non_crit_damage"]))
        .unwrap();
    assert_eq!(at_five, ExactFraction::new(6, 5).unwrap());
    let at_expiry = exact(&hits[12]["non_crit_damage"])
        .div(exact(&hits[0]["non_crit_damage"]))
        .unwrap();
    assert_eq!(at_expiry, ExactFraction::new(1, 1).unwrap());
    let stack_events = report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|entry| entry["event_kind"] == "oraora_basic_attack_stack")
        .collect::<Vec<_>>();
    assert_eq!(stack_events.len(), 11);
    assert_eq!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|entry| entry["event_kind"] == "oraora_stack_capped")
            .count(),
        2
    );
    assert!(
        report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|entry| entry["event_kind"] == "oraora_stack_capped")
            .all(|entry| {
                entry["from_actor"] == 0
                    && entry["data"]["admitted"] == false
                    && entry["data"]["stacks"] == 10
                    && entry["data"].get("expires_at_us").is_none()
            })
    );
    let mut snapshot_request = request();
    snapshot_request.relative_duration_us = 5_000_000;
    for _ in 0..10 {
        append(&mut snapshot_request, 0, "A");
    }
    append(&mut snapshot_request, 4_999_999, "A");
    let snapshot = run(&snapshot_request);
    let instances = crate::simulation::tests::support::final_buffs(&snapshot);
    assert_eq!(instances.len(), 10);
    assert!(
        instances
            .iter()
            .all(|buff| buff["duration"]["expires_at_us"] == 10_000_000)
    );
}

#[test]
fn oraora_off_field_delivery_preserves_deadlines_with_live_zankou_team_buffs() {
    let mut equipped = request();
    let mut ally = equipped.actors[0].clone();
    ally.slot = 1;
    ally.character_id = "character:1036".into();
    ally.awakening = ntesim_data::AwakeningSelection::prefix(6);
    ally.weapon_id.clear();
    ally.gear_procs.clear();
    equipped.actors.push(ally);
    equipped.relative_duration_us = 6_000_000;
    append_for(&mut equipped, 0, "A", 0, 2_000_000);
    equipped.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 1_000_000,
        target_actor_index: 1,
        cooldown_us: 0,
    });
    append_for(&mut equipped, 2_500_000, "A", 1, 0);
    let ally_command_index = equipped.actions.len() - 1;
    equipped.action_metadata.remove(&ally_command_index);
    if let GameActionCommand::InvokeAbility {
        ability_id,
        damage_effect_id,
        ..
    } = &mut equipped.actions[ally_command_index]
    {
        *ability_id = "GA_Zankou_Melee".into();
        *damage_effect_id = "GE_Player_Zankou_Melee1_Damage".into();
    }
    equipped.actions.push(GameActionCommand::SwitchActor {
        timestamp_us: 3_600_000,
        target_actor_index: 0,
        cooldown_us: 0,
    });
    append_for(&mut equipped, 5_000_000, "A", 0, 0);
    let mut baseline = equipped.clone();
    baseline.actors[0].gear_procs.clear();
    baseline.actors[0].weapon_id.clear();
    let report = run(&equipped);
    let base = run(&baseline);
    let hits: Vec<_> = report["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| {
            hit["damage_effect_id"] != crate::characters::nanally::support::ENTRY.damage_effect_id
                && hit["damage_effect_id"]
                    != crate::simulation::reactions::hexed::FOLLOWUP_EFFECT_ID
        })
        .collect();
    let base_hits: Vec<_> = base["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|hit| {
            hit["damage_effect_id"] != crate::characters::nanally::support::ENTRY.damage_effect_id
                && hit["damage_effect_id"]
                    != crate::simulation::reactions::hexed::FOLLOWUP_EFFECT_ID
        })
        .collect();
    assert_eq!(hits.len(), 4);
    assert_eq!(hits[0]["actor_index"], 0);
    assert_eq!(hits[0]["time_us"], 2_000_000);
    assert_eq!(hits[1]["actor_index"], 1);
    assert_eq!(hits[3]["actor_index"], 0);
    assert_eq!(
        exact(&hits[3]["non_crit_damage"])
            .div(exact(&base_hits[3]["non_crit_damage"]))
            .unwrap(),
        ExactFraction::new(51, 50).unwrap()
    );
    assert_eq!(hits[1]["non_crit_damage"], base_hits[1]["non_crit_damage"]);
    assert_eq!(hits[2]["non_crit_damage"], base_hits[2]["non_crit_damage"]);
    let all_buffs = crate::simulation::tests::support::final_buffs(&report);
    assert!(
        all_buffs
            .iter()
            .any(|buff| buff["id"] == "zankou_delusion_dot_crit")
    );
    let instances = all_buffs
        .iter()
        .filter(|buff| buff["id"] == "oraora.basic_attack_damage")
        .collect::<Vec<_>>();
    assert_eq!(instances.len(), 2);
    assert!(instances.iter().all(|buff| buff["owner_actor"] == 0));
    let mut expiries = instances
        .iter()
        .map(|buff| buff["duration"]["expires_at_us"].as_u64().unwrap())
        .collect::<Vec<_>>();
    expiries.sort_unstable();
    assert_eq!(expiries, [12_000_000, 15_000_000]);
    assert_eq!(report["final_buff_state"]["time_us"], 6_000_000);
}

#[test]
fn oraora_zone_math_hand_derived_t1_t5() {
    fn gcd(mut a: u128, mut b: u128) -> u128 {
        while b != 0 {
            let rest = a % b;
            a = b;
            b = rest;
        }
        a
    }
    fn reduced(n: u128, d: u128) -> (u128, u128) {
        let g = gcd(n, d);
        (n / g, d / g)
    }
    // T1 per-stack 1/50: n stacks -> M_bonus = 1 + n/50 (additive, zones §1).
    // 10 stacks -> 60/50 = 6/5 = 1.2.
    assert_eq!(reduced(50 + 10, 50), (6, 5));
    // T5 per-stack 9/250: 10 stacks -> 1 + 90/250 = 340/250 = 34/25 = 1.36.
    assert_eq!(reduced(250 + 90, 250), (34, 25));
    // Multiplicative misreadings are rejected: (51/50)^10 != 6/5, (259/250)^10 != 34/25.
    assert_ne!(reduced(51u128.pow(10), 50u128.pow(10)), (6, 5));
    assert_ne!(reduced(259u128.pow(10), 250u128.pow(10)), (34, 25));
}

#[test]
fn oraora_does_not_stack_zero_damage_or_defeated_target_hits() {
    let mut zero = request();
    append(&mut zero, 0, "A");
    if let GameActionCommand::InvokeAbility { atk_rate, .. } = &mut zero.actions[0] {
        *atk_rate = Ratio::zero();
    }
    let zero_report = run(&zero);
    assert_eq!(zero_report["hit_receipts"][0]["damage"], json!("0/1"));
    assert!(
        !zero_report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["event_kind"] == "oraora_basic_attack_stack")
    );

    let mut defeated = request();
    defeated.targets[0].hp_current = 0;
    append(&mut defeated, 0, "A");
    let defeated_report = run(&defeated);
    assert!(
        !defeated_report["trace"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["event_kind"] == "oraora_basic_attack_stack")
    );
}
