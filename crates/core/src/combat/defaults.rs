use super::*;

pub(crate) fn default_stat() -> String {
    "atk".into()
}

pub(crate) fn default_element() -> String {
    "Physical".into()
}

pub(crate) fn default_damage_kind() -> DamageKind {
    DamageKind::Elemental
}

pub(crate) fn default_crit_rule() -> CritRule {
    CritRule::Normal
}

pub(crate) fn default_one() -> f64 {
    1.0
}

pub(crate) fn default_rounding() -> Rounding {
    Rounding::Floor
}
