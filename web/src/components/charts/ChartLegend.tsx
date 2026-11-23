import type { Series } from './models';

export function ChartLegend({ series, label }: { series: Series[]; label: string }) {
  return (
    <div className="stack-legend" role="list" aria-label={label}>
      {series.map((entry, i) => (
        <span key={i} role="listitem">
          <i style={{ background: entry.color }} />
          {entry.name}
        </span>
      ))}
    </div>
  );
}
