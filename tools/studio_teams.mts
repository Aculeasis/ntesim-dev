import { readFileSync, writeFileSync, mkdirSync, existsSync } from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { NtesimWasm } from "../web/src/infrastructure/wasm/client";
import type {
  TeamCatalog,
  TeamEntry,
} from "../web/src/domain/contracts/team-list";
import {
  CLOCK_CONTRACT,
  TIMER_POLICY,
} from "../web/src/domain/contracts/clocks";
import {
  readStudioPresets,
  sha256,
  teamGeneratorHash,
  currentEngineHash,
  teamCatalogPath,
  checkTeamCatalog,
  validateTeamCatalog,
} from "./studio_presets.mjs";

const root = fileURLToPath(new URL("../", import.meta.url));

export function calculateTeam(
  engine: NtesimWasm,
  script: { id: string; label: string; source: string },
) {
  const inspection = engine.inspectScript(script.source);
  if (!inspection.party)
    throw new Error(
      `Invalid roster for ${script.id}: ${JSON.stringify(inspection.diagnostics)}`,
    );
  const analysis = engine.compileSource(script.source);
  const entry: TeamEntry = {
    id: script.id,
    label: script.label,
    source_sha256: sha256(script.source),
    actors: inspection.party.actors.map(
      ({ character_id, awakening, level, weapon, set }) => ({
        character_id,
        awakening,
        level,
        weapon,
        set,
      }),
    ),
    options: analysis.options,
    targets: analysis.compiled?.request.targets ?? [],
    assumptions: inspection.party.model_assumption_ids,
    quality: "incomplete",
    dps: { status: "unavailable", reason: "Calculation did not complete" },
  };
  const { compiled } = analysis;
  let raw: unknown = analysis;
  try {
    if (!compiled || analysis.diagnostics.length)
      throw new Error(
        analysis.diagnostics.map((d) => d.message).join("; ") ||
          "Compilation failed",
      );
    if (
      compiled.request.relative_duration_us !== 120_000_000 ||
      (compiled.request.stop ?? "time") !== "time" ||
      compiled.request.clock_contract !== CLOCK_CONTRACT ||
      compiled.request.timer_policy !== TIMER_POLICY
    )
      throw new Error(
        "Team calculations require 120 relative seconds and the current timer policy.",
      );
    engine.startSourceTrials(compiled);
    let offset = 0;
    while (offset < compiled.trials) {
      const update = engine.stepSourceTrials(
        offset,
        Math.min(25, compiled.trials - offset),
        0,
      );
      raw = update;
      if (update.first_execution) {
        entry.assumptions = Object.entries(
          update.first_execution.report.resolved_model_assumptions ?? {},
        ).map(([id, description]) => `${id}: ${description}`);
      }
      if (update.failure)
        throw new Error(
          update.failure.diagnostic?.message ??
            update.failure.report.halt_diagnostic ??
            "Unsupported or incomplete execution",
        );
      if (!update.samples_delta.length)
        throw new Error("Trial session made no progress");
      offset += update.samples_delta.length;
    }
    const aggregate = engine.snapshotSourceTrials(0);
    raw = aggregate;
    entry.quality = aggregate.quality;
    if (
      aggregate.completed !== compiled.trials ||
      !aggregate.game_dps_computed ||
      aggregate.quality === "incomplete" ||
      !aggregate.metrics.dps?.relative
    )
      throw new Error("Incomplete or unsupported DPS calculation");
    entry.dps = { status: "ready", mean: aggregate.metrics.dps.relative.mean };
  } catch (error) {
    entry.dps = {
      status: "unavailable",
      reason: error instanceof Error ? error.message : String(error),
    };
  }
  return { entry, raw };
}

export function cachedCalculation(
  script: { id: string; label: string; source: string },
  identity: Pick<
    TeamCatalog,
    "engine_source_sha256" | "wasm_sha256" | "generator_sha256"
  >,
  cache: string,
  fresh: boolean,
  calculate: () => ReturnType<typeof calculateTeam>,
) {
  const key = sha256(
    JSON.stringify([
      script,
      identity.engine_source_sha256,
      identity.wasm_sha256,
      identity.generator_sha256,
      CLOCK_CONTRACT,
      TIMER_POLICY,
      120_000_000,
    ]),
  );
  const file = path.join(cache, `${key}.json`);
  if (!fresh && existsSync(file))
    return JSON.parse(readFileSync(file, "utf8")) as ReturnType<
      typeof calculateTeam
    >;
  const result = calculate();
  writeFileSync(file, JSON.stringify(result) + "\n");
  return result;
}

async function main() {
  if (process.argv.includes("--check")) {
    checkTeamCatalog();
    return;
  }
  const bytes = readFileSync(
    path.join(root, "target/wasm-bindgen/ntesim_wasm_bg.wasm"),
  );
  const engine = await NtesimWasm.create(bytes);
  const presets = readStudioPresets();
  const identity = engine.version();
  if (identity.engine_source_sha256 !== currentEngineHash())
    throw new Error(
      "Rebuild the current WASM engine before explicitly refreshing Team list.",
    );
  const catalog: TeamCatalog = {
    clock_contract: CLOCK_CONTRACT,
    timer_policy: TIMER_POLICY,
    relative_duration_us: 120_000_000,
    schema_version: 1,
    engine_version: identity.version,
    engine_source_sha256: identity.engine_source_sha256,
    wasm_sha256: sha256(bytes),
    generator_sha256: teamGeneratorHash(),
    entries: [],
  };
  const cache = path.join(root, "cache/studio-teams");
  mkdirSync(cache, { recursive: true });
  for (const script of presets.scripts.filter((s) => s.selectable)) {
    const result = cachedCalculation(
      script,
      catalog,
      cache,
      process.argv.includes("--fresh"),
      () => calculateTeam(engine, script),
    );
    catalog.entries.push(result.entry);
    process.stdout.write(
      `${script.id}: ${result.entry.dps.status === "ready" ? result.entry.dps.mean.toFixed(2) + " DPS" : result.entry.dps.reason}\n`,
    );
  }
  validateTeamCatalog(catalog, presets);
  const prettier = await import(
    pathToFileURL(path.join(root, "cache/web/node_modules/prettier/index.mjs"))
      .href
  );
  writeFileSync(
    teamCatalogPath,
    await prettier.format(JSON.stringify(catalog), {
      ...(await prettier.resolveConfig(teamCatalogPath)),
      filepath: teamCatalogPath,
    }),
  );
}
if (
  process.argv[1] &&
  path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)
)
  await main();
