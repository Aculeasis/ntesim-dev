import type { SignedFraction } from './ratio';
import type { ScenarioRequest, ScenarioProgramRequest } from './scenario';
export interface ActorDisplay {
  maxLevel?: number;
  setCount?: number;
  snapshot?: boolean;
  contributions?: Record<string, number> | null;
  extraStats?: Record<string, number>;
  base?: { atk?: number; hp?: number; def?: number };
  unmodeledAttributes?: Record<string, SignedFraction>;
}
export interface DisplayConfig {
  actors: Record<string, ActorDisplay>;
  targets: { level?: number; radius?: number }[];
  editor: { font: number; theme: string; wrap: boolean };
}
export type OptionValue = string | number | null;
export interface OptionRange {
  name: string;
  start: number;
  valueStart: number;
  end: number;
}
export interface CompiledConfig {
  request: ScenarioRequest;
  trials: number;
  display: DisplayConfig;
  warnings: string[];
  language: string;
  optionRanges: OptionRange[];
  options: Record<string, OptionValue>;
  steps: number;
  actionLocations: { line: number; column: number }[];
  incomingDamageLocations?: { line: number; column: number }[];
  liveProgram?: ScenarioProgramRequest;
}
