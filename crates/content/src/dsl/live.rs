//! Resumable, exact rotation control flow. The host owns combat and time.
//!
//! Each call runs only until the next authored input/wait. Resuming samples the
//! host again, without replaying earlier expressions or resetting lexical state.

use super::{
    evaluate::{Bindings, EvaluationError, Evaluator, ExpressionContext, Value},
    *,
};
use std::collections::BTreeMap;

#[derive(Debug, PartialEq)]
pub enum Input<'a> {
    TargetGauge {
        target: &'a Expression,
        gauge: Option<Value>,
    },
    Action {
        actor: &'a str,
        action: &'a str,
        params: Vec<(&'a str, Value)>,
    },
    Wait(Value),
    WaitUntil(Value),
    WaitForEnergy(Value),
}

#[derive(Debug, PartialEq)]
pub struct Occurrence<'a> {
    /// Monotonic trial-local identity; repeated source ranges remain distinct.
    pub index: usize,
    pub loc: &'a SourceLocation,
    pub input: Input<'a>,
}

struct Variable {
    value: Value,
    constant: bool,
}

struct Context<'a, H> {
    scopes: &'a [BTreeMap<String, Variable>],
    host: &'a H,
}

impl<H: ExpressionContext> ExpressionContext for Context<'_, H> {
    fn identifier(&self, name: &str) -> Result<Value, String> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name))
            .map(|variable| variable.value.clone())
            .ok_or_else(|| format!("Unknown variable '{name}'."))
    }

    fn field(&self, parts: &[String]) -> Result<Value, String> {
        self.host.field(parts)
    }
}

enum Loop<'a> {
    While(&'a Expression, &'a [Statement]),
    Repeat(u64, &'a [Statement]),
    For(&'a ForStatement, bool),
}

enum Frame<'a> {
    Statements(&'a [Statement]),
    LeaveScope,
    Loop {
        kind: Loop<'a>,
        loc: &'a SourceLocation,
        exit_depth: usize,
    },
    Actions(&'a str, &'a [ActionInvocation]),
    Action(&'a str, &'a ActionInvocation, u64),
}

pub struct Rotation<'a> {
    frames: Vec<Frame<'a>>,
    scopes: Vec<BTreeMap<String, Variable>>,
    evaluator: Evaluator,
    max_inputs: usize,
    inputs: usize,
}

impl<'a> Rotation<'a> {
    pub fn new(
        statements: &'a [Statement],
        constants: &Bindings,
        max_steps: usize,
        max_inputs: usize,
    ) -> Self {
        Self {
            frames: vec![Frame::Statements(statements)],
            scopes: vec![
                constants
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
            evaluator: Evaluator::new(max_steps),
            max_inputs,
            inputs: 0,
        }
    }

    pub fn steps(&self) -> usize {
        self.evaluator.steps()
    }

    fn eval(
        &mut self,
        expression: &Expression,
        host: &impl ExpressionContext,
    ) -> Result<Value, EvaluationError> {
        self.evaluator.evaluate(
            expression,
            &Context {
                scopes: &self.scopes,
                host,
            },
        )
    }

    fn error(message: impl Into<String>, loc: &SourceLocation) -> EvaluationError {
        EvaluationError {
            code: validation::DiagnosticCode::Evaluation,
            message: message.into(),
            loc: loc.clone(),
        }
    }

    fn declare(
        &mut self,
        name: &str,
        value: Value,
        constant: bool,
        loc: &SourceLocation,
    ) -> Result<(), EvaluationError> {
        value
            .number()
            .map_err(|message| Self::error(message, loc))?;
        let scope = self.scopes.last_mut().expect("root scope");
        if scope.contains_key(name) {
            return Err(Self::error(format!("Duplicate variable '{name}'."), loc));
        }
        scope.insert(name.into(), Variable { value, constant });
        Ok(())
    }

    fn assign(
        &mut self,
        name: &str,
        value: Value,
        loc: &SourceLocation,
    ) -> Result<(), EvaluationError> {
        value
            .number()
            .map_err(|message| Self::error(message, loc))?;
        let variable = self
            .scopes
            .iter_mut()
            .rev()
            .find_map(|scope| scope.get_mut(name))
            .ok_or_else(|| Self::error(format!("Unknown variable '{name}'."), loc))?;
        if variable.constant {
            return Err(Self::error(
                format!("Cannot assign constant '{name}'."),
                loc,
            ));
        }
        variable.value = value;
        Ok(())
    }

    fn block(&mut self, body: &'a [Statement]) {
        self.scopes.push(BTreeMap::new());
        self.frames.push(Frame::LeaveScope);
        self.frames.push(Frame::Statements(body));
    }

    fn condition(
        &mut self,
        expr: &Expression,
        host: &impl ExpressionContext,
    ) -> Result<bool, EvaluationError> {
        self.eval(expr, host)?
            .truthy()
            .map_err(|message| Self::error(message, &expr.loc))
    }

    fn count(
        &mut self,
        expr: &Expression,
        host: &impl ExpressionContext,
    ) -> Result<u64, EvaluationError> {
        self.eval(expr, host)?
            .number()
            .and_then(number::ExactNumber::to_safe_integer)
            .map_err(|message| Self::error(message, &expr.loc))
    }

    fn control(&mut self, continuing: bool, loc: &SourceLocation) -> Result<(), EvaluationError> {
        while let Some(frame) = self.frames.pop() {
            if let Frame::Loop {
                ref kind,
                exit_depth,
                ..
            } = frame
            {
                self.scopes.truncate(
                    exit_depth + usize::from(continuing && matches!(kind, Loop::For(..))),
                );
                if continuing {
                    self.frames.push(frame);
                }
                return Ok(());
            }
        }
        Err(Self::error("Loop control requires an enclosing loop.", loc))
    }

    fn iteration(
        &mut self,
        kind: Loop<'a>,
        loc: &'a SourceLocation,
        exit_depth: usize,
        host: &impl ExpressionContext,
    ) -> Result<(), EvaluationError> {
        self.evaluator.tick(loc)?;
        let (next, body) = match kind {
            Loop::While(condition, body) => (
                self.condition(condition, host)?
                    .then_some(Loop::While(condition, body)),
                body,
            ),
            Loop::Repeat(count, body) => ((count > 0).then(|| Loop::Repeat(count - 1, body)), body),
            Loop::For(loop_, first) => {
                if !first {
                    let value = self.eval(&loop_.update_expr, host)?;
                    self.assign(&loop_.update_var, value, loc)?;
                }
                (
                    self.condition(&loop_.condition, host)?
                        .then_some(Loop::For(loop_, false)),
                    loop_.body.as_slice(),
                )
            }
        };
        if let Some(kind) = next {
            self.frames.push(Frame::Loop {
                kind,
                loc,
                exit_depth,
            });
            self.block(body);
        } else {
            self.scopes.truncate(exit_depth);
        }
        Ok(())
    }

    fn statement(
        &mut self,
        statement: &'a Statement,
        host: &impl ExpressionContext,
    ) -> Result<Option<Input<'a>>, EvaluationError> {
        let loc = &statement.loc;
        self.evaluator.tick(loc)?;
        match &statement.kind {
            StatementKind::TargetGauge { target, gauge } => {
                return Ok(Some(Input::TargetGauge {
                    target,
                    gauge: gauge
                        .as_ref()
                        .map(|expr| self.eval(expr, host))
                        .transpose()?,
                }));
            }
            StatementKind::Variable {
                name,
                is_const,
                value,
            } => {
                let value = self.eval(value, host)?;
                self.declare(name, value, *is_const, loc)?;
            }
            StatementKind::Assignment { target, value } => {
                let value = self.eval(value, host)?;
                self.assign(target, value, loc)?;
            }
            StatementKind::ActionList { actor, actions } => {
                self.frames.push(Frame::Actions(actor, actions))
            }
            StatementKind::Expression(expr) => {
                if let ExpressionKind::Call { callee, args } = &expr.kind
                    && ["wait", "wait_until", "wait_for_energy"].contains(&callee.as_str())
                {
                    if args.len() != 1 {
                        return Err(Self::error(
                            format!("{callee} requires one argument."),
                            &expr.loc,
                        ));
                    }
                    let value = self.eval(&args[0], host)?;
                    return Ok(Some(match callee.as_str() {
                        "wait" => Input::Wait(value),
                        "wait_until" => Input::WaitUntil(value),
                        _ => Input::WaitForEnergy(value),
                    }));
                }
                self.eval(expr, host)?;
            }
            StatementKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let branch = if self.condition(condition, host)? {
                    then_branch
                } else {
                    else_branch.as_deref().unwrap_or(&[])
                };
                self.block(branch);
            }
            StatementKind::Repeat { count, body } => {
                let count = self.count(count, host)?;
                self.frames.push(Frame::Loop {
                    kind: Loop::Repeat(count, body),
                    loc,
                    exit_depth: self.scopes.len(),
                });
            }
            StatementKind::While { condition, body } => self.frames.push(Frame::Loop {
                kind: Loop::While(condition, body),
                loc,
                exit_depth: self.scopes.len(),
            }),
            StatementKind::For(loop_) => {
                let value = self.eval(&loop_.init, host)?;
                let exit_depth = self.scopes.len();
                self.scopes.push(BTreeMap::new());
                self.declare(&loop_.var_name, value, false, loc)?;
                self.frames.push(Frame::Loop {
                    kind: Loop::For(loop_, true),
                    loc,
                    exit_depth,
                });
            }
            StatementKind::Break => self.control(false, loc)?,
            StatementKind::Continue => self.control(true, loc)?,
        }
        Ok(None)
    }

    /// Samples only fields actually reached by exact, short-circuit evaluation.
    /// Work is bounded even for empty loops and arbitrarily many zero-time waits.
    pub fn next(
        &mut self,
        host: &impl ExpressionContext,
    ) -> Result<Option<Occurrence<'a>>, EvaluationError> {
        while let Some(frame) = self.frames.pop() {
            let (input, loc) = match frame {
                Frame::LeaveScope => {
                    self.scopes.pop();
                    continue;
                }
                Frame::Statements(statements) => {
                    let Some((statement, rest)) = statements.split_first() else {
                        continue;
                    };
                    self.frames.push(Frame::Statements(rest));
                    let Some(input) = self.statement(statement, host)? else {
                        continue;
                    };
                    (input, &statement.loc)
                }
                Frame::Loop {
                    kind,
                    loc,
                    exit_depth,
                } => {
                    self.iteration(kind, loc, exit_depth, host)?;
                    continue;
                }
                Frame::Actions(actor, actions) => {
                    let Some((action, rest)) = actions.split_first() else {
                        continue;
                    };
                    let count = action
                        .count
                        .as_ref()
                        .map_or(Ok(1), |expr| self.count(expr, host))?;
                    if count == 0 {
                        return Err(Self::error(
                            "Action repeater must be positive.",
                            &action.loc,
                        ));
                    }
                    self.frames.push(Frame::Actions(actor, rest));
                    self.frames.push(Frame::Action(actor, action, count));
                    continue;
                }
                Frame::Action(actor, action, count) => {
                    self.evaluator.tick(&action.loc)?;
                    if count > 1 {
                        self.frames.push(Frame::Action(actor, action, count - 1));
                    }
                    let params = action
                        .params
                        .iter()
                        .map(|(name, expr)| {
                            let value = if matches!(name.as_str(), "target" | "window") {
                                let ExpressionKind::Identifier(target) = &expr.kind else {
                                    return Err(Self::error(
                                        format!("{name} requires a static identifier."),
                                        &expr.loc,
                                    ));
                                };
                                Value::Text(target.clone())
                            } else {
                                self.eval(expr, host)?
                            };
                            Ok((name.as_str(), value))
                        })
                        .collect::<Result<_, EvaluationError>>()?;
                    (
                        Input::Action {
                            actor,
                            action: &action.action,
                            params,
                        },
                        &action.loc,
                    )
                }
            };
            if self.inputs >= self.max_inputs {
                return Err(EvaluationError {
                    code: validation::DiagnosticCode::ActionLimit,
                    message: format!(
                        "Script input budget exhausted: limit {} inputs (actions and waits).",
                        self.max_inputs
                    ),
                    loc: loc.clone(),
                });
            }
            let index = self.inputs;
            self.inputs += 1;
            return Ok(Some(Occurrence { index, loc, input }));
        }
        Ok(None)
    }
}

#[cfg(test)]
#[path = "live_tests.rs"]
mod tests;
