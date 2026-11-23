import assert from "node:assert/strict";

// The compact UI has no Resume button; exercise its existing Worker contract.
export async function resumeInBrowser(page, serverUrl, artifacts, retained) {
  const workerFile = Object.keys(artifacts).find((file) =>
    /\/worker-.*\.js$/.test(file),
  );
  const wasmFile = Object.keys(artifacts).find(
    (file) => file.includes("ntesim_wasm_bg") && file.endsWith(".wasm"),
  );
  return page.evaluate(
    async ({ workerUrl, wasmUrl, retained }) => {
      const worker = new Worker(workerUrl, { type: "module" });
      try {
        return await new Promise((resolve, reject) => {
          const timer = setTimeout(
            () => reject(Error("Resume Worker timeout")),
            30000,
          );
          worker.onerror = (event) => {
            clearTimeout(timer);
            reject(Error(event.message));
          };
          worker.onmessage = ({ data }) => {
            if (data.type === "SUCCESS" || data.type === "ERROR") {
              clearTimeout(timer);
              resolve(data);
            }
          };
          worker.postMessage({
            id: 1,
            type: "SIMULATE_NTE",
            wasmSource: wasmUrl,
            payload: { source: retained.source, resume: retained },
          });
        });
      } finally {
        worker.terminate();
      }
    },
    {
      workerUrl: new URL(workerFile.replace("target/web/", ""), serverUrl).href,
      wasmUrl: new URL(wasmFile.replace("target/web/", ""), serverUrl).href,
      retained,
    },
  );
}

export async function importResumedReport(page, prefix, response) {
  assert.equal(response.type, "SUCCESS");
  await page.locator("#tools-btn").click();
  await page.locator("#import-report-input").setInputFiles({
    name: "resumed.json",
    mimeType: "application/json",
    buffer: Buffer.from(
      JSON.stringify({
        schema_version: 1,
        ...response.result,
        source: prefix.source,
        cancelled: false,
        failure: null,
        failed_trial: null,
      }),
    ),
  });
  await page.locator("#tools-dialog").waitFor({ state: "hidden" });
}
