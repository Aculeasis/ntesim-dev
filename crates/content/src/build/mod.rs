use crate::{
    damage_values, equipment,
    execution::ExecutionLimits,
    formula::Ratio,
    simulation::{
        ExactFraction, GameActionCommand, GameActorStaticStats, GameScenarioRequest,
        GearProcConfig, run_game_scenario,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

mod decomposition;
mod model;
pub use decomposition::{BuildDecomposition, StatContribution, StatSource};
pub use model::{
    ActionProjection, ActionProjectionPolicy, ActorBuild, Compiled, GearProjection,
    GearProjectionPolicy, ProjectionPolicy, Request,
};
mod receipts;
pub(crate) use receipts::{integer, nearest_integer, ratio, receipt};
mod stats;
pub use stats::{element_attribute, project};
mod actions;
#[cfg(test)]
use actions::project_actions;
pub(crate) use actions::project_actions_with_levels;
pub(crate) use actions::supports_hp_packet;
pub(crate) use actions::{PacketCoefficientRequest, project_native_packet_coefficient};
mod gear;
mod weapon_projection;
pub(crate) use gear::project_gear;
pub(crate) use weapon_projection::{
    WeaponParameters, compile_weapon_values, finish_weapon_projection, weapon_seconds_to_us,
    weapon_wearer_item,
};
mod assumptions;
mod compiler;
pub mod preview_projection;
mod public_actions;
pub use public_actions::PublicAction;
pub mod studio;
mod studio_equipment;
pub use studio_equipment::EquipmentProjection;
pub mod studio_projection;
pub use compiler::{compile, run};

#[cfg(test)]
mod tests;
