use crate::runtime_api::CallbackContext;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Event {
    ExtraCaptureEnd {
        actor_index: usize,
        group: String,
    },
    SkillHeal {
        actor_index: usize,
        command_index: usize,
    },
    CombatLeaveGrant {
        actor_index: usize,
        generation: u64,
    },
    BurstBuff {
        actor_index: usize,
        command_index: usize,
        progress_us: u64,
    },
    FormDeadline {
        actor_index: usize,
        generation: u64,
    },
    Bird {
        actor_index: usize,
        generation: u64,
        key: usize,
        target_index: usize,
    },
    SupportMalice {
        actor_index: usize,
        command_index: usize,
        progress_us: u64,
    },
    SkillPermission {
        actor_index: usize,
        command_index: usize,
        progress_us: u64,
    },
}
impl Event {
    pub(crate) fn clock_anchor(&self, owner: usize, command: usize) -> Option<u64> {
        match self {
            Self::BurstBuff {
                actor_index,
                command_index,
                progress_us,
            }
            | Self::SupportMalice {
                actor_index,
                command_index,
                progress_us,
            }
            | Self::SkillPermission {
                actor_index,
                command_index,
                progress_us,
            } if *actor_index == owner && *command_index == command => Some(*progress_us),
            _ => None,
        }
    }
}
pub(crate) fn apply(ctx: &mut CallbackContext<'_>, event: Event) -> Result<(), String> {
    match event {
        Event::ExtraCaptureEnd { actor_index, group } => {
            ctx.state.actors[actor_index]
                .character
                .blackbird_mut()?
                .malice_extras
                .remove(&group);
            Ok(())
        }
        Event::SkillHeal {
            actor_index,
            command_index,
        } => {
            let base = crate::simulation::ExactFraction::new(
                u128::from(ctx.request.actors[actor_index].static_stats.base_atk) * 4,
                1,
            )?;
            let heal = ctx.settle_healing(actor_index, actor_index, base)?;
            ctx.state.origins.register(
                ctx.request,
                crate::simulation::GameAutonomousEventReceipt {
                    actor_index,
                    active_actor_index: ctx.state.active_actor_index,
                    ability_id: "GA_BlackBird_Evolve_Skill".into(),
                    time_us: ctx.state.current_time_us,
                    trigger: crate::simulation::GameCausalTrigger::Action { command_index },
                    recipient_healing: Some(crate::simulation::GameAutonomousHealing {
                        recipient_actor_index: actor_index,
                        base,
                        gross: heal.gross,
                        applied: heal.applied,
                        excess: heal.excess,
                        hp_before: heal.before_hp,
                        hp_after: heal.after_hp,
                    }),
                },
            )?;
            Ok(())
        }
        Event::CombatLeaveGrant {
            actor_index,
            generation,
        } => super::combat::leave_grant(ctx, actor_index, generation),
        Event::SkillPermission { actor_index, .. } => {
            let actor = &mut ctx.state.actors[actor_index];
            if actor.hp == 0 || !super::form::active(actor) {
                return Ok(());
            }
            let state = actor.character.blackbird_mut()?;
            state.skill_permission_until_us = Some(
                ctx.state
                    .current_time_us
                    .checked_add(30_000_000)
                    .ok_or("permission deadline overflow")?,
            );
            ctx.trace.push(crate::simulation::GameEventReceipt {
                time_us: ctx.state.current_time_us,
                phase: "BuffTickOrExpiry".into(),
                event_kind: "blackbird_skill_permission".into(),
                active_actor: ctx.state.active_actor_index,
                hit_index: None,
                from_actor: None,
                to_actor: Some(actor_index),
                energy_change: None,
                energy_fanout: None,
                data: Some(crate::simulation::GameEventPayload {
                    duration_us: Some(30_000_000),
                    ..Default::default()
                }),
            });
            Ok(())
        }
        Event::FormDeadline {
            actor_index,
            generation,
        } => super::form::expire(ctx, actor_index, generation),
        Event::BurstBuff { actor_index, .. } => {
            if ctx.state.actors[actor_index].hp == 0 {
                return Ok(());
            }
            super::buffs::burst(
                &mut ctx.state.buffs,
                &ctx.request.actors[actor_index],
                actor_index,
                ctx.state.mode_clock.relative_us(),
            )
        }
        Event::Bird {
            actor_index,
            generation,
            key,
            target_index,
        } => super::bird::deliver(ctx, actor_index, generation, key, target_index),
        Event::SupportMalice {
            actor_index,
            command_index,
            ..
        } => {
            let actor = &mut ctx.state.actors[actor_index];
            if actor.hp == 0 || super::form::active(actor) {
                return Ok(());
            }
            let state = actor.character.blackbird_mut()?;
            if state.qte_malice_commands.contains(&command_index) {
                return Ok(());
            }
            let pool = state
                .malice
                .as_mut()
                .ok_or("Blackbird QTE marker has no exact Malice pool")?;
            pool.grant(crate::simulation::ExactFraction::new(20, 1)?)?;
            state.qte_malice_commands.insert(command_index);
            ctx.trace.push(crate::simulation::GameEventReceipt {
                time_us: ctx.state.current_time_us,
                phase: "BuffTickOrExpiry".into(),
                event_kind: "blackbird_qte_malice".into(),
                active_actor: ctx.state.active_actor_index,
                hit_index: None,
                from_actor: None,
                to_actor: Some(actor_index),
                energy_change: None,
                energy_fanout: None,
                data: Some(crate::simulation::GameEventPayload {
                    command_index: Some(command_index),
                    exact_amount: Some(crate::simulation::ExactFraction::new(20, 1)?),
                    ..Default::default()
                }),
            });
            Ok(())
        }
    }
}
