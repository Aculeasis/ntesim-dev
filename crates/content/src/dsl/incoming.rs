//! Fixed, named-recipient scenario damage; settlement remains in the engine.
use super::{
    Declaration, DeclarationKind, SourceLocation,
    actor::ActorDraft,
    configuration::failure,
    evaluate::{Bindings, Evaluator},
    validation::Diagnostic,
};
use crate::simulation::GameActorDamageEvent;

#[derive(Clone, Debug)]
pub struct CompiledIncomingEvent {
    pub event: GameActorDamageEvent,
    pub source_target: Option<usize>,
    pub basis: Option<crate::simulation::GameIncomingDamageBasis>,
    pub loc: SourceLocation,
}

pub(super) fn compile(
    declaration: &Declaration,
    actors: &[ActorDraft],
    target_count: usize,
    constants: &Bindings,
    evaluator: &mut Evaluator,
) -> Result<CompiledIncomingEvent, Diagnostic> {
    let DeclarationKind::Incoming { actor, pairs } = &declaration.kind else {
        return Err(failure(
            "Expected incoming declaration.".into(),
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
    let mut damage = None;
    let mut source_target = None;
    let mut basis = None;
    for (key, expression) in pairs {
        match key.as_str() {
            "at" => {
                let value = evaluator
                    .evaluate(expression, constants)
                    .map_err(Diagnostic::from)?;
                let number = value
                    .number()
                    .map_err(|error| failure(error, &expression.loc))?;
                let val = number
                    .to_microseconds()
                    .map_err(|error| failure(error, &expression.loc))?;
                if at.replace(val).is_some() {
                    return Err(failure(
                        "Duplicate incoming field 'at'.".into(),
                        &expression.loc,
                    ));
                }
            }
            "damage" => {
                let value = evaluator
                    .evaluate(expression, constants)
                    .map_err(Diagnostic::from)?;
                let number = value
                    .number()
                    .map_err(|error| failure(error, &expression.loc))?;
                let val = number
                    .to_safe_integer()
                    .map_err(|error| failure(error, &expression.loc))?;
                if damage.replace(val).is_some() {
                    return Err(failure(
                        "Duplicate incoming field 'damage'.".into(),
                        &expression.loc,
                    ));
                }
            }
            "source" => {
                let target = match &expression.kind {
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
                if source_target.replace(target).is_some() {
                    return Err(failure(
                        "Duplicate incoming field 'source'.".into(),
                        &expression.loc,
                    ));
                }
            }
            "basis" => {
                let basis_val = match &expression.kind {
                    crate::dsl::ExpressionKind::Identifier(name) => match name.as_str() {
                        "before_defense" => {
                            crate::simulation::GameIncomingDamageBasis::BeforeDefense
                        }
                        "externally_mitigated" => {
                            crate::simulation::GameIncomingDamageBasis::ExternallyMitigated
                        }
                        _ => {
                            return Err(failure(
                                format!("Unsupported incoming basis '{name}'."),
                                &expression.loc,
                            ));
                        }
                    },
                    _ => {
                        return Err(failure(
                            "Expected before_defense or externally_mitigated for basis.".into(),
                            &expression.loc,
                        ));
                    }
                };
                if basis.replace(basis_val).is_some() {
                    return Err(failure(
                        "Duplicate incoming field 'basis'.".into(),
                        &expression.loc,
                    ));
                }
            }
            _ => {
                return Err(failure(
                    format!("Unsupported incoming field '{key}'."),
                    &expression.loc,
                ));
            }
        }
    }
    let timestamp_us =
        at.ok_or_else(|| failure("incoming requires at.".into(), &declaration.loc))?;
    let delivered_damage =
        damage.ok_or_else(|| failure("incoming requires damage.".into(), &declaration.loc))?;
    Ok(CompiledIncomingEvent {
        event: GameActorDamageEvent {
            timestamp_us,
            actor_index,
            delivered_damage,
        },
        source_target,
        basis,
        loc: declaration.loc.clone(),
    })
}
