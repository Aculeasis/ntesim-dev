import type { TrialResult } from '../../domain/contracts/trials';
import { format } from '../../components/ui/format';
import { MetricCard } from './MetricCard';
export function Summary({
  result,
  timeBasis = 'relative',
  onTimeBasisChange,
}: {
  result: TrialResult;
  timeBasis?: 'relative' | 'absolute';
  onTimeBasisChange?: (basis: 'relative' | 'absolute') => void;
}) {
  const { request, aggregate: a } = result;
  const eventCount =
    result.samples?.[0]?.events ??
    result.report?.total_events_processed ??
    Math.round(a.metrics?.events?.mean ?? 0);
  const durationText =
    request.stop === 'victory' && a.metrics?.duration?.mean
      ? format(a.metrics.duration.mean, 1) + 's'
      : request.relative_duration_us / 1e6 + 's';
  const metadata = [
    ['iterations', `${a.completed} / ${a.requested}`],
    ['duration', durationText],
    ['seed', request.seed + '…' + (request.seed + a.completed - 1)],
    ['targets', request.targets.length],
    ['events', eventCount],
  ];
  return (
    <>
      <div className="metadata" id="result-meta">
        {metadata.map(([k, v]) => (
          <span key={k}>
            {k} <b>{v}</b>
          </span>
        ))}
      </div>
      <div id="rollup-cards" className="rollup-grid">
        <MetricCard
          label="Damage Per Second (DPS)"
          stat={
            timeBasis === 'relative'
              ? (a.metrics.dps?.relative ?? null)
              : (a.metrics.dps?.absolute ?? null)
          }
          color="#dc512d"
          control={
            <button
              type="button"
              className="time-basis-toggle"
              aria-label="Damage per second time basis"
              onClick={() =>
                onTimeBasisChange?.(timeBasis === 'relative' ? 'absolute' : 'relative')
              }
            >
              {timeBasis === 'relative' ? 'relative' : 'absolute'}
            </button>
          }
        />
        <MetricCard
          label="Effective Energy Per Second (EPS)"
          stat={
            timeBasis === 'relative'
              ? (a.metrics.eps?.relative ?? null)
              : (a.metrics.eps?.absolute ?? null)
          }
          color="#3993d5"
        />
        <MetricCard label="Damage" stat={a.metrics.damage} color="#aa439f" />
        <MetricCard
          label="Healing Per Second (HPS)"
          stat={
            timeBasis === 'relative'
              ? (a.metrics.hps?.relative ?? null)
              : (a.metrics.hps?.absolute ?? null)
          }
          color="#62d8b1"
          unit="HP/s"
        />
        <MetricCard
          label="Effective Shield HP (SHP)"
          stat={
            timeBasis === 'relative'
              ? (a.metrics.shp?.relative ?? null)
              : (a.metrics.shp?.absolute ?? null)
          }
          color="#ecdb68"
          unit="HP"
        />
        <MetricCard
          label="Sim Duration (Dur)"
          stat={timeBasis === 'relative' ? a.metrics.duration : a.metrics.absolute_duration}
          color="#1bac9f"
          unit="s"
        />
      </div>
    </>
  );
}
