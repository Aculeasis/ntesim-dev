import type { CSSProperties } from 'react';
import type { TeamActor } from '../../domain/contracts/team-list';
import { CATALOG } from '../../domain/catalog';
import { Artwork } from '../../components/ui/Artwork';
import { elementColors } from '../../components/ui/format';

export function CompactCharacter({ actor }: { actor?: TeamActor }) {
  if (!actor)
    return (
      <div className="compact-character empty-slot" aria-label="Empty slot">
        <span>Empty slot</span>
      </div>
    );
  const character = CATALOG.characters.find((c) => c.id === actor.character_id);
  const weapon = actor.weapon;
  const weaponName = weapon && (CATALOG.weapons[weapon.id.toLowerCase()] ?? weapon.id);
  const set = actor.set;
  const gear = [
    weapon
      ? `${weaponName} · T${weapon.refinement} · Lvl ${weapon.level}/${weapon.max_level}`
      : 'No Arc equipped',
    set ? `${CATALOG.sets[set.id] ?? set.id} · ${set.count}-piece set` : 'No cartridge set',
  ].join('\n');
  return (
    <div
      className="compact-character"
      data-character-id={actor.character_id}
      style={{ '--element': elementColors[character?.element ?? ''] } as CSSProperties}
    >
      <Artwork
        category="characters"
        id={character?.slug}
        alt={character?.name ?? actor.character_id}
        className="compact-portrait"
      />
      <strong>{character?.name ?? actor.character_id}</strong>
      <small>
        A{actor.awakening.length} · Lvl {actor.level}
      </small>
      <span className="compact-equipment" tabIndex={0} aria-label={gear} title={gear}>
        {character && (
          <Artwork category="elements" id={character.element} alt={character.element} />
        )}
        {weapon && (
          <span>
            <Artwork category="weapons" id={weapon.id.toLowerCase()} alt={weaponName!} />
            <small>T{weapon.refinement}</small>
          </span>
        )}
        {set && (
          <span>
            <Artwork category="cartridges" id={set.id} alt={CATALOG.sets[set.id] ?? set.id} />
            <small>{set.count}</small>
          </span>
        )}
        <span className="compact-gear-details">{gear}</span>
      </span>
    </div>
  );
}
