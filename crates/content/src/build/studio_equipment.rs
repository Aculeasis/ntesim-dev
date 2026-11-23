//! Bind selected equipment through the same owner descriptors as catalog export.
use super::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum EquipmentProjection {
    Supported {
        gear_procs: Vec<GearProcConfig>,
        model_assumptions: BTreeMap<String, String>,
    },
    Unsupported {
        diagnostic: String,
    },
}

pub(super) fn describe(
    tables: &BTreeMap<String, Value>,
    request: &studio_projection::Request,
    efficiency: Ratio,
) -> EquipmentProjection {
    let mut model_assumptions = BTreeMap::new();
    match project(tables, request, efficiency, &mut model_assumptions) {
        Ok(gear_procs) => EquipmentProjection::Supported {
            gear_procs,
            model_assumptions,
        },
        Err(diagnostic) => EquipmentProjection::Unsupported { diagnostic },
    }
}

pub(super) fn project(
    tables: &BTreeMap<String, Value>,
    request: &studio_projection::Request,
    efficiency: Ratio,
    assumptions: &mut BTreeMap<String, String>,
) -> Result<Vec<GearProcConfig>, String> {
    let mut procs = Vec::new();
    if efficiency.numerator != efficiency.denominator {
        procs.push(GearProcConfig::ChargeEfficiency {
            multiplier: efficiency,
        });
        let model = crate::HypothesisId::EnergyEfficiencyCarryV1;
        assumptions.insert(model.as_str().into(), model.description().into());
    }
    if request.weapon_id.is_none() && request.set_id.is_none() {
        return Ok(procs);
    }
    let catalog = studio::gear_catalog(
        tables,
        &request.content_id,
        Some(request.weapon_id.as_deref().unwrap_or("")),
    )?;
    if let Some(weapon) = &request.weapon_id {
        let proc = catalog["weapon_procs"][weapon.to_ascii_lowercase()]
            .get(request.refinement.saturating_sub(1) as usize)
            .ok_or_else(|| format!("Equipped weapon passive is unsupported: {weapon}."))?;
        procs.push(serde_json::from_value(proc.clone()).map_err(|error| error.to_string())?);
    }
    if let Some(set) = &request.set_id {
        for effect in catalog["sets"][set]
            .as_array()
            .ok_or("set descriptor missing")?
        {
            let threshold = effect["threshold"]
                .as_u64()
                .ok_or("set threshold missing")?;
            if u64::from(request.set_count) < threshold || effect["has_buff"] != true {
                continue;
            }
            let descriptor = &catalog["set_procs"][set];
            if descriptor["threshold"].as_u64() != Some(threshold) {
                return Err(format!(
                    "Equipped set effect is unsupported: {set} ({threshold})."
                ));
            }
            let selected: Vec<GearProcConfig> = serde_json::from_value(descriptor["procs"].clone())
                .map_err(|error| error.to_string())?;
            procs.extend(selected);
        }
    }
    for proc in &procs {
        let serialized = serde_json::to_value(proc).map_err(|error| error.to_string())?;
        let kind = serialized["kind"]
            .as_str()
            .ok_or("gear proc kind missing")?;
        if let Some(values) = catalog["gear_assumptions"][kind].as_object() {
            for (id, text) in values {
                assumptions.insert(
                    id.clone(),
                    text.as_str().ok_or("gear assumption missing")?.into(),
                );
            }
        }
        if matches!(
            proc,
            GearProcConfig::BlastCandyAtk { .. }
                | GearProcConfig::GoodBoyTeamAtk { .. }
                | GearProcConfig::SpeedyHedgehogTeamAtk { .. }
                | GearProcConfig::LostRadianceDefIgnore { .. }
                | GearProcConfig::Twinbirds { .. }
                | GearProcConfig::LunarPhase { .. }
                | GearProcConfig::HpSuitMaxHp { .. }
                | GearProcConfig::CrimsonAtk { .. }
        ) {
            assumptions.insert(
                crate::gear::CAST_BUFF_HYPOTHESIS_ID.as_str().into(),
                crate::gear::CAST_BUFF_HYPOTHESIS_ID.description().into(),
            );
        }
    }
    Ok(procs)
}
