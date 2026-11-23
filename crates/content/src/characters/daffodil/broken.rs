//! R6 target Chaos resistance follows the real Broken deadline, including extensions.
use super::parameters::PARAMETERS;
use crate::simulation::state::{GameSimulatorState, TargetStatusDeclaration, apply_target_status};
use crate::simulation::{GameEventReceipt, GameScenarioRequest};

pub(crate) const HYPOTHESIS: crate::HypothesisId =
    crate::HypothesisId::DaffodilBrokenChaosResistanceV1;
const SOURCE: &str = "character:1054.r6";
const ELEMENT: &str = "ECharacterElementType::CHARACTER_ELEMENT_TYPE_CHAOS";

fn id(target: usize) -> String {
    format!("ntesim.daffodil.broken_chaos.target:{target}")
}

fn deadline(state: &GameSimulatorState, target: usize) -> Result<u64, String> {
    state
        .targets
        .get(target)
        .and_then(|target| target.break_state.as_ref())
        .and_then(|bar| bar.broken_until_us)
        .filter(|end| *end > state.current_time_us)
        .ok_or_else(|| "Daffodil R6 requires a live Broken target deadline".into())
}

fn apply(
    state: &mut GameSimulatorState,
    target: usize,
    owner: usize,
    end: u64,
) -> Result<(), String> {
    apply_target_status(
        &mut state.buffs,
        TargetStatusDeclaration {
            source_actor: owner,
            source: SOURCE.into(),
            target_index: target,
            id: id(target),
            now_us: state.mode_clock.relative_us(),
            duration_us: end
                .checked_sub(state.current_time_us)
                .filter(|duration| *duration > 0)
                .ok_or("Daffodil R6 target deadline must be in the future")?,
            reductions: std::collections::BTreeMap::from([(
                ELEMENT.into(),
                PARAMETERS.broken_resistance_reduction,
            )]),
        },
    )
}

fn receipt(
    state: &GameSimulatorState,
    target: usize,
    owner: usize,
    end: u64,
    kind: &str,
) -> GameEventReceipt {
    GameEventReceipt {
        time_us: state.current_time_us,
        phase: "ReactionOrBreak".into(),
        event_kind: kind.into(),
        active_actor: state.active_actor_index,
        hit_index: None,
        from_actor: Some(owner),
        to_actor: None,
        energy_change: None,
        energy_fanout: None,
        data: Some(crate::simulation::GameEventPayload {
            target_index: Some(target),
            exact_amount: Some(PARAMETERS.broken_resistance_reduction),
            expires_at_us: Some(end),
            ..Default::default()
        }),
    }
}

pub(crate) fn start(
    request: &GameScenarioRequest,
    state: &mut GameSimulatorState,
    target: usize,
    trace: &mut Vec<GameEventReceipt>,
) -> Result<(), String> {
    let end = deadline(state, target)?;
    for (owner, config) in request.actors.iter().enumerate() {
        if config.character_id != "character:1054"
            || config.awakening.count() < 6
            || state.actors[owner].hp == 0
        {
            continue;
        }
        crate::runtime_api::record_hypothesis_id(
            request,
            &mut state.execution_model_assumptions,
            HYPOTHESIS,
        )?;
        apply(state, target, owner, end)?;
        trace.push(receipt(
            state,
            target,
            owner,
            end,
            "daffodil_broken_chaos_applied",
        ));
    }
    Ok(())
}

pub(crate) fn extend(
    state: &mut GameSimulatorState,
    target: usize,
    trace: &mut Vec<GameEventReceipt>,
) -> Result<(), String> {
    let end = deadline(state, target)?;
    let status_id = id(target);
    let owners: Vec<_> = state
        .buffs
        .instances()
        .iter()
        .filter(|buff| {
            buff.id == status_id
                && buff.source == SOURCE
                && buff.recipient.matches_target_index(target)
        })
        .map(|buff| buff.owner_actor)
        .collect();
    for owner in owners {
        apply(state, target, owner, end)?;
        trace.push(receipt(
            state,
            target,
            owner,
            end,
            "daffodil_broken_chaos_extended",
        ));
    }
    Ok(())
}

pub(crate) fn clear(state: &mut GameSimulatorState, target: usize) {
    let status_id = id(target);
    for owner in 0..state.actors.len() {
        state.buffs.remove_owned_buff(&status_id, SOURCE, owner);
    }
}
