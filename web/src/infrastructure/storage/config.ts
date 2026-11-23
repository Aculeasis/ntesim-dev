import { DEFAULT_PRESET_ID, getPresetSource } from '../../domain/presets';
import { object } from './object';
import type { NtesimWasm } from '../wasm/client';
import type { DisplayConfig } from '../../domain/contracts/display';
import { throwDiagnostics } from '../wasm/script';
export interface SavedConfig {
  source: string;
  preset: string;
  storageWarning?: string;
  editor?: DisplayConfig['editor'];
}
export const CONFIG_KEY = 'ntesim_config_v3';
function browserStorage(): Storage | undefined {
  try {
    return globalThis.localStorage;
  } catch {
    return undefined;
  }
}
export function normalizeSource(source: string, engine: NtesimWasm): string {
  const response = engine.compileSource(source);
  throwDiagnostics(response.diagnostics);
  if (!response.compiled) throw new Error('Missing compiled source response.');
  return source;
}
export function loadConfig(
  storage: Pick<Storage, 'getItem'> | undefined = browserStorage(),
): SavedConfig {
  try {
    const raw = storage?.getItem(CONFIG_KEY);
    if (raw) {
      const value = object(JSON.parse(raw));
      const settings = value.editor && typeof value.editor === 'object' ? object(value.editor) : {};
      const editor = {
        font:
          typeof settings.font === 'number' && [13, 14, 16, 18].includes(settings.font)
            ? settings.font
            : 14,
        theme:
          typeof settings.theme === 'string' &&
          ['tomorrow-night', 'monokai', 'light'].includes(settings.theme)
            ? settings.theme
            : 'tomorrow-night',
        wrap: settings.wrap === true,
      };
      if (typeof value.source === 'string')
        return {
          source: value.source,
          preset: typeof value.preset === 'string' ? value.preset : DEFAULT_PRESET_ID,
          editor,
        };
      return {
        source: getPresetSource(),
        preset: DEFAULT_PRESET_ID,
        storageWarning:
          'Saved config has no source and was left untouched. Edit or load a config to replace it.',
      };
    }
  } catch {
    return {
      source: getPresetSource(),
      preset: DEFAULT_PRESET_ID,
      storageWarning:
        'Saved config could not be read and was left untouched. Edit or load a config to replace it.',
    };
  }
  return { source: getPresetSource(), preset: DEFAULT_PRESET_ID };
}
export function saveConfig(
  config: SavedConfig,
  storage: Pick<Storage, 'setItem'> | undefined = browserStorage(),
): boolean {
  try {
    storage?.setItem(CONFIG_KEY, JSON.stringify({ schema_version: 3, ...config }));
    return !!storage;
  } catch {
    return false;
  }
}
