import type { FailedSourceTrial } from '../../domain/contracts/source';
import type { SimulationController } from '../../infrastructure/worker/controller';
import type { ExportFile } from '../import-export/ExportDialog';
import { SourceView } from '../playback/SourceView';
import { ReportTools } from './ReportTools';
import { JsonDetails } from '../../components/ui/JsonDetails';

export function FailureReport({
  failure,
  controller,
  busy,
  onBusy,
  onExport,
}: {
  failure: FailedSourceTrial;
  controller: () => SimulationController;
  busy: boolean;
  onBusy: (busy: boolean) => void;
  onExport: (file: ExportFile) => void;
}) {
  const e = failure.execution;
  return (
    <section className="panel" aria-label="Failed trial report">
      <h2>Failed seed {e.report.seed} · excluded from statistics</h2>
      <p>
        {e.diagnostic?.code} · {e.diagnostic?.message ?? e.report.halt_diagnostic}
      </p>
      <SourceView execution={e} event={null} />
      <JsonDetails
        title="Actual failed request, diagnostic, recorded state and receipts"
        value={e}
      />
      <ReportTools
        compiled={failure.compiled}
        execution={e}
        controller={controller}
        disabled={busy}
        onBusy={onBusy}
        onExport={onExport}
      />
    </section>
  );
}
