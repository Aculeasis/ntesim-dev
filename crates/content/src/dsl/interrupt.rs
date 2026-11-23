//! Compile explicit scenario actor interruptions.

use super::{
    Declaration, DeclarationKind,
    actor::ActorDraft,
    configuration::failure,
    evaluate::{Bindings, Evaluator},
    validation::Diagnostic,
};
use crate::simulation::GameActorInterruptEvent;

pub(super) fn compile(
    declaration: &Declaration,
    actors: &[ActorDraft],
    constants: &Bindings,
    evaluator: &mut Evaluator,
) -> Result<GameActorInterruptEvent, Diagnostic> {
    let DeclarationKind::Interrupt { actor, pairs } = &declaration.kind else {
        return Err(failure(
            "Expected interrupt declaration.".into(),
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
                        "Duplicate interrupt field 'at'.".into(),
                        &expression.loc,
                    ));
                }
            }
            _ => {
                return Err(failure(
                    format!("Unsupported interrupt field '{key}'."),
                    &expression.loc,
                ));
            }
        }
    }

    let timestamp_us =
        at.ok_or_else(|| failure("interrupt requires at.".into(), &declaration.loc))?;

    Ok(GameActorInterruptEvent {
        timestamp_us,
        actor_index,
        armor_damage: None,
    })
}
