import { normalizeSource } from './config';
import { object } from './object';
import { loadEngine } from '../wasm/load';
import type { NtesimWasm } from '../wasm/client';
import { parseLosslessJson } from './lossless-json';
import type { TrialResult } from '../../domain/contracts/trials';
import type { FailedSourceTrial } from '../../domain/contracts/source';
import { english } from '../../domain/localization';
import { SCRIPT_FILE_LIMIT, checkReportFileSize, checkReportJsonSize } from './report-limits';
import { isReportContainer } from './report-container';
import { extractPngReport, isPng } from './report-png';
import { reportCodec } from './report-codec';
import { validateReportBundle } from './report-validation';

export interface ImportedReportBundle {
  source: string;
  result: TrialResult;
  failure?: string | null;
  failedTrial?: FailedSourceTrial | null;
  cancelled?: boolean;
}

export async function importFile(
  file: Pick<File, 'size' | 'name' | 'text'>,
  engine?: NtesimWasm,
): Promise<string> {
  if (file.size > SCRIPT_FILE_LIMIT) throw new Error('Import exceeds 50 MB.');
  const text = await file.text();
  const wasm = engine ?? (await loadEngine());
  let source = text;
  if (file.name.toLowerCase().endsWith('.json')) {
    const value = object(parseLosslessJson(text));
    if (typeof value.source === 'string') source = value.source;
    else if (
      value.schema_version === 1 &&
      value.execution &&
      typeof value.execution === 'object' &&
      typeof object(value.execution).source === 'string'
    )
      source = object(value.execution).source as string;
    else {
      throw new Error(
        'An engine request cannot recover the authored rotation or additive build inputs. Import a .nte script or a JSON export containing source.',
      );
    }
  }
  return normalizeSource(source, wasm);
}

export async function importReportFile(
  file: Pick<File, 'size' | 'name' | 'text'> & Partial<Pick<File, 'arrayBuffer'>>,
  engine?: NtesimWasm,
  decode = (bytes: Uint8Array<ArrayBuffer>) => reportCodec('decode', bytes),
): Promise<ImportedReportBundle> {
  checkReportFileSize(file.size);
  let text: string;
  if (file.arrayBuffer) {
    let bytes = new Uint8Array(await file.arrayBuffer());
    checkReportFileSize(bytes.length);
    if (bytes.length !== file.size)
      throw new Error('Damaged report: file length changed while reading.');
    if (isPng(bytes)) bytes = await decode(extractPngReport(bytes).slice());
    else if (isReportContainer(bytes)) bytes = await decode(bytes);
    else if (!file.name.toLowerCase().endsWith('.json'))
      throw new Error(
        'Unsupported report format. Choose JSON (.json), compressed report (.nter) or PNG (.png).',
      );
    checkReportJsonSize(bytes.length);
    try {
      text = new TextDecoder('utf-8', { fatal: true }).decode(bytes);
    } catch {
      throw new Error('Damaged report: invalid UTF-8 JSON.');
    }
  } else {
    if (!file.name.toLowerCase().endsWith('.json'))
      throw new Error('Unsupported report format. Choose JSON format (.json), .nter or .png.');
    text = await file.text();
    const size = new TextEncoder().encode(text).length;
    checkReportFileSize(size);
    checkReportJsonSize(size);
  }
  return importReportJson(text, engine);
}

async function importReportJson(text: string, engine?: NtesimWasm): Promise<ImportedReportBundle> {
  let raw: unknown;
  try {
    raw = parseLosslessJson(text);
  } catch {
    throw new Error('Damaged report: invalid report JSON.');
  }
  const wasm = engine ?? (await loadEngine());
  const value = object(raw);
  const target = value;
  if (typeof value.source !== 'string')
    throw new Error('Report JSON does not contain the simulation source script.');

  if (!target.aggregate || typeof target.aggregate !== 'object') {
    throw new Error(
      "File does not contain simulation report data. Use 'Import script' to import a script only.",
    );
  }

  validateReportBundle(target, value);
  const source = normalizeSource(value.source, wasm);
  const {
    party,
    report,
    samples,
    request,
    compiled,
    actionLocations,
    occurrences,
    trials,
    aggregate,
    warnings,
    partial,
  } = target as unknown as TrialResult;
  const result: TrialResult = {
    party,
    report,
    samples,
    request,
    compiled,
    actionLocations,
    occurrences,
    trials,
    aggregate,
    warnings,
    partial,
  };
  if (value.cancelled && !result.partial)
    throw new Error('Invalid simulation report data: cancelled results must remain partial.');

  return {
    source,
    result,
    failure: typeof value.failure === 'string' ? value.failure : null,
    failedTrial:
      value.failed_trial && typeof value.failed_trial === 'object'
        ? (value.failed_trial as FailedSourceTrial)
        : null,
    cancelled: Boolean(value.cancelled),
  };
}
export interface ExportableSimulationState {
  source: string;
  result: TrialResult | null;
  status: string;
  failure: string | null;
  failedTrial: FailedSourceTrial | null;
}

export function serializeReportBundle(state: ExportableSimulationState, compact = false) {
  if (!state.result) throw new Error('No delivered simulation results are available.');
  const text = JSON.stringify(
    {
      schema_version: 1,
      quality: state.result?.aggregate.quality,
      game_dps_computed: state.result?.aggregate.game_dps_computed,
      source: state.source,
      ...state.result,
      cancelled: state.status === 'stopped',
      failure: state.failure,
      failed_trial: state.failedTrial,
    },
    null,
    compact ? undefined : 2,
  );
  const size = new TextEncoder().encode(text).length;
  checkReportJsonSize(size);
  if (!compact) checkReportFileSize(size);
  return text;
}

export function generateTimestamp(date = new Date()): string {
  const pad = (n: number) => String(n).padStart(2, '0');
  const y = date.getFullYear();
  const m = pad(date.getMonth() + 1);
  const d = pad(date.getDate());
  const h = pad(date.getHours());
  const min = pad(date.getMinutes());
  const s = pad(date.getSeconds());
  return `${y}-${m}-${d}_${h}-${min}-${s}`;
}

export function generateConfigFilename(characters: string[] = []): string {
  const ts = generateTimestamp();
  const chars = characters
    .map((c) =>
      english
        .character(c)
        .toLowerCase()
        .replace(/[^a-z0-9_-]+/g, '-')
        .replace(/^-+|-+$/g, ''),
    )
    .filter(Boolean);
  const charPart = chars.length ? `-${chars.join('-')}` : '';
  return `${ts}${charPart}.nte`;
}

export function generateReportFilename(format: 'json' | 'nter' | 'png' = 'json'): string {
  const ts = generateTimestamp();
  return `${ts}-report.${format}`;
}

export function downloadFile(name: string, content: BlobPart, type = 'application/json') {
  const url = URL.createObjectURL(new Blob([content], { type })),
    a = document.createElement('a');
  a.href = url;
  a.download = name;
  document.body.append(a);
  a.click();
  a.remove();
  setTimeout(() => URL.revokeObjectURL(url), 1000);
}
