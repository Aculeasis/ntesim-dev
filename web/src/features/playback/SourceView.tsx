import { useEffect, useRef, useState } from 'react';
import type { SourceExecution } from '../../domain/contracts/source';
import type { ReplayEvent } from '../../domain/replay/events';
import { textareaRange } from '../editor/CodeEditor';
import { Panel } from '../../components/ui/Panel';
import { eventLocation } from '../../domain/replay/source';
import type { SourceLocation } from '../../domain/contracts/script';

export function SourceView({
  execution,
  event,
}: {
  execution: SourceExecution;
  event: ReplayEvent | null;
}) {
  const area = useRef<HTMLTextAreaElement>(null);
  const [selected, setSelected] = useState<SourceLocation | null>(null);
  useEffect(() => setSelected(null), [event, execution]);
  const location = selected ?? eventLocation(execution, event) ?? execution.diagnostic?.loc ?? null;
  useEffect(() => {
    if (!location || !area.current) return;
    area.current.setSelectionRange(...textareaRange(execution.source, location));
    area.current.scrollTop = Math.max(0, (location.line - 3) * 20);
  }, [execution, location]);
  return (
    <Panel title="Executed source">
      <p className="chart-note">
        Seed {execution.report.seed} ·{' '}
        {location
          ? `Line ${location.line}, column ${location.column}`
          : 'Select an authored event or occurrence.'}
      </p>
      <label className="source-occurrence">
        Source occurrence{' '}
        <select
          aria-label="Source occurrence"
          value=""
          onChange={(e) => {
            const occurrence = execution.occurrences[Number(e.target.value)];
            if (!occurrence) return;
            // Occurrence navigation also covers waits and control statements.
            setSelected(occurrence.loc);
          }}
        >
          <option value="" disabled>
            Select an executed occurrence
          </option>
          {execution.occurrences.map((o, i) => (
            <option key={o.index} value={i}>
              Line {o.loc.line}:{o.loc.column} · {o.kind} · {o.started_us} µs
            </option>
          ))}
        </select>
      </label>
      <button
        disabled={!location}
        onClick={() => {
          area.current?.focus();
          if (location)
            area.current?.setSelectionRange(...textareaRange(execution.source, location));
        }}
      >
        Select source range
      </button>
      <textarea
        ref={area}
        aria-label="Executed source"
        value={execution.source}
        readOnly
        spellCheck={false}
        rows={10}
        style={{ width: '100%', lineHeight: '20px', fontFamily: 'var(--mono)' }}
      />
    </Panel>
  );
}
