use crate::simulation::state::{ActorRuntime, TimedBuffDeclaration, apply_timed_buff};
use crate::simulation::{ExactFraction, GameScenarioRequest};
use ntesim_core::buffs::dispatch::{BuffManager, BuffRecipient, BuffStackingScope};
use std::collections::BTreeMap;

pub(crate) const MODEL: crate::HypothesisId = crate::HypothesisId::AdlerTemperanceV1;
pub(crate) const RNG_STREAM: &str = "adler.temperance";
pub(crate) const DURATION_US: u64 = 15_000_000;
const SOURCE: &str = "character:1033";
const OUTCOMES: [&str; 3] = [
    "adler.temperance_atk",
    "adler.temperance_resistance",
    "adler.temperance_break",
];

pub(crate) fn owner(
    request: &GameScenarioRequest,
    actors: &[ActorRuntime],
) -> Result<Option<usize>, String> {
    let Some((owner, config)) = request
        .actors
        .iter()
        .enumerate()
        .find(|(index, config)| config.character_id == SOURCE && actors[*index].hp > 0)
    else {
        return Ok(None);
    };
    let progression = ntesim_data::skill_levels::progression(
        SOURCE,
        config.character_level,
        config.character_max_level,
    )?;
    let breakthrough = progression["breakthrough"]
        .as_u64()
        .ok_or("Adler breakthrough missing")?;
    Ok((breakthrough >= 2).then_some(owner))
}

pub(crate) fn apply(
    buffs: &mut BuffManager,
    owner: usize,
    target: usize,
    now_us: u64,
    outcome: u64,
) -> Result<&'static str, String> {
    let tenth = ExactFraction::new(1, 10)?;
    let modifiers = match outcome {
        0 => BTreeMap::from([("atk_percent_reduction".into(), ExactFraction::new(1, 5)?)]),
        1 => [
            "PHYSICAL",
            "MENTAL",
            "ECharacterElementType::CHARACTER_ELEMENT_TYPE_COSMOS",
            "ECharacterElementType::CHARACTER_ELEMENT_TYPE_NATURE",
            "ECharacterElementType::CHARACTER_ELEMENT_TYPE_INCANTATION",
            "ECharacterElementType::CHARACTER_ELEMENT_TYPE_CHAOS",
            "ECharacterElementType::CHARACTER_ELEMENT_TYPE_PSYCHE",
            "ECharacterElementType::CHARACTER_ELEMENT_TYPE_LAKSHANA",
        ]
        .into_iter()
        .map(|channel| (format!("resistance_reduction:{channel}"), tenth))
        .collect(),
        2 => BTreeMap::from([("incoming_break_efficiency".into(), tenth)]),
        _ => return Err("invalid Adler Temperance outcome".into()),
    };
    let id = OUTCOMES[outcome as usize];
    apply_timed_buff(
        buffs,
        TimedBuffDeclaration::reset(
            id,
            SOURCE,
            BuffRecipient::SpecificTarget(target.into()),
            DURATION_US,
            BuffStackingScope::PerSource,
            modifiers,
        ),
        owner,
        now_us,
    )?;
    Ok(id)
}

pub(crate) fn clear_target(buffs: &mut BuffManager, target: usize) {
    let target = target.to_string();
    for id in OUTCOMES {
        buffs.remove_target_buff(id, SOURCE, &target);
    }
}
