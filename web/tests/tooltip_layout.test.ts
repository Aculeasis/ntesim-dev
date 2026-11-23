import assert from 'node:assert/strict';
import { tooltipLayout } from '../src/components/ui/Tooltip';

// Below viewport boundary
{
  const pos = tooltipLayout({
    host: { left: 100, top: 100, bottom: 140 },
    box: { width: 285, height: 80 },
    viewport: { width: 1024, height: 768 },
  });
  assert.deepEqual(pos, { left: 100, top: 148 });
}

// Bottom boundary hit: flips above host
{
  const pos = tooltipLayout({
    host: { left: 100, top: 700, bottom: 740 },
    box: { width: 285, height: 80 },
    viewport: { width: 1024, height: 768 },
  });
  assert.deepEqual(pos, { left: 100, top: 612 });
}

// Both below and above overflow: clamps to viewport without exceeding bounds
{
  const pos = tooltipLayout({
    host: { left: 100, top: 50, bottom: 250 },
    box: { width: 285, height: 260 },
    viewport: { width: 1024, height: 350 },
  });
  assert.deepEqual(pos, { left: 100, top: 82 });
}

// Right boundary hit: clamps to right viewport margin
{
  const pos = tooltipLayout({
    host: { left: 900, top: 100, bottom: 140 },
    box: { width: 285, height: 80 },
    viewport: { width: 1000, height: 768 },
  });
  assert.deepEqual(pos, { left: 707, top: 148 });
}

// Left boundary hit: clamps to left margin 8
{
  const pos = tooltipLayout({
    host: { left: -10, top: 100, bottom: 140 },
    box: { width: 285, height: 80 },
    viewport: { width: 1000, height: 768 },
  });
  assert.deepEqual(pos, { left: 8, top: 148 });
}

// Excessively tall box: clamps to top margin 8
{
  const pos = tooltipLayout({
    host: { left: 50, top: 100, bottom: 150 },
    box: { width: 285, height: 500 },
    viewport: { width: 1000, height: 400 },
  });
  assert.deepEqual(pos, { left: 50, top: 8 });
}

console.log('Tooltip layout: boundary clipping, edge flipping and clamping verified');
