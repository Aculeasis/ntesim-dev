import {
  useCallback,
  useEffect,
  useId,
  useMemo,
  useRef,
  useState,
  type CSSProperties,
  type UIEvent,
} from 'react';
import { english } from '../../domain/localization';
import { format, palette } from '../../components/ui/format';
import type { ReplayEvent } from '../../domain/replay/events';
import type { Playback } from './usePlayback';
import { Panel } from '../../components/ui/Panel';
import { EventTooltip } from './EventTooltip';

const ROW_HEIGHT = 60;
const OVERSCAN = 20;

export function EventLog({ playback: p }: { playback: Playback }) {
  const [query, setQuery] = useState(''),
    [kind, setKind] = useState('all'),
    { model: m, state: s } = p;
  const [tooltip, setTooltip] = useState<{ event: ReplayEvent; anchor: HTMLElement } | null>(null);
  const [scrollTop, setScrollTop] = useState(0);
  const [viewportHeight, setViewportHeight] = useState(620);
  const containerRef = useRef<HTMLDivElement>(null);
  const tooltipId = useId();
  const hideTooltip = useCallback(() => setTooltip(null), []);

  const groups = useMemo(() => {
    const groups = new Map<number, ReplayEvent[]>();
    for (const e of m.events) {
      if (
        (kind !== 'all' && e.kind !== kind) ||
        (query &&
          ![
            e.actor,
            e.action,
            e.detail,
            e.trace?.data?.ability_id,
            e.trace?.data?.damage_effect_id,
            e.trace?.data?.buff_id,
            e.trace?.data?.utility_id,
          ]
            .join(' ')
            .toLowerCase()
            .includes(query.toLowerCase()))
      )
        continue;
      if (!groups.has(e.time)) groups.set(e.time, []);
      groups.get(e.time)!.push(e);
    }
    return [...groups];
  }, [m, kind, query]);

  useEffect(() => {
    const el = containerRef.current;
    if (!el) return;
    const updateHeight = () => {
      if (el.clientHeight > 0) {
        setViewportHeight(el.clientHeight);
      }
    };
    updateHeight();
    if (typeof ResizeObserver !== 'undefined') {
      const observer = new ResizeObserver(updateHeight);
      observer.observe(el);
      return () => observer.disconnect();
    }
  }, []);

  useEffect(() => {
    if (containerRef.current) {
      containerRef.current.scrollTop = 0;
    }
    setScrollTop(0);
  }, [query, kind]);

  const handleScroll = useCallback(
    (e: UIEvent<HTMLDivElement>) => {
      setScrollTop(e.currentTarget.scrollTop);
      if (tooltip) setTooltip(null);
    },
    [tooltip],
  );

  const shouldVirtualize = groups.length > 40;
  const startIndex = shouldVirtualize
    ? Math.max(0, Math.floor(scrollTop / ROW_HEIGHT) - OVERSCAN)
    : 0;
  const endIndex = shouldVirtualize
    ? Math.min(groups.length, Math.ceil((scrollTop + viewportHeight) / ROW_HEIGHT) + OVERSCAN)
    : groups.length;

  const visibleGroups = shouldVirtualize ? groups.slice(startIndex, endIndex) : groups;
  const topSpacer = shouldVirtualize ? startIndex * ROW_HEIGHT : 0;
  const bottomSpacer = shouldVirtualize ? (groups.length - endIndex) * ROW_HEIGHT : 0;

  return (
    <Panel title="Combat event log">
      <span className="chart-note">Click an event to seek · hover or focus for details</span>
      <div className="trace-controls">
        <input
          id="trace-search"
          type="search"
          aria-label="Filter combat events"
          placeholder="Filter actor, action or event…"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
        />
        <select
          id="trace-kind"
          aria-label="Event kind"
          value={kind}
          onChange={(e) => setKind(e.target.value)}
        >
          <option value="all">All events</option>
          {[...new Set(m.events.map((e) => e.kind))].map((k) => (
            <option key={k} value={k}>
              {k.replaceAll('_', ' ')}
            </option>
          ))}
        </select>
      </div>
      <div id="combat-timeline" ref={containerRef} onScroll={handleScroll}>
        <div
          className="sample-log"
          style={{ '--columns': m.request.actors.length + 1 } as CSSProperties}
        >
          <div className="sample-log-header">
            <span>Time (s)</span>
            {m.request.actors.map((a, i) => (
              <strong key={a.slot} style={{ '--actor': palette[i % 4] } as CSSProperties}>
                {english.character(a.character_id)}
              </strong>
            ))}
            <strong>Team / system</strong>
          </div>
          {groups.length ? (
            <>
              {topSpacer > 0 && <div style={{ height: topSpacer }} aria-hidden="true" />}
              {visibleGroups.map(([time, entries]) => (
                <div className="sample-log-row" key={time}>
                  <time>{(time / 1e6).toFixed(3)}</time>
                  {[...m.request.actors.map((a) => a.slot), null].map((slot) => (
                    <div key={slot ?? 'global'}>
                      {entries
                        .filter((e) => e.owner === slot)
                        .map((e) => (
                          <button
                            className={`log-event ${e.hit ? 'log-hit' : ''} ${e.index < s.count ? 'event-past' : ''} ${e.index === s.last?.index ? 'event-current' : ''}`}
                            data-log-event={e.index}
                            key={e.index}
                            aria-describedby={tooltip?.event === e ? tooltipId : undefined}
                            onPointerEnter={(event) =>
                              setTooltip({ event: e, anchor: event.currentTarget })
                            }
                            onPointerLeave={hideTooltip}
                            onFocus={(event) =>
                              setTooltip({ event: e, anchor: event.currentTarget })
                            }
                            onBlur={hideTooltip}
                            onClick={() => p.seek(e.time, e.index + 1)}
                          >
                            <span>{e.action}</span>
                            {e.hit && (
                              <b>
                                {e.hit.critical ? 'CRIT · ' : ''}
                                {BigInt(e.hit.rounded_damage).toLocaleString()}
                              </b>
                            )}
                            <small>{e.kind.replaceAll('_', ' ')}</small>
                          </button>
                        ))}
                    </div>
                  ))}
                </div>
              ))}
              {bottomSpacer > 0 && <div style={{ height: bottomSpacer }} aria-hidden="true" />}
            </>
          ) : (
            <p className="unavailable">No matching events.</p>
          )}
        </div>
      </div>
      {tooltip && (
        <EventTooltip
          id={tooltipId}
          anchor={tooltip.anchor}
          detail={tooltip.event.detail}
          onClose={hideTooltip}
        />
      )}
    </Panel>
  );
}
