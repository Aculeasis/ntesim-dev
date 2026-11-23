export interface StudioPresetBundle {
  default: string;
  scripts: { id: string; label: string; selectable: boolean; source: string }[];
}

export const presetDirectory: string;
export const presetBundlePath: string;
export function readStudioPresets(directory?: string): StudioPresetBundle;
export function syncStudioPresets(): void;
export const teamCatalogPath: string;
export function sha256(bytes: string | Uint8Array): string;
export function teamGeneratorHash(): string;
export function currentEngineHash(): string;
export function checkTeamCatalog(): void;
export function validateTeamCatalog(catalog: unknown, presets: StudioPresetBundle, identity: { engine: string; wasm: string; generator: string }): void;
