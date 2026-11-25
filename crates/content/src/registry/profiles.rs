//! Explicit lookup of historical preview profiles; lookup does not imply readiness.
use crate::simulation::GearProcConfig;
use ntesim_core::formula::Ratio;

#[derive(Clone, Debug)]
pub struct CharacterDescriptor {
    pub slug: &'static str,
    pub character_id: &'static str,
    pub element: String,
}

pub fn character_descriptors() -> Result<Vec<CharacterDescriptor>, String> {
    let mut descriptors = ntesim_data::character::definitions()
        .iter()
        .map(|definition| get_character_descriptor(definition.slug))
        .collect::<Result<Vec<_>, _>>()?;
    descriptors.sort_unstable_by_key(|descriptor| descriptor.slug);
    Ok(descriptors)
}

pub fn get_character_descriptor(slug: &str) -> Result<CharacterDescriptor, String> {
    let profile = get_character_profile(slug)?;
    let element = ntesim_data::character::definition(profile.character_id)?
        .element
        .key()
        .to_owned();
    Ok(CharacterDescriptor {
        slug: profile.slug,
        character_id: profile.character_id,
        element,
    })
}

pub fn get_character_descriptor_by_id(id: &str) -> Result<CharacterDescriptor, String> {
    character_descriptor_by_id(id).cloned()
}

pub(crate) fn character_descriptor_by_id(id: &str) -> Result<&'static CharacterDescriptor, String> {
    static DESCRIPTORS: std::sync::LazyLock<Result<Vec<CharacterDescriptor>, String>> =
        std::sync::LazyLock::new(character_descriptors);
    DESCRIPTORS
        .as_ref()
        .map_err(Clone::clone)?
        .iter()
        .find(|d| d.character_id == id)
        .ok_or_else(|| format!("unknown character identity {id}"))
}

#[derive(Clone, Debug)]
pub struct PreviewCharacterProfile {
    pub slug: &'static str,
    pub character_id: &'static str,
    pub awakening: ntesim_data::AwakeningSelection,
    pub weapon_id: &'static str,
    pub set_id: Option<&'static str>,
    pub weapon_tier: u32,
    pub base_atk: u64,
    pub atk_ratio: Ratio,
    pub hp: u64,
    pub def: u64,
    pub ability_id: &'static str,
    pub damage_effect_id: &'static str,
    /// Historical external claim retained for source audit only; execution
    /// derives the coefficient from the selected frozen native packet.
    pub external_atk_rate_claim: Ratio,
    pub energy_cost: u64,
    pub gear_procs: Vec<GearProcConfig>,
}

pub fn get_character_profile(slug: &str) -> Result<PreviewCharacterProfile, String> {
    match slug {
        "adler" => Ok(crate::characters::adler::preview::profile()),
        "akane" => Ok(crate::characters::akane::preview::profile()),
        "aurelia" => Ok(crate::characters::aurelia::preview::profile()),
        "baicang" => Ok(crate::characters::baicang::preview::profile()),
        "blackbird" => Ok(crate::characters::blackbird::preview::profile()),
        "chaos" => Ok(crate::characters::chaos::preview::profile()),
        "chiz" => Ok(crate::characters::chiz::preview::profile()),
        "daffodil" => Ok(crate::characters::daffodil::preview::profile()),
        "edgar" => Ok(crate::characters::edgar::preview::profile()),
        "fadia" => Ok(crate::characters::fadia::preview::profile()),
        "haniel" => Ok(crate::characters::haniel::preview::profile()),
        "hathor" => Ok(crate::characters::hathor::preview::profile()),
        "hotori" => Ok(crate::characters::hotori::preview::profile()),
        "iroi" => Ok(crate::characters::iroi::preview::profile()),
        "jiuyuan" => Ok(crate::characters::jiuyuan::preview::profile()),
        "lacrimosa" => Ok(crate::characters::lacrimosa::preview::profile()),
        "linko" => Ok(crate::characters::linko::preview::profile()),
        "mint" => Ok(crate::characters::mint::preview::profile()),
        "nanally" => Ok(crate::characters::nanally::preview::profile()),
        "sakiri" => Ok(crate::characters::sakiri::preview::profile()),
        "shinku" => Ok(crate::characters::shinku::preview::profile()),
        "skia" => Ok(crate::characters::skia::preview::profile()),
        "zankou" => Ok(crate::characters::zankou::preview::profile()),
        "zero" => Ok(crate::characters::zero::preview::profile()),
        other => Err(format!("unknown character slug: {other}")),
    }
}

pub fn validate_awakening_selection(
    character_id: &str,
    selection: &ntesim_data::AwakeningSelection,
) -> Result<(), String> {
    if !selection.is_prefix()
        && !matches!(
            character_id,
            "character:1003"
                | "character:1004"
                | "character:1008"
                | "character:1010"
                | "character:1019"
                | "character:1020"
                | "character:1021"
                | "character:1023"
                | "character:1025"
                | "character:1033"
                | "character:1036"
                | "character:1039"
                | "character:1042"
                | "character:1046"
                | "character:1052"
                | "character:1054"
                | "character:1055"
                | "character:1057"
                | "character:1070"
                | "character:1071"
                | "character:1072"
                | "character:1073"
                | "character:1075"
                | "character:1076"
        )
    {
        return Err(format!(
            "character {character_id} does not support selected awakening effects {selection:?}"
        ));
    }
    Ok(())
}
