//! Word counters, owner healing and direct debt settlement.
use crate::callbacks::ContentEvent;
use crate::runtime_api::CallbackContext;

pub(crate) fn apply(event: ContentEvent, ctx: &mut CallbackContext<'_>) -> Result<(), String> {
    match event {
        ContentEvent::BaicangWordHpConsumed {
            actor_index,
            amount,
        } => {
            if ctx.state.actors[actor_index].hp == 0 {
                return Ok(());
            }
            crate::weapons::jingmotingyuan::on_hp_consumed(ctx, actor_index, amount)
        }
        ContentEvent::BaicangFadiaCheck {
            actor_index,
            trigger,
        } => super::fadia_assist::check(ctx, actor_index, trigger),
        ContentEvent::BaicangHitstop {
            command_index: _,
            actor_index,
            duration_us,
            rate_ppm,
        } => super::clock::event(ctx, actor_index, duration_us, rate_ppm),
        ContentEvent::BaicangWordSilenceA1 { actor_index } => {
            if ctx.state.actors[actor_index].hp == 0
                || !ctx.request.actors[actor_index].awakening.has_effect(1)
            {
                return Ok(());
            }
            if let crate::characters::RuntimeState::Baicang(state) =
                &mut ctx.state.actors[actor_index].character
            {
                state.a1_silence_stacks = (state.a1_silence_stacks + 1).min(4);
            }
            Ok(())
        }
        ContentEvent::BaicangWordObjurgateA1 { actor_index } => {
            if ctx.state.actors[actor_index].hp == 0
                || !ctx.request.actors[actor_index].awakening.has_effect(1)
            {
                return Ok(());
            }
            let actor = &mut ctx.state.actors[actor_index];
            let cd_val = actor
                .ability_cooldowns
                .get("GA_Cang_Skill")
                .copied()
                .unwrap_or(0);
            if cd_val > ctx.state.current_time_us {
                if let Some(cd) = actor.ability_cooldowns.get("GA_Cang_Skill").copied() {
                    actor.ability_cooldowns.insert(
                        "GA_Cang_Skill".into(),
                        cd.saturating_sub(4_000_000).max(ctx.state.current_time_us),
                    );
                }
            } else if let crate::characters::RuntimeState::Baicang(state) = &mut actor.character {
                state.a1_objurgate_credits = (state.a1_objurgate_credits + 1).min(4);
            }
            Ok(())
        }
        ContentEvent::BaicangWordHeal {
            actor_index,
            percent,
        } => {
            if ctx.state.actors[actor_index].hp == 0 {
                return Ok(());
            }
            let actor = &ctx.state.actors[actor_index];
            let max_hp = crate::simulation::state::effective_hp_max(
                &ctx.state.buffs,
                actor.hp_max_base,
                &actor.hp_max_static_bonus,
                actor_index,
                actor.hp_max,
            )?;
            let heal = ctx.settle_healing(
                actor_index,
                actor_index,
                crate::simulation::ExactFraction::new(
                    u128::from(max_hp) * u128::from(percent),
                    100,
                )?,
            )?;
            ctx.trace.push(crate::simulation::GameEventReceipt {
                time_us: ctx.state.current_time_us,
                phase: "VitalSettlement".into(),
                event_kind: "baicang_word_heal".into(),
                active_actor: ctx.state.active_actor_index,
                hit_index: None,
                from_actor: Some(actor_index),
                to_actor: Some(actor_index),
                energy_change: None,
                energy_fanout: None,
                data: Some(crate::simulation::GameEventPayload {
                    gross: Some(heal.gross),
                    applied: Some(heal.applied),
                    overflow: Some(heal.excess),
                    hp_before: Some(heal.before_hp),
                    hp_after: Some(heal.after_hp),
                    ..Default::default()
                }),
            });
            Ok(())
        }
        ContentEvent::BaicangFatalDebtInstallment {
            actor_index,
            amount,
        } => {
            let runtime = &ctx.state.actors[actor_index];
            let max_hp = crate::simulation::state::effective_hp_max(
                &ctx.state.buffs,
                runtime.hp_max_base,
                &runtime.hp_max_static_bonus,
                actor_index,
                runtime.hp_max,
            )?;
            let actor = &mut ctx.state.actors[actor_index];
            if let crate::characters::RuntimeState::Baicang(field) = &mut actor.character {
                field.fatal_hit_debt_total = field.fatal_hit_debt_total.saturating_sub(amount);
            }
            if actor.hp > 0 {
                if actor.hp <= amount {
                    let paid = actor.hp;
                    actor.hp = 0;
                    actor.record_hp_loss(paid, max_hp);
                    ctx.trace.push(crate::simulation::GameEventReceipt {
                        time_us: ctx.state.current_time_us,
                        phase: "VitalSettlement".into(),
                        event_kind: "actor_defeated".into(),
                        active_actor: ctx.state.active_actor_index,
                        hit_index: None,
                        from_actor: None,
                        to_actor: Some(actor_index),
                        energy_change: None,
                        energy_fanout: None,
                        data: Some(crate::simulation::GameEventPayload {
                            amount: Some(amount),
                            reason: Some("fatal_debt_installment".into()),
                            ..Default::default()
                        }),
                    });
                    crate::registry::on_actor_defeated(ctx, actor_index)?;
                } else {
                    actor.hp -= amount;
                    actor.record_hp_loss(amount, max_hp);
                    ctx.trace.push(crate::simulation::GameEventReceipt {
                        time_us: ctx.state.current_time_us,
                        phase: "VitalSettlement".into(),
                        event_kind: "fatal_debt_installment".into(),
                        active_actor: ctx.state.active_actor_index,
                        hit_index: None,
                        from_actor: None,
                        to_actor: Some(actor_index),
                        energy_change: None,
                        energy_fanout: None,
                        data: Some(crate::simulation::GameEventPayload {
                            amount: Some(amount),
                            hp_after: Some(actor.hp),
                            ..Default::default()
                        }),
                    });
                }
            }
            Ok(())
        }
        ContentEvent::BaicangDomainEnd {
            actor_index,
            generation,
        } => crate::characters::baicang::skill::on_domain_end(ctx, actor_index, generation),
        ContentEvent::BaicangFieldWords {
            actor_index,
            command_index,
            generation,
            array_index,
        } => crate::characters::baicang::words::field_start(
            ctx,
            actor_index,
            command_index,
            generation,
            array_index,
        ),
        _ => Err("unexpected Baicang callback".into()),
    }
}
