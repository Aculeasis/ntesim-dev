//! InputID2 selects the non-damaging RemoveSkill while Authority is live.
pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::NanallyAuthorityRemovalV1;
// Real cooldown duration from the reached owner-clock notification.
pub(crate) const SKILL_COOLDOWN_US: u64 = 16_000_000;

pub(crate) fn select(
    actor: &crate::simulation::state::ActorRuntime,
    ability: &str,
    now: u64,
) -> Option<crate::runtime_api::UtilityInvocation> {
    if ability != "GA_Nanally_Skill" {
        return None;
    }
    let super::super::RuntimeState::Nanally(state) = &actor.character else {
        return None;
    };
    (state.authority_until_us > now).then_some(crate::runtime_api::UtilityInvocation {
        ability_id: "GA_Nanally_RemoveSkill",
        ready_at_us: state.authority_recast_unlock_us,
        recovery_us: 0,
        model: MODEL,
    })
}
