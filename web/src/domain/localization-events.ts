import type { ExactFraction } from './contracts/ratio';
import type { ActorState, HitReceipt, TraceEvent, TraceEventPayload } from './contracts/report';
import { energySourceDetails } from './reports/energy-sources';
import { fraction } from './reports/statistics';

export interface EventLocalizationContext {
  hasMessage(key: string): boolean;
  text(key: string, values?: Record<string, string | number>): string;
  character(id: unknown): string;
  skill(id: string): string;
  actor(slot: string | number, states?: Pick<ActorState, 'slot' | 'character_id'>[]): string;
}

type ActorStates = Pick<ActorState, 'slot' | 'character_id'>[];

function formatScalar(value: ExactFraction | undefined) {
  if (!value) return '0';
  const slash = value.indexOf('/');
  if (slash > 0 && value.slice(slash + 1) === '1') return value.slice(0, slash);
  return value;
}

function formatNumber(value: number, maxDecimals = 3) {
  if (!Number.isFinite(value)) return String(value);
  const rounded = value.toFixed(maxDecimals);
  return rounded.includes('.') ? rounded.replace(/\.?0+$/, '') : rounded;
}

function formatPercent(value: ExactFraction | undefined) {
  if (!value) return '0';
  return formatNumber(fraction(value) * 100, 3);
}

function formatSeconds(us: number) {
  return formatNumber(us / 1e6, 3);
}

function humanizeKind(kind: string) {
  if (!kind) return 'Action';
  const spaced = kind.replaceAll('_', ' ');
  return spaced[0].toUpperCase() + spaced.slice(1);
}

function resolveOwnerSlot(ev: TraceEvent, hit?: HitReceipt): number | undefined {
  if (ev.event_kind === 'switch_actor') return ev.active_actor;
  return (
    ev.energy_change?.recipient_actor_index ??
    hit?.actor_index ??
    ev.from_actor ??
    ev.to_actor ??
    ev.energy_fanout?.source_actor_index
  );
}

function resolveActorLabel(
  ctx: EventLocalizationContext,
  ev: TraceEvent,
  states: ActorStates,
  hit?: HitReceipt,
) {
  const owner = resolveOwnerSlot(ev, hit);
  if (owner !== undefined) return ctx.actor(owner, states);
  if (ev.event_kind === 'sakiri_atk_buff') return ctx.character('sakiri');
  if (
    ev.event_kind === 'iroi_skill_buff' ||
    ev.event_kind === 'iroi_a5_atk_buff' ||
    ev.event_kind === 'iroi_skill_heal' ||
    ev.event_kind === 'iroi_burst_heal'
  )
    return ctx.character('iroi');
  return ctx.text('global');
}

function resolveActionLabel(ctx: EventLocalizationContext, ev: TraceEvent) {
  if (ev.event_kind && ctx.hasMessage(ev.event_kind)) return ctx.text(ev.event_kind);
  if (ev.energy_fanout) {
    return energySourceDetails(ev.energy_fanout.source, ev.event_kind).label;
  }
  if (ev.energy_change && ev.energy_change.source !== 'ability_cost') {
    return energySourceDetails(ev.energy_change.source, ev.event_kind).label;
  }
  return humanizeKind(ev.event_kind);
}

function pushPayloadSummary(
  ctx: EventLocalizationContext,
  data: TraceEventPayload,
  states: ActorStates,
  parts: string[],
) {
  if (data.ability_id) parts.push(ctx.skill(data.ability_id));
  if (data.damage_effect_id) parts.push(data.damage_effect_id);
  if (data.utility_id) parts.push(data.utility_id);
  if (data.buff_id) parts.push(data.buff_id);
  if (data.shield_id) parts.push(data.shield_id);
  if (data.reaction) parts.push(data.reaction);
  if (data.element) parts.push(data.element);
  if (data.input) parts.push(data.input);
  if (data.reason) parts.push(data.reason);
  if (data.cause) parts.push(data.cause);
  if (data.condition) parts.push(`${data.condition}${data.outcome ? `=${data.outcome}` : ''}`);
  else if (data.outcome) parts.push(data.outcome);
  if (data.mark) parts.push(`mark ${data.mark}`);
  if (data.action) parts.push(data.action);
  if (data.dummy_event) {
    parts.push(`${data.dummy_event.kind} on target ${data.dummy_event.target_index + 1}`);
  } else if (data.target_index !== undefined) {
    parts.push(`target ${data.target_index + 1}`);
  }
  if (data.to_target_index !== undefined) parts.push(`to target ${data.to_target_index + 1}`);
  if (data.next_target_index !== undefined) parts.push(`next target ${data.next_target_index + 1}`);
  if (data.stacks !== undefined) {
    parts.push(
      data.max_stacks !== undefined
        ? `stacks ${data.stacks}/${data.max_stacks}`
        : `stacks ${data.stacks}`,
    );
  }
  if (data.level !== undefined) parts.push(`level ${data.level}`);
  if (data.count !== undefined) parts.push(`count ${data.count}`);
  if (data.consumed !== undefined) parts.push(`consumed ${data.consumed}`);
  if (data.remaining !== undefined) parts.push(`remaining ${data.remaining}`);
  if (data.duration_us !== undefined) parts.push(`${formatSeconds(data.duration_us)}s`);
  if (data.delay_us !== undefined) parts.push(`delay ${formatSeconds(data.delay_us)}s`);
  if (data.gross !== undefined || data.applied !== undefined) {
    const gross = data.gross ?? 0;
    const applied = data.applied ?? 0;
    const overflow = data.overflow !== undefined ? `, overflow ${data.overflow}` : '';
    parts.push(`gross ${gross}, applied ${applied}${overflow}`);
  } else if (data.amount !== undefined) parts.push(`amount ${data.amount}`);
  if (data.delivered_damage !== undefined) parts.push(`damage ${data.delivered_damage}`);
  if (data.absorbed_by_shields !== undefined && Number(data.absorbed_by_shields) > 0) {
    parts.push(`shield absorbed ${data.absorbed_by_shields}`);
  }
  if (data.hp_before !== undefined && data.hp_after !== undefined) {
    parts.push(`HP ${data.hp_before}->${data.hp_after}`);
  }
  if (data.atk_add) parts.push(`ATK +${formatScalar(data.atk_add)}`);
  if (data.atk_percent) parts.push(`ATK +${formatPercent(data.atk_percent)}%`);
  if (data.def_percent) parts.push(`DEF ${formatPercent(data.def_percent)}%`);
  if (data.def_ignore) parts.push(`DEF ignore +${formatPercent(data.def_ignore)}%`);
  if (data.crit_rate_bonus) parts.push(`CRIT +${formatPercent(data.crit_rate_bonus)}%`);
  if (data.crit_damage_bonus) parts.push(`CRIT DMG +${formatPercent(data.crit_damage_bonus)}%`);
  if (data.damage_bonus) parts.push(`DMG +${formatPercent(data.damage_bonus)}%`);
  if (data.mental_damage_bonus)
    parts.push(`Mental DMG +${formatPercent(data.mental_damage_bonus)}%`);
  if (data.exact_amount) parts.push(`exact ${formatScalar(data.exact_amount)}`);
  if (data.exact_delta) parts.push(`delta ${formatScalar(data.exact_delta)}`);
  if (data.recipients?.length) {
    parts.push(`to ${data.recipients.map((slot) => ctx.actor(slot, states)).join(', ')}`);
  }
  if (data.resource_deposits) {
    for (const [key, deposit] of Object.entries(data.resource_deposits)) {
      parts.push(`${key} +${formatScalar(deposit.credited)}`);
    }
  }
}

function formatFallbackDetail(
  ctx: EventLocalizationContext,
  ev: TraceEvent,
  states: ActorStates,
  hit?: HitReceipt,
  actionLabel?: string,
) {
  const parts: string[] = [];
  if (ev.from_actor !== undefined && ev.to_actor !== undefined && ev.from_actor !== ev.to_actor) {
    parts.push(`${ctx.actor(ev.from_actor, states)} → ${ctx.actor(ev.to_actor, states)}`);
  } else if (ev.to_actor !== undefined) {
    parts.push(ctx.actor(ev.to_actor, states));
  }
  if (hit) {
    parts.push(
      `${ctx.skill(hit.ability_id)} on target ${hit.target_index + 1} (${hit.rounded_damage})`,
    );
  }
  if (ev.data) pushPayloadSummary(ctx, ev.data, states, parts);
  if (parts.length > 0) return parts.join(' · ');
  return actionLabel || humanizeKind(ev.event_kind);
}

export function formatTraceEvent(
  ctx: EventLocalizationContext,
  ev: TraceEvent,
  states: ActorStates = [],
  hit?: HitReceipt,
) {
  const actor = resolveActorLabel(ctx, ev, states, hit);
  const action = resolveActionLabel(ctx, ev);
  const data = ev.data;
  const energy = ev.energy_change;
  const cast = energy?.source === 'ability_cost' ? energy : undefined;
  let detail: string;

  if (hit?.break_contribution) {
    detail = ctx.text('breakContributionDetail', { target: hit.target_index + 1 });
  } else if (cast) {
    detail = ctx.text('castDetail', {
      skill: ctx.skill(cast.ability_id),
      actor: ctx.actor(cast.recipient_actor_index, states),
      energy: cast.spent,
    });
  } else if (ev.event_kind === 'hit_damage' && hit) {
    detail = ctx.text('hitDetail', {
      skill: ctx.skill(hit.ability_id),
      target: hit.target_index + 1,
      nonCrit: formatScalar(hit.non_crit_damage),
      crit: formatScalar(hit.crit_damage),
      stacks: data?.stacks ?? 0,
    });
  } else if (ev.event_kind === 'switch_actor') {
    detail = ctx.text('switchDetail', {
      from: ev.from_actor !== undefined ? ctx.actor(ev.from_actor, states) : ctx.text('global'),
      to: ctx.actor(ev.to_actor ?? ev.active_actor, states),
    });
  } else if (ev.event_kind === 'switch_actor_noop') {
    detail = ctx.text('switchNoopDetail', {
      actor: ctx.actor(ev.from_actor ?? ev.active_actor, states),
      reason: data?.reason ?? 'already_active',
    });
  } else if (
    ev.event_kind === 'sakiri_atk_buff' &&
    data?.atk_add &&
    data.duration_us !== undefined
  ) {
    detail = ctx.text('sakiriBuffDetail', {
      actor:
        ev.from_actor !== undefined ? ctx.actor(ev.from_actor, states) : ctx.character('sakiri'),
      atk: formatScalar(data.atk_add),
      duration: formatSeconds(data.duration_us),
    });
  } else if (
    ev.event_kind === 'iroi_skill_buff' &&
    data?.atk_add &&
    data.def_ignore &&
    data.duration_us !== undefined
  ) {
    detail = ctx.text('iroiBuffDetail', {
      actor: ev.from_actor !== undefined ? ctx.actor(ev.from_actor, states) : ctx.character('iroi'),
      atk: formatScalar(data.atk_add),
      ignore: formatPercent(data.def_ignore),
      duration: formatSeconds(data.duration_us),
    });
  } else if (
    ev.event_kind === 'iroi_a5_atk_buff' &&
    data?.atk_add &&
    data.duration_us !== undefined
  ) {
    detail = ctx.text('iroiA5BuffDetail', {
      actor: ev.from_actor !== undefined ? ctx.actor(ev.from_actor, states) : ctx.character('iroi'),
      atk: formatScalar(data.atk_add),
      duration: formatSeconds(data.duration_us),
    });
  } else if ((ev.event_kind === 'iroi_skill_heal' || ev.event_kind === 'iroi_burst_heal') && data) {
    const recipientSlot = ev.to_actor ?? ev.active_actor;
    const mode = data.input === 'burst' || ev.event_kind === 'iroi_burst_heal' ? 'Burst' : 'Skill';
    detail = ctx.text('iroiHealDetail', {
      actor: ev.from_actor !== undefined ? ctx.actor(ev.from_actor, states) : ctx.character('iroi'),
      action: mode,
      recipient: ctx.actor(recipientSlot, states),
      atk: formatScalar(data.exact_raw),
      ratio: data.atk_ratio ?? '0/1',
      gross: data.gross ?? 0,
      applied: data.applied ?? 0,
      overflow: data.overflow ?? 0,
      before: data.hp_before ?? 0,
      after: data.hp_after ?? 0,
    });
  } else if (
    (ev.event_kind === 'fireflies_crit_stack' || ev.event_kind === 'fireflies_stack_capped') &&
    data
  ) {
    detail = ctx.text('firefliesDetail', {
      actor: ev.to_actor !== undefined ? ctx.actor(ev.to_actor, states) : actor,
      admitted: String(data.admitted ?? ev.event_kind === 'fireflies_crit_stack'),
      stacks: data.stacks ?? 0,
    });
  } else if (ev.event_kind === 'fireflies_stack_expired' && data?.stacks !== undefined) {
    detail = ctx.text('firefliesExpiredDetail', { stacks: data.stacks });
  } else if (ev.event_kind === 'actor_shield_expired' && data?.count !== undefined) {
    detail = ctx.text('shieldExpiredDetail', { count: data.count });
  } else if (ev.event_kind === 'max_hp_stack_expired' && data) {
    detail = ctx.text('maxHpExpiredDetail', {
      actor: ev.to_actor !== undefined ? ctx.actor(ev.to_actor, states) : actor,
      before: data.hp_before ?? 0,
      after: data.hp_after ?? 0,
      hpMax: data.hp_max ?? 0,
    });
  } else if (ev.energy_fanout) {
    const fanout = ev.energy_fanout;
    const recipients = fanout.recipients
      .map((r) =>
        ctx.text('energyFanoutRecipient', {
          actor: ctx.actor(r.recipient_actor_index, states),
          amount: r.gross,
          distribution: r.distribution ?? '1/1',
          energy: r.energy_after,
          overflow: r.overflow,
          carryBefore: r.charge_remainder_before ?? '0/1',
          carryAfter: r.charge_remainder_after ?? '0/1',
        }),
      )
      .join('; ');
    detail = ctx.text('energyFanoutDetail', {
      source: energySourceDetails(fanout.source, ev.event_kind).label,
      base: fanout.base ?? 'flat',
      actor: ctx.actor(fanout.source_actor_index, states),
      recipients,
    });
  } else if (energy && energy.source !== 'ability_cost') {
    detail = ctx.text('energyDetail', {
      source: energySourceDetails(energy.source, ev.event_kind).label,
      amount: energy.gross,
      actor: ctx.actor(energy.recipient_actor_index, states),
      energy: energy.energy_after,
      overflow: energy.overflow,
      basis: energy.charge_basis ?? energy.hit_basis?.base ?? 'flat',
      carryBefore: energy.charge_remainder_before ?? '0',
      carryAfter: energy.charge_remainder_after ?? '0',
    });
  } else if (energy?.source === 'ability_cost') {
    detail = ctx.text('energySpendDetail', {
      ability: ctx.skill(energy.ability_id),
      spent: energy.spent,
      energy: energy.energy_after,
      carry: energy.charge_remainder_after ?? '0',
    });
  } else {
    detail = formatFallbackDetail(ctx, ev, states, hit, action);
  }

  return { actor, detail, action };
}
