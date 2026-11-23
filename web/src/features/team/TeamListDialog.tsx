import { useEffect, useRef, useState } from 'react';
import { Dialog } from '../../components/ui/Dialog';
import { TEAM_CATALOG, filterTeams, type TeamFilters } from '../../domain/team-list';
import { TeamListRow } from './TeamListRow';
import { RosterPicker } from './RosterPicker';

export function TeamListDialog({
  filters,
  onFilters,
  onSelect,
  onClose,
  busy,
}: {
  filters: TeamFilters;
  onFilters: (filters: TeamFilters) => void;
  onSelect: (id: string) => void;
  onClose: () => void;
  busy: boolean;
}) {
  const [picker, setPicker] = useState<'include' | 'exclude' | null>(null);
  const controls = useRef<HTMLDivElement>(null);
  useEffect(() => {
    const outside = (event: PointerEvent) => {
      if (!controls.current?.contains(event.target as Node)) setPicker(null);
    };
    document.addEventListener('pointerdown', outside);
    return () => document.removeEventListener('pointerdown', outside);
  }, []);
  const entries = filterTeams(TEAM_CATALOG.entries, filters);
  return (
    <Dialog id="team-list-dialog" title="Team list" onClose={onClose} customHeading>
      <div
        className="team-list-content"
        onKeyDown={(event) => {
          if (event.key === 'Escape' && picker) {
            event.preventDefault();
            event.stopPropagation();
            controls.current?.querySelector<HTMLButtonElement>(`[data-filter=${picker}]`)?.focus();
            setPicker(null);
          }
        }}
      >
        <header className="team-list-header">
          <h2 id="team-list-dialog-title">Team list</h2>
          <input
            type="search"
            aria-label="Search characters"
            placeholder="Search characters…"
            value={filters.query}
            onChange={(event) => onFilters({ ...filters, query: event.target.value })}
          />
          <div className="roster-controls" ref={controls}>
            {(['include', 'exclude'] as const).map((mode) => (
              <button
                key={mode}
                data-filter={mode}
                aria-expanded={picker === mode}
                aria-controls="roster-picker"
                onClick={() => setPicker(picker === mode ? null : mode)}
              >
                {mode === 'include' ? 'Include' : 'Exclude'}{' '}
                <span className="filter-count">{filters[mode].length}</span>
              </button>
            ))}
            {picker && (
              <RosterPicker
                mode={picker}
                selected={filters[picker]}
                onChange={(ids) => onFilters({ ...filters, [picker]: ids })}
              />
            )}
          </div>
          <button
            className="dialog-close team-list-close"
            aria-label="Close Team list"
            onClick={onClose}
          >
            ×
          </button>
        </header>
        <div className="team-list-meta" role="status">
          {entries.length} teams
          {busy
            ? ' · Stop or finish the current execution to load a team.'
            : ' · Select a team to open its scenario'}
        </div>
        <div className="team-list-rows">
          {entries.map((entry) => (
            <TeamListRow
              key={entry.id}
              entry={entry}
              busy={busy}
              onSelect={() => {
                if (!busy) onSelect(entry.id);
              }}
            />
          ))}
          {!entries.length && (
            <p className="team-list-empty" role="status">
              No teams match your search and filters.
            </p>
          )}
        </div>
      </div>
    </Dialog>
  );
}
