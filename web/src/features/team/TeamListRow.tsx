import type { TeamEntry } from '../../domain/contracts/team-list';
import { CompactCharacter } from './CompactCharacter';
import { format } from '../../components/ui/format';

export function TeamListRow({
  entry,
  busy,
  onSelect,
}: {
  entry: TeamEntry;
  busy: boolean;
  onSelect: () => void;
}) {
  return (
    <article
      className="team-list-row"
      data-team-id={entry.id}
      onClick={(event) => {
        if (!busy && event.target instanceof Element && !event.target.closest('.compact-equipment'))
          onSelect();
      }}
    >
      <button className="team-row-select" disabled={busy} aria-label={`Load ${entry.label}`}>
        <span className="team-row-label">{entry.label}</span>
      </button>
      <div className="team-row-cards">
        {Array.from({ length: 4 }, (_, i) => (
          <CompactCharacter key={i} actor={entry.actors[i]} />
        ))}
      </div>
      <div className="team-row-dps">
        <strong>{entry.dps.status === 'ready' ? format(entry.dps.mean) : 'Unavailable'}</strong>
        <small>{entry.dps.status === 'ready' ? 'DPS' : 'DPS unavailable'}</small>
        {entry.dps.status === 'unavailable' && (
          <small className="team-dps-reason">{entry.dps.reason}</small>
        )}
      </div>
    </article>
  );
}
