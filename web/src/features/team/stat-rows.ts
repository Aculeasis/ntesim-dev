import type { ActorInspection } from '../../domain/contracts/script';
import type { Ratio } from '../../domain/contracts/scenario';
import { fraction } from '../../domain/reports/statistics';
import { format } from '../../components/ui/format';

export type StatRow = [icon: string, label: string, value: string, exact: string];
const elementLabels: Record<string, string> = {
  NATURE: 'Anima',
  CHAOS: 'Chaos',
  COSMOS: 'Cosmos',
  INCANTATION: 'Incantation',
  LAKSHANA: 'Lakshana',
  PSYCHE: 'Psyche',
};

/** Format engine fields only; derived values and contribution sums stay in Rust. */
export function statRows(actor: ActorInspection): StatRow[] {
  const s = actor.static_stats;
  const rows: StatRow[] = [];
  const integer = (icon: string, label: string, value: number) => {
    if (value !== 0) rows.push([icon, label, format(value), String(value)]);
  };
  const ratio = (
    icon: string,
    label: string,
    value: Ratio | undefined,
    percent = false,
    neutral = 0,
  ) => {
    if (!value) return;
    const match = /^(-?(?:0|[1-9]\d*))\/([1-9]\d*)$/.exec(value);
    if (!match || BigInt(match[1]) === BigInt(neutral) * BigInt(match[2])) return;
    rows.push([
      icon,
      label,
      percent ? format(fraction(value) * 100, 2) + '%' : format(fraction(value), 3),
      value,
    ]);
  };
  integer('hp', 'HP', s.hp);
  ratio('atk', 'ATK', s.atk);
  integer('def', 'DEF', s.def);
  integer('atk', 'Base ATK', s.base_atk);
  if (s.base_hp !== undefined) integer('hp', 'Base HP', s.base_hp);
  if (s.base_def !== undefined) integer('def', 'Base DEF', s.base_def);
  ratio('cr', 'CRIT Rate', s.crit_rate, true);
  ratio('cd', 'CRIT DMG', s.crit_damage_bonus, true);
  ratio('dmg', 'General DMG Bonus', s.damage_bonus, true);
  // Native script inspection projects bonuses under their specific element names.
  for (const [id, value] of Object.entries(s.elemental_bonuses ?? {}))
    ratio('dmg', `${elementLabels[id]} DMG Bonus`, value, true);
  ratio('heal', 'Outgoing Healing Bonus', s.healing_bonus, true);
  ratio('shield', 'Outgoing Shield Bonus', s.shield_bonus, true);
  ratio('dmg', 'Mental DMG Bonus', s.mental_damage_bonus, true);
  ratio('dmg', 'Cycle Intensity', s.cycle_intensity);
  ratio('dmg', 'Break Intensity', s.break_intensity);
  ratio('dmg', 'Break Efficiency Multiplier', s.break_efficiency_multiplier, false, 1);
  ratio('energy', 'Charge Efficiency Multiplier', actor.charge_efficiency, false, 1);
  if (actor.initial_hp !== s.hp) integer('hp', 'Starting HP', actor.initial_hp);
  if (actor.initial_energy !== 0)
    rows.push([
      'energy',
      'Starting Energy',
      `${actor.initial_energy} / ${actor.energy_capacity}`,
      `${actor.initial_energy} / ${actor.energy_capacity}`,
    ]);
  integer('dmg', 'Starting Cycle', actor.initial_cycle);
  for (const [id, value] of Object.entries(actor.initial_resources))
    ratio('energy', `Initial ${id}`, value);
  return rows;
}
