import type { RefObject } from 'react';
import type { ConfigState } from '../features/editor/useConfig';
import type { EditorHandle } from '../features/editor/CodeEditor';
import { SettingsDialog } from '../features/settings/SettingsDialog';
import { SearchDialog } from '../features/editor/SearchDialog';
import { ExportDialog, type ExportFile } from '../features/import-export/ExportDialog';
import { LanguageReference } from '../features/editor/LanguageReference';
import { Dialog } from '../components/ui/Dialog';
import { TeamListDialog } from '../features/team/TeamListDialog';
import type { TeamFilters } from '../domain/team-list';
export type StudioDialog =
  | { kind: 'settings' | 'tools' | 'language' | 'parameters' | 'teams' }
  | { kind: 'search'; category: string }
  | { kind: 'export'; file: ExportFile }
  | null;
export function StudioDialogs({
  dialog,
  setDialog,
  config: c,
  editorRef,
  teamFilters,
  onTeamFilters,
  onTeamSelect,
  busy,
}: {
  dialog: StudioDialog;
  setDialog: (dialog: StudioDialog) => void;
  config: ConfigState;
  editorRef: RefObject<EditorHandle | null>;
  teamFilters: TeamFilters;
  onTeamFilters: (filters: TeamFilters) => void;
  onTeamSelect: (id: string) => void;
  busy: boolean;
}) {
  const close = () => setDialog(null);
  if (!dialog) return null;
  switch (dialog.kind) {
    case 'teams':
      return (
        <TeamListDialog
          filters={teamFilters}
          onFilters={onTeamFilters}
          onSelect={onTeamSelect}
          onClose={close}
          busy={busy}
        />
      );
    case 'settings':
      return c.options ? (
        <SettingsDialog options={c.options} onApply={c.changeOptions} onClose={close} />
      ) : null;
    case 'search':
      return (
        <SearchDialog
          category={dialog.category}
          onInsert={(code) => editorRef.current?.insert('\n' + code + '\n')}
          onClose={close}
        />
      );
    case 'export':
      return <ExportDialog file={dialog.file} onClose={close} />;
    case 'tools':
      return null;
    case 'language':
      return (
        <Dialog id="content-dialog" title="NTE DSL · Rotation syntax" onClose={close}>
          <LanguageReference />
        </Dialog>
      );
    case 'parameters':
      return (
        <Dialog id="content-dialog" title="Effective configuration" onClose={close}>
          <pre>
            {JSON.stringify(
              c.party
                ? {
                    options: c.options,
                    party: c.party,
                    compiled: c.compiled,
                    diagnostics: c.diagnostics,
                  }
                : { error: c.error?.message, diagnostics: c.diagnostics },
              null,
              2,
            )}
          </pre>
        </Dialog>
      );
  }
}
