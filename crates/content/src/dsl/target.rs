//! Compile an authored dummy target into the existing native request schema.

use std::collections::BTreeMap;

use ntesim_core::formula::Ratio;

use super::{
    evaluate::{Evaluator, ExpressionContext, Value},
    number::ExactNumber,
    validation::{Diagnostic, DiagnosticCode},
    *,
};
use crate::simulation::GameTargetConfig;

const ELEMENTS: [(&str, &str); 6] = [
    ("anima", "NATURE"),
    ("chaos", "CHAOS"),
    ("cosmos", "COSMOS"),
    ("incantation", "INCANTATION"),
    ("lakshana", "LAKSHANA"),
    ("psyche", "PSYCHE"),
];

pub fn compile_target(
    declaration: &Declaration,
    target_index: usize,
    context: &impl ExpressionContext,
    evaluator: &mut Evaluator,
) -> Result<GameTargetConfig, Diagnostic> {
    let DeclarationKind::Target { pairs } = &declaration.kind else {
        return Err(configuration(
            "Expected a target declaration.".into(),
            &declaration.loc,
        ));
    };
    let fields = Fields {
        pairs,
        loc: &declaration.loc,
    };
    let mut values = BTreeMap::new();
    for (key, expression) in pairs {
        if ![
            "hp",
            "max_hp",
            "def",
            "base_def",
            "original_atk",
            "control_immune",
            "resist",
            "boss",
            "lvl",
            "break_gauge",
            "max_break_gauge",
            "broken_duration",
            "discord_fraction",
        ]
        .contains(&key.as_str())
            && !ELEMENTS.iter().any(|(name, _)| name == key)
        {
            return Err(fields.error(key, format!("Unsupported target field '{key}'.")));
        }
        let value = evaluator
            .evaluate(expression, context)
            .map_err(Diagnostic::from)?;
        if values.insert(key.as_str(), value).is_some() {
            return Err(fields.error(key, format!("Duplicate target field '{key}'.")));
        }
    }
    let hp = fields.integer(&values, "hp")?;
    let max_hp = if values.contains_key("max_hp") {
        fields.integer(&values, "max_hp")?
    } else {
        hp
    };
    if hp == 0 || max_hp < hp {
        return Err(fields.error("hp", "target requires hp > 0 and max_hp >= hp.".into()));
    }
    let defense = fields.integer(&values, "def")?;
    let resistance = fields.ratio(&values, "resist")?;
    let is_boss = match values.get("boss") {
        None => false,
        Some(Value::Boolean(value)) => *value,
        _ => return Err(fields.error("boss", "target boss must be true or false.".into())),
    };
    let level = if values.contains_key("lvl") {
        u32::try_from(fields.integer(&values, "lvl")?)
            .map_err(|_| fields.error("lvl", "Target level exceeds its integer range.".into()))?
    } else {
        80
    };
    let mut elemental_resistances = BTreeMap::new();
    for (name, element) in ELEMENTS {
        if values.contains_key(name) {
            elemental_resistances.insert(element.into(), fields.ratio(&values, name)?);
        }
    }
    let target = GameTargetConfig {
        original_atk: values
            .contains_key("original_atk")
            .then(|| fields.integer(&values, "original_atk"))
            .transpose()?,
        base_defense: values
            .contains_key("base_def")
            .then(|| fields.integer(&values, "base_def"))
            .transpose()?,
        control_immune: match values.get("control_immune") {
            None => false,
            Some(Value::Boolean(value)) => *value,
            _ => {
                return Err(fields.error(
                    "control_immune",
                    "target control_immune must be true or false.".into(),
                ));
            }
        },
        target_index,
        is_boss,
        level,
        hp_max: max_hp,
        hp_current: hp,
        defense,
        resistance,
        elemental_resistances,
        airborne_until_us: 0,
        break_profile: fields.break_profile(&values, target_index)?,
    };
    Ok(target)
}

fn configuration(message: String, loc: &SourceLocation) -> Diagnostic {
    Diagnostic {
        code: DiagnosticCode::Configuration,
        message,
        loc: loc.clone(),
    }
}

struct Fields<'a> {
    pairs: &'a [(String, Expression)],
    loc: &'a SourceLocation,
}

impl Fields<'_> {
    fn break_profile(
        &self,
        values: &BTreeMap<&str, Value>,
        target_index: usize,
    ) -> Result<Option<crate::simulation::GameTargetBreakProfileConfig>, Diagnostic> {
        let keys = ["break_gauge", "max_break_gauge", "broken_duration"];
        if !keys.iter().any(|key| values.contains_key(key)) {
            if values.contains_key("discord_fraction") {
                return Err(self.error(
                    "discord_fraction",
                    "target discord_fraction requires an explicit Break gauge profile.".into(),
                ));
            }
            return Ok(None);
        }
        let discord_fraction = if values.contains_key("discord_fraction") {
            let fraction = self.ratio(values, "discord_fraction")?;
            let exact = crate::simulation::ExactFraction::from_ratio(fraction)
                .map_err(|message| self.error("discord_fraction", message))?;
            if exact.numerator == 0 || exact.numerator > exact.denominator {
                return Err(self.error(
                    "discord_fraction",
                    "target discord_fraction must be in (0, 1].".into(),
                ));
            }
            Some(fraction)
        } else {
            None
        };
        let profile = crate::simulation::GameTargetBreakProfileConfig {
            capacity: self.ratio(values, "max_break_gauge")?,
            current_gauge: self.ratio(values, "break_gauge")?,
            broken_duration_us: self
                .numeric(values, "broken_duration")?
                .to_microseconds()
                .map_err(|message| self.error("broken_duration", message))?,
            discord_fraction,
        };
        crate::simulation::validate_target_break_profile(&profile, target_index)
            .map_err(|message| self.error("break_gauge", message))?;
        Ok(Some(profile))
    }

    fn error(&self, key: &str, message: String) -> Diagnostic {
        let loc = self
            .pairs
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, expression)| &expression.loc)
            .unwrap_or(self.loc);
        configuration(message, loc)
    }

    fn numeric<'a>(
        &self,
        values: &'a BTreeMap<&str, Value>,
        key: &str,
    ) -> Result<&'a ExactNumber, Diagnostic> {
        values
            .get(key)
            .ok_or_else(|| self.error(key, format!("target requires {key}.")))?
            .number()
            .map_err(|message| self.error(key, message))
    }

    fn integer(&self, values: &BTreeMap<&str, Value>, key: &str) -> Result<u64, Diagnostic> {
        self.numeric(values, key)?
            .to_safe_integer()
            .map_err(|message| self.error(key, message))
    }

    fn ratio(&self, values: &BTreeMap<&str, Value>, key: &str) -> Result<Ratio, Diagnostic> {
        let (numerator, denominator) = self
            .numeric(values, key)?
            .to_safe_ratio()
            .map_err(|message| self.error(key, message))?;
        Ok(Ratio {
            numerator,
            denominator,
        })
    }
}
