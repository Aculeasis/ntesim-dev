use crate::runtime_api::CallbackContext;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Event {
    SkillFinish {
        owner: usize,
        command: usize,
        progress: u64,
    },
    RemoraCleanup {
        owner: usize,
        key: usize,
    },
    Warrant {
        owner: usize,
        command: usize,
        all: bool,
        progress: u64,
    },
    EnterForm {
        owner: usize,
        command: usize,
        generation: u64,
        progress: u64,
    },
    AutoCrime {
        owner: usize,
        generation: u64,
        tick: u8,
    },
    VerdictRead {
        owner: usize,
        command: usize,
        progress: u64,
    },
    VerdictCommit {
        owner: usize,
        command: usize,
        progress: u64,
    },
    SupportFinish {
        owner: usize,
        command: usize,
        progress: u64,
    },
}

impl Event {
    pub(super) fn anchor(&self, actor: usize, action: usize) -> Option<u64> {
        match self {
            Self::Warrant {
                owner,
                command,
                progress,
                ..
            }
            | Self::EnterForm {
                owner,
                command,
                progress,
                ..
            }
            | Self::VerdictRead {
                owner,
                command,
                progress,
            }
            | Self::VerdictCommit {
                owner,
                command,
                progress,
            }
            | Self::SkillFinish {
                owner,
                command,
                progress,
            }
            | Self::SupportFinish {
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
        Event::SkillFinish { owner, .. } => crate::gear::on_successful_skill_finish(ctx, owner),
        Event::RemoraCleanup { owner, key } => {
            ctx.state.actors[owner]
                .character
                .chaos_mut()?
                .remora_bonuses
                .remove(&key);
            Ok(())
        }
        Event::Warrant { owner, all, .. } => super::warrant::apply(ctx, owner, all),
        Event::EnterForm {
            owner, generation, ..
        } => super::form::enter(ctx, owner, generation),
        Event::AutoCrime {
            owner,
            generation,
            tick,
        } => super::form::tick(ctx, owner, generation, tick),
        Event::VerdictRead {
            owner,
            command,
            progress,
        } => super::verdict::prepare_read(ctx, owner, command, progress),
        Event::VerdictCommit { owner, command, .. } => super::verdict::read(ctx, owner, command),
        Event::SupportFinish { owner, .. } => super::support::finish(ctx, owner),
    }
}
