import { useState } from 'react';
import type { CompiledSourceProgram } from '../../domain/contracts/script';
import type { SourceExecution } from '../../domain/contracts/source';
import type { ReportBundle } from '../../domain/contracts/source-report';
import type { SimulationController } from '../../infrastructure/worker/controller';
import type { ExportFile } from '../import-export/ExportDialog';
import { ExactReport } from './ExactReport';
import { errorMessage } from '../../components/ui/format';
import { sameObservation } from '../../domain/reports/identity';

export function ReportTools({
  compiled,
  execution,
  controller,
  disabled,
  onBusy,
  onExport,
}: {
  compiled: CompiledSourceProgram;
  execution: SourceExecution;
  controller: () => SimulationController;
  disabled: boolean;
  onBusy: (busy: boolean) => void;
  onExport: (file: ExportFile) => void;
}) {
  const [bundle, setBundle] = useState<ReportBundle | null>(null),
    [busy, setBusy] = useState(false),
    [status, setStatus] = useState(''),
    [error, setError] = useState('');
  const perform = async (work: () => Promise<void>) => {
    if (disabled || busy) return;
    setBusy(true);
    onBusy(true);
    setStatus('');
    setError('');
    try {
      await work();
    } catch (e) {
      setError(errorMessage(e));
    } finally {
      setBusy(false);
      onBusy(false);
    }
  };
  return (
    <>
      <div className="panel report-tools">
        <button
          id="prepare-exact-report"
          disabled={disabled || busy}
          onClick={() =>
            void perform(async () => {
              const next = await controller().execute('EXPORT_SOURCE_REPORT', {
                compiled,
                seed: execution.report.seed,
              });
              if (!sameObservation(next.report.execution, execution))
                throw new Error(
                  'Exact report differs from the selected executed source/request/receipts.',
                );
              setBundle(next);
              setStatus('Report ready.');
            })
          }
        >
          Prepare exact report &amp; CSV
        </button>
        <label>
          {' '}
          Verify saved native report (up to 64 MiB)
          <input
            id="verify-native-report"
            type="file"
            accept=".json"
            disabled={disabled || busy}
            onChange={(e) => {
              const file = e.target.files?.[0];
              e.target.value = '';
              if (file)
                void perform(async () => {
                  if (file.size > 64 * 1024 * 1024)
                    throw new Error('Native report exceeds 64 MiB.');
                  const result = await controller().execute('REPLAY_SOURCE_REPORT', {
                    report_json: await file.text(),
                  });
                  setStatus(
                    `Reconciled ${result.source_sha256} · ${result.execution_completed ? 'Completed' : 'Incomplete'}`,
                  );
                });
            }}
          />
        </label>
        {busy && (
          <>
            <p role="status">Verifying source/native replay…</p>
            <button onClick={() => controller().cancel()}>Cancel report task</button>
          </>
        )}
        {status && <p role="status">{status}</p>}
        {error && <p role="alert">{error}</p>}
      </div>
      {bundle && <ExactReport bundle={bundle} onExport={onExport} />}
    </>
  );
}
