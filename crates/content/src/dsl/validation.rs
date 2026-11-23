//! Frontend diagnostics. Passing this pass does not imply scenario or gameplay readiness.

use std::collections::BTreeSet;

use super::{evaluate::EvaluationError, *};

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticCode {
    Lexical,
    Syntax,
    Binding,
    Configuration,
    Evaluation,
    Arithmetic,
    WorkLimit,
    ActionLimit,
    EventLimit,
    Admission,
    Horizon,
    EnergyGate,
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Diagnostic {
    pub code: DiagnosticCode,
    pub message: String,
    pub loc: SourceLocation,
}

impl From<DslError> for Diagnostic {
    fn from(error: DslError) -> Self {
        match error {
            DslError::Lexer(error) => Self {
                code: DiagnosticCode::Lexical,
                message: error.message,
                loc: error.loc,
            },
            DslError::Parser(error) => Self {
                code: DiagnosticCode::Syntax,
                message: error.message,
                loc: error.loc,
            },
        }
    }
}

impl From<EvaluationError> for Diagnostic {
    fn from(error: EvaluationError) -> Self {
        Self {
            code: error.code,
            message: error.message,
            loc: error.loc,
        }
    }
}

pub(super) fn binding(message: String, loc: &SourceLocation) -> Diagnostic {
    Diagnostic {
        code: DiagnosticCode::Binding,
        message,
        loc: loc.clone(),
    }
}

/// Checks syntax and declaration ownership. Catalog/type checking and compilation follow
/// in separate passes; callers must not use an empty result to admit a scenario.
pub fn validate_frontend(source: &str) -> Vec<Diagnostic> {
    match parse_nte(source) {
        Ok(script) => validate_bindings(&script),
        Err(error) => vec![error.into()],
    }
}

pub fn validate_bindings(script: &Script) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut actors = BTreeSet::new();
    let mut weapons = BTreeSet::new();
    let mut cartridges = BTreeSet::new();
    let mut active = false;
    for declaration in &script.declarations {
        let loc = &declaration.loc;
        let owner = match &declaration.kind {
            DeclarationKind::Character {
                actor, awakening, ..
            } => {
                if !actors.insert(actor.clone()) {
                    diagnostics.push(binding(
                        format!("Character '{actor}' is declared more than once."),
                        loc,
                    ));
                }
                if let Some(selection) = awakening {
                    let valid = match selection {
                        AwakeningSelection::Rank(rank) => *rank <= 6,
                        AwakeningSelection::Explicit(indices) => {
                            let distinct: BTreeSet<_> = indices.iter().collect();
                            indices.iter().all(|i| (1..=6).contains(i))
                                && distinct.len() == indices.len()
                        }
                    };
                    if !valid {
                        diagnostics.push(binding("Awakening requires a rank from 0 to 6 or distinct effect indices from 1 to 6.".into(), loc));
                    }
                }
                None
            }
            DeclarationKind::Weapon { actor, .. } => {
                if !weapons.insert(actor.clone()) {
                    diagnostics.push(binding(
                        format!("Character '{actor}' already has a weapon."),
                        loc,
                    ));
                }
                Some(actor)
            }
            DeclarationKind::Cartridge { actor, .. } => {
                if !cartridges.insert(actor.clone()) {
                    diagnostics.push(binding(
                        format!("Multiple cartridge declarations for '{actor}' are not supported."),
                        loc,
                    ));
                }
                Some(actor)
            }
            DeclarationKind::Active { actor } => {
                if active {
                    diagnostics.push(binding("Declare active exactly once.".into(), loc));
                }
                active = true;
                Some(actor)
            }
            DeclarationKind::Stats { actor, .. }
            | DeclarationKind::Incoming { actor, .. }
            | DeclarationKind::Interrupt { actor, .. }
            | DeclarationKind::Combat { actor, .. } => Some(actor),
            DeclarationKind::Target { .. }
            | DeclarationKind::Hurt { .. }
            | DeclarationKind::Dummy { .. } => None,
        };
        if let Some(owner) = owner
            && !actors.contains(owner)
        {
            diagnostics.push(binding(
                format!("Declare '{owner} char ...;' before using that character."),
                loc,
            ));
        }
    }
    validate_action_owners(&script.statements, &actors, &mut diagnostics, 0);
    super::symbols::validate_symbols(script, &mut diagnostics);
    diagnostics.sort_by_key(|diagnostic| diagnostic.loc.offset);
    diagnostics
}

fn validate_action_owners(
    statements: &[Statement],
    actors: &BTreeSet<String>,
    diagnostics: &mut Vec<Diagnostic>,
    depth: usize,
) {
    for statement in statements {
        if depth >= 128 {
            diagnostics.push(binding(
                "Statement nesting exceeds its validation limit.".into(),
                &statement.loc,
            ));
            return;
        }
        match &statement.kind {
            StatementKind::ActionList { actor, .. } => {
                if !actors.contains(actor) {
                    diagnostics.push(binding(
                        format!("Unknown action character '{actor}'."),
                        &statement.loc,
                    ));
                }
            }
            StatementKind::If {
                then_branch,
                else_branch,
                ..
            } => {
                validate_action_owners(then_branch, actors, diagnostics, depth + 1);
                if let Some(branch) = else_branch {
                    validate_action_owners(branch, actors, diagnostics, depth + 1);
                }
            }
            StatementKind::Repeat { body, .. } | StatementKind::While { body, .. } => {
                validate_action_owners(body, actors, diagnostics, depth + 1);
            }
            StatementKind::For(loop_) => {
                validate_action_owners(&loop_.body, actors, diagnostics, depth + 1)
            }
            _ => {}
        }
    }
}
