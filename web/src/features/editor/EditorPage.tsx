import type { RefObject } from 'react';
import { useState } from 'react';
import type { ConfigState } from './useConfig';
import { CodeEditor, type EditorHandle } from './CodeEditor';
import { TeamCards } from '../team/TeamCards';
import { Select } from '../../components/ui/Panel';
import { errorMessage } from '../../components/ui/format';
import type { OptionValue } from '../../domain/contracts/display';
export function EditorPage({
  config: c,
  editorRef,
  onSearch,
}: {
  config: ConfigState;
  editorRef: RefObject<EditorHandle | null>;
  onSearch: (category: string) => void;
}) {
  const [optionError, setOptionError] = useState(''),
    editor = c.editor;
  const change = (values: Record<string, OptionValue>) => {
    try {
      c.changeOptions(values);
      setOptionError('');
    } catch (error) {
      setOptionError(errorMessage(error));
    }
  };
  return (
    <>
      <div className="section-title">
        <span>Team</span>
      </div>
      <div className="team-grid" id="team-cards" aria-live="polite">
        {c.party && (
          <TeamCards party={c.party} onNavigate={(loc) => editorRef.current?.reveal(loc)} />
        )}
      </div>
      <div className="section-title">
        <span>Name Search</span>
      </div>
      <div className="name-search">
        {[
          ['characters', '♧ Characters'],
          ['weapons', '⚔ Weapons'],
          ['cartridges', '◇ Cartridges'],
          ['targets', '♙ Targets'],
          ['actions', '♧ Actions'],
          ['stats', '▥ Stats'],
        ].map(([category, label]) => (
          <button key={category} data-search={category} onClick={() => onSearch(category)}>
            {label}
          </button>
        ))}
      </div>
      <div className="section-title">
        <span>Action List</span>
      </div>
      <div className="editor-toolbar">
        <span id="save-status">
          {c.storageWarning || (c.saved ? 'Saved locally' : 'Storage unavailable')}
        </span>
        <Select
          id="font-size"
          label="Font Size"
          value={String(editor.font)}
          onChange={(v) => c.setEditor({ ...editor, font: Number(v) })}
          options={['13', '14', '16', '18'].map((v) => [v, v])}
        />
        <Select
          id="editor-theme"
          label="Editor Theme"
          value={editor.theme}
          onChange={(v) => c.setEditor({ ...editor, theme: v })}
          options={[
            ['tomorrow-night', 'tomorrow_night'],
            ['monokai', 'monokai'],
            ['light', 'github'],
          ]}
        />
        <button
          id="wrap-btn"
          aria-pressed={editor.wrap}
          onClick={() => c.setEditor({ ...editor, wrap: !editor.wrap })}
        >
          Wrap lines
        </button>
      </div>
      <CodeEditor ref={editorRef} source={c.source} onChange={c.setSource} editor={editor} />
      <div className="editor-status">
        <span id="syntax-status" role="status">
          {c.compiled
            ? `✓ ${c.compiled.request.actors.length} characters · ${c.compiled.trials} iterations`
            : 'Configuration needs attention'}
        </span>
      </div>
      {(c.error || optionError || c.diagnostics.length > 0) && (
        <div id="diagnostics" role="alert">
          {c.error?.message || optionError}
          {c.diagnostics.map((diagnostic, index) => (
            <p key={index}>
              <button
                className="text-button"
                data-diagnostic-code={diagnostic.code}
                onClick={() => editorRef.current?.reveal(diagnostic.loc)}
              >
                Line {diagnostic.loc.line}, column {diagnostic.loc.column} · {diagnostic.code}:{' '}
                {diagnostic.message}
              </button>
            </p>
          ))}
        </div>
      )}
    </>
  );
}
