import { renderToStaticMarkup } from 'react-dom/server';
import { StackedBars } from '../src/components/charts/StackedBars';
import type { Series } from '../src/components/charts/models';
import type { ComponentProps } from 'react';
import { CharacterStats } from '../src/features/team/CharacterStats';
import { BarChart } from '../src/components/charts/BarChart';
export function stackedBars(rows: { name: string; values: number[] }[], series: Series[]) {
  return renderToStaticMarkup(<StackedBars rows={rows} series={series} />);
}
export function cardStats(actor: ComponentProps<typeof CharacterStats>['actor'], mode = 'total') {
  return renderToStaticMarkup(<CharacterStats actor={actor} mode={mode} />);
}
export function horizontalBars(rows: ComponentProps<typeof BarChart>['rows']) {
  return renderToStaticMarkup(<BarChart rows={rows} />);
}
