use super::*;
pub(crate) fn set_stats(
    tables: &BTreeMap<String, Value>,
    set_id: &str,
    count: u32,
) -> Result<Value, String> {
    if count > 4 {
        return Err("set count must be in 0..=4".into());
    }
    let suits = crate::set_definitions::table(tables)?;
    let (id, row) = suits.lookup(
        set_id
            .strip_prefix("set:")
            .ok_or("canonical set ID required")?,
    )?;
    let mut attributes = BTreeMap::new();
    let mut evidence = vec![suits.evidence(id)];
    let mut contributions = Vec::new();
    let mut buffs = Vec::new();
    for effect in row["SuitStructList"]
        .as_array()
        .ok_or("set effects missing")?
    {
        let threshold = effect["SuitCondition"]
            .as_u64()
            .ok_or("set threshold missing")?;
        if !matches!(threshold, 2 | 4) {
            return Err("set threshold must be 2 or 4".into());
        }
        if u64::from(count) < threshold {
            continue;
        }
        let pack = text(effect, "SuitModifyPackID")?;
        if pack != "None" {
            let (values, receipt) = crate::set_modifiers::compile_scalars(tables, pack)?;
            for (name, value) in &values {
                accumulate_scalar(&mut attributes, name, *value)?;
            }
            contributions.push(json!({"category":"set_unconditional_pack","threshold":threshold,
                "attributes":values.iter().map(|(k,v)|(k.clone(),v.receipt())).collect::<BTreeMap<_,_>>(),"evidence":[receipt.clone()]}));
            evidence.push(receipt);
        }
        if !effect["SuitBuff"].is_null() {
            buffs.push(json!({"threshold":threshold,"buff":effect["SuitBuff"],"description":effect["SuitBuffDescription"]["LocalizedString"]}));
        }
    }
    if id == "Suit5" && count >= 4 {
        let report = crate::set_passive_values::street_boxer(tables)?;
        let parameter = &report["parameters"]["Lakshana_4_1"];
        let selected = &parameter["selected"];
        let numerator = selected["numerator"]
            .as_str()
            .ok_or("Street Boxer baseline numerator missing")?
            .parse::<u128>()
            .map_err(|_| "Street Boxer baseline numerator invalid")?;
        let denominator = selected["denominator"]
            .as_str()
            .ok_or("Street Boxer baseline denominator missing")?
            .parse::<u128>()
            .map_err(|_| "Street Boxer baseline denominator invalid")?;
        if numerator == 0 || numerator > denominator {
            return Err("Street Boxer static baseline must be a positive fraction <=1".into());
        }
        let baseline = crate::native::from_receipt(selected)?;
        accumulate_scalar(&mut attributes, "CritBase", baseline)?;
        contributions.push(
            json!({"category":"set_unconditional_buff_base","threshold":4,
            "attributes":{"CritBase":baseline.receipt()},"evidence":[parameter["evidence"]]}),
        );
        evidence.push(parameter["evidence"].clone());
    }
    Ok(json!({"set_id":format!("set:{id}"),"set_count":count,
        "attributes":attributes.iter().map(|(k,v)|(k.clone(),v.receipt())).collect::<BTreeMap<_,_>>(),
        "contributions":contributions,"buffs":buffs,"evidence":evidence}))
}
