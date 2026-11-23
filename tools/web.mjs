import { existsSync, readdirSync, mkdirSync, copyFileSync } from "node:fs";
import { fileURLToPath, pathToFileURL } from "node:url";
import path from "node:path";
import { syncStudioPresets, checkTeamCatalog } from "./studio_presets.mjs";
import { runLogged } from "./tool_output.mjs";
import { selectWebTests } from "./web_tests.mjs";

const root = fileURLToPath(new URL("../", import.meta.url));
const modules = path.join(root, "cache/web/node_modules");
const command = process.argv[2] || "check";
const extra = process.argv.slice(3);
const verbose =
  extra.includes("--verbose") || process.env.NTESIM_TOOL_VERBOSE === "1";
const forwarded = extra.filter((arg) => arg !== "--verbose");
const run = (binary, args, options = {}) =>
  runLogged(binary, args, {
    cwd: root,
    label: `web-${command}`,
    verbose: verbose || command === "dev",
    ...options,
  });
if (command === "install") {
  mkdirSync(path.join(root, "cache/web"), { recursive: true });
  for (const file of ["package.json", "package-lock.json"])
    copyFileSync(
      path.join(root, "web", file),
      path.join(root, "cache/web", file),
    );
  const args = [
    "ci",
    "--prefix",
    "cache/web",
    "--cache",
    "cache/npm",
    "--ignore-scripts",
    "--no-audit",
    "--no-fund",
  ];
  const result =
    process.platform === "win32"
      ? run(process.env.ComSpec || "cmd.exe", [
          "/d",
          "/s",
          "/c",
          "npm.cmd " + args.join(" "),
        ])
      : run("npm", args);
  process.exit(result);
}
const binaries = {
  dev: "vite/bin/vite.js",
  build: "vite/bin/vite.js",
  check: "typescript/bin/tsc",
  format: "prettier/bin/prettier.cjs",
  "format-check": "prettier/bin/prettier.cjs",
  test: "tsx/dist/cli.mjs",
  "verify-trials": "tsx/dist/cli.mjs",
  teams: "tsx/dist/cli.mjs",
  "teams-check": "tsx/dist/cli.mjs",
};
if (!binaries[command] || !existsSync(path.join(modules, binaries[command]))) {
  throw new Error(
    "Install the documented cached web toolchain before running this command.",
  );
}
syncStudioPresets();
if (!["teams", "format", "format-check"].includes(command)) checkTeamCatalog();
if (
  ["dev", "build", "verify-trials"].includes(command) &&
  !existsSync(path.join(root, "target/wasm-bindgen/ntesim_wasm_bg.wasm"))
) {
  throw new Error(
    "Build the browser engine first: python tools/cargo.py build -p ntesim-wasm --release --target wasm32-unknown-unknown --locked --offline",
  );
}
const tests = readdirSync(path.join(root, "web/tests"))
  .filter((name) => /\.test\.(ts|tsx|mjs|cjs)$/.test(name))
  .map((name) => "tests/" + name);
const selection =
  command === "test" ? selectWebTests(root, tests, forwarded) : null;
const args = {
  dev: ["--configLoader", "native", "--host", "127.0.0.1", ...forwarded],
  build: ["build", "--configLoader", "native"],
  check: ["--project", "tsconfig.json"],
  format: [
    "--write",
    "src",
    "tests",
    "vite.config.ts",
    "*.json",
    "index.html",
    "offline-worker.template.js",
    "../tools/web.mjs",
    "../tools/web_tests.mjs",
    "../tools/tool_output.mjs",
    "../tools/throughput.mjs",
    "../tools/throughput_worker.mjs",
    "../tools/browser_acceptance.mjs",
    "../tools/browser_support.mjs",
    "../tools/browser_team_list.mjs",
    "../tools/browser_resume.mjs",
    "../tools/browser_report_exports.mjs",
    "../tools/studio_presets.mjs",
    "../tools/studio_teams.mts",
  ],
  "format-check": [
    "--check",
    "src",
    "tests",
    "vite.config.ts",
    "*.json",
    "index.html",
    "offline-worker.template.js",
    "../tools/web.mjs",
    "../tools/web_tests.mjs",
    "../tools/tool_output.mjs",
    "../tools/throughput.mjs",
    "../tools/throughput_worker.mjs",
    "../tools/browser_acceptance.mjs",
    "../tools/browser_support.mjs",
    "../tools/browser_team_list.mjs",
    "../tools/browser_resume.mjs",
    "../tools/browser_report_exports.mjs",
    "../tools/studio_presets.mjs",
    "../tools/studio_teams.mts",
  ],
  test: [
    "--import",
    pathToFileURL(path.join(modules, "tsx/dist/loader.mjs")).href,
    "--test",
    "--test-concurrency=4",
    "--test-timeout=120000",
    ...(selection?.flags ?? []),
    ...(selection?.tests ?? []),
  ],
  "verify-trials": [
    "--tsconfig",
    "tsconfig.runtime.json",
    "../tools/verify_browser_trials.mjs",
    ...forwarded,
  ],
  teams: [
    "--tsconfig",
    "tsconfig.runtime.json",
    "../tools/studio_teams.mts",
    ...forwarded,
  ],
  "teams-check": [
    "--tsconfig",
    "tsconfig.runtime.json",
    "../tools/studio_teams.mts",
    "--check",
  ],
}[command];
if (command === "test" && selection.needsBuild) {
  const build = run("python", [
    path.join(root, "tools/cpu_limit.py"),
    "--",
    process.execPath,
    path.join(root, "tools/web.mjs"),
    "build",
  ]);
  if (build !== 0) process.exit(build);
}
const result = run(
  ["test", "verify-trials", "teams", "teams-check"].includes(command)
    ? "python"
    : process.execPath,
  ["test", "verify-trials", "teams", "teams-check"].includes(command)
    ? [
        path.join(root, "tools/cpu_limit.py"),
        "--",
        process.execPath,
        ...(command === "test"
          ? args
          : [path.join(modules, binaries[command]), ...args]),
      ]
    : [path.join(modules, binaries[command]), ...args],
  {
    cwd: path.join(root, "web"),
    env: {
      ...process.env,
      NODE_PATH: modules,
      TSX_TSCONFIG_PATH: path.join(root, "web/tsconfig.runtime.json"),
    },
    ...(command === "test"
      ? { timeout: Number(process.env.NTESIM_TEST_TIMEOUT_MS || 180000) }
      : {}),
  },
);
process.exit(result);
