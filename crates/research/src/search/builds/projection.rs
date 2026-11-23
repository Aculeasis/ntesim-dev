use super::*;

/// Candidate profiles are not source-bound to legal equipment, so they cannot
/// be projected into an executable combat scenario.
pub fn builds_to_static_stats(
    _builds: &BTreeMap<String, CharacterBuildProfile>,
) -> Result<BTreeMap<String, GameActorStaticStats>, String> {
    Err("candidate profiles cannot drive combat until they include a source-bound equipment legality receipt".into())
}
