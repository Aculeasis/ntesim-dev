import { format, palette } from '../ui/format';
import type { Series } from './models';
export function StackedBars({
  rows,
  series,
  unit = 'DPS',
}: {
  rows: { name: string; values: number[] }[];
  series: Series[];
  unit?: string;
}) {
  const max = Math.max(1, ...rows.map((r) => r.values.reduce((n, v) => n + v, 0)));
  if (!rows.length)
    return <p className="empty-chart">No delivered observations for this selection.</p>;
  return (
    <>
      <div className="stack-legend">
        {series.map((s, i) => (
          <span key={i}>
            <i style={{ background: s.color || palette[i % 4] }} />
            {s.name}
          </span>
        ))}
      </div>
      <div className="stacked-chart">
        {rows.map((r, index) => (
          <div className="chart-row" key={index}>
            <span className="chart-name" title={r.name}>
              {r.name}
            </span>
            <div className="stack-space">
              {r.values.map((v, i) => (
                <div
                  className="stack-segment"
                  key={i}
                  style={{
                    width: `${(v / max) * 100}%`,
                    background: series[i].color || palette[i % 4],
                  }}
                  tabIndex={v ? 0 : undefined}
                  aria-label={`${r.name} ${series[i].name} ${format(v, 2)} ${unit}`}
                >
                  <span className="chart-tooltip">
                    <strong>{r.name}</strong>
                    <br />
                    {series[i].name}: {format(v, 2)} {unit}
                    <br />
                    Row total:{' '}
                    {format(
                      r.values.reduce((n, v) => n + v, 0),
                      2,
                    )}{' '}
                    {unit}
                  </span>
                </div>
              ))}
            </div>
            <span className="chart-value">
              {format(
                r.values.reduce((n, v) => n + v, 0),
                2,
              )}
            </span>
          </div>
        ))}
        <div className="bar-axis">
          {[0, 0.25, 0.5, 0.75, 1].map((f) => (
            <span key={f}>{format(max * f, 2)}</span>
          ))}
        </div>
      </div>
    </>
  );
}
