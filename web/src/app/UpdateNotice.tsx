import { useEffect, useState } from 'react';
import { observeUpdates } from '../infrastructure/offline/updates';

export function UpdateNotice({ busy }: { busy: boolean }) {
  const [apply, setApply] = useState<(() => boolean) | null>(null),
    [applying, setApplying] = useState(false);
  useEffect(() => {
    if (!import.meta.env.PROD || !('serviceWorker' in navigator)) return;
    let disposed = false,
      stop = () => {};
    navigator.serviceWorker
      .register(`${import.meta.env.BASE_URL}sw.js`, { updateViaCache: 'none' })
      .then((registration) => {
        if (disposed) return;
        const unobserve = observeUpdates(registration, (action) => setApply(() => action));
        const check = () => {
          if (document.visibilityState === 'visible') void registration.update().catch(() => {}); // Offline use remains available.
        };
        const timer = window.setInterval(check, 5 * 60_000);
        window.addEventListener('focus', check);
        window.addEventListener('online', check);
        document.addEventListener('visibilitychange', check);
        stop = () => {
          unobserve();
          window.clearInterval(timer);
          window.removeEventListener('focus', check);
          window.removeEventListener('online', check);
          document.removeEventListener('visibilitychange', check);
        };
      })
      .catch((error) => console.warn('Offline cache unavailable:', error));
    return () => {
      disposed = true;
      stop();
    };
  }, []);
  if (!apply) return null;
  return (
    <aside className="update-notice" aria-label="Application update">
      <span role="status">
        <strong>A new version is ready.</strong>{' '}
        {busy
          ? 'Update after the current operation finishes.'
          : 'Reloading clears current results.'}
      </span>
      <button
        id="update-app"
        disabled={busy || applying}
        onClick={() => {
          setApplying(apply());
        }}
      >
        {applying ? 'Updating…' : 'Update & reload'}
      </button>
    </aside>
  );
}
