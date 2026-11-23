//! Read a complete configuration without lowering or running its rotation.
//! Presentation clients receive canonical IDs and the engine's exact static stats.

use std::collections::BTreeMap;

use ntesim_core::formula::Ratio;
use serde::{Deserialize, Serialize};

use super::{
    SourceLocation,
    configuration::{CompiledConfiguration, compile_configuration_with_event_limit},
    parse_nte,
    validation::Diagnostic,
};
use crate::{execution::ExecutionLimits, simulation::GameActorStaticStats};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InspectionRequest {
    pub source: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct InspectionResponse {
    pub party: Option<PartyInspection>,
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PartyInspection {
    pub schema_version: u32,
    pub content_id: String,
    pub active_actor_index: usize,
    pub start_fons: Option<u64>,
    pub actors: Vec<ActorInspection>,
    pub model_assumption_ids: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ActorInspection {
    pub build_decomposition: crate::build::BuildDecomposition,
    pub slot: usize,
    pub character_id: String,
    pub loc: SourceLocation,
    pub level: u32,
    pub max_level: Option<u32>,
    pub awakening: ntesim_data::AwakeningSelection,
    pub talent_levels: [u32; 3],
    pub purchased_qte_level: u32,
    pub weapon: Option<WeaponInspection>,
    pub set: Option<SetInspection>,
    /// Includes authored additions and unconditional equipped effects once.
    /// Conditional/live buffs are not a pre-combat snapshot.
    pub static_stats: GameActorStaticStats,
    pub charge_efficiency: Ratio,
    pub initial_hp: u64,
    pub initial_energy: u64,
    pub energy_capacity: u64,
    pub initial_cycle: u64,
    pub initial_resources: crate::simulation::GameInitialResources,
    pub additive_stats: BTreeMap<String, Ratio>,
    pub unmodeled_attribute_ids: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WeaponInspection {
    pub id: String,
    /// Omission uses the shared projector's level-80 defaults.
    pub level: u32,
    pub max_level: u32,
    pub refinement: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SetInspection {
    pub id: String,
    pub count: u32,
}

impl PartyInspection {
    pub(crate) fn from_configuration(config: &CompiledConfiguration) -> Self {
        Self {
            schema_version: 1,
            content_id: "ntesim-current".into(),
            active_actor_index: config.active_actor_index,
            start_fons: config.options.start_fons,
            model_assumption_ids: config.model_assumptions.keys().cloned().collect(),
            actors: config
                .actors
                .iter()
                .map(|compiled| {
                    let actor = &compiled.actor;
                    ActorInspection {
                        build_decomposition: compiled.projection.decomposition.clone(),
                        slot: actor.slot,
                        character_id: actor.character_id.clone(),
                        loc: compiled.loc.clone(),
                        level: actor.character_level,
                        max_level: actor.character_max_level,
                        awakening: actor.awakening.clone(),
                        // Configuration compilation always supplies all three purchases.
                        talent_levels: actor.talent_levels.expect("compiled configuration talents"),
                        purchased_qte_level: actor
                            .purchased_qte_level
                            .expect("compiled configuration QTE"),
                        weapon: (!actor.weapon_id.is_empty()).then(|| WeaponInspection {
                            id: actor.weapon_id.clone(),
                            level: compiled.weapon_level.unwrap_or(80),
                            max_level: compiled.weapon_max_level.unwrap_or(80),
                            refinement: actor.weapon_tier,
                        }),
                        set: actor.set_id.as_ref().map(|id| SetInspection {
                            id: id.clone(),
                            count: compiled.set_count,
                        }),
                        static_stats: actor.static_stats.clone(),
                        charge_efficiency: compiled.projection.charge_efficiency,
                        initial_hp: actor.initial_hp,
                        initial_energy: actor.initial_energy,
                        energy_capacity: actor.energy_capacity,
                        initial_cycle: actor.initial_cycle,
                        initial_resources: actor.initial_resources.clone(),
                        additive_stats: compiled.contributions.clone(),
                        unmodeled_attribute_ids: compiled
                            .projection
                            .unmodeled_attributes
                            .keys()
                            .cloned()
                            .collect(),
                    }
                })
                .collect(),
        }
    }
}

pub(crate) fn prepare(
    source: &str,
) -> Result<(super::Script, CompiledConfiguration), Vec<Diagnostic>> {
    prepare_with_event_limit(source, ExecutionLimits::default().max_events)
}

pub(crate) fn prepare_with_event_limit(
    source: &str,
    max_events: u64,
) -> Result<(super::Script, CompiledConfiguration), Vec<Diagnostic>> {
    let script = {
        #[cfg(feature = "performance-diagnostics")]
        let _span = crate::performance::Span::new("source.parse");
        parse_nte(source).map_err(|error| vec![error.into()])?
    };
    let config = {
        #[cfg(feature = "performance-diagnostics")]
        let _span = crate::performance::Span::new("source.configuration");
        compile_configuration_with_event_limit(
            &script,
            ntesim_data::stat_projection::frozen_tables(),
            max_events,
        )?
    };
    Ok((script, config))
}

pub fn inspect_response(request: &InspectionRequest) -> InspectionResponse {
    match prepare(&request.source) {
        Ok((_, config)) => InspectionResponse {
            party: Some(PartyInspection::from_configuration(&config)),
            diagnostics: Vec::new(),
        },
        Err(diagnostics) => InspectionResponse {
            party: None,
            diagnostics,
        },
    }
}
