//! Captured native action identity shared by committed owner continuations.
use crate::runtime_api::{CallbackContext, CastContext};
use crate::simulation::metadata::ActionMetadata;
#[derive(Clone, Debug)]
pub(super) struct Source {
    pub(super) command: usize,
    pub(super) coordinate: usize,
    pub(super) action: ActionMetadata,
}
impl Source {
    pub(super) fn capture(ctx: &CastContext<'_>) -> Result<Self, String> {
        Ok(Self {
            command: ctx.command_index,
            coordinate: ctx
                .native_array_index
                .ok_or("Chiz native source coordinate missing")?,
            action: ctx.action_metadata.clone(),
        })
    }
    pub(super) fn with_cast<T>(
        &self,
        ctx: &mut CallbackContext<'_>,
        owner: usize,
        run: impl FnOnce(&mut CastContext<'_>) -> Result<T, String>,
    ) -> Result<T, String> {
        let now = ctx.state.current_time_us;
        run(&mut CastContext {
            origins: &mut ctx.state.origins,
            command_index: self.command,
            native_array_index: Some(self.coordinate),
            execution_assumptions: &mut ctx.state.execution_model_assumptions,
            config: &ctx.request.actors[owner],
            request: ctx.request,
            actor_index: owner,
            active_actor_index: ctx.state.active_actor_index,
            target_index: ctx.request.aim_target(self.command),
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
        })
    }
    pub(super) fn emit(
        &self,
        ctx: &mut CallbackContext<'_>,
        owner: usize,
        effect: &str,
    ) -> Result<String, String> {
        self.with_cast(ctx, owner, |cast| {
            cast.schedule_native_aoe_group(effect, cast.time_us)
        })
    }
}
