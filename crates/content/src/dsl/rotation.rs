//! Scheduled Studio export. Native source uses the resumable combat executor.

use super::{
    compiler::CompileLimits,
    configuration::{CompiledConfiguration, failure, merge_assumptions},
    evaluate::{Evaluator, ExpressionContext, Value},
    number::ExactNumber,
    validation::Diagnostic,
    *,
};
use crate::simulation::{
    GameActionCommand, GameLiveBranch, GameLiveBranchCondition, GameLiveEnergyWait,
    GameScenarioRequest,
};
use std::collections::BTreeMap;
mod plunge;

struct ActionOptions {
    target: Option<usize>,
    critical: bool,
    direction: Option<crate::simulation::GameDashDirection>,
    hold: Option<crate::simulation::GameHoldInput>,
    plunge: Option<crate::simulation::GamePlungeInput>,
    branch: Option<crate::simulation::GameBranchInput>,
}

#[derive(Clone)]
struct Variable {
    value: Value,
    constant: bool,
}

struct Context<'a> {
    scopes: &'a [BTreeMap<String, Variable>],
    clock: u64,
    duration: u64,
    fons: Option<u64>,
}

impl ExpressionContext for Context<'_> {
    fn identifier(&self, name: &str) -> Result<Value, String> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name))
            .map(|var| var.value.clone())
            .ok_or_else(|| format!("Unknown variable '{name}'."))
    }

    fn field(&self, parts: &[String]) -> Result<Value, String> {
        let integer = match parts
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .as_slice()
        {
            ["time"] => self.clock,
            ["duration"] => self.duration,
            ["team", "fons"] => {
                return self
                    .fons
                    .ok_or_else(|| "Missing options start_fons required by .team.fons.".to_owned())
                    .and_then(|n| ExactNumber::decimal(&n.to_string()))
                    .map(Value::Number);
            }
            _ => {
                return Err(format!(
                    "Field .{} is unsupported outside a live observation condition.",
                    parts.join(".")
                ));
            }
        };
        ExactNumber::decimal(&integer.to_string())?
            .divide(&ExactNumber::from(1_000_000))
            .map(Value::Number)
    }
}

#[derive(PartialEq)]
enum Control {
    Next,
    Break,
    Continue,
}

pub(super) struct RotationCompiler<'a> {
    config: &'a CompiledConfiguration,
    pub request: GameScenarioRequest,
    pub locations: Vec<SourceLocation>,
    pub branches: Vec<GameLiveBranch>,
    pub energy_waits: Vec<GameLiveEnergyWait>,
    pub warnings: Vec<String>,
    evaluator: Evaluator,
    scopes: Vec<BTreeMap<String, Variable>>,
    clock: u64,
    active: usize,
    limits: CompileLimits,
}

impl<'a> RotationCompiler<'a> {
    pub fn new(
        config: &'a CompiledConfiguration,
        request: GameScenarioRequest,
        limits: CompileLimits,
    ) -> Self {
        Self {
            config,
            active: request.active_actor_index,
            request,
            locations: Vec::new(),
            branches: Vec::new(),
            energy_waits: Vec::new(),
            warnings: Vec::new(),
            evaluator: Evaluator::new(limits.max_steps),
            scopes: vec![
                config
                    .constants
                    .0
                    .iter()
                    .map(|(name, value)| {
                        (
                            name.clone(),
                            Variable {
                                value: value.clone(),
                                constant: true,
                            },
                        )
                    })
                    .collect(),
            ],
            clock: 0,
            limits,
        }
    }

    pub fn steps(&self) -> usize {
        self.evaluator.steps()
    }

    pub fn run(&mut self, statements: &[Statement]) -> Result<(), Diagnostic> {
        if self.execute(statements, 0)? != Control::Next {
            return Err(failure(
                "Loop control requires an enclosing loop.".into(),
                &statements[0].loc,
            ));
        }
        Ok(())
    }

    fn eval(&mut self, expression: &Expression) -> Result<Value, Diagnostic> {
        let context = Context {
            scopes: &self.scopes,
            clock: self.clock,
            duration: self.request.relative_duration_us,
            fons: self.request.start_fons,
        };
        self.evaluator
            .evaluate(expression, &context)
            .map_err(Into::into)
    }

    fn integer(&mut self, expression: &Expression) -> Result<u64, Diagnostic> {
        self.eval(expression)?
            .number()
            .and_then(ExactNumber::to_safe_integer)
            .map_err(|message| failure(message, &expression.loc))
    }

    fn duration(&mut self, expression: &Expression) -> Result<u64, Diagnostic> {
        self.eval(expression)?
            .number()
            .and_then(ExactNumber::to_microseconds)
            .map_err(|message| failure(message, &expression.loc))
    }

    fn condition(&mut self, expression: &Expression) -> Result<bool, Diagnostic> {
        self.eval(expression)?
            .truthy()
            .map_err(|message| failure(message, &expression.loc))
    }

    fn push(&mut self, command: GameActionCommand, loc: &SourceLocation) -> Result<(), Diagnostic> {
        if self.request.actions.len() >= self.limits.max_actions {
            return Err(failure("Action compilation budget exceeded.".into(), loc));
        }
        self.request.actions.push(command);
        self.locations.push(loc.clone());
        Ok(())
    }

    fn advance(&mut self, duration: u64, loc: &SourceLocation) -> Result<(), Diagnostic> {
        let value = self
            .clock
            .checked_add(duration)
            .ok_or_else(|| failure("Compiled timestamp overflow.".into(), loc))?;
        if value > 9_007_199_254_740_991 {
            return Err(failure(
                "Compiled timestamp exceeds the safe integer range.".into(),
                loc,
            ));
        }
        self.clock = value;
        Ok(())
    }

    fn block(&mut self, statements: &[Statement], depth: usize) -> Result<Control, Diagnostic> {
        self.scopes.push(BTreeMap::new());
        let result = self.execute(statements, depth + 1);
        self.scopes.pop();
        result
    }

    fn assign(
        &mut self,
        name: &str,
        expression: &Expression,
        loc: &SourceLocation,
    ) -> Result<(), Diagnostic> {
        let value = self.eval(expression)?;
        value.number().map_err(|message| failure(message, loc))?;
        let variable = self
            .scopes
            .iter_mut()
            .rev()
            .find_map(|scope| scope.get_mut(name))
            .ok_or_else(|| failure(format!("Unknown variable '{name}'."), loc))?;
        if variable.constant {
            return Err(failure(format!("Cannot assign constant '{name}'."), loc));
        }
        variable.value = value;
        Ok(())
    }

    fn declare(
        &mut self,
        name: &str,
        value: Value,
        constant: bool,
        loc: &SourceLocation,
    ) -> Result<(), Diagnostic> {
        value.number().map_err(|message| failure(message, loc))?;
        let scope = self.scopes.last_mut().expect("root scope");
        if scope.contains_key(name) {
            return Err(failure(format!("Duplicate variable '{name}'."), loc));
        }
        scope.insert(name.into(), Variable { value, constant });
        Ok(())
    }

    fn execute(&mut self, statements: &[Statement], depth: usize) -> Result<Control, Diagnostic> {
        for statement in statements {
            if depth >= 128 {
                return Err(failure(
                    "Statement nesting limit exceeded.".into(),
                    &statement.loc,
                ));
            }
            self.evaluator
                .tick(&statement.loc)
                .map_err(Diagnostic::from)?;
            match &statement.kind {
                StatementKind::TargetGauge { .. } => {
                    return Err(failure(
                        "Manual target inputs require native live source execution.".into(),
                        &statement.loc,
                    ));
                }
                StatementKind::Variable {
                    name,
                    is_const,
                    value,
                } => {
                    let value = self.eval(value)?;
                    self.declare(name, value, *is_const, &statement.loc)?;
                }
                StatementKind::Assignment { target, value } => {
                    self.assign(target, value, &statement.loc)?
                }
                StatementKind::ActionList { actor, actions } => {
                    for action in actions {
                        self.action(actor, action)?;
                    }
                }
                StatementKind::Expression(expression) => self.expression_statement(expression)?,
                StatementKind::If {
                    condition,
                    then_branch,
                    else_branch,
                } => {
                    if !self.live_branch(
                        condition,
                        then_branch,
                        else_branch.as_deref().unwrap_or(&[]),
                        &statement.loc,
                    )? {
                        let condition = self.condition(condition)?;
                        let branch = if condition {
                            then_branch.as_slice()
                        } else {
                            else_branch.as_deref().unwrap_or(&[])
                        };
                        let control = self.block(branch, depth)?;
                        if control != Control::Next {
                            return Ok(control);
                        }
                    }
                }
                StatementKind::Repeat { count, body } => {
                    let count = self.integer(count)?;
                    if count > self.limits.max_steps as u64 {
                        return Err(failure(
                            "Repeat exceeds execution budget.".into(),
                            &statement.loc,
                        ));
                    }
                    for _ in 0..count {
                        self.evaluator
                            .tick(&statement.loc)
                            .map_err(Diagnostic::from)?;
                        if self.block(body, depth)? == Control::Break {
                            break;
                        }
                    }
                }
                StatementKind::While { condition, body } => {
                    while self.condition(condition)? {
                        if self.block(body, depth)? == Control::Break {
                            break;
                        }
                    }
                }
                StatementKind::For(loop_) => {
                    let value = self.eval(&loop_.init)?;
                    self.scopes.push(BTreeMap::new());
                    self.declare(&loop_.var_name, value, false, &statement.loc)?;
                    while self.condition(&loop_.condition)? {
                        if self.block(&loop_.body, depth)? == Control::Break {
                            break;
                        }
                        self.assign(&loop_.update_var, &loop_.update_expr, &statement.loc)?;
                    }
                    self.scopes.pop();
                }
                StatementKind::Break => return Ok(Control::Break),
                StatementKind::Continue => return Ok(Control::Continue),
            }
        }
        Ok(Control::Next)
    }

    fn expression_statement(&mut self, expression: &Expression) -> Result<(), Diagnostic> {
        if let ExpressionKind::Call { callee, args } = &expression.kind
            && ["wait", "wait_until", "wait_for_energy"].contains(&callee.as_str())
        {
            if args.len() != 1 {
                return Err(failure(
                    format!("{callee} requires one argument."),
                    &expression.loc,
                ));
            }
            if callee == "wait_for_energy" {
                return self.energy_wait(&args[0], &expression.loc);
            }
            let duration = self.duration(&args[0])?;
            if callee == "wait_until" {
                if duration <= self.clock {
                    self.warnings.push(format!(
                        "Line {}: wait_until has already been reached.",
                        expression.loc.line
                    ));
                } else {
                    self.advance(duration - self.clock, &expression.loc)?;
                }
            } else {
                if duration == 0 {
                    self.warnings.push(format!(
                        "Line {}: wait(0) does not advance time.",
                        expression.loc.line
                    ));
                }
                self.push(
                    GameActionCommand::Wait {
                        timestamp_us: self.clock,
                        duration_us: duration,
                    },
                    &expression.loc,
                )?;
                self.advance(duration, &expression.loc)?;
            }
            return Ok(());
        }
        self.eval(expression).map(|_| ())
    }

    fn action_options(
        &mut self,
        action: &ActionInvocation,
        descriptor: &crate::build::PublicAction,
    ) -> Result<ActionOptions, Diagnostic> {
        let loc = &action.loc;
        let mut critical = false;
        let mut direction = None;
        let mut moving = None;
        let mut stocks = None;
        let mut hold = None;
        let mut target = None;
        let plunge = self.plunge_input(action)?;
        let branch = if plunge.is_none() {
            let values = action
                .params
                .iter()
                .filter(|(key, _)| {
                    matches!(key.as_str(), "launchable" | "landing" | "height_bonus")
                })
                .map(|(key, expression)| self.eval(expression).map(|value| (key.as_str(), value)))
                .collect::<Result<Vec<_>, _>>()?;
            super::action_parameters::branch(
                &values,
                action.params.iter().any(|(key, _)| key == "hold"),
            )
            .map_err(|message| failure(message, &action.loc))?
        } else {
            None
        };
        for (key, expression) in &action.params {
            if key == "approach" {
                return Err(failure(
                    "Approach selection requires native live source execution.".into(),
                    &expression.loc,
                ));
            }
            if key == "window" {
                return Err(failure(
                    "Input windows require native live source execution.".into(),
                    &expression.loc,
                ));
            }
            if key == "target" {
                target = Some(
                    super::action_parameters::target(expression, self.config.targets.len())
                        .map_err(|message| failure(message, &expression.loc))?,
                );
                continue;
            }
            if plunge.is_some() {
                continue;
            }
            if matches!(key.as_str(), "launchable" | "landing" | "height_bonus") {
                continue;
            }
            if key == "stocks" {
                stocks = Some(
                    super::action_parameters::stocks(&self.eval(expression)?)
                        .map_err(|message| failure(message, &expression.loc))?,
                );
                continue;
            }
            if key == "moving" {
                moving = Some(
                    super::action_parameters::boolean(
                        &self.eval(expression)?,
                        &format!("{} moving", action.action),
                    )
                    .map_err(|message| failure(message, &expression.loc))?,
                );
                continue;
            }
            if key == "hold" {
                if !matches!(descriptor.command, GameActionCommand::InvokeAbility { .. })
                    && !(matches!(descriptor.command, GameActionCommand::SwitchActor { .. })
                        && branch.is_some())
                {
                    return Err(failure(
                        "hold requires an ability input.".into(),
                        &expression.loc,
                    ));
                }
                hold = Some(crate::simulation::GameHoldInput {
                    duration_us: self.duration(expression)?,
                    moving: false,
                    reserved_stocks: None,
                });
                continue;
            }
            if action.action != "dash" || !matches!(key.as_str(), "critical" | "back") {
                return Err(failure(
                    format!("Unsupported {} action parameter '{key}'.", action.action),
                    &expression.loc,
                ));
            }
            let flag =
                super::action_parameters::boolean(&self.eval(expression)?, &format!("dash {key}"))
                    .map_err(|message| failure(message, &expression.loc))?;
            if key == "critical" {
                critical = flag;
            } else {
                direction = Some(if flag {
                    crate::simulation::GameDashDirection::Back
                } else {
                    crate::simulation::GameDashDirection::Front
                });
            }
        }
        if let Some(moving) = moving {
            hold.as_mut()
                .ok_or_else(|| failure("moving requires an explicit hold duration.".into(), loc))?
                .moving = moving;
        }
        if let Some(stocks) = stocks {
            hold.as_mut()
                .ok_or_else(|| failure("stocks require an explicit hold duration.".into(), loc))?
                .reserved_stocks = Some(stocks);
        }
        Ok(ActionOptions {
            target,
            critical,
            direction,
            hold,
            plunge,
            branch,
        })
    }

    fn action(&mut self, name: &str, action: &ActionInvocation) -> Result<(), Diagnostic> {
        let loc = &action.loc;
        let index = self
            .config
            .actors
            .iter()
            .position(|actor| actor.name == name)
            .ok_or_else(|| failure(format!("Unknown actor '{name}'."), loc))?;
        let mut descriptor = self.config.actors[index]
            .projection
            .public_actions
            .get(&action.action)
            .cloned()
            .ok_or_else(|| {
                failure(
                    format!("Unsupported {} model for {name}.", action.action),
                    loc,
                )
            })?;
        let ActionOptions {
            target,
            critical,
            direction,
            hold,
            plunge,
            branch,
        } = self.action_options(action, &descriptor)?;
        descriptor
            .model_assumptions
            .extend(crate::characters::source_choices::assumptions(
                &self.config.actors[index].actor.character_id,
                &action.action,
                hold.is_some(),
            ));
        if branch.is_some() && self.config.actors[index].actor.character_id == "character:1025" {
            descriptor
                .model_assumptions
                .extend(crate::characters::hathor::branch::source_assumptions());
        }
        if let Some(input) = hold {
            descriptor.advance_us = descriptor.advance_us.max(input.duration_us);
        }
        let count = action
            .count
            .as_ref()
            .map_or(Ok(1), |count| self.integer(count))?;
        if count == 0 || count > self.limits.max_actions as u64 {
            return Err(failure(
                "Action repeater must be positive and within budget.".into(),
                loc,
            ));
        }
        for _ in 0..count {
            let mut emitted = false;
            if self.active != index {
                let swap = self.config.actors[index]
                    .projection
                    .public_actions
                    .get("swap")
                    .cloned()
                    .ok_or_else(|| failure("Switch descriptor is missing.".into(), loc))?;
                if action.action == "swap" {
                    self.emit(index, &descriptor, false, loc)?;
                    emitted = true;
                } else {
                    self.emit(index, &swap, false, loc)?;
                }
                self.active = index;
            }
            if action.action != "swap" {
                self.emit(index, &descriptor, critical, loc)?;
                emitted = true;
            }
            if emitted {
                if let Some(target) = target {
                    self.request
                        .aim_targets
                        .insert(self.request.actions.len() - 1, target);
                }
                if let Some(direction) = direction {
                    self.request
                        .dash_directions
                        .insert(self.request.actions.len() - 1, direction);
                }
                if let Some(input) = hold {
                    self.request
                        .hold_inputs
                        .insert(self.request.actions.len() - 1, input);
                }
                if let Some(input) = plunge {
                    self.request
                        .plunge_inputs
                        .insert(self.request.actions.len() - 1, input);
                    self.advance(input.landing_delay_us, loc)?;
                }
                if let Some(input) = branch {
                    self.request
                        .branch_inputs
                        .insert(self.request.actions.len() - 1, input);
                }
            }
        }
        Ok(())
    }

    fn emit(
        &mut self,
        index: usize,
        descriptor: &crate::build::PublicAction,
        critical: bool,
        loc: &SourceLocation,
    ) -> Result<(), Diagnostic> {
        merge_assumptions(
            &mut self.request.model_assumptions,
            &descriptor.model_assumptions,
        )
        .map_err(|message| failure(message, loc))?;
        let mut command = descriptor.command.clone();
        match &mut command {
            GameActionCommand::InvokeAbility {
                timestamp_us,
                actor_index,
                ..
            } => {
                *timestamp_us = self.clock;
                *actor_index = index;
            }
            GameActionCommand::SwitchActor {
                timestamp_us,
                target_actor_index,
                ..
            } => {
                *timestamp_us = self.clock;
                *target_actor_index = index;
            }
            GameActionCommand::PlayerInput {
                timestamp_us,
                actor_index,
                critical_success,
                ..
            } => {
                *timestamp_us = self.clock;
                *actor_index = index;
                *critical_success = critical;
            }
            _ => return Err(failure("Invalid public action descriptor.".into(), loc)),
        }
        self.push(command, loc)?;
        self.advance(descriptor.advance_us, loc)
    }

    fn energy_wait(
        &mut self,
        expression: &Expression,
        loc: &SourceLocation,
    ) -> Result<(), Diagnostic> {
        if !self.branches.is_empty() {
            return Err(failure(
                "Energy waits cannot be combined with a live observation branch.".into(),
                loc,
            ));
        }
        let threshold = self.integer(expression)?;
        if threshold > self.request.actors[self.active].energy_capacity {
            return Err(failure(
                "Energy threshold exceeds the active actor capacity.".into(),
                loc,
            ));
        }
        self.energy_waits.push(GameLiveEnergyWait {
            timestamp_us: self.clock,
            after_action_index: self.request.actions.len(),
            actor_index: self.active,
            energy_at_least: threshold,
            source_line: loc.line as u32,
            source_column: loc.column as u32,
        });
        self.push(
            GameActionCommand::Wait {
                timestamp_us: self.clock,
                duration_us: 0,
            },
            loc,
        )
    }

    fn branch_wait(&mut self, body: &[Statement], loc: &SourceLocation) -> Result<u64, Diagnostic> {
        if let [
            Statement {
                kind:
                    StatementKind::Expression(Expression {
                        kind: ExpressionKind::Call { callee, args },
                        ..
                    }),
                ..
            },
        ] = body
            && callee == "wait"
            && args.len() == 1
        {
            let duration = self.duration(&args[0])?;
            if duration > 0 {
                return Ok(duration);
            }
        }
        Err(failure(
            "Each live branch arm must contain one positive wait(duration).".into(),
            loc,
        ))
    }

    fn live_branch(
        &mut self,
        condition: &Expression,
        then_branch: &[Statement],
        else_branch: &[Statement],
        loc: &SourceLocation,
    ) -> Result<bool, Diagnostic> {
        let ExpressionKind::Binary { op, left, right } = &condition.kind else {
            return Ok(false);
        };
        let ExpressionKind::FieldAccess(parts) = &left.kind else {
            return Ok(false);
        };
        if op != "<" || !(parts == &["target", "hp"] || parts == &["actor", "energy"]) {
            return Ok(false);
        }
        if !self.branches.is_empty() || !self.energy_waits.is_empty() {
            return Err(failure(
                "Only one live observation branch, without energy waits, is supported.".into(),
                loc,
            ));
        }
        let threshold = self.integer(right)?;
        let wait_if_below_us = self.branch_wait(then_branch, loc)?;
        let wait_otherwise_us = self.branch_wait(else_branch, loc)?;
        let condition = if parts[0] == "target" {
            GameLiveBranchCondition::TargetHpBelow {
                target_index: 0,
                threshold,
            }
        } else {
            GameLiveBranchCondition::ActiveActorEnergyBelow { threshold }
        };
        self.push(
            GameActionCommand::Wait {
                timestamp_us: self.clock,
                duration_us: wait_if_below_us,
            },
            loc,
        )?;
        self.branches.push(GameLiveBranch {
            timestamp_us: self.clock,
            after_action_index: self.request.actions.len(),
            condition,
            wait_if_below_us,
            wait_otherwise_us,
            source_line: loc.line as u32,
            source_column: loc.column as u32,
        });
        self.advance(wait_if_below_us, loc)?;
        Ok(true)
    }
}
