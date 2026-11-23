use super::*;

#[test]
fn pre_hit_status_is_rolled_back_when_final_damage_is_zero() {
    for (character, ability, effect, awakening, zero_attack, full_resistance) in [
        (
            "character:1071",
            "GA_Chaos071_Skill",
            "GE_Player_Chaos_Skill1_Damage",
            6,
            true,
            false,
        ),
        (
            "character:1055",
            "GA_Kuhara_Skill",
            "GE_Player_Kuhara_Skill1_Damage",
            6,
            true,
            false,
        ),
        (
            "character:1008",
            "GA_Skia_Skill",
            "GE_Player_Skia_Skill1_Damage",
            2,
            false,
            true,
        ),
    ] {
        let mut req = research_request();
        req.actors[0].character_id = character.into();
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.actors[0].gear_procs.clear();
        req.stochastic_crit = None;
        if zero_attack {
            req.actors[0].static_stats.atk = Ratio::zero();
            req.actors[0].static_stats.base_atk = 0;
        }
        if full_resistance {
            req.targets[0].resistance = Ratio::one();
        }
        req.action_metadata.clear();
        req.actions = vec![GameActionCommand::InvokeAbility {
            timestamp_us: 0,
            actor_index: 0,
            ability_id: ability.into(),
            damage_effect_id: effect.into(),
            array_index: 0,
            atk_rate: Ratio::one(),
            scaling_stat: crate::simulation::NativeScalingStat::Atk,
            energy_cost: 0,
            recovery_us: 0,
            cooldown_us: 0,
            hit_delay_us: 0,
        }];
        req.relative_duration_us = 1;
        let report = run_game_scenario(&req, &ExecutionLimits::default()).unwrap();
        assert_eq!(report["hit_receipts"][0]["damage"], "0/1", "{ability}");
        assert_eq!(
            report["final_target_states"][0]
                .get("statuses")
                .and_then(serde_json::Value::as_array)
                .map_or(0, Vec::len),
            0,
            "{ability}"
        );
    }
}
