import assert from 'node:assert/strict';
import './team-list-cases';
import { readFileSync, readdirSync } from 'node:fs';
import { renderToStaticMarkup } from 'react-dom/server';
import './build-identity-cases';
import { TeamCards } from '../src/features/team/TeamCards';
import { MetricCard } from '../src/features/results/MetricCard';
import { BarChart } from '../src/components/charts/BarChart';
import { barScale } from '../src/components/charts/models';
import { Histogram } from '../src/components/charts/Histogram';
import { TimeChart } from '../src/components/charts/TimeChart';
import { PieChart } from '../src/components/charts/PieChart';
import { EnergyPanels } from '../src/features/results/EnergyPanels';
import { DamagePanels } from '../src/features/results/DamagePanels';
import type { TrialResult } from '../src/domain/contracts/trials';
import { Artwork } from '../src/components/ui/Artwork';
import { ExactReport } from '../src/features/results/ExactReport';
import { StatePanel } from '../src/features/playback/StatePanel';
import { SourceView } from '../src/features/playback/SourceView';
import { createReplay } from '../src/domain/replay/model';
import type { ReportBundle } from '../src/domain/contracts/source-report';
import type { Playback } from '../src/features/playback/usePlayback';
import { sameObservation } from '../src/domain/reports/identity';
import { highlightTokens } from '../src/features/editor/highlight';
import { textareaRange } from '../src/features/editor/CodeEditor';
import { CATALOG } from '../src/domain/catalog';
import { ARTWORK } from '../src/domain/artwork';
import { compileNte, getPresetSource, engine } from './native-compiler';
import {
  loadConfig,
  saveConfig,
  CONFIG_KEY,
  normalizeSource,
} from '../src/infrastructure/storage/config';
import { checkStudioReportRoundtrip } from './report-import-cases.mjs';
const compiled = engine.compileSource(getPresetSource('s5')).compiled!;
const inspection = engine.inspectScript(getPresetSource('s5')).party!;
const reportSource =
  'options duration=3 iteration=1; mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1; target hp=100000000 def=0 resist=0; active mint; mint attack; wait(1);';
const reportCompiled = engine.compileSource(reportSource).compiled!;
const execution = engine.runSourceSample({ compiled: reportCompiled, seed: 42 });
const bundle = (
  engine.invoke('export_source_report', {
    compiled: reportCompiled,
    seed: execution.report.seed,
  }) as { data: ReportBundle }
).data;
assert.ok(sameObservation(bundle.report.execution, { ...execution, report: execution.report }));
assert.ok(sameObservation({ a: '9007199254740993', b: 1 }, { b: 1, a: '9007199254740993' }));
assert.ok(!sameObservation({ a: '9007199254740993' }, { a: '9007199254740992' }));
const exactReport = renderToStaticMarkup(<ExactReport bundle={bundle} onExport={() => {}} />);
for (const text of ['Exact JSON', 'Exact CSV', 'overkill', bundle.report.input_sha256, 'On'])
  assert.ok(exactReport.includes(text));
const model = createReplay(execution.request, execution.report);
const playback: Playback = {
  model,
  state: model.stateAt(model.horizon),
  playing: false,
  speed: 1,
  setSpeed: () => {},
  seek: () => {},
  step: () => {},
  toggle: () => {},
};
assert.match(renderToStaticMarkup(<StatePanel playback={playback} />), /Recorded actor states/);
assert.match(
  renderToStaticMarkup(
    <SourceView execution={execution} event={model.events.find((e) => !!e.admitted)!} />,
  ),
  /Select source range/,
);
const independent = structuredClone(inspection);
independent.actors[0].talent_levels = [1, 2, 3];
assert.match(renderToStaticMarkup(<TeamCards party={independent} />), /Skills 1 \/ 2 \/ 3/);
const cards = renderToStaticMarkup(<TeamCards party={inspection} initialExpanded />);
assert.equal((cards.match(/class="character-card"/g) || []).length, 4);
for (const name of ['Mint', 'Sakiri', 'Zero', 'Iroi', 'Mind Royale', 'Fireflies and the Forest'])
  assert.ok(cards.includes(name));
const hostile = structuredClone(inspection);
hostile.actors[0].weapon!.id = '<img src=x onerror=x>';
assert.doesNotMatch(renderToStaticMarkup(<TeamCards party={hostile} />), /<img src=x/);
// UI rendering consumes supplied engine values, without adding gear or inferring resources.
independent.actors[0].static_stats.atk = '98765/7';
independent.actors[0].initial_resources = {
  grains: '0/1',
  loans: '7/3',
};
const exactCards = renderToStaticMarkup(<TeamCards party={independent} initialExpanded />);
assert.match(exactCards, /Exact: 98765\/7/);
assert.doesNotMatch(exactCards, /Initial grains/);
assert.match(exactCards, /Initial loans<\/td><td>2.333/);
assert.match(exactCards, /data-active="true"/);
const invalidRotation = getPresetSource('s5').replace(/# Rotation[\s\S]*/, 'wait(-1);');
const live = engine.compileSource(invalidRotation);
assert.equal(live.compiled, null);
assert.deepEqual(live.party, inspection);
assert.equal(live.options!.trials, 1000);
// Imported CRLF text keeps its authored bytes while textarea values use LF.
const crlf = '# 🦀\r\n' + getPresetSource('s5').replace(/\n/g, '\r\n');
const imported = engine.compileSource(crlf);
const textarea = crlf.replace(/\r\n?/g, '\n');
assert.equal(textarea.slice(...textareaRange(crlf, imported.party!.actors[0].loc)), 'mint');
const broken = crlf + '\r\nwait(1 2);';
const located = engine.compileSource(broken).diagnostics[0];
assert.equal(broken.replace(/\r\n?/g, '\n').slice(...textareaRange(broken, located.loc)), '2');
const source = 'mint add weapon="<script>"; # <img onerror=x>';
assert.equal(
  highlightTokens(source)
    .map((t) => t.text)
    .join(''),
  source + '\n',
);
assert.match(
  renderToStaticMarkup(<Artwork category="weapons" alt="Unknown" />),
  /asset-placeholder/,
);
const stat = { count: 2, min: 0.1, max: 0.1, mean: 0.1, std: 0, p25: 0.1, p50: 0.1, p75: 0.1 };
const metric = renderToStaticMarkup(<MetricCard label="DPS" stat={stat} color="#fff" />);
for (const label of ['min', 'max', 'std', 'p25', 'p50', 'p75']) assert.ok(metric.includes(label));
assert.match(
  renderToStaticMarkup(<MetricCard label="Reactions" stat={null} color="#fff" />),
  /Not available/,
);
const bars = renderToStaticMarkup(
  <BarChart rows={[{ name: '<script>', value: 10, min: 5, max: 20, std: 3 }]} />,
);
assert.match(bars, /width:50%/);
assert.doesNotMatch(bars, /<script>/);
assert.equal(barScale([{ name: 'Zankou', value: 22854, max: 26000 }]), 28000);
assert.equal(barScale([{ name: 'Scorch', value: 9437, max: 10800 }]), 12000);
const hist = renderToStaticMarkup(<Histogram samples={[{ dps: 0 }, { dps: 0 }]} />);
assert.match(hist, /DPS Distribution/);
assert.match(hist, /mean 0/);
assert.doesNotMatch(hist, /One deterministic outcome/);
// Presentation checks use native aggregates; synthetic sentinels detect unit/slot mixups.
const aggregate = (
  engine.invoke('aggregate_trials', {
    request: execution.request,
    total: 1,
    reports: [execution.report],
    elapsed_ms: 0,
  }) as { data: Pick<TrialResult, 'aggregate' | 'samples'> }
).data;
const result: TrialResult = {
  ...aggregate,
  report: execution.report,
  request: execution.request,
  party: engine.inspectScript(reportSource).party!,
  compiled: reportCompiled,
  trials: 1,
  partial: false,
  warnings: [],
  actionLocations: [],
  occurrences: execution.occurrences,
};
const observed = (v: number) => ({
  count: 1,
  min: v,
  max: v,
  mean: v,
  std: 0,
  p25: v,
  p50: v,
  p75: v,
});
const point = {
  time: 1.25,
  dps: { ...observed(10), min: 2, max: 20, std: 3, p25: 5, p50: 9, p75: 14 },
  cumulative: observed(125),
};
const time = renderToStaticMarkup(<TimeChart points={[point]} horizon={2.5} />);
for (const text of [
  '1.25s',
  'mean 10',
  'std 3',
  'p25 5',
  'p50 9',
  'p75 14',
  'Simulation time (seconds)',
  'std-band',
])
  assert.ok(time.includes(text));
assert.doesNotMatch(time, /NaN|Infinity/);
assert.match(time, /class="stat-line stat-mean" points="[\d.]+,[\d.]+"/);
const cumulative = renderToStaticMarkup(
  <TimeChart points={[point]} horizon={2.5} cumulative statKey="cumulative" />,
);
for (const series of ['min', 'max', 'p25', 'p50', 'p75'])
  assert.ok(cumulative.includes(`stat-${series}`));
assert.doesNotMatch(cumulative, /std-band/);
assert.match(
  renderToStaticMarkup(<TimeChart points={[]} horizon={0} />),
  /No delivered observations/,
);
const zeroTime = renderToStaticMarkup(
  <TimeChart points={[{ time: 1, dps: observed(0), cumulative: observed(0) }]} horizon={1} />,
);
assert.doesNotMatch(zeroTime, /NaN|Infinity/);
assert.match(renderToStaticMarkup(<PieChart rows={[]} label="DPS" />), /No delivered observations/);
assert.match(
  renderToStaticMarkup(<PieChart rows={[{ name: 'Zero', value: 0 }]} label="DPS" />),
  /No delivered observations/,
);
const pie = renderToStaticMarkup(<PieChart rows={[{ name: '<script>', value: 12 }]} label="DPS" />);
assert.match(pie, /<circle/);
assert.match(pie, /100%/);
assert.doesNotMatch(pie, /<script>/);
const resources = structuredClone(result);
resources.aggregate.actors = [
  { ...resources.aggregate.actors[0], slot: 3, character_id: 'mint' },
  { ...resources.aggregate.actors[0], slot: 1, character_id: 'sakiri' },
];
const grant = (
  slot: number,
  source: (typeof resources.aggregate.energy_sources)[number]['source'],
  kind: string,
  [e, f, o]: number[],
) => ({
  actor_index: slot,
  source,
  kind,
  energy: observed(e),
  effective: observed(f),
  overflow: observed(o),
});
resources.aggregate.energy_sources = [
  grant(3, 'mint_hit', 'mint_hit_energy', [7, 5, 2]),
  grant(1, 'mint_hit', 'mint_hit_energy', [11, 8, 3]),
  grant(1, 'sakiri_hit', 'sakiri_hit_energy', [5, 5, 0]),
  grant(1, 'sakiri_hit', 'sakiri_direct_packet_energy_grant', [13, 9, 4]),
  grant(1, 'sakiri_hit', 'sakiri_riposte_energy_grant', [6, 6, 0]),
  grant(1, 'sakiri_hit', 'sakiri_unmapped_ordinary_variant', [4, 4, 0]),
];
const energy = renderToStaticMarkup(<EnergyPanels result={resources} />);
for (const text of [
  'Gross 46',
  'Retained 37',
  'Overflow 9',
  'Mint ordinary hit',
  'Sakiri ordinary hit',
  'Sakiri direct packet',
  'Sakiri Riposte',
  'Mint 7 energy',
  'Sakiri 11 energy',
  '#65cdb6',
  '#eadb37',
  'Row total: 18',
])
  assert.ok(energy.includes(text), text);
assert.equal(
  (energy.match(/class="chart-name" title="Sakiri ordinary hit"/g) || []).length,
  1,
  'Identically labeled energy sources must merge into a single chart row',
);
const damage = renderToStaticMarkup(<DamagePanels result={result} />);
assert.match(damage, /Damage Timeline \(absolute time\)/);
assert.match(damage, /Damage per second/);
assert.match(damage, /Cumulative Damage/);
assert.match(damage, /DPS by Character Element/);
assert.match(damage, /Owner: Mint/);
assert.match(
  damage,
  /<div class="panel-title-group"><h2>Source DPS<\/h2><button[^>]*class="time-basis-toggle"[^>]*aria-label="Source damage view mode"[^>]*>full<\/button><\/div>/,
);
const compactDamage = renderToStaticMarkup(<DamagePanels result={result} sourceMode="comp" />);
assert.match(
  compactDamage,
  /<div class="panel-title-group"><h2>Source DPS<\/h2><button[^>]*class="time-basis-toggle"[^>]*aria-label="Source damage view mode"[^>]*>comp<\/button><\/div>/,
);
assert.doesNotMatch(compactDamage, /class="bar-error"/);
assert.ok(damage.indexOf('Damage Timeline (absolute time)') < damage.indexOf('Cumulative Damage'));
const storage = new Map<string, string>(),
  adapter = {
    getItem: (key: string) => storage.get(key) ?? null,
    setItem: (key: string, value: string) => {
      storage.set(key, value);
    },
  };
assert.equal(loadConfig(adapter).preset, 's5');
assert.ok(saveConfig({ source: 'invalid draft', preset: 's1' }, adapter));
assert.equal(loadConfig(adapter).source, 'invalid draft');
assert.deepEqual([...storage.keys()], [CONFIG_KEY]);
const blockedStorage = {
  getItem: () => {
    throw Error('blocked');
  },
  setItem: () => {
    throw Error('blocked');
  },
};
assert.equal(saveConfig({ source: 'x', preset: 's1' }, blockedStorage), false);
assert.equal(loadConfig(blockedStorage).preset, 's5');
const malformedSavedConfig = '{not-json';
let preservedMalformedValue = malformedSavedConfig;
const recovery = loadConfig({
  getItem: () => preservedMalformedValue,
});
assert.ok(recovery.storageWarning?.includes('left untouched'));
assert.equal(preservedMalformedValue, malformedSavedConfig);
assert.throws(
  () => normalizeSource('let scenario=' + JSON.stringify(compiled.request) + ';', engine),
  /unexpected token/,
);
for (const character of CATALOG.characters) {
  const image = renderToStaticMarkup(
    <Artwork category="characters" id={character.slug} alt={character.name} />,
  );
  assert.ok(image.includes('<img'));
  assert.ok(image.includes(`assets/artwork/${ARTWORK.characters[character.slug]}`));
}
for (const weaponId of Object.keys(CATALOG.weapons)) {
  const image = renderToStaticMarkup(
    <Artwork category="weapons" id={weaponId} alt={CATALOG.weapons[weaponId]} />,
  );
  assert.ok(image.includes('<img'));
  assert.ok(image.includes(`assets/artwork/${ARTWORK.weapons[weaponId]}`));
}
const artworkDirectory = new URL('../public/assets/artwork/', import.meta.url);
const mappedFiles = Object.values(ARTWORK).flatMap(Object.values).sort();
assert.deepEqual(readdirSync(artworkDirectory).sort(), mappedFiles);
for (const [category, entries] of Object.entries(ARTWORK)) {
  for (const [id, file] of Object.entries(entries)) {
    const image = renderToStaticMarkup(<Artwork category={category} id={id} alt={id} />);
    assert.ok(image.includes(`assets/artwork/${file}`));
    const bytes = readFileSync(new URL(file, artworkDirectory));
    assert.equal(bytes.toString('ascii', 0, 4), 'RIFF');
    assert.equal(bytes.toString('ascii', 8, 12), 'WEBP');
  }
}
for (const id of ['unknown', '__proto__', '../outside'])
  assert.match(
    renderToStaticMarkup(<Artwork category="weapons" id={id} alt={id} />),
    /asset-placeholder/,
  );
for (const category of ['characters', 'weapons', 'cartridges', 'elements'])
  assert.ok(cards.includes(`assets/artwork/${category}_`));
await checkStudioReportRoundtrip(
  { source: reportSource, result, status: 'completed', failure: null, failedTrial: null },
  engine,
  execution,
  model,
);
console.log(
  'Studio: escaped React rendering, missing observations, exact chart ranges, local artwork, current-key storage, invalid drafts, report export/import round-trip and unavailable storage passed',
);
