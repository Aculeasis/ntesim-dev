//! World's Tide Katana chain: five Q-category stages, the count markers and
//! the single finisher selected at exactly ten.
use crate::HypothesisId;
use crate::callbacks::ContentEvent;
use crate::runtime_api::{CallbackContext, CastContext, NormalActionOverride};
use crate::simulation::events::{GameEventData, ScheduledGameEvent};
use crate::simulation::state::ActorRuntime;
use crate::simulation::timer_queue::EventQueue as BinaryHeap;
use ntesim_core::execution::CausalPriority;

pub(crate) const MODEL: HypothesisId = HypothesisId::HotoriKatanaV1;

const ALL_IN_RANGE: HypothesisId = HypothesisId::HotoriAoeAllInRangeDummyV1;

struct Stage {
    /// Local source hit coordinates; the first row is the admitted primary.
    hits: &'static [(&'static str, u64)],
    marker_us: u64,
    any_recovery_us: u64,
    /// Floor of the native queued-next window plus the stage's source pause.
    next_window_us: u64,
}

const STAGES: [Stage; 5] = [
    Stage {
        hits: &[("GE_Player_Jin_UltraSkillMelee1_Damage", 183_431)],
        marker_us: 199_812,
        any_recovery_us: 1_017_981,
        // .09591392 + 13/60 + .012
        next_window_us: 324_580,
    },
    Stage {
        hits: &[
            ("GE_Player_Jin_UltraSkillMelee2_Damage", 158_106),
            ("GE_Player_Jin_UltraSkillMelee2_1_Damage", 333_093),
        ],
        marker_us: 183_166,
        any_recovery_us: 1_011_180,
        // .092007264 + 22/60 + .024
        next_window_us: 482_673,
    },
    Stage {
        hits: &[
            ("GE_Player_Jin_UltraSkillMelee3_Damage", 199_802),
            ("GE_Player_Jin_UltraSkillMelee3_Damage", 267_777),
            ("GE_Player_Jin_UltraSkillMelee3_Damage", 332_899),
            ("GE_Player_Jin_UltraSkillMelee3_1_Damage", 549_835),
        ],
        marker_us: 225_437,
        any_recovery_us: 1_467_706,
        // .09848472 + 38/60 + .060
        next_window_us: 791_818,
    },
    Stage {
        hits: &[
            ("GE_Player_Jin_UltraSkillMelee4_Damage", 83_744),
            ("GE_Player_Jin_UltraSkillMelee4_1_Damage", 266_776),
            ("GE_Player_Jin_UltraSkillMelee4_2_Damage", 490_245),
            ("GE_Player_Jin_UltraSkillMelee4_3_Damage", 632_842),
        ],
        marker_us: 99_513,
        any_recovery_us: 1_342_477,
        // .165074 + 33/60 + .100
        next_window_us: 815_074,
    },
    Stage {
        hits: &[
            ("GE_Player_Jin_UltraSkillMelee5_Damage", 212_672),
            ("GE_Player_Jin_UltraSkillMelee5_1_Damage", 272_579),
        ],
        marker_us: 240_311,
        any_recovery_us: 422_204,
        // .4222042 + .050
        next_window_us: 472_204,
    },
];

const FINISHER: &str = "GE_Player_Jin_UltraSkillMelee6_Damage";
const FINISHER_B4: &str = "GE_Player_Jin_UltraSkillMelee6Pas2_Damage";
const FINISHER_HIT_US: u64 = 1_457_857;
const FINISHER_CLEAR_US: u64 = 1_752_009;
const FINISHER_ANY_US: u64 = 2_136_724;
const FINISHER_SWITCH_LOCK_US: u64 = 2_160_101;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Event {
    Count { owner: usize, mode_generation: u64 },
    Finisher { owner: usize, mode_generation: u64 },
    Clear { owner: usize, mode_generation: u64 },
}

pub(crate) fn is_katana_packet(damage_effect_id: &str) -> bool {
    damage_effect_id.starts_with("GE_Player_Jin_UltraSkillMelee")
}

/// Marching Beyond Time's consumed Wastetime raises critic damage for the whole
/// initial Burst command family. The weapon applies it to the root Ultimate
/// packets; the Katana follow-ups reuse that consumed value until mode end.
pub(crate) fn family_crit_damage(
    ctx: &crate::runtime_api::DamageContext<'_>,
) -> Result<Option<crate::simulation::ExactFraction>, String> {
    if !is_katana_packet(ctx.damage_effect_id) {
        return Ok(None);
    }
    let crate::characters::RuntimeState::Hotori(state) = &ctx.actor.character else {
        return Ok(None);
    };
    let timepiece = &state.timepiece;
    let Some(mode) = timepiece.mode.as_ref() else {
        return Ok(None);
    };
    if !timepiece.mode_active(ctx.time_us) {
        return Ok(None);
    }
    Ok(ctx
        .actor
        .equipment
        .marching_beyond_time
        .ultimate_crit_damage
        .get(&mode.root_command_index)
        .copied())
}

fn finisher_packet(breakthrough: u8) -> &'static str {
    if breakthrough >= 4 {
        FINISHER_B4
    } else {
        FINISHER
    }
}

/// Zero-based current chain selection; stage five is the one finisher input.
pub(crate) fn next_stage(count: u32) -> Option<usize> {
    match count {
        0..=9 => Some(count as usize % STAGES.len()),
        10 => Some(STAGES.len()),
        _ => None,
    }
}

/// Time Stop replaces an admitted Basic with the next stage of the count-based
/// Katana chain; at exactly ten it selects the one breakthrough finisher.
pub(crate) fn replacement(actor: &ActorRuntime) -> Option<NormalActionOverride> {
    let crate::characters::RuntimeState::Hotori(state) = &actor.character else {
        return None;
    };
    let timepiece = &state.timepiece;
    timepiece.mode.as_ref()?;
    match timepiece.katana_count {
        0..=9 => {
            let stage = &STAGES[next_stage(timepiece.katana_count)?];
            Some(NormalActionOverride {
                packet: stage.hits[0].0,
                role: None,
                model: MODEL,
                timing: Some((stage.any_recovery_us, stage.hits[0].1)),
            })
        }
        10 => Some(NormalActionOverride {
            packet: finisher_packet(timepiece.breakthrough),
            role: None,
            model: MODEL,
            timing: Some((FINISHER_ANY_US, FINISHER_HIT_US)),
        }),
        _ => None,
    }
}

/// The next queued Katana input may start at its native next-Basic window even
/// though the previous input's Any recovery has not elapsed.
pub(crate) fn recovery_lock(
    actor: &ActorRuntime,
    ability_id: &str,
    now_us: u64,
) -> Option<(u64, HypothesisId)> {
    if ability_id != "GA_Jin_Melee" {
        return None;
    }
    let crate::characters::RuntimeState::Hotori(state) = &actor.character else {
        return None;
    };
    let timepiece = &state.timepiece;
    if !timepiece.mode_active(now_us) {
        return None;
    }
    timepiece
        .katana_next_input_us
        .map(|deadline| (deadline, MODEL))
}

fn schedule(
    queue: &mut BinaryHeap,
    sequence: &mut u64,
    time_us: u64,
    priority: CausalPriority,
    event: Event,
) -> Result<(), String> {
    crate::runtime_api::increment_sequence(sequence)?;
    queue.push(ScheduledGameEvent {
        time_us,
        priority,
        seq: *sequence,
        data: GameEventData::Content(ContentEvent::HotoriKatana(event)),
    });
    Ok(())
}

pub(crate) fn cast(ctx: &mut CastContext<'_>) -> Result<(), String> {
    let Some(packet) = ctx
        .packet_metadata
        .map(|metadata| metadata.damage_effect_id.clone())
    else {
        return Err("Hotori Katana input requires an admitted packet".into());
    };
    let (mode_generation, count) = {
        let state = ctx.actor.character.hotori_mut()?;
        let timepiece = &state.timepiece;
        if !timepiece.mode_active(ctx.time_us) {
            return Err("Hotori Katana inputs require an active World's Tide Time Stop".into());
        }
        (timepiece.mode_generation, timepiece.katana_count)
    };
    if let Some(index) = STAGES
        .iter()
        .position(|stage| stage.hits[0].0 == packet.as_str())
    {
        if count >= 10 || Some(index) != next_stage(count) {
            return Err(format!(
                "Hotori Katana stage {} cannot follow count {count}",
                index + 1
            ));
        }
        return cast_stage(ctx, index, mode_generation);
    }
    if packet == FINISHER || packet == FINISHER_B4 {
        return cast_finisher(ctx, &packet, mode_generation, count);
    }
    Err(format!(
        "Hotori Katana input selected unsupported packet {packet}"
    ))
}

fn cast_stage(ctx: &mut CastContext<'_>, index: usize, mode_generation: u64) -> Result<(), String> {
    let stage = &STAGES[index];
    let packet = stage.hits[0].0;
    let owner = ctx.actor_index;
    let now_us = ctx.time_us;
    ctx.record_hypothesis(MODEL)?;
    if ctx.request.targets.len() > 1 {
        ctx.record_hypothesis(ALL_IN_RANGE)?;
    }
    ctx.scope_primary_aoe(packet)?;
    for (effect, offset_us) in &stage.hits[1..] {
        let time_us = now_us
            .checked_add(*offset_us)
            .ok_or("Hotori Katana hit deadline overflow")?;
        ctx.schedule_native_aoe_group(effect, time_us)?;
    }
    let marker_us = now_us
        .checked_add(stage.marker_us)
        .ok_or("Hotori Katana count marker overflow")?;
    schedule(
        ctx.queue,
        ctx.sequence,
        marker_us,
        CausalPriority::ModePause,
        Event::Count {
            owner,
            mode_generation,
        },
    )?;
    let next_input_us = now_us
        .checked_add(stage.next_window_us)
        .ok_or("Hotori Katana next input overflow")?;
    ctx.actor
        .character
        .hotori_mut()?
        .timepiece
        .katana_next_input_us = Some(next_input_us);
    Ok(())
}

fn cast_finisher(
    ctx: &mut CastContext<'_>,
    packet: &str,
    mode_generation: u64,
    count: u32,
) -> Result<(), String> {
    if count != 10 {
        return Err(format!(
            "Hotori Katana finisher requires count 10, found {count}"
        ));
    }
    let owner = ctx.actor_index;
    let now_us = ctx.time_us;
    let breakthrough = ctx.actor.character.hotori_mut()?.timepiece.breakthrough;
    if packet != finisher_packet(breakthrough) {
        return Err(format!(
            "Hotori Katana finisher {packet} does not match breakthrough {breakthrough}"
        ));
    }
    ctx.record_hypothesis(MODEL)?;
    if ctx.request.targets.len() > 1 {
        ctx.record_hypothesis(ALL_IN_RANGE)?;
    }
    ctx.scope_primary_aoe(packet)?;
    let hit_us = now_us
        .checked_add(FINISHER_HIT_US)
        .ok_or("Hotori finisher hit overflow")?;
    schedule(
        ctx.queue,
        ctx.sequence,
        hit_us,
        CausalPriority::ProcCallback,
        Event::Finisher {
            owner,
            mode_generation,
        },
    )?;
    let clear_us = now_us
        .checked_add(FINISHER_CLEAR_US)
        .ok_or("Hotori finisher clear overflow")?;
    schedule(
        ctx.queue,
        ctx.sequence,
        clear_us,
        CausalPriority::ModePause,
        Event::Clear {
            owner,
            mode_generation,
        },
    )?;
    let switch_lock_us = now_us
        .checked_add(FINISHER_SWITCH_LOCK_US)
        .ok_or("Hotori finisher switch lock overflow")?;
    let timepiece = &mut ctx.actor.character.hotori_mut()?.timepiece;
    timepiece.katana_next_input_us = None;
    timepiece.switch_locked_until_us = switch_lock_us;
    Ok(())
}

fn count_marker(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    mode_generation: u64,
) -> Result<(), String> {
    if ctx.request.actors[owner].character_id != "character:1052" || ctx.state.actors[owner].hp == 0
    {
        return Ok(());
    }
    let now_us = ctx.state.current_time_us;
    let count = {
        let state = ctx.state.actors[owner].character.hotori_mut()?;
        let timepiece = &mut state.timepiece;
        if timepiece.mode_generation != mode_generation
            || !timepiece.mode_active(now_us)
            || timepiece.katana_count >= 10
        {
            return Ok(());
        }
        timepiece.katana_count += 1;
        timepiece.katana_count
    };
    crate::runtime_api::record_hypothesis_id(
        ctx.request,
        &mut ctx.state.execution_model_assumptions,
        MODEL,
    )?;
    crate::characters::hotori::timepiece::record_trace(
        ctx,
        "ModePause",
        "hotori_katana_count",
        Some(owner),
        Some(owner),
        Some(crate::simulation::GameEventPayload {
            count: Some(usize::try_from(count).map_err(|_| "Hotori katana count overflow")?),
            max_stacks: Some(10),
            ..Default::default()
        }),
    )
}

fn clear_marker(
    ctx: &mut CallbackContext<'_>,
    owner: usize,
    mode_generation: u64,
) -> Result<(), String> {
    if ctx.request.actors[owner].character_id != "character:1052" {
        return Ok(());
    }
    let _now_us = ctx.state.current_time_us;
    let ended = {
        let state = ctx.state.actors[owner].character.hotori_mut()?;
        let timepiece = &state.timepiece;
        timepiece.mode_generation != mode_generation || timepiece.mode.is_none()
    };
    if !ended {
        crate::characters::hotori::timepiece::finisher_delivered(ctx, owner, mode_generation)?;
    }
    crate::characters::hotori::timepiece::record_trace(
        ctx,
        "ModePause",
        "hotori_time_stop_clear",
        Some(owner),
        Some(owner),
        None,
    )
}

pub(crate) fn event(ctx: &mut CallbackContext<'_>, event: Event) -> Result<(), String> {
    match event {
        Event::Count {
            owner,
            mode_generation,
        } => count_marker(ctx, owner, mode_generation),
        Event::Finisher {
            owner,
            mode_generation,
        } => crate::characters::hotori::timepiece::finisher_delivered(ctx, owner, mode_generation),
        Event::Clear {
            owner,
            mode_generation,
        } => clear_marker(ctx, owner, mode_generation),
    }
}
