use super::*;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CharacterRole {
    MainDps,
    SubDps,
    BufferSupport,
    HealerSupport,
    BreakEnabler,
}

pub fn get_character_role(slug: &str) -> CharacterRole {
    match slug {
        "sakiri" | "iroi" => CharacterRole::BufferSupport,
        "adler" | "daffodil" | "zankou" => CharacterRole::BreakEnabler,
        "fadia" | "haniel" | "linko" => CharacterRole::HealerSupport,
        "lacrimosa" | "zero" | "hathor" => CharacterRole::SubDps,
        _ => CharacterRole::MainDps,
    }
}

pub fn get_character_element(slug: &str) -> Result<String, String> {
    let element = ntesim_content::registry::get_character_descriptor(slug)?.element;
    let mut chars = element.chars();
    let first = chars.next().ok_or("character element missing")?;
    Ok(format!(
        "{}{}",
        first.to_ascii_uppercase(),
        chars.as_str().to_ascii_lowercase()
    ))
}
