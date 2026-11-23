import { useEffect, useLayoutEffect, useRef, useState } from 'react';
import { createPortal } from 'react-dom';

export function EventTooltip({
  id,
  anchor,
  detail,
  onClose,
}: {
  id: string;
  anchor: HTMLElement;
  detail: string;
  onClose: () => void;
}) {
  const popup = useRef<HTMLDivElement>(null);
  const [position, setPosition] = useState({ left: 8, top: 8 });
  useLayoutEffect(() => {
    const box = popup.current!.getBoundingClientRect();
    const host = anchor.getBoundingClientRect();
    const above = host.top - box.height - 8;
    setPosition({
      left: Math.max(8, Math.min(host.left, innerWidth - box.width - 8)),
      top: Math.max(
        8,
        Math.min(above >= 8 ? above : host.bottom + 8, innerHeight - box.height - 8),
      ),
    });
  }, [anchor, detail]);
  useEffect(() => {
    const key = (event: KeyboardEvent) => {
      if (event.key === 'Escape') onClose();
    };
    document.addEventListener('keydown', key);
    document.addEventListener('scroll', onClose, true);
    window.addEventListener('resize', onClose);
    return () => {
      document.removeEventListener('keydown', key);
      document.removeEventListener('scroll', onClose, true);
      window.removeEventListener('resize', onClose);
    };
  }, [onClose]);
  return createPortal(
    <div ref={popup} id={id} className="event-detail" role="tooltip" style={position}>
      {detail}
    </div>,
    document.body,
  );
}
