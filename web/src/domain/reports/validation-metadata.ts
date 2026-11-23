import type { ScenarioRequest } from '../contracts/scenario';
import { INPUT_ACTION_ROLES } from '../contracts/action-role';
import type {
  ActionMetadataReceipt,
  ActionPacketMetadataReceipt,
  MetadataProvenance,
  NativeEvidence,
  PacketMetadataReceipt,
} from '../contracts/report';
import {
  exactFraction,
  isActionConfigTable,
  primarySlotForInputRole,
  resolveEvidence,
  validateEvidence,
  validateProvenance,
} from './validation-support';

export function createMetadataValidator(
  request: ScenarioRequest,
  evidenceCatalog: NativeEvidence[],
  metadataCatalog: ActionPacketMetadataReceipt[],
  resolvedAssumptions: Record<string, string>,
  recordResolvedProvenance: (provenance: MetadataProvenance) => void,
) {
  const validateActionMetadata = (
    actionMetadata: ActionMetadataReceipt,
    requireCategories: boolean,
  ) => {
    validateProvenance(
      actionMetadata.provenance,
      actionMetadata.source_evidence_id ?? null,
      evidenceCatalog,
      request.content_id,
      resolvedAssumptions,
    );
    recordResolvedProvenance(actionMetadata.provenance);
    if (
      actionMetadata.primary_slot !== undefined &&
      actionMetadata.primary_slot !== null &&
      !['normal', 'skill', 'ultimate'].includes(actionMetadata.primary_slot)
    )
      throw new Error('Invalid owner primary input role receipt.');
    const gameplayTags = actionMetadata.gameplay_tags ?? [];
    if (
      !Array.isArray(actionMetadata.categories) ||
      (requireCategories && !actionMetadata.categories.length) ||
      !actionMetadata.categories.every((category) =>
        ['melee', 'skill', 'ultra_skill', 'qte', 'other'].includes(category),
      ) ||
      !Array.isArray(gameplayTags) ||
      !gameplayTags.every((tag) => typeof tag === 'string')
    )
      throw new Error('Invalid action category or native gameplay tag receipt.');
    if (
      actionMetadata.native_ability_index !== undefined &&
      actionMetadata.native_ability_index !== null &&
      (!Number.isSafeInteger(actionMetadata.native_ability_index) ||
        actionMetadata.native_ability_index < 0)
    )
      throw new Error('Invalid native AbilityIndex receipt.');
    if (
      actionMetadata.input_role !== undefined &&
      actionMetadata.input_role !== null &&
      !INPUT_ACTION_ROLES.includes(actionMetadata.input_role)
    )
      throw new Error('Invalid owner input role receipt.');
    if (
      actionMetadata.input_role &&
      (actionMetadata.primary_slot ?? null) !== primarySlotForInputRole(actionMetadata.input_role)
    )
      throw new Error('Owner primary slot differs from its input-role declaration.');
    const identityEvidence = resolveEvidence(
      actionMetadata.identity_evidence_id ?? null,
      evidenceCatalog,
      'identity',
    );
    if (identityEvidence) validateEvidence(identityEvidence, request.content_id);
    else if (
      actionMetadata.primary_slot &&
      (actionMetadata.provenance.kind !== 'accepted_model_hypothesis' ||
        actionMetadata.input_role_provenance?.kind !== 'accepted_model_hypothesis')
    )
      throw new Error('Owner primary slot has no native identity or accepted declaration.');
    if (
      actionMetadata.primary_slot &&
      (identityEvidence
        ? !isActionConfigTable(identityEvidence.table) ||
          !identityEvidence.pointer.includes('/CharacterAbilityList/')
        : actionMetadata.provenance.kind !== 'accepted_model_hypothesis' ||
          actionMetadata.input_role_provenance?.kind !== 'accepted_model_hypothesis')
    )
      throw new Error('Native primary slot has no character action identity binding.');
    if (actionMetadata.input_role_provenance) {
      validateProvenance(
        actionMetadata.input_role_provenance,
        actionMetadata.identity_evidence_id ?? null,
        evidenceCatalog,
        request.content_id,
        resolvedAssumptions,
      );
      recordResolvedProvenance(actionMetadata.input_role_provenance);
      if (actionMetadata.input_role === undefined || actionMetadata.input_role === null)
        throw new Error('Input role provenance has no resolved role.');
      if (
        !identityEvidence &&
        (actionMetadata.provenance.kind !== 'accepted_model_hypothesis' ||
          actionMetadata.input_role_provenance.kind !== 'accepted_model_hypothesis' ||
          actionMetadata.provenance.hypothesis_id !==
            actionMetadata.input_role_provenance.hypothesis_id)
      )
        throw new Error('Research input role is not bound to its accepted action declaration.');
    } else if (actionMetadata.input_role !== undefined && actionMetadata.input_role !== null) {
      throw new Error('Resolved input role has no provenance.');
    }
  };
  const validatePacketInterpretations = (packet: PacketMetadataReceipt) => {
    const fixedCritRate = packet.fixed_crit_rate;
    if (fixedCritRate !== undefined && fixedCritRate !== null) {
      const rate = exactFraction(fixedCritRate);
      if (rate.numerator < 0n || rate.numerator > rate.denominator)
        throw new Error('Invalid packet fixed-crit probability.');
      if (!packet.fixed_crit_rate_provenance)
        throw new Error('Packet fixed-crit value has no provenance.');
      validateProvenance(
        packet.fixed_crit_rate_provenance,
        packet.source_evidence_id ?? null,
        evidenceCatalog,
        request.content_id,
        resolvedAssumptions,
      );
      recordResolvedProvenance(packet.fixed_crit_rate_provenance);
    } else if (packet.fixed_crit_rate_provenance) {
      throw new Error('Packet fixed-crit provenance has no resolved value.');
    }
    const semanticTags = packet.semantic_tags || [];
    if (
      !Array.isArray(semanticTags) ||
      !semanticTags.every((tag) => typeof tag === 'string') ||
      new Set(semanticTags).size !== semanticTags.length
    )
      throw new Error('Invalid packet semantic tags.');
    if (semanticTags.length > 0) {
      if (!packet.semantic_provenance) throw new Error('Packet semantic tags have no provenance.');
      validateProvenance(
        packet.semantic_provenance,
        packet.source_evidence_id ?? null,
        evidenceCatalog,
        request.content_id,
        resolvedAssumptions,
      );
      recordResolvedProvenance(packet.semantic_provenance);
    } else if (packet.semantic_provenance) {
      throw new Error('Packet semantic provenance has no resolved tags.');
    }
  };
  const validateBinding = (metadata: ActionPacketMetadataReceipt) => {
    const { binding, action, packet } = metadata;
    if (
      !binding ||
      !['native_direct', 'owner_child', 'research_declared'].includes(binding.kind) ||
      binding.initiating_ability_id !== action.ability_id ||
      (binding.packet_ga_name ?? null) !== (packet.native_ga_name ?? null)
    )
      throw new Error('Action/packet relationship differs from its native identity receipt.');
    if (binding.kind === 'native_direct') {
      if (
        (binding.provenance ?? null) !== null ||
        typeof binding.packet_ga_name !== 'string' ||
        binding.packet_ga_name.toLowerCase() !== binding.initiating_ability_id.toLowerCase()
      )
        throw new Error('Native direct action/packet binding is inconsistent.');
      return;
    }
    if (!binding.provenance) throw new Error('Non-direct action/packet binding has no provenance.');
    validateProvenance(
      binding.provenance,
      packet.source_evidence_id ?? null,
      evidenceCatalog,
      request.content_id,
      resolvedAssumptions,
    );
    recordResolvedProvenance(binding.provenance);
    if (binding.kind === 'owner_child') {
      const bindingProvenance = binding.provenance;
      const modeledChild =
        (binding.packet_ga_name ?? null) === null &&
        action.provenance.kind === 'accepted_model_hypothesis' &&
        packet.provenance.kind === 'accepted_model_hypothesis' &&
        bindingProvenance.kind === 'accepted_model_hypothesis' &&
        [action.provenance, packet.provenance].every(
          (provenance) => provenance.hypothesis_id === bindingProvenance.hypothesis_id,
        );
      if (
        (!binding.packet_ga_name && !modeledChild) ||
        binding.provenance.kind !== 'accepted_model_hypothesis'
      )
        throw new Error('Owner child action/packet binding is not an accepted exact relationship.');
    }
  };
  return (metadataId: number, requireCategories: boolean): ActionPacketMetadataReceipt => {
    if (
      !Number.isSafeInteger(metadataId) ||
      metadataId < 0 ||
      metadataId >= metadataCatalog.length ||
      !metadataCatalog[metadataId]
    )
      throw new Error('Invalid or missing action/packet metadata catalog reference.');
    const metadata = metadataCatalog[metadataId];
    validateActionMetadata(metadata.action, requireCategories);
    validateProvenance(
      metadata.packet.provenance,
      metadata.packet.source_evidence_id ?? null,
      evidenceCatalog,
      request.content_id,
      resolvedAssumptions,
    );
    recordResolvedProvenance(metadata.packet.provenance);
    validatePacketInterpretations(metadata.packet);
    validateBinding(metadata);
    return metadata;
  };
}
