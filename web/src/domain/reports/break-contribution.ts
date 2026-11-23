import type {
  ActionMetadataOccurrence,
  ActionPacketMetadataReceipt,
  HitReceipt,
  PlayerInputReceipt,
} from '../contracts/report';
import type { ScenarioRequest } from '../contracts/scenario';
import {
  exactFraction,
  validateHitCommand as validateInitiatingCommand,
} from './validation-support';

export function validateHitCommand(
  request: ScenarioRequest,
  hit: Pick<
    HitReceipt,
    | 'command_index'
    | 'autonomous_event_index'
    | 'actor_index'
    | 'ability_id'
    | 'damage_effect_id'
    | 'break_contribution'
    | 'metadata_id'
    | 'switch_origin'
  >,
  metadataCatalog: ActionPacketMetadataReceipt[],
) {
  validateBreakContribution(hit, metadataCatalog);
  validateInitiatingCommand(request, hit);
}

function validateBreakContribution(
  hit: Pick<
    HitReceipt,
    | 'command_index'
    | 'autonomous_event_index'
    | 'ability_id'
    | 'damage_effect_id'
    | 'break_contribution'
    | 'metadata_id'
  >,
  metadataCatalog: ActionPacketMetadataReceipt[],
) {
  const contribution = hit.break_contribution;
  if (!contribution) return;
  if (
    hit.ability_id !== 'ntesim.break' ||
    hit.damage_effect_id !== 'ntesim.break.contribution' ||
    !Number.isSafeInteger(hit.command_index ?? hit.autonomous_event_index) ||
    (hit.command_index ?? hit.autonomous_event_index)! < 0 ||
    !Number.isSafeInteger(contribution.generation) ||
    contribution.generation <= 0 ||
    !Number.isSafeInteger(contribution.character_level) ||
    contribution.character_level <= 0
  )
    throw new Error('Invalid Break contribution observation identity.');
  for (const value of [
    contribution.base_break,
    contribution.original_capacity,
    contribution.intensity,
    contribution.bonus,
    contribution.defense_ignore,
  ]) {
    if (exactFraction(value).numerator < 0n)
      throw new Error('Invalid Break contribution fraction.');
  }
  const metadata = metadataCatalog[hit.metadata_id];
  if (!metadata) throw new Error('Break contribution has missing metadata catalog entry.');
  for (const provenance of [
    metadata.action.provenance,
    metadata.packet.provenance,
    metadata.binding.provenance,
  ]) {
    if (
      !provenance ||
      provenance.kind !== 'accepted_model_hypothesis' ||
      provenance.hypothesis_id !== 'combat.live_break_damage_v1'
    ) {
      throw new Error('Break contribution has no matching modeled relationship.');
    }
  }
}

export function hasInitiatingOccurrence(
  hit: HitReceipt,
  ledger: ActionMetadataOccurrence[],
  metadataCatalog: ActionPacketMetadataReceipt[],
  inputs: PlayerInputReceipt[] = [],
) {
  const hitMetadata = metadataCatalog[hit.metadata_id];
  if (!hitMetadata) return false;
  // Reactions and Break from persistent Support damage can originate in the
  // admitted switch without a damaging ability cast. Command participants are
  // checked separately by validateHitCommand.
  if (
    hit.switch_origin?.kind === 'esper_cycle' ||
    hit.switch_origin?.kind === 'esper_cycle_replacement' ||
    (hit.break_contribution && hit.switch_origin?.kind === 'support')
  )
    return true;
  if (
    inputs.some(
      (input) =>
        input.input === 'arc' &&
        input.command_index === hit.command_index &&
        input.actor_index === hit.actor_index &&
        input.ability_id === hit.ability_id &&
        input.time_us <= hit.time_us &&
        hitMetadata.binding.kind === 'owner_child' &&
        hitMetadata.binding.initiating_ability_id === input.ability_id,
    )
  )
    return true;
  return ledger.some((occurrence) => {
    if (hit.command_index !== undefined && !occurrence.command_indices.includes(hit.command_index))
      return false;
    // Break is a consequence of the initiating command, with independent contributor ownership.
    if (hit.break_contribution) return true;
    const occurrenceMetadata = metadataCatalog[occurrence.metadata_id];
    if (!occurrenceMetadata) return false;
    return (
      occurrence.actor_index === hit.actor_index &&
      occurrence.ability_id === hit.ability_id &&
      JSON.stringify(occurrenceMetadata.action) === JSON.stringify(hitMetadata.action) &&
      (occurrence.damage_effect_id !== hit.damage_effect_id ||
        occurrence.metadata_id === hit.metadata_id ||
        JSON.stringify(occurrenceMetadata) === JSON.stringify(hitMetadata))
    );
  });
}
