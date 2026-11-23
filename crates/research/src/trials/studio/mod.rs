//! Rust-owned streaming Studio observations shared by native and WASM consumers.
use ntesim_content::simulation::{GameScenarioReport, GameScenarioRequest};
use ntesim_contracts::{ENGINE_SOURCE_SHA256, ENGINE_VERSION, Quality};
use std::collections::BTreeSet;

mod model;
mod observe;
mod statistics;
use model::*;
pub use model::{Aggregate, AggregateMetrics, Observation, ReactionAggregate, Sample};
pub use observe::{observe, observe_batch};
use statistics::{Statistics, summarize};

pub struct Collector {
    request: GameScenarioRequest,
    requested: u64,
    rows: Vec<Observation>,
    bucket_seconds: f64,
    bins_count: usize,
    quality: Quality,
}

impl Collector {
    pub fn new(request: GameScenarioRequest, requested: u64) -> Result<Self, String> {
        let horizon = request.relative_duration_us as f64 / 1e6;
        if requested == 0 || horizon <= 0. {
            return Err("trials and duration must be positive".into());
        }
        Ok(Self {
            request,
            requested,
            rows: Vec::new(),
            bucket_seconds: 1.,
            bins_count: 0,
            quality: Quality::Incomplete,
        })
    }

    pub fn completed(&self) -> usize {
        self.rows.len()
    }

    pub fn add(&mut self, report: &GameScenarioReport) -> Result<(), String> {
        let seed = self
            .request
            .seed
            .checked_add(self.rows.len() as u64)
            .ok_or("seed overflow")?;
        if self.rows.len() as u64 >= self.requested
            || report.content_id != self.request.content_id
            || report.relative_duration_us != self.request.relative_duration_us
            || report.absolute_limit_us != self.request.absolute_limit_us
            || report.clock_contract != self.request.clock_contract
            || report.timer_policy != self.request.timer_policy
            || report.seed != seed
            || report.engine_version != ENGINE_VERSION
            || report.engine_source_sha256 != ENGINE_SOURCE_SHA256
            || report.model_assumptions != self.request.model_assumptions
        {
            return Err(
                "Report identity differs from the declared scenario/engine/assumptions".into(),
            );
        }
        let is_sample = self.rows.is_empty();
        let row = observe::observe(&self.request, report, is_sample)?;
        self.add_observation(row)
    }

    pub fn add_observation(&mut self, row: Observation) -> Result<(), String> {
        let seed = self
            .request
            .seed
            .checked_add(self.rows.len() as u64)
            .ok_or("seed overflow")?;
        if self.rows.len() as u64 >= self.requested || row.sample.seed != seed {
            return Err("observation seed mismatch or limit reached".into());
        }
        if !self.rows.is_empty() && row.quality != self.quality {
            return Err("trial quality differs from the first execution".into());
        }
        let quality = row.quality.clone();
        self.align_observation(row)?;
        self.quality = quality;
        Ok(())
    }

    fn align_observation(&mut self, mut row: Observation) -> Result<(), String> {
        let total = row
            .bins
            .iter()
            .try_fold(0_u128, |sum, value| sum.checked_add(*value))
            .ok_or("rounded observation total overflow")?;
        let field_totals =
            row.field_times
                .iter()
                .try_fold((0_u64, 0_u64), |(relative, absolute), pair| {
                    let (r, a) = (pair.relative?, pair.absolute?);
                    (r <= a).then_some((relative.checked_add(r)?, absolute.checked_add(a)?))
                });
        if row.bucket_us < 1_000_000
            || !row.bucket_us.is_multiple_of(1_000_000)
            || !(row.bucket_us / 1_000_000).is_power_of_two()
            || row.bins.len() > 240
            || row.bins.len() as u64
                != row
                    .sample
                    .absolute_elapsed_us
                    .div_ceil(row.bucket_us)
                    .max(1)
            || row.sample.relative_elapsed_us > row.sample.absolute_elapsed_us
            || row.sample.absolute_elapsed_us > self.request.absolute_limit_us
            || !row.sample.damage.is_finite()
            || row.sample.damage < 0.
            || row.sample.dps.relative.is_some_and(|rate| {
                !rate.is_finite()
                    || row.sample.relative_elapsed_us == 0
                    || rate != row.sample.damage / row.sample.relative_seconds()
            })
            || row.sample.dps.absolute.is_some_and(|rate| {
                !rate.is_finite()
                    || row.sample.absolute_elapsed_us == 0
                    || rate != row.sample.damage / row.sample.absolute_seconds()
            })
            || (row.sample.relative_elapsed_us == 0
                && (row.sample.dps.relative.is_some()
                    || row.sample.eps.relative.is_some()
                    || row.sample.granted_eps.relative.is_some()
                    || row.sample.hps.relative.is_some()
                    || row.sample.shp.relative.is_some()))
            || (row.sample.absolute_elapsed_us == 0
                && (row.sample.dps.absolute.is_some()
                    || row.sample.eps.absolute.is_some()
                    || row.sample.granted_eps.absolute.is_some()
                    || row.sample.hps.absolute.is_some()
                    || row.sample.shp.absolute.is_some()))
            || [
                row.sample.eps.relative,
                row.sample.eps.absolute,
                row.sample.granted_eps.relative,
                row.sample.granted_eps.absolute,
                row.sample.hps.relative,
                row.sample.hps.absolute,
                row.sample.shp.relative,
                row.sample.shp.absolute,
            ]
            .iter()
            .any(|v| v.is_some_and(|rate| !rate.is_finite() || rate < 0.))
            || row.sample.damage != total as f64
            || row.field_times.len() != self.request.actors.len()
            || field_totals
                != Some((
                    row.sample.relative_elapsed_us,
                    row.sample.absolute_elapsed_us,
                ))
        {
            return Err("invalid compact clock observation or absolute bucket grid".into());
        }
        let current_us = (self.bucket_seconds * 1e6) as u64;
        let target_us = row.bucket_us.max(current_us);
        // Checked nonnegative full sums above bound every coarsened/prefix sum.
        fn coarsen(row: &mut Observation, target_us: u64) {
            while row.bucket_us < target_us {
                let count = row.bins.len().div_ceil(2);
                for i in 0..count {
                    row.bins[i] = row.bins[2 * i] + row.bins.get(2 * i + 1).copied().unwrap_or(0);
                }
                row.bins.truncate(count);
                row.bucket_us *= 2;
            }
        }
        if target_us > current_us {
            for previous in &mut self.rows {
                coarsen(previous, target_us);
            }
            self.bins_count = (self.bins_count as u64).div_ceil(target_us / current_us) as usize;
        }
        coarsen(&mut row, target_us);
        self.bucket_seconds = target_us as f64 / 1e6;
        self.bins_count = self.bins_count.max(row.bins.len());
        self.rows.push(row);
        Ok(())
    }

    pub fn samples(&self) -> Vec<Sample> {
        self.rows.iter().map(|row| row.sample.clone()).collect()
    }

    fn stats(&self, get: impl Fn(&Observation) -> f64) -> Statistics {
        summarize(self.rows.iter().map(get).collect())
    }

    fn rate_stats(
        &self,
        damage: impl Fn(&Observation) -> f64,
    ) -> ntesim_contracts::TimeBasisPair<Statistics> {
        let relative = self
            .rows
            .iter()
            .all(|row| row.sample.dps.relative.is_some())
            .then(|| self.stats(|row| damage(row) / row.sample.relative_seconds()));
        let absolute = self
            .rows
            .iter()
            .all(|row| row.sample.dps.absolute.is_some())
            .then(|| self.stats(|row| damage(row) / row.sample.absolute_seconds()));
        ntesim_contracts::TimeBasisPair { relative, absolute }
    }

    pub fn snapshot(&self, elapsed_ms: f64) -> Result<Aggregate, String> {
        if self.rows.is_empty() || !elapsed_ms.is_finite() || elapsed_ms < 0. {
            return Err("snapshot requires completed observations and finite elapsed time".into());
        }
        let actors = self
            .request
            .actors
            .iter()
            .enumerate()
            .map(|(i, actor)| {
                let keys: BTreeSet<_> = self
                    .rows
                    .iter()
                    .flat_map(|row| row.actions[i].keys().cloned())
                    .collect();
                Actor {
                    slot: actor.slot,
                    character_id: actor.character_id.clone(),
                    dps: self.rate_stats(|row| row.actors[i]["damage"]),
                    field: ntesim_contracts::TimeBasisPair {
                        relative: Some(
                            self.stats(|row| row.field_times[i].relative.unwrap() as f64 / 1e6),
                        ),
                        absolute: Some(
                            self.stats(|row| row.field_times[i].absolute.unwrap() as f64 / 1e6),
                        ),
                    },
                    metrics: ["damage", "hits", "energy", "granted", "effective"]
                        .into_iter()
                        .map(|key| (key.into(), self.stats(|row| row.actors[i][key])))
                        .collect(),
                    actions: keys
                        .into_iter()
                        .map(|key| {
                            let stats = self.stats(|row| *row.actions[i].get(&key).unwrap_or(&0.));
                            (key, stats)
                        })
                        .collect(),
                }
            })
            .collect();
        let source_keys: BTreeSet<_> = self
            .rows
            .iter()
            .flat_map(|row| row.sources.keys().cloned())
            .collect();
        let mut sources: Vec<_> = source_keys
            .into_iter()
            .map(|key| Source {
                actor_index: key.0,
                ability_id: key.1.clone(),
                effect_id: key.2.clone(),
                damage: self.stats(|row| row.sources.get(&key).map_or(0., |v| v.0)),
                dps: self.rate_stats(|row| row.sources.get(&key).map_or(0., |v| v.0)),
                hits: self.stats(|row| row.sources.get(&key).map_or(0., |v| v.1)),
            })
            .collect();
        sources.sort_by(|a, b| b.damage.mean.total_cmp(&a.damage.mean));
        let energy_keys: BTreeSet<_> = self
            .rows
            .iter()
            .flat_map(|row| row.energy.keys().cloned())
            .collect();
        let energy_sources = energy_keys
            .into_iter()
            .map(|key| Energy {
                actor_index: key.0,
                source: key.1,
                kind: key.2.clone(),
                energy: self.stats(|row| row.energy.get(&key).map_or(0., |v| v.0)),
                effective: self.stats(|row| row.energy.get(&key).map_or(0., |v| v.1)),
                overflow: self.stats(|row| row.energy.get(&key).map_or(0., |v| v.2)),
            })
            .collect();
        let max_duration = self
            .rows
            .iter()
            .map(|row| row.sample.absolute_seconds())
            .fold(0.0, f64::max);
        let effective_horizon = max_duration;
        let mut cumulative = vec![0_u128; self.rows.len()];
        let timeline = (0..self.bins_count)
            .map(|i| {
                let time = effective_horizon.min((i + 1) as f64 * self.bucket_seconds);
                let width = (time - i as f64 * self.bucket_seconds).max(1e-6);
                let dps = self.stats(|row| row.bins.get(i).copied().unwrap_or(0) as f64 / width);
                let totals = self.rows.iter().zip(&mut cumulative).map(|(row, total)| {
                    *total += row.bins.get(i).copied().unwrap_or(0);
                    *total as f64
                });
                Timeline {
                    time,
                    dps,
                    cumulative: summarize(totals.collect()),
                }
            })
            .collect();
        let dps = ntesim_contracts::TimeBasisPair {
            relative: self
                .rows
                .iter()
                .all(|row| row.sample.dps.relative.is_some())
                .then(|| self.stats(|row| row.sample.dps.relative.unwrap())),
            absolute: self
                .rows
                .iter()
                .all(|row| row.sample.dps.absolute.is_some())
                .then(|| self.stats(|row| row.sample.dps.absolute.unwrap())),
        };
        let eps = ntesim_contracts::TimeBasisPair {
            relative: self
                .rows
                .iter()
                .all(|row| row.sample.eps.relative.is_some())
                .then(|| self.stats(|row| row.sample.eps.relative.unwrap())),
            absolute: self
                .rows
                .iter()
                .all(|row| row.sample.eps.absolute.is_some())
                .then(|| self.stats(|row| row.sample.eps.absolute.unwrap())),
        };
        let granted_eps = ntesim_contracts::TimeBasisPair {
            relative: self
                .rows
                .iter()
                .all(|row| row.sample.granted_eps.relative.is_some())
                .then(|| self.stats(|row| row.sample.granted_eps.relative.unwrap())),
            absolute: self
                .rows
                .iter()
                .all(|row| row.sample.granted_eps.absolute.is_some())
                .then(|| self.stats(|row| row.sample.granted_eps.absolute.unwrap())),
        };
        let hps = ntesim_contracts::TimeBasisPair {
            relative: self
                .rows
                .iter()
                .all(|row| row.sample.hps.relative.is_some())
                .then(|| self.stats(|row| row.sample.hps.relative.unwrap())),
            absolute: self
                .rows
                .iter()
                .all(|row| row.sample.hps.absolute.is_some())
                .then(|| self.stats(|row| row.sample.hps.absolute.unwrap())),
        };
        let shp = ntesim_contracts::TimeBasisPair {
            relative: self
                .rows
                .iter()
                .all(|row| row.sample.shp.relative.is_some())
                .then(|| self.stats(|row| row.sample.shp.relative.unwrap())),
            absolute: self
                .rows
                .iter()
                .all(|row| row.sample.shp.absolute.is_some())
                .then(|| self.stats(|row| row.sample.shp.absolute.unwrap())),
        };
        let metrics = AggregateMetrics {
            dps,
            eps,
            granted_eps,
            hps,
            shp,
            damage: self.stats(|row| row.sample.damage),
            duration: self.stats(|row| row.sample.relative_seconds()),
            absolute_duration: self.stats(|row| row.sample.absolute_seconds()),
            hits: self.stats(|row| row.sample.hits),
            events: self.stats(|row| row.sample.events),
            gross_healing: self.stats(|row| row.support["gross_healing"]),
            healing: self.stats(|row| row.support["healing"]),
            excess_healing: self.stats(|row| row.support["excess_healing"]),
            shield_absorbed: self.stats(|row| row.support["shield_absorbed"]),
        };
        let available = available_reactions(&self.request);
        let mut reactions: Vec<_> = available
            .into_iter()
            .map(|name| {
                let stats = self.stats(|row| row.reactions.get(&name).copied().unwrap_or(0) as f64);
                ReactionAggregate { name, stats }
            })
            .collect();
        reactions.sort_by(|a, b| {
            b.stats
                .mean
                .total_cmp(&a.stats.mean)
                .then_with(|| a.name.cmp(&b.name))
        });
        Ok(Aggregate {
            completed: self.rows.len(),
            requested: self.requested,
            elapsed_ms,
            trials_per_second: (elapsed_ms > 0.)
                .then(|| self.rows.len() as f64 / (elapsed_ms / 1000.)),
            metrics,
            actors,
            sources,
            energy_sources,
            targets: self
                .request
                .targets
                .iter()
                .enumerate()
                .map(|(i, target)| Target {
                    target_index: target.target_index,
                    damage: self.stats(|row| row.targets[i]),
                    dps: self.rate_stats(|row| row.targets[i]),
                })
                .collect(),
            timeline,
            bucket_seconds: self.bucket_seconds,
            quality: self.quality.clone(),
            game_dps_computed: self
                .rows
                .iter()
                .all(|row| row.sample.dps.relative.is_some()),
            reactions,
        })
    }
}

pub fn available_reactions(request: &GameScenarioRequest) -> Vec<String> {
    use ntesim_data::constants::character::Element;
    use std::collections::BTreeSet;

    let elements: BTreeSet<Element> = request
        .actors
        .iter()
        .filter_map(|actor| {
            if actor.character_id == "character:1051" || actor.character_id == "character:1046" {
                return Some(Element::Cosmos);
            }
            ntesim_data::character::definition(&actor.character_id)
                .map(|def| def.element)
                .ok()
                .or_else(|| {
                    ntesim_content::registry::get_character_descriptor_by_id(&actor.character_id)
                        .ok()
                        .and_then(|desc| match desc.element.to_uppercase().as_str() {
                            "CHAOS" => Some(Element::Chaos),
                            "COSMOS" => Some(Element::Cosmos),
                            "INCANTATION" => Some(Element::Incantation),
                            "LAKSHANA" => Some(Element::Lakshana),
                            "NATURE" => Some(Element::Nature),
                            "PSYCHE" => Some(Element::Psyche),
                            _ => None,
                        })
                })
        })
        .collect();

    let mut available = Vec::new();
    if elements.contains(&Element::Cosmos) && elements.contains(&Element::Nature) {
        available.push("Blossom".to_string());
    }
    if elements.contains(&Element::Nature) && elements.contains(&Element::Incantation) {
        available.push("Hexed".to_string());
    }
    if elements.contains(&Element::Incantation) && elements.contains(&Element::Chaos) {
        available.push("Scorch".to_string());
    }
    if elements.contains(&Element::Chaos) && elements.contains(&Element::Psyche) {
        available.push("Nova".to_string());
    }
    if elements.contains(&Element::Psyche) && elements.contains(&Element::Lakshana) {
        available.push("Stain".to_string());
    }
    if elements.contains(&Element::Lakshana) && elements.contains(&Element::Cosmos) {
        available.push("Remora".to_string());
    }
    if elements.contains(&Element::Lakshana)
        && elements.contains(&Element::Cosmos)
        && elements.contains(&Element::Nature)
    {
        available.push("Charge".to_string());
    }
    if elements.contains(&Element::Incantation)
        && elements.contains(&Element::Chaos)
        && elements.contains(&Element::Psyche)
    {
        available.push("Discord".to_string());
    }
    available
}

#[cfg(test)]
mod tests;
