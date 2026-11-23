import { useLayoutEffect, useRef, type ReactNode } from 'react';
export function Dialog({
  id,
  title,
  onClose,
  children,
  customHeading = false,
  accessibleTitle = false,
}: {
  id: string;
  title: string;
  onClose: () => void;
  children: ReactNode;
  customHeading?: boolean;
  accessibleTitle?: boolean;
}) {
  const ref = useRef<HTMLDialogElement>(null);
  useLayoutEffect(() => {
    const dialog = ref.current!;
    const opener = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    dialog.showModal();
    return () => {
      if (dialog.open) dialog.close();
      if (opener?.isConnected) opener.focus({ preventScroll: true });
    };
  }, []);
  return (
    <dialog
      ref={ref}
      id={id}
      aria-labelledby={accessibleTitle ? undefined : id + '-title'}
      aria-label={accessibleTitle ? title : undefined}
      onClose={onClose}
      onKeyDown={(event) => {
        if (event.key === 'Escape') {
          event.preventDefault();
          event.stopPropagation();
          onClose();
        }
      }}
      onClick={(event) => {
        if (event.target === ref.current) {
          const rect = ref.current.getBoundingClientRect();
          const isInDialog =
            rect.top <= event.clientY &&
            event.clientY <= rect.top + rect.height &&
            rect.left <= event.clientX &&
            event.clientX <= rect.left + rect.width;
          if (!isInDialog) {
            onClose();
          }
        }
      }}
    >
      {!customHeading && (
        <div className="dialog-heading">
          <h2 id={id + '-title'}>{title}</h2>
          <button onClick={onClose}>Close</button>
        </div>
      )}
      {children}
    </dialog>
  );
}
