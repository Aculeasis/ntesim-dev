use super::*;

pub(crate) fn group_tables() -> BTreeMap<String, Value> {
    BTreeMap::from([(
        "DT_ForkItemData".into(),
        crate::weapon_definitions::overlay(),
    )])
}
