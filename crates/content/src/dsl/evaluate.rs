//! Pure expressions shared by static compilation and future live-state evaluation.

use super::{Expression, ExpressionKind, SourceLocation, TimeUnit, number::ExactNumber};

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Number(ExactNumber),
    Boolean(bool),
    Text(String),
    Range(Box<Value>, Box<Value>),
}

impl Value {
    pub fn number(&self) -> Result<&ExactNumber, String> {
        match self {
            Self::Number(value) => Ok(value),
            _ => Err("Expected a number; boolean flags are not numeric aliases.".into()),
        }
    }

    pub fn range(&self) -> Result<(&ExactNumber, &ExactNumber), String> {
        match self {
            Self::Range(min, max) => Ok((min.number()?, max.number()?)),
            Self::Number(value) => Ok((value, value)),
            _ => Err("Expected a number or range.".into()),
        }
    }

    pub(super) fn truthy(&self) -> Result<bool, String> {
        match self {
            Self::Number(value) => Ok(!value.is_zero()),
            Self::Boolean(value) => Ok(*value),
            _ => Err("Expected a numeric condition or boolean flag.".into()),
        }
    }

    fn condition(value: bool) -> Self {
        Self::Number(ExactNumber::from(i64::from(value)))
    }
}

/// A caller resolves names and samples fields; the evaluator never fabricates live state.
pub trait ExpressionContext {
    fn identifier(&self, name: &str) -> Result<Value, String>;
    fn field(&self, parts: &[String]) -> Result<Value, String>;
}

pub struct EmptyContext;

#[derive(Clone, Debug, Default)]
pub struct Bindings(pub std::collections::BTreeMap<String, Value>);

impl ExpressionContext for Bindings {
    fn identifier(&self, name: &str) -> Result<Value, String> {
        self.0
            .get(name)
            .cloned()
            .ok_or_else(|| format!("Unknown variable '{name}'."))
    }

    fn field(&self, parts: &[String]) -> Result<Value, String> {
        EmptyContext.field(parts)
    }
}

impl ExpressionContext for EmptyContext {
    fn identifier(&self, name: &str) -> Result<Value, String> {
        Err(format!("Unknown variable '{name}'."))
    }

    fn field(&self, parts: &[String]) -> Result<Value, String> {
        Err(format!(
            "Field .{} is unavailable in this context.",
            parts.join(".")
        ))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct EvaluationError {
    pub code: super::validation::DiagnosticCode,
    pub message: String,
    pub loc: SourceLocation,
}

/// One budget can be shared across many expressions during a compilation or trial.
pub struct Evaluator {
    remaining_nodes: usize,
    initial_budget: usize,
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new(100_000)
    }
}

impl Evaluator {
    pub fn new(node_budget: usize) -> Self {
        Self {
            remaining_nodes: node_budget,
            initial_budget: node_budget,
        }
    }

    pub fn steps(&self) -> usize {
        self.initial_budget - self.remaining_nodes
    }

    pub(super) fn tick(&mut self, loc: &SourceLocation) -> Result<(), EvaluationError> {
        if self.remaining_nodes == 0 {
            return Err(EvaluationError {
                code: super::validation::DiagnosticCode::WorkLimit,
                message: format!(
                    "Script operation budget exhausted: limit {} operations. This work limit is independent of simulated combat time.",
                    self.initial_budget
                ),
                loc: loc.clone(),
            });
        }
        self.remaining_nodes -= 1;
        Ok(())
    }

    pub fn evaluate(
        &mut self,
        expression: &Expression,
        context: &impl ExpressionContext,
    ) -> Result<Value, EvaluationError> {
        self.visit(expression, context, 0)
    }

    fn visit(
        &mut self,
        expression: &Expression,
        context: &impl ExpressionContext,
        depth: usize,
    ) -> Result<Value, EvaluationError> {
        let fail = |message: String| EvaluationError {
            code: expression_failure_code(&expression.kind),
            message,
            loc: expression.loc.clone(),
        };
        if depth >= 128 {
            return Err(EvaluationError {
                code: super::validation::DiagnosticCode::WorkLimit,
                message:
                    "Expression nesting limit reached: 128 levels. Simplify nested expressions."
                        .into(),
                loc: expression.loc.clone(),
            });
        }
        self.tick(&expression.loc)?;
        let result = match &expression.kind {
            ExpressionKind::Number { decimal, unit } => {
                ExactNumber::decimal(decimal).and_then(|value| {
                    match unit {
                        None | Some(TimeUnit::Seconds) => Ok(value),
                        Some(TimeUnit::Milliseconds) => value.divide(&ExactNumber::from(1000)),
                        Some(TimeUnit::Microseconds) => value.divide(&ExactNumber::from(1_000_000)),
                    }
                    .map(Value::Number)
                })
            }
            ExpressionKind::StringLiteral(value) => Ok(Value::Text(value.clone())),
            ExpressionKind::Boolean(value) => Ok(Value::Boolean(*value)),
            ExpressionKind::Identifier(name) => context.identifier(name),
            ExpressionKind::FieldAccess(parts) => context.field(parts),
            ExpressionKind::Unary { op, expr } => {
                let value = self.visit(expr, context, depth + 1)?;
                match op.as_str() {
                    "!" => value.truthy().map(|v| Value::condition(!v)),
                    "+" => value.number().cloned().map(Value::Number),
                    "-" => value.number().map(|v| Value::Number(v.negate())),
                    _ => Err(format!("Unknown unary operator '{op}'.")),
                }
            }
            ExpressionKind::Binary { op, left, right } => {
                let left = self.visit(left, context, depth + 1)?;
                if op == "&&" || op == "||" {
                    let truth = left.truthy().map_err(fail)?;
                    if (op == "&&" && !truth) || (op == "||" && truth) {
                        return Ok(Value::condition(truth));
                    }
                    self.visit(right, context, depth + 1)?
                        .truthy()
                        .map(Value::condition)
                } else {
                    let right = self.visit(right, context, depth + 1)?;
                    binary(op, &left, &right)
                }
            }
            ExpressionKind::Call { callee, args } => {
                let values = args
                    .iter()
                    .map(|arg| self.visit(arg, context, depth + 1))
                    .collect::<Result<Vec<_>, _>>()?;
                builtin(callee, &values)
            }
            ExpressionKind::Range { min, max } => {
                let min_val = self.visit(min, context, depth + 1)?;
                let max_val = self.visit(max, context, depth + 1)?;
                Ok(Value::Range(Box::new(min_val), Box::new(max_val)))
            }
        };
        result.map_err(fail)
    }
}

fn expression_failure_code(kind: &ExpressionKind) -> super::validation::DiagnosticCode {
    use super::validation::DiagnosticCode;
    let arithmetic = match kind {
        ExpressionKind::Number { .. } => true,
        ExpressionKind::Binary { op, .. } => matches!(op.as_str(), "+" | "-" | "*" | "/" | "%"),
        ExpressionKind::Unary { op, .. } => matches!(op.as_str(), "+" | "-"),
        ExpressionKind::Call { callee, .. } => matches!(
            callee.as_str(),
            "abs" | "floor" | "ceil" | "min" | "max" | "clamp"
        ),
        _ => false,
    };
    if arithmetic {
        DiagnosticCode::Arithmetic
    } else {
        DiagnosticCode::Evaluation
    }
}

fn binary(op: &str, left: &Value, right: &Value) -> Result<Value, String> {
    if op == "==" || op == "!=" {
        return Ok(Value::condition(if op == "==" {
            left == right
        } else {
            left != right
        }));
    }
    let (left, right) = (left.number()?, right.number()?);
    let number = match op {
        "+" => left.add(right),
        "-" => left.subtract(right),
        "*" => left.multiply(right),
        "/" => left.divide(right),
        "%" => left.remainder(right),
        "<" => return Ok(Value::condition(left < right)),
        "<=" => return Ok(Value::condition(left <= right)),
        ">" => return Ok(Value::condition(left > right)),
        ">=" => return Ok(Value::condition(left >= right)),
        _ => return Err(format!("Unknown binary operator '{op}'.")),
    }?;
    Ok(Value::Number(number))
}

fn builtin(name: &str, args: &[Value]) -> Result<Value, String> {
    let numbers = args
        .iter()
        .map(Value::number)
        .collect::<Result<Vec<_>, _>>()?;
    let result = match (name, numbers.as_slice()) {
        ("abs", [value]) => value.absolute(),
        ("floor", [value]) => value.floor(),
        ("ceil", [value]) => value.ceil(),
        ("min", [_, ..]) => (*numbers.iter().min().expect("nonempty slice")).clone(),
        ("max", [_, ..]) => (*numbers.iter().max().expect("nonempty slice")).clone(),
        ("clamp", [value, low, high]) => {
            if low > high {
                return Err("clamp lower bound exceeds upper bound.".into());
            }
            (*value).clamp(low, high).clone()
        }
        ("abs" | "floor" | "ceil" | "min" | "max" | "clamp", _) => {
            return Err(format!("Invalid argument count for '{name}'."));
        }
        _ => return Err(format!("Unknown numeric built-in '{name}'.")),
    };
    Ok(Value::Number(result))
}
