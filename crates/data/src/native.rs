use serde_json::{Value, json};
use std::{borrow::Cow, collections::BTreeMap, sync::OnceLock};

mod curves;
pub use curves::{constant_curve_receipt, curve_selection_receipt};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Scalar {
    n: u128,
    d: u128,
}

/// Exact non-negative native scalar parsed from serde_json's normalized
/// numeric spelling. This preserves decimal and exponent forms without a
/// binary floating-point conversion.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NativeScalar {
    numerator: u128,
    denominator: u128,
}

impl NativeScalar {
    pub fn numerator(self) -> u128 {
        self.numerator
    }

    pub fn denominator(self) -> u128 {
        self.denominator
    }
}

fn gcd(mut a: u128, mut b: u128) -> u128 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

impl Scalar {
    pub(crate) fn new(n: u128, d: u128) -> Self {
        let g = gcd(n, d);
        Self { n: n / g, d: d / g }
    }

    // Exact arithmetic over serde_json's normalized numeric spelling. This is
    // not a claim to recover the original native binary floating-point value.
    pub(crate) fn parse(value: &Value) -> Result<Self, String> {
        let text = value
            .as_number()
            .ok_or("native constant scalar must be numeric")?
            .to_string();
        if text.starts_with('-') {
            return Err("negative native constant requires a separate contract".into());
        }
        let (mantissa, exponent) = match text.split_once(['e', 'E']) {
            Some((m, e)) => (m, e.parse::<i32>().map_err(|_| "invalid scalar exponent")?),
            None => (text.as_str(), 0),
        };
        let fraction = mantissa.split_once('.').map_or(0, |(_, f)| f.len());
        let digits = mantissa.replace('.', "");
        let mut n = digits
            .parse::<u128>()
            .map_err(|_| "native constant scalar overflow")?;
        let scale = i32::try_from(fraction)
            .map_err(|_| "native constant scalar overflow")?
            .checked_sub(exponent)
            .ok_or("native constant scalar overflow")?;
        let power = 10_u128
            .checked_pow(scale.unsigned_abs())
            .ok_or("native constant scalar overflow")?;
        let d = if scale >= 0 {
            power
        } else {
            n = n
                .checked_mul(power)
                .ok_or("native constant scalar overflow")?;
            1
        };
        Ok(Self::new(n, d))
    }

    pub(crate) fn add(self, rhs: Self) -> Result<Self, String> {
        let g = gcd(self.d, rhs.d);
        let n = self
            .n
            .checked_mul(rhs.d / g)
            .and_then(|left| {
                rhs.n
                    .checked_mul(self.d / g)
                    .and_then(|right| left.checked_add(right))
            })
            .ok_or("native constant sum overflow")?;
        let d = (self.d / g)
            .checked_mul(rhs.d)
            .ok_or("native constant sum overflow")?;
        Ok(Self::new(n, d))
    }

    pub(crate) fn zero() -> Self {
        Self { n: 0, d: 1 }
    }

    pub(crate) fn one() -> Self {
        Self { n: 1, d: 1 }
    }

    pub(crate) fn to_f64(self) -> f64 {
        self.n as f64 / self.d as f64
    }

    pub(crate) fn mul(self, rhs: Self) -> Result<Self, String> {
        let g1 = gcd(self.n, rhs.d);
        let g2 = gcd(rhs.n, self.d);
        let n = (self.n / g1)
            .checked_mul(rhs.n / g2)
            .ok_or("native constant multiplication overflow")?;
        let d = (self.d / g2)
            .checked_mul(rhs.d / g1)
            .ok_or("native constant multiplication overflow")?;
        Ok(Self::new(n, d))
    }

    pub(crate) fn receipt(self) -> Value {
        json!({"numerator":self.n.to_string(),"denominator":self.d.to_string()})
    }
}

/// Parse a native JSON number using the shared exact scalar contract.
///
/// Strings are intentionally rejected: native numeric fields must retain
/// their serialized JSON number shape. Exponents are accepted and normalized
/// exactly, subject to checked `u128` bounds.
pub fn parse_native_scalar(value: &Value) -> Result<NativeScalar, String> {
    let scalar = Scalar::parse(value)?;
    Ok(NativeScalar {
        numerator: scalar.n,
        denominator: scalar.d,
    })
}

pub(crate) struct Table<'a> {
    name: &'static str,
    rows: &'a serde_json::Map<String, Value>,
    index: Cow<'a, BTreeMap<String, &'a str>>,
}

type FrozenIndexes = BTreeMap<String, Result<BTreeMap<String, &'static str>, String>>;
static FROZEN_INDEXES: OnceLock<FrozenIndexes> = OnceLock::new();

fn row_index<'a>(
    name: &str,
    rows: &'a serde_json::Map<String, Value>,
) -> Result<BTreeMap<String, &'a str>, String> {
    let mut index = BTreeMap::new();
    for key in rows.keys() {
        if !key.is_ascii()
            || index
                .insert(key.to_ascii_lowercase(), key.as_str())
                .is_some()
        {
            return Err(format!(
                "ambiguous or non-ASCII native constant table key: {name}:{key}"
            ));
        }
    }
    Ok(index)
}

impl<'a> Table<'a> {
    pub(crate) fn new(name: &'static str, value: &'a Value) -> Result<Self, String> {
        if value["schema_version"] != 1 || value["source_id"] != format!("table:{name}") {
            return Err(format!(
                "native constant table schema/source mismatch: {name}"
            ));
        }
        let rows = value["rows"]
            .as_object()
            .ok_or("native constant table rows missing")?;
        let index = if crate::skill_levels::frozen::table(name)
            .is_some_and(|frozen| std::ptr::eq(frozen, value))
        {
            let indexes = FROZEN_INDEXES.get_or_init(|| {
                crate::skill_levels::frozen::tables()
                    .iter()
                    .map(|(name, table)| {
                        let index = table["rows"]
                            .as_object()
                            .ok_or_else(|| "native constant table rows missing".to_owned())
                            .and_then(|rows| row_index(name, rows));
                        (name.clone(), index)
                    })
                    .collect()
            });
            Cow::Borrowed(indexes[name].as_ref().map_err(Clone::clone)?)
        } else {
            Cow::Owned(row_index(name, rows)?)
        };
        Ok(Self { name, rows, index })
    }

    pub(crate) fn lookup(&self, key: &str) -> Result<(&'a str, &'a Value), String> {
        let canonical = self
            .index
            .get(&key.to_ascii_lowercase())
            .ok_or_else(|| format!("missing native constant row {}:{key}", self.name))?;
        Ok((canonical, &self.rows[*canonical]))
    }

    pub(crate) fn evidence(&self, row: &str) -> Value {
        let escaped = row.replace('~', "~0").replace('/', "~1");
        json!({"source_id":format!("table:{}",self.name),"table":self.name,"row":row,"pointer":format!("/rows/{escaped}")})
    }

    pub(crate) fn rows(&self) -> &'a serde_json::Map<String, Value> {
        self.rows
    }
}

pub(crate) fn text<'a>(value: &'a Value, key: &str) -> Result<&'a str, String> {
    value[key]
        .as_str()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| format!("native constant field missing: {key}"))
}

pub(crate) fn accumulate_scalar(
    attributes: &mut BTreeMap<String, Scalar>,
    property: &str,
    scalar: Scalar,
) -> Result<(), String> {
    if property.is_empty()
        || !property.is_ascii()
        || attributes
            .keys()
            .any(|k| k.eq_ignore_ascii_case(property) && k != property)
    {
        return Err("ambiguous or non-ASCII property".into());
    }
    let old = attributes.get(property).copied().unwrap_or(Scalar::zero());
    attributes.insert(property.to_owned(), old.add(scalar)?);
    Ok(())
}

pub(crate) fn accumulate(
    attributes: &mut BTreeMap<String, Scalar>,
    property: &str,
    value: &Value,
) -> Result<(), String> {
    accumulate_scalar(attributes, property, Scalar::parse(value)?)
}

/// Static packs have the same interpretation for characters, weapons and gear.
/// Missing condition metadata is unknown, rather than an unconditional pack.
pub(crate) fn apply_additive_pack(
    attributes: &mut BTreeMap<String, Scalar>,
    pack: &Value,
) -> Result<(), String> {
    let conditions = pack["ConditionArray"]
        .as_array()
        .ok_or("static modifier conditions missing or malformed")?;
    if !conditions.is_empty() {
        return Err("conditional stat pack requires a combat adapter".into());
    }
    let modifiers = pack["ModifyData"]
        .as_array()
        .ok_or("static modifiers missing or malformed")?;
    let mut result = attributes.clone();
    for entry in modifiers {
        if entry["ModifierOp"] != "EModifyModOp::MODIFY_MODOP_ADDITIVE" {
            return Err("unsupported static modifier operation".into());
        }
        accumulate(&mut result, text(entry, "PropName")?, &entry["PropValue"])?;
    }
    *attributes = result;
    Ok(())
}

pub fn additive_pack_receipts(pack: &Value) -> Result<BTreeMap<String, Value>, String> {
    let mut attributes = BTreeMap::new();
    apply_additive_pack(&mut attributes, pack)?;
    Ok(attributes
        .into_iter()
        .map(|(name, value)| (name, value.receipt()))
        .collect())
}

pub(crate) fn from_receipt(value: &Value) -> Result<Scalar, String> {
    let read = |key: &str| {
        value[key]
            .as_str()
            .ok_or_else(|| format!("missing exact native receipt {key}"))?
            .parse::<u128>()
            .map_err(|_| format!("invalid exact native receipt {key}"))
    };
    let denominator = read("denominator")?;
    if denominator == 0 {
        return Err("zero native receipt denominator".into());
    }
    Ok(Scalar::new(read("numerator")?, denominator))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_embedded_table_identity_reuses_the_frozen_index() {
        let frozen = &crate::skill_levels::frozen::tables()[crate::native_damage::TABLE_NAME];
        let embedded = Table::new(crate::native_damage::TABLE_NAME, frozen).unwrap();
        assert!(matches!(embedded.index, Cow::Borrowed(_)));
        let mut custom = frozen.clone();
        custom["rows"]["test_packet"] = json!({"test": 42});
        let injected = Table::new(crate::native_damage::TABLE_NAME, &custom).unwrap();
        assert!(matches!(injected.index, Cow::Owned(_)));
        assert_eq!(injected.lookup("TEST_PACKET").unwrap().1["test"], 42);
        assert!(embedded.lookup("test_packet").is_err());
        custom["rows"]["TEST_PACKET"] = json!({"test": 43});
        assert!(Table::new(crate::native_damage::TABLE_NAME, &custom).is_err());
    }

    #[test]
    fn new_stat_pack_uses_shared_rules_without_entity_registration() {
        let mut attributes = BTreeMap::from([("NewStat".into(), Scalar::new(1, 3))]);
        apply_additive_pack(&mut attributes, &json!({"ConditionArray":[], "ModifyData":[
            {"ModifierOp":"EModifyModOp::MODIFY_MODOP_ADDITIVE", "PropName":"NewStat", "PropValue":0.125},
            {"ModifierOp":"EModifyModOp::MODIFY_MODOP_ADDITIVE", "PropName":"NewStat", "PropValue":0.25}
        ]})).unwrap();
        assert_eq!(attributes["NewStat"], Scalar::new(17, 24));
    }

    #[test]
    fn public_native_scalar_parser_preserves_exact_exponent_numbers_and_rejects_strings() {
        assert_eq!(
            parse_native_scalar(&serde_json::from_str::<Value>("1e-7").unwrap()).unwrap(),
            NativeScalar {
                numerator: 1,
                denominator: 10_000_000,
            }
        );
        assert_eq!(
            parse_native_scalar(&json!(0.4125)).unwrap(),
            NativeScalar {
                numerator: 33,
                denominator: 80,
            }
        );
        assert!(parse_native_scalar(&json!("0.5")).is_err());
        assert!(parse_native_scalar(&json!(-0.5)).is_err());
        assert!(parse_native_scalar(&serde_json::from_str::<Value>("1e100").unwrap()).is_err());
    }

    #[test]
    fn unknown_conditions_and_partial_invalid_packs_cannot_supply_static_stats() {
        let original = BTreeMap::from([("AtkBase".into(), Scalar::new(100, 1))]);
        let valid = json!({"ConditionArray":[], "ModifyData":[
            {"ModifierOp":"EModifyModOp::MODIFY_MODOP_ADDITIVE", "PropName":"AtkBase", "PropValue":1}
        ]});
        let mut invalid = Vec::new();
        for conditions in [Value::Null, json!({}), json!(["combat_condition"])] {
            let mut pack = valid.clone();
            pack["ConditionArray"] = conditions;
            invalid.push(pack);
        }
        for (field, value) in [
            ("ModifierOp", json!("multiply")),
            ("PropName", json!("atkbase")),
            ("PropValue", json!(-1)),
        ] {
            let mut pack = valid.clone();
            let mut bad = pack["ModifyData"][0].clone();
            bad[field] = value;
            pack["ModifyData"].as_array_mut().unwrap().push(bad);
            invalid.push(pack);
        }
        for pack in invalid {
            let mut attributes = original.clone();
            assert!(apply_additive_pack(&mut attributes, &pack).is_err());
            assert_eq!(attributes, original);
        }
    }
}
