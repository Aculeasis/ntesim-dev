use super::*;

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ActorLifecycle {
    pub actor_index: usize,
    pub character_id: String,
    pub action_state: ActionStateMachine,
    pub form_state: FormState,
    pub recast_state: RecastState,
}

impl ActorLifecycle {
    pub fn new(actor_index: usize, character_id: String) -> Self {
        Self {
            actor_index,
            character_id,
            action_state: ActionStateMachine::default(),
            form_state: FormState::default(),
            recast_state: RecastState::default(),
        }
    }
}
