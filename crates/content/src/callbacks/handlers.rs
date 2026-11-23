use super::events::ContentEvent;
use crate::runtime_api::CallbackContext;

pub(super) fn apply_gear_callback(
    event: ContentEvent,
    ctx: &mut CallbackContext<'_>,
) -> Result<(), String> {
    match event {
        ContentEvent::MotorCandyTick {
            actor_index,
            generation,
        } => crate::weapons::motor_candy::tick(ctx, actor_index, generation),
        ContentEvent::QuietManorMentalStack(grant) => crate::sets::quiet_manor::apply(ctx, grant),
        ContentEvent::HpSuitMaxHpStack(grant) => crate::sets::hp_suit::apply(ctx, grant),
        ContentEvent::CrimsonAtkStack(grant) => crate::sets::crimson::apply(ctx, grant),
        ContentEvent::RagingFlamesUltimateGrant {
            actor_index,
            redirect_ultimate_bonus,
            duration_us,
        } => crate::weapons::raging_flames::apply_ultimate(
            ctx,
            actor_index,
            redirect_ultimate_bonus,
            duration_us,
        ),
        ContentEvent::RagingFlamesSkillStack {
            actor_index,
            redirect_skill_stack_bonus,
        } => crate::weapons::raging_flames::apply_skill_stack(
            ctx,
            actor_index,
            redirect_skill_stack_bonus,
        ),
        ContentEvent::LunarPhaseProcGrant {
            actor_index,
            cosmos_bonus,
            def_ignore,
            duration_us,
        } => crate::weapons::lunarphase::apply(
            ctx,
            actor_index,
            cosmos_bonus,
            def_ignore,
            duration_us,
        ),
        ContentEvent::DemonBladeCritStack {
            actor_index,
            stack_amount,
            duration_us,
        } => crate::weapons::demon_blade::apply(ctx, actor_index, stack_amount, duration_us),
        ContentEvent::MoonCritStack {
            actor_index,
            stack_amount,
            duration_us,
        } => crate::weapons::moon::apply(ctx, actor_index, stack_amount, duration_us),
        ContentEvent::GoldRecordUltCritStack {
            actor_index,
            stack_amount,
            duration_us,
        } => crate::weapons::gold_record::apply(ctx, actor_index, stack_amount, duration_us),
        ContentEvent::SpiderKnowledgeStack { actor_index } => {
            crate::weapons::spider::apply_knowledge_stack(ctx, actor_index)
        }
        ContentEvent::SpiderTeamAtkGrant {
            actor_index,
            team_atk_bonus,
            duration_us,
        } => crate::weapons::spider::apply_team_atk(ctx, actor_index, team_atk_bonus, duration_us),
        ContentEvent::BitGameOffFieldStack {
            actor_index,
            off_field_atk_per_stack,
        } => crate::weapons::bit_game::apply_off_field_stack(
            ctx,
            actor_index,
            off_field_atk_per_stack,
        ),
        ContentEvent::BitGameOnFieldStack {
            actor_index,
            on_field_psyche_per_stack,
        } => crate::weapons::bit_game::apply_on_field_stack(
            ctx,
            actor_index,
            on_field_psyche_per_stack,
        ),
        ContentEvent::FarSideMoonGrant {
            actor_index,
            basic_attack_damage_bonus,
            lakshana_damage_bonus,
            duration_us,
        } => crate::weapons::far_side_moon::apply(
            ctx,
            actor_index,
            basic_attack_damage_bonus,
            lakshana_damage_bonus,
            duration_us,
        ),
        ContentEvent::KnightCandyCritStack {
            actor_index,
            stack_amount,
            duration_us,
        } => crate::weapons::knight_candy::apply(ctx, actor_index, stack_amount, duration_us),
        ContentEvent::OraoraBasicAttackStack {
            actor_index,
            stack_amount,
            duration_us,
        } => crate::weapons::oraora::apply(ctx, actor_index, stack_amount, duration_us),
        ContentEvent::MindRoyaleEnergyGrant {
            actor_index,
            amount,
        } => crate::weapons::mind_royale::callback::apply(ctx, actor_index, amount),
        ContentEvent::FirefliesCritStack {
            actor_index,
            stack_amount,
            max_stacks,
            duration_us,
        } => crate::sets::fireflies::callback::apply(
            ctx,
            actor_index,
            stack_amount,
            max_stacks,
            duration_us,
        ),
        ContentEvent::PoliceRatWhiskerTick {
            actor_index,
            target_index,
            generation,
            end_time_us,
            interval_us,
            damage_ratio,
        } => crate::weapons::police_rat::arc::whisker_tick(
            ctx,
            actor_index,
            target_index,
            generation,
            end_time_us,
            interval_us,
            damage_ratio,
        ),
        ContentEvent::BlackBookDesignateTick {
            actor_index,
            generation,
            end_time_us,
            interval_us,
        } => crate::weapons::black_book::arc::designate_tick(
            ctx,
            actor_index,
            generation,
            end_time_us,
            interval_us,
        ),
        ContentEvent::BlackBookArcStartupComplete {
            actor_index,
            command_index,
            generation,
            cooldown_until_us,
            parameters,
        } => crate::weapons::black_book::arc::startup_complete(
            ctx,
            actor_index,
            command_index,
            generation,
            cooldown_until_us,
            parameters,
        ),
        ContentEvent::RishiEclipseField { actor_index } => {
            crate::weapons::rishi::apply_field(ctx, actor_index)
        }
        ContentEvent::RishiEclipseEnergy {
            actor_index,
            amount,
        } => crate::weapons::rishi::apply(ctx, actor_index, amount),
        _ => Err("unsupported callback routed to gear settlement".into()),
    }
}

pub(super) fn apply_periodic_callback(
    event: ContentEvent,
    ctx: &mut CallbackContext<'_>,
) -> Result<(), String> {
    match event {
        ContentEvent::EdgarHealTick {
            actor_index,
            generation,
            end_time_us,
            interval_us,
            flat_heal,
            max_hp_ratio,
        } => crate::characters::edgar::skill::heal_tick(
            ctx,
            actor_index,
            generation,
            end_time_us,
            interval_us,
            flat_heal,
            max_hp_ratio,
        ),
        ContentEvent::BaicangDomainTick {
            actor_index,
            target_index,
            command_index,
            generation,
            end_time_us,
            interval_us,
            ability_id,
            damage_effect_id,
            atk_rate,
            scaling_stat,
            metadata,
        } => crate::characters::baicang::skill::domain_tick(
            ctx,
            actor_index,
            target_index,
            command_index,
            generation,
            end_time_us,
            interval_us,
            ability_id,
            damage_effect_id,
            atk_rate,
            scaling_stat,
            *metadata,
        ),
        ContentEvent::AdlerSkillTick(tick) => {
            crate::characters::adler::skill::skill_tick(ctx, tick)
        }
        _ => Err("unexpected periodic content callback".into()),
    }
}

pub(super) fn apply_vitals_callback(
    event: ContentEvent,
    ctx: &mut CallbackContext<'_>,
) -> Result<(), String> {
    match event {
        ContentEvent::BeHappyHeal {
            actor_index,
            target_index,
            base_heal,
        } => crate::weapons::be_happy::apply(ctx, actor_index, target_index, base_heal),
        ContentEvent::WhaleHeal {
            actor_index,
            target_index,
            base_heal,
        } => crate::weapons::whale::apply(ctx, actor_index, target_index, base_heal),
        ContentEvent::NakupedaProc {
            actor_index,
            outcome_index,
            recipients,
            base_amounts,
            shield_duration_us,
        } => crate::weapons::nakupeda::apply(
            ctx,
            actor_index,
            outcome_index,
            &recipients,
            &base_amounts,
            shield_duration_us,
        ),
        ContentEvent::AdlerBlessingShield {
            granter_index,
            recipient_index,
            amount,
            duration_us,
        } => crate::characters::adler::skill::apply_blessing_shield(
            ctx,
            granter_index,
            recipient_index,
            amount,
            duration_us,
        ),
        ContentEvent::IroiSkillHeal {
            source_actor_index,
            recipient_actor_index,
            atk_ratio,
        } => crate::characters::iroi::callback::apply_team_heal(
            ctx,
            source_actor_index,
            recipient_actor_index,
            atk_ratio,
            "iroi_skill_heal",
            "Skill",
        ),
        ContentEvent::IroiBurstHeal {
            command_index: _,
            source_actor_index,
            recipient_actor_index,
            atk_ratio,
        } => crate::characters::iroi::callback::apply_team_heal(
            ctx,
            source_actor_index,
            recipient_actor_index,
            atk_ratio,
            "iroi_burst_heal",
            "Burst",
        ),
        _ => Err("unsupported callback routed to vital settlement".into()),
    }
}
