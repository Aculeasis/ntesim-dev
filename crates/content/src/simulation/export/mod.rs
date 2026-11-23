//! Portable native source reports with exact views and reconciled replay.
use super::{SourceScenarioReport, analysis::DamageAnalysis};
use crate::dsl::{
    compiler::CompileLimits,
    inspection::{InspectionRequest, PartyInspection},
};
use crate::execution::ExecutionLimits;
use ntesim_contracts::{ENGINE_SOURCE_SHA256, ENGINE_VERSION, sha256};
use serde::{Deserialize, Serialize};

mod csv;

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceReportExport {
    pub schema_version: u32,
    pub engine_version: String,
    pub engine_source_sha256: String,
    pub source_sha256: String,
    pub request_sha256: String,
    pub input_sha256: String,
    pub source_limits: CompileLimits,
    pub execution_limits: ExecutionLimits,
    pub party: PartyInspection,
    pub damage: DamageAnalysis,
    pub execution: SourceScenarioReport,
    pub uncertainty_policy: String,
}

#[derive(Debug, Serialize)]
pub struct SourceReplayVerification<'a> {
    pub reconciled: bool,
    pub source_sha256: &'a str,
    pub input_sha256: &'a str,
    pub request_sha256: &'a str,
    pub success: bool,
    pub execution_completed: bool,
    pub quality: &'a ntesim_contracts::Quality,
    pub damage: &'a DamageAnalysis,
}

fn bytes(value: &impl Serialize) -> Result<Vec<u8>, String> {
    serde_json::to_vec(value).map_err(|e| e.to_string())
}

fn input_identity(
    source: &str,
    request: &str,
    source_limits: CompileLimits,
    limits: ExecutionLimits,
) -> Result<String, String> {
    Ok(sha256(&bytes(&(
        "native-source-report-v1",
        ENGINE_VERSION,
        ENGINE_SOURCE_SHA256,
        source,
        request,
        source_limits,
        limits,
    ))?))
}

fn validate_limits(source: CompileLimits, limits: ExecutionLimits) -> Result<(), String> {
    let defaults = ExecutionLimits::default();
    if source.max_steps == 0
        || source.max_steps > 1_000_000
        || source.max_actions == 0
        || source.max_actions > 100_000
        || limits.max_events == 0
        || limits.max_events > defaults.max_events
        || limits.max_horizon_us > defaults.max_horizon_us
        || limits.max_pool_capacity > defaults.max_pool_capacity
        || limits.max_proc_depth > defaults.max_proc_depth
        || limits.max_active_shields > defaults.max_active_shields
    {
        return Err("source report export limits exceed supported bounds".into());
    }
    Ok(())
}

impl SourceReportExport {
    pub fn replay_verification(&self) -> Result<SourceReplayVerification<'_>, String> {
        self.replay()?;
        Ok(SourceReplayVerification {
            reconciled: true,
            source_sha256: &self.source_sha256,
            input_sha256: &self.input_sha256,
            request_sha256: &self.request_sha256,
            success: self.execution.report.success,
            execution_completed: self.execution.report.execution_completed,
            quality: &self.execution.report.quality,
            damage: &self.damage,
        })
    }

    pub fn new(
        execution: SourceScenarioReport,
        source_limits: CompileLimits,
        execution_limits: ExecutionLimits,
    ) -> Result<Self, String> {
        validate_limits(source_limits, execution_limits)?;
        if execution.report.engine_version != ENGINE_VERSION
            || execution.report.engine_source_sha256 != ENGINE_SOURCE_SHA256
        {
            return Err("source report export engine identity mismatch".into());
        }
        execution.report.validate_state_timeline()?;
        let damage = execution.report.damage_analysis()?;
        let inspection = crate::dsl::inspection::inspect_response(&InspectionRequest {
            source: execution.source.clone(),
        });
        let party = inspection
            .party
            .ok_or("export source configuration is invalid")?;
        for actor in &party.actors {
            actor
                .build_decomposition
                .validate_static_stats(&actor.static_stats)?;
        }
        let source_sha256 = sha256(execution.source.as_bytes());
        let request_sha256 = sha256(&bytes(&execution.request)?);
        let input_sha256 = input_identity(
            &source_sha256,
            &request_sha256,
            source_limits,
            execution_limits,
        )?;
        Ok(Self {
            schema_version: 1, engine_version: ENGINE_VERSION.into(),
            engine_source_sha256: ENGINE_SOURCE_SHA256.into(),
            source_sha256, request_sha256, input_sha256,
            source_limits, execution_limits, party, damage, execution,
            uncertainty_policy: "Seed sampling describes variation within the declared model; it does not measure model uncertainty or establish game evidence.".into(),
        })
    }

    /// Equality checks preserve expected failed/incomplete receipts. Successful
    /// verification does not promote their execution or comparison eligibility.
    pub fn replay(&self) -> Result<(), String> {
        if self.schema_version != 1
            || self.engine_version != ENGINE_VERSION
            || self.engine_source_sha256 != ENGINE_SOURCE_SHA256
            || self.execution.report.engine_version != ENGINE_VERSION
            || self.execution.report.engine_source_sha256 != ENGINE_SOURCE_SHA256
            || self.source_sha256 != sha256(self.execution.source.as_bytes())
            || self.request_sha256 != sha256(&bytes(&self.execution.request)?)
            || self.input_sha256
                != input_identity(
                    &self.source_sha256,
                    &self.request_sha256,
                    self.source_limits,
                    self.execution_limits,
                )?
        {
            return Err("source report export identity mismatch".into());
        }
        let limits = self.execution_limits;
        validate_limits(self.source_limits, limits)?;
        let program =
            super::PreparedSourceScenario::new(&self.execution.source, self.source_limits, &limits)
                .map_err(|d| format!("export source no longer compiles: {d:?}"))?;
        let rerun = program.run(self.execution.report.seed)?;
        let expected = Self::new(rerun, self.source_limits, limits)?;
        if bytes(self)? != bytes(&expected)? {
            return Err(
                "source report export differs from executed source, builds or exact report views"
                    .into(),
            );
        }
        let native = super::run_game_scenario_report(&self.execution.request, &limits)?;
        if bytes(&native)? != bytes(&self.execution.report)? {
            return Err("source report export differs from native replay".into());
        }
        Ok(())
    }

    pub fn csv_tables(&self) -> Result<std::collections::BTreeMap<&'static str, String>, String> {
        csv::tables(self)
    }
}
