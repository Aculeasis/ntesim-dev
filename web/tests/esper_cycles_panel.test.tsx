import assert from 'node:assert/strict';
import { renderToStaticMarkup } from 'react-dom/server';
import { EnergyPanels } from '../src/features/results/EnergyPanels';
import type { TrialResult, ReactionAggregate } from '../src/domain/contracts/trials';
import { engine } from './native-compiler';

const reportSource =
  'options duration=3 iteration=1; mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1; target hp=100000000 def=0 resist=0; active mint; mint attack; wait(1);';
const reportCompiled = engine.compileSource(reportSource).compiled!;
const execution = engine.runSourceSample({ compiled: reportCompiled, seed: 42 });
const aggregate = (
  engine.invoke('aggregate_trials', {
    request: execution.request,
    total: 1,
    reports: [execution.report],
    elapsed_ms: 0,
  }) as { data: Pick<TrialResult, 'aggregate' | 'samples'> }
).data;
const baseResult: TrialResult = {
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

// 1. Team with no available reactions (reactions is empty) hides Esper Cycles panel and renders two-column layout
const emptyResult = structuredClone(baseResult);
emptyResult.aggregate.reactions = [];
const emptyHtml = renderToStaticMarkup(<EnergyPanels result={emptyResult} />);
assert.ok(emptyHtml.includes('two-column'), 'Should render two-column layout');
assert.ok(!emptyHtml.includes('three-column'), 'Should not render three-column layout');
assert.ok(!emptyHtml.includes('Esper Cycles'), 'Should not render Esper Cycles panel');

// 2. Team with available reactions renders three-column layout and Esper Cycles panel with error bars
const mockReactions: ReactionAggregate[] = [
  {
    name: 'Blossom',
    count: 10,
    mean: 12.5,
    min: 8,
    max: 16,
    std: 2.1,
    p25: 10,
    p50: 12,
    p75: 14,
  },
  {
    name: 'Remora',
    count: 10,
    mean: 4.0,
    min: 0,
    max: 6,
    std: 1.5,
    p25: 3,
    p50: 4,
    p75: 5,
  },
  {
    name: 'Charge',
    count: 10,
    mean: 0,
    min: 0,
    max: 0,
    std: 0,
    p25: 0,
    p50: 0,
    p75: 0,
  },
];

const reactionResult = structuredClone(baseResult);
reactionResult.aggregate.reactions = mockReactions;
const reactionHtml = renderToStaticMarkup(<EnergyPanels result={reactionResult} />);

assert.ok(reactionHtml.includes('three-column'), 'Should render three-column layout');
assert.ok(reactionHtml.includes('Esper Cycles'), 'Should render Esper Cycles panel');

// Verify reactions are rendered in order (Blossom first, then Remora, then Charge)
const blossomIdx = reactionHtml.indexOf('Blossom');
const remoraIdx = reactionHtml.indexOf('Remora');
const chargeIdx = reactionHtml.indexOf('Charge');
assert.ok(blossomIdx !== -1 && remoraIdx !== -1 && chargeIdx !== -1);
assert.ok(blossomIdx < remoraIdx, 'Blossom should precede Remora');
assert.ok(remoraIdx < chargeIdx, 'Remora should precede Charge');

// Verify colors
assert.ok(reactionHtml.includes('#62d8b1'), 'Blossom color');
assert.ok(reactionHtml.includes('#ecdb68'), 'Remora color');
assert.ok(reactionHtml.includes('#eadb37'), 'Charge color');

// Verify error bar (whiskers) rendered for reactions with variance (max > min)
assert.ok(reactionHtml.includes('bar-error'), 'Should render bar-error whiskers');

// Verify tooltip contents (mean, min, max, std)
assert.ok(reactionHtml.includes('mean 12.5'), 'Should format mean');
assert.ok(reactionHtml.includes('min 8'), 'Should format min');
assert.ok(reactionHtml.includes('max 16'), 'Should format max');
assert.ok(reactionHtml.includes('std 2.1'), 'Should format std');

// Available reaction with zero triggers displays 0 without error bar
assert.ok(reactionHtml.includes('mean 0'), 'Charge with 0 triggers should display mean 0');
