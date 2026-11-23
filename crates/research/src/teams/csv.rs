use super::*;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct PublishedVariant {
    pub variant_index: usize,
    pub archetype: String,
    pub tier: String,
    pub slots: [String; 4],
}

pub fn parse_published_variants_csv(csv_content: &str) -> Result<Vec<PublishedVariant>, String> {
    let mut variants = Vec::new();
    let mut lines = csv_content.lines();
    let header = lines
        .next()
        .ok_or("empty csv file")?
        .trim_start_matches('\u{feff}')
        .trim();
    if header != "archetype,tier,slot1,slot2,slot3,slot4" {
        return Err(format!("unexpected csv header: {header}"));
    }

    for (idx, line) in lines.enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        if cols.len() != 6 {
            return Err(format!(
                "line {} has {} columns, expected 6",
                idx + 2,
                cols.len()
            ));
        }
        let archetype = cols[0].to_string();
        let tier = cols[1].to_string();
        let slots = [
            cols[2].to_string(),
            cols[3].to_string(),
            cols[4].to_string(),
            cols[5].to_string(),
        ];

        for s in &slots {
            get_character_profile(s)?;
        }

        let mut set = std::collections::BTreeSet::new();
        for s in &slots {
            if !set.insert(s.clone()) {
                return Err(format!("duplicate character in variant {}: {s}", idx + 1));
            }
        }

        variants.push(PublishedVariant {
            variant_index: idx + 1,
            archetype,
            tier,
            slots,
        });
    }

    if variants.is_empty() {
        return Err("no valid variants found in csv".into());
    }

    Ok(variants)
}
