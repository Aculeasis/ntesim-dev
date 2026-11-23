//! Compile explicit scenario actor combat state transitions.

use super::{
    Declaration, DeclarationKind,
    actor::ActorDraft,
    configuration::failure,
    evaluate::{Bindings, Evaluator},
    validation::Diagnostic,
};
use crate::simulation::GameActorCombatTransitionEvent;

pub(super) fn compile(
    declaration: &Declaration,
    actors: &[ActorDraft],
    constants: &Bindings,
    evaluator: &mut Evaluator,
) -> Result<GameActorCombatTransitionEvent, Diagnostic> {
    let DeclarationKind::Combat { actor, pairs } = &declaration.kind else {
        return Err(failure(
            "Expected combat declaration.".into(),
            &declaration.loc,
        ));
    };
    let actor_index = actors
        .iter()
        .position(|draft| draft.name == *actor)
        .ok_or_else(|| {
            failure(
                format!("Declare '{actor} char ...;' first."),
                &declaration.loc,
            )
        })?;

    let mut at = None;
    let mut leaving = None;
    for (key, expression) in pairs {
        match key.as_str() {
            "at" => {
                let val = evaluator
                    .evaluate(expression, constants)
                    .map_err(Diagnostic::from)?;
                let number = val.number().map_err(|err| failure(err, &expression.loc))?;
                let ts = number
                    .to_microseconds()
                    .map_err(|err| failure(err, &expression.loc))?;
                if at.replace(ts).is_some() {
                    return Err(failure(
                        "Duplicate combat field 'at'.".into(),
                        &expression.loc,
                    ));
                }
            }
            "leaving" => {
                let val = evaluator
                    .evaluate(expression, constants)
                    .map_err(Diagnostic::from)?;
                let flag = super::action_parameters::boolean(&val, "leaving")
                    .map_err(|err| failure(err, &expression.loc))?;
                if leaving.replace(flag).is_some() {
                    return Err(failure(
                        "Duplicate combat field 'leaving'.".into(),
                        &expression.loc,
                    ));
                }
            }
            _ => {
                return Err(failure(
                    format!("Unsupported combat field '{key}'."),
                    &expression.loc,
                ));
            }
        }
    }

    let timestamp_us = at.ok_or_else(|| failure("combat requires at.".into(), &declaration.loc))?;
    let leaving =
        leaving.ok_or_else(|| failure("combat requires leaving.".into(), &declaration.loc))?;

    Ok(GameActorCombatTransitionEvent {
        timestamp_us,
        actor_index,
        leaving,
    })
}
