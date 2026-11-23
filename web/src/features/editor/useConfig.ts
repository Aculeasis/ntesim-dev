import { useEffect, useMemo, useState } from 'react';
import { getPresetSource } from '../../domain/presets';
import type { OptionValue } from '../../domain/contracts/display';
import type { SourceCompileResponse } from '../../domain/contracts/script';
import { loadConfig, saveConfig } from '../../infrastructure/storage/config';
import { loadEngine } from '../../infrastructure/wasm/load';
import type { NtesimWasm } from '../../infrastructure/wasm/client';

const empty: SourceCompileResponse = {
  compiled: null,
  party: null,
  options: null,
  diagnostics: [],
};
export function useConfig() {
  const [draft, setDraft] = useState(loadConfig),
    [editor, setEditor] = useState(
      draft.editor ?? { font: 14, theme: 'tomorrow-night', wrap: false },
    ),
    [saved, setSaved] = useState(() => !draft.storageWarning),
    [engine, setEngine] = useState<NtesimWasm>(),
    [wasmBuildTime, setWasmBuildTime] = useState<string>(),
    [engineError, setEngineError] = useState<Error>();
  useEffect(() => {
    let active = true;
    loadEngine()
      .then((wasm) => {
        if (active) {
          setEngine(wasm);
          setWasmBuildTime(wasm.buildStartedAt());
        }
      })
      .catch((error) => {
        if (active) setEngineError(error instanceof Error ? error : new Error(String(error)));
      });
    return () => {
      active = false;
    };
  }, []);
  const analysis = useMemo(() => {
    try {
      if (!engine)
        return { ...empty, error: engineError ?? new Error('Loading WebAssembly engine…') };
      return { ...engine.compileSource(draft.source), error: null };
    } catch (error) {
      return { ...empty, error: error instanceof Error ? error : new Error(String(error)) };
    }
  }, [draft.source, engine, engineError]);
  useEffect(() => {
    if (draft.storageWarning) return;
    setSaved(saveConfig({ ...draft, editor }));
  }, [draft, editor]);
  const setSource = (source: string) => setDraft((v) => ({ source, preset: v.preset }));
  const setPreset = (preset: string) => setDraft({ preset, source: getPresetSource(preset) });
  const changeOptions = (changes: Record<string, OptionValue>) => {
    if (!engine) throw new Error('Loading WebAssembly engine…');
    setSource(engine.editOptions(draft.source, changes));
  };
  return {
    ...draft,
    ...analysis,
    saved,
    setSource,
    setPreset,
    changeOptions,
    editor,
    setEditor,
    wasmBuildTime,
  };
}
export type ConfigState = ReturnType<typeof useConfig>;
