export function markerLayout(times: number[], horizon: number, width: number) {
  // Hidden tabs have no measurable width; lay out when the observer sees them.
  if (width <= 0) return { rows: times.map(() => 0), count: 1 };
  const ends: number[] = [];
  const rows = times.map((time) => {
    const left = horizon ? (time / horizon) * width : 0;
    let row = ends.findIndex((end) => left - end >= 27);
    if (row < 0) row = ends.length;
    ends[row] = left;
    return row;
  });
  return { rows, count: Math.max(1, ends.length) };
}
