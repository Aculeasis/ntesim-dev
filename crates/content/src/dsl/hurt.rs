//! Periodic seeded scenario damage.

use super::{
    Declaration, DeclarationKind, SourceLocation,
    actor::ActorDraft,
    configuration::failure,
    evaluate::{Bindings, Evaluator},
    validation::Diagnostic,
};
use crate::rng::Rng;
use crate::simulation::GameActorDamageEvent;

pub(crate) const SCENARIO_STREAM: &str = "scenario";
pub(crate) const SCENARIO_SEQUENCE: u64 = 0x53_43_45_4E; // "SCEN"

#[derive(Clone, Debug)]
pub struct CompiledHurtDeclaration {
    pub interval_min_us: u64,
    pub interval_max_us: u64,
    pub amount_min: u64,
    pub amount_max: u64,
    pub actor_index: usize, // usize::MAX represents dynamic active actor at delivery
    pub loc: SourceLocation,
}

impl CompiledHurtDeclaration {
    pub fn is_stochastic(&self) -> bool {
        self.interval_min_us != self.interval_max_us || self.amount_min != self.amount_max
    }
}

pub(super) fn compile(
    declaration: &Declaration,
    actors: &[ActorDraft],
    constants: &Bindings,
    evaluator: &mut Evaluator,
) -> Result<CompiledHurtDeclaration, Diagnostic> {
    let DeclarationKind::Hurt { pairs } = &declaration.kind else {
        return Err(failure(
            "Expected hurt declaration.".into(),
            &declaration.loc,
        ));
    };

    let mut interval = None;
    let mut amount = None;
    let mut actor_index = None;

    for (key, expression) in pairs {
        match key.as_str() {
            "interval" => {
                let val = evaluator
                    .evaluate(expression, constants)
                    .map_err(Diagnostic::from)?;
                let (min_num, max_num) =
                    val.range().map_err(|err| failure(err, &expression.loc))?;
                let min_us = min_num
                    .to_microseconds()
                    .map_err(|err| failure(err, &expression.loc))?;
                let max_us = max_num
                    .to_microseconds()
                    .map_err(|err| failure(err, &expression.loc))?;
                if min_us == 0 {
                    return Err(failure(
                        "hurt every interval must be positive.".into(),
                        &expression.loc,
                    ));
                }
                if min_us > max_us {
                    return Err(failure(
                        "hurt every interval minimum exceeds maximum.".into(),
                        &expression.loc,
                    ));
                }
                if interval.replace((min_us, max_us)).is_some() {
                    return Err(failure(
                        "Duplicate hurt field 'interval'.".into(),
                        &expression.loc,
                    ));
                }
            }
            "amount" => {
                let val = evaluator
                    .evaluate(expression, constants)
                    .map_err(Diagnostic::from)?;
                let (min_num, max_num) =
                    val.range().map_err(|err| failure(err, &expression.loc))?;
                let min_amt = min_num
                    .to_safe_integer()
                    .map_err(|err| failure(err, &expression.loc))?;
                let max_amt = max_num
                    .to_safe_integer()
                    .map_err(|err| failure(err, &expression.loc))?;
                if min_amt > max_amt {
                    return Err(failure(
                        "hurt every amount minimum exceeds maximum.".into(),
                        &expression.loc,
                    ));
                }
                if amount.replace((min_amt, max_amt)).is_some() {
                    return Err(failure(
                        "Duplicate hurt field 'amount'.".into(),
                        &expression.loc,
                    ));
                }
            }
            "actor" => {
                let parsed_actor = match &expression.kind {
                    crate::dsl::ExpressionKind::Identifier(name) => {
                        if name == "active" {
                            usize::MAX
                        } else {
                            actors
                                .iter()
                                .position(|draft| draft.name == *name)
                                .ok_or_else(|| {
                                    failure(
                                        format!("Declare '{name} char ...;' first."),
                                        &expression.loc,
                                    )
                                })?
                        }
                    }
                    _ => {
                        return Err(failure(
                            "Expected actor name or 'active' for hurt actor.".into(),
                            &expression.loc,
                        ));
                    }
                };
                if actor_index.replace(parsed_actor).is_some() {
                    return Err(failure(
                        "Duplicate hurt field 'actor'.".into(),
                        &expression.loc,
                    ));
                }
            }
            _ => {
                return Err(failure(
                    format!("Unsupported hurt field '{key}'."),
                    &expression.loc,
                ));
            }
        }
    }

    let (interval_min_us, interval_max_us) = interval
        .ok_or_else(|| failure("hurt every requires interval.".into(), &declaration.loc))?;
    let (amount_min, amount_max) =
        amount.ok_or_else(|| failure("hurt every requires amount.".into(), &declaration.loc))?;
    let actor_index = actor_index.unwrap_or(usize::MAX);

    Ok(CompiledHurtDeclaration {
        interval_min_us,
        interval_max_us,
        amount_min,
        amount_max,
        actor_index,
        loc: declaration.loc.clone(),
    })
}

pub fn sample_hurt_events(
    declarations: &[CompiledHurtDeclaration],
    seed: u64,
    relative_duration_us: u64,
) -> Result<Vec<(GameActorDamageEvent, SourceLocation)>, String> {
    let has_stochastic = declarations.iter().any(|d| d.is_stochastic());
    let mut rng = if has_stochastic {
        Some(Rng::new(
            seed,
            [(SCENARIO_STREAM.into(), SCENARIO_SEQUENCE)].into(),
        )?)
    } else {
        None
    };

    let mut events = Vec::new();
    for decl in declarations {
        let mut current_us = 0u64;
        while current_us < relative_duration_us {
            let interval = if decl.interval_min_us == decl.interval_max_us {
                decl.interval_min_us
            } else {
                let span = decl.interval_max_us - decl.interval_min_us + 1;
                let sample = rng
                    .as_mut()
                    .ok_or("scenario RNG not initialized")?
                    .uniform(SCENARIO_STREAM, span)?;
                decl.interval_min_us + sample.value
            };
            let timestamp_us = match current_us.checked_add(interval) {
                Some(t) if t <= relative_duration_us => t,
                _ => break,
            };
            let amount = if decl.amount_min == decl.amount_max {
                decl.amount_min
            } else {
                let span = decl.amount_max - decl.amount_min + 1;
                let sample = rng
                    .as_mut()
                    .ok_or("scenario RNG not initialized")?
                    .uniform(SCENARIO_STREAM, span)?;
                decl.amount_min + sample.value
            };
            events.push((
                GameActorDamageEvent {
                    timestamp_us,
                    actor_index: decl.actor_index,
                    delivered_damage: amount,
                },
                decl.loc.clone(),
            ));
            current_us = timestamp_us;
        }
    }
    Ok(events)
}
