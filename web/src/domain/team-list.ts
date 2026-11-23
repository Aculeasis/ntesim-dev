import catalog from '../generated/team-list.json';
import { CATALOG } from './catalog';
import type { TeamCatalog, TeamEntry } from './contracts/team-list';

export const TEAM_CATALOG: TeamCatalog = catalog as TeamCatalog;
export interface TeamFilters {
  query: string;
  include: string[];
  exclude: string[];
}
export const emptyTeamFilters: TeamFilters = { query: '', include: [], exclude: [] };
export function filterTeams(entries: TeamEntry[], filters: TeamFilters): TeamEntry[] {
  const terms = filters.query.trim().toLowerCase().split(/\s+/).filter(Boolean);
  return entries
    .filter((entry) => {
      const ids = entry.actors.map((actor) => actor.character_id);
      const names = ids.map((id) =>
        (CATALOG.characters.find((c) => c.id === id)?.name ?? id).toLowerCase(),
      );
      return (
        filters.include.every((id) => ids.includes(id)) &&
        !filters.exclude.some((id) => ids.includes(id)) &&
        terms.every((term) => names.some((name) => name.includes(term)))
      );
    })
    .sort((a, b) => {
      if (a.dps.status !== b.dps.status) return a.dps.status === 'ready' ? -1 : 1;
      const diff =
        a.dps.status === 'ready' && b.dps.status === 'ready' ? b.dps.mean - a.dps.mean : 0;
      return diff || a.id.localeCompare(b.id);
    });
}
