export interface BarRow {
  name: string;
  value: number;
  color?: string;
  min?: number;
  max?: number;
  std?: number;
  detail?: string;
}
export interface Series {
  name: string;
  color?: string;
}
export function barScale(rows: BarRow[], max: number | null = null) {
  const raw = max ?? Math.max(1, ...rows.map((r) => r.max ?? r.value)),
    power = 10 ** Math.floor(Math.log10(raw));
  return (
    max ??
    [1, 1.2, 1.4, 1.5, 1.6, 1.8, 2, 2.2, 2.4, 2.5, 2.8, 3, 3.2, 3.5, 4, 4.5, 5, 6, 7, 8, 9, 10]
      .map((v) => Math.round(v * power * 1e4) / 1e4)
      .find((v) => v >= raw)!
  );
}
export function distributionModel(
  samples: readonly object[],
  key = 'dps',
  basis: 'relative' | 'absolute' = 'relative',
) {
  const values = samples
    .map((s) => {
      const row = s as Record<string, unknown>;
      if (key === 'duration')
        return basis === 'absolute'
          ? typeof row.absolute_elapsed_us === 'number'
            ? row.absolute_elapsed_us / 1e6
            : undefined
          : typeof row.relative_elapsed_us === 'number'
            ? row.relative_elapsed_us / 1e6
            : undefined;
      if (key === 'absolute_duration')
        return typeof row.absolute_elapsed_us === 'number'
          ? row.absolute_elapsed_us / 1e6
          : undefined;
      const target = row[key];
      if (target && typeof target === 'object' && ('relative' in target || 'absolute' in target)) {
        const val = (target as Record<string, unknown>)[basis];
        return typeof val === 'number' && Number.isFinite(val) ? val : undefined;
      }
      return target;
    })
    .filter((v): v is number => typeof v === 'number' && Number.isFinite(v))
    .toSorted((a, b) => a - b);
  if (!values.length || values.length !== samples.length) return null;
  const min = values[0],
    max = values.at(-1)!,
    constant = min === max,
    count = constant ? 1 : 20;
  const padding = constant ? Math.max(Math.abs(min) * 0.05, 1) : 0,
    low = min - padding,
    high = max + padding,
    width = (high - low) / count;
  const bins = Array.from({ length: count }, (_, i) => ({
    low: low + i * width,
    high: low + (i + 1) * width,
    count: 0,
  }));
  for (const v of values)
    bins[constant ? 0 : Math.min(count - 1, Math.floor((v - low) / width))].count++;
  const q = (p: number) => {
    const index = (values.length - 1) * p,
      i = Math.floor(index);
    return values[i] + (values[Math.ceil(index)] - values[i]) * (index - i);
  };
  return {
    min,
    max,
    low,
    high,
    constant,
    bins,
    count: values.length,
    unique: new Set(values).size,
    mean: values.reduce((a, b) => a + b, 0) / values.length,
    p25: q(0.25),
    p50: q(0.5),
    p75: q(0.75),
  };
}
