use super::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PublishedVariantRow {
    pub archetype: String,
    pub tier: String,
    pub slot1: String,
    pub slot2: String,
    pub slot3: String,
    pub slot4: String,
    pub status: String,
    pub dps_1: Option<f64>,
    pub dps_4: Option<f64>,
    pub uncertainty: String,
    pub gear_optimized: bool,
    pub rotation_validated: bool,
    pub character_contributions: String,
    pub reaction_damage: String,
    pub break_damage: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ArchetypeSummaryRow {
    pub archetype: String,
    pub tier: String,
    pub valid_variants: usize,
    pub best_team: String,
    pub dps_1: Option<f64>,
    pub dps_4: Option<f64>,
    pub dps_first20: Option<f64>,
    pub dps_steady: Option<f64>,
    pub uncertainty: String,
    pub quality: String,
}

pub fn generate_published_variants_csv(variants: &[PublishedVariant]) -> String {
    let mut out = String::with_capacity(variants.len() * 120);
    out.push_str("archetype,tier,slot1,slot2,slot3,slot4,status,dps_1,dps_4,uncertainty,gear_optimized,rotation_validated,character_contributions,reaction_damage,break_damage\n");

    for v in variants {
        out.push_str(&format!(
            "{},{},{},{},{},{},INCOMPLETE,,,не ограничена,False,False,,,\n",
            v.archetype, v.tier, v.slots[0], v.slots[1], v.slots[2], v.slots[3]
        ));
    }

    out
}

pub fn generate_archetypes_csv(variants: &[PublishedVariant]) -> String {
    let mut archetype_counts = BTreeMap::new();
    let mut archetype_order = Vec::new();
    let mut archetype_tier = BTreeMap::new();

    for v in variants {
        if !archetype_counts.contains_key(&v.archetype) {
            archetype_order.push(v.archetype.clone());
            archetype_tier.insert(v.archetype.clone(), v.tier.clone());
        }
        *archetype_counts
            .entry(v.archetype.clone())
            .or_insert(0usize) += 1;
    }

    let mut out = String::with_capacity(archetype_order.len() * 120);
    out.push_str("archetype,tier,valid_variants,best_team,dps_1,dps_4,dps_first20,dps_steady,uncertainty,quality\n");

    for name in &archetype_order {
        let tier = &archetype_tier[name];
        let count = archetype_counts[name];
        out.push_str(&format!(
            "{},{},{},не определена,,,,,Нет обоснованного интервала,Неполные данные\n",
            name, tier, count
        ));
    }

    out
}
