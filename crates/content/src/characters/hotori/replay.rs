//! Recording of allied Support/Redirect actions and their off-field playback.
use crate::HypothesisId;
use crate::characters::RuntimeState;
use crate::characters::hotori::timepiece::Record;
use crate::runtime_api::{CallbackContext, CastContext};
use crate::simulation::admission::{AdmissionOrigin, team_hp_snapshot};
use crate::simulation::events::{GameEventData, ScheduledGameEvent};
use crate::simulation::metadata::InputActionRole;
use crate::simulation::state::GameSimulatorState;
use crate::simulation::{GameEventReceipt, GameScenarioRequest};
use ntesim_core::execution::CausalPriority;

pub(crate) const MODEL: HypothesisId = HypothesisId::HotoriRecordedReplayV1;

/// The recording event opens 0.16747499s after Present Replay commits; the
/// rewind deadline is Skill+5.16747499, so the window is exactly five seconds.
pub(crate) const RECORD_WINDOW_US: u64 = 5_000_000;

pub(crate) const MAX_RECORDS: usize = 3;

/// Virtual actions retain ordinary source effects, while their finite montage
/// state and admission controls are independent of the real owner's action.
pub(crate) struct ReplayControl {
    clock: Option<crate::characters::replay_clock::ReplayClock>,
    locked_until_us: u64,
    switch_action_locked_until_us: u64,
    last_action_start_us: u64,
    cooldowns: ntesim_core::kernel::timing::ProjectedDeadlines<String>,
    chain: Option<(usize, Option<u64>)>,
    owner_gates: Option<(u64, u64, u64)>,
}
impl ReplayControl {
    fn suspend(actor: &mut crate::simulation::state::ActorRuntime) -> Self {
        Self {
            clock: crate::characters::replay_clock::ReplayClock::take(&mut actor.character),
            locked_until_us: actor.locked_until_us,
            switch_action_locked_until_us: actor.switch_action_locked_until_us,
            last_action_start_us: actor.last_action_start_us,
            cooldowns: actor.ability_cooldowns.clone(),
            chain: actor
                .character
                .normal_chain_fields()
                .map(|(stage, start)| (*stage, *start)),
            owner_gates: match &actor.character {
                RuntimeState::Blackbird(s) => Some((s.qte_switch_until_us, 0, 0)),
                RuntimeState::Chaos(s) => Some((
                    s.burst_switch_until_us,
                    s.burst_immunity_until_us,
                    s.verdict_immunity_until_us,
                )),
                _ => None,
            },
        }
    }
    fn restore(
        self,
        actor: &mut crate::simulation::state::ActorRuntime,
    ) -> Option<crate::characters::replay_clock::ReplayClock> {
        let replay = crate::characters::replay_clock::ReplayClock::take(&mut actor.character);
        if let Some(clock) = self.clock {
            clock.restore(&mut actor.character);
        }
        actor.locked_until_us = self.locked_until_us;
        actor.switch_action_locked_until_us = self.switch_action_locked_until_us;
        actor.last_action_start_us = self.last_action_start_us;
        actor.ability_cooldowns = self.cooldowns;
        match (&mut actor.character, self.owner_gates) {
            (RuntimeState::Blackbird(s), Some((gate, _, _))) => s.qte_switch_until_us = gate,
            (RuntimeState::Chaos(s), Some((gate, burst, verdict))) => {
                s.burst_switch_until_us = gate;
                s.burst_immunity_until_us = burst;
                s.verdict_immunity_until_us = verdict;
            }
            _ => {}
        }
        if let Some((stage, start)) = self.chain
            && let Some((target_stage, target_start)) = actor.character.normal_chain_fields()
        {
            *target_stage = stage;
            *target_start = start;
        }
        replay
    }
}

pub(crate) fn begin_callback(
    state: &mut GameSimulatorState,
    context: ntesim_core::kernel::timing::PauseOwner,
) -> ReplayControl {
    let saved = ReplayControl::suspend(&mut state.actors[context.owner]);
    if let Some(clock) = state.replay_clocks.remove(&context) {
        clock.restore(&mut state.actors[context.owner].character);
    }
    saved
}
pub(crate) fn finish_callback(
    state: &mut GameSimulatorState,
    queue: &crate::simulation::timer_queue::EventQueue,
    context: ntesim_core::kernel::timing::PauseOwner,
    control: ReplayControl,
) {
    if let Some(clock) = control.restore(&mut state.actors[context.owner])
        && queue.has_replay(context)
    {
        state.replay_clocks.insert(context, clock);
    }
}

fn qualifies(origin: AdmissionOrigin, role: Option<InputActionRole>) -> bool {
    match origin {
        AdmissionOrigin::SupportDispatch => role == Some(InputActionRole::Qte),
        AdmissionOrigin::Authored => matches!(
            role,
            Some(InputActionRole::RedirectSkill | InputActionRole::Qte)
        ),
    }
}

pub(crate) fn observe_admission(
    request: &GameScenarioRequest,
    state: &mut GameSimulatorState,
    trace: &mut Vec<GameEventReceipt>,
    origin: AdmissionOrigin,
    action: crate::registry::RecordedAdmission<'_>,
) -> Result<(), String> {
    if !qualifies(origin, action.metadata.action.input_role) {
        return Ok(());
    }
    let now_us = state.current_time_us;
    if state.actors[action.owner].hp == 0 {
        return Ok(());
    }
    for hotori in 0..request.actors.len() {
        if request.actors[hotori].character_id != "character:1052" || hotori == action.owner {
            continue;
        }
        let slot = {
            let RuntimeState::Hotori(state) = &mut state.actors[hotori].character else {
                continue;
            };
            let timepiece = &mut state.timepiece;
            let Some(window_end) = timepiece.rewind_until_us else {
                continue;
            };
            let Some(window_start) = window_end.checked_sub(RECORD_WINDOW_US) else {
                continue;
            };
            if now_us < window_start || now_us >= window_end {
                continue;
            }
            if timepiece
                .records
                .iter()
                .any(|record| record.owner == action.owner)
            {
                continue;
            }
            if timepiece.records.len() >= MAX_RECORDS {
                continue;
            }
            let position = timepiece
                .records
                .partition_point(|record| (record.time_us, record.owner) <= (now_us, action.owner));
            timepiece.records.insert(
                position,
                Record {
                    owner: action.owner,
                    command_index: action.command_index,
                    time_us: now_us,
                    ability_id: action.ability_id.into(),
                    array_index: action.array_index,
                    damage_effect_id: action.damage_effect_id.into(),
                    atk_rate: action.atk_rate,
                    scaling_stat: action.scaling_stat,
                    hit_delay_us: action.hit_delay_us,
                    metadata: action.metadata.clone(),
                },
            );
            position
        };
        crate::runtime_api::record_hypothesis_id(
            request,
            &mut state.execution_model_assumptions,
            MODEL,
        )?;
        trace.push(GameEventReceipt {
            time_us: now_us,
            phase: "ActionAdmission".into(),
            event_kind: "hotori_record_slot".into(),
            active_actor: state.active_actor_index,
            hit_index: None,
            from_actor: Some(action.owner),
            to_actor: Some(hotori),
            energy_change: None,
            energy_fanout: None,
            data: Some(crate::simulation::GameEventPayload {
                slot: Some(slot),
                ability_id: Some(action.ability_id.into()),
                damage_effect_id: Some(action.damage_effect_id.into()),
                ..Default::default()
            }),
        });
    }
    Ok(())
}

pub(crate) fn playback(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    generation: u64,
) -> Result<(), String> {
    if ctx
        .request
        .actors
        .get(owner)
        .map(|config| config.character_id.as_str())
        != Some("character:1052")
    {
        return Ok(());
    }
    let now_us = ctx.state.current_time_us;
    let records = {
        let RuntimeState::Hotori(state) = &mut ctx.state.actors[owner].character else {
            return Ok(());
        };
        let timepiece = &mut state.timepiece;
        let Some(mode) = timepiece.mode.as_ref() else {
            return Ok(());
        };
        if timepiece.mode_generation != generation || now_us < mode.entry_us {
            return Ok(());
        }
        std::mem::take(&mut timepiece.records)
    };
    if records.is_empty() {
        return Ok(());
    }
    crate::runtime_api::record_hypothesis_id(
        ctx.request,
        &mut ctx.state.execution_model_assumptions,
        MODEL,
    )?;
    crate::characters::hotori::timepiece::record_trace(
        ctx,
        "ModePause",
        "hotori_replay_clear",
        Some(owner),
        Some(owner),
        Some(crate::simulation::GameEventPayload {
            count: Some(records.len()),
            ..Default::default()
        }),
    )?;
    for record in &records {
        if ctx.state.actors[record.owner].hp == 0 {
            crate::characters::hotori::timepiece::record_trace(
                ctx,
                "ModePause",
                "hotori_replay_skipped",
                Some(owner),
                Some(record.owner),
                Some(crate::simulation::GameEventPayload {
                    reason: Some("owner_defeated".into()),
                    ..Default::default()
                }),
            )?;
            continue;
        }
        dispatch(ctx, owner, record)?;
    }
    Ok(())
}

fn dispatch(ctx: &mut CallbackContext<'_>, hotori: usize, record: &Record) -> Result<(), String> {
    ctx.queue
        .begin_replay(ntesim_core::kernel::timing::PauseOwner {
            owner: record.owner,
            generation: record.command_index as u64,
        })?;
    let result = dispatch_finite(ctx, hotori, record);
    ctx.queue.end_replay();
    result
}

fn dispatch_finite(
    ctx: &mut CallbackContext<'_>,
    hotori: usize,
    record: &Record,
) -> Result<(), String> {
    let now_us = ctx.state.current_time_us;
    let owner = record.owner;
    let hit_time_us = now_us
        .checked_add(record.hit_delay_us)
        .ok_or("replayed hit delivery deadline overflow")?;
    let mut team_hp_buf = [(0u64, 0u64); 4];
    let team_hp_len = team_hp_snapshot(ctx.state, &mut team_hp_buf)?;
    let team_hp_snapshot = &team_hp_buf[..team_hp_len];
    let mut metadata = record.metadata.clone();
    metadata.binding.energy_group_id = None;
    crate::runtime_api::increment_sequence(ctx.sequence)?;
    ctx.queue.push(ScheduledGameEvent {
        time_us: hit_time_us,
        priority: CausalPriority::HitDelivery,
        seq: *ctx.sequence,
        data: GameEventData::HitDelivery {
            command_index: record.command_index,
            actor_index: owner,
            target_index: ctx.request.aim_target(record.command_index),
            ability_id: record.ability_id.clone(),
            damage_effect_id: record.damage_effect_id.clone(),
            atk_rate: record.atk_rate,
            scaling_stat: record.scaling_stat,
            metadata: Box::new(metadata),
        },
    });
    let control = ReplayControl::suspend(&mut ctx.state.actors[owner]);
    let mut assumptions = std::collections::BTreeMap::new();
    let cast_result = (|| -> Result<(), String> {
        let state = &mut *ctx.state;
        let mut cast = CastContext {
            origins: &mut state.origins,
            command_index: record.command_index,
            native_array_index: Some(record.array_index),
            execution_assumptions: &mut assumptions,
            config: &ctx.request.actors[owner],
            request: ctx.request,
            actor_index: owner,
            active_actor_index: state.active_actor_index,
            target_index: ctx.request.aim_target(record.command_index),
            ability_id: &record.ability_id,
            action_metadata: &record.metadata.action,
            packet_metadata: Some(&record.metadata.packet),
            hit_time_us,
            world_time_us: state.mode_clock.relative_us(),
            time_us: now_us,
            buffs: &mut state.buffs,
            actor: &mut state.actors[owner],
            stamina: state.combat_stamina.as_mut(),
            team_hp_snapshot,
            targets: &state.targets,
            queue: ctx.queue,
            sequence: ctx.sequence,
        };
        crate::gear::on_cast(&mut cast)?;
        crate::registry::on_cast(&mut cast)?;
        Ok(())
    })();
    let context = ntesim_core::kernel::timing::PauseOwner {
        owner,
        generation: record.command_index as u64,
    };
    finish_callback(ctx.state, ctx.queue, context, control);
    cast_result?;
    ctx.state.execution_model_assumptions.extend(assumptions);
    crate::characters::hotori::timepiece::record_trace(
        ctx,
        "ModePause",
        "hotori_replay_dispatch",
        Some(owner),
        Some(hotori),
        Some(crate::simulation::GameEventPayload {
            ability_id: Some(record.ability_id.clone()),
            damage_effect_id: Some(record.damage_effect_id.clone()),
            completes_at_us: Some(hit_time_us),
            ..Default::default()
        }),
    )
}
