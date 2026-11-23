//! Retain the authored input and nominal packet template, marking its selected
//! non-damaging replacement before ordinary cast observers can run.
use super::*;

pub(super) fn select(
    request: &super::super::GameScenarioRequest,
    cmd: &mut AdmittedInvocation,
    prepared: &super::super::metadata::ActionPacketMetadata,
    utility: crate::runtime_api::UtilityInvocation,
) -> Result<super::super::metadata::ActionPacketMetadata, String> {
    use super::super::metadata::{ActionCategory, InputActionRole, MetadataProvenance};
    utility.validate(request)?;
    cmd.energy_cost = 0;
    cmd.cooldown_us = 0;
    cmd.recovery_us = utility.recovery_us;
    cmd.utility = Some(utility);
    let mut metadata = prepared.clone();
    let provenance = MetadataProvenance::AcceptedModelHypothesis {
        id: utility.model.as_str().into(),
        description: utility.model.description().into(),
    };
    metadata.action.categories = std::collections::BTreeSet::from([ActionCategory::Other]);
    metadata.action.gameplay_tags.clear();
    metadata.action.primary_slot = None;
    metadata.action.input_role = Some(InputActionRole::Other);
    metadata.action.input_role_provenance = Some(provenance.clone());
    metadata.action.provenance = provenance;
    Ok(metadata)
}
