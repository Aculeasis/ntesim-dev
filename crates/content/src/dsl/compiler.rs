//! Shared Rust source compiler for the existing bounded research scenario adapter.

use super::{
    configuration::{CompiledConfiguration, failure},
    inspection::PartyInspection,
    number::ExactNumber,
    options::CriticalPolicy,
    rotation::RotationCompiler,
    validation::Diagnostic,
    *,
};
use crate::simulation::{GameScenarioProgramRequest, GameScenarioRequest};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields, rename_all = "camelCase")]
pub struct CompileLimits {
    pub max_steps: usize,
    pub max_actions: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SourceRequest {
    pub source: String,
    #[serde(default)]
    pub limits: CompileLimits,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SourceResponse {
    pub compiled: Option<CompiledScript>,
    pub party: Option<PartyInspection>,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn compile_response(request: &SourceRequest) -> SourceResponse {
    let (script, config) = match super::inspection::prepare(&request.source) {
        Ok(prepared) => prepared,
        Err(diagnostics) => {
            return SourceResponse {
                compiled: None,
                party: None,
                diagnostics,
            };
        }
    };
    let party = Some(PartyInspection::from_configuration(&config));
    match compile(&script, config, request.limits) {
        Ok(compiled) => SourceResponse {
            compiled: Some(compiled),
            party,
            diagnostics: Vec::new(),
        },
        Err(diagnostic) => SourceResponse {
            compiled: None,
            party,
            diagnostics: vec![diagnostic],
        },
    }
}

impl Default for CompileLimits {
    fn default() -> Self {
        Self {
            max_steps: 60_000,
            max_actions: 10_000,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompiledScript {
    pub request: GameScenarioRequest,
    pub trials: u64,
    pub display: Value,
    pub warnings: Vec<String>,
    pub language: String,
    pub option_ranges: Vec<OptionRange>,
    pub options: Value,
    pub steps: usize,
    pub action_locations: Vec<SourceLocation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub incoming_damage_locations: Vec<SourceLocation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub live_program: Option<GameScenarioProgramRequest>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OptionRange {
    pub name: String,
    pub start: usize,
    pub value_start: usize,
    pub end: usize,
}

pub fn compile_scenario(
    source: &str,
    limits: CompileLimits,
) -> Result<CompiledScript, Vec<Diagnostic>> {
    let (script, config) = super::inspection::prepare(source)?;
    compile(&script, config, limits).map_err(|error| vec![error])
}

pub fn validate_script(source: &str) -> Vec<Diagnostic> {
    compile_scenario(source, CompileLimits::default())
        .err()
        .unwrap_or_default()
}

fn compile(
    script: &Script,
    config: CompiledConfiguration,
    limits: CompileLimits,
) -> Result<CompiledScript, Diagnostic> {
    let loc = &script.options[0].loc;
    if limits.max_steps == 0
        || limits.max_steps > 1_000_000
        || limits.max_actions == 0
        || limits.max_actions > 100_000
    {
        return Err(failure("Invalid source compilation limits.".into(), loc));
    }
    let (display, mut warnings) = display(&config).map_err(|message| failure(message, loc))?;
    let request = scenario_request(&config);
    let mut compiler = RotationCompiler::new(&config, request, limits);
    compiler.run(&config.rotation)?;
    warnings.append(&mut compiler.warnings);
    let steps = compiler.steps();
    let locations = compiler.locations;
    let mut request = compiler.request;
    if let Some((projected, _)) = crate::simulation::project_talent_inputs(&request)
        .map_err(|message| failure(message, loc))?
    {
        // Native execution still projects action coefficients itself. Retain the
        // same authored action templates while validating purchases and metadata.
        request.model_assumptions = projected.model_assumptions;
    }
    crate::simulation::validate_game_request(
        &request,
        &crate::execution::ExecutionLimits::default(),
    )
    .map_err(|message| failure(message, loc))?;
    let live_program = if compiler.branches.is_empty() && compiler.energy_waits.is_empty() {
        None
    } else {
        Some(GameScenarioProgramRequest {
            request: request.clone(),
            branches: compiler.branches,
            energy_waits: compiler.energy_waits,
        })
    };
    let mut options = json!({ "iteration":config.options.trials, "duration":config.options.relative_duration_us as f64 / 1e6,
        "seed":config.options.seed, "crit":config.options.crit, "start_fons":config.options.start_fons });
    if let Some(profile) = &config.options.combat_stamina {
        options["combat_stamina"] =
            serde_json::to_value(profile).map_err(|error| failure(error.to_string(), loc))?;
    }
    Ok(CompiledScript {
        request,
        trials: config.options.trials,
        display,
        warnings,
        language: "nte-gcsl-1".into(),
        option_ranges: script
            .options
            .iter()
            .map(|option| OptionRange {
                name: option.key.clone(),
                start: option.loc.offset,
                value_start: option.value_range.offset,
                end: option.value_range.offset + option.value_range.length,
            })
            .collect(),
        options,
        steps,
        action_locations: locations,
        incoming_damage_locations: config
            .incoming_damage
            .iter()
            .map(|(_, loc)| loc.clone())
            .collect(),
        live_program,
    })
}

/// Configuration-only request shared by scheduled export and live execution.
pub(crate) fn scenario_request(config: &CompiledConfiguration) -> GameScenarioRequest {
    let mut request = GameScenarioRequest {
        clock_contract: Default::default(),
        timer_policy: Default::default(),
        absolute_limit_us: crate::execution::ExecutionLimits::default().max_horizon_us,
        schema_version: 1,
        action_admission_order: Default::default(),
        source_exhausted: false,
        source_halt: None,
        input_time_contexts: Vec::new(),
        content_id: "ntesim-current".into(),
        model_assumptions: config.model_assumptions.clone(),
        relative_duration_us: config.options.relative_duration_us,
        stop: config.options.stop,
        start_fons: config.options.start_fons,
        combat_stamina: config.options.combat_stamina.clone(),
        seed: config.options.seed,
        stochastic_crit: None,
        action_metadata: Default::default(),
        actors: config.actors.iter().map(|a| a.actor.clone()).collect(),
        active_actor_index: config.active_actor_index,
        targets: config.targets.clone(),
        incoming_damage_events: config
            .incoming_damage
            .iter()
            .map(|(event, _)| event.clone())
            .collect(),
        hp_cost_events: Vec::new(),
        interrupt_events: config.interrupt_events.clone(),
        tenacity_events: Vec::new(),
        parry_events: Vec::new(),
        combat_transition_events: config.combat_transition_events.clone(),
        dummy_combat_events: config.dummy_combat.clone(),
        incoming_dummy_sources: config.incoming_dummy_sources.clone(),
        incoming_damage_basis: config.incoming_damage_basis.clone(),
        plunge_inputs: Default::default(),
        approach_inputs: Default::default(),
        hold_inputs: Default::default(),
        branch_inputs: Default::default(),
        dash_directions: Default::default(),
        aim_targets: Default::default(),
        actions: Vec::new(),
    };
    if config.options.crit == CriticalPolicy::Random {
        for actor in &config.actors {
            let policy = actor.projection.stochastic_crit.clone();
            if let Some(existing) = &mut request.stochastic_crit {
                existing.actor_sequences.extend(policy.actor_sequences);
                existing.eligible_effects.extend(policy.eligible_effects);
            } else {
                request.stochastic_crit = Some(policy);
            }
        }
    }
    request
}

fn display(config: &CompiledConfiguration) -> Result<(Value, Vec<String>), String> {
    let mut actors = serde_json::Map::new();
    let mut warnings = Vec::new();
    for actor in &config.actors {
        let mut contributions = serde_json::Map::new();
        for (key, ratio) in &actor.contributions {
            let value = ExactNumber::from(0)
                .add(&ExactNumber::decimal(&ratio.numerator.to_string())?)?
                .divide(&ExactNumber::decimal(&ratio.denominator.to_string())?)?
                .to_display_number()?;
            contributions.insert(super::actor::stat_label(key).into(), json!(value));
        }
        let mut item = json!({ "maxLevel":actor.actor.character_max_level,"snapshot":false,"contributions":contributions,"base":actor.projection.base });
        if actor.actor.set_id.is_some() {
            // Count is presentation metadata; the projector already owns effect thresholds.
            item["setCount"] = json!(actor.set_count);
        }
        if !actor.projection.unmodeled_attributes.is_empty() {
            item["unmodeledAttributes"] = json!(actor.projection.unmodeled_attributes);
            warnings.push(format!(
                "{}: native properties retained without a combat adapter: {}.",
                actor.name,
                actor
                    .projection
                    .unmodeled_attributes
                    .keys()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }
        actors.insert(actor.name.clone(), item);
    }
    Ok((
        json!({ "actors":actors,"targets":config.targets.iter().map(|target| json!({"level":target.level})).collect::<Vec<_>>(),
        "editor":{"font":14,"theme":"tomorrow-night","wrap":false} }),
        warnings,
    ))
}
