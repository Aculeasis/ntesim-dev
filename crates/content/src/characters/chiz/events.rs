use crate::runtime_api::CallbackContext;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Event {
    Barren(super::barren::Event),
    Market(super::market::Event),
    HeldPhase {
        owner: usize,
        command: usize,
        phase: super::pressed::held::Phase,
        progress: u64,
    },
    StockRefill {
        owner: usize,
    },
    PressSettlement {
        owner: usize,
        command: usize,
    },
    PressPause {
        owner: usize,
        command: usize,
        duration: u64,
        rate: u32,
    },
    Interest {
        owner: usize,
        generation: u64,
    },
    HitPause {
        owner: usize,
        duration_us: u64,
        rate: u32,
    },
}

pub(crate) fn apply(ctx: &mut CallbackContext<'_>, event: Event) -> Result<(), String> {
    match event {
        Event::Barren(event) => super::barren::apply(ctx, event),
        Event::Market(event) => super::market::apply(ctx, event),
        Event::HeldPhase {
            owner,
            command,
            phase,
            ..
        } => super::pressed::held::apply(ctx, owner, command, phase),
        Event::StockRefill { owner } => super::pressed::refill(ctx, owner),
        Event::PressSettlement { owner, command } => super::pressed::settle(ctx, owner, command),
        Event::PressPause {
            owner,
            command,
            duration,
            rate,
        } => {
            super::pressed::pause(ctx, owner, command, duration, rate)?;
            if super::secondary::accepted(ctx.request) && ctx.state.active_actor_index == owner {
                super::clock::pause(ctx, owner, duration, rate)?;
                super::pressed::pause_avatar(ctx, owner, Some(command), duration, rate)?;
            }
            Ok(())
        }
        Event::Interest { owner, generation } => super::finance::interest(ctx, owner, generation),
        Event::HitPause {
            owner,
            duration_us,
            rate,
        } => {
            if super::secondary::accepted(ctx.request) && ctx.state.active_actor_index != owner {
                return Ok(());
            }
            super::clock::pause(ctx, owner, duration_us, rate)?;
            if super::secondary::accepted(ctx.request) && ctx.state.active_actor_index == owner {
                super::pressed::pause_avatar(ctx, owner, None, duration_us, rate)?;
            }
            Ok(())
        }
    }
}
