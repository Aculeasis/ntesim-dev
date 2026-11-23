import { useCallback, useEffect, useRef, useState } from 'react';
import { useConfig } from '../features/editor/useConfig';
import { EditorPage } from '../features/editor/EditorPage';
import type { EditorHandle } from '../features/editor/CodeEditor';
import { useSimulation } from '../features/simulation/useSimulation';
import { RunProgress } from '../features/simulation/RunProgress';
import { ResultsPage } from '../features/results/ResultsPage';
import { SamplePage } from '../features/playback/SamplePage';
import { FailureReport } from '../features/results/FailureReport';
import { downloadFile, generateConfigFilename } from '../infrastructure/storage/files';
import type { ExportFile } from '../features/import-export/ExportDialog';
import { Tabs, type View } from './Tabs';
import { StudioDialogs, type StudioDialog } from './StudioDialogs';
import { ToolsDropdown } from '../features/import-export/ToolsDialog';
import { ReportExport } from '../features/import-export/ReportExport';
import { UpdateNotice } from './UpdateNotice';
import { emptyTeamFilters } from '../domain/team-list';
import { BuildIdentity } from './BuildIdentity';
export function App() {
  const config = useConfig(),
    run = useSimulation(),
    editor = useRef<EditorHandle>(null);
  const [view, setView] = useState<View>('config'),
    [dialog, setDialog] = useState<StudioDialog>(null),
    [toolsPlacement, setToolsPlacement] = useState<'bottom' | 'top'>('bottom'),
    [sampling, setSampling] = useState(false);
  const [teamFilters, setTeamFilters] = useState(emptyTeamFilters);
  const available = !!run.result,
    busy = run.status === 'running' || sampling;
  const show = (view: View) => {
    setView(view);
    window.scrollTo({ top: 0, behavior: 'instant' });
  };
  const start = useCallback(() => {
    if (!config.compiled || busy) return;
    setView('config');
    void run.run(config.source);
  }, [config.compiled, config.source, busy, run.run]);
  useEffect(() => {
    if (available) setView('results');
  }, [available]);
  useEffect(() => {
    const handle = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
        e.preventDefault();
        start();
      }
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') {
        e.preventDefault();
        setView('config');
        setDialog({ kind: 'search', category: 'actions' });
      }
    };
    document.addEventListener('keydown', handle);
    return () => document.removeEventListener('keydown', handle);
  }, [start]);
  const exportFile = (file: ExportFile) => setDialog({ kind: 'export', file });
  return (
    <>
      <a className="skip-link" href="#main-content">
        Skip to simulator
      </a>
      <UpdateNotice busy={busy} />
      <header className="site-header">
        <a href="./" className="brand">
          <span className="brand-mark">N</span>nte<span>sim</span>
        </a>
        <BuildIdentity build={__STUDIO_BUILD__} wasmBuildTime={config.wasmBuildTime} />
      </header>
      <main id="main-content" tabIndex={-1}>
        <div className="studio-navigation">
          <Tabs view={view} available={available} onChange={show} />
          <button
            id="team-list-btn"
            aria-haspopup="dialog"
            onClick={() => setDialog({ kind: 'teams' })}
          >
            Team list
          </button>
        </div>
        <div className="view-header">
          <div className="view-header-title">
            <div className="view-heading">
              <h1 id="view-title">
                {
                  { config: 'Simulator', results: 'Simulation results', sample: 'Sample playback' }[
                    view
                  ]
                }
              </h1>
            </div>
            <p className="intro">
              Configure an NTE team, run trials, inspect every event.{' '}
              <button
                id="language-help"
                className="text-button"
                onClick={() => setDialog({ kind: 'language' })}
              >
                Language &amp; supported mechanics
              </button>
            </p>
          </div>
          <div className="editor-actions" data-view={view}>
            <button
              id="settings-btn"
              disabled={!config.options}
              onClick={() => setDialog({ kind: 'settings' })}
            >
              Settings
            </button>
            <div className="tools-anchor">
              <button
                id="tools-btn"
                aria-haspopup="menu"
                aria-expanded={dialog?.kind === 'tools'}
                onClick={() => setDialog((d) => (d?.kind === 'tools' ? null : { kind: 'tools' }))}
              >
                Tools{' '}
                <span className="dropdown-caret" aria-hidden="true">
                  {toolsPlacement === 'top' && dialog?.kind === 'tools' ? '▴' : '▾'}
                </span>
              </button>
              {dialog?.kind === 'tools' && (
                <ToolsDropdown
                  onPlacementChange={setToolsPlacement}
                  onImportScript={(source) => {
                    config.setSource(source);
                    setView('config');
                    setDialog(null);
                  }}
                  onImportReport={(bundle) => {
                    config.setSource(bundle.source);
                    run.loadReport(
                      bundle.source,
                      bundle.result,
                      bundle.failure,
                      bundle.failedTrial,
                    );
                    setView('results');
                    setDialog(null);
                  }}
                  onExport={() => {
                    const characterIds = config.party?.actors.map((a) => a.character_id) ?? [];
                    downloadFile(generateConfigFilename(characterIds), config.source, 'text/plain');
                    setDialog(null);
                  }}
                  onParameters={() => setDialog({ kind: 'parameters' })}
                  onLanguage={() => setDialog({ kind: 'language' })}
                  onClose={() => setDialog(null)}
                />
              )}
            </div>
            {view === 'results' ? (
              <ReportExport run={run} busy={busy} />
            ) : (
              <button
                id="run-btn"
                className="run-button"
                disabled={!config.compiled || busy}
                onClick={start}
              >
                ▷ Run
              </button>
            )}
            {run.status === 'running' && (
              <button id="stop-btn" onClick={run.cancel}>
                ■ Stop
              </button>
            )}
          </div>
        </div>
        <RunProgress run={run} />
        {run.failedTrial && (
          <FailureReport
            key={run.runId}
            failure={run.failedTrial}
            controller={run.controller}
            busy={busy}
            onBusy={setSampling}
            onExport={exportFile}
          />
        )}
        <section
          id="config"
          role="tabpanel"
          aria-labelledby="tab-config"
          hidden={view !== 'config'}
        >
          <EditorPage
            config={config}
            editorRef={editor}
            onSearch={(category) => setDialog({ kind: 'search', category })}
          />
        </section>
        <section
          id="results"
          role="tabpanel"
          aria-labelledby="tab-results"
          hidden={view !== 'results'}
        >
          {run.result && (
            <ResultsPage run={run} onExportFile={exportFile} onBusy={setSampling} busy={busy} />
          )}
        </section>
        <section
          id="sample-view"
          role="tabpanel"
          aria-labelledby="tab-sample"
          hidden={view !== 'sample'}
        >
          {run.result && (
            <SamplePage
              key={run.runId}
              result={run.result}
              visible={view === 'sample'}
              running={busy}
              controller={run.controller}
              onBusy={setSampling}
              onExport={exportFile}
            />
          )}
        </section>
        <footer>ntesim · Original code and icons MIT · Third-party licenses included</footer>
      </main>
      <StudioDialogs
        dialog={dialog}
        setDialog={setDialog}
        config={config}
        editorRef={editor}
        teamFilters={teamFilters}
        onTeamFilters={setTeamFilters}
        busy={busy}
        onTeamSelect={(id) => {
          if (!busy) {
            config.setPreset(id);
            show('config');
            setDialog(null);
          }
        }}
      />
    </>
  );
}
