use crate::runtime_api::DamageContext;
use crate::runtime_api::{HitModifiers, RecipientContext};
use crate::simulation::ExactFraction;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WordType {
    Silence,
    Objurgate,
    Bless,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct StoredWord {
    pub(crate) word_type: WordType,
    pub(crate) target_index: usize,
    pub(crate) born_us: u64,
    pub(crate) expiry_us: u64,
    pub(crate) generation: u64,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct State {
    pub(crate) domain_start_us: u64,
    pub(crate) domain_until_us: u64,
    pub(crate) power_word_stacks: u32,
    pub(crate) power_word_until_us: u64,
    pub(crate) normal_stage: usize,
    pub(crate) normal_started_us: Option<u64>,
    pub(crate) domain_generation: u64,

    // Phase 2 additions
    pub(crate) normal_word_counter: u32,
    pub(crate) word_generation_counter: u64,
    pub(crate) stored_words: Vec<StoredWord>,
    pub(crate) a1_silence_stacks: u32,
    pub(crate) a1_objurgate_credits: u32,
    pub(crate) a2_stack_expiries: Vec<u64>,
    pub(crate) fatal_hit_armed: bool,
    pub(crate) fatal_hit_immune_until_us: u64,
    pub(crate) dodge_immune_until_us: u64,
    pub(crate) heavy_dodge_until_us: u64,
    pub(crate) cancel_window: Option<super::cancel::Window>,
    pub(crate) clock: Box<super::clock::State>,
    pub(crate) skia_assist_until_us: u64,
    pub(crate) entry_jump_pending: bool,
    pub(crate) fatal_hit_debt_total: u64,
    pub(crate) derived_skill_stage: u32,
    pub(crate) derived_skill_stage_expires_at_us: u64,
    pub(crate) riposte_ready: bool,
    pub(crate) skill_stacks_by_command: std::collections::BTreeMap<usize, u32>,
    pub(crate) connected_commands: std::collections::BTreeSet<usize>,
    pub(crate) power_words_mode: bool,
}

pub(crate) fn source_atk_percent(
    config: &crate::simulation::GameActorConfig,
    actor: &crate::simulation::state::ActorRuntime,
    now: u64,
    phase2: bool,
) -> Result<ExactFraction, String> {
    let crate::characters::RuntimeState::Baicang(state) = &actor.character else {
        return ExactFraction::new(0, 1);
    };
    let base = if phase2 && super::etd::has_passive(config) {
        ExactFraction::new(1, 5)?
    } else {
        ExactFraction::new(0, 1)?
    };
    let stacks = if !config.awakening.has_effect(2) {
        0
    } else if phase2 {
        state
            .a2_stack_expiries
            .iter()
            .filter(|&&expiry| expiry > now)
            .count() as u128
    } else if state.power_word_until_us > now {
        u128::from(state.power_word_stacks)
    } else {
        0
    };
    base.add(ExactFraction::new(stacks * 3, 50)?)
}

fn apply_phase2_skill_bonus(
    ctx: &DamageContext<'_>,
    state: &State,
    is_skill: bool,
    mut dmg_bonus: HitModifiers,
) -> Result<HitModifiers, String> {
    if is_skill && ctx.config.awakening.has_effect(1) {
        let stacks = u128::from(
            *state
                .skill_stacks_by_command
                .get(&ctx.command_index)
                .unwrap_or(&0),
        );
        let bonus = ExactFraction::new(stacks * 3, 10)?;
        dmg_bonus = dmg_bonus.add_damage(bonus)?;
    }

    Ok(dmg_bonus)
}

fn apply_legacy_bonus(
    ctx: &DamageContext<'_>,
    has_domain: bool,
    is_skill: bool,
    mut dmg_bonus: HitModifiers,
) -> Result<HitModifiers, String> {
    if ctx.config.awakening.has_effect(1) && has_domain && is_skill {
        dmg_bonus = dmg_bonus.add_damage(ExactFraction::new(3, 10)?)?;
    }

    Ok(dmg_bonus)
}

pub(crate) fn damage_bonus(
    ctx: &DamageContext<'_>,
    dmg_bonus: HitModifiers,
) -> Result<HitModifiers, String> {
    let actor_config = ctx.config;

    if actor_config.character_id != "character:1023" {
        return Ok(dmg_bonus);
    }

    let has_domain = matches!(
        &ctx.actor.character,
        crate::characters::RuntimeState::Baicang(state) if state.domain_start_us <= ctx.time_us && state.domain_until_us > ctx.time_us
    );

    let is_skill = ctx
        .action_metadata
        .categories
        .contains(&crate::simulation::metadata::ActionCategory::Skill);

    if let crate::characters::RuntimeState::Baicang(state) = &ctx.actor.character {
        let bonus = dmg_bonus.add_atk_percent(source_atk_percent(
            actor_config,
            ctx.actor,
            ctx.time_us,
            state.power_words_mode,
        )?)?;
        if state.power_words_mode {
            apply_phase2_skill_bonus(ctx, state, is_skill, bonus)
        } else {
            apply_legacy_bonus(ctx, has_domain, is_skill, bonus)
        }
    } else {
        Ok(dmg_bonus)
    }
}

pub(crate) fn crit_bonus(ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
    if ctx.config.character_id != "character:1023" {
        return ExactFraction::new(0, 1);
    }
    ExactFraction::new(0, 1)
}

/// Individual node Effect6: per 1023AwakenEffect.json / DT_CangEffectFigure.json Buff_Cang023_Level6_CritAdd = 0.30.
pub(crate) fn crit_rate_bonus(ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
    if ctx.config.character_id != "character:1023" {
        return ExactFraction::new(0, 1);
    }

    let has_domain = matches!(
        &ctx.actor.character,
        crate::characters::RuntimeState::Baicang(state) if state.domain_start_us <= ctx.time_us && state.domain_until_us > ctx.time_us
    );

    if ctx.config.awakening.has_effect(6) && has_domain {
        ExactFraction::new(3, 10)
    } else {
        ExactFraction::new(0, 1)
    }
}

pub(crate) fn defense_ignore(ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
    if ctx.config.character_id != "character:1023" {
        return ExactFraction::new(0, 1);
    }
    ExactFraction::new(0, 1)
}

pub(crate) fn recipient_modifiers(
    ctx: &RecipientContext<'_>,
    mut value: HitModifiers,
) -> Result<HitModifiers, String> {
    let active = matches!(
        &ctx.source_actor.character,
        crate::characters::RuntimeState::Baicang(state)
            if ctx.source_actor.hp > 0 && state.domain_start_us <= ctx.time_us && state.domain_until_us > ctx.time_us
    );
    // Milestone resonance_6 (Buff_Cang023_Level6_Extra_TeamAtkUP = 0.20):
    // activates at 6 total selected awakenings (distinct from individual node Effect6).
    if ctx.source_config.awakening.has_resonance_6() && active {
        value = value.add_atk_percent(ExactFraction::new(1, 5)?)?;
    }
    Ok(value)
}
