use super::edgar_actions::{make_action, request, set_a5_vitals};
use super::*;
use serde_json::json;

fn make_adler_actor(base: &GameActorConfig) -> GameActorConfig {
    let mut adler = base.clone();
    adler.slot = 1;
    adler.character_id = "character:1033".into();
    adler.awakening = ntesim_data::AwakeningSelection::prefix(0);
    adler.static_stats.base_def = Some(1000);
    adler.static_stats.def = 1240;
    adler.static_stats.def_percent = Ratio {
        numerator: 1,
        denominator: 5,
    };
    adler.static_stats.def_flat = Ratio {
        numerator: 40,
        denominator: 1,
    };
    adler.static_stats.atk = Ratio {
        numerator: 700,
        denominator: 1,
    };
    adler
}

#[test]
fn edgar_a6_def_window_runs_native_adler_skill_and_burst_damage() {
    let mut req = request();
    req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(6);
    set_a5_vitals(&mut req.actors[0], 13_000);
    req.action_metadata.clear();
    req.actors.push(make_adler_actor(&req.actors[0]));
    req.targets[0].defense = 0;
    req.targets[0].resistance = Ratio::zero();
    let add_hyp = |m: &mut BTreeMap<String, String>, id: crate::HypothesisId| {
        m.insert(id.as_str().into(), id.description().into());
    };
    add_hyp(
        &mut req.model_assumptions,
        crate::simulation::metadata::FIXED_CRIT_PROBABILITY_HYPOTHESIS_ID,
    );
    add_hyp(
        &mut req.model_assumptions,
        crate::characters::packet_semantics::ADLER_SKILL_DOT_PACKET_HYPOTHESIS_ID,
    );
    req.relative_duration_us = 11_100_000;
    let adler_action = |timestamp_us, ability_id: &str, damage_effect_id: &str, rate| {
        GameActionCommand::InvokeAbility {
            timestamp_us,
            actor_index: 1,
            ability_id: ability_id.into(),
            damage_effect_id: damage_effect_id.into(),
            array_index: 0,
            atk_rate: rate,
            scaling_stat: crate::simulation::NativeScalingStat::Def,
            energy_cost: 0,
            recovery_us: 0,
            cooldown_us: 0,
            hit_delay_us: 100_000,
        }
    };
    req.actions = vec![
        GameActionCommand::SwitchActor {
            timestamp_us: 0,
            target_actor_index: 1,
            cooldown_us: 0,
        },
        adler_action(
            200_000,
            "GA_Adler_Skill",
            "GE_Player_Adler_Skill2_Damage",
            Ratio {
                numerator: 1,
                denominator: 5,
            },
        ),
        GameActionCommand::SwitchActor {
            timestamp_us: 500_000,
            target_actor_index: 0,
            cooldown_us: 0,
        },
        make_action(
            1_000_000,
            0,
            "GA_Edgar_UltraSkill",
            "GE_Player_Edgar_UltraSkill_Damage",
            100,
            20_000_000,
        ),
        GameActionCommand::SwitchActor {
            timestamp_us: 1_500_000,
            target_actor_index: 1,
            cooldown_us: 0,
        },
        adler_action(
            2_000_000,
            "GA_Adler_Skill",
            "GE_Player_Adler_Skill2_Damage",
            Ratio {
                numerator: 1,
                denominator: 5,
            },
        ),
        adler_action(
            11_500_000,
            "GA_Adler_UltraSkill",
            "GE_Player_Adler_UltraSkill_Damage",
            Ratio::one(),
        ),
    ];
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    let hits = report["hit_receipts"].as_array().unwrap();
    let adler_hits = hits
        .iter()
        .filter(|hit| hit["actor_index"] == 1)
        .collect::<Vec<_>>();
    assert_eq!(adler_hits.len(), 26);
    assert_eq!(adler_hits[0]["time_us"], 300_000);
    assert_eq!(adler_hits[0]["rounded_damage"], 868);
    let second_skill = adler_hits
        .iter()
        .find(|hit| hit["time_us"] == 2_100_000 && hit["ability_id"] == "GA_Adler_Skill")
        .unwrap();
    assert_eq!(second_skill["rounded_damage"], 973);
    let burst_hit = adler_hits
        .iter()
        .find(|hit| hit["time_us"] == 11_600_000 && hit["ability_id"] == "GA_Adler_UltraSkill")
        .unwrap();
    assert_eq!(burst_hit["rounded_damage"], 1240);
    assert_eq!(
        report["resolved_model_assumptions"]
            [crate::characters::edgar::skill::A6_DEF_HYPOTHESIS_ID.as_str()],
        crate::characters::edgar::skill::A6_DEF_HYPOTHESIS_ID.description()
    );
    assert_eq!(report["final_buff_state"]["instances"], json!([]));

    let mut active_window = req.clone();
    active_window.relative_duration_us = 1_500_000;
    active_window.actions.truncate(4);
    let active_window = run_game_scenario(&active_window, &ExecutionLimits::default()).unwrap();
    let active_window_buffs = crate::simulation::tests::support::final_buffs(&active_window);
    let buff = &active_window_buffs[0];
    assert_eq!(buff["id"], "edgar.a6_wake_team_def");
    assert_eq!(buff["owner_actor"], 0);
    assert_eq!(buff["recipient"], json!({"type":"team_all"}));
    assert_eq!(buff["source_lifetime"], "while_alive");
    assert_eq!(buff["stacking_scope"], "shared");
    assert_eq!(buff["stacks"], 1);
    assert_eq!(buff["duration"]["expires_at_us"], 11_000_000);
    assert_eq!(buff["stat_modifiers"]["def_percent"], json!("3/20"));

    let mut defeated_source = req.clone();
    defeated_source.relative_duration_us = 3_100_000;
    defeated_source.actions.truncate(6);
    defeated_source.actions.push(adler_action(
        2_800_000,
        "GA_Adler_Skill",
        "GE_Player_Adler_Skill2_Damage",
        Ratio {
            numerator: 1,
            denominator: 5,
        },
    ));
    defeated_source.incoming_damage_events = vec![crate::simulation::GameActorDamageEvent {
        timestamp_us: 2_000_000,
        actor_index: 0,
        delivered_damage: 20_000,
    }];
    add_hyp(
        &mut defeated_source.model_assumptions,
        crate::simulation::events::INCOMING_DAMAGE_HYPOTHESIS_ID,
    );
    let defeated_source = run_game_scenario(&defeated_source, &ExecutionLimits::default()).unwrap();
    let dead_source_skill = defeated_source["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|hit| hit["actor_index"] == 1 && hit["time_us"] == 2_900_000)
        .unwrap();
    assert_eq!(dead_source_skill["rounded_damage"], 868);
    let living_source_skill = defeated_source["hit_receipts"]
        .as_array()
        .unwrap()
        .iter()
        .find(|hit| hit["actor_index"] == 1 && hit["time_us"] == 2_100_000)
        .unwrap();
    assert_eq!(living_source_skill["rounded_damage"], 973);
    assert_eq!(defeated_source["final_actor_states"][0]["hp"], 0);
    let defeated_source_buffs = crate::simulation::tests::support::final_buffs(&defeated_source);
    let retained_buff = &defeated_source_buffs[0];
    assert_eq!(retained_buff["owner_actor"], 0);
    assert_eq!(retained_buff["source_lifetime"], "while_alive");
}

#[test]
fn edgar_a6_def_recasts_replace_sources_and_failed_admission_grants_none() {
    let mut recast = request();
    recast.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(6);
    set_a5_vitals(&mut recast.actors[0], 13_000);
    recast.relative_duration_us = 11_597_775;
    recast.actions = vec![
        make_action(
            0,
            0,
            "GA_Edgar_UltraSkill",
            "GE_Player_Edgar_UltraSkill_Damage",
            0,
            0,
        ),
        make_action(
            5_000_000,
            0,
            "GA_Edgar_UltraSkill",
            "GE_Player_Edgar_UltraSkill_Damage",
            0,
            0,
        ),
    ];
    let recast = run_game_scenario(&recast, &ExecutionLimits::default()).unwrap();
    let instances = recast["final_buff_state"]["instances"].as_array().unwrap();
    assert_eq!(instances.len(), 1);
    assert_eq!(instances[0]["duration"]["expires_at_us"], 11_597_776);

    let mut multiple_owners = request();
    multiple_owners.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(6);
    set_a5_vitals(&mut multiple_owners.actors[0], 13_000);
    let mut second_edgar = multiple_owners.actors[0].clone();
    second_edgar.slot = 1;
    multiple_owners.actors.push(second_edgar);
    multiple_owners.relative_duration_us = 2_500_000;
    multiple_owners.actions = vec![
        make_action(
            0,
            0,
            "GA_Edgar_UltraSkill",
            "GE_Player_Edgar_UltraSkill_Damage",
            0,
            0,
        ),
        GameActionCommand::SwitchActor {
            timestamp_us: 1_000_000,
            target_actor_index: 1,
            cooldown_us: 0,
        },
        make_action(
            2_000_000,
            1,
            "GA_Edgar_UltraSkill",
            "GE_Player_Edgar_UltraSkill_Damage",
            0,
            0,
        ),
    ];
    let multiple_owners = run_game_scenario(&multiple_owners, &ExecutionLimits::default()).unwrap();
    let instances = crate::simulation::tests::support::final_buffs(&multiple_owners);
    assert_eq!(instances.len(), 1);
    assert_eq!(instances[0]["owner_actor"], 1);
    assert_eq!(instances[0]["stacks"], 1);

    let mut failed_admission = request();
    failed_admission.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(6);
    set_a5_vitals(&mut failed_admission.actors[0], 13_000);
    failed_admission.actors[0].initial_energy = 0;
    failed_admission.relative_duration_us = 1_500_000;
    failed_admission.actions = vec![make_action(
        0,
        0,
        "GA_Edgar_UltraSkill",
        "GE_Player_Edgar_UltraSkill_Damage",
        100,
        0,
    )];
    let failed_admission =
        run_game_scenario(&failed_admission, &ExecutionLimits::default()).unwrap();
    assert_eq!(failed_admission["final_buff_state"]["instances"], json!([]));
}
