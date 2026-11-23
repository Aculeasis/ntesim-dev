//! Lexical variable and control-flow binding, independent of catalog and live state.

use std::collections::BTreeMap;

use super::{
    validation::{Diagnostic, binding},
    *,
};

pub(super) fn validate_symbols(script: &Script, diagnostics: &mut Vec<Diagnostic>) {
    let mut validator = Symbols {
        scopes: vec![BTreeMap::new()],
        diagnostics,
    };
    for option in &script.options {
        if !["crit", "stop"].contains(&option.key.as_str()) {
            validator.expression(&option.value, 0);
        }
    }
    for item in script.items() {
        match item {
            ScriptItem::Declaration(declaration) => validator.declaration(declaration),
            ScriptItem::Statement(statement) => validator.statement(statement, 0, 0),
        }
    }
}

struct Symbols<'a> {
    scopes: Vec<BTreeMap<String, bool>>,
    diagnostics: &'a mut Vec<Diagnostic>,
}

impl Symbols<'_> {
    fn declare(&mut self, name: &str, constant: bool, loc: &SourceLocation) {
        let scope = self.scopes.last_mut().expect("root scope");
        if scope.contains_key(name)
            || [
                "scenario",
                "trials",
                "__proto__",
                "prototype",
                "constructor",
            ]
            .contains(&name)
        {
            self.diagnostics.push(binding(
                format!("Duplicate or reserved variable '{name}'."),
                loc,
            ));
        } else {
            scope.insert(name.into(), constant);
        }
    }

    fn lookup(&self, name: &str) -> Option<bool> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.get(name).copied())
    }

    fn assignment(&mut self, name: &str, loc: &SourceLocation) {
        let message = match self.lookup(name) {
            None => Some(format!("Unknown variable '{name}'.")),
            Some(true) => Some(format!("Cannot assign constant '{name}'.")),
            Some(false) => None,
        };
        if let Some(message) = message {
            self.diagnostics.push(binding(message, loc));
        }
    }

    fn expression(&mut self, expression: &Expression, depth: usize) {
        if depth >= 128 {
            self.diagnostics.push(binding(
                "Expression nesting exceeds its validation limit.".into(),
                &expression.loc,
            ));
            return;
        }
        match &expression.kind {
            ExpressionKind::Identifier(name) => {
                if self.lookup(name).is_none() {
                    self.diagnostics.push(binding(
                        format!("Unknown variable '{name}'."),
                        &expression.loc,
                    ));
                }
            }
            ExpressionKind::Unary { expr, .. } => self.expression(expr, depth + 1),
            ExpressionKind::Binary { left, right, .. } => {
                self.expression(left, depth + 1);
                self.expression(right, depth + 1);
            }
            ExpressionKind::Call { callee, args } => {
                if ![
                    "min",
                    "max",
                    "abs",
                    "floor",
                    "ceil",
                    "clamp",
                    "wait",
                    "wait_until",
                    "wait_for_energy",
                ]
                .contains(&callee.as_str())
                {
                    self.diagnostics.push(binding(
                        format!("Unknown built-in '{callee}'."),
                        &expression.loc,
                    ));
                }
                for argument in args {
                    self.expression(argument, depth + 1);
                }
            }
            ExpressionKind::Range { min, max } => {
                self.expression(min, depth + 1);
                self.expression(max, depth + 1);
            }
            _ => {}
        }
    }

    fn declaration(&mut self, declaration: &Declaration) {
        match &declaration.kind {
            DeclarationKind::Character {
                talents,
                qte,
                extra_fields,
                ..
            } => {
                if let Some(talents) = talents {
                    for expression in [&talents.normal, &talents.skill, &talents.burst] {
                        self.expression(expression, 0);
                    }
                }
                if let Some(qte) = qte {
                    self.expression(qte, 0);
                }
                self.pairs(extra_fields);
            }
            DeclarationKind::Weapon { extra_fields, .. } => self.pairs(extra_fields),
            DeclarationKind::Stats { pairs, .. }
            | DeclarationKind::Target { pairs }
            | DeclarationKind::Hurt { pairs }
            | DeclarationKind::Incoming { pairs, .. }
            | DeclarationKind::Dummy { pairs, .. }
            | DeclarationKind::Interrupt { pairs, .. }
            | DeclarationKind::Combat { pairs, .. } => self.pairs(pairs),
            _ => {}
        }
    }

    fn pairs(&mut self, pairs: &[(String, Expression)]) {
        for (key, expression) in pairs {
            // Catalog recipients and template names are resolved by their owning pass.
            if ![
                "target",
                "window",
                "type",
                "recipient",
                "actor",
                "source",
                "basis",
            ]
            .contains(&key.as_str())
            {
                self.expression(expression, 0);
            }
        }
    }

    fn block(&mut self, body: &[Statement], loops: usize, depth: usize) {
        self.scopes.push(BTreeMap::new());
        for statement in body {
            self.statement(statement, loops, depth + 1);
        }
        self.scopes.pop();
    }

    fn statement(&mut self, statement: &Statement, loops: usize, depth: usize) {
        if depth >= 128 {
            self.diagnostics.push(binding(
                "Statement nesting exceeds its validation limit.".into(),
                &statement.loc,
            ));
            return;
        }
        match &statement.kind {
            StatementKind::TargetGauge {
                gauge: Some(value), ..
            } => self.expression(value, 0),
            StatementKind::Variable {
                name,
                is_const,
                value,
            } => {
                self.expression(value, 0);
                self.declare(name, *is_const, &statement.loc);
            }
            StatementKind::Assignment { target, value } => {
                self.assignment(target, &statement.loc);
                self.expression(value, 0);
            }
            StatementKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.expression(condition, 0);
                self.block(then_branch, loops, depth);
                if let Some(body) = else_branch {
                    self.block(body, loops, depth);
                }
            }
            StatementKind::Repeat { count, body } => {
                self.expression(count, 0);
                self.block(body, loops + 1, depth);
            }
            StatementKind::While { condition, body } => {
                self.expression(condition, 0);
                self.block(body, loops + 1, depth);
            }
            StatementKind::For(loop_) => {
                self.expression(&loop_.init, 0);
                self.scopes.push(BTreeMap::new());
                self.declare(&loop_.var_name, false, &statement.loc);
                self.expression(&loop_.condition, 0);
                self.assignment(&loop_.update_var, &statement.loc);
                self.expression(&loop_.update_expr, 0);
                self.block(&loop_.body, loops + 1, depth);
                self.scopes.pop();
            }
            StatementKind::Break | StatementKind::Continue if loops == 0 => {
                self.diagnostics.push(binding(
                    "Loop control requires an enclosing loop.".into(),
                    &statement.loc,
                ));
            }
            StatementKind::Expression(expression) => self.expression(expression, 0),
            StatementKind::ActionList { actions, .. } => {
                for action in actions {
                    self.pairs(&action.params);
                    if let Some(count) = &action.count {
                        self.expression(count, 0);
                    }
                }
            }
            _ => {}
        }
    }
}
