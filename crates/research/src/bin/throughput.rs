//! Explicit development driver over the production source and Studio collector.
use ntesim_content::{
    dsl::compiler::SourceRequest,
    simulation::{PreparedSourceScenario, SourceTrialOutcome},
};
use ntesim_research::trials::studio::{Collector, observe, observe_batch};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::time::Instant;

#[cfg(feature = "performance-diagnostics")]
mod allocations {
    use std::alloc::{GlobalAlloc, Layout, System};
    use std::sync::atomic::{AtomicU64, Ordering::Relaxed};

    struct Counter;
    static COUNT: AtomicU64 = AtomicU64::new(0);
    static BYTES: AtomicU64 = AtomicU64::new(0);
    static LIVE: AtomicU64 = AtomicU64::new(0);
    static PEAK: AtomicU64 = AtomicU64::new(0);

    fn allocated(size: usize) {
        COUNT.fetch_add(1, Relaxed);
        BYTES.fetch_add(size as u64, Relaxed);
        let live = LIVE.fetch_add(size as u64, Relaxed) + size as u64;
        PEAK.fetch_max(live, Relaxed);
    }

    // This allocator exists only in the explicitly selected development binary,
    // never in the library, CLI, WASM or Studio runtime.
    unsafe impl GlobalAlloc for Counter {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            let pointer = unsafe { System.alloc(layout) };
            if !pointer.is_null() {
                allocated(layout.size());
            }
            pointer
        }
        unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
            LIVE.fetch_sub(layout.size() as u64, Relaxed);
            unsafe { System.dealloc(pointer, layout) };
        }
        unsafe fn realloc(&self, pointer: *mut u8, old: Layout, size: usize) -> *mut u8 {
            let result = unsafe { System.realloc(pointer, old, size) };
            if !result.is_null() {
                LIVE.fetch_sub(old.size() as u64, Relaxed);
                allocated(size);
            }
            result
        }
        unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
            let pointer = unsafe { System.alloc_zeroed(layout) };
            if !pointer.is_null() {
                allocated(layout.size());
            }
            pointer
        }
    }
    #[global_allocator]
    static ALLOCATOR: Counter = Counter;

    pub fn snapshot() -> serde_json::Value {
        serde_json::json!({"allocations":COUNT.load(Relaxed), "allocated_bytes":BYTES.load(Relaxed),
            "live_bytes":LIVE.load(Relaxed), "peak_live_bytes":PEAK.load(Relaxed)})
    }
}

fn main() -> Result<(), String> {
    let mut args = std::env::args().skip(1);
    let source_path = args
        .next()
        .ok_or("usage: ntesim-throughput SOURCE COUNT REPETITIONS")?;
    let count: u64 = args
        .next()
        .ok_or("missing count")?
        .parse()
        .map_err(|_| "invalid count")?;
    let repetitions: u32 = args
        .next()
        .ok_or("missing repetitions")?
        .parse()
        .map_err(|_| "invalid repetitions")?;
    let oracle = args.next();
    if args.next().is_some()
        || oracle.as_deref().is_some_and(|arg| arg != "--oracle")
        || count == 0
        || repetitions == 0
    {
        return Err("positive count/repetitions and optional --oracle required".into());
    }
    let source = std::fs::read_to_string(source_path).map_err(|e| e.to_string())?;
    let input = SourceRequest {
        source,
        limits: Default::default(),
    };
    let mut results = Vec::new();
    for repetition in 0..repetitions {
        #[cfg(feature = "performance-diagnostics")]
        let allocation_start = allocations::snapshot();
        let start = Instant::now();
        let compiled = ntesim_content::dsl::source::compile_response(&input);
        let compile_ns = start.elapsed().as_nanos();
        let artifact = compiled
            .compiled
            .ok_or_else(|| format!("{:?}", compiled.diagnostics))?;
        if count > artifact.trials {
            return Err("count exceeds authored trials".into());
        }
        let start = Instant::now();
        let artifact_bytes = serde_json::to_vec(&artifact).map_err(|e| e.to_string())?;
        let export_json_ns = start.elapsed().as_nanos();
        let start = Instant::now();
        let program = PreparedSourceScenario::from_export(&artifact, &Default::default())?;
        let reconstruct_ns = start.elapsed().as_nanos();
        let start = Instant::now();
        let mut collector = Collector::new(artifact.request.clone(), count)?;
        let collector_ns = start.elapsed().as_nanos();
        let mut execution_ns = 0;
        let mut collection_ns = 0;
        let mut events = 0;
        let mut hits = 0;
        let mut observations = Vec::new();
        let mut first_ns = 0;
        let mut oracles = Vec::new();
        for offset in 0..count {
            let seed = artifact
                .request
                .seed
                .checked_add(offset)
                .ok_or("seed overflow")?;
            let start = Instant::now();
            let (execution, observation, duration) = if offset == 0 || oracle.is_some() {
                let detailed = program.run_sample(seed)?;
                let duration = start.elapsed().as_nanos();
                events += detailed.report.total_events_processed;
                hits += detailed.report.hits_settled;
                let observation = observe(&artifact.request, &detailed.report, offset == 0)?;
                (Some(detailed), observation, duration)
            } else {
                let trial = match program.run_batch(seed)? {
                    SourceTrialOutcome::Completed(trial) => trial,
                    SourceTrialOutcome::Failed(detailed) => {
                        return Err(format!(
                            "benchmark seed {seed} did not complete: {:?}",
                            detailed.report.halt_diagnostic
                        ));
                    }
                };
                let duration = start.elapsed().as_nanos();
                events += trial.events;
                hits += trial.hits;
                (None, observe_batch(trial), duration)
            };
            execution_ns += duration;
            if offset == 0 {
                first_ns = duration;
            }
            collector.add_observation(observation.clone())?;
            collection_ns += start.elapsed().as_nanos() - duration;
            if oracle.is_some() {
                // Deliberate untimed oracle output for exact cross-artifact comparisons.
                let execution = execution.expect("detailed oracle execution");
                let mut mechanical = serde_json::to_value(&execution).map_err(|e| e.to_string())?;
                for field in [
                    "engine_source_sha256",
                    "total_damage_dealt",
                    "total_damage_exact",
                ] {
                    mechanical["report"]
                        .as_object_mut()
                        .ok_or("missing report")?
                        .remove(field);
                }
                let mechanical_sha256 = format!(
                    "{:x}",
                    Sha256::digest(serde_json::to_vec(&mechanical).map_err(|e| e.to_string())?)
                );
                let mut rounded = observation.clone();
                let mut total = 0_u128;
                let mut actors = vec![0_u128; rounded.actors.len()];
                let mut targets = vec![0_u128; rounded.targets.len()];
                let mut bins = vec![0_u128; rounded.bins.len()];
                let mut sources = std::collections::BTreeMap::<_, u128>::new();
                let mut rounded_hits = Vec::new();
                for hit in &execution.report.hit_receipts {
                    let damage = u128::from(hit.rounded_damage);
                    total = total.checked_add(damage).ok_or("oracle damage overflow")?;
                    actors[hit.actor_index] = actors[hit.actor_index]
                        .checked_add(damage)
                        .ok_or("oracle actor overflow")?;
                    targets[hit.target_index] = targets[hit.target_index]
                        .checked_add(damage)
                        .ok_or("oracle target overflow")?;
                    let bin = (hit.time_us / rounded.bucket_us) as usize;
                    let bin = bin.min(bins.len() - 1);
                    bins[bin] = bins[bin].checked_add(damage).ok_or("oracle bin overflow")?;
                    let value = sources
                        .entry((
                            hit.actor_index,
                            hit.ability_id.clone(),
                            hit.damage_effect_id.clone(),
                        ))
                        .or_default();
                    *value = value.checked_add(damage).ok_or("oracle source overflow")?;
                    rounded_hits.push(hit.rounded_damage);
                }
                rounded.sample.damage = total as f64;
                rounded.sample.dps.relative = (rounded.sample.relative_elapsed_us > 0)
                    .then(|| total as f64 / rounded.sample.relative_seconds());
                rounded.sample.dps.absolute = (rounded.sample.absolute_elapsed_us > 0)
                    .then(|| total as f64 / rounded.sample.absolute_seconds());
                for (actor, damage) in rounded.actors.iter_mut().zip(actors) {
                    actor.insert("damage".into(), damage as f64);
                }
                for (key, values) in &mut rounded.sources {
                    values.0 = sources[key] as f64;
                }
                rounded.targets = targets.into_iter().map(|v| v as f64).collect();
                rounded.bins = bins;
                oracles.push(json!({"seed":seed,"mechanical_sha256":mechanical_sha256,"rounded_hits":rounded_hits,"rounded_total":total.to_string(),"rounded_observation":rounded}));
            }
            observations.push(observation);
        }
        let start = Instant::now();
        let aggregate = collector.snapshot(0.)?;
        let snapshot_ns = start.elapsed().as_nanos();
        #[cfg(feature = "performance-diagnostics")]
        let diagnostics = json!({
            "spans":ntesim_content::performance::take(),
            "arithmetic_operands":ntesim_core::performance::take_arithmetic_operands(),
        });
        #[cfg(feature = "performance-diagnostics")]
        let allocation_metrics = json!({"start":allocation_start,"end":allocations::snapshot()});
        #[cfg(not(feature = "performance-diagnostics"))]
        let diagnostics = ();
        #[cfg(not(feature = "performance-diagnostics"))]
        let allocation_metrics = ();
        results.push(json!({"repetition":repetition,"compile_ns":compile_ns,"export_json_ns":export_json_ns,
            "reconstruct_ns":reconstruct_ns,"collector_ns":collector_ns,"first_sample_ns":first_ns,
            "execution_ns":execution_ns,"collection_ns":collection_ns,"snapshot_ns":snapshot_ns,
            "artifact_bytes":artifact_bytes.len(),"events":events,"hits":hits,"observations":observations,
            "samples":collector.samples(),"aggregate":aggregate,"diagnostics":diagnostics,"allocations":allocation_metrics,"oracles":oracles}));
    }
    println!(
        "{}",
        json!({"engine_source_sha256":ntesim_contracts::ENGINE_SOURCE_SHA256,
        "engine_version":ntesim_contracts::ENGINE_VERSION,"count":count,"repetitions":results})
    );
    Ok(())
}
