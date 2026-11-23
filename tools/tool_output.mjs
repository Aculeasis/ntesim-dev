import { spawnSync } from "node:child_process";
import {
  closeSync,
  mkdirSync,
  openSync,
  readSync,
  statSync,
  writeSync,
} from "node:fs";
import { randomUUID } from "node:crypto";
import { StringDecoder } from "node:string_decoder";
import { fileURLToPath } from "node:url";
import path from "node:path";

const root = fileURLToPath(new URL("../", import.meta.url));
const logs = path.join(root, "cache/tool-logs");

function failureExcerpt(fd, size) {
  const buffer = Buffer.alloc(12000),
    decoder = new StringDecoder("utf8"),
    selected = [];
  let context = 0,
    pending = "";
  const consume = (line) => {
    if (
      /\berror(?:\[|:| TS)|\b[A-Za-z]*Error\b|Exception:|^FAIL[: ]|^ERROR[: ]|^Traceback|panicked at|^not ok|^✖|FAILED/i.test(
        line,
      )
    )
      context = 16;
    if (context) {
      if (
        !/^\s*(?:ok \d|test .+ \.\.\. ok$|✔|✓|PASS\b|# Subtest:|# tests|# pass|# duration_ms|\.{2,}$|Ran \d+ tests|OK$|ℹ (?:tests|suites|pass|cancelled|skipped|todo|duration_ms)\b)| passed[.!]?$/i.test(
          line,
        )
      )
        selected.push(line);
      context--;
    }
  };
  for (let offset = 0; offset < size && selected.length < 60;) {
    const count = readSync(
      fd,
      buffer,
      0,
      Math.min(buffer.length, size - offset),
      offset,
    );
    if (!count) break;
    offset += count;
    pending += decoder.write(buffer.subarray(0, count));
    let newline;
    while ((newline = pending.indexOf("\n")) >= 0 && selected.length < 60) {
      consume(pending.slice(0, newline).replace(/\r$/, ""));
      pending = pending.slice(newline + 1);
    }
    // Also bound memory for reports containing an enormous single JSON line.
    if (pending.length > 12000) {
      consume(pending.slice(0, 12000));
      pending = pending.slice(12000);
    }
  }
  if (pending && selected.length < 60) consume(pending + decoder.end());
  if (selected.length)
    return Buffer.from(selected.join("\n")).subarray(0, 12000).toString("utf8");
  const count = readSync(
    fd,
    buffer,
    0,
    Math.min(size, buffer.length),
    Math.max(0, size - buffer.length),
  );
  return buffer
    .subarray(0, count)
    .toString("utf8")
    .trimEnd()
    .split(/\r?\n/)
    .slice(-60)
    .join("\n");
}

export function runLogged(command, args, options = {}) {
  const { verbose = false, label = "web", ...spawnOptions } = options;
  if (verbose) {
    const result = spawnSync(command, args, {
      ...spawnOptions,
      stdio: "inherit",
    });
    if (result.error) console.error(`ntesim: ${result.error.message}`);
    return result.status ?? 1;
  }
  mkdirSync(logs, { recursive: true });
  const log = path.join(
    logs,
    `${label.replace(/[^a-zA-Z0-9_-]/g, "-")}-${randomUUID()}.log`,
  );
  const fd = openSync(log, "w+");
  let status;
  try {
    const result = spawnSync(command, args, {
      ...spawnOptions,
      stdio: ["inherit", fd, fd],
      env: { ...(spawnOptions.env ?? process.env), NTESIM_TOOL_VERBOSE: "1" },
    });
    if (result.error) writeSync(fd, `${result.error.message}\n`);
    status = result.status ?? 1;
    if (status !== 0) {
      const size = statSync(log).size;
      const tail = failureExcerpt(fd, size);
      console.error(`ntesim: ${label} failed (exit ${status}); log: ${log}`);
      if (tail) console.error(tail);
    }
  } finally {
    closeSync(fd);
  }
  return status;
}
