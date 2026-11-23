import assert from 'node:assert/strict';
import { readdirSync, readFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
const root = fileURLToPath(new URL('../src/', import.meta.url));
function walk(dir) {
  return readdirSync(dir, { withFileTypes: true }).flatMap((e) =>
    e.isDirectory() ? walk(path.join(dir, e.name)) : [path.join(dir, e.name)],
  );
}
const files = walk(root),
  code = files.filter((f) => /\.(ts|tsx|css)$/.test(f));
for (const file of walk(fileURLToPath(new URL('../tests/', import.meta.url)))) {
  if (!/\.(mjs|ts|tsx)$/.test(file)) continue;
  assert.ok(
    readFileSync(file, 'utf8').split('\n').length <= 350,
    `${file}: split test modules exceeding 350 lines`,
  );
}
for (const file of code) {
  const source = readFileSync(file, 'utf8');
  assert.ok(source.split('\n').length <= 350, `${file}: split modules exceeding 350 lines`);
  assert.doesNotMatch(source, /@ts-ignore|@ts-nocheck|\bas any\b|dangerouslySetInnerHTML/);
  assert.doesNotMatch(file, /kqm/i);
  if (file.includes(path.join('src', 'domain')))
    assert.doesNotMatch(
      source,
      /from ['"][^'"]*(?:features|components|infrastructure|react)[^'"]*['"]/,
      'Domain cannot depend on UI or infrastructure',
    );
  if (file.includes(path.join('src', 'infrastructure')))
    assert.doesNotMatch(
      source,
      /from ['"][^'"]*(?:features|\/app\/|components)[^'"]*['"]/,
      'Infrastructure cannot depend on features',
    );
}
assert.ok(!files.some((f) => f.endsWith('.js')), 'Application source is TypeScript');
assert.ok(
  !files.some((file) => file.includes(path.join('domain', 'language'))),
  'Rust owns the source compiler',
);
for (const file of code)
  assert.doesNotMatch(
    readFileSync(file, 'utf8'),
    /class ExactNumber|class Parser\b|function compileNte\b/,
    'No shadow compiler or exact arithmetic in Studio',
  );
console.log(
  `Architecture: ${code.length} modules/styles under 350 lines, inward dependencies and no untyped escape hatches or legacy application JS`,
);
