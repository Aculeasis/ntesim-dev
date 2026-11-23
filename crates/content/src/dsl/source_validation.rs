//! Validate closed scalar inputs without interpreting control flow or combat.
use super::{
    evaluate::{Bindings, Evaluator},
    validation::Diagnostic,
    *,
};
use std::collections::BTreeSet;

fn closed(expression: &Expression, constants: &Bindings, shadowed: &BTreeSet<String>) -> bool {
    match &expression.kind {
        ExpressionKind::Identifier(name) => {
            constants.0.contains_key(name) && !shadowed.contains(name)
        }
        ExpressionKind::FieldAccess(_) => false,
        ExpressionKind::Unary { expr, .. } => closed(expr, constants, shadowed),
        ExpressionKind::Binary { left, right, .. } => {
            closed(left, constants, shadowed) && closed(right, constants, shadowed)
        }
        ExpressionKind::Call { args, .. } => {
            args.iter().all(|arg| closed(arg, constants, shadowed))
        }
        _ => true,
    }
}

fn scalar(
    expression: &Expression,
    time: bool,
    constants: &Bindings,
    shadowed: &BTreeSet<String>,
    evaluator: &mut Evaluator,
) -> Result<(), Diagnostic> {
    if closed(expression, constants, shadowed) {
        let value = evaluator
            .evaluate(expression, constants)
            .map_err(Diagnostic::from)?;
        let result = value.number().and_then(|number| {
            if time {
                number.to_microseconds()
            } else {
                number.to_safe_integer()
            }
        });
        result.map_err(|message| super::configuration::failure(message, &expression.loc))?;
    }
    Ok(())
}

fn action_scalar_inputs(
    action: &ActionInvocation,
    target_count: usize,
    constants: &Bindings,
    shadowed: &BTreeSet<String>,
    evaluator: &mut Evaluator,
) -> Result<(), Diagnostic> {
    branch_inputs(action, constants, shadowed, evaluator)?;
    let has_hold = action.params.iter().any(|(name, _)| name == "hold");
    for (name, expression) in &action.params {
        if name == "window" {
            super::action_parameters::window(expression)
                .map_err(|message| super::configuration::failure(message, &expression.loc))?;
            if !matches!(
                action.action.as_str(),
                "attack" | "skill" | "burst" | "dash" | "plunge"
            ) {
                return Err(super::configuration::failure(
                    "window requires a montage-producing action.".into(),
                    &expression.loc,
                ));
            }
        }
        if name == "target" {
            super::action_parameters::target(expression, target_count)
                .map_err(|message| super::configuration::failure(message, &expression.loc))?;
        }
        let flag = name == "approach"
            || name == "parry"
            || name == "moving"
            || action.action == "dash" && matches!(name.as_str(), "critical" | "back");
        if matches!(name.as_str(), "moving" | "stocks") && !has_hold {
            return Err(super::configuration::failure(
                if name == "moving" {
                    "moving requires an explicit hold duration."
                } else {
                    "stocks require an explicit hold duration."
                }
                .into(),
                &expression.loc,
            ));
        }
        if (flag || name == "stocks") && closed(expression, constants, shadowed) {
            let value = evaluator
                .evaluate(expression, constants)
                .map_err(Diagnostic::from)?;
            let valid = if name == "stocks" {
                super::action_parameters::stocks(&value).map(|_| ())
            } else {
                super::action_parameters::boolean(&value, &format!("{} {name}", action.action))
                    .map(|_| ())
            };
            valid.map_err(|message| super::configuration::failure(message, &expression.loc))?;
        }
    }
    Ok(())
}

fn branch_inputs(
    action: &ActionInvocation,
    constants: &Bindings,
    shadowed: &BTreeSet<String>,
    evaluator: &mut Evaluator,
) -> Result<(), Diagnostic> {
    if action.action == "plunge" {
        return Ok(());
    }
    let choices = action
        .params
        .iter()
        .filter(|(name, _)| matches!(name.as_str(), "launchable" | "landing" | "height_bonus"))
        .collect::<Vec<_>>();
    if choices.is_empty() {
        return Ok(());
    }
    for required in ["hold", "launchable", "landing"] {
        if !action.params.iter().any(|(name, _)| name == required) {
            return Err(super::configuration::failure(
                format!("Branch requires {required}."),
                &action.loc,
            ));
        }
    }
    for (name, expression) in choices {
        if closed(expression, constants, shadowed) {
            let value = evaluator
                .evaluate(expression, constants)
                .map_err(Diagnostic::from)?;
            let valid = match name.as_str() {
                "launchable" => super::action_parameters::boolean(&value, "launchable").map(|_| ()),
                "landing" => super::action_parameters::branch_landing(&value).map(|_| ()),
                _ => super::action_parameters::branch_height(&value).map(|_| ()),
            };
            valid.map_err(|message| super::configuration::failure(message, &expression.loc))?;
        }
    }
    Ok(())
}

pub(super) fn validate(
    statements: &[Statement],
    constants: &Bindings,
    targets: &[crate::simulation::GameTargetConfig],
) -> Result<(), Diagnostic> {
    let mut evaluator = Evaluator::default();
    let mut blocks = vec![(statements, BTreeSet::new())];
    while let Some((block, mut shadowed)) = blocks.pop() {
        for statement in block {
            match &statement.kind {
                StatementKind::TargetGauge { target, gauge } => {
                    let index = super::action_parameters::target(target, targets.len())
                        .map_err(|message| super::configuration::failure(message, &target.loc))?;
                    if targets[index].break_profile.is_none() {
                        return Err(super::configuration::failure(
                            format!("Target t{index} requires an explicit Break profile."),
                            &target.loc,
                        ));
                    }
                    if let Some(expression) = gauge
                        && closed(expression, constants, &shadowed)
                    {
                        let value = evaluator
                            .evaluate(expression, constants)
                            .map_err(Diagnostic::from)?;
                        value
                            .number()
                            .and_then(|number| number.to_safe_ratio())
                            .map_err(|message| {
                                super::configuration::failure(message, &expression.loc)
                            })?;
                    }
                }
                StatementKind::ActionList { actions, .. } => {
                    for action in actions {
                        action_scalar_inputs(
                            action,
                            targets.len(),
                            constants,
                            &shadowed,
                            &mut evaluator,
                        )?;
                    }
                }
                StatementKind::Expression(expression) => {
                    if let ExpressionKind::Call { callee, args } = &expression.kind
                        && ["wait", "wait_until", "wait_for_energy"].contains(&callee.as_str())
                    {
                        if args.len() != 1 {
                            return Err(super::configuration::failure(
                                format!("{callee} requires one argument."),
                                &expression.loc,
                            ));
                        }
                        scalar(
                            &args[0],
                            callee != "wait_for_energy",
                            constants,
                            &shadowed,
                            &mut evaluator,
                        )?;
                    }
                }
                StatementKind::Variable { name, .. } => {
                    shadowed.insert(name.clone());
                }
                StatementKind::Repeat { count, body } => {
                    scalar(count, false, constants, &shadowed, &mut evaluator)?;
                    blocks.push((body, shadowed.clone()));
                }
                StatementKind::While { body, .. } => blocks.push((body, shadowed.clone())),
                StatementKind::For(loop_) => {
                    let mut local = shadowed.clone();
                    local.insert(loop_.var_name.clone());
                    blocks.push((&loop_.body, local));
                }
                StatementKind::If {
                    then_branch,
                    else_branch,
                    ..
                } => {
                    blocks.push((then_branch, shadowed.clone()));
                    if let Some(branch) = else_branch {
                        blocks.push((branch, shadowed.clone()));
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
}
