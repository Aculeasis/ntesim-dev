use super::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AcceptanceDeliverables {
    pub schema_version: u32,
    pub content_id: String,
    pub status: String,
    pub quality: Quality,
    pub game_dps_computed: bool,
    pub acceptance_ready: bool,
    pub total_compositions: usize,
    pub total_variants: usize,
    pub total_archetypes: usize,
    pub conditions_summary: AcceptanceConditionsSummary,
    pub scenarios: Vec<ScenarioInfo>,
    pub all_compositions_csv_sha256: String,
    pub published_variants_csv_sha256: String,
    pub archetypes_csv_sha256: String,
    pub run_manifest_sha256: String,
    pub all_compositions_csv_lines: usize,
    pub published_variants_csv_lines: usize,
    pub archetypes_csv_lines: usize,
    pub blockers: Vec<String>,
    pub engine_version: String,
    pub engine_source_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ScenarioInfo {
    pub code: String,
    pub description: String,
    pub target_count: usize,
    pub horizon_seconds: u64,
    pub is_mode_clock: bool,
    pub is_break_enabled: bool,
}

pub fn compile_acceptance_deliverables(
    coverage: &Value,
    conditions: &Value,
    variants_csv: &str,
) -> Result<(AcceptanceDeliverables, String, String, String, String), String> {
    let comp_report: CompositionReport =
        composition::enumerate_compositions(coverage, variants_csv)?;
    let all_comp_csv = composition::generate_csv(&comp_report);
    let all_comp_sha256 = sha256(all_comp_csv.as_bytes());

    let variants = parse_published_variants_csv(variants_csv)?;
    if variants.is_empty() {
        return Err("published variants missing".into());
    }

    let variants_csv_text = generate_published_variants_csv(&variants);
    let variants_sha256 = sha256(variants_csv_text.as_bytes());

    let archetypes_csv_text = generate_archetypes_csv(&variants);
    let archetypes_sha256 = sha256(archetypes_csv_text.as_bytes());

    let run_state = AcceptanceRunState {
        composition_enumeration_complete: true,
        compositions: comp_report.total_compositions,
        published_archetypes: comp_report.distinct_archetypes_count,
        published_variants: variants.len(),
        ..AcceptanceRunState::default()
    };
    let manifest_value = generate_run_manifest(conditions, &run_state);
    let manifest_text = serde_json::to_string_pretty(&manifest_value).map_err(|e| e.to_string())?;
    let manifest_sha256 = sha256(manifest_text.as_bytes());

    let scenarios = AcceptanceScenarioKind::ALL
        .iter()
        .map(|k| ScenarioInfo {
            code: k.code().to_string(),
            description: k.description().to_string(),
            target_count: k.target_count(),
            horizon_seconds: k.horizon_us() / 1_000_000,
            is_mode_clock: k.is_mode_clock(),
            is_break_enabled: k.is_break_enabled(),
        })
        .collect();

    let deliverables = AcceptanceDeliverables {
        schema_version: 1,
        content_id: "ntesim-current".into(),
        status: "INCOMPLETE".into(),
        quality: Quality::Incomplete,
        game_dps_computed: false,
        acceptance_ready: false,
        total_compositions: comp_report.total_compositions,
        total_variants: variants.len(),
        total_archetypes: comp_report.distinct_archetypes_count,
        conditions_summary: serde_json::from_value(conditions.clone()).map_err(|e| format!("invalid acceptance conditions: {e}"))?,
        scenarios,
        all_compositions_csv_sha256: all_comp_sha256,
        published_variants_csv_sha256: variants_sha256,
        archetypes_csv_sha256: archetypes_sha256,
        run_manifest_sha256: manifest_sha256,
        all_compositions_csv_lines: all_comp_csv.lines().count(),
        published_variants_csv_lines: variants_csv_text.lines().count(),
        archetypes_csv_lines: archetypes_csv_text.lines().count(),
        blockers: vec![
            "Full evidenced character kit adapters and native event execution remain open".into(),
            "Actual enemy break profiles and break multiplier timelines are unresolved".into(),
            "Game-mode timer dilation mechanics (Hotori) lack verified native timestamps".into(),
            "Top-50 per-scenario rotation optimization and 1,000-seed finalist Monte Carlo trials remain pending complete combat models".into(),
        ],
        engine_version: ENGINE_VERSION.to_string(),
        engine_source_sha256: ENGINE_SOURCE_SHA256.to_string(),
    };

    Ok((
        deliverables,
        all_comp_csv,
        variants_csv_text,
        archetypes_csv_text,
        manifest_text,
    ))
}
