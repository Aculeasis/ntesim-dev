//! More Than Passionate: a living B2 party member enhances newly born Buds.
use crate::runtime_api::BlossomEmissionProfile;
use crate::simulation::state::ActorRuntime;

pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::NanallyBlossomEnhancementV1;

pub(crate) fn profile(actor: &ActorRuntime) -> Option<BlossomEmissionProfile> {
    (actor.hp > 0
        && matches!(&actor.character, super::super::RuntimeState::Nanally(state) if state.blossom_enhancement_unlocked))
    .then_some(BlossomEmissionProfile {
        count: 10,
        interval_us: 1_000_000,
        model: MODEL,
    })
}
