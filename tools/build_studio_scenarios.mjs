import fs from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { fileURLToPath, pathToFileURL } from "node:url";
const root = fileURLToPath(new URL("../", import.meta.url));
const cli = path.join(
  root,
  "target/debug",
  process.platform === "win32" ? "ntesim.exe" : "ntesim",
);
const inputs = {
  s1: "examples/game-scenario.json",
  s2: "examples/s2-game-scenario.json",
  s3: "examples/s3-game-scenario.json",
  s4: "examples/s4-game-scenario.json",
  s5: "examples/s5-game-scenario.json",
};
const scenarios = {};
for (const [id, input] of Object.entries(inputs)) {
  const result = spawnSync(cli, ["run-game", input], {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 32 * 1024 * 1024,
  });
  if (result.status !== 0)
    throw new Error(`${input}: ${result.stderr || result.stdout}`);
  const report = JSON.parse(result.stdout);
  if (
    !report.execution_completed ||
    !report.success ||
    !report.invariants_passed
  )
    throw new Error(`${input}: native execution did not complete successfully`);
  scenarios[id] = JSON.parse(fs.readFileSync(path.join(root, input), "utf8"));
}
const { format } = await import(
  pathToFileURL(path.join(root, "cache/web/node_modules/prettier/index.mjs"))
    .href
);
fs.writeFileSync(
  path.join(root, "web/src/generated/scenarios.json"),
  await format(JSON.stringify(scenarios), {
    parser: "json",
    printWidth: 100,
  }),
);
console.log(
  `Generated ${Object.keys(scenarios).length} natively executed studio scenarios`,
);
