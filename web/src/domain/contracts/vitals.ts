export interface HpCostEvent {
  timestamp_us: number;
  actor_index: number;
  source: string;
  amount: number | bigint;
}

export interface ActorDamageReceipt {
  event_index: number;
  time_us: number;
  actor_index: number;
  delivered_damage: number;
  prevented_hp_loss?: number;
  absorbed_by_shields?: { shield: string; amount: number }[];
  absorbed_by_effects?: {
    effect_id: string;
    source_actor_index: number;
    broken_shield: string;
    amount: number;
  }[];
  hp_before: number;
  actual_hp_loss: number;
  overflow: number;
  hp_after: number;
  defeated: boolean;
}

export interface HpCostReceipt {
  event_index: number;
  time_us: number;
  actor_index: number;
  source: string;
  requested_hp: number | string;
  hp_before: number;
  actual_hp_loss: number;
  hp_after: number;
}
