import type { ExactFraction, Ratio } from './ratio';

export interface CombatStaminaProfile {
  capacity: Ratio;
  initial: Ratio;
  use_efficiency: Ratio;
  recovery_rate: Ratio;
}

export interface CombatStaminaReport {
  profile: CombatStaminaProfile;
  current: ExactFraction;
  total_spent: ExactFraction;
  total_recovered: ExactFraction;
  recovery_ready_at_us: number;
  updated_at_us: number;
  spends: {
    command_index: number;
    actor_index: number;
    time_us: number;
    amount: ExactFraction;
    before: ExactFraction;
    after: ExactFraction;
  }[];
}
