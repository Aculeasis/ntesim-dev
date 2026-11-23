use super::*;
use serde_json::json;

#[test]
fn chiz_grain_acquisition_and_missing_market_have_explicit_contracts() {
    let limits = ExecutionLimits::default();
    for (awakening, expected_grain, capacity) in [
        (0, 450, 15000),
        (1, 450, 16500),
        (4, 540, 16500),
        (5, 16500, 16500),
        (6, 16500, 16500),
    ] {
        let mut req = research_request();
        req.stochastic_crit = None;
        req.actors[0].character_id = "character:1073".into();
        req.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        req.action_metadata.clear();
        req.actions.truncate(1);
        req.action_metadata
            .retain(|index, _| *index < req.actions.len());
        if let GameActionCommand::InvokeAbility {
            ability_id,
            damage_effect_id,
            ..
        } = &mut req.actions[0]
        {
            *ability_id = "GA_Chiichan073_Melee".into();
            *damage_effect_id = "GE_Player_Chiichan073_Melee1_Damage".into();
        }
        let report = run_game_scenario(&req, &limits).unwrap();
        // Source curves CoinMelee1@80=450, CoinMax@80=15000; A1 cap+10%, A4 acquisition+20%, A5 starts full.
        assert_eq!(
            report["final_actor_states"][0]["fractional_resources"]["chiz.grain"]["current"],
            json!(format!("{}/{}", expected_grain, 1))
        );
        assert_eq!(
            report["final_actor_states"][0]["fractional_resources"]["chiz.grain"]["capacity"],
            json!(format!("{}/{}", capacity, 1))
        );
        // A2/A4 alter resource contracts; no substitute DMG scalar is applied.
        assert_eq!(
            serde_json::from_value::<crate::simulation::ExactFraction>(
                report["hit_receipts"][0]["non_crit_damage"].clone(),
            )
            .unwrap()
            .numerator,
            400
        );
        if let GameActionCommand::InvokeAbility {
            ability_id,
            damage_effect_id,
            ..
        } = &mut req.actions[0]
        {
            *ability_id = "GA_Chiichan073_UltraSkill".into();
            *damage_effect_id = "GE_Player_Chiichan073_UltraSkill1_Damage".into();
        }
        assert!(
            run_game_scenario(&req, &limits)
                .unwrap_err()
                .contains("UNSUPPORTED_CHIZ_GRAIN_MARKET")
        );
    }
}
