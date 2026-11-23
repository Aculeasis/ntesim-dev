use crate::runtime_api::DamageContext;
use crate::simulation::ExactFraction;

#[derive(Clone, Debug, Default)]
pub(crate) struct State {
    pub(crate) grain: Option<ntesim_core::kernel::state::ExactPool>,
    pub(crate) normal_stage: usize,
    pub(crate) normal_started_us: Option<u64>,
    pub(crate) grain_inputs: std::collections::BTreeSet<(usize, super::resource::GrantKey)>,
    pub(crate) hoarding_active: bool,
    pub(crate) clock: Option<Box<super::clock::Clock>>,
    pub(crate) finance: Option<Box<super::finance::State>>,
    pub(crate) pressed: Option<Box<super::pressed::State>>,
    pub(crate) riposte_ready: bool,
    pub(crate) barren: super::barren::State,
    pub(crate) out_of_combat: bool,
    pub(crate) battle_entries: std::collections::BTreeSet<usize>,
}

pub(crate) fn damage_bonus(
    ctx: &DamageContext<'_>,
    dmg_bonus: crate::runtime_api::HitModifiers,
) -> Result<crate::runtime_api::HitModifiers, String> {
    super::pressed::damage(ctx, dmg_bonus)
}
pub(crate) fn crit_bonus(ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
    if ctx.config.character_id != "character:1073" {
        return ExactFraction::new(0, 1);
    }
    ExactFraction::new(0, 1)
}

pub(crate) fn crit_rate_bonus(ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
    if ctx.config.character_id != "character:1073" {
        return ExactFraction::new(0, 1);
    }
    ExactFraction::new(0, 1)
}

pub(crate) fn defense_ignore(ctx: &DamageContext<'_>) -> Result<ExactFraction, String> {
    if ctx.config.character_id != "character:1073" {
        return ExactFraction::new(0, 1);
    }
    ExactFraction::new(0, 1)
}
