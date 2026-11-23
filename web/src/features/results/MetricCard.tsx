import type { CSSProperties, ReactNode } from 'react';
import type { Statistics } from '../../domain/reports/statistics';
import { format } from '../../components/ui/format';
export function MetricCard({
  label,
  stat,
  color,
  unit = '',
  note = '',
  control,
}: {
  label: string;
  stat: Statistics | null;
  color: string;
  unit?: string;
  note?: string;
  control?: ReactNode;
}) {
  return (
    <article className="rollup-card" style={{ '--metric': color } as CSSProperties}>
      <div className="rollup-heading">
        <h3>{label}</h3>
        {control}
      </div>
      <div className="rollup-value">
        {stat ? format(stat.mean, stat.mean < 100 ? 2 : 0) : '—'}
        <small>{unit}</small>
      </div>
      {stat ? (
        <div className="stat-grid">
          {(['min', 'max', 'std', 'p25', 'p50', 'p75'] as const).map((k) => (
            <span key={k}>
              {k}
              <b>{format(stat[k], stat[k] < 100 ? 2 : 0)}</b>
            </span>
          ))}
        </div>
      ) : (
        <p className="unavailable">Not available in this partial model</p>
      )}
      {note && <p className="metric-note">{note}</p>}
    </article>
  );
}
