pub(crate) mod arc;
pub(crate) mod break_intensity;
mod buff_tracking;
mod charge_efficiency;
mod config;
mod dispatch;
pub(crate) mod source_stats;
mod state;
pub(crate) mod validation;

pub(crate) use buff_tracking::{
    active_stacks, post_expire_gear_buffs, pre_expire_gear_buffs, stacks,
};
pub(crate) use charge_efficiency::validate_multiplier as validate_charge_multiplier;
pub use config::GearProcConfig;
pub(crate) use dispatch::{
    CAST_BUFF_HYPOTHESIS_ID, QTE_CAST_BUFF_HYPOTHESIS_ID, SKILL_CAST_BUFF_HYPOTHESIS_ID,
    crit_rate_bonus, damage_bonus, needs_successful_skill_finish, observe_team_admission,
    on_broken_target_hit, on_cast, on_control, on_damage_received, on_hit, on_hp_loss_hit,
    on_reaction_participation, on_research_hit, on_successful_skill_finish, on_switch,
    on_target_break_damage, on_target_defeated, on_team_hit, schedule_cast_buff,
    validate_hit_delivery,
};
pub(crate) use state::{
    RuntimeState, initialize, initialize_runtime, proc_rng_receipts, proc_selection_receipts,
    validate_rng_selectors,
};
pub(crate) use validation::{validate_action_deadlines, validate_config};
