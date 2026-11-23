//! Compile authored dummy combat observations.

use super::{
    Declaration, DeclarationKind,
    actor::ActorDraft,
    configuration::failure,
    evaluate::{Bindings, Evaluator},
    validation::Diagnostic,
};
use crate::simulation::GameDummyCombatEvent;

pub(super) fn compile(
    declaration: &Declaration,
    actors: &[ActorDraft],
    target_count: usize,
    constants: &Bindings,
    evaluator: &mut Evaluator,
) -> Result<GameDummyCombatEvent, Diagnostic> {
    let DeclarationKind::Dummy {
        kind,
        target,
        pairs,
    } = &declaration.kind
    else {
        return Err(failure(
            "Expected dummy declaration.".into(),
            &declaration.loc,
        ));
    };

    let mut at = None;
    let mut actor_index = None;
    let mut in_proximity = None;
    let mut explicit_target = None;

    if let Some(t_str) = target {
        let index = super::action_parameters::target_name(t_str, target_count)
            .map_err(|err| failure(err, &declaration.loc))?;
        explicit_target = Some(index);
    }

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
                        "Duplicate dummy field 'at'.".into(),
                        &expression.loc,
                    ));
                }
            }
            "target" => {
                let target_idx = match &expression.kind {
                    crate::dsl::ExpressionKind::Identifier(name) => {
                        super::action_parameters::target_name(name, target_count)
                            .map_err(|err| failure(err, &expression.loc))?
                    }
                    _ => {
                        let value = evaluator
                            .evaluate(expression, constants)
                            .map_err(Diagnostic::from)?;
                        let number = value
                            .number()
                            .map_err(|error| failure(error, &expression.loc))?;
                        let val = number
                            .to_safe_integer()
                            .map_err(|error| failure(error, &expression.loc))?
                            as usize;
                        if val >= target_count {
                            return Err(failure(
                                format!("Unknown target 't{val}'."),
                                &expression.loc,
                            ));
                        }
                        val
                    }
                };
                if explicit_target.replace(target_idx).is_some() {
                    return Err(failure(
                        "Duplicate dummy field 'target'.".into(),
                        &expression.loc,
                    ));
                }
            }
            "actor" => {
                let parsed_actor = match &expression.kind {
                    crate::dsl::ExpressionKind::Identifier(name) => actors
                        .iter()
                        .position(|draft| draft.name == *name)
                        .ok_or_else(|| {
                            failure(
                                format!("Declare '{name} char ...;' first."),
                                &expression.loc,
                            )
                        })?,
                    _ => {
                        return Err(failure(
                            "Expected actor identifier.".into(),
                            &expression.loc,
                        ));
                    }
                };
                if actor_index.replace(parsed_actor).is_some() {
                    return Err(failure(
                        "Duplicate dummy field 'actor'.".into(),
                        &expression.loc,
                    ));
                }
            }
            "in_proximity" | "proximity" => {
                let val = evaluator
                    .evaluate(expression, constants)
                    .map_err(Diagnostic::from)?;
                let flag = super::action_parameters::boolean(&val, "in_proximity")
                    .map_err(|err| failure(err, &expression.loc))?;
                if in_proximity.replace(flag).is_some() {
                    return Err(failure(
                        "Duplicate dummy field 'in_proximity'.".into(),
                        &expression.loc,
                    ));
                }
            }
            _ => {
                return Err(failure(
                    format!("Unsupported dummy field '{key}'."),
                    &expression.loc,
                ));
            }
        }
    }

    let timestamp_us = at.ok_or_else(|| failure("dummy requires at.".into(), &declaration.loc))?;
    let target_index = explicit_target.unwrap_or(0);
    if target_index >= target_count {
        return Err(failure(
            format!("Unknown target 't{target_index}'."),
            &declaration.loc,
        ));
    }

    match kind.as_str() {
        "attack_start" => {
            let actor_index = actor_index.ok_or_else(|| {
                failure(
                    "dummy attack_start requires actor.".into(),
                    &declaration.loc,
                )
            })?;
            Ok(GameDummyCombatEvent::AttackStart {
                timestamp_us,
                target_index,
                actor_index,
            })
        }
        "proximity" => {
            let in_proximity = in_proximity.ok_or_else(|| {
                failure(
                    "dummy proximity requires in_proximity.".into(),
                    &declaration.loc,
                )
            })?;
            Ok(GameDummyCombatEvent::Proximity {
                timestamp_us,
                target_index,
                in_proximity,
            })
        }
        _ => Err(failure(
            format!("Unknown dummy combat kind '{kind}'."),
            &declaration.loc,
        )),
    }
}
