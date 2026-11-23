import assert from 'node:assert/strict';
import { compileNte, getPresetSource, editOptions, engine } from './native-compiler.ts';
import { horizontalBars, cardStats } from './render-helpers.tsx';
import { distributionModel } from '../src/components/charts/models.ts';
import { createElement } from 'react';
import { renderToStaticMarkup } from 'react-dom/server';
import { Targets } from '../src/features/results/Targets.tsx';
import { DamagePanels } from '../src/features/results/DamagePanels.tsx';
import { Summary } from '../src/features/results/Summary.tsx';
import { checkFieldTimeControls } from './field-time-cases.mjs';
import { ResultsPage } from '../src/features/results/ResultsPage.tsx';
import { ReportExport } from '../src/features/import-export/ReportExport.tsx';
import { elementColors, palette } from '../src/components/ui/format.ts';
import { sourceLabel } from '../src/domain/reports/source-label.ts';
import {
  importFile,
  importReportFile,
  serializeReportBundle,
} from '../src/infrastructure/storage/files.ts';
import { checkPortableReportImports } from './report-import-cases.mjs';
import { checkCompactSourceDamage } from './compact-source-cases.mjs';
const base = getPresetSource('s1').replace('mint attack;', 'mint attack:5;');
const unavailableExport = renderToStaticMarkup(
  createElement(ReportExport, { run: { result: null, runId: 0 }, busy: false }),
);
assert.match(unavailableExport, /id="export-report-btn"[^>]*disabled/);
assert.match(unavailableExport, /Run a simulation to deliver results/);
const source = '# kept comment\n' + base.replace('mint attack:5;', 'repeat 5 { mint attack; }');
const changed = editOptions(source, { iteration: 30, seed: 120, crit: 'random' }),
  c = compileNte(changed);
assert.equal(c.trials, 30);
assert.equal(c.request.seed, 120);
assert.equal(c.optionRanges.filter((r) => r.name === 'iteration').length, 1);
assert.deepEqual(c.request.actions, compileNte(base).request.actions);
assert.ok(changed.startsWith('# kept comment\n'));
assert.ok(changed.includes('repeat 5 { mint attack; }'));
const commented = source.replace(
  'options iteration',
  '# options is a keyword in this comment\noptions iteration',
);
assert.equal(
  compileNte(editOptions(commented, { seed: 1234, iteration: 12, crit: 'random' })).request.seed,
  1234,
);
assert.equal(editOptions(changed, { seed: 120 }), changed);
assert.throws(() => editOptions(base, { crit: 'expected' }), /crit must be fixed or random/);
assert.throws(() => editOptions(base, { editor_font: 18 }), /Unsupported option/);
assert.throws(() => editOptions(base, { time_unit: 'frames' }), /Unsupported option/);
assert.match(
  cardStats(engine.inspectScript(changed).party.actors[0], 'equipment'),
  /Build contributions/,
);
const actor = structuredClone(engine.inspectScript(changed).party.actors[0]);
const zero = '0/1';
Object.assign(actor.static_stats, {
  damage_bonus: zero,
  elemental_bonus: zero,
  elemental_bonuses: { NATURE: zero, COSMOS: zero },
  healing_bonus: zero,
  shield_bonus: zero,
  mental_damage_bonus: zero,
  cycle_intensity: zero,
  break_intensity: zero,
  break_efficiency_multiplier: '2/2',
});
Object.assign(actor, {
  charge_efficiency: '3/3',
  initial_hp: actor.static_stats.hp,
  initial_energy: 0,
  initial_cycle: 0,
  initial_resources: {},
});
const emptyStats = cardStats(actor).split('</table>')[0];
assert.match(emptyStats, /CRIT Rate/);
assert.doesNotMatch(
  emptyStats,
  /DMG Bonus|Healing Bonus|Shield Bonus|Intensity|Efficiency Multiplier|Starting HP|Starting Energy|Starting Cycle/,
);
Object.assign(actor.static_stats, {
  damage_bonus: '-1/5',
  elemental_bonuses: { NATURE: '3/10' },
  break_efficiency_multiplier: '9/10',
});
actor.charge_efficiency = '3/2';
const changedStats = cardStats(actor);
assert.match(changedStats, /General DMG Bonus<\/td><td>-20%/);
assert.match(changedStats, /Anima DMG Bonus<\/td><td>30%/);
assert.match(changedStats, /Break Efficiency Multiplier<\/td><td>0.9/);
assert.match(changedStats, /Charge Efficiency Multiplier<\/td><td>1.5/);
const model = distributionModel([{ dps: 0 }, { dps: 10 }, { dps: 10 }, { dps: 20 }, { dps: 30 }]);
assert.equal(
  model.bins.reduce((n, b) => n + b.count, 0),
  5,
);
assert.deepEqual([model.mean, model.p25, model.p50, model.p75], [14, 10, 10, 20]);
assert.equal(distributionModel([{ dps: 5 }, { dps: null }]), null);
const durationRows = [
  { relative_elapsed_us: 120_000_000, absolute_elapsed_us: 140_000_000 },
  { relative_elapsed_us: 120_000_000, absolute_elapsed_us: 150_000_000 },
];
assert.equal(distributionModel(durationRows, 'duration').mean, 120);
assert.equal(distributionModel(durationRows, 'absolute_duration').mean, 145);

assert.equal(distributionModel(Array(20).fill({ dps: 0 })).bins[0].count, 20);
assert.doesNotMatch(
  horizontalBars([{ name: 'x', value: 18, min: 18, max: 18, std: 0 }]),
  /bar-error/,
);
const target = structuredClone(c.request.targets[0]);
target.resistance = '1/5';
target.elemental_resistances = {
  NATURE: '3/10',
  PSYCHE: '1/10',
};
const targets = renderToStaticMarkup(createElement(Targets, { targets: [target] }));
assert.match(targets, /Anima resistance: 30% \(element override\)/);
assert.match(targets, /Psyche resistance: 10%/);
assert.match(targets, /Cosmos resistance: 20% \(base resistance\)/);
assert.equal((targets.match(/class="target-resistance"/g) || []).length, 6);
assert.deepEqual(elementColors, {
  Anima: '#62d8b1',
  Incantation: '#f45e88',
  Cosmos: '#e4e8d5',
  Chaos: '#a992e8',
  Lakshana: '#ecdb68',
  Psyche: '#64c4e5',
});
for (const [effect_id, name] of [
  ['ntesim.reaction.scorch', 'Scorch (DoT)'],
  ['ntesim.reaction.blossom', 'Blossom · Vita pistil'],
  ['ntesim.reaction.nova', 'Nova'],
  ['ntesim.reaction.hexed_followup', 'Hexed follow-up'],
]) {
  assert.equal(sourceLabel({ ability_id: 'ntesim.reaction', effect_id }), name);
}
assert.match(
  sourceLabel({ ability_id: 'GA_Oneiroi_Skill', effect_id: 'GE_Player_Oneiroi_Skill_Gun_Damage' }),
  /Gun lamb/,
);
assert.equal(
  sourceLabel({ ability_id: 'GA_Lacrimosa_Skill', effect_id: 'GE_Player_Lacrimosa_Blood_Damage' }),
  'Morning Tomato · Nightmare (DoT)',
);
assert.equal(
  sourceLabel({
    ability_id: 'GA_lacrimosa_Passive_1',
    effect_id: 'GE_Player_Lacrimosa_AnHunZhouTwo_Damage',
  }),
  'Tomato Banquet',
);
assert.equal(
  sourceLabel({ ability_id: 'unmapped', effect_id: 'distinct_packet' }),
  'unmapped · distinct_packet',
);
const result = {
  aggregate: { actors: [], timeline: [], sources: [], targets: [], metrics: {} },
  request: engine.compileSource(getPresetSource('s5')).compiled.request,
};
const damageMarkup = renderToStaticMarkup(createElement(DamagePanels, { result }));
assert.ok(!damageMarkup.includes('character-group'));
assert.ok(!damageMarkup.includes('>Grouping<'));
const legend = damageMarkup.match(/aria-label="Source damage owners">([\s\S]*?)<\/div>/)[1];
for (const name of ['Mint', 'Sakiri', 'Zero', 'Iroi']) assert.ok(legend.includes(name));
assert.equal((legend.match(/role="listitem"/g) || []).length, 4);

checkCompactSourceDamage();
const healingSource =
  'options duration=10 iteration=2; iroi char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_hp%=0.99; target hp=100000000 def=0 resist=0; active iroi; iroi skill;';
const artifact = engine.compileSource(healingSource).compiled;
const execution = engine.runSourceSample({ compiled: artifact, seed: 42 });
const support = engine.invoke('aggregate_trials', {
  request: execution.request,
  total: 2,
  reports: [execution.report],
  elapsed_ms: 0,
}).data;
assert.ok(support.aggregate.metrics.hps.relative.mean > 0);
assert.equal(
  support.aggregate.metrics.hps.relative.mean,
  support.aggregate.metrics.gross_healing.mean / 10,
);
assert.equal(support.samples[0].hps.relative, support.aggregate.metrics.hps.relative.mean);
assert.equal(support.samples[0].shp.relative, 0);
assert.equal(support.aggregate.metrics.shp.relative.mean, 0);
const hpsDist = distributionModel(support.samples, 'hps');
assert.deepEqual([hpsDist.count, hpsDist.min], [1, support.samples[0].hps.relative]);
const shpDist = distributionModel(support.samples, 'shp');
assert.deepEqual([shpDist.count, shpDist.min], [1, 0]);

const shieldSource =
  'options duration=10 iteration=2; adler char lvl=80/80 awakening=0 talent=8,8,8 qte=8 start_energy=0; target hp=100000000 def=0 resist=0; active adler; adler skill; wait(1);';
const shieldArtifact = engine.compileSource(shieldSource).compiled;
const shieldExecution = engine.runSourceSample({ compiled: shieldArtifact, seed: 42 });
const shieldSupport = engine.invoke('aggregate_trials', {
  request: shieldExecution.request,
  total: 2,
  reports: [shieldExecution.report],
  elapsed_ms: 0,
}).data;
assert.ok(shieldSupport.aggregate.metrics.shp.relative.mean > 0);
assert.equal(
  shieldSupport.samples[0].shp.relative,
  shieldSupport.aggregate.metrics.shp.relative.mean,
);
assert.equal(shieldSupport.aggregate.metrics.shield_absorbed.mean, 0);
const adlerShpDist = distributionModel(shieldSupport.samples, 'shp');
assert.deepEqual(
  [adlerShpDist.count, adlerShpDist.min],
  [1, shieldSupport.samples[0].shp.relative],
);

const summary = renderToStaticMarkup(
  createElement(Summary, { result: { request: execution.request, aggregate: support.aggregate } }),
);
for (const re of [
  /Damage Per Second \(DPS\)/,
  /aria-label="Damage per second time basis"/,
  /<button[^>]*aria-label="Damage per second time basis"[^>]*>relative<\/button>/,
  /Healing Per Second \(HPS\)/,
  /Effective Shield HP \(SHP\)/,
  /HP\/s/,
  /HP/,
])
  assert.match(summary, re);
for (const re of [
  /Includes overheal/,
  /Time-averaged available shield capacity/,
  /Effective Healing Per Second/,
  /Shield Absorption Per Second/,
  /tooltip|caption|<select|<option/i,
])
  assert.doesNotMatch(summary, re);

const summaryAbsolute = renderToStaticMarkup(
  createElement(Summary, {
    result: { request: execution.request, aggregate: support.aggregate },
    timeBasis: 'absolute',
  }),
);
assert.match(
  summaryAbsolute,
  /<button[^>]*aria-label="Damage per second time basis"[^>]*>absolute<\/button>/,
);
assert.doesNotMatch(summaryAbsolute, /tooltip|caption|<select|<option/i);

const importedScript = await importFile(
  { name: 'test.nte', size: 50, text: async () => base },
  engine,
);
assert.ok(importedScript.includes('options'));

const reportState = {
  source: healingSource,
  status: 'stopped',
  failure: null,
  failedTrial: null,
  result: {
    compiled: artifact,
    party: engine.inspectScript(healingSource).party,
    request: execution.request,
    aggregate: support.aggregate,
    samples: support.samples,
    report: execution.report,
    actionLocations: [],
    occurrences: [],
    warnings: [],
    trials: 2,
    partial: true,
  },
};
const exportedReportJson = serializeReportBundle(reportState);
const importedReport = await importReportFile(
  { name: 'report.json', size: exportedReportJson.length, text: async () => exportedReportJson },
  engine,
);
assert.equal(importedReport.source, healingSource);
assert.ok(importedReport.result.aggregate);

await checkPortableReportImports(reportState, importedReport, engine, execution);
await checkFieldTimeControls(execution, support, engine, exportedReportJson);

const baseRun = {
  status: 'completed',
  source: base,
  controller: null,
  stage: 'Completed',
  progress: 100,
  failure: null,
  failedTrial: null,
  cancel: () => {},
  resume: () => {},
  run: async () => {},
  loadReport: () => {},
};
const resultsMarkup = renderToStaticMarkup(
  createElement(ResultsPage, {
    run: { ...baseRun, result: importedReport.result, runId: 'test-run' },
    onExportFile: () => {},
    onBusy: () => {},
    busy: false,
  }),
);
for (const id of ['results-state', 'edit-config', 'open-sample', 'export-report'])
  assert.doesNotMatch(resultsMarkup, new RegExp(`id="${id}"`));
assert.match(resultsMarkup, /Target Information/);
assert.match(
  resultsMarkup,
  /<div class="panel-title-group"><h2>Source DPS<\/h2><button type="button" class="time-basis-toggle" aria-label="Source damage view mode">full<\/button><\/div>/,
);

const targetsMarkup = renderToStaticMarkup(
  createElement(Targets, { targets: execution.request.targets }),
);
assert.match(targetsMarkup, /class="target-header-line"/);
assert.match(targetsMarkup, /Target 1:/);

const multiTargetResult = structuredClone(importedReport.result);
multiTargetResult.request.targets = [
  ...execution.request.targets,
  { ...execution.request.targets[0], target_index: 1 },
  { ...execution.request.targets[0], target_index: 2 },
];
const multiTargetMarkup = renderToStaticMarkup(
  createElement(ResultsPage, {
    run: { ...baseRun, result: multiTargetResult, runId: 'multi-run' },
    onExportFile: () => {},
    onBusy: () => {},
    busy: false,
  }),
);
assert.doesNotMatch(multiTargetMarkup, /Target Information/);

await assert.rejects(
  () => importReportFile({ name: 'bad.nte', size: 10, text: async () => 'not json' }, engine),
  /JSON format/,
);
await assert.rejects(
  () =>
    importReportFile(
      { name: 'no-aggregate.json', size: 20, text: async () => JSON.stringify({ source: base }) },
      engine,
    ),
  /simulation report data/,
);

console.log(
  'Report controls: one options declaration, source/comment/loop preservation, separate editor preferences, histogram, import script/report and source edits passed',
);
