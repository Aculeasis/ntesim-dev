import path from "node:path";

export function selectWebTests(root, available, forwarded) {
  const flags = forwarded
    .filter((arg) => arg.startsWith("--"))
    .map((arg) => {
      const concurrency = /^--test-concurrency=(\d+)$/.exec(arg);
      return concurrency && Number(concurrency[1]) > 4
        ? "--test-concurrency=4"
        : arg;
    });
  const paths = forwarded.filter((arg) => !arg.startsWith("--"));
  const tests = paths.length
    ? paths.map((arg) => {
        const normalized = arg.replaceAll("\\", "/").replace(/^\.\//, "");
        const absolute = path.isAbsolute(arg)
          ? arg
          : path.resolve(
              root,
              normalized.startsWith("tests/")
                ? `web/${normalized}`
                : normalized,
            );
        const relative = path
          .relative(path.join(root, "web"), absolute)
          .replaceAll("\\", "/");
        if (!available.includes(relative))
          throw new Error(`Unknown web test: ${arg}`);
        return relative;
      })
    : available;
  return {
    tests: [...new Set(tests)],
    flags,
    needsBuild: tests.includes("tests/offline.test.mjs"),
  };
}
