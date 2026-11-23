use super::vile_ash::VileAshTrain;
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Form {
    #[default]
    Reality,
    Illusion,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct State {
    pub(crate) normal_stage: usize,
    pub(crate) normal_started_us: Option<u64>,
    pub(crate) form: Form,
    pub(crate) form_until_us: u64,
    pub(crate) form_generation: u64,
    pub(crate) clock: Option<super::clock::Clock>,
    pub(crate) pending_skill: Option<super::forms::ActiveSkill>,
    pub(crate) bloodfeast_ready: bool,
    pub(crate) riposte_ready: bool,
    pub(crate) airborne: bool,
    pub(crate) armor_pressure_halves: Option<crate::simulation::ExactFraction>,
    pub(crate) healing_next_us: Option<u64>,
    pub(crate) a5_targets: std::collections::BTreeSet<usize>,
    pub(crate) a4: bool,
    pub(crate) a1: bool,
    pub(crate) paid_burst_until_us: u64,
    pub(crate) free_inferno_until_us: u64,
    pub(crate) free_inferno_generation: u64,
    pub(crate) hunt_until_us: u64,
    pub(crate) hunt_retained_until_us: u64,
    pub(crate) delusion_until_us: u64,
    pub(crate) delusion_retained_until_us: u64,
    pub(crate) kindling_stacks: u32,
    pub(crate) kindling_active_for_ultra: bool,
    pub(crate) r6_enhancement_until_us: u64,
    pub(crate) vile_ash_trains: BTreeMap<usize, VileAshTrain>,
    pub(crate) heartwrench_trains: BTreeMap<usize, VileAshTrain>,
}

impl State {
    pub(crate) fn is_in_hunt(&self, now_us: u64) -> bool {
        self.a1 || self.hunt_until_us > now_us
    }

    pub(crate) fn form_at(&self, now: u64) -> Form {
        if self.form == Form::Illusion && now < self.form_until_us {
            Form::Illusion
        } else {
            Form::Reality
        }
    }

    pub(crate) fn skill_variant(&self, now: u64) -> super::forms::Variant {
        use super::forms::Variant;
        match self.form_at(now) {
            Form::Reality => Variant::Entry(super::clock::entry_ready(self.clock.as_ref(), now)),
            Form::Illusion => Variant::Return(self.form_until_us - now <= 3_500_000),
        }
    }
}
