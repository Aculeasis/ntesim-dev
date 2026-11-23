import type { ExportableSimulationState } from '../src/infrastructure/storage/files';
import type { NtesimWasm } from '../src/infrastructure/wasm/client';
import type { SourceExecution } from '../src/domain/contracts/source';
import type { createReplay } from '../src/domain/replay/model';

export function checkStudioReportRoundtrip(
  state: ExportableSimulationState,
  engine: NtesimWasm,
  execution: SourceExecution,
  model: ReturnType<typeof createReplay>,
): Promise<void>;
