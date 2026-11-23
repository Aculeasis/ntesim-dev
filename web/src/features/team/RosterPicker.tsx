import { CATALOG } from '../../domain/catalog';
import { Artwork } from '../../components/ui/Artwork';

export function RosterPicker({
  mode,
  selected,
  onChange,
}: {
  mode: 'include' | 'exclude';
  selected: string[];
  onChange: (ids: string[]) => void;
}) {
  return (
    <div
      className="roster-picker"
      id="roster-picker"
      role="group"
      aria-label={`${mode === 'include' ? 'Include' : 'Exclude'} characters`}
    >
      <button className="roster-clear" onClick={() => onChange([])}>
        Clear All
      </button>
      <div className="roster-grid">
        {[...CATALOG.characters]
          .sort((a, b) => a.name.localeCompare(b.name))
          .map((character) => (
            <button
              key={character.id}
              aria-pressed={selected.includes(character.id)}
              aria-label={character.name}
              onClick={() =>
                onChange(
                  selected.includes(character.id)
                    ? selected.filter((id) => id !== character.id)
                    : [...selected, character.id],
                )
              }
            >
              <Artwork category="characters" id={character.slug} alt="" />
              <span>{character.name}</span>
            </button>
          ))}
      </div>
    </div>
  );
}
