//! Held Normal Branch entry, its launchable landing and the AirAttack fallback.
use super::clock::{Kind, Plan};
use crate::runtime_api::{CallbackContext, CastContext};
use crate::simulation::{
    ExactFraction, GameActionCommand, GameScenarioRequest,
    events::{GameEventData, ScheduledGameEvent},
};
pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::HathorBranchV1;
pub(crate) const PACKET: &str = "GE_Player_Hathor_Branch_Damage";
pub(crate) fn source_assumptions() -> std::collections::BTreeMap<String, String> {
    [MODEL, super::air::MODEL]
        .into_iter()
        .map(|id| (id.into(), id.description().into()))
        .collect()
}
pub(super) const GATES: [u64; 5] = [1_300_000, 500_000, 500_000, 900_000, 600_000];
pub(super) const ENTRY_GATE: u64 = 500_000;
pub(super) const FALLBACK: u64 = 520_000;
const AIR_CONTACT: u64 = 116_417;
pub(super) const RECOVERY: u64 = 737_229;
pub(super) const JUMP: u64 = 549_863;
pub(super) const SWITCH: u64 = 753_203;
pub(super) const AIR_RECOVERY: u64 = 584_348;
pub(super) const AIR_JUMP: u64 = 484_688;
pub(super) const AIR_SWITCH: u64 = 584_348;
#[derive(Clone, Debug)]
pub(crate) struct State {
    pub(super) command: usize,
    pub(super) launchable: bool,
    pub(super) height_bonus: crate::formula::Ratio,
    /// Authored real landing from Branch admission; never re-derived from height.
    pub(super) landing: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Boundary {
    Land,
    Fallback,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Event {
    pub(super) owner: usize,
    pub(super) command: usize,
    pub(super) boundary: Boundary,
}
pub(crate) fn accepted(req: &GameScenarioRequest) -> bool {
    req.model_assumptions
        .get(MODEL.as_str())
        .map(String::as_str)
        == Some(MODEL.description())
}
/// Entry Branch holds bind to the owner's automatic attack through the switch
/// command index rather than an InvokeAbility.
pub(crate) fn entry_hold(req: &GameScenarioRequest, index: usize) -> bool {
    matches!(
        req.actions.get(index),
        Some(GameActionCommand::SwitchActor {
            target_actor_index,
            ..
        }) if req.actors[*target_actor_index].character_id == "character:1025"
    ) && accepted(req)
        && req.branch_inputs.contains_key(&index)
        && req.hold_inputs.contains_key(&index)
}
pub(crate) fn validate(req: &GameScenarioRequest) -> Result<(), String> {
    if !req.branch_inputs.is_empty() && !accepted(req) {
        return Err("Hathor Branch inputs require branch_v1".into());
    }
    if !accepted(req) {
        return Ok(());
    }
    if !super::clock::accepted(req) || !super::air::accepted(req) {
        return Err("Hathor branch_v1 requires ordinary_clock_v1 and airattack_v1".into());
    }
    for (&index, input) in &req.branch_inputs {
        let timestamp_us = match req.actions.get(index) {
            Some(GameActionCommand::InvokeAbility {
                actor_index,
                ability_id,
                timestamp_us,
                ..
            }) if req.actors[*actor_index].character_id == "character:1025"
                && ability_id == "GA_Hathor_Melee" =>
            {
                *timestamp_us
            }
            Some(GameActionCommand::SwitchActor {
                target_actor_index: actor_index,
                timestamp_us,
                ..
            }) if req.actors[*actor_index].character_id == "character:1025" => *timestamp_us,
            _ => {
                return Err(format!(
                    "branch_inputs index {index} must identify a Hathor held Normal or its entry switch"
                ));
            }
        };
        let Some(hold) = req.hold_inputs.get(&index) else {
            return Err(format!(
                "branch_inputs index {index} requires the authored hold"
            ));
        };
        if hold.moving || hold.reserved_stocks.is_some() {
            return Err(format!(
                "branch_inputs index {index} requires a canonical stationary unreserved hold"
            ));
        }
        if hold.duration_us <= ENTRY_GATE {
            return Err(format!(
                "branch_inputs index {index} requires a hold strictly beyond its0.5s source gate"
            ));
        }
        if req.plunge_inputs.contains_key(&index) {
            return Err(format!(
                "branch_inputs index {index} conflicts with a declared plunge"
            ));
        }
        if input.landing_delay_us == 0
            || input.height_bonus.denominator == 0
            || input.height_bonus.numerator > input.height_bonus.denominator
        {
            return Err(format!(
                "branch_inputs index {index} requires positive landing time and height bonus0..1"
            ));
        }
        if !input.can_be_launched && input.landing_delay_us <= FALLBACK {
            return Err(format!(
                "branch_inputs index {index} nonlaunchable landing must follow the0.52s fallback"
            ));
        }
        timestamp_us
            .checked_add(input.landing_delay_us)
            .and_then(|landing| landing.checked_add(RECOVERY))
            .and_then(|deadline| deadline.checked_add(AIR_RECOVERY))
            .ok_or_else(|| format!("branch_inputs index {index} deadline overflow"))?;
    }
    Ok(())
}
pub(super) fn opened(
    ctx: &mut CastContext<'_>,
    plan: &Plan,
) -> Result<Option<(u64, State)>, String> {
    if !accepted(ctx.request) {
        return Ok(None);
    }
    let (Some(hold), Some(input)) = (
        ctx.request.hold_inputs.get(&ctx.command_index),
        ctx.request.branch_inputs.get(&ctx.command_index),
    ) else {
        return Ok(None);
    };
    let gate = match plan.kind {
        Kind::Normal(stage) => GATES[stage],
        Kind::Entry => ENTRY_GATE,
        _ => {
            return Err(
                "Hathor Branch requires the authored held Normal or ordinary entry input".into(),
            );
        }
    };
    if hold.duration_us <= gate {
        return Err(format!(
            "Hathor Branch hold {}us does not strictly exceed its source {gate}us gate",
            hold.duration_us
        ));
    }
    ctx.record_hypothesis(MODEL)?;
    let landing = ctx
        .time_us
        .checked_add(input.landing_delay_us)
        .ok_or("Hathor Branch landing overflow")?;
    Ok(Some((
        gate,
        State {
            command: ctx.command_index,
            launchable: input.can_be_launched,
            height_bonus: input.height_bonus,
            landing,
        },
    )))
}
pub(super) fn gate(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    command: usize,
) -> Result<(), String> {
    let now = ctx.state.current_time_us;
    if ctx.state.actors[owner].hp == 0 {
        return Ok(());
    }
    let Some(branch) = ctx.state.actors[owner]
        .character
        .hathor_mut()?
        .branch
        .as_ref()
        .filter(|b| b.command == command)
        .cloned()
    else {
        return Ok(());
    };
    let landing = branch.landing.max(now);
    let Some(due) = super::clock::enter_branch(
        &mut ctx.state.actors[owner],
        ctx.queue,
        now,
        command,
        branch.launchable,
        landing,
    )?
    else {
        return Ok(());
    };
    if branch.launchable {
        ctx.state.actors[owner]
            .character
            .hathor_mut()?
            .branch
            .as_mut()
            .ok_or("Hathor Branch state disappeared")?
            .landing = due;
    }
    if due <= ctx.request.absolute_limit_us {
        push(
            ctx,
            Event {
                owner,
                command,
                boundary: if branch.launchable {
                    Boundary::Land
                } else {
                    Boundary::Fallback
                },
            },
            due,
        )?;
    }
    ctx.trace.push(crate::simulation::GameEventReceipt {
        time_us: now,
        phase: "ModePause".into(),
        event_kind: "hathor_branch_gate".into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor: Some(owner),
        to_actor: Some(owner),
        energy_change: None,
        energy_fanout: None,
        data: Some(crate::simulation::GameEventPayload {
            command_index: Some(command),
            launchable: Some(branch.launchable),
            expires_at_us: Some(due),
            ..Default::default()
        }),
    });
    Ok(())
}
pub(crate) fn event(ctx: &mut CallbackContext<'_>, event: Event) -> Result<(), String> {
    if ctx.state.actors[event.owner].hp == 0 {
        return Ok(());
    }
    match event.boundary {
        Boundary::Land => land(ctx, event.owner, event.command),
        Boundary::Fallback => fallback(ctx, event.owner, event.command),
    }
}
fn land(ctx: &mut CallbackContext<'_>, owner: usize, command: usize) -> Result<(), String> {
    let Some(impact) = deliver(
        ctx,
        owner,
        command,
        Packet {
            kind: Kind::Branch,
            effect: PACKET,
            height: None,
            recovery: RECOVERY,
            jump: JUMP,
            switch: SWITCH,
            progress: 0,
        },
    )?
    else {
        return Ok(());
    };
    finish(ctx, owner, command)?;
    ctx.trace.push(crate::simulation::GameEventReceipt {
        time_us: ctx.state.current_time_us,
        phase: "ModePause".into(),
        event_kind: "hathor_branch_landing".into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor: Some(owner),
        to_actor: Some(owner),
        energy_change: None,
        energy_fanout: None,
        data: Some(crate::simulation::GameEventPayload {
            command_index: Some(command),
            completes_at_us: Some(impact),
            ..Default::default()
        }),
    });
    Ok(())
}
fn fallback(ctx: &mut CallbackContext<'_>, owner: usize, command: usize) -> Result<(), String> {
    let now = ctx.state.current_time_us;
    let Some(branch) = ctx.state.actors[owner]
        .character
        .hathor_mut()?
        .branch
        .as_ref()
        .filter(|b| b.command == command && !b.launchable)
        .cloned()
    else {
        return Ok(());
    };
    if branch.landing <= now {
        return air_land(ctx, owner, command);
    }
    super::air::begin_loop(ctx, owner, command, now, branch.landing)?;
    ctx.trace.push(crate::simulation::GameEventReceipt {
        time_us: now,
        phase: "ModePause".into(),
        event_kind: "hathor_branch_fallback".into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor: Some(owner),
        to_actor: Some(owner),
        energy_change: None,
        energy_fanout: None,
        data: Some(crate::simulation::GameEventPayload {
            command_index: Some(command),
            expires_at_us: Some(branch.landing),
            ..Default::default()
        }),
    });
    Ok(())
}
pub(super) fn air_land(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    command: usize,
) -> Result<(), String> {
    let now = ctx.state.current_time_us;
    let Some(branch) = ctx.state.actors[owner]
        .character
        .hathor_mut()?
        .branch
        .as_ref()
        .filter(|b| b.command == command && !b.launchable)
        .cloned()
    else {
        return Ok(());
    };
    let height = ExactFraction::new(1, 1)?.add(ExactFraction::from_ratio(branch.height_bonus)?)?;
    let Some(impact) = deliver(
        ctx,
        owner,
        command,
        Packet {
            kind: Kind::BranchFallback,
            effect: super::air::PACKET,
            height: Some(height),
            recovery: AIR_RECOVERY,
            jump: AIR_JUMP,
            switch: AIR_SWITCH,
            progress: AIR_CONTACT,
        },
    )?
    else {
        return Ok(());
    };
    finish(ctx, owner, command)?;
    ctx.trace.push(crate::simulation::GameEventReceipt {
        time_us: now,
        phase: "ModePause".into(),
        event_kind: "hathor_branch_fallback_landing".into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor: Some(owner),
        to_actor: Some(owner),
        energy_change: None,
        energy_fanout: None,
        data: Some(crate::simulation::GameEventPayload {
            command_index: Some(command),
            completes_at_us: Some(impact),
            ..Default::default()
        }),
    });
    Ok(())
}
fn finish(ctx: &mut CallbackContext<'_>, owner: usize, command: usize) -> Result<(), String> {
    let state = ctx.state.actors[owner].character.hathor_mut()?;
    if state.branch.as_ref().is_some_and(|b| b.command == command) {
        state.branch = None;
    }
    Ok(())
}
struct Packet<'a> {
    kind: Kind,
    effect: &'a str,
    height: Option<ExactFraction>,
    recovery: u64,
    jump: u64,
    switch: u64,
    progress: u64,
}
fn deliver(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    command: usize,
    packet: Packet<'_>,
) -> Result<Option<u64>, String> {
    let now = ctx.state.current_time_us;
    let config = &ctx.request.actors[owner];
    let coordinate = crate::simulation::inputs::normal_coordinate(config, "GA_Hathor_Melee")?;
    let (rate, scaling) = crate::simulation::inputs::coefficient(
        ctx.request,
        config,
        "GA_Hathor_Melee",
        packet.effect,
        coordinate,
    )?;
    let rate = if let Some(height) = packet.height {
        let scaled = ExactFraction::from_ratio(rate)?.mul(height)?;
        crate::formula::Ratio {
            numerator: scaled
                .numerator
                .try_into()
                .map_err(|_| "Hathor Branch packet rate numerator overflow")?,
            denominator: scaled
                .denominator
                .try_into()
                .map_err(|_| "Hathor Branch packet rate denominator overflow")?,
        }
    } else {
        rate
    };
    let mut metadata = crate::simulation::metadata::resolve_requested_action_packet(
        &ctx.request.content_id,
        ntesim_data::skill_levels::frozen::tables(),
        &config.character_id,
        "GA_Hathor_Melee",
        packet.effect,
        None,
        &ctx.request.model_assumptions,
    )?;
    let group = crate::runtime_api::next_packet_group(owner, &mut ctx.state.actors[owner])?;
    metadata.binding.energy_group_id = Some(group.clone());
    let Some(impact) = super::clock::branch_landing(
        &mut ctx.state.actors[owner],
        ctx.queue,
        now,
        super::clock::Landing {
            command,
            expected: packet.kind,
            recovery: packet.recovery,
            jump: packet.jump,
            switch: packet.switch,
            group: &group,
            progress: packet.progress,
        },
    )?
    else {
        return Ok(None);
    };
    let mut events = Vec::new();
    for target_index in 0..ctx.state.targets.len() {
        if ctx.state.targets[target_index].hp_current == 0 {
            continue;
        }
        crate::runtime_api::increment_sequence(ctx.sequence)?;
        events.push(ScheduledGameEvent {
            time_us: impact,
            priority: ntesim_core::execution::CausalPriority::HitDelivery,
            seq: *ctx.sequence,
            data: GameEventData::HitDelivery {
                command_index: command,
                actor_index: owner,
                target_index,
                ability_id: "GA_Hathor_Melee".into(),
                damage_effect_id: packet.effect.into(),
                atk_rate: rate,
                scaling_stat: scaling,
                metadata: Box::new(metadata.clone()),
            },
        });
    }
    ctx.queue.extend(events);
    ctx.trace.push(crate::simulation::GameEventReceipt {
        time_us: now,
        phase: "HitDelivery".into(),
        event_kind: "hathor_branch_packet_scheduled".into(),
        active_actor: ctx.state.active_actor_index,
        hit_index: None,
        from_actor: Some(owner),
        to_actor: None,
        energy_change: None,
        energy_fanout: None,
        data: Some(crate::simulation::GameEventPayload {
            command_index: Some(command),
            damage_effect_id: Some(packet.effect.into()),
            completes_at_us: Some(impact),
            ..Default::default()
        }),
    });
    Ok(Some(impact))
}
fn push(ctx: &mut CallbackContext<'_>, event: Event, time: u64) -> Result<(), String> {
    crate::runtime_api::increment_sequence(ctx.sequence)?;
    ctx.queue.push(ScheduledGameEvent {
        time_us: time,
        priority: ntesim_core::execution::CausalPriority::ModePause,
        seq: *ctx.sequence,
        data: GameEventData::Content(crate::callbacks::ContentEvent::HathorBranch(event)),
    });
    Ok(())
}

impl Event {
    pub(crate) fn timer_clock(&self) -> ntesim_core::kernel::timing::ClockDomain {
        crate::simulation::timer_policy::action(self.owner, self.command, false)
    }
}
