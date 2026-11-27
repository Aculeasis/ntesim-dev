use crate::runtime_api::CallbackContext;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Event {
    WatchStart { owner: usize, generation: u64 },
    WatchTick { owner: usize, generation: u64 },
    EndVerdictRage { owner: usize, command: usize },
    ScarletDescentHeal { owner: usize, command: usize },
}

pub(crate) fn apply(ctx: &mut CallbackContext<'_>, event: Event) -> Result<(), String> {
    match event {
        Event::WatchStart { owner, generation } => {
            super::passive::on_watch_start(ctx, owner, generation)
        }
        Event::WatchTick { owner, generation } => {
            super::passive::on_watch_tick(ctx, owner, generation)
        }
        Event::EndVerdictRage { owner, .. } => super::skill::end_verdict_rage(ctx, owner),
        Event::ScarletDescentHeal { owner, .. } => super::skill::apply_a5_heal(ctx, owner),
    }
}
