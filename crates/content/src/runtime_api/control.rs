use super::ActorHitContext;
use crate::simulation::{GameControlApplication, GameControlKind};
use ntesim_core::buffs::dispatch::{DecayPolicy, RefreshPolicy};

impl ActorHitContext<'_> {
    /// Owner-requested control after a connected hit; no movement or target AI.
    pub(crate) fn apply_control(
        &mut self,
        kind: GameControlKind,
        duration_us: u64,
    ) -> Result<bool, String> {
        crate::runtime_api::record_hypothesis_id(
            self.request,
            self.execution_assumptions,
            crate::HypothesisId::CombatControlEffectsV1,
        )?;
        if !self.target_alive
            || self.actor.hp == 0
            || self.request.targets[self.target_index].control_immune
        {
            return Ok(false);
        }
        if duration_us == 0 {
            return Err("control duration must be positive".into());
        }
        let expires_at_us = self
            .time_us
            .checked_add(duration_us)
            .ok_or("control deadline overflow")?;
        let tag = match kind {
            GameControlKind::Suppress => "ntesim.control.suppress",
        };
        crate::simulation::state::apply_target_tagged_status(
            self.buffs,
            crate::simulation::state::TargetStatusDeclaration {
                source_actor: self.actor_index,
                source: self.config.character_id.clone(),
                target_index: self.target_index,
                id: tag.into(),
                now_us: self.world_time_us,
                duration_us,
                reductions: Default::default(),
            },
            std::collections::BTreeSet::from([tag.into()]),
            1,
            RefreshPolicy::Reset,
            DecayPolicy::AllAtOnce,
        )?;
        self.control_applications.push(GameControlApplication {
            autonomous_event_index: self.autonomous_event_index,
            kind,
            source_actor_index: self.actor_index,
            target_index: self.target_index,
            command_index: self.authored_command_index,
            duration_us,
            expires_at_us,
        });
        Ok(true)
    }
}
