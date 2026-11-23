use super::model::{Observation, Sample, Values};
use ntesim_content::simulation::{CompletedSourceTrial, GameScenarioReport, GameScenarioRequest};
use std::collections::BTreeMap;
#[cfg(test)]
use {ntesim_content::simulation::GameActorShieldSummary, std::collections::BTreeSet};

pub fn observe_batch(trial: CompletedSourceTrial) -> Observation {
    let mut actors = Vec::with_capacity(trial.final_energy.len());
    let mut granted = 0.;
    let mut retained = 0.;
    for (slot, (energy, (damage, hits))) in trial
        .final_energy
        .into_iter()
        .zip(trial.damage_by_actor)
        .enumerate()
    {
        let (gross, effective) = trial
            .energy
            .iter()
            .filter(|((actor, _, _), _)| *actor == slot)
            .fold((0., 0.), |totals, (_, energy)| {
                (totals.0 + energy.0, totals.1 + energy.1)
            });
        granted += gross;
        retained += effective;
        actors.push(values([
            ("damage", damage as f64),
            ("hits", hits as f64),
            ("energy", energy as f64),
            ("granted", gross),
            ("effective", effective),
        ]));
    }
    let rates = |total: f64| ntesim_contracts::TimeBasisPair {
        relative: (trial.relative_elapsed_us > 0)
            .then(|| total / (trial.relative_elapsed_us as f64 / 1e6)),
        absolute: Some(total / (trial.absolute_elapsed_us as f64 / 1e6)),
    };
    let [gross, applied, excess] = trial.healing;
    Observation {
        quality: trial.quality,
        bucket_us: trial.bucket_us,
        sample: Sample {
            seed: trial.seed,
            damage: trial.total_damage as f64,
            dps: rates(trial.total_damage as f64),
            eps: rates(retained),
            granted_eps: rates(granted),
            hps: rates(gross),
            shp: trial.available_shield_hp,
            relative_elapsed_us: trial.relative_elapsed_us,
            absolute_elapsed_us: trial.absolute_elapsed_us,
            hits: trial.hits as f64,
            events: trial.events as f64,
        },
        support: values([
            ("gross_healing", gross),
            ("healing", applied),
            ("excess_healing", excess),
            ("shield_absorbed", trial.absorbed),
        ]),
        actors,
        actions: trial
            .actions
            .into_iter()
            .map(|counts| values(counts.into_iter().map(|(key, count)| (key, count as f64))))
            .collect(),
        sources: trial
            .damage_by_source
            .into_iter()
            .map(|(key, (damage, hits))| (key, (damage as f64, hits as f64)))
            .collect(),
        energy: trial.energy,
        targets: trial
            .damage_by_target
            .into_iter()
            .map(|damage| damage as f64)
            .collect(),
        bins: trial.bins,
        reactions: trial.reaction_counts,
        field_times: trial.field_times,
    }
}

fn values(values: impl IntoIterator<Item = (&'static str, f64)>) -> Values {
    values
        .into_iter()
        .map(|(key, value)| (key.into(), value))
        .collect()
}

pub fn observe(
    request: &GameScenarioRequest,
    report: &GameScenarioReport,
    is_sample: bool,
) -> Result<Observation, String> {
    super::super::eligibility::validate_completed(report)?;
    report.validate_clocks()?;
    if is_sample {
        report.validate_state_timeline()?;
        report.damage_analysis()?;
    }
    if report.absolute_elapsed_us == 0
        || report.final_actor_states.len() != request.actors.len()
        || report.final_target_states.len() != request.targets.len()
        || report
            .final_target_states
            .iter()
            .enumerate()
            .any(|(i, target)| target.target_index != i)
    {
        return Err("missing actor states or positive execution duration".into());
    }
    let duration = report.absolute_elapsed_us as f64 / 1e6;
    let relative_duration = report.relative_elapsed_us as f64 / 1e6;
    let mut bucket_us = 1_000_000_u64;
    while report.absolute_elapsed_us.div_ceil(bucket_us) > 240 {
        bucket_us = bucket_us
            .checked_mul(2)
            .ok_or("absolute timeline bucket overflow")?;
    }
    let bins_count = report.absolute_elapsed_us.div_ceil(bucket_us).max(1) as usize;
    let mut row = Observation {
        quality: report.quality.clone(),
        bucket_us,
        sample: Sample {
            seed: report.seed,
            damage: report.total_damage_dealt,
            dps: ntesim_contracts::TimeBasisPair {
                relative: (report.game_dps_computed && report.relative_elapsed_us > 0)
                    .then(|| report.total_damage_dealt / relative_duration),
                absolute: (report.total_damage_dealt.is_finite()
                    && report.absolute_elapsed_us > 0
                    && (report.game_dps_computed
                        || report.dps_unavailable_reason
                            == Some(ntesim_content::simulation::GameDpsUnavailableReason::ZeroRelativeDuration)))
                .then(|| report.total_damage_dealt / duration),
            },
            eps: ntesim_contracts::TimeBasisPair::default(),
            granted_eps: ntesim_contracts::TimeBasisPair::default(),
            hps: ntesim_contracts::TimeBasisPair::default(),
            shp: report.available_shield_hp,
            relative_elapsed_us: report.relative_elapsed_us,
            absolute_elapsed_us: report.absolute_elapsed_us,
            hits: report.hits_settled as f64,
            events: report.total_events_processed as f64,
        },
        support: Values::new(),
        actors: vec![Values::new(); request.actors.len()],
        actions: vec![Values::new(); request.actors.len()],
        sources: BTreeMap::new(),
        energy: BTreeMap::new(),
        targets: vec![0.; request.targets.len()],
        bins: vec![0; bins_count],
        reactions: report.reaction_counts.clone(),
        field_times: report.field_times.clone(),
    };
    for actor in &mut row.actors {
        for key in ["damage", "hits", "energy", "granted", "effective"] {
            actor.insert(key.into(), 0.);
        }
    }
    accumulate_hits(&report.hit_receipts, &mut row)?;
    let mut energy_acc: ntesim_core::FastHashMap<
        (usize, ntesim_content::simulation::GameEnergySource, &str),
        (f64, f64, f64),
    > = ntesim_core::FastHashMap::default();
    for event in &report.trace {
        for grant in event.energy_grants() {
            let key = (
                grant.recipient_actor_index,
                grant.source,
                event.event_kind.as_str(),
            );
            let energy = energy_acc.entry(key).or_default();
            energy.0 += grant.gross as f64;
            energy.1 += grant.retained as f64;
            energy.2 += grant.overflow as f64;
        }
        if let Some(spend) = event.energy_spend()
            && event.event_kind == "invoke_ability"
        {
            let occurrence = report
                .action_metadata_ledger
                .iter()
                .find(|entry| entry.command_indices.contains(&spend.command_index));
            let role = occurrence
                .and_then(|entry| report.action_metadata(entry))
                .and_then(|metadata| metadata.action.primary_slot.as_ref());
            let key = match role {
                Some(ntesim_content::simulation::metadata::PrimaryActionSlot::Normal) => "attack",
                Some(ntesim_content::simulation::metadata::PrimaryActionSlot::Skill) => "skill",
                Some(ntesim_content::simulation::metadata::PrimaryActionSlot::Ultimate) => "burst",
                None => "invoke",
            };
            // Player controls have their own receipts, not a second cast count.
            if !report
                .player_input_receipts
                .iter()
                .any(|input| input.command_index == spend.command_index)
            {
                *row.actions
                    .get_mut(spend.recipient_actor_index)
                    .ok_or("invalid admitted actor")?
                    .entry(key.into())
                    .or_default() += 1.;
            }
        }
    }
    for ((recipient, source, kind), values) in energy_acc {
        row.energy
            .insert((recipient, source, kind.to_string()), values);
    }
    for input in &report.player_input_receipts {
        let key = match input.input {
            ntesim_content::simulation::PlayerInput::Special => "special",
            ntesim_content::simulation::PlayerInput::Dash => "dash",
            ntesim_content::simulation::PlayerInput::Arc => "arc",
            ntesim_content::simulation::PlayerInput::Jump => "jump",
        };
        *row.actions
            .get_mut(input.actor_index)
            .ok_or("invalid player input actor")?
            .entry(key.into())
            .or_default() += 1.;
    }
    settle_energy(&mut row, request, report, relative_duration, duration)?;
    // Autonomous healing has a second causal receipt, but settles through this canonical ledger.
    let (gross, applied, excess) =
        report
            .healing_receipts
            .iter()
            .fold((0., 0., 0.), |totals, healing| {
                (
                    totals.0 + healing.gross as f64,
                    totals.1 + healing.applied as f64,
                    totals.2 + healing.excess as f64,
                )
            });
    let absorbed = shield_absorption(&report.incoming_damage_receipts);
    row.support = [
        ("gross_healing", gross),
        ("healing", applied),
        ("excess_healing", excess),
        ("shield_absorbed", absorbed),
    ]
    .into_iter()
    .map(|(key, value)| (key.into(), value))
    .collect();
    row.sample.hps = ntesim_contracts::TimeBasisPair {
        relative: (report.relative_elapsed_us > 0).then(|| gross / relative_duration),
        absolute: (report.absolute_elapsed_us > 0).then(|| gross / duration),
    };
    if report
        .available_shield_hp
        .relative
        .is_some_and(|shp| !shp.is_finite() || shp < 0.)
    {
        return Err("relative shield HP must be finite and nonnegative".into());
    }
    if report
        .available_shield_hp
        .absolute
        .is_some_and(|shp| !shp.is_finite() || shp < 0.)
    {
        return Err("absolute shield HP must be finite and nonnegative".into());
    }
    if report.relative_elapsed_us > 0 && report.available_shield_hp.relative.is_none() {
        return Err("missing relative shield HP for positive relative duration".into());
    }
    if report.absolute_elapsed_us > 0 && report.available_shield_hp.absolute.is_none() {
        return Err("missing absolute shield HP for positive absolute duration".into());
    }
    row.sample.shp = report.available_shield_hp;
    Ok(row)
}

fn accumulate_hits(
    hit_receipts: &[ntesim_content::simulation::GameHitReceipt],
    row: &mut Observation,
) -> Result<(), String> {
    let mut sources: ntesim_core::FastHashMap<(usize, &str, &str), (u128, u128)> =
        ntesim_core::FastHashMap::default();
    let mut actors = vec![(0_u128, 0_u128); row.actors.len()];
    let mut targets = vec![0_u128; row.targets.len()];
    let mut bins = vec![0_u128; row.bins.len()];
    fn add(total: &mut u128, amount: u128) -> Result<(), String> {
        *total = total
            .checked_add(amount)
            .ok_or("rounded observation overflow")?;
        Ok(())
    }
    for hit in hit_receipts {
        let amount = u128::from(hit.rounded_damage);
        let actor = actors
            .get_mut(hit.actor_index)
            .ok_or("invalid damage actor")?;
        add(&mut actor.0, amount)?;
        add(&mut actor.1, 1)?;
        add(
            targets
                .get_mut(hit.target_index)
                .ok_or("invalid damage target")?,
            amount,
        )?;
        let source = sources
            .entry((
                hit.actor_index,
                hit.ability_id.as_str(),
                hit.damage_effect_id.as_str(),
            ))
            .or_default();
        add(&mut source.0, amount)?;
        add(&mut source.1, 1)?;
        let bin = (hit.time_us / row.bucket_us).min(bins.len() as u64 - 1) as usize;
        add(&mut bins[bin], amount)?;
    }
    for (values, (damage, hits)) in row.actors.iter_mut().zip(actors) {
        values.insert("damage".into(), damage as f64);
        values.insert("hits".into(), hits as f64);
    }
    row.targets = targets.into_iter().map(|v| v as f64).collect();
    row.bins = bins;
    row.sources = sources
        .into_iter()
        .map(|((actor, ability, effect), (damage, hits))| {
            (
                (actor, ability.to_owned(), effect.to_owned()),
                (damage as f64, hits as f64),
            )
        })
        .collect();
    Ok(())
}

#[cfg(test)]
pub(super) fn calculate_available_shield_hp(
    request: &GameScenarioRequest,
    report: &GameScenarioReport,
) -> Result<ntesim_contracts::TimeBasisPair<f64>, String> {
    if report.absolute_elapsed_us == 0 {
        return Err("elapsed duration must be positive".into());
    }
    let initial_frame = report
        .state_timeline
        .first()
        .ok_or("missing initial state frame")?;
    if initial_frame.actors.len() != request.actors.len() {
        return Err("initial actor count differs from request".into());
    }
    let mut current_actors = initial_frame
        .actors
        .iter()
        .map(|a| a.into_summary())
        .collect::<Result<Vec<_>, _>>()?;
    for (i, actor) in current_actors.iter().enumerate() {
        if actor.slot != i {
            return Err("actor slot index mismatch in initial frame".into());
        }
        validate_shields(&actor.active_shields)?;
    }
    let mut active_actor = initial_frame.active_actor_index;
    if active_actor >= current_actors.len() {
        return Err("initial active actor index out of bounds".into());
    }
    let mut previous_time_us = initial_frame.time_us;
    let mut total_hp_us: u128 = 0;
    let mut total_relative_hp_us: u128 = 0;

    let pauses: Vec<[u64; 2]> = report
        .timer_pauses
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .map(|p| [p.start_absolute_us, p.end_absolute_us])
        .collect();
    let merged_pauses = ntesim_core::kernel::timing::merge_intervals(pauses);

    for frame in &report.state_timeline[1..] {
        if frame.active_actor_index >= current_actors.len() {
            return Err("active actor index out of bounds in timeline frame".into());
        }
        if frame.time_us < previous_time_us || frame.time_us > report.absolute_elapsed_us {
            return Err("timeline frame time out of bounds".into());
        }
        if frame.time_us > previous_time_us {
            if current_actors[active_actor].hp > 0 {
                for shield in &current_actors[active_actor].active_shields {
                    let active_end = frame
                        .time_us
                        .min(shield.expires_at_us)
                        .min(report.absolute_elapsed_us);
                    if active_end > previous_time_us {
                        let dt = u128::from(active_end - previous_time_us);
                        let hp = u128::from(shield.remaining_hp);
                        let term = hp.checked_mul(dt).ok_or("shield HP*us product overflow")?;
                        total_hp_us = total_hp_us
                            .checked_add(term)
                            .ok_or("shield HP*us sum overflow")?;

                        let mut paused_us = 0_u64;
                        for &[p_start, p_end] in &merged_pauses {
                            let overlap_start = previous_time_us.max(p_start);
                            let overlap_end = active_end.min(p_end);
                            if overlap_end > overlap_start {
                                paused_us = paused_us
                                    .checked_add(overlap_end - overlap_start)
                                    .ok_or("paused overlap overflow")?;
                            }
                        }
                        let relative_dt =
                            u128::from((active_end - previous_time_us).saturating_sub(paused_us));
                        let rel_term = hp
                            .checked_mul(relative_dt)
                            .ok_or("relative shield HP*us product overflow")?;
                        total_relative_hp_us = total_relative_hp_us
                            .checked_add(rel_term)
                            .ok_or("relative shield HP*us sum overflow")?;
                    }
                }
            }
            previous_time_us = frame.time_us;
        }
        active_actor = frame.active_actor_index;
        let mut seen_slots = BTreeSet::new();
        for delta in &frame.actors {
            if delta.slot >= current_actors.len() {
                return Err("actor slot out of bounds in timeline frame".into());
            }
            if !seen_slots.insert(delta.slot) {
                return Err("duplicate actor slot in timeline frame".into());
            }
            delta.apply_to(&mut current_actors[delta.slot])?;
            validate_shields(&current_actors[delta.slot].active_shields)?;
        }
    }

    if previous_time_us < report.absolute_elapsed_us {
        return Err("timeline does not reach elapsed duration".into());
    }

    Ok(ntesim_contracts::TimeBasisPair {
        relative: if report.relative_elapsed_us > 0 {
            Some(total_relative_hp_us as f64 / report.relative_elapsed_us as f64)
        } else {
            None
        },
        absolute: Some(total_hp_us as f64 / report.absolute_elapsed_us as f64),
    })
}

#[cfg(test)]
fn validate_shields(shields: &[GameActorShieldSummary]) -> Result<(), String> {
    let mut seen = BTreeSet::new();
    for shield in shields {
        if !seen.insert(&shield.id) {
            return Err(format!("duplicate shield id {}", shield.id));
        }
        if shield.remaining_hp > shield.max_hp {
            return Err(format!(
                "shield {} remaining HP {} exceeds max HP {}",
                shield.id, shield.remaining_hp, shield.max_hp
            ));
        }
    }
    Ok(())
}

pub(super) fn shield_absorption(
    deliveries: &[ntesim_content::simulation::GameActorDamageReceipt],
) -> f64 {
    deliveries.iter().fold(0., |total, delivery| {
        total
            + delivery
                .absorbed_by_shields
                .iter()
                .map(|shield| shield.amount as f64)
                .sum::<f64>()
            + shield_absorption(&delivery.redirected)
    })
}

fn settle_energy(
    row: &mut Observation,
    request: &GameScenarioRequest,
    report: &GameScenarioReport,
    relative_duration: f64,
    absolute_duration: f64,
) -> Result<(), String> {
    let mut total_granted = 0.;
    let mut total_retained = 0.;
    for (slot, state) in report.final_actor_states.iter().enumerate() {
        if state.slot != slot || state.character_id != request.actors[slot].character_id {
            return Err("actor state identity differs from request".into());
        }
        let retained = state
            .granted_energy
            .checked_sub(state.overflow_energy)
            .ok_or("energy overflow exceeds granted energy")?;
        if state.initial_energy != request.actors[slot].initial_energy
            || state.energy > state.energy_capacity
            || u128::from(state.initial_energy) + u128::from(state.granted_energy)
                != u128::from(state.spent_energy)
                    + u128::from(state.overflow_energy)
                    + u128::from(state.energy)
        {
            return Err("actor energy ledger does not conserve energy".into());
        }
        let sums = row
            .energy
            .iter()
            .filter(|((actor, _, _), _)| *actor == slot)
            .fold((0., 0., 0.), |s, (_, e)| (s.0 + e.0, s.1 + e.1, s.2 + e.2));
        if sums.0 != state.granted_energy as f64
            || sums.2 != state.overflow_energy as f64
            || sums.1 != retained as f64
        {
            return Err(
                "Energy source trace does not reconcile with gross/retained/overflow ledger."
                    .into(),
            );
        }
        row.actors[slot].insert("energy".into(), state.energy as f64);
        row.actors[slot].insert("granted".into(), sums.0);
        row.actors[slot].insert("effective".into(), sums.1);
        total_granted += sums.0;
        total_retained += sums.1;
    }
    row.sample.granted_eps = ntesim_contracts::TimeBasisPair {
        relative: (report.relative_elapsed_us > 0).then(|| total_granted / relative_duration),
        absolute: (report.absolute_elapsed_us > 0).then(|| total_granted / absolute_duration),
    };
    row.sample.eps = ntesim_contracts::TimeBasisPair {
        relative: (report.relative_elapsed_us > 0).then(|| total_retained / relative_duration),
        absolute: (report.absolute_elapsed_us > 0).then(|| total_retained / absolute_duration),
    };
    Ok(())
}
