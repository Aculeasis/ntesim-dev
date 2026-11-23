//! Canonical authored actions follow the owner's current replacement form.
use crate::simulation::{GameActionCommand, state::ActorRuntime};

pub(crate) fn assumptions() -> std::collections::BTreeMap<String, String> {
    [
        super::clock::MODEL,
        super::malice::MODEL,
        super::bird::MODEL,
        super::secondary::MODEL,
        super::secondary::EVOLVED_MODEL,
        super::support::MODEL,
        super::support::EVOLVED_MODEL,
        super::form::MODEL,
        super::skill::EVOLVE_HYPOTHESIS_ID,
    ]
    .into_iter()
    .map(|id| (id.into(), id.description().into()))
    .collect()
}

pub(crate) fn select(actor: &ActorRuntime, command: &mut GameActionCommand) -> Result<(), String> {
    if !super::form::active(actor) {
        return Ok(());
    }
    let GameActionCommand::InvokeAbility {
        ability_id,
        damage_effect_id,
        energy_cost,
        cooldown_us,
        recovery_us,
        ..
    } = command
    else {
        return Ok(());
    };
    let evolved = match ability_id.as_str() {
        "GA_BlackBird_Melee" => "GA_BlackBird_Evolve_Melee",
        "GA_BlackBird_Skill" => "GA_BlackBird_Evolve_Skill",
        "GA_BlackBird_UltraSkill" => "GA_BlackBird_Evolve_UltraSkill",
        _ => return Ok(()),
    };
    let packet = if damage_effect_id == super::secondary::PLUNGE {
        super::secondary::EVOLVE_PLUNGE
    } else {
        super::primary_packet(evolved).ok_or("Blackbird evolved canonical packet is missing.")?
    };
    *ability_id = evolved.into();
    *damage_effect_id = packet.into();
    if evolved == "GA_BlackBird_Evolve_UltraSkill" {
        *energy_cost = 0;
        *cooldown_us = 0;
        *recovery_us = 4_083_334;
    }
    Ok(())
}
