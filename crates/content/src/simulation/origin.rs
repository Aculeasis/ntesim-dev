//! Opaque causal keys keep existing owner contexts without inventing actions.
use super::{GameAutonomousEventReceipt, GameCausalTrigger, GameScenarioRequest};

// Identical on native and WASM32. Authored commands occupy the lower half.
const AUTONOMOUS_START: usize = 0x8000_0000;

#[derive(Clone, Debug, Default)]
pub(crate) struct Origins {
    events: Vec<GameAutonomousEventReceipt>,
}

impl Origins {
    pub(crate) fn validate(request: &GameScenarioRequest) -> Result<(), String> {
        if request.actions.len() > AUTONOMOUS_START {
            return Err("authored actions overlap autonomous causal keys".into());
        }
        Ok(())
    }

    pub(crate) fn register(
        &mut self,
        request: &GameScenarioRequest,
        event: GameAutonomousEventReceipt,
    ) -> Result<usize, String> {
        self.register_checked(request, event, None)
    }

    pub(crate) fn register_from_hit(
        &mut self,
        request: &GameScenarioRequest,
        event: GameAutonomousEventReceipt,
        hits: &[super::GameHitReceipt],
    ) -> Result<usize, String> {
        self.register_checked(request, event, Some(hits))
    }

    fn register_checked(
        &mut self,
        request: &GameScenarioRequest,
        event: GameAutonomousEventReceipt,
        hits: Option<&[super::GameHitReceipt]>,
    ) -> Result<usize, String> {
        Self::validate(request)?;
        if event.actor_index >= request.actors.len()
            || event.active_actor_index >= request.actors.len()
            || event.time_us > request.absolute_limit_us
            || event.ability_id.is_empty()
        {
            return Err("invalid autonomous event identity".into());
        }
        let valid = match event.trigger {
            GameCausalTrigger::SettledHit { hit_index } => hits
                .and_then(|values| values.get(hit_index))
                .is_some_and(|hit| {
                    hit.time_us == event.time_us
                        && hit.actor_index < request.actors.len()
                        && hit.target_index < request.targets.len()
                }),
            GameCausalTrigger::CoordinatedHit {
                cast_command_index,
                hit_index,
            } => {
                matches!(request.actions.get(cast_command_index), Some(super::GameActionCommand::InvokeAbility { actor_index, timestamp_us, .. })
                    if *actor_index == event.actor_index && *timestamp_us <= event.time_us)
                    && hits
                        .and_then(|values| values.get(hit_index))
                        .is_some_and(|hit| {
                            hit.applied_hp_loss > 0
                                && hit.time_us == event.time_us
                                && hit.actor_index < request.actors.len()
                                && hit.target_index < request.targets.len()
                        })
            }
            GameCausalTrigger::FieldEntry { command_index } => matches!(
                request.actions.get(command_index),
                Some(super::GameActionCommand::SwitchActor { .. })
            ),
            GameCausalTrigger::Action { command_index } => command_index < request.actions.len(),
            GameCausalTrigger::IncomingDamage { event_index } => {
                event_index < request.incoming_damage_events.len()
            }
            GameCausalTrigger::HpCost { event_index } => event_index < request.hp_cost_events.len(),
            GameCausalTrigger::OwnerDeadline {
                autonomous_event_index,
            } => autonomous_event_index < self.events.len(),
            GameCausalTrigger::InitialState | GameCausalTrigger::VitalChange { .. } => true,
        };
        if !valid {
            return Err("autonomous trigger has no real causal parent".into());
        }
        let key = Self::event_key(self.events.len())?;
        self.events.push(event);
        Ok(key)
    }

    pub(crate) fn event_key(index: usize) -> Result<usize, String> {
        AUTONOMOUS_START
            .checked_add(index)
            .filter(|key| u32::try_from(*key).is_ok())
            .ok_or_else(|| "autonomous causal key overflow".into())
    }

    pub(crate) fn identify(
        &self,
        request: &GameScenarioRequest,
        key: usize,
    ) -> Result<(Option<usize>, Option<usize>), String> {
        if key < AUTONOMOUS_START {
            return if key < request.actions.len() {
                Ok((Some(key), None))
            } else {
                Err("hit has no authored command or autonomous event".into())
            };
        }
        let index = key - AUTONOMOUS_START;
        if index >= self.events.len() {
            return Err("unknown autonomous causal key".into());
        }
        Ok((None, Some(index)))
    }

    pub(crate) fn get(&self, index: usize) -> Result<&GameAutonomousEventReceipt, String> {
        self.events
            .get(index)
            .ok_or_else(|| "unknown autonomous event index".into())
    }

    pub(crate) fn healing(
        &mut self,
        index: usize,
        healing: super::GameAutonomousHealing,
    ) -> Result<(), String> {
        let event = self
            .events
            .get_mut(index)
            .ok_or("unknown autonomous healing event")?;
        if event.recipient_healing.is_some() {
            return Err("duplicate autonomous recipient healing".into());
        }
        event.recipient_healing = Some(healing);
        Ok(())
    }

    pub(crate) fn receipts(&self) -> Vec<GameAutonomousEventReceipt> {
        self.events.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn event(trigger: GameCausalTrigger) -> GameAutonomousEventReceipt {
        GameAutonomousEventReceipt {
            actor_index: 0,
            active_actor_index: 0,
            ability_id: "GA_Cang_WorldPas_Fadia".into(),
            time_us: 0,
            trigger,
            recipient_healing: None,
        }
    }

    #[test]
    fn authored_and_autonomous_identity_are_disjoint_portable_and_checked_without_fake_actions() {
        let req = crate::simulation::tests::sample_s1_request();
        let original = req.actions.len();
        let mut sources = Origins::default();
        assert_eq!(sources.identify(&req, 0).unwrap(), (Some(0), None));
        let key = sources
            .register(&req, event(GameCausalTrigger::InitialState))
            .unwrap();
        assert_eq!(key, 2_147_483_648);
        assert_eq!(sources.identify(&req, key).unwrap(), (None, Some(0)));
        assert_eq!(req.actions.len(), original);
        assert!(sources.identify(&req, key + 1).is_err());
        assert!(
            sources
                .register(
                    &req,
                    event(GameCausalTrigger::OwnerDeadline {
                        autonomous_event_index: 1
                    })
                )
                .is_err()
        );
        assert_eq!(sources.receipts().len(), 1);
        let second = sources
            .register(
                &req,
                event(GameCausalTrigger::OwnerDeadline {
                    autonomous_event_index: 0,
                }),
            )
            .unwrap();
        assert_eq!(sources.identify(&req, second).unwrap(), (None, Some(1)));
        assert_eq!(Origins::event_key(2_147_483_647).unwrap(), 4_294_967_295);
        assert!(Origins::event_key(2_147_483_648).is_err());
    }
}
