//! Retained admitted identity/talent for later owner packets, without stat snapshots.
use super::{CallbackContext, CastContext};
use crate::simulation::metadata::ActionMetadata;
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct OwnedAction {
    command: usize,
    coordinate: usize,
    action: ActionMetadata,
}
impl OwnedAction {
    pub(crate) fn command(&self) -> usize {
        self.command
    }
    pub(crate) fn capture(ctx: &CastContext<'_>) -> Result<Self, String> {
        Ok(Self {
            command: ctx.command_index,
            coordinate: ctx
                .native_array_index
                .ok_or("Owned packet needs a native talent coordinate")?,
            action: ctx.action_metadata.clone(),
        })
    }
    pub(crate) fn with_cast<T>(
        &self,
        ctx: &mut CallbackContext<'_>,
        owner: usize,
        target: usize,
        apply: impl FnOnce(&mut CastContext<'_>) -> Result<T, String>,
    ) -> Result<T, String> {
        let now = ctx.state.current_time_us;
        let mut cast = CastContext {
            origins: &mut ctx.state.origins,
            command_index: self.command,
            native_array_index: Some(self.coordinate),
            execution_assumptions: &mut ctx.state.execution_model_assumptions,
            config: &ctx.request.actors[owner],
            request: ctx.request,
            actor_index: owner,
            active_actor_index: ctx.state.active_actor_index,
            target_index: target,
            ability_id: &self.action.ability_id,
            action_metadata: &self.action,
            packet_metadata: None,
            hit_time_us: now,
            world_time_us: ctx.state.mode_clock.relative_us(),
            time_us: now,
            buffs: &mut ctx.state.buffs,
            actor: &mut ctx.state.actors[owner],
            stamina: ctx.state.combat_stamina.as_mut(),
            team_hp_snapshot: &[],
            targets: &ctx.state.targets,
            queue: ctx.queue,
            sequence: ctx.sequence,
        };
        apply(&mut cast)
    }
}
