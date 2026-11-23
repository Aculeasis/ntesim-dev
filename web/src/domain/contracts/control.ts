export interface ControlApplication {
  kind: 'suppress';
  source_actor_index: number;
  target_index: number;
  command_index?: number;
  autonomous_event_index?: number;
  duration_us: number;
  expires_at_us: number;
}
