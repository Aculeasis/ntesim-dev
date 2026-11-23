import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import vm from 'node:vm';
import { observeUpdates } from '../src/infrastructure/offline/updates.ts';
const root = fileURLToPath(new URL('../../target/web/', import.meta.url));
const source = readFileSync(path.join(root, 'sw.js'), 'utf8'),
  handlers = {},
  stores = new Map([
    ['ntesim-offline-v24', new Map()],
    ['other-app', new Map()],
  ]);
let claimed = false,
  waiting = false;
const origin = 'https://example.test';
const context = {
  URL,
  self: {
    location: { origin },
    addEventListener: (type, fn) => (handlers[type] = fn),
    skipWaiting: () => {
      waiting = true;
    },
    clients: {
      claim: () => {
        claimed = true;
      },
    },
  },
  caches: {
    keys: async () => [...stores.keys()],
    delete: async (key) => stores.delete(key),
    open: async (name) => {
      if (!stores.has(name)) stores.set(name, new Map());
      const store = stores.get(name);
      return {
        addAll: async (files) => {
          for (const file of files)
            store.set(
              new URL(file, origin + '/').href,
              readFileSync(path.join(root, file === './' ? 'index.html' : file)),
            );
        },
        match: async (request) => store.get(request.url),
      };
    },
  },
  fetch: async (request) => 'network:' + request.url,
};
vm.runInNewContext(source, context);
let pending;
handlers.install({ waitUntil: (p) => (pending = p) });
await pending;
assert.equal(waiting, false, 'Downloading a new cache must not activate it');
handlers.message({ data: { type: 'unrelated' }, waitUntil: () => assert.fail('Unknown message') });
handlers.message({ data: { type: 'ACTIVATE_UPDATE' }, waitUntil: (p) => (pending = p) });
await pending;
assert.ok(waiting, 'Only an explicit update request skips waiting');
const current = [...stores.keys()].find(
  (k) => k !== 'ntesim-offline-v24' && k.startsWith('ntesim-offline-'),
);
function walk(dir) {
  return readdirSync(dir, { withFileTypes: true }).flatMap((e) =>
    e.isDirectory()
      ? walk(path.join(dir, e.name))
      : [path.relative(root, path.join(dir, e.name)).replaceAll('\\', '/')],
  );
}
for (const file of walk(root).filter((f) => f !== 'sw.js'))
  assert.ok(stores.get(current).has(origin + '/' + file), 'Manifest omits ' + file);
assert.ok([...stores.get(current).keys()].some((k) => k.endsWith('.wasm')));
const wasmUrls = [...stores.get(current).keys()].filter((k) => k.endsWith('.wasm'));
assert.equal(wasmUrls.length, 2);
const engineWasmUrl = wasmUrls.find((url) => url.includes('ntesim_wasm_bg'));
assert.deepEqual(
  stores.get(current).get(engineWasmUrl),
  readFileSync(new URL('../../target/wasm-bindgen/ntesim_wasm_bg.wasm', import.meta.url)),
);
assert.ok([...stores.get(current).keys()].some((k) => /worker-.*\.js$/.test(k)));
// Execute the bundled Worker without Node's require/module globals. Source-only
// Worker tests cannot detect a CommonJS loader leaking into the browser bundle.
const workerFile = walk(root).find((file) => /\/worker-.*\.js$/.test(file));
const workerMessages = [];
const previousPost = globalThis.postMessage,
  previousHandler = globalThis.onmessage;
try {
  globalThis.postMessage = (message) => workerMessages.push(message);
  // Force ESM without tsx's CommonJS file loader, matching a browser Worker.
  const bundled = readFileSync(path.join(root, workerFile)).toString('base64');
  await import(`data:text/javascript;base64,${bundled}`);
  await globalThis.onmessage({
    data: {
      id: 1,
      type: 'VERSION',
      wasmSource: stores.get(current).get(engineWasmUrl),
    },
  });
} finally {
  globalThis.postMessage = previousPost;
  globalThis.onmessage = previousHandler;
}
assert.equal(workerMessages.at(-1).type, 'SUCCESS', JSON.stringify(workerMessages));
assert.equal(workerMessages.at(-1).result.status, 'INCOMPLETE');
handlers.activate({ waitUntil: (p) => (pending = p) });
await pending;
assert.ok(claimed);
assert.ok(!stores.has('ntesim-offline-v24'));
assert.ok(stores.has('other-app'));
context.fetch = async () => {
  throw Error('offline');
};
for (const url of stores.get(current).keys()) {
  handlers.fetch({ request: { url, method: 'GET' }, respondWith: (p) => (pending = p) });
  assert.ok((await pending).length > 0);
}
let intercepted = false;
for (const request of [
  { url: origin + '/x', method: 'POST' },
  { url: 'https://another.test/x', method: 'GET' },
])
  handlers.fetch({
    request,
    respondWith: () => {
      intercepted = true;
    },
  });
assert.equal(intercepted, false);

function lifecycle(controlled = true, alreadyWaiting = false, active = controlled) {
  const messages = [],
    offers = [];
  const worker = Object.assign(new EventTarget(), {
    postMessage: (message) => messages.push(message),
  });
  const registration = Object.assign(new EventTarget(), {
    installing: alreadyWaiting ? null : worker,
    waiting: alreadyWaiting ? worker : null,
    active: active ? {} : null,
  });
  const container = Object.assign(new EventTarget(), { controller: controlled ? {} : null });
  let reloads = 0;
  const stop = observeUpdates(
    registration,
    (apply) => offers.push(apply),
    container,
    () => reloads++,
  );
  return { registration, container, worker, messages, offers, stop, reloads: () => reloads };
}
const first = lifecycle(false);
first.registration.waiting = first.worker;
first.worker.dispatchEvent(new Event('statechange'));
first.registration.waiting = null;
first.registration.active = first.worker;
first.container.controller = first.worker;
first.container.dispatchEvent(new Event('controllerchange'));
assert.equal(first.offers.length, 0, 'First installation is silent');
assert.equal(first.reloads(), 0);
first.stop();

const update = lifecycle();
update.worker.dispatchEvent(new Event('statechange'));
assert.equal(update.offers.length, 0, 'Incomplete downloads cannot be offered');
update.registration.waiting = update.worker;
update.worker.dispatchEvent(new Event('statechange'));
assert.equal(update.offers.length, 1);
assert.equal(update.reloads(), 0, 'Keep the current page until clicked');
update.offers[0]();
update.offers[0]();
assert.deepEqual(update.messages, [{ type: 'ACTIVATE_UPDATE' }]);
assert.equal(update.reloads(), 0, 'Wait for the new controller before reloading');
update.container.controller = update.worker;
update.container.dispatchEvent(new Event('controllerchange'));
update.container.dispatchEvent(new Event('controllerchange'));
assert.equal(update.reloads(), 1, 'Activation reloads exactly once');
update.stop();

const replaced = lifecycle(true, true);
replaced.registration.waiting = null;
assert.equal(replaced.offers[0](), false, 'A superseded update cannot leave the button stuck');
replaced.registration.waiting = replaced.worker;
assert.equal(replaced.offers[0](), true);
replaced.stop();

const otherTab = lifecycle(true, true);
assert.equal(otherTab.offers.length, 1, 'A previously downloaded update is offered on page load');
otherTab.registration.waiting = null;
otherTab.container.controller = otherTab.worker;
otherTab.container.dispatchEvent(new Event('controllerchange'));
assert.equal(otherTab.reloads(), 0, 'Activation by another tab must not interrupt this page');
otherTab.offers.at(-1)();
assert.equal(otherTab.reloads(), 1, 'An already active update only needs a reload');
otherTab.stop();
const hardReload = lifecycle(false, true, true);
assert.equal(
  hardReload.offers.length,
  1,
  'Hard-reloaded pages with an active SW still offer updates',
);
assert.equal(hardReload.offers[0](), true);
hardReload.container.controller = hardReload.worker;
hardReload.container.dispatchEvent(new Event('controllerchange'));
assert.equal(
  hardReload.reloads(),
  1,
  'Requested update reloads even without an initial controller',
);
hardReload.stop();
const hardReloadInstall = lifecycle(false, false, true);
assert.equal(hardReloadInstall.offers.length, 0);
hardReloadInstall.registration.waiting = hardReloadInstall.worker;
hardReloadInstall.worker.dispatchEvent(new Event('statechange'));
assert.equal(hardReloadInstall.offers.length, 1);
assert.equal(hardReloadInstall.offers[0](), true);
hardReloadInstall.container.controller = hardReloadInstall.worker;
hardReloadInstall.container.dispatchEvent(new Event('controllerchange'));
assert.equal(
  hardReloadInstall.reloads(),
  1,
  'Downloaded update on hard-reloaded page reloads on click',
);
hardReloadInstall.stop();
const hardReloadOtherTab = lifecycle(false, true, true);
assert.equal(hardReloadOtherTab.offers.length, 1);
hardReloadOtherTab.registration.waiting = null;
hardReloadOtherTab.container.controller = hardReloadOtherTab.worker;
hardReloadOtherTab.container.dispatchEvent(new Event('controllerchange'));
assert.equal(hardReloadOtherTab.reloads(), 0);
assert.equal(hardReloadOtherTab.offers.at(-1)(), true);
assert.equal(
  hardReloadOtherTab.reloads(),
  1,
  'Offered update activated elsewhere still reloads on click',
);
hardReloadOtherTab.stop();
const stopped = lifecycle();
stopped.stop();
stopped.registration.waiting = stopped.worker;
stopped.worker.dispatchEvent(new Event('statechange'));
stopped.registration.dispatchEvent(new Event('updatefound'));
stopped.container.controller = stopped.worker;
stopped.container.dispatchEvent(new Event('controllerchange'));
assert.equal(stopped.offers.length, 0, 'Disposal removes lifecycle listeners');
console.log(
  'Offline: complete build cache, explicit update activation, page/multi-tab lifecycle, isolation and execution without network',
);
