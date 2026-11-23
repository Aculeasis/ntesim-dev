import names from '../generated/names.json';
import messages from '../generated/messages.json';
import type { ActorState, HitReceipt, TraceEvent } from './contracts/report';
import { formatTraceEvent } from './localization-events';

const EN: {
  characters: Record<string, string>;
  aliases: Record<string, string>;
  abilities: Record<string, string>;
  messages: Record<string, string>;
} = { ...names, messages };

const catalogs: Record<string, typeof EN> = { en: EN };
const lookup = (map: Record<string, string>, key: string) =>
  Object.hasOwn(map || {}, key) ? map[key] : undefined;

export class Localization {
  locale: string;
  catalog: typeof EN;
  constructor(locale = 'en') {
    this.locale = Object.hasOwn(catalogs, locale) ? locale : 'en';
    this.catalog = catalogs[this.locale];
  }

  hasMessage(key: string) {
    return (
      lookup(this.catalog.messages, key) !== undefined || lookup(EN.messages, key) !== undefined
    );
  }

  text(key: string, values: Record<string, string | number> = {}) {
    const template = lookup(this.catalog.messages, key) ?? lookup(EN.messages, key) ?? key;
    return template.replace(/\{(\w+)\}/g, (match, name) => String(values[name] ?? match));
  }

  character(id: unknown) {
    const raw = String(id ?? '');
    const normalized = raw.replace(/^character[-_]/, 'character:');
    const canonical =
      lookup(this.catalog.aliases, raw) ??
      lookup(EN.aliases, raw) ??
      lookup(this.catalog.aliases, normalized) ??
      lookup(EN.aliases, normalized) ??
      (/^\d+$/.test(raw) ? `character:${raw}` : normalized);
    return lookup(this.catalog.characters, canonical) ?? lookup(EN.characters, canonical) ?? raw;
  }

  skill(id: string) {
    if (id === 'ntesim.break') return this.text('break_damage');
    return lookup(this.catalog.abilities, id) ?? lookup(EN.abilities, id) ?? id;
  }

  actor(slot: string | number, states: Pick<ActorState, 'slot' | 'character_id'>[] = []) {
    const state = states.find((actor) => actor.slot === Number(slot));
    return state ? this.character(state.character_id) : this.text('actor', { slot });
  }

  event(
    ev: TraceEvent,
    states: Pick<ActorState, 'slot' | 'character_id'>[] = [],
    hit?: HitReceipt,
  ) {
    return formatTraceEvent(this, ev, states, hit);
  }
}

export const english = new Localization('en');

// Engine reports and imported names are data, never HTML.
export function escapeHtml(value: unknown) {
  return String(value ?? '').replace(
    /[&<>"']/g,
    (char) =>
      (
        ({
          '&': '&amp;',
          '<': '&lt;',
          '>': '&gt;',
          '"': '&quot;',
          "'": '&#39;',
        }) as Record<string, string>
      )[char],
  );
}
