// Validate the current bundle contract before any UI state is replaced. This
// boundary does not infer successful observations or rebuild missing run data.
import { ClockIndex } from '../../domain/reports/clocks';
import type { ScenarioReport } from '../../domain/contracts/report';
import type { ScenarioRequest } from '../../domain/contracts/scenario';
import { validateObservations } from '../../domain/reports/validation';
import { CLOCK_CONTRACT, TIMER_POLICY } from '../../domain/contracts/clocks';

export function validateReportBundle(
  target: Record<string, unknown>,
  bundle: Record<string, unknown>,
) {
  const invalid = () => new Error('Missing or invalid simulation report data.');
  const record = (value: unknown): Record<string, unknown> => {
    if (!value || typeof value !== 'object' || Array.isArray(value)) throw invalid();
    return value as Record<string, unknown>;
  };
  const array = (value: unknown): unknown[] => {
    if (!Array.isArray(value)) throw invalid();
    return value;
  };
  const count = (value: unknown) => {
    if (typeof value !== 'number' || !Number.isSafeInteger(value) || value < 0) throw invalid();
    return value;
  };
  if (bundle.schema_version !== 1 || typeof bundle.source !== 'string' || !bundle.source.trim())
    throw invalid();
  if (
    typeof bundle.cancelled !== 'boolean' ||
    !(bundle.failure === null || typeof bundle.failure === 'string')
  )
    throw invalid();
  if (bundle.failed_trial !== null) {
    const failed = record(bundle.failed_trial),
      execution = record(failed.execution);
    record(record(failed.compiled).input);
    record(execution.request);
    record(execution.report);
    new ClockIndex(execution.report as unknown as ScenarioReport);
    array(execution.action_locations);
    array(execution.occurrences);
    if (typeof execution.source !== 'string') throw invalid();
    if (execution.diagnostic !== null) record(execution.diagnostic);
    if (execution.failure_context !== null) record(execution.failure_context);
  }
  const aggregate = record(target.aggregate),
    report = record(target.report),
    request = record(target.request);
  const compiled = record(target.compiled),
    party = record(target.party);
  if (
    typeof target.partial !== 'boolean' ||
    typeof compiled.input !== 'object' ||
    record(compiled.input).source !== bundle.source
  )
    throw invalid();
  if (
    count(aggregate.completed) > count(aggregate.requested) ||
    count(target.trials) !== aggregate.requested
  )
    throw invalid();
  if (array(target.samples).length !== aggregate.completed) throw invalid();
  if (aggregate.completed !== aggregate.requested && !target.partial) throw invalid();
  for (const key of ['quality', 'game_dps_computed']) {
    if (
      typeof aggregate[key] !== (key === 'quality' ? 'string' : 'boolean') ||
      typeof report[key] !== (key === 'quality' ? 'string' : 'boolean')
    )
      throw invalid();
  }
  for (const key of ['actors', 'sources', 'energy_sources', 'targets', 'timeline'])
    array(aggregate[key]);
  const metrics = record(aggregate.metrics);
  if (request.clock_contract !== CLOCK_CONTRACT || request.timer_policy !== TIMER_POLICY)
    throw new Error('Unsupported combat clock contract or timer policy. Regenerate this report.');
  new ClockIndex(report as unknown as ScenarioReport);
  if (
    report.clock_contract !== request.clock_contract ||
    report.timer_policy !== request.timer_policy ||
    report.absolute_limit_us !== request.absolute_limit_us ||
    report.relative_duration_us !== request.relative_duration_us
  )
    throw invalid();
  for (const key of [
    'damage',
    'duration',
    'absolute_duration',
    'hits',
    'events',
    'gross_healing',
    'healing',
    'excess_healing',
    'shield_absorbed',
  ]) {
    const stats = record(metrics[key]);
    for (const field of ['mean', 'min', 'max', 'std'])
      if (typeof stats[field] !== 'number' || !Number.isFinite(stats[field])) throw invalid();
  }
  for (const sample of array(target.samples)) {
    const row = record(sample);
    if (
      !(typeof row.seed === 'number' && Number.isSafeInteger(row.seed) && row.seed >= 0) &&
      !(
        typeof row.seed === 'string' &&
        /^(0|[1-9]\d*)$/.test(row.seed) &&
        BigInt(row.seed) <= 18446744073709551615n
      )
    )
      throw invalid();
    for (const key of ['damage', 'relative_elapsed_us', 'absolute_elapsed_us', 'hits', 'events'])
      if (typeof row[key] !== 'number' || !Number.isFinite(row[key])) throw invalid();
    const relative = count(row.relative_elapsed_us),
      absolute = count(row.absolute_elapsed_us);
    if (relative > absolute || absolute > count(request.absolute_limit_us)) throw invalid();

    for (const key of ['dps', 'eps', 'granted_eps', 'hps', 'shp']) {
      const pair = record(row[key]);
      if (relative === 0) {
        if (pair.relative !== null) throw invalid();
      } else {
        if (
          typeof pair.relative !== 'number' ||
          !Number.isFinite(pair.relative) ||
          pair.relative < 0
        )
          throw invalid();
      }
      if (absolute === 0) {
        if (pair.absolute !== null) throw invalid();
      } else {
        if (
          typeof pair.absolute !== 'number' ||
          !Number.isFinite(pair.absolute) ||
          pair.absolute < 0
        )
          throw invalid();
      }
    }
    const dpsPair = record(row.dps);
    if (relative > 0 && dpsPair.relative !== (row.damage as number) / (relative / 1e6))
      throw invalid();
    if (absolute > 0 && dpsPair.absolute !== (row.damage as number) / (absolute / 1e6))
      throw invalid();
  }
  const firstSample = record(array(target.samples)[0]);
  const firstSampleDps = record(firstSample.dps);
  const firstSampleShp = record(firstSample.shp);
  const reportShp = record(report.available_shield_hp);
  if (
    String(firstSample.seed) !== String(report.seed) ||
    firstSample.relative_elapsed_us !== report.relative_elapsed_us ||
    firstSample.absolute_elapsed_us !== report.absolute_elapsed_us ||
    firstSample.damage !== report.total_damage_dealt ||
    firstSample.hits !== report.hits_settled ||
    firstSample.events !== report.total_events_processed ||
    (firstSampleDps.relative === null) !== !report.game_dps_computed ||
    firstSampleShp.relative !== reportShp.relative ||
    firstSampleShp.absolute !== reportShp.absolute
  )
    throw invalid();
  if (report.relative_elapsed_us === 0) {
    if (reportShp.relative !== null) throw invalid();
  } else {
    if (
      typeof reportShp.relative !== 'number' ||
      !Number.isFinite(reportShp.relative) ||
      (reportShp.relative as number) < 0
    )
      throw invalid();
  }
  if (report.absolute_elapsed_us === 0) {
    if (reportShp.absolute !== null) throw invalid();
  } else {
    if (
      typeof reportShp.absolute !== 'number' ||
      !Number.isFinite(reportShp.absolute) ||
      (reportShp.absolute as number) < 0
    )
      throw invalid();
  }
  const extent =
    array(target.samples).reduce<number>(
      (max, sample) => Math.max(max, count(record(sample).absolute_elapsed_us)),
      0,
    ) / 1e6;
  const timeline = array(aggregate.timeline);
  if (!timeline.length || record(timeline[timeline.length - 1]).time !== extent) throw invalid();
  for (const key of [
    'hit_receipts',
    'healing_receipts',
    'state_timeline',
    'trace',
    'final_actor_states',
    'final_target_states',
  ])
    array(report[key]);
  record(report.final_buff_state);
  array(record(report.final_buff_state).instances);
  for (const hit of array(report.hit_receipts)) {
    const row = record(hit);
    if (typeof row.ability_id !== 'string') throw invalid();
  }
  const statistics = (value: unknown) => {
    const stats = record(value);
    for (const key of ['count', 'mean', 'min', 'max', 'std', 'p25', 'p50', 'p75'])
      if (typeof stats[key] !== 'number' || !Number.isFinite(stats[key])) throw invalid();
  };
  const isAvailable = (key: string, b: 'relative' | 'absolute') =>
    array(target.samples).every((sample) => record(record(sample)[key])[b] !== null);

  for (const key of ['dps', 'eps', 'granted_eps', 'hps', 'shp']) {
    const pair = record(metrics[key]);
    for (const b of ['relative', 'absolute'] as const) {
      if (isAvailable(key, b)) statistics(pair[b]);
      else if (pair[b] !== null) throw invalid();
    }
  }
  const rate = (value: unknown) => {
    const pair = record(value);
    for (const b of ['relative', 'absolute'] as const) {
      if (isAvailable('dps', b)) statistics(pair[b]);
      else if (pair[b] !== null) throw invalid();
    }
  };
  for (const row of array(aggregate.actors)) {
    const actor = record(row);
    count(actor.slot);
    if (typeof actor.character_id !== 'string') throw invalid();
    rate(actor.dps);
    for (const key of ['damage', 'hits', 'energy', 'granted', 'effective']) statistics(actor[key]);
    const field = record(actor.field);
    statistics(field.relative);
    statistics(field.absolute);
    for (const value of Object.values(record(actor.actions))) statistics(value);
  }
  for (const row of array(aggregate.sources)) {
    const entry = record(row);
    count(entry.actor_index);
    if (typeof entry.ability_id !== 'string' || typeof entry.effect_id !== 'string')
      throw invalid();
    rate(entry.dps);
    for (const key of ['damage', 'hits']) statistics(entry[key]);
  }
  for (const row of array(aggregate.energy_sources)) {
    const entry = record(row);
    count(entry.actor_index);
    if (typeof entry.source !== 'string') throw invalid();
    if (typeof entry.kind !== 'string') throw invalid();
    for (const key of ['energy', 'effective', 'overflow']) statistics(entry[key]);
  }
  for (const row of array(aggregate.targets)) {
    const entry = record(row);
    count(entry.target_index);
    statistics(entry.damage);
    rate(entry.dps);
  }
  for (const row of array(aggregate.timeline)) {
    const entry = record(row);
    if (typeof entry.time !== 'number' || !Number.isFinite(entry.time)) throw invalid();
    statistics(entry.dps);
    statistics(entry.cumulative);
  }
  if (array(party.actors).length !== array(request.actors).length || !array(request.targets).length)
    throw invalid();
  for (const actor of array(party.actors)) {
    const row = record(actor);
    if (typeof row.character_id !== 'string') throw invalid();
    record(row.static_stats);
    record(row.build_decomposition);
    array(row.awakening);
    array(row.talent_levels);
    if (!Number.isSafeInteger(row.purchased_qte_level) || Number(row.purchased_qte_level) < 1)
      throw invalid();
  }
  for (const actor of array(request.actors)) record(record(actor).static_stats);
  for (const target of array(request.targets)) record(target);
  array(request.actions);
  array(record(compiled.request).actors);
  array(record(compiled.request).targets);
  for (const key of ['actionLocations', 'occurrences', 'warnings']) array(target[key]);
  if (
    report.schema_version !== 1 ||
    request.schema_version !== 1 ||
    typeof report.engine_source_sha256 !== 'string'
  )
    throw invalid();
  validateObservations(request as unknown as ScenarioRequest, report as unknown as ScenarioReport);
}
