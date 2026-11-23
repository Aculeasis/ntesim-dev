import type { Playback } from './usePlayback';
import { Panel } from '../../components/ui/Panel';
import { english } from '../../domain/localization';
import { resolveBuffRefs } from '../../domain/reports/timeline-fold';
import { JsonDetails } from '../../components/ui/JsonDetails';

export function StatePanel({ playback: p }: { playback: Playback }) {
  const n = p.state.native;
  const timers = n.timers.map((timer) => {
    const definition = p.model.report.timer_catalog![timer.timer_def_id];
    return {
      ...definition,
      remaining_us: Math.max(
        0,
        timer.expires_at_us - p.model.clocks.elapsedAt(definition.clock_basis, p.state.time),
      ),
    };
  });
  const buffs = n.buffs.map((ref) => {
    const definition = p.model.report.buff_catalog![ref.buff_def_id];
    return {
      ...resolveBuffRefs([ref], p.model.report.buff_catalog)[0],
      remaining_us:
        ref.duration.kind === 'timed'
          ? Math.max(
              0,
              ref.duration.expires_at_us -
                p.model.clocks.elapsedAt(definition.clock_basis, p.state.time),
            )
          : null,
    };
  });
  return (
    <Panel title="Combat state">
      <p className="chart-note">
        Event {n.frameIndex} · {n.time_us} µs · {n.boundary.kind}
      </p>
      <div className="timeline-container">
        <table className="timeline-table" aria-label="Recorded actor states">
          <thead>
            <tr>
              <th>Actor</th>
              <th>HP</th>
              <th>Energy</th>
              <th>Cycle</th>
              <th>Resources</th>
              <th>Shields</th>
            </tr>
          </thead>
          <tbody>
            {n.actors.map((a) => (
              <tr key={a.slot}>
                <td>{english.character(a.character_id)}</td>
                <td>
                  {a.hp} / {a.hp_max}
                </td>
                <td>
                  {a.energy} / {a.energy_capacity}
                </td>
                <td>{a.cycle ? `${a.cycle.current} / ${a.cycle.capacity}` : 'Undeclared'}</td>
                <td>
                  <pre>
                    {JSON.stringify(
                      { pools: a.resources, fractional: a.fractional_resources },
                      null,
                      2,
                    )}
                  </pre>
                </td>
                <td>
                  <pre>
                    {JSON.stringify(
                      (a.active_shields ?? []).map((shield) => ({
                        ...shield,
                        remaining_us: Math.max(
                          0,
                          shield.expires_at_us -
                            n.time_us -
                            (p.model.clocks.elapsedAt(shield.clock_basis, p.state.time) -
                              p.model.clocks.elapsedAt(shield.clock_basis, n.time_us)),
                        ),
                      })),
                      null,
                      2,
                    )}
                  </pre>
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
      <JsonDetails
        title="Targets, statuses, break, buffs, stamina and exact ledgers"
        value={{ ...n, buffs, timers }}
      />
      <JsonDetails
        title="Healing receipts at this prefix"
        value={p.model.report.healing_receipts.slice(0, n.healing_end)}
      />
    </Panel>
  );
}
