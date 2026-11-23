import type { ScenarioReport } from '../contracts/report';

// Identity validation only; Rust owns follow-up eligibility and damage.
export function validateDamageParent(report: ScenarioReport, hitIndex: number) {
  const hit = report.hit_receipts[hitIndex];
  const parentIndex = hit.parent_hit_index;
  if (parentIndex === undefined) return false;
  if (!Number.isSafeInteger(parentIndex) || parentIndex < 0 || parentIndex >= hitIndex)
    throw new Error('Damage parent must precede its follow-up receipt.');
  const parent = report.hit_receipts[parentIndex];
  if (
    parent.time_us > hit.time_us ||
    parent.actor_index !== hit.actor_index ||
    parent.target_index !== hit.target_index ||
    parent.command_index !== hit.command_index ||
    parent.autonomous_event_index !== hit.autonomous_event_index ||
    JSON.stringify(parent.switch_origin) !== JSON.stringify(hit.switch_origin)
  )
    throw new Error('Damage follow-up differs from its parent identity.');
  return true;
}
