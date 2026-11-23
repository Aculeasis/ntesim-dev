use super::*;

#[test]
fn defeated_target_cancels_pending_packets_before_damage_rng_and_callbacks() {
    for (ability, effect) in [
        ("GA_Mint019_Melee", "GE_Player_Mint_Melee1_Damage_New"),
        ("GA_Mint019_Skill", "GE_Player_Mint_Skill1_Damage_New"),
    ] {
        let mut request = research_request();
        request.actors[0].gear_procs.clear();
        request.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(0);
        request.actors[0].static_stats.atk = Ratio {
            numerator: 100,
            denominator: 1,
        };
        request.actors[0].static_stats.crit_rate = Ratio {
            numerator: 1,
            denominator: 2,
        };
        request.targets[0].defense = 0;
        request.targets[0].resistance = Ratio {
            numerator: 0,
            denominator: 1,
        };
        request.targets[0].hp_current = 1;
        request.actions = [0, 1_000_000]
            .into_iter()
            .map(|timestamp_us| GameActionCommand::InvokeAbility {
                timestamp_us,
                actor_index: 0,
                ability_id: ability.into(),
                array_index: 0,
                damage_effect_id: effect.into(),
                atk_rate: Ratio {
                    numerator: 1,
                    denominator: 1,
                },
                scaling_stat: crate::simulation::NativeScalingStat::Atk,
                energy_cost: 0,
                recovery_us: 0,
                cooldown_us: 0,
                hit_delay_us: 0,
            })
            .collect();
        request.action_metadata.clear();
        request.stochastic_crit = Some(ResearchCritPolicy {
            roll_policy: ResearchCritRollPolicy::IndependentPerTargetHitV1,
            hypothesis: "Test seeded critical sampling on live deliveries only.".into(),
            eligible_effects: BTreeMap::from([(
                request.actors[0].character_id.clone(),
                BTreeSet::from([effect.into()]),
            )]),
            actor_sequences: BTreeMap::from([(request.actors[0].character_id.clone(), 1)]),
        });
        let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
        assert_eq!(report["success"], true);
        assert_eq!(report["hits_settled"], 1);
        assert_eq!(report["hit_receipts"].as_array().unwrap().len(), 1);
        assert_eq!(
            report["hit_receipts"][0]["non_crit_damage"],
            serde_json::json!("100/1")
        );
        assert_eq!(
            report["hit_receipts"][0]["crit_damage"],
            serde_json::json!("200/1")
        );
        assert_eq!(report["hit_receipts"][0]["applied_hp_loss"], 1);
        assert_eq!(report["final_target_states"][0]["hp_current"], 0);
        assert_eq!(
            report["total_damage_exact"],
            report["hit_receipts"][0]["damage"]
        );
        assert_eq!(
            report["rng"]["streams"][&request.actors[0].character_id]["draws"],
            1
        );
        assert_eq!(
            report["trace"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|event| event["event_kind"] == "hit_target_unavailable")
                .count(),
            if ability == "GA_Mint019_Skill" { 3 } else { 2 }
        );

        request.targets[0].hp_current = 0;
        let report = run_game_scenario(&request, &ExecutionLimits::default()).unwrap();
        assert_eq!(report["hits_settled"], 0);
        assert_eq!(report["total_damage_exact"], serde_json::json!("0/1"));
        assert_eq!(
            report["rng"]["streams"][&request.actors[0].character_id]["draws"],
            0
        );
    }
}
