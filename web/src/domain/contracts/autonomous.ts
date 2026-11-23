import type { ExactFraction } from './scenario';

export type CausalTrigger =
  | { kind: 'initial_state' }
  | { kind: 'field_entry' | 'action'; command_index: number }
  | { kind: 'incoming_damage' | 'hp_cost'; event_index: number }
  | { kind: 'vital_change'; event_sequence: number }
  | { kind: 'coordinated_hit'; cast_command_index: number; hit_index: number }
  | { kind: 'settled_hit'; hit_index: number }
  | { kind: 'owner_deadline'; autonomous_event_index: number };
export interface AutonomousEventReceipt {
  actor_index: number;
  active_actor_index: number;
  ability_id: string;
  time_us: number;
  trigger: CausalTrigger;
  recipient_healing?: {
    recipient_actor_index: number;
    base: ExactFraction;
    gross: number;
    applied: number;
    excess: number;
    hp_before: number;
    hp_after: number;
  };
}
