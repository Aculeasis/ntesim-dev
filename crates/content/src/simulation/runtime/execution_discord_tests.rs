use super::*;
use crate::formula::Ratio;
use crate::simulation::{ExactFraction, GameTargetBreakProfileConfig};

fn discord_switch_request(
    fraction: Option<Ratio>,
    accepted: &[crate::HypothesisId],
) -> GameScenarioRequest {
    let mut request = crate::simulation::tests::research_request();
    request.actions.clear();
    request.action_metadata.clear();
    request.stochastic_crit = None;
    request.model_assumptions.clear();

    let mut actors = Vec::new();
    for (slot, character_id) in [
        "character:1003", // Incantation: applies Scorch when switched to Chaos.
        "character:1054", // Daffodil: Chaos and the assumed B2 cap effect.
        "character:1020", // Psyche: creates Nova while Scorch is live.
    ]
    .into_iter()
    .enumerate()
    {
        let mut actor = request.actors[0].clone();
        actor.slot = slot;
        actor.character_id = character_id.into();
        actor.initial_cycle = 100;
        actor.static_stats.cycle_intensity = Ratio::zero();
        actor.gear_procs.clear();
        actors.push(actor);
    }
    request.actors = actors;

    request.targets[0].break_profile = Some(GameTargetBreakProfileConfig {
        capacity: Ratio {
            numerator: 100,
            denominator: 1,
        },
        current_gauge: Ratio {
            numerator: 100,
            denominator: 1,
        },
        broken_duration_us: 1_000_000,
        discord_fraction: fraction,
    });
    let base = crate::HypothesisId::CombatEsperCycleReactionsV1;
    request
        .model_assumptions
        .insert(base.as_str().into(), base.description().into());
    for id in accepted {
        request
            .model_assumptions
            .insert(id.as_str().into(), id.description().into());
    }
    request
}

#[test]
fn daffodil_discord_admission_errors_leave_switch_state_unchanged() {
    let discord_model = crate::HypothesisId::CombatDiscordBreakGaugeV1;
    let daffodil_model = crate::HypothesisId::DaffodilDiscordCapReductionV1;
    for target in [0, 1] {
        let fraction = None;
        let accepted = vec![discord_model, daffodil_model];
        let expected_error = "discord_fraction";
        let mut request = discord_switch_request(fraction, &accepted);
        request.targets.push(request.targets[0].clone());
        request.targets[1].target_index = 1;
        request.targets[1 - target].break_profile = None;
        request.aim_targets = [(0, target), (1, target)].into();
        let (mut execution, _) = PreparedExecution::new(&request, request.seed, &[], &[]).unwrap();
        assert!(execution.switch_actor(&request, &[], 0, 1, 0).unwrap());

        let active_before = execution.state.active_actor_index;
        let cycle_before = execution.state.actors[1].cycle.current();
        let assumptions_before = execution.state.execution_model_assumptions.clone();
        let queue_before = execution.queue.len();
        let sequence_before = execution.seq;
        let trace_before = execution.trace.len();
        let buff_ids_before = execution
            .state
            .buffs
            .instances()
            .iter()
            .map(|buff| buff.id.clone())
            .collect::<Vec<_>>();
        let break_before = execution.state.targets[target]
            .break_state
            .as_ref()
            .unwrap();
        let cap_before = break_before.capacity;
        let gauge_before = break_before.gauge;

        let error = execution.switch_actor(&request, &[], 1, 2, 0).unwrap_err();
        assert!(error.contains(expected_error), "{error}");
        assert_eq!(execution.state.active_actor_index, active_before);
        assert_eq!(execution.state.actors[1].cycle.current(), cycle_before);
        assert_eq!(
            execution.state.execution_model_assumptions,
            assumptions_before
        );
        assert_eq!(execution.queue.len(), queue_before);
        assert_eq!(execution.seq, sequence_before);
        assert_eq!(execution.trace.len(), trace_before);
        assert_eq!(
            execution
                .state
                .buffs
                .instances()
                .iter()
                .map(|buff| buff.id.clone())
                .collect::<Vec<_>>(),
            buff_ids_before
        );
        let break_after = execution.state.targets[target]
            .break_state
            .as_ref()
            .unwrap();
        assert_eq!(break_after.capacity, cap_before);
        assert_eq!(break_after.gauge, gauge_before);
        assert_eq!(cap_before, ExactFraction::new(100, 1).unwrap());
    }
}
