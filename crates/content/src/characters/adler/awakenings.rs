use crate::simulation::ExactFraction as F;

pub(crate) const KARMA_CAPACITY: u32 = 20;

pub(crate) fn initial_karma(initial: Option<crate::formula::Ratio>) -> Result<u32, String> {
    let value = initial.map(F::from_ratio).transpose()?.unwrap_or_default();
    if value.denominator != 1 || value.numerator > u128::from(KARMA_CAPACITY) {
        return Err("Adler Karma must be integral in 0..20.".into());
    }
    u32::try_from(value.numerator).map_err(|_| "Adler initial Karma overflow".into())
}

#[derive(Clone, Debug, Default)]
pub(crate) struct State {
    pub(crate) normal_stage: usize,
    pub(crate) normal_started_us: Option<u64>,
    pub(crate) karma_stacks: u32,
    pub(crate) skill_generation: u64,
    pub(crate) skill_deadlines: std::collections::BTreeMap<usize, u64>,
    pub(crate) skill_bonus_by_command: std::collections::BTreeMap<usize, u32>,
    pub(crate) riposte_ready: bool,
}
