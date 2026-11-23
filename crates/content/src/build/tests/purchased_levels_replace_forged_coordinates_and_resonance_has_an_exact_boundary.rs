use super::*;

#[test]
fn purchased_levels_replace_forged_coordinates_and_resonance_has_an_exact_boundary() {
    let (tables, coverage, mut request) = skill_fixture();
    for awakening in [0, 2, 3, 6] {
        request.scenario.actors[0].awakening = ntesim_data::AwakeningSelection::prefix(awakening);
        for action in &mut request.scenario.actions {
            if let GameActionCommand::InvokeAbility {
                array_index,
                atk_rate,
                ..
            } = action
            {
                *array_index = usize::MAX;
                *atk_rate = Ratio::one();
            }
        }
        let compiled = compile(&tables, &coverage, &request, &ExecutionLimits::default()).unwrap();
        let bonus = u64::from(awakening >= 3);
        assert_eq!(
            compiled.receipt["action_receipts"][1]["compiled_array_index"],
            7 + bonus
        );
        assert_eq!(
            compiled.receipt["action_receipts"][1]["skill_level"]["effective_level"],
            8 + bonus
        );
        assert_eq!(
            compiled.receipt["action_receipts"][1]["compiled_atk_rate"],
            if bonus == 0 {
                json!("771/500")
            } else {
                json!("833/500")
            }
        );
        assert!(matches!(
            request.scenario.actions[0],
            GameActionCommand::InvokeAbility {
                array_index: usize::MAX,
                ..
            }
        ));
        assert!(run_game_scenario(&compiled.scenario, &ExecutionLimits::default()).unwrap()["execution_completed"].as_bool().unwrap());
    }
}
