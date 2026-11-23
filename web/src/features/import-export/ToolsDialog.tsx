import { useEffect, useRef, useState } from 'react';
import {
  importFile,
  importReportFile,
  type ImportedReportBundle,
} from '../../infrastructure/storage/files';
import { errorMessage } from '../../components/ui/format';
export function ToolsDialog({
  onImportScript,
  onImport,
  onImportReport,
  onExport,
  onParameters,
  onLanguage,
  onClose,
  onPlacementChange,
}: {
  onImportScript?: (source: string) => void;
  onImport?: (source: string) => void;
  onImportReport?: (report: ImportedReportBundle) => void;
  onExport: () => void;
  onParameters: () => void;
  onLanguage: () => void;
  onClose: () => void;
  onPlacementChange?: (placement: 'bottom' | 'top') => void;
}) {
  const container = useRef<HTMLDivElement>(null),
    scriptInput = useRef<HTMLInputElement>(null),
    reportInput = useRef<HTMLInputElement>(null),
    [placement, setPlacement] = useState<'bottom' | 'top'>('bottom'),
    [error, setError] = useState('');

  const importScriptHandler = onImportScript ?? onImport;

  useEffect(() => {
    const update = () => {
      const el = container.current;
      if (!el) return;
      const parent = el.parentElement;
      const rect = (parent ?? el).getBoundingClientRect();
      const height = el.offsetHeight || 220;
      const spaceBelow = window.innerHeight - rect.bottom;
      const spaceAbove = rect.top;
      const next = spaceBelow < height + 8 && spaceAbove > spaceBelow ? 'top' : 'bottom';
      setPlacement(next);
      onPlacementChange?.(next);
    };
    update();
    window.addEventListener('resize', update);
    window.addEventListener('scroll', update, true);
    return () => {
      window.removeEventListener('resize', update);
      window.removeEventListener('scroll', update, true);
    };
  }, [onPlacementChange]);

  useEffect(() => {
    const onPointer = (e: MouseEvent) => {
      if (container.current && !container.current.parentElement?.contains(e.target as Node)) {
        onClose();
      }
    };
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    };
    document.addEventListener('pointerdown', onPointer);
    document.addEventListener('keydown', onKey);
    return () => {
      document.removeEventListener('pointerdown', onPointer);
      document.removeEventListener('keydown', onKey);
    };
  }, [onClose]);

  const loadScript = async (file?: File) => {
    if (!file) return;
    try {
      if (importScriptHandler) {
        importScriptHandler(await importFile(file));
      }
      onClose();
    } catch (error) {
      setError(errorMessage(error));
    } finally {
      if (scriptInput.current) scriptInput.current.value = '';
    }
  };

  const loadReport = async (file?: File) => {
    if (!file) return;
    try {
      if (onImportReport) {
        onImportReport(await importReportFile(file));
      }
      onClose();
    } catch (error) {
      setError(errorMessage(error));
    } finally {
      if (reportInput.current) reportInput.current.value = '';
    }
  };

  return (
    <div
      ref={container}
      id="tools-dialog"
      className="tools-dropdown"
      data-placement={placement}
      role="menu"
    >
      <button id="import-btn" role="menuitem" onClick={() => scriptInput.current?.click()}>
        Import script
      </button>
      <button id="import-report-btn" role="menuitem" onClick={() => reportInput.current?.click()}>
        Import report
      </button>
      <button id="export-script" role="menuitem" onClick={onExport}>
        Export script
      </button>
      <button id="expand-config" role="menuitem" onClick={onParameters}>
        Inspect compiled model
      </button>
      <button id="language-tool" role="menuitem" onClick={onLanguage}>
        Language reference
      </button>
      <input
        ref={scriptInput}
        id="import-input"
        type="file"
        accept=".txt,.nte,.json"
        hidden
        onChange={(e) => loadScript(e.target.files?.[0])}
      />
      <input
        ref={reportInput}
        id="import-report-input"
        type="file"
        accept=".json,.nter,.png"
        hidden
        onChange={(e) => loadReport(e.target.files?.[0])}
      />
      {error && <p role="alert">{error}</p>}
    </div>
  );
}

export const ToolsDropdown = ToolsDialog;
