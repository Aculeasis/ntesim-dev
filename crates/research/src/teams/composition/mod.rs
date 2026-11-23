use crate::published_teams::parse_published_variants_csv;
use ntesim_contracts::{ENGINE_SOURCE_SHA256, ENGINE_VERSION, Quality};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

pub const DEFAULT_REASON: &str =
    "timings; energy; hit geometry; kit adapters; Global applicability";

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct MatchedVariantInfo {
    pub variant_index: usize,
    pub archetype: String,
    pub tier: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CompositionRecord {
    pub index: usize,
    pub composition: String,
    pub slots: [String; 4],
    pub character_ids: [String; 4],
    pub status: String,
    pub quality: Quality,
    pub game_dps_computed: bool,
    pub is_published_variant: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub matched_variant: Option<MatchedVariantInfo>,
    pub dps_1_real120: Option<f64>,
    pub dps_4_real120: Option<f64>,
    pub dps_1_mode120: Option<f64>,
    pub dps_4_mode120: Option<f64>,
    pub dps_no_break: Option<f64>,
    pub reason: String,
    pub rotation_search_performed: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CompositionReport {
    pub schema_version: u32,
    pub content_id: String,
    pub status: String,
    pub quality: Quality,
    pub game_dps_computed: bool,
    pub acceptance_ready: bool,
    pub total_characters: usize,
    pub total_compositions: usize,
    pub published_variants_count: usize,
    pub matched_published_compositions: usize,
    pub unconventional_compositions: usize,
    pub distinct_archetypes_count: usize,
    pub csv_sha256: String,
    pub compositions: Vec<CompositionRecord>,
    pub engine_version: String,
    pub engine_source_sha256: String,
}

pub fn enumerate_compositions(
    coverage: &Value,
    variants_csv: &str,
) -> Result<CompositionReport, String> {
    let entities = coverage["entities"]
        .as_array()
        .ok_or("coverage entities missing")?;
    let mut released_char_ids = BTreeSet::new();
    for e in entities {
        if e["kind"] == "character" && e["availability"] == "released" {
            let id = e["id"].as_str().ok_or("character id missing")?;
            released_char_ids.insert(id.to_string());
        }
    }

    if released_char_ids.len() < 4 {
        return Err("composition enumeration requires at least four released characters".into());
    }

    // Content owns the registry; enumeration consumes the released intersection.
    let mut slug_to_id = BTreeMap::new();
    for descriptor in ntesim_content::registry::character_descriptors()? {
        if released_char_ids.contains(descriptor.character_id) {
            slug_to_id.insert(
                descriptor.slug.to_string(),
                descriptor.character_id.to_string(),
            );
        }
    }
    if slug_to_id.values().collect::<BTreeSet<_>>().len() != released_char_ids.len() {
        return Err(
            "released coverage contains identities missing from the content registry".into(),
        );
    }
    let sorted_slugs = slug_to_id.keys().cloned().collect::<Vec<_>>();

    let variants = parse_published_variants_csv(variants_csv)?;
    let mut variant_by_set: BTreeMap<BTreeSet<String>, MatchedVariantInfo> = BTreeMap::new();
    let mut archetypes = BTreeSet::new();

    for v in &variants {
        if let Some(slug) = v.slots.iter().find(|slug| !slug_to_id.contains_key(*slug)) {
            return Err(format!(
                "variant references a character outside released inventory: {slug}"
            ));
        }
        let set: BTreeSet<String> = v.slots.iter().cloned().collect();
        archetypes.insert(v.archetype.clone());
        variant_by_set.insert(
            set,
            MatchedVariantInfo {
                variant_index: v.variant_index,
                archetype: v.archetype.clone(),
                tier: v.tier.clone(),
            },
        );
    }

    let n = sorted_slugs.len();
    let mut compositions = Vec::new();
    let mut matched_count = 0usize;
    let mut comp_index = 1usize;

    for i in 0..n {
        for j in (i + 1)..n {
            for k in (j + 1)..n {
                for l in (k + 1)..n {
                    let s0 = sorted_slugs[i].to_string();
                    let s1 = sorted_slugs[j].to_string();
                    let s2 = sorted_slugs[k].to_string();
                    let s3 = sorted_slugs[l].to_string();

                    let comp_str = format!("{s0}+{s1}+{s2}+{s3}");
                    let id0 = slug_to_id[&s0].clone();
                    let id1 = slug_to_id[&s1].clone();
                    let id2 = slug_to_id[&s2].clone();
                    let id3 = slug_to_id[&s3].clone();

                    let set = BTreeSet::from([s0.clone(), s1.clone(), s2.clone(), s3.clone()]);
                    let matched_variant = variant_by_set.get(&set).cloned();
                    let is_published = matched_variant.is_some();
                    if is_published {
                        matched_count += 1;
                    }

                    compositions.push(CompositionRecord {
                        index: comp_index,
                        composition: comp_str,
                        slots: [s0, s1, s2, s3],
                        character_ids: [id0, id1, id2, id3],
                        status: "INCOMPLETE".into(),
                        quality: Quality::Incomplete,
                        game_dps_computed: false,
                        is_published_variant: is_published,
                        matched_variant,
                        dps_1_real120: None,
                        dps_4_real120: None,
                        dps_1_mode120: None,
                        dps_4_mode120: None,
                        dps_no_break: None,
                        reason: DEFAULT_REASON.into(),
                        rotation_search_performed: false,
                    });

                    comp_index += 1;
                }
            }
        }
    }

    let unconventional = compositions.len() - matched_count;
    let csv_text = generate_csv_from_records(&compositions);
    let csv_sha256 = ntesim_contracts::sha256(csv_text.as_bytes());

    Ok(CompositionReport {
        schema_version: 1,
        content_id: "ntesim-current".into(),
        status: "INCOMPLETE".into(),
        quality: Quality::Incomplete,
        game_dps_computed: false,
        acceptance_ready: false,
        total_characters: n,
        total_compositions: compositions.len(),
        published_variants_count: variants.len(),
        matched_published_compositions: matched_count,
        unconventional_compositions: unconventional,
        distinct_archetypes_count: archetypes.len(),
        csv_sha256,
        compositions,
        engine_version: ENGINE_VERSION.to_string(),
        engine_source_sha256: ENGINE_SOURCE_SHA256.to_string(),
    })
}

pub fn generate_csv_from_records(records: &[CompositionRecord]) -> String {
    let mut out = String::with_capacity(records.len() * 120);
    out.push_str("composition,status,dps_1_real120,dps_4_real120,dps_1_mode120,dps_4_mode120,dps_no_break,reason,rotation_search_performed\n");

    for c in records {
        let fmt_opt =
            |opt: Option<f64>| -> String { opt.map(|v| format!("{v:.6}")).unwrap_or_default() };
        let dps_1_real120 = fmt_opt(c.dps_1_real120);
        let dps_4_real120 = fmt_opt(c.dps_4_real120);
        let dps_1_mode120 = fmt_opt(c.dps_1_mode120);
        let dps_4_mode120 = fmt_opt(c.dps_4_mode120);
        let dps_no_break = fmt_opt(c.dps_no_break);
        let rotation_search = if c.rotation_search_performed {
            "True"
        } else {
            "False"
        };

        out.push_str(&format!(
            "{},{},{},{},{},{},{},{},{}\n",
            c.composition,
            c.status,
            dps_1_real120,
            dps_4_real120,
            dps_1_mode120,
            dps_4_mode120,
            dps_no_break,
            c.reason,
            rotation_search
        ));
    }

    out
}

pub fn generate_csv(report: &CompositionReport) -> String {
    generate_csv_from_records(&report.compositions)
}

#[cfg(test)]
mod tests;
