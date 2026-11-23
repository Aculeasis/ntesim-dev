import type { ActorInspection, CompiledOptions } from './script';
import type { Target } from './target';

export type TeamActor = Pick<
  ActorInspection,
  'character_id' | 'awakening' | 'level' | 'weapon' | 'set'
>;
export interface TeamEntry {
  id: string;
  label: string;
  source_sha256: string;
  actors: TeamActor[];
  options: CompiledOptions | null;
  targets: Target[];
  assumptions: string[];
  quality: string;
  dps: { status: 'ready'; mean: number } | { status: 'unavailable'; reason: string };
}
export interface TeamCatalog {
  clock_contract: string;
  timer_policy: string;
  relative_duration_us: number;
  schema_version: number;
  engine_version: string;
  engine_source_sha256: string;
  wasm_sha256: string;
  generator_sha256: string;
  entries: TeamEntry[];
}
