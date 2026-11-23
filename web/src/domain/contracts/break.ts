import type { ExactFraction } from './scenario';

export interface BreakContribution {
  generation: number;
  character_level: number;
  base_break: ExactFraction;
  original_capacity: ExactFraction;
  intensity: ExactFraction;
  bonus: ExactFraction;
  defense_ignore: ExactFraction;
}
