//! Exact owner declarations for packet-level gameplay semantics.
//! Shared combat algorithms consume these tags without inferring them from IDs.

use crate::HypothesisId;
pub(crate) const ZANKOU_DOT_PACKET_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::CharacterZankouDotPacketSemanticsV1;
pub(crate) const BAICANG_DOT_PACKET_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::CharacterBaicangDotPacketSemanticsV1;
pub(crate) const ADLER_SKILL_DOT_PACKET_HYPOTHESIS_ID: HypothesisId =
    HypothesisId::CharacterAdlerSkillDotPacketSemanticsV1;
pub(crate) const DOT_DAMAGE_TAG: &str = "State.Damage.Dot";
pub(crate) const FOLLOW_UP_DAMAGE_TAG: &str = "State.Damage.Followup";
/// Content-owned target-status label; it is not asserted to be the native
/// GameplayTag name because the frozen tables do not expose that binding.
pub(crate) const SCORCH_STATUS_TAG: &str = "ntesim.status.scorch";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PacketSemantics {
    pub(crate) tags: &'static [&'static str],
    pub(crate) hypothesis_id: HypothesisId,
}

pub(crate) fn resolve(effect_id: &str) -> Option<PacketSemantics> {
    let (tags, hypothesis_id) = match effect_id {
        "GE_Player_Zankou_Skill3_Damage"
        | "GE_Player_Zankou_Skill3_1_Damage"
        | "GE_Player_Zankou_Skill3_2_Damage"
        | "GE_Player_Zankou_Skill4_Damage"
        | "GE_Player_Zankou_Skill4_1_Damage"
        | "GE_Player_Zankou_Skill4_2_Damage" => (
            &[
                "State.Damage",
                "State.Damage.TriggerParry",
                "State.Damage.Skill",
                "Ability.Player.Nanally.XieTongDamage",
            ][..],
            crate::characters::zankou::forms::MODEL,
        ),
        "GE_Player_Zankou_MagicUltraSkill1_Damage" | "GE_Player_Zankou_MagicUltraSkill2_Damage" => {
            (
                &[
                    "State.Damage",
                    "State.Damage.UltraSkill",
                    "Ability.Player.Nanally.XieTongDamage",
                    "Ability.Player.Zankou.UltraSkill",
                    "Ability.Player.Zankou.MagicUltra",
                ][..],
                crate::characters::zankou::forms::MODEL,
            )
        }
        "GE_Player_Zankou_MagicMelee1_Damage"
        | "GE_Player_Zankou_MagicMelee2_Damage"
        | "GE_Player_Zankou_MagicMelee2_1_Damage"
        | "GE_Player_Zankou_MagicMelee2_2_Damage"
        | "GE_Player_Zankou_MagicMelee3_Damage"
        | "GE_Player_Zankou_MagicMelee3_1_Damage"
        | "GE_Player_Zankou_MagicMelee4_Damage"
        | "GE_Player_Zankou_MagicMelee4_1_Damage"
        | "GE_Player_Zankou_MagicBranch1_Damage"
        | "GE_Player_Zankou_MagicBranch2_Damage"
        | "GE_Player_Zankou_MagicBranch3_Damage"
        | "GE_Player_Zankou_MagicBranch4_Damage" => (
            &[
                "State.Damage",
                "State.Damage.Melee",
                "Ability.Player.Nanally.XieTongDamage",
            ][..],
            crate::characters::zankou::forms::MODEL,
        ),
        "GE_Player_Zankou_DotDamage" | "GE_Player_Zankou_DotUltraDamage" => (
            &["State.Damage", "State.Damage.Dot", "State.Damage.Melee"][..],
            ZANKOU_DOT_PACKET_HYPOTHESIS_ID,
        ),
        "GE_Player_Cang_UltraSkill_Damage" => (
            &["State.Damage", "State.Damage.Dot"][..],
            BAICANG_DOT_PACKET_HYPOTHESIS_ID,
        ),
        "GE_Player_Adler_Skill_Damage" => (
            &["State.Damage", "State.Damage.Dot"][..],
            ADLER_SKILL_DOT_PACKET_HYPOTHESIS_ID,
        ),
        "GE_Player_Chiichan073_Skill1_CoinDamage" => (
            &[
                "State.Damage",
                "State.Damage.Skill",
                "State.Damage.Skill.ChiichanCoin1",
            ][..],
            crate::characters::chiz::pressed::MODEL,
        ),
        "GE_Player_Chiichan073_Skill2_CoinDamage" => (
            &[
                "State.Damage",
                "State.Damage.Skill",
                "State.Damage.Skill.ChiichanCoin2",
            ][..],
            crate::characters::chiz::pressed::held::MODEL,
        ),
        "GE_Player_Kuhara_Seed_Damage" | "GE_Player_Kuhara_BudEnd_Damage" => (
            &[
                "State.Damage",
                "State.Damage.Attachment",
                "ntesim.damage.attachment",
            ][..],
            crate::characters::jiuyuan::pact::HYPOTHESIS_ID,
        ),
        crate::characters::haniel::hootie::ENSEMBLE
        | crate::characters::haniel::hootie::SYMPHONY
        | crate::characters::haniel::hootie::EXTRA => (
            &["State.Damage", FOLLOW_UP_DAMAGE_TAG][..],
            crate::characters::haniel::hootie::HYPOTHESIS,
        ),
        _ => return None,
    };
    Some(PacketSemantics {
        tags,
        hypothesis_id,
    })
}

/// An owner may identify a source sentinel whose selected meaning differs
/// from the generic probability model. The raw value remains checked.
pub(crate) fn fixed_crit_encoding(
    effect: &str,
) -> Option<(
    crate::simulation::ExactFraction,
    crate::simulation::ExactFraction,
    HypothesisId,
)> {
    crate::characters::chiz::pressed::fixed_crit_encoding(effect)
}
