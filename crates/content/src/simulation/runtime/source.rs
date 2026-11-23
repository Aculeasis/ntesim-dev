//! Per-trial source execution over the existing combat scheduler and admission.

use super::{PreparedGameScenario, execution::PreparedExecution};
use crate::dsl::{
    SourceLocation,
    compiler::{CompileLimits, scenario_request},
    configuration::{CompiledConfiguration, failure},
    evaluate::Value,
    live::{Input, Occurrence, Rotation},
    number::ExactNumber,
    validation::{Diagnostic, DiagnosticCode},
};
use crate::execution::ExecutionLimits;
use crate::simulation::{
    GameActionCommand, GameScenarioRequest, SourceOccurrence, SourceOccurrenceKind,
    SourceRunResponse, SourceScenarioReport,
};
use std::collections::BTreeMap;

mod actions;
mod diagnostics;
mod lifecycle;
pub use lifecycle::{SourceBoundary, SourceCheckpoint, SourceControlReport};
mod observation;
mod targets;
mod time;
mod windows;

#[cfg(test)]
#[path = "../tests/source_scheduler.rs"]
mod scheduler_tests;

pub fn run_source(
    request: &crate::dsl::compiler::SourceRequest,
    limits: &ExecutionLimits,
) -> Result<SourceRunResponse, String> {
    match PreparedSourceScenario::new(&request.source, request.limits, limits) {
        Ok(program) => Ok(SourceRunResponse {
            execution: Some(program.run(program.seed())?),
            diagnostics: Vec::new(),
        }),
        Err(diagnostics) => Ok(SourceRunResponse {
            execution: None,
            diagnostics,
        }),
    }
}

pub fn run_compiled_source(
    artifact: &crate::dsl::source::SourceCompileResponse,
    limits: &ExecutionLimits,
) -> Result<SourceRunResponse, String> {
    if !artifact.diagnostics.is_empty() {
        return Err("Cannot execute a source artifact with compilation diagnostics.".into());
    }
    let compiled = artifact
        .compiled
        .as_ref()
        .ok_or("Source artifact has no compiled program.")?;
    let program = PreparedSourceScenario::from_export(compiled, limits)?;
    let expected_party =
        crate::dsl::inspection::PartyInspection::from_configuration(&program.config);
    if serde_json::to_value(&artifact.party).map_err(|e| e.to_string())?
        != serde_json::to_value(Some(expected_party)).map_err(|e| e.to_string())?
    {
        return Err("Compiled source party inspection differs from its authored input.".into());
    }
    if artifact.options.as_ref() != Some(&program.config.options) {
        return Err("Compiled source options differ from its authored input.".into());
    }
    Ok(SourceRunResponse {
        execution: Some(program.run(program.seed())?),
        diagnostics: Vec::new(),
    })
}

pub struct PreparedSourceScenario {
    source: String,
    options_location: SourceLocation,
    config: CompiledConfiguration,
    request: GameScenarioRequest,
    source_limits: CompileLimits,
    limits: ExecutionLimits,
    damage_bonuses: std::sync::Arc<[crate::simulation::hit::PreparedDamageBonuses]>,
    descriptor_ids: Vec<std::collections::BTreeMap<String, usize>>,
    action_cache:
        std::sync::RwLock<ntesim_core::fast_hash::FastHashMap<ActionCacheKey, CachedAction>>,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) struct ActionCacheKey {
    pub(crate) actor: usize,
    pub(crate) command: GameActionCommand,
    pub(crate) descriptor: usize,
    pub(crate) approach: bool,
    pub(crate) plunge: Option<crate::simulation::GamePlungeInput>,
    pub(crate) target: Option<usize>,
    pub(crate) direction: Option<crate::simulation::GameDashDirection>,
    pub(crate) hold: Option<crate::simulation::GameHoldInput>,
    pub(crate) branch: Option<crate::simulation::GameBranchInput>,
    pub(crate) critical: bool,
    pub(crate) parry: bool,
}

type CachedAction = Result<std::sync::Arc<ResolvedActionDescriptor>, String>;

#[derive(Clone, Debug)]
pub(crate) struct ResolvedActionDescriptor {
    pub(crate) projected_command: GameActionCommand,
    pub(crate) replay_command: GameActionCommand,
    pub(crate) packet: Option<crate::simulation::metadata::ActionPacketMetadata>,
    pub(crate) talent_receipts: Vec<serde_json::Value>,
    pub(crate) model_assumptions: std::collections::BTreeMap<String, String>,
}

impl PreparedSourceScenario {
    pub fn new(
        source: &str,
        source_limits: CompileLimits,
        limits: &ExecutionLimits,
    ) -> Result<Self, Vec<Diagnostic>> {
        let (script, config) = crate::dsl::inspection::prepare(source)?;
        let loc = &script.options[0].loc;
        Self::from_configuration(source, config, source_limits, limits, loc)
            .map_err(|diagnostic| vec![diagnostic])
    }

    pub(crate) fn from_configuration(
        source: &str,
        config: CompiledConfiguration,
        source_limits: CompileLimits,
        limits: &ExecutionLimits,
        loc: &SourceLocation,
    ) -> Result<Self, Diagnostic> {
        #[cfg(feature = "performance-diagnostics")]
        let _span = crate::performance::Span::new("source.from_configuration");
        if source_limits.max_steps == 0
            || source_limits.max_steps > 1_000_000
            || source_limits.max_actions == 0
            || source_limits.max_actions > 100_000
        {
            return Err(failure("Invalid source compilation limits.".into(), loc));
        }
        {
            #[cfg(feature = "performance-diagnostics")]
            let _span = crate::performance::Span::new("source.rotation_validation");
            crate::dsl::source::validate_rotation(&config)?;
        }
        let mut request = scenario_request(&config);
        request.absolute_limit_us = limits.max_horizon_us;
        if !config.options.duration_specified {
            request.relative_duration_us = limits.max_horizon_us;
        }
        request.action_admission_order =
            crate::simulation::GameActionAdmissionOrder::AfterSettledEvents;
        let required_actions = actions::declare_assumptions(&config, &mut request)?;
        {
            #[cfg(feature = "performance-diagnostics")]
            let _span = crate::performance::Span::new("source.talent_projection");
            if let Some((projected, _)) = crate::simulation::project_talent_inputs(&request)
                .map_err(|message| failure(message, loc))?
            {
                request = projected;
            }
        }
        {
            #[cfg(feature = "performance-diagnostics")]
            let _span = crate::performance::Span::new("source.request_validation");
            crate::simulation::validate_game_request(&request, limits)
                .map_err(|message| failure(message, loc))?;
        }
        let mut initial_cache = ntesim_core::fast_hash::FastHashMap::default();
        let mut descriptor_ids = Vec::with_capacity(config.actors.len());
        let initial_state = (!required_actions.is_empty()).then(|| {
            crate::simulation::state::GameSimulatorState::new_with_seed(&request, request.seed)
        });
        for (actor_index, actor) in config.actors.iter().enumerate() {
            let mut ids = std::collections::BTreeMap::new();
            for (descriptor_id, (name, descriptor)) in
                actor.projection.public_actions.iter().enumerate()
            {
                ids.insert(name.clone(), descriptor_id);
                if !required_actions.contains(&(actor_index, name.as_str()))
                    || initial_cache.len() >= source_limits.max_actions
                {
                    continue;
                }
                #[cfg(feature = "performance-diagnostics")]
                let _span = crate::performance::Span::new("source.eager_descriptor");
                let mut command = match crate::registry::source_command(
                    &request.actors[actor_index],
                    &initial_state
                        .as_ref()
                        .expect("required action has initial state")
                        .actors[actor_index],
                    &descriptor.command,
                ) {
                    Ok(cmd) => cmd,
                    Err(_) => continue,
                };
                match &mut command {
                    GameActionCommand::InvokeAbility {
                        timestamp_us,
                        actor_index: idx,
                        ..
                    }
                    | GameActionCommand::PlayerInput {
                        timestamp_us,
                        actor_index: idx,
                        ..
                    } => {
                        *timestamp_us = 0;
                        *idx = actor_index;
                    }
                    GameActionCommand::SwitchActor {
                        timestamp_us,
                        target_actor_index,
                        ..
                    } => {
                        *timestamp_us = 0;
                        *target_actor_index = actor_index;
                    }
                    _ => continue,
                }
                let params = actions::Parameters::default();
                let key = ActionCacheKey {
                    actor: actor_index,
                    command: command.clone(),
                    descriptor: descriptor_id,
                    approach: params.approach,
                    plunge: params.plunge,
                    target: params.target,
                    direction: params.direction,
                    hold: params.hold,
                    branch: params.branch,
                    critical: params.critical,
                    parry: params.parry,
                };
                let resolution = Self::compute_action_resolution_static(
                    &request, limits, descriptor, &command, &params,
                );
                initial_cache.insert(key, resolution.map(std::sync::Arc::new));
            }
            descriptor_ids.push(ids);
        }
        let damage_bonuses = request
            .actors
            .iter()
            .map(crate::simulation::hit::PreparedDamageBonuses::new)
            .collect();
        Ok(Self {
            source: source.into(),
            options_location: loc.clone(),
            config,
            request,
            source_limits,
            limits: *limits,
            damage_bonuses,
            descriptor_ids,
            action_cache: std::sync::RwLock::new(initial_cache),
        })
    }

    pub(crate) fn resolve_action(
        &self,
        actor: usize,
        name: &str,
        descriptor: &crate::build::PublicAction,
        command: GameActionCommand,
        params: &actions::Parameters,
    ) -> CachedAction {
        #[cfg(feature = "performance-diagnostics")]
        let _span = crate::performance::Span::new("source.resolve_action");
        let key = ActionCacheKey {
            actor,
            command,
            // A name denotes one immutable descriptor within this owner and
            // prepared program, including its complete model assumptions.
            descriptor: self.descriptor_ids[actor][name],
            approach: params.approach,
            plunge: params.plunge,
            target: params.target,
            direction: params.direction,
            hold: params.hold,
            branch: params.branch,
            critical: params.critical,
            parry: params.parry,
        };
        {
            let cache = self
                .action_cache
                .read()
                .map_err(|_| "Action cache lock poisoned")?;
            if let Some(cached) = cache.get(&key) {
                return cached.clone();
            }
        }
        let resolution = self
            .compute_action_resolution(descriptor, &key.command, params)
            .map(std::sync::Arc::new);
        let mut cache = self
            .action_cache
            .write()
            .map_err(|_| "Action cache lock poisoned")?;
        if cache.len() < self.source_limits.max_actions {
            cache.insert(key, resolution.clone());
        }
        resolution
    }

    fn compute_action_resolution(
        &self,
        descriptor: &crate::build::PublicAction,
        command: &GameActionCommand,
        params: &actions::Parameters,
    ) -> Result<ResolvedActionDescriptor, String> {
        Self::compute_action_resolution_static(
            &self.request,
            &self.limits,
            descriptor,
            command,
            params,
        )
    }

    fn compute_action_resolution_static(
        request: &GameScenarioRequest,
        limits: &ExecutionLimits,
        descriptor: &crate::build::PublicAction,
        command: &GameActionCommand,
        params: &actions::Parameters,
    ) -> Result<ResolvedActionDescriptor, String> {
        #[cfg(feature = "performance-diagnostics")]
        let _span = crate::performance::Span::new("source.compute_descriptor");
        let mut selected = request.clone();
        crate::dsl::configuration::merge_assumptions(
            &mut selected.model_assumptions,
            &descriptor.model_assumptions,
        )?;
        if params.approach {
            selected.approach_inputs.insert(0);
        }
        if let Some(plunge) = params.plunge {
            selected.plunge_inputs.insert(0, plunge);
        }
        if let Some(target) = params.target {
            selected.aim_targets.insert(0, target);
        }
        if let Some(direction) = params.direction {
            selected.dash_directions.insert(0, direction);
        }
        if let Some(hold) = params.hold {
            if !matches!(command, GameActionCommand::InvokeAbility { .. })
                && !(matches!(command, GameActionCommand::SwitchActor { .. })
                    && params.branch.is_some())
            {
                return Err("hold requires an ability input.".into());
            }
            selected.hold_inputs.insert(0, hold);
        }
        if let Some(branch) = params.branch {
            selected.branch_inputs.insert(0, branch);
        }
        selected.actions.push(command.clone());
        if params.parry {
            selected
                .parry_events
                .push(crate::simulation::GameActorParryEvent::AfterAction { command_index: 0 });
        }
        let prepared = PreparedGameScenario::new(&selected, limits)?;
        Ok(ResolvedActionDescriptor {
            projected_command: prepared.request.actions[0].clone(),
            replay_command: selected.actions[0].clone(),
            packet: prepared.packets[0].clone(),
            talent_receipts: prepared.talent_receipts,
            model_assumptions: prepared
                .request
                .model_assumptions
                .into_iter()
                .filter(|(id, description)| request.model_assumptions.get(id) != Some(description))
                .collect(),
        })
    }

    pub fn seed(&self) -> u64 {
        self.request.seed
    }

    pub fn request(&self) -> &GameScenarioRequest {
        &self.request
    }

    pub fn matches_source(&self, request: &crate::dsl::compiler::SourceRequest) -> bool {
        self.source == request.source && self.source_limits == request.limits
    }

    pub fn compile_response(&self) -> crate::dsl::source::SourceCompileResponse {
        crate::dsl::source::SourceCompileResponse {
            compiled: Some(self.export()),
            party: Some(crate::dsl::inspection::PartyInspection::from_configuration(
                &self.config,
            )),
            options: Some(self.config.options.clone()),
            diagnostics: Vec::new(),
        }
    }

    pub fn scenario_assumptions(&self) -> crate::simulation::SourceScenarioAssumptions {
        use crate::simulation::{
            SourceFixedDamageAssumption, SourcePeriodicDamageAssumption, SourceScenarioAssumptions,
        };
        SourceScenarioAssumptions {
            duration_specified: self.config.options.duration_specified,
            periodic_damage: self
                .config
                .hurt_declarations
                .iter()
                .map(|declaration| SourcePeriodicDamageAssumption {
                    interval_min_us: declaration.interval_min_us,
                    interval_max_us: declaration.interval_max_us,
                    amount_min: declaration.amount_min,
                    amount_max: declaration.amount_max,
                    actor_index: (declaration.actor_index != usize::MAX)
                        .then_some(declaration.actor_index),
                })
                .collect(),
            fixed_damage: self
                .config
                .authored_incoming
                .iter()
                .map(|declaration| SourceFixedDamageAssumption {
                    event: declaration.event.clone(),
                    source_target: declaration.source_target,
                    basis: declaration.basis,
                })
                .collect(),
        }
    }

    pub fn export(&self) -> crate::dsl::source::CompiledSourceProgram {
        #[cfg(feature = "performance-diagnostics")]
        let _span = crate::performance::Span::new("source.export");
        crate::dsl::source::CompiledSourceProgram {
            schema_version: 1,
            engine_version: ntesim_contracts::ENGINE_VERSION.into(),
            engine_source_sha256: ntesim_contracts::ENGINE_SOURCE_SHA256.into(),
            input: crate::dsl::compiler::SourceRequest {
                source: self.source.clone(),
                limits: self.source_limits,
            },
            request: self.request.clone(),
            trials: self.config.options.trials,
        }
    }

    pub fn from_export(
        artifact: &crate::dsl::source::CompiledSourceProgram,
        limits: &ExecutionLimits,
    ) -> Result<Self, String> {
        #[cfg(feature = "performance-diagnostics")]
        let _span = crate::performance::Span::new("source.from_export");
        if artifact.schema_version != 1
            || artifact.engine_version != ntesim_contracts::ENGINE_VERSION
            || artifact.engine_source_sha256 != ntesim_contracts::ENGINE_SOURCE_SHA256
        {
            return Err("Compiled source engine/schema identity mismatch.".into());
        }
        let program = Self::new(&artifact.input.source, artifact.input.limits, limits).map_err(
            |diagnostics| {
                serde_json::to_string(&diagnostics)
                    .unwrap_or_else(|_| "Invalid compiled source.".into())
            },
        )?;
        if serde_json::to_vec(&program.request).map_err(|e| e.to_string())?
            != serde_json::to_vec(&artifact.request).map_err(|e| e.to_string())?
            || program.config.options.trials != artifact.trials
        {
            return Err("Compiled source configuration differs from its authored input.".into());
        }
        Ok(program)
    }

    /// Fresh lexical state, event queue, combat resources and RNG for each seed.
    pub fn run(&self, seed: u64) -> Result<SourceScenarioReport, String> {
        self.run_sample(seed)
    }

    pub fn run_sample(&self, seed: u64) -> Result<SourceScenarioReport, String> {
        Ok(self.run_controlled(seed, None, |_| Ok(false))?.execution)
    }

    /// Summarize a successful trial and rerun failed seeds for inspection.
    pub fn run_batch(&self, seed: u64) -> Result<super::SourceTrialOutcome, String> {
        let (trial, assumptions, _) = self.execute_trial(seed, None, false, |_| Ok(false))?;
        if trial.diagnostic.is_none()
            && let Some(summary) = trial
                .execution
                .into_batch(&trial.request, seed, assumptions)?
        {
            return Ok(super::SourceTrialOutcome::Completed(summary));
        }
        Ok(super::SourceTrialOutcome::Failed(Box::new(
            self.run_sample(seed)?,
        )))
    }

    /// Cooperatively stop between complete authored inputs. Resume reconstructs
    /// the scheduler by deterministic replay, then continues the same trial.
    pub fn run_controlled(
        &self,
        seed: u64,
        resume: Option<&SourceCheckpoint>,
        cancel: impl FnMut(SourceBoundary) -> Result<bool, String>,
    ) -> Result<SourceControlReport, String> {
        let (mut trial, lifecycle, checkpoint) = self.execute_trial(seed, resume, true, cancel)?;
        let failure_context = trial.failure_context();
        let mut report = trial
            .execution
            .into_report(&trial.request, seed, lifecycle)?;
        report.talent_projection_receipts = trial.talents;
        trial.request.actions = trial.replay_actions;
        Ok(SourceControlReport {
            execution: SourceScenarioReport {
                source: self.source.clone(),
                request: trial.request,
                report,
                action_locations: trial.locations,
                occurrences: trial.occurrences,
                diagnostic: trial.diagnostic,
                failure_context,
            },
            checkpoint,
        })
    }

    fn execute_trial(
        &self,
        seed: u64,
        resume: Option<&SourceCheckpoint>,
        record_timeline: bool,
        mut cancel: impl FnMut(SourceBoundary) -> Result<bool, String>,
    ) -> Result<
        (
            Trial<'_>,
            BTreeMap<String, String>,
            Option<SourceCheckpoint>,
        ),
        String,
    > {
        if let Some(checkpoint) = resume {
            checkpoint.validate(self, seed)?;
        }
        let mut request = self.request.clone();
        request.seed = seed;
        if seed != self.request.seed && self.config.has_stochastic_hurt() {
            let (events, sources, bases) = self.config.sample_incoming_for_seed(seed)?;
            request.incoming_damage_events = events;
            request.incoming_dummy_sources = sources;
            request.incoming_damage_basis = bases;
        }
        let (mut execution, lifecycle) =
            PreparedExecution::new_with_timeline_option(&request, seed, &[], &[], record_timeline)?;
        execution.state.damage_bonuses = Some(self.damage_bonuses.clone());
        execution.live_source = true;
        let mut trial = Trial {
            program: self,
            request,
            execution,
            packets: Vec::new(),
            talents: Vec::new(),
            locations: Vec::new(),
            replay_actions: Vec::new(),
            occurrences: Vec::new(),
            diagnostic: None,
            failure_code: DiagnosticCode::Evaluation,
        };
        trial.advance(0)?;
        let mut verified = resume.is_none();
        let mut checkpoint = None;
        if trial.execution.success && trial.execution.execution_completed {
            trial.execute(&mut |trial| {
                if let Some(saved) = resume.filter(|_| !verified) {
                    if trial.occurrences.len() < saved.completed_inputs {
                        return Ok(false);
                    }
                    if trial.boundary_digest()? != saved.prefix_sha256 {
                        return Err("Source checkpoint prefix mismatch.".into());
                    }
                    verified = true;
                }
                if !cancel(trial.boundary())? {
                    return Ok(false);
                }
                checkpoint = Some(SourceCheckpoint::capture(trial, seed)?);
                Ok(true)
            })?;
        }
        if !verified {
            return Err("Source checkpoint boundary was not reached.".into());
        }
        if checkpoint.is_some() {
            trial.execution.record_pending_events();
            trial.execution.halt_diagnostic =
                Some("SOURCE_CANCELLED: retained completed input boundary.".into());
        } else {
            trial.retain_source_halt();
        }
        if trial.execution.success && trial.execution.execution_completed {
            trial.execution.process_events(
                &trial.request,
                &self.limits,
                &trial.packets,
                &[],
                &[],
            )?;
        }
        if checkpoint.is_none() {
            trial.ensure_failure_diagnostic();
        }
        if trial.execution.limit_reached.is_some() {
            trial.retain_source_halt();
        }
        Ok((trial, lifecycle, checkpoint))
    }
}

struct Trial<'a> {
    program: &'a PreparedSourceScenario,
    request: GameScenarioRequest,
    execution: PreparedExecution,
    packets: Vec<Option<crate::simulation::metadata::ActionPacketMetadata>>,
    // Preserve the actual input before native talent projection for exact replay.
    replay_actions: Vec<GameActionCommand>,
    talents: Vec<serde_json::Value>,
    locations: Vec<SourceLocation>,
    occurrences: Vec<SourceOccurrence>,
    diagnostic: Option<Diagnostic>,
    failure_code: DiagnosticCode,
}

impl Trial<'_> {
    fn execute(
        &mut self,
        boundary: &mut impl FnMut(&Self) -> Result<bool, String>,
    ) -> Result<(), String> {
        let mut rotation = Rotation::new(
            &self.program.config.rotation,
            &self.program.config.constants,
            self.program.source_limits.max_steps,
            self.program.source_limits.max_actions,
        );
        loop {
            if self.victory() || self.time_completed() {
                self.request.source_exhausted = true;
                break;
            }
            if boundary(self)? {
                break;
            }
            let host = observation::Observation {
                state: &self.execution.state,
                request: &self.request,
                actors: &self.program.config.actors,
                options: &self.program.config.options,
            };
            let occurrence = match rotation.next(&host) {
                Ok(Some(occurrence)) => occurrence,
                Ok(None) => {
                    self.request.source_exhausted = true;
                    break;
                }
                Err(error) => {
                    self.fail(error.into());
                    break;
                }
            };
            let record = SourceOccurrence {
                index: occurrence.index,
                loc: occurrence.loc.clone(),
                kind: match occurrence.input {
                    Input::TargetGauge { .. } => SourceOccurrenceKind::TargetGauge,
                    Input::Action { .. } => SourceOccurrenceKind::Action,
                    Input::Wait(_) => SourceOccurrenceKind::Wait,
                    Input::WaitUntil(_) => SourceOccurrenceKind::WaitUntil,
                    Input::WaitForEnergy(_) => SourceOccurrenceKind::WaitForEnergy,
                },
                started_us: self.execution.state.current_time_us,
                completed_us: None,
                action_window: None,
                command_indices: Vec::new(),
            };
            self.occurrences.push(record);
            let first_command = self.request.actions.len();
            self.failure_code = DiagnosticCode::Evaluation;
            if let Err(message) = self.input(&occurrence) {
                self.fail(Diagnostic {
                    code: self.failure_code.clone(),
                    message,
                    loc: occurrence.loc.clone(),
                });
            }
            let record = self.occurrences.last_mut().expect("current occurrence");
            record.command_indices = (first_command..self.request.actions.len()).collect();
            if !self.execution.success || !self.execution.execution_completed {
                self.ensure_failure_diagnostic();
                break;
            }
            record.completed_us = Some(self.execution.state.current_time_us);
        }
        Ok(())
    }

    fn fail(&mut self, diagnostic: Diagnostic) {
        self.execution.success = false;
        self.execution.execution_completed = false;
        self.execution.halt_diagnostic = Some(diagnostic.message.clone());
        self.diagnostic = Some(diagnostic);
    }

    fn input(&mut self, occurrence: &Occurrence<'_>) -> Result<(), String> {
        match &occurrence.input {
            Input::TargetGauge { target, gauge } => {
                self.target_gauge(target, gauge.as_ref(), occurrence.loc)
            }
            Input::Action {
                actor,
                action,
                params,
            } => self.action(actor, action, params, occurrence.loc),
            Input::Wait(value) | Input::WaitUntil(value) => {
                let time = value.number()?.to_microseconds()?;
                let now = self.execution.state.current_time_us;
                let deadline = if matches!(occurrence.input, Input::WaitUntil(_)) {
                    time.max(now)
                } else {
                    now.checked_add(time).ok_or("Source timestamp overflow.")?
                };
                self.append_wait(deadline - now, occurrence.loc)?;
                let command_index = self.request.actions.len() - 1;
                let context = self.begin_time_context(
                    if matches!(occurrence.input, Input::WaitUntil(_)) {
                        crate::simulation::GameTimeContextKind::WaitUntil { command_index }
                    } else {
                        crate::simulation::GameTimeContextKind::Wait { command_index }
                    },
                    occurrence.loc,
                );
                let result = self.advance(deadline);
                self.end_time_context(context);
                result
            }
            Input::WaitForEnergy(value) => {
                let required = value.number()?.to_safe_integer()?;
                let context = self.begin_time_context(
                    crate::simulation::GameTimeContextKind::Energy { required },
                    occurrence.loc,
                );
                let result = self.energy_wait(value);
                self.end_time_context(context);
                result
            }
        }
    }

    fn append_wait(&mut self, duration_us: u64, loc: &SourceLocation) -> Result<(), String> {
        self.command_budget()?;
        let command = GameActionCommand::Wait {
            timestamp_us: self.execution.state.current_time_us,
            duration_us,
        };
        if self.execution.state.record_detail {
            self.replay_actions.push(command.clone());
            self.locations.push(loc.clone());
        }
        self.request.actions.push(command);
        self.packets.push(None);
        Ok(())
    }

    fn command_budget(&mut self) -> Result<(), String> {
        if self.request.actions.len() >= self.program.source_limits.max_actions {
            self.failure_code = DiagnosticCode::ActionLimit;
            Err(format!(
                "Script action budget exhausted: limit {} native commands (including implicit switches and waits).",
                self.program.source_limits.max_actions
            ))
        } else {
            Ok(())
        }
    }

    fn advance(&mut self, deadline: u64) -> Result<(), String> {
        self.execution.advance_to(
            &self.request,
            &self.program.limits,
            &self.packets,
            deadline.min(self.request.absolute_limit_us),
        )?;
        if deadline > self.execution.state.current_time_us
            && self.execution.success
            && !self.victory()
            && !self.time_completed()
        {
            self.failure_code = DiagnosticCode::Horizon;
            self.execution.execution_completed = false;
            let absolute_limit = self.execution.state.current_time_us
                >= self.request.absolute_limit_us
                && self.request.stop == crate::simulation::GameStopPolicy::Time
                && self.execution.state.mode_clock.relative_us()
                    < self.request.relative_duration_us;
            if absolute_limit {
                self.execution.limit_reached =
                    Some(crate::simulation::GameExecutionLimit::AbsoluteTime {
                        limit_us: self.request.absolute_limit_us,
                    });
            }
            self.execution.halt_diagnostic = Some(if absolute_limit {
                format!(
                    "ABSOLUTE_TIME_LIMIT: absolute ceiling {} microseconds reached at {} relative microseconds of {} requested.",
                    self.request.absolute_limit_us,
                    self.execution.state.mode_clock.relative_us(),
                    self.request.relative_duration_us
                )
            } else if self.request.stop == crate::simulation::GameStopPolicy::Victory {
                super::stopping::victory_time_limit(self.request.absolute_limit_us)
            } else {
                "SOURCE_HORIZON_PENDING: authored input exceeds the horizon".into()
            });
            self.record_pending_continuation();
        }
        Ok(())
    }

    fn victory(&self) -> bool {
        self.execution.all_targets_defeated()
    }

    fn time_completed(&self) -> bool {
        self.request.stop == crate::simulation::GameStopPolicy::Time
            && self.execution.state.mode_clock.relative_us() >= self.request.relative_duration_us
    }

    fn energy_wait(&mut self, value: &Value) -> Result<(), String> {
        let threshold = value.number()?.to_safe_integer()?;
        let actor = self.execution.state.active_actor_index;
        if threshold > self.request.actors[actor].energy_capacity {
            return Err("Energy threshold exceeds the active actor capacity.".into());
        }
        while self.execution.state.actors[actor].energy.current() < threshold {
            let next = self.execution.queue.peek().map(|event| event.time_us);
            let deadline = next
                .unwrap_or(self.request.absolute_limit_us)
                .min(self.execution.stop_deadline(&self.request)?);
            self.advance(deadline)?;
            if !self.execution.success || self.victory() {
                break;
            }
            if self.execution.state.actors[actor].energy.current() >= threshold {
                break;
            }
            if deadline == self.execution.stop_deadline(&self.request)? {
                self.failure_code = DiagnosticCode::EnergyGate;
                self.execution.execution_completed = false;
                self.execution.halt_diagnostic = Some(format!(
                    "LIVE_ENERGY_WAIT_PENDING: actor {actor} did not reach energy {threshold} before the horizon"
                ));
                self.record_pending_continuation();
                break;
            }
        }
        Ok(())
    }

    fn record_pending_continuation(&mut self) {
        self.execution.pending_events = self.execution.queue.len() + 1;
    }
}
