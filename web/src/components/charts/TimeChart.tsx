import { useEffect, useRef, useState, type CSSProperties } from 'react';
import type { TimelinePoint } from '../../domain/contracts/trials';
import type { Statistics } from '../../domain/reports/statistics';
import { format } from '../ui/format';
import { ChartInspector } from './ChartInspector';

const colors = {
  min: '#6eafe2',
  max: '#c5a65c',
  mean: '#eadb37',
  p25: '#c7a6d8',
  p50: '#eadb37',
  p75: '#65cdb6',
};
const axisNumber = (v: number) =>
  v >= 1e6 ? format(v / 1e6, 1) + 'M' : v >= 1e3 ? format(v / 1e3, 1) + 'k' : format(v, 1);

export function TimeChart({
  points,
  horizon,
  cumulative = false,
  statKey = 'dps',
  label = 'Damage over time',
}: {
  points: TimelinePoint[];
  horizon: number;
  cumulative?: boolean;
  statKey?: 'dps' | 'cumulative';
  label?: string;
}) {
  const container = useRef<HTMLDivElement>(null);
  const [width, setWidth] = useState(900);
  useEffect(() => {
    const observer = new ResizeObserver(([entry]) => {
      if (entry.contentRect.width > 0) setWidth(Math.max(280, entry.contentRect.width));
    });
    if (container.current) observer.observe(container.current);
    return () => observer.disconnect();
  }, []);
  const stats = points.map((p) => p[statKey]);
  if (!points.length || horizon <= 0)
    return (
      <div ref={container}>
        <p className="empty-chart">No delivered observations for this selection.</p>
      </div>
    );
  const max = Math.max(1, ...stats.map((s) => s.max));
  const x = (t: number) => 58 + (t / horizon) * (width - 76);
  const y = (v: number) => 222 - (v / max) * 194;
  const line = (value: (s: Statistics) => number) =>
    points.map((p, i) => `${x(p.time)},${y(value(stats[i]))}`).join(' ');
  const values = (key: keyof typeof colors) =>
    (cumulative ? `${x(0)},${y(0)} ` : '') + line((s) => s[key]);
  const band =
    line((s) => Math.min(s.max, s.mean + s.std)) +
    ' ' +
    points
      .map((p, i) => `${x(p.time)},${y(Math.max(stats[i].min, stats[i].mean - stats[i].std))}`)
      .reverse()
      .join(' ');
  const series: (keyof typeof colors)[] = cumulative
    ? ['min', 'max', 'p25', 'p50', 'p75']
    : ['min', 'max', 'mean'];
  const ticks = width >= 700 ? 10 : 4;
  return (
    <div ref={container}>
      <ChartInspector>
        <div className="time-legend">
          {series.map((key) => (
            <span key={key} style={{ '--line': colors[key] } as CSSProperties}>
              {key}
            </span>
          ))}
          {!cumulative && <span className="legend-std">± std</span>}
        </div>
        <svg className="time-chart" viewBox={`0 0 ${width} 280`} role="img" aria-label={label}>
          {[0, 0.25, 0.5, 0.75, 1].map((f) => (
            <g key={f}>
              <line className="grid-line" x1="58" x2={width - 18} y1={y(max * f)} y2={y(max * f)} />
              <text className="axis-label" x="50" y={y(max * f) + 4} textAnchor="end">
                {axisNumber(max * f)}
              </text>
            </g>
          ))}
          {Array.from({ length: ticks + 1 }, (_, i) => i / ticks).map((f) => (
            <g key={f}>
              <line
                className="grid-line"
                x1={x(horizon * f)}
                x2={x(horizon * f)}
                y1="28"
                y2="222"
              />
              <text className="axis-label" x={x(horizon * f)} y="242" textAnchor="middle">
                {format(horizon * f, 1)}
              </text>
            </g>
          ))}
          <text className="axis-label" x={(width + 40) / 2} y="267" textAnchor="middle">
            Simulation time (seconds)
          </text>
          <text
            className="axis-label"
            transform="translate(12 125) rotate(-90)"
            textAnchor="middle"
          >
            {cumulative ? 'Damage' : 'DPS'}
          </text>
          {!cumulative && <polygon className="std-band" points={band} />}
          {series.map((key) => (
            <polyline
              key={key}
              className={`stat-line stat-${key}`}
              points={values(key)}
              stroke={colors[key]}
            />
          ))}
          {points.map((p, i) => {
            const s = stats[i];
            const tip = `${format(p.time, 3)}s · ${label}\nmean ${format(s.mean, 2)}\nmin ${format(s.min, 2)} · max ${format(s.max, 2)}\nstd ${format(s.std, 2)}\np25 ${format(s.p25, 2)} · p50 ${format(s.p50, 2)} · p75 ${format(s.p75, 2)}`;
            return (
              <circle
                className="chart-point"
                key={i}
                cx={x(p.time)}
                cy={y(cumulative ? s.p50 : s.mean)}
                r="6"
                tabIndex={0}
                data-chart-tip={tip}
                aria-label={tip}
              >
                <title>{tip}</title>
              </circle>
            );
          })}
        </svg>
      </ChartInspector>
    </div>
  );
}
