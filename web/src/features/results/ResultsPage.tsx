import { useState } from 'react';
import type { SimulationState } from '../simulation/useSimulation';
import { Panel, Select } from '../../components/ui/Panel';
import { Histogram } from '../../components/charts/Histogram';
import { TeamCards } from '../team/TeamCards';
import { Targets } from './Targets';
import { Summary } from './Summary';
import { DamagePanels } from './DamagePanels';
import { EnergyPanels } from './EnergyPanels';
import { ComparisonPanel } from './ComparisonPanel';
import type { ExportFile } from '../import-export/ExportDialog';
export function ResultsPage({
  run,
  onExportFile,
  onBusy,
  busy,
}: {
  run: SimulationState;
  onExportFile: (file: ExportFile) => void;
  onBusy: (busy: boolean) => void;
  busy: boolean;
}) {
  const [distribution, setDistribution] = useState('dps'),
    [basisByRun, setBasisByRun] = useState<Record<number | string, 'relative' | 'absolute'>>({}),
    [sourceModeByRun, setSourceModeByRun] = useState<Record<number | string, 'full' | 'comp'>>({}),
    result = run.result;
  if (!result) return null;
  const timeBasis = basisByRun[run.runId] ?? 'relative';
  const setTimeBasis = (basis: 'relative' | 'absolute') =>
    setBasisByRun((prev) => ({ ...prev, [run.runId]: basis }));
  const sourceMode = sourceModeByRun[run.runId] ?? 'full';
  const setSourceMode = (mode: 'full' | 'comp') =>
    setSourceModeByRun((prev) => ({ ...prev, [run.runId]: mode }));
  const { aggregate: a, request } = result;
  const details = {
    quality: a.quality,
    game_dps_computed: a.game_dps_computed,
    completed: a.completed,
    requested: a.requested,
    elapsed_ms: a.elapsed_ms,
    trials_per_second: a.trials_per_second,
    first_seed: result.report.seed,
    engine_source_sha256: result.report.engine_source_sha256,
    exact_first_trial_damage: result.report.total_damage_exact,
    model_assumptions: result.report.model_assumptions || {},
    resolved_model_assumptions: result.report.resolved_model_assumptions || {},
    evidence_catalog: result.report.evidence_catalog || [],
    metadata_catalog: result.report.metadata_catalog || [],
    action_metadata_ledger: result.report.action_metadata_ledger || [],
    hit_metadata: result.report.hit_receipts.map((hit) => ({
      time_us: hit.time_us,
      actor_index: hit.actor_index,
      target_index: hit.target_index,
      ability_id: hit.ability_id,
      damage_effect_id: hit.damage_effect_id,
      metadata_id: hit.metadata_id,
      metadata: result.report.metadata_catalog?.[hit.metadata_id],
    })),
    final_target_states: result.report.final_target_states,
    warnings: result.warnings,
    occurrences: result.occurrences,
  };
  return (
    <div className="simulation-report">
      <div id="report-capture" className="simulation-report">
        <div className="team-grid result-team" id="result-team">
          <TeamCards party={result.party} />
        </div>
        <Summary
          key={run.runId}
          result={result}
          timeBasis={timeBasis}
          onTimeBasisChange={setTimeBasis}
        />
        <div className="two-column report-overview">
          <Panel title={request.targets.length >= 3 ? undefined : 'Target Information'}>
            <div id="target-information">
              <Targets targets={request.targets} />
            </div>
          </Panel>
          <Panel
            title="Distribution"
            controls={
              <Select
                id="distribution-type"
                label="Type"
                value={distribution}
                onChange={setDistribution}
                options={[
                  ['dps', 'DPS'],
                  ['eps', 'EPS'],
                  ['hps', 'HPS'],
                  ['shp', 'SHP'],
                  ['duration', 'Duration'],
                  ['hits', 'Damage instances'],
                ]}
              />
            }
          >
            <div id="distribution-chart">
              <Histogram samples={result.samples} statKey={distribution} timeBasis={timeBasis} />
            </div>
          </Panel>
        </div>
        <DamagePanels
          result={result}
          timeBasis={timeBasis}
          sourceMode={sourceMode}
          onSourceModeChange={setSourceMode}
        />
        <EnergyPanels result={result} timeBasis={timeBasis} onTimeBasisChange={setTimeBasis} />
      </div>
      <ComparisonPanel
        key={run.runId}
        baseline={result.compiled.input.source}
        controller={run.controller}
        onBusy={onBusy}
        disabled={busy}
        onExport={onExportFile}
      />
      <details className="panel">
        <summary>Run details</summary>
        <pre id="report-details">{JSON.stringify(details, null, 2)}</pre>
      </details>
    </div>
  );
}
