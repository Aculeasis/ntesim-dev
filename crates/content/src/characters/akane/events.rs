use crate::runtime_api::CallbackContext;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Event {
    ModeOvertone {
        owner: usize,
        command: usize,
        target: usize,
        progress: u64,
    },
    ModeReverb {
        owner: usize,
        command: usize,
        progress: u64,
    },
    SyncedBuffs {
        owner: usize,
        command: usize,
        progress: u64,
    },
    PerfectPitchTick {
        owner: usize,
        generation: u64,
    },
    PerfectPitchCleanup {
        owner: usize,
        key: usize,
    },
    SkillFinish {
        owner: usize,
        command: usize,
        progress: u64,
    },
}

impl Event {
    pub(super) fn anchor(&self, actor: usize, action: usize) -> Option<u64> {
        match self {
            Self::ModeOvertone {
                owner,
                command,
                progress,
                ..
            }
            | Self::ModeReverb {
                owner,
                command,
                progress,
            }
            | Self::SyncedBuffs {
                owner,
                command,
                progress,
            }
            | Self::SkillFinish {
                owner,
                command,
                progress,
            } if *owner == actor && *command == action => Some(*progress),
            _ => None,
        }
    }
}

pub(crate) fn apply(ctx: &mut CallbackContext<'_>, event: Event) -> Result<(), String> {
    match event {
        Event::ModeOvertone {
            owner,
            command,
            target,
            ..
        } => super::skill::activate_overtone(ctx, owner, command, target),
        Event::ModeReverb { owner, command, .. } => {
            super::skill::activate_reverb(ctx, owner, command)
        }
        Event::SyncedBuffs { owner, .. } => super::awakenings::apply_synced_buffs(ctx, owner),
        Event::PerfectPitchTick { owner, generation } => {
            super::passive::on_tick(ctx, owner, generation)
        }
        Event::PerfectPitchCleanup { owner, key } => super::passive::cleanup(ctx.state, owner, key),
        Event::SkillFinish { owner, .. } => crate::gear::on_successful_skill_finish(ctx, owner),
    }
}
