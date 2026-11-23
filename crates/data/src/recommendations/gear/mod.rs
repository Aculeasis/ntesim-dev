use crate::{
    equipment,
    native::{Table, text},
    weapon,
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

pub const TABLES: [&str; 0] = [];

fn csv_records(csv: &str) -> Result<Vec<Vec<String>>, String> {
    let mut chars = csv.trim_start_matches('\u{feff}').chars().peekable();
    let mut records = Vec::new();
    let mut row = Vec::new();
    let mut field = String::new();
    let mut quoted = false;
    let mut closed = false;
    while let Some(c) = chars.next() {
        if quoted {
            if c == '"' {
                if chars.peek() == Some(&'"') {
                    chars.next();
                    field.push('"');
                } else {
                    quoted = false;
                    closed = true;
                }
            } else {
                field.push(c);
            }
            continue;
        }
        if closed && !matches!(c, ',' | '\r' | '\n') {
            return Err("unexpected text after quoted CSV field".into());
        }
        match c {
            '"' if field.is_empty() && !closed => quoted = true,
            '"' => return Err("quote inside unquoted CSV field".into()),
            ',' => {
                row.push(std::mem::take(&mut field));
                closed = false;
            }
            '\r' | '\n' => {
                if c == '\r' && chars.next() != Some('\n') {
                    return Err("CSV row CR must be followed by LF".into());
                }
                row.push(std::mem::take(&mut field));
                records.push(std::mem::take(&mut row));
                closed = false;
            }
            _ => field.push(c),
        }
    }
    if quoted {
        return Err("unterminated quoted CSV field".into());
    }
    if !row.is_empty() || !field.is_empty() || closed {
        row.push(field);
        records.push(row);
    }
    Ok(records)
}

fn field_evidence(table: &Table, id: &str, field: &str) -> Value {
    let mut evidence = table.evidence(id);
    evidence["pointer"] = json!(format!("{}{field}", evidence["pointer"].as_str().unwrap()));
    evidence
}

fn boolean(value: &str) -> Result<bool, String> {
    match value {
        "True" => Ok(true),
        "False" => Ok(false),
        _ => Err("recommendation boolean must be True or False".into()),
    }
}

pub fn bind(
    tables: &BTreeMap<String, Value>,
    coverage: &Value,
    csv: &str,
) -> Result<Value, String> {
    if csv.len() > 1024 * 1024 {
        return Err("recommended gear CSV exceeds 1 MiB".into());
    }
    let records = csv_records(csv)?;
    let header = [
        "character",
        "kind",
        "name",
        "requested_T",
        "level",
        "catalogue_match",
        "source_url",
        "optimizer_selected",
    ];
    if records
        .first()
        .map(|v| v.iter().map(String::as_str).collect::<Vec<_>>())
        != Some(header.to_vec())
        || !(2..=1001).contains(&records.len())
    {
        return Err("unsupported recommended gear CSV header or record count".into());
    }
    let entities = coverage["entities"]
        .as_array()
        .ok_or("coverage entities missing")?;
    let weapons = &crate::weapon_definitions::table(tables)?;
    let sets = &crate::set_definitions::table(tables)?;
    let mut output = Vec::new();
    let mut seen = BTreeSet::new();
    let mut character_domains = BTreeMap::<String, (String, usize, usize)>::new();
    let mut weapon_count = 0;
    let mut set_count = 0;
    for (index, row) in records[1..].iter().enumerate() {
        if row.len() != 8 || row[0].is_empty() || row[2].is_empty() || row[6].is_empty() {
            return Err(format!("invalid recommendation CSV record {}", index + 2));
        }
        let released = entities
            .iter()
            .filter(|e| {
                e["kind"] == "character" && e["slug"] == row[0] && e["availability"] == "released"
            })
            .collect::<Vec<_>>();
        if released.len() != 1 {
            return Err(format!(
                "recommendation slug must identify one released character: {}",
                row[0]
            ));
        }
        let character = released[0];
        let character_id = text(character, "id")?;
        let native_id = crate::character::definition(character_id)?.native_id;
        if character_id != format!("character:{native_id}") {
            return Err("recommendation character coverage ID is not canonical".into());
        }
        let level = row[4]
            .parse::<u32>()
            .map_err(|_| "recommendation level must be integer")?;
        let catalogue_match = boolean(&row[5])?;
        let optimizer_selected = boolean(&row[7])?;
        let domain = character_domains
            .entry(row[0].clone())
            .or_insert((character_id.into(), 0, 0));
        let binding = match row[1].as_str() {
            "arc" => {
                let tier = row[3]
                    .parse::<u32>()
                    .map_err(|_| "recommended weapon refinement must be integer")?;
                if level != 80 || !(1..=5).contains(&tier) {
                    return Err("recommended weapon binding supports level 80 and T1..T5".into());
                }
                let matches = weapons
                    .rows()
                    .iter()
                    .filter(|(_, value)| value["ItemName"]["LocalizedString"] == row[2])
                    .collect::<Vec<_>>();
                if matches.len() != 1 {
                    return Err(format!(
                        "weapon name requires one exact native display match: {}",
                        row[2]
                    ));
                }
                let (id, _) = matches[0];
                let weapon_id = format!("weapon:{id}");
                let groups = equipment::weapon_group_binding(tables, character_id, &weapon_id)?;
                let constants = weapon::compile(
                    tables,
                    &weapon::Request {
                        schema_version: 1,
                        content_id: "ntesim-current".into(),
                        weapon_id: weapon_id.clone(),
                        level,
                        breakthrough: 6,
                        refinement: tier,
                    },
                )?;
                weapon_count += 1;
                domain.1 += 1;
                json!({"id":weapon_id, "level":level, "refinement":tier, "breakthrough":6,
                    "name_evidence":field_evidence(weapons, id, "/ItemName/LocalizedString"),
                    "group_binding":groups, "constants":constants, "full_build_legality_checked":false})
            }
            "set" => {
                if !row[3].is_empty() || level != 20 {
                    return Err(
                        "recommended set binding requires empty T and equipment level 20".into(),
                    );
                }
                let matches = sets
                    .rows()
                    .iter()
                    .filter(|(_, value)| value["SuitTitle"]["LocalizedString"] == row[2])
                    .collect::<Vec<_>>();
                if matches.len() != 1 {
                    return Err(format!(
                        "set name requires one exact native display match: {}",
                        row[2]
                    ));
                }
                let (id, set) = matches[0];
                set_count += 1;
                domain.2 += 1;
                json!({"id":format!("set:{id}"), "equipment_level":level, "set_count":4,
                    "name_evidence":field_evidence(sets, id, "/SuitTitle/LocalizedString"),
                    "native_set_row":set, "evidence":[sets.evidence(id)], "full_build_legality_checked":false})
            }
            _ => return Err("recommended gear kind must be arc or set".into()),
        };
        let canonical = binding["id"]
            .as_str()
            .ok_or("bound gear identity missing")?;
        if !seen.insert((
            row[0].clone(),
            row[1].clone(),
            canonical.to_owned(),
            binding["refinement"].as_u64().unwrap_or(0),
            level,
        )) {
            return Err("duplicate canonical recommended gear choice".into());
        }
        output.push(json!({"csv_record":index+2, "slug":row[0], "character_id":character_id,
            "kind":row[1], "name":row[2], "source_url":row[6],
            "author_catalogue_match":catalogue_match, "author_optimizer_selected":optimizer_selected,
            "release_evidence":crate::character::definition(character_id)?.evidence(),
            "binding":binding}));
    }
    if character_domains
        .values()
        .any(|(_, weapons, sets)| *weapons == 0 || *sets == 0)
    {
        return Err("each recommended character must have weapon and set choices".into());
    }
    Ok(
        json!({"kind":"recommended-gear-domain-v1", "schema_version":1, "content_id":"ntesim-current",
        "status":"INCOMPLETE", "quality":"incomplete", "game_dps_computed":false,
        "rows":output, "recommendation_records":output.len(), "weapon_choices":weapon_count, "set_choices":set_count,
        "characters":character_domains.iter().map(|(slug,(id,weapon_count,set_count))|
            json!({"slug":slug,"character_id":id,"weapon_choices":weapon_count,"set_choices":set_count})).collect::<Vec<_>>(),
        "character_count":character_domains.len(), "full_builds_generated":false, "recommended_loadouts_validated":false,
        "scope":"Exact source display-name and native-group binding; terminal weapon stage 6 policy. CSV author flags are retained claims, not legality/release/optimality evidence. Native allocation/passive/application, legal packing/substats and strict gameplay remain unresolved."}),
    )
}

#[cfg(test)]
mod tests;
