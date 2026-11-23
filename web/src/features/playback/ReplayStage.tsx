import type { CSSProperties } from 'react';
import { CATALOG } from '../../domain/catalog';
import { english } from '../../domain/localization';
import { Artwork } from '../../components/ui/Artwork';
import { format, palette } from '../../components/ui/format';
import type { Playback } from './usePlayback';
import { RotationLanes } from './RotationLanes';
export function ReplayStage({ playback: p }: { playback: Playback }) {
  const { model: m, state: s } = p;
  return (
    <div id="combat-replay" data-playing={p.playing}>
      <div className="replay-controls">
        <button
          data-replay="play"
          aria-label={p.playing ? 'Pause sample' : 'Play sample'}
          onClick={p.toggle}
        >
          {p.playing ? 'Ⅱ Pause' : '▶ Play'}
        </button>
        <button data-replay="reset" onClick={() => p.seek(0)}>
          ↺ Restart
        </button>
        <button data-replay="previous" aria-label="Previous event" onClick={() => p.step(-1)}>
          ← Event
        </button>
        <button data-replay="next" aria-label="Next event" onClick={() => p.step(1)}>
          Event →
        </button>
        <label>
          Speed{' '}
          <select
            data-replay="speed"
            value={p.speed}
            onChange={(e) => p.setSpeed(Number(e.target.value))}
          >
            {[0.25, 0.5, 1, 2, 4].map((v) => (
              <option value={v} key={v}>
                {v}×
              </option>
            ))}
          </select>
        </label>
        <strong className="replay-clock">
          Absolute {(s.time / 1e6).toFixed(2)} / {(m.horizon / 1e6).toFixed(2)} s
          <br />
          Relative {(m.clocks.relativeAt(s.time) / 1e6).toFixed(2)} /{' '}
          {(m.report.relative_elapsed_us / 1e6).toFixed(2)} s
        </strong>
      </div>
      <input
        className="replay-seek"
        type="range"
        min="0"
        max={m.horizon}
        step="1"
        value={s.time}
        aria-label="Sample time"
        onChange={(e) => p.seek(Number(e.target.value))}
      />
      <div className="replay-arena">
        <div className="replay-actors">
          {s.actors.map((a, i) => {
            const c = CATALOG.characters.find((c) => c.id === a.character_id),
              recent = m.events
                .slice(0, s.count)
                .filter((e) => e.owner === a.slot && e.hit && s.time - e.time < 650000),
              recentDamage = recent.reduce(
                (sum, event) => sum + BigInt(event.hit!.rounded_damage),
                0n,
              );
            return (
              <article
                className={`replay-actor ${a.slot === s.active ? 'on-field' : ''} ${recent.length ? 'hit-cue' : ''} ${a.lastAction && s.time - a.lastAction.time < 450000 ? 'cast-cue' : ''}`}
                key={a.slot}
                data-slot={a.slot}
                style={{ '--actor': palette[i % 4] } as CSSProperties}
              >
                <div className="replay-portrait">
                  <Artwork
                    category="characters"
                    id={c?.slug}
                    alt={english.character(a.character_id)}
                  />
                  <span className="on-field-label" hidden={a.slot !== s.active}>
                    ON FIELD
                  </span>
                  <span className="replay-hit" aria-hidden="true">
                    {recent.length
                      ? `${recent.some((e) => e.hit!.critical) ? 'CRIT ' : ''}${recentDamage.toLocaleString()}`
                      : ''}
                  </span>
                </div>
                <div className="replay-actor-body">
                  <strong>{english.character(a.character_id)}</strong>
                  <span className="replay-action">{a.lastAction?.action ?? 'Ready'}</span>
                  <div className="energy-heading">
                    <span>Energy</span>
                    <b className="replay-energy-value">
                      {a.energy} / {a.capacity}
                    </b>
                  </div>
                  <div className="energy-track">
                    <i style={{ width: `${a.capacity ? (a.energy / a.capacity) * 100 : 0}%` }} />
                  </div>
                  <span className="replay-actor-damage">
                    {format(a.damage, 2)} damage · {a.hits} hits
                  </span>
                </div>
              </article>
            );
          })}
        </div>
        <div className="replay-targets">
          {s.targets.map((t) => (
            <div className="replay-target" key={t.index} data-target={t.index}>
              <span className="target-symbol">◎</span>
              <strong>Target {t.index + 1}</strong>
              <span className="target-hp">
                HP {format(t.hp)} / {format(t.max)}
              </span>
              <span className="target-damage">{format(t.damage, 2)} damage</span>
            </div>
          ))}
        </div>
      </div>
      <p className="replay-caption" aria-live="polite">
        {s.last
          ? `${(s.last.time / 1e6).toFixed(3)}s · ${s.last.actor} · ${s.last.action}`
          : 'Initial state'}
      </p>
      <RotationLanes playback={p} />
      <p className="chart-note">
        Absolute time · shaded intervals pause relative time · A: normal attack / S: skill / B:
        ultimate
      </p>
    </div>
  );
}
