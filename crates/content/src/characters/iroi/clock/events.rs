//! Montage notifications may be cancelled; born actors own their world ticks.
use super::*;
pub(super) fn apply(ctx: &mut CallbackContext<'_>, event: Event) -> Result<(), String> {
    match event {
        Event::A2Creation {
            iroi_idx,
            candidate,
            command_index,
        } => a2::creation(ctx, iroi_idx, candidate, command_index),
        Event::LambAi {
            owner,
            ally_index,
            lamb_type,
            skill_number,
        } => super::super::regression::on_lamb_ai_tick(
            ctx,
            owner,
            ally_index,
            lamb_type,
            skill_number,
        ),
        Event::Actor {
            owner,
            source,
            remaining,
            kind,
        } => actor(ctx, owner, source, remaining, kind),
        Event::Montage {
            owner,
            command,
            notify,
            ..
        } => {
            let now = ctx.state.current_time_us;
            if ctx.state.actors[owner].hp == 0 {
                return Ok(());
            }
            let Some(action) = ctx.state.actors[owner]
                .character
                .iroi_mut()?
                .clock
                .as_ref()
                .and_then(|c| c.active.clone())
                .filter(|a| a.command == command)
            else {
                return Ok(());
            };
            match notify {
                Notify::NormalHit => hit(ctx, &action),
                Notify::NormalActor => spawn(ctx, action, ActorKind::NormalThree, 200000, 8),
                Notify::SkillGun => spawn(ctx, action, ActorKind::SkillGun, 1200000, 1),
                Notify::SkillSyringe => spawn(ctx, action, ActorKind::SkillSyringe, 0, 7),
                Notify::SkillTrumpet => spawn(ctx, action, ActorKind::SkillTrumpet, 0, 1),
                Notify::BranchTag { stage } => {
                    spawn(ctx, action.clone(), ActorKind::BranchGun, 250000, 1)?;
                    if matches!(
                        stage,
                        held::BranchStage::Stage2
                            | held::BranchStage::Stage3Early
                            | held::BranchStage::Stage3Full
                    ) {
                        spawn(ctx, action.clone(), ActorKind::BranchSyringe, 420000, 1)?;
                    }
                    if matches!(
                        stage,
                        held::BranchStage::Stage3Early | held::BranchStage::Stage3Full
                    ) {
                        spawn(ctx, action, ActorKind::BranchTrumpet, 300000, 1)?;
                    }
                    Ok(())
                }
                Notify::LucidShot(_) => {
                    let effect = super::super::branch::BRANCH5;
                    let group = action.source.with_cast(ctx, action.owner, 0, |cast| {
                        cast.schedule_native_aoe_group(effect, cast.time_us)
                    })?;
                    let clock = ctx.state.actors[action.owner]
                        .character
                        .iroi_mut()?
                        .clock
                        .as_mut()
                        .ok_or("Iroi LucidShot lost clock")?;
                    clock.pause_packets.insert(group.clone());
                    clock
                        .active
                        .as_mut()
                        .ok_or("Iroi LucidShot lost action")?
                        .groups
                        .insert(group);
                    Ok(())
                }
                Notify::SkillHeal => skill::heal(ctx, &action),
                Notify::SupportHit => support_hit(ctx, &action),
                Notify::SupportHeal => support_heal(ctx, &action),
                Notify::BurstHeal
                | Notify::BurstHit
                | Notify::BurstCarrier
                | Notify::BurstRelease => burst::notify(ctx, &action, notify),
                Notify::SkillCooldown => {
                    let state = &mut ctx.state.actors[owner];
                    state.ability_cooldowns.insert(
                        skill::ABILITY.into(),
                        now.checked_add(12_000_000)
                            .ok_or("Iroi Skill cooldown overflow")?,
                    );
                    state
                        .character
                        .iroi_mut()?
                        .clock
                        .as_mut()
                        .and_then(|c| c.active.as_mut())
                        .ok_or("Iroi Skill notify lost action")?
                        .cooldown_started = true;
                    Ok(())
                }
            }
        }
    }
}
fn support_hit(ctx: &mut CallbackContext<'_>, action: &Action) -> Result<(), String> {
    let target = ctx.request.aim_target(action.command);
    let group = action.source.with_cast(ctx, action.owner, target, |cast| {
        cast.schedule_native_capped_aoe_packet(QTE_PACKET, cast.time_us, 12)
    })?;
    let clock = ctx.state.actors[action.owner]
        .character
        .iroi_mut()?
        .clock
        .as_mut()
        .ok_or("Iroi Support lost clock")?;
    clock.pause_packets.insert(group.clone());
    clock
        .active
        .as_mut()
        .ok_or("Iroi Support lost action")?
        .groups
        .insert(group);
    Ok(())
}

fn support_heal(ctx: &mut CallbackContext<'_>, action: &Action) -> Result<(), String> {
    let ratio = action.source.with_cast(ctx, action.owner, 0, |cast| {
        let coordinate = super::super::skill::heal_coordinate(cast, "Support")?;
        super::super::parameters::heal_ratio(super::super::parameters::HealKind::Qte, coordinate)
    })?;
    super::super::callback::apply_support_heal(ctx, action.owner, ratio)
}

fn spawn(
    ctx: &mut CallbackContext<'_>,
    action: Action,
    kind: ActorKind,
    delay: u64,
    remaining: u8,
) -> Result<(), String> {
    push(
        ctx.queue,
        ctx.sequence,
        ctx.state
            .current_time_us
            .checked_add(delay)
            .ok_or("Iroi actor birth overflow")?,
        Event::Actor {
            owner: action.owner,
            source: Box::new(action.source),
            remaining,
            kind,
        },
    )
}
fn actor(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    source: Box<OwnedAction>,
    remaining: u8,
    kind: ActorKind,
) -> Result<(), String> {
    if ctx.state.actors[owner].hp == 0 {
        return Ok(());
    }
    let effect = skill::effect(kind);
    let group = source.with_cast(ctx, owner, 0, |cast| {
        cast.schedule_native_aoe_group(effect, cast.time_us)
    })?;
    if profile::pause(effect).is_some() {
        ctx.state.actors[owner]
            .character
            .iroi_mut()?
            .clock
            .as_mut()
            .ok_or("Iroi born actor lost caster clock")?
            .pause_packets
            .insert(group);
    }
    if remaining > 1 {
        push(
            ctx.queue,
            ctx.sequence,
            ctx.state
                .current_time_us
                .checked_add(100000)
                .ok_or("Iroi actor tick overflow")?,
            Event::Actor {
                owner,
                source,
                remaining: remaining - 1,
                kind,
            },
        )?;
    }
    Ok(())
}
fn hit(ctx: &mut CallbackContext<'_>, action: &Action) -> Result<(), String> {
    let Kind::Normal(stage) = action.kind else {
        return Err("Iroi Normal notify has wrong section".into());
    };
    let target = ctx.request.aim_target(action.command);
    let target = if ctx.state.targets[target].hp_current > 0 {
        Some(target)
    } else {
        ctx.state.targets.iter().position(|t| t.hp_current > 0)
    };
    let Some(target) = target else {
        return Ok(());
    };
    let group = action.source.with_cast(ctx, action.owner, target, |cast| {
        if stage < 2 {
            cast.schedule_native_target_group(profile::EFFECTS[stage], target, cast.time_us)
        } else {
            cast.schedule_native_aoe_group(profile::EFFECTS[stage], cast.time_us)
        }
    })?;
    let clock = ctx.state.actors[action.owner]
        .character
        .iroi_mut()?
        .clock
        .as_mut()
        .ok_or("Iroi contact lost clock")?;
    clock.pause_packets.insert(group.clone());
    clock
        .active
        .as_mut()
        .ok_or("Iroi contact lost action")?
        .groups
        .insert(group);
    Ok(())
}
