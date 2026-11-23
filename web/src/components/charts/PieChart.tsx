import { format, palette } from '../ui/format';
import type { BarRow } from './models';
import { ChartInspector } from './ChartInspector';

export function PieChart({ rows, label }: { rows: BarRow[]; label: string }) {
  const total = rows.reduce((n, r) => n + r.value, 0);
  if (total <= 0)
    return <p className="empty-chart">No delivered observations for this selection.</p>;
  let angle = -Math.PI / 2;
  return (
    <div className="pie-layout">
      <ChartInspector className="pie-shell">
        <svg
          className="pie-chart"
          viewBox="0 0 180 180"
          role="img"
          aria-label={`${label} distribution`}
        >
          {rows.map((row, i) => {
            if (row.value <= 0) return null;
            const share = row.value / total,
              start = angle;
            angle += share * 2 * Math.PI;
            const tip = `${row.name}: ${format(share * 100, 1)}%\n${format(row.value, 2)} ${label}`;
            const props = {
              className: 'pie-segment',
              fill: row.color || palette[i % palette.length],
              tabIndex: 0,
              'data-chart-tip': tip,
              'aria-label': tip,
            };
            return share >= 1 ? (
              <circle key={i} cx="90" cy="90" r="74" {...props}>
                <title>{tip}</title>
              </circle>
            ) : (
              <path
                key={i}
                d={`M90,90 L${90 + 74 * Math.cos(start)},${90 + 74 * Math.sin(start)} A74,74 0 ${share > 0.5 ? 1 : 0},1 ${90 + 74 * Math.cos(angle)},${90 + 74 * Math.sin(angle)} Z`}
                {...props}
              >
                <title>{tip}</title>
              </path>
            );
          })}
        </svg>
      </ChartInspector>
      <div className="chart-legend">
        {rows.map((r, i) => (
          <div
            className="legend-row"
            key={i}
            tabIndex={0}
            title={`${r.name}: ${format(r.value, 2)} ${label}`}
          >
            <i style={{ background: r.color || palette[i % palette.length] }} />
            <span>{r.name}</span>
            <b>{format((r.value / total) * 100, 1)}%</b>
          </div>
        ))}
      </div>
    </div>
  );
}
