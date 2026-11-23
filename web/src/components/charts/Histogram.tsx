import { distributionModel } from './models';
import { ChartInspector } from './ChartInspector';
import { format } from '../ui/format';
export function Histogram({
  samples,
  statKey = 'dps',
  timeBasis = 'relative',
}: {
  samples: readonly object[];
  statKey?: string;
  timeBasis?: 'relative' | 'absolute';
}) {
  const m = distributionModel(samples, statKey, timeBasis);
  if (!m) return <p className="empty-chart">Distribution unavailable for this seed set.</p>;
  const top = Math.max(...m.bins.map((b) => b.count)),
    step = 10 ** Math.floor(Math.log10(Math.max(1, top / 5))),
    tick = Math.max(
      1,
      [1, 2, 2.5, 5, 10].map((v) => v * step).find((v) => v >= top / 5)!,
    ),
    ymax = Math.ceil(top / tick) * tick;
  const x = (v: number) => 65 + ((v - m.low) / (m.high - m.low)) * 500,
    y = (v: number) => 218 - (v / ymax) * 165,
    unit =
      (
        {
          dps: 'DPS',
          eps: 'EPS',
          hps: 'HPS',
          shp: 'SHP',
          duration: 'seconds',
          hits: 'instances',
        } as Record<string, string>
      )[statKey] || statKey;
  const rows = Array.from({ length: Math.round(ymax / tick) + 1 }, (_, i) => i * tick);
  return (
    <>
      <ChartInspector className="histogram-shell">
        <svg
          className="distribution-svg"
          viewBox="0 0 600 290"
          role="img"
          aria-label={`${unit} distribution: ${m.count} trials, ${m.unique} distinct outcomes`}
        >
          <text className="axis-label" x="315" y="20" textAnchor="middle">
            {unit} Distribution
          </text>
          <text
            className="axis-label"
            transform="translate(17 146) rotate(-90)"
            textAnchor="middle"
          >
            Number of iterations
          </text>
          {rows.map((v) => (
            <g key={v}>
              <line className="grid-line" x1="65" x2="565" y1={y(v)} y2={y(v)} />
              <text className="axis-label" x="57" y={y(v) + 4} textAnchor="end">
                {v}
              </text>
            </g>
          ))}
          {m.bins.map((b, i) => {
            const tip = `${b.count} of ${m.count} completed trials (${format((b.count / m.count) * 100, 2)}%)\n${m.constant ? format(m.min, 3) : format(b.low, 3) + ' – ' + format(b.high, 3)} ${unit}`;
            return (
              <rect
                className="hist-bar"
                key={i}
                tabIndex={0}
                data-chart-tip={tip}
                aria-label={tip}
                x={m.constant ? x(m.min) - 18 : x(b.low) + 0.5}
                y={y(b.count)}
                width={m.constant ? 36 : x(b.high) - x(b.low) - 1}
                height={218 - y(b.count)}
              >
                <title>{tip}</title>
              </rect>
            );
          })}
          <line className="hist-mean" x1={x(m.mean)} x2={x(m.mean)} y1="36" y2="218" />
          {[0, 0.25, 0.5, 0.75, 1].map((f) => (
            <text className="axis-label" key={f} x={65 + f * 500} y="240" textAnchor="middle">
              {format(m.low + (m.high - m.low) * f, 2)}
            </text>
          ))}
          <line className="hist-box" x1={x(m.min)} x2={x(m.max)} y1="265" y2="265" />
          <rect
            className="hist-box"
            x={x(m.p25)}
            y="260"
            width={Math.max(1, x(m.p75) - x(m.p25))}
            height="10"
          />
          {[m.min, m.p50, m.max].map((v, i) => (
            <line className="hist-box" key={i} x1={x(v)} x2={x(v)} y1="260" y2="270" />
          ))}
        </svg>
      </ChartInspector>
      <div className="distribution-summary">
        <span>mean {format(m.mean, 2)} (dashed)</span>
        <span>median {format(m.p50, 2)}</span>
        <span>
          IQR {format(m.p25, 2)} – {format(m.p75, 2)}
        </span>
      </div>
    </>
  );
}
