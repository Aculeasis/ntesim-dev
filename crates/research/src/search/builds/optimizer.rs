use super::*;

pub fn optimize_character_build(slug: &str) -> Result<CharacterBuildProfile, String> {
    Err(format!(
        "legal build search is unsupported for {slug}: no executable optimizer is bound to the frozen equipment compiler"
    ))
}

pub fn search_and_optimize_all_characters()
-> Result<BTreeMap<String, CharacterBuildProfile>, String> {
    Err("legal build search is unsupported: no executable optimizer is bound to the frozen equipment compiler".into())
}
