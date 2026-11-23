import assert from 'node:assert/strict';
import { renderToStaticMarkup } from 'react-dom/server';
import { TEAM_CATALOG, filterTeams, emptyTeamFilters } from '../src/domain/team-list';
import { CATALOG } from '../src/domain/catalog';
import { TeamListRow } from '../src/features/team/TeamListRow';
import { RosterPicker } from '../src/features/team/RosterPicker';
import { SCRIPT_PRESETS } from '../src/domain/presets';

const entries = TEAM_CATALOG.entries;
assert.deepEqual(
  entries.map((e) => e.id),
  SCRIPT_PRESETS.map((s) => s.id),
);
const id = (name: string) => CATALOG.characters.find((c) => c.name === name)!.id;
const ids = (include: string[], exclude: string[] = [], query = '') =>
  filterTeams(entries, { include: include.map(id), exclude: exclude.map(id), query }).map(
    (e) => e.id,
  );
assert.deepEqual(ids([], [], 'mint SAK'), ['s5']);
assert.deepEqual(ids(['Nanally', 'Hotori'], ['Iroi']), ['nanally-blossom']);
assert.deepEqual(ids(['Mint', 'Sakiri'], ['Zero']), []);
assert.deepEqual(ids(['Mint'], ['Mint']), []);
assert.deepEqual(ids([], [], 'NoSuchCharacter'), []);
assert.ok(ids([], ['Mint']).includes('zankou-insight'));
assert.equal(filterTeams(entries, emptyTeamFilters).length, entries.length);
assert.ok(!entries.some((entry) => ['s1', 's2', 's3', 's4', 'baicang-domain'].includes(entry.id)));
const base = entries[0];
const unavailable = {
  ...base,
  id: 'unavailable',
  dps: { status: 'unavailable' as const, reason: '<bad>' },
};
const ranked = filterTeams(
  [
    unavailable,
    { ...base, id: 'z', dps: { status: 'ready', mean: 0 } },
    { ...base, id: 'b', dps: { status: 'ready', mean: 100.02 } },
    { ...base, id: 'a', dps: { status: 'ready', mean: 100.02 } },
    { ...base, id: 'c', dps: { status: 'ready', mean: 100.01 } },
  ],
  emptyTeamFilters,
);
assert.deepEqual(
  ranked.map((e) => e.id),
  ['a', 'b', 'c', 'z', 'unavailable'],
);
const partial = { ...base, actors: [entries.find((e) => e.id === 's5')!.actors[0]] };
const row = renderToStaticMarkup(<TeamListRow entry={partial} busy={false} onSelect={() => {}} />);
assert.equal((row.match(/class="compact-character/g) ?? []).length, 4);
assert.equal((row.match(/empty-slot/g) ?? []).length, 3);
assert.match(row, /Mint/);
assert.match(row, /Mind Royale/);
assert.match(row, /Fireflies and the Forest/);
assert.match(row, /T5/);
assert.doesNotMatch(row, /weapon-card|card-toggle|card-details/);
assert.doesNotMatch(row, /<details|<summary|Calculation details/);
assert.match(row, /compact-equipment" tabindex="0"/);
const s5 = renderToStaticMarkup(
  <TeamListRow entry={entries.find((e) => e.id === 's5')!} busy onSelect={() => {}} />,
);
assert.match(s5, /disabled=""/);
assert.match(s5, /A6/);
assert.doesNotMatch(s5, /empty-slot/);
const missing = renderToStaticMarkup(
  <TeamListRow entry={unavailable} busy={false} onSelect={() => {}} />,
);
assert.match(missing, /Unavailable/);
assert.match(missing, /&lt;bad&gt;/);
const picker = renderToStaticMarkup(
  <RosterPicker mode="include" selected={[id('Mint')]} onChange={() => {}} />,
);
assert.match(picker, /Clear All/);
assert.equal((picker.match(/aria-pressed="true"/g) ?? []).length, 1);
