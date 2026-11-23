use super::ExactFraction;
use ntesim_core::combat::DamageKind;
use ntesim_data::native_action_metadata::NativeEvidence;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use ntesim_contracts::hypotheses::HypothesisId;

pub(crate) const NORMAL_PHYSICAL_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::DamageTypeNormalAsPhysicalV1;
#[cfg(test)]
pub(crate) const NORMAL_PHYSICAL_ID: HypothesisId = NORMAL_PHYSICAL_HYPOTHESIS_ID;
pub(crate) const NORMAL_PHYSICAL_HYPOTHESIS: &str = NORMAL_PHYSICAL_HYPOTHESIS_ID.as_str();
pub(crate) const NORMAL_PHYSICAL_DESCRIPTION: &str = NORMAL_PHYSICAL_HYPOTHESIS_ID.description();

pub(crate) const FIXED_CRIT_PROBABILITY_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::DamageFixedCritRateAsProbabilityV1;
#[cfg(test)]
pub(crate) const FIXED_CRIT_PROBABILITY_ID: HypothesisId = FIXED_CRIT_PROBABILITY_HYPOTHESIS_ID;
pub(crate) const FIXED_CRIT_PROBABILITY_HYPOTHESIS: &str =
    FIXED_CRIT_PROBABILITY_HYPOTHESIS_ID.as_str();
pub(crate) const FIXED_CRIT_PROBABILITY_DESCRIPTION: &str =
    FIXED_CRIT_PROBABILITY_HYPOTHESIS_ID.description();

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ActionCategory {
    Melee,
    Skill,
    UltraSkill,
    Qte,
    Other,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PrimaryActionSlot {
    Normal,
    Skill,
    Ultimate,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InputActionRole {
    Normal,
    RedirectSkill,
    Ultimate,
    Qte,
    CriticalRiposte,
    ParryAttack,
    Other,
}

impl PrimaryActionSlot {
    pub(super) fn from_input_role(role: InputActionRole) -> Option<Self> {
        match role {
            InputActionRole::Normal => Some(Self::Normal),
            InputActionRole::RedirectSkill => Some(Self::Skill),
            InputActionRole::Ultimate => Some(Self::Ultimate),
            InputActionRole::Qte
            | InputActionRole::CriticalRiposte
            | InputActionRole::ParryAttack
            | InputActionRole::Other => None,
        }
    }
}

impl ActionCategory {
    pub(crate) fn from_native_tag(value: &str) -> Result<Self, String> {
        match value {
            "Ability.Melee" => Ok(Self::Melee),
            "Ability.Skill" => Ok(Self::Skill),
            "Ability.UltraSkill" => Ok(Self::UltraSkill),
            _ => Err(format!("unsupported native action gameplay tag: {value}")),
        }
    }

    pub fn native_tag(self) -> &'static str {
        match self {
            Self::Melee => "Ability.Melee",
            Self::Skill => "Ability.Skill",
            Self::UltraSkill => "Ability.UltraSkill",
            Self::Qte => "Owner.QTE",
            Self::Other => "Owner.Other",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum MetadataProvenance {
    Native { evidence: NativeEvidence },
    AcceptedModelHypothesis { id: String, description: String },
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ActionMetadata {
    pub categories: BTreeSet<ActionCategory>,
    pub gameplay_tags: Vec<String>,
    pub ability_id: String,
    /// Native config coordinate is distinct from combat gameplay categories.
    pub native_ability_index: Option<u64>,
    pub input_role: Option<InputActionRole>,
    pub input_role_provenance: Option<MetadataProvenance>,
    pub primary_slot: Option<PrimaryActionSlot>,
    pub source_evidence: Option<NativeEvidence>,
    pub identity_evidence: Option<NativeEvidence>,
    pub provenance: MetadataProvenance,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DamageElement {
    Physical,
    Cosmos,
    Nature,
    Chaos,
    Incantation,
    Lakshana,
    Psyche,
}

impl DamageElement {
    pub(crate) const ACTOR_ELEMENTS: [Option<&'static str>; 7] = [
        None,
        Some("COSMOS"),
        Some("NATURE"),
        Some("CHAOS"),
        Some("INCANTATION"),
        Some("LAKSHANA"),
        Some("PSYCHE"),
    ];

    pub(crate) fn actor_element_name(&self) -> Option<&'static str> {
        Self::ACTOR_ELEMENTS[self.actor_element_index()]
    }

    pub(crate) fn actor_element_index(&self) -> usize {
        match self {
            Self::Physical => 0,
            Self::Cosmos => 1,
            Self::Nature => 2,
            Self::Chaos => 3,
            Self::Incantation => 4,
            Self::Lakshana => 5,
            Self::Psyche => 6,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum PacketClassification {
    #[default]
    Standard,
    SeparateParry,
    ArcChain,
}

impl PacketClassification {
    pub fn is_standard(&self) -> bool {
        *self == Self::Standard
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum NormalizedMetadataProvenance {
    Native { evidence_id: usize },
    AcceptedModelHypothesis { hypothesis_id: String },
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct NormalizedActionMetadata {
    pub categories: BTreeSet<ActionCategory>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub gameplay_tags: Vec<String>,
    pub ability_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native_ability_index: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_role: Option<InputActionRole>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_role_provenance: Option<NormalizedMetadataProvenance>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary_slot: Option<PrimaryActionSlot>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_evidence_id: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identity_evidence_id: Option<usize>,
    pub provenance: NormalizedMetadataProvenance,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct NormalizedPacketMetadata {
    pub damage_effect_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native_ga_name: Option<String>,
    pub native_damage_type_ex: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native_damage_source_category: Option<String>,
    pub damage_kind: DamageKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub element: Option<DamageElement>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fixed_crit_rate: Option<ExactFraction>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fixed_crit_rate_provenance: Option<NormalizedMetadataProvenance>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unbal_value: Option<ExactFraction>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unbal_value_provenance: Option<NormalizedMetadataProvenance>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub semantic_tags: BTreeSet<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub semantic_provenance: Option<NormalizedMetadataProvenance>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_evidence_id: Option<usize>,
    pub provenance: NormalizedMetadataProvenance,
    #[serde(default, skip_serializing_if = "PacketClassification::is_standard")]
    pub packet_kind: PacketClassification,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct NormalizedActionPacketBinding {
    pub kind: ActionPacketBindingKind,
    pub initiating_ability_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub packet_ga_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provenance: Option<NormalizedMetadataProvenance>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct NormalizedActionPacketMetadata {
    pub action: NormalizedActionMetadata,
    pub packet: NormalizedPacketMetadata,
    pub binding: NormalizedActionPacketBinding,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct PacketMetadata {
    pub damage_effect_id: String,
    pub native_ga_name: Option<String>,
    pub native_damage_type_ex: String,
    pub native_damage_source_category: Option<String>,
    pub damage_kind: DamageKind,
    /// `None` is intentional for mental/true damage; it never means actor element.
    pub element: Option<DamageElement>,
    /// Exact override probability; `None` is the frozen zero/no-override sentinel.
    /// `Some(0/1)` remains a distinct explicit fixed-zero probability.
    pub fixed_crit_rate: Option<ExactFraction>,
    pub fixed_crit_rate_provenance: Option<MetadataProvenance>,
    /// Frozen DT_SkillDamageData.UnbalValue as an exact source scalar. This
    /// metadata is observational only; it does not model gauge depletion.
    pub unbal_value: Option<ExactFraction>,
    pub unbal_value_provenance: Option<MetadataProvenance>,
    pub semantic_tags: BTreeSet<String>,
    pub semantic_provenance: Option<MetadataProvenance>,
    pub source_evidence: Option<NativeEvidence>,
    pub provenance: MetadataProvenance,
    #[serde(default)]
    pub packet_kind: PacketClassification,
}

impl PacketMetadata {
    pub fn is_separate_parry(&self) -> bool {
        self.packet_kind == PacketClassification::SeparateParry
    }

    pub fn is_arc_chain(&self) -> bool {
        self.packet_kind == PacketClassification::ArcChain
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ActionPacketMetadata {
    pub action: ActionMetadata,
    pub packet: PacketMetadata,
    pub binding: ActionPacketBinding,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActionPacketBindingKind {
    NativeDirect,
    OwnerChild,
    ResearchDeclared,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ActionPacketBinding {
    pub kind: ActionPacketBindingKind,
    pub initiating_ability_id: String,
    pub packet_ga_name: Option<String>,
    pub provenance: Option<MetadataProvenance>,
    /// Execution-local fanout identity; native source identity stays separate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub energy_group_id: Option<String>,
}
