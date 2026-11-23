import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";

export async function teamListAcceptance(page, root, output) {
  const presets = JSON.parse(
    fs.readFileSync(path.join(root, "web/src/generated/script-presets.json")),
  );
  const entries = JSON.parse(
    fs.readFileSync(path.join(root, "web/src/generated/team-list.json")),
  ).entries;
  const characters = JSON.parse(
    fs.readFileSync(path.join(root, "web/src/generated/catalog.json")),
  ).characters;
  const characterId = (name) => characters.find((c) => c.name === name).id;
  const sortTeams = (list) =>
    [...list].sort((a, b) => {
      if (a.dps.status !== b.dps.status)
        return a.dps.status === "ready" ? -1 : 1;
      const diff =
        a.dps.status === "ready" && b.dps.status === "ready"
          ? b.dps.mean - a.dps.mean
          : 0;
      return diff || a.id.localeCompare(b.id);
    });
  const matching = (include = [], exclude = []) =>
    sortTeams(
      entries.filter(
        (entry) =>
          include.every((name) =>
            entry.actors.some((a) => a.character_id === characterId(name)),
          ) &&
          exclude.every((name) =>
            entry.actors.every((a) => a.character_id !== characterId(name)),
          ),
      ),
    ).map((entry) => entry.id);
  const original = await page.locator("#script-input").inputValue();
  const audit = () =>
    page.evaluate(() => ({
      workers: window.__browserAudit.workers,
      messages: window.__browserAudit.messages,
    }));
  const before = await audit();
  const open = () => page.locator("#team-list-btn").click();
  const navigation = await page.evaluate(() => {
    const nav = document
      .querySelector(".studio-navigation")
      .getBoundingClientRect();
    const tabs = document.querySelector(".view-tabs").getBoundingClientRect();
    const button = document
      .querySelector("#team-list-btn")
      .getBoundingClientRect();
    return {
      navLeft: nav.left,
      tabsLeft: tabs.left,
      buttonCenter: button.left + button.width / 2,
      navCenter: nav.left + nav.width / 2,
    };
  });
  assert.ok(
    Math.abs(navigation.tabsLeft - navigation.navLeft) <= 1,
    "View tabs stay left aligned",
  );
  assert.ok(
    Math.abs(navigation.buttonCenter - navigation.navCenter) <= 1,
    "Team list is centered independently of tabs",
  );
  await page.screenshot({
    path: path.join(output, "team-list-navigation.png"),
  });
  const rows = page.locator(".team-list-row");
  const filter = (name) => page.locator(`[data-filter=${name}]`).click();
  const portrait = (name) =>
    page.locator("#roster-picker").getByRole("button", { name, exact: true });
  const ids = () =>
    rows.evaluateAll((nodes) => nodes.map((node) => node.dataset.teamId));
  await open();
  const headerCenters = await page
    .locator(".team-list-header")
    .evaluate((header) =>
      [...header.children].map((node) => {
        const rect = node.getBoundingClientRect();
        return rect.top + rect.height / 2;
      }),
    );
  assert.ok(
    Math.max(...headerCenters) - Math.min(...headerCenters) <= 1,
    "Team list header controls share a vertical center",
  );
  assert.equal(await rows.count(), entries.length);
  assert.equal(
    await page.locator('[data-team-id="s4"], .team-list-row details').count(),
    0,
  );
  assert.equal(await page.locator("#preset-select").count(), 0);
  assert.deepEqual(
    await ids(),
    sortTeams(entries).map((e) => e.id),
  );
  assert.equal(
    await page.locator(".team-row-cards .compact-character").count(),
    entries.length * 4,
  );
  assert.equal(
    await page
      .locator(".team-list-row button button, .team-list-row button summary")
      .count(),
    0,
  );
  const headerTop = await page
    .locator(".team-list-header")
    .evaluate((node) => node.getBoundingClientRect().top);
  await page.locator(".team-list-rows").evaluate((node) => {
    node.scrollTop = node.scrollHeight;
  });
  assert.equal(
    await page
      .locator(".team-list-header")
      .evaluate((node) => node.getBoundingClientRect().top),
    headerTop,
  );
  await page.locator(".team-list-rows").evaluate((node) => {
    node.scrollTop = 0;
  });
  await page.screenshot({ path: path.join(output, "team-list-desktop.png") });
  await filter("include");
  await portrait("Mint").click();
  await portrait("Sakiri").click();
  assert.deepEqual(await ids(), matching(["Mint", "Sakiri"]));
  await page.screenshot({ path: path.join(output, "team-list-include.png") });
  await filter("exclude");
  await portrait("Zero").focus();
  await page.keyboard.press("Space");
  assert.deepEqual(await ids(), matching(["Mint", "Sakiri"], ["Zero"]));
  await page.keyboard.press("Escape");
  assert.equal(await page.locator("#roster-picker").count(), 0);
  assert.equal(await page.locator("#team-list-dialog[open]").count(), 1);
  await page.keyboard.press("Escape");
  assert.equal(
    await page
      .locator("#team-list-btn")
      .evaluate((node) => node === document.activeElement),
    true,
  );
  assert.equal(await page.locator("#script-input").inputValue(), original);
  await open();
  assert.deepEqual(await ids(), matching(["Mint", "Sakiri"], ["Zero"]));
  assert.equal(
    await page.locator("[data-filter=include] .filter-count").textContent(),
    "2",
  );
  assert.equal(
    await page.locator("[data-filter=exclude] .filter-count").textContent(),
    "1",
  );
  await filter("exclude");
  await page.getByRole("button", { name: "Clear All", exact: true }).click();
  assert.deepEqual(await ids(), matching(["Mint", "Sakiri"]));
  await portrait("Mint").click();
  assert.equal(await rows.count(), 0);
  assert.match(
    await page.locator(".team-list-empty").textContent(),
    /No teams/,
  );
  await filter("include");
  await page.getByRole("button", { name: "Clear All", exact: true }).click();
  assert.deepEqual(await ids(), matching([], ["Mint"]));
  await filter("exclude");
  await page.getByRole("button", { name: "Clear All", exact: true }).click();
  await page.getByRole("searchbox", { name: "Search characters" }).click();
  assert.equal(await page.locator("#roster-picker").count(), 0);
  await page.getByRole("searchbox").fill("mint SAK");
  assert.deepEqual(await ids(), matching(["Mint", "Sakiri"]));
  // Escape on a search input first clears its native value in some browsers.
  await page.locator(".team-list-close").click();
  await open();
  assert.equal(await page.getByRole("searchbox").inputValue(), "mint SAK");
  await page.getByRole("searchbox").fill("");
  const equipment = page.locator(".compact-equipment").first();
  await equipment.focus();
  assert.equal(
    await equipment.locator(".compact-gear-details").isVisible(),
    true,
  );
  for (let i = 0; i < 12; i++) {
    await page.keyboard.press("Tab");
    assert.ok(
      await page.evaluate(() =>
        document
          .querySelector("#team-list-dialog")
          .contains(document.activeElement),
      ),
    );
  }
  await page.keyboard.press("Escape");
  for (const preset of presets.scripts.filter((script) => script.selectable)) {
    await open();
    await page
      .locator(`[data-team-id="${preset.id}"] .team-row-select`)
      .click();
    assert.equal(await page.locator("#team-list-dialog").count(), 0);
    assert.equal(
      await page.locator("#script-input").inputValue(),
      preset.source.replace(/\r\n?/g, "\n"), // Textareas normalize authored line endings.
    );
    assert.equal(
      await page.locator("#tab-config").getAttribute("aria-selected"),
      "true",
    );
  }
  assert.deepEqual(
    await audit(),
    before,
    "Browsing, filters and loading cannot create simulation work",
  );
  await page.setViewportSize({ width: 390, height: 844 });
  await open();
  assert.ok(
    await page.evaluate(
      () => document.documentElement.scrollWidth <= innerWidth,
    ),
  );
  await page.screenshot({ path: path.join(output, "team-list-390.png") });
  await filter("exclude");
  await portrait("Mint").click();
  const bounds = await page.locator("#roster-picker").boundingBox();
  assert.ok(
    bounds.x >= 0 &&
      bounds.x + bounds.width <= 390 &&
      bounds.y + bounds.height <= 844,
  );
  await page.screenshot({
    path: path.join(output, "team-list-picker-390.png"),
  });
  await page.getByRole("button", { name: "Clear All", exact: true }).click();
  await page.keyboard.press("Escape");
  await page.keyboard.press("Escape");
  await page.setViewportSize({ width: 1440, height: 1000 });
  await open();
  await page.locator('[data-team-id="s5"] .team-row-select').click();
}

export async function pickerHeaderAcceptance(page, output) {
  for (const width of [1440, 390]) {
    await page.setViewportSize({ width, height: 1000 });
    for (const category of [
      "characters",
      "weapons",
      "cartridges",
      "targets",
      "actions",
      "stats",
    ]) {
      await page.locator(`[data-search=${category}]`).click();
      const dialog = page.getByRole("dialog", {
        name: category[0].toUpperCase() + category.slice(1),
        exact: true,
      });
      assert.equal(await dialog.locator("h2").count(), 0);
      const search = dialog.getByRole("searchbox", { name: "Search names" });
      const close = dialog.getByRole("button", { name: "Close name search" });
      const a = await search.boundingBox(),
        b = await close.boundingBox();
      assert.ok(Math.abs(a.y + a.height / 2 - b.y - b.height / 2) <= 1);
      assert.ok(b.x >= a.x + a.width && b.x + b.width <= width);
      await search.fill("NoSuchReferenceName");
      assert.match(await dialog.textContent(), /No matching names/);
      await search.fill("");
      if (category === "characters") {
        await page.screenshot({
          path: path.join(output, `name-search-${width}.png`),
        });
      }
      await close.click();
      assert.equal(await page.locator("#search-dialog").count(), 0);
    }
  }
  await page.setViewportSize({ width: 1440, height: 1000 });
}

export async function teamListBusyAcceptance(page, exported) {
  await page.locator("#result-meta").waitFor({ state: "visible" });
  await page.locator("#team-list-btn").click();
  assert.equal(
    await page.locator(".team-row-select:disabled").count(),
    await page.locator(".team-list-row").count(),
  );
  assert.match(
    await page.locator(".team-list-meta").textContent(),
    /current execution/,
  );
  await page.keyboard.press("Escape");
  await page.locator("#stop-btn").click();
  await page.waitForFunction(() =>
    document
      .querySelector("#run-status")
      ?.textContent.startsWith("Simulation stopped"),
  );
  const previous = await exported(page);
  await page.locator("#team-list-btn").click();
  await page
    .locator('[data-team-id="nanally-blossom"] .team-row-select')
    .click();
  await page.locator("#tab-results").click();
  const retained = await exported(page);
  assert.equal(retained.source, previous.source);
  assert.deepEqual(retained.aggregate, previous.aggregate);
  assert.deepEqual(retained.samples, previous.samples);
}
