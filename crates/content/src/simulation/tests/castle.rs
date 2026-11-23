use super::*;

fn fixture() -> GameScenarioRequest {
    let mut request = sample_s1_request();
    request.relative_duration_us = 15_000_000;
    request.actors[0].character_id = "character:1021".into();
    request.actors[0].weapon_id = "weapon:fork_Castle".into();
    request.actors[0].weapon_tier = 5;
    request.actors[0].set_id = None;
    request.actors[0].gear_procs = vec![GearProcConfig::CastleHealingBonus {
        healing_bonus: Ratio {
            numerator: 1,
            denominator: 5,
        },
        duration_us: 10_000_000,
    }];
    request.actors[0].initial_hp = 1_000;
    request.actors[0].initial_hp_max = 20_000;
    request.actors[0].static_stats.hp = 20_000;
    request.actors[0].talent_levels = Some([1, 1, 1]);
    request.actors[0].character_level = 80;
    request.actors[0].character_max_level = Some(80);
    request.actors[0].initial_energy = 100;
    request.actors[0].energy_capacity = 100;
    request.targets[0].hp_max = 10_000_000;
    request.targets[0].hp_current = 10_000_000;
    request.targets[0].defense = 1080;
    request.targets[0].resistance = Ratio::zero();
    request
}

fn ability(
    timestamp_us: u64,
    actor_index: usize,
    ability_id: &str,
    effect_id: &str,
) -> GameActionCommand {
    GameActionCommand::InvokeAbility {
        timestamp_us,
        actor_index,
        ability_id: ability_id.into(),
        damage_effect_id: effect_id.into(),
        array_index: 0,
        atk_rate: Ratio::one(),
        scaling_stat: crate::simulation::NativeScalingStat::Atk,
        energy_cost: 0,
        recovery_us: 0,
        cooldown_us: 0,
        hit_delay_us: 0,
    }
}

fn ticks(report: &serde_json::Value) -> Vec<&serde_json::Value> {
    report["trace"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|event| event["event_kind"] == "edgar_healing_tick")
        .collect()
}

#[test]
fn admitted_redirect_skill_applies_castle_bonus_to_edgar_healing_until_exact_expiry() {
    let mut request = fixture();
    request.actions = vec![
        ability(0, 0, "GA_Edgar_Skill", "GE_Player_Edgar_Skill_Damage"),
        ability(
            2_000_000,
            0,
            "GA_Edgar_UltraSkill",
            "GE_Player_Edgar_UltraSkill_Damage",
        ),
    ];
    let report =
        crate::simulation::run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    let heals = ticks(&report);
    assert_eq!(
        heals
            .iter()
            .map(|event| event["time_us"].as_u64().unwrap())
            .collect::<Vec<_>>(),
        (3..=13)
            .map(|second| second * 1_000_000 + 3_402_224)
            .collect::<Vec<_>>()
    );
    assert!(heals[..7].iter().all(|event| event["data"]["gross"] == 684));
    assert!(heals[7..].iter().all(|event| event["data"]["gross"] == 570));
    assert_eq!(
        report["resolved_model_assumptions"][crate::weapons::castle::HYPOTHESIS_ID.as_str()],
        crate::weapons::castle::HYPOTHESIS_ID.description()
    );
    assert_eq!(
        report["resolved_model_assumptions"]
            [crate::runtime_api::HEALING_MODIFIERS_HYPOTHESIS_ID.as_str()],
        crate::runtime_api::HEALING_MODIFIERS_HYPOTHESIS_ID.description()
    );
}

#[test]
fn castle_bonus_is_wearer_owned_when_edgar_heals_a_switched_in_recipient() {
    let mut request = fixture();
    let mut mint = request.actors[0].clone();
    mint.slot = 1;
    mint.character_id = "character:1019".into();
    mint.weapon_tier = 1;
    mint.gear_procs = vec![GearProcConfig::CastleHealingBonus {
        healing_bonus: Ratio {
            numerator: 3,
            denominator: 25,
        },
        duration_us: 10_000_000,
    }];
    mint.initial_hp = 1_000;
    request.actors.push(mint);
    request.actions = vec![
        ability(0, 0, "GA_Edgar_Skill", "GE_Player_Edgar_Skill_Damage"),
        GameActionCommand::SwitchActor {
            timestamp_us: 1_000_000,
            target_actor_index: 1,
            cooldown_us: 0,
        },
        ability(
            1_000_000,
            1,
            "GA_Mint019_Skill",
            "GE_Player_Mint_Skill1_Damage_New",
        ),
        GameActionCommand::SwitchActor {
            timestamp_us: 2_000_000,
            target_actor_index: 0,
            cooldown_us: 0,
        },
        ability(
            2_000_000,
            0,
            "GA_Edgar_UltraSkill",
            "GE_Player_Edgar_UltraSkill_Damage",
        ),
        GameActionCommand::SwitchActor {
            timestamp_us: 2_500_000,
            target_actor_index: 1,
            cooldown_us: 0,
        },
    ];
    let report =
        crate::simulation::run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    let heals = ticks(&report);
    assert_eq!(heals.first().unwrap()["active_actor"], 1);
    assert_eq!(heals[0]["data"]["gross"], 684);
    assert_eq!(heals[7]["data"]["gross"], 570);
    assert_eq!(report["final_actor_states"][1]["hp"], 8_068);
}

#[test]
fn castle_redirect_recast_resets_one_instance_and_failed_or_other_roles_do_not_grant() {
    let mut request = fixture();
    request.actions = vec![
        ability(0, 0, "GA_Edgar_Skill", "GE_Player_Edgar_Skill_Damage"),
        ability(
            1_000_000,
            0,
            "GA_Edgar_UltraSkill",
            "GE_Player_Edgar_UltraSkill_Damage",
        ),
        ability(
            13_402_224,
            0,
            "GA_Edgar_Skill",
            "GE_Player_Edgar_Skill_Damage",
        ),
    ];
    let report =
        crate::simulation::run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
    let heals = ticks(&report);
    let refreshed_tick = heals
        .iter()
        .find(|event| event["time_us"] == 13_402_224)
        .expect("the exact-deadline healing tick must be present");
    assert_eq!(refreshed_tick["data"]["gross"], 684);
    let buffs = crate::simulation::tests::support::final_buffs(&report);
    let castle_buffs: Vec<_> = buffs
        .iter()
        .filter(|buff| buff["id"] == "call_of_the_twisted_city.healing_bonus")
        .collect();
    assert_eq!(castle_buffs.len(), 1);
    assert_eq!(castle_buffs[0]["stacks"], 1);
    assert_eq!(castle_buffs[0]["source"], "weapon:fork_Castle");
    assert_eq!(castle_buffs[0]["duration"]["expires_at_us"], 20_000_000);

    let mut non_skill = fixture();
    non_skill.actions = vec![
        ability(0, 0, "GA_Edgar_QTE", "GE_Player_Edgar_QTE_Damage"),
        ability(
            1_000_000,
            0,
            "GA_Edgar_UltraSkill",
            "GE_Player_Edgar_UltraSkill_Damage",
        ),
    ];
    let report =
        crate::simulation::run_game_scenario(&non_skill, &ExecutionLimits::default()).unwrap();
    assert!(
        !crate::simulation::tests::support::final_buffs(&report)
            .iter()
            .any(|buff| buff["id"] == "call_of_the_twisted_city.healing_bonus")
    );
    assert_eq!(ticks(&report)[0]["data"]["gross"], 570);

    let mut failed = fixture();
    failed.actors[0].initial_energy = 0;
    failed.actions = vec![ability(
        0,
        0,
        "GA_Edgar_Skill",
        "GE_Player_Edgar_Skill_Damage",
    )];
    if let GameActionCommand::InvokeAbility { energy_cost, .. } = &mut failed.actions[0] {
        *energy_cost = 100;
    }
    let report =
        crate::simulation::run_game_scenario(&failed, &ExecutionLimits::default()).unwrap();
    assert_eq!(report["execution_completed"], false);
    assert!(
        !crate::simulation::tests::support::final_buffs(&report)
            .iter()
            .any(|buff| buff["id"] == "call_of_the_twisted_city.healing_bonus")
    );
}

#[test]
fn model_registry_accepts_castle_and_healing_hypotheses_and_rejects_conflicts() {
    let mut request = fixture();
    let mut reached = std::collections::BTreeMap::new();
    for (id, description) in [
        (
            crate::weapons::castle::HYPOTHESIS_ID,
            crate::weapons::castle::HYPOTHESIS_ID.description(),
        ),
        (
            crate::runtime_api::HEALING_MODIFIERS_HYPOTHESIS_ID,
            crate::runtime_api::HEALING_MODIFIERS_HYPOTHESIS_ID.description(),
        ),
    ] {
        crate::runtime_api::record_hypothesis(&request, &mut reached, id, description).unwrap();
        assert_eq!(reached[id.as_str()], description);
    }
    request.model_assumptions.insert(
        crate::weapons::castle::HYPOTHESIS_ID.into(),
        "different trigger or refresh model".into(),
    );
    let error = crate::runtime_api::record_hypothesis(
        &request,
        &mut reached,
        crate::weapons::castle::HYPOTHESIS_ID,
        crate::weapons::castle::HYPOTHESIS_ID.description(),
    )
    .unwrap_err();
    assert!(error.contains("conflicting reached model hypothesis"));
}

fn projection_tables() -> std::collections::BTreeMap<String, serde_json::Value> {
    super::support::weapon_projection_tables()
}

fn projection_actor() -> GameActorConfig {
    serde_json::from_value(serde_json::json!({
        "slot":0,"character_id":"character:1021","initial_hp":1,"initial_hp_max":1,"initial_energy":0,
        "energy_capacity":100,"weapon_id":"weapon:fork_Castle","weapon_tier":1,
        "static_stats":{"base_atk":1,"atk":"1/1","hp":1,"def":1,
        "crit_rate":"0/1","crit_damage_bonus":"0/1"}
    }))
    .unwrap()
}

#[test]
fn castle_projection_binds_five_refinement_tiers() {
    // Hand-derived from frozen CT_ForkBuff rows (independent of the implementation):
    // buff_Castle_HealUp T1..T5 = 0.12/0.14/0.16/0.18/0.20 = 3/25, 7/50, 4/25, 9/50, 1/5;
    // buff_Castle_CD = 10.0s.
    let tables = projection_tables();
    let expected_bonus = [(3, 25), (7, 50), (4, 25), (9, 50), (1, 5)];
    let mut actor = projection_actor();
    for (tier, (num, den)) in (1..=5).zip(expected_bonus) {
        actor.weapon_tier = tier as u32;
        actor.gear_procs.clear();
        let report = crate::weapons::castle::projection::project(
            &tables,
            1,
            "ntesim-current",
            &mut actor,
            &[],
        )
        .unwrap();
        let proc = &report["compiled_proc"];
        assert_eq!(
            proc["healing_bonus"],
            serde_json::json!(format!("{}/{}", num, den))
        );
        assert_eq!(proc["duration_us"], 10_000_000);
        assert_eq!(report["native_parameter_report"]["parameter_count"], 2);
        // The flag was flipped false -> true by the coordinator (batch 14): the grant runs
        // through the real gear::on_cast dispatch, the buff executes, and healing settlement
        // consumes the bonus (570 -> 684) via the live_healing_modifiers_v1 hypothesis path.
        assert_eq!(report["runtime_execution_validated"], true);
        assert_eq!(actor.gear_procs.len(), 1);
    }
    // Group TWO enforcement: character:1003 is CHARACTER_GROUP_TYPE_FOUR.
    actor.character_id = "character:1003".into();
    assert!(
        crate::weapons::castle::projection::project(&tables, 1, "ntesim-current", &mut actor, &[])
            .unwrap_err()
            .contains("group is incompatible")
    );
}

#[test]
fn castle_zone_math_hand_derived_t1_t5() {
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
    // Healing-zone multipliers 1 + bonus for T1..T5, hand-derived exact fractions,
    // no core calls. Frozen buff_Castle_HealUp: 0.12/0.14/0.16/0.18/0.20.
    // Gross healing against the 570 base (nearest integer, ties up, contract section 6):
    // 570 x 28/25 = 638.4 -> 638; 570 x 57/50 = 649.8 -> 650; 570 x 29/25 = 661.2 -> 661;
    // 570 x 59/50 = 672.6 -> 673; 570 x 6/5 = 684.
    let cases: [((u128, u128), u128); 5] = [
        ((28, 25), 638),
        ((57, 50), 650),
        ((29, 25), 661),
        ((59, 50), 673),
        ((6, 5), 684),
    ];
    let bonuses = [(3, 25), (7, 50), (4, 25), (9, 50), (1, 5)];
    for (i, ((en, ed), gross)) in cases.iter().enumerate() {
        let (n, d) = (bonuses[i].0 as u128, bonuses[i].1 as u128);
        assert_eq!(reduced(d + n, d), (*en, *ed));
        let exact = 570.0 * (d + n) as f64 / d as f64;
        assert_eq!(exact.round() as u128, *gross);
    }
    // Tier-confusion falsifier: each tier's multiplier is distinct from its neighbors,
    // guarding against shifted refinement rows (T4 is 59/50, not T2's 57/50).
    assert_ne!(reduced(50 + 9, 50), (57, 50));
    assert_ne!(reduced(50 + 9, 50), (6, 5));
}
