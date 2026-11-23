import catalog from '../generated/catalog.json';
import scenarios from '../generated/scenarios.json';
import type { ScenarioRequest } from './contracts/scenario';
export const CATALOG: {
  characters: { id: string; slug: string; name: string; element: string }[];
  weapons: Record<string, string>;
  sets: Record<string, string>;
} = catalog;
export const CANONICAL_SCENARIO_PAYLOADS = scenarios as Record<string, ScenarioRequest>;
