use crate::runtime_api::NormalActionOverride;
use crate::simulation::{
    GameActorConfig, GameScenarioRequest, NativeScalingStat,
    metadata::{ActionCategory, ActionPacketMetadata, MetadataProvenance, PrimaryActionSlot},
};

pub(super) struct SelectedNormal {
    pub(super) metadata: ActionPacketMetadata,
    pub(super) rate: crate::formula::Ratio,
    pub(super) scaling: NativeScalingStat,
    pub(super) timing: Option<(u64, u64)>,
}

pub(super) fn select(
    request: &GameScenarioRequest,
    command_index: usize,
    config: &GameActorConfig,
    replacement: Option<NormalActionOverride>,
    ability: &str,
    coordinate: usize,
    prepared: &ActionPacketMetadata,
) -> Result<Option<SelectedNormal>, String> {
    if prepared.action.primary_slot != Some(PrimaryActionSlot::Normal)
        || request.plunge_inputs.contains_key(&command_index)
    {
        return Ok(None);
    }
    let Some(NormalActionOverride {
        packet,
        role,
        model,
        timing,
    }) = replacement
    else {
        return Ok(None);
    };
    let mut accepted = request.model_assumptions.clone();
    crate::runtime_api::record_hypothesis_id(request, &mut accepted, model)?;
    let (rate, scaling) =
        super::super::inputs::coefficient(request, config, ability, packet, coordinate)?;
    let mut metadata = crate::simulation::metadata::resolve_requested_action_packet(
        &request.content_id,
        ntesim_data::skill_levels::frozen::tables(),
        &config.character_id,
        ability,
        packet,
        None,
        &accepted,
    )?;
    if let Some(role) = role {
        metadata.action.input_role = Some(role);
        metadata.action.primary_slot = None;
        metadata.action.categories = std::collections::BTreeSet::from([ActionCategory::Other]);
        metadata.action.gameplay_tags.clear();
        metadata.action.input_role_provenance = Some(MetadataProvenance::AcceptedModelHypothesis {
            id: model.as_str().into(),
            description: model.description().into(),
        });
    }
    Ok(Some(SelectedNormal {
        metadata,
        rate,
        scaling,
        timing,
    }))
}
