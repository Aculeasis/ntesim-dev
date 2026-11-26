import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { Localization, english } from '../src/domain/localization';
import names from '../src/generated/names.json';
import catalog from '../src/generated/catalog.json';
assert.equal(Object.keys(names.characters).length, 25);
for (const [slug, id] of Object.entries(names.aliases)) {
  const native = id.split(':')[1],
    expected = catalog.characters.find((p) => p.slug === slug && p.id === id)?.name;
  assert.ok(expected);
  assert.equal(english.character(id), expected);
  assert.equal(english.character(slug), expected);
  assert.equal(english.character(native), expected);
  assert.equal(english.character(id.replace(':', '_')), expected);
}
const binary = fileURLToPath(
  new URL(
    `../../target/debug/ntesim${process.platform === 'win32' ? '.exe' : ''}`,
    import.meta.url,
  ),
);
const nativeCatalog = JSON.parse(
  execFileSync(binary, ['studio-build-catalog'], {
    encoding: 'utf8',
    maxBuffer: 32 * 1024 * 1024,
  }),
);
assert.deepEqual(names.abilities, nativeCatalog.ability_names);
assert.equal(new Localization('unsupported').locale, 'en');
assert.equal(english.character('unknown'), 'unknown');
assert.equal(english.skill('unknown'), 'unknown');
assert.equal(english.text('unknown'), 'unknown');
const teamStates = [
  { slot: 0, character_id: 'character:1075' },
  { slot: 1, character_id: 'character:1019' },
];
const burstHeal = english.event(
  {
    time_us: 100,
    phase: 'ProcCallback',
    event_kind: 'iroi_burst_heal',
    from_actor: 0,
    to_actor: 1,
    active_actor: 0,
    data: {
      input: 'burst',
      exact_raw: '10000/1',
      atk_ratio: '1413/1000',
      gross: 14130,
      applied: 9000,
      overflow: 5130,
      hp_before: 1000,
      hp_after: 10000,
    },
  },
  teamStates,
);
assert.equal(burstHeal.actor, english.character('character:1075'));
assert.equal(burstHeal.action, 'Iroi Burst healing');
assert.ok(burstHeal.detail.includes(english.character('character:1019')));
assert.ok(burstHeal.detail.includes('ATK 10000'));
assert.ok(burstHeal.detail.includes('ratio 1413/1000'));
assert.ok(burstHeal.detail.includes('gross 14130, applied 9000'));

const hitWithReceipt = english.event(
  {
    time_us: 200,
    phase: 'HitDelivery',
    event_kind: 'hit_damage',
    hit_index: 0,
    active_actor: 1,
    data: { stacks: 2 },
  },
  teamStates,
  {
    time_us: 200,
    actor_index: 0,
    target_index: 0,
    ability_id: 'GA_1075_Nor_01_Ability',
    damage_effect_id: 'GE_1075_Nor_01',
    metadata_id: 0,
    non_crit_damage: '1200/1',
    crit_damage: '2400/1',
    crit_rate: '1/2',
    critical: false,
    damage: '1200/1',
    rounded_damage: 1200,
    applied_hp_loss: 1200,
  },
);
assert.equal(hitWithReceipt.actor, english.character('character:1075'));
assert.equal(hitWithReceipt.action, 'Hit');
assert.ok(hitWithReceipt.detail.includes('target 1: non-crit 1200, crit 2400 with 2 stacks'));

const iroiBuff = english.event(
  {
    time_us: 300,
    phase: 'ProcCallback',
    event_kind: 'iroi_skill_buff',
    from_actor: 0,
    active_actor: 0,
    data: { atk_add: '400/1', def_ignore: '1/20', duration_us: 20_000_000 },
  },
  teamStates,
);
assert.equal(iroiBuff.actor, english.character('character:1075'));
assert.equal(iroiBuff.action, 'Iroi skill buff');
assert.ok(iroiBuff.detail.includes('+400'));
assert.ok(iroiBuff.detail.includes('+5%'));
assert.ok(iroiBuff.detail.includes('20s'));

const fanoutEvent = english.event(
  {
    time_us: 400,
    phase: 'HitDelivery',
    event_kind: 'mint_hit_energy',
    from_actor: 1,
    active_actor: 1,
    energy_fanout: {
      source: 'mint_hit',
      source_actor_index: 1,
      base: '3/2',
      recipients: [
        {
          recipient_actor_index: 0,
          distribution: '3/5',
          energy_before: 10,
          gross: 0,
          retained: 0,
          overflow: 0,
          energy_after: 10,
          charge_remainder_before: '0/1',
          charge_remainder_after: '9/10',
          charge_basis: '9/10',
        },
        {
          recipient_actor_index: 1,
          distribution: '1/1',
          energy_before: 20,
          gross: 1,
          retained: 1,
          overflow: 0,
          energy_after: 21,
          charge_remainder_before: '0/1',
          charge_remainder_after: '1/2',
          charge_basis: '3/2',
        },
      ],
    },
  },
  teamStates,
);
assert.equal(fanoutEvent.actor, english.character('character:1019'));
assert.equal(fanoutEvent.action, 'Mint ordinary hit');
assert.ok(fanoutEvent.detail.includes('base 3/2'));
assert.ok(fanoutEvent.detail.includes(english.character('character:1075')));
assert.ok(fanoutEvent.detail.includes(english.character('character:1019')));
console.log(
  'Localization: all 25 released identities, typed Rust action titles and fallback behavior match the native catalog',
);
