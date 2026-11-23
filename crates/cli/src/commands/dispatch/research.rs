use crate::commands::definition::{CliCommand, completion, pilot_completion, tabular_completion};
use crate::reports::*;
use serde_json::Value;
use std::{fs, path::Path};

pub(crate) fn dispatch_research(cmd: &CliCommand<'_>) -> Result<Option<(Value, bool)>, String> {
    match cmd {
        CliCommand::PublishedVariants { input } => {
            let csv_content = crate::report_context::variants(input.map(Path::new))?;
            Ok(Some((published_variants_report(&csv_content)?, true)))
        }
        CliCommand::ProfileAudit { input } => {
            let bytes = crate::report_context::variants(input.map(Path::new))?.into_bytes();
            Ok(Some((published_profile_audit_report(&bytes)?, true)))
        }
        CliCommand::RecommendedGear => {
            let csv = ntesim_data::research_inputs::RECOMMENDED_GEAR_CSV.as_bytes();
            Ok(Some((published_gear_report(csv)?, true)))
        }
        CliCommand::BuiltVariants { csv_path, input } => {
            let bytes = fs::read(input).map_err(|e| e.to_string())?;
            let report = published_built_variants_report(csv_path.map(Path::new), &bytes)?;
            Ok(Some((report, true)))
        }
        CliCommand::SearchBuilds { input } => {
            let csv_content = crate::report_context::variants(input.map(Path::new))?;
            let report = published_search_report(&csv_content)?;
            let complete = report["execution_status"] == "SUCCESS";
            Ok(Some((report, complete)))
        }
        CliCommand::Compositions { variants_csv } => Ok(Some((
            composition_enumeration_report(variants_csv.map(Path::new))?,
            true,
        ))),
        CliCommand::AcceptanceSpec { out_dir } => Ok(Some((
            published_acceptance_report(out_dir.map(Path::new))?,
            true,
        ))),
        CliCommand::ValidateFinalist {
            variants_csv,
            input,
        } => completion(
            published_finalist_report(
                &fs::read(input).map_err(|e| e.to_string())?,
                variants_csv.map(Path::new),
            )?,
            &["/all_invariants_passed", "/all_energy_conserved"],
        )
        .map(Some),
        CliCommand::VerifyTabular { fixture_path } => {
            tabular_completion(verify_tabular_report(fixture_path.map(Path::new))?).map(Some)
        }
        CliCommand::Pilot { input, checkpoint } => pilot_completion(pilot_report(
            input.map(Path::new),
            checkpoint.map(Path::new),
        )?)
        .map(Some),
        CliCommand::AcceptanceRun {
            input,
            checkpoint,
            variants_csv,
            conditions_json,
        } => completion(
            acceptance_run_report(
                input.map(Path::new),
                checkpoint.map(Path::new),
                variants_csv.map(Path::new),
                conditions_json.map(Path::new),
            )?,
            &["/checkpoint/is_complete"],
        )
        .map(Some),
        _ => Ok(None),
    }
}
