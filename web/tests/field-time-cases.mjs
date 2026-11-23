import assert from 'node:assert/strict';
import { createElement } from 'react';
import { renderToStaticMarkup } from 'react-dom/server';
import { EnergyPanels } from '../src/features/results/EnergyPanels.tsx';
import { importReportFile } from '../src/infrastructure/storage/files.ts';

export async function checkFieldTimeControls(execution, support, engine, exportedReportJson) {
  const energyPanelsRelative = renderToStaticMarkup(
    createElement(EnergyPanels, {
      result: { request: execution.request, aggregate: support.aggregate },
      timeBasis: 'relative',
    }),
  );
  assert.match(energyPanelsRelative, /Field Time Distribution/);
  assert.match(energyPanelsRelative, /aria-label="Field time distribution time basis"/);
  const fieldTimeHeadingRel = energyPanelsRelative.match(
    /<div class="panel-heading"><h2>Field Time Distribution<\/h2>[\s\S]*?<\/div>/,
  )?.[0];
  assert.ok(fieldTimeHeadingRel);
  assert.match(
    fieldTimeHeadingRel,
    /<button[^>]*aria-label="Field time distribution time basis"[^>]*>relative<\/button>/,
  );
  assert.doesNotMatch(fieldTimeHeadingRel, /tooltip|caption|<select|<option/i);

  const energyPanelsAbsolute = renderToStaticMarkup(
    createElement(EnergyPanels, {
      result: { request: execution.request, aggregate: support.aggregate },
      timeBasis: 'absolute',
    }),
  );
  const fieldTimeHeadingAbs = energyPanelsAbsolute.match(
    /<div class="panel-heading"><h2>Field Time Distribution<\/h2>[\s\S]*?<\/div>/,
  )?.[0];
  assert.ok(fieldTimeHeadingAbs);
  assert.match(
    fieldTimeHeadingAbs,
    /<button[^>]*aria-label="Field time distribution time basis"[^>]*>absolute<\/button>/,
  );
  assert.doesNotMatch(fieldTimeHeadingAbs, /tooltip|caption|<select|<option/i);

  const twoCharResult = {
    request: execution.request,
    aggregate: {
      ...support.aggregate,
      actors: [
        {
          character_id: 'mint',
          slot: 0,
          actions: {},
          energy: { mean: 0, min: 0, max: 0, std: 0 },
          field: {
            relative: { count: 1, mean: 4, min: 4, max: 4, std: 0, p25: 4, p50: 4, p75: 4 },
            absolute: { count: 1, mean: 8, min: 8, max: 8, std: 0, p25: 8, p50: 8, p75: 8 },
          },
        },
        {
          character_id: 'sakiri',
          slot: 1,
          actions: {},
          energy: { mean: 0, min: 0, max: 0, std: 0 },
          field: {
            relative: { count: 1, mean: 4, min: 4, max: 4, std: 0, p25: 4, p50: 4, p75: 4 },
            absolute: { count: 1, mean: 4, min: 4, max: 4, std: 0, p25: 4, p50: 4, p75: 4 },
          },
        },
      ],
    },
  };
  const twoCharRelHtml = renderToStaticMarkup(
    createElement(EnergyPanels, { result: twoCharResult, timeBasis: 'relative' }),
  );
  assert.match(twoCharRelHtml, /Mint: 50%/);
  assert.match(twoCharRelHtml, /Sakiri: 50%/);
  assert.match(twoCharRelHtml, /4 seconds/);

  const twoCharAbsHtml = renderToStaticMarkup(
    createElement(EnergyPanels, { result: twoCharResult, timeBasis: 'absolute' }),
  );
  assert.match(twoCharAbsHtml, /Mint: 66\.7%/);
  assert.match(twoCharAbsHtml, /8 seconds/);
  assert.match(twoCharAbsHtml, /Sakiri: 33\.3%/);
  assert.match(twoCharAbsHtml, /4 seconds/);

  const zeroRelativeResult = {
    request: execution.request,
    aggregate: {
      ...support.aggregate,
      actors: [
        {
          character_id: 'mint',
          slot: 0,
          actions: {},
          energy: { mean: 0, min: 0, max: 0, std: 0 },
          field: {
            relative: { count: 1, mean: 0, min: 0, max: 0, std: 0, p25: 0, p50: 0, p75: 0 },
            absolute: { count: 1, mean: 5, min: 5, max: 5, std: 0, p25: 5, p50: 5, p75: 5 },
          },
        },
      ],
    },
  };
  const zeroRelativeHtml = renderToStaticMarkup(
    createElement(EnergyPanels, { result: zeroRelativeResult, timeBasis: 'relative' }),
  );
  assert.match(zeroRelativeHtml, /No delivered observations for this selection\./);
  assert.doesNotMatch(zeroRelativeHtml, /pie-chart/);

  const zeroAbsoluteHtml = renderToStaticMarkup(
    createElement(EnergyPanels, { result: zeroRelativeResult, timeBasis: 'absolute' }),
  );
  assert.match(zeroAbsoluteHtml, /pie-chart/);
  assert.match(zeroAbsoluteHtml, /Mint: 100%/);
  assert.match(zeroAbsoluteHtml, /5 seconds/);

  if (exportedReportJson && engine) {
    const badAbsoluteOnly = JSON.parse(exportedReportJson);
    badAbsoluteOnly.aggregate.actors[0].field = { mean: 10, min: 10, max: 10, std: 0 };
    await assert.rejects(
      () =>
        importReportFile(
          {
            name: 'obsolete-field.json',
            size: 100,
            text: async () => JSON.stringify(badAbsoluteOnly),
          },
          engine,
        ),
      /simulation report data/,
    );
  }
}
