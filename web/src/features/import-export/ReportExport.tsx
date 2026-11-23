import { useEffect, useRef, useState } from 'react';
import type { SimulationState } from '../simulation/useSimulation';
import {
  downloadFile,
  generateReportFilename,
  serializeReportBundle,
} from '../../infrastructure/storage/files';
import { reportCodec } from '../../infrastructure/storage/report-codec';
import { attachPngReport } from '../../infrastructure/storage/report-png';
import { captureResults } from './report-capture';

export function ReportExport({ run, busy }: { run: SimulationState; busy: boolean }) {
  const [open, setOpen] = useState(false),
    [placement, setPlacement] = useState('bottom');
  const [preparing, setPreparing] = useState(false),
    [error, setError] = useState('');
  const anchor = useRef<HTMLDivElement>(null),
    opener = useRef<HTMLButtonElement>(null);
  const menu = useRef<HTMLDivElement>(null),
    pending = useRef<AbortController | null>(null);
  const current = useRef({ runId: run.runId, result: run.result, busy });
  current.current = { runId: run.runId, result: run.result, busy };
  useEffect(() => () => pending.current?.abort(), []);
  const close = () => {
    setOpen(false);
    opener.current?.focus();
  };
  useEffect(() => {
    if (!open) return;
    const update = () => {
      const rect = anchor.current!.getBoundingClientRect(),
        height = menu.current!.offsetHeight;
      setPlacement(
        window.innerHeight - rect.bottom < height + 8 && rect.top > window.innerHeight - rect.bottom
          ? 'top'
          : 'bottom',
      );
    };
    update();
    menu.current?.querySelector<HTMLButtonElement>('button:not(:disabled)')?.focus();
    const outside = (event: PointerEvent) => {
      if (!anchor.current?.contains(event.target as Node)) setOpen(false);
    };
    const key = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        event.preventDefault();
        close();
      }
    };
    document.addEventListener('pointerdown', outside);
    document.addEventListener('keydown', key);
    window.addEventListener('resize', update);
    window.addEventListener('scroll', update, true);
    return () => {
      document.removeEventListener('pointerdown', outside);
      document.removeEventListener('keydown', key);
      window.removeEventListener('resize', update);
      window.removeEventListener('scroll', update, true);
    };
  }, [open]);
  const exportAs = async (format: 'json' | 'nter' | 'png') => {
    if (pending.current || !run.result || (format === 'png' && busy)) return;
    close();
    setError('');
    setPreparing(true);
    const controller = new AbortController();
    pending.current = controller;
    const identity = current.current;
    const stable = () =>
      !controller.signal.aborted &&
      current.current.runId === identity.runId &&
      current.current.result === identity.result &&
      !current.current.busy;
    try {
      const text = serializeReportBundle(run, format !== 'json');
      if (format === 'json') downloadFile(generateReportFilename(format), text);
      else {
        let image: Blob | undefined;
        if (format === 'png') {
          const region = document.getElementById('report-capture');
          if (!region) throw new Error('Results image is unavailable.');
          image = await captureResults(region, stable);
        }
        const container = await reportCodec(
          'encode',
          new TextEncoder().encode(text),
          controller.signal,
        );
        if (format === 'nter')
          downloadFile(generateReportFilename(format), container, 'application/octet-stream');
        else {
          const output = attachPngReport(new Uint8Array(await image!.arrayBuffer()), container);
          if (!stable()) throw new Error('Results changed during export. Try again.');
          downloadFile(generateReportFilename(format), output, 'image/png');
        }
      }
    } catch (error) {
      if (!controller.signal.aborted)
        setError(error instanceof Error ? error.message : 'Report preparation failed.');
    } finally {
      pending.current = null;
      setPreparing(false);
    }
  };
  return (
    <div className="tools-anchor" ref={anchor}>
      <button
        id="export-report-btn"
        ref={opener}
        disabled={!run.result || preparing}
        title={!run.result ? 'Run a simulation to deliver results before exporting.' : undefined}
        aria-haspopup="menu"
        aria-expanded={open}
        aria-controls="report-export-menu"
        onClick={() => setOpen(!open)}
        onKeyDown={(event) => {
          if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
            event.preventDefault();
            setOpen(true);
          }
        }}
      >
        Export report{' '}
        <span className="dropdown-caret" aria-hidden="true">
          {open && placement === 'top' ? '▴' : '▾'}
        </span>
      </button>
      {open && (
        <div
          id="report-export-menu"
          className="tools-dropdown"
          role="menu"
          ref={menu}
          data-placement={placement}
          onKeyDown={(event) => {
            const items = Array.from(
              menu.current!.querySelectorAll<HTMLButtonElement>('button:not(:disabled)'),
            );
            const index = items.indexOf(document.activeElement as HTMLButtonElement);
            if (['ArrowDown', 'ArrowUp', 'Home', 'End'].includes(event.key)) {
              event.preventDefault();
              items[
                event.key === 'Home'
                  ? 0
                  : event.key === 'End'
                    ? items.length - 1
                    : (index + (event.key === 'ArrowDown' ? 1 : items.length - 1)) % items.length
              ]?.focus();
            } else if (event.key === 'Tab') setOpen(false);
          }}
        >
          <button id="export-report-json" role="menuitem" onClick={() => void exportAs('json')}>
            JSON
          </button>
          <button id="export-report-nter" role="menuitem" onClick={() => void exportAs('nter')}>
            Compressed report
          </button>
          <button
            id="export-report-png"
            role="menuitem"
            disabled={busy}
            aria-describedby={busy ? 'report-png-reason' : undefined}
            onClick={() => void exportAs('png')}
          >
            PNG with report
          </button>
          {busy && (
            <p id="report-png-reason">
              PNG is available when simulation and sample execution finish or stop.
            </p>
          )}
        </div>
      )}
      {preparing && <span role="status">Preparing report…</span>}
      {error && <p role="alert">{error}</p>}
    </div>
  );
}
