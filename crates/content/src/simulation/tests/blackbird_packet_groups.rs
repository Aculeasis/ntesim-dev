use super::*;
use serde_json::{Value, json};

fn two_targets() -> GameScenarioRequest {
    let mut req = super::blackbird_actions::request();
    req.actors[0].initial_energy = 0;
    req.relative_duration_us = 10_000_000;
    let mut target = req.targets[0].clone();
    target.target_index = 1;
    req.targets.push(target);
    req
}

fn groups(report: &Value) -> BTreeMap<String, Vec<&Value>> {
    let mut groups = BTreeMap::<String, Vec<_>>::new();
    for hit in report["hit_receipts"].as_array().unwrap() {
        let group = hit["energy_group_id"].as_str().unwrap();
        groups.entry(group.into()).or_default().push(hit);
    }
    groups
}

#[test]
fn blackbird_normal_chain_has_seventeen_primary_groups_and_one_resource_grant_per_group() {
    for first_dead in [false, true] {
        let mut req = two_targets();
        if first_dead {
            req.targets[0].hp_current = 0;
        }
        let template = req.actions[0].clone();
        req.actions = (0..5)
            .map(|index| {
                let mut action = template.clone();
                if let GameActionCommand::InvokeAbility { timestamp_us, .. } = &mut action {
                    *timestamp_us = index * 1000;
                }
                action
            })
            .collect();
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        let packet_groups = groups(&report);
        assert_eq!(packet_groups.len(), 17);
        for hits in packet_groups.values() {
            assert_eq!(hits.len(), if first_dead { 1 } else { 2 });
            if !first_dead {
                assert_eq!(hits[0]["target_index"], 0);
                assert_eq!(hits[1]["target_index"], 1);
                assert_eq!(hits[0]["non_crit_damage"], hits[1]["non_crit_damage"]);
            }
        }
        // .838 +2*1.726 +4*1.066 +5*.913 +5*1.289 =19.564, not per dummy.
        assert_eq!(report["final_actor_states"][0]["granted_energy"], 19);
        assert_eq!(
            report["final_actor_states"][0]["charge_remainder"],
            json!("141/250")
        );
    }
}

#[test]
fn blackbird_skill_clones_the_admitted_primary_and_keeps_four_energy_opportunities() {
    for first_dead in [false, true] {
        let mut req = two_targets();
        req.actors[0].talent_levels = None;
        if first_dead {
            req.targets[0].hp_current = 0;
        }
        if let GameActionCommand::InvokeAbility {
            ability_id,
            damage_effect_id,
            atk_rate,
            ..
        } = &mut req.actions[0]
        {
            *ability_id = "GA_BlackBird_Skill".into();
            *damage_effect_id = "GE_Player_BlackBird_Skill_Damage".into();
            *atk_rate = Ratio {
                numerator: 3,
                denominator: 1,
            };
        }
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(
            report["execution_completed"], true,
            "{} {}",
            report["halt_diagnostic"], report["rejected_action"]
        );
        let packet_groups = groups(&report);
        assert_eq!(packet_groups.len(), 4);
        let mut primaries = 0;
        for hits in packet_groups.values() {
            assert_eq!(hits.len(), if first_dead { 1 } else { 2 });
            for hit in hits {
                if hit["scaling_rate"] == json!("3/1") {
                    primaries += 1;
                    assert_eq!(hit["non_crit_damage"], json!("1500/1"));
                } else {
                    assert_eq!(hit["non_crit_damage"], json!("500/1"));
                }
            }
        }
        assert_eq!(primaries, if first_dead { 1 } else { 2 });
        // Four1.89 source grants retain7.56 energy even when target zero is dead.
        assert_eq!(report["final_actor_states"][0]["granted_energy"], 7);
        assert_eq!(
            report["final_actor_states"][0]["charge_remainder"],
            json!("14/25")
        );
    }
}

#[test]
fn blackbird_burst_keeps_nine_distinct_groups_and_clones_the_admitted_coefficient() {
    let mut req = two_targets();
    req.actors[0].talent_levels = None;
    req.actors[0].initial_energy = 100;
    if let GameActionCommand::InvokeAbility {
        ability_id,
        damage_effect_id,
        atk_rate,
        energy_cost,
        ..
    } = &mut req.actions[0]
    {
        *ability_id = "GA_BlackBird_UltraSkill".into();
        *damage_effect_id = "GE_Player_BlackBird_UltraSkill_Damage".into();
        *atk_rate = Ratio {
            numerator: 3,
            denominator: 1,
        };
        *energy_cost = 100;
    }
    let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
    assert_eq!(
        report["execution_completed"], true,
        "{} {}",
        report["halt_diagnostic"], report["rejected_action"]
    );
    let packet_groups = groups(&report);
    assert_eq!(packet_groups.len(), 9);
    let mut primary_count = 0;
    for hits in packet_groups.values() {
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0]["non_crit_damage"], hits[1]["non_crit_damage"]);
        for hit in hits {
            if hit["scaling_rate"] == json!("3/1") {
                primary_count += 1;
                assert_eq!(hit["non_crit_damage"], json!("1800/1"));
            } else {
                assert_eq!(hit["non_crit_damage"], json!("1401/5"));
            }
        }
    }
    assert_eq!(primary_count, 2);
    assert_eq!(report["final_actor_states"][0]["spent_energy"], 100);
    assert_eq!(report["final_actor_states"][0]["granted_energy"], 0);
}
