//! Live source selects the native clock before initial field callbacks.
use super::MODEL;

pub(crate) fn assumptions() -> std::collections::BTreeMap<String, String> {
    [(MODEL.into(), MODEL.description().into())].into()
}

pub(crate) fn select(command: &mut crate::simulation::GameActionCommand) {
    if let crate::simulation::GameActionCommand::InvokeAbility {
        ability_id,
        cooldown_us,
        ..
    } = command
        && ability_id == "GA_Mint019_UltraSkill"
    {
        *cooldown_us = 15_000_000;
    }
}
