import assert from 'node:assert/strict';
import { renderToStaticMarkup } from 'react-dom/server';
import { BuildIdentity, localBuildTime } from '../src/app/BuildIdentity';

const previousZone = process.env.TZ;
try {
  for (const [zone, instant, expected] of [
    ['UTC', '2026-11-14T10:20:30Z', '14.11.26 · 10:20'],
    ['Europe/Istanbul', '2026-11-14T10:20:30Z', '14.11.26 · 13:20'],
    ['UTC', '2026-01-02T03:04:05Z', '02.01.26 · 03:04'],
    ['America/New_York', '2026-03-08T06:59:59Z', '08.03.26 · 01:59'],
    ['America/New_York', '2026-03-08T07:00:00Z', '08.03.26 · 03:00'],
    ['America/New_York', '2026-01-01T02:00:00Z', '31.12.25 · 21:00'],
  ]) {
    process.env.TZ = zone;
    assert.equal(localBuildTime(instant), expected);
  }
} finally {
  if (previousZone === undefined) delete process.env.TZ;
  else process.env.TZ = previousZone;
}
for (const [label, ref] of [
  ['bb887a5', 'bb887a5'],
  ['some/branch/bb887a5', 'bb887a5'],
  ['release/v1+#%', 'refs/tags/release%2Fv1%2B%23%25'],
]) {
  const build = {
    commit: 'bb887a5',
    label,
    url: `https://github.com/Aculeasis/ntesim-dev/tree/${ref}`,
    repository_url: 'https://github.com/Aculeasis/ntesim-dev',
    build_started_at: '2026-11-14T10:20:30Z',
  };
  const markup = renderToStaticMarkup(<BuildIdentity build={build} />);
  assert.ok(markup.includes(`href="${build.url}"`));
  assert.ok(markup.includes(`href="${build.repository_url}"`));
  assert.ok(markup.includes('aria-label="GitHub repository"'));
  assert.ok(markup.includes(`aria-label="Build source: ${label}"`));
  assert.ok(markup.includes(`dateTime="${build.build_started_at}"`));
  assert.equal(renderToStaticMarkup(<BuildIdentity build={build} />), markup);

  const wasmInstant = '2026-11-21T02:00:00Z';
  const withWasm = renderToStaticMarkup(
    <BuildIdentity build={build} wasmBuildTime={wasmInstant} />,
  );
  assert.ok(withWasm.includes(`dateTime="${wasmInstant}"`));
  assert.ok(withWasm.includes(localBuildTime(wasmInstant)));
}
