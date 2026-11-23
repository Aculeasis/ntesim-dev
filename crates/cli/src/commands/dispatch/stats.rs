use crate::commands::definition::CliCommand;
use crate::reports::*;
use serde_json::Value;
use std::fs;

pub(crate) fn dispatch_package_and_stats(
    cmd: &CliCommand<'_>,
) -> Result<Option<(Value, bool)>, String> {
    match cmd {
        CliCommand::StudioBuildCatalog => Ok(Some((studio_build_catalog()?, true))),
        CliCommand::ProjectStudioActors { input } => Ok(Some((
            actor_suite_report(&fs::read(input).map_err(|e| e.to_string())?)?,
            true,
        ))),
        CliCommand::ProjectStudioActor { input } => {
            let bytes = fs::read(input).map_err(|e| e.to_string())?;
            let request: ntesim_content::build::studio_projection::Request =
                serde_json::from_slice(&bytes)
                    .map_err(|e| format!("invalid actor projection request: {e}"))?;
            let tables = ntesim_data::stat_projection::frozen_tables();
            let result = ntesim_content::build::studio_projection::project(tables, &request)?;
            Ok(Some((
                serde_json::to_value(result).map_err(|e| e.to_string())?,
                true,
            )))
        }
        CliCommand::CharacterStats { input } => Ok(Some((
            character_report(&fs::read(input).map_err(|e| e.to_string())?)?,
            true,
        ))),
        CliCommand::WeaponStats { input } => Ok(Some((
            weapon_report(&fs::read(input).map_err(|e| e.to_string())?)?,
            true,
        ))),
        CliCommand::WeaponPassiveValues { input } => Ok(Some((
            weapon_passive_values_report(&fs::read(input).map_err(|e| e.to_string())?)?,
            true,
        ))),
        CliCommand::DamageValues { input } => Ok(Some((
            damage_values_report(&fs::read(input).map_err(|e| e.to_string())?)?,
            true,
        ))),
        CliCommand::BuildStats { input } => Ok(Some((
            build_report(&fs::read(input).map_err(|e| e.to_string())?)?,
            true,
        ))),
        CliCommand::TeamStats { input } => Ok(Some((
            team_report(&fs::read(input).map_err(|e| e.to_string())?)?,
            true,
        ))),
        CliCommand::CombatStats { input } => Ok(Some((
            combat_report(&fs::read(input).map_err(|e| e.to_string())?)?,
            true,
        ))),
        CliCommand::LifecycleStats { input } => Ok(Some((
            lifecycle_report(&fs::read(input).map_err(|e| e.to_string())?)?,
            true,
        ))),
        CliCommand::DispatchStats { input } => Ok(Some((
            dispatch_report(&fs::read(input).map_err(|e| e.to_string())?)?,
            true,
        ))),
        CliCommand::TimingStats { input } => Ok(Some((
            timing_report(&fs::read(input).map_err(|e| e.to_string())?)?,
            true,
        ))),
        CliCommand::ReactionStats { input } => Ok(Some((
            reaction_report(&fs::read(input).map_err(|e| e.to_string())?)?,
            true,
        ))),
        CliCommand::ExecutionStats { input } => Ok(Some((
            execution_report(&fs::read(input).map_err(|e| e.to_string())?)?,
            true,
        ))),
        _ => Ok(None),
    }
}
