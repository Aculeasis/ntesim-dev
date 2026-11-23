import { useState } from 'react';
import type { PartyInspection, SourceLocation } from '../../domain/contracts/script';
import { CharacterCard } from './CharacterCard';
export function TeamCards({
  party,
  initialExpanded = false,
  onNavigate,
}: {
  party: PartyInspection;
  initialExpanded?: boolean;
  onNavigate?: (location: SourceLocation) => void;
}) {
  const [expanded, setExpanded] = useState(initialExpanded),
    [mode, setMode] = useState('total');
  return (
    <>
      {party.actors.map((actor, index) => (
        <CharacterCard
          key={actor.slot}
          actor={actor}
          index={index}
          active={actor.slot === party.active_actor_index}
          expanded={expanded}
          statsMode={mode}
          onToggle={() => setExpanded((v) => !v)}
          onNavigate={onNavigate && (() => onNavigate(actor.loc))}
          onInspect={() => {
            setMode((v) => (v === 'total' ? 'equipment' : 'total'));
            setExpanded(true);
          }}
        />
      ))}
    </>
  );
}
