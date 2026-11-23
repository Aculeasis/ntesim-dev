import { readFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";

const root = fileURLToPath(new URL("../", import.meta.url));
export const presetDirectory = path.join(root, "presets/studio");
export const presetBundlePath = path.join(
  root,
  "web/src/generated/script-presets.json",
);
export const teamCatalogPath = path.join(
  root,
  "web/src/generated/team-list.json",
);
export const sha256 = (bytes) =>
  createHash("sha256").update(bytes).digest("hex");
export function teamGeneratorHash() {
  return sha256(
    [
      "tools/studio_teams.mts",
      "tools/studio_presets.mjs",
      "web/src/domain/contracts/team-list.ts",
    ]
      .map((file) => readFileSync(path.join(root, file), "utf8"))
      .join("\n"),
  );
}
export function currentEngineHash() {
  return execFileSync(
    "python",
    [
      "-c",
      "import sys; sys.path.insert(0, 'tools'); from cargo import engine_source_sha256; print(engine_source_sha256())",
    ],
    { cwd: root, encoding: "utf8" },
  ).trim();
}
export function validateTeamCatalog(catalog, presets) {
  const fail = () => {
    throw new Error("Team list is invalid. Run node tools/web.mjs teams.");
  };
  if (
    catalog.schema_version !== 1 ||
    typeof catalog.engine_version !== "string" ||
    !catalog.engine_version ||
    !Array.isArray(catalog.entries)
  )
    fail();
  const selectable = presets.scripts.filter((entry) => entry.selectable);
  if (catalog.entries.length !== selectable.length) fail();
  for (let i = 0; i < selectable.length; i++) {
    const entry = catalog.entries[i],
      source = selectable[i];
    if (
      entry.id !== source.id ||
      entry.label !== source.label ||
      !Array.isArray(entry.actors) ||
      !entry.actors.length ||
      entry.actors.length > 4 ||
      !Array.isArray(entry.targets) ||
      !Array.isArray(entry.assumptions) ||
      typeof entry.quality !== "string"
    )
      fail();
    if (entry.dps?.status === "ready") {
      if (
        catalog.clock_contract !== "relative-combat-v1" ||
        catalog.timer_policy !== "paused-world-v1" ||
        catalog.relative_duration_us !== 120_000_000 ||
        entry.options?.relative_duration_us !== 120_000_000 ||
        !Number.isFinite(entry.dps.mean) ||
        entry.dps.mean < 0 ||
        !entry.options
      )
        fail();
    } else if (
      entry.dps?.status !== "unavailable" ||
      typeof entry.dps.reason !== "string" ||
      !entry.dps.reason
    )
      fail();
  }
}
export function checkTeamCatalog() {
  let catalog;
  try {
    catalog = JSON.parse(readFileSync(teamCatalogPath, "utf8"));
  } catch {
    throw new Error("Missing Team list. Run node tools/web.mjs teams.");
  }
  validateTeamCatalog(catalog, readStudioPresets());
}

export function readStudioPresets(directory = presetDirectory) {
  const manifest = JSON.parse(
    readFileSync(path.join(directory, "index.json"), "utf8"),
  );
  if (!Array.isArray(manifest.scripts) || !manifest.scripts.length)
    throw new Error("Studio preset catalog must contain scripts.");
  const ids = new Set();
  const scripts = manifest.scripts.map((entry) => {
    if (
      typeof entry.id !== "string" ||
      !/^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(entry.id) ||
      ids.has(entry.id)
    )
      throw new Error(`Invalid or duplicate studio preset id: ${entry.id}`);
    ids.add(entry.id);
    if (typeof entry.label !== "string" || !entry.label.trim())
      throw new Error(`Missing studio preset label: ${entry.id}`);
    if (typeof entry.selectable !== "boolean")
      throw new Error(`Missing selectable flag: ${entry.id}`);
    if (typeof entry.file !== "string" || !/^[a-z0-9-]+\.nte$/.test(entry.file))
      throw new Error(`Invalid studio preset file: ${entry.id}`);
    const source = readFileSync(path.join(directory, entry.file), "utf8");
    if (!source.trim()) throw new Error(`Empty studio preset: ${entry.id}`);
    return {
      id: entry.id,
      label: entry.label,
      selectable: entry.selectable,
      source,
    };
  });
  if (
    !scripts.some((entry) => entry.id === manifest.default && entry.selectable)
  )
    throw new Error("Default studio preset must name a selectable script.");
  return { default: manifest.default, scripts };
}

export function syncStudioPresets() {
  const content = JSON.stringify(readStudioPresets(), null, 2) + "\n";
  let previous;
  try {
    previous = readFileSync(presetBundlePath, "utf8");
  } catch (error) {
    if (error.code !== "ENOENT") throw error;
  }
  if (previous !== content) writeFileSync(presetBundlePath, content);
}
