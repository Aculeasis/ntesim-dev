//! Checked cached absolute projections for owner timers. Deployed-object
//! lifetime has its own policy; independent status and proc timers use world time.
use crate::characters::RuntimeState;
pub(crate) fn future(deadline: &mut u64, from: u64, delta: u64) -> Result<(), String> {
    if *deadline > from {
        anchor(deadline, delta)?;
    }
    Ok(())
}
pub(crate) fn anchor(time: &mut u64, delta: u64) -> Result<(), String> {
    *time = time
        .checked_add(delta)
        .ok_or("paused owner timer projection overflow")?;
    Ok(())
}
pub(crate) fn owner_timers(
    state: &mut RuntimeState,
    from: u64,
    delta: u64,
    world: bool,
    object: bool,
) -> Result<(), String> {
    match state {
        RuntimeState::Mint(state) => {
            if world && let Some(deadline) = &mut state.a1_guard_until_us {
                future(deadline, from, delta)?;
            }
        }
        RuntimeState::Iroi(state) => {
            if world {
                future(&mut state.a2_cooldown_until_us, from, delta)?;
            }
            if world {
                for deadline in state.lamb_cooldowns.values_mut() {
                    future(deadline, from, delta)?;
                }
                if let Some(time) = &mut state.last_off_field_us {
                    anchor(time, delta)?;
                }
            }
            if object {
                future(&mut state.dream_end_us, from, delta)?;
            }
            if object && state.dream_active && state.dream_release_us <= from {
                anchor(&mut state.dream_release_us, delta)?;
            }
        }
        RuntimeState::Daffodil(state) => {
            if world {
                if let Some(deadline) = &mut state.phantom.expires_at_us {
                    future(deadline, from, delta)?;
                }
                for cap in state.discord_caps.values_mut() {
                    future(&mut cap.expires_at_us, from, delta)?;
                }
            }
            if world {
                future(&mut state.insight_until_us, from, delta)?;
            }
        }
        RuntimeState::Nanally(state) => {
            if world {
                future(&mut state.authority_until_us, from, delta)?;
            }
            if world {
                future(&mut state.authority_recast_unlock_us, from, delta)?;
            }
            if world {
                future(&mut state.fair_duel_ready_us, from, delta)?;
            }
            if object {
                future(&mut state.underboss_until_us, from, delta)?;
            }
            if world {
                future(&mut state.a1_ready_us, from, delta)?;
            }
            if world {
                future(&mut state.a3_ready_us, from, delta)?;
            }
            if world {
                future(&mut state.a5_until_us, from, delta)?;
            }
        }
        RuntimeState::Edgar(state) => {
            if world {
                future(&mut state.key_ready_us, from, delta)?;
            }
            if object {
                future(&mut state.finnegan_wake_until_us, from, delta)?;
            }
            if object && let Some(field) = &mut state.field {
                future(&mut field.until, from, delta)?;
            }
        }
        RuntimeState::Blackbird(state) => {
            if world {
                if let Some(form) = &mut state.form {
                    form.pause_world(delta)?;
                }
                state.bird.pause_world(from, delta)?;
            }
            if world && let Some(deadline) = &mut state.skill_permission_until_us {
                future(deadline, from, delta)?;
            }
        }
        RuntimeState::Skia(state) => {
            if world {
                future(&mut state.gnaw_until_us, from, delta)?;
            }
        }
        RuntimeState::Haniel(state) => {
            if world && let Some(token) = &mut state.token {
                future(&mut token.until, from, delta)?;
            }
            if world && let Some(deadline) = &mut state.riposte_ready_us {
                future(deadline, from, delta)?;
            }
            if object && let Some(deadline) = &mut state.hootie_until_us {
                future(deadline, from, delta)?;
            }
            if world && let Some(deadline) = &mut state.paranormal_ace_until_us {
                future(deadline, from, delta)?;
            }
            if world {
                future(&mut state.harmony_ready_us, from, delta)?;
            }
        }
        RuntimeState::Fadia(state) => {
            if world && let Some(deadline) = &mut state.lilith_until_us {
                future(deadline, from, delta)?;
            }
        }
        RuntimeState::Lacrimosa(state) => {
            if world {
                state.sleep_in.pause_world(from, delta)?;
            }
            if world {
                future(&mut state.bat_evade_until_us, from, delta)?;
            }
            if world {
                future(&mut state.baicang_tag_until_us, from, delta)?;
            }
            if world {
                for carrier in state.nightmare.values_mut() {
                    anchor(&mut carrier.deadline_us, delta)?;
                    anchor(&mut carrier.grid_origin_us, delta)?;
                }
            }
        }
        RuntimeState::Zankou(state) => {
            if world {
                if let Some(deadline) = &mut state.healing_next_us {
                    future(deadline, from, delta)?;
                }
                for train in state
                    .vile_ash_trains
                    .values_mut()
                    .chain(state.heartwrench_trains.values_mut())
                {
                    if train.deadline_us > from {
                        anchor(&mut train.next_tick_us, delta)?;
                        anchor(&mut train.deadline_us, delta)?;
                    }
                }
            }
            if world {
                future(&mut state.form_until_us, from, delta)?;
            }
            if world {
                future(&mut state.paid_burst_until_us, from, delta)?;
            }
            if world {
                future(&mut state.free_inferno_until_us, from, delta)?;
            }
            if world {
                future(&mut state.hunt_until_us, from, delta)?;
            }
            if world {
                future(&mut state.hunt_retained_until_us, from, delta)?;
            }
            if world {
                future(&mut state.delusion_until_us, from, delta)?;
            }
            if world {
                future(&mut state.delusion_retained_until_us, from, delta)?;
            }
            if world {
                future(&mut state.r6_enhancement_until_us, from, delta)?;
            }
        }
        RuntimeState::Hathor(state) => {
            if world && let Some(gaze) = &mut state.gaze {
                gaze.pause_world(from, delta)?;
            }
            if world && let Some(deadline) = &mut state.riposte_ready_us {
                future(deadline, from, delta)?;
            }
            if world {
                future(&mut state.riposte_reward_until_us, from, delta)?;
            }
            if world {
                future(&mut state.emergency_delivery_until_us, from, delta)?;
            }
            if world && let Some(burst) = &mut state.burst {
                burst.pause_world(from, delta)?;
            }
        }
        RuntimeState::Shinku(state) => {
            if world {
                future(&mut state.surging_crimson_until_us, from, delta)?;
            }
        }
        RuntimeState::Baicang(state) => {
            if object && state.domain_until_us > from {
                anchor(&mut state.domain_start_us, delta)?;
            }
            if world {
                for word in &mut state.stored_words {
                    future(&mut word.expiry_us, from, delta)?;
                }
                for deadline in &mut state.a2_stack_expiries {
                    future(deadline, from, delta)?;
                }
            }
            if object {
                future(&mut state.domain_until_us, from, delta)?;
            }
            if world {
                future(&mut state.power_word_until_us, from, delta)?;
            }
            if world {
                future(&mut state.fatal_hit_immune_until_us, from, delta)?;
            }
            if world {
                future(&mut state.dodge_immune_until_us, from, delta)?;
            }
            if world {
                future(&mut state.heavy_dodge_until_us, from, delta)?;
            }
            if world {
                future(&mut state.skia_assist_until_us, from, delta)?;
            }
            if world {
                future(&mut state.derived_skill_stage_expires_at_us, from, delta)?;
            }
        }
        RuntimeState::Aurelia(state) => {
            if world {
                future(&mut state.cadenza_until_us, from, delta)?;
            }
            if world {
                future(&mut state.crescendo_expires_at_us, from, delta)?;
            }
        }
        RuntimeState::Chaos(state) => {
            if world && let Some(form) = &mut state.form {
                form.pause_world(delta)?;
            }
        }
        RuntimeState::Linko(state) => {
            if world && let Some(deadline) = &mut state.oversync_until_us {
                future(deadline, from, delta)?;
            }
            if world {
                for deadline in state.hexed_targets.values_mut() {
                    future(deadline, from, delta)?;
                }
                if let Some(deadline) = &mut state.next_pulse_us {
                    future(deadline, from, delta)?;
                }
            }
        }
        RuntimeState::Chiz(state) => {
            if world && let Some(finance) = &mut state.finance {
                finance.pause_world(from, delta)?;
            }
            if world && let Some(pressed) = &mut state.pressed {
                pressed.pause_world(from, delta)?;
            }
        }
        RuntimeState::Sakiri(state) => {
            if world {
                for deadline in &mut state.a3_kill_bonus_deadlines {
                    future(deadline, from, delta)?;
                }
            }
        }
        RuntimeState::Jiuyuan(state) => {
            if world {
                for pact in state.pacts.values_mut() {
                    anchor(&mut pact.planted_us, delta)?;
                    anchor(&mut pact.matures_us, delta)?;
                    anchor(&mut pact.deadline_us, delta)?;
                    if let crate::characters::jiuyuan::pact::PactPhase::Closing { closes_at_us } =
                        &mut pact.phase
                    {
                        anchor(closes_at_us, delta)?;
                    }
                }
            }
        }
        RuntimeState::Adler(state) if world => {
            for deadline in state.skill_deadlines.values_mut() {
                future(deadline, from, delta)?;
            }
        }
        _ => {}
    }
    Ok(())
}

pub(crate) fn owner_action(
    state: &mut RuntimeState,
    owner: usize,
    start: u64,
    duration: u64,
) -> Result<(), String> {
    if let Some((_, Some(anchor_us))) = state.normal_chain_fields() {
        anchor(anchor_us, duration)?;
    }
    match state {
        RuntimeState::Mint(state) => {
            if let Some(clock) = &mut state.clock {
                clock.global_freeze(owner, start, duration)?;
            }
        }
        RuntimeState::Nanally(state) => state.clock.global_freeze(owner, start, duration)?,
        RuntimeState::Iroi(state) => {
            if let Some(clock) = &mut state.clock {
                clock.global_freeze(owner, start, duration)?;
            }
        }
        RuntimeState::Hathor(state) => {
            if let Some(clock) = &mut state.clock {
                clock.global_freeze(owner, start, duration)?;
            }
        }
        RuntimeState::Haniel(state) => {
            if let Some(clock) = &mut state.clock {
                clock.global_freeze(owner, start, duration)?;
            }
        }
        RuntimeState::Fadia(state) => {
            if let Some(clock) = &mut state.clock {
                clock.global_freeze(owner, start, duration)?;
            }
        }
        RuntimeState::Edgar(state) => {
            if let Some(clock) = &mut state.clock {
                clock.global_freeze(owner, start, duration)?;
            }
        }
        RuntimeState::Daffodil(state) => {
            if let Some(clock) = &mut state.clock {
                clock.global_freeze(owner, start, duration)?;
            }
        }
        RuntimeState::Zankou(state) => {
            if let Some(clock) = &mut state.clock {
                clock.global_freeze(owner, start, duration)?;
            }
        }
        RuntimeState::Blackbird(state) => {
            future(&mut state.qte_switch_until_us, start, duration)?;
            state.clock.global_freeze(owner, start, duration)?;
        }
        RuntimeState::Chaos(state) => {
            future(&mut state.burst_switch_until_us, start, duration)?;
            future(&mut state.burst_immunity_until_us, start, duration)?;
            future(&mut state.verdict_immunity_until_us, start, duration)?;
            if let Some(clock) = &mut state.clock {
                clock.global_freeze(owner, start, duration)?;
            }
        }
        RuntimeState::Baicang(state) => state.clock.global_freeze(owner, start, duration)?,
        RuntimeState::Chiz(state) => {
            if let Some(clock) = &mut state.clock {
                clock.global_freeze(owner, start, duration)?;
            }
            if let Some(pressed) = &mut state.pressed {
                pressed.pause_action(owner, start, duration)?;
            }
        }
        _ => {}
    }
    Ok(())
}

pub(crate) fn gear_timers(
    state: &mut crate::gear::RuntimeState,
    from: u64,
    delta: u64,
) -> Result<(), String> {
    future(&mut state.be_happy.ready_at_us, from, delta)?;
    future(&mut state.bopu.ready_at_us, from, delta)?;
    future(&mut state.demon_blade.ready_at_us, from, delta)?;
    future(&mut state.far_side_moon.ready_at_us, from, delta)?;
    future(&mut state.mind_royale.ready_at_us, from, delta)?;
    future(&mut state.moon.ready_at_us, from, delta)?;
    future(&mut state.noros.ready_at_us, from, delta)?;
    future(&mut state.spider.ready_at_us, from, delta)?;
    future(&mut state.whale.ready_at_us, from, delta)?;
    future(&mut state.bit_game.off_field_ready_at_us, from, delta)?;
    future(&mut state.bit_game.on_field_ready_at_us, from, delta)?;
    future(&mut state.black_book.arc_active_until_us, from, delta)?;
    future(&mut state.police_rat.arc_active_until_us, from, delta)?;
    future(&mut state.rishi.field_until_us, from, delta)?;
    if let Some(deadline) = &mut state.bittercake.next_trigger_us {
        future(deadline, from, delta)?;
    }
    if let Some(deadline) = &mut state.tiger_tally.left_talisman_until_us {
        future(deadline, from, delta)?;
    }
    if let Some(deadline) = &mut state.tiger_tally.right_talisman_until_us {
        future(deadline, from, delta)?;
    }
    if let Some(proc) = &mut state.nakupeda {
        future(&mut proc.ready_at_us, from, delta)?;
    }
    state.good_boy.pause_world(from, delta)?;
    Ok(())
}
