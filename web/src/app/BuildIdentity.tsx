import { useEffect, useState } from 'react';
import { loadEngine } from '../infrastructure/wasm/load';

export interface StudioBuild {
  commit: string;
  label: string;
  url: string;
  repository_url: string;
  build_started_at: string;
}

export function localBuildTime(instant: string) {
  const date = new Date(instant),
    pad = (value: number) => String(value).padStart(2, '0');
  return `${pad(date.getDate())}.${pad(date.getMonth() + 1)}.${pad(date.getFullYear() % 100)} · ${pad(date.getHours())}:${pad(date.getMinutes())}`;
}

export function BuildIdentity({
  build,
  wasmBuildTime: initialWasmBuildTime,
}: {
  build: StudioBuild;
  wasmBuildTime?: string;
}) {
  const [wasmBuildTime, setWasmBuildTime] = useState<string | undefined>(initialWasmBuildTime);

  useEffect(() => {
    if (initialWasmBuildTime !== undefined) {
      setWasmBuildTime(initialWasmBuildTime);
      return;
    }
    let active = true;
    loadEngine()
      .then((engine) => {
        if (active) setWasmBuildTime(engine.buildStartedAt());
      })
      .catch(() => {});
    return () => {
      active = false;
    };
  }, [initialWasmBuildTime]);

  const instant = wasmBuildTime || build.build_started_at;

  return (
    <div className="build-identity">
      <div className="build-source">
        <a href={build.repository_url} aria-label="GitHub repository" className="build-github">
          <svg viewBox="0 0 24 24" width="16" height="16" aria-hidden="true" fill="currentColor">
            <path d="M12 .75a11.25 11.25 0 0 0-3.56 21.92c.56.1.77-.24.77-.54v-2.09c-3.13.68-3.79-1.33-3.79-1.33-.51-1.3-1.25-1.65-1.25-1.65-1.02-.7.08-.69.08-.69 1.13.08 1.73 1.16 1.73 1.16 1 1.72 2.63 1.22 3.27.93.1-.73.39-1.22.71-1.5-2.5-.28-5.13-1.25-5.13-5.56 0-1.23.44-2.23 1.16-3.02-.12-.28-.5-1.43.11-2.98 0 0 .95-.3 3.1 1.15a10.8 10.8 0 0 1 5.63 0c2.15-1.45 3.1-1.15 3.1-1.15.61 1.55.23 2.7.11 2.98.72.79 1.16 1.79 1.16 3.02 0 4.32-2.63 5.28-5.14 5.56.4.35.76 1.03.76 2.08v3.09c0 .3.2.65.77.54A11.25 11.25 0 0 0 12 .75Z" />
          </svg>
        </a>
        <a
          href={build.url}
          title={build.label}
          aria-label={`Build source: ${build.label}`}
          className="build-ref"
        >
          {build.label}
        </a>
      </div>
      <time dateTime={instant} title={`Build started: ${instant}`}>
        {localBuildTime(instant)}
      </time>
    </div>
  );
}
