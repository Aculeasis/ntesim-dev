import assert from 'node:assert/strict';
import { markerLayout } from '../src/features/playback/markerLayout';

// S5's opening Mint normal/burst/skill buttons overlapped in a 120s lane.
const times = [9.15e6, 9.95e6, 11.95e6, 13.15e6, 13.95e6, 14.75e6, 15.55e6, 16.35e6];
for (const width of [180, 600, 1200]) {
  const layout = markerLayout(times, 120e6, width);
  assert.equal(layout.rows.length, times.length);
  for (let i = 0; i < times.length; i++)
    for (let j = i + 1; j < times.length; j++)
      if (layout.rows[i] === layout.rows[j])
        assert.ok(((times[j] - times[i]) / 120e6) * width >= 27);
  assert.ok(layout.count > 1);
}
assert.deepEqual(markerLayout([0, 1, 2], 0, 100), { rows: [0, 1, 2], count: 3 });
assert.deepEqual(markerLayout([], 100, 100), { rows: [], count: 1 });
assert.deepEqual(markerLayout(times, 120e6, 0), { rows: times.map(() => 0), count: 1 });
assert.deepEqual(markerLayout([0, 50, 100], 100, 100), { rows: [0, 0, 0], count: 1 });
console.log(
  'Replay markers: dense S5 actions remain separately clickable at narrow and wide widths',
);
