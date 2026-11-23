import type { ScenarioRequest } from '../contracts/scenario';
import type { ScenarioReport } from '../contracts/report';
import type { ClockIndex } from './clocks';

function integer(value: number, label: string, minimum = 0): void {
  if (!Number.isSafeInteger(value) || value < minimum)
    throw new Error(`Invalid ${label} in combat report.`);
}

function u64(value: number | string | bigint, label: string): bigint {
  try {
    const parsed = BigInt(value);
    if (parsed < 0n || parsed > (1n << 64n) - 1n) throw new Error();
    return parsed;
  } catch {
    throw new Error(`Invalid ${label} in combat report.`);
  }
}

function validateRng(audit: NonNullable<ScenarioReport['rng']>, seed: number): void {
  if (
    typeof audit.version !== 'string' ||
    audit.version.length === 0 ||
    audit.seed !== seed ||
    !audit.streams
  )
    throw new Error('Invalid RNG audit header.');
  for (const [name, stream] of Object.entries(audit.streams)) {
    if (!name) throw new Error('RNG audit contains an unnamed stream.');
    u64(stream.sequence, 'RNG sequence');
    u64(stream.state, 'RNG state');
    integer(stream.draws, 'RNG draw count');
  }
}

function validateIncoming(
  request: ScenarioRequest,
  report: ScenarioReport,
  clocks: ClockIndex,
): void {
  const incoming = report.incoming_damage_receipts || [];
  const actorsByIndex = request.actors;
  const incomingIndices = new Set<number>();
  for (const receipt of incoming) {
    integer(receipt.event_index, 'incoming event index');
    integer(receipt.time_us, 'incoming event time');
    integer(receipt.actor_index, 'incoming actor index');
    integer(receipt.delivered_damage, 'delivered damage');
    integer(receipt.hp_before, 'HP before damage');
    integer(receipt.actual_hp_loss, 'HP loss');
    integer(receipt.overflow, 'damage overflow');
    integer(receipt.prevented_hp_loss ?? 0, 'prevented HP loss');
    integer(receipt.hp_after, 'HP after damage');
    if (receipt.time_us > report.absolute_elapsed_us || receipt.actor_index >= actorsByIndex.length)
      throw new Error('Incoming damage receipt is outside its actor or time range.');
    const declared = request.incoming_damage_events?.[receipt.event_index];
    if (
      !declared ||
      incomingIndices.has(receipt.event_index) ||
      declared.timestamp_us !== clocks.relativeAt(receipt.time_us) ||
      declared.actor_index !== receipt.actor_index ||
      declared.delivered_damage !== receipt.delivered_damage
    )
      throw new Error('Incoming damage receipt differs from its declared event.');
    incomingIndices.add(receipt.event_index);
    const absorbed = receipt.absorbed_by_shields || [];
    let absorbedTotal = 0n;
    const shieldIds = new Set<string>();
    for (const shield of absorbed) {
      if (!shield.shield || shieldIds.has(shield.shield))
        throw new Error('Shield absorption has a missing or duplicate shield owner.');
      shieldIds.add(shield.shield);
      integer(shield.amount, 'shield absorption', 1);
      absorbedTotal += BigInt(shield.amount);
    }
    const absorbedEffects = receipt.absorbed_by_effects || [];
    const effectSources = new Set<number>();
    let effectAbsorbedTotal = 0n;
    for (const effect of absorbedEffects) {
      integer(effect.source_actor_index, 'incoming effect source');
      integer(effect.amount, 'incoming effect absorption');
      if (
        typeof effect.effect_id !== 'string' ||
        !effect.effect_id ||
        effect.source_actor_index >= actorsByIndex.length ||
        !absorbed.some((shield) => shield.shield === effect.broken_shield && shield.amount > 0) ||
        effectSources.has(effect.source_actor_index)
      )
        throw new Error('Incoming effect absorption has an unknown or unqualified owner.');
      effectSources.add(effect.source_actor_index);
      effectAbsorbedTotal += BigInt(effect.amount);
    }
    if (
      absorbedTotal +
        effectAbsorbedTotal +
        BigInt(receipt.prevented_hp_loss ?? 0) +
        BigInt(receipt.actual_hp_loss) +
        BigInt(receipt.overflow) !==
        BigInt(receipt.delivered_damage) ||
      receipt.actual_hp_loss > receipt.hp_before ||
      (receipt.overflow > 0 && receipt.hp_after !== 0) ||
      receipt.hp_after !== receipt.hp_before - receipt.actual_hp_loss ||
      receipt.defeated !== (receipt.hp_after === 0)
    )
      throw new Error('Incoming damage receipt does not conserve shield, HP, and overflow.');
  }
  if (report.success && report.execution_completed)
    for (const [index, event] of (request.incoming_damage_events || []).entries())
      if (event.timestamp_us <= report.relative_elapsed_us && !incomingIndices.has(index))
        throw new Error('Incoming damage report is missing a settled event.');
}

function validateHpCosts(request: ScenarioRequest, report: ScenarioReport): void {
  const seen = new Set<number>();
  for (const receipt of report.hp_cost_receipts ?? []) {
    for (const key of [
      'event_index',
      'time_us',
      'actor_index',
      'hp_before',
      'actual_hp_loss',
      'hp_after',
    ] as const)
      integer(receipt[key], `HP cost ${key}`);
    const declared = request.hp_cost_events?.[receipt.event_index];
    if (
      !declared ||
      seen.has(receipt.event_index) ||
      !request.actors[receipt.actor_index] ||
      receipt.time_us > report.absolute_elapsed_us ||
      declared.timestamp_us !== receipt.time_us ||
      declared.actor_index !== receipt.actor_index ||
      declared.source !== receipt.source ||
      u64(declared.amount, 'requested HP cost') !== u64(receipt.requested_hp, 'receipt HP cost')
    )
      throw new Error('HP cost receipt differs from its declared event.');
    seen.add(receipt.event_index);
  }
  if (report.execution_completed)
    for (const [index, event] of (request.hp_cost_events ?? []).entries())
      if (event.timestamp_us <= report.absolute_elapsed_us && !seen.has(index))
        throw new Error('Completed report is missing an HP cost receipt.');
}

function validateShields(request: ScenarioRequest, report: ScenarioReport): void {
  const observedTime = report.final_buff_state.time_us;
  const finalBySlot = new Map(report.final_actor_states.map((actor) => [actor.slot, actor]));
  for (const config of request.actors) {
    const actor = finalBySlot.get(config.slot);
    if (!actor) continue;
    const shields = actor.active_shields || [];
    const ids = new Set<string>();
    for (const shield of shields) {
      if (!shield.id || !shield.source || ids.has(shield.id))
        throw new Error('Final actor shield state has a missing or duplicate identity.');
      ids.add(shield.id);
      integer(shield.remaining_hp, 'remaining shield HP', 1);
      integer(shield.max_hp, 'maximum shield HP', 1);
      integer(shield.expires_at_us, 'shield expiry');
      if (
        shield.remaining_hp > shield.max_hp ||
        shield.clock_basis?.kind !== 'world' ||
        shield.expires_at_us <= observedTime ||
        actor.hp === 0
      )
        throw new Error(
          'Final actor shield is expired, over capacity, or belongs to a defeated actor.',
        );
    }
  }
}

function validateProc(request: ScenarioRequest, report: ScenarioReport): void {
  const audits = new Map<number, NonNullable<ScenarioReport['rng']>>();
  for (const entry of report.proc_rng ?? []) {
    integer(entry.actor_index, 'proc RNG actor');
    if (!request.actors[entry.actor_index] || audits.has(entry.actor_index))
      throw new Error('Unexpected or duplicate proc RNG owner.');
    validateRng(entry.rng, report.seed);
    audits.set(entry.actor_index, entry.rng);
  }
  for (const receipt of report.proc_selection_receipts ?? []) {
    integer(receipt.time_us, 'proc time');
    integer(receipt.actor_index, 'proc actor');
    integer(receipt.outcome_index, 'proc outcome');
    integer(receipt.cooldown_until_us, 'proc cooldown deadline');
    const sample = receipt.sample;
    if (
      !request.actors[receipt.actor_index] ||
      receipt.time_us > report.absolute_elapsed_us ||
      !audits.get(receipt.actor_index)?.streams[receipt.stream]
    )
      throw new Error('Proc receipt has an unknown actor, time, or stream.');
    integer(sample.bound, 'sample bound', 1);
    integer(sample.value, 'sample value');
    if (
      sample.value >= sample.bound ||
      sample.value !== receipt.outcome_index ||
      !Array.isArray(sample.draws) ||
      sample.draws.length === 0
    )
      throw new Error('Proc sample has invalid bounds or outcome metadata.');
  }
}

// RNG outcomes, eligibility and cooldown rules are checked by the Rust producer.

export function validateVitalsAndProc(
  request: ScenarioRequest,
  report: ScenarioReport,
  clocks: ClockIndex,
): void {
  if (report.reaction_rng) validateRng(report.reaction_rng, report.seed);
  validateIncoming(request, report, clocks);
  validateHpCosts(request, report);
  validateShields(request, report);
  validateProc(request, report);
}
