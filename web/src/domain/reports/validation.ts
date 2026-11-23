import type { LiveBranch, LiveEnergyWait, ScenarioRequest } from '../contracts/scenario';
import type { MetadataProvenance, ScenarioReport } from '../contracts/report';
import { fraction } from './statistics';
import {
  exactFraction,
  validateAssumptions,
  validateAssumptionIds,
  validateEnergyGroupId,
  validateEvidence,
  validateTargetStatuses,
  validateTraceTimes,
  validateNonnegativeObservation as finite,
} from './validation-support';
import { createMetadataValidator } from './validation-metadata';
import { validateEnergy } from './energy';
import { validateRejectedAction } from './rejection';
import { validatePlayerInputs } from './player-inputs';
import { validateVitalsAndProc } from './vitals';
import { validateLiveBranchReceipts } from './live-branch-validation';
import { createTargetVitalsValidator } from './target-vitals';
import { validateHitCommand } from './break-contribution';
import { validateDamageParent } from './damage-parent';
import { validateReactionOrigin } from './effect-origin';
import {
  hasAutonomousOrigin,
  validateAutonomousEvents,
  validateInitiatingOccurrences,
} from './autonomous';
import { validateStateTimeline } from './timeline';
import { ClockIndex } from './clocks';

export function validateObservations(
  request: ScenarioRequest,
  report: ScenarioReport,
  liveBranches: LiveBranch[] = [],
  liveEnergyWaits: LiveEnergyWait[] = [],
) {
  const clocks = new ClockIndex(report);
  if (
    report.schema_version !== request.schema_version ||
    report.content_id !== request.content_id ||
    report.clock_contract !== request.clock_contract ||
    report.timer_policy !== request.timer_policy ||
    report.absolute_limit_us !== request.absolute_limit_us ||
    report.relative_duration_us !== request.relative_duration_us ||
    report.seed !== request.seed
  )
    throw new Error('Report identity differs from the declared scenario.');
  validateLiveBranchReceipts(request, liveBranches, liveEnergyWaits, report);
  const declaredAssumptions = request.model_assumptions || {},
    observedAssumptions = report.model_assumptions || {};
  if (
    Object.keys(declaredAssumptions).length !== Object.keys(observedAssumptions).length ||
    Object.entries(declaredAssumptions).some(
      ([id, description]) => observedAssumptions[id] !== description,
    )
  )
    throw new Error('Report assumptions differ from the declared scenario.');
  const resolvedAssumptions = report.resolved_model_assumptions || {};
  validateAssumptions(resolvedAssumptions, 'resolved model assumptions');
  const usedResolvedAssumptions = new Map<string, string>();
  for (const [id, description] of Object.entries(resolvedAssumptions))
    if (declaredAssumptions[id] !== undefined && declaredAssumptions[id] !== description)
      throw new Error('Resolved model assumption conflicts with the declared scenario.');
  const recordAssumptionId = (hypothesisId: string, expectedDescription?: string) => {
    const resolved = resolvedAssumptions[hypothesisId];
    if (
      typeof resolved !== 'string' ||
      !resolved.trim() ||
      (expectedDescription !== undefined && resolved !== expectedDescription)
    )
      throw new Error('Resolved model assumptions differ from action and hit receipt provenance.');
    const previous = usedResolvedAssumptions.get(hypothesisId);
    if (previous !== undefined && previous !== resolved)
      throw new Error('Action or hit receipts contain conflicting resolved model assumptions.');
    usedResolvedAssumptions.set(hypothesisId, resolved);
  };
  const recordResolvedProvenance = (provenance: MetadataProvenance) => {
    if (provenance.kind !== 'accepted_model_hypothesis') return;
    recordAssumptionId(provenance.hypothesis_id);
  };
  if (report.evidence_catalog !== undefined && !Array.isArray(report.evidence_catalog))
    throw new Error('Invalid native action or packet evidence catalog.');
  const evidenceCatalog = report.evidence_catalog ?? [];
  for (const evidence of evidenceCatalog) validateEvidence(evidence, request.content_id);
  if (report.metadata_catalog !== undefined && !Array.isArray(report.metadata_catalog))
    throw new Error('Invalid action/packet metadata catalog.');
  const metadataCatalog = report.metadata_catalog ?? [];
  const resolveCatalogMetadata = createMetadataValidator(
    request,
    evidenceCatalog,
    metadataCatalog,
    resolvedAssumptions,
    recordResolvedProvenance,
  );
  validateAssumptions(report.callback_model_assumptions || {}, 'callback model assumptions');
  for (const [id, description] of Object.entries(report.callback_model_assumptions || {}))
    recordAssumptionId(id, description);
  const actors = new Map(request.actors.map((actor) => [actor.slot, actor.character_id]));
  const targets = new Set(request.targets.map((target) => target.target_index));
  if (
    report.final_actor_states.length !== actors.size ||
    new Set(report.final_actor_states.map((state) => state.slot)).size !== actors.size ||
    report.final_actor_states.some((state) => actors.get(state.slot) !== state.character_id)
  )
    throw new Error('Report actor states differ from the declared team.');
  if (
    report.final_target_states.length !== targets.size ||
    new Set(report.final_target_states.map((state) => state.target_index)).size !== targets.size ||
    report.final_target_states.some((state) => !targets.has(state.target_index))
  )
    throw new Error('Report target states differ from the declared targets.');
  validateTargetStatuses(request, report.final_target_states);
  validateVitalsAndProc(request, report, clocks);
  validateAutonomousEvents(request, report);
  finite(report.total_damage_dealt);
  if (report.hits_settled !== report.hit_receipts.length)
    throw new Error('Report hit count differs from receipts.');
  let receiptDamage = 0n;
  const targetVitals = createTargetVitalsValidator(request);
  for (const [hitIndex, hit] of report.hit_receipts.entries()) {
    if (
      hit.autonomous_event_index !== undefined &&
      !hasAutonomousOrigin(hit, report.autonomous_events ?? [])
    )
      throw new Error('Hit receipt has no matching autonomous event.');
    const metadata = resolveCatalogMetadata(hit.metadata_id, false);
    if (
      metadata.action.ability_id !== hit.ability_id ||
      metadata.packet.damage_effect_id !== hit.damage_effect_id
    )
      throw new Error('Hit receipt metadata does not match its ability and packet owner.');
    if (!validateDamageParent(report, hitIndex)) validateHitCommand(request, hit, metadataCatalog);
    validateReactionOrigin(report, hitIndex);
    validateAssumptionIds(
      hit.execution_model_assumption_ids,
      resolvedAssumptions,
      'hit execution assumptions',
      recordAssumptionId,
    );
    validateEnergyGroupId(hit.energy_group_id);
    if (!actors.has(hit.actor_index) || !targets.has(hit.target_index))
      throw new Error('Hit receipt has an unknown owner or target.');
    finite(hit.time_us);
    if (hit.time_us > report.absolute_elapsed_us)
      throw new Error('Hit receipt is outside the executed absolute interval.');
    finite(fraction(hit.damage));
    finite(hit.applied_hp_loss);
    receiptDamage += targetVitals.settle(hit, exactFraction(hit.damage));
  }
  targetVitals.finish(report.final_target_states);
  const actionLedger = report.action_metadata_ledger;
  for (const receipt of validatePlayerInputs(request, report)) {
    validateAssumptionIds(
      receipt.execution_model_assumption_ids,
      resolvedAssumptions,
      'player input execution assumptions',
      recordAssumptionId,
    );
  }
  if (actionLedger !== undefined && !Array.isArray(actionLedger))
    throw new Error('Invalid action metadata occurrence ledger.');
  for (const occurrence of actionLedger || []) {
    if (!Array.isArray(occurrence.command_indices) || occurrence.command_indices.length === 0)
      throw new Error('Action metadata occurrence must have at least one command index.');
    if (new Set(occurrence.command_indices).size !== occurrence.command_indices.length)
      throw new Error('Action metadata occurrence contains duplicate command indices.');
    validateAssumptionIds(
      occurrence.execution_model_assumption_ids,
      resolvedAssumptions,
      'action execution assumptions',
      recordAssumptionId,
    );
    validateEnergyGroupId(occurrence.energy_group_id);
    const metadata = resolveCatalogMetadata(occurrence.metadata_id, true);
    if (
      !actors.has(occurrence.actor_index) ||
      typeof occurrence.ability_id !== 'string' ||
      typeof occurrence.damage_effect_id !== 'string' ||
      metadata.action.ability_id !== occurrence.ability_id ||
      metadata.packet.damage_effect_id !== occurrence.damage_effect_id
    )
      throw new Error('Action metadata occurrence differs from its admitted owner and packet.');
    const binding = metadata.binding;
    for (const commandIndex of occurrence.command_indices) {
      if (!Number.isSafeInteger(commandIndex) || commandIndex < 0)
        throw new Error('Action metadata occurrence differs from its admitted owner and packet.');
      validateHitCommand(
        request,
        {
          command_index: commandIndex,
          actor_index: occurrence.actor_index,
          ability_id: occurrence.ability_id,
          damage_effect_id: occurrence.damage_effect_id,
          metadata_id: occurrence.metadata_id,
          switch_origin: occurrence.switch_origin,
        },
        metadataCatalog,
      );
      const declaration = request.action_metadata?.[String(commandIndex)];
      if (binding.kind === 'research_declared') {
        if (
          !declaration ||
          binding.provenance?.kind !== 'accepted_model_hypothesis' ||
          declaration.hypothesis_id !== binding.provenance.hypothesis_id ||
          (declaration.packet_ga_name_binding ?? null) !== (binding.packet_ga_name ?? null)
        )
          throw new Error('Research packet relationship differs from its declared action binding.');
      } else if (declaration?.packet_ga_name_binding) {
        throw new Error('Declared research packet GAName binding was not used.');
      }
    }
  }
  validateInitiatingOccurrences(report);
  if (
    usedResolvedAssumptions.size !== Object.keys(resolvedAssumptions).length ||
    Object.entries(resolvedAssumptions).some(
      ([id, description]) => usedResolvedAssumptions.get(id) !== description,
    )
  )
    throw new Error('Resolved model assumptions differ from action and hit receipt provenance.');
  if (report.total_damage_exact !== `${receiptDamage}/1`)
    throw new Error('Total damage differs from the rounded hit receipt ledger.');
  const exactAsNumber = Number(receiptDamage);
  if (!Number.isFinite(exactAsNumber) || report.total_damage_dealt !== exactAsNumber)
    throw new Error('Rounded total damage differs from the exact hit receipt ledger.');
  validateTraceTimes(report);
  validateEnergy(request, report);
  validateRejectedAction(request, report);
  validateStateTimeline(report);
}
