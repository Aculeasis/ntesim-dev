use crate::runtime_api::CallbackContext;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Event {
    ActionAddE {
        owner: usize,
        command: usize,
        progress: u64,
        grant_e1: bool,
        grant_e2: bool,
    },
    ActionManage {
        owner: usize,
        command: usize,
        progress: u64,
    },
    SkillFinish {
        owner: usize,
        command: usize,
        progress: u64,
    },
    ExpirePowerE1 {
        owner: usize,
        generation: u64,
    },
    ExpirePowerE2 {
        owner: usize,
        generation: u64,
    },
    ExpireOpticalIllusion {
        owner: usize,
        generation: u64,
    },
    ExpireQAtkBuff {
        owner: usize,
        generation: u64,
    },
    OnTheLineTick {
        owner: usize,
        generation: u64,
    },
    RemoraEnhancement {
        owner: usize,
        target: usize,
        command: usize,
        generation: u64,
    },
}

impl Event {
    pub(super) fn anchor(&self, actor: usize, action: usize) -> Option<u64> {
        match self {
            Self::ActionAddE {
                owner,
                command,
                progress,
                ..
            }
            | Self::ActionManage {
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
        Event::ActionAddE {
            owner,
            grant_e1,
            grant_e2,
            ..
        } => super::skill::on_add_e_notify(ctx, owner, grant_e1, grant_e2),
        Event::ActionManage { owner, .. } => super::skill::on_skill2_manage_notify(ctx, owner),
        Event::SkillFinish { owner, .. } => crate::gear::on_successful_skill_finish(ctx, owner),
        Event::ExpirePowerE1 { owner, generation } => {
            super::skill::expire_power_e1(ctx.state, owner, generation)
        }
        Event::ExpirePowerE2 { owner, generation } => {
            super::skill::expire_power_e2(ctx.state, owner, generation)
        }
        Event::ExpireOpticalIllusion { owner, generation } => {
            super::skill::expire_optical_illusion(ctx.state, owner, generation)
        }
        Event::ExpireQAtkBuff { owner, generation } => {
            super::skill::expire_q_atk_buff(ctx.state, owner, generation)
        }
        Event::OnTheLineTick { owner, generation } => {
            super::on_the_line::on_tick(ctx, owner, generation)
        }
        Event::RemoraEnhancement {
            owner,
            target,
            command,
            generation,
        } => super::passive::on_delayed_remora(ctx, owner, target, command, generation),
    }
}
