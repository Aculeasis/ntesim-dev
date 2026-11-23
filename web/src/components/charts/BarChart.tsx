import type { CSSProperties } from 'react';
import { format, palette } from '../ui/format';
import { barScale, type BarRow } from './models';
export function BarChart({
  rows,
  unit = 'DPS',
  max = null,
}: {
  rows: BarRow[];
  unit?: string;
  max?: number | null;
}) {
  const scale = barScale(rows, max);
  if (!rows.length)
    return <p className="empty-chart">No delivered observations for this selection.</p>;
  return (
    <div className="bar-chart">
      {rows.map((r, i) => (
        <div className="chart-row" key={`${r.name}:${i}`}>
          <span className="chart-name" title={r.name}>
            {r.name}
          </span>
          <div
            className="bar-space"
            tabIndex={0}
            aria-label={`${r.name} ${format(r.value, 2)} ${unit}`}
          >
            <div
              className="chart-bar"
              style={
                {
                  width: `${Math.max(0, (r.value / scale) * 100)}%`,
                  '--series': r.color || palette[i % 4],
                } as CSSProperties
              }
            />
            {r.min !== undefined && r.max !== undefined && r.max > r.min && (
              <span
                className="bar-error"
                style={{
                  left: `${(r.min / scale) * 100}%`,
                  width: `${((r.max - r.min) / scale) * 100}%`,
                }}
              />
            )}
            <span className="chart-tooltip">
              <strong>
                {r.name} {unit}
              </strong>
              <br />
              mean {format(r.value, 2)}
              {r.min !== undefined && (
                <>
                  <br />
                  min {format(r.min, 2)}
                  <br />
                  max {format(r.max, 2)}
                  <br />
                  std {format(r.std, 2)}
                </>
              )}
              {r.detail && (
                <>
                  <br />
                  {r.detail}
                </>
              )}
            </span>
          </div>
          <span className="chart-value">{format(r.value, r.value < 100 ? 2 : 0)}</span>
        </div>
      ))}
      <div className="bar-axis">
        {[0, 0.25, 0.5, 0.75, 1].map((f) => (
          <span key={f}>{format(scale * f, 2)}</span>
        ))}
      </div>
    </div>
  );
}
