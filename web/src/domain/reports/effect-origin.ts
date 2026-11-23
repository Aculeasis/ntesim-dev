import type { ScenarioReport } from '../contracts/report';

/** Structural causal links; reaction eligibility and damage belong to Rust. */
export function validateReactionOrigin(report: ScenarioReport, hitIndex: number) {
  const hit = report.hit_receipts[hitIndex];
  const origin = hit.switch_origin;
  if (origin?.kind !== 'esper_cycle_replacement') return;
  const index = origin.trigger_hit_index;
  const trigger = report.hit_receipts[index];
  if (
    !Number.isSafeInteger(index) ||
    index < 0 ||
    index >= hitIndex ||
    !trigger ||
    trigger.time_us > hit.time_us
  )
    throw new Error(
      `Reaction replacement has an invalid earlier hit link (${hitIndex}->${index}, target ${hit.target_index}->${trigger?.target_index}, time ${hit.time_us}->${trigger?.time_us}).`,
    );
}
