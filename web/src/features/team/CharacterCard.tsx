import type { CSSProperties } from 'react';
import { CATALOG } from '../../domain/catalog';
import type { ActorInspection } from '../../domain/contracts/script';
import { Artwork } from '../../components/ui/Artwork';
import { Tooltip } from '../../components/ui/Tooltip';
import { elementColors, palette } from '../../components/ui/format';
import { CharacterStats } from './CharacterStats';
export function CharacterCard({
  actor: a,
  index,
  active,
  expanded,
  statsMode,
  onToggle,
  onInspect,
  onNavigate,
}: {
  actor: ActorInspection;
  index: number;
  active: boolean;
  expanded: boolean;
  statsMode: string;
  onToggle: () => void;
  onInspect: () => void;
  onNavigate?: () => void;
}) {
  const c = CATALOG.characters.find((c) => c.id === a.character_id);
  const weaponId = a.weapon?.id.toLowerCase();
  const weapon = weaponId ? CATALOG.weapons[weaponId] || a.weapon!.id : null;
  const set = a.set ? CATALOG.sets[a.set.id] || a.set.id : null;
  return (
    <article
      className="character-card"
      data-character-id={a.character_id}
      data-active={active}
      style={
        {
          '--character': palette[index % 4],
          '--element': elementColors[c?.element ?? ''],
        } as CSSProperties
      }
    >
      <div className="character-header">
        <Artwork
          category="characters"
          id={c?.slug}
          alt={c?.name ?? a.character_id}
          className="character-portrait"
        />
        <div className="card-controls">
          <button
            className="card-toggle"
            data-toggle={index}
            aria-label="Toggle all character stats"
            aria-expanded={expanded}
            onClick={onToggle}
          >
            {expanded ? '⌃' : '⌄'}
          </button>
          <button
            className="card-inspect"
            data-inspect={index}
            aria-label="Toggle stats and build contributions for all characters"
            aria-pressed={statsMode === 'equipment'}
            onClick={onInspect}
          >
            ⌕
          </button>
        </div>
        <div
          className="character-title"
          title={`Selected awakenings: ${a.awakening.join(', ') || 'none'}`}
        >
          A{a.awakening.length} {c?.name ?? a.character_id}
        </div>
        <div className="character-level">
          Lvl {a.level} / {a.max_level ?? a.level}
          {active ? ' · Active' : ''}
        </div>
        <div className="character-talent">
          Skills {a.talent_levels.join(' / ')} · QTE {a.purchased_qte_level}
        </div>
        <div className="character-equipment">
          {c && (
            <span className="element-item hint" tabIndex={0}>
              <Artwork category="elements" id={c.element} alt={c.element} />
              <Tooltip label={c.element} />
            </span>
          )}
          {a.set && (
            <span className="set-item hint" tabIndex={0}>
              <Artwork category="cartridges" id={a.set.id} alt={set!} />
              <span>{a.set.count}</span>
              <Tooltip label={set!}>{a.set.count}-piece set</Tooltip>
            </span>
          )}
        </div>
      </div>
      <div className="weapon-card hint" tabIndex={0}>
        <Artwork category="weapons" id={weaponId} alt={weapon || 'No weapon'} />
        <div>
          <span>{weapon ? `${weapon} T${a.weapon!.refinement}` : 'No weapon equipped'}</span>
          <small>
            {a.weapon ? `Lvl ${a.weapon.level} / ${a.weapon.max_level}` : 'Unequipped build'}
          </small>
        </div>
        <Tooltip label={weapon || 'No weapon'} />
      </div>
      <div className="card-details" hidden={!expanded}>
        <CharacterStats actor={a} mode={statsMode} />
      </div>
    </article>
  );
}
