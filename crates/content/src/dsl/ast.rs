//! AST definitions for Neverness to Everness (NTE) domain-specific language.

use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
    pub offset: usize,
    /// Source range in UTF-16 code units, matching browser editor coordinates.
    pub length: usize,
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Script {
    pub options: Vec<OptionPair>,
    pub declarations: Vec<Declaration>,
    pub statements: Vec<Statement>,
}

pub enum ScriptItem<'a> {
    Declaration(&'a Declaration),
    Statement(&'a Statement),
}

impl ScriptItem<'_> {
    pub fn loc(&self) -> &SourceLocation {
        match self {
            Self::Declaration(value) => &value.loc,
            Self::Statement(value) => &value.loc,
        }
    }
}

impl Script {
    /// Recover authored order, including constants between configuration declarations.
    pub fn items(&self) -> Vec<ScriptItem<'_>> {
        let mut items: Vec<_> = self
            .declarations
            .iter()
            .map(ScriptItem::Declaration)
            .chain(self.statements.iter().map(ScriptItem::Statement))
            .collect();
        items.sort_by_key(|item| item.loc().offset);
        items
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct OptionPair {
    pub key: String,
    pub value: Expression,
    pub loc: SourceLocation,
    pub value_range: SourceLocation,
}

#[derive(Clone, Debug, PartialEq)]
pub enum AwakeningSelection {
    Rank(u32),
    Explicit(Vec<u32>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct LevelPair {
    pub current: u32,
    pub cap: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TalentTriple {
    pub normal: Expression,
    pub skill: Expression,
    pub burst: Expression,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DeclarationKind {
    Character {
        actor: String,
        level: Option<LevelPair>,
        awakening: Option<AwakeningSelection>,
        talents: Option<Box<TalentTriple>>,
        qte: Option<Box<Expression>>,
        extra_fields: Vec<(String, Expression)>,
    },
    Weapon {
        actor: String,
        weapon_id: String,
        refinement: Option<u32>,
        level: Option<LevelPair>,
        extra_fields: Vec<(String, Expression)>,
    },
    Cartridge {
        actor: String,
        cartridge_id: String,
        count: u32,
    },
    Stats {
        actor: String,
        pairs: Vec<(String, Expression)>,
    },
    Target {
        pairs: Vec<(String, Expression)>,
    },
    Active {
        actor: String,
    },
    Hurt {
        pairs: Vec<(String, Expression)>,
    },
    Incoming {
        actor: String,
        pairs: Vec<(String, Expression)>,
    },
    Dummy {
        kind: String,
        target: Option<String>,
        pairs: Vec<(String, Expression)>,
    },
    Interrupt {
        actor: String,
        pairs: Vec<(String, Expression)>,
    },
    Combat {
        actor: String,
        pairs: Vec<(String, Expression)>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Declaration {
    pub kind: DeclarationKind,
    pub loc: SourceLocation,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ActionInvocation {
    pub action: String,
    pub params: Vec<(String, Expression)>,
    pub count: Option<Expression>,
    pub loc: SourceLocation,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ForStatement {
    pub var_name: String,
    pub init: Expression,
    pub condition: Expression,
    pub update_var: String,
    pub update_expr: Expression,
    pub body: Vec<Statement>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum StatementKind {
    TargetGauge {
        target: Expression,
        gauge: Option<Expression>,
    },
    ActionList {
        actor: String,
        actions: Vec<ActionInvocation>,
    },
    Variable {
        name: String,
        is_const: bool,
        value: Expression,
    },
    Assignment {
        target: String,
        value: Expression,
    },
    If {
        condition: Expression,
        then_branch: Vec<Statement>,
        else_branch: Option<Vec<Statement>>,
    },
    Repeat {
        count: Expression,
        body: Vec<Statement>,
    },
    While {
        condition: Expression,
        body: Vec<Statement>,
    },
    For(Box<ForStatement>),
    Expression(Expression),
    Break,
    Continue,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Statement {
    pub kind: StatementKind,
    pub loc: SourceLocation,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TimeUnit {
    Seconds,
    Milliseconds,
    Microseconds,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ExpressionKind {
    Number {
        /// Exact authored decimal, without a time suffix. Never round via f64.
        decimal: String,
        unit: Option<TimeUnit>,
    },
    StringLiteral(String),
    Boolean(bool),
    Identifier(String),
    FieldAccess(Vec<String>),
    Unary {
        op: String,
        expr: Box<Expression>,
    },
    Binary {
        op: String,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Call {
        callee: String,
        args: Vec<Expression>,
    },
    Range {
        min: Box<Expression>,
        max: Box<Expression>,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Expression {
    pub kind: ExpressionKind,
    pub loc: SourceLocation,
}
