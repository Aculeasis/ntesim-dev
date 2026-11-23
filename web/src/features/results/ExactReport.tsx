import { useState } from 'react';
import type { ExactFraction } from '../../domain/contracts/ratio';
import type { ReportBundle } from '../../domain/contracts/source-report';
import type { ExportFile } from '../import-export/ExportDialog';
import { Panel } from '../../components/ui/Panel';
import { JsonDetails } from '../../components/ui/JsonDetails';

const exact = (v: ExactFraction | null) => v ?? 'Unavailable';
const damage = (v: ExactFraction) => BigInt(v.split('/')[0]).toLocaleString();
export function ExactReport({
  bundle,
  onExport,
}: {
  bundle: ReportBundle;
  onExport: (file: ExportFile) => void;
}) {
  const [table, setTable] = useState('damage.csv');
  const r = bundle.report;
  return (
    <Panel title="Exact sample report">
      <p className="chart-note">
        Seed {r.execution.report.seed} ·{' '}
        {r.execution.report.execution_completed ? 'Completed' : 'Incomplete'}
      </p>
      <dl className="report-identities">
        <dt>Source SHA256</dt>
        <dd>{r.source_sha256}</dd>
        <dt>Request SHA256</dt>
        <dd>{r.request_sha256}</dd>
        <dt>Input SHA256</dt>
        <dd>{r.input_sha256}</dd>
        <dt>Engine SHA256</dt>
        <dd>{r.engine_source_sha256}</dd>
      </dl>
      <p>
        Damage {damage(r.damage.total.exact)} · HP loss {r.damage.total.applied_hp_loss} · overkill{' '}
        {r.damage.total.overkill} · DPS {exact(r.damage.total.dps)}
      </p>
      <div className="results-toolbar">
        <button
          id="export-native-json"
          onClick={() =>
            onExport({
              name: `nte-sample-${r.execution.report.seed}.json`,
              type: 'application/json',
              text: bundle.report_json,
            })
          }
        >
          Exact JSON
        </button>
        <label>
          CSV table{' '}
          <select aria-label="CSV table" value={table} onChange={(e) => setTable(e.target.value)}>
            {Object.keys(bundle.csv_tables).map((name) => (
              <option key={name}>{name}</option>
            ))}
          </select>
        </label>
        <button
          id="export-native-csv"
          onClick={() =>
            onExport({ name: table, type: 'text/csv', text: bundle.csv_tables[table] })
          }
        >
          Exact CSV
        </button>
      </div>
      <div className="timeline-container">
        <table className="timeline-table" aria-label="Exact native contributions">
          <thead>
            <tr>
              {[
                'Actor',
                'Target',
                'Ability / effect',
                'Field',
                'Hits',
                'Damage',
                'HP loss',
                'Overkill',
                'DPS',
              ].map((label) => (
                <th key={label}>{label}</th>
              ))}
            </tr>
          </thead>
          <tbody>
            {r.damage.contributions.map((c, i) => (
              <tr key={i}>
                <td>{c.actor_index + 1}</td>
                <td>{c.target_index + 1}</td>
                <td>
                  {c.ability_id} / {c.damage_effect_id}
                </td>
                <td>{c.on_field ? 'On' : 'Off'}</td>
                <td>{c.totals.hits}</td>
                <td>{damage(c.totals.exact)}</td>
                <td>{c.totals.applied_hp_loss}</td>
                <td>{c.totals.overkill}</td>
                <td>{exact(c.totals.dps)}</td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <JsonDetails
        title="Input time, healing, state, assumptions and raw receipts"
        value={r.execution}
      />
    </Panel>
  );
}
