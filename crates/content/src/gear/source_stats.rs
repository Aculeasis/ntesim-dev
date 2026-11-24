//! Packet-independent live stat contributions shared by damage, healing and shields.
use super::GearProcConfig;
use crate::simulation::state::ActorRuntime;
use crate::simulation::{ExactFraction, GameActorConfig, GameScenarioRequest};

pub(crate) fn team_has_three_elements(request: &GameScenarioRequest) -> Result<bool, String> {
    let mut first = None;
    let mut second = None;
    let mut three = false;
    for actor in &request.actors {
        let element = crate::registry::character_descriptor_by_id(&actor.character_id)?
            .element
            .as_str();
        if first == Some(element) || second == Some(element) {
            continue;
        }
        if first.is_none() {
            first = Some(element);
        } else if second.is_none() {
            second = Some(element);
        } else {
            three = true;
        }
    }
    Ok(three)
}

pub(crate) fn def_percent(
    config: &GameActorConfig,
    three_elements: bool,
) -> Result<ExactFraction, String> {
    let mut total = ExactFraction::new(0, 1)?;
    if three_elements {
        for proc in &config.gear_procs {
            if let GearProcConfig::KoinoboriStats { def_bonus, .. } = proc {
                total = total.add(ExactFraction::from_ratio(*def_bonus)?)?;
            }
        }
    }
    Ok(total)
}

pub(crate) fn atk_percent(
    config: &GameActorConfig,
    actor: &ActorRuntime,
    time_us: u64,
    three_elements: bool,
) -> Result<ExactFraction, String> {
    let mut total = ExactFraction::new(0, 1)?;
    for proc in &config.gear_procs {
        let bonus = match proc {
            GearProcConfig::WrongGate { atk_bonus, .. }
            | GearProcConfig::GoldRecord { atk_bonus, .. }
            | GearProcConfig::LunarPhase { atk_bonus, .. }
            | GearProcConfig::MarchingBeyondTime { atk_bonus, .. }
            | GearProcConfig::PoliceRatBoss { atk_bonus, .. }
            | GearProcConfig::RoseThorn { atk_bonus, .. }
            | GearProcConfig::RishiAtk { atk_bonus, .. }
            | GearProcConfig::TigerTallyAtk { atk_bonus, .. }
            | GearProcConfig::WhaleAtk { atk_bonus, .. }
            | GearProcConfig::EchoesOnHold { atk_bonus, .. } => Some(*atk_bonus),
            GearProcConfig::ForgottenHpStats { atk_bonus, .. }
                if u128::from(actor.hp) * 2 > u128::from(actor.hp_max) =>
            {
                Some(*atk_bonus)
            }
            GearProcConfig::KoinoboriStats { atk_bonus, .. } if three_elements => Some(*atk_bonus),
            GearProcConfig::SnowmanShieldAtk { atk_bonus }
                if actor.hp > 0
                    && actor.shields.iter().any(|shield| {
                        shield.remaining_hp > 0 && shield.expires_at_us > time_us
                    }) =>
            {
                Some(*atk_bonus)
            }
            _ => None,
        };
        if let Some(bonus) = bonus {
            total = total.add(ExactFraction::from_ratio(bonus)?)?;
        }
    }
    Ok(total)
}
