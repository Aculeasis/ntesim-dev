import bundle from '../generated/script-presets.json';

export const DEFAULT_PRESET_ID: string = bundle.default;
export const SCRIPT_PRESETS = bundle.scripts.filter((entry) => entry.selectable);

export function getPresetSource(id = DEFAULT_PRESET_ID): string {
  const preset = bundle.scripts.find((entry) => entry.id === id);
  if (!preset) throw new Error(`Unknown configuration '${id}'.`);
  return preset.source;
}
