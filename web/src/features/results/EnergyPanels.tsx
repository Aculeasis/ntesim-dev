import { useState } from 'react';
import type { TrialResult } from '../../domain/contracts/trials';
import { english } from '../../domain/localization';
import { palette, format } from '../../components/ui/format';
import { Panel, Select } from '../../components/ui/Panel';
import { BarChart } from '../../components/charts/BarChart';
import { PieChart } from '../../components/charts/PieChart';
import { StackedBars } from '../../components/charts/StackedBars';
import { energySourceDetails } from '../../domain/reports/energy-sources';
export function EnergyPanels({
  result,
  timeBasis = 'relative',
  onTimeBasisChange,
}: {
  result: TrialResult;
  timeBasis?: 'relative' | 'absolute';
  onTimeBasisChange?: (basis: 'relative' | 'absolute') => void;
}) {
  const { request, aggregate: a } = result,
    [field, setField] = useState('energy');
  const energyField = field as 'energy' | 'effective' | 'overflow';
  const total = (key: 'energy' | 'effective' | 'overflow') =>
    a.energy_sources.reduce((n, s) => n + s[key].mean, 0);
  const actionTypes = [...new Set(a.actors.flatMap((x) => Object.keys(x.actions)))];
  const actionLabels: Record<string, string> = {
    attack: 'Normal attack',
    skill: 'Skill',
    burst: 'Ultimate',
    special: 'Special (G)',
    dash: 'Dash',
    invoke: 'Other ability',
  };
  const actionColors = ['#ca7ead', '#e3d64a', '#ed8c3a', '#59abd8', '#a4cc42', '#38ba9f'];
  const reactionColors: Record<string, string> = {
    Blossom: '#62d8b1',
    Hexed: '#f45e88',
    Scorch: '#ed8c3a',
    Nova: '#a992e8',
    Stain: '#64c4e5',
    Remora: '#ecdb68',
    Charge: '#eadb37',
    Discord: '#ca7ead',
  };
  const sources = new Map<string, { name: string; values: number[] }>();
  for (const source of a.energy_sources) {
    const details = energySourceDetails(source.source, source.kind);
    const key = JSON.stringify([source.source, details.label]);
    const row = sources.get(key) ?? {
      name: details.label,
      values: a.actors.map(() => 0),
    };
    const recipient = a.actors.findIndex((actor) => actor.slot === source.actor_index);
    if (recipient >= 0) row.values[recipient] += source[energyField].mean;
    sources.set(key, row);
  }
  const hasReactions = Boolean(a.reactions && a.reactions.length > 0);
  return (
    <>
      <div className={hasReactions ? 'three-column' : 'two-column'}>
        <Panel title="Actions">
          <div id="action-chart">
            <StackedBars
              rows={a.actors.map((x) => ({
                name: english.character(x.character_id),
                values: actionTypes.map((type) => x.actions[type]?.mean || 0),
              }))}
              series={actionTypes.map((type, i) => ({
                name: actionLabels[type] || type,
                color: actionColors[i % actionColors.length],
              }))}
              unit="admitted actions"
            />
          </div>
        </Panel>
        <Panel
          title="Field Time Distribution"
          controls={
            <button
              type="button"
              className="time-basis-toggle"
              aria-label="Field time distribution time basis"
              onClick={() =>
                onTimeBasisChange?.(timeBasis === 'relative' ? 'absolute' : 'relative')
              }
            >
              {timeBasis === 'relative' ? 'relative' : 'absolute'}
            </button>
          }
        >
          <div id="field-time">
            <PieChart
              rows={a.actors.map((x) => ({
                name: english.character(x.character_id),
                value: x.field[timeBasis]?.mean ?? 0,
                color: palette[x.slot % palette.length],
              }))}
              label="seconds"
            />
          </div>
        </Panel>
        {hasReactions && (
          <Panel title="Esper Cycles">
            <div id="esper-cycles">
              <BarChart
                rows={a.reactions.map((r, i) => ({
                  name: r.name,
                  value: r.mean,
                  min: r.min,
                  max: r.max,
                  std: r.std,
                  color: reactionColors[r.name] || palette[i % palette.length],
                }))}
                unit="reactions"
              />
            </div>
          </Panel>
        )}
      </div>
      <Panel
        title="Energy Per Source"
        controls={
          <Select
            id="energy-type"
            label="Accounting"
            value={field}
            onChange={setField}
            options={[
              ['energy', 'Gross granted'],
              ['effective', 'Retained'],
              ['overflow', 'Overflow'],
            ]}
          />
        }
      >
        <div id="energy-sources">
          <div className="energy-accounting">
            <span>Gross {format(total('energy'), 2)}</span>
            <span>Retained {format(total('effective'), 2)}</span>
            <span>Overflow {format(total('overflow'), 2)}</span>
          </div>
          <StackedBars
            rows={[...sources.values()].sort(
              (x, y) => y.values.reduce((n, v) => n + v, 0) - x.values.reduce((n, v) => n + v, 0),
            )}
            series={a.actors.map((x) => ({
              name: english.character(x.character_id),
              color: palette[x.slot % palette.length],
            }))}
            unit="energy"
          />
        </div>
      </Panel>
      <Panel title="Character Energy at End of Simulation">
        <div id="ending-energy">
          <BarChart
            rows={a.actors.map((x) => ({
              name: english.character(x.character_id),
              value: x.energy.mean,
              min: x.energy.min,
              max: x.energy.max,
              std: x.energy.std,
              color: palette[x.slot % palette.length],
            }))}
            unit="energy"
            max={Math.max(...request.actors.map((x) => x.energy_capacity))}
          />
        </div>
      </Panel>
    </>
  );
}
