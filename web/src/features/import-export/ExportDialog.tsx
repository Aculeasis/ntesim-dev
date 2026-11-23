import { useRef, useState } from 'react';
import { Dialog } from '../../components/ui/Dialog';
import { downloadFile } from '../../infrastructure/storage/files';
export interface ExportFile {
  name: string;
  text: string;
  type: string;
}
export function ExportDialog({ file, onClose }: { file: ExportFile; onClose: () => void }) {
  const ref = useRef<HTMLTextAreaElement>(null),
    [status, setStatus] = useState('');
  const copy = async () => {
    try {
      await navigator.clipboard.writeText(file.text);
      setStatus('Copied');
    } catch {
      ref.current?.select();
      setStatus('Press Ctrl+C to copy');
    }
  };
  return (
    <Dialog id="export-dialog" title={file.name} onClose={onClose}>
      <textarea
        ref={ref}
        id="export-content"
        aria-label="Export content"
        value={file.text}
        readOnly
        spellCheck={false}
      />
      <div className="export-actions">
        <span id="export-status" role="status">
          {status}
        </span>
        <button id="export-copy" onClick={copy}>
          Copy text
        </button>
        <button
          id="export-download"
          className="run-button"
          onClick={() => downloadFile(file.name, file.text, file.type)}
        >
          Download file
        </button>
      </div>
    </Dialog>
  );
}
