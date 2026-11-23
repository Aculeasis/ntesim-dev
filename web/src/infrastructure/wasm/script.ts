import type { CompiledConfig } from '../../domain/contracts/display';
import type { ScriptDiagnostic } from '../../domain/contracts/script';

export class ScriptError extends Error {
  code?: ScriptDiagnostic['code'];
  line: number;
  column: number;
  offset?: number;
  length?: number;
  constructor(
    message: string,
    loc: { line: number; column: number; offset?: number; length?: number } = {
      line: 1,
      column: 1,
    },
    code?: ScriptDiagnostic['code'],
  ) {
    super(message);
    this.line = loc.line;
    this.column = loc.column;
    this.offset = loc.offset;
    this.length = loc.length;
    this.code = code;
  }
}

export interface SourceResponse {
  compiled: CompiledConfig | null;
  diagnostics: ScriptDiagnostic[];
}

export function throwDiagnostics(diagnostics: ScriptDiagnostic[]): void {
  const first = diagnostics[0];
  if (first) throw new ScriptError(first.message, first.loc, first.code);
}
