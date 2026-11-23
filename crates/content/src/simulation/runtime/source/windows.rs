//! Yield to the author at an existing owner input gate, without admitting it.
use super::Trial;
use crate::simulation::{GameActionCommand, PlayerInput, SourceActionWindow};

impl Trial<'_> {
    pub(super) fn advance_action_window(
        &mut self,
        actor: usize,
        window: Option<SourceActionWindow>,
    ) -> Result<(), String> {
        while self.execution.success && self.execution.execution_completed {
            if self.execution.all_targets_defeated() || self.time_completed() {
                break;
            }
            let now = self.execution.state.current_time_us;
            let runtime = &self.execution.state.actors[actor];
            if runtime.hp == 0 {
                break;
            }
            let deadline = match window {
                Some(window) => self.input_window(actor, window)?,
                None => crate::characters::source_choices::completion_deadline(runtime, now),
            };
            if deadline <= now {
                break;
            }
            // A contact can add hitlag or an interruption can end recovery.
            // Recompute only after actual queued events or the native gate.
            let next = self
                .execution
                .queue
                .peek()
                .map_or(deadline, |event| event.time_us.max(now).min(deadline));
            self.advance(next)?;
        }
        Ok(())
    }

    fn input_window(&self, actor: usize, window: SourceActionWindow) -> Result<u64, String> {
        let runtime = &self.execution.state.actors[actor];
        let now = self.execution.state.current_time_us;
        if window == SourceActionWindow::Hold {
            return match crate::registry::held_input_window(&self.request, runtime, now) {
                // A forced release can remove the producer before its gate.
                Err(_) if runtime.locked_until_us <= now => Ok(now),
                result => result,
            };
        }
        let owner = &self.program.config.actors[actor];
        let descriptor = owner
            .projection
            .public_actions
            .get(window.label())
            .ok_or_else(|| {
                format!(
                    "Unsupported {} input window for {}.",
                    window.label(),
                    owner.name
                )
            })?;
        let mut candidate = descriptor.command.clone();
        candidate.set_timestamp_us(self.execution.state.current_time_us);
        let command =
            crate::registry::source_command(&self.request.actors[actor], runtime, &candidate)?;
        let ability = match &command {
            GameActionCommand::InvokeAbility { ability_id, .. } => ability_id.as_str(),
            GameActionCommand::PlayerInput {
                input: PlayerInput::Jump,
                ..
            } => "jump",
            GameActionCommand::PlayerInput {
                input: PlayerInput::Dash,
                ..
            } => "dash",
            GameActionCommand::SwitchActor { .. } => "switch_actor",
            _ => return Err("Input window requires an ordinary owner input family.".into()),
        };
        Ok(crate::registry::recovery_lock_override(
            &self.request,
            &self.request.actors[actor],
            runtime,
            ability,
            now,
        )?
        .map_or(runtime.locked_until_us, |(deadline, _)| deadline))
    }
}
