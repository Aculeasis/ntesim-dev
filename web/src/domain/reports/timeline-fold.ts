import type {
  ActiveBuffRef,
  ActorStateDelta,
  BuffCatalogEntry,
  BuffTimelineDiff,
} from '../contracts/timeline';
import type { ActorState, ScenarioReport } from '../contracts/report';
import { exactFraction } from './validation-support';

const DELTA_FIELDS = [
  'character_id',
  'attack_mode',
  'hp',
  'hp_max',
  'energy',
  'energy_capacity',
  'active_shields',
  'fireflies_stacks',
  'active_fireflies_stacks',
  'energy_conserved',
  'initial_energy',
  'granted_energy',
  'spent_energy',
  'overflow_energy',
  'charge_remainder',
  'cycle',
  'resources',
  'fractional_resources',
] as const;

const ATTACK_MODES = ['normal', 'aim', 'ranged', 'melee'] as const;

export function assertTimelineInt(value: unknown, label: string, min = 0): number {
  if (typeof value !== 'number' || !Number.isSafeInteger(value) || value < min)
    throw new Error(`Invalid ${label} in state timeline.`);
  return value;
}

export function actorDeltaHasChanges(delta: ActorStateDelta): boolean {
  return DELTA_FIELDS.some((key) => delta[key] !== undefined);
}

export function actorSummaryFromDelta(delta: ActorStateDelta): ActorState {
  if (
    typeof delta.character_id !== 'string' ||
    !delta.character_id ||
    typeof delta.attack_mode !== 'string' ||
    !(ATTACK_MODES as readonly string[]).includes(delta.attack_mode) ||
    typeof delta.energy_conserved !== 'boolean' ||
    !Array.isArray(delta.active_fireflies_stacks)
  )
    throw new Error('Missing required boundary actor fields in state timeline.');
  const summary: ActorState = {
    slot: assertTimelineInt(delta.slot, 'actor slot'),
    character_id: delta.character_id,
    attack_mode: delta.attack_mode,
    hp: assertTimelineInt(delta.hp, 'actor hp'),
    hp_max: assertTimelineInt(delta.hp_max, 'actor hp_max'),
    energy: assertTimelineInt(delta.energy, 'actor energy'),
    energy_capacity: assertTimelineInt(delta.energy_capacity, 'actor energy_capacity'),
    fireflies_stacks: assertTimelineInt(delta.fireflies_stacks, 'fireflies_stacks'),
    active_fireflies_stacks: structuredClone(delta.active_fireflies_stacks),
    energy_conserved: delta.energy_conserved,
    initial_energy: assertTimelineInt(delta.initial_energy, 'initial_energy'),
    granted_energy: assertTimelineInt(delta.granted_energy, 'granted_energy'),
    spent_energy: assertTimelineInt(delta.spent_energy, 'spent_energy'),
    overflow_energy: assertTimelineInt(delta.overflow_energy, 'overflow_energy'),
  };
  if (Array.isArray(delta.active_shields) && delta.active_shields.length > 0)
    summary.active_shields = structuredClone(delta.active_shields);
  if (delta.charge_remainder !== undefined && delta.charge_remainder !== null) {
    exactFraction(delta.charge_remainder);
    summary.charge_remainder = delta.charge_remainder;
  }
  if (delta.cycle !== undefined && delta.cycle !== null)
    summary.cycle = structuredClone(delta.cycle);
  if (delta.resources && Object.keys(delta.resources).length > 0)
    summary.resources = structuredClone(delta.resources);
  if (delta.fractional_resources && Object.keys(delta.fractional_resources).length > 0)
    summary.fractional_resources = structuredClone(delta.fractional_resources);
  return summary;
}

export function applyActorDelta(previous: ActorState, delta: ActorStateDelta): ActorState {
  if (!previous || previous.slot !== delta.slot)
    throw new Error('Actor delta slot does not match folded actor.');
  const next: ActorState = { ...previous };
  if (delta.character_id !== undefined) next.character_id = delta.character_id;
  if (delta.attack_mode !== undefined) next.attack_mode = delta.attack_mode;
  if (delta.hp !== undefined) next.hp = assertTimelineInt(delta.hp, 'actor hp');
  if (delta.hp_max !== undefined) next.hp_max = assertTimelineInt(delta.hp_max, 'actor hp_max');
  if (delta.energy !== undefined) next.energy = assertTimelineInt(delta.energy, 'actor energy');
  if (delta.energy_capacity !== undefined)
    next.energy_capacity = assertTimelineInt(delta.energy_capacity, 'actor energy_capacity');
  if (delta.fireflies_stacks !== undefined)
    next.fireflies_stacks = assertTimelineInt(delta.fireflies_stacks, 'fireflies_stacks');
  if (delta.active_fireflies_stacks !== undefined)
    next.active_fireflies_stacks = structuredClone(delta.active_fireflies_stacks);
  if (delta.energy_conserved !== undefined) next.energy_conserved = delta.energy_conserved;
  if (delta.initial_energy !== undefined)
    next.initial_energy = assertTimelineInt(delta.initial_energy, 'initial_energy');
  if (delta.granted_energy !== undefined)
    next.granted_energy = assertTimelineInt(delta.granted_energy, 'granted_energy');
  if (delta.spent_energy !== undefined)
    next.spent_energy = assertTimelineInt(delta.spent_energy, 'spent_energy');
  if (delta.overflow_energy !== undefined)
    next.overflow_energy = assertTimelineInt(delta.overflow_energy, 'overflow_energy');
  if (delta.active_shields !== undefined) {
    if (delta.active_shields.length === 0) delete next.active_shields;
    else next.active_shields = delta.active_shields.map((s) => ({ ...s }));
  }
  if (delta.resources !== undefined) {
    if (Object.keys(delta.resources).length === 0) delete next.resources;
    else next.resources = structuredClone(delta.resources);
  }
  if (delta.fractional_resources !== undefined) {
    if (Object.keys(delta.fractional_resources).length === 0) delete next.fractional_resources;
    else next.fractional_resources = structuredClone(delta.fractional_resources);
  }
  if (delta.charge_remainder !== undefined) {
    if (delta.charge_remainder === null) delete next.charge_remainder;
    else {
      exactFraction(delta.charge_remainder);
      next.charge_remainder = delta.charge_remainder;
    }
  }
  if (delta.cycle !== undefined) {
    if (delta.cycle === null) delete next.cycle;
    else next.cycle = { ...delta.cycle };
  }
  return next;
}

export function applyBuffDiff(current: ActiveBuffRef[], diff: BuffTimelineDiff): ActiveBuffRef[] {
  const removeIds = diff.remove?.length ? new Set(diff.remove) : null;
  const next = removeIds
    ? current.filter((item) => !removeIds.has(item.instance_id))
    : [...current];
  for (const item of diff.upsert ?? []) {
    const idx = next.findIndex((existing) => existing.instance_id === item.instance_id);
    const copy: ActiveBuffRef = { ...item, duration: { ...item.duration } };
    if (idx >= 0) next[idx] = copy;
    else next.push(copy);
  }
  return next;
}

export function resolveBuffRefs(refs: ActiveBuffRef[], catalog: BuffCatalogEntry[] = []) {
  return refs.map((ref) => {
    const def = catalog[ref.buff_def_id];
    if (!def) return ref;
    return {
      instance_id: ref.instance_id,
      id: def.id,
      source: def.source,
      clock_basis: def.clock_basis,
      owner_actor: def.owner_actor,
      recipient: def.recipient,
      duration: ref.duration,
      stacks: ref.stacks,
      max_stacks: def.max_stacks,
      decay_policy: def.decay_policy,
      refresh_policy: def.refresh_policy,
      stacking_scope: def.stacking_scope ?? 'per_source',
      source_lifetime: def.source_lifetime ?? 'snapshot',
      semantic_tags: def.semantic_tags ?? [],
      stat_modifiers: def.stat_modifiers,
    };
  });
}

export function validateBuffRef(
  ref: ActiveBuffRef,
  catalog: BuffCatalogEntry[],
  actorCount: number,
) {
  const instanceId = assertTimelineInt(ref.instance_id, 'buff instance_id', 1);
  const defId = assertTimelineInt(ref.buff_def_id, 'buff_def_id');
  const def = catalog[defId];
  if (!def) throw new Error('Buff reference points to unknown buff_catalog entry.');
  if (
    def.clock_basis?.kind !== 'world' ||
    !def.id ||
    !def.source ||
    !Number.isSafeInteger(def.owner_actor) ||
    def.owner_actor < 0 ||
    def.owner_actor >= actorCount ||
    !Number.isSafeInteger(def.max_stacks) ||
    def.max_stacks <= 0
  )
    throw new Error('Buff catalog definition violates timeline bounds.');
  const stacks = assertTimelineInt(ref.stacks, 'buff stacks', 1);
  if (stacks > def.max_stacks)
    throw new Error('Buff reference stacks exceed buff_catalog max_stacks.');
  if (ref.duration.kind === 'timed') {
    assertTimelineInt(ref.duration.duration_us, 'buff duration_us');
    assertTimelineInt(ref.duration.expires_at_us, 'buff expires_at_us');
  } else if (ref.duration.kind !== 'permanent') {
    throw new Error('Invalid buff duration kind.');
  }
  return instanceId;
}

export function validateUniqueBuffs(
  buffs: ActiveBuffRef[],
  catalog: BuffCatalogEntry[],
  nActors: number,
) {
  const seen = new Set<number>();
  for (const buff of buffs) {
    const id = validateBuffRef(buff, catalog, nActors);
    if (seen.has(id)) throw new Error('Duplicate buff instance_id in boundary snapshot.');
    seen.add(id);
  }
}

export function advanceBuffsTo(buffs: ActiveBuffRef[], catalog: BuffCatalogEntry[], nowUs: number) {
  const hpMaxChanged = new Set<number>();
  let i = 0;
  while (i < buffs.length) {
    const def = catalog[buffs[i].buff_def_id];
    const dur = buffs[i].duration;
    const isExpired = dur.kind === 'timed' && nowUs >= dur.expires_at_us;
    if (!isExpired) {
      i++;
      continue;
    }
    if (def.id === 'hp_suit.max_hp' && Object.hasOwn(def.stat_modifiers, 'max_hp_bonus'))
      hpMaxChanged.add(def.owner_actor);
    if (def.decay_policy === 'one_by_one' && buffs[i].stacks > 1 && dur.kind === 'timed') {
      buffs[i] = {
        ...buffs[i],
        stacks: buffs[i].stacks - 1,
        duration: {
          kind: 'timed',
          duration_us: dur.duration_us,
          expires_at_us: dur.expires_at_us + dur.duration_us,
        },
      };
    } else {
      buffs.splice(i, 1);
    }
  }
  return hpMaxChanged;
}

export function reconstructTargetStatuses(
  buffs: ActiveBuffRef[],
  catalog: BuffCatalogEntry[],
  targetIndex: number,
) {
  const statuses: NonNullable<ScenarioReport['final_target_states'][number]['statuses']> = [];
  for (const ref of buffs) {
    const def = catalog[ref.buff_def_id];
    if (def.recipient.type !== 'specific_target') continue;
    if (String(def.recipient.target) !== String(targetIndex)) continue;
    if (ref.duration.kind !== 'timed')
      throw new Error('Target status cannot have permanent duration.');
    const resistance_reductions: Record<string, string> = {};
    for (const [key, value] of Object.entries(def.stat_modifiers)) {
      if (key.startsWith('resistance_reduction:'))
        resistance_reductions[key.slice('resistance_reduction:'.length)] = value;
    }
    statuses.push({
      clock_basis: def.clock_basis,
      source_actor_index: def.owner_actor,
      id: def.id,
      expires_at_us: ref.duration.expires_at_us,
      ...(def.semantic_tags?.length ? { semantic_tags: def.semantic_tags } : {}),
      stacks: ref.stacks,
      resistance_reductions,
    });
  }
  return statuses;
}
