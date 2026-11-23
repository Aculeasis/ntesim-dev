import { useEffect, useId, useLayoutEffect, useRef, useState, type ReactNode } from 'react';
import { createPortal } from 'react-dom';

export function tooltipLayout({
  host,
  box,
  viewport,
}: {
  host: { left: number; top: number; bottom: number };
  box: { width: number; height: number };
  viewport: { width: number; height: number };
}): { left: number; top: number } {
  const fitsBelow = host.bottom + box.height + 8 <= viewport.height - 8;
  const above = host.top - box.height - 8;
  return {
    left: Math.max(8, Math.min(host.left, viewport.width - box.width - 8)),
    top: fitsBelow
      ? Math.max(8, host.bottom + 8)
      : above >= 8
        ? above
        : Math.max(8, viewport.height - box.height - 8),
  };
}

export function Tooltip({ label, children }: { label: string; children?: ReactNode }) {
  const ref = useRef<HTMLSpanElement>(null),
    popup = useRef<HTMLDivElement>(null),
    id = useId();
  const [target, setTarget] = useState<HTMLElement | null>(null);
  const [position, setPosition] = useState<{ left: number; top: number } | null>(null);

  useEffect(() => {
    const host = ref.current?.closest<HTMLElement>('.hint,.stat-row');
    if (!host) return;
    const hide = () => {
      setTarget(null);
      setPosition(null);
      host.removeAttribute('aria-describedby');
    };
    const show = () => {
      setTarget(host);
      host.setAttribute('aria-describedby', id);
    };
    const key = (e: KeyboardEvent) => {
      if (e.key === 'Escape') hide();
    };
    host.addEventListener('pointerenter', show);
    host.addEventListener('pointerleave', hide);
    host.addEventListener('focusin', show);
    host.addEventListener('focusout', hide);
    document.addEventListener('keydown', key);
    document.addEventListener('scroll', hide, true);
    window.addEventListener('resize', hide);
    return () => {
      host.removeEventListener('pointerenter', show);
      host.removeEventListener('pointerleave', hide);
      host.removeEventListener('focusin', show);
      host.removeEventListener('focusout', hide);
      document.removeEventListener('keydown', key);
      document.removeEventListener('scroll', hide, true);
      window.removeEventListener('resize', hide);
      host.removeAttribute('aria-describedby');
    };
  }, [id]);

  useLayoutEffect(() => {
    if (!target || !popup.current) return;
    setPosition(
      tooltipLayout({
        host: target.getBoundingClientRect(),
        box: popup.current.getBoundingClientRect(),
        viewport: { width: innerWidth, height: innerHeight },
      }),
    );
  }, [target, label]);

  const content = (
    <>
      <strong>{label}</strong>
      {children}
    </>
  );

  return (
    <>
      <span ref={ref} className="tooltip" role="tooltip">
        {content}
      </span>
      {target &&
        createPortal(
          <div
            ref={popup}
            id={id}
            className="floating-card-tooltip"
            role="tooltip"
            style={{
              left:
                position?.left ??
                Math.max(8, Math.min(innerWidth - 296, target.getBoundingClientRect().left)),
              top: position?.top ?? Math.max(8, target.getBoundingClientRect().bottom + 8),
              visibility: position ? undefined : 'hidden',
            }}
          >
            {content}
          </div>,
          document.body,
        )}
    </>
  );
}
