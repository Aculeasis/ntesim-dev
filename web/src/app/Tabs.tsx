import { useRef } from 'react';
export type View = 'config' | 'results' | 'sample';
const tabs: [View, string, string][] = [
  ['config', 'Simulator', 'config'],
  ['results', 'Results', 'results'],
  ['sample', 'Sample', 'sample-view'],
];
export function Tabs({
  view,
  available,
  onChange,
}: {
  view: View;
  available: boolean;
  onChange: (view: View) => void;
}) {
  const ref = useRef<HTMLElement>(null);
  return (
    <nav
      ref={ref}
      className="view-tabs"
      role="tablist"
      aria-label="Simulation"
      onKeyDown={(e) => {
        if (!['ArrowRight', 'ArrowLeft', 'Home', 'End'].includes(e.key)) return;
        e.preventDefault();
        const enabled = Array.from(
            ref.current!.querySelectorAll<HTMLButtonElement>('[role=tab]'),
          ).filter((b) => !b.disabled),
          index = enabled.indexOf(document.activeElement as HTMLButtonElement);
        const next =
          e.key === 'Home'
            ? 0
            : e.key === 'End'
              ? enabled.length - 1
              : (index + (e.key === 'ArrowRight' ? 1 : -1) + enabled.length) % enabled.length;
        enabled[next].focus();
        onChange(enabled[next].dataset.view as View);
      }}
    >
      {tabs.map(([key, label, panel]) => (
        <button
          id={'tab-' + key}
          key={key}
          role="tab"
          data-view={key}
          aria-selected={view === key}
          aria-controls={panel}
          tabIndex={view === key ? 0 : -1}
          disabled={key !== 'config' && !available}
          onClick={() => onChange(key)}
        >
          {label}
        </button>
      ))}
    </nav>
  );
}
