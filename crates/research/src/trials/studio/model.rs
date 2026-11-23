use super::statistics::Statistics;
use ntesim_content::simulation::GameEnergySource;
use ntesim_contracts::{Quality, TimeBasisPair};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Sample {
    pub seed: u64,
    pub damage: f64,
    pub dps: TimeBasisPair<f64>,
    pub eps: TimeBasisPair<f64>,
    pub granted_eps: TimeBasisPair<f64>,
    pub hps: TimeBasisPair<f64>,
    pub shp: TimeBasisPair<f64>,
    pub relative_elapsed_us: u64,
    pub absolute_elapsed_us: u64,
    pub hits: f64,
    pub events: f64,
}

impl Sample {
    pub fn relative_seconds(&self) -> f64 {
        self.relative_elapsed_us as f64 / 1e6
    }
    pub fn absolute_seconds(&self) -> f64 {
        self.absolute_elapsed_us as f64 / 1e6
    }
}

#[derive(Debug, Serialize)]
pub struct Actor {
    pub slot: usize,
    pub character_id: String,
    pub dps: TimeBasisPair<Statistics>,
    pub field: TimeBasisPair<Statistics>,
    #[serde(flatten)]
    pub metrics: BTreeMap<String, Statistics>,
    pub actions: BTreeMap<String, Statistics>,
}

#[derive(Debug, Serialize)]
pub struct Source {
    pub actor_index: usize,
    pub ability_id: String,
    pub effect_id: String,
    pub damage: Statistics,
    pub dps: TimeBasisPair<Statistics>,
    pub hits: Statistics,
}

#[derive(Debug, Serialize)]
pub struct Energy {
    pub actor_index: usize,
    pub source: GameEnergySource,
    pub kind: String,
    pub energy: Statistics,
    pub effective: Statistics,
    pub overflow: Statistics,
}

#[derive(Debug, Serialize)]
pub struct Target {
    pub target_index: usize,
    pub damage: Statistics,
    pub dps: TimeBasisPair<Statistics>,
}

#[derive(Debug, Serialize)]
pub struct Timeline {
    pub time: f64,
    pub dps: Statistics,
    pub cumulative: Statistics,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct AggregateMetrics {
    pub dps: TimeBasisPair<Statistics>,
    pub eps: TimeBasisPair<Statistics>,
    pub granted_eps: TimeBasisPair<Statistics>,
    pub hps: TimeBasisPair<Statistics>,
    pub shp: TimeBasisPair<Statistics>,
    pub damage: Statistics,
    pub duration: Statistics,
    pub absolute_duration: Statistics,
    pub hits: Statistics,
    pub events: Statistics,
    pub gross_healing: Statistics,
    pub healing: Statistics,
    pub excess_healing: Statistics,
    pub shield_absorbed: Statistics,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ReactionAggregate {
    pub name: String,
    #[serde(flatten)]
    pub stats: Statistics,
}

#[derive(Debug, Serialize)]
pub struct Aggregate {
    pub completed: usize,
    pub requested: u64,
    pub elapsed_ms: f64,
    pub trials_per_second: Option<f64>,
    pub metrics: AggregateMetrics,
    pub actors: Vec<Actor>,
    pub sources: Vec<Source>,
    pub energy_sources: Vec<Energy>,
    pub targets: Vec<Target>,
    pub timeline: Vec<Timeline>,
    pub bucket_seconds: f64,
    pub quality: Quality,
    pub game_dps_computed: bool,
    pub reactions: Vec<ReactionAggregate>,
}

pub type Values = BTreeMap<String, f64>;
pub type SourceKey = (usize, String, String);

mod map_as_vec {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::collections::BTreeMap;

    pub fn serialize<K, V, S>(map: &BTreeMap<K, V>, serializer: S) -> Result<S::Ok, S::Error>
    where
        K: Serialize + Ord,
        V: Serialize,
        S: Serializer,
    {
        map.iter().collect::<Vec<_>>().serialize(serializer)
    }

    pub fn deserialize<'de, K, V, D>(deserializer: D) -> Result<BTreeMap<K, V>, D::Error>
    where
        K: Deserialize<'de> + Ord,
        V: Deserialize<'de>,
        D: Deserializer<'de>,
    {
        let vec = Vec::<(K, V)>::deserialize(deserializer)?;
        Ok(vec.into_iter().collect())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Observation {
    pub quality: Quality,
    pub bucket_us: u64,
    pub sample: Sample,
    pub support: Values,
    pub actors: Vec<Values>,
    pub actions: Vec<Values>,
    #[serde(with = "map_as_vec")]
    pub sources: BTreeMap<SourceKey, (f64, f64)>,
    #[serde(with = "map_as_vec")]
    pub energy: BTreeMap<(usize, GameEnergySource, String), (f64, f64, f64)>,
    pub targets: Vec<f64>,
    #[serde(deserialize_with = "deserialize_bins")]
    pub bins: Vec<u128>,
    #[serde(default)]
    pub reactions: BTreeMap<String, u32>,
    pub field_times: Vec<TimeBasisPair<u64>>,
}

// The current wire protocol transports unsafe integers as decimal strings.
// Keep small JSON integers numeric and accept that lossless wire representation.
fn deserialize_bins<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec<u128>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Integer {
        Number(serde_json::Number),
        Decimal(String),
    }
    Vec::<Integer>::deserialize(d)?
        .into_iter()
        .map(|value| {
            let text = match value {
                Integer::Number(n) => n.to_string(),
                Integer::Decimal(s) => s,
            };
            if text.is_empty()
                || !text.bytes().all(|b| b.is_ascii_digit())
                || (text.len() > 1 && text.starts_with('0'))
            {
                return Err(serde::de::Error::custom("invalid rounded bin integer"));
            }
            text.parse().map_err(serde::de::Error::custom)
        })
        .collect()
}
