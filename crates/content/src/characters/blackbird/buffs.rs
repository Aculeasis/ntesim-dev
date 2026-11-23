use super::parameters::PARAMETERS;
use crate::simulation::{
    GameActorConfig,
    state::{TimedBuffDeclaration, apply_timed_buff},
};
use ntesim_core::buffs::dispatch::{BuffManager, BuffRecipient, BuffStackingScope};
use std::collections::BTreeMap;

pub(super) fn burst(
    buffs: &mut BuffManager,
    config: &GameActorConfig,
    owner: usize,
    now: u64,
) -> Result<(), String> {
    let duration = if config.awakening.has_effect(4) {
        PARAMETERS.a4_team_duration_us
    } else {
        PARAMETERS.team_duration_us
    };
    let mut stats = BTreeMap::from([("atk_percent".into(), PARAMETERS.team_atk_percent)]);
    if config.awakening.has_effect(4) {
        stats.insert("damage_bonus".into(), PARAMETERS.a4_team_damage_bonus);
    }
    apply_timed_buff(
        buffs,
        TimedBuffDeclaration::reset(
            "blackbird.requies_animarum_team",
            &config.character_id,
            BuffRecipient::TeamAll,
            duration,
            BuffStackingScope::PerSource,
            stats,
        ),
        owner,
        now,
    )?;
    if config.awakening.has_effect(6) {
        apply_timed_buff(
            buffs,
            TimedBuffDeclaration::reset(
                "blackbird.a6_psyche_damage",
                &config.character_id,
                BuffRecipient::SelfActor,
                PARAMETERS.a6_psyche_duration_us,
                BuffStackingScope::PerSource,
                BTreeMap::from([(
                    "damage_bonus:ECharacterElementType::CHARACTER_ELEMENT_TYPE_PSYCHE".into(),
                    PARAMETERS.a6_psyche_damage_bonus,
                )]),
            ),
            owner,
            now,
        )?;
    }
    Ok(())
}
