use super::*;
use crate::simulation::GameActorConfig;

pub(crate) fn weapon_wearer_item<'a>(
    tables: &'a BTreeMap<String, Value>,
    character_id: &str,
    native_id: &str,
    label: &str,
) -> Result<&'a serde_json::Map<String, Value>, String> {
    let character = ntesim_data::character::definition(character_id)?;
    let item = ntesim_data::weapon_definitions::lookup(tables, native_id)?
        .1
        .as_object()
        .ok_or_else(|| format!("{label} weapon row {native_id} malformed"))?;
    let group = |value: &Value| -> Result<String, String> {
        let group = value
            .as_str()
            .ok_or_else(|| format!("{label} native group missing"))?;
        if !matches!(
            group,
            "ECharacterGroupType::CHARACTER_GROUP_TYPE_ONE"
                | "ECharacterGroupType::CHARACTER_GROUP_TYPE_TWO"
                | "ECharacterGroupType::CHARACTER_GROUP_TYPE_THREE"
                | "ECharacterGroupType::CHARACTER_GROUP_TYPE_FOUR"
                | "ECharacterGroupType::CHARACTER_GROUP_TYPE_FIVE"
        ) {
            return Err(format!("{label} unsupported native group"));
        }
        Ok(group.into())
    };
    if group(&item["ElementData"]["ApplyGroupType"])? != character.weapon_group.native_key() {
        return Err(format!("{label} group is incompatible with the wearer"));
    }
    Ok(item)
}

pub(crate) fn compile_static_hp_weapon(
    tables: &BTreeMap<String, Value>,
    schema_version: u32,
    content_id: &str,
    character_id: &str,
    weapon_id: &str,
    refinement: u32,
) -> Result<Option<Value>, String> {
    let (native_id, label, expected, hp_parameter, hypothesis_id) =
        if weapon_id.eq_ignore_ascii_case("weapon:fork_yuren") {
            (
                "fork_yuren",
                "Rainman",
                crate::weapons::yuren::projection::PARAMETERS,
                "buff_bowenchanggui_HP",
                crate::weapons::yuren::HYPOTHESIS_ID,
            )
        } else if weapon_id.eq_ignore_ascii_case("weapon:fork_Nakupeda") {
            (
                "fork_Nakupeda",
                "Nacupeda",
                crate::weapons::nakupeda::projection::PARAMETERS,
                "buff_Nakipeda_Hp",
                crate::weapons::nakupeda::HYPOTHESIS_ID,
            )
        } else {
            return Ok(None);
        };
    weapon_wearer_item(tables, character_id, native_id, label)?;
    let native = ntesim_data::weapon_passive_values::compile(
        tables,
        &ntesim_data::weapon_passive_values::Request {
            schema_version,
            content_id: content_id.into(),
            weapon_id: weapon_id.into(),
            refinement,
        },
    )?;
    let parameters = WeaponParameters::new(&native, expected, label)?;
    let bonus = ratio(parameters.fraction(hp_parameter)?)?;
    let parameter = native["parameters"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| {
            p["name_id"]
                .as_str()
                .is_some_and(|id| id.eq_ignore_ascii_case(hp_parameter))
        })
        .unwrap()
        .clone();
    Ok(Some(json!({"weapon_id":weapon_id,"refinement":refinement,
        "hp_bonus":bonus,"native_parameter":parameter,"native_parameter_report":native,
        "hypothesis_id":hypothesis_id,"hypothesis":hypothesis_id.description()})))
}

pub(crate) struct WeaponParameters<'a> {
    values: &'a [Value],
    label: &'a str,
}

impl<'a> WeaponParameters<'a> {
    pub(crate) fn new(
        native: &'a Value,
        expected: &[(&str, bool)],
        label: &'a str,
    ) -> Result<Self, String> {
        let values = native["parameters"]
            .as_array()
            .ok_or_else(|| format!("{label} parameters missing"))?;
        if values.len() != expected.len()
            || expected.iter().any(|(name, percent)| {
                values
                    .iter()
                    .filter(|value| {
                        value["name_id"]
                            .as_str()
                            .is_some_and(|id| id.eq_ignore_ascii_case(name))
                            && value["b_is_percent"] == *percent
                    })
                    .count()
                    != 1
            })
        {
            return Err(format!("unsupported {label} parameters or unit hint"));
        }
        Ok(Self { values, label })
    }

    pub(crate) fn selected(&self, name: &str) -> Result<&'a Value, String> {
        self.values
            .iter()
            .find(|value| {
                value["name_id"]
                    .as_str()
                    .is_some_and(|id| id.eq_ignore_ascii_case(name))
            })
            .map(|value| &value["selected"]["value"])
            .ok_or_else(|| format!("{} required parameter {name} missing", self.label))
    }

    pub(crate) fn fraction(&self, name: &str) -> Result<ExactFraction, String> {
        receipt(self.selected(name)?)
    }
}

pub(crate) fn weapon_seconds_to_us(seconds: ExactFraction, label: &str) -> Result<u64, String> {
    let micros = seconds.mul(ExactFraction::new(1_000_000, 1)?)?;
    if micros.denominator != 1 || micros.is_zero() {
        return Err(format!(
            "{label} requires positive integral microseconds without rounding"
        ));
    }
    u64::try_from(micros.numerator).map_err(|_| format!("{label} exceeds u64"))
}

pub(crate) fn compile_weapon_values(
    tables: &BTreeMap<String, Value>,
    schema_version: u32,
    content_id: &str,
    actor: &GameActorConfig,
) -> Result<Value, String> {
    ntesim_data::weapon_passive_values::compile(
        tables,
        &ntesim_data::weapon_passive_values::Request {
            schema_version,
            content_id: content_id.into(),
            weapon_id: actor.weapon_id.clone(),
            refinement: actor.weapon_tier,
        },
    )
}

pub(crate) fn finish_weapon_projection(
    actor: &mut GameActorConfig,
    positions: &[usize],
    proc: GearProcConfig,
    native: Value,
    metadata: Value,
) -> Result<Value, String> {
    let replaced = positions
        .first()
        .map(|index| {
            serde_json::to_value(&actor.gear_procs[*index]).map_err(|error| error.to_string())
        })
        .transpose()?;
    let mut report = json!({
        "slot": actor.slot,
        "weapon_id": actor.weapon_id,
        "refinement": actor.weapon_tier,
        "replaced_proc": replaced,
        "compiled_proc": proc,
        "native_parameter_report": native,
    });
    report.as_object_mut().unwrap().extend(
        metadata
            .as_object()
            .ok_or("weapon projection metadata must be an object")?
            .clone(),
    );
    if let Some(index) = positions.first() {
        actor.gear_procs[*index] = proc;
    } else {
        actor.gear_procs.push(proc);
    }
    Ok(report)
}
