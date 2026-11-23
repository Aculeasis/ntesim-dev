/** Original admission and optional earlier-hit cause of a delivered effect. */
export type SwitchOrigin =
  | { kind: 'support'; incoming_actor_index: number }
  | { kind: 'esper_cycle'; outgoing_actor_index: number; incoming_actor_index: number }
  | {
      kind: 'esper_cycle_replacement';
      outgoing_actor_index: number;
      incoming_actor_index: number;
      owner_actor_index: number;
      trigger_hit_index: number;
    };
