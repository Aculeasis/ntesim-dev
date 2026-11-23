import { useState, type ReactNode, type SyntheticEvent } from 'react';
export function ChartInspector({
  children,
  className = 'time-chart-shell',
}: {
  children: ReactNode;
  className?: string;
}) {
  const [tip, setTip] = useState('');
  const show = (e: SyntheticEvent) => {
    if (e.target instanceof Element) {
      const point = e.target.closest<HTMLElement>('[data-chart-tip]');
      if (point) setTip(point.dataset.chartTip ?? '');
    }
  };
  return (
    <div
      className={className}
      onPointerOver={show}
      onFocusCapture={show}
      onPointerOut={() => setTip('')}
      onBlurCapture={() => setTip('')}
    >
      {children}
      <output className="time-inspector" hidden={!tip}>
        {tip}
      </output>
    </div>
  );
}
