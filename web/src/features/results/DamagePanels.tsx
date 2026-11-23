import { useState } from 'react';
import type { TrialResult } from '../../domain/contracts/trials';
import { english } from '../../domain/localization';
import { CATALOG } from '../../domain/catalog';
import { sourceLabel, resolveAbilityTitle } from '../../domain/reports/source-label';
import { palette, elementColors } from '../../components/ui/format';
import { Panel, Select } from '../../components/ui/Panel';
import { TimeChart } from '../../components/charts/TimeChart';
import { PieChart } from '../../components/charts/PieChart';
import { BarChart } from '../../components/charts/BarChart';
import { ChartLegend } from '../../components/charts/ChartLegend';
export function DamagePanels({
  result,
  timeBasis = 'relative',
  sourceMode: controlledSourceMode,
  onSourceModeChange,
  sourceKind: controlledSourceKind,
  onSourceKindChange,
}: {
  result: TrialResult;
  timeBasis?: 'relative' | 'absolute';
  sourceMode?: 'full' | 'comp';
  onSourceModeChange?: (mode: 'full' | 'comp') => void;
  sourceKind?: 'dps' | 'hits';
  onSourceKindChange?: (kind: string) => void;
}) {
  const { aggregate: a, request } = result,
    lastTime = a.timeline[a.timeline.length - 1]?.time,
    horizon = lastTime ?? 0;
  const [timeline, setTimeline] = useState('dps'),
    [kind, setKind] = useState('dps'),
    [uncontrolledSourceKind, setUncontrolledSourceKind] = useState('dps'),
    [selected, setSelected] = useState('all'),
    [uncontrolledSourceMode, setUncontrolledSourceMode] = useState<'full' | 'comp'>('full'),
    [expandedSources, setExpandedSources] = useState(false);

  const sourceMode = controlledSourceMode ?? uncontrolledSourceMode;
  const setSourceMode = (mode: 'full' | 'comp') => {
    if (onSourceModeChange) onSourceModeChange(mode);
    else setUncontrolledSourceMode(mode);
  };
  const sourceKind = controlledSourceKind ?? uncontrolledSourceKind;
  const setSourceKind = (kind: string) => {
    if (onSourceKindChange) onSourceKindChange(kind);
    else setUncontrolledSourceKind(kind);
  };
  const elements = new Map<string, number>();
  const element = (id: string) => CATALOG.characters.find((c) => c.id === id)?.element || 'Unknown';
  for (const x of a.actors) {
    const name = element(x.character_id);
    const dpsStat = x.dps?.[timeBasis];
    if (dpsStat) elements.set(name, (elements.get(name) || 0) + dpsStat.mean);
  }
  const aggregateDps = a.metrics.dps?.[timeBasis] ?? null;
  const field = kind === 'hits' ? 'hits' : kind === 'dps' ? 'dps' : 'damage',
    sourceField = sourceKind === 'hits' ? 'hits' : 'dps';
  const units: [string, string][] = [
    ['dps', 'DPS'],
    ['damage', 'Damage'],
    ['hits', 'Damage instances'],
  ];
  return (
    <>
      <div className="report-timelines">
        <Panel
          title="Damage Timeline (absolute time)"
          controls={
            <Select
              id="timeline-type"
              label="Type"
              value={timeline}
              onChange={setTimeline}
              options={[
                ['dps', 'Damage per second'],
                ['cumulative', 'Cumulative Damage'],
              ]}
            />
          }
        >
          <div id="damage-timeline">
            <TimeChart
              points={a.timeline}
              horizon={horizon}
              cumulative={timeline === 'cumulative'}
              statKey={timeline === 'cumulative' ? 'cumulative' : 'dps'}
              label={timeline === 'cumulative' ? 'Cumulative damage' : 'Damage per second'}
            />
          </div>
        </Panel>
      </div>
      {aggregateDps ? (
        <div className="three-column">
          <Panel title="Character DPS Distribution">
            <div id="character-pie">
              <PieChart
                rows={a.actors.map((x) => ({
                  name: english.character(x.character_id),
                  value: x.dps[timeBasis]!.mean,
                  color: palette[x.slot % palette.length],
                }))}
                label="DPS"
              />
            </div>
          </Panel>
          <Panel title="DPS by Character Element">
            <div id="element-pie">
              <PieChart
                rows={[...elements].map(([name, value]) => ({
                  name,
                  value,
                  color: elementColors[name],
                }))}
                label="DPS"
              />
            </div>
          </Panel>
          <Panel title="Target DPS Distribution">
            <div id="target-pie">
              <PieChart
                rows={a.targets.map((t, i) => ({
                  name: 'Target ' + (t.target_index + 1),
                  value: t.dps[timeBasis]!.mean,
                  color: palette[i % palette.length],
                }))}
                label="DPS"
              />
            </div>
          </Panel>
        </div>
      ) : (
        <p className="unavailable">
          {timeBasis === 'relative' ? 'Relative' : 'Absolute'} DPS unavailable for this seed set.
        </p>
      )}
      <Panel
        title={
          kind === 'dps'
            ? 'Character DPS'
            : kind === 'hits'
              ? 'Character Damage Instances'
              : 'Character Damage'
        }
        controls={
          <Select
            id="character-type"
            label="Type"
            value={kind}
            onChange={setKind}
            options={units}
          />
        }
      >
        <div id="character-chart">
          {field === 'dps' && !aggregateDps ? (
            <p className="unavailable">
              {timeBasis === 'relative' ? 'Relative' : 'Absolute'} DPS unavailable for this seed
              set.
            </p>
          ) : (
            <BarChart
              rows={[...a.actors]
                .map((x) => {
                  const stat = field === 'dps' ? x.dps[timeBasis]! : x[field]!;
                  return {
                    name: english.character(x.character_id),
                    value: stat.mean,
                    min: stat.min,
                    max: stat.max,
                    std: stat.std,
                    color: palette[x.slot % palette.length],
                  };
                })
                .sort((x, y) => x.value - y.value)}
              unit={kind === 'hits' ? 'damage instances' : kind === 'damage' ? 'damage' : 'DPS'}
            />
          )}
        </div>
      </Panel>
      <Panel
        title={sourceKind === 'dps' ? 'Source DPS' : 'Source Damage Instances'}
        titleControl={
          <button
            type="button"
            className="time-basis-toggle"
            aria-label="Source damage view mode"
            onClick={() => setSourceMode(sourceMode === 'full' ? 'comp' : 'full')}
          >
            {sourceMode}
          </button>
        }
        controls={
          <>
            <Select
              id="source-type"
              label="Type"
              value={sourceKind}
              onChange={setSourceKind}
              options={[
                ['dps', 'DPS'],
                ['hits', 'Damage Instances'],
              ]}
            />
            <Select
              id="source-character"
              label="Character"
              value={selected}
              onChange={setSelected}
              options={[
                ['all', 'All'],
                ...request.actors.map(
                  (a) => [String(a.slot), english.character(a.character_id)] as [string, string],
                ),
              ]}
            />
          </>
        }
      >
        <div id="source-chart">
          <ChartLegend
            label="Source damage owners"
            series={request.actors
              .filter((actor) => selected === 'all' || String(actor.slot) === selected)
              .map((actor) => ({
                name: english.character(actor.character_id),
                color: palette[actor.slot % palette.length],
              }))}
          />
          {(() => {
            if (sourceField === 'dps' && !aggregateDps)
              return (
                <p className="unavailable">
                  {timeBasis === 'relative' ? 'Relative' : 'Absolute'} DPS unavailable for this seed
                  set.
                </p>
              );
            const filteredSources = a.sources.filter(
              (s) => selected === 'all' || String(s.actor_index) === selected,
            );
            const allSourceRows =
              sourceMode === 'comp'
                ? (() => {
                    const grouped = new Map<
                      string,
                      { actor_index: number; ability_id: string; value: number }
                    >();
                    for (const s of filteredSources) {
                      const stat = sourceField === 'dps' ? s.dps[timeBasis]! : s[sourceField]!;
                      const key = `${s.actor_index}:${s.ability_id}`;
                      const existing = grouped.get(key);
                      if (existing) {
                        existing.value += stat.mean;
                      } else {
                        grouped.set(key, {
                          actor_index: s.actor_index,
                          ability_id: s.ability_id,
                          value: stat.mean,
                        });
                      }
                    }
                    return [...grouped.values()]
                      .map((g) => ({
                        name: resolveAbilityTitle(g.ability_id),
                        value: g.value,
                        color: palette[g.actor_index % palette.length],
                        detail:
                          'Owner: ' +
                          english.character(
                            request.actors.find((x) => x.slot === g.actor_index)!.character_id,
                          ),
                      }))
                      .sort((x, y) => y.value - x.value);
                  })()
                : filteredSources
                    .map((s) => {
                      const stat = sourceField === 'dps' ? s.dps[timeBasis]! : s[sourceField]!;
                      return {
                        name: sourceLabel(s),
                        value: stat.mean,
                        min: stat.min,
                        max: stat.max,
                        std: stat.std,
                        color: palette[s.actor_index % palette.length],
                        detail:
                          'Owner: ' +
                          english.character(
                            request.actors.find((x) => x.slot === s.actor_index)!.character_id,
                          ) +
                          `\nAbility: ${s.ability_id}\nEffect: ${s.effect_id}`,
                      };
                    })
                    .sort((x, y) => y.value - x.value);
            const shouldFold = allSourceRows.length >= 20;
            const visibleRows =
              shouldFold && !expandedSources ? allSourceRows.slice(0, 15) : allSourceRows;
            return (
              <>
                <BarChart rows={visibleRows} unit={sourceKind === 'dps' ? 'DPS' : 'Instances'} />
                {shouldFold && (
                  <button
                    type="button"
                    className="chart-expand-bar"
                    aria-expanded={expandedSources}
                    aria-label={expandedSources ? 'Collapse sources list' : 'Expand sources list'}
                    onClick={() => setExpandedSources((v) => !v)}
                  >
                    <span className="chart-expand-pill">
                      <svg
                        viewBox="0 0 24 24"
                        width="14"
                        height="14"
                        aria-hidden="true"
                        style={{ display: 'block' }}
                      >
                        <path
                          d={expandedSources ? 'M6 15l6-6 6 6' : 'M6 9l6 6 6-6'}
                          fill="none"
                          stroke="currentColor"
                          strokeWidth="2"
                          strokeLinecap="round"
                          strokeLinejoin="round"
                        />
                      </svg>
                    </span>
                  </button>
                )}
              </>
            );
          })()}
        </div>
      </Panel>
    </>
  );
}
