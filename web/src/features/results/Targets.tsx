import type { Target } from '../../domain/contracts/scenario';
import { fraction } from '../../domain/reports/statistics';
import { format, palette } from '../../components/ui/format';
import { Artwork } from '../../components/ui/Artwork';
import type { CSSProperties } from 'react';

const elements = [
  ['NATURE', 'Anima'],
  ['LAKSHANA', 'Lakshana'],
  ['PSYCHE', 'Psyche'],
  ['INCANTATION', 'Incantation'],
  ['CHAOS', 'Chaos'],
  ['COSMOS', 'Cosmos'],
] as const;
export function Targets({ targets }: { targets: Target[] }) {
  return (
    <>
      {targets.map((t) => (
        <div
          className="target-card"
          key={t.target_index}
          style={{ '--target-color': palette[t.target_index % palette.length] } as CSSProperties}
        >
          <div className="target-header-line">
            <strong>Target {t.target_index + 1}:</strong>
            <div className="target-stats">
              <span>
                Lvl <b>{t.level ?? '—'}</b>
              </span>
              <span>
                HP <b>{format(t.hp_current)}</b> / {format(t.hp_max)}
              </span>
              <span>
                DEF <b>{format(t.defense)}</b>
              </span>
            </div>
          </div>
          <div
            className="target-resistances"
            aria-label={`Target ${t.target_index + 1} elemental resistances`}
          >
            {elements.map(([key, name]) => {
              const overridden = t.elemental_resistances?.[key];
              const value = format(fraction(overridden ?? t.resistance) * 100, 2) + '%';
              return (
                <span
                  key={key}
                  className="target-resistance"
                  title={`${name} resistance: ${value} (${overridden ? 'element override' : 'base resistance'})`}
                >
                  <Artwork category="elements" id={name} alt={name} />
                  <span>{name}</span>
                  <b>{value}</b>
                </span>
              );
            })}
          </div>
        </div>
      ))}
    </>
  );
}
