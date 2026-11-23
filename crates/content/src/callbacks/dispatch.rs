use super::events::ContentEvent;
use super::handlers::{apply_gear_callback, apply_periodic_callback, apply_vitals_callback};
use crate::characters::hotori;
use crate::runtime_api::CallbackContext;

fn apply_jiuyuan(event: ContentEvent, ctx: &mut CallbackContext<'_>) -> Result<(), String> {
    match event {
        ContentEvent::JiuyuanPactApply(application) => {
            crate::characters::jiuyuan::pact::apply(ctx, application)
        }
        ContentEvent::JiuyuanPactTick {
            actor_index,
            target_index,
            planted_us,
        } => crate::characters::jiuyuan::pact::tick(ctx, actor_index, target_index, planted_us),
        ContentEvent::JiuyuanPactClose {
            actor_index,
            target_index,
            generation,
        } => crate::characters::jiuyuan::pact::close(ctx, actor_index, target_index, generation),
        ContentEvent::JiuyuanForceGrow { actor_index } => {
            crate::characters::jiuyuan::pact::force_grow(ctx, actor_index)
        }
        _ => unreachable!(),
    }
}

fn apply_iroi(event: ContentEvent, ctx: &mut CallbackContext<'_>) -> Result<(), String> {
    match event {
        ContentEvent::IroiSkillBuffGrant {
            source_actor_index,
            atk_add,
            def_ignore,
            a5_atk_add,
            duration_us,
        } => crate::characters::iroi::callback::apply(
            ctx,
            source_actor_index,
            atk_add,
            def_ignore,
            a5_atk_add,
            duration_us,
        ),
        ContentEvent::IroiA2Assist { actor_index } => {
            crate::characters::iroi::callback::apply_a2_assist(ctx, actor_index)
        }
        ContentEvent::IroiA6ReviveGrant { actor_index } => {
            crate::characters::iroi::callback::apply_a6_revive(ctx, actor_index)
        }
        ContentEvent::IroiDreamRelease {
            actor_index,
            generation,
            ..
        } => crate::characters::iroi::dream::release(ctx, actor_index, generation),
        ContentEvent::IroiDreamTick {
            actor_index,
            generation,
            tick_index,
            coordinate,
        } => crate::characters::iroi::dream::on_tick(
            ctx,
            actor_index,
            generation,
            tick_index,
            coordinate,
        ),
        ContentEvent::IroiDreamEnd {
            actor_index,
            generation,
        } => crate::characters::iroi::dream::on_dream_end(ctx, actor_index, generation),
        ContentEvent::IroiBudCopyCheck { bud_id, snapshot } => {
            crate::characters::iroi::bud_copy::on_bud_copy_check(ctx, bud_id, *snapshot)
        }
        _ => unreachable!(),
    }
}

pub(crate) fn apply(event: ContentEvent, ctx: &mut CallbackContext<'_>) -> Result<(), String> {
    match event {
        ContentEvent::ZankouCombat(event) => {
            crate::characters::zankou::combat_passives::event(ctx, event)
        }
        ContentEvent::ZankouBurst(event) => crate::characters::zankou::burst::event(ctx, event),
        ContentEvent::ZankouForms(event) => crate::characters::zankou::forms::event(ctx, event),
        ContentEvent::IroiClock(event) => crate::characters::iroi::clock::event(ctx, event),
        ContentEvent::MintClock(event) => crate::characters::mint::clock::event(ctx, event),
        ContentEvent::NanallyClock(event) => crate::characters::nanally::clock::notify(ctx, event),
        ContentEvent::HathorEmergency(event) => {
            crate::characters::hathor::burst::expire(ctx, event)
        }
        ContentEvent::HathorGaze(event) => crate::characters::hathor::gaze::event(ctx, event),
        ContentEvent::HathorAir(event) => crate::characters::hathor::air::event(ctx, event),
        ContentEvent::HathorBranch(event) => crate::characters::hathor::branch::event(ctx, event),
        ContentEvent::HathorClock(event) => crate::characters::hathor::clock::event(ctx, event),
        ContentEvent::HathorKill(event) => crate::characters::hathor::power::kill(ctx, event),
        ContentEvent::HotoriTimepiece(event) => hotori::timepiece::event(ctx, event),
        ContentEvent::HotoriKatana(event) => hotori::katana::event(ctx, event),
        ContentEvent::HathorRiposte(event) => {
            crate::characters::hathor::secondary::reward(ctx, event)
        }
        ContentEvent::HanielClock(event) => crate::characters::haniel::clock::event(ctx, event),
        ContentEvent::HanielCannon(event) => crate::characters::haniel::cannon::event(ctx, event),
        ContentEvent::Fadia(event) => crate::characters::fadia::clock::event(ctx, event),
        ContentEvent::FadiaEffects(event) => crate::characters::fadia::effects::event(ctx, event),
        ContentEvent::Edgar(event) => crate::characters::edgar::clock::event(ctx, event),
        ContentEvent::SkiaEntryState {
            actor_index,
            generation,
            boundary,
        } => crate::characters::skia::entry_state::event(ctx, actor_index, generation, boundary),
        ContentEvent::JiuyuanPactApply(_)
        | ContentEvent::JiuyuanPactTick { .. }
        | ContentEvent::JiuyuanPactClose { .. }
        | ContentEvent::JiuyuanForceGrow { .. } => apply_jiuyuan(event, ctx),
        ContentEvent::Haniel(event) => crate::characters::haniel::hootie::apply(event, ctx),
        ContentEvent::TimedBuffGrant(grant) => grant.apply(ctx),
        ContentEvent::ZankouVileAshApply(application) => {
            crate::characters::zankou::vile_ash::apply(ctx, application)
        }
        ContentEvent::ZankouSpread {
            source_actor,
            from_target,
        } => crate::characters::zankou::spread::apply(ctx, source_actor, from_target),
        ContentEvent::LacrimosaNightmareApply(application) => {
            crate::characters::lacrimosa::dot::apply(ctx, application)
        }
        event @ (ContentEvent::MotorCandyTick { .. }
        | ContentEvent::QuietManorMentalStack(_)
        | ContentEvent::HpSuitMaxHpStack(_)
        | ContentEvent::CrimsonAtkStack(_)
        | ContentEvent::RagingFlamesUltimateGrant { .. }
        | ContentEvent::RagingFlamesSkillStack { .. }
        | ContentEvent::LunarPhaseProcGrant { .. }
        | ContentEvent::DemonBladeCritStack { .. }
        | ContentEvent::MoonCritStack { .. }
        | ContentEvent::GoldRecordUltCritStack { .. }
        | ContentEvent::SpiderKnowledgeStack { .. }
        | ContentEvent::SpiderTeamAtkGrant { .. }
        | ContentEvent::BitGameOffFieldStack { .. }
        | ContentEvent::BitGameOnFieldStack { .. }
        | ContentEvent::FarSideMoonGrant { .. }
        | ContentEvent::FarSideMoonSkillStack { .. }
        | ContentEvent::KnightCandyCritStack { .. }
        | ContentEvent::OraoraBasicAttackStack { .. }
        | ContentEvent::MindRoyaleEnergyGrant { .. }
        | ContentEvent::FirefliesCritStack { .. }
        | ContentEvent::PoliceRatWhiskerTick { .. }
        | ContentEvent::BlackBookDesignateTick { .. }
        | ContentEvent::BlackBookArcStartupComplete { .. }
        | ContentEvent::RishiEclipseField { .. }
        | ContentEvent::RishiEclipseEnergy { .. }) => apply_gear_callback(event, ctx),
        event @ (ContentEvent::BeHappyHeal { .. }
        | ContentEvent::WhaleHeal { .. }
        | ContentEvent::IroiSkillHeal { .. }
        | ContentEvent::IroiBurstHeal { .. }
        | ContentEvent::NakupedaProc { .. }
        | ContentEvent::AdlerBlessingShield { .. }) => apply_vitals_callback(event, ctx),
        event @ (ContentEvent::EdgarHealTick { .. }
        | ContentEvent::BaicangDomainTick { .. }
        | ContentEvent::AdlerSkillTick(_)) => apply_periodic_callback(event, ctx),
        event @ (ContentEvent::BaicangFadiaCheck { .. }
        | ContentEvent::BaicangHitstop { .. }
        | ContentEvent::BaicangWordSilenceA1 { .. }
        | ContentEvent::BaicangWordObjurgateA1 { .. }
        | ContentEvent::BaicangWordHpConsumed { .. }
        | ContentEvent::BaicangWordHeal { .. }
        | ContentEvent::BaicangFatalDebtInstallment { .. }
        | ContentEvent::BaicangDomainEnd { .. }
        | ContentEvent::BaicangFieldWords { .. }) => {
            crate::characters::baicang::events::apply(event, ctx)
        }
        ContentEvent::Blackbird(event) => crate::characters::blackbird::events::apply(ctx, event),
        ContentEvent::Chaos(event) => crate::characters::chaos::events::apply(ctx, event),
        ContentEvent::Chiz(event) => crate::characters::chiz::events::apply(ctx, event),
        ContentEvent::Daffodil(event) => crate::characters::daffodil::clock::event(ctx, event),
        ContentEvent::BuffLifecycleExpiry => Ok(()),
        ContentEvent::BlossomBudExpiry { bud_id, is_copy } => {
            crate::simulation::reactions::expire_bud(ctx, bud_id, is_copy)
        }
        ContentEvent::SakiriAtkBuffGrant {
            a4,
            source_actor_index,
            atk_add,
            duration_us,
        } => crate::characters::sakiri::callback::apply(
            ctx,
            a4,
            source_actor_index,
            atk_add,
            duration_us,
        ),
        ContentEvent::IroiSkillBuffGrant { .. }
        | ContentEvent::IroiA2Assist { .. }
        | ContentEvent::IroiA6ReviveGrant { .. }
        | ContentEvent::IroiDreamRelease { .. }
        | ContentEvent::IroiDreamTick { .. }
        | ContentEvent::IroiDreamEnd { .. }
        | ContentEvent::IroiBudCopyCheck { .. } => apply_iroi(event, ctx),
        ContentEvent::MintA6EnergyGrant {
            actor_index,
            amount,
        } => crate::characters::mint::callback::apply(ctx, actor_index, amount),
        ContentEvent::ZeroA2EnergyGrant {
            actor_index,
            amount,
        } => crate::characters::zero::callback::apply_a2_energy(ctx, actor_index, amount),
        ContentEvent::ZeroA5AtkBuffGrant {
            actor_index,
            duration_us,
        } => crate::characters::zero::callback::apply_a5_buff(ctx, actor_index, duration_us),
        ContentEvent::NanallyA1EnergyGrant { actor_index } => {
            crate::characters::nanally::callback::apply_a1_energy(ctx, actor_index)
        }
        ContentEvent::NanallyUnderboss(attack) => {
            crate::characters::nanally::underboss::deliver(ctx, *attack)
        }
        ContentEvent::AdlerA1KarmaGrant { actor_index } => {
            crate::characters::adler::callback::apply_a1_karma(ctx, actor_index)
        }
        ContentEvent::AdlerA5EnergyGrant { actor_index } => {
            crate::characters::adler::callback::apply_a5_energy(ctx, actor_index)
        }
        ContentEvent::NanallyA5AtkBuff {
            actor_index,
            duration_us,
        } => crate::characters::nanally::callback::apply_a5_buff(ctx, actor_index, duration_us),
        ContentEvent::AureliaHarmonicsHit {
            actor_index,
            target_index,
            command_key,
        } => crate::characters::aurelia::harmonics::deliver_harmonics_hit(
            ctx,
            actor_index,
            target_index,
            command_key,
        ),
        ContentEvent::AureliaSkillFinish { actor_index } => {
            crate::characters::aurelia::skill::handle_skill_finish(ctx, actor_index)
        }
        ContentEvent::LinkoXiaozhenPulse {
            actor_index,
            generation,
            pulse_index,
        } => crate::characters::linko::skill::on_pulse(ctx, actor_index, generation, pulse_index),
    }
}
