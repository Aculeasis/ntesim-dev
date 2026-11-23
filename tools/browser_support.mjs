import assert from "node:assert/strict";
import { createServer } from "node:http";
import fs from "node:fs";
import path from "node:path";

export async function serve(root, prefix = "/studio/") {
  let revision = 0;
  const server = createServer((req, res) => {
    const route = decodeURIComponent(
      new URL(req.url, "http://localhost").pathname,
    );
    const relative = route.slice(prefix.length);
    const file = path.resolve(root, relative === "" ? "index.html" : relative);
    if (
      !route.startsWith(prefix) ||
      !file.startsWith(root + path.sep) ||
      !fs.existsSync(file) ||
      !fs.statSync(file).isFile()
    ) {
      res.writeHead(404).end();
      return;
    }
    const mime = {
      ".js": "text/javascript",
      ".wasm": "application/wasm",
      ".css": "text/css",
      ".html": "text/html",
      ".png": "image/png",
      ".webp": "image/webp",
      ".svg": "image/svg+xml",
      ".json": "application/json",
    };
    let bytes = fs.readFileSync(file);
    if (relative === "sw.js" && revision)
      bytes = Buffer.from(
        bytes
          .toString()
          .replace(
            /ntesim-offline-([^']+)/,
            `ntesim-offline-update${revision}-$1`,
          ),
      );
    res
      .writeHead(200, {
        "Content-Type": mime[path.extname(file)] || "text/plain",
        "Cache-Control": "no-store",
      })
      .end(bytes);
  });
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  return {
    url: `http://127.0.0.1:${server.address().port}${prefix}`,
    upgrade: () => revision++,
    close: () => new Promise((resolve) => server.close(resolve)),
  };
}

export async function instrument(context) {
  await context.addInitScript(() => {
    const NativeWorker = window.Worker;
    window.__browserAudit = {
      workers: 0,
      messages: 0,
      bytes: 0,
      encode_ms: 0,
      post_ms: 0,
      updates: [],
      corruptResume: false,
      executionRequests: 0,
      failWithCancellationWord: false,
    };
    window.Worker = class extends NativeWorker {
      commits = new Map();
      constructor(...args) {
        super(...args);
        window.__browserAudit.workers++;
        this.addEventListener("message", ({ data }) => {
          const a = window.__browserAudit,
            start = performance.now();
          a.bytes += new TextEncoder().encode(JSON.stringify(data)).length;
          a.encode_ms += performance.now() - start;
          a.messages++;
          if (data.type === "UPDATE")
            a.updates.push({
              offset: data.result.sample_offset,
              count: data.result.samples_delta.length,
              completed: data.result.aggregate.completed,
              at_ms: performance.now(),
            });
          const count = this.commits.get(data.id);
          if (count !== undefined && data.type === "SUCCESS") {
            const offset = a.updates.at(-1)?.completed ?? 0;
            a.updates.push({
              offset,
              count,
              completed: offset + count,
              at_ms: performance.now(),
            });
            this.commits.delete(data.id);
          }
        });
      }
      postMessage(message, ...args) {
        if (message.type === "SIMULATE_NTE" || message.type === "SOURCE_SAMPLE")
          window.__browserAudit.executionRequests++;
        if (message.type === "ADD_SOURCE_OBSERVATIONS")
          this.commits.set(message.id, message.payload.observations.length);
        if (
          message.type === "SIMULATE_NTE" ||
          (message.type === "INVOKE" &&
            message.payload.action === "compile_source")
        ) {
          const a = window.__browserAudit;
          a.updates = [];
          if (a.corruptResume && message.payload.resume)
            message.payload.resume.compiled.engine_source_sha256 = "stale";
          if (a.failWithCancellationWord) {
            setTimeout(
              () =>
                this.dispatchEvent(
                  new MessageEvent("message", {
                    data: {
                      id: message.id,
                      type: "ERROR",
                      error: "upstream cancelled operation failed",
                    },
                  }),
                ),
              0,
            );
            return;
          }
        }
        const start = performance.now();
        super.postMessage(message, ...args);
        window.__browserAudit.post_ms += performance.now() - start;
      }
    };
  });
}

export async function exported(
  page,
  selector = "#export-report-btn",
  raw = false,
) {
  if (selector === "#export-report-btn") {
    const download = page.waitForEvent("download");
    await page.locator(selector).click();
    await page.locator("#export-report-json").click();
    const file = await download;
    const text = fs.readFileSync(await file.path(), "utf8");
    assert.match(file.suggestedFilename(), /-report\.json$/);
    return raw ? text : JSON.parse(text);
  }
  await page.locator(selector).click();
  const text = await page.locator("#export-content").inputValue();
  await page.keyboard.press("Escape");
  await page.locator("dialog[open]").waitFor({ state: "hidden" });
  return raw ? text : JSON.parse(text);
}

export async function source(page, text) {
  await page.locator("#tab-config").click();
  await page.locator("#script-input").fill(text);
  await page.waitForFunction(
    () => !document.querySelector("#run-btn").disabled,
  );
}

export async function completed(page) {
  await page.waitForFunction(
    () =>
      document
        .querySelector("#run-status")
        ?.textContent.startsWith("Completed"),
    null,
    { timeout: 30000 },
  );
}

export async function upgradedCache(page) {
  const deadline = performance.now() + 30000;
  while (
    !(await page.evaluate(async () => {
      const registration = await navigator.serviceWorker.getRegistration();
      const names = (await caches.keys()).filter((name) =>
        name.startsWith("ntesim-offline-"),
      );
      return (
        registration?.active?.state === "activated" &&
        !registration.installing &&
        !registration.waiting &&
        names.length === 1 &&
        names[0].includes("update1")
      );
    }))
  ) {
    assert.ok(
      performance.now() < deadline,
      "Service Worker upgrade did not settle",
    );
    await new Promise((resolve) => setTimeout(resolve, 50));
  }
}

export async function heapUsage(send) {
  await send("HeapProfiler.collectGarbage");
  const heap = await send("Runtime.getHeapUsage");
  try {
    const prototype = await send("Runtime.evaluate", {
      expression: "WebAssembly.Memory.prototype",
      objectGroup: "ntesim-audit",
    });
    const objects = await send("Runtime.queryObjects", {
      prototypeObjectId: prototype.result.objectId,
      objectGroup: "ntesim-audit",
    });
    const memories = await send("Runtime.callFunctionOn", {
      objectId: objects.objects.objectId,
      functionDeclaration:
        "function(){return this.map(memory => memory.buffer.byteLength)}",
      returnByValue: true,
    });
    heap.wasm_memory_bytes = memories.result.value;
    assert.ok(
      heap.wasm_memory_bytes.length > 0,
      "Actual WASM memory must be measured",
    );
    return heap;
  } finally {
    await send("Runtime.releaseObjectGroup", { objectGroup: "ntesim-audit" });
  }
}

export async function workerHeap(cdp, marker = "", excluded = []) {
  const { targetInfos } = await cdp.send("Target.getTargets");
  const worker = targetInfos.find(
    (t) =>
      t.type === "worker" &&
      !excluded.includes(t.targetId) &&
      /worker-.*\.js(?:\?.*)?$/.test(t.url) &&
      t.url.includes(marker),
  );
  assert.ok(worker, "Actual bundled Worker must be present");
  const { sessionId } = await cdp.send("Target.attachToTarget", {
    targetId: worker.targetId,
    flatten: false,
  });
  let serial = 0;
  const send = (method, params = {}) => {
    const id = ++serial;
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        cdp.off("Target.receivedMessageFromTarget", receive);
        reject(Error("Worker heap measurement timed out"));
      }, 5000);
      const receive = (event) => {
        if (event.sessionId !== sessionId) return;
        const message = JSON.parse(event.message);
        if (message.id !== id) return;
        clearTimeout(timer);
        cdp.off("Target.receivedMessageFromTarget", receive);
        if (message.error) reject(Error(message.error.message));
        else resolve(message.result);
      };
      cdp.on("Target.receivedMessageFromTarget", receive);
      cdp
        .send("Target.sendMessageToTarget", {
          sessionId,
          message: JSON.stringify({ id, method, params }),
        })
        .catch(reject);
    });
  };
  try {
    return await heapUsage(send);
  } catch (error) {
    // Unassigned pool workers have no WASM instance. Measure an initialized one.
    if (error.message === "Actual WASM memory must be measured")
      return await workerHeap(cdp, marker, [...excluded, worker.targetId]);
    throw error;
  } finally {
    await cdp.send("Target.detachFromTarget", { sessionId });
  }
}

export async function nativeSearch(page, workerUrl, wasmUrl, request) {
  return page.evaluate(
    async ({ workerUrl, wasmUrl, request }) => {
      const invoke = (payload) =>
        new Promise((resolve, reject) => {
          const worker = new Worker(workerUrl, { type: "module" });
          window.__searchWorker = worker;
          const timer = setTimeout(() => {
            worker.terminate();
            reject(Error("Native search timed out"));
          }, 30000);
          worker.onerror = (event) => {
            clearTimeout(timer);
            worker.terminate();
            reject(Error(event.message));
          };
          worker.onmessage = ({ data }) => {
            if (data.type === "SUCCESS" || data.type === "ERROR") {
              clearTimeout(timer);
              if (data.type === "ERROR") {
                worker.terminate();
                reject(Error(data.error));
              } else resolve(data.result.data);
            }
          };
          worker.postMessage({
            id: 1,
            type: "INVOKE",
            wasmSource: wasmUrl,
            payload: { action: "search_source_reports", payload },
          });
        });
      const prefix = await invoke({ request, max_new_trials: 2 });
      if (prefix.search.complete || prefix.search.checkpoint.runs.length !== 2)
        throw Error("Search prefix was not retained");
      window.__searchWorker.terminate();
      const result = await invoke({
        request,
        checkpoint_json: prefix.checkpoint_json,
        max_new_trials: 2,
      });
      window.__searchResult = result;
      return {
        complete: result.search.complete,
        success: result.search.success,
        ranking: result.search.ranking,
        engine_source_sha256: result.search.engine_source_sha256,
        events: result.search.checkpoint.events_charged,
        retained_receipt_bytes: result.search.checkpoint.retained_receipt_bytes,
        report_bytes: new TextEncoder().encode(result.report_json).length,
        checkpoint_bytes: new TextEncoder().encode(result.checkpoint_json)
          .length,
        prefix_trials: prefix.search.checkpoint.runs.length,
        completed: result.search.checkpoint.runs.length,
      };
    },
    { workerUrl, wasmUrl, request },
  );
}

export async function contrast(page, selectors) {
  const values = await page.evaluate((selectors) => {
    const rgb = (s) => [...s.matchAll(/[\d.]+/g)].map((m) => Number(m[0]));
    const luminance = (c) =>
      c
        .slice(0, 3)
        .map((v) => v / 255)
        .map((v) => (v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4))
        .reduce((sum, v, i) => sum + v * [0.2126, 0.7152, 0.0722][i], 0);
    return selectors.map((selector) => {
      const el = document.querySelector(selector),
        style = getComputedStyle(el);
      let background = el,
        color;
      while (background) {
        color = rgb(getComputedStyle(background).backgroundColor);
        if (color.length === 3 || color[3] === 1) break;
        background = background.parentElement;
      }
      const a = luminance(rgb(style.color)),
        b = luminance(color);
      return {
        selector,
        ratio: (Math.max(a, b) + 0.05) / (Math.min(a, b) + 0.05),
      };
    });
  }, selectors);
  for (const value of values)
    assert.ok(
      value.ratio >= 4.5,
      `Text contrast ${value.selector}: ${value.ratio}`,
    );
  return values;
}
