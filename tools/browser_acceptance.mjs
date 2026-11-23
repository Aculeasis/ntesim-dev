import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import os from "node:os";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { fileURLToPath, pathToFileURL } from "node:url";
import { resumeInBrowser, importResumedReport } from "./browser_resume.mjs";
import {
  portableReportAcceptance,
  offlineReportAcceptance,
} from "./browser_report_exports.mjs";
import {
  teamListAcceptance,
  teamListBusyAcceptance,
  pickerHeaderAcceptance,
} from "./browser_team_list.mjs";
import {
  serve,
  instrument,
  exported,
  source,
  completed,
  upgradedCache,
  heapUsage,
  workerHeap,
  nativeSearch,
  contrast,
} from "./browser_support.mjs";

const root = fileURLToPath(new URL("../", import.meta.url));
const clockBaseline = process.argv.includes("--clock-baseline");
const throughputBaseline = process.argv.includes("--throughput-baseline");
const throughputOnly =
  throughputBaseline || process.argv.includes("--throughput");
const archivedBaseline = clockBaseline || throughputBaseline;
const clockOnly =
  clockBaseline ||
  throughputOnly ||
  process.argv.includes("--clock-performance");
const baselineRoot = path.resolve(
  root,
  throughputBaseline
    ? process.env.NTESIM_THROUGHPUT_REFERENCE ||
        "cache/exact-throughput/original"
    : "cache/relative-duration/baseline",
);
const baselineIdentity = archivedBaseline
  ? JSON.parse(
      fs.readFileSync(
        path.join(baselineRoot, "artifact-identity.json"),
        "utf8",
      ),
    )
  : null;
const site = path.resolve(
  process.env.NTESIM_BROWSER_SITE ||
    (archivedBaseline
      ? path.join(baselineRoot, "site")
      : path.join(root, "target/web")),
);
const modulePath = process.env.NTESIM_PLAYWRIGHT_MODULE;
const output = path.resolve(
  root,
  process.env.NTESIM_BROWSER_OUTPUT || "cache/browser-acceptance/current",
);
assert.ok(output.startsWith(path.resolve(root, "cache") + path.sep));
fs.mkdirSync(output, { recursive: true });
const receipt = {
  passed: false,
  status: "running",
  measured_utc: new Date().toISOString(),
  platform: os.platform(),
  release: os.release(),
  cpu: os.cpus()[0].model,
  cpu_limit: 4,
  node: process.version,
  runs: [],
  checks: [],
};
const save = () =>
  fs.writeFileSync(
    path.join(output, "measurement.json"),
    JSON.stringify(receipt, null, 2) + "\n",
  );
save();
let server, browser, context, page, browserCDP, pageCDP;
const errors = [];
const cpu = async () =>
  (await browserCDP.send("SystemInfo.getProcessInfo")).processInfo.reduce(
    (n, p) => n + p.cpuTime,
    0,
  );
const sha = (bytes) => createHash("sha256").update(bytes).digest("hex");
const buildFiles = (directory) =>
  fs
    .readdirSync(directory, { withFileTypes: true })
    .flatMap((entry) =>
      entry.isDirectory()
        ? buildFiles(path.join(directory, entry.name))
        : [path.join(directory, entry.name)],
    );
const artifactHashes = () =>
  Object.fromEntries(
    buildFiles(site)
      .sort()
      .map((file) => [
        "target/web/" + path.relative(site, file).replaceAll("\\", "/"),
        sha(fs.readFileSync(file)),
      ]),
  );
const audit = () =>
  page.evaluate(() => ({
    messages: window.__browserAudit.messages,
    bytes: window.__browserAudit.bytes,
    encode_ms: window.__browserAudit.encode_ms,
    post_ms: window.__browserAudit.post_ms,
    updates: window.__browserAudit.updates,
  }));
const retainedMemory = (...heaps) =>
  heaps.reduce(
    (sum, h) =>
      sum +
      h.usedSize +
      h.backingStorageSize +
      h.wasm_memory_bytes.reduce((n, bytes) => n + bytes, 0),
    0,
  );
async function measure(name, text) {
  await source(page, text);
  const before = await audit();
  const baseline = await cpu(),
    start = performance.now();
  await page.locator("#run-btn").click();
  await completed(page);
  const wall = performance.now() - start,
    cpuSeconds = (await cpu()) - baseline;
  const heap = await heapUsage(pageCDP.send.bind(pageCDP)),
    worker = await workerHeap(browserCDP);
  const result = await exported(page);
  const transport = await audit();
  for (const field of ["messages", "bytes", "encode_ms", "post_ms"])
    transport[field] -= before[field];
  let offset = 0;
  for (const update of transport.updates) {
    assert.equal(update.offset, offset);
    offset += update.count;
    assert.equal(update.completed, offset);
  }
  assert.equal(offset, result.trials);
  assert.equal(transport.updates[0].completed, 1);
  assert.equal(
    result.report.engine_source_sha256,
    receipt.engine_source_sha256,
    "Production engine must match current source",
  );
  const row = {
    name,
    source_sha256: sha(text),
    engine_source_sha256: result.report.engine_source_sha256,
    requested: result.trials,
    completed: result.samples.length,
    events: result.samples.reduce((n, s) => n + s.events, 0),
    wall_ms: wall,
    cpu_seconds: cpuSeconds,
    main_heap: heap,
    worker_heap: worker,
    serialized_retention_bytes: Buffer.byteLength(JSON.stringify(result)),
    transport,
  };
  assert.equal(result.samples.length, result.trials);
  assert.ok(wall <= 30000 && cpuSeconds <= 30, "Browser workload time budget");
  const memory = retainedMemory(heap, worker);
  row.retained_memory_bytes = memory;
  assert.ok(memory <= 512 * 1024 * 1024, "Retained JS/WASM memory budget");
  assert.ok(row.serialized_retention_bytes <= 64 * 1024 * 1024);
  assert.ok(row.events <= 100000);
  receipt.runs.push(row);
  save();
  return result;
}

async function clockWorkloads() {
  const party = `mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
    zero char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
    sakiri char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
    edgar char lvl=80/80 awakening=0 talent=1,1,1 qte=1;`;
  const workloads = [
    [
      "Clock no-Ultimate control",
      `options duration=120 iteration=8 crit=fixed; ${party}
      target hp=1000000000 def=0 resist=0; active mint;
      repeat 10 { mint attack; wait(1); }`,
    ],
    [
      "Clock ordinary multi-character Ultimates",
      `options duration=120 iteration=8 crit=fixed; ${party}
      target hp=1000000000 def=0 resist=0; active mint;
      mint burst; zero burst; sakiri burst; edgar burst;`,
    ],
    [
      "Clock continuing Hotori",
      `options duration=120 iteration=8 crit=fixed;
      mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
      hotori char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100 start_energy=0;
      hathor char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_cycle=100;
      zero char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
      target hp=1000000000 def=0 resist=0; active mint;
      wait(20); hotori swap; wait(1); hathor swap; wait(1); hotori burst;
      while .hotori.time_stop.active { hotori attack[window=attack]; }
      wait(20); hotori burst;
      while .hotori.time_stop.active { hotori attack[window=attack]; }`,
    ],
  ];
  for (const [name, text] of workloads) {
    const result = await measure(name, text);
    const row = receipt.runs.at(-1);
    // The archived pre-change engine retains its original report format only
    // inside this measurement boundary; Studio never accepts obsolete reports.
    row.clock_contract = clockBaseline
      ? "archived-single-combat-clock"
      : result.report.clock_contract;
    row.absolute_elapsed_us = clockBaseline
      ? result.report.elapsed_us
      : result.report.absolute_elapsed_us;
    row.relative_elapsed_us = clockBaseline
      ? null
      : result.report.relative_elapsed_us;
    row.matching_absolute_prefix_us = 120_000_000;
    const mix = (through) => {
      const hits = result.report.hit_receipts.filter(
        (hit) => hit.time_us <= through,
      );
      return {
        hits: hits.length,
        damage: hits.reduce((sum, hit) => sum + hit.rounded_damage, 0),
        packets: Object.fromEntries(
          [...new Set(hits.map((hit) => hit.damage_effect_id))]
            .sort()
            .map((packet) => [
              packet,
              hits.filter((hit) => hit.damage_effect_id === packet).length,
            ]),
        ),
        trace: Object.fromEntries(
          [...new Set(result.report.trace.map((event) => event.event_kind))]
            .sort()
            .map((kind) => [
              kind,
              result.report.trace.filter(
                (event) =>
                  event.event_kind === kind && event.time_us <= through,
              ).length,
            ]),
        ),
      };
    };
    row.absolute_prefix_mix = mix(120_000_000);
    row.complete_mix = mix(row.absolute_elapsed_us);
    row.raw_json_bytes = Buffer.byteLength(
      await exported(page, "#export-report-btn", true),
    );
    row.compact_json_bytes = Buffer.byteLength(JSON.stringify(result));
    const downloading = page.waitForEvent("download");
    await page.locator("#export-report-btn").click();
    await page.locator("#export-report-nter").click();
    const compressed = await downloading;
    row.compressed_report_bytes = fs.statSync(await compressed.path()).size;
    row.pause_records = result.report.timer_pauses?.length ?? 0;
    row.buff_definitions = result.report.buff_catalog?.length ?? 0;
    row.timer_definitions = result.report.timer_catalog?.length ?? 0;
    row.state_boundaries = result.report.state_timeline.length;
    await page.locator("#tab-sample").click();
    const slider = page.getByRole("slider", { name: "Sample time" });
    await slider.waitFor();
    row.seek = await slider.evaluate(async (element) => {
      const start = performance.now();
      for (let index = 0; index < 30; index++) {
        element.value = String((Number(element.max) * ((index * 7) % 30)) / 30);
        element.dispatchEvent(new Event("input", { bubbles: true }));
        await new Promise(requestAnimationFrame);
      }
      return { seeks: 30, wall_ms: performance.now() - start };
    });
    await page.locator("#tab-results").click();
    save();
  }
  receipt.checks.push(
    "Source-matched clock workloads: actual production Worker, 120-second absolute prefix mix, complete work, CPU/wall, retained memory, report encodings and absolute seeks",
  );
}

async function throughputWorkloads() {
  // This explicit, long development experiment has no ordinary acceptance
  // budget override. All ordinary workloads retain their existing limits.
  const text = fs.readFileSync(
    path.join(root, "presets/studio/zankou-insight.nte"),
    "utf8",
  );
  const startupProfile = process.argv.includes("--startup-profile");
  receipt.startup_profile = startupProfile;
  for (const label of startupProfile
    ? ["cold", "repeated"]
    : ["cold", "repeated", "edited", "replacement"]) {
    if (label === "replacement") {
      await page.reload();
      await page.waitForFunction(
        () => !document.querySelector("#run-btn")?.disabled,
      );
    }
    const authored =
      label === "edited" ? text + "\n# startup edited-source probe\n" : text;
    await source(page, authored);
    const transportBefore = await audit();
    const traceEvents = [];
    const collectTrace = (data) => traceEvents.push(...data.value);
    if (startupProfile) {
      browserCDP.on("Tracing.dataCollected", collectTrace);
      await browserCDP.send("Tracing.start", {
        categories: "v8,disabled-by-default-v8.wasm",
        transferMode: "ReportEvents",
      });
    }
    const cpuBefore = await cpu();
    const start = performance.now();
    await page.locator("#run-btn").click();
    if (startupProfile) {
      await page.waitForFunction(
        () => window.__throughputAudit.first_render_ms !== null,
      );
      const finished = new Promise((resolve) =>
        browserCDP.once("Tracing.tracingComplete", resolve),
      );
      await browserCDP.send("Tracing.end");
      await finished;
      browserCDP.off("Tracing.dataCollected", collectTrace);
      fs.writeFileSync(
        path.join(output, `startup-trace-${label}.json`),
        JSON.stringify({ traceEvents }),
      );
    }
    await page.waitForFunction(
      () =>
        document
          .querySelector("#run-status")
          ?.textContent.startsWith("Completed"),
      null,
      { timeout: 600000 },
    );
    const completed_ms = performance.now() - start;
    await page.evaluate(
      () =>
        new Promise((resolve) =>
          requestAnimationFrame(() => requestAnimationFrame(resolve)),
        ),
    );
    const rendered_ms = performance.now() - start;
    const cpuSeconds = (await cpu()) - cpuBefore;
    const result = await exported(page);
    assert.equal(
      result.report.engine_source_sha256,
      receipt.engine_source_sha256,
    );
    assert.equal(result.samples.length, 1000);
    assert.equal(result.compiled.input.source, authored);
    assert.equal(result.samples[0].seed, 42);
    assert.equal(result.samples.at(-1).seed, 1041);
    assert.ok(
      result.samples.every(
        (sample, index) =>
          sample.relative_elapsed_us === 120000000 &&
          sample.seed === 42 + index,
      ),
    );
    const transport = await audit();
    for (const field of ["messages", "bytes", "encode_ms", "post_ms"])
      transport[field] -= transportBefore[field];
    const timeline = await page.evaluate(() => ({
      ...window.__throughputAudit,
      hardware_concurrency: navigator.hardwareConcurrency,
      device_memory: navigator.deviceMemory,
    }));
    const heap = await heapUsage(pageCDP.send.bind(pageCDP));
    const { targetInfos } = await browserCDP.send("Target.getTargets");
    const workerHeaps = [];
    for (const target of targetInfos.filter(
      (target) =>
        target.type === "worker" && /worker-.*\.js(?:\?.*)?$/.test(target.url),
    )) {
      const excluded = targetInfos
        .filter((other) => other.targetId !== target.targetId)
        .map((other) => other.targetId);
      workerHeaps.push(await workerHeap(browserCDP, "", excluded));
    }
    const row = {
      name: label,
      source_sha256: sha(authored),
      completed_ms,
      completed_render_ms: rendered_ms,
      cpu_seconds: cpuSeconds,
      timeline,
      main_heap: heap,
      worker_heaps: workerHeaps,
      retained_memory_bytes: retainedMemory(heap, ...workerHeaps),
      transport,
      serialized_retention_bytes: Buffer.byteLength(JSON.stringify(result)),
      samples: result.samples,
      aggregate: result.aggregate,
    };
    receipt.runs.push(row);
    save();
  }
}
try {
  if (!modulePath)
    throw Error(
      "Set NTESIM_PLAYWRIGHT_MODULE to the installed Playwright index.mjs.",
    );
  const { chromium } = await import(
    pathToFileURL(path.resolve(modulePath)).href
  );
  receipt.playwright = JSON.parse(
    fs.readFileSync(path.join(path.dirname(modulePath), "package.json")),
  ).version;
  const environment = JSON.parse(
    execFileSync(
      "python",
      [
        "-c",
        'import sys,json; sys.path.insert(0, sys.argv[1]); from cargo import engine_source_sha256; from cpu_limit import limit_cpus; print(json.dumps({"engine":engine_source_sha256(),"cpus":limit_cpus(4)}))',
        path.join(root, "tools"),
      ],
      { encoding: "utf8" },
    ),
  );
  receipt.engine_source_sha256 = archivedBaseline
    ? baselineIdentity.engine_source_sha256
    : environment.engine;
  receipt.clock_measurement = clockBaseline
    ? "archived-before"
    : "current-after";
  receipt.cpu_affinity = environment.cpus;
  receipt.artifacts = artifactHashes();
  if (throughputBaseline) {
    assert.deepEqual(
      receipt.artifacts,
      Object.fromEntries(
        Object.entries(baselineIdentity.assets).map(([name, digest]) => [
          "target/web/" + name,
          digest,
        ]),
      ),
      "Archived throughput site differs from its manifest",
    );
  }
  receipt.tools = Object.fromEntries(
    [
      "browser_acceptance.mjs",
      "browser_support.mjs",
      "browser_team_list.mjs",
      "browser_resume.mjs",
      "browser_report_exports.mjs",
      "verify_browser_trials.mjs",
    ].map((file) => [
      file,
      sha(fs.readFileSync(path.join(root, "tools", file))),
    ]),
  );
  const deployedWasm = buildFiles(site).filter((file) =>
    file.endsWith(".wasm"),
  );
  assert.equal(deployedWasm.length, 2);
  assert.equal(
    sha(
      fs.readFileSync(
        deployedWasm.find((file) => file.includes("ntesim_wasm_bg")),
      ),
    ),
    sha(
      fs.readFileSync(
        archivedBaseline
          ? path.join(baselineRoot, "wasm-bindgen/ntesim_wasm_bg.wasm")
          : path.join(root, "target/wasm-bindgen/ntesim_wasm_bg.wasm"),
      ),
    ),
    "Production WASM is stale",
  );
  server = await serve(site, "/ntesim-dev/");
  browser = await chromium.launch({
    channel: process.env.NTESIM_BROWSER_CHANNEL || "chrome",
    headless: true,
  });
  context = await browser.newContext({
    viewport: { width: 1440, height: 1000 },
    reducedMotion: "reduce",
    acceptDownloads: true,
  });
  await instrument(context);
  if (throughputOnly) {
    await context.addInitScript(() => {
      const audit = (window.__throughputAudit = {
        created: 0,
        run_at_ms: null,
        requests: [],
        responses: [],
        dom: [],
        first_render_ms: null,
      });
      const NativeWorker = window.Worker;
      window.Worker = class extends NativeWorker {
        constructor(...args) {
          super(...args);
          const worker = audit.created++;
          this.__measurementId = worker;
          this.addEventListener("message", ({ data }) => {
            if (audit.run_at_ms === null) return;
            const result = data.result?.data ?? data.result;
            audit.responses.push({
              worker,
              id: data.id,
              type: data.type,
              stage: data.stage,
              first_execution:
                result?.first_execution != null || data.type === "UPDATE",
              at_ms: performance.now() - audit.run_at_ms,
            });
          });
        }
        postMessage(message, ...args) {
          if (audit.run_at_ms !== null)
            audit.requests.push({
              worker: this.__measurementId,
              id: message.id,
              type: message.type,
              action: message.payload?.action,
              offset: message.payload?.start_offset,
              count: message.payload?.count,
              at_ms: performance.now() - audit.run_at_ms,
            });
          super.postMessage(message, ...args);
        }
      };
      document.addEventListener(
        "click",
        (event) => {
          if (event.target.closest?.("#run-btn"))
            Object.assign(audit, {
              run_at_ms: performance.now(),
              requests: [],
              responses: [],
              dom: [],
              first_render_ms: null,
            });
        },
        true,
      );
      document.addEventListener("DOMContentLoaded", () => {
        let previous;
        new MutationObserver(() => {
          if (audit.run_at_ms === null) return;
          const status = document.querySelector("#run-status")?.textContent;
          if (status !== previous) {
            audit.dom.push({
              status,
              at_ms: performance.now() - audit.run_at_ms,
            });
            previous = status;
          }
          if (
            audit.first_render_ms === null &&
            /iterations\s+1\s*\/\s*1000/.test(
              document.querySelector("#result-meta")?.textContent ?? "",
            )
          )
            audit.first_render_ms = performance.now() - audit.run_at_ms;
        }).observe(document.body, {
          childList: true,
          subtree: true,
          characterData: true,
        });
      });
    });
  }
  page = await context.newPage();
  page.setDefaultTimeout(30000);
  page.on("pageerror", (error) => errors.push(String(error)));
  browserCDP = await browser.newBrowserCDPSession();
  pageCDP = await context.newCDPSession(page);
  receipt.browser = browser.version();
  receipt.wasm_sha256 = sha(
    fs.readFileSync(
      archivedBaseline
        ? path.join(baselineRoot, "wasm-bindgen/ntesim_wasm_bg.wasm")
        : path.join(root, "target/wasm-bindgen/ntesim_wasm_bg.wasm"),
    ),
  );
  receipt.web_index_sha256 = sha(
    fs.readFileSync(path.join(site, "index.html")),
  );
  await page.goto(server.url);
  await page.waitForFunction(
    () => document.querySelector("#run-btn")?.disabled === false,
  );
  await page.waitForFunction(() => {
    const images = [...document.querySelectorAll(".character-card img")];
    return (
      images.length >= 4 &&
      images.every((image) => image.complete && image.naturalWidth > 0)
    );
  });
  if (throughputOnly) await throughputWorkloads();
  else await clockWorkloads();
  if (!clockOnly) {
    await page.locator("#tab-config").click();
    const workerFile = Object.keys(receipt.artifacts).find((file) =>
      /\/worker-.*\.js$/.test(file),
    );
    const wasmFile = Object.keys(receipt.artifacts).find(
      (file) => file.includes("ntesim_wasm_bg") && file.endsWith(".wasm"),
    );
    const workerURL = new URL(
      workerFile.replace("target/web/", ""),
      server.url,
    );
    const wasmURL = new URL(wasmFile.replace("target/web/", ""), server.url);
    const build = JSON.parse(
      fs.readFileSync(path.join(site, "build-info.json")),
    ).studio_build;
    const repositoryLink = page.getByRole("link", {
      name: "GitHub repository",
      exact: true,
    });
    const sourceLink = page.getByRole("link", {
      name: `Build source: ${build.label}`,
      exact: true,
    });
    assert.equal(
      await repositoryLink.getAttribute("href"),
      build.repository_url,
    );
    assert.equal(await sourceLink.getAttribute("href"), build.url);
    assert.equal(await sourceLink.getAttribute("title"), build.label);
    const instant = await page.evaluate(
      async ({ workerURL, wasmURL }) => {
        const worker = new Worker(workerURL, { type: "module" });
        let timeout;
        try {
          return await new Promise((resolve, reject) => {
            timeout = setTimeout(
              () =>
                reject(new Error("Timed out reading the served WASM version")),
              10000,
            );
            worker.onerror = (event) => reject(new Error(event.message));
            worker.onmessage = ({ data }) => {
              if (data.type === "SUCCESS")
                resolve(data.result.build_started_at);
              if (data.type === "ERROR") reject(new Error(data.error));
            };
            worker.postMessage({ id: 1, type: "VERSION", wasmSource: wasmURL });
          });
        } finally {
          clearTimeout(timeout);
          worker.terminate();
        }
      },
      { workerURL: workerURL.href, wasmURL: wasmURL.href },
    );
    assert.ok(Number.isFinite(Date.parse(instant)));
    await page.waitForFunction(
      (value) =>
        document.querySelector(".build-identity time")?.dateTime === value,
      instant,
    );
    assert.equal(
      await page.locator(".build-identity time").textContent(),
      await page.evaluate((value) => {
        const parts = Object.fromEntries(
          new Intl.DateTimeFormat("en-GB", {
            day: "2-digit",
            month: "2-digit",
            year: "2-digit",
            hour: "2-digit",
            minute: "2-digit",
            hourCycle: "h23",
          })
            .formatToParts(new Date(value))
            .map(({ type, value }) => [type, value]),
        );
        return `${parts.day}.${parts.month}.${parts.year} · ${parts.hour}:${parts.minute}`;
      }, instant),
    );
    await repositoryLink.focus();
    await page.keyboard.press("Tab");
    assert.equal(
      await sourceLink.evaluate((node) => node === document.activeElement),
      true,
    );
    assert.notEqual(
      await sourceLink.evaluate((node) => getComputedStyle(node).outlineStyle),
      "none",
    );
    await sourceLink.evaluate((node) => node.blur());
    for (const width of [1440, 390]) {
      await page.setViewportSize({ width, height: 1000 });
      const sourceBounds = await page.locator(".build-source").boundingBox(),
        dateBounds = await page.locator(".build-identity time").boundingBox();
      assert.ok(Math.abs(sourceBounds.y - dateBounds.y) < 1);
      assert.ok(Math.abs(sourceBounds.height - dateBounds.height) < 1);
      assert.ok(
        Math.abs(dateBounds.x - sourceBounds.x - sourceBounds.width - 12) < 1,
      );
      assert.ok(
        await page.evaluate(
          () => document.documentElement.scrollWidth <= innerWidth,
        ),
      );
      await page
        .locator(".site-header")
        .screenshot({ path: path.join(output, `header-${width}.png`) });
      // Long refs remain title-inspectable and cannot expand the page.
      await sourceLink.evaluate((node) => {
        node.textContent = "release/".repeat(40);
      });
      assert.ok(
        await page.evaluate(
          () => document.documentElement.scrollWidth <= innerWidth,
        ),
      );
      await sourceLink.evaluate((node, label) => {
        node.textContent = label;
      }, build.label);
    }
    await page.setViewportSize({ width: 1440, height: 1000 });
    receipt.studio_build = build;
    receipt.checks.push(
      "Header source/repository links, keyboard focus, fixed local timestamp, desktop/390px long refs at /ntesim-dev/",
    );
    // Exercise every bundled image under a directory prefix, including off-screen search assets.
    const artworkFiles = buildFiles(path.join(site, "assets/artwork"));
    for (const file of artworkFiles) {
      assert.ok(
        await page.evaluate(
          async (url) => {
            const image = new Image();
            image.src = url;
            await image.decode();
            return image.naturalWidth > 0 && image.naturalHeight > 0;
          },
          new URL("assets/artwork/" + path.basename(file), server.url).href,
        ),
      );
    }
    receipt.checks.push(
      `${artworkFiles.length} bundled artwork images decoded under /ntesim-dev/`,
    );
    await page.screenshot({ path: path.join(output, "artwork.png") });
    await page.evaluate(async () => {
      await navigator.serviceWorker.ready;
    });
    await pickerHeaderAcceptance(page, output);
    await teamListAcceptance(page, root, output);
    receipt.checks.push(
      "Team list: all presets, native ranking, filters, keyboard, no simulation work and 390px layout",
    );
    await page.reload();
    await page.waitForFunction(
      () => document.querySelector("#run-btn")?.disabled === false,
    );
    const small =
      "options duration=10 iteration=1000 seed=42 crit=random;\nmint char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=0;\ntarget hp=100000000 def=0 resist=0; active mint;\nwhile .time < 3 { mint attack; wait(1); }";
    const full = await measure("1000 random source trials", small);
    assert.equal(full.samples[0].seed, 42);
    assert.equal(full.samples.at(-1).seed, 1041);
    for (const file of [
      "examples/mint-native-cycles.nte",
      "examples/baselines/daffodil.nte",
      "examples/incoming-shield-cycles.nte",
    ]) {
      let text = fs.readFileSync(path.join(root, file), "utf8");
      text = /iteration=\d+/.test(text)
        ? text.replace(/iteration=\d+/, "iteration=3")
        : text.replace("options ", "options iteration=3 ");
      await measure(file, text);
    }
    // A stopped run stays bound to its executed source when the draft changes.
    receipt.portable_reports = await portableReportAcceptance(
      page,
      root,
      output,
    );
    receipt.checks.push(receipt.portable_reports.checks);
    await source(page, small.replace("iteration=1000", "iteration=10000"));
    await page.locator("#run-btn").click();
    await page.locator("#stop-btn").waitFor();
    await page.locator("#result-meta").waitFor({ state: "visible" });
    await page.locator("#export-report-btn").click();
    assert.equal(await page.locator("#export-report-png").isDisabled(), true);
    assert.match(
      await page.locator("#report-png-reason").innerText(),
      /finish or stop/,
    );
    await page.keyboard.press("Escape");
    await teamListBusyAcceptance(page, exported);
    receipt.checks.push(
      "Team list busy protection and prior executed result identity",
    );
    await source(page, small);
    await page.locator("#run-btn").click();
    await page.locator("#stop-btn").waitFor();
    const stopStarted = performance.now();
    await page.locator("#result-meta").waitFor({ state: "visible" });
    await page.locator("#stop-btn").click();
    await page.waitForFunction(() =>
      document
        .querySelector("#run-status")
        ?.textContent.startsWith("Simulation stopped"),
    );
    receipt.stop_latency_ms = performance.now() - stopStarted;
    const prefix = await exported(page);
    assert.ok(prefix.samples.length > 0 && prefix.samples.length < 1000);
    const workers = await page.evaluate(() => window.__browserAudit.workers);
    await source(page, small + "\n# changed editor draft");
    await page.locator("#tab-results").click();
    const resumeStarted = performance.now();
    const resumedResponse = await resumeInBrowser(
      page,
      server.url,
      receipt.artifacts,
      prefix,
    );
    await importResumedReport(page, prefix, resumedResponse);
    await completed(page);
    const resumed = await exported(page);
    assert.deepEqual(resumed.samples, full.samples);
    assert.deepEqual(resumed.aggregate.metrics, full.aggregate.metrics);
    assert.equal(resumed.source, small);
    assert.ok(
      (await page.evaluate(() => window.__browserAudit.workers)) > workers,
    );
    receipt.resume = {
      retained_trials: prefix.samples.length,
      wall_ms: performance.now() - resumeStarted,
      active_work_ms:
        resumed.aggregate.elapsed_ms - prefix.aggregate.elapsed_ms,
    };
    receipt.checks.push(
      "Stop retains a strict delivered prefix; fresh Worker resume matches uninterrupted seeds/statistics and ignores the edited draft",
    );
    // Simulate a saved compiled artifact from stale assets without changing production code.
    // Reuse the verified incomplete prefix instead of racing another short run.
    const stalePrefix = structuredClone(prefix);
    await page.evaluate(() => {
      window.__browserAudit.corruptResume = true;
    });
    const rejectedResume = await resumeInBrowser(
      page,
      server.url,
      receipt.artifacts,
      stalePrefix,
    );
    assert.equal(rejectedResume.type, "ERROR");
    assert.match(rejectedResume.error, /identity|mismatch|differ|fingerprint/);
    await page.evaluate(() => {
      window.__browserAudit.corruptResume = false;
    });
    await source(page, small.replace("iteration=1000", "iteration=2"));
    await page.locator("#run-btn").click();
    await completed(page);
    receipt.checks.push(
      "Incompatible compiled identity remains a visible failure; next run recovers in a fresh Worker",
    );
    await page.evaluate(() => {
      window.__browserAudit.failWithCancellationWord = true;
    });
    await source(page, small.replace("iteration=1000", "iteration=2"));
    await page.locator("#run-btn").click();
    await page.waitForFunction(() =>
      document
        .querySelector("#run-status")
        ?.textContent.includes("Execution failed: upstream cancelled"),
    );
    await page.evaluate(() => {
      window.__browserAudit.failWithCancellationWord = false;
    });
    receipt.checks.push(
      "Error prose containing cancelled remains Failed, never Stopped",
    );
    // Native paired comparison is the browser bounded candidate-selection workload.
    const candidate = fs.readFileSync(
      path.join(root, "examples/mint-native-cycles.nte"),
      "utf8",
    );
    await measure(
      "Mint comparison baseline",
      candidate.replace(/iteration=\d+/, "iteration=2"),
    );
    await page
      .locator("#comparison-source")
      .fill(
        fs.readFileSync(
          path.join(root, "examples/mint-native-cycles-r1.nte"),
          "utf8",
        ),
      );
    await page.getByLabel("Comparison seeds", { exact: true }).fill("42,43");
    const start = performance.now(),
      baselineCPU = await cpu();
    await page.locator("#compare-sources").click();
    await page.locator("#export-comparison").waitFor();
    const comparisonText = await exported(page, "#export-comparison", true);
    const search = JSON.parse(comparisonText);
    assert.equal(search.complete, true);
    assert.equal(search.pairs.length, 2);
    receipt.comparison = {
      wall_ms: performance.now() - start,
      cpu_seconds: (await cpu()) - baselineCPU,
      bytes: Buffer.byteLength(comparisonText),
      engine_source_sha256: search.baseline.engine_source_sha256,
      pairs: search.pairs.length,
      events: [...search.baseline.runs, ...search.candidate.runs].reduce(
        (n, run) => n + run.execution.report.total_events_processed,
        0,
      ),
      complete: search.complete,
      main_heap: await heapUsage(pageCDP.send.bind(pageCDP)),
      worker_heap: await workerHeap(browserCDP),
    };
    receipt.comparison.retained_memory_bytes = retainedMemory(
      receipt.comparison.main_heap,
      receipt.comparison.worker_heap,
    );
    assert.ok(
      receipt.comparison.retained_memory_bytes <= 512 * 1024 * 1024 &&
        receipt.comparison.wall_ms <= 30000 &&
        receipt.comparison.cpu_seconds <= 30 &&
        receipt.comparison.bytes <= 64 * 1024 * 1024,
    );
    const searchRequest = {
      schema_version: 1,
      experiment: {
        schema_version: 1,
        seeds: [42, 43],
        max_events_per_run: 10000,
        max_total_events: 100000,
        max_receipt_bytes: 33554432,
      },
      objective: { metric: "mean_dps", direction: "maximize" },
      candidates: [
        {
          id: "mind-royale-r1",
          source: fs.readFileSync(
            path.join(root, "examples/mint-native-cycles-r1.nte"),
            "utf8",
          ),
        },
        { id: "mind-royale-r5", source: candidate },
      ],
    };
    const searchCPU = await cpu(),
      searchStarted = performance.now();
    receipt.search = await nativeSearch(
      page,
      new URL("?source-search-audit", workerURL).href,
      wasmURL.href,
      searchRequest,
    );
    receipt.search.wall_ms = performance.now() - searchStarted;
    receipt.search.cpu_seconds = (await cpu()) - searchCPU;
    receipt.search.main_heap = await heapUsage(pageCDP.send.bind(pageCDP));
    receipt.search.worker_heap = await workerHeap(
      browserCDP,
      "source-search-audit",
    );
    receipt.search.retained_memory_bytes = retainedMemory(
      receipt.search.main_heap,
      receipt.search.worker_heap,
    );
    await page.evaluate(() => {
      window.__searchWorker.terminate();
      delete window.__searchWorker;
      delete window.__searchResult;
    });
    assert.equal(receipt.search.complete, true);
    assert.equal(receipt.search.success, true);
    assert.equal(receipt.search.completed, 4);
    assert.equal(receipt.search.ranking.length, 2);
    assert.equal(
      receipt.search.engine_source_sha256,
      receipt.engine_source_sha256,
    );
    assert.ok(
      receipt.search.wall_ms <= 30000 &&
        receipt.search.cpu_seconds <= 30 &&
        receipt.search.report_bytes <= 64 * 1024 * 1024 &&
        receipt.search.retained_memory_bytes <= 512 * 1024 * 1024,
    );
    receipt.checks.push(
      "Actual Rust candidate search over two refinements/two seeds, exact checkpoint text and fresh Worker continuation",
    );
    await page.locator("#tab-config").click();
    // Keyboard, dialog focus restoration, tooltips and all three views at 390px.
    await page.locator('#config [data-toggle="0"]').click();
    await page.locator("#settings-btn").focus();
    await page.keyboard.press("Enter");
    assert.equal(await page.locator("#setting-mode, #setting-crit").count(), 0);
    assert.ok(
      await page
        .locator("dialog[open]")
        .evaluate((d) => d.contains(document.activeElement)),
    );
    await page.keyboard.press("Escape");
    assert.equal(
      await page
        .locator("#settings-btn")
        .evaluate((e) => e === document.activeElement),
      true,
    );
    await page.locator("#settings-btn").focus();
    await page.keyboard.press("Tab");
    assert.ok(
      await page
        .locator("#tools-btn")
        .evaluate((e) => parseFloat(getComputedStyle(e).outlineWidth) >= 2),
    );
    await page.locator("#tab-config").focus();
    await page.keyboard.press("ArrowRight");
    assert.equal(
      await page.locator("#tab-results").getAttribute("aria-selected"),
      "true",
    );
    await page.keyboard.press("End");
    assert.equal(
      await page.locator("#tab-sample").getAttribute("aria-selected"),
      "true",
    );
    await page.getByRole("slider", { name: "Sample time" }).focus();
    await page.keyboard.press("End");
    assert.equal(
      await page
        .getByRole("slider", { name: "Sample time" })
        .evaluate((e) => e.value === e.max),
      true,
    );
    await page.locator("#tab-config").click();
    await page.locator("#team-cards .weapon-card").first().focus();
    await page.getByRole("tooltip").filter({ visible: true }).first().waitFor();
    await page.keyboard.press("Escape");
    await page.keyboard.press("Control+k");
    await page.locator("#reference-search").fill("attack");
    assert.ok((await page.locator("#reference-list button").count()) > 0);
    await page.keyboard.press("Escape");
    await page.locator("dialog[open]").waitFor({ state: "hidden" });
    receipt.contrast = await contrast(page, [
      "#run-btn",
      "#settings-btn",
      "#tools-btn",
      "#syntax-status",
      "#save-status",
    ]);
    assert.equal(
      await page.evaluate(
        () => getComputedStyle(document.documentElement).scrollBehavior,
      ),
      "auto",
    );
    await page.setViewportSize({ width: 390, height: 844 });
    for (const tab of ["config", "results", "sample"]) {
      await page.locator("#tab-" + tab).click();
      assert.ok(
        await page.evaluate(
          () => document.documentElement.scrollWidth <= innerWidth,
        ),
        `390px ${tab} overflow`,
      );
      await page.screenshot({
        path: path.join(output, `${tab}-390.png`),
        fullPage: true,
      });
    }
    receipt.checks.push(
      "Keyboard tabs, slider, dialog focus/Escape, search, focus tooltips, primary text contrast, reduced motion and 390px Simulator/Results/Sample",
    );
    await page.locator("#tab-config").click();
    await page.locator("#tools-btn").click();
    await page.locator("#import-input").setInputFiles({
      name: "oversize.nte",
      mimeType: "text/plain",
      buffer: Buffer.alloc(50000001, 32),
    });
    assert.match(
      await page.locator("#tools-dialog [role=alert]").textContent(),
      /50 MB/,
    );
    await page.locator("#import-input").setInputFiles({
      name: "bad.json",
      mimeType: "application/json",
      buffer: Buffer.from('{"source":false}'),
    });
    assert.match(
      await page.locator("#tools-dialog [role=alert]").textContent(),
      /source/,
    );
    await page.locator("#import-input").setInputFiles({
      name: "valid.json",
      mimeType: "application/json",
      buffer: Buffer.from(
        JSON.stringify({
          source: small.replace("iteration=1000", "iteration=2"),
        }),
      ),
    });
    await page.locator("dialog[open]").waitFor({ state: "hidden" });
    await page.locator("#script-input").fill("options iteration=;");
    assert.equal(await page.locator("#run-btn").isDisabled(), true);
    await page.reload();
    assert.equal(
      await page.locator("#script-input").inputValue(),
      "options iteration=;",
    );
    await source(page, small.replace("iteration=1000", "iteration=2"));
    receipt.checks.push(
      "50 MB import rejection, invalid JSON shape, valid source JSON, invalid draft preservation and blocked Run",
    );
    // Real Service Worker directory-prefix install, upgrade isolation, offline cold reload and Worker execution.
    await page.evaluate(async () => {
      await caches.open("other-app-keep");
    });
    assert.equal(
      await page.locator("#update-app").count(),
      0,
      "First install is silent",
    );
    const originalPage = await page.evaluate(() => performance.timeOrigin);
    server.upgrade();
    await page.evaluate(() => window.dispatchEvent(new Event("focus")));
    await page.locator("#update-app").waitFor();
    assert.equal(
      await page.evaluate(() => performance.timeOrigin),
      originalPage,
    );
    assert.ok(
      await page.evaluate(
        async () => !!(await navigator.serviceWorker.ready).waiting,
      ),
    );
    await page.screenshot({
      path: path.join(output, "update-390.png"),
      fullPage: true,
    });
    assert.ok(
      await page.evaluate(
        () => document.documentElement.scrollWidth <= innerWidth,
      ),
    );
    // Reloading before acceptance keeps the old build and re-offers the waiting update.
    await page.reload();
    await page.locator("#update-app").waitFor();
    const otherTab = await context.newPage();
    await otherTab.goto(server.url);
    await otherTab.locator("#update-app").waitFor();
    const otherOrigin = await otherTab.evaluate(() => performance.timeOrigin);
    await page.locator("#update-app").click();
    await page.waitForFunction(
      () =>
        !document.querySelector("#update-app") &&
        document.querySelector("#run-btn")?.disabled === false,
    );
    await upgradedCache(page);
    assert.equal(
      await page.locator("#script-input").inputValue(),
      small.replace("iteration=1000", "iteration=2"),
    );
    assert.equal(
      await otherTab.evaluate(() => performance.timeOrigin),
      otherOrigin,
    );
    await otherTab.locator("#update-app").click();
    await otherTab.waitForFunction(
      () => !document.querySelector("#update-app"),
    );
    await otherTab.close();
    const inventory = await page.evaluate(async () => {
      const keys = await caches.keys(),
        name = keys.find((k) => k.startsWith("ntesim-offline-"));
      return {
        keys,
        urls: (await (await caches.open(name)).keys()).map((r) => r.url),
      };
    });
    assert.ok(inventory.keys.includes("other-app-keep"));
    assert.equal(
      inventory.keys.filter((k) => k.startsWith("ntesim-offline-")).length,
      1,
    );
    for (const kind of [
      ".wasm",
      ".css",
      "/licenses/runtime.txt",
      "/assets/artwork/",
    ])
      assert.ok(inventory.urls.some((url) => url.includes(kind)));
    assert.ok(
      inventory.urls.every((url) =>
        new URL(url).pathname.startsWith("/ntesim-dev/"),
      ),
    );
    assert.equal(
      await page.evaluate(
        async () => (await navigator.serviceWorker.ready).scope,
      ),
      server.url,
    );
    await context.setOffline(true);
    await page.reload();
    await page.waitForFunction(
      () => document.querySelector("#run-btn")?.disabled === false,
    );
    await page.waitForFunction(() => {
      const images = [...document.querySelectorAll(".character-portrait")];
      return (
        images.length > 0 &&
        images.every((image) => image.complete && image.naturalWidth > 0)
      );
    });
    assert.ok(
      await page.evaluate(async (urls) => {
        for (const url of urls) {
          const r = await fetch(url);
          if (!r.ok) return false;
        }
        return true;
      }, inventory.urls),
    );
    await page.locator("#run-btn").click();
    await completed(page);
    receipt.offline_assets = inventory.urls.length;
    await offlineReportAcceptance(page);
    receipt.checks.push(
      "Directory-prefix full offline inventory, explicit update/reload, waiting and multi-tab notice, mobile banner, cache isolation, offline cold reload and actual Worker/WASM execution",
    );
    await context.setOffline(false);
    const denied = await browser.newContext();
    await denied.addInitScript(() => {
      Object.defineProperty(window, "localStorage", {
        get() {
          throw Error("blocked");
        },
      });
    });
    const deniedPage = await denied.newPage();
    await deniedPage.goto(server.url);
    await source(deniedPage, small.replace("iteration=1000", "iteration=2"));
    assert.match(
      await deniedPage.locator("#save-status").textContent(),
      /unavailable|could not be read/,
    );
    await deniedPage.locator("#run-btn").click();
    await completed(deniedPage);
    await denied.close();
    receipt.checks.push(
      "Storage getter failure preserves editing and actual execution",
    );
    const broken = await browser.newContext({ serviceWorkers: "block" });
    await broken.route("**/*.wasm", (route) =>
      route.fulfill({
        status: 200,
        contentType: "application/wasm",
        body: "corrupted stale asset",
      }),
    );
    const brokenPage = await broken.newPage();
    brokenPage.on("pageerror", (error) => errors.push(String(error)));
    await brokenPage.goto(server.url);
    await brokenPage
      .locator("#diagnostics")
      .filter({ hasText: /WebAssembly|wasm|magic word/ })
      .waitFor();
    assert.equal(await brokenPage.locator("#run-btn").isDisabled(), true);
    assert.equal(
      await brokenPage.locator(".build-identity time").getAttribute("datetime"),
      build.build_started_at,
    );
    await broken.close();
    receipt.checks.push(
      "Corrupt WASM asset produces visible diagnostics and blocks Run",
    );
  }
  assert.deepEqual(
    artifactHashes(),
    receipt.artifacts,
    "Build artifacts changed during acceptance",
  );
  assert.deepEqual(errors, []);
  receipt.errors = errors;
  receipt.passed = true;
  receipt.status = "complete";
  save();
  console.log(
    `${throughputOnly ? "Browser throughput measurement completed" : "Browser acceptance passed"}; ${receipt.runs.length} measured workloads; receipt ${path.join(output, "measurement.json")}`,
  );
} catch (error) {
  receipt.status = "failed";
  receipt.error = String(error);
  receipt.errors = errors;
  if (page && !page.isClosed()) {
    receipt.visible_status = await page.evaluate(() => ({
      run: document.querySelector("#run-status")?.textContent,
      results: document.querySelector("#run-status")?.textContent,
      diagnostics: document.querySelector("#diagnostics")?.textContent,
    }));
    await page.screenshot({
      path: path.join(output, "failure.png"),
      fullPage: true,
    });
  }
  save();
  throw error;
} finally {
  try {
    await context?.close();
    await browser?.close();
    await server?.close();
  } catch (error) {
    receipt.passed = false;
    receipt.status = "failed";
    receipt.error = `Cleanup failed: ${error}`;
    save();
    throw error;
  }
}
