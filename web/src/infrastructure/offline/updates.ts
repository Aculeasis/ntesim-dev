// Installation prepares the full cache; only an explicit click activates an update.
export function observeUpdates(
  registration: ServiceWorkerRegistration,
  onAvailable: (apply: () => boolean) => void,
  serviceWorkers: ServiceWorkerContainer = navigator.serviceWorker,
  reload: () => void = () => window.location.reload(),
) {
  let controller = serviceWorkers.controller,
    offeredWaiting: ServiceWorker | null = null,
    changed = false,
    requested = false,
    reloaded = false;
  const reloadOnce = () => {
    if (reloaded) return;
    reloaded = true;
    reload();
  };
  const apply = () => {
    if (requested) return true;
    if (registration.waiting) {
      requested = true;
      registration.waiting.postMessage({ type: 'ACTIVATE_UPDATE' });
    } else if (changed) {
      requested = true;
      reloadOnce();
    }
    return requested;
  };
  const available = () => {
    if (registration.active && registration.waiting) {
      offeredWaiting = registration.waiting;
      onAvailable(apply);
    }
  };
  const workers = new Set<ServiceWorker>();
  const watchInstalling = () => {
    const worker = registration.installing;
    if (!worker || workers.has(worker)) return;
    workers.add(worker);
    worker.addEventListener('statechange', available);
    available();
  };
  const controllerChanged = () => {
    const previous = controller;
    controller = serviceWorkers.controller;
    if (!controller || previous === controller) return;
    // The first install claims the page without prompting or reloading it.
    if (!previous && !requested && controller !== offeredWaiting) return;
    changed = true;
    if (requested) reloadOnce();
    else onAvailable(apply); // An update activated in another tab still needs a reload here.
  };
  registration.addEventListener('updatefound', watchInstalling);
  serviceWorkers.addEventListener('controllerchange', controllerChanged);
  watchInstalling();
  available();
  return () => {
    registration.removeEventListener('updatefound', watchInstalling);
    serviceWorkers.removeEventListener('controllerchange', controllerChanged);
    for (const worker of workers) worker.removeEventListener('statechange', available);
  };
}
