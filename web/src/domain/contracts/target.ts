import type { Ratio } from './scenario';

export interface Target {
  control_immune?: boolean;
  base_defense?: number;
  target_index: number;
  is_boss?: boolean;
  level?: number;
  hp_max: number;
  hp_current: number;
  defense: number;
  resistance: Ratio;
  elemental_resistances?: Partial<
    Record<'NATURE' | 'CHAOS' | 'COSMOS' | 'INCANTATION' | 'LAKSHANA' | 'PSYCHE', Ratio>
  >;
}
