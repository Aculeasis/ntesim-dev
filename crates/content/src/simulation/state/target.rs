use crate::formula::Ratio;
use crate::simulation::ExactFraction;

#[derive(Clone, Debug)]
pub(crate) struct TargetRuntime {
    pub(crate) attack: Option<ExactFraction>,
    pub(crate) original_attack: Option<u64>,
    pub(crate) control_immune: bool,
    pub(crate) base_defense: Option<u64>,
    pub(crate) is_boss: bool,
    pub(crate) hp_current: u64,
    pub(crate) hp_max: u64,
    pub(crate) defense: u64,
    pub(crate) resistance: Ratio,
    pub(crate) airborne_until_us: u64,
    pub(crate) break_state: Option<TargetBreakRuntime>,
}

#[derive(Clone, Debug)]
pub(crate) struct TargetBreakRuntime {
    pub(crate) capacity: ExactFraction,
    pub(crate) original_capacity: ExactFraction,
    pub(crate) incoming_efficiency_multiplier: ExactFraction,
    pub(crate) gauge: ExactFraction,
    pub(crate) broken_duration_us: u64,
    pub(crate) broken_until_us: Option<u64>,
    pub(crate) generation: u64,
    pub(crate) broken_by_actor: Option<usize>,
    pub(crate) rose_extension_used: bool,
}
