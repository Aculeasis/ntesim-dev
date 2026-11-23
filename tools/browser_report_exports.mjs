import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { exported, source, completed } from "./browser_support.mjs";

async function downloadFormat(page, format, output) {
  const download = page.waitForEvent("download");
  await page.locator("#export-report-btn").click();
  await page.locator("#export-report-" + format).click();
  const file = await download;
  assert.ok(file.suggestedFilename().endsWith("-report." + format));
  if (output) await file.saveAs(output);
  return {
    name: file.suggestedFilename(),
    buffer: fs.readFileSync(await file.path()),
  };
}
async function importReport(page, payload, success = true) {
  const executions = await page.evaluate(
    () => window.__browserAudit.executionRequests,
  );
  await page.locator("#tools-btn").click();
  await page
    .locator("#import-report-input")
    .setInputFiles({ ...payload, mimeType: "application/octet-stream" });
  if (success) await page.locator("#tools-dialog").waitFor({ state: "hidden" });
  else await page.locator('#tools-dialog [role="alert"]').waitFor();
  assert.equal(
    await page.evaluate(() => window.__browserAudit.executionRequests),
    executions,
    "import must not simulate",
  );
}

export async function portableReportAcceptance(page, root, output) {
  const text = `options duration=120 iteration=2 crit=fixed;
    hotori char lvl=80/80 awakening=0 talent=1,1,1 qte=1 start_energy=0;
    mint char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
    sakiri char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
    haniel char lvl=80/80 awakening=0 talent=1,1,1 qte=1;
    target hp=1000000000 def=0 resist=0; active hotori;
    repeat 2 { wait(20); hotori burst;
      while .hotori.time_stop.active { hotori attack[window=attack]; } }`;
  await source(page, text);
  await page.locator("#run-btn").click();
  await completed(page);
  const expected = await exported(page);
  assert.equal(expected.report.relative_elapsed_us, 120_000_000);
  assert.ok(expected.report.absolute_elapsed_us > 120_000_000);
  await source(page, text + "\n# a later editor draft");
  await page.locator("#tab-results").click();
  let downloads = 0;
  const count = () => downloads++;
  page.on("download", count);
  await page.locator("#export-report-btn").click();
  assert.equal(
    await page.locator('#report-export-menu [role="menuitem"]').count(),
    3,
  );
  assert.equal(downloads, 0, "opening the menu does not download");
  assert.equal(
    await page.locator("#export-report-btn").getAttribute("aria-expanded"),
    "true",
  );
  assert.equal(
    await page
      .locator("#export-report-json")
      .evaluate((el) => document.activeElement === el),
    true,
  );
  await page.keyboard.press("ArrowDown");
  assert.equal(
    await page
      .locator("#export-report-nter")
      .evaluate((el) => document.activeElement === el),
    true,
  );
  await page.keyboard.press("Escape");
  assert.equal(
    await page
      .locator("#export-report-btn")
      .evaluate((el) => document.activeElement === el),
    true,
  );
  await page.locator("#export-report-btn").click();
  await page.locator("#view-title").click();
  assert.equal(await page.locator("#report-export-menu").count(), 0);
  const compressed = await downloadFormat(
    page,
    "nter",
    path.join(output, "portable-report.nter"),
  );
  await importReport(page, compressed);
  assert.deepEqual(await exported(page), expected);
  assert.equal(
    await page.locator("#stop-btn").count(),
    0,
    "import must not start simulation",
  );

  const images = [];
  for (const width of [1440, 390]) {
    await page.setViewportSize({ width, height: 1000 });
    const dpsBasis = page.getByRole("button", {
      name: "Damage per second time basis",
    });
    const fieldBasis = page.getByRole("button", {
      name: "Field time distribution time basis",
    });
    assert.equal(await dpsBasis.innerText(), "relative");
    assert.equal(await fieldBasis.innerText(), "relative");

    const relativeDps = await page
      .locator("#rollup-cards .rollup-card")
      .first()
      .innerText();
    const fieldTip = () =>
      page
        .locator("#field-time [data-chart-tip]")
        .first()
        .getAttribute("data-chart-tip");
    const relativeField = await fieldTip();
    const executions = await page.evaluate(
      () => window.__browserAudit.executionRequests,
    );

    if (width === 1440) {
      // Exercise pointer switch on DPS button
      await dpsBasis.click();
      assert.equal(await dpsBasis.innerText(), "absolute");
      assert.equal(await fieldBasis.innerText(), "absolute");
      const absoluteDps = await page
        .locator("#rollup-cards .rollup-card")
        .first()
        .innerText();
      const absoluteField = await fieldTip();
      assert.notEqual(absoluteDps, relativeDps);
      assert.notEqual(absoluteField, relativeField);

      // Exercise keyboard switch on field time button
      await fieldBasis.focus();
      await page.keyboard.press("Space");
      assert.equal(await dpsBasis.innerText(), "relative");
      assert.equal(await fieldBasis.innerText(), "relative");
      assert.equal(
        await page.locator("#rollup-cards .rollup-card").first().innerText(),
        relativeDps,
      );
      assert.equal(await fieldTip(), relativeField);

      // Exercise pointer switch on field time button
      await fieldBasis.click();
      assert.equal(await dpsBasis.innerText(), "absolute");
      assert.equal(await fieldBasis.innerText(), "absolute");

      // Exercise keyboard switch on DPS button
      await dpsBasis.focus();
      await page.keyboard.press("Enter");
      assert.equal(await dpsBasis.innerText(), "relative");
      assert.equal(await fieldBasis.innerText(), "relative");

      // Set to absolute for export capture
      await dpsBasis.click();
      assert.equal(await dpsBasis.innerText(), "absolute");
      assert.equal(await fieldBasis.innerText(), "absolute");
    } else {
      // 390px width: exercise keyboard on DPS button and pointer on field time button
      await dpsBasis.focus();
      await page.keyboard.press("Enter");
      assert.equal(await dpsBasis.innerText(), "absolute");
      assert.equal(await fieldBasis.innerText(), "absolute");

      await fieldBasis.click();
      assert.equal(await dpsBasis.innerText(), "relative");
      assert.equal(await fieldBasis.innerText(), "relative");

      await fieldBasis.focus();
      await page.keyboard.press("Space");
      assert.equal(await dpsBasis.innerText(), "absolute");
      assert.equal(await fieldBasis.innerText(), "absolute");

      await dpsBasis.click();
      assert.equal(await dpsBasis.innerText(), "relative");
      assert.equal(await fieldBasis.innerText(), "relative");

      // Set to absolute for export capture
      await fieldBasis.click();
      assert.equal(await dpsBasis.innerText(), "absolute");
      assert.equal(await fieldBasis.innerText(), "absolute");
    }

    const currentDps = await page
      .locator("#rollup-cards .rollup-card")
      .first()
      .innerText();
    const currentField = await fieldTip();
    assert.notEqual(currentDps, relativeDps);
    assert.notEqual(currentField, relativeField);

    assert.equal(
      await page
        .locator("#rollup-cards .rollup-card")
        .last()
        .locator(".stat-grid b")
        .count(),
      6,
    );
    await page.locator("#tab-sample").click();
    assert.ok(await page.locator(".timer-pause-segment").count());
    await page.locator("#tab-results").click();
    assert.equal(await dpsBasis.innerText(), "absolute");
    assert.equal(await fieldBasis.innerText(), "absolute");
    assert.equal(
      await page.locator("#rollup-cards .rollup-card").first().innerText(),
      currentDps,
    );
    assert.equal(await fieldTip(), currentField);
    assert.equal(
      await page.evaluate(() => window.__browserAudit.executionRequests),
      executions,
      "switches and navigation must issue no simulation or replay requests",
    );
    await page.locator("#distribution-type").selectOption("hits");
    await page.locator("#timeline-type").selectOption("cumulative");
    await page.locator("#energy-type").selectOption("overflow");
    await page
      .locator("#result-team .character-card")
      .first()
      .getByRole("button")
      .first()
      .click();
    const folds = page.locator("#report-capture .chart-expand-bar");
    if (await folds.count()) await folds.first().click();
    const selections = await page.locator("#report-capture").evaluate((el) => ({
      selects: [...el.querySelectorAll("select")].map((n) => [n.id, n.value]),
      expanded: [...el.querySelectorAll("[aria-expanded]")].map((n) =>
        n.getAttribute("aria-expanded"),
      ),
    }));
    const region = page.locator("#report-capture");
    await region.evaluate(async (el) => {
      await document.fonts.ready;
      await Promise.all([...el.querySelectorAll("img")].map((n) => n.decode()));
    });
    assert.ok(
      (await region.innerText()).includes(
        "Character Energy at End of Simulation",
      ),
    );
    assert.ok(!(await region.innerText()).includes("Run details"));
    assert.equal(await region.locator("#result-team").count(), 1);
    assert.equal(await region.locator("#ending-energy").count(), 1);
    const bounds = await region.boundingBox();
    assert.ok(bounds.height > 1000);
    // Retain both the live equivalent and the exported raster for visual QA.
    await region.screenshot({
      path: path.join(output, `report-live-${width}.png`),
      animations: "disabled",
    });
    const image = await downloadFormat(
      page,
      "png",
      path.join(output, `report-export-${width}.png`),
    );
    const dimensions = await page.evaluate(
      async ({ buffer }) => {
        const bytes = Uint8Array.from(atob(buffer), (c) => c.charCodeAt(0));
        const image = await createImageBitmap(
          new Blob([bytes], { type: "image/png" }),
        );
        const result = { width: image.width, height: image.height };
        image.close();
        return result;
      },
      { buffer: image.buffer.toString("base64") },
    );
    assert.deepEqual(dimensions, {
      width: Math.ceil(bounds.width),
      height: Math.ceil(bounds.height),
    });
    assert.deepEqual(
      await region.evaluate((el) => ({
        selects: [...el.querySelectorAll("select")].map((n) => [n.id, n.value]),
        expanded: [...el.querySelectorAll("[aria-expanded]")].map((n) =>
          n.getAttribute("aria-expanded"),
        ),
      })),
      selections,
      "capture does not change live selections/expansion",
    );
    await importReport(page, image);
    assert.deepEqual(await exported(page), expected);
    assert.equal(
      await dpsBasis.innerText(),
      "relative",
      "imported data defaults to Relative",
    );
    assert.equal(await fieldBasis.innerText(), "relative");
    const importExecutions = await page.evaluate(
      () => window.__browserAudit.executionRequests,
    );
    await dpsBasis.click();
    assert.equal(await dpsBasis.innerText(), "absolute");
    assert.equal(await fieldBasis.innerText(), "absolute");
    await fieldBasis.click();
    assert.equal(await dpsBasis.innerText(), "relative");
    assert.equal(await fieldBasis.innerText(), "relative");
    assert.equal(
      await page.evaluate(() => window.__browserAudit.executionRequests),
      importExecutions,
      "basis switches after import must not simulate",
    );
    images.push(image);
  }

  // A normal raster without metadata gets a specific error, preserving draft
  // and results; malformed bundles follow the same atomic UI boundary.
  const beforeSource = await page.locator("#script-input").inputValue();
  const corruptContainer = Buffer.from(compressed.buffer);
  corruptContainer[16] ^= 1;
  const plain = {
    name: "plain.png",
    buffer: fs.readFileSync(path.join(output, "report-live-390.png")),
  };
  for (const payload of [
    plain,
    { name: "corrupt.nter", buffer: corruptContainer },
    {
      name: "invalid.json",
      buffer: Buffer.from(
        JSON.stringify({
          schema_version: 1,
          source: "wait(0);",
          aggregate: {},
        }),
      ),
    },
  ]) {
    await importReport(page, payload, false);
    const error = await page
      .locator('#tools-dialog [role="alert"]')
      .innerText();
    assert.match(
      error,
      payload === plain
        ? /no embedded simulation report/
        : payload.name === "corrupt.nter"
          ? /checksum/
          : /simulation report data/,
    );
    assert.equal(
      await page.locator("#script-input").inputValue(),
      beforeSource,
    );
    await page.keyboard.press("Escape");
    assert.deepEqual(await exported(page), expected);
  }
  // Change the run while capture awaits fonts. The saved DOM cannot be paired
  // with another run, and a later attempt must recover normally.
  await page.evaluate(() => {
    const fonts = document.fonts;
    Object.defineProperty(fonts, "ready", {
      configurable: true,
      value: new Promise((resolve) => {
        window.__releaseReportFonts = resolve;
      }),
    });
  });
  await page.locator("#export-report-btn").click();
  await page.locator("#export-report-png").click();
  const beforeStale = downloads;
  await importReport(page, compressed);
  await page.evaluate(() => {
    delete document.fonts.ready;
    window.__releaseReportFonts();
  });
  await page
    .locator('.tools-anchor [role="alert"]')
    .filter({ hasText: /Results changed/ })
    .waitFor();
  assert.equal(downloads, beforeStale);
  await page.evaluate(() => {
    window.__originalReportToBlob = HTMLCanvasElement.prototype.toBlob;
    HTMLCanvasElement.prototype.toBlob = function (callback) {
      callback(null);
    };
  });
  await page.locator("#export-report-btn").click();
  await page.locator("#export-report-png").click();
  await page
    .locator('.tools-anchor [role="alert"]')
    .filter({ hasText: /could not encode/ })
    .waitFor();
  assert.equal(downloads, beforeStale);
  await page.evaluate(() => {
    HTMLCanvasElement.prototype.toBlob = window.__originalReportToBlob;
  });
  await downloadFormat(page, "png");
  page.off("download", count);
  await page.setViewportSize({ width: 1440, height: 1000 });
  return {
    checks:
      "Portable JSON/nter/PNG menu, keyboard, executed identity, desktop/390px raster, original PNG restoration, atomic failures, stale-run rejection and recovery",
    images: images.map((file) => file.buffer.length),
  };
}

export async function offlineReportAcceptance(page) {
  const compressed = await downloadFormat(page, "nter");
  const image = await downloadFormat(page, "png");
  const expected = await exported(page);
  await importReport(page, compressed);
  assert.deepEqual(await exported(page), expected);
  await importReport(page, image);
  assert.deepEqual(await exported(page), expected);
}
