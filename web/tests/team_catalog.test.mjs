import assert from 'node:assert/strict';
import { readFileSync, mkdirSync, rmSync } from 'node:fs';
import path from 'node:path';
import { CATALOG } from '../src/domain/catalog';
import { engine } from './native-compiler.ts';
import { calculateTeam, cachedCalculation } from '../../tools/studio_teams.mts';
import { readStudioPresets, validateTeamCatalog } from '../../tools/studio_presets.mjs';
const catalog = JSON.parse(
  readFileSync(new URL('../src/generated/team-list.json', import.meta.url)),
);
const presets = readStudioPresets();
validateTeamCatalog(catalog, presets);
const missing = structuredClone(catalog);
missing.entries.pop();
assert.throws(() => validateTeamCatalog(missing, presets));
const edited = structuredClone(presets);
edited.scripts.find((s) => s.selectable).source += '\n# local edit';
validateTeamCatalog(catalog, edited);
const cache = path.resolve('../cache/test-team-cache-' + process.pid);
assert.ok(cache.startsWith(path.resolve('../cache') + path.sep));
mkdirSync(cache, { recursive: true });
try {
  let calls = 0;
  const script = presets.scripts.find((s) => s.selectable);
  const calculate = () => ({
    entry: { ...catalog.entries[0], dps: { status: 'ready', mean: ++calls } },
    raw: null,
  });
  const cached = (identity = catalog, fresh = false) =>
    cachedCalculation(script, identity, cache, fresh, calculate);
  assert.equal(cached().entry.dps.mean, 1);
  assert.equal(cached().entry.dps.mean, 1);
  for (const key of ['engine_source_sha256', 'wasm_sha256', 'generator_sha256']) {
    const before = calls;
    assert.equal(
      cached({ ...catalog, [key]: catalog[key] + '-changed' }).entry.dps.mean,
      before + 1,
    );
  }
  const before = calls;
  cachedCalculation(
    { ...script, source: script.source + '\n# edited' },
    catalog,
    cache,
    false,
    calculate,
  );
  assert.equal(calls, before + 1);
  cached(catalog, true);
  assert.equal(calls, before + 2);
} finally {
  rmSync(cache, { recursive: true, force: true });
}
assert.equal(catalog.clock_contract, 'relative-combat-v1');
assert.equal(catalog.timer_policy, 'paused-world-v1');
assert.equal(catalog.relative_duration_us, 120_000_000);
assert.equal(presets.default, 's5');
const control = presets.scripts.find((script) => script.id === presets.default);
assert.ok(
  engine.inspectScript(control.source).party.actors.every((a) => a.purchased_qte_level === 8),
);
const curated = presets.scripts.filter((s) => s.selectable && s.id !== 's5');
assert.equal(curated.length, 22);
assert.ok(catalog.entries.every((entry) => entry.dps.status === 'ready'));
const rosters = new Set();
const standard = new Set(['baicang', 'daffodil', 'jiuyuan', 'sakiri', 'fadia', 'hathor']);
for (const script of curated) {
  const entry = catalog.entries.find((e) => e.id === script.id);
  assert.equal(entry.options.trials, 1000);
  assert.equal(entry.options.relative_duration_us, 120_000_000);
  assert.equal(entry.options.seed, 42);
  assert.equal(entry.options.crit, 'random');
  assert.equal(entry.options.start_fons, 250000);
  assert.deepEqual(entry.targets, [
    {
      airborne_until_us: 0,
      break_profile: {
        broken_duration_us: 10_000_000,
        capacity: '30/1',
        current_gauge: '30/1',
        discord_fraction: '1/10',
      },
      control_immune: false,
      defense: 1080,
      hp_current: 1_000_000_000,
      hp_max: 1_000_000_000,
      is_boss: true,
      level: 80,
      original_atk: 1000,
      resistance: '1/5',
      target_index: 0,
    },
  ]);
  const { party, diagnostics } = engine.inspectScript(script.source);
  assert.deepEqual(diagnostics, []);
  assert.equal(party.actors.length, 4);
  assert.ok(
    !rosters.has(
      party.actors
        .map((a) => a.character_id)
        .sort()
        .join(','),
    ),
  );
  rosters.add(
    party.actors
      .map((a) => a.character_id)
      .sort()
      .join(','),
  );
  assert.equal(party.actors.filter((a) => a.set?.id === 'set:Suit11').length, 1);
  const lines = [
    ...script.source.matchAll(/^# C: (.*); III x4: (.*); IV: (.*); II: (.*) \/ (.*)\.$/gm),
  ];
  assert.equal(lines.length, 4);
  for (const line of lines) {
    assert.equal(
      line[1].split(',').length + 4 * line[2].split(',').length + line[3].split(',').length + 2,
      15,
    );
  }
  for (const a of party.actors) {
    const character = CATALOG.characters.find((c) => c.id === a.character_id);
    assert.ok(character);
    assert.equal(a.level, 80);
    assert.deepEqual(a.talent_levels, [8, 8, 8]);
    assert.equal(a.purchased_qte_level, 8);
    assert.equal(a.weapon.level, 80);
    assert.equal(a.weapon.refinement, 1);
    const count =
      character.rarity === 'A' || character.slug === 'chiz' || character.slug === 'zero'
        ? 6
        : standard.has(character.slug)
          ? 1
          : 0;
    const special =
      ['zankou-insight', 'zankou-blackbird'].includes(script.id) && character.slug === 'sakiri'
        ? [4]
        : ['zankou-insight', 'zankou-blackbird'].includes(script.id) &&
            character.slug === 'daffodil'
          ? [3, 4, 5]
          : script.id === 'zankou-break' && character.slug === 'daffodil'
            ? [3, 5]
            : script.id === 'zankou-break' && character.slug === 'zankou'
              ? [5]
              : null;
    if (special) assert.deepEqual(a.awakening, special);
    else assert.equal(a.awakening.length, count);
  }
}
const names = (id) =>
  catalog.entries
    .find((e) => e.id === id)
    .actors.map((a) => CATALOG.characters.find((c) => c.id === a.character_id).slug)
    .sort();
assert.deepEqual(names('nanally-blossom'), ['hotori', 'jiuyuan', 'nanally', 'zero']);
assert.deepEqual(names('zankou-insight'), ['daffodil', 'iroi', 'sakiri', 'zankou']);
assert.deepEqual(names('shinku-elyms'), ['akane', 'elyms', 'iroi', 'shinku']);
const source =
  'options duration=120 iteration=2 seed=42 crit=random; mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1; target hp=100000000 def=0 resist=0; active mint; wait(0.1);';
const zero = calculateTeam(engine, { id: 'zero', label: 'Zero activity', source });
assert.deepEqual(zero.entry.dps, { status: 'ready', mean: 0 });
assert.deepEqual(
  calculateTeam(engine, { id: 'zero', label: 'Zero activity', source }),
  zero,
  'Cache-free generation is deterministic',
);
const failed = calculateTeam(engine, {
  id: 'failed',
  label: 'Failed',
  source: source.replace('wait(0.1);', 'mint skill, skill;'),
});
assert.equal(failed.entry.dps.status, 'unavailable');
assert.ok(failed.entry.dps.reason.length > 0);
console.log(
  'Team catalog: current identity, invalidation, fresh Rust session parity, zero, failure and deterministic projection passed',
);
