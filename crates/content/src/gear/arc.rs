use crate::HypothesisId;
use crate::runtime_api::CallbackContext;
use crate::simulation::{GameActorConfig, GearProcConfig};

pub(crate) enum Arc {
    SilentGarden(crate::weapons::jingmotingyuan::arc::Parameters),
    Eclipse { duration_us: u64, cooldown_us: u64 },
    OfficerWhisker(crate::weapons::police_rat::arc::Parameters),
    BlackBook(crate::weapons::black_book::arc::Parameters),
}

pub(crate) fn hypothesis(weapon_id: &str) -> Option<HypothesisId> {
    if weapon_id.eq_ignore_ascii_case("weapon:fork_jingmotingyuan") {
        Some(crate::weapons::jingmotingyuan::HYPOTHESIS_ID)
    } else if weapon_id.eq_ignore_ascii_case("weapon:fork_rishi") {
        Some(crate::weapons::rishi::HYPOTHESIS_ID)
    } else if weapon_id.eq_ignore_ascii_case("weapon:fork_PoliceRat") {
        Some(crate::weapons::police_rat::HYPOTHESIS_ID)
    } else if weapon_id.eq_ignore_ascii_case("weapon:fork_BlackBook") {
        Some(crate::weapons::black_book::HYPOTHESIS_ID)
    } else {
        None
    }
}

impl Arc {
    /// A live Black Tome startup owns the actor's recovery lock. Owner action
    /// overrides cannot shorten that window; only the explicit interruption
    /// event can cancel it.
    pub(crate) fn startup_recovery_deadline(
        actor: &crate::simulation::state::ActorRuntime,
        current_time_us: u64,
    ) -> Option<u64> {
        let pending = actor.equipment.black_book.pending_startup?;
        if current_time_us >= pending.completes_at_us {
            return None;
        }
        Some(pending.completes_at_us)
    }

    pub(crate) fn project(actor: &GameActorConfig) -> Result<Self, String> {
        match hypothesis(&actor.weapon_id) {
            Some(crate::HypothesisId::WeaponsJingmotingyuanV1) => {
                if !actor
                    .gear_procs
                    .iter()
                    .any(|proc| matches!(proc, GearProcConfig::JingmotingyuanCrit { .. }))
                {
                    return Err("Arc: Silent Garden requires equipped Silent Garden with its projected passive".into());
                }
                Ok(Self::SilentGarden(
                    crate::weapons::jingmotingyuan::arc::parameters(actor)?,
                ))
            }
            Some(crate::HypothesisId::WeaponsRishiV2) => {
                let (duration_us, cooldown_us) = crate::weapons::rishi::arc_parameters(actor)?;
                Ok(Self::Eclipse {
                    duration_us,
                    cooldown_us,
                })
            }
            Some(crate::HypothesisId::WeaponsPoliceRatV3) => Ok(Self::OfficerWhisker(
                crate::weapons::police_rat::arc::parameters(actor)?,
            )),
            Some(crate::HypothesisId::WeaponsBlackBookV1) => {
                Ok(Self::BlackBook(crate::weapons::black_book::arc::parameters(
                    actor,
                )?))
            }
            _ => Err("Arc input requires equipped Silent Garden, Eclipse, Justice Executioner, or Black Tome".into()),
        }
    }

    pub(crate) fn ability_id(&self) -> &'static str {
        match self {
            Self::SilentGarden(_) => crate::weapons::jingmotingyuan::arc::ABILITY,
            Self::Eclipse { .. } => crate::weapons::rishi::ARC_ABILITY,
            Self::OfficerWhisker(_) => crate::weapons::police_rat::arc::ABILITY,
            Self::BlackBook(_) => crate::weapons::black_book::arc::ABILITY,
        }
    }

    pub(crate) fn hypothesis(&self) -> HypothesisId {
        match self {
            Self::SilentGarden(_) => crate::weapons::jingmotingyuan::HYPOTHESIS_ID,
            Self::Eclipse { .. } => crate::weapons::rishi::HYPOTHESIS_ID,
            Self::OfficerWhisker(_) => crate::weapons::police_rat::HYPOTHESIS_ID,
            Self::BlackBook(_) => crate::weapons::black_book::HYPOTHESIS_ID,
        }
    }

    pub(crate) fn cooldown_us(&self) -> u64 {
        match self {
            Self::SilentGarden(parameters) => parameters.cooldown_us,
            Self::Eclipse { cooldown_us, .. } => *cooldown_us,
            Self::OfficerWhisker(parameters) => parameters.cooldown_us,
            Self::BlackBook(_) => 0,
        }
    }

    pub(crate) fn validate_deadline(&self, timestamp_us: u64) -> Result<(), String> {
        match self {
            Self::BlackBook(parameters) => {
                let start_us = timestamp_us
                    .checked_add(parameters.startup_us)
                    .ok_or("Arc startup deadline overflow")?;
                let _end_us = start_us
                    .checked_add(parameters.duration_us)
                    .ok_or("Arc duration deadline overflow")?;
                start_us
                    .checked_add(parameters.designate_interval_us)
                    .ok_or("Arc designation deadline overflow")?;
                Ok(())
            }
            _ => {
                timestamp_us
                    .checked_add(self.cooldown_us())
                    .ok_or("Arc cooldown deadline overflow")?;
                Ok(())
            }
        }
    }

    pub(crate) fn check_admission(
        &self,
        actor_index: usize,
        actor: &crate::simulation::state::ActorRuntime,
        current_time_us: u64,
    ) -> Option<crate::simulation::rejection::AdmissionFailure> {
        match self {
            Self::BlackBook(_) => {
                let state = &actor.equipment.black_book;
                let message = if let Some(pending) = state.pending_startup {
                    Some(format!(
                        "Arc: Black Tome startup is pending until {}us",
                        pending.completes_at_us
                    ))
                } else if state.arc_active_until_us > 0
                    && current_time_us < state.arc_active_until_us
                {
                    Some(format!(
                        "Arc: Black Tome summon is active until {}us",
                        state.arc_active_until_us
                    ))
                } else if state.chains_remaining > 0 {
                    Some(format!(
                        "Arc: Black Tome is sealed by {} chain(s)",
                        state.chains_remaining
                    ))
                } else {
                    None
                };
                message.map(|message| crate::simulation::rejection::AdmissionFailure {
                        reason: crate::simulation::GameAdmissionReason::Other {
                            message: message.clone(),
                        },
                        message: if state.chains_remaining > 0
                            && state.pending_startup.is_none()
                            && !(state.arc_active_until_us > 0
                                && current_time_us < state.arc_active_until_us)
                        {
                            format!(
                                "SEALED_ARC: actor {actor_index} cannot activate Arc: Black Tome while sealed ({} chains remaining)",
                                state.chains_remaining
                            )
                        } else {
                            format!("ARC_UNAVAILABLE: actor {actor_index}; {message}")
                        },
                    })
            }
            _ => None,
        }
    }

    pub(crate) fn activate(
        self,
        ctx: &mut CallbackContext<'_>,
        command_index: usize,
        actor_index: usize,
        ready_at_us: u64,
    ) -> Result<(), String> {
        match self {
            Self::SilentGarden(parameters) => crate::weapons::jingmotingyuan::arc::activate(
                ctx,
                command_index,
                actor_index,
                parameters,
                ready_at_us,
            ),
            Self::Eclipse { duration_us, .. } => {
                crate::weapons::rishi::activate_arc(ctx, actor_index, duration_us)
            }
            Self::OfficerWhisker(parameters) => crate::weapons::police_rat::arc::activate(
                ctx,
                command_index,
                actor_index,
                parameters,
                ready_at_us,
            ),
            Self::BlackBook(parameters) => crate::weapons::black_book::arc::activate(
                ctx,
                command_index,
                actor_index,
                parameters,
                ready_at_us,
            ),
        }
    }

    /// A Black Tome startup owns its own bounded interruption window, so it
    /// can be interrupted even when the character has no general interrupt
    /// model. Other Arc families return false and keep their owner policy.
    pub(crate) fn validates_interrupt(
        request: &crate::simulation::GameScenarioRequest,
        actor_index: usize,
        timestamp_us: u64,
    ) -> Result<bool, String> {
        let Some(config) = request.actors.get(actor_index) else {
            return Ok(false);
        };
        if !config
            .weapon_id
            .eq_ignore_ascii_case("weapon:fork_BlackBook")
        {
            return Ok(false);
        }
        let matched = request.actions.iter().any(|action| {
            matches!(
                action,
                crate::simulation::GameActionCommand::PlayerInput {
                    timestamp_us: start_us,
                    actor_index: owner,
                    input: crate::simulation::PlayerInput::Arc,
                    ..
                } if *owner == actor_index
                    && *start_us < timestamp_us
                    && start_us
                        .checked_add(crate::weapons::black_book::arc::STARTUP_US)
                        .is_some_and(|ready| timestamp_us < ready)
            )
        });
        if matched {
            crate::weapons::black_book::arc::validate_interrupt_deadline(timestamp_us)?;
        }
        Ok(matched)
    }

    pub(crate) fn interrupt_startup(
        ctx: &mut CallbackContext<'_>,
        actor_index: usize,
        event_index: usize,
    ) -> Result<Option<crate::simulation::GameActorInterruptReceipt>, String> {
        let recovery_before_us = ctx.state.actors[actor_index].locked_until_us;
        let Some(command_index) =
            crate::weapons::black_book::arc::interrupt_startup(ctx, actor_index)?
        else {
            return Ok(None);
        };
        let now_us = ctx.state.current_time_us;
        ctx.state.actors[actor_index].locked_until_us = now_us;
        Ok(Some(crate::simulation::GameActorInterruptReceipt {
            event_index,
            actor_index,
            time_us: now_us,
            interrupted_command_index: Some(command_index),
            cancelled_hit_deliveries: 0,
            recovery_before_us,
            recovery_after_us: now_us,
            skill_cooldown_before_us: None,
            skill_cooldown_after_us: None,
            field_cancelled: false,
        }))
    }
}
