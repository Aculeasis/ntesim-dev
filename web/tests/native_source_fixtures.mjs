import assert from 'node:assert/strict';
import { readdirSync } from 'node:fs';

// Keep native source transport coverage when a new owner fixture is added.
export const nativeSourceFixtures = readdirSync(new URL('../../presets/native/', import.meta.url))
  .filter((name) => name.endsWith('.nte'))
  .map((name) => name.slice(0, -4))
  .sort();
assert.ok(nativeSourceFixtures.length > 0);
