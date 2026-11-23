import type { ActiveBuffRef, BuffCatalogEntry, StateFrame } from '../contracts/timeline';
import type { ActorState, ScenarioReport } from '../contracts/report';
import { sameObservation } from './identity';
import { ClockIndex } from './clocks';
import { validateNonnegativeObservation as finite } from './validation-support';
import {
  actorDeltaHasChanges,
  actorSummaryFromDelta,
  advanceBuffsTo,
  applyActorDelta,
  applyBuffDiff,
  assertTimelineInt,
  reconstructTargetStatuses,
  validateBuffRef,
  validateUniqueBuffs,
} from './timeline-fold';

function assertIdleGapReconciliation(
  foldedActors: ActorState[],
  foldedTargets: ScenarioReport['final_target_states'],
  foldedBuffs: ActiveBuffRef[],
  catalog: BuffCatalogEntry[],
  finalTimeUs: number,
  hpMaxChanged: Set<number>,
  report: ScenarioReport,
) {
  for (let i = 0; i < foldedActors.length; i++) {
    const folded = foldedActors[i],
      final = report.final_actor_states[i];
    for (const key of [
      'slot',
      'character_id',
      'attack_mode',
      'energy',
      'energy_capacity',
      'energy_conserved',
      'initial_energy',
      'granted_energy',
      'spent_energy',
      'overflow_energy',
      'charge_remainder',
      'cycle',
      'resources',
    ] as const) {
      if (!sameObservation(folded[key], final[key]))
        throw new Error('Folded timeline actor non-expiry state differs from final state.');
    }
    const { ['hotori.timepiece']: _f, ...foldedFrac } = folded.fractional_resources ?? {};
    const { ['hotori.timepiece']: _l, ...finalFrac } = final.fractional_resources ?? {};
    if (!sameObservation(foldedFrac, finalFrac))
      throw new Error('Folded timeline fractional resources differ from final state.');
    const survivingShields = (folded.active_shields ?? []).filter(
      (s) => s.expires_at_us > finalTimeUs,
    );
    if (!sameObservation(survivingShields, final.active_shields ?? []))
      throw new Error('Folded timeline shields differ from final actor state.');
    const ffStacks = foldedBuffs
      .filter((r) => {
        const def = catalog[r.buff_def_id];
        return (
          def.id === 'fireflies.crit_damage' &&
          def.source === 'set:fireflies' &&
          def.owner_actor === folded.slot &&
          def.recipient.type === 'specific_actor' &&
          def.recipient.target === folded.slot
        );
      })
      .map((r) => {
        if (r.duration.kind !== 'timed') throw new Error('Fireflies stacks must be timed.');
        const bonus = catalog[r.buff_def_id].stat_modifiers.crit_damage_bonus;
        if (!bonus) throw new Error('Fireflies stack lacks critical damage value.');
        return {
          stacks: r.stacks,
          expires_at_us: r.duration.expires_at_us,
          crit_damage_bonus: bonus,
        };
      });
    const ffTotal = ffStacks.reduce((sum, s) => sum + s.stacks, 0);
    const activeFf = ffStacks.map(({ expires_at_us, crit_damage_bonus }) => ({
      expires_at_us,
      crit_damage_bonus,
    }));
    if (
      ffTotal !== final.fireflies_stacks ||
      !sameObservation(activeFf, final.active_fireflies_stacks)
    )
      throw new Error('Folded timeline Fireflies stacks differ from final actor state.');
    if (hpMaxChanged.has(folded.slot)) {
      if (final.hp_max > folded.hp_max || final.hp > folded.hp || final.hp > final.hp_max)
        throw new Error('Folded timeline HP clamp after buff expiry does not reconcile.');
    } else if (folded.hp !== final.hp || folded.hp_max !== final.hp_max) {
      throw new Error('Folded timeline actor HP differs from final actor state.');
    }
  }
  for (let j = 0; j < foldedTargets.length; j++) {
    const candidate = structuredClone(foldedTargets[j]);
    const statuses = reconstructTargetStatuses(foldedBuffs, catalog, j);
    if (statuses.length > 0) candidate.statuses = statuses;
    else delete candidate.statuses;
    if (!sameObservation(candidate, report.final_target_states[j]))
      throw new Error('Folded timeline target state differs from final target state.');
  }
}

export function validateStateTimeline(report: ScenarioReport) {
  const frames: StateFrame[] = report.state_timeline;
  if (!Array.isArray(frames) || frames.length < 2)
    throw new Error('State timeline requires initial and final boundary frames.');
  const nActors = report.final_actor_states.length,
    nTargets = report.final_target_states.length;
  if (report.buff_catalog !== undefined && !Array.isArray(report.buff_catalog))
    throw new Error('Invalid buff_catalog in combat report.');
  const catalog = report.buff_catalog ?? [];
  for (const def of catalog) {
    if (
      !def ||
      typeof def.id !== 'string' ||
      !def.id ||
      typeof def.source !== 'string' ||
      !def.source ||
      !Number.isSafeInteger(def.owner_actor) ||
      def.owner_actor < 0 ||
      def.owner_actor >= nActors ||
      !Number.isSafeInteger(def.max_stacks) ||
      def.max_stacks <= 0
    )
      throw new Error('buff_catalog contains invalid definition.');
  }
  const first = frames[0],
    last = frames[frames.length - 1];
  const clocks = new ClockIndex(report);
  if (!Array.isArray(first.timers) || !Array.isArray(last.timers))
    throw new Error('Initial and final frames require native timer snapshots.');
  if (!Array.isArray(first.buffs) || !Array.isArray(last.buffs))
    throw new Error('Initial and final state frames require buffs snapshot.');
  const initialActors = (first.actors ?? []).map(actorSummaryFromDelta),
    finalActors = (last.actors ?? []).map(actorSummaryFromDelta);
  if (
    first.boundary.kind !== 'initial' ||
    first.time_us !== 0 ||
    first.buff_diff ||
    last.boundary.kind !== 'final' ||
    last.time_us !== report.absolute_elapsed_us ||
    last.time_us !== report.final_buff_state.time_us ||
    last.buff_diff ||
    initialActors.length !== nActors ||
    initialActors.some((a, i) => a.slot !== i) ||
    (first.targets ?? []).length !== nTargets ||
    (first.targets ?? []).some((t, j) => t.target_index !== j) ||
    !sameObservation(finalActors, report.final_actor_states) ||
    !sameObservation(last.targets ?? [], report.final_target_states) ||
    !sameObservation(last.buffs, report.final_buff_state.instances) ||
    last.active_actor_index !== report.final_buff_state.active_actor_index ||
    last.hits_end !== report.hit_receipts.length ||
    last.healing_end !== report.healing_receipts.length ||
    last.trace_end !== report.trace.length
  )
    throw new Error('State timeline does not reconcile native boundary states or receipts.');
  validateUniqueBuffs(first.buffs, catalog, nActors);
  validateUniqueBuffs(last.buffs, catalog, nActors);

  const foldedActors = initialActors,
    foldedTargets = structuredClone(first.targets ?? []);
  let foldedBuffs = structuredClone(first.buffs),
    foldedStamina = first.stamina ?? null;
  let lastNonFinalTimeUs = first.time_us,
    prevTime = 0,
    ordinal = 0,
    ends = [0, 0, 0] as [number, number, number];
  const lastIdx = frames.length - 1;

  for (let idx = 0; idx < frames.length; idx++) {
    const frame = frames[idx];
    if (frame.timers !== undefined && frame.timers !== null) {
      if (!Array.isArray(frame.timers)) throw new Error('Invalid native timer state.');
      let previous = -1;
      for (const timer of frame.timers) {
        const definition = report.timer_catalog![timer.timer_def_id];
        if (
          !Number.isSafeInteger(timer.timer_def_id) ||
          timer.timer_def_id <= previous ||
          !definition ||
          !Number.isSafeInteger(timer.expires_at_us) ||
          timer.expires_at_us <= clocks.elapsedAt(definition.clock_basis, frame.time_us)
        )
          throw new Error('Invalid native timer reference or remainder.');
        previous = timer.timer_def_id;
      }
    }
    finite(frame.time_us);
    if (
      frame.time_us < prevTime ||
      frame.time_us > report.absolute_elapsed_us ||
      frame.hits_end < ends[0] ||
      frame.hits_end > report.hit_receipts.length ||
      frame.healing_end < ends[1] ||
      frame.healing_end > report.healing_receipts.length ||
      frame.trace_end < ends[2] ||
      frame.trace_end > report.trace.length ||
      !Number.isSafeInteger(frame.active_actor_index) ||
      frame.active_actor_index < 0 ||
      frame.active_actor_index >= nActors
    )
      throw new Error('State timeline order, actor, or receipt prefix is invalid.');
    if (idx > 0 && idx < lastIdx) {
      if (frame.boundary.kind !== 'event')
        throw new Error('Intermediate state timeline frame must have event boundary.');
      if (
        !Number.isSafeInteger(frame.boundary.ordinal) ||
        frame.boundary.ordinal <= ordinal ||
        frame.boundary.ordinal > report.total_events_processed
      )
        throw new Error('State timeline event ordinal is invalid.');
      ordinal = frame.boundary.ordinal;
      if (frame.buffs !== undefined && frame.buffs !== null)
        throw new Error('Intermediate state timeline frame must use buff_diff instead of buffs.');
      const seenSlots = new Set<number>();
      for (const delta of frame.actors ?? []) {
        const slot = assertTimelineInt(delta.slot, 'actor delta slot');
        if (slot >= nActors || seenSlots.has(slot))
          throw new Error('Invalid or duplicate actor delta slot in timeline frame.');
        seenSlots.add(slot);
        if (!actorDeltaHasChanges(delta))
          throw new Error('Empty actor delta emitted in timeline frame.');
        foldedActors[slot] = applyActorDelta(foldedActors[slot], delta);
      }
      const seenTargets = new Set<number>();
      for (const target of frame.targets ?? []) {
        const tIdx = assertTimelineInt(target.target_index, 'target slot');
        if (tIdx >= nTargets || seenTargets.has(tIdx))
          throw new Error('Invalid or duplicate target delta slot in timeline frame.');
        seenTargets.add(tIdx);
        foldedTargets[tIdx] = structuredClone(target);
      }
      if (frame.buff_diff !== undefined && frame.buff_diff !== null) {
        const remove = frame.buff_diff.remove ?? [],
          upsert = frame.buff_diff.upsert ?? [];
        if (remove.length === 0 && upsert.length === 0)
          throw new Error('Empty buff_diff emitted in timeline frame.');
        const removeIds = new Set<number>();
        for (const id of remove) {
          if (removeIds.has(id)) throw new Error('Duplicate id in buff_diff.remove.');
          removeIds.add(id);
          if (!foldedBuffs.some((b) => b.instance_id === id))
            throw new Error('buff_diff.remove references inactive instance_id.');
        }
        const upsertIds = new Set<number>();
        for (const item of upsert) {
          const id = validateBuffRef(item, catalog, nActors);
          if (upsertIds.has(id)) throw new Error('Duplicate instance_id in buff_diff.upsert.');
          if (removeIds.has(id))
            throw new Error('buff_diff cannot both remove and upsert the same instance_id.');
          upsertIds.add(id);
          const existing = foldedBuffs.find((b) => b.instance_id === id);
          if (existing && sameObservation(existing, item))
            throw new Error('No-op buff_diff.upsert emitted.');
        }
        foldedBuffs = applyBuffDiff(foldedBuffs, frame.buff_diff);
      }
      if (frame.stamina !== undefined && frame.stamina !== null)
        foldedStamina = structuredClone(frame.stamina);
      lastNonFinalTimeUs = frame.time_us;
    }
    for (let h = ends[0]; h < frame.hits_end; h++)
      if (report.hit_receipts[h].time_us !== frame.time_us)
        throw new Error('State timeline differs from hit time.');
    for (let h = ends[1]; h < frame.healing_end; h++) {
      const heal = report.healing_receipts[h];
      if (
        heal.time_us !== frame.time_us ||
        heal.source_actor_index < 0 ||
        heal.source_actor_index >= nActors ||
        heal.recipient_actor_index < 0 ||
        heal.recipient_actor_index >= nActors ||
        heal.applied + heal.excess !== heal.gross ||
        heal.hp_before + heal.applied !== heal.hp_after
      )
        throw new Error('State timeline healing settlement does not reconcile.');
    }
    ends = [frame.hits_end, frame.healing_end, frame.trace_end];
    prevTime = frame.time_us;
  }

  const idleGap = last.time_us > lastNonFinalTimeUs;
  const hpMaxChanged = idleGap
    ? advanceBuffsTo(foldedBuffs, catalog, report.relative_elapsed_us)
    : new Set<number>();
  if (!sameObservation(foldedBuffs, report.final_buff_state.instances))
    throw new Error('Folded timeline buff_diff does not match final_buff_state.');
  if (!idleGap) {
    if (!sameObservation(foldedActors, report.final_actor_states))
      throw new Error('Folded timeline actor deltas do not match final_actor_states.');
    if (!sameObservation(foldedTargets, report.final_target_states))
      throw new Error('Folded timeline target states do not match final_target_states.');
    if (!sameObservation(foldedStamina, last.stamina ?? null))
      throw new Error('Folded timeline stamina does not match final frame.');
  } else {
    assertIdleGapReconciliation(
      foldedActors,
      foldedTargets,
      foldedBuffs,
      catalog,
      last.time_us,
      hpMaxChanged,
      report,
    );
  }
}
