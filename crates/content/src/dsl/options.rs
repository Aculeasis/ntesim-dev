//! Typed options compilation. This is a building block, not a full scenario compiler.

use std::collections::BTreeSet;

use super::{
    evaluate::{EmptyContext, Evaluator},
    validation::{Diagnostic, DiagnosticCode},
    *,
};

const STAMINA_KEYS: [&str; 4] = [
    "stamina_capacity",
    "initial_stamina",
    "stamina_use_efficiency",
    "stamina_recovery_rate",
];

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CriticalPolicy {
    Fixed,
    Random,
}

use crate::simulation::GameStopPolicy as StopPolicy;

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CompiledOptions {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub combat_stamina: Option<crate::simulation::CombatStaminaProfile>,
    pub trials: u64,
    pub relative_duration_us: u64,
    pub seed: u64,
    pub crit: CriticalPolicy,
    pub start_fons: Option<u64>,
    pub stop: StopPolicy,
    pub duration_specified: bool,
}

fn error(message: String, loc: &SourceLocation) -> Diagnostic {
    Diagnostic {
        code: DiagnosticCode::Configuration,
        message,
        loc: loc.clone(),
    }
}

/// Options are evaluated before declarations/constants and cannot observe combat state.
pub fn compile_options(script: &Script) -> Result<CompiledOptions, Vec<Diagnostic>> {
    let mut compiled = CompiledOptions {
        combat_stamina: None,
        trials: 1000,
        relative_duration_us: 0,
        seed: 42,
        crit: CriticalPolicy::Random,
        start_fons: None,
        stop: StopPolicy::Time,
        duration_specified: false,
    };
    let mut evaluator = Evaluator::default();
    let mut diagnostics = Vec::new();
    let mut keys = BTreeSet::new();
    let mut stamina = std::collections::BTreeMap::new();
    for option in &script.options {
        if !keys.insert(option.key.as_str()) {
            diagnostics.push(error(
                format!("Duplicate option '{}'.", option.key),
                &option.loc,
            ));
            continue;
        }
        let result = if STAMINA_KEYS.contains(&option.key.as_str()) {
            stamina_option(&mut stamina, option, &mut evaluator)
        } else {
            apply_option(&mut compiled, option, &mut evaluator)
        };
        if let Err(diagnostic) = result {
            diagnostics.push(diagnostic);
        }
    }
    match stamina_profile(&stamina, script) {
        Ok(profile) => compiled.combat_stamina = profile,
        Err(diagnostic) => diagnostics.push(diagnostic),
    }
    if compiled.stop == StopPolicy::Time && !compiled.duration_specified {
        let loc = script
            .options
            .first()
            .map(|o| &o.loc)
            .or_else(|| script.declarations.first().map(|d| &d.loc))
            .or_else(|| script.statements.first().map(|s| &s.loc))
            .cloned()
            .unwrap_or(SourceLocation {
                line: 1,
                column: 1,
                offset: 0,
                length: 0,
            });
        diagnostics.push(error("options requires duration in seconds.".into(), &loc));
    }
    if compiled.stop == StopPolicy::Victory && !compiled.duration_specified {
        compiled.relative_duration_us = crate::execution::ExecutionLimits::default().max_horizon_us;
    }
    if diagnostics.is_empty() {
        Ok(compiled)
    } else {
        Err(diagnostics)
    }
}

fn stamina_option<'a>(
    values: &mut std::collections::BTreeMap<&'a str, crate::formula::Ratio>,
    option: &'a OptionPair,
    evaluator: &mut Evaluator,
) -> Result<(), Diagnostic> {
    let value = evaluator
        .evaluate(&option.value, &EmptyContext)
        .map_err(Diagnostic::from)?;
    let (numerator, denominator) = value
        .number()
        .and_then(|number| number.to_safe_ratio())
        .map_err(|message| error(message, &option.value.loc))?;
    values.insert(
        &option.key,
        crate::formula::Ratio {
            numerator,
            denominator,
        },
    );
    Ok(())
}

fn stamina_profile(
    values: &std::collections::BTreeMap<&str, crate::formula::Ratio>,
    script: &Script,
) -> Result<Option<crate::simulation::CombatStaminaProfile>, Diagnostic> {
    if values.is_empty() {
        return Ok(None);
    }
    let loc = &script.options[0].loc;
    if values.len() != STAMINA_KEYS.len() {
        return Err(error("combat stamina requires stamina_capacity, initial_stamina, stamina_use_efficiency and stamina_recovery_rate".into(), loc));
    }
    let profile = crate::simulation::CombatStaminaProfile {
        capacity: values["stamina_capacity"],
        initial: values["initial_stamina"],
        use_efficiency: values["stamina_use_efficiency"],
        recovery_rate: values["stamina_recovery_rate"],
    };
    profile.validate().map_err(|message| error(message, loc))?;
    Ok(Some(profile))
}

fn apply_option(
    compiled: &mut CompiledOptions,
    option: &OptionPair,
    evaluator: &mut Evaluator,
) -> Result<(), Diagnostic> {
    let key = option.key.as_str();
    let fail = |message: String| error(message, &option.value.loc);
    if key == "crit" || key == "stop" {
        let name = match &option.value.kind {
            ExpressionKind::Identifier(name) | ExpressionKind::StringLiteral(name) => name.as_str(),
            _ => return Err(fail(format!("{key} requires a named policy."))),
        };
        match (key, name) {
            ("crit", "fixed") => compiled.crit = CriticalPolicy::Fixed,
            ("crit", "random") => compiled.crit = CriticalPolicy::Random,
            ("stop", "time") => compiled.stop = StopPolicy::Time,
            ("stop", "victory") => compiled.stop = StopPolicy::Victory,
            ("crit", _) => return Err(fail("crit must be fixed or random.".into())),
            _ => return Err(fail("stop must be time or victory.".into())),
        }
        return Ok(());
    }
    if !["iteration", "duration", "seed", "start_fons"].contains(&key) {
        return Err(error(format!("Unsupported option '{key}'."), &option.loc));
    }
    let value = evaluator
        .evaluate(&option.value, &EmptyContext)
        .map_err(Diagnostic::from)?;
    let number = value.number().map_err(fail)?;
    let integer = if key == "duration" {
        number.to_microseconds()
    } else {
        number.to_safe_integer()
    }
    .map_err(fail)?;
    if (key == "duration" || key == "iteration") && integer == 0 {
        return Err(fail(format!("{key} must be positive.")));
    }
    match key {
        "iteration" => compiled.trials = integer,
        "duration" => {
            compiled.relative_duration_us = integer;
            compiled.duration_specified = true;
        }
        "seed" => compiled.seed = integer,
        "start_fons" => compiled.start_fons = Some(integer),
        _ => unreachable!("supported numeric option"),
    }
    Ok(())
}
