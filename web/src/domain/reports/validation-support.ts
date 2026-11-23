import type { ExactFraction } from '../contracts/ratio';
import type {
  ActionMetadataReceipt,
  MetadataProvenance,
  NativeEvidence,
  ScenarioReport,
} from '../contracts/report';
import type { ScenarioRequest } from '../contracts/scenario';

export function validateNonnegativeObservation(value: number) {
  if (!Number.isFinite(value) || value < 0) throw new Error('Invalid report observation.');
}

export function validateTraceTimes(report: ScenarioReport) {
  for (const event of report.trace) {
    validateNonnegativeObservation(event.time_us);
    if (event.time_us > report.absolute_elapsed_us)
      throw new Error('Trace event is outside the report horizon.');
  }
}

export function expectedActionTime(
  request: ScenarioRequest,
  report: ScenarioReport,
  commandIndex: number,
) {
  const command = request.actions[commandIndex];
  if (!command) return undefined;
  const delta =
    (report.live_branch_receipts ?? [])
      .filter((branch) => commandIndex >= branch.after_action_index)
      .reduce((sum, branch) => sum + branch.selected_wait_us - branch.baseline_wait_us, 0) +
    (report.live_energy_wait_receipts ?? [])
      .filter((wait) => commandIndex >= wait.after_action_index && wait.resumed_time_us !== null)
      .reduce((sum, wait) => sum + wait.resumed_time_us! - wait.started_time_us, 0);
  const time = command.timestamp_us + delta;
  if (!Number.isSafeInteger(time) || time < 0)
    throw new Error('Live program produced an invalid action timestamp.');
  return time;
}

export function validateHitCommand(
  request: ScenarioRequest,
  hit: Pick<
    ScenarioReport['hit_receipts'][number],
    'command_index' | 'actor_index' | 'ability_id' | 'break_contribution' | 'switch_origin'
  >,
) {
  if (hit.command_index === undefined) {
    if (hit.switch_origin) throw new Error('Switch effect has no initiating command.');
    return;
  }
  const command = request.actions[hit.command_index];
  const origin = hit.switch_origin;
  if (command?.kind === 'switch_actor' && origin) {
    const incoming = origin.incoming_actor_index;
    const validIncoming =
      Number.isSafeInteger(incoming) &&
      !!request.actors[incoming] &&
      incoming === command.target_actor_index;
    const validOwner =
      origin.kind === 'support'
        ? hit.break_contribution || hit.actor_index === incoming
        : (origin.kind === 'esper_cycle' || origin.kind === 'esper_cycle_replacement') &&
          Number.isSafeInteger(origin.outgoing_actor_index) &&
          !!request.actors[origin.outgoing_actor_index] &&
          origin.outgoing_actor_index !== incoming &&
          (origin.kind === 'esper_cycle'
            ? [origin.outgoing_actor_index, incoming].includes(hit.actor_index)
            : Number.isSafeInteger(origin.owner_actor_index) &&
              !!request.actors[origin.owner_actor_index] &&
              origin.owner_actor_index === hit.actor_index);
    if (
      Number.isSafeInteger(hit.command_index) &&
      hit.command_index >= 0 &&
      validIncoming &&
      validOwner
    )
      return;
    throw new Error('Switch effect differs from its initiating participants.');
  }
  if (
    !Number.isSafeInteger(hit.command_index) ||
    hit.command_index < 0 ||
    !command ||
    origin ||
    command.kind === 'wait' ||
    command.kind === 'switch_actor' ||
    (!hit.break_contribution && command.actor_index !== hit.actor_index) ||
    (!hit.break_contribution &&
      command.kind === 'invoke_ability' &&
      command.ability_id !== hit.ability_id)
  )
    throw new Error('Hit receipt command identity differs from its initiating action.');
}

export function gcd(left: bigint, right: bigint) {
  let a = left < 0n ? -left : left,
    b = right < 0n ? -right : right;
  while (b !== 0n) [a, b] = [b, a % b];
  return a;
}

const CANONICAL_RATIONAL = /^(-?(?:0|[1-9]\d*))\/([1-9]\d*)$/;

export function exactFraction(value: ExactFraction) {
  if (typeof value !== 'string') throw new Error('Invalid report damage fraction string.');
  const match = CANONICAL_RATIONAL.exec(value);
  if (!match) throw new Error('Invalid canonical report fraction format.');
  const numerator = BigInt(match[1]),
    denominator = BigInt(match[2]);
  if (denominator <= 0n) throw new Error('Invalid report damage denominator.');
  const divisor = gcd(numerator, denominator);
  return { numerator: numerator / divisor, denominator: denominator / divisor };
}

export function equalExact(
  left: { numerator: bigint; denominator: bigint },
  right: { numerator: bigint; denominator: bigint },
) {
  return left.numerator === right.numerator && left.denominator === right.denominator;
}
export function validateAssumptions(values: Record<string, string>, name: string) {
  if (Object.keys(values).length > 64) throw new Error(`Report ${name} exceeds 64 entries.`);
  for (const [id, description] of Object.entries(values))
    if (
      !/^[a-zA-Z0-9_:\-./]{1,128}$/.test(id) ||
      typeof description !== 'string' ||
      !description.trim() ||
      new TextEncoder().encode(description).length > 4096
    )
      throw new Error(`Invalid report ${name} entry.`);
}
export function validateAssumptionIds(
  ids: string[] | undefined,
  resolved: Record<string, string>,
  name: string,
  record: (hypothesisId: string) => void,
) {
  if (ids === undefined) return;
  if (!Array.isArray(ids) || ids.length > 64)
    throw new Error(`Invalid report ${name} entry list in resolved model assumptions.`);
  const seen = new Set<string>();
  for (const id of ids) {
    if (typeof id !== 'string' || !/^[a-zA-Z0-9_:\-./]{1,128}$/.test(id) || seen.has(id))
      throw new Error(`Invalid report ${name} entry.`);
    const description = resolved[id];
    if (typeof description !== 'string' || !description.trim())
      throw new Error(`Invalid or unresolved report ${name} entry in resolved model assumptions.`);
    seen.add(id);
    record(id);
  }
}
export function validateEnergyGroupId(energyGroupId: string | undefined) {
  if (
    energyGroupId !== undefined &&
    (typeof energyGroupId !== 'string' ||
      energyGroupId.length > 64 ||
      !/^packet:[0-9]+$/.test(energyGroupId))
  )
    throw new Error('Invalid packet energy-group identity.');
}
export function primarySlotForInputRole(role: NonNullable<ActionMetadataReceipt['input_role']>) {
  switch (role) {
    case 'normal':
      return 'normal';
    case 'redirect_skill':
      return 'skill';
    case 'ultimate':
      return 'ultimate';
    default:
      return null;
  }
}
export function isActionConfigTable(table: string | undefined) {
  return table === 'typed_character_actions' || table === 'DT_CharacterAbilityConfig';
}
export function validateEvidence(evidence: NativeEvidence, packageId: string) {
  if (
    !evidence ||
    evidence.content_id !== packageId ||
    typeof evidence.table !== 'string' ||
    typeof evidence.row !== 'string' ||
    typeof evidence.pointer !== 'string' ||
    !(
      evidence.pointer.startsWith('/rows/') ||
      (isActionConfigTable(evidence.table) && evidence.pointer.startsWith('/CharacterAbilityList/'))
    ) ||
    !/^[a-f0-9]{64}$/i.test(evidence.row_sha256)
  )
    throw new Error('Invalid native action or packet source evidence.');
}
export function resolveEvidence(
  evidenceId: number | null | undefined,
  evidenceCatalog: NativeEvidence[],
  label: string,
): NativeEvidence | null {
  if (evidenceId === undefined || evidenceId === null) return null;
  if (
    !Number.isSafeInteger(evidenceId) ||
    evidenceId < 0 ||
    evidenceId >= evidenceCatalog.length ||
    !evidenceCatalog[evidenceId]
  )
    throw new Error(`Invalid native action or packet ${label} evidence reference.`);
  return evidenceCatalog[evidenceId];
}
export function validateProvenance(
  provenance: MetadataProvenance,
  evidenceId: number | null | undefined,
  evidenceCatalog: NativeEvidence[],
  packageId: string,
  resolved: Record<string, string>,
) {
  const evidence = resolveEvidence(evidenceId, evidenceCatalog, 'source');
  if (provenance.kind === 'native') {
    const provenanceEvidence = resolveEvidence(
      provenance.evidence_id,
      evidenceCatalog,
      'provenance',
    );
    if (!provenanceEvidence) throw new Error('Native provenance has invalid evidence reference.');
    validateEvidence(provenanceEvidence, packageId);
    if (!evidence) throw new Error('Native provenance has no matching source evidence.');
    validateEvidence(evidence, packageId);
    if (
      provenance.evidence_id !== evidenceId &&
      (evidence.content_id !== provenanceEvidence.content_id ||
        evidence.table !== provenanceEvidence.table ||
        evidence.row !== provenanceEvidence.row ||
        evidence.pointer !== provenanceEvidence.pointer ||
        evidence.row_sha256 !== provenanceEvidence.row_sha256)
    )
      throw new Error('Native provenance differs from its source evidence.');
    return;
  }
  if (
    provenance.kind !== 'accepted_model_hypothesis' ||
    !provenance.hypothesis_id ||
    !/^[a-zA-Z0-9_:\-./]{1,128}$/.test(provenance.hypothesis_id) ||
    typeof resolved[provenance.hypothesis_id] !== 'string' ||
    !resolved[provenance.hypothesis_id].trim()
  )
    throw new Error('Resolved model provenance is absent from resolved model assumptions.');
  if (evidence) validateEvidence(evidence, packageId);
}

export function validateTargetStatuses(
  request: ScenarioRequest,
  targets: ScenarioReport['final_target_states'],
) {
  for (const target of targets) {
    for (const status of target.statuses || []) {
      if (
        !Number.isSafeInteger(status.source_actor_index) ||
        status.source_actor_index < 0 ||
        status.source_actor_index >= request.actors.length ||
        status.clock_basis?.kind !== 'world' ||
        typeof status.id !== 'string' ||
        !status.id ||
        !Number.isSafeInteger(status.expires_at_us) ||
        status.expires_at_us < 0 ||
        !Number.isSafeInteger(status.stacks) ||
        status.stacks < 1 ||
        !status.resistance_reductions ||
        typeof status.resistance_reductions !== 'object' ||
        !Array.isArray(status.semantic_tags || []) ||
        !(status.semantic_tags || []).every((tag) => typeof tag === 'string')
      )
        throw new Error('Invalid target status receipt.');
      for (const reduction of Object.values(status.resistance_reductions)) exactFraction(reduction);
    }
  }
}
