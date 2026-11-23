use crate::{
    build, equipment,
    execution::ExecutionLimits,
    published_teams::{self, ActorOverride},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub const TABLES: [&str; equipment::TABLES.len()] = equipment::TABLES;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub schema_version: u32,
    pub content_id: String,
    /// Explicit static build projection hypothesis.
    pub hypothesis: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variants_csv: Option<String>,
    pub builds: BTreeMap<String, equipment::Request>,
}

pub fn run(
    tables: &BTreeMap<String, Value>,
    coverage: &Value,
    default_csv: &str,
    request: &Request,
    limits: &ExecutionLimits,
) -> Result<Value, String> {
    if request.schema_version != 1 {
        return Err("built variants requires schema_version 1".into());
    }
    if request.content_id != "ntesim-current" {
        return Err("unsupported content_id in built variants request".into());
    }
    if request.hypothesis.trim().is_empty() {
        return Err("built variants requires an explicit hypothesis".into());
    }
    if request.builds.is_empty() {
        return Err("built variants requires at least one character build".into());
    }

    let csv_content = request.variants_csv.as_deref().unwrap_or(default_csv);
    let variants = published_teams::parse_published_variants_csv(csv_content)?;

    let mut actor_overrides = BTreeMap::new();
    let mut build_receipts = BTreeMap::new();

    for (slug, build_req) in &request.builds {
        if build_req.content_id != request.content_id {
            return Err(format!("build content_id mismatch for character {slug}"));
        }
        let profile = published_teams::get_character_profile(slug)
            .map_err(|_| format!("unknown character slug in builds: {slug}"))?;
        if build_req.character_id != profile.character_id {
            return Err(format!(
                "build character_id {} disagrees with canonical profile character_id {} for slug {slug}",
                build_req.character_id, profile.character_id
            ));
        }

        let released = coverage["entities"]
            .as_array()
            .ok_or("coverage entities missing")?
            .iter()
            .any(|e| e["id"] == build_req.character_id && e["availability"] == "released");
        if !released {
            return Err(format!("character {slug} is not in released inventory"));
        }

        let report = equipment::compile(tables, build_req)?;

        let native_element = ntesim_data::character::definition(&build_req.character_id)?
            .element
            .native_key();
        let element_bonus_attr = build::element_attribute(native_element)?;
        let static_stats = build::project(&report, element_bonus_attr)?;

        actor_overrides.insert(
            slug.clone(),
            ActorOverride {
                weapon_id: Some(build_req.weapon_id.clone()),
                weapon_tier: Some(build_req.weapon_tier),
                set_id: Some(build_req.set_id.clone()),
                static_stats: static_stats.clone(),
            },
        );

        build_receipts.insert(
            slug.clone(),
            json!({
                "slug": slug,
                "character_id": profile.character_id,
                "weapon_id": build_req.weapon_id,
                "weapon_tier": build_req.weapon_tier,
                "set_id": build_req.set_id,
                "native_element": native_element,
                "element_bonus_attribute": element_bonus_attr,
                "compiled_static_stats": static_stats,
                "build_report": report,
            }),
        );
    }

    for variant in &variants {
        for slug in &variant.slots {
            if !actor_overrides.contains_key(slug) {
                return Err(format!(
                    "variant {} ({}) contains character '{slug}' without an audited build override",
                    variant.variant_index, variant.archetype
                ));
            }
        }
    }

    let batch_report = published_teams::run_published_batch_with_actor_overrides(
        csv_content,
        Some(&actor_overrides),
        limits,
    )?;

    Ok(json!({
        "kind": "published-built-variants-v1",
        "schema_version": request.schema_version,
        "content_id": request.content_id,
        "hypothesis": request.hypothesis,
        "status": "INCOMPLETE",
        "quality": "incomplete",
        "game_dps_computed": false,
        "acceptance_ready": false,
        "audited_characters_count": actor_overrides.len(),
        "character_build_receipts": build_receipts,
        "batch": batch_report,
    }))
}

#[cfg(test)]
mod tests;
