//! Only character identity/aliases, damage element, weapon group consumed by the simulator. No item, audio, world or UI asset dump.
//! Global DT_Character metadata was checked against the cached source for all
//! 25 released identities. Preserve Daffodill/Crimson/Akane Rin/Elyms as distinct script aliases;
//! every other native name duplicates the registry slug or is blank (Zankou).
use super::values::Base;
use serde_json::{Value, json};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Element {
    Chaos,
    Cosmos,
    Incantation,
    Lakshana,
    Nature,
    Psyche,
}
impl Element {
    pub const fn key(self) -> &'static str {
        match self {
            Self::Chaos => "CHAOS",
            Self::Cosmos => "COSMOS",
            Self::Incantation => "INCANTATION",
            Self::Lakshana => "LAKSHANA",
            Self::Nature => "NATURE",
            Self::Psyche => "PSYCHE",
        }
    }
    pub const fn native_key(self) -> &'static str {
        match self {
            Self::Chaos => "ECharacterElementType::CHARACTER_ELEMENT_TYPE_CHAOS",
            Self::Cosmos => "ECharacterElementType::CHARACTER_ELEMENT_TYPE_COSMOS",
            Self::Incantation => "ECharacterElementType::CHARACTER_ELEMENT_TYPE_INCANTATION",
            Self::Lakshana => "ECharacterElementType::CHARACTER_ELEMENT_TYPE_LAKSHANA",
            Self::Nature => "ECharacterElementType::CHARACTER_ELEMENT_TYPE_NATURE",
            Self::Psyche => "ECharacterElementType::CHARACTER_ELEMENT_TYPE_PSYCHE",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WeaponGroup {
    One,
    Two,
    Three,
    Four,
    Five,
}
impl WeaponGroup {
    pub const fn native_key(self) -> &'static str {
        match self {
            Self::One => "ECharacterGroupType::CHARACTER_GROUP_TYPE_ONE",
            Self::Two => "ECharacterGroupType::CHARACTER_GROUP_TYPE_TWO",
            Self::Three => "ECharacterGroupType::CHARACTER_GROUP_TYPE_THREE",
            Self::Four => "ECharacterGroupType::CHARACTER_GROUP_TYPE_FOUR",
            Self::Five => "ECharacterGroupType::CHARACTER_GROUP_TYPE_FIVE",
        }
    }
    pub fn from_native(value: &str) -> Result<Self, String> {
        match value {
            "ECharacterGroupType::CHARACTER_GROUP_TYPE_ONE" => Ok(Self::One),
            "ECharacterGroupType::CHARACTER_GROUP_TYPE_TWO" => Ok(Self::Two),
            "ECharacterGroupType::CHARACTER_GROUP_TYPE_THREE" => Ok(Self::Three),
            "ECharacterGroupType::CHARACTER_GROUP_TYPE_FOUR" => Ok(Self::Four),
            "ECharacterGroupType::CHARACTER_GROUP_TYPE_FIVE" => Ok(Self::Five),
            _ => Err("unsupported native character/weapon compatibility group".into()),
        }
    }
}
pub struct Definition {
    pub slug: &'static str,
    pub released: bool,
    pub native_id: &'static str,
    pub element: Element,
    pub weapon_group: WeaponGroup,
    pub script_alias: Option<&'static str>,
    pub(super) base: Base,
}
impl Definition {
    const fn new(
        native_id: &'static str,
        slug: &'static str,
        released: bool,
        element: Element,
        weapon_group: WeaponGroup,
        script_alias: Option<&'static str>,
        base: Base,
    ) -> Self {
        Self {
            slug,
            released,
            native_id,
            element,
            weapon_group,
            script_alias,
            base,
        }
    }
    pub const fn base_energy(&self) -> u32 {
        self.base.energy
    }
    pub fn evidence(&self) -> Value {
        json!({"kind":"typed_character_definition","character_id":format!("character:{}",self.native_id),
            "source":"crates/data/src/constants/character/definitions.rs", "element":self.element.key(),
            "weapon_group":self.weapon_group.native_key(), "availability":if self.released {"released"} else {"unreleased"}})
    }
}
static DEFINITIONS: [Definition; 25] = [
    Definition::new(
        "1003",
        "sakiri",
        true,
        Element::Incantation,
        WeaponGroup::Four,
        None,
        Base::new(1360, 78, 75, 120),
    ),
    Definition::new(
        "1004",
        "lacrimosa",
        true,
        Element::Chaos,
        WeaponGroup::Two,
        None,
        Base::new(1320, 80, 75, 120),
    ),
    Definition::new(
        "1008",
        "skia",
        true,
        Element::Lakshana,
        WeaponGroup::Four,
        None,
        Base::new(1100, 70, 55, 100),
    ),
    Definition::new(
        "1010",
        "nanally",
        true,
        Element::Nature,
        WeaponGroup::Three,
        None,
        Base::new(1320, 80, 75, 100),
    ),
    Definition::new(
        "1019",
        "mint",
        true,
        Element::Nature,
        WeaponGroup::Two,
        None,
        Base::new(1000, 75, 60, 100),
    ),
    Definition::new(
        "1020",
        "haniel",
        true,
        Element::Psyche,
        WeaponGroup::One,
        None,
        Base::new(1130, 62, 70, 120),
    ),
    Definition::new(
        "1021",
        "edgar",
        true,
        Element::Cosmos,
        WeaponGroup::Two,
        None,
        Base::new(1350, 53, 68, 100),
    ),
    Definition::new(
        "1023",
        "baicang",
        true,
        Element::Incantation,
        WeaponGroup::Five,
        None,
        Base::new(1370, 81, 66, 120),
    ),
    Definition::new(
        "1025",
        "hathor",
        true,
        Element::Lakshana,
        WeaponGroup::Three,
        None,
        Base::new(1320, 81, 73, 100),
    ),
    Definition::new(
        "1033",
        "adler",
        true,
        Element::Incantation,
        WeaponGroup::Five,
        None,
        Base::new(1180, 55, 82, 120),
    ),
    Definition::new(
        "1036",
        "zankou",
        true,
        Element::Incantation,
        WeaponGroup::Four,
        None,
        Base::new(1280, 83, 75, 120),
    ),
    Definition::new(
        "1039",
        "fadia",
        true,
        Element::Psyche,
        WeaponGroup::Five,
        None,
        Base::new(1410, 75, 75, 120),
    ),
    Definition::new(
        "1042",
        "blackbird",
        true,
        Element::Psyche,
        WeaponGroup::Four,
        None,
        Base::new(1280, 84, 73, 100),
    ),
    Definition::new(
        "1046",
        "zero",
        true,
        Element::Cosmos,
        WeaponGroup::One,
        None,
        Base::new(1250, 85, 75, 100),
    ),
    Definition::new(
        "1052",
        "hotori",
        true,
        Element::Cosmos,
        WeaponGroup::One,
        None,
        Base::new(1330, 82, 70, 100),
    ),
    Definition::new(
        "1054",
        "daffodil",
        true,
        Element::Chaos,
        WeaponGroup::Two,
        Some("Daffodill"),
        Base::new(1335, 81, 72, 120),
    ),
    Definition::new(
        "1055",
        "jiuyuan",
        true,
        Element::Nature,
        WeaponGroup::One,
        None,
        Base::new(1290, 82, 75, 100),
    ),
    Definition::new(
        "1057",
        "akane",
        true,
        Element::Lakshana,
        WeaponGroup::Two,
        Some("Akane Rin"),
        Base::new(1320, 80, 76, 100),
    ),
    Definition::new(
        "1070",
        "aurelia",
        true,
        Element::Psyche,
        WeaponGroup::Three,
        None,
        Base::new(980, 76, 60, 120),
    ),
    Definition::new(
        "1071",
        "chaos",
        true,
        Element::Lakshana,
        WeaponGroup::Five,
        None,
        Base::new(1300, 80, 76, 100),
    ),
    Definition::new(
        "1072",
        "linko",
        true,
        Element::Nature,
        WeaponGroup::Three,
        None,
        Base::new(1280, 82, 76, 100),
    ),
    Definition::new(
        "1073",
        "chiz",
        true,
        Element::Cosmos,
        WeaponGroup::Four,
        None,
        Base::new(1280, 83, 75, 100),
    ),
    Definition::new(
        "1075",
        "iroi",
        true,
        Element::Nature,
        WeaponGroup::Two,
        None,
        Base::new(1420, 75, 75, 100),
    ),
    Definition::new(
        "1076",
        "shinku",
        true,
        Element::Cosmos,
        WeaponGroup::Five,
        Some("Crimson"),
        Base::new(1330, 80, 74, 100),
    ),
    Definition::new(
        "1090",
        "elyms",
        true,
        Element::Cosmos,
        WeaponGroup::One,
        Some("Elyms"),
        Base::new(1270, 83, 75, 100),
    ),
];
pub fn definitions() -> &'static [Definition] {
    &DEFINITIONS
}
pub fn definition(character_id: &str) -> Result<&'static Definition, String> {
    let native = character_id
        .strip_prefix("character:")
        .ok_or("expected character:native_id")?;
    DEFINITIONS
        .iter()
        .find(|c| c.native_id.eq_ignore_ascii_case(native))
        .ok_or_else(|| format!("unsupported character identity: {character_id}"))
}
/// Released identities from the consumed native definitions, independent of kit readiness.
pub fn released_ids() -> impl Iterator<Item = String> {
    definitions()
        .iter()
        .filter(|definition| definition.released)
        .map(|definition| format!("character:{}", definition.native_id))
}

/// JSON transport projection for consumers that require an explicit released inventory.
pub fn released_inventory() -> Value {
    json!({"entities": definitions().iter().filter(|definition| definition.released).map(|definition| json!({
        "id": format!("character:{}", definition.native_id), "slug": definition.slug,
        "kind": "character", "availability": "released"
    })).collect::<Vec<_>>()})
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn independent_group_element_and_alias_bindings() {
        for (id, group) in [
            ("1020", WeaponGroup::One),
            ("1019", WeaponGroup::Two),
            ("1010", WeaponGroup::Three),
            ("1003", WeaponGroup::Four),
            ("1033", WeaponGroup::Five),
        ] {
            let c = definition(&format!("character:{id}")).unwrap();
            assert_eq!(c.weapon_group, group);
            assert_eq!(WeaponGroup::from_native(group.native_key()).unwrap(), group);
        }
        assert_eq!(
            definition("character:1003").unwrap().element,
            Element::Incantation
        );
        assert_eq!(
            definition("character:1046").unwrap().element,
            Element::Cosmos
        );
        assert_eq!(
            definition("character:1054").unwrap().script_alias,
            Some("Daffodill")
        );
        assert_eq!(
            definition("character:1076").unwrap().script_alias,
            Some("Crimson")
        );
        assert_eq!(
            definition("character:1057").unwrap().script_alias,
            Some("Akane Rin")
        );
        assert_eq!(
            definition("character:1090").unwrap().script_alias,
            Some("Elyms")
        );
        assert_eq!(definition("character:1036").unwrap().script_alias, None);
        assert_eq!(definitions().len(), 25);
        assert!(definition("character:1051").is_err());
        assert!(definition("1003").is_err());
        assert!(WeaponGroup::from_native("ECharacterGroupType::CHARACTER_GROUP_TYPE_SIX").is_err());
    }
}
