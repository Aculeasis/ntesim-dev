//! Envelope clocks are selected from native event variants and finite contexts.
use super::events::GameEventData;
use ntesim_core::kernel::timing::{ClockDomain, PauseOwner};
pub(crate) fn action(owner: usize, command: usize, active_stop: bool) -> ClockDomain {
    ClockDomain::Action {
        context: PauseOwner {
            owner,
            generation: command as u64,
        },
        advance_during_active_stop: active_stop,
    }
}

pub(crate) fn immediate_callback(event: &GameEventData) -> bool {
    use crate::callbacks::ContentEvent::*;
    match event {
        GameEventData::ReactionNovaEnd(_) => true, // explicit reapplication/Discord closure
        GameEventData::Content(content) => !matches!(
            content,
            BuffLifecycleExpiry
                | BlossomBudExpiry { .. }
                | MotorCandyTick { .. }
                | EdgarHealTick { .. }
                | BaicangDomainTick { .. }
                | BaicangDomainEnd { .. }
                | PoliceRatWhiskerTick { .. }
                | BlackBookDesignateTick { .. }
                | JiuyuanPactTick { .. }
                | JiuyuanPactClose { .. }
                | IroiDreamTick { .. }
                | IroiDreamEnd { .. }
                | IroiBudCopyCheck { .. }
                | LinkoXiaozhenPulse { .. }
                | NanallyUnderboss(_)
                | Haniel(
                    crate::characters::haniel::hootie::Event::Ensemble { .. }
                        | crate::characters::haniel::hootie::Event::Extra { .. }
                        | crate::characters::haniel::hootie::Event::Expire { .. }
                )
                | ZankouCombat(_)
                | Akane(crate::characters::akane::events::Event::PerfectPitchTick { .. })
                | IroiClock(crate::characters::iroi::clock::Event::LambAi { .. })
                | Edgar(crate::characters::edgar::clock::Event::FieldTick { .. })
        ),
        _ => false,
    }
}
pub(crate) fn domain(event: &GameEventData) -> ClockDomain {
    use GameEventData::*;
    match event {
        CinematicRelease { context } => action(context.owner, context.generation as usize, false),
        HitDelivery { metadata, .. }
            if metadata
                .packet
                .semantic_tags
                .contains(crate::characters::packet_semantics::DOT_DAMAGE_TAG) =>
        {
            ClockDomain::World
        }
        HitDelivery {
            actor_index,
            command_index,
            ..
        }
        | TargetUnbalance {
            actor_index,
            command_index,
            ..
        } => action(*actor_index, *command_index, false),
        FollowupHit { packet, .. } => action(packet.actor_index, packet.command_index, false),
        DaffodilStepAfterSupport { owner, command } => action(*owner, *command, false),
        Content(event) => content_domain(event),
        TargetBreakRecovery { .. }
        | ReactionPistil(_)
        | ReactionScorchTick(_)
        | ReactionScorchRefresh { .. }
        | ReactionNovaDetonate(_)
        | ReactionNovaEnd(_)
        | ReactionRemoraEnd { .. }
        | IncomingDamage { .. }
        | DummyCombat { .. } => ClockDomain::World,
        ActionAdmission { .. }
        | SwitchActor { .. }
        | HpCost { .. }
        | ActorInterrupt { .. }
        | ActorParry { .. }
        | CombatTransition { .. }
        | LiveBranch { .. }
        | LiveEnergyWait { .. }
        | ResumeLiveEnergyWait
        | TargetBreakDamage(_) => ClockDomain::Absolute,
    }
}
fn content_domain(event: &crate::callbacks::ContentEvent) -> ClockDomain {
    use crate::callbacks::ContentEvent::*;
    match event {
        MintClock(event) => event.timer_clock(),
        NanallyClock(event) => event.timer_clock(),
        HathorClock(event) => event.timer_clock(),
        HathorAir(event) => event.timer_clock(),
        HathorBranch(event) => event.timer_clock(),
        HanielClock(event) => event.timer_clock(),
        Fadia(event) => event.timer_clock(),
        Blackbird(
            crate::characters::blackbird::events::Event::BurstBuff {
                actor_index,
                command_index,
                ..
            }
            | crate::characters::blackbird::events::Event::SkillPermission {
                actor_index,
                command_index,
                ..
            }
            | crate::characters::blackbird::events::Event::SupportMalice {
                actor_index,
                command_index,
                ..
            }
            | crate::characters::blackbird::events::Event::SkillHeal {
                actor_index,
                command_index,
            },
        ) => action(*actor_index, *command_index, false),
        Chaos(
            crate::characters::chaos::events::Event::SkillFinish { owner, command, .. }
            | crate::characters::chaos::events::Event::Warrant { owner, command, .. }
            | crate::characters::chaos::events::Event::EnterForm { owner, command, .. }
            | crate::characters::chaos::events::Event::VerdictRead { owner, command, .. }
            | crate::characters::chaos::events::Event::VerdictCommit { owner, command, .. }
            | crate::characters::chaos::events::Event::SupportFinish { owner, command, .. },
        )
        | Akane(
            crate::characters::akane::events::Event::ModeOvertone { owner, command, .. }
            | crate::characters::akane::events::Event::ModeReverb { owner, command, .. }
            | crate::characters::akane::events::Event::SyncedBuffs { owner, command, .. }
            | crate::characters::akane::events::Event::SkillFinish { owner, command, .. },
        )
        | Chiz(
            crate::characters::chiz::events::Event::HeldPhase { owner, command, .. }
            | crate::characters::chiz::events::Event::PressSettlement { owner, command }
            | crate::characters::chiz::events::Event::PressPause { owner, command, .. },
        )
        | ZankouBurst(
            crate::characters::zankou::burst::Event::Grant { owner, command }
            | crate::characters::zankou::burst::Event::Finish { owner, command, .. },
        )
        | ZankouForms(
            crate::characters::zankou::forms::Event::A5Impulse { owner, command }
            | crate::characters::zankou::forms::Event::Transition { owner, command }
            | crate::characters::zankou::forms::Event::Bloodfeast { owner, command },
        ) => action(*owner, *command, false),
        HotoriTimepiece(
            crate::characters::hotori::timepiece::Event::ModeEntry { .. }
            | crate::characters::hotori::timepiece::Event::ModeDeadline { .. }
            | crate::characters::hotori::timepiece::Event::Playback { .. },
        )
        | HotoriKatana(_) => ClockDomain::Absolute,
        Edgar(crate::characters::edgar::clock::Event::Action { owner, command, .. })
        | IroiClock(crate::characters::iroi::clock::Event::Montage { owner, command, .. })
        | Daffodil(
            crate::characters::daffodil::clock::Event::SkillCooldown { owner, command }
            | crate::characters::daffodil::clock::Event::FinisherCheck { owner, command },
        ) => action(*owner, *command, false),
        Chiz(crate::characters::chiz::events::Event::Market(
            crate::characters::chiz::market::Event::Enter { owner, command },
        )) => action(*owner, *command, false),
        Haniel(crate::characters::haniel::hootie::Event::Spawn(spawn)) => spawn.timer_clock(),
        JiuyuanPactApply(application) => {
            action(application.actor_index, application.command_index, false)
        }
        LacrimosaNightmareApply(application) => {
            action(application.actor_index, application.command_index, false)
        }
        FadiaEffects(crate::characters::fadia::effects::Event::Heal { owner, command, .. }) => {
            action(*owner, *command, false)
        }
        ZankouVileAshApply(application) => {
            action(application.actor_index, application.command_index, false)
        }
        IroiClock(crate::characters::iroi::clock::Event::Actor { owner, source, .. }) => {
            action(*owner, source.command(), false)
        }
        BaicangFieldWords {
            actor_index,
            command_index,
            ..
        } => action(*actor_index, *command_index, false),
        IroiBurstHeal {
            source_actor_index: actor_index,
            command_index,
            ..
        }
        | BaicangHitstop {
            actor_index,
            command_index,
            ..
        }
        | IroiDreamRelease {
            actor_index,
            command_index,
            ..
        } => action(*actor_index, *command_index, false),
        Edgar(crate::characters::edgar::clock::Event::FieldExpiry { .. })
        | BlossomBudExpiry { .. }
        | Haniel(crate::characters::haniel::hootie::Event::Expire { .. })
        | BaicangDomainEnd { .. }
        | IroiDreamEnd { .. } => ClockDomain::ObjectLifetime,
        _ => ClockDomain::World,
    }
}

/// Project callback-owned absolute deadlines on the same declared envelope
/// clock. Producer lifetimes with a different basis are read from live state.
pub(crate) fn project_payload(event: &mut GameEventData, paused_us: u64) {
    use crate::callbacks::ContentEvent;
    let shift = |time: &mut u64| {
        *time = time
            .checked_add(paused_us)
            .expect("validated absolute timer projection")
    };
    if let GameEventData::Content(content) = event {
        match content {
            ContentEvent::JiuyuanPactTick { planted_us, .. } => shift(planted_us),
            ContentEvent::PoliceRatWhiskerTick { end_time_us, .. }
            | ContentEvent::BlackBookDesignateTick { end_time_us, .. } => shift(end_time_us),
            ContentEvent::Haniel(crate::characters::haniel::hootie::Event::Expire {
                until,
                ..
            }) => shift(until),
            ContentEvent::Haniel(crate::characters::haniel::hootie::Event::Extra {
                token, ..
            }) => shift(&mut token.until),
            ContentEvent::BlackBookArcStartupComplete {
                cooldown_until_us, ..
            } => shift(cooldown_until_us),
            _ => {}
        }
    }
}
