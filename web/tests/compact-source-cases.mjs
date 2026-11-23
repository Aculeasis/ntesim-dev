import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createElement } from 'react';
import { renderToStaticMarkup } from 'react-dom/server';
import { DamagePanels } from '../src/features/results/DamagePanels.tsx';
import { palette } from '../src/components/ui/format.ts';

export function checkCompactSourceDamage() {
  const syntheticSources = [
    // Zankou (actor 0) Wildfire effects: rank 1, rank 18, rank 22
    {
      actor_index: 0,
      ability_id: 'GA_Zankou_Melee',
      effect_id: 'GE_Player_Zankou_Melee1_Damage',
      dps: {
        relative: { mean: 100, min: 90, max: 110, std: 5 },
        absolute: { mean: 90, min: 80, max: 100, std: 5 },
      },
      hits: { mean: 10, min: 8, max: 12, std: 1 },
    },
    {
      actor_index: 0,
      ability_id: 'GA_Zankou_Melee',
      effect_id: 'GE_Player_Zankou_Melee2_Damage',
      dps: {
        relative: { mean: 3, min: 2, max: 4, std: 1 },
        absolute: { mean: 2.7, min: 1.8, max: 3.6, std: 0.9 },
      },
      hits: { mean: 2, min: 1, max: 3, std: 0.5 },
    },
    {
      actor_index: 0,
      ability_id: 'GA_Zankou_Melee',
      effect_id: 'GE_Player_Zankou_Melee3_Damage',
      dps: {
        relative: { mean: 1, min: 0.5, max: 1.5, std: 0.5 },
        absolute: { mean: 0.9, min: 0.45, max: 1.35, std: 0.45 },
      },
      hits: { mean: 1, min: 1, max: 1, std: 0 },
    },
    // Zankou (actor 0) Reactions: Scorch (50) + Nova (20) = 70
    {
      actor_index: 0,
      ability_id: 'ntesim.reaction',
      effect_id: 'ntesim.reaction.scorch',
      dps: {
        relative: { mean: 50, min: 45, max: 55, std: 3 },
        absolute: { mean: 45, min: 40, max: 50, std: 3 },
      },
      hits: { mean: 5, min: 4, max: 6, std: 0.5 },
    },
    {
      actor_index: 0,
      ability_id: 'ntesim.reaction',
      effect_id: 'ntesim.reaction.nova',
      dps: {
        relative: { mean: 20, min: 18, max: 22, std: 1 },
        absolute: { mean: 18, min: 16, max: 20, std: 1 },
      },
      hits: { mean: 2, min: 2, max: 2, std: 0 },
    },
    // Mint (actor 1) Reactions: Blossom (30)
    {
      actor_index: 1,
      ability_id: 'ntesim.reaction',
      effect_id: 'ntesim.reaction.blossom',
      dps: {
        relative: { mean: 30, min: 25, max: 35, std: 2 },
        absolute: { mean: 27, min: 22, max: 32, std: 2 },
      },
      hits: { mean: 3, min: 2, max: 4, std: 0.5 },
    },
  ];
  for (let i = 0; i < 19; i++) {
    syntheticSources.push({
      actor_index: 1,
      ability_id: `filler_ability_${i}`,
      effect_id: `filler_effect_${i}`,
      dps: {
        relative: { mean: 4 + (i % 5), min: 2, max: 10, std: 1 },
        absolute: { mean: 3.6 + 0.9 * (i % 5), min: 1.8, max: 9, std: 0.9 },
      },
      hits: { mean: 2, min: 1, max: 3, std: 0.5 },
    });
  }
  const syntheticResult = {
    aggregate: {
      actors: [],
      timeline: [],
      sources: syntheticSources,
      targets: [],
      metrics: { dps: { relative: { mean: 300 }, absolute: { mean: 270 } } },
    },
    request: {
      actors: [
        { slot: 0, character_id: 'zankou' },
        { slot: 1, character_id: 'mint' },
      ],
      targets: [],
    },
  };

  // Default is detailed view (full)
  const defaultDetailedMarkup = renderToStaticMarkup(
    createElement(DamagePanels, { result: syntheticResult }),
  );
  assert.match(defaultDetailedMarkup, /Wildfire · Combo 1/);
  assert.match(defaultDetailedMarkup, /Scorch \(DoT\)/);
  assert.match(defaultDetailedMarkup, /Nova/);
  assert.match(defaultDetailedMarkup, /Blossom · Vita pistil/);
  // In folded detailed slice (top 15), effect 2 (mean 3) and effect 3 (mean 1) are outside visible 15
  assert.doesNotMatch(defaultDetailedMarkup, /Wildfire · Combo 2/);
  assert.doesNotMatch(defaultDetailedMarkup, /Wildfire · Combo 3/);
  assert.match(defaultDetailedMarkup, /class="bar-error"/);

  // Compact view: relative DPS
  const compactRelativeMarkup = renderToStaticMarkup(
    createElement(DamagePanels, { result: syntheticResult, sourceMode: 'comp' }),
  );
  // Effect-free ability label
  assert.match(compactRelativeMarkup, /title="Wildfire"/);
  assert.doesNotMatch(compactRelativeMarkup, /Wildfire · Combo/);
  // Sums include effects outside detailed visible slice: 100 + 3 + 1 = 104
  assert.match(compactRelativeMarkup, /aria-label="Wildfire 104 DPS"/);
  assert.match(compactRelativeMarkup, /Owner: Zankou/);
  // Mean-only display: no error bars
  assert.doesNotMatch(compactRelativeMarkup, /class="bar-error"/);

  // Owner separation & shared Reactions label
  assert.match(compactRelativeMarkup, /aria-label="Reactions 70 DPS"/);
  assert.match(compactRelativeMarkup, /aria-label="Reactions 30 DPS"/);
  assert.match(
    compactRelativeMarkup,
    new RegExp(
      `aria-label="Reactions 70 DPS"[\\s\\S]*?--series:${palette[0]}[\\s\\S]*?Owner: Zankou`,
    ),
  );
  assert.match(
    compactRelativeMarkup,
    new RegExp(
      `aria-label="Reactions 30 DPS"[\\s\\S]*?--series:${palette[1]}[\\s\\S]*?Owner: Mint`,
    ),
  );

  // Compact view: absolute DPS basis
  const compactAbsoluteMarkup = renderToStaticMarkup(
    createElement(DamagePanels, {
      result: syntheticResult,
      sourceMode: 'comp',
      timeBasis: 'absolute',
    }),
  );
  // 90 + 2.7 + 0.9 = 93.6
  assert.match(compactAbsoluteMarkup, /aria-label="Wildfire 93.6 DPS"/);
  // 45 + 18 = 63
  assert.match(compactAbsoluteMarkup, /aria-label="Reactions 63 DPS"/);
  assert.match(compactAbsoluteMarkup, /aria-label="Reactions 27 DPS"/);

  // Compact view: damage instances (hits)
  const compactHitsMarkup = renderToStaticMarkup(
    createElement(DamagePanels, {
      result: syntheticResult,
      sourceMode: 'comp',
      sourceKind: 'hits',
    }),
  );
  // 10 + 2 + 1 = 13
  assert.match(compactHitsMarkup, /aria-label="Wildfire 13 Instances"/);
  // 5 + 2 = 7
  assert.match(compactHitsMarkup, /aria-label="Reactions 7 Instances"/);
  assert.match(compactHitsMarkup, /aria-label="Reactions 3 Instances"/);

  // Title-adjacent toggle button: keyboard operable (type="button") and shows active mode
  assert.match(
    defaultDetailedMarkup,
    /<div class="panel-title-group"><h2>Source DPS<\/h2><button type="button" class="time-basis-toggle" aria-label="Source damage view mode">full<\/button><\/div>/,
  );
  assert.match(
    compactRelativeMarkup,
    /<div class="panel-title-group"><h2>Source DPS<\/h2><button type="button" class="time-basis-toggle" aria-label="Source damage view mode">comp<\/button><\/div>/,
  );

  // 390-pixel responsive viewport styles
  const panelsCss = readFileSync(
    new URL('../src/styles/studio-panels.css', import.meta.url),
    'utf8',
  );
  const responsiveCss = readFileSync(
    new URL('../src/styles/studio-responsive.css', import.meta.url),
    'utf8',
  );
  assert.match(panelsCss, /\.panel-title-group\s*\{[^}]*display:\s*flex/);
  assert.match(responsiveCss, /\.panel-title-group\s*h2\s*\{[^}]*width:\s*auto/);

  // Folding based on displayed row count:
  // 1) syntheticResult has 22 grouped rows (>= 20) in compact mode -> folded to 15, has expand button
  assert.match(compactRelativeMarkup, /class="chart-expand-bar"/);
  assert.equal((compactRelativeMarkup.match(/class="chart-row"/g) || []).length, 15);

  // 2) 25 total effects grouped into 5 abilities (< 20 grouped rows):
  // - Detailed mode (full): 25 rows >= 20 -> folded to 15, has expand button
  // - Compact mode (comp): 5 grouped rows < 20 -> not folded, all 5 visible without expand button
  // - Switching back to full: restores detailed effect rows
  const fewAbilitiesSources = [];
  for (let a = 0; a < 5; a++) {
    for (let e = 0; e < 5; e++) {
      fewAbilitiesSources.push({
        actor_index: 0,
        ability_id: `ability_${a}`,
        effect_id: `ability_${a}_effect_${e}`,
        dps: {
          relative: { mean: 10 + a + e, min: 5, max: 20, std: 2 },
          absolute: { mean: 9 + a + e, min: 4, max: 18, std: 2 },
        },
        hits: { mean: 2, min: 1, max: 3, std: 0.5 },
      });
    }
  }
  const fewAbilitiesResult = {
    aggregate: {
      actors: [],
      timeline: [],
      sources: fewAbilitiesSources,
      targets: [],
      metrics: { dps: { relative: { mean: 350 }, absolute: { mean: 320 } } },
    },
    request: { actors: [{ slot: 0, character_id: 'mint' }], targets: [] },
  };

  const fewDetailedMarkup = renderToStaticMarkup(
    createElement(DamagePanels, { result: fewAbilitiesResult, sourceMode: 'full' }),
  );
  assert.match(fewDetailedMarkup, /class="chart-expand-bar"/);
  assert.equal((fewDetailedMarkup.match(/class="chart-row"/g) || []).length, 15);

  const fewCompactMarkup = renderToStaticMarkup(
    createElement(DamagePanels, { result: fewAbilitiesResult, sourceMode: 'comp' }),
  );
  assert.doesNotMatch(fewCompactMarkup, /class="chart-expand-bar"/);
  assert.equal((fewCompactMarkup.match(/class="chart-row"/g) || []).length, 5);

  // Restoring detailed view
  const restoredDetailedMarkup = renderToStaticMarkup(
    createElement(DamagePanels, { result: fewAbilitiesResult, sourceMode: 'full' }),
  );
  assert.equal(restoredDetailedMarkup, fewDetailedMarkup);
}
