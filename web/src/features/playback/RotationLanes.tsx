import { useLayoutEffect, useRef, useState, type CSSProperties } from 'react';
import { english } from '../../domain/localization';
import { palette, format } from '../../components/ui/format';
import type { Playback } from './usePlayback';
import { markerLayout } from './markerLayout';
export function RotationLanes({ playback: p }: { playback: Playback }) {
  const { model: m, state } = p;
  const track = useRef<HTMLDivElement>(null);
  const [width, setWidth] = useState(0);
  useLayoutEffect(() => {
    const element = track.current;
    if (!element) return;
    const measure = () => setWidth(element.clientWidth);
    measure();
    const observer = new ResizeObserver(measure);
    observer.observe(element);
    return () => observer.disconnect();
  }, [m]);
  return (
    <div className="rotation-lanes">
      <div className="rotation-axis">
        <span />
        <div>
          {[0, 0.25, 0.5, 0.75, 1].map((f) => (
            <span key={f} style={{ left: `${f * 100}%` }}>
              {format((m.horizon * f) / 1e6, 2)}s
            </span>
          ))}
        </div>
      </div>
      {m.request.actors.map((a, i) => {
        const actions = m.events.filter((e) => e.owner === a.slot && e.admitted);
        const layout = markerLayout(
          actions.map((e) => e.time),
          m.horizon,
          width,
        );
        return (
          <div className="rotation-row" key={a.slot}>
            <span>{english.character(a.character_id)}</span>
            <div
              ref={i === 0 ? track : undefined}
              className="rotation-track"
              style={{ '--actor': palette[i % 4], height: layout.count * 31 } as CSSProperties}
            >
              {m.clocks.intervals.map(([start, end], index) => (
                <span
                  key={`pause-${index}`}
                  className="timer-pause-segment"
                  style={{
                    left: `${(start / Math.max(1, m.horizon)) * 100}%`,
                    width: `${((end - start) / Math.max(1, m.horizon)) * 100}%`,
                  }}
                  title={`Relative timer paused: absolute ${(start / 1e6).toFixed(3)}–${(end / 1e6).toFixed(3)}s`}
                />
              ))}
              {m.segments
                .filter((s) => s.slot === a.slot)
                .map((s, i) => (
                  <span
                    className="field-segment"
                    key={i}
                    style={{
                      left: `${(s.start / m.horizon) * 100}%`,
                      width: `${((s.end - s.start) / m.horizon) * 100}%`,
                    }}
                    title={`On field ${(s.start / 1e6).toFixed(3)}–${(s.end / 1e6).toFixed(3)}s`}
                  />
                ))}
              {actions.map((e, index) => (
                <button
                  className={`action-marker ${e.index === state.last?.index ? 'selected-event' : ''}`}
                  key={e.index}
                  data-event={e.index}
                  style={{
                    left: `${(e.time / m.horizon) * 100}%`,
                    top: layout.rows[index] * 31 + 1,
                  }}
                  aria-label={`${e.actor} ${e.action} at ${format(e.time / 1e6, 3)} seconds`}
                  title={`${e.action} · ${(e.time / 1e6).toFixed(3)}s`}
                  onClick={() => p.seek(e.time, e.index + 1)}
                >
                  {e.primarySlot === 'ultimate'
                    ? 'B'
                    : e.primarySlot === 'normal'
                      ? 'A'
                      : e.primarySlot === 'skill'
                        ? 'S'
                        : e.inputRole === 'qte'
                          ? 'Q'
                          : '·'}
                </button>
              ))}
              <i className="playhead" style={{ left: `${(state.time / m.horizon) * 100}%` }} />
            </div>
          </div>
        );
      })}
    </div>
  );
}
