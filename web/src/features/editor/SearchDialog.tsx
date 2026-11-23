import { useMemo, useState } from 'react';
import { Dialog } from '../../components/ui/Dialog';
import { Artwork } from '../../components/ui/Artwork';
import { buildSearchItems } from './snippets';
export function SearchDialog({
  category,
  onInsert,
  onClose,
}: {
  category: string;
  onInsert: (code: string) => void;
  onClose: () => void;
}) {
  const [query, setQuery] = useState(''),
    items = useMemo(() => buildSearchItems(category, query), [category, query]);
  return (
    <Dialog
      id="search-dialog"
      title={category[0].toUpperCase() + category.slice(1)}
      onClose={onClose}
      customHeading
      accessibleTitle
    >
      <header className="search-dialog-header">
        <input
          type="search"
          id="reference-search"
          aria-label="Search names"
          placeholder="Search…"
          autoFocus
          value={query}
          onChange={(e) => setQuery(e.target.value)}
        />
        <button className="dialog-close" aria-label="Close name search" onClick={onClose}>
          ×
        </button>
      </header>
      <p id="search-note" className="chart-note">
        Click to insert at the editor cursor.
      </p>
      <div id="reference-list" className="reference-gallery">
        {items.length ? (
          items.map((x, i) => (
            <button
              className="reference-item"
              key={i}
              data-entry={i}
              onClick={() => {
                onInsert(x.code);
                onClose();
              }}
            >
              {x.category && <Artwork category={x.category} id={x.id} alt={x.name} />}
              <strong>{x.name}</strong>
              <small>{x.description}</small>
            </button>
          ))
        ) : (
          <p>No matching names.</p>
        )}
      </div>
    </Dialog>
  );
}
