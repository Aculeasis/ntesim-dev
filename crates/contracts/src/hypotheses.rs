//! Strongly typed hypothesis identifiers and centralized hypothesis catalog.
//! Typed runtime assumption identifiers and descriptions; no JSON data dependency.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::ops::Deref;

/// Strongly typed hypothesis identifier representing modeled combat mechanics and assumptions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u16)]
pub enum HypothesisId {
    /// `ability_index.primary_action_role_v1`
    AbilityIndexPrimaryActionRoleV1 = 0,
    /// `actions.caller_timing_v1`
    ActionsCallerTimingV1 = 1,
    /// `actions.primary_packet_v1`
    ActionsPrimaryPacketV1 = 2,
    /// `adler.a5_burst_energy_refund_v1`
    AdlerA5BurstEnergyRefundV1 = 3,
    /// `adler.blessing_a4_overflow_v1`
    AdlerBlessingA4OverflowV1 = 4,
    /// `adler.blessing_r6_v1`
    AdlerBlessingR6V1 = 5,
    /// `adler.blessing_shield_v1`
    AdlerBlessingShieldV1 = 6,
    /// `adler.burst_sunyas_v1`
    AdlerBurstSunyasV1 = 7,
    /// `adler.karma_v1`
    AdlerKarmaV1 = 8,
    /// `adler.normal_chain_v1`
    AdlerNormalChainV1 = 9,
    /// `adler.ordinary_hit_energy_v1`
    AdlerOrdinaryHitEnergyV1 = 10,
    /// `adler.skill_all_in_range_dummy_v1`
    AdlerSkillAllInRangeDummyV1 = 11,
    /// `adler.skill_train_v1`
    AdlerSkillTrainV1 = 12,
    /// `aim.mode_transitions_v1`
    AimModeTransitionsV1 = 13,
    /// `aurelia.airattack_evadebranch_direct_packet_energy_v1`
    AureliaAirattackEvadebranchDirectPacketEnergyV1 = 14,
    /// `aurelia.aoe_all_in_range_dummy_v1`
    AureliaAoeAllInRangeDummyV1 = 15,
    /// `aurelia.direct_packet_inputs_v1`
    AureliaDirectPacketInputsV1 = 16,
    /// `aurelia.normal_chain_v1`
    AureliaNormalChainV1 = 17,
    /// `aurelia.ordinary_hit_energy_v1`
    AureliaOrdinaryHitEnergyV1 = 18,
    /// `aurelia.skill_burst_packets_v1`
    AureliaSkillBurstPacketsV1 = 19,
    /// `baicang.aoe_all_in_range_dummy_v1`
    BaicangAoeAllInRangeDummyV1 = 20,
    /// `baicang.judgment_of_autumn_domain_v1`
    BaicangJudgmentOfAutumnDomainV1 = 21,
    /// `baicang.normal_chain_v1`
    BaicangNormalChainV1 = 22,
    /// `baicang.ordinary_hit_energy_v1`
    BaicangOrdinaryHitEnergyV1 = 23,
    /// `blackbird.aoe_all_in_range_dummy_v1`
    BlackbirdAoeAllInRangeDummyV1 = 24,
    /// `blackbird.normal_chain_v1`
    BlackbirdNormalChainV1 = 25,
    /// `blackbird.ordinary_hit_energy_v1`
    BlackbirdOrdinaryHitEnergyV1 = 26,
    /// `blackbird.skill_burst_packets_v1`
    BlackbirdSkillBurstPacketsV1 = 27,
    /// `build.action_projection_v1`
    BuildActionProjectionV1 = 28,
    /// `build.base_overrides_v1`
    BuildBaseOverridesV1 = 29,
    /// `build.gear_projection_v1`
    BuildGearProjectionV1 = 30,
    /// `build.stat_quantization_v1`
    BuildStatQuantizationV1 = 31,
    /// `build.static_additive_v1`
    BuildStaticAdditiveV1 = 32,
    /// `build.unmodeled_native_attributes_v1`
    BuildUnmodeledNativeAttributesV1 = 33,
    /// `chaos.aoe_all_in_range_dummy_v1`
    ChaosAoeAllInRangeDummyV1 = 34,
    /// `chaos.normal_chain_v1`
    ChaosNormalChainV1 = 35,
    /// `chaos.ordinary_hit_energy_v1`
    ChaosOrdinaryHitEnergyV1 = 36,
    /// `chaos.qte_direct_packet_energy_v1`
    ChaosQteDirectPacketEnergyV1 = 37,
    /// `chaos.skill_bites_v1`
    ChaosSkillBitesV1 = 38,
    /// `chaos.suppression_packets_v1`
    ChaosSuppressionPacketsV1 = 39,
    /// `character.adler.skill_dot_packet_semantics_v1`
    CharacterAdlerSkillDotPacketSemanticsV1 = 40,
    /// `character.baicang.dot_packet_semantics_v1`
    CharacterBaicangDotPacketSemanticsV1 = 41,
    /// `character.zankou.dot_packet_semantics_v1`
    CharacterZankouDotPacketSemanticsV1 = 42,
    /// `character.zankou.vile_ash_dot_scheduling_v1`
    CharacterZankouVileAshDotSchedulingV1 = 43,
    /// `chiz.aoe_all_in_range_dummy_v1`
    ChizAoeAllInRangeDummyV1 = 44,
    /// `chiz.normal_chain_v1`
    ChizNormalChainV1 = 45,
    /// `chiz.ordinary_hit_energy_v1`
    ChizOrdinaryHitEnergyV1 = 46,
    /// `combat.incoming_damage_fixture_v1`
    CombatIncomingDamageFixtureV1 = 47,
    /// `combat.live_healing_modifiers_v1`
    CombatLiveHealingModifiersV1 = 48,
    /// `combat.unbal_value_as_target_gauge_v1`
    CombatUnbalValueAsTargetGaugeV1 = 49,
    /// `crit.fixed_noncrit_preview_v1`
    CritFixedNoncritPreviewV1 = 50,
    /// `daffodil.aoe_all_in_range_dummy_v1`
    DaffodilAoeAllInRangeDummyV1 = 51,
    /// `daffodil.branch_entry_direct_packet_energy_v1`
    DaffodilBranchEntryDirectPacketEnergyV1 = 52,
    /// `daffodil.branch_entry_direct_packet_inputs_v1`
    DaffodilBranchEntryDirectPacketInputsV1 = 53,
    /// `daffodil.normal_chain_v1`
    DaffodilNormalChainV1 = 54,
    /// `daffodil.ordinary_hit_energy_v1`
    DaffodilOrdinaryHitEnergyV1 = 55,
    /// `daffodil.skill_burst_packets_v1`
    DaffodilSkillBurstPacketsV1 = 56,
    /// `damage.fixed_crit_rate_as_probability_v1`
    DamageFixedCritRateAsProbabilityV1 = 57,
    /// `damage_type.normal_as_physical_v1`
    DamageTypeNormalAsPhysicalV1 = 58,
    /// `edgar.a5_max_hp_v1`
    EdgarA5MaxHpV1 = 59,
    /// `edgar.a6_wake_team_def_v1`
    EdgarA6WakeTeamDefV1 = 60,
    /// `edgar.aoe_all_in_range_dummy_v1`
    EdgarAoeAllInRangeDummyV1 = 61,
    /// `edgar.key_of_truth_v1`
    EdgarKeyOfTruthV1 = 62,
    /// `edgar.normal_chain_v1`
    EdgarNormalChainV1 = 63,
    /// `edgar.ordinary_hit_energy_v1`
    EdgarOrdinaryHitEnergyV1 = 64,
    /// `edgar.skill_burst_packets_v1`
    EdgarSkillBurstPacketsV1 = 65,
    /// `edgar.ultimate_periodic_healing_v1`
    EdgarUltimatePeriodicHealingV1 = 66,
    /// `energy.efficiency_carry_v1`
    EnergyEfficiencyCarryV1 = 67,
    /// `fadia.aoe_all_in_range_dummy_v1`
    FadiaAoeAllInRangeDummyV1 = 68,
    /// `fadia.full_hp_initial_burst_v1`
    FadiaFullHpInitialBurstV1 = 69,
    /// `fadia.normal_chain_v1`
    FadiaNormalChainV1 = 70,
    /// `fadia.ordinary_hit_energy_v1`
    FadiaOrdinaryHitEnergyV1 = 71,
    /// `fireflies.all_in_range_v1`
    FirefliesAllInRangeV1 = 72,
    /// `fireflies.proc_v1`
    FirefliesProcV1 = 73,
    /// `fireflies.solo_proc_v2`
    FirefliesSoloProcV2 = 74,
    /// `fireflies.team_anima_hit_v1`
    FirefliesTeamAnimaHitV1 = 75,
    /// `gear.cast_buff_before_hits_v1`
    GearCastBuffBeforeHitsV1 = 76,
    /// `gear.qte_cast_buff_before_hits_v1`
    GearQteCastBuffBeforeHitsV1 = 77,
    /// `gear.skill_cast_buff_before_hits_v1`
    GearSkillCastBuffBeforeHitsV1 = 78,
    /// `haniel.aoe_all_in_range_dummy_v1`
    HanielAoeAllInRangeDummyV1 = 79,
    /// `haniel.normal_chain_v1`
    HanielNormalChainV1 = 80,
    /// `haniel.ordinary_hit_energy_v1`
    HanielOrdinaryHitEnergyV1 = 81,
    /// `haniel.skill_burst_packets_v1`
    HanielSkillBurstPacketsV1 = 82,
    /// `haniel.team_atk_hootie_clock_v1`
    HanielTeamAtkHootieClockV1 = 83,
    /// `hathor.aoe_all_in_range_dummy_v1`
    HathorAoeAllInRangeDummyV1 = 84,
    /// `hathor.branch_airattack_direct_packet_energy_v1`
    HathorBranchAirattackDirectPacketEnergyV1 = 85,
    /// `hathor.branch_airattack_direct_packet_inputs_v1`
    HathorBranchAirattackDirectPacketInputsV1 = 86,
    /// `hathor.express_delivery_state_v1`
    HathorExpressDeliveryStateV1 = 87,
    /// `hathor.normal_chain_v1`
    HathorNormalChainV1 = 88,
    /// `hathor.ordinary_hit_energy_v1`
    HathorOrdinaryHitEnergyV1 = 89,
    /// `hathor.skill_burst_packets_v1`
    HathorSkillBurstPacketsV1 = 90,
    /// `hotori.aoe_all_in_range_dummy_v1`
    HotoriAoeAllInRangeDummyV1 = 91,
    /// `hotori.normal_chain_v1`
    HotoriNormalChainV1 = 92,
    /// `hotori.ordinary_hit_energy_v1`
    HotoriOrdinaryHitEnergyV1 = 93,
    /// `hotori.skill_burst_packets_v1`
    HotoriSkillBurstPacketsV1 = 94,
    /// `inputs.critical_dodge_assumed_success_v1`
    InputsCriticalDodgeAssumedSuccessV1 = 95,
    /// `inputs.special_dodge_timing_v1`
    InputsSpecialDodgeTimingV1 = 96,
    /// `iroi.aoe_all_in_range_dummy_v1`
    IroiAoeAllInRangeDummyV1 = 97,
    /// `iroi.burst_post_cast_team_heal_v1`
    IroiBurstPostCastTeamHealV1 = 98,
    /// `iroi.burst_primary_packet_v1`
    IroiBurstPrimaryPacketV1 = 99,
    /// `iroi.effect3_healing_outgoing_v1`
    IroiEffect3HealingOutgoingV1 = 100,
    /// `iroi.normal_chain_v1`
    IroiNormalChainV1 = 101,
    /// `iroi.ordinary_hit_energy_v1`
    IroiOrdinaryHitEnergyV1 = 102,
    /// `iroi.r3_team_def_aura_v1`
    IroiR3TeamDefAuraV1 = 103,
    /// `iroi.r6_team_crit_aura_v1`
    IroiR6TeamCritAuraV1 = 104,
    /// `iroi.skill_nine_packet_train_v1`
    IroiSkillNinePacketTrainV1 = 105,
    /// `iroi.skill_team_heal_v1`
    IroiSkillTeamHealV1 = 106,
    /// `jiuyuan.aoe_all_in_range_dummy_v1`
    JiuyuanAoeAllInRangeDummyV1 = 107,
    /// `jiuyuan.normal_chain_v1`
    JiuyuanNormalChainV1 = 108,
    /// `jiuyuan.ordinary_hit_energy_v1`
    JiuyuanOrdinaryHitEnergyV1 = 109,
    /// `jiuyuan.skill_burst_packets_v1`
    JiuyuanSkillBurstPacketsV1 = 110,
    /// `lacrimosa.aoe_all_in_range_dummy_v1`
    LacrimosaAoeAllInRangeDummyV1 = 111,
    /// `lacrimosa.forms_v1`
    LacrimosaFormsV1 = 112,
    /// `lacrimosa.nightmare_dot_scheduling_v1`
    LacrimosaNightmareDotSchedulingV1 = 113,
    /// `lacrimosa.ordinary_hit_energy_v1`
    LacrimosaOrdinaryHitEnergyV1 = 114,
    /// `lacrimosa.ranged_normal_chain_v1`
    LacrimosaRangedNormalChainV1 = 115,
    /// `lacrimosa.skill_burst_packets_v1`
    LacrimosaSkillBurstPacketsV1 = 116,
    /// `linko.aoe_all_in_range_dummy_v1`
    LinkoAoeAllInRangeDummyV1 = 117,
    /// `linko.normal_chain_v1`
    LinkoNormalChainV1 = 118,
    /// `linko.ordinary_hit_energy_v1`
    LinkoOrdinaryHitEnergyV1 = 119,
    /// `linko.skill_four_hits_v1`
    LinkoSkillFourHitsV1 = 120,
    /// `mint.a3_reaction_trigger_v1`
    MintA3ReactionTriggerV1 = 121,
    /// `mint.aoe_all_in_range_dummy_v1`
    MintAoeAllInRangeDummyV1 = 122,
    /// `mint.normal_chain_v1`
    MintNormalChainV1 = 123,
    /// `mint.ordinary_hit_energy_v1`
    MintOrdinaryHitEnergyV1 = 124,
    /// `mint.resonance_3_v1`
    MintResonance3V1 = 125,
    /// `mint.skill_burst_packets_v1`
    MintSkillBurstPacketsV1 = 126,
    /// `mint.skill_coordinates_v1`
    MintSkillCoordinatesV1 = 127,
    /// `nanally.authority_a3_follow_up_v1`
    NanallyAuthorityA3FollowUpV1 = 128,
    /// `nanally.burst_seven_hits_v1`
    NanallyBurstSevenHitsV1 = 129,
    /// `nanally.burst_summon_state_v1`
    NanallyBurstSummonStateV1 = 130,
    /// `nanally.normal_chain_v1`
    NanallyNormalChainV1 = 131,
    /// `nanally.ordinary_hit_energy_v1`
    NanallyOrdinaryHitEnergyV1 = 132,
    /// `nanally.skill_five_hits_v1`
    NanallySkillFiveHitsV1 = 133,
    /// `nanally.underboss_coordination_v2`
    NanallyUnderbossCoordinationV2 = 134,
    /// `owner.action_role.character_1003_qte_v1`
    OwnerActionRoleCharacter1003QteV1 = 135,
    /// `owner.action_role.character_1004_melee_v1`
    OwnerActionRoleCharacter1004MeleeV1 = 136,
    /// `owner.action_role.character_1004_qte_v1`
    OwnerActionRoleCharacter1004QteV1 = 137,
    /// `owner.action_role.character_1004_skill_v1`
    OwnerActionRoleCharacter1004SkillV1 = 138,
    /// `owner.action_role.character_1004_ultimate_v1`
    OwnerActionRoleCharacter1004UltimateV1 = 139,
    /// `owner.action_role.character_1008_qte_v1`
    OwnerActionRoleCharacter1008QteV1 = 140,
    /// `owner.action_role.character_1010_qte_v1`
    OwnerActionRoleCharacter1010QteV1 = 141,
    /// `owner.action_role.character_1019_qte_v1`
    OwnerActionRoleCharacter1019QteV1 = 142,
    /// `owner.action_role.character_1020_qte_v1`
    OwnerActionRoleCharacter1020QteV1 = 143,
    /// `owner.action_role.character_1021_qte_v1`
    OwnerActionRoleCharacter1021QteV1 = 144,
    /// `owner.action_role.character_1023_qte_v1`
    OwnerActionRoleCharacter1023QteV1 = 145,
    /// `owner.action_role.character_1025_qte_v1`
    OwnerActionRoleCharacter1025QteV1 = 146,
    /// `owner.action_role.character_1033_qte_v1`
    OwnerActionRoleCharacter1033QteV1 = 147,
    /// `owner.action_role.character_1036_qte_v1`
    OwnerActionRoleCharacter1036QteV1 = 148,
    /// `owner.action_role.character_1039_qte_v1`
    OwnerActionRoleCharacter1039QteV1 = 149,
    /// `owner.action_role.character_1042_qte_v1`
    OwnerActionRoleCharacter1042QteV1 = 150,
    /// `owner.action_role.character_1046_melee_v1`
    OwnerActionRoleCharacter1046MeleeV1 = 151,
    /// `owner.action_role.character_1046_qte_v1`
    OwnerActionRoleCharacter1046QteV1 = 152,
    /// `owner.action_role.character_1046_skill_v1`
    OwnerActionRoleCharacter1046SkillV1 = 153,
    /// `owner.action_role.character_1046_ultimate_v1`
    OwnerActionRoleCharacter1046UltimateV1 = 154,
    /// `owner.action_role.character_1051_qte_v1`
    OwnerActionRoleCharacter1051QteV1 = 155,
    /// `owner.action_role.character_1052_qte_v1`
    OwnerActionRoleCharacter1052QteV1 = 156,
    /// `owner.action_role.character_1054_qte_v1`
    OwnerActionRoleCharacter1054QteV1 = 157,
    /// `owner.action_role.character_1055_qte_v1`
    OwnerActionRoleCharacter1055QteV1 = 158,
    /// `owner.action_role.character_1057_qte_v1`
    OwnerActionRoleCharacter1057QteV1 = 159,
    /// `owner.action_role.character_1070_qte_v1`
    OwnerActionRoleCharacter1070QteV1 = 160,
    /// `owner.action_role.character_1071_qte_v1`
    OwnerActionRoleCharacter1071QteV1 = 161,
    /// `owner.action_role.character_1071_skill_v1`
    OwnerActionRoleCharacter1071SkillV1 = 162,
    /// `owner.action_role.character_1071_ultimate_v1`
    OwnerActionRoleCharacter1071UltimateV1 = 163,
    /// `owner.action_role.character_1072_qte_v1`
    OwnerActionRoleCharacter1072QteV1 = 164,
    /// `owner.action_role.character_1073_qte_v1`
    OwnerActionRoleCharacter1073QteV1 = 165,
    /// `owner.action_role.character_1075_qte_v1`
    OwnerActionRoleCharacter1075QteV1 = 166,
    /// `owner.action_role.character_1076_qte_v1`
    OwnerActionRoleCharacter1076QteV1 = 167,
    /// `owner.packet_binding.character_1010_skill_lv3_v1`
    OwnerPacketBindingCharacter1010SkillLv3V1 = 168,
    /// `published.preview_inputs_v1`
    PublishedPreviewInputsV1 = 169,
    /// `research.preview.single_packet_v1`
    ResearchPreviewSinglePacketV1 = 170,
    /// `sakiri.aoe_all_in_range_dummy_v1`
    SakiriAoeAllInRangeDummyV1 = 171,
    /// `sakiri.direct_plunge_packet_energy_v1`
    SakiriDirectPlungePacketEnergyV1 = 172,
    /// `sakiri.direct_plunge_packet_v1`
    SakiriDirectPlungePacketV1 = 173,
    /// `sakiri.normal_chain_v1`
    SakiriNormalChainV1 = 174,
    /// `sakiri.ordinary_hit_energy_v1`
    SakiriOrdinaryHitEnergyV1 = 175,
    /// `sakiri.skill_burst_packets_v1`
    SakiriSkillBurstPacketsV1 = 176,
    /// `sets.crimson_v1`
    SetsCrimsonV1 = 177,
    /// `sets.hp_suit_v1`
    SetsHpSuitV1 = 178,
    /// `sets.lost_radiance_v1`
    SetsLostRadianceV1 = 179,
    /// `sets.quiet_manor_v1`
    SetsQuietManorV1 = 180,
    /// `sets.shadow_creed_v1`
    SetsShadowCreedV1 = 181,
    /// `shinku.aoe_all_in_range_dummy_v1`
    ShinkuAoeAllInRangeDummyV1 = 182,
    /// `shinku.burst_packets_v1`
    ShinkuBurstPacketsV1 = 183,
    /// `shinku.direct_ultimate_followups_v1`
    ShinkuDirectUltimateFollowupsV1 = 184,
    /// `shinku.normal_chain_v1`
    ShinkuNormalChainV1 = 185,
    /// `shinku.ordinary_hit_energy_v1`
    ShinkuOrdinaryHitEnergyV1 = 186,
    /// `shinku.pre_dawn_refresh_v1`
    ShinkuPreDawnRefreshV1 = 187,
    /// `shinku.skill_packets_v1`
    ShinkuSkillPacketsV1 = 188,
    /// `skia.normal_chain_v1`
    SkiaNormalChainV1 = 189,
    /// `skia.ordinary_hit_energy_v1`
    SkiaOrdinaryHitEnergyV1 = 190,
    /// `skia.skill_burst_packets_v1`
    SkiaSkillBurstPacketsV1 = 191,
    /// `skill_levels.coordinates_v1`
    SkillLevelsCoordinatesV1 = 192,
    /// `skill_levels.resonance_3_v1`
    SkillLevelsResonance3V1 = 193,
    /// `speedy_hedgehog.proc_v1`
    SpeedyHedgehogProcV1 = 194,
    /// `weapons.appliance_v1`
    WeaponsApplianceV1 = 195,
    /// `weapons.be_happy_v1`
    WeaponsBeHappyV1 = 196,
    /// `weapons.bit_game_v1`
    WeaponsBitGameV1 = 197,
    /// `weapons.bittercake_damage_event_v1`
    WeaponsBittercakeDamageEventV1 = 198,
    /// `weapons.black_book_v1`
    WeaponsBlackBookV1 = 199,
    /// `weapons.blast_candy_v1`
    WeaponsBlastCandyV1 = 200,
    /// `weapons.bopu_v1`
    WeaponsBopuV1 = 201,
    /// `weapons.boxing_candy_v1`
    WeaponsBoxingCandyV1 = 202,
    /// `weapons.call_of_the_twisted_city_v1`
    WeaponsCallOfTheTwistedCityV1 = 203,
    /// `weapons.contemplative_cat_v1`
    WeaponsContemplativeCatV1 = 204,
    /// `weapons.crowbar_v1`
    WeaponsCrowbarV1 = 205,
    /// `weapons.demon_blade_v1`
    WeaponsDemonBladeV1 = 206,
    /// `weapons.dustbin_v1`
    WeaponsDustbinV1 = 207,
    /// `weapons.eternal_waltz_v1`
    WeaponsEternalWaltzV1 = 208,
    /// `weapons.far_side_moon_v2`
    WeaponsFarSideMoonV2 = 209,
    /// `weapons.forgotten_hp_threshold_v1`
    WeaponsForgottenHpThresholdV1 = 210,
    /// `weapons.gold_record_v1`
    WeaponsGoldRecordV1 = 211,
    /// `weapons.gold_wool_v1`
    WeaponsGoldWoolV1 = 212,
    /// `weapons.good_boy_v1`
    WeaponsGoodBoyV1 = 213,
    /// `weapons.jiaojuan_v1`
    WeaponsJiaojuanV1 = 214,
    /// `weapons.jingmotingyuan_v1`
    WeaponsJingmotingyuanV1 = 215,
    /// `weapons.kite_v1`
    WeaponsKiteV1 = 216,
    /// `weapons.knight_candy_v1`
    WeaponsKnightCandyV1 = 217,
    /// `weapons.koinobori_v1`
    WeaponsKoinoboriV1 = 218,
    /// `weapons.lunarphase_v1`
    WeaponsLunarphaseV1 = 219,
    /// `weapons.marching_beyond_time_v1`
    WeaponsMarchingBeyondTimeV1 = 220,
    /// `weapons.mind_royale_v1`
    WeaponsMindRoyaleV1 = 221,
    /// `weapons.moon_v1`
    WeaponsMoonV1 = 222,
    /// `weapons.motor_candy_v1`
    WeaponsMotorCandyV1 = 223,
    /// `weapons.nakupeda_v1`
    WeaponsNakupedaV1 = 224,
    /// `weapons.native_curve_v1`
    WeaponsNativeCurveV1 = 225,
    /// `weapons.nest_bird_v1`
    WeaponsNestBirdV1 = 226,
    /// `weapons.noros_v1`
    WeaponsNorosV1 = 227,
    /// `weapons.oraora_v1`
    WeaponsOraoraV1 = 228,
    /// `weapons.paper_plane_v1`
    WeaponsPaperPlaneV1 = 229,
    /// `weapons.police_rat_v2`
    WeaponsPoliceRatV2 = 230,
    /// `weapons.prokaryon_v1`
    WeaponsProkaryonV1 = 231,
    /// `weapons.raging_flames_v1`
    WeaponsRagingFlamesV1 = 232,
    /// `weapons.reality_refuge_v1`
    WeaponsRealityRefugeV1 = 233,
    /// `weapons.rishi_v2`
    WeaponsRishiV2 = 234,
    /// `weapons.rose_break_extension_v1`
    WeaponsRoseBreakExtensionV1 = 235,
    /// `weapons.rose_v1`
    WeaponsRoseV1 = 236,
    /// `weapons.snowman_v1`
    WeaponsSnowmanV1 = 237,
    /// `weapons.spider_v1`
    WeaponsSpiderV1 = 238,
    /// `weapons.the_wrong_gate_healing_v1`
    WeaponsTheWrongGateHealingV1 = 239,
    /// `weapons.thief_candy_v1`
    WeaponsThiefCandyV1 = 240,
    /// `weapons.tiger_tally_v2`
    WeaponsTigerTallyV2 = 241,
    /// `weapons.tuansanlang_v1`
    WeaponsTuansanlangV1 = 242,
    /// `weapons.twinbirds_v1`
    WeaponsTwinbirdsV1 = 243,
    /// `weapons.whale_v1`
    WeaponsWhaleV1 = 244,
    /// `weapons.worldrain_v1`
    WeaponsWorldrainV1 = 245,
    /// `weapons.yaodao_separate_packet_v1`
    WeaponsYaodaoSeparatePacketV1 = 246,
    /// `weapons.yaodao_v1`
    WeaponsYaodaoV1 = 247,
    /// `weapons.yuren_v1`
    WeaponsYurenV1 = 248,
    /// `zankou.normal_chain_v1`
    ZankouNormalChainV1 = 249,
    /// `zankou.ordinary_hit_energy_v1`
    ZankouOrdinaryHitEnergyV1 = 250,
    /// `zero.a1_first_hit_v1`
    ZeroA1FirstHitV1 = 251,
    /// `zero.aoe_all_in_range_dummy_v1`
    ZeroAoeAllInRangeDummyV1 = 252,
    /// `zero.burst_a3_crit_rate_v1`
    ZeroBurstA3CritRateV1 = 253,
    /// `zero.burst_a4_damage_bonus_v1`
    ZeroBurstA4DamageBonusV1 = 254,
    /// `zero.burst_all_in_range_v1`
    ZeroBurstAllInRangeV1 = 255,
    /// `zero.burst_packets_v1`
    ZeroBurstPacketsV1 = 256,
    /// `zero.normal_chain_v1`
    ZeroNormalChainV1 = 257,
    /// `zero.ordinary_hit_energy_v1`
    ZeroOrdinaryHitEnergyV1 = 258,
    /// `zero.skill_packets_v1`
    ZeroSkillPacketsV1 = 259,
    /// `combat.hp_cost_fixture_v1`
    CombatHpCostFixtureV1 = 260,
    /// `sets.devils_blood_v1`
    SetsDevilsBloodV1 = 261,
    /// `combat.live_break_damage_v1`
    CombatLiveBreakDamageV1 = 262,
    /// `daffodil.resonance_skill_capture_v1`
    DaffodilResonanceSkillCaptureV1 = 263,
    /// `daffodil.insight_break_capture_v1`
    DaffodilInsightBreakCaptureV1 = 264,
    /// `daffodil.broken_chaos_resistance_v1`
    DaffodilBrokenChaosResistanceV1 = 265,
    /// `sets.diabolos_v1`
    SetsDiabolosV1 = 266,
    /// `sets.street_boxer_v1`
    SetsStreetBoxerV1 = 267,
    /// `combat.support_dispatch_v1`
    CombatSupportDispatchV1 = 268,
    /// `combat.esper_cycle_reactions_v1`
    CombatEsperCycleReactionsV1 = 269,
    /// `character.zankou.vile_ash_dot_scheduling_v2`
    CharacterZankouVileAshDotSchedulingV2 = 270,
    /// `weapons.nest_bird_v2`
    WeaponsNestBirdV2 = 271,
    /// `lacrimosa.nightmare_all_targets_v1`
    LacrimosaNightmareAllTargetsV1 = 272,
    /// `weapons.twinbirds_v2`
    WeaponsTwinbirdsV2 = 273,
    /// `sets.diabolos_v2`
    SetsDiabolosV2 = 274,
    /// `combat.stain_switch_v1`
    CombatStainSwitchV1 = 275,
    /// `combat.remora_charge_v1`
    CombatRemoraChargeV1 = 276,
    /// `sets.street_boxer_v2`
    SetsStreetBoxerV2 = 277,
    /// `character.zankou.crimson_reverie_v1`
    CharacterZankouCrimsonReverieV1 = 278,
    /// `character.zankou.silent_sunset_v1`
    CharacterZankouSilentSunsetV1 = 279,
    /// `character.zankou.heartwrench_skill_v1`
    CharacterZankouHeartwrenchSkillV1 = 280,
    /// `combat.support_dispatch_v2`
    CombatSupportDispatchV2 = 281,
    /// `weapons.police_rat_v3`
    WeaponsPoliceRatV3 = 282,
    /// `zero.support_packets_v1`
    ZeroSupportPacketsV1 = 283,
    /// `combat.support_dispatch_v3`
    CombatSupportDispatchV3 = 284,
    /// `sakiri.support_packets_v1`
    SakiriSupportPacketsV1 = 285,
    /// `haniel.harmony_ensemble_v1`
    HanielHarmonyEnsembleV1 = 286,
    /// `iroi.live_atk_healing_v1`
    IroiLiveAtkHealingV1 = 287,
    /// `nanally.support_packets_v1`
    NanallySupportPacketsV1 = 288,
    /// `combat.support_dispatch_v4`
    CombatSupportDispatchV4 = 289,
    /// `weapons.gold_record_v2`
    WeaponsGoldRecordV2 = 290,
    /// `weapons.jiaojuan_v2`
    WeaponsJiaojuanV2 = 291,
    /// `weapons.kite_v2`
    WeaponsKiteV2 = 292,
    /// `weapons.koinobori_v2`
    WeaponsKoinoboriV2 = 293,
    /// `chiz.normal_grain_v1`
    ChizNormalGrainV1 = 294,
    /// `weapons.good_boy_v2`
    WeaponsGoodBoyV2 = 295,
    /// `combat.control_effects_v1`
    CombatControlEffectsV1 = 296,
    /// `sakiri.suppress_v1`
    SakiriSuppressV1 = 297,
    /// `weapons.good_boy_v3`
    WeaponsGoodBoyV3 = 298,
    /// `jiuyuan.natural_pact_v1`
    JiuyuanNaturalPactV1 = 299,
    /// `weapons.reality_refuge_v2`
    WeaponsRealityRefugeV2 = 300,
    /// `combat.stamina_v1`
    CombatStaminaV1 = 301,
    /// `nanally.critical_riposte_v1`
    NanallyRiposteV1 = 302,
    /// `mint.riposte_v1`
    MintRiposteV1 = 303,
    /// `mint.plunge_v1`
    MintPlungeV1 = 304,
    /// `mint.hold_attack_v1`
    MintHoldAttackV1 = 305,
    /// `mint.passive2_entry_def_v1`
    MintPassive2EntryDefV1 = 306,
    /// `sakiri.critical_riposte_v1`
    SakiriRiposteV1 = 307,
    /// `iroi.critical_riposte_v1`
    IroiRiposteV1 = 308,
    /// `iroi.plunge_v1`
    IroiPlungeV1 = 309,
    /// `iroi.daydream_branch_v1`
    IroiDaydreamBranchV1 = 310,
    /// `iroi.dream_state_v1`
    IroiDreamStateV1 = 311,
    /// `iroi.regression_v1`
    IroiRegressionV1 = 312,
    /// `iroi.bud_copy_b2_v1`
    IroiBudCopyB2V1 = 313,
    /// `adler.critical_riposte_v1`
    AdlerRiposteV1 = 314,
    /// `adler.plunge_v1`
    AdlerPlungeV1 = 315,
    /// `adler.normal_chain_13_v1`
    AdlerNormalChain13V1 = 316,
    /// `adler.burst_intervals_v1`
    AdlerBurstIntervalsV1 = 317,
    /// `adler.righteous_heart_v1`
    AdlerRighteousHeartV1 = 318,
    /// `adler.blessing_party_interception_v1`
    AdlerBlessingPartyInterceptionV1 = 319,
    /// `aurelia.critical_riposte_v1`
    AureliaRiposteV1 = 320,
    /// `aurelia.plunge_v1`
    AureliaPlungeV1 = 321,
    /// `aurelia.cadenza_power_select_v1`
    AureliaCadenzaPowerSelectV1 = 322,
    /// `aurelia.staccato_v1`
    AureliaStaccatoV1 = 323,
    /// `aurelia.staccato_perfect_dodge_v1`
    AureliaStaccatoPerfectDodgeV1 = 324,
    /// `aurelia.crescendo_b4_v1`
    AureliaCrescendoB4V1 = 325,
    /// `aurelia.harmonics_b2_v1`
    AureliaHarmonicsB2V1 = 326,
    /// `aurelia.a2_cadenza_v1`
    AureliaA2CadenzaV1 = 327,
    /// `aurelia.r6_psyche_v1`
    AureliaR6PsycheV1 = 328,
    /// `aurelia.shadow_creed_skill_finish_v1`
    AureliaShadowCreedSkillFinishV1 = 329,
    /// `baicang.power_words_v1`
    BaicangPowerWordsV1 = 330,
    /// `baicang.hold_branches_v1`
    BaicangHoldBranchesV1 = 331,
    /// `baicang.execute_and_fatal_hit_debt_v1`
    BaicangExecuteAndFatalHitDebtV1 = 332,
    /// `baicang.plunge_v1`
    BaicangPlungeV1 = 333,
    /// `baicang.riposte_v1`
    BaicangRiposteV1 = 334,
    /// `combat.packet_cycle_accrual_v1`
    CombatPacketCycleAccrualV1 = 335,
    /// `baicang.moderate_mischief_v1`
    BaicangModerateMischiefV1 = 336,
    /// `baicang.height_aware_plunge_v1`
    BaicangHeightAwarePlungeV1 = 337,
    /// `blackbird.b2_stain_v1`
    BlackbirdB2StainV1 = 338,
    /// `blackbird.b2_nova_v1`
    BlackbirdB2NovaV1 = 339,
    /// `blackbird.primary_packet_groups_v1`
    BlackbirdPrimaryPacketGroupsV1 = 340,
    /// `blackbird.skill_cycle_fill_v1`
    BlackbirdSkillCycleFillV1 = 341,
    /// `baicang.cancel_windows_v1`
    BaicangCancelWindowsV1 = 342,
    /// `baicang.forced_interrupt_v1`
    BaicangForcedInterruptV1 = 343,
    /// `baicang.actor_hitstop_v1`
    BaicangActorHitstopV1 = 344,
    /// `skia.gnaw_entry_state_v1`
    SkiaGnawEntryStateV1 = 345,
    /// `baicang.skia_entry_v1`
    BaicangSkiaEntryV1 = 346,
    /// `baicang.fadia_assist_v1`
    BaicangFadiaAssistV1 = 347,
    /// `baicang.lacrimosa_entry_jump_v1`
    BaicangLacrimosaEntryJumpV1 = 348,
    /// `lacrimosa.bat_evade_exit_v1`
    LacrimosaBatEvadeExitV1 = 349,
    /// `baicang.qualified_support_v1`
    BaicangQualifiedSupportV1 = 350,
    /// `blackbird.ordinary_malice_v1`
    BlackbirdOrdinaryMaliceV1 = 351,
    /// `blackbird.coordinated_bird_v1`
    BlackbirdCoordinatedBirdV1 = 352,
    /// `blackbird.ordinary_clock_v1`
    BlackbirdOrdinaryClockV1 = 353,
    /// `blackbird.ordinary_secondary_v1`
    BlackbirdOrdinarySecondaryV1 = 354,
    /// `blackbird.qualified_support_v1`
    BlackbirdQualifiedSupportV1 = 355,
    /// `blackbird.primary_form_v1`
    BlackbirdPrimaryFormV1 = 356,
    /// `blackbird.evolve_actions_v1`
    BlackbirdEvolveActionsV1 = 357,
    /// `blackbird.evolved_secondary_v1`
    BlackbirdEvolvedSecondaryV1 = 358,
    /// `blackbird.evolved_support_v1`
    BlackbirdEvolvedSupportV1 = 359,
    /// `owner.action_role.character_1042_evolve_qte_v1`
    OwnerActionRoleCharacter1042EvolveQteV1 = 360,
    /// `blackbird.branches_v1`
    BlackbirdBranchesV1 = 361,
    /// `blackbird.cancel_gates_v1`
    BlackbirdCancelGatesV1 = 362,
    /// `blackbird.combat_transitions_v1`
    BlackbirdCombatTransitionsV1 = 363,
    /// `blackbird.explicit_parry_v1`
    BlackbirdExplicitParryV1 = 364,
    /// `chaos.ordinary_crime_v1`
    ChaosOrdinaryCrimeV1 = 365,
    /// `chaos.ordinary_clock_v1`
    ChaosOrdinaryClockV1 = 366,
    /// `chaos.primary_form_v1`
    ChaosPrimaryFormV1 = 367,
    /// `chaos.forced_interrupt_v1`
    ChaosForcedInterruptV1 = 368,
    /// `chaos.neutral_verdict_v1`
    ChaosNeutralVerdictV1 = 369,
    /// `chaos.jump_cancel_v1`
    ChaosJumpCancelV1 = 370,
    /// `chaos.normal_combo_verdict_v1`
    ChaosNormalComboVerdictV1 = 371,
    /// `chaos.warrant_transfer_v1`
    ChaosWarrantTransferV1 = 372,
    /// `chaos.qualified_support_v1`
    ChaosQualifiedSupportV1 = 373,
    /// `chaos.verdict_continuation_v1`
    ChaosVerdictContinuationV1 = 374,
    /// `chaos.secondary_actions_v1`
    ChaosSecondaryActionsV1 = 375,
    /// `hathor.b2_remora_v1`
    HathorB2RemoraV1 = 376,
    /// `chaos.b2_remora_end_v1`
    ChaosB2RemoraEndV1 = 377,
    /// `inputs.dash_directions_v1`
    InputsDashDirectionsV1 = 378,
    /// `weapons.gold_wool_chaos_realfinish_v1`
    WeaponsGoldWoolChaosRealfinishV1 = 379,
    /// `chiz.exact_grain_v1`
    ChizExactGrainV1 = 380,
    /// `chiz.charge_passives_v1`
    ChizChargePassivesV1 = 381,
    /// `chiz.ordinary_clock_v1`
    ChizOrdinaryClockV1 = 382,
    /// `chiz.exact_finance_v1`
    ChizExactFinanceV1 = 383,
    /// `chiz.pressed_skill_v1`
    ChizPressedSkillV1 = 384,
    /// `chiz.held_skill_v1`
    ChizHeldSkillV1 = 385,
    /// `chiz.held_full_v1`
    ChizHeldFullV1 = 386,
    /// `chiz.action_cancels_v1`
    ChizActionCancelsV1 = 387,
    /// `chiz.secondary_actions_v1`
    ChizSecondaryActionsV1 = 388,
    /// `chiz.market_v1`
    ChizMarketV1 = 389,
    /// `chiz.recorded_market_trace_v1`
    ChizRecordedMarketTraceV1 = 390,
    /// `chiz.barren_v1`
    ChizBarrenV1 = 391,
    /// `chiz.battle_entry_v1`
    ChizBattleEntryV1 = 392,
    /// `daffodil.ordinary_clock_v1`
    DaffodilOrdinaryClockV1 = 393,
    /// `daffodil.action_cancels_v1`
    DaffodilActionCancelsV1 = 394,
    /// `daffodil.secondary_actions_v1`
    DaffodilSecondaryActionsV1 = 395,
    /// `daffodil.phantom_step_v1`
    DaffodilPhantomStepV1 = 396,
    /// `daffodil.held_normal_v1`
    DaffodilHeldNormalV1 = 397,
    /// `combat.discord_break_gauge_v1`
    CombatDiscordBreakGaugeV1 = 398,
    /// `daffodil.discord_cap_reduction_v1`
    DaffodilDiscordCapReductionV1 = 399,
    /// `edgar.full_kit_v1`
    EdgarFullKitV1 = 400,
    /// `fadia.incoming_share_v1`
    FadiaIncomingShareV1 = 401,
    /// `fadia.ordinary_clock_v1`
    FadiaOrdinaryClockV1 = 402,
    /// `fadia.full_kit_v1`
    FadiaFullKitV1 = 403,
    /// `haniel.nova_extraction_v1`
    HanielNovaExtractionV1 = 404,
    /// `haniel.ordinary_clock_v1`
    HanielOrdinaryClockV1 = 405,
    /// `haniel.cannon_ricochet_v1`
    HanielCannonRicochetV1 = 406,
    /// `haniel.full_kit_v1`
    HanielFullKitV1 = 407,
    /// `hathor.ordinary_clock_v1`
    HathorOrdinaryClockV1 = 408,
    /// `hathor.tap_power_v1`
    HathorTapPowerV1 = 409,
    /// `hathor.gaze_v1`
    HathorGazeV1 = 410,
    /// `hathor.held_skill_v1`
    HathorHeldSkillV1 = 411,
    /// `combat.incoming_defense_v1`
    CombatIncomingDefenseV1 = 412,
    /// `hathor.burst_emergency_v1`
    HathorBurstEmergencyV1 = 413,
    /// `hathor.hybrid_emergency_v1`
    HathorHybridEmergencyV1 = 414,
    /// `hathor.cyclone_v1`
    HathorCycloneV1 = 415,
    /// `hathor.combat_lifecycle_v1`
    HathorCombatLifecycleV1 = 416,
    /// `hathor.secondary_actions_v1`
    HathorSecondaryActionsV1 = 417,
    /// `hathor.airattack_v1`
    HathorAirattackV1 = 418,
    /// `hathor.branch_v1`
    HathorBranchV1 = 419,
    /// `hotori.katana_chain_v1`
    HotoriKatanaV1 = 420,
    /// `hotori.mode_damage_v1`
    HotoriModeDamageV1 = 421,
    /// `hotori.recorded_replay_v1`
    HotoriRecordedReplayV1 = 422,
    /// `hotori.time_stop_clock_v1`
    HotoriTimeStopClockV1 = 423,
    /// `hotori.timepiece_v1`
    HotoriTimepieceV1 = 424,
    /// `nanally.cycle_followups_v1`
    NanallyCycleFollowUpsV1 = 425,
    /// `nanally.blossom_enhancement_v1`
    NanallyBlossomEnhancementV1 = 426,
    /// `nanally.combat_lifecycle_v1`
    NanallyCombatLifecycleV1 = 427,
    /// `nanally.authority_removal_v1`
    NanallyAuthorityRemovalV1 = 428,
    /// `nanally.action_clock_v1`
    NanallyActionClockV1 = 429,
    /// `linko.xiaozhen_pet_v1`
    LinkoXiaozhenPetV1 = 430,
    /// `lacrimosa.sleep_in_v1`
    LacrimosaSleepInV1 = 431,
    /// `mint.ordinary_clock_v1`
    MintOrdinaryClockV1 = 432,
    /// `iroi.action_clock_v1`
    IroiActionClockV1 = 433,
    /// `lacrimosa.broken_shatter_v1`
    LacrimosaBrokenShatterV1 = 434,
    /// `zankou.forms_v1`
    ZankouFormsV1 = 435,
    /// `zankou.form_modifiers_v1`
    ZankouFormModifiersV1 = 436,
    ZankouCombatPassivesV1 = 437,
    ZankouStoryModeV1 = 438,
    SakiriBurstAtkV1 = 439,
    CombatRelativeClockPolicyV1 = 440,
    AdlerTemperanceV1 = 441,
    ZeroCosmosCycleAtkV1 = 442,
    CombatSinglePacketSupportV1 = 443,
    LacrimosaA6FreeReactionEntryV1 = 444,
    /// `weapons.echoes_on_hold_v1`
    WeaponsEchoesOnHoldV1 = 445,
    /// `akane.ordinary_clock_v1`
    AkaneOrdinaryClockV1 = 446,
    /// `akane.normal_chain_v1`
    AkaneNormalChainV1 = 447,
    /// `akane.ordinary_hit_energy_v1`
    AkaneOrdinaryHitEnergyV1 = 448,
    /// `akane.string_energy_v1`
    AkaneStringEnergyV1 = 449,
    /// `akane.rhythm_wave_v1`
    AkaneRhythmWaveV1 = 450,
    /// `akane.sonic_boom_v1`
    AkaneSonicBoomV1 = 451,
    /// `akane.perfect_pitch_v1`
    AkanePerfectPitchV1 = 452,
    /// `akane.awakenings_v1`
    AkaneAwakeningsV1 = 453,
    /// `akane.aoe_all_in_range_dummy_v1`
    AkaneAoeAllInRangeDummyV1 = 454,
    /// `akane.secondary_actions_v1`
    AkaneSecondaryActionsV1 = 455,
}

struct Hypothesis {
    id: &'static str,
    description: &'static str,
}

static HYPOTHESES: [Hypothesis; 456] = [
    Hypothesis {
        id: "ability_index.primary_action_role_v1",
        description: "Interpret DT_CharacterAbilityConfig.CharacterAbilityList[].Value.AbilityIndex values 1, 2, 3, and 4 as Normal, RedirectSkill, Ultimate, and Qte action roles. This is an explicit model mapping kept separate from AbilityGameplayTag and does not establish native timing, reachability, cost, or weapon trigger semantics.",
    },
    Hypothesis {
        id: "actions.caller_timing_v1",
        description: "Authored action duration advances world and simulation clocks; native animation duration and cancel windows are unresolved.",
    },
    Hypothesis {
        id: "actions.primary_packet_v1",
        description: "The action's primary damage effect delivers at the start of the action window under the caller's target and coefficient assumptions; native sub-hit packet schedules are unresolved.",
    },
    Hypothesis {
        id: "adler.a5_burst_energy_refund_v1",
        description: "When Adler awakening effect 5 is selected, one successful native GA_Adler_UltraSkill admission schedules a same-time ProcCallback restoring 20 whole Ultimate Energy to the captured Adler. This flat refund uses restore_energy, so Charge Efficiency and fractional carry do not scale or consume it; capacity excess is overflow. It is independent of hit delivery, target count and later field switches, but requires the captured owner to remain alive when the callback runs. Frozen 1033AwakenEffect.json Effect5 localized text says Adler gains 20 Ultimate Energy when casting Tranquility and DT_AdlerEffectFigure.Adler_lv5=20; the exact native timing/applicability remains unverified. Optional PC_140 ChargeExecution20 does not establish runtime wiring. Other awakening mechanics remain pending.",
    },
    Hypothesis {
        id: "adler.blessing_a4_overflow_v1",
        description: "Frozen 1033AwakenEffect.json Effect4 says Blessing absorbs overflow based on 50% of Adler DEF when it breaks; the referenced blueprint is unavailable, so settlement details are explicit hypotheses. A positive, nonexpired Adler Blessing whose source selected effect 4 and which is depleted by ordinary incoming shield settlement triggers one cap per distinct living source whose Blessing was depleted. Each cap is round(live exact effective DEF/2), sampled at incoming settlement using base DEF*(1+static+live DEF%)+static+live flat DEF, and applies after ordinary shields but before HP/overkill in broken-Blessing absorption order. Live rather than cast-time DEF, nearest-integer rounding, one event only, living-source eligibility, no ShieldEfficiency/A3 multiplier, no stored remainder, and first-broken-shield ordering across multiple sources are chosen bounded rules because Effect4 text and the missing blueprint do not establish those branches. Exact depletion reaches the rule but absorbs zero; expiry, removal or replacement does not trigger it. No shield, healing or revival is created.",
    },
    Hypothesis {
        id: "adler.blessing_r6_v1",
        description: "At hit delivery, a living Adler with six selected awakening effects grants one additive +10 percentage-point Incantation damage bonus to all team Incantation packets while any living actor holds that Adler source's positive, nonexpired Blessing shield. The packet owner and active field actor do not affect eligibility; shield depletion, half-open expiry, recipient death or source Adler death removes the bonus. Multiple eligible sources enable the same single +10% contribution without stacking. This is a tested bounded hypothesis based on 1033AwakenEffect.json resonance_6 text; the referenced Buff_Adler033_Level6_extra blueprint does not expose its property or protected-actor predicate in frozen evidence. DamageUpIncantationBase identifies the matching native-style stat, and the bonus is modeled in the additive damage-bonus zone.",
    },
    Hypothesis {
        id: "adler.blessing_shield_v1",
        description: "On successful GA_Adler_Skill admission, capture the current active actor and Adler's exact effective DEF (base DEF scaled by static+live DefUp, then static+live flat DEF); a same-time ProcCallback grants that captured actor a shield using the native flat+coefficient curve at the effective Skill level. The grant samples Adler's static+live outgoing ShieldEfficiency and adds 20% capacity only when awakening effect 3 is selected, in the same additive shield-bonus zone, then rounds once. Resonance count does not substitute for effect 3. Recasting replaces only this Adler source shield on that recipient; the shield persists across switching, expires at its half-open deadline, and recipient death clears it. Shield amount is a cast-time snapshot. This is a tested bounded hypothesis because native source tables do not establish recipient snapshot, stat snapshot or ShieldEfficiency stacking.",
    },
    Hypothesis {
        id: "adler.burst_sunyas_v1",
        description: "Frozen GA_Adler_UltraSkill text specifies five Sunya strikes per enemy, doubled when only one enemy remains. One admitted Burst delivers GE_Player_Adler_UltraSkill_Damage five times per living declared target, or ten times if exactly one target is alive at admission. Each packet uses its own admitted effective native DEF coefficient and delivery-time modifiers, crit and target survival. All declared targets are eligible because the description says all enemy targets. Admission-time population capture, strike-major target order and coincident delivery at the primary impact timestamp are tested bounded hypotheses pending native selection/interval evidence; a later defeat cannot expand the captured train. One action spends energy once and keeps the admitted owner across switches.",
    },
    Hypothesis {
        id: "adler.karma_v1",
        description: "A connected modeled Deliverance normal-chain packet grants one held Karma stack; a connected aimed Liberation packet grants two; each grant is capped at 20. A successful Tranquility admission adds 20 stacks only when effect 1 is selected, up to the same cap. Skill admission captures and clears all held stacks before damage. Optional PC_140 decoded AdlerSkill_DamUp and Buff_AdlerQiHai_SkillDamUP assets reconstruct additive FinalDamageUp of 2% per captured stack, selected as 3% only when effect 2 is selected; Global cutoff applicability and runtime reachability are unproven. The frozen .01 Adler_lv2 figure and localized descriptions conflict, and a separate live-held A2 general damage bonus is omitted to avoid double application of this Skill carrier. Karma gain follows its connected packet; zero-damage, unmodeled melee siblings, and other Karma producers remain unsupported.",
    },
    Hypothesis {
        id: "adler.normal_chain_v1",
        description: "Five admitted ordinary inputs advance stages Melee1 through Melee5 with multiplicities 1,1,7,2,1 (twelve hits: Melee1, Melee2, 7x Melee3, Melee4_1+Melee4_2, Melee5). The chain loops, retaining native effective talent projection including resonance_3.",
    },
    Hypothesis {
        id: "adler.ordinary_hit_energy_v1",
        description: "For Adler character:1033 only, modeled ordinary Normal, aimed Normal, and GA_Adler_Skill native packets grant their exact nonnegative DT_SkillDamageData.ChargeAdd once after positive actual HP loss against target zero. Fatal damage qualifies; zero HP loss, unavailable targets and cancelled packets do not. Skill2's native initial row has ChargeAdd 4; Skill3 siblings and Skill_Damage periodic rows have zero and grant nothing. The living source owner receives 1x; each other living configured recipient receives the explicit source-element to recipient-element coefficient from frozen DT_SkillTeamChargeRelationData (the six recipient-element rows for Adler INCANTATION are 0.6). Each recipient applies their own Charge Efficiency and exact fractional carry, then existing capacity and overflow accounting. Delivery-time owner and source attribution survive switching. Burst, defensive passives, alternate forms, summons, Branch, AirAttack, QTE, non-native branches, additional targets and callbacks outside the modeled packet train remain unsupported. Eligibility, cadence, recipient and off-field rules are tested hypotheses because native caller evidence is unavailable; ChargeAdd and relation coefficients are frozen native rows.",
    },
    Hypothesis {
        id: "adler.skill_all_in_range_dummy_v1",
        description: "For one admitted GA_Adler_Skill command, capture every living scenario-declared dummy in ascending target_index at admission; the product scenario assumes each is in range and in the attack area, with no geometry, recipient cap, or distribution check. Schedule one modeled Skill2/Skill3x2 initial train and one +1s through +10s periodic train per captured target. The target-zero primary packet supplies its Skill2; child packets supply Skill2 on other captured targets. Initial packet delivery and each periodic stream recheck target survival; all streams share owner survival and the cast generation, so recasting replaces them together. One Skill command spends its action cost once. Additional Skill2 packets do not extend the currently target-zero-only energy adapter. This is a product dummy semantic, not a native target selection/count/cap/geometry claim.",
    },
    Hypothesis {
        id: "adler.skill_train_v1",
        description: "Under the separate adler.skill_all_in_range_dummy_v1 product assumption, schedule one Skill2 and two Skill3 coincident initial DEF packets and GE_Player_Adler_Skill_Damage at +1s through +10s inclusive for each living captured target. The mapped initial Skill2 row and published 70% DEF x3 description disagree about multiplicity; Skill3 activation is not proven, and using it twice is a bounded numeric-three-hit hypothesis. The 1s/10s frozen figure curves ground the tick train, but exact impact/refresh timing is assumed. At Skill admission, consumed Karma is captured and cleared; each captured stack adds 2% FinalDamageUp, or 3% only when Adler awakening effect 2 is selected, to every packet admitted by that Skill command, including initial packets that deliver after a later recast. This uses optional PC_140 decoded AdlerSkill_DamUp and Buff_AdlerQiHai_SkillDamUP evidence (Global cutoff applicability and runtime reachability unproven); the frozen .01 A2 figure and localized descriptions conflict, and no separate held-stack general A2 bonus is modeled to avoid double applying the reconstructed FinalDamageUp carrier. Recast cancels only the old periodic train; living owners continue off-field, defeated owners cancel, and each defeated target stops its own future ticks. One admission spends action cost once; complete kit behavior remains pending.",
    },
    Hypothesis {
        id: "aim.mode_transitions_v1",
        description: "Adler, Nanally and Jiuyuan Aim enters a persistent shooting mode; each ordinary attack input selects the frozen Aim packet. A repeated Aim press toggles the mode off. Admitted dash, Skill, Ultimate or field exit exits aiming; no-op swap and waits preserve it. Reset and toggle rules follow the reported player controls pending native transition evidence. Aim entry itself has no damage. Crosshair accuracy, travel, AoE and complete kits remain unsupported.",
    },
    Hypothesis {
        id: "aurelia.airattack_evadebranch_direct_packet_energy_v1",
        description: "For Aurelia character:1070 only, an explicitly modeled GA_Mitsuki_Melee AirAttack or EvadeBranch direct packet grants its frozen DT_SkillDamageData.ChargeAdd once after positive actual HP loss against target zero (6.5 and 1.3 respectively). Fatal damage qualifies; zero HP loss, unavailable targets and cancelled packets do not. The living source owner receives 1x; each other living configured recipient receives the frozen source-element to recipient-element coefficient from DT_SkillTeamChargeRelationData (3/5 for Aurelia PSYCHE), then applies their own Charge Efficiency, exact fractional carry, capacity and overflow accounting. This exact direct-input policy is a tested hypothesis, not evidence of native eligibility. Natural AirAttack/EvadeBranch dispatch, jump/dodge legality, stamina, costs, recovery, timing, powered variants, extra packets, other targets and callbacks remain unresolved.",
    },
    Hypothesis {
        id: "aurelia.aoe_all_in_range_dummy_v1",
        description: "Aurelia Cadenza Aria and Canon Chorus deliver their full packet trains to all living scenario-declared dummies under the all-in-range dummy contract: by scenario definition all declared dummies are in range. Targets are delivered in ascending target_index order at the primary impact time. Spatial geometry, recipient caps, distribution and falloff remain unresolved.",
    },
    Hypothesis {
        id: "aurelia.direct_packet_inputs_v1",
        description: "An explicit InvokeAbility command for GA_Mitsuki_Melee may select the frozen GE_Player_Mitsuki_AirAttack_Damage or GE_Player_Mitsuki_EvadeBranch_Damage row as one direct packet at the caller-selected time, using the configured Normal talent coordinate including resonance_3. This packet does not advance or reset the modeled Normal chain and receives no ordinary-hit energy grant. The command does not model jump/dodge legality, natural dispatch, cost, recovery, travel, projectile tracking, recipients, hit timing, power variants or native energy eligibility.",
    },
    Hypothesis {
        id: "aurelia.normal_chain_v1",
        description: "Aurelia ordinary Legato advances four admitted stages with multiplicities 1,1,3,4 matching frozen descriptions and native Melee1..4 coefficients at all nine talent levels. It loops after stage four and resets on a different admitted ability, field exit or an input-start gap greater than 2 seconds. Reset rules and ordered coincident delivery at the research primary impact time are tested hypotheses pending animation/cancel evidence. This models ordinary unenhanced packets; powered projectile triggers, tracking, AoE, Staccato, plunge, riposte, Crescendo and complete kit behavior remain unsupported.",
    },
    Hypothesis {
        id: "aurelia.ordinary_hit_energy_v1",
        description: "For Aurelia character:1070 only, an ordinary Normal or Skill native packet grants its exact nonnegative DT_SkillDamageData.ChargeAdd once after positive actual HP loss against target zero. Fatal damage qualifies; zero HP loss, unavailable targets and cancelled packets do not. The living source owner receives 1x; each other living configured recipient receives the explicit source-element to recipient-element coefficient from frozen DT_SkillTeamChargeRelationData (the six recipient-element rows for Aurelia PSYCHE are 0.6). Each recipient applies their own Charge Efficiency and exact fractional carry, then existing capacity and overflow accounting. Delivery-time owner and source attribution survive switching. The explicit GE_Player_Mitsuki_AirAttack_Damage and GE_Player_Mitsuki_EvadeBranch_Damage packet inputs are excluded from this energy adapter; native plunge/evade energy eligibility remains unresolved. Burst, enhanced/alternate forms, summons, other Branch packets, QTE, non-native branches, additional targets and callbacks outside the modeled packet train remain unsupported. Eligibility, cadence, recipient and off-field rules are tested hypotheses because native caller evidence is unavailable; ChargeAdd and relation coefficients are frozen native rows.",
    },
    Hypothesis {
        id: "aurelia.skill_burst_packets_v1",
        description: "Aurelia Cadenza Aria delivers one Skill hit on target zero; Canon Chorus delivers six UltraSkill hits (five GE_Player_Mitsuki_UltraSkill_Damage plus one GE_Player_Mitsuki_UltraSkill_Damage2). All children use the admitted effective native talent coordinate and sample live modifiers independently. Ordered coincident delivery at the selected primary impact timestamp is a timing hypothesis pending animation intervals; it does not model travel, AoE coverage, jellyfish tracking, projectile enhancement or complete kit behavior.",
    },
    Hypothesis {
        id: "baicang.aoe_all_in_range_dummy_v1",
        description: "Baicang Judgment of Autumn initial expansion and Skill deliver to all living scenario-declared dummies under the all-in-range dummy contract: by scenario definition all declared dummies are in range. Targets are delivered in ascending target_index order at the primary impact time. Spatial geometry, recipient caps, distribution and falloff remain unresolved.",
    },
    Hypothesis {
        id: "baicang.judgment_of_autumn_domain_v1",
        description: "A successfully admitted native Baicang GA_Cang_UltraSkill keeps the existing GE_Player_Cang_UltraSkill2_Damage expansion hit once and starts one generation of GE_Player_Cang_UltraSkill_Damage on target zero at cast+2s, +4s, and every 2s through the 12s endpoint inclusive (six ticks) or A5 16s endpoint inclusive (eight ticks). Each tick uses the admitted effective Burst talent coordinate and captured owner/action/packet metadata, samples the living owner's current ATK and buffs plus target mitigation at delivery, and retains native FixedCritRate 0.5 under damage.fixed_crit_rate_as_probability_v1. Recast replaces prior pending ticks; a living off-field owner continues, a defeated owner cancels, and a defeated target receives no later hit or revival. Domain buffs keep their existing half-open deadline, so the inclusive endpoint tick does not receive them. One target and the inclusive scheduling/endpoint rules are bounded hypotheses; this does not model the description's execute, fatal immunity, Power Words Silence/Objurgate/Bless, or the complete kit. If a domain expansion or tick leaves a living target below 10% HP, execution rejects because the required execute branch is unsupported; target kind is unavailable, so this conservatively uses the description's 10% threshold.",
    },
    Hypothesis {
        id: "baicang.normal_chain_v1",
        description: "Baicang ordinary Normal advances five stages with multiplicities 1,1,1,7,1 from frozen neverness_cang descriptions and native DT_SkillDamageData coefficients. The chain loops after stage five and resets on a different admitted ability, field exit or a gap greater than 2 seconds between input starts. Reset rules, gap and ordered coincident delivery at the research primary impact time are tested hypotheses pending animation/cancel evidence. Heavy attack variations, freeze interactions and complete kit behavior remain unsupported.",
    },
    Hypothesis {
        id: "baicang.ordinary_hit_energy_v1",
        description: "For Baicang character:1023 only, a modeled Normal or Skill native packet grants its exact nonnegative DT_SkillDamageData.ChargeAdd once after positive actual HP loss against target zero. Fatal damage qualifies; zero HP loss, unavailable targets and cancelled packets do not. The living source owner receives 1x; each other living configured recipient receives the explicit source-element to recipient-element coefficient from frozen DT_SkillTeamChargeRelationData (the six recipient-element rows for Baicang INCANTATION are 0.6). Each recipient applies their own Charge Efficiency and exact fractional carry, then existing capacity and overflow accounting. Delivery-time owner and source attribution survive switching. Burst, enhanced/alternate forms, summons, Branch, AirAttack, QTE, non-native branches, additional targets and callbacks outside the modeled packet train remain unsupported. Eligibility, cadence, recipient and off-field rules are tested hypotheses because native caller evidence is unavailable; ChargeAdd and relation coefficients are frozen native rows.",
    },
    Hypothesis {
        id: "blackbird.aoe_all_in_range_dummy_v1",
        description: "Blackbird Netherplume Rite and Requies Animarum deliver their full packet trains to all living scenario-declared dummies under the all-in-range dummy contract: by scenario definition all declared dummies are in range. Targets are delivered in ascending target_index order at the primary impact time. Spatial geometry, recipient caps, distribution and falloff remain unresolved.",
    },
    Hypothesis {
        id: "blackbird.normal_chain_v1",
        description: "Blackbird ordinary Normal advances five stages with multiplicities 1,2,4,5,5 from frozen neverness_blackbird descriptions and native DT_SkillDamageData coefficients. The chain loops after stage five and resets on a different admitted ability, field exit or a gap greater than 2 seconds between input starts. Reset rules, gap and ordered coincident delivery at the research primary impact time are tested hypotheses pending animation/cancel evidence. Evolve forms, Malice, travel, AoE, plunge, riposte and complete kit behavior remain unsupported.",
    },
    Hypothesis {
        id: "blackbird.ordinary_hit_energy_v1",
        description: "For Blackbird character:1042 only, an ordinary Normal or Skill native packet grants its exact nonnegative DT_SkillDamageData.ChargeAdd once after positive actual HP loss against target zero. Fatal damage qualifies; zero HP loss, unavailable targets and cancelled packets do not. The living source owner receives 1x; each other living configured recipient receives the explicit source-element to recipient-element coefficient from frozen DT_SkillTeamChargeRelationData (the six recipient-element rows for Blackbird PSYCHE are 0.6). Each recipient applies their own Charge Efficiency and exact fractional carry, then existing capacity and overflow accounting. Delivery-time owner and source attribution survive switching. Burst, enhanced/alternate forms, summons, Branch, AirAttack, QTE, non-native branches, additional targets and callbacks outside the modeled packet train remain unsupported. Eligibility, cadence, recipient and off-field rules are tested hypotheses because native caller evidence is unavailable; ChargeAdd and relation coefficients are frozen native rows.",
    },
    Hypothesis {
        id: "blackbird.skill_burst_packets_v1",
        description: "Blackbird Netherplume Rite delivers four Skill hits on target zero; Requies Animarum delivers nine UltraSkill hits. All children use the admitted effective native talent coordinate and sample live modifiers independently. Ordered coincident delivery at the selected primary impact timestamp is a timing hypothesis pending animation intervals; it does not model travel, AoE coverage, Evolve forms, Malice gauge or complete kit behavior.",
    },
    Hypothesis {
        id: "build.action_projection_v1",
        description: "Selected action projection policy applies native skill levels, multipliers, and awakening bonuses to configured actions.",
    },
    Hypothesis {
        id: "build.base_overrides_v1",
        description: "Explicit caller base overrides replace source-bound native base stats in this research projection.",
    },
    Hypothesis {
        id: "build.gear_projection_v1",
        description: "Selected gear projection policy applies native weapon refinement curves and equipment set passives to configured actors.",
    },
    Hypothesis {
        id: "build.stat_quantization_v1",
        description: "For native static build projection, Studio and explicit target stat packs, derive Max HP and DEF from the exact sum of base, percentage and flat contributions, then quantize once to the nearest whole point with exact halves upward. This is a bounded product runtime policy, not verified native game behavior. It does not define damage, healing or shield settlement rounding; native stat quantization remains open.",
    },
    Hypothesis {
        id: "build.static_additive_v1",
        description: "Additive stat projection sums module, weapon, and talent contributions within each stat zone before combat entry.",
    },
    Hypothesis {
        id: "build.unmodeled_native_attributes_v1",
        description: "Native character and weapon attributes not represented in the simulator model are omitted from combat calculations.",
    },
    Hypothesis {
        id: "chaos.aoe_all_in_range_dummy_v1",
        description: "Chaos Doubtmark and Retribution Suppression deliver their full packet trains to all living scenario-declared dummies under the all-in-range dummy contract: by scenario definition all declared dummies are in range. Targets are delivered in ascending target_index order at the primary impact time. Spatial geometry, recipient caps, distribution and falloff remain unresolved.",
    },
    Hypothesis {
        id: "chaos.normal_chain_v1",
        description: "Chaos ordinary Normal advances five stages with multiplicities 1,2,2,4,3 from frozen neverness_chaos descriptions and native DT_SkillDamageData coefficients. The chain loops after stage five and resets on a different admitted ability, field exit or a gap greater than 2 seconds between input starts. Reset rules, gap and ordered coincident delivery at the research primary impact time are tested hypotheses pending animation/cancel evidence. Crime resource accumulation, Warrant multi-target extensions, and branch/plunge attacks remain unsupported.",
    },
    Hypothesis {
        id: "chaos.ordinary_hit_energy_v1",
        description: "For Chaos character:1071 only, an ordinary Normal or Skill native packet grants its exact nonnegative DT_SkillDamageData.ChargeAdd once after positive actual HP loss against target zero. Fatal damage qualifies; zero HP loss, unavailable targets and cancelled packets do not. The living source owner receives 1x; each other living configured recipient receives the explicit source-element to recipient-element coefficient from frozen DT_SkillTeamChargeRelationData (the six recipient-element rows for Chaos LAKSHANA are 0.6). Each recipient applies their own Charge Efficiency and exact fractional carry, then existing capacity and overflow accounting. Delivery-time owner and source attribution survive switching. Burst, AirAttack, Branch, PerfectEvadeAttack, _Ultra form variants, non-native branches, additional targets and callbacks outside the modeled packet train remain unsupported. The exact QTE packet input has a separate bounded energy hypothesis. Eligibility, cadence, recipient and off-field rules are tested hypotheses because native caller evidence is unavailable; ChargeAdd and relation coefficients are frozen native rows.",
    },
    Hypothesis {
        id: "chaos.qte_direct_packet_energy_v1",
        description: "For Chaos character:1071 only, the exact explicit GA_Chaos071_QTE / GE_Player_Chaos_QTE1_Damage scenario input grants its frozen DT_SkillDamageData.ChargeAdd (6.4) once after positive actual HP loss against target zero. Fatal damage qualifies; zero HP loss, unavailable targets and cancelled packets do not. The living source owner receives 1x; each other living configured recipient receives the frozen source-element to recipient-element coefficient from DT_SkillTeamChargeRelationData (3/5 for Chaos LAKSHANA), then applies their own Charge Efficiency, exact fractional carry, capacity and overflow accounting through the shared policy. Delivery-time recipient and source ownership follow the shared tested policy. This exact direct-input energy behavior is a bounded scenario hypothesis, not evidence of native QTE eligibility or dispatch. QTE trigger, legality, cost, timing, multiplicity, additional targets and callbacks remain unresolved; other QTE packets are unsupported.",
    },
    Hypothesis {
        id: "chaos.skill_bites_v1",
        description: "Chaos Doubtmark delivers the two native bites once per successful cast on target zero, with the admitted effective talent coordinate for both ATK coefficients. The primary Skill1 packet precedes Skill2 at the selected impact timestamp. Coincident delivery is a timing hypothesis pending native animation evidence; each bite samples current wearer/target modifiers and applies the existing Warrant model. Crime, additional targets and unrelated kit branches remain pending.",
    },
    Hypothesis {
        id: "chaos.suppression_packets_v1",
        description: "Chaos Retribution's Suppression delivers four GE_Player_Chaos_UltraSkill1_Damage packets followed by one GE_Player_Chaos_UltraSkill2_Damage packet on target zero from one successful cast. The primary UltraSkill1 is first; three more UltraSkill1 children and one UltraSkill2 child use the admitted effective Burst talent coordinate at the primary impact timestamp. Ordered coincident delivery is a timing hypothesis pending native animation evidence. Every packet samples live modifiers and runs Chaos Warrant, A2 DEF ignore, A5 single-target damage, and A6 Lakshana resistance reduction independently; A3's cast-granted Crit DMG buff is active for the train. Crime, extra targets, and unrelated branches remain pending.",
    },
    Hypothesis {
        id: "character.adler.skill_dot_packet_semantics_v1",
        description: "Treat only Adler GE_Player_Adler_Skill_Damage as State.Damage and State.Damage.Dot because its frozen periodic Skill description schedules this exact effect. This packet-semantic mapping is a bounded hypothesis; the separate Skill model owns scheduling, target scope, recast, and duration.",
    },
    Hypothesis {
        id: "character.baicang.dot_packet_semantics_v1",
        description: "Treat Baicang GE_Player_Cang_UltraSkill_Damage as a DoT damage source carrying State.Damage and State.Damage.Dot, as disclosed by the researched Judgment of Autumn description. This owner declaration maps packet semantics; the separate domain model owns scheduling and target scope.",
    },
    Hypothesis {
        id: "character.zankou.dot_packet_semantics_v1",
        description: "Treat the exact Zankou GE_Player_Zankou_DotDamage and GE_Player_Zankou_DotUltraDamage packet rows as DoT damage sources carrying State.Damage, State.Damage.Dot, and State.Damage.Melee. This maps the researched serialized periodic-effect source tags; it does not model stack application, target spreading, or periodic scheduling.",
    },
    Hypothesis {
        id: "character.zankou.vile_ash_dot_scheduling_v1",
        description: "GA_Zankou_UltraSkill (Bloodfeast Reverie) applies 5 Vile Ash stacks per living target, capped at 10, and starts a 30-second tick train: the first tick lands 1s after the Ultra hit and ticks repeat every 1s (30 ticks) while the source lives. Each tick is a native GE_Player_Zankou_DotUltraDamage HitDelivery carrying State.Damage.Dot, with atk_rate = k_native * (1 + clamp(stacks - 1, 0, 10)) read from the train's live stack count at scheduling time. A re-cast adds 5 stacks (cap 10), replaces the deadline and bumps the generation; at most one already-queued stale tick still settles with its snapshot and does not continue the old train. A defeated source stops the train; an off-field source keeps ticking. Ordinary-Skill stack application, Silent Sunset Scorch replacement and per-tick stack re-query beyond the train count remain unmodeled.",
    },
    Hypothesis {
        id: "chiz.aoe_all_in_range_dummy_v1",
        description: "Chiz GA_Chiichan073_Skill delivers its primary GE_Player_Chiichan073_Skill1_1Damage packet to all living scenario-declared dummies under the all-in-range dummy contract: by scenario definition all declared dummies are in range. Targets are delivered in ascending target_index order at the primary impact time. Spatial geometry, recipient caps, distribution and falloff remain unresolved.",
    },
    Hypothesis {
        id: "chiz.normal_chain_v1",
        description: "Five admitted ordinary inputs advance stages Melee1 through Melee5 with multiplicities 1,3,1,5,2 (twelve hits: Melee1, 3x Melee2, Melee3, 5x Melee4, 2x Melee5). The chain loops, retaining native effective talent projection.",
    },
    Hypothesis {
        id: "chiz.ordinary_hit_energy_v1",
        description: "For Chiz character:1073 only, a modeled ordinary Normal native packet grants its exact nonnegative DT_SkillDamageData.ChargeAdd once after positive actual HP loss against target zero. Fatal damage qualifies; zero HP loss, unavailable targets and cancelled packets do not. The living source owner receives 1x; each other living configured recipient receives the explicit source-element to recipient-element coefficient from frozen DT_SkillTeamChargeRelationData (the six recipient-element rows for Chiz COSMOS are 0.6). Each recipient applies their own Charge Efficiency and exact fractional carry, then existing capacity and overflow accounting. Delivery-time owner and source attribution survive switching. This is independent of the existing normal-hit Grain grants. Skill charges, Surplus, Debt, Burst market settlement, alternate forms, summons, Branch, AirAttack, QTE, non-native branches, additional targets and callbacks outside the modeled packet train remain unsupported. Eligibility, cadence, recipient and off-field rules are tested hypotheses because native caller evidence is unavailable; ChargeAdd and relation coefficients are frozen native rows.",
    },
    Hypothesis {
        id: "combat.incoming_damage_fixture_v1",
        description: "Explicit actor-damage events represent already-delivered nonnegative damage after mitigation. At a timestamp, action admissions run before incoming damage; incoming events run before dynamically queued direct hit deliveries. Timed buffs expire before events at their deadline. Ordinary shields absorb first, followed by reached owner-specific overflow absorption; HP loss is min(residual damage, current HP), overkill is the remaining residual, and damage to a defeated actor settles as zero HP loss with all delivered damage reported as overflow. Admitted NativeDirect packets are cancelled with a trace when their owner is defeated before delivery; other postmortem packet ownership remains unsupported. This is a fixture settlement model and does not model enemy damage mitigation or AI.",
    },
    Hypothesis {
        id: "combat.live_healing_modifiers_v1",
        description: "At supported healing callbacks, base healing is multiplied by (1 + the healer's static native/build healing_bonus plus its live healing_bonus modifiers) and then by (1 + the live sum of the current recipient's incoming_healing_bonus), sampled at callback world time with BuffManager expiry and living-source rules. Round the exact result once to nearest integer, then cap it to missing HP through VitalState; defeated recipients cannot be revived. A healer's incoming bonuses and a recipient's outgoing bonuses do not cross over. Native sources do not settle modifier ordering or this interaction; this is an explicit tested formula hypothesis where supported.",
    },
    Hypothesis {
        id: "combat.unbal_value_as_target_gauge_v1",
        description: "For a living target with an explicit break_profile, each positive-damage delivered packet with a positive frozen UnbalValue removes that exact value times the outgoing Break efficiency multiplier from the target gauge. The multiplier is the source-bound UnbalAccrueEfficiencyBase (or explicit scenario value, default one) plus summed live break_efficiency bonuses at ReactionOrBreak settlement, honoring buff recipients, expiry and source lifetime; zero removes no gauge. For an explicitly selected target stat pack, multiply once more by its exact nonnegative UnbalAntiAccrueEfficiencyBase (including inherited row defaults); zero prevents gauge loss. Targets without a stat pack use one. This selects a direct incoming multiplier, not one minus resistance or the enemy outgoing UnbalAccrueEfficiencyBase; the native consumer mapping remains unproved. Reaching zero applies the content-owned ntesim.status.broken tag for the profile duration, then restores gauge to capacity. Rose may extend this modeled Broken deadline only through its separate weapons.rose_break_extension_v1 hypothesis. This is an explicit scenario hypothesis: no target profile means no gauge model; no native efficiency consumer, enemy profile, break damage, BI settlement, or native recovery rule is inferred.",
    },
    Hypothesis {
        id: "crit.fixed_noncrit_preview_v1",
        description: "Fixed non-crit preview evaluates combat calculations assuming no critical strikes occur.",
    },
    Hypothesis {
        id: "daffodil.aoe_all_in_range_dummy_v1",
        description: "Daffodil Resonance (Skill) and Finale (UltraSkill) deliver their full packet trains to all living scenario-declared dummies under the all-in-range dummy contract: by scenario definition all declared dummies are in range. Targets are delivered in ascending target_index order at the primary impact time. Spatial geometry, recipient caps, distribution and falloff remain unresolved.",
    },
    Hypothesis {
        id: "daffodil.branch_entry_direct_packet_energy_v1",
        description: "For Daffodil character:1054 only, an explicitly modeled GA_Daffodill_Melee Branch1-7 or EntryAttack1/2/4 direct packet grants its frozen DT_SkillDamageData.ChargeAdd once after positive actual HP loss against target zero. Fatal damage qualifies; zero HP loss, unavailable targets and cancelled packets do not. The living source owner receives 1x; each other living configured recipient receives the frozen source-element to recipient-element coefficient from DT_SkillTeamChargeRelationData (3/5 for Daffodil CHAOS), then applies their own Charge Efficiency, exact fractional carry, capacity and overflow accounting. This exact direct-input policy is a tested hypothesis, not evidence of native eligibility. Natural Branch/Entry dispatch, animation, timing, target rules, jump/dodge legality, stamina, costs, cadence, extra packets, other targets and callbacks remain unresolved; Branch6/7 are frozen packet rows without a published-tip binding.",
    },
    Hypothesis {
        id: "daffodil.branch_entry_direct_packet_inputs_v1",
        description: "When the caller explicitly selects GE_Player_Daffodill_Branch1_Damage through Branch7 or GE_Player_Daffodill_EntryAttack1_Damage, EntryAttack2_Damage or EntryAttack4_Damage as the damage_effect_id of GA_Daffodill_Melee, execute that one native packet at the selected effective Normal coordinate and impact time. These rows name GA_Daffodill_Melee; the public attack tips bind Branch1–5 and EntryAttack1/2/4, while Branch6/7 are present only in the frozen damage rows. Single-packet delivery to target zero, no natural branch/entry dispatch, no chain transition, no additional packets, no extra energy grant and no native timing/target rule are bounded API-input assumptions. If an explicit target break_profile is supplied, the separate shared combat.unbal_value_as_target_gauge_v1 hypothesis may consume the packet's native UnbalValue; this does not establish Daffodil-specific Break eligibility or effects. AirAttack, EntryAttack3, natural Phantom Step/Parry admission, native Daffodil Break rules and packet multiplicities remain unsupported.",
    },
    Hypothesis {
        id: "daffodil.normal_chain_v1",
        description: "Five admitted ordinary inputs advance stages Melee1 through Melee5 with multiplicities 1,2,6,3,7 (nineteen hits). The chain loops, retaining native effective talent projection including resonance_3.",
    },
    Hypothesis {
        id: "daffodil.ordinary_hit_energy_v1",
        description: "For Daffodil character:1054 only, an ordinary Normal or Skill native packet grants its exact nonnegative DT_SkillDamageData.ChargeAdd once after positive actual HP loss against target zero. Fatal damage qualifies; zero HP loss, unavailable targets and cancelled packets do not. The living source owner receives 1x; each other living configured recipient receives the explicit source-element to recipient-element coefficient from frozen DT_SkillTeamChargeRelationData (the six recipient-element rows for Daffodil CHAOS are 0.6). Each recipient applies their own Charge Efficiency and exact fractional carry, then existing capacity and overflow accounting. Delivery-time owner and source attribution survive switching. Burst, Branch, AirAttack, EntryAttack, PerfectEvadeAttack, QTE, non-native branches, additional targets and callbacks outside the modeled packet train remain unsupported. Eligibility, cadence, recipient and off-field rules are tested hypotheses because native caller evidence is unavailable; ChargeAdd and relation coefficients are frozen native rows.",
    },
    Hypothesis {
        id: "daffodil.skill_burst_packets_v1",
        description: "Daffodil Resonance delivers eight Skill hits on target zero (two GE_Player_Daffodill_Skill1_Damage, five GE_Player_Daffodill_Skill2_Damage, and one GE_Player_Daffodill_Skill5_Damage); Finale delivers fourteen UltraSkill hits (three GE_Player_Daffodill_UltraSkill1_Damage, three GE_Player_Daffodill_UltraSkill2_Damage, seven GE_Player_Daffodill_UltraSkill4_Damage, and one GE_Player_Daffodill_UltraSkill5_Damage). All children use the admitted effective native talent coordinate and sample live modifiers independently. Ordered coincident delivery at the selected primary impact timestamp is a timing hypothesis pending animation intervals; it does not model travel, AoE coverage, Phantom Step, Parry Attack or complete kit behavior.",
    },
    Hypothesis {
        id: "damage.fixed_crit_rate_as_probability_v1",
        description: "Interpret a nonzero native FixedCritRate scalar as the packet's replacement critical-hit probability. Preserve normal critical-damage calculation and only roll when the packet's damage kind is crit-eligible; this does not claim native stat-capture or periodic-tick timing semantics.",
    },
    Hypothesis {
        id: "damage_type.normal_as_physical_v1",
        description: "Treat native DAMAGE_TYPE_NORMAL packets as physical damage: use defense and resistance, without an elemental bonus. This mapping is a tested model hypothesis; the frozen enum label alone does not prove its mitigation semantics.",
    },
    Hypothesis {
        id: "edgar.a5_max_hp_v1",
        description: "Edgar Awakening 5 converts excess healing into temporary Max HP increase up to the configured cap.",
    },
    Hypothesis {
        id: "edgar.a6_wake_team_def_v1",
        description: "For the collapsed in-range team scenario, each successfully admitted A6 Edgar GA_Edgar_UltraSkill grants 15% DEF percent to every living configured team actor, including Edgar and all configured actors regardless of field position. The runner has no actor coordinates, so this does not prove Wake range or geometry. The grant starts at admission before same-time hits, adds to existing DEF percent on base DEF, resets to the latest admitted A6 Edgar source, does not stack across Edgar owners, ends at the sampled native Wake deadline using half-open expiry, and follows its latest source's life; a dead recipient receives no benefit and cannot be revived. Recipient/range, team inclusion, cast timing, expiry, source lifetime and shared latest-owner refresh are explicit tested hypotheses pending native lifecycle evidence.",
    },
    Hypothesis {
        id: "edgar.aoe_all_in_range_dummy_v1",
        description: "Edgar Wild Current (Skill) and Finnegan's Wake (UltraSkill) deliver their full packet trains to all living scenario-declared dummies under the all-in-range dummy contract: by scenario definition all declared dummies are in range. Targets are delivered in ascending target_index order at the primary impact time. Spatial geometry, recipient caps, distribution and falloff remain unresolved.",
    },
    Hypothesis {
        id: "edgar.key_of_truth_v1",
        description: "A successful admitted Edgar Skill or QTE grants Edgar one Key of Truth at every awakening, to a maximum of three; admission rather than packet delivery is the trigger. At A1+, a successful admitted QTE by another living team actor grants one key to each living Edgar wearer by actor index, including off-field Edgar and another Edgar actor. The QTE wearer's actor index is excluded from team observation so its own QTE grants exactly once. Failed admissions, dead wearers and dead QTE actors grant no key. Keys persist across Burst and field switches because the frozen sources do not specify consumption. A Burst samples current keys at admission to extend Finnegan's Wake by one second per key at every awakening; keys gained during an active Wake affect only a later Burst. These timing and lifecycle rules are explicit tested hypotheses.",
    },
    Hypothesis {
        id: "edgar.normal_chain_v1",
        description: "Edgar Basic Combat Practice advances five stages per admitted Normal input, with two Melee3 projectiles. Native coefficients come from frozen DT_SkillDamageData and multiplicity from neverness_edgar. The chain loops after stage five and resets on a different admitted ability, field exit or a gap greater than 2 seconds between input starts. The gap and reset rules are tested hypotheses pending animation/cancel evidence. The third-stage projectiles deliver in order at the primary impact time; travel, AoE, plunge and riposte remain unsupported.",
    },
    Hypothesis {
        id: "edgar.ordinary_hit_energy_v1",
        description: "For Edgar character:1021 only, an ordinary Normal or Skill native packet grants its exact nonnegative DT_SkillDamageData.ChargeAdd once after positive actual HP loss against target zero. Fatal damage qualifies; zero HP loss, unavailable targets and cancelled packets do not. The living source owner receives 1x; each other living configured recipient receives the explicit source-element to recipient-element coefficient from frozen DT_SkillTeamChargeRelationData (the six recipient-element rows for Edgar COSMOS are 0.6). Each recipient applies their own Charge Efficiency and exact fractional carry, then existing capacity and overflow accounting. Delivery-time owner and source attribution survive switching. Burst (including Duration, PerKeyAddDuration and Treat_MaxHPPer variants), AirAttack, PerfectEvadeAttack, QTE, non-native branches, additional targets and callbacks outside the modeled packet train remain unsupported. Eligibility, cadence, recipient and off-field rules are tested hypotheses because native caller evidence is unavailable; ChargeAdd and relation coefficients are frozen native rows.",
    },
    Hypothesis {
        id: "edgar.skill_burst_packets_v1",
        description: "Edgar Wild Current delivers seven Skill hits on target zero (seven GE_Player_Edgar_Skill_Damage); Finnegan's Wake delivers one UltraSkill hit (one GE_Player_Edgar_UltraSkill_Damage). All children use the admitted effective native talent coordinate and sample live modifiers independently. Ordered coincident delivery at the selected primary impact timestamp is a timing hypothesis pending animation intervals; it does not model travel, AoE coverage, channeled healing or complete kit behavior.",
    },
    Hypothesis {
        id: "edgar.ultimate_periodic_healing_v1",
        description: "An admitted Edgar GA_Edgar_UltraSkill starts one healing tick per second beginning one second after cast through the native Finnegan's Wake duration endpoint inclusive. Each tick selects the currently active living actor, including Edgar, uses Edgar's live MaxHP at callback time and the cast's effective native Burst talent coordinate for the frozen flat-plus-MaxHP heal curves, then caps at missing HP. Ticks cannot revive a recipient; dead Edgar cancels the stream, off-field living Edgar continues, and a later admitted Burst replaces prior ticks. Full-HP overflow, source MaxHP sampling, recipient switching, inclusive endpoint and recast invalidation are tested hypotheses. Reached live healing bonuses use the separate explicit combat.live_healing_modifiers_v1 formula hypothesis. A5+ admission requires base HP, HP percent excluding Edgar's A5, HP flat, static HP, configured MaxHP and runtime MaxHP to agree after one nearest-integer settlement; caller-provided initial MaxHP is already final and is never boosted a second time. A6's allied DEF grant is outside this healing result.",
    },
    Hypothesis {
        id: "energy.efficiency_carry_v1",
        description: "Recipient charge efficiency scales implemented numeric grants with exact rational carry; explicit pool-fill restoration bypasses efficiency. Source eligibility, team distribution and native fractional precision remain disclosed owner/shared hypotheses.",
    },
    Hypothesis {
        id: "fadia.aoe_all_in_range_dummy_v1",
        description: "Fadia's initial GA_Fadia_UltraSkill delivers its full packet train (one GE_Player_Fadia_UltraSkill1_Damage, four GE_Player_Fadia_UltraSkill2_Damage, and one GE_Player_Fadia_UltraSkill3_Damage) to all living scenario-declared dummies under the all-in-range dummy contract: by scenario definition all declared dummies are in range. Targets are delivered in ascending target_index order at the primary impact time. Spatial geometry, recipient caps, distribution and falloff remain unresolved.",
    },
    Hypothesis {
        id: "fadia.full_hp_initial_burst_v1",
        description: "At awakening 0 and exactly full runtime HP, Fadia's initial GA_Fadia_UltraSkill delivers the native UltraSkill1 primary followed by four UltraSkill2 packets and one UltraSkill3 packet at the primary impact timestamp against target 0. Admission starts a 5-second Lilith form window; field exit or another admitted skill ends it. This bounded initial burst and window are modeled hypotheses; damaged-health thresholds, healing, form attacks, and the rest of Fadia's kit remain unsupported.",
    },
    Hypothesis {
        id: "fadia.normal_chain_v1",
        description: "Fadia's ordinary Normal advances five stages per admitted input: Melee1; Melee2+Melee3; Melee4; Melee5+Melee6; Melee7+Melee8. The eight native packets use the frozen ATK coefficients and Psyche metadata. The chain loops after stage five and resets on a different admitted ability, field exit or a gap greater than 2 seconds between input starts. Reset rules, gap and ordered delivery at the research primary impact time are tested hypotheses pending animation/cancel evidence. Plunge, riposte, threshold, damage redirection and complete kit behavior remain unsupported.",
    },
    Hypothesis {
        id: "fadia.ordinary_hit_energy_v1",
        description: "For Fadia character:1039 only, a modeled ordinary Normal native packet grants its exact nonnegative DT_SkillDamageData.ChargeAdd once after positive actual HP loss against target zero. Fatal damage qualifies; zero HP loss, unavailable targets and cancelled packets do not. The living source owner receives 1x; each other living configured recipient receives the explicit source-element to recipient-element coefficient from frozen DT_SkillTeamChargeRelationData (the six recipient-element rows for Fadia PSYCHE are 0.6). Each recipient applies their own Charge Efficiency and exact fractional carry, then existing capacity and overflow accounting. Delivery-time owner and source attribution survive switching. Skill, Burst, alternate forms, summons, DoT, Branch, AirAttack, QTE, non-native branches, additional targets and callbacks outside the modeled packet train remain unsupported. Eligibility, cadence, recipient and off-field rules are tested hypotheses because native caller evidence is unavailable; ChargeAdd and relation coefficients are frozen native rows.",
    },
    Hypothesis {
        id: "fireflies.all_in_range_v1",
        description: "Under the all-in-range dummy contract the source's \"nearby enemy\" condition is a declared scenario rule: every eligible scenario-declared dummy is nearby by definition, so no position, radius, shape or placement check gates the trigger on a connected Anima hit. Defeated dummies never produce connected hits and keep their existing skip rules; the source must be living. This is not a claim about native proximity radius or recipient selection; native trigger consumer behavior remains unresolved.",
    },
    Hypothesis {
        id: "fireflies.proc_v1",
        description: "Fireflies set passive activates when four matching modules are equipped, granting stacking CRIT DMG upon dealing damage.",
    },
    Hypothesis {
        id: "fireflies.solo_proc_v2",
        description: "Four matching equipped module shapes activate Fireflies; selected scalar is fractional CRIT damage, stack count is integral, and CD is per-stack seconds. Solo Anima deliveries to declared all-in-range dummies count as nearby team damage. Stacks expire independently before equal-time hits; capped hits do not refresh. Native trigger and refresh semantics remain unresolved.",
    },
    Hypothesis {
        id: "fireflies.team_anima_hit_v1",
        description: "A connected, nonzero elemental Anima packet from any living team member grants one independent Fireflies CRIT DMG stack to each living team member wearing Fireflies, including off-field wearers. A wearer does not grant an extra stack merely for being the source; each wearer receives exactly one stack per packet.",
    },
    Hypothesis {
        id: "gear.cast_buff_before_hits_v1",
        description: "Successful Ultimate admission activates equipped cast-triggered ATK buffs after costs and before same-time delivered hits. Weapon and set effects retain their own recipient, stacking and expiry policies. This cast-to-hit order is a tested timing hypothesis; native animation timing remains unresolved.",
    },
    Hypothesis {
        id: "gear.qte_cast_buff_before_hits_v1",
        description: "Successful Support Skill admission activates equipped cast-triggered buffs after costs and before same-time delivered hits. Source-owned recipient, stacking and expiry rules still apply. This cast-to-hit order is a tested timing hypothesis.",
    },
    Hypothesis {
        id: "gear.skill_cast_buff_before_hits_v1",
        description: "Successful Skill admission activates equipped cast-triggered ATK buffs after costs and before same-time delivered hits. Source-owned recipient, stacking and expiry rules still apply. This cast-to-hit order is a tested timing hypothesis.",
    },
    Hypothesis {
        id: "haniel.aoe_all_in_range_dummy_v1",
        description: "Under the all-in-range dummy contract, declared scenario dummies receive Haniel's Burst and Skill packet deliveries (Skill: 2x GE_Player_Haniel_Skill1_Damage; Burst: 1x GE_Player_Haniel_Ultraskill1_Damage) across all living dummies in ascending target_index order at the primary impact time. Spatial geometry, recipient caps, distribution and falloff remain unresolved.",
    },
    Hypothesis {
        id: "haniel.normal_chain_v1",
        description: "Haniel Normal advances five stages per admitted input with multiplicities 1,2,1,4,1 from frozen neverness_haniel and native DT_SkillDamageData coefficients. The chain loops after stage five and resets on a different admitted ability, field exit or a gap greater than 2 seconds between input starts. Reset rules and the gap are tested hypotheses pending animation/cancel evidence. Each stage's packets deliver in order at the research primary impact time; travel, AoE, plunge, riposte, Four-Part Harmony and complete kit behavior remain unsupported.",
    },
    Hypothesis {
        id: "haniel.ordinary_hit_energy_v1",
        description: "For Haniel character:1020 only, an ordinary Normal or Skill native packet grants its exact nonnegative DT_SkillDamageData.ChargeAdd once after positive actual HP loss against target zero. Fatal damage qualifies; zero HP loss, unavailable targets and cancelled packets do not. The living source owner receives 1x; each other living configured recipient receives the explicit source-element to recipient-element coefficient from frozen DT_SkillTeamChargeRelationData (the six recipient-element rows for Haniel PSYCHE are 0.6). Each recipient applies their own Charge Efficiency and exact fractional carry, then existing capacity and overflow accounting. Delivery-time owner and source attribution survive switching. Burst, enhanced/alternate forms, summons, Branch, AirAttack, QTE, non-native branches, additional targets and callbacks outside the modeled packet train remain unsupported. Eligibility, cadence, recipient and off-field rules are tested hypotheses because native caller evidence is unavailable; ChargeAdd and relation coefficients are frozen native rows.",
    },
    Hypothesis {
        id: "haniel.skill_burst_packets_v1",
        description: "Haniel Silent Moonlit Forest Guardian delivers two Skill hits on target zero (two GE_Player_Haniel_Skill1_Damage); A Melody Named Haniel delivers one UltraSkill hit (one GE_Player_Haniel_Ultraskill1_Damage). All children use the admitted effective native talent coordinate and sample live modifiers independently. Ordered coincident delivery at the selected primary impact timestamp is a timing hypothesis pending animation intervals; it does not model travel, AoE coverage, Hootie speaker positioning, Four-Part Harmony stacks or complete kit behavior.",
    },
    Hypothesis {
        id: "haniel.team_atk_hootie_clock_v1",
        description: "Skill and A Melody Named Haniel each grant every teammate flat ATK equal to Haniel base ATK times that action's effective native talent ratio, with separate nonstacking refresh keys whose grants add while both are active. Skill grants last 8 seconds, or 12 seconds at A2, and refresh Hootie's remaining duration to that full duration; Burst grants last 10 seconds at every awakening. While Paranormal Ace is active, Hootie's remaining duration pauses; Burst preserves the unexpired remainder and resumes it after the refreshed 10-second Ace ends, while Skill during Ace refreshes the paused remainder to its full 8- or 12-second duration. Exact-expiry casts see the prior effect expired, and Burst does not recreate an expired Hootie grant. Grants apply before same-time hits and delivery uses live buffs. TeamAll includes Haniel. These recipient, overlap, refresh and clock rules are explicit hypotheses; source death lifecycle, Harmony, Ensemble, copies and summon effects remain unsupported.",
    },
    Hypothesis {
        id: "hathor.aoe_all_in_range_dummy_v1",
        description: "Under the all-in-range dummy contract, declared scenario dummies receive Hathor's Burst and Skill packet deliveries (Skill: 2x GE_Player_Hathor_Skill1_Damage plus 1x GE_Player_Hathor_Skill11_Damage; Burst: 2x GE_Player_Hathor_UltraSkill1_Damage plus 1x GE_Player_Hathor_UltraSkill2_Damage) across all living dummies in ascending target_index order at the primary impact time. Spatial geometry, recipient caps, distribution and falloff remain unresolved.",
    },
    Hypothesis {
        id: "hathor.branch_airattack_direct_packet_energy_v1",
        description: "For Hathor character:1025 only, an explicitly modeled GA_Hathor_Melee direct Branch or AirAttack packet grants its frozen DT_SkillDamageData.ChargeAdd once after positive actual HP loss against target zero (Branch 12.0; AirAttack 6.3). Fatal damage qualifies; zero HP loss, unavailable targets and cancelled packets do not. The living source owner receives 1x; each other living configured recipient receives the frozen source-element to recipient-element coefficient from DT_SkillTeamChargeRelationData (3/5 for Hathor LAKSHANA), then applies their own Charge Efficiency, exact fractional carry, capacity and overflow accounting. Delivery-time recipients and source attribution use the shared tested policy. The policy covers both caller-authored direct packets and natural held-Normal Branch/BranchEnd and fallback AirAttack landing dispatch. Native eligibility, jump/dodge legality, stamina, costs, recovery, cadence, extra packets, other targets and callbacks remain unresolved. ChargeAdd and relation values are frozen rows; grant eligibility and distribution are not native caller evidence.",
    },
    Hypothesis {
        id: "hathor.branch_airattack_direct_packet_inputs_v1",
        description: "An explicit InvokeAbility command for GA_Hathor_Melee may select GE_Player_Hathor_Branch_Damage or GE_Player_Hathor_AirAttack_Damage as one direct packet at the caller-selected time, using the configured Normal talent coordinate including resonance_3. The packet leaves the ordinary Normal chain unchanged. On positive HP loss it may use the separate hathor.branch_airattack_direct_packet_energy_v1 scenario hypothesis; it does not use the ordinary-hit energy rule. With an explicit target break_profile, the separate shared combat.unbal_value_as_target_gauge_v1 hypothesis may subtract the packet's native UnbalValue; this does not establish Hathor-specific Break rules. This bounded API input does not establish natural dispatch, jump/dodge legality, stamina use, cost, recovery, travel, AoE coverage, additional packets, timing, targets or native energy eligibility.",
    },
    Hypothesis {
        id: "hathor.express_delivery_state_v1",
        description: "For Hathor, each modeled unheld GA_Hathor_Skill cast outside Emergency Delivery grants the frozen-description 2 Express Delivery Power stacks before its modeled hit train. GA_Hathor_UltraSkill starts Emergency Delivery for the frozen 5-second duration, leaves existing stacks unchanged and exposes the distinct GA_Hathor_Skill2 Cyclone Strike input; the base Aerial Command input is rejected while that replacement is active. Field exit ends Emergency Delivery immediately and retains the stack count. This bounded input model does not implement the A1 first-hold bonus, held Skill/Five-Star Tracking, A2 natural-expiry gains, Critical Riposte/Passive1 gains, Cyclone Strike dispatch or Power spending, or a general Power cap. Cast-time state application and the existing +15% ATK interpretation during Emergency Delivery remain tested model policies.",
    },
    Hypothesis {
        id: "hathor.normal_chain_v1",
        description: "Hathor ordinary Normal advances five stages with multiplicities 1,5,2,3,2 from frozen neverness_hathor descriptions and native DT_SkillDamageData coefficients. The chain loops after stage five and resets on a different admitted ability, field exit or a gap greater than 2 seconds between input starts. Reset rules, gap and ordered coincident delivery at the research primary impact time are tested hypotheses pending animation/cancel evidence. Stamina, positional movement, plunge, riposte and complete kit behavior remain unsupported.",
    },
    Hypothesis {
        id: "hathor.ordinary_hit_energy_v1",
        description: "For Hathor character:1025 only, an ordinary Normal or Skill native packet grants its exact nonnegative DT_SkillDamageData.ChargeAdd once after positive actual HP loss against target zero. Fatal damage qualifies; zero HP loss, unavailable targets and cancelled packets do not. The living source owner receives 1x; each other living configured recipient receives the explicit source-element to recipient-element coefficient from frozen DT_SkillTeamChargeRelationData (the six recipient-element rows for Hathor LAKSHANA are 0.6). Each recipient applies their own Charge Efficiency and exact fractional carry, then existing capacity and overflow accounting. Delivery-time owner and source attribution survive switching. Direct Branch/AirAttack inputs are excluded from this ordinary policy and use hathor.branch_airattack_direct_packet_energy_v1; natural dispatch, Burst, enhanced/alternate forms, summons, QTE, non-native branches, additional targets and callbacks outside the modeled packet train remain unsupported. Eligibility, cadence, recipient and off-field rules are tested hypotheses because native caller evidence is unavailable; ChargeAdd and relation coefficients are frozen native rows.",
    },
    Hypothesis {
        id: "hathor.skill_burst_packets_v1",
        description: "Hathor Aerial Command delivers three Skill hits on target zero (two GE_Player_Hathor_Skill1_Damage plus one GE_Player_Hathor_Skill11_Damage); Rider Express delivers three UltraSkill hits (two GE_Player_Hathor_UltraSkill1_Damage plus one GE_Player_Hathor_UltraSkill2_Damage). All children use the admitted effective native talent coordinate and sample live modifiers independently. Ordered coincident delivery at the selected primary impact timestamp is a timing hypothesis pending animation intervals; it does not model travel, AoE coverage, Cyclone Strike variants or complete kit behavior.",
    },
    Hypothesis {
        id: "hotori.aoe_all_in_range_dummy_v1",
        description: "Under the all-in-range dummy contract, declared scenario dummies receive Hotori's Burst, Skill, Katana and finisher packet deliveries (Skill: 4x GE_Player_Jin_Skill1_Damage, 2x GE_Player_Jin_Skill3_Damage, 1x GE_Player_Jin_Skill2_Damage; Burst: 1x GE_Player_Jin_UltraSkill1_Damage, 5x GE_Player_Jin_UltraSkill1_1..5_Damage, 1x GE_Player_Jin_UltraSkill2_Damage, 1x GE_Player_Jin_UltraSkill4_Damage, 2x GE_Player_Jin_UltraSkill3_Damage; Katana: the selected stage's UltraSkillMelee rows and the one selected UltraSkillMelee6 or UltraSkillMelee6Pas2 finisher row) across all living dummies in ascending target_index order at each packet's own impact time. Spatial geometry, recipient caps, distribution and falloff remain unresolved.",
    },
    Hypothesis {
        id: "hotori.normal_chain_v1",
        description: "Hotori ordinary Normal advances five stages with multiplicities 1,4,5,3,2 from frozen neverness_jin descriptions and native DT_SkillDamageData coefficients. The chain loops after stage five and resets on a different admitted ability, field exit or a gap greater than 2 seconds between input starts. Reset rules, gap and ordered delivery at the research impact coordinates are tested hypotheses pending animation/cancel evidence. Katana combo mode replaces this chain only while World's Tide Time Stop is active and is covered separately by hotori.katana_chain_v1.",
    },
    Hypothesis {
        id: "hotori.ordinary_hit_energy_v1",
        description: "For Hotori character:1052 only, any reachable ordinary Normal, Skill, Burst or Katana native packet grants its exact nonnegative DT_SkillDamageData.ChargeAdd (an explicit zero for the Q rows) once after positive actual HP loss against target zero. Fatal damage qualifies; zero HP loss, unavailable targets and cancelled packets do not. The living source owner receives 1x; each other living configured recipient receives the explicit source-element to recipient-element coefficient from frozen DT_SkillTeamChargeRelationData (the six recipient-element rows for Hotori COSMOS are 0.6). Each recipient applies their own Charge Efficiency and exact fractional carry, then existing capacity and overflow accounting. Delivery-time owner and source attribution survive switching. Branch, AirAttack, PerfectEvadeAttack, QTE, non-native branches, additional targets and callbacks outside the modeled packet trains remain unsupported. Eligibility, cadence, recipient and off-field rules are tested hypotheses because native caller evidence is unavailable; ChargeAdd and relation coefficients are frozen native rows.",
    },
    Hypothesis {
        id: "hotori.skill_burst_packets_v1",
        description: "Hotori Present Replay delivers seven Skill hits on target zero at the native real coordinates (four GE_Player_Jin_Skill1_Damage and two GE_Player_Jin_Skill3_Damage following the Skill.3/Skill.1 schedule, plus one final GE_Player_Jin_Skill2_Damage) with total Any recovery 1.4086605 after the final hit's 0.1 source pause. World's Tide delivers ten UltraSkill hits (one GE_Player_Jin_UltraSkill1_Damage, five GE_Player_Jin_UltraSkill1_1..5_Damage, one GE_Player_Jin_UltraSkill2_Damage, one GE_Player_Jin_UltraSkill4_Damage, and two GE_Player_Jin_UltraSkill3_Damage) at their native post-entry coordinates with no source pause. All children use the admitted effective native talent coordinate and sample live modifiers independently, and every packet hits all eligible dummies under hotori.aoe_all_in_range_dummy_v1. Delivery at those source coordinates is the selected timing hypothesis and does not model travel. Katana stage and finisher packets are covered by hotori.katana_chain_v1.",
    },
    Hypothesis {
        id: "inputs.critical_dodge_assumed_success_v1",
        description: "A dash marked critical=true is an explicit scenario assumption that Critical Dodge succeeded. It has no enemy hit or damage, uses ordinary dash admission and recovery, and only dispatches owner transitions whose bounded models accept this success. It does not establish native dodge windows or eligibility.",
    },
    Hypothesis {
        id: "inputs.special_dodge_timing_v1",
        description: "Research input timing: special recovery 800ms and dash recovery 500ms, with zero energy cost and no cooldown. Inputs respect recovery admission; animation cancellation, stamina, invulnerability and dash damage are not modeled. These timings are tested scheduling hypotheses, not measured native durations.",
    },
    Hypothesis {
        id: "iroi.aoe_all_in_range_dummy_v1",
        description: "Under the all-in-range dummy contract, declared scenario dummies receive Iroi's direct Burst packet (1x GE_Player_Oneiroi_UltraSkill_Damage) across all living dummies in ascending target_index order at the primary impact time. Spatial geometry, recipient caps, distribution and falloff remain unresolved.",
    },
    Hypothesis {
        id: "iroi.burst_post_cast_team_heal_v1",
        description: "A successfully admitted GA_Oneiroi_UltraSkill with its canonical direct primary settles one Oneiroi_UltraSkill_Treat heal for each configured actor in stable slot order, including Iroi. The admitted effective Burst coordinate (array_index + 1, after talent/resonance projection) selects the frozen post-cast healing ratio, modeled as a ratio of Iroi's effective MaxHP sampled at callback time. The team recipient scope, MaxHP basis, callback at the selected hit timestamp after direct HitDelivery, slot ordering, supported live outgoing/recipient incoming modifiers, nearest-integer settlement, missing-HP cap, no-revive behavior and positive gross Wrong Gate callback reuse the bounded Skill-heal policy. The frozen source identifies a post-cast ratio but does not establish these execution details or recipient scope. Continuous TreatPeriod healing, dream lifecycle, Imagination, damage-over-time and Regression/revival remain unsupported.",
    },
    Hypothesis {
        id: "iroi.burst_primary_packet_v1",
        description: "An ordinary Iroi UltraSkill cast delivers the canonical GE_Player_Oneiroi_UltraSkill_Damage primary packet once at the selected impact timestamp, using the admitted effective Burst talent coordinate and native ATK array. This hypothesis covers only the direct packet; the sourced one-time post-cast heal has its own bounded model assumption. Conditional continuous healing, continuous damage, Imagination threshold/cost, dream duration and Regression revival remain unsupported; no periodic ticks or extra targets are inferred.",
    },
    Hypothesis {
        id: "iroi.effect3_healing_outgoing_v1",
        description: "Iroi Effect3 (Trait: Cooperative, 1075AwakenEffect.json) increases Iroi's healing effects by 36%. This is modeled as a scenario-start self-scoped WhileAlive buff Buff_Oneiroi075_Level3 granting +9/25 healing_bonus, which feeds the outgoing zone of the shared H_raw = base x (1 + outgoing) x (1 + incoming) formula; settle_healing itself is unchanged and the amplified gross reaches the Wrong Gate callback.",
    },
    Hypothesis {
        id: "iroi.normal_chain_v1",
        description: "Iroi ordinary Collaboration advances five stages with packet multiplicities 1,3,8,1,3, using native Melee1 through Melee5 rows; frozen neverness_oneiroi descriptions identify the same ratios and hit counts. The chain loops after stage five and resets on a different admitted ability, field exit or a gap greater than 2 seconds between input starts. Reset rules, gap and ordered coincident delivery at the selected primary impact time are tested hypotheses pending animation/cancel evidence. Branch, plunge, Regression and summon behavior remain unsupported.",
    },
    Hypothesis {
        id: "iroi.ordinary_hit_energy_v1",
        description: "For Iroi character:1075 only, an ordinary Normal or Skill native packet grants its exact nonnegative DT_SkillDamageData.ChargeAdd once after positive actual HP loss against target zero. Fatal damage qualifies; zero HP loss, unavailable targets and cancelled packets do not. The living source owner receives 1x; each other living configured recipient receives the explicit source-element to recipient-element coefficient from frozen DT_SkillTeamChargeRelationData (the six recipient-element rows for Iroi NATURE are 0.6). Each recipient applies their own Charge Efficiency and exact fractional carry, then existing capacity and overflow accounting. Delivery-time owner and source attribution survive switching. Burst, Branch, AirAttack, QTE, non-native branches, additional targets and callbacks outside the modeled packet train remain unsupported. Eligibility, cadence, recipient and off-field rules are tested hypotheses because native caller evidence is unavailable; ChargeAdd and relation coefficients are frozen native rows.",
    },
    Hypothesis {
        id: "iroi.r3_team_def_aura_v1",
        description: "Iroi resonance_3 (Primate Among All Things, 1075AwakenEffect.json) increases all team members' DEF by 30% while Iroi is on the team. This is modeled as a scenario-start TeamAll buff Buff_Oneiroi075_Level2_Def granting +3/10 def_percent with WhileAlive source lifetime; the live percent applies to base DEF through the shared DEF zone, and living-source filtering drops the grant when Iroi is defeated. The skill-level +1 clause is modeled separately in talent coordinates and is untouched.",
    },
    Hypothesis {
        id: "iroi.r6_team_crit_aura_v1",
        description: "Iroi resonance_6 (Self-Proclaimed Crown of Creation, 1075AwakenEffect.json) increases all team members' Crit DMG by 25% while Iroi is on the team. This is modeled as a scenario-start TeamAll buff Buff_Oneiroi075_Level6_extra granting +1/4 to the additive crit_damage_bonus zone with WhileAlive source lifetime; living-source filtering drops the grant when Iroi is defeated, which is the executable form of 'while Iroi is on the team'. The per-packet crit_bonus registry hook returns zero because the aura also covers Iroi herself and the additive zone would otherwise double-count it. The Regression-clause DMG dealt and Crit Rate bonus remains unmodeled because Regression is not modeled.",
    },
    Hypothesis {
        id: "iroi.skill_nine_packet_train_v1",
        description: "An ordinary Iroi Skill cast admits the native Gun primary packet, then delivers seven Syringe child packets and one Trumpet child packet at the primary impact timestamp. Each packet selects its own frozen ATK array at the admitted effective talent coordinate, including resonance. Target-zero, ordered coincident delivery and delivery-time buff semantics are tested hypotheses; A2 adds one separate assistance packet. Further summon attacks, Regression and lifecycle remain pending.",
    },
    Hypothesis {
        id: "iroi.skill_team_heal_v1",
        description: "A successfully admitted ordinary GA_Oneiroi_Skill with its canonical Gun primary schedules one heal for each configured actor in stable slot order, including Iroi. The heal uses Oneiroi_Skill_Treat at the admitted effective Skill talent coordinate (array_index + 1, including resonance) as a ratio of Iroi's effective Max HP sampled at callback time. Settlement uses current supported outgoing and recipient incoming healing modifiers, rounds once to nearest integer, caps to missing HP through shared VitalState, and cannot revive. A positive rounded pre-cap heal invokes the existing Wrong Gate callback after settlement, including full-HP overheal; Wrong Gate retains its own wearer, source/recipient life, recipient, and buff rules. The heal callback is scheduled at the admitted cast timestamp at ProcCallback priority; cast-time ordering and MaxHP ratio basis are bounded tested hypotheses because the frozen tip specifies a one-time team heal and ratio but not timing or base-stat semantics. Conditional/continuous UltraSkill healing, Imagination and summon lifecycles remain unsupported.",
    },
    Hypothesis {
        id: "jiuyuan.aoe_all_in_range_dummy_v1",
        description: "Jiuyuan Intel Hunter and Final Reckoning deliver their full packet trains to all living scenario-declared dummies under the all-in-range dummy contract: by scenario definition all declared dummies are in range. Targets are delivered in ascending target_index order at the primary impact time. Spatial geometry, recipient caps, distribution and falloff remain unresolved.",
    },
    Hypothesis {
        id: "jiuyuan.normal_chain_v1",
        description: "Jiuyuan ordinary Normal advances five stages with multiplicities 1,2,3,4,1 from frozen neverness_kuhara descriptions and native DT_SkillDamageData coefficients. The chain loops after stage five and resets on a different admitted ability, field exit or a gap greater than 2 seconds between input starts. Reset rules, gap and ordered coincident delivery at the research primary impact time are tested hypotheses pending animation/cancel evidence. Aim and plunge variants, bullet resource model, and Pact settlement triggers remain unsupported.",
    },
    Hypothesis {
        id: "jiuyuan.ordinary_hit_energy_v1",
        description: "For Jiuyuan character:1055 only, a modeled Normal or Skill native packet grants its exact nonnegative DT_SkillDamageData.ChargeAdd once after positive actual HP loss against target zero. Fatal damage qualifies; zero HP loss, unavailable targets and cancelled packets do not. The living source owner receives 1x; each other living configured recipient receives the explicit source-element to recipient-element coefficient from frozen DT_SkillTeamChargeRelationData (the six recipient-element rows for Jiuyuan NATURE are 0.6). Each recipient applies their own Charge Efficiency and exact fractional carry, then existing capacity and overflow accounting. Delivery-time owner and source attribution survive switching. This also covers the modeled Jiuyuan aimed Normal packet GE_Player_Kuhara_AimAttack_Damage with frozen ChargeAdd 1. Burst, enhanced/alternate forms, summons, Branch, AirAttack, QTE, non-native branches, additional targets and callbacks outside the modeled packet train remain unsupported. Eligibility, cadence, recipient and off-field rules are tested hypotheses because native caller evidence is unavailable; ChargeAdd and relation coefficients are frozen native rows.",
    },
    Hypothesis {
        id: "jiuyuan.skill_burst_packets_v1",
        description: "Jiuyuan Intel Hunter delivers six Skill hits on target zero (one GE_Player_Kuhara_Skill1_Damage, four GE_Player_Kuhara_Skill2_Damage, and one GE_Player_Kuhara_Skill3_Damage); Final Reckoning delivers twelve UltraSkill hits (one GE_Player_Kuhara_UltraSkill1_Damage, three GE_Player_Kuhara_UltraSkill2_Damage, seven GE_Player_Kuhara_UltraSkill3_Damage, and one GE_Player_Kuhara_UltraSkill4_Damage). All children use the admitted effective native talent coordinate and sample live modifiers independently. Ordered coincident delivery at the selected primary impact timestamp is a timing hypothesis pending animation intervals; it does not model travel, AoE coverage, bullet consumption, Pact settlement triggers or complete kit behavior.",
    },
    Hypothesis {
        id: "lacrimosa.aoe_all_in_range_dummy_v1",
        description: "Under the all-in-range dummy contract, declared scenario dummies receive Lacrimosa's Burst packet delivery (4x GE_Player_Lacrimosa_UltraSkill1_Damage plus 1x GE_Player_Lacrimosa_UltraSkill2_Damage) across all living dummies in ascending target_index order at the primary impact time. Spatial geometry, recipient caps, distribution and falloff remain unresolved.",
    },
    Hypothesis {
        id: "lacrimosa.forms_v1",
        description: "Start in the existing ranged Tomato Percussion form. Flavor Variation admits the current-form transition: ranged to melee uses SwitchModB once, melee to ranged uses Pan twice per published ratios. Switch form at admission and reset the ordinary chain; dash resets the chain but preserves form. Skill, Ultimate and field exit preserve form. Melee stages use B_Melee1+B_Melee2; B_Melee3 x2; B_Melee4+B_Melee3_1 x3; B_Melee5+B_Melee6; B_Melee7 x3+B_Melee8. An uninterrupted grounded melee stage three shatters the stage-two projectile once, adding B_Melee3_Explode. Melee-form Ultimate uses UltraSkillModB x2, ModB2 x5 and ModB3 x3. All packets settle in authored order at the primary impact time. Initial form, admission transitions, chain resets, guaranteed nearby projectile shatter and coincident timing are tested hypotheses; spatial misses, travel, AoE, Nightmare ticks, stolen techniques, Critical Dodge windows and Sleep In resets remain unsupported.",
    },
    Hypothesis {
        id: "lacrimosa.nightmare_dot_scheduling_v1",
        description: "Lacrimosa GA_Lacrimosa_Skill and GA_Lacrimosa_UltraSkill casts apply the Nightmare blood DoT (GE_Player_Lacrimosa_Blood_Damage) to target zero: +5 stacks capped at 20, aggregate duration refreshed to 3s (6s when awakening effect 4 is selected). Ticks are pre-scheduled HitDelivery events on a 1s grid anchored at first application (NeverReset; first tick at +1s, no immediate tick) and rebuilt on every reapply; a reapply after full expiry re-anchors the grid. Stack expiry follows RemoveSingleStackAndRefreshDuration with expiry processed before a tick on coinciding timestamps. Each tick carries the native per-stack ATK ratio at the applying cast's talent coordinate and the State.Damage.Dot tag; the stack multiplier ratio*clamp(count-1,0,20) is added in the coefficient-addition zone from the live count at tick time (positive expansion bypass is modeled as never active). Melee, QTE, LV6 packet selection, E/Q value conflicts and per-application own-deadline expiry remain unbound; stack cap 20 conflicts with the serialized GE stack_limit=10. Tested bounded hypothesis; native executor, tick order and clock remain unproven.",
    },
    Hypothesis {
        id: "lacrimosa.ordinary_hit_energy_v1",
        description: "For Lacrimosa character:1004 only, a modeled ranged/melee Normal, Flavor Variation transition or ordinary Skill native packet grants its exact nonnegative DT_SkillDamageData.ChargeAdd once after positive actual HP loss against target zero. Fatal damage qualifies; zero HP loss, unavailable targets and cancelled packets do not. The living source owner receives 1x; each other living configured recipient receives the explicit source-element to recipient-element coefficient from frozen DT_SkillTeamChargeRelationData (the six recipient-element rows for Lacrimosa INCANTATION are 0.6). Each recipient applies their own Charge Efficiency and exact fractional carry, then existing capacity and overflow accounting. Delivery-time owner and source attribution survive switching. The modeled melee projectile shatter has its own frozen ChargeAdd and qualifies once per delivered explosion. Burst, blood DoT, Sleep In cooldown resets, Discord, summons, Branch, AirAttack, QTE, non-native branches, additional targets and callbacks outside the modeled packet train remain unsupported. Eligibility, cadence, recipient and off-field rules are tested hypotheses because native caller evidence is unavailable; ChargeAdd and relation coefficients are frozen native rows.",
    },
    Hypothesis {
        id: "lacrimosa.ranged_normal_chain_v1",
        description: "Lacrimosa Normal retains the existing ranged primary binding and advances five stages from frozen neverness_lacrimosa MeleeModA descriptions and native DT_SkillDamageData: Melee1; Melee2 x3; Melee3; Melee4 x3 plus Melee6; Melee5. The chain loops after stage five and resets on a different admitted ability, field exit or a gap greater than 2 seconds between input starts. Reset rules, gap and ordered coincident delivery at the research primary impact time are tested hypotheses pending animation/cancel evidence. This hypothesis covers the ranged chain only; travel, AoE, Nightmare DoT, Sleep In cooldown resets and complete kit behavior remain unsupported.",
    },
    Hypothesis {
        id: "lacrimosa.skill_burst_packets_v1",
        description: "Lacrimosa Morning Tomato delivers one Skill hit on target zero; Working Day Judgement delivers five UltraSkill hits (four GE_Player_Lacrimosa_UltraSkill1_Damage plus one GE_Player_Lacrimosa_UltraSkill2_Damage). All children use the admitted effective native talent coordinate and sample live modifiers independently. Ordered coincident delivery at the selected primary impact timestamp is a timing hypothesis pending animation intervals; it does not model travel, AoE coverage, Nightmare DoT ticks or complete kit behavior.",
    },
    Hypothesis {
        id: "linko.aoe_all_in_range_dummy_v1",
        description: "Under the all-in-range dummy contract, declared scenario dummies receive Linko's direct Burst packet (1x GE_Player_Radio072_UltralSkillLTE_AOE_Level4_Damage) across all living dummies in ascending target_index order at the primary impact time. Spatial geometry, recipient caps, distribution and falloff remain unresolved.",
    },
    Hypothesis {
        id: "linko.normal_chain_v1",
        description: "Five admitted ordinary inputs advance stages Melee1 through Melee5 with multiplicities 1,2,1,8,1 (thirteen hits). The chain loops, retaining native effective talent projection with Normal attack excluded from resonance_3 level bonus.",
    },
    Hypothesis {
        id: "linko.ordinary_hit_energy_v1",
        description: "For Linko character:1072 only, an ordinary Normal or Skill native packet grants its exact nonnegative DT_SkillDamageData.ChargeAdd once after positive actual HP loss against target zero. Fatal damage qualifies; zero HP loss, unavailable targets and cancelled packets do not. The living source owner receives 1x; each other living configured recipient receives the explicit source-element to recipient-element coefficient from frozen DT_SkillTeamChargeRelationData (the six recipient-element rows for Linko NATURE are 0.6). Each recipient applies their own Charge Efficiency and exact fractional carry, then existing capacity and overflow accounting. Delivery-time owner and source attribution survive switching. Burst, enhanced/alternate forms, summons, Branch, AirAttack, QTE, non-native branches, additional targets and callbacks outside the modeled packet train remain unsupported. Eligibility, cadence, recipient and off-field rules are tested hypotheses because native caller evidence is unavailable; ChargeAdd and relation coefficients are frozen native rows.",
    },
    Hypothesis {
        id: "linko.skill_four_hits_v1",
        description: "One admitted GA_Radio072_Skill delivers the native Skill1 primary followed by Skill2, Skill3, and Skill4 once each on target zero, using the admitted effective Skill talent coordinate and sampling live modifiers per packet. Ordered coincident delivery at the primary impact timestamp is a timing hypothesis; animation intervals, travel, AoE, and summon behavior remain unresolved.",
    },
    Hypothesis {
        id: "mint.a3_reaction_trigger_v1",
        description: "Mint A3 Attendance Bonus grants Mint +15% ATK (Mint_Adcance_3_1 = 0.15) for 15s upon triggering Blossom or Hexed against an enemy, matching the published in-game and community description rather than the asset QTE_BEGIN event. In supported team rotations triggering reactions, the ATK bonus is active in the additive stat zone.",
    },
    Hypothesis {
        id: "mint.aoe_all_in_range_dummy_v1",
        description: "Mint Super Claws and Thunderous Whirlwind Slash deliver their full packet trains to all living scenario-declared dummies under the all-in-range dummy contract: by scenario definition all declared dummies are in range. Targets are delivered in ascending target_index order at the primary impact time. Spatial geometry, recipient caps, distribution and falloff remain unresolved.",
    },
    Hypothesis {
        id: "mint.normal_chain_v1",
        description: "Mint Normal advances five admitted stages [Melee1,Melee2], [Melee3], [Melee4,Melee5], [Melee6], [Melee7,Melee8] matching frozen descriptions at all nine talent levels. It loops after stage five and resets on a different admitted ability, field exit or an input-start gap greater than 2 seconds. Reset rules and ordered coincident delivery at the research primary impact time are tested hypotheses pending animation/cancel evidence. Travel, AoE, jump, hold, plunge, riposte, automatic passives and complete kit behavior remain unsupported.",
    },
    Hypothesis {
        id: "mint.ordinary_hit_energy_v1",
        description: "For Mint character:1019 only, an ordinary Normal or Skill native packet grants its exact nonnegative DT_SkillDamageData.ChargeAdd once after positive actual HP loss against target zero. Fatal damage qualifies; zero HP loss, unavailable targets and cancelled packets do not. The living source owner receives 1x; each other living configured recipient receives the explicit source-element to recipient-element coefficient from frozen DT_SkillTeamChargeRelationData (the six recipient-element rows for Mint NATURE are 0.6). Each recipient applies their own Charge Efficiency and exact fractional carry, then existing capacity and overflow accounting. Delivery-time owner and source attribution survive switching. Burst, QTE, non-native branches, additional targets and callbacks outside the modeled packet train remain unsupported. Eligibility, cadence, recipient and off-field rules are tested hypotheses because native caller evidence is unavailable; ChargeAdd and relation coefficients are frozen native rows.",
    },
    Hypothesis {
        id: "mint.resonance_3_v1",
        description: "At awakening 3..6, resonance_3 adds one to Melee, Skill and UltraSkill; QTE receives no bonus.",
    },
    Hypothesis {
        id: "mint.skill_burst_packets_v1",
        description: "Mint Super Claws delivers Skill1 twice; hit 2 follows the selected .365246s animation interval after hit 1, so hit 1's Fireflies stack is live for hit 2, while a miss keeps the nominal coordinate without damage. Thunderous Whirlwind Slash delivers UtraSkill1, UtraSkill2..6 once each, then UtraSkill7, on target zero per admitted cast at the selected primary impact timestamp. All children use the admitted effective native talent coordinate and sample live modifiers independently. Travel, AoE coverage, interruptions or complete kit behavior remain unresolved.",
    },
    Hypothesis {
        id: "mint.skill_coordinates_v1",
        description: "Initial purchased level 1; native upgrade coordinate L buys L+1; damage array coordinate is effective level minus one.",
    },
    Hypothesis {
        id: "nanally.authority_a3_follow_up_v1",
        description: "Selected Nanally Effect3 reacts to positive settled HP loss from her modeled native Normal/Aim, Skill, Burst, QTE or Underboss packets while Authority is live at delivery. Against surviving target zero it queues one same-time GE_Nanally010_Lv3_Damage follow-up, using the effective Normal talent coordinate (including resonance), native A/NATURE metadata and the triggering command/action. The frozen row and published text ground Normal scaling and the 1s ICD; positive-HP-loss gating, target-zero surviving-recipient choice, ordered post-parent delivery and a persistent source-owned ICD across switches are bounded execution policies. Authority expiry is half-open, and field exit ends Authority before delayed hits; absent Effect3, zero HP loss, defeated targets and A3's own packet cannot create follow-ups. On positive connected A3 delivery (including its fatal hit), one selected A1 and one selected A5 callback run after same-time hits through the existing shared ICD/energy/stack owners. No cast-time A3 or extra action cost is created. The Cycle bridge additionally accepts actual own Cycle and connected Fair Duel damage through the same guard; it retains the actual Authority command and hit as autonomous parents and captures a living trigger recipient or lowest-index living fallback. Native callback selection and action timing remain revisable policies; full-kit acceptance is separate.",
    },
    Hypothesis {
        id: "nanally.burst_seven_hits_v1",
        description: "Nanally Burst's finite rate-one action_clock_v1 delivers UltraSkill1 once, UltraSkill2 five times and UltraSkill3 once at2300192/2382032/2468449/2547640/2634465/2716783/3015465us progress. Frozen effective Burst coefficients and native Q/NATURE metadata remain separate from live per-hit stats. Packets hit all living stationary dummies; no ordinary energy or free Cat attack is inferred. Public Burst pays100 energy and starts its15s cooldown at commitment; interruption preserves paid cost/cooldown, settled damage and already applied effects while cancelling unissued owner packets and unborn summon notification. Underboss applies separately at3415870us via burst_summon_state_v1. General ability/Jump recovery3582140us, switch3583025us and movement3756871us retain distinct gates. No positive pause duration is invented from rate-only Q source rows. Saved asset progress, conservative playback, uncapped recipients and cancellation policy are revisable models, not measured animation fidelity.",
    },
    Hypothesis {
        id: "nanally.burst_summon_state_v1",
        description: "Nanally's ordinary Burst applies one Underboss generation at the reached3415870us progress notification, after all seven direct Q markers, through action_clock_v1. The captured effective Burst coordinate and actual admitted source command remain attached. It lasts6 ObjectLifetime seconds or9 with selected Effect2, with half-open expiry and generation replacement only when a new summon actually applies. Cancelling before birth issues no new summon; cancelling after birth preserves its remaining lifetime. At equal time BuffTickOrExpiry application precedes explicit SettlementAndDeath interruption. Owner death clears the summon and cancels affected groups; field exit preserves remaining lifetime and already dispatched independent children. Creation/direct Q produce no Cat_AirAttack hit, coordination or A1/A5 callback. The budget is time only, without an eight-input cap or unused auxiliary-minion stocks. Later supported inputs select their own Cat groups. These are selected revisable execution policies, not frame-verified game behavior.",
    },
    Hypothesis {
        id: "nanally.normal_chain_v1",
        description: "Five admitted ordinary inputs advance stages Melee1 through Melee5 with multiplicities 1,1,3,7,2 (fourteen hits). The chain loops, retaining native effective talent projection including resonance_3.",
    },
    Hypothesis {
        id: "nanally.ordinary_hit_energy_v1",
        description: "For Nanally character:1010 only, each scoped ordinary Normal/heavy/Aim/plunge or Skill packet group grants its exact nonnegative DT_SkillDamageData.ChargeAdd once after its first positive actual HP loss, including a surviving non-primary dummy. Recipient multiplicity cannot duplicate charge. Fatal damage qualifies; zero HP loss, unavailable targets and cancelled packets do not. The living source owner receives1x; each other living configured recipient receives the frozen source-element to recipient-element coefficient (Nanally NATURE's six recipient-element rows are.6). Each recipient applies their own Charge Efficiency, exact carry and capacity/overflow. Delivery-time owner attribution survives switching for already emitted independent packets. Aim uses native.7 charge; landing uses native2.5 charge without the authored height multiplier. Burst, Cat, unsupported Branch rows, PerfectEvadeAttack, QTE and callbacks outside these bindings do not use this grant. Recipient/contact policies remain selected hypotheses; ChargeAdd and relation coefficients are frozen native rows.",
    },
    Hypothesis {
        id: "nanally.skill_five_hits_v1",
        description: "Nanally Colucci Howling Technique uses finite rate-one action_clock_v1: five E packets at429637/497363/563001/631341/696485us progress, rounded upward from saved source coordinates. Each packet hits all living stationary dummies and grants ordinary energy once per positive group. Skill cooldown starts at the425571us notify and lasts16 World seconds; pre-notify cancellation leaves none and later cancellation preserves it. Authority begins at429917us, after the first hit, adding30 percentage points CRIT DMG for12 World seconds or selected Effect6's15 in combat/20 outside, captured at actual application via combat_lifecycle_v1. Field exit, death, expiry and explicit removal end it. Separate next-ability/Jump1164219us and movement1512797us gates apply; interruption cancels unissued packets and unborn notifications, without refunding settled resources. Live Authority selects non-damaging RemoveSkill after2 World seconds from its application through authority_removal_v1; removal preserves ordinary cooldown and invokes no ordinary cast observers. A3 follows settled positive owner damage under live Authority with its own1s guard, so a fresh E's second packet can request it. Conservative playback, uncapped recipients, dispatch and positive-loss policies are revisable models, not frame verification. Heavy/plunge remain separate bindings.",
    },
    Hypothesis {
        id: "nanally.underboss_coordination_v2",
        description: "A living Nanally summon starts at the reached Burst3415870us notification and lasts6 ObjectLifetime seconds, or9 with selected Effect2, with half-open expiry and generation replacement. It coordinates one published packet group per later supported Normal or Skill input or reached heavy section; native Support, Critical Riposte and plunge bindings select their own groups. The summon uses a time budget only: no separate three-minion pool or inferred eight-group limit. Normal groups are Cat_Melee1; Cat_Melee2; Cat_Melee3+4; Cat_Melee5 x3; Cat_Melee6+7. Skill is Cat_Skill x5. Heavy groups are Cat_Branch1; Cat_Branch2+3+4; Cat_Branch5 x3, with selected uncapped recipients and one performed group each. Each uses the creating Burst talent coordinate with native A/NATURE metadata and zero ordinary energy/Cycle/Break. One admitted group is one performed follow-up attack for A1/A5, even when it contains several packets. After surviving group delivery one selected A1 callback uses its shared one-second ICD and exact efficiency/carry, and one selected A5 stack adds 2 percent base-ATK scaling to a ten-stack cap with 15-second refresh and a fresh stack pool after half-open expiry. A5 cannot regrant off field or outside combat and clears on field exit or an explicitly modeled combat exit. Coincident post-parent delivery against the selected recipient is a scheduling approximation; no synthetic 0.3-second FIFO is imposed. Waiting creates no autonomous attack. Owner death, target loss, expiry or replacement cancels pending groups; switching preserves eligible pending groups and remaining summon lifetime. Burst direct hits, summon creation and Aim create no Cat group. These selected rules are revisable models, not frame-verified game claims.",
    },
    Hypothesis {
        id: "owner.action_role.character_1003_qte_v1",
        description: "Classify this exact frozen character action as QTE because its DT_CharacterAbilityConfig identity names the owner action as QTE while DT_CharacterAbilityEffectConfig has no row; this role is an owner-reviewed model hypothesis for this ability only.",
    },
    Hypothesis {
        id: "owner.action_role.character_1004_melee_v1",
        description: "Classify this exact Lacrimosa action as Melee because the frozen character action identity names its primary attack and the effect-config row is absent; owner role is an explicit model hypothesis.",
    },
    Hypothesis {
        id: "owner.action_role.character_1004_qte_v1",
        description: "Classify this exact frozen character action as QTE because its DT_CharacterAbilityConfig identity names the owner action as QTE while DT_CharacterAbilityEffectConfig has no row; this role is an owner-reviewed model hypothesis for this ability only.",
    },
    Hypothesis {
        id: "owner.action_role.character_1004_skill_v1",
        description: "Classify this exact Lacrimosa action as Skill because the frozen character action identity names its normal skill and the effect-config row is absent; owner role is an explicit model hypothesis.",
    },
    Hypothesis {
        id: "owner.action_role.character_1004_ultimate_v1",
        description: "Classify this exact Lacrimosa action as UltraSkill because the frozen character action identity names its ultimate and the effect-config row is absent; owner role is an explicit model hypothesis.",
    },
    Hypothesis {
        id: "owner.action_role.character_1008_qte_v1",
        description: "Classify this exact frozen character action as QTE because its DT_CharacterAbilityConfig identity names the owner action as QTE while DT_CharacterAbilityEffectConfig has no row; this role is an owner-reviewed model hypothesis for this ability only.",
    },
    Hypothesis {
        id: "owner.action_role.character_1010_qte_v1",
        description: "Classify this exact frozen character action as QTE because its DT_CharacterAbilityConfig identity names the owner action as QTE while DT_CharacterAbilityEffectConfig has no row; this role is an owner-reviewed model hypothesis for this ability only.",
    },
    Hypothesis {
        id: "owner.action_role.character_1019_qte_v1",
        description: "Classify this exact frozen character action as QTE because its DT_CharacterAbilityConfig identity names the owner action as QTE while DT_CharacterAbilityEffectConfig has no row; this role is an owner-reviewed model hypothesis for this ability only.",
    },
    Hypothesis {
        id: "owner.action_role.character_1020_qte_v1",
        description: "Classify this exact frozen character action as QTE because its DT_CharacterAbilityConfig identity names the owner action as QTE while DT_CharacterAbilityEffectConfig has no row; this role is an owner-reviewed model hypothesis for this ability only.",
    },
    Hypothesis {
        id: "owner.action_role.character_1021_qte_v1",
        description: "Classify this exact frozen character action as QTE because its DT_CharacterAbilityConfig identity names the owner action as QTE while DT_CharacterAbilityEffectConfig has no row; this role is an owner-reviewed model hypothesis for this ability only.",
    },
    Hypothesis {
        id: "owner.action_role.character_1023_qte_v1",
        description: "Classify this exact frozen character action as QTE because its DT_CharacterAbilityConfig identity names the owner action as QTE while DT_CharacterAbilityEffectConfig has no row; this role is an owner-reviewed model hypothesis for this ability only.",
    },
    Hypothesis {
        id: "owner.action_role.character_1025_qte_v1",
        description: "Classify this exact frozen character action as QTE because its DT_CharacterAbilityConfig identity names the owner action as QTE while DT_CharacterAbilityEffectConfig has no row; this role is an owner-reviewed model hypothesis for this ability only.",
    },
    Hypothesis {
        id: "owner.action_role.character_1033_qte_v1",
        description: "Classify this exact frozen character action as QTE because its DT_CharacterAbilityConfig identity names the owner action as QTE while DT_CharacterAbilityEffectConfig has no row; this role is an owner-reviewed model hypothesis for this ability only.",
    },
    Hypothesis {
        id: "owner.action_role.character_1036_qte_v1",
        description: "Classify this exact frozen character action as QTE because its DT_CharacterAbilityConfig identity names the owner action as QTE while DT_CharacterAbilityEffectConfig has no row; this role is an owner-reviewed model hypothesis for this ability only.",
    },
    Hypothesis {
        id: "owner.action_role.character_1039_qte_v1",
        description: "Classify this exact frozen character action as QTE because its DT_CharacterAbilityConfig identity names the owner action as QTE while DT_CharacterAbilityEffectConfig has no row; this role is an owner-reviewed model hypothesis for this ability only.",
    },
    Hypothesis {
        id: "owner.action_role.character_1042_qte_v1",
        description: "Classify this exact frozen character action as QTE because its DT_CharacterAbilityConfig identity names the owner action as QTE while DT_CharacterAbilityEffectConfig has no row; this role is an owner-reviewed model hypothesis for this ability only.",
    },
    Hypothesis {
        id: "owner.action_role.character_1046_melee_v1",
        description: "Classify this exact Zero action as Melee because the frozen character action identity names its primary attack and the effect-config row is absent; owner role is an explicit model hypothesis.",
    },
    Hypothesis {
        id: "owner.action_role.character_1046_qte_v1",
        description: "Classify this exact frozen character action as QTE because its DT_CharacterAbilityConfig identity names the owner action as QTE while DT_CharacterAbilityEffectConfig has no row; this role is an owner-reviewed model hypothesis for this ability only.",
    },
    Hypothesis {
        id: "owner.action_role.character_1046_skill_v1",
        description: "Classify this exact Zero action as Skill because the frozen character action identity names its normal skill and the effect-config row is absent; owner role is an explicit model hypothesis.",
    },
    Hypothesis {
        id: "owner.action_role.character_1046_ultimate_v1",
        description: "Classify this exact Zero action as UltraSkill because the frozen character action identity names its ultimate and the effect-config row is absent; owner role is an explicit model hypothesis.",
    },
    Hypothesis {
        id: "owner.action_role.character_1051_qte_v1",
        description: "Classify this exact frozen character action as QTE because its DT_CharacterAbilityConfig identity names the owner action as QTE while DT_CharacterAbilityEffectConfig has no row; this role is an owner-reviewed model hypothesis for this ability only.",
    },
    Hypothesis {
        id: "owner.action_role.character_1052_qte_v1",
        description: "Classify this exact frozen character action as QTE because its DT_CharacterAbilityConfig identity names the owner action as QTE while DT_CharacterAbilityEffectConfig has no row; this role is an owner-reviewed model hypothesis for this ability only.",
    },
    Hypothesis {
        id: "owner.action_role.character_1054_qte_v1",
        description: "Classify this exact frozen character action as QTE because its DT_CharacterAbilityConfig identity names the owner action as QTE while DT_CharacterAbilityEffectConfig has no row; this role is an owner-reviewed model hypothesis for this ability only.",
    },
    Hypothesis {
        id: "owner.action_role.character_1055_qte_v1",
        description: "Classify this exact frozen character action as QTE because its DT_CharacterAbilityConfig identity names the owner action as QTE while DT_CharacterAbilityEffectConfig has no row; this role is an owner-reviewed model hypothesis for this ability only.",
    },
    Hypothesis {
        id: "owner.action_role.character_1057_qte_v1",
        description: "Classify this exact frozen character action as QTE because its DT_CharacterAbilityConfig identity names the owner action as QTE while DT_CharacterAbilityEffectConfig has no row; this role is an owner-reviewed model hypothesis for this ability only.",
    },
    Hypothesis {
        id: "owner.action_role.character_1070_qte_v1",
        description: "Classify this exact frozen character action as QTE because its DT_CharacterAbilityConfig identity names the owner action as QTE while DT_CharacterAbilityEffectConfig has no row; this role is an owner-reviewed model hypothesis for this ability only.",
    },
    Hypothesis {
        id: "owner.action_role.character_1071_qte_v1",
        description: "Classify this exact frozen character action as QTE because its DT_CharacterAbilityConfig identity names the owner action as QTE while DT_CharacterAbilityEffectConfig has no row; this role is an owner-reviewed model hypothesis for this ability only.",
    },
    Hypothesis {
        id: "owner.action_role.character_1071_skill_v1",
        description: "Classify this exact Chaos action as Skill because the frozen character action identity names its normal skill and the effect-config row is absent; owner role is an explicit model hypothesis.",
    },
    Hypothesis {
        id: "owner.action_role.character_1071_ultimate_v1",
        description: "Classify this exact Chaos action as UltraSkill because the frozen character action identity names its ultimate and the effect-config row is absent; owner role is an explicit model hypothesis.",
    },
    Hypothesis {
        id: "owner.action_role.character_1072_qte_v1",
        description: "Classify this exact frozen character action as QTE because its DT_CharacterAbilityConfig identity names the owner action as QTE while DT_CharacterAbilityEffectConfig has no row; this role is an owner-reviewed model hypothesis for this ability only.",
    },
    Hypothesis {
        id: "owner.action_role.character_1073_qte_v1",
        description: "Classify this exact frozen character action as QTE because its DT_CharacterAbilityConfig identity names the owner action as QTE while DT_CharacterAbilityEffectConfig has no row; this role is an owner-reviewed model hypothesis for this ability only.",
    },
    Hypothesis {
        id: "owner.action_role.character_1075_qte_v1",
        description: "Classify this exact frozen character action as QTE because its DT_CharacterAbilityConfig identity names the owner action as QTE while DT_CharacterAbilityEffectConfig has no row; this role is an owner-reviewed model hypothesis for this ability only.",
    },
    Hypothesis {
        id: "owner.action_role.character_1076_qte_v1",
        description: "Classify this exact frozen character action as QTE because its DT_CharacterAbilityConfig identity names the owner action as QTE while DT_CharacterAbilityEffectConfig has no row; this role is an owner-reviewed model hypothesis for this ability only.",
    },
    Hypothesis {
        id: "owner.packet_binding.character_1010_skill_lv3_v1",
        description: "Bind Nanally's exact GA_Nanally_Skill initiator to GE_Nanally010_Lv3_Damage, whose frozen damage row names GA_Nanally_Melee. The initiator and child row are retained independently; this owner-reviewed edge follows the A3 follow-up declaration.",
    },
    Hypothesis {
        id: "published.preview_inputs_v1",
        description: "Published team preview inputs use standardized combat attributes and default target resistances.",
    },
    Hypothesis {
        id: "research.preview.single_packet_v1",
        description: "A single explicitly selected native damage packet is used as a preview action; complete action hit sequencing and reachability are not established.",
    },
    Hypothesis {
        id: "sakiri.aoe_all_in_range_dummy_v1",
        description: "Sakiri Devour Whole and Feast of Gluttony deliver their full packet trains to all living scenario-declared dummies under the all-in-range dummy contract: by scenario definition all declared dummies are in range. Targets are delivered in ascending target_index order at the primary impact time. Spatial geometry, recipient caps, distribution and falloff remain unresolved.",
    },
    Hypothesis {
        id: "sakiri.direct_plunge_packet_energy_v1",
        description: "For Sakiri character:1003 only, an explicitly modeled GA_Sagiri_Melee AirAttack1 direct packet grants its frozen DT_SkillDamageData.ChargeAdd (2.5) once after positive actual HP loss against target zero. Fatal damage qualifies; zero HP loss, unavailable targets and cancelled packets do not. The living source owner receives 1x; each other living configured recipient receives the frozen source-element to recipient-element coefficient from DT_SkillTeamChargeRelationData (3/5 for Sakiri INCANTATION), then applies their own Charge Efficiency, exact fractional carry, capacity and overflow accounting. This exact direct-input policy is a tested hypothesis, not evidence of native eligibility. Natural jump/plunge dispatch, legality, costs, recovery, timing, extra packets, other targets and callbacks remain unresolved.",
    },
    Hypothesis {
        id: "sakiri.direct_plunge_packet_v1",
        description: "An explicit InvokeAbility command for GA_Sagiri_Melee may select the frozen GE_Player_Sagiri_AirAttack1_Damage row as one direct packet at the caller-selected time, using the configured Normal talent coordinate including resonance_3. This packet does not advance or reset the modeled Normal chain and receives no ordinary-hit energy grant. The command does not model jump/plunge legality, natural dispatch, cost, recovery, travel, recipients, hit timing or native energy eligibility.",
    },
    Hypothesis {
        id: "sakiri.normal_chain_v1",
        description: "Sakiri Normal advances five stages per admitted input: native Melee1; Melee2+3; Melee8+4; Melee5 three times; Melee6+7. Frozen neverness_sagiri descriptions and DT_SkillDamageData coefficients agree on all nine released talent levels. The chain loops after stage five and resets on a different admitted ability, field exit or a gap greater than 2 seconds between input starts. Reset rules and the gap are tested hypotheses pending animation/cancel evidence. Each stage's packets deliver in order at the research primary impact time; travel, AoE, plunge, riposte and complete kit behavior remain unsupported.",
    },
    Hypothesis {
        id: "sakiri.ordinary_hit_energy_v1",
        description: "For Sakiri character:1003 only, an ordinary Normal or Skill native packet grants its exact nonnegative DT_SkillDamageData.ChargeAdd once after positive actual HP loss against target zero. Fatal damage qualifies; zero HP loss, unavailable targets and cancelled packets do not. The living source owner receives 1x; each other living configured recipient receives the explicit source-element to recipient-element coefficient from frozen DT_SkillTeamChargeRelationData (the six recipient-element rows for Sakiri INCANTATION are 0.6). Each recipient applies their own Charge Efficiency and exact fractional carry, then existing capacity and overflow accounting. Delivery-time owner and source attribution survive switching. The explicit GE_Player_Sagiri_AirAttack1_Damage packet input is excluded from this energy adapter; native plunge energy eligibility remains unresolved. Burst, QTE, non-native branches, additional targets and callbacks outside the modeled packet train remain unsupported. Eligibility, cadence, recipient and off-field rules are tested hypotheses because native caller evidence is unavailable; ChargeAdd and relation coefficients are frozen native rows.",
    },
    Hypothesis {
        id: "sakiri.skill_burst_packets_v1",
        description: "Sakiri Devour Whole delivers Skill1 and Skill2 once each on target zero; Feast of Gluttony delivers UltraSkill1 five times and UltraSkill2 once. All children use the admitted effective native talent coordinate and sample live modifiers independently. Ordered coincident delivery at the selected primary impact timestamp is a timing hypothesis pending animation intervals; it does not model travel, AoE coverage, eating mode or complete kit behavior.",
    },
    Hypothesis {
        id: "sets.crimson_v1",
        description: "Crimson: Twin Butterflies binds Incantation_4_1/4_2/4_3 to the four-piece description (.06/6/10). Each connected delivered wearer hit grants one independent self ATK stack before same-time queued hits, including multi-packet inputs. The cap discards new applications without refreshing existing stacks. Each stack expires independently on world time, before equal-time hits; switching preserves stacks and allies cannot grant or receive them. The native trigger is AsyncTaskCharacterAttacked with an element check the frozen evidence does not fully resolve; hit-granted stacks are a tested hypothesis. Trigger timing and cap behavior are tested hypotheses; set count declares matching native geometry. Global release availability remains unverified in the frozen catalog.",
    },
    Hypothesis {
        id: "sets.hp_suit_v1",
        description: "Suit10 grants its frozen +4% Max HP per stack on positive settled HP loss and ten stacks on admitted Ultimate cast. Each stack lasts ten seconds and expires independently; the cap discards new applications without refreshing existing stacks. Per EQUIPMENT_CONTRACT.md set packs use the additive pack interpreter: static HPMaxUp (2pc +10%) and dynamic stack bonuses sum in one pool, effective = base * (1 + static + dynamic), rounded once to nearest integer; current HP is unchanged on grant and clamped to the effective max on expiry. Damage grants apply after HP settlement; Ultimate grants apply before same-time hits. Trigger timing, cap behavior and the HP settlement are tested hypotheses; set count declares matching native geometry. Global release availability remains unverified in the frozen catalog.",
    },
    Hypothesis {
        id: "sets.lost_radiance_v1",
        description: "Lost Radiance binds Cosmos2_4_DefIgnore and Cosmos2_4_CD to the current four-piece description (.25/20). It adds its ratio to the wearer's DEF ignore zone. Successful Ultimate admission refreshes one nonstacking effect before its delivered hits; switching preserves it until world-time expiry. Set count declares matching geometry; native parameter-caller and clock applicability remain hypotheses. Global release availability remains unverified in the frozen catalog.",
    },
    Hypothesis {
        id: "sets.quiet_manor_v1",
        description: "Quiet Manor binds Psychically_4/CD/stack to the four-piece description (.12/6/3). Each successfully admitted wearer Normal Attack input grants one independent self Mental damage stack before same-time delivered hits, including inputs with multiple hits or no living target. The cap discards new applications without refreshing existing stacks. Each stack expires independently on world time, before equal-time hits; switching preserves stacks and allies cannot grant or receive them. Trigger timing and cap behavior are tested hypotheses; set count declares matching native geometry. Global release availability remains unverified in the frozen catalog.",
    },
    Hypothesis {
        id: "sets.shadow_creed_v1",
        description: "Shadow Creed binds Attack2_4_AtkUp/CD to the current four-piece description (.25/20), instead of conflicting Attack_4/CD (.35/10), as a description-matched hypothesis. It uses the selected ratio as a self base-ATK percentage. Successful Skill admission refreshes one nonstacking effect before its delivered hits; switching preserves it until world-time expiry. Set count declares matching geometry; native parameter-caller and clock applicability remain hypotheses. Global release availability remains unverified in the frozen catalog.",
    },
    Hypothesis {
        id: "shinku.aoe_all_in_range_dummy_v1",
        description: "Shinku High-Speed Breach and Crimson Fury deliver their full packet trains to all living scenario-declared dummies under the all-in-range dummy contract: by scenario definition all declared dummies are in range. Targets are delivered in ascending target_index order at the primary impact time. Spatial geometry, recipient caps, distribution and falloff remain unresolved.",
    },
    Hypothesis {
        id: "shinku.burst_packets_v1",
        description: "Shinku's initial Crimson Fury Burst cast delivers UltraSkill1 once, UltraSkill2 once, UltraSkill3 five times, UltraSkill4 once, and UltraSkill5 once, matching the ordered terms and five-hit multiplier in frozen GA_Shinku_UltraSkill.GALevelUpArray[0].DamageGEList and DT_SkillDamageData rows. All packets use the admitted primary impact time; animation intervals and travel are unresolved. Frozen DT_GameplayAbilityTipsData maps GA_Shinku_UltraSkill2_des to the separate UltraSkillPre_Rage packet and repeated Ultimate presses for Crimson Judgment, and GA_Shinku_UltraSkill3_des to the separate UltraSkill_Rage packet requiring another press after Judgment uses are consumed or Surging Crimson ends. Those follow-up actions are explicit inputs outside this first-cast train; their resource, timing, and AoE rules remain unmodeled.",
    },
    Hypothesis {
        id: "shinku.direct_ultimate_followups_v1",
        description: "An explicit InvokeAbility command using configured GA_Shinku_UltraSkill may select the frozen GE_Player_Shinku_UltraSkillPre_Rage_Damage or GE_Player_Shinku_UltraSkill_Rage_Damage row as one direct packet at the caller-selected time. The packet row GAName remains GA_Shinku_UltraSkill and its coefficient uses the configured UltraSkill talent coordinate. This input does not emit the initial Burst packet train or activate/refresh Surging Crimson; an already active state and its live modifiers still apply at delivery. The command does not model the Crimson Judgment/Dragonflame Verdict prerequisites, allowed uses, cost, cooldown, timing, travel, targets, AoE coverage, phase-specific callbacks, or other Rage packets.",
    },
    Hypothesis {
        id: "shinku.normal_chain_v1",
        description: "Shinku ordinary Normal advances five stages with multiplicities 1,2,4,2,1 from frozen neverness_shinku descriptions and native DT_SkillDamageData coefficients. The chain loops after stage five and resets on a different admitted ability, field exit or a gap greater than 2 seconds between input starts. Reset rules, gap and ordered coincident delivery at the research primary impact time are tested hypotheses pending animation/cancel evidence. Rage mode variants (Melee 6-10), Branch attacks, Evade attacks, and rage resource mechanics remain unsupported.",
    },
    Hypothesis {
        id: "shinku.ordinary_hit_energy_v1",
        description: "For Shinku character:1076 only, a modeled ordinary Normal or Skill native packet outside the Charge reaction state grants its exact nonnegative DT_SkillDamageData.ChargeAdd once after positive actual HP loss against target zero. Fatal damage qualifies; zero HP loss, unavailable targets and cancelled packets do not. The living source owner receives 1x; each other living configured recipient receives the explicit source-element to recipient-element coefficient from frozen DT_SkillTeamChargeRelationData (the six recipient-element rows for Shinku COSMOS are 0.6). Each recipient applies their own Charge Efficiency and exact fractional carry, then existing capacity and overflow accounting. Delivery-time owner and source attribution survive switching. GA_Shinku_Passive_1 describes equal off-field Charge as a Charge Enhancement; it does not replace ordinary Ultimate Energy distribution outside that reaction. Charge reaction state, its equal off-field Charge and AoE/ATK callbacks, Menacing Gaze, special gauges, Rage packets, Burst, Branch, AirAttack, QTE, additional targets and callbacks outside the modeled packet train remain unsupported. Eligibility, cadence, recipient and off-field rules are tested hypotheses because native caller evidence is unavailable; ChargeAdd and relation coefficients are frozen native rows.",
    },
    Hypothesis {
        id: "shinku.pre_dawn_refresh_v1",
        description: "Shinku A2 Pre-Dawn Training Grounds applies a target-scoped Cosmos resistance reduction whose 20s window anchors to each Ultimate packet hit time, per DT_ShinkuEffectFigure.json Shinku_DmgResistDownCosmo = -0.1 and Shinku_DmgResistDownCosmo_Dur = 20.0. Reapplication on later Ultimate hits resets the window to the new hit time (RefreshPolicy::Reset), and hits landing exactly at expiry see the status expired. The reset-on-reapplication timing is a model choice pending native reapplication evidence; return condition: native timing rows for repeated Ultimate casts. Non-Cosmos packets are unaffected, and no evidenced +10% DMG bonus exists.",
    },
    Hypothesis {
        id: "shinku.skill_packets_v1",
        description: "Shinku ordinary Skill delivers Skill1_1, Skill1_2, then Skill1_3 once each from one cast, matching the three ordered ratios in frozen neverness_shinku and native damage rows. Children deliver at the admitted primary impact time; animation intervals and travel are unresolved. Rage Skill packets remain a separate action model.",
    },
    Hypothesis {
        id: "skia.normal_chain_v1",
        description: "Skia Normal advances five stages from frozen neverness_skia descriptions and native DT_SkillDamageData: Melee1; Melee2 x2 plus Melee4; Melee5 plus Melee6; Melee7; Melee8 plus Melee9. The chain loops after stage five and resets on a different admitted ability, field exit or a gap greater than 2 seconds between input starts. Reset rules, gap and ordered coincident delivery at the research primary impact time are tested hypotheses pending animation/cancel evidence. Travel, AoE, plunge, riposte, Shadow Hound Gnaw, shadow clones and complete kit behavior remain unsupported.",
    },
    Hypothesis {
        id: "skia.ordinary_hit_energy_v1",
        description: "For Skia character:1008 only, an ordinary Normal or Skill native packet grants its exact nonnegative DT_SkillDamageData.ChargeAdd once after positive actual HP loss against target zero. Fatal damage qualifies; zero HP loss, unavailable targets and cancelled packets do not. The living source owner receives 1x; each other living configured recipient receives the explicit source-element to recipient-element coefficient from frozen DT_SkillTeamChargeRelationData (the six recipient-element rows for Skia LAKSHANA are 0.6). Each recipient applies their own Charge Efficiency and exact fractional carry, then existing capacity and overflow accounting. Delivery-time owner and source attribution survive switching. Burst, enhanced/alternate forms, summons, Branch, AirAttack, QTE, non-native branches, additional targets and callbacks outside the modeled packet train remain unsupported. Eligibility, cadence, recipient and off-field rules are tested hypotheses because native caller evidence is unavailable; ChargeAdd and relation coefficients are frozen native rows.",
    },
    Hypothesis {
        id: "skia.skill_burst_packets_v1",
        description: "Skia Shadow Hound Chase delivers four Skill1 hits and two Dive hits (Skill2 and Skill3) on target zero; The Pack delivers one UltraSkill1 spread hit to each living all-in-range dummy in ascending target index, followed by three UltraSkill2 converge hits on primary target zero only. All children use the admitted effective native talent coordinate and sample live modifiers independently. Ordered coincident delivery at the selected primary impact timestamp is a timing hypothesis pending animation intervals; its spread recipient scope follows the stationary all-in-range dummy contract, without travel, geometry, Shadow Hound Gnaw or complete kit behavior.",
    },
    Hypothesis {
        id: "skill_levels.coordinates_v1",
        description: "Initial purchased level 1; native upgrade coordinate L buys L+1; damage array coordinate is effective level minus one.",
    },
    Hypothesis {
        id: "skill_levels.resonance_3_v1",
        description: "At awakening 3..6, resonance_3 adds one to configured resonance skills; unlisted skills receive no bonus.",
    },
    Hypothesis {
        id: "speedy_hedgehog.proc_v1",
        description: "Speedy Hedgehog set passive grants movement speed and ATK bonus upon dodge or dash.",
    },
    Hypothesis {
        id: "weapons.appliance_v1",
        description: "Real Music adds its frozen refinement ratio to the additive damage-bonus zone for wearer packets with native DamageSourceCategory E, interpreted as Redirect Skill damage. Eligibility follows delivered packets, including off-field delivery, independently of input role and element. The effect is permanent. Global release availability remains unverified in the frozen catalog.",
    },
    Hypothesis {
        id: "weapons.be_happy_v1",
        description: "Be Happy heals its wearer for the frozen fraction of live Max HP captured exactly after the wearer causes an alive-to-dead target transition with delivered damage. Teammate defeats, zero/nonfatal hits, and already defeated targets do not trigger it. Delayed packets retain their owner's credit while off field. Healing is capped at missing HP, cannot revive, and uses one nearest-integer conversion at the callback. One wearer has a 20s world-time cooldown beginning on each qualifying defeat, including a trigger at full HP; the exact deadline is ready. Simultaneous defeats share that cooldown and separate wearers have independent deadlines. Trigger credit, full-HP cooldown use, and exact cooldown boundary are tested hypotheses. Live bonus handling is separately covered by combat.live_healing_modifiers_v1. Global release availability remains unverified.",
    },
    Hypothesis {
        id: "weapons.bit_game_v1",
        description: "Blow up the Crowd (Beat King) increases the active character's ATK while the wearer is off-field, and dealing damage while off-field grants the active character an additional stacking ATK bonus up to 4 stacks with a 2.0s ICD, resetting when the wearer becomes active. While the wearer is active, it increases Psyche DMG dealt, and dealing Psyche DMG with Basic Attacks grants an additional stacking Psyche DMG bonus up to 10 stacks with a 0.3s ICD, resetting on character switch. Off-field damage detection, 2.0s and 0.3s ICDs, active-actor recipients, and switch resets are tested hypotheses. Release availability at the cutoff is unverified.",
    },
    Hypothesis {
        id: "weapons.bittercake_damage_event_v1",
        description: "On positive actual HP loss, Bittercake applies its DEF buff after actor HP has been settled, including when that loss is fatal. The first trigger is ready at time zero, suppression does not move the cooldown, and the cooldown starts on successful activation. Buff expiry precedes equal-time events; same-time incoming damage precedes later dynamically queued hit deliveries. Native text establishes damage received, duration and cooldown, but not trigger readiness, fatal-hit activation or exact same-time ordering.",
    },
    Hypothesis {
        id: "weapons.black_book_v1",
        description: "Black Tome grants the wearer its frozen Break Intensity points unconditionally. The points join the break damage zone as 1 + BI/300 + break_bonus_sum; they do not enter gauge accumulation. Arc: Black Tome is sealed by two chains by default; each team Support Skill cast unleashes one chain. When fully unleashed, explicit Arc admission enters a 0.2-second startup. Successful startup summons the Black Tome for 20 seconds with no normal cooldown and reseals the Arc with two chains; an interruption during startup instead applies a 30-second cancellation cooldown and does not summon or reseal. A pending startup or live summon prevents overlapping activation. The summon designates an enemy at spawn and refreshes designation every 5 seconds, preferring living locked targets else lowest index. The wearer gains its frozen Chaos damage bonus against designated enemies. When a designated enemy takes Break damage while the summon is active, the wearer deals an immediate Chaos ATK hit. Switching preserves the summon; death during startup cancels it, while ticks and designation stop on wearer death and pending hits cancel postmortem. Global release availability remains unverified in the frozen catalog.",
    },
    Hypothesis {
        id: "weapons.blast_candy_v1",
        description: "Blast Candy uses the frozen refinement ratio as a self base-ATK percentage. Successful Ultimate admission refreshes one nonstacking effect before its delivered hits; switching preserves it until world-time expiry. Global release availability remains unverified in the frozen catalog.",
    },
    Hypothesis {
        id: "weapons.bopu_v1",
        description: "Cosmos Daze, Wild Reverie (Bopp) grants its frozen damage-dealt bonus ratio for its duration after successful Support Skill admission, before same-time hits. One nonstacking buff uses a separate per-wearer world-time trigger cooldown. Casts inside cooldown do not refresh it; exact expiry precedes equal-time hits and exact cooldown readiness permits a new grant. Switching preserves buff and cooldown; delayed packets sample delivery-time stats. These order/clock/scaling rules are tested hypotheses; frozen Global release availability remains unverified in the frozen catalog.",
    },
    Hypothesis {
        id: "weapons.boxing_candy_v1",
        description: "Boxing Candy adds the frozen ordinary damage bonus, replaced by the enhanced bonus against a target strictly below 50 percent maximum HP before each delivered hit. Bonuses share the additive damage zone; the wearer retains the effect off-field. Delivery-time HP sampling is a tested hypothesis. Global release availability remains unverified in the frozen catalog.",
    },
    Hypothesis {
        id: "weapons.call_of_the_twisted_city_v1",
        description: "On successful wearer Redirect Skill admission, Call of the Twisted City grants one wearer-only healing_bonus instance for the frozen 10s duration before same-time callbacks. A recast resets the same source instance; the living off-field wearer keeps it until world-time expiry, with BuffManager expiry ordering at an exact deadline. Failed admissions, QTE, Ultimate and Normal inputs do not trigger it. Trigger role, callback ordering, refresh, source lifetime and expiry are tested model hypotheses. Global release availability remains unverified.",
    },
    Hypothesis {
        id: "weapons.contemplative_cat_v1",
        description: "Contemplative Cat adds stacks times its frozen refinement Cosmos ratio to the additive damage-bonus zone of the wearer's Cosmos packets, where stacks = min(10, floor(start_fons / 100000)). The explicit party-wide starting Fons balance is shared by every reached consumer; zero is valid and a missing value fails only when this passive is reached. Global release availability remains unverified in the frozen catalog.",
    },
    Hypothesis {
        id: "weapons.crowbar_v1",
        description: "Time Bandit grants its frozen Break Intensity points to the wearer for the frozen duration after an admitted wearer Redirect Skill cast. The buff is wearer-only, capped at one stack with no refresh: casts admitted while it is active neither stack nor extend it, and a cast admitted after expiry grants a fresh duration. The grant survives the wearer switching off-field. Break Intensity points join the break damage zone as 1 + BI/300 + break_bonus_sum; actual gauge and HP settlement require the separate explicit target Break models. Arc: Picky is non-combat and unmodeled. Global release availability remains unverified in the frozen catalog.",
    },
    Hypothesis {
        id: "weapons.demon_blade_v1",
        description: "Ravenous Blade adds its frozen Crit Rate ratio to the wearer's crit rate in the additive base-rate zone. Dealing Incantation DMG grants one wearer stack of Crit DMG through ProcCallback up to seven stacks, with a 0.3s trigger cooldown. Each trigger refreshes the group's duration to 15s, including at cap. Switching preserves stacks and cooldown; exact expiry precedes equal-time hits. Cooldown, stacking, and world-time clocks are tested hypotheses. Global release availability remains unverified in the frozen catalog.",
    },
    Hypothesis {
        id: "weapons.dustbin_v1",
        description: "Decomposer grants the wearer its frozen Break Intensity points only after its packet actually lowers a living target's modeled Break gauge under combat.unbal_value_as_target_gauge_v1. Semantic tags alone do not trigger it. The wearer-only buff lasts 10 seconds without refresh, with a 20-second per-wearer ICD, both sampled on the real settlement clock with half-open deadlines. The grant precedes live Break contribution capture, so the gauge-breaking hit benefits from the new BI. Partial reductions and delivered off-field packets qualify; missing bars, zero effective loss, zero starting gauge, already Broken targets and defeated recipients do not. Pending accepted reductions preserve their source attribution. BI adds as points in 1 + BI/300 + break_bonus_sum; ranks T1-T5 grant 60/66/72/78/84. This is a bounded causal model derived from the reducing-Break tooltip and frozen curves, not proof of native trigger ordering, enemy profile semantics or Global release availability.",
    },
    Hypothesis {
        id: "weapons.eternal_waltz_v1",
        description: "Eternal Waltz adds its frozen Max HP ratio in the compiled build HP-percent domain. Successful wearer Ultimate admission refreshes one wearer-only Mental damage bonus before same-time hits for the frozen duration; switching preserves it until world-time expiry, and expiry precedes equal-time hits. Native PSYCHE packets do not match the Mental bonus. Static HP integer settlement, cast order and clocks are tested model hypotheses. Global release availability remains unverified in the frozen catalog.",
    },
    Hypothesis {
        id: "weapons.far_side_moon_v2",
        description: "Far Side of the Moon adds its refinement Crit Rate to the wearer's base crit rate. Positive HP loss from the wearer's delivered native A-category packet grants one stack in ProcCallback, up to four, with a 300ms acquisition gate across all recipients. Each grant renews one shared five-second stack deadline, including at cap. Delivered native E-category packets add the current stack total. Successful wearer Redirect Skill admission refreshes one 15-second Basic Attack window before same-time hits. Lakshana damage gains its refinement bonus during overlap of the two carriers, additive with any matching category bonus. Switching preserves both effects and their acquisition gate. Effects and gate use world time, expiry precedes equal-time hits, and delayed packets read delivery-time state. Shared stack refresh, clock choice and callback order are tested model assumptions.",
    },
    Hypothesis {
        id: "weapons.forgotten_hp_threshold_v1",
        description: "The Forgotten grants the frozen ATK percentage while the wearer is strictly above half HP and the frozen DEF percentage while strictly below half HP; at exactly half neither applies. The wearer's current HP is sampled immediately before each delivered hit, including delayed and off-field hits. The threshold sampling and DEF-scaled packet integration are tested hypotheses. The frozen Global source identifies the item and weapon group ONE; release availability at the cutoff is unverified.",
    },
    Hypothesis {
        id: "weapons.gold_record_v1",
        description: "Voice of the Voyager (Golden Record) adds its frozen ATK ratio to the wearer's base ATK percentage zone. When the wearer casts or deals damage with Support Skill (QTE), its frozen Support Skill Crit DMG ratio is added to the wearer's Crit DMG. Dealing damage with a triggered Support Skill grants one wearer stack of Ultimate Crit DMG through ProcCallback up to three stacks, lasting 3s of world time. Each trigger resets the duration to 3s. Stacks expire all at once. Switching preserves stacks and duration; exact expiry precedes equal-time hits. Cooldown, stacking, and world-time clocks are tested hypotheses. Global release availability remains unverified in the frozen catalog.",
    },
    Hypothesis {
        id: "weapons.gold_wool_v1",
        description: "What's Desired adds its frozen Lakshana ratio to matching delivered packets in the additive damage-bonus zone. Successful Redirect Skill or Ultimate admission refreshes one nonstacking self CRIT damage modifier before same-time hits, with world-time duration and delivery-time buff sampling. Switching preserves the modifier; exact expiry precedes equal-time hits. Cast order and clocks are tested hypotheses. Global release availability remains unverified in the frozen catalog.",
    },
    Hypothesis {
        id: "weapons.good_boy_v1",
        description: "Good Boy grants party ATK percentage bonus upon wearer Ultimate cast under native curve scaling.",
    },
    Hypothesis {
        id: "weapons.jiaojuan_v1",
        description: "Film - MANISH grants the wearer its frozen Break Intensity points unconditionally and adds its frozen Broken ratio to the single additive damage-bonus zone (M_bonus) only while the target carries an active Broken status. The Break Intensity points join the break damage zone as 1 + BI/300 + break_bonus_sum; they do not enter gauge accumulation, and Break Damage/BI settlement still has no content-simulation consumer. Broken-state detection uses the content-owned ntesim.status.broken target status tag; the bounded content runner can produce that tag only for an explicit target profile under combat.unbal_value_as_target_gauge_v1. The tag name is not asserted to be the native GameplayTag because the frozen tables expose no native break-gauge binding. The damage ratio is not a final, special, or multiplicative factor. Global release availability remains unverified in the frozen catalog.",
    },
    Hypothesis {
        id: "weapons.jingmotingyuan_v1",
        description: "Silent Garden grants one wearer-only additive CRIT DMG stack after positive settled HP consumption, capped at four with shared refreshed five-second world-time expiry. Zero consumption, dead actors, incoming damage and action tags alone grant nothing. The explicit arc input requires the living active wearer outside recovery and its source-bound 25-second cooldown. It consumes the source-bound 5% of current HP, rounded to nearest integer with exact halves upward and clamped to preserve one HP, then grants the consumption stack before damage. One independent seeded wearer stream chooses equally between five Incantation ATK packets and six with a doubled final coefficient; refinement curves remain source-bound. The bounded atomic timing model resolves ordered packets at activation without added recovery, across all in-range dummies, with one choice and cost per activation and no ordinary character-hit energy or chain advancement. Native attack intervals, recovery, recipient selection, HP rounding, probabilities and other ability HP costs remain unresolved; this model does not establish full weapon readiness. Switching preserves owner cooldown and RNG. Global release availability remains unverified in the frozen catalog.",
    },
    Hypothesis {
        id: "weapons.kite_v1",
        description: "On admitted wearer Redirect Skill casts, Watch Your Heads! grants one wearer-only ATK percentage instance for the frozen 15s duration before same-time callbacks. The instance neither stacks nor refreshes on recast: a grant inside the active window keeps the first expiry. Failed admissions and non-Redirect roles do not grant. The ATK bonus adds to the live ATK percent zone. Lakshana packets delivered while the window is active reach an explicit unsupported return condition: the native bonus requires the target to be affected by Remora and Stain, which have no engine model, so the bonus is refused rather than silently applied or dropped. Trigger role, wearer-only scope, no-refresh lifecycle, world-time expiry and the unsupported Lakshana condition are tested model hypotheses. Global release availability remains unverified in the frozen catalog.",
    },
    Hypothesis {
        id: "weapons.knight_candy_v1",
        description: "Fluff of Ferocity adds wearer CRIT damage after each connected critical packet/target through ProcCallback, up to ten stacks. Every trigger resets the whole group's duration, including at cap. Hits already queued at the same time use prior stacks; expiry precedes equal-time hits. Stacks persist off-field. Multiplicity, callback order and world-time duration are tested hypotheses. Global release availability remains unverified in the frozen catalog.",
    },
    Hypothesis {
        id: "weapons.koinobori_v1",
        description: "A Time Will Come grants the wearer its frozen ATK, DEF, and Max HP percentage bonuses when the deployed team composition contains at least 3 distinct Esper elements. When fewer than 3 distinct elements are present, no bonus applies. The Max HP bonus settles in the compiled build HP-percent domain; ATK and DEF percentages apply to the wearer's delivered hits. Team element evaluation and stat integration are tested hypotheses. Release availability at the cutoff is unverified.",
    },
    Hypothesis {
        id: "weapons.lunarphase_v1",
        description: "Blushing Mirage adds its frozen ATK ratio to matching delivered packets in the additive base-ATK percentage zone. Successful Ultimate admission refreshes one wearer Cosmos damage modifier and one wearer DEF ignore modifier before same-time hits, with world-time duration. Switching preserves both modifiers; exact expiry precedes equal-time hits. Cast order and clocks are tested hypotheses. Global release availability remains unverified in the frozen catalog.",
    },
    Hypothesis {
        id: "weapons.marching_beyond_time_v1",
        description: "Marching Beyond Time adds its frozen ATK ratio unconditionally. An admitted wearer Redirect Skill enters Wastelab and clears stacks; each admitted Redirect Skill or Support Skill by another living teammate while Wastelab is active adds one stack up to three, including off-field wearer observation. An admitted wearer Ultimate exits Wastelab and consumes current stacks; only while active it adds base State Crit DMG plus per-stack Crit DMG to every packet delivered from that Ultimate command, including delayed packets. Three consumed stacks also grant wearer-only DEF ignore beginning at Ultimate admission and ending exactly at the frozen duration boundary; each hit samples it at delivery. Rejected inputs, other roles, stack overflow, and Ultimates outside Wastelab do not trigger those effects. These action-time, recipient, and expiry boundaries are explicit tested hypotheses; frozen data does not establish Global release availability.",
    },
    Hypothesis {
        id: "weapons.mind_royale_v1",
        description: "Mind Royale schedules its frozen integer base energy on successful wearer Redirect Skill admission under its owner cooldown. At delivery, only a living wearer receives energy, including off-field; death cancels the pending grant without mutating energy, overflow or fractional carry, and without refunding the admitted cooldown. Living settlement uses the owner's charge_multiplier, fractional carry and capacity/overflow accounting. Delivery-time living eligibility and persistence across switching are checked bounded hypotheses, consistent with living-recipient team generation; native eligibility and charge scaling remain unresolved.",
    },
    Hypothesis {
        id: "weapons.moon_v1",
        description: "Stellar Veil adds its frozen Psyche ratio to matching delivered packets in the additive damage-bonus zone. Dealing Psyche DMG grants one wearer stack of Crit DMG through ProcCallback up to ten stacks, with a 0.1s trigger cooldown. Each trigger refreshes the group's duration to 5s, including at cap. Switching preserves stacks and cooldown; exact expiry precedes equal-time hits. Cooldown, stacking, and world-time clocks are tested hypotheses. Global release availability remains unverified in the frozen catalog.",
    },
    Hypothesis {
        id: "weapons.motor_candy_v1",
        description: "Fluff of Fleetness starts at zero stacks on initial active-field entry and every real entry. Each full world-time interval adds one self base-ATK percentage stack up to five while the wearer is alive and active. Successful departure clears stacks and invalidates old timers; same-actor and rejected switches preserve them. Admission precedes equal-time ticks, which precede hits. Delayed hits use delivery-time stacks. Initialization, clock and snapshot rules are tested hypotheses. Global release availability remains unverified in the frozen catalog.",
    },
    Hypothesis {
        id: "weapons.nakupeda_v1",
        description: "Nacupeda adds its frozen HP ratio in the compiled build HP-percent domain: compiled HP = compiled HP + native base HP * ratio, settled once to nearest integer, following the Eternal Waltz static precedent. On each successfully admitted wearer Ultimate while its 30s world-time proc cooldown is ready, reserve cooldown from admission even at full HP, then make one unbiased seeded equal-choice selection among its three effects; failed admissions and casts before the exact deadline draw nothing. At admission, capture the outcome, affected recipients, and each involved effective Max HP before same-time cast buff callbacks; a same-time ProcCallback then applies live healing modifiers and rounds once to nearest integer. Owner defeat before callback cancels settlement while retaining selection and cooldown provenance. Effect 1 heals one living teammate with the lowest exact current-HP/Max-HP fraction (slot order breaks ties) by recipient Max HP times its ratio; effect 2 grants the wearer a timed shield from captured exact wearer Max HP times its ratio, multiplied at the grant callback by one plus the granter static and live shield_bonus sum, then rounded once for the frozen duration; later modifier expiry does not rescale its capacity; effect 3 heals each living teammate by that recipient's Max HP times its ratio. Healing and shield settlement use shared VitalState, cap healing at missing HP, never revive, and shields absorb before HP, expire at their half-open deadline, persist through switching, replace this proc's prior shield, and are removed on wearer death. These recipient, equal-choice, captured-base, trigger/ICD, rounding and lifetime rules are reviewed tested hypotheses from the frozen ratios/native description, not verified native caller behavior. Global release availability remains unverified in the frozen catalog.",
    },
    Hypothesis {
        id: "weapons.native_curve_v1",
        description: "Equipped native weapon parameters select exact refinement values: Mind Royale grants energy on Skill; Good Boy grants team ATK percentage on Ultimate; Bittercake grants DEF on explicit incoming-damage triggers.",
    },
    Hypothesis {
        id: "weapons.nest_bird_v1",
        description: "Tears Beneath the Mask marks enemies hit by the wearer's connected Ultimate (native DAMAGE_SOURCE_CATEGORY_Q) packets with Warning Gaze: the marked enemy's dealt damage is reduced by the frozen refinement ratio for 20s, non-stacking so re-application refreshes the window. Hit deliveries currently address only target index 0. Enemy outgoing damage has no content-runtime consumer yet, so the mark is stored with its ratio and expiry while the reduction awaits enemy attack execution. buff_NestBird_CD2 (30s) is an observed frozen curve without an evidenced gameplay role. Global release availability remains unverified in the frozen catalog.",
    },
    Hypothesis {
        id: "weapons.noros_v1",
        description: "Noros grants the frozen self base-ATK percentage for its duration after successful Redirect Skill admission, before same-time hits. One nonstacking buff uses a separate per-wearer world-time trigger cooldown. Casts inside cooldown do not refresh it; exact expiry precedes equal-time hits and exact cooldown readiness permits a new grant. Switching preserves buff and cooldown; delayed packets sample delivery-time stats. These order/clock/scaling rules are tested hypotheses; frozen Global release availability remains unverified.",
    },
    Hypothesis {
        id: "weapons.oraora_v1",
        description: "Oraora! grants one wearer Basic Attack DMG stack after each connected delivered native A-category packet/target, up to ten independently expiring stacks lasting ten seconds of world time. The delivery category, independent per-packet/target multiplicity, same-time queued-hit ordering, switch persistence, off-field ownership, exact-expiry order, and discarding triggers at cap without refreshing existing lifetimes are tested hypotheses; the frozen text does not specify these edge semantics. Frozen Global release availability remains unverified.",
    },
    Hypothesis {
        id: "weapons.paper_plane_v1",
        description: "Paper Plane adds its frozen refinement ratio to the damage-bonus zone for the wearer's Anima (NATURE) packets with native DamageSourceCategory E or Q, interpreted as Redirect Skill or Ultimate damage. Eligibility follows delivered packets, including off-field delivery, independently of input role. Global release availability remains unverified in the frozen catalog.",
    },
    Hypothesis {
        id: "weapons.police_rat_v2",
        description: "Justice Executioner grants the wearer its frozen ATK percentage unconditionally, joining the live base-ATK-scaled ATK sum, and adds its frozen Boss ratio to the single additive damage-bonus zone (M_bonus) when the explicitly supplied scenario target metadata is_boss is true. Missing is_boss uses the runner's false default only; this does not classify a target as non-Boss in the native game. The Boss ratio is not a final, special, or multiplicative factor. The frozen package does not establish the native metadata that identifies a Boss. Arc: Officer Whisker: explicit Arc admission by the living active wearer summons Officer Whisker for 30 seconds with its 60-second cooldown, delivering periodic Physical ATK hits every 1 second. Ticks continue while the wearer is off-field, stop immediately on wearer death, and pending hits cancel postmortem. Global release availability remains unverified in the frozen catalog.",
    },
    Hypothesis {
        id: "weapons.prokaryon_v1",
        description: "Us. adds its frozen refinement ratio to the additive damage-bonus zone for wearer packets with native DamageSourceCategory A, interpreted as Basic Attack damage. Eligibility follows delivered packets, including off-field delivery, independently of input role and element. The effect is permanent. Global release availability remains unverified in the frozen catalog.",
    },
    Hypothesis {
        id: "weapons.raging_flames_v1",
        description: "Raging Flames adds its frozen Lakshana ratio to matching delivered packets in the additive damage-bonus zone. Successful wearer Ultimate admission refreshes a wearer-only 10-second window before same-time hits and clears Redirect Skill stacks; each admitted Redirect Skill during the active window adds one stack before same-time hits, capped at two, without extending the window. The base bonus applies to delivered native E/Q packets; per-cast stacks apply only to delivered native E packets. Input action role gates cast admission, while delivered packet category gates damage bonuses. Switching preserves the window and stacks; exact expiry precedes equal-time hits. Ultimate refresh clearing stacks is a tested lifecycle hypothesis because native buff lifecycle details are unavailable. Cast order and clocks are tested hypotheses. Global release availability remains unverified in the frozen catalog.",
    },
    Hypothesis {
        id: "weapons.reality_refuge_v1",
        description: "Reality Refuge adds its frozen Anima ratio to the wearer's Nature-damage packets and its frozen Attachment ratio to the wearer's Attachment-tagged packets, both summed into the additive damage-bonus zone. Anima damage is modeled as Nature damage following the frozen The Wrong Gate Anima precedent. Attachment packets are recognized by the content-owned ntesim.damage.attachment semantic tag, which no modeled character currently emits, so both Attachment bonuses are structurally present but inert in the current roster. Successful wearer Ultimate admission refreshes one wearer-only timed buff that replaces the static Attachment bonus with the frozen Ultimate value until world-time expiry; a recast resets the same instance without stacking, and switching preserves it until expiry. Cast admission order and BuffManager expiry sampling at hit delivery are tested timing hypotheses. Global release availability remains unverified in the frozen catalog.",
    },
    Hypothesis {
        id: "weapons.rishi_v2",
        description: "Eclipse grants the wearer its frozen ATK percentage unconditionally, joining the live base-ATK-scaled ATK sum. The ratio is not a final, special, or multiplicative factor. Arc: Eclipse: explicit Arc admission by the living active wearer outside recovery creates a 40s Eclipse field; each wearer-credited enemy defeat during the field restores flat frozen energy (6/7/8/9/10 per T1-T5) up to 5 defeats per field; re-creating the field waits a 300s arc cooldown. Arc activation uses the shared action admission and cooldown service, with no HP or energy cost and no added recovery under the bounded instantaneous field-creation model. Ultimate casts alone create no field. Wearer-kill credit (not team kills), field-window boundaries, charge scaling and instantaneous activation remain tested hypotheses. The energy grant flows through actor.grant_energy, so it scales by charge_multiplier (tested; native scaling unknown). Global release availability remains unverified in the frozen catalog. A dead wearer cannot claim further field defeats or activate pending field/energy callbacks. A grant already reserved before death is skipped without changing energy, overflow or carry; its consumed defeat slot and admitted cooldown are not refunded. Living off-field wearers remain eligible. Delivery-time living eligibility is a bounded inference consistent with living-recipient team generation and absence of a revival rule, not native callback proof.",
    },
    Hypothesis {
        id: "weapons.rose_break_extension_v1",
        description: "Under an explicit target break_profile, a connected positive-damage hit by a Bohemian Rose wearer extends a target that was already in the content runner's Broken state by the frozen buff_Rose_UnbalTime once during that modeled Break generation. The deadline moves from its previous value by exactly the extension duration; the target Broken tag and recovery event follow it. A hit that itself causes the Break cannot extend it, and defeated targets, targets without an explicit profile, unbroken targets and repeated hits in the same Break do not extend. This is a bounded runner hypothesis, not native target-profile, packet-eligibility, recovery, or Break Effect evidence.",
    },
    Hypothesis {
        id: "weapons.rose_v1",
        description: "Bohemian Rose grants the wearer its frozen ATK percentage unconditionally, joining the live base-ATK-scaled ATK sum. Each time the wearer deals DoT, one Chaos Thorn stack is granted (wearer-only timed buff, 3s, refresh on retrigger, 10-stack cap, 0.3s trigger ICD); each stack adds its frozen CRIT DMG ratio to the additive critical-damage zone. A Redirect Skill cast grants 10 stacks immediately, bypassing the DoT ICD. Redirect admission grants stacks before its hits; each connected wearer DoT packet grants after damage settlement and cannot enhance that same hit. DoT delivery is identified by the owner-declared State.Damage.Dot semantic tag, including delayed off-field packets. Native lacrimosa blood DoT scheduling remains pending. Global release availability remains unverified in the frozen catalog.",
    },
    Hypothesis {
        id: "weapons.snowman_v1",
        description: "Solifugae Vita grants the wearer its frozen ATK percentage only while the wearer's settled actor-shield state contains a positive nonexpired shield at packet delivery. The bonus joins the live base-ATK-scaled ATK sum (base_atk * p added to ATK), never multiplying compiled static ATK. Global release availability remains unverified in the frozen catalog.",
    },
    Hypothesis {
        id: "weapons.spider_v1",
        description: "Failing You, Heavy in My Heart grants the wearer 1 stack of Spider Knowledge when dealing damage with Basic Attacks, up to 8 stacks with an internal cooldown of 0.5s between stack gains. Casting Ultimate consumes all Spider Knowledge stacks; if at least 1 stack is consumed, it grants the entire team an ATK percentage bonus for 10 seconds equal to the per-stack bonus multiplied by the consumed stacks, plus an additional flat bonus if all 8 stacks were consumed. Basic Attack damage detection, 0.5s ICD enforcement, Ultimate cast consumption, and team ATK scaling are tested hypotheses. Release availability at the cutoff is unverified.",
    },
    Hypothesis {
        id: "weapons.the_wrong_gate_healing_v1",
        description: "The Wrong Gate's static ATK percentage applies to the wearer on every hit. An Edgar Ultimate or Iroi ordinary Skill healing callback triggers only when the living equipped wearer heals a living team actor and the once-rounded pre-cap amount is positive; a full-HP overheal qualifies. After settlement, one shared-name reset effect grants the wearer Anima (NATURE) DMG and every other living configured team actor general DMG for 20 world seconds. Repeated triggers refresh at callback time; source attribution is the latest trigger and buffs capture a snapshot that persists if its source later dies. Failed/dead-recipient callbacks and nonpositive rounded healing do not trigger. These trigger, recipient, clock, order, snapshot, and shared-name rules are tested hypotheses because the frozen descriptions do not specify them. Global release availability remains unverified.",
    },
    Hypothesis {
        id: "weapons.thief_candy_v1",
        description: "Thief's Candy grants the wearer a timed damage bonus after a Critical Dodge: each reached assumed-success trigger adds one stack of a wearer-only BuffManager buff (id thief_candy.critical_dodge_damage_up, source weapon:fork_ThiefCandy), up to 3 stacks, and resets the 10s duration on every trigger. The per-stack bonus joins the additive M_bonus damage zone as 1 + bonus_sum. Critical Dodge remains an explicit scenario assumption (`dash[critical=true]`); this model does not establish native success windows, invulnerability, stamina, a counter window or a Riposte packet. Direct owner tests may use the internal semantic tag, but public rotations dispatch only from the typed assumed-success input. Stack cap 3, duration reset on retrigger and wearer-only scope are tested model hypotheses. Global release availability remains unverified in the frozen catalog.",
    },
    Hypothesis {
        id: "weapons.tiger_tally_v2",
        description: "Commander Tiger Talisman grants the wearer its frozen ATK percentage unconditionally, joining the live base-ATK-scaled ATK sum. Each admitted wearer Redirect Skill or Ultimate cast adds one wearer-only damage instance of the frozen Normal ratio for the frozen duration, stacking to 2 with independent per-instance expiry; a grant at the cap is dropped without refreshing survivors. The damage bonus joins the additive M_bonus zone and applies only to delivered packets classified as native Basic Attack (A) or Critical Riposte (Z), independently of the initiating input role; other recognized packet categories are excluded and missing/unknown reached classifications diagnose. Redirect Skill records one Left Tiger Talisman and Ultimate records one Right Tiger Talisman until cast time + the frozen 15s window; a same-side grant refreshes that side, and receiving the opposite side strictly before its deadline consumes the pair and immediately activates Arc. The automatic activation and pair consumption are bounded hypotheses because the text says the pair unlocks Arc but no separate Arc action is represented. Arc refreshes one wearer-only 10s damage bonus in M_bonus, read only against targets whose explicit scenario is_boss flag is true; retriggers reset its duration. Talisman refresh/consumption, Arc auto-activation/refresh and scenario Boss eligibility are tested model hypotheses. Critical Riposte category Z receives the Basic Attack damage bonus; its owner admission remains a separate mechanic. Global release availability remains unverified in the frozen catalog.",
    },
    Hypothesis {
        id: "weapons.tuansanlang_v1",
        description: "Danzaburou grants its frozen Break Intensity points to every team member (including the wearer) whose Esper Type matches the wearer's, once the team holds 3 or more characters of that Esper Type. Esper Type is the character's native CharacterElementType. Grants through this path do not stack: when several wearers qualify for the same element, the largest bonus applies. Break Intensity points join the break damage zone as 1 + BI/300 + break_bonus_sum under the separate explicit target Break models. The roster threshold counts deployed slots, including defeated matching peers, while only living carriers supply the aura. Field presence is not required. Each new Break generation captures the eligible strongest aura with the other source parameters; subsequent carrier death does not alter accepted packets. Global release availability remains unverified in the frozen catalog.",
    },
    Hypothesis {
        id: "weapons.twinbirds_v1",
        description: "Twinflight adds its frozen Psyche ratio to matching delivered packets in the additive damage-bonus zone. Successful Ultimate admission refreshes one team CRIT damage modifier and one wearer additional CRIT damage modifier before same-time hits, with world-time duration. Switching preserves both modifiers; exact expiry precedes equal-time hits. Cast order and clocks are tested hypotheses. Global release availability remains unverified in the frozen catalog.",
    },
    Hypothesis {
        id: "weapons.whale_v1",
        description: "Song of the Whale grants the wearer its frozen ATK percentage unconditionally, joining the live base-ATK-scaled ATK sum. Against targets carrying an active Broken status it adds its frozen Broken ratio to the additive damage-bonus zone. Defeating such a target heals the wearer for the frozen fraction of live Max HP, settled with live healing modifiers, capped at missing HP, unable to revive, rounded once to nearest integer. One per-wearer 30s world-time cooldown starts on each qualifying defeat and is consumed even when the wearer is defeated before the callback settles; simultaneous defeats share it and separate wearers are independent. Broken-state detection through the content-owned ntesim.status.broken target status tag, trigger credit, cooldown consumption and heal timing are tested hypotheses; the frozen tables expose no native break-gauge binding. Global release availability remains unverified.",
    },
    Hypothesis {
        id: "weapons.worldrain_v1",
        description: "Worldrain adds its frozen refinement ratio to the damage-bonus zone for the wearer's Cosmos packets with native DamageSourceCategory E or Q, interpreted as Redirect Skill or Ultimate damage. Eligibility follows delivered packets, including off-field delivery, independently of input role. Successful wearer Redirect Skill admission refreshes one wearer-only Cycle Intensity buff from the frozen Mag points for the frozen CD seconds before equal-time consequences. It joins live CI readers, never the ordinary damage-bonus zone. Recasting resets its world-time expiry without stacking; switching preserves it and exact expiry precedes equal-time reads. Global release availability remains unverified in the frozen catalog.",
    },
    Hypothesis {
        id: "weapons.yaodao_separate_packet_v1",
        description: "Bounded Crimson Hexblade packet model: for each connected parent packet explicitly tagged ntesim.trigger.parry_attack on the configured wearer, emit one separate Incantation ATK-scaled child at the parent's delivery time and target, with its own receipt, standard damage calculation and an independent per-hit crit draw. The runner applies pre-hit modifiers selected by its ordinary packet-metadata rules, but native eligibility of those modifiers for this child is unknown. It omits post-hit callbacks (ordinary-hit energy, character/gear/Fireflies callbacks, Break and target-defeat callbacks) because their eligibility is unverified; target HP still receives the child damage. Only the separate additional Incantation ATK ratio is frozen-source-backed; native Parry admission, child multiplicity across multi-hit/multi-target actions, timing, crit policy, damage-bonus and callback eligibility are not established. This caller-declared slice makes no native coverage claim.",
    },
    Hypothesis {
        id: "weapons.yaodao_v1",
        description: "Crimson Hexblade retains the frozen ATK ratio for an additional Incantation packet after a wearer Parry Attack. Native Parry admission remains unsupported; the content-owned ntesim.trigger.parry_attack tag is an explicit caller declaration. The bounded runtime model emits one separate child for each tagged connected wearer packet, at the parent delivery time and target, with standard damage calculation and an independent per-hit crit draw. Native multiplicity, timing, crit policy, pre-hit modifier eligibility, post-hit callbacks and energy eligibility are unverified; child post-hit callbacks are intentionally omitted. Reject multi-hit/multi-target native claims. Global release availability remains unverified in the frozen catalog.",
    },
    Hypothesis {
        id: "weapons.yuren_v1",
        description: "Rainman grants the wearer its frozen HP ratio in the compiled build HP-percent domain: compiled HP = compiled HP + native base HP * ratio, settled once to nearest integer, following the Eternal Waltz static precedent. Frozen localized text says Rainman strengthens wearer shields above 50% HP, supporting wearer-only application. At an ordinary shield grant to its wearer, the shield-strength ratio contributes to the same additive outgoing shield_bonus sum as static and live ShieldEfficiency only when the living wearer has current HP strictly above half of live effective Max HP; outgoing-zone addition and grant-time sampling are tested hypotheses. The comparison is exact. The settled capacity is not recalculated as HP or bonuses change. A Rainman wearer granting a shield to another actor is unsupported. Global release availability remains unverified in the frozen catalog.",
    },
    Hypothesis {
        id: "zankou.normal_chain_v1",
        description: "Zankou canonical Normal derives the current Reality five-stage2/1/3/5/6 or Illusion four-stage1/3/4/3 packet chain from live form. Exact native ATK coefficients retain A talent and Incantation scaling; Illusion direct Normal/held packets carry their source Follow-up tag. Source montage spacing, explicit contact pauses and action gates use zankou.forms_v1. Conservative ordinary continuation uses AnySkill recovery followed by a maximum two-second idle gap. A different admitted action, form transition, field exit or longer idle resets the chain. Current form Normal1/2 permits its held branch after at least200000us declared hold; Reality and Illusion branches retain four and five direct contacts. Reality Normal4/5 and held branch earn enhanced entry only inside their live Skill window. Positive HP resource channels are per causal packet, independent of recipient count. These finite timing, continuation and prerequisite interpretations are bounded source models, not measured playback.",
    },
    Hypothesis {
        id: "zankou.ordinary_hit_energy_v1",
        description: "For Zankou character:1036 only, a modeled ordinary Normal native packet grants its exact nonnegative DT_SkillDamageData.ChargeAdd once after positive actual HP loss against target zero. Fatal damage qualifies; zero HP loss, unavailable targets and cancelled packets do not. The living source owner receives 1x; each other living configured recipient receives the explicit source-element to recipient-element coefficient from frozen DT_SkillTeamChargeRelationData (the six recipient-element rows for Zankou INCANTATION are 0.6). Each recipient applies their own Charge Efficiency and exact fractional carry, then existing capacity and overflow accounting. Delivery-time owner and source attribution survive switching. Skill, Burst, alternate forms, summons, DoT, Branch, AirAttack, QTE, non-native branches, additional targets and callbacks outside the modeled packet train remain unsupported. Eligibility, cadence, recipient and off-field rules are tested hypotheses because native caller evidence is unavailable; ChargeAdd and relation coefficients are frozen native rows.",
    },
    Hypothesis {
        id: "zero.a1_first_hit_v1",
        description: "Zero A1 triggers on the first nonzero connected Skill hit per admitted activation and target, only when target level <= Zero level. Prefer the explicit Chinese not-above-level condition over the English lower-level translation; child packets cannot retrigger and retain the struck target.",
    },
    Hypothesis {
        id: "zero.aoe_all_in_range_dummy_v1",
        description: "Under the all-in-range dummy contract, declared scenario dummies receive Zero's ordered seven-packet Burst train across all living dummies in ascending target_index order at the primary impact time. Spatial geometry, recipient caps, distribution and falloff remain unresolved.",
    },
    Hypothesis {
        id: "zero.burst_a3_crit_rate_v1",
        description: "Zero A3 Anomalies Record adds 0.50 to ordinary crit rate for each native Burst packet, in addition to static, live, and gear crit rate; normal packet eligibility and the shared 1.0 crit-rate clamp still apply. The native 1046AwakenEffect Effect3 text and DT_Female051EffectFigure Female051_LV3 support the value; additive stacking is the tested interpretation.",
    },
    Hypothesis {
        id: "zero.burst_a4_damage_bonus_v1",
        description: "Zero A4 Undecided Factors adds min(compiled base ATK × DT_Female051EffectFigure Female051_LV4_2, Female051_LV4_3) to the ordinary damage-bonus zone for native Burst packets. The 1046AwakenEffect Effect4 text and Female051_LV4_1/2/3 figures bind the formula; assignment to the additive damage-bonus zone is an explicit tested hypothesis.",
    },
    Hypothesis {
        id: "zero.burst_all_in_range_v1",
        description: "Zero Burst delivers its ordered seven-packet train to every eligible scenario-declared dummy under the all-in-range dummy contract: by scenario definition all eligible dummies are in range and inside attack areas, so no position, radius, shape, or placement check gates delivery. Targets are delivered in ascending target_index order; for each target, UltraSkill1, UltraSkill2 four times, UltraSkill3, then UltraSkill4 (target-major order), all at the admitted primary impact time. Defeated or otherwise ineligible dummies keep their existing skip rules. This is a declared scenario rule, not a claim about native AoE geometry; native recipient caps/selection/distribution and packet spacing remain unresolved.",
    },
    Hypothesis {
        id: "zero.burst_packets_v1",
        description: "Zero Burst delivers native UltraSkill1 once, UltraSkill2 four times, UltraSkill3 once, and UltraSkill4 once, in that order, matching DT_GameplayAbilityTipsData.GA_Female046_UltraSkill.GALevelUpArray[0] and each native DT_SkillDamageData row. All seven packets use the admitted primary impact time and target 0, the simulator's current default target. Native packet intervals remain unresolved.",
    },
    Hypothesis {
        id: "zero.normal_chain_v1",
        description: "Five admitted ordinary inputs advance stages Melee1 through Melee5 with multiplicities 1,2,3,2,2 (ten hits). The chain loops, retaining native effective talent projection including resonance_3.",
    },
    Hypothesis {
        id: "zero.ordinary_hit_energy_v1",
        description: "For Zero character:1046 only, an ordinary Normal or Skill native packet grants its exact nonnegative DT_SkillDamageData.ChargeAdd once after positive actual HP loss against target zero. Fatal damage qualifies; zero HP loss, unavailable targets and cancelled packets do not. The living source owner receives 1x; each other living configured recipient receives the explicit source-element to recipient-element coefficient from frozen DT_SkillTeamChargeRelationData (the six recipient-element rows for Zero COSMOS are 0.6). Each recipient applies their own Charge Efficiency and exact fractional carry, then existing capacity and overflow accounting. Delivery-time owner and source attribution survive switching. Burst, QTE, non-native branches, additional targets and callbacks outside the modeled packet train remain unsupported. Eligibility, cadence, recipient and off-field rules are tested hypotheses because native caller evidence is unavailable; ChargeAdd and relation coefficients are frozen native rows.",
    },
    Hypothesis {
        id: "zero.skill_packets_v1",
        description: "Zero ordinary Skill delivers Skill1, Skill2, Skill3, and Skill4 once each from one admitted cast, matching the four ordered ratios in frozen neverness_zero and native DT_SkillDamageData rows. The admitted Skill fills this owner's Cycle pool to its existing capacity of 100 once, including semantic Hotori replay, independently of damage recipient count, ordinary energy and hit-generated channels. Existing charge is retained without duplicate grant or overflow. Cast-time fill is the selected interpretation of the published instant-Cycle Skill; native callback timing remains unverified. Children use the admitted primary impact time; animation intervals and travel are unresolved.",
    },
    Hypothesis {
        id: "combat.hp_cost_fixture_v1",
        description: "Explicit research hp_cost_events consume a caller-declared integer amount of the selected actor HP at the declared combat timestamp. Consumption bypasses shields and incoming-damage effects, is nonlethal (at most current HP minus one), and pays zero for a dead actor. Each positive settled consumption dispatches one HP-consumption callback regardless of amount; zero and clamped zero do not trigger. Events retain their source and HP before/after in typed receipts. At equal time, prequeued incoming damage settles before HP costs in authored order, and HP costs settle before newly admitted hit deliveries. This fixture does not establish native ability costs, affordability, timing or automatic dispatch; owner abilities must bind those separately.",
    },
    Hypothesis {
        id: "sets.devils_blood_v1",
        description: "Equipped Devil's Blood four-piece adds its source-bound base bonus to the wearer's ordinary damage-bonus zone. At packet delivery, any live status on the selected target carrying Ability.Player.Buff.Reaction.AnHun or Ability.Player.Buff.Reaction.HunXiang adds the conditional bonus once, regardless of source or stack count; both tags still grant only one extra bonus. Off-field deliveries retain wearer ownership and target statuses expire at their half-open deadlines. These evaluation and lifecycle rules are a bounded tooltip/tag model. Native GetReactionResultType handling and automatic Nova/Stain application remain separate shared-system work; True damage retains its existing bonus bypass.",
    },
    Hypothesis {
        id: "combat.live_break_damage_v1",
        description: "Each new modeled Break generation snapshots one elemental HP-damage contribution per living deployed actor in slot order. Select the exact DT_GlobalCommonData.UnbaldamagePara key at character level 1..80. Damage is base*(1+live BI/300+eligible Break bonus)*(original capacity/3)*DEF*source-element RES*eligible special factors, with exact intermediates and one nearest-integer HP settlement per contribution. Static source BI, eligible live break_intensity/def_ignore buffs, Film - MANISH and Black Tome wearer BI, and the strongest eligible Danzaburou same-element roster aura add within their zones. Danzaburou requires three matching deployed slots and a living carrier; duplicate auras do not stack. Daffodil has intrinsic Break bonus .20 and selected A6 DEF ignore .15; reached unbound Insight A4/A5 and R6 dependencies diagnose. Contributions snapshot together after Broken admission, retain causal command/generation and individual owners, do not crit or consume RNG, ignore ordinary ATK/damage bonuses and grant no ordinary energy/Cycle/gauge/proc callbacks. Target defeat invokes shared defeat callbacks and cancels remaining group deliveries; accepted snapshots survive later source death/field changes. Source selection, living eligibility, capture and callback policy are a bounded content model, not native consumer or game-timing proof.",
    },
    Hypothesis {
        id: "daffodil.resonance_skill_capture_v1",
        description: "Each admitted native-role party Support grants one persistent Daffodil Resonance stack, capped at two. Skill consumes and captures its pool at admission: per stack general Skill damage +0.50 and additive Break Efficiency +1.25, replaced by +1.00 and +2.50 at selected A1. Two stacks select six Skill3 packets and one Skill4 finisher (Echoes). Later Supports affect only the next pool; failed admission consumes nothing. This bounded owner model does not establish native animation timing or automatic Support eligibility.",
    },
    Hypothesis {
        id: "daffodil.insight_break_capture_v1",
        description: "Finale applies independent source-owned target Insight: one refreshed stack for 45 seconds, or up to two stacks with shared refreshed 9999-second expiry at selected A3. Self Eyes remains 30 seconds at every awakening. At a new target Break, selected A4 adds 0.15 Break bonus per own Insight stack and A5 adds one captured own contribution per stack. Capture uses this recipient and source only; marks survive field exit/source death, expire half-open, and clear on target death. Base roster contributions precede repeats in source-slot order; defeated recipients cancel pending contributions. This is a bounded owner model, not proof of native callbacks or montage timing; R6 Broken Chaos shred remains unbound.",
    },
    Hypothesis {
        id: "daffodil.broken_chaos_resistance_v1",
        description: "At each actual new Broken generation, a living deployed R6 Daffodil applies source-owned Chaos resistance reduction 0.15 to that recipient before Break HP capture, including from off field. The triggering ordinary packet used pre-Broken resistance. Each source has one refreshed status; its snapshot lasts exactly to the target Broken deadline, survives source field exit/death, and only existing statuses follow a Rose extension. Recovery and target death clear it; manual tags, absent bars and zero packet efficiency do not invent a Broken transition. This target debuff also affects other Chaos contributors and ordinary packets, with no effect on other elemental channels. The Broken-linked lifetime is a bounded owner model, not native callback proof.",
    },
    Hypothesis {
        id: "sets.diabolos_v1",
        description: "Equipped Diabolos four-piece subtracts its frozen Chaos ignore ratio from target Chaos resistance for the wearer's delivered Chaos packets, attacker/packet-scoped in the additive resistance-reduction zone and summed with live target resistance-reduction statuses before the resistance factor. Non-Chaos packets, Break damage settlement and other actors are unchanged; off-field delivery retains wearer ownership. The frozen reaction window (Chaos2_4_IgnoreChaos2 for Chaos2_4_CD seconds after the wearer participates in Nova or Scorch) remains unmodeled until reaction participation events exist. Attacker-side zone placement, wearer scope and the absent native trigger are bounded model hypotheses.",
    },
    Hypothesis {
        id: "sets.street_boxer_v1",
        description: "Equipped Street Boxer four-piece adds its frozen crit ratio to the wearer's additive crit-rate zone for every delivered packet with an ordinary crit draw, including off-field delivery. The frozen Remora/Stain window value equals the baseline (Lakshana_4_2 equals Lakshana_4_1 in CT_Equipmentbuff), so that branch changes no number in this model; its trigger and Lakshana_4_CD duration remain unmodeled until reaction participation events exist. Zone placement, wearer scope and the numerically identical powered branch are bounded model hypotheses.",
    },
    Hypothesis {
        id: "combat.support_dispatch_v1",
        description: "On each admitted switch to a different living actor whose owner has a bound Support entry, the engine automatically dispatches that owner's Support Skill once through the ordinary admission path, at the switch timestamp, using the owner's frozen switch-in timing (hit delay, recovery window, cooldown), zero energy cost and the effective talent coordinate (purchase plus owner resonance). Same-character swaps and switches whose incoming owner has no bound entry dispatch nothing. The dispatch is skipped with a trace receipt when the incoming actor is still in recovery, the Support is on its cooldown, energy is insufficient, or packet projection is unavailable in this mode; a skipped dispatch never rejects the switch itself. A dispatched Support sets the incoming actor's ordinary recovery lock and Support cooldown, queues its packets with owner attribution, and feeds the same team admission observers as a manual cast. During the Phase 2 fixture migration this dispatch executes only when the scenario explicitly accepts this hypothesis in its model assumptions; strict mode requires that acceptance regardless. Owner binding (currently Sakiri only), timing values, once-per-switch semantics and skip behavior are bounded model hypotheses from the reviewed owner card; native eligibility windows remain unverified.",
    },
    Hypothesis {
        id: "combat.esper_cycle_reactions_v1",
        description: "A switch admitted between two living actors whose native elements form an adjacent DT_ReactionData pair qualifies for an Esper Cycle when the outgoing actor's declared Cycle pool holds at least 100: admission spends exactly 100 outgoing Cycle, retains the incoming pool, and creates the pair's reaction on the living locked target zero. Ownership snapshots the higher-Cycle-Intensity participant (static authored cycle_intensity plus live cycle_intensity buffs; ties select the outgoing actor), capturing owner level, CI, element and crit damage at admission. Blossom creates one party-capped (3, oldest replaced, replaced buds spend their pending emissions) bud that emits five pistils at +2/+4/+6/+8/+10s, each exploding once on every living target with base(level)*(1+CI/600)*live DEF factor*live NATURE resistance, no crit. Scorch applies a 15s target status (ntesim.status.scorch and Ability.Player.Buff.Reaction.ZhouAn tags) ticking fifteen times at +0.99s multiples with base*(1+CI/600)*fixed-50 crit draw with captured crit damage*live DEF*live owner-element resistance; reapplication replaces the snapshot and refreshes expiry while retaining the existing tick phase; target death cancels remaining ticks. Nova applies a 5.1s target status (Ability.Player.Buff.Reaction.AnHun) and settles one Mental packet at +5.0s with base*(1+CI/600)*live generic resistance, bypassing DEF and crit; a qualified reapplication before the packet detonates the pending generation once immediately, and a fired generation never duplicates. Hexed, Stain and Remora pairs are recognized by the table but their follow-up effects remain unimplemented. Reaction damage never crits except Scorch's fixed draw, grants no energy/Cycle/Break, and settles through the ordinary HP/death path with receipts. Scenarios without a full outgoing Cycle pool, same-element or nonadjacent switches and dead locked targets stay ordinary. Switch-based admission, ownership capture, phase retention, refresh detonation, the declared Cycle input (no fabricated accrual) and the authored-CI input (no evidenced native attribute) are bounded model hypotheses from the reviewed cycle cards; native aura application/consumption, activation ICD and GetReactionResultType remain unresolved.",
    },
    Hypothesis {
        id: "character.zankou.vile_ash_dot_scheduling_v2",
        description: "The current canonical Zankou Bloodfeast variant applies five Vile Ash stacks at4216667us montage progress after same-time direct damage, to each eligible living declared dummy. Ordinary and free enhanced Inferno do not apply it. Each original source/target carrier caps at ten, lasts thirty World seconds and ticks once per World second strictly before expiry, without immediate execution. A live reapplication adds its captured stack count, refreshes duration and preserves the existing tick phase; an expired carrier starts a fresh phase with no old stacks. Pending old deliveries are cancelled before replacement. Its native coefficient is multiplied once by1+clamp(stacks-1,0,10); positive native expansion bypasses that additional stack term. Owner-specific spread notifications copy captured attached carriers, including to their donor, preserving ownership and independent caps; DoT ticks never recursively spread. Living sources continue off field, source defeat cancels pending deliveries and target defeat removes that recipient. Native talent and live source stats remain authoritative. These composition, finite montage and half-open refresh/expiry interpretations are bounded models, not recovered native executor playback.",
    },
    Hypothesis {
        id: "weapons.nest_bird_v2",
        description: "Each connected Ultimate packet (native DAMAGE_SOURCE_CATEGORY_Q) from a Tears Beneath the Mask wearer grants Warning Gaze to that packet's actual declared target after damage settlement. The per-source/per-target mark reduces outgoing damage by the frozen refinement ratio for twenty seconds, has one stack, and refreshes on another eligible hit. Delayed DoT deliveries retain their source, target and category after field exit; other targets and teammates receive no mark. The ratio and expiry are reported without inventing attacks for stationary dummies. This supersedes v1's target-zero limitation. Trigger classification, post-hit ordering and refresh are bounded modeled policies; native dispatch and Global release availability remain unverified.",
    },
    Hypothesis {
        id: "lacrimosa.nightmare_all_targets_v1",
        description: "Requires the separately accepted lacrimosa.nightmare_dot_scheduling_v1 model and extends it to all living declared dummies, including the uncapped Skill explosion. Each admitted Skill/Burst captures source, command, effective Normal-talent Blood coefficient and A0/A4 duration during admission, then applies five Nightmare stacks per living target in a ProcCallback at its primary impact time, after same-time HitDelivery. Dead sources apply nothing; target eligibility is rechecked at application. Each source/target retains an independent aggregate, expiry and NeverReset one-second grid; refresh cancels only that target's superseded ticks. Blood uses the effective Normal talent, including resonance, independently of Skill/Burst talent. Without this extension the legacy target-zero/cast-coordinate policy remains. The cap-20/remove-one-and-refresh model is retained despite the serialized cap-10 conflict; awakening variant/settlement/healing policies remain unresolved. Recipients, callback order and coordinate selection are bounded owner-model rules, not verified native dispatch.",
    },
    Hypothesis {
        id: "weapons.twinbirds_v2",
        description: "Crime and Punishment grants its wearer the frozen refinement Cycle Intensity as additive stat points while alive, including before the first cast and off field. Its permanent self-only cycle_intensity modifier adds with authored CI and other eligible live modifiers; Esper Cycle ownership and damage capture consume the resulting total. It never grants Cycle Energy and is not an ATK or damage percentage. The wearer's Psyche damage bonus and Ultimate-triggered fifteen-second shared team plus additional self critical-damage buffs retain the v1 rules, including refresh on retrigger. This supersedes v1's unapplied Cycle Intensity parameter. Trigger timing, recipients, source lifetime and stack/refresh rules are bounded modeled policies from the reviewed owner card; native dispatch and Global release availability remain unverified.",
    },
    Hypothesis {
        id: "sets.diabolos_v2",
        description: "An equipped Diabolos four-piece wearer ignores twelve Chaos resistance percentage points for its own Chaos packets. On a qualified engine Esper Cycle admission that produces Nova or Scorch, both living outgoing/incoming actors are participants independently of captured reaction ownership; each equipped wearer receives one self-only twenty-second window before Cycle spend and reaction production. During that window the frozen twenty-four-point ignore replaces the baseline twelve, rather than adding to it. Further eligible participation refreshes the nonstacking window; field exit preserves its world-clock deadline and exact expiry restores the baseline. Ineligible switches, same-actor swaps, other reactions, nonwearers and non-Chaos packets receive no powered benefit. Participant effects are prepared atomically before publishing buffs/receipts. This extends the switch-based reaction model and supersedes v1's unconditional-only implementation; native participation dispatch and clock fidelity remain unverified.",
    },
    Hypothesis {
        id: "combat.stain_switch_v1",
        description: "Extends combat.esper_cycle_reactions_v1 to qualified Psyche/Lakshana switches: spend exactly one hundred outgoing Cycle and apply one twelve-second Stain status to living locked target zero. Reapplication replaces its expiry without stacking; its captured applier may leave the field or die without removing the status, while target death clears it. Each elemental Psyche/Lakshana packet against that target multiplies a separate Stain zone by (6/5)*(1+(1/5)*CI/(CI+180)), using the damage actor's live authored plus buff CI, independently of the pair winner. Generic damage/final bonuses and crit retain their own zones; Mental Nova, other elements and other targets receive no Stain factor. Stain creates no independent damage packet, crit draw, energy or Break grant. The modeled status supplies the existing HunXiang target tag for equipped predicates without certifying the native reaction getter. Duration, switch admission, live capture and the selected factor are reviewed bounded owner-model decisions; native dispatch and owner-specific Stain enhancements remain unresolved.",
    },
    Hypothesis {
        id: "combat.remora_charge_v1",
        description: "Extends combat.esper_cycle_reactions_v1 to qualified Lakshana/Cosmos switches: spend one hundred outgoing Cycle and apply one refreshed nonstacking five-second Remora to living locked target zero. Its accepted duration survives field exit and source death; target death clears it. Stationary dummies have no movement or attack-slow DPS behavior. Each continuing Blossom pistil that hits at least one living target with active Remora grants ten Ultimate Energy once to the currently active living actor, through that recipient's Charge Efficiency, exact fractional carry and cap/overflow ledger. Multi-target hits do not multiply grants, and no qualifying target or defeated active recipient grants nothing. Charge does not consume Remora or the bud, create damage/critical draws, distribute to peers, or grant Cycle/Break. The fifth emission qualifies normally. Slow decay, owner-specific Remora extensions and native dispatch fidelity remain outside this bounded stationary-target model.",
    },
    Hypothesis {
        id: "sets.street_boxer_v2",
        description: "Equipped Street Boxer four-piece adds the frozen baseline crit ratio once in its wearer's additive crit-rate zone. Any qualified team Esper Cycle producing Stain or Remora gives each living equipped wearer a self-only twenty-second nonstacking window, including off-field wearers who are neither pair participant nor reaction owner. Reapplication resets expiry; field exit preserves the world-clock deadline. The window's frozen Lakshana_4_2 crit ratio replaces Lakshana_4_1, never adds to it, and expiry restores the baseline. Both frozen ratios equal fourteen percent; the causal window and native parameter receipts remain explicit despite unchanged current damage. Other reactions and ordinary/unqualified swaps do not refresh it; nonwearers get no bonus. All team equipment effects are prepared atomically before publication. This supersedes v1's unconditional-only scope; native dispatch/capture and clock fidelity remain unverified.",
    },
    Hypothesis {
        id: "character.zankou.crimson_reverie_v1",
        description: "Zankou Crimson Reverie sets her living owner's Cycle pool to exactly one hundred once at scenario combat entry, replacing the authored starting value without adding Ultimate Energy. Field returns, waits and continuing cycles never repeat that initialization. One permanent nonstacking self modifier adds one hundred Cycle Intensity stat points to Zankou on or off field; teammates receive none. It composes with her authored and equipped CI before live/captured reaction consumers. An initially defeated owner receives no entry Cycle. Current player Passive2 configuration and its self carrier establish the recipient correction from the earlier team-aura shortcut. The thirty-second entry cooldown has no second combat entry to govern in this single-combat model.",
    },
    Hypothesis {
        id: "character.zankou.silent_sunset_v1",
        description: "With a living team Zankou, qualified Scorch uses one source-preserving target aggregate capped at three stacks. Initial application gives one stack; further qualified applications and each connected ordinary DoT delivery on that live enhanced target add one, cap at three, and refresh fifteen seconds without resetting the .99-second tick phase or adding an immediate tick. A DoT-triggered replacement is a newly inflicted Scorch by Zankou: all existing stacks take his live level, Cycle Intensity, Incantation element and captured ordinary plus DoT crit damage, using the shared Scorch base and fixed fifty-percent crit policy. It does not copy the ally's applied HP damage or its DoT coefficient/duration. This selects the description's newly inflicted effect as the new Scorch and the native carrier's fifteen-second/.99-second clocks. Replacement receipts retain the original switch and link the exact earlier triggering hit separately from the new damage owner. Scorch's own ticks cannot trigger this effect; zero HP-loss connected DoT hits may, following the reached callback's absent positive-damage gate. Dead Zankou cannot create replacements; existing captured Scorch remains independent. R6 kill enhancement, native prediction/callback fidelity and unbound ordinary Skill application remain outside this model.",
    },
    Hypothesis {
        id: "character.zankou.heartwrench_skill_v1",
        description: "The bounded ordinary Zankou Skill input uses the selected Demon Strike lead Skill3 and cleave Skill3_2 on every living declared in-range dummy, with the cleave four hundred milliseconds after the admitted primary impact. Public completed-cast defaults override generic caller timing with impacts at .15/.55s, recovery at .8s and cooldown readiness at12.12s (.12s notify plus12s); raw admitted timings remain authored. Each direct packet has one energy group and grants native ChargeAdd once from its first positive HP-loss recipient, even when target zero is dead or immune:1.101 plus13.877 per cast, using existing source/peer efficiency, carry, capacity and overflow. After the living source's cleave, apply one ordinary Heartwrench stack to each then-living dummy. One unenhanced application is an explicit inferred count; enhanced/form-dependent variants and interruption transitions remain unbound. Heartwrench uses GE_Player_Zankou_DotDamage and its independent effective Normal talent, never the applying Skill coordinate. Its own source/target aggregate caps at ten, lasts thirty seconds with half-open expiry, has no immediate execution and ticks every second; refresh replaces the pending delivery and deadline while preserving the original next-tick phase. Expired stacks do not carry. The separate Burst Vile Ash class retains its existing five-stack/all-target/restarted-phase/inclusive-endpoint model without sharing stacks. Both trains read live owner/target damage stats, continue off field while the source lives, and cancel after source or target defeat. Periodic zero-damage deliveries continue their clock without connected-hit equipment or energy procs. Real connected DoT deliveries feed the bound Silent Sunset model without recursive Scorch. Fixed critical probabilities and packet bindings remain separately declared metadata assumptions.",
    },
    Hypothesis {
        id: "combat.support_dispatch_v2",
        description: "On each admitted switch to a different living actor with a bound Support entry, the engine automatically attempts that owner's Support Skill once through ordinary admission at the switch timestamp. Research mode records this reached model without a scenario opt-in; strict mode requires exact acceptance before any Support admission mutation. Missing/conflicting evidence, recovery, cooldown, insufficient resources or unavailable packet projection skip the Support with a trace receipt and never reject the switch itself. Successful dispatch uses effective purchased talent plus owner resonance, zero energy cost, and the owner's frozen switch-in hit delay, recovery and cooldown; currently Sakiri .1s/.6s/15s. It sets the ordinary incoming recovery and Support cooldown, queues owner-attributed packets, and reaches the same team admission observers as a manual Support cast. Same-character swaps, defeated incoming actors and unbound owners dispatch nothing. Explicit and implicit switches share this path. No readiness polling, automatic waits or new rotation command is introduced; subsequent ordinary actions must respect Support recovery. This supersedes the v1 fixture-migration opt-in gate; owner eligibility/timing remain bounded reviewed-card models and native eligibility windows remain uncertified.",
    },
    Hypothesis {
        id: "weapons.police_rat_v3",
        description: "Justice Executioner retains its frozen unconditional live base-ATK-scaled ATK percentage and additive Boss damage bonus only when the dummy's authored is_boss flag is true. Missing is_boss uses the runner's false default only; this does not classify a target as non-Boss in the native game. No native enemy identity or Boss classification is inferred. Explicit living active-wearer Arc admission summons Officer Whisker for thirty seconds with a sixty-second cooldown. One Physical ATK hit is delivered every second after admission, with no immediate hit and an inclusive deadline. Each attack selects one living in-range dummy: keep the previous recipient while alive, otherwise choose the lowest living target index at the summon tick; no eligible targets ends the train because target revival is unmodeled. This single-recipient deterministic selection is an explicit interpretation of the tooltip's per-hit attack, not certified native target acquisition. Same-time target defeat after selection can cancel that delivery without a second selection. Each train preserves its actual admitted Arc command, and renewal supplies a new generation and parent. Damage reads live wearer/target stats, continues off field and stops on wearer defeat; already queued owner packets cancel postmortem. Resolved input-role provenance uses this same accepted weapon model. This supersedes v2's fixed primary-target and command-zero execution limits while retaining its stat zones, periodic Physical model and unverified Global release availability.",
    },
    Hypothesis {
        id: "zero.support_packets_v1",
        description: "Zero Support QTE delivers the frozen Cosmos/category-H QTE1 and QTE2 packets once each, with the second impact 0.250 seconds after the admitted first impact. Engine entry timing is 0.100/0.350 seconds, 0.600-second recovery, from the reviewed Zero card's bounded timing model rather than measured native montage seconds. Both uncapped packets hit all living script-authored stationary dummies in range and read live owner/target stats; each packet grants its frozen 2.899/4.801 raw energy once across recipients on its first positive hit, without granting Cycle. The purchased Support talent coordinate applies to both packets; the frozen Zero resonance row upgrades Normal/Skill/Burst only, so it grants no QTE level despite the older card summary. Child projection uses the actual admitted action coordinate even when the source command is a switch or another implicit action, preserving the original command and Support switch origin. Zero A5's existing ten-percent base-ATK buff is granted once on admission before both impacts. Switching alone preserves launched follow-ups; source defeat cancels pending owner hits, while defeated targets simply miss that packet. Exact target caps and native cast/eligibility timing remain unverified; this is an explicit all-in-range dummy model, not geometry or enemy identity.",
    },
    Hypothesis {
        id: "combat.support_dispatch_v3",
        description: "Retains v2 automatic Support attempts on an admitted switch to a different living bound owner, with exact strict-policy acceptance, no opt-in, no hidden waits and skips that never reject the switch. Bound entries are Sakiri's single impact at .100s and Zero's two impacts at .100/.350s; both have .600s ordinary recovery and fifteen-second Support cooldowns. Owner packet models and every required follow-up coefficient/metadata projection are preflighted before Support admission changes resources, cooldowns, buffs, packets or observers. A later owner/gear observer failure restores the pre-Support state, queue, causal sequence, trace and admission ledger while preserving the already admitted entry switch. Effective purchased Support talent plus resonance is carried by the admitted native context rather than inferred from the original source switch/implicit input command. Explicit and implicit entries share the same owner packet path and preserve the original source command; explicit switches supply incoming Support provenance. Team/gear observers run once per admitted cast. Same-character swaps remain no-ops; recovery, cooldown and projection/evidence failures produce a skipped-attempt trace. This supersedes v2's Sakiri-only binding and preserves its historical description. Eligibility and owner timing remain disclosed reviewed-card models rather than measured native gameplay windows.",
    },
    Hypothesis {
        id: "sakiri.support_packets_v1",
        description: "Sakiri Squash Support delivers one frozen Incantation/category-H QTE1 packet at the admitted .100s impact, with the reviewed .600s recovery. The saved public Squash description explicitly deals damage to an area; with no recovered cap, the bounded stationary-dummy model hits every living in-range target. Live wearer/target stats and purchased Support talent apply; the frozen resonance row upgrades Normal/Skill/Burst only, so QTE receives no inferred level bonus. Native ChargeAdd is 10.900 and UnbalValue 2.500, superseding the older card summary's 7.500 energy and Cosmos label. A shared packet group grants the native raw energy once on its first positive recipient, even when target zero is defeated or immune; living source receives 1/1 and living teammates 3/5 with independent efficiency, carry, capacity and overflow. QTE grants no Cycle. Switching alone retains launched packets and source ownership; source defeat cancels pending hits and grants. The owner model is preflighted before automatic admission. Cast/eligibility timing and uncapped all-in-range recipient selection remain disclosed reviewed-card models rather than measured native geometry or caps.",
    },
    Hypothesis {
        id: "haniel.harmony_ensemble_v1",
        description: "Haniel Hootie and its team ATK aura start in a callback after both direct Skill impacts at the admitted hit time, including an immune or absent primary target fallback. Recasting direct hits can feed an existing Hootie before refresh. Exact native Skill and Burst montage admission remains unbound: admitted impact timing is used and Ace starts on Burst admission, not measured native montage progress. This supersedes v1's before-impact Skill aura grant. Within that bounded clock, one eight-ObjectLifetime-second Hootie (twelve with A2) persists off field and preserves its remaining lifetime through ten-second Ace and recasts. Positive direct packets from the living active actor grant one source-wide Harmony at most every .5 World seconds, including a killing hit, excluding reaction, gear, summon and token damage. Four Harmony consume four and cause one live Psyche/category-E Ensemble across all living stationary dummies, with the captured admitted Skill talent coordinate and current owner stats. Ensemble grants one persistent Plot Armor (two in Ace), capped at four or six with A1. Each Ace Ensemble additionally emits one Psyche/category-Q Symphony and replaces a three-second single-use extra-attack token. The next positive active direct hit on a still living recipient consumes the prior token before Harmony processing; its Haniel-owned Q packet uses the admitted Burst coordinate and current stats, doubles its coefficient only when A4 Haniel herself triggered the arming Ensemble, and does not recurse. Tokens survive Ace ending and follow the active actor. Expiry resets Harmony but preserves Plot; source defeat clears summon, Ace, tokens, Plot and source buffs. Same-time AoE and pellets share the World .5-second gate; no periodic summon polling or Cycle/energy is invented for the three native zero-ChargeAdd rows. Cannon consumption and exact native admission/impact timing remain separate unbound mechanics; this is executable summon coverage, not a claim of the complete Haniel kit.",
    },
    Hypothesis {
        id: "iroi.live_atk_healing_v1",
        description: "Iroi ordinary Skill and the currently bound one-time Burst healing scale their frozen 13-level .943/1.413 starting coefficients from live source final ATK, superseding the historical MaxHP model. The accepted owner card's recovered GetAtk calculator owns this stat choice; the native coefficients are unchanged. Read compiled source ATK plus base-scaled live ATK percentages and flat buffs, including applicable equipped passive stats, at each real recipient callback without creating a damage packet or applying damage/crit/target modifiers. Apply source outgoing and recipient incoming healing bonuses, including selected A3 exactly once, then round gross once and clamp to missing HP. Each living recipient gets its actual settlement and positive gross, including overheal, may trigger compatible equipment. Dead sources and recipients do not heal or trigger listeners. Effective admitted talent coordinates are preserved. Skill's existing callback-at-admission and Burst's existing post-impact callback timing remain explicitly bounded caller models; native montage dispatch, QTE/A2 healing, Dream periodic healing, Moon gains and application-level B4/A5 dispatch remain separate queued bindings. This does not certify a complete Iroi kit.",
    },
    Hypothesis {
        id: "nanally.support_packets_v1",
        description: "Nanally Support delivers one frozen Anima/category-H QTE1 at .666410 seconds after accepted entry and retains the parent reaction-qualified switch. Rate-one impact, distinct switch/next-ability gates, delayed positive-contact hitstop and cancellation execute through action_clock_v1: nominal recovery1.108275s becomes1.308275s after contact, switch1.083416 becomes1.283416, while a miss retains nominal recovery. Reaction qualification has no Support cooldown. The uncapped all-in-range dummy model hits every living recipient with live owner stats, purchased Support talent and no R3 Support upgrade. Native7.000 charge is granted once on the first positive recipient per packet, with living teammates at the existing .6 relation, exact efficiency/carry/cap/overflow and no Cycle. A living current Underboss generation also emits one primary-recipient Cat QTE packet at that admitted impact under the existing generation/death/expiry policy, using its captured effective Burst coordinate independently from Support talent. The Cat packet keeps the actual Support parent, native GA_Nanally_UltraSkill and category A; its1.5-starting coefficient is not another Support or Ultimate packet and grants no ordinary energy/Break/Cycle. Existing coordinated A1/A5 callbacks remain owner rules. Interruption cancels unissued owner packets/unlaunched Cat groups; switching preserves already launched independent children and the summon generation. Native caps, measured playback and exact summon travel remain uncertified.",
    },
    Hypothesis {
        id: "combat.support_dispatch_v4",
        description: "Retains v3 automatic entry attempts, strict exact model acceptance, actual admitted coordinates, projection preflight and complete rollback on later observer failure while preserving the admitted switch. Adds Nanally's one H packet and conditional existing Underboss QTE to Sakiri and Zero. Nanally impact/recovery is .666410/1.310 seconds; all three bound entries retain the reviewed fifteen-second Support cooldown. This supersedes v3's two-owner binding limit without changing same-character no-ops, ordinary recovery, no hidden waits or visible skipped-attempt diagnostics. Timing, uncapped dummy selection and switch eligibility remain declared owner models, not native-game certification.",
    },
    Hypothesis {
        id: "weapons.gold_record_v2",
        description: "Voice of the Voyager retains its frozen base-ATK-scaled ATK percentage and source-owned critical bonuses. Support critical damage applies only to actual Support input packets with native category H; Ultimate critical stacks apply only to Ultimate input packets with native category Q. Explicit accepted research packets without a native category retain their declared input-role boundary. A category-A summon child does not inherit H/Q bonuses or Support triggers merely from its initiating action. Each positive actual HP-loss Support delivery grants one wearer Ultimate-critical stack through ProcCallback, up to three; separate AoE recipients may each grant a stack because the tooltip specifies no ICD. Zero rounded HP loss, cancelled/immune packets and non-Support children do not grant. Repeated triggers reset one three-second world-time instance, preserved on switching; expiry precedes equal-time deliveries. New source-bound Nanally entries and continuing cycles exercise T1-T5, including captured-Burst Cat QTE ownership. This supersedes v1's input-role-only packet filtering and connected-but-zero-HP trigger boundary; native target caps and release availability remain unverified.",
    },
    Hypothesis {
        id: "weapons.jiaojuan_v2",
        description: "Film - MANISH retains its frozen unconditional wearer-only Break Intensity points and Broken-only additive ordinary damage bonus. Unlike historical v1, both have live content-simulation consumers: actual native UnbalValue depletes the authored dummy profile, Broken expiry restores its original capacity, and each living source contributes Break damage with its own live intensity. Weapon BI adds in 1 + total BI/300 + Break bonus, never gauge efficiency or ordinary ATK/damage zones. Its Broken ratio joins the ordinary damage-bonus sum only while that recipient's half-open Broken status is active; the hit that depletes the bar precedes application. It does not multiply Break damage or another source's contribution, create stacks or an ICD, or reset on switching. T1-T5 three-cycle Edgar/Mint sources preserve actual energy/carry and compare native/WASM/Worker reports, aggregation and replay. Gauge units, Break formula and all-in-range dummy assumptions remain the existing declared combat models; this does not certify native game timing or Global release availability.",
    },
    Hypothesis {
        id: "weapons.kite_v2",
        description: "Watch Your Heads retains v1's source-owned fifteen-second nonstacking, nonrefreshing Redirect Skill ATK window. Its native Con_Kite target branch uses HasAnyMatchingGameplayTags on HunXiang/GuangXiang: a recipient carrying either modeled Stain or Remora qualifies, not necessarily both. During the wearer window, Lakshana packets add one frozen refinement damage ratio to the ordinary additive zone; dual tags do not double it. Eligibility reads each actual target and half-open status lifetime at delivery, including off-field packets. Untagged targets and other elements receive only the live base-ATK-scaled ATK window; expiry removes both effects. This supersedes v1's unconditional unsupported error for Lakshana because both target statuses now have live shared consumers. Source evidence is the recovered conditional custom-application carrier and Con_Kite target tag return path. The separate native source GetReactionResultType fallback accepting ordinals5/6 and exact application-time capture remain unverified: this bounded executable model uses current target statuses and does not fabricate a permanent source reaction classification. Existing qualified-switch resource costs and status clocks remain the declared combat models. T1-T5 continuing Skia sources compare native/WASM/Worker reports, aggregation and replay.",
    },
    Hypothesis {
        id: "weapons.koinobori_v2",
        description: "A Time Will Come retains v1's deployed-party threshold of three distinct Esper elements, independent of field ownership. ATK/DEF/MaxHP percentages each scale their native base stat, summing with existing percentages instead of multiplying the compiled total. MaxHP settles once in build projection; live ATK is shared by damage and healing. This revision also applies the same packet-independent DEF contribution to Adler Blessing's existing admission snapshot and its separate A4 live overflow mitigation; it does not create damage packets to query source stats or alter existing shield/mitigation timing. Living-source buffs and flat DEF remain separate source inputs. The shield snapshot persists through switches and later DEF changes, while A4 reads live source DEF at actual depletion and excludes dead granters. Team element qualification uses authored party composition rather than living-count changes. T1-T5 continuing native Adler Skill/Normal sources and independent shield/overflow checks cover qualified and unqualified teams without resource resets; native/WASM/actual Worker preserve complete reports, aggregation and replay. Native Blessing party interception and remaining owner lifecycle branches retain their existing explicit limits; this does not certify the full Adler kit.",
    },
    Hypothesis {
        id: "chiz.normal_grain_v1",
        description: "Each admitted Chiz Normal stage grants its level-selected Grain once at its first positive actual HP-loss packet. Native child pulses and additional recipients retain the parent command identity and cannot grant again, including after later inputs or field exit. Zero rounded HP loss and cancelled delivery grant nothing; a later positive packet of that same input remains eligible until the first grant. At level 80 each of five ordinary inputs grants 450, totaling 2250 for twelve damage packets; selected A4 multiplies each grant by 1.2, totaling 2700. Selected A1/A5 capacity and initial balance remain independent. Grain is capped combat currency, separate from party Fons and Ultimate Energy. Acquisition cadence and positive-HP eligibility follow the reviewed Chiz collection model; remaining Branch, held Skill, loans and market execution are outside this slice.",
    },
    Hypothesis {
        id: "weapons.good_boy_v2",
        description: "Good Boy retains the admitted-Ultimate, twenty-second, nonstacking team base-ATK-percentage window. Its frozen T1-T5 unconditional Charge Efficiency bonus of 18/21/24/27/30 percentage points now adds once to the wearer's existing received-energy multiplier, rather than multiplying it or changing raw source generation. The wearer keeps this stat off field; other recipients retain their own multiplier. Ordinary energy, external team grants and weapon callbacks use the same fractional carry, capacity, overflow and spending ledger without a cycle reset. The static build compiler consumes this bonus once into the existing generic received-energy multiplier; explicit legacy TeamAtk-only procs do not synthesize a static build bonus. This closes the previously omitted unconditional charge consumer; the separate additional ATK condition for an Ultimate actually controlling a recipient remains unimplemented and is not inferred from damage, airborne input or boss classification. Native control admission and ordering remain separate owner work.",
    },
    Hypothesis {
        id: "combat.control_effects_v1",
        description: "A content owner may request a finite control effect after a connected delivered hit. The anonymous dummy's authored control_immune flag defaults false and is independent of Boss classification, airborne inputs and damage magnitude. A living source can control a living nonimmune recipient; a defeated source/target or immune recipient produces no successful-control receipt or status. Positive exact hits that round to zero HP loss still reach after-hit control, distinct from positive-HP-loss consumers. Successful applications preserve source actor, actual target and parent command, record exact world-time duration/expiry and apply one target-scoped status. Control represents gameplay eligibility only, without movement, geometry or target AI. Source death does not erase an already applied finite target effect. Native immunity and after-hit ordering remain explicit scenario/owner hypotheses.",
    },
    Hypothesis {
        id: "sakiri.suppress_v1",
        description: "Sakiri's first connected Ultimate packet per admitted input and actual recipient attempts the source-bound Suppress effect once. Child pulses and other recipients retain their causal command identity; switching does not rearm an earlier attempt. Suppress lasts 3 World seconds, or 6 with selected A1, using Sagiri_Suppress and Sagiri_lv1_Suppress. Only successful control applies the Impish Trick child DEF reduction: one shared target instance subtracts 10 percent of authored base DEF for 20 seconds, independently of the shorter control lifetime. If base_def is omitted, authored total DEF is also the assumed base; an explicit base 1000/total 1500 yields 1400, not 1350. The reduction refreshes at the next successful control and clamps effective DEF at zero. These choices bind the recovered SKILL_EVENT_AFTER_HIT -> suppress object -> control marker and DEF-child chain, not every damage input or mere preexisting airborne state. Native control immunity, hybrid-server routing, exact application/capture clocks and passive unlock dispatch remain unverified; the reviewed baseline enables the passive. Other floating/Skill control and A5 transitions retain their separate owner limits.",
    },
    Hypothesis {
        id: "weapons.good_boy_v3",
        description: "Good Boy retains the once-compiled additive Charge Efficiency from v2 and its admitted-Ultimate shared team ATK window. A successful control receipt produced by that wearer's same admitted Ultimate upgrades the single active named window from its frozen base ATK ratio to base plus the frozen 6/7/8/9/10 percent refinement ratio. The extra ratio latches once per cast across recipients and survives the control status ending; it lasts only the remainder of the original 20-second world-time deadline. It never stacks per target, refreshes on repeated hit/control callbacks, or extends from control time. Expired windows, stale commands and an older wearer's control after another wearer replaced the shared snapshot cannot upgrade it. Ordinary Ultimate damage, mere airborne state, Boss metadata, other actors and non-Ultimate control are insufficient; the engine's actual successful-control receipt is required. This supersedes v2's missing-control-consumer limit under the explicit combat.control_effects_v1 model, currently produced by Sakiri Suppress. Native conditional capture, original application ordering and cross-wearer priority remain tested product assumptions. Legacy explicit TeamAtk-only procs default their omitted control ratio to zero; native projectors bind it at every refinement.",
    },
    Hypothesis {
        id: "jiuyuan.natural_pact_v1",
        description: "Jiuyuan Normal5 and Skill contact plant one source-target Rose Pact without refreshing an existing Seed or Bud. Seed lasts five World seconds and Bud a further twenty-five, with one live-ATK native Normal-talent Attachment packet per World second, including both phase endpoints, and one native BudEnd settlement after the thirtieth tick. No tick executes on application or conversion. Selected A2 doubles only periodic coefficients. The Pact continues off field and stops after source or target defeat; no death settlement is emitted. Natural settlement retains the target attachment for its two-tenths-second closing interval. Current direct-action clocks and Normal5 target-zero policy remain the existing admitted models. Burst force-growth/active settlement, powered-shot consumption, ammo and reaction planting remain unbound separate branches. Native periodic endpoint and contact ordering are checked product assumptions, not observed native callbacks.",
    },
    Hypothesis {
        id: "weapons.reality_refuge_v2",
        description: "Reality Refuge adds its frozen Anima ratio to wearer Nature packets and its frozen Attachment ratio to wearer Attachment packets in one additive damage-bonus zone. Jiuyuan natural Seed/Bud and BudEnd now supply native Attachment packets through jiuyuan.natural_pact_v1, retaining live wearer ATK, Normal talent, source-target identity and continuing clocks. Successful wearer Ultimate admission refreshes one six-second self window replacing the base Attachment ratio with the refinement Ultimate value; bonuses do not stack, and off-field delivery samples the current window. Exact expiry precedes an equal-time hit; another actor's Ultimate cannot grant or refresh the wearer window. Native cast/clock semantics remain checked assumptions, and unbound active Pact settlement and character branches remain separate limits. This supersedes v1's inert-Attachment roster limit without certifying native callback execution or Global availability.",
    },
    Hypothesis {
        id: "combat.stamina_v1",
        description: "An explicitly declared party-wide combat stamina profile owns exact capacity, starting amount, multiplicative use efficiency and recovery units per World second. An admitted dash spends 15 times use efficiency, recovers after a refreshed two-second spending pause and has a modeled 0.5-second recovery and 0.9-second actor cooldown. Recovery integrates analytically on paused World time, continues off field and clamps to capacity; no periodic polling, cycle refill or account default is synthesized. Rejected inputs do not spend, reset the recovery pause or grant successful Critical Dodge callbacks. Other stamina-consuming owner actions and native recovery-carrier applicability remain separate work.",
    },
    Hypothesis {
        id: "nanally.critical_riposte_v1",
        description: "Nanally's explicitly successful paid Critical Dodge arms one following Normal input as a native category-Z PerfectEvadeAttack using effective Normal talent. Waiting preserves the opportunity; an admitted other action or field exit clears it. Native GA_Nanally_ExtremEvadeAtk remains the packet binding beneath the authored Melee input. Finite action_clock_v1 places impact at267562us, nominal recovery841593us and positive-contact recovery1138593us, adding one .3*(1-.01) pause regardless recipients. Jump/Skill may cancel from zero progress; unissued owner packets and unlaunched Cat groups are removed without refunding the paid dodge or consumed permission. All living in-range dummies receive the native packet, with4.2 raw Ultimate charge once per positive packet group through the existing recipient/efficiency/carry policy. A current Underboss generation also supplies its one native Riposte child with captured Burst talent and existing admitted coordination/lifetime policy. This Critical Riposte does not itself assert a successful Parry or activate Parry-only equipment. Cycle accrual, measured playback and exact Cat travel remain separate limits.",
    },
    Hypothesis {
        id: "mint.riposte_v1",
        description: "Mint Critical Riposte replaces Normal input upon an active paid Critical Dodge opportunity, delivering one frozen Anima/category-Z GE_Player_Mint_PerfectEvadeAttack_Damage packet with live ATK scaling and Normal talent coordinates. Impact is at227216us. With ordinary_clock_v1, nominal recovery443505us receives300000us of connected actor pause at rate.01 once across recipients, giving740505us recovery; zero HP loss adds no pause. The caller-timed packet profile retains its explicit740505us recovery. Upon positive target HP loss,4.4 raw energy is granted to active Mint with0.6 distribution to living teammates. A1 Task Force Operation removes running Skill cooldown once per hit opportunity and applies a3.0s non-refreshing self guard. Defeat cancels pending damage.",
    },
    Hypothesis {
        id: "mint.plunge_v1",
        description: "Mint Plunge landing delivers one frozen Anima/category-X GE_Player_Mint_AirAttack1_Damage packet with live ATK scaling and Normal talent coordinate. Impact is64514us after declared landing, nominal recovery333451us and positive HP loss grants3.3 raw energy. With ordinary_clock_v1, a new admitted Jump opens one flight, plunge declares positive landing delay and height bonus0..1, and coefficient scales by1+bonus. No descent packet, extra plunge cost, inferred geometry or implicit landing is added. Before declared touchdown, another grounded input requires an explicit interrupt or field exit; this conservative flight admission is a selected model. The caller-timed packet profile uses its declared fall_damage_bonus.",
    },
    Hypothesis {
        id: "mint.hold_attack_v1",
        description: "Mint held Basic executes Hold Opener with three GE_Player_Mint_Evade1_Damage and one GE_Player_Mint_Evade4_Damage packet, followed by Hold Spin delivering up to thirty GE_Player_Mint_Evade5_Damage packets with live ATK scaling and Normal talent coordinate, granting per-packet ChargeAdd on positive target HP loss. With ordinary_clock_v1, preserve reached Normal prefix contacts and section-local hold thresholds, then use the selected finite source coordinates with owner-local hitlag. Each spin opportunity atomically pays2 times declared stamina use efficiency even on a miss; an unaffordable opportunity enters End without a partial debit or damage. Wind suppresses only account stamina recovery from spin entry until exit, preserving other gates and the real2s spend deadline. Release, natural5.41 progress expiry or exhaustion cancels future montage packets and enters the616639us End section once; an admitted replacement, field exit, interruption or death clears the hold without refunds. Explicit native tenacity_events apply the selected.55 real-second player cue: actor progress combines.01 dilation with contact pauses by minimum rate, while only SkillTriggerEnd pauses fully. Each application retains its own expiry; one ending source cannot unpause another. Replacements cancel the named hold timer but retain a still-live cue and its actor slowdown; real resources, buffs, cooldowns and the DPS clock continue. This is the selected cue applicability/clock inference, not automatic incoming stagger or measured game timing. Native entry/approach and declared plunge prefixes retain their own450000/530000us hold thresholds. A pending landing hold keeps default source completion beyond ordinary plunge recovery while explicit native gates can still cancel it; End retains remaining caused pause. Native source inputs preserve these same durations and payments.",
    },
    Hypothesis {
        id: "mint.passive2_entry_def_v1",
        description: "Mint Passive 2 applies +20% DEF on field entry (including initial active assignment) while at breakthrough 4 or higher. The buff persists while on field and is removed immediately on field exit or source death. Initially benched Mint does not receive the buff.",
    },
    Hypothesis {
        id: "sakiri.critical_riposte_v1",
        description: "Sakiri's explicitly successful paid Critical Dodge arms one following Normal input as a native category-Z PerfectEvadeAttack using effective Normal talent. Waiting preserves the opportunity; an admitted other action or field exit clears it. Native GA_Sagiri_ExtremEvadeAtk remains the packet binding beneath the authored Melee input. The conservative rate-one source clock places impact at 180000us and recovery at 750000us; no early cancel or measured animation fidelity is claimed. All living in-range dummies receive the native packet, with 5.4 raw Ultimate charge once per positive packet group through the existing recipient/efficiency/carry policy. Defeat cancels pending damage.",
    },
    Hypothesis {
        id: "iroi.critical_riposte_v1",
        description: "Iroi Critical Riposte consumes perfect evade window upon Normal input, delivering GE_Player_Oneiroi_PerfectEvadeAttack_Damage with live ATK scaling and Normal talent coordinates, granting 4.5 raw energy on positive target HP loss.",
    },
    Hypothesis {
        id: "iroi.plunge_v1",
        description: "Iroi AirAttack plunge landing delivers GE_Player_Oneiroi_AirAttack_Damage with live ATK scaling and Normal talent coordinates, granting 3.5 raw energy on positive target HP loss.",
    },
    Hypothesis {
        id: "iroi.daydream_branch_v1",
        description: "Admitted held basic attack costs 20 Moon outside Dream (fallback to Normal1 if Moon < 20). Daydream charge stages emit Branch1, Branch2, Branch3. Lucid Dream emits up to 35 Branch5 projectiles costing 10 Moon once.",
    },
    Hypothesis {
        id: "iroi.dream_state_v1",
        description: "Burst admission captures Moon >= 30 as Ivory (direct UltraSkill damage and pulses) or Horn (healing only). Initial pause release at 4.0047474s starts 8 Moon/s drain, blocking all Moon gains. Every 1s provides periodic team heal and Ivory pulse until Moon reaches 0.",
    },
    Hypothesis {
        id: "iroi.regression_v1",
        description: "Ally defeat or held Transform regresses other allies into lambs (Gun, Syringe, Trumpet) using Iroi base ATK on native cooldowns. Ivory Dream permits one temporary revival.",
    },
    Hypothesis {
        id: "iroi.bud_copy_b2_v1",
        description: "When an eligible parent Vita Bud reaches 3s age, living Iroi B2 creates a nonrecursive copy (cap 3) that delivers 20 ticks of 0.375 parent pistil damage base every 0.5s.",
    },
    Hypothesis {
        id: "adler.critical_riposte_v1",
        description: "Adler Critical Riposte (GA_Adler_Melee / GE_Player_Adler_PerfectEvadeAttack_Damage) consumes one armed critical dodge opportunity from a successful Critical Dodge, delivering native Category Z Incantation damage with nominal damage offset .34929872s, recovery 1.354915s, and granting 4.4 Ultimate Energy without granting Karma.",
    },
    Hypothesis {
        id: "adler.plunge_v1",
        description: "Adler Plunge landing (GE_Player_Adler_AirAttack_Damage) delivers native Category X Incantation damage with nominal landing offset .050169s and grants 1 Karma and 2.3 Ultimate Energy on hit.",
    },
    Hypothesis {
        id: "adler.normal_chain_13_v1",
        description: "Adler thirteen-packet Normal chain extends stage 4 with Melee4_1, Melee4_2, and Melee4_3 at .23622027s, .48173124s, and .73400503s, dealing native Category A Incantation damage and granting 1 Karma per connected hit.",
    },
    Hypothesis {
        id: "adler.burst_intervals_v1",
        description: "Adler Burst delivers five Sunya projectiles at 2.1851034s and a second conditional five-projectile volley at 2.343074s if exactly one dummy was living at admission. A1 (+20 Karma) and A5 (+20 flat Ultimate Energy) callbacks occur at UltraSkill.2 (2.343074s) if the source is alive.",
    },
    Hypothesis {
        id: "adler.righteous_heart_v1",
        description: "Adler Breakthrough 4 passive Righteous Heart adds .20 DEF% to Adler base-DEF zone, active in party on/off field.",
    },
    Hypothesis {
        id: "adler.blessing_party_interception_v1",
        description: "Adler Blessing shield acts as a party shield pool that protects the current living active actor, following field ownership on switch without resetting capacity, snapshot or expiry.",
    },
    Hypothesis {
        id: "aurelia.critical_riposte_v1",
        description: "Aurelia Critical Riposte (GA_Mitsuki_Melee / GE_Player_Mitsuki_PerfectEvadeAtk_Damage) consumes one armed critical dodge opportunity from a successful Critical Dodge, delivering native Category Z Psyche damage with nominal damage offset .406143s, recovery 1.610207s, and granting 6.5 Ultimate Energy.",
    },
    Hypothesis {
        id: "aurelia.plunge_v1",
        description: "Aurelia Plunge landing (GE_Player_Mitsuki_AirAttack_Damage) delivers native Category X Psyche damage with fall-height bonus (1 + h), nominal landing offset .050169s, recovery .215212s, and granting 6.5 Ultimate Energy.",
    },
    Hypothesis {
        id: "aurelia.cadenza_power_select_v1",
        description: "At stage 3 Normal emission, evaluate Cadenza: if active, select GE_Player_Mitsuki_Melee3_Power_Damage across all in-range dummies; if inactive, select GE_Player_Mitsuki_Melee3_Damage on the single locked target.",
    },
    Hypothesis {
        id: "aurelia.staccato_v1",
        description: "Staccato battle ride (Mitsuki_EvadeBranch): consumes 15 admission stamina plus 38/s drain under EnergyStop, terminating at <= 40 stamina or release, emitting paired blasts at loop offsets (powered with AoE if Cadenza active, else ordinary single-target) with End recovery 1.072441s and A4 +20% base DEF.",
    },
    Hypothesis {
        id: "aurelia.staccato_perfect_dodge_v1",
        description: "During Staccato, an explicit successful Critical Dodge spends 15 dodge stamina (setting 0.9s cooldown) and emits four PerfectAtkBullet slots at +.2, +.4, +.6, +.8s (powered if Cadenza active, else ordinary).",
    },
    Hypothesis {
        id: "aurelia.crescendo_b4_v1",
        description: "Aurelia Breakthrough 4 passive Crescendo: each connected own State.Damage.Bullet recipient hit adds one stack (+1% base-ATK as ATK), cap 10 (cap 20 with A6), with shared 5s expiry refreshed on hit, extending by +2s on crossing to 10 stacks once. A6 grants 10 stacks on Skill finish.",
    },
    Hypothesis {
        id: "aurelia.harmonics_b2_v1",
        description: "Aurelia Breakthrough 2 passive Harmonics: on accepted Nova removal from a living target, schedule three Psyche hits of GE_Reaction_4_new_1070_Damage at removal +.3, +.6, +.9s using Aurelia live stats (0.50 ATK each).",
    },
    Hypothesis {
        id: "aurelia.a2_cadenza_v1",
        description: "Aurelia Awakening 2 Unique Melody: a successful Critical Dodge or QTE cast enters Cadenza for 5s if absent, or extends active Cadenza by 5s clamped to max 12s from current time.",
    },
    Hypothesis {
        id: "aurelia.r6_psyche_v1",
        description: "Aurelia Awakening 6 Resonance: entering Cadenza or extending its duration grants all living allies +8% Psyche DMG for 15s (BuffRecipient::TeamAll), refreshing duration without stacking.",
    },
    Hypothesis {
        id: "aurelia.shadow_creed_skill_finish_v1",
        description: "Shadow Creed 4pc set effect activates at Skill recovery (Skill REALFINISH at 1.128081s), granting +25% base ATK for 20s, refreshing on recast.",
    },
    Hypothesis {
        id: "baicang.power_words_v1",
        description: "Baicang Normal 3rd input, Riposte, QTE, and derived Skills generate Power Words (Silence, Objurgate, Bless) with 3/4 storage cap, 3%/1.5% current-HP cost, 15s lifespan outside domain, and immediate or field-start domain activation.",
    },
    Hypothesis {
        id: "baicang.hold_branches_v1",
        description: "Native Formless/Unrestrained inputs deliver one uncapped AoE direct hit and three explosion slots, count one connected Normal input and arm Such Crime through recovery plus 2s. Authored Normal holds below .2s remain taps; at least .2s selects Formless after Normal1/2 or Unrestrained after Normal3, or horizontal/vertical by declared movement after an admitted dodge. Branch timing starts on release, retaining exact held combat time. A heavy discards unused Riposte and pays no extra stamina. Dodge inputs with the Word or Riposte model require declared stamina, pay one 15-point efficiency-scaled cost, use .9s cooldown/.5s recovery, grant half-open .35s incoming immunity and arm Silenced Thought through recovery plus 2s. Cancellation and source hold lowering remain pending.",
    },
    Hypothesis {
        id: "baicang.execute_and_fatal_hit_debt_v1",
        description: "While Judgment of Autumn domain is active, positive HP damage executes living targets below 10% HP (or 5% for boss) by remaining current-HP subtraction. Fatal incoming damage to Baicang during domain is intercepted once, leaving at 1 HP with 3s immunity, domain shortened to +3s, and stored lethal damage repaid as 10 installments upon domain end.",
    },
    Hypothesis {
        id: "baicang.plunge_v1",
        description: "Baicang ground-near AirAttack landing fixture delivers one uncapped AoE GE_Player_Cang_AirAttack_Damage at landing+0.049876s with recovery 0.198548s. The fixture declares height bonus zero; nonzero-height input remains unsupported.",
    },
    Hypothesis {
        id: "baicang.riposte_v1",
        description: "Baicang Critical Riposte after Critical Dodge delivers GE_Player_Cang_PerfectEvadeAttack_Damage and generates Power Word Bless.",
    },
    Hypothesis {
        id: "combat.packet_cycle_accrual_v1",
        description: "An owner-bound native damage packet grants its exact frozen HeterochromeAdd once after positive actual target HP loss, independent of AoE recipient count. Only the living source owner receives Cycle, scaled by declared cycle_gain_efficiency (default 1), with integer cap 100, exact fractional carry and explicit integer overflow. Qualified switches spend 100 without resetting carry; grants and spends reconcile the owner ledger. No Ultimate team distribution, Charge Efficiency, reaction damage, execute, hit-count inference or unbound owner rows grant Cycle.",
    },
    Hypothesis {
        id: "baicang.moderate_mischief_v1",
        description: "Baicang breakthrough 2 refreshes the live locked target's existing Scorch once per generated Word, preserving its original offensive snapshot, attribution and tick phase. Ordinary Scorch stays capped at one; the existing Zankou extension retains its three-stack admission. Expired or absent Scorch creates no application and refresh cannot generate a Word.",
    },
    Hypothesis {
        id: "baicang.height_aware_plunge_v1",
        description: "An explicitly authored positive airborne landing delay and dimensionless height bonus0..1 select Baicang's one AirAttack packet. Multiply its selected native coefficient by1+height_bonus, preserve one AoE energy opportunity, land at admission+airtime, impact at landing+0.049876s and recover at landing+0.198548s. No geometry, jump velocity or extra AirAttack1/2 hits are inferred. Omitted profiles retain only the separate ground-near fixture.",
    },
    Hypothesis {
        id: "blackbird.b2_stain_v1",
        description: "A living Blackbird with breakthrough2 enhances newly applied Stain to20 World seconds and base multiplier1.3, replacing1.2 rather than multiplying it by1.1. Capture enhancement once per generation, retain it after field exit/source defeat, refresh without stacking, and use each eligible Psyche/Lakshana damage actor's live Cycle Intensity. No extra HP packet or resource grant is created.",
    },
    Hypothesis {
        id: "blackbird.b2_nova_v1",
        description: "Living breakthrough2 Blackbird captures baseATK per target Nova generation and accumulates actual positive applied HP loss during its half-open lifetime, excluding its triggering Support/QTE command and children, all ordinary Nova ticks and Blackbird settlement packets. At end, after pending shared damage, settle min(accumulatedHP/5,1000*capturedBaseATK) as fixed Mental HP damage unaffected by crit, DEF, RES or buffs, with no resources. The initial generation may create one fresh shared Nova with its own accumulator; that child cannot reapply. A new authored refresh takes precedence over the automatic child. Source defeat disables the extra/reapplication; target death cancels without postmortem settlement.",
    },
    Hypothesis {
        id: "blackbird.primary_packet_groups_v1",
        description: "Each Blackbird ordinary Normal, Skill and transformation Burst primary packet hits every living all-in-range dummy once, with recipients ordered within each packet and one shared causal resource group. Clone the admitted primary's coefficient and metadata unchanged for its other recipients; do not reproject a research primary from the native child curve. A dead target zero cannot suppress a surviving recipient's one positive-hit resource opportunity. Separate packets remain distinct groups even at an equal delivery time.",
    },
    Hypothesis {
        id: "blackbird.skill_cycle_fill_v1",
        description: "An admitted ordinary Blackbird Skill executes its Skill.2 HeterochromeExecution once at activation, filling the owner's Cycle pool to100 before any damage even when all packets miss. Grant only the missing whole amount, independent of Cycle gain efficiency, preserve exact fractional carry and prior overflow/spend accounting, and keep Ultimate Energy unchanged. Connected Skill packets retain their independent HeterochromeAdd channels and cap overflow. Failed admission, other actions and field entry do not fill Cycle; evolved Skill is outside this binding.",
    },
    Hypothesis {
        id: "baicang.cancel_windows_v1",
        description: "With the Power Word clock selected, Normal1/2/3 permit an early Skill at .378894/.379190/.477995 real seconds, retaining connected Word counts and rejecting other early inputs. Explicit native jump cancels Formless at release+.506294s, Unrestrained at release+.300989s, default Skill at+.249464s, derived Skills at+.247532s and Burst at+2.871112s. These valid jump windows follow every mapped actor/projectile emission, so emitted damage and committed energy/cooldowns survive without refunds. Jump clears unused Riposte and chain state, ends derived-ability persistence at jump+2s, pays no second dodge cost and supplies no implicit plunge profile. No pre-trigger, forced-interrupt or unsupported cancel window is inferred.",
    },
    Hypothesis {
        id: "baicang.forced_interrupt_v1",
        description: "Explicit scenario interruption events end Baicang's current admitted action after same-time hit/proc settlement. Cancel only un-emitted native deliveries; retain spawned actors/projectiles, triggered Words, existing field ticks and committed costs. Default/Such Crime actors emit at+.018070/+.016652s; Silenced Thought emits groups at+.216575/+.216581/+.216683s and its single delayed explosion is committed by the first group. Branch1 explosion actors emit at release+.486667s, Branch2 missiles at release+.150174/+.167079/+.183235s. A Skill interrupted before its first emission uses3s cooldown from interruption; otherwise retain the committed shared cooldown and captured A1 disposition. A Burst interrupted before its field marker cancels that generation without refunding energy or deleting old debt. Interruption supplies no incoming damage, immunity, geometry or implicit jump/plunge. Requires the Power Word model; no other owner interruption is inferred.",
    },
    Hypothesis {
        id: "baicang.actor_hitstop_v1",
        description: "With the Power Word model, Baicang montage coordinates advance through causal positive-HP-contact pauses using exact PPM integration and earliest integer deadlines. Normal1/2/5 request .033s at .1; Normal3 requests .067s at .1 after .04s; each Normal4 contact requests .033s at .1 after .033s; Formless contacts request .1s at .3, Riposte .3s at .01, QTE .3s at .3. Coincident recipients share a request and overlaps use the strongest slowdown. Raw Normal/recovery/cancel coordinates replace baked local pause additions; Formless actors emit with the direct source marker and travel .25s. Released actors/projectiles, activated Words/field lifetimes, cooldowns, costs and combat time remain real. Unreleased producers, field activation, recovery and legal cancel windows follow caster progress across actions, including residual pauses after jump/interruption/switch. Execute requests .3s at .3 only for the owner's locked target (dummy0). No pause is invented for zero HP loss, missing duration, other actors or target AI.",
    },
    Hypothesis {
        id: "skia.gnaw_entry_state_v1",
        description: "Bind Skia's six ordinary Skill contacts at .153592/.216117/.296382/.369313/.871421/.913168s, the Skill.4 ShadowControl marker at1.048984s, ordinary recovery at1.335873s and12s cooldown from the first notify. Skill.4 grants actual state38 for5 World seconds to a living active owner. A different field exit clears control before future callbacks; exact same-time input still precedes the grant. This finite model exposes real state to teammate entry selectors and does not certify the remaining Gnaw kit: a retained marked-target coordination at+.35s, natural outro, Normal/Support exit or incoming-HP immunity/ground-exit dependency diagnoses rather than silently omitting damage. Invalidated generations do not grant or flush exit damage after switching/death. Requires the existing native Skill packet model; no initial or party-only stealth flag is inferred.",
    },
    Hypothesis {
        id: "baicang.skia_entry_v1",
        description: "On an admitted switch into living B4-enabled Baicang, select the special Skia entry only when the actual outgoing character1008 is in live state38 and Baicang's independent30s Skia-assist cooldown is ready. Capture that state before outgoing cleanup, start the cooldown only on accepted entry, use the invested Normal coordinate, release the actor at+.378030s and deliver three uncapped AoE pulses at+.578030/+.658030/+.738030s with recovery+.399466s. One connected pulse generates one A Silence for the entry, irrespective of recipients/pulse count; the Word model retains its nonlethal HP cost. Retain the actual switch command as causal parent and never duplicate an ordinary entry. Shared qualified-switch Cycle spend/reaction rules remain independent; no free Support or reaction is inferred from Skia's presence alone. Requires the Word model and an engine-selected entry; proactive forged assist packets reject.",
    },
    Hypothesis {
        id: "baicang.fadia_assist_v1",
        description: "With the Word model, living B4-enabled Baicang coordinates while living active Fadia has HP<=.20 of live effective MaxHP. Initial/field entry, actual HP/MaxHP settlements with oldHP>=newHP and the independent cooldown-ready deadline request a live recheck; rising healing alone does not request it. A valid locked dummy0 is required. Accepted coordination starts a separate30s real Fadia cooldown, preserves active control and emits one immediate uncapped AoE native Fadia packet using invested Normal talent. Its first connected hit generates one A Silence attached to locked dummy0 and one zero-travel heal to the captured Fadia, based on2.7 times Baicang's live ATK after Word/A2 settlement, ordinary healing zones and missing-HP clipping. Recipient fanout never duplicates the Word/heal/resources. No target means no admitted assist, cooldown spend, Word or heal-only emission. Shared Skill cooldown/A1 credits, input recovery and costs are untouched; no ordinary Normal input or immunity is inferred. Typed autonomous event receipts retain the real initial/HP/switch/deadline cause, and all emitted descendants retain that event instead of a fake authored command. Schedule cooldown-ready checks only within the declared horizon; no periodic readiness polling.",
    },
    Hypothesis {
        id: "baicang.lacrimosa_entry_jump_v1",
        description: "An admitted ordinary Lacrimosa dodge immediately enables the native Evade Cang carrier for the existing modeled half-second dodge recovery. Leaving during that window creates the native1.5s Ability.Player.Lacrimosa.Cang tag; ordinary ability end removes the carrier, not a previously created tag. Capture this actual outgoing state before cleanup. A living B4 Baicang entering without an already active ability receives native0.2s JumpZ with jump multiplier1.2, as a non-damaging autonomous entry effect linked to the real switch. It does not invent a Normal input, energy, damage, height or flight duration and preserves independently qualified Support/reactions. A following plunge requires explicit landing time and final height bonus; jump multiplier never multiplies its damage or authors its landing geometry. The immediate Evade marker uses the finite admitted-input clock, not measured animation timing; a Critical Dodge alone does not establish the ordinary Evade carrier.",
    },
    Hypothesis {
        id: "lacrimosa.bat_evade_exit_v1",
        description: "An admitted ordinary Lacrimosa dash enables its native Cang carrier at the selected immediate first Evade marker, lasting until the existing half-second input recovery ends. Source role-out-begin during GA_Lacrimosa_Evade adds Ability.Player.Lacrimosa.Cang for1.5s and removes the carrier; ability end also removes that carrier. Permit switch cancellation after the admitted marker during this actual ordinary dodge, preserving paid stamina, dodge cooldown, switch cooldown and resources. Cancel only this dodge recovery on field exit, retain the already created tag through re-entry until its original half-open deadline, and never refresh it without a new qualifying dodge exit. Critical Dodge alone does not prove this ordinary Evade carrier. These are bounded event/clock and arbitration choices from the reached native graph, not measured animation or bat-flight geometry; no attack, immunity, damage or free Support is added.",
    },
    Hypothesis {
        id: "baicang.qualified_support_v1",
        description: "Baicang ordinary Support is engine-owned on a living admitted entry only when the real outgoing Cycle pool is full, the pair qualifies a bound shared Esper Cycle reaction and the locked dummy is alive. Use purchased Support talent and the native Cang QTE packet once per living in-range dummy, with the Word model's1.000358s contact and1.566945s recovery (or its separately selected caster clock). QTE creates one A Silence on first connection and one resource opportunity across recipients. Shared switch settlement spends the outgoing100 Cycle and admits its reaction exactly once; do not spend Ultimate energy; the reached QTE specifies no local cooldown, so select no additional Support timer without claiming proof of inherited native defaults. Preserve incoming recovery and ordinary dispatch failures. Lacrimosa JumpZ composes with this qualified Support; it cannot create a free unqualified attack or replace the Support with a fake Normal. This opt-in preserves historical packet-only entry boundaries and does not establish every other owner's dispatch eligibility.",
    },
    Hypothesis {
        id: "blackbird.ordinary_malice_v1",
        description: "Track ordinary Blackbird Malice as an exact nonnegative pool capped60, separate from Charge and Cycle. Living combat initialization grants20 once; no four-second in-combat refill. Ordinary Skill.2 grants40 once per admitted cast, even without a connected recipient. Ordinary positive-HP Normal packets grant exact native amounts .857919/1.762896/1.089593/.933937/1.317647 for stages1..5, once per causal packet across uncapped AoE recipients; a complete17-packet chain grants20.000003. Cap excess remains visible in a conserved exact ledger through waits and switches. This bounded resource binding does not certify transformed spending/extras, coordinated birds, ordinary secondary actions, QTE/parry or out-of-combat transitions, whose admission and lifetimes require their owner models; legacy packet-only fixtures retain their existing boundaries.",
    },
    Hypothesis {
        id: "blackbird.coordinated_bird_v1",
        description: "Ordinary Skill re-arms one Blackbird bird controller per cast generation, clears marks/count, starts ready at cast time and enforces an inclusive2s interval between accepted requests, with five accepted requests maximum. Each positive surviving Skill recipient refreshes its own15s mark without resetting the controller. Positive HP damage by a different currently active living ally to a live marked dummy accepts once per causal packet, selecting the first/lowest live marked recipient. Blackbird own/off-field/zero-HP/Nova/bird damage cannot request coordination. Dispatch one full uncapped native SkillAssist AoE after the causal packet HP/resource/Break settlement, using invested Skill talent, zero inferred travel, ordinary live stats and per-target RNG, one Charge/Cycle/Break opportunity across recipients. An accepted count is not refunded on target death. Recast cancels uncommitted old-generation requests; field exit preserves marks/count, owner death disables new requests. A6 positively connected birds grant4 exact Malice once outside form through the separate resource model. Autonomous receipts retain both the original Skill command and real earlier triggering hit without inventing an authored cast. Requires ordinary Malice; does not establish transformed action/form lifetimes.",
    },
    Hypothesis {
        id: "blackbird.ordinary_clock_v1",
        description: "Bind ordinary Blackbird Normal1..5 and four-hit Skill to their native montage progress coordinates, rounded once to nearest microsecond, at rate1. Positive applied HP from each causal primary packet adds one .03s owner hit-pause at .01 rate, shared across AoE recipients; zero HP loss adds none. The shared exact HitlagTracker projects only pending montage contacts and owner recovery, retaining real cooldowns, marks, birds, buffs and other actors. Simultaneous recipients retain their already emitted packet time. Normal continuity extends through its actual full recovery and2 World seconds afterward, with stage wrap/reset on other actions or field exit. Ordinary Skill has16s real cooldown, independent of hit-pause; generic caller clocks remain the historical boundary when this model is absent. This bounded clock does not certify Burst/form replacement, transformed/secondary actions or early cancel gates, which retain their separately tracked owner admission dependencies.",
    },
    Hypothesis {
        id: "blackbird.ordinary_secondary_v1",
        description: "Ordinary Blackbird plunge requires a fresh explicit airborne-entry profile on that command: positive landing time and final height bonus0..1, with coefficient multiplied once by1+h across uncapped AoE, contact landing+.053784s and recovery landing+.281249s before one connected .13s/.01 owner pause. No geometry or automatic maximum height is inferred. An explicitly successful paid Critical Dodge authorizes one next-Normal native PerfectEvade packet, preserving category Z through an owner-bound Melee-to-PerfectEvade relationship. Use .282278s contact/.903675s recovery before one .13s/.01 owner pause, invested Normal talent and once-per-packet charge3/Cycle4.2/Break2.5/Malice4.615385. Plunge grants native1.68/2.3/.3/Malice1.723077 on positive HP. Any other action/input or field exit clears unused Riposte permission; ordinary dodge does not grant it. Select front-dodge recovery1.004760s and retain stamina15 plus the real .9s dodge cooldown; no free counter or parry20 is inferred. Requires ordinary clock and Malice models. Early cancels, branches, transformed secondary actions and explicit parry remain separate owner gaps.",
    },
    Hypothesis {
        id: "blackbird.qualified_support_v1",
        description: "Ordinary Blackbird QTE is engine-owned on a living qualified entry: actual outgoing full100 Cycle, a supported adjacent-element reaction, living locked dummy0 and incoming recovery. Dispatch invested Support talent with the native QTE row once per living in-range dummy, no Ultimate cost or additional inferred QTE cooldown beyond real qualification/recovery. Use native progress contact1.000222s/recovery1.333181s and a separate switch lock1.344509s, with one positive-HP .13s/.01 caster pause across all recipients. B4 QTE.2 at progress1.002488s independently grants20 exact Malice once even if damage misses; project that actual marker through causal pauses, do not grant per target or attach it to QTE's zero Malice damage row. Owner death cancels an uncommitted marker. The shared switch service spends the original outgoing100 Cycle and owns its reaction once. Preserve separate charge7.56/Cycle0/Break2.5 and all continuing state. Requires ordinary clock and Malice models; an authored free QTE input rejects when this complete entry model is selected. Legacy packet-only boundaries remain available, and transformed Support/form replacement remains separate required work.",
    },
    Hypothesis {
        id: "blackbird.primary_form_v1",
        description: "Bind primary Blackbird Burst to the declared successful zero-delay mesh callback, native100 Ultimate cost and20s World cooldown. Capture the non-hybrid21.5s on-field World form lifetime at accepted commit; its9 primary contacts use native progress with .067s/.3 owner pauses and5.118154s recovery progress. Form time includes local hitstop while shared World progression is enabled, stops off field, resumes remaining time on entry and ends before equal-time ordinary admission; cooldowns, team buffs, reactions, bird requests and committed packets keep their declared clock policies. UltraSkill.3 at progress3.166927s applies the15s team base-ATK20% buff, replaced by A4's25s plus general damage12%; A6 independently grants own Psyche25% for25s. Already applied buffs persist source death. A2 adds20% base DEF only while the form lives, including its off-field pause; no phantom CritDamage is added. R3 commit grants20 exact Malice once, while ordinary gains including A6 birds are suppressed in form. Natural/death cleanup retains resources/cooldowns and committed packet identity. This bounded lifetime/primary binding requires all ordinary owner models; reached evolved inputs and qualified transformed Support explicitly diagnose until their own bindings are supplied. Finisher permissions, evolved attacks/extras and mesh callback failure/timeout are separate required owner work, never fabricated as ordinary attacks or successful failed callbacks.",
    },
    Hypothesis {
        id: "blackbird.evolve_actions_v1",
        description: "Bind Blackbird evolved abilities during active transformation form: GA_BlackBird_Evolve_Melee (5 stages delivering 2, 5, 6, 1, 6 primary packets with native progress and hit-pause .0297s, Melee5 .015s), GA_BlackBird_Evolve_Skill (4 packets, 1s cooldown, requiring and consuming the 30s permission granted by Melee5 at progress .351288s, healing 4x base ATK with A2, multiplied by 2.5 with A1), and GA_BlackBird_Evolve_UltraSkill (finishing Burst delivering 7 packets, removing form at commit, multiplied by 1.5 with R3). Each connected primary hit consuming Malice spends up to its native threshold (Melee1..5: .541591, .369949, .349951, 2.258020, .784891; Skill: 3.25) and enqueues one separate category A Extra packet at hit+.067s (Melee5 hit+.100s) with coefficient k*spent (k+k3 with A3, k reading Normal talent level). With A1 finisher first hit spends all remaining Malice (max 60) for one extra; without A1 finisher preserves Malice with no extra. Requires primary form and exact Malice models.",
    },
    Hypothesis {
        id: "blackbird.evolved_secondary_v1",
        description: "Transformed Blackbird plunge and Critical Riposte bind the evolved native rows only during active form. The plunge keeps the fresh explicit airborne landing and height bonus0..1, multiplies coefficient once by1+h across uncapped AoE, and uses contact landing+.100643s/recovery landing+.281249s. An explicitly successful paid Critical Dodge still authorizes exactly one next-Normal riposte; in form it selects the evolved PerfectEvade row through the owner-bound Evolve_Melee relationship with contact .225959s/recovery .839275s. Both use invested evolved-Normal talent, one .13s/.01 owner pause on positive HP across recipients, and once-per-packet charge/evolved-Cycle/Break. The evolved QTE is a separate qualified-Support binding. Connected evolved plunge/riposte primaries spend up to .933204/2.099708 exact Malice and enqueue one category A extra at hit+.067s with the ordinary k*spent curve. Any other action/input or field exit clears unused permission; the transformed row applies only to the current form. Requires ordinary clock, exact Malice, primary form and evolved-action models; branches, early cancels, explicit parry and combat transitions remain separate owner models.",
    },
    Hypothesis {
        id: "blackbird.evolved_support_v1",
        description: "While Blackbird transformation form is active, an otherwise qualified entry selects the evolved Support GA_BlackBird_Evolve_QTE instead of the ordinary QTE. Dispatch invested Support talent with the native evolved QTE row once per living in-range dummy, no Ultimate cost or added cooldown, contact .878181s, recovery1.167929s and a separate switch lock1.266940s, with one positive-HP .13s/.01 caster pause across recipients. The evolved row has charge6, zero Cycle add and Break2.5. It has no ordinary QTE.2 marker and therefore never grants the B4 twenty Malice; instead its connected primary spends up to1.666435 exact Malice and enqueues one category A extra at hit+.067s with the ordinary k*spent curve. The shared switch service still spends outgoing100 Cycle once and owns its reaction. Requires form, ordinary clock, exact Malice and evolved-action models; an unaccepted transformed entry diagnoses rather than falling back to the ordinary QTE.",
    },
    Hypothesis {
        id: "owner.action_role.character_1042_evolve_qte_v1",
        description: "Classify this exact frozen evolved character action as QTE because its resolved parent DT_CharacterAbilityConfig identity is the owner QTE and DT_CharacterAbilityEffectConfig has no evolved row; this role is an owner-reviewed model hypothesis for this ability only.",
    },
    Hypothesis {
        id: "blackbird.branches_v1",
        description: "Hold Blackbird ordinary Normal after stages1/2/3 with recovery-relative continuity and a hold of at least .2 real seconds selects Roostcall (the single native Branch1 row). In form, hold after evolved stage2 selects Bestowed Grace (Evolve_Branch1), and hold during an unfinished Bestowed Grace or evolved Skill selects Tidings Unto All (Evolve_Branch2). The held command must reach the current montage's earliest applicable cancel gate; it replaces the unfinished montage immediately and pays its full hold interval on the real axis before branch progress starts at release. Remaining montage-bound requests are removed while spawned actors and captured Malice extras continue. Use native branch progress Bestowed Grace .617712/.733904/.852485/.967517/1.083830 with recovery1.345821 and Tidings .143845/.283795/.441973/.582454/.730677 with recovery .975109 (Roostcall .200476/recovery .757210), rounded once to microseconds, full uncapped AoE, invested Normal talent and .03s/.01 owner pauses. Roostcall grants4.123077 exact ordinary Malice on positive HP; evolved branch primaries spend up to .719900/.673240 and enqueue their ordinary k*spent extras. Completion resets the Normal stage. Requires ordinary clock and exact Malice; transformed branches additionally require form and evolved-action models.",
    },
    Hypothesis {
        id: "blackbird.cancel_gates_v1",
        description: "Bind Blackbird montage cancel gates from native progress, resolved through the same owner hit-pause tracker so only pauses actually caused before the gate are added. Earliest applicable gates per ordinary Normal stage1..5: Jump .184549/.336276/.368364/.888358/.765616, Skill .333077/.502016/.667925/.639014/AnySkill, Move .754313/.752055/1.001773/1.301300/1.286865; evolved Jump .173066/.103925/.175358/.185907/.524134, Skill .644213/.579390/.966451/AnySkill/AnySkill, Move1.010629/.769638/1.752073/1.054906/1.847584. Roostcall .250451/.667489/1.010329, Bestowed Grace .666329/1.199905/AnySkill, Tidings .840242/AnySkill/1.762101 (Jump/Skill/Move); plunge Jump .149206 and Move .777534 ordinary/.522255 evolved; riposte Jump/Skill0 and Move AnySkill; ordinary Skill .288606/AnySkill/1.526221, evolved Skill .835975/AnySkill/1.007483; transformation Burst5.044123 all; finisher AnySkill; ordinary QTE AnySkill/AnySkill/1.335498, evolved QTE1.274009/AnySkill/1.271542. AnySkill equals recovery; use the earlier of the input-specific gate and AnySkill. A gated admission shortens only the lock to the gate and removes the previous montage's pending bound requests; spawned actor pulses survive once spawned and captured extras always survive. The same packet cancellation applies on field exit; stages/permission/riposte follow their existing reset rules. Requires ordinary clock and Malice models.",
    },
    Hypothesis {
        id: "blackbird.combat_transitions_v1",
        description: "Explicit Blackbird combat transitions use the ordinary exact Malice pool. Leaving combat grants no immediate Malice and schedules one20 grant after4 World seconds only while the owner is still out of combat; a later explicit re-entry cancels that pending callback and grants the distinct new entry20 immediately, clamped by the60 cap. Neither is periodic and the initial battle-entry20 remains initialization. Both grants are ordinary Malice gains, so they are suppressed while the transformation form is active even though the in-combat flag and callback cancellation still apply. A generation guard prevents a cancelled leave callback from firing. Requires ordinary Malice; no other resource, damage or action is inferred.",
    },
    Hypothesis {
        id: "blackbird.explicit_parry_v1",
        description: "An explicit successful Blackbird parry scenario event at a declared real timestamp grants20 exact Malice once when the owner is alive and outside the transformation form. While the form is active it instead grants or refreshes the same cap1 thirty-second evolved-Skill permission as evolved Melee5. The parry event is distinct from Critical Dodge, never inferred from ordinary evades, and adds no damage, dodge authorization, energy, immunity or other effect. Requires ordinary Malice; transformed grants additionally require form and evolved-action models.",
    },
    Hypothesis {
        id: "chaos.ordinary_crime_v1",
        description: "Bind Chaos ordinary Normal and Skill positive-HP Crime to one exact pool starting0, capped1000. Native GE-key gains are Normal1 22; Normal2 33/15; Normal3 28 each; Normal4 35/13/13/19; Normal5 41/41/94; Skill50 per bite, for382 per full twelve-packet chain and100 per Skill. Gain once per causal primary packet irrespective of dummy count. Selected A1 adds .4 of that mapped amount exactly once if any connected recipient has this source's Warrant; a later marked recipient can supply only the missing A1 increment, preserving recipient-order independence and cap overflow. Misses/zero HP/dead owners grant nothing. These primaries use full uncapped AoE with shared packet identities and separately bind native Ultimate charge and HeterochromeAdd through their existing exact ledgers. Normal continuity lasts through caller-authored recovery and2 World seconds afterward, with reset on another action or field exit. Caller packet timing and existing Warrant dispatch remain explicit historical boundaries; this resource prerequisite does not implement Crime auto ticks/spending, transformation, held Verdict, qualified QTE or full-kit acceptance. Crime and both ledgers persist across waits, switches and cycle boundaries.",
    },
    Hypothesis {
        id: "chaos.ordinary_clock_v1",
        description: "Requires chaos.ordinary_crime_v1 and binds ordinary Chaos Normal1..5 and both Skill bites to selected native progress at1/s, rounded once to microseconds. Positive HP once per primary packet delays pending owner contacts and recovery by the documented effective pause: Normal1/3 .0198s, Normal2 .005/.016, Normal4 first .0495, Skill .0495 per bite; other reached ordinary rows add0. These effective progress losses are represented as exact full-stop intervals after contact, a selected implementation rule rather than a claim about the native slow-motion curve. AoE never multiplies pause. Normal continuity uses actual recovery plus2s; cooldowns, Warrant and DPS time stay real. Skill CD is15s from its .243640 progress marker. Skill applies Warrant at .283733 progress to the first two living eligible dummies in stable order, before damage, for15 World seconds, with source-owned live B4 replacement bonus .30 instead of .20. Existing marks continue off field. Field exit cancels future montage requests and an unreached mark marker. No polling or hidden wait; early cancel gates and interrupted Skill3/15 CD remain separate bindings. Transformation/Burst requires the separately accepted primary form model and diagnoses if absent; secondary actions and natural QTE remain separate. Selected A5 counts living source-owned marks, independently of dummy count.",
    },
    Hypothesis {
        id: "chaos.primary_form_v1",
        description: "Requires Chaos ordinary Crime and clock models. Burst spends100 Ultimate energy and starts15s cooldown at accepted commit; its four UltraSkill1 and one UltraSkill2 main contacts follow native progress2.807466/3.180295/3.501765/3.947290/4.792259, recovery4.915031 and independent switch lock4.944578. Commit immunity ends at progress4.802493, not at recovery or form expiry. Burst marks all living dummies at2.7 and enters Dread Echo at2.718888 for12 World seconds, with no added Burst hit pause. While living/on field, one30 Crime tick occurs every.2s, first entry+.2, through the final endpoint at entry+12; the final tick grants then expires form before equal-time action admission. Off-field ticks are skipped without banking or resetting phase; expiry follows World time. Recast invalidates the old generation. Selected A6 fills the Crime pool to1000 once at commit. Live form selects native Normal _Ultra rows at admission, including stage2's swapped coefficient/charge/Cycle/Break and15/33 Crime mapping; captured packets survive natural expiry. Transformed stage5 uses its own .752119 final contact/1.090035 recovery; other Normal coordinates and effective pauses retain their owner profiles. Cleanup preserves exact resources, cooldowns, marks and committed packets, resets the chain, and cancels new generation on owner defeat. This binds primary transformation only; Verdict spending/branches, secondary actions, natural QTE and remaining team interactions remain separate.",
    },
    Hypothesis {
        id: "chaos.forced_interrupt_v1",
        description: "Requires Chaos ordinary clock and an explicit authored interrupt event. While a living owner is in montage recovery, cancel future primary contact requests and unreached owner markers, release recovery immediately, clear montage immunity/switch lock and reset the Normal chain. Skill cancellation at real elapsed less than .2s starts3s cooldown at cancellation; exactly .2 or later starts15s there. Preserve committed energy/Crime costs, delivered damage, marks and an already entered Dread Echo with its real tick phase. Burst cooldown remains15s from commit; interruption before its entry marker prevents that form. Interruption adds no invented stun, free attack, refund or implicit wait; voluntary early cancel gates remain separate.",
    },
    Hypothesis {
        id: "chaos.neutral_verdict_v1",
        description: "Requires Chaos ordinary Crime/clock and binds a neutral held canonical Normal input with at least1 real second of held lead-in, paid in full before stage1. Capture ordinary/transformed Branch1 rows and effective Normal talent at admission; two contacts use native progress, then read live Crime at stage1 recovery after due auto Crime. At1000 spend1000 once and launch two captured Branch2 A-category contacts; otherwise preserve the partial pool and finish stage1. Branch hits generate0 Crime. Stage2 positive packet contact adds .185253 effective pause once across all recipients, preserving native coefficients, charge/Cycle/Break and real elapsed time. Full stage grants1.1 real seconds damage immunity capped by actual recovery/cancellation. Natural form expiry cannot cancel or reclassify an admitted branch. Explicit interruption cancels future contacts and a pending spend decision without refunding a committed spend. Combo hold predicates and voluntary cancels remain separate and cannot bypass current recovery. This is a selected executable neutral-input and causal-spend model, not proof of native Blueprint mutation order.",
    },
    Hypothesis {
        id: "chaos.jump_cancel_v1",
        description: "Requires Chaos ordinary clock. Explicit Jump cancels an unfinished owner montage only at its selected native Jump progress plus already caused owner pauses, clamped to natural recovery. Normal gates .263727/.464920/.502412/1.209621/1.061822, transformed Normal3 .483244; Verdict stage1 .183883/.182860 and full stage2 .674412/.632402 ordinary/transformed; Skill1.324696 and Burst5.412379. At commit remove pending montage contacts/decision, clear branch immunity and reset Normal stages while retaining resources, delivered effects and real form ticks. Jump itself adds0 recovery and no flight, damage, stamina refund or automatic follow-up; invalid early input fails without waiting. Dodge, plunge/riposte, QTE and other secondary bindings remain separate.",
    },
    Hypothesis {
        id: "chaos.normal_combo_verdict_v1",
        description: "Requires neutral Verdict. A held canonical Normal may replace an unfinished Normal1..5 after at least.2 real seconds held within that montage and actor progress strictly above .3/.6/.4/1.09/.85 ordinary or .3/.64/.4/1.09/.9 transformed. The authored combo command is the hold release; both hold and progress predicates must already hold. Capture the old montage form for its predicate, select live form for new Branch1, cancel future old contacts, and launch stage1 without an additional idle1s hold. At equality fail with the next representable progress deadline; never poll or insert a retry. Stage1 still performs the live Crime decision. Neutral holds retain their paid lead-in after the previous recovery. Other source Appear/Pre/plunge/continuation predicates require their own action bindings.",
    },
    Hypothesis {
        id: "chaos.warrant_transfer_v1",
        description: "Requires Chaos ordinary clock and selected individual A4. A real party-owned positive-HP kill of a source-marked recipient transfers that source's Warrant once to the lowest-index living unmarked eligible dummy, excluding the victim and all already marked recipients. Shared all-in-range assumption satisfies proximity. Use a fresh15 World seconds and the same source-owned personal Warrant/R6 modifier; no candidate removes the dying mark without a replacement. Check each source independently, including while off field; existing source marks persist through owner defeat under their declared lifetime. Ordinary packet, reaction and finishing kills share the pre-cleanup observer; expiry/dispel/scene teardown never fabricate a kill. Remove the dying mark after source decisions so duplicate callbacks cannot transfer twice. Later deaths may cause later transfers, without recursive callbacks from mark application. Party attribution and stable candidate selection are chosen lifecycle semantics, not verified native nearest-target dispatch.",
    },
    Hypothesis {
        id: "chaos.qualified_support_v1",
        description: "Requires Chaos ordinary clock and binds one engine-owned QTE after an eligible adjacent-element entry switch spending the outgoing owner's100 Cycle; incoming Ultimate energy is not spent or required. Its native H packet hits all living dummies once at.704643 progress, recovery1.376352, no hit pause or extra cooldown, using its own Support talent coordinate in ordinary/form state. Native channels6.4 charge,0 Cycle,2.5 Break and0 Crime apply once per packet across recipients. QTE's switch lock starts at.000100 and ends at.875734, before AnySkill recovery, granting no early switch. Successful full recovery authorizes one later authored Appear Normal with the ordinary Normal1 row and .217820 first contact; it never emits a hidden attack. Consuming that Normal or another action/exit clears the permission; interruption before completion cannot grant it. Its combo hold, when the Normal combo model is accepted, requires.25 real seconds within the montage and progress strictly above.3, choosing live form for the new branch. Preserve direct-QTE charge policy, party distribution and real form phase. This is selected qualified entry/timing/continuation behavior, not proof of native automatic montage dispatch.",
    },
    Hypothesis {
        id: "chaos.verdict_continuation_v1",
        description: "Requires neutral Verdict. A full Branch2 may admit a new authored held Normal after actor progress strictly above1.06, with at least.2 real seconds held inside that stage ordinary or .1 transformed. The branch remains a continuation source after its earlier AnySkill recovery .633193 until the reached EndAbilitySkill marker1.088377 ordinary or1.102773 transformed, exclusive and projected through only caused pauses. The accepted Jump binding also retains its native gate through this bounded full-stage lifetime. Per-input owner deadlines diagnose early hold/Jump even when general next-action recovery is ready. A different admitted action, exit, interruption or defeat ends that source. This separates readiness from the bounded owner continuation lifetime without retaining unused pose-tail duration, polling or extending the original recovery/immunity. At the new release choose live form, remove remaining old requests and start plain Branch1 with no idle lead-in; its ordinary live-Crime decision still owns enhancement. Insufficient hold, equality/early progress or an expired source cannot fabricate a free idle Verdict. No resource refund/reset or extra full-stage spend is implied. These are selected source-marker and predicate semantics, not verified native dispatch ordering.",
    },
    Hypothesis {
        id: "chaos.secondary_actions_v1",
        description: "Requires Chaos ordinary Crime/clock. A front Dash pays15 stamina from an explicit shared profile, deals no damage, recovers at.998284, with Jump.082544 and Skill.333716 gates. Explicit successful Critical Dodge authorizes one later canonical Normal as Critical Riposte; another action/input/exit/defeat consumes it, with no invented timeout or stacked permissions. Riposte retains the native GA_Chaos071_PerfectEvade row through its actual Normal initiator, both forms use that common Z row at effective Normal talent, contact.250157/recovery.366207 and once-positive .297 owner pause,100 Crime,2.7 charge/4.4 Cycle/2.5 Break. An authored plunge supplies positive landing delay and height bonus0..1; choose ordinary/_Ultra X row at admission, coefficient multiplied by1+height, one all-target contact at landing+.047594, recovery landing+.159289,30 Crime/2.3 charge/3.8 Cycle/.5 Break. Capture form through expiry; no free airborne entry, geometry or added plunge tax. Native movement/Skill gates consume only reached pauses; full-stage controls retain the bounded EndAbilitySkill lifetime. Plunge combo uses.2 real hold overlapping the authored flight and progress strictly above.12 after landing. Costs, charge/Cycle carry, real form ticks and committed packets retain causal state on valid cancels. Back dodge/parry and Pre entry remain separate bindings; no passive immunity or Crime is invented for the non-damaging dodge.",
    },
    Hypothesis {
        id: "hathor.b2_remora_v1",
        description: "A qualifying real Cosmos/Lakshana Remora generation captures living party Hathor breakthrough2 Delay Warning: duration12 World seconds instead of5 and target-conditioned allied CR+.10. Reapplication resets one shared target status and chooses the then-living source policy; no duplicate stack or explosion. The captured generation survives Hathor exit/defeat, while a later generation cannot acquire a dead source's enhancement. Only living ordinary damage actors against that specific enhanced target receive the bonus, live before ordinary CR clamping; native fixed-critical/no-critical overrides keep precedence. Charge uses the same half-open extended status lifetime. No unconditional team crit, per-hit energy, orphan native explosion row or additional reaction damage is inferred. These are selected description/lifecycle semantics pending native consumer-body recovery.",
    },
    Hypothesis {
        id: "chaos.b2_remora_end_v1",
        description: "Living deployed Chaos breakthrough2 listens once to each real shared target Remora, including another actor's application. Refresh resets elapsed time and replaces the listener without an explosion. Natural expiry or explicit non-death removal on a living target emits one native8ATK Lakshana/Q packet to that target only, through an autonomous GA_Chaos_Passive_1 event caused by the actual qualified switch. Its native row GAName remains GA_Chaos071_UltraSkill through the exact owner binding, without an authored Burst or its cost/cast procs. Live ATK/crit/DEF/RES/Warrant and eligible bonuses apply; separate final bonus is min(3,.45*max(0,elapsedSeconds-5)) from the last real application. Baseline5s yields8ATK;10s26ATK;12s32ATK. This inferred duration curve reconciles the passive description with the native Hathor-gated final3 without adding it twice. Target death, source-owner death and teardown cancel pending owner explosions; off-field preserves them. Zero charge/Cycle/Break and no QTE exclusion or universal extra packet are inferred. Same-time expiry removes Remora before the new ordinary packet, preserving live status precedence.",
    },
    Hypothesis {
        id: "inputs.dash_directions_v1",
        description: "Reached for explicit Back Dash intent only. The action-indexed Front/Back profile selects a bound owner dodge without coordinates, distance, enemy geometry or auto movement. Front and absence preserve the existing front model and report; this reached receipt hypothesis identifies an actual Back selection. Back requires the owner's accepted secondary model and an explicit paid stamina profile; costs and shared dodge cooldown remain unchanged. Chaos Back has recovery.795195, Jump.0001, Skill.590149 and movement.599939. Blackbird Back captures ordinary1.009134 or evolved1.008023 recovery at admission, Jump0, Skill.531889 and movement at AnySkill. Later form changes do not rewrite an admitted dodge. Exact owner gates, reached pauses, pending-effect cancellation, real command identity and one assumed-success Critical Dodge permission retain their existing models; Back itself adds no damage, Crime/Malice, immunity or free Riposte. Profiles on non-Dash commands, unsupported owners and unknown directions diagnose. Public source lowering remains Phase3; native request transport preserves this explicit intent.",
    },
    Hypothesis {
        id: "weapons.gold_wool_chaos_realfinish_v1",
        description: "For Chaos's accepted ordinary owner clock, What's Desired retains its native permanent Lakshana bonus and one self20s40/50/60/70/80% CRIT DMG window, but selects the recovered callback distinction: ordinary Skill grants on successful REALFINISH at projected full recovery, while Burst grants on Q_SKILL_BEGIN/accepted commit. This overrides only the default gold_wool_v1 Skill-admission trigger for that selected profile. Finish is anchored to the real admitted command and owner montage, projected through reached positive-hit and inherited pauses; cancellation, exit or defeat before completion removes it. A rejected or cancelled Skill grants no new window, without deleting an already granted window. Equal-time ordinary next inputs see a completed grant; preexisting ordinary immunity/resource/mark clocks are unchanged. Refresh is once under the same nonstacking key, not admission plus finish, and world expiry/field-exit persistence remain. Other owners and legacy caller-timed inputs retain their separately disclosed default trigger. Native callback distinction is selected source evidence; real-clock conversion and lifecycle are checked owner-model semantics.",
    },
    Hypothesis {
        id: "chiz.exact_grain_v1",
        description: "Chiz's selected financial model keeps own Grain as an exact bounded rational ledger, using the character-level frozen capacity and acquisition curves with constant-left-key selection. A1 multiplies capacity by1.1 and A4 ordinary acquisition by1.2 without rounding their fractional results. Normal cadence and positive-HP once-per-input eligibility retain chiz.normal_grain_v1. A5's first battle-entry grant credits one unmultiplied capacity through the ledger, with initial savings zero; repeated field entry does not regrant it. Exact granted/current/spent/overflow amounts reconcile in fractional_resources.chiz.grain, separate from Fons, Ultimate Energy and Esper Cycle. This prerequisite does not enable unbound non-Normal acquisition, loans, market, interest or held Skill settlement, whose missing handlers continue to diagnose.",
    },
    Hypothesis {
        id: "chiz.charge_passives_v1",
        description: "Chiz's ordinary breakthrough unlocks bind Hoarding at B4 and Hanahaki at B2. Hoarding adds exactly.20 percentage points to her own received Ultimate Energy efficiency only while on field; initial entry, exit and re-entry update one contribution without stacking or multiplying an existing efficiency. All actual ordinary, shared, external and Charge generation retain exact carry/cap/overflow using the live receiver multiplier; explicit fills remain unscaled. The currently bound No Grain Skill1_1 primary uses its native.966 ChargeAdd with ordinary source/ally distribution after positive HP loss, including off-field afterimage delivery; other unbound Skill/financial packets remain separate work. A living unlocked Hanahaki provider, including an off-field Chiz, adds4 raw energy only to the active living actor's actual Blossom-pistil-on-Remora Charge event,10 to14 before that recipient's efficiency, once per pistil rather than per target. Multiple eligible providers do not stack this owner effect. No Charge means no Hanahaki energy; Cosmos hits, Grain acquisition/spending and loans do not trigger it. Unlocks use actual character progression, not awakening count. This is the selected owner/shared callback model, independent from the still unbound Skill and financial market handlers.",
    },
    Hypothesis {
        id: "chiz.ordinary_clock_v1",
        description: "Chiz's selected ordinary Normal clock projects the five native montage inputs and twelve primary requests at one actor-progress second per real second. Exact nearest-microsecond coordinates replace caller hit/recovery values. Normal1/3 positive HP contact schedules one delayed.0167s pause with duration.06 and rate.01; each Normal5 contact uses duration.1/rate.01. Pauses are once per causal request, never per target, and only their real onset changes projected unreleased coordinates/recovery; a miss adds none. Normal4 releases one finite five-pulse actor at progress.236106 with real.15s intervals, independent of subsequent montage pauses/field exit, while defeat cancels future owner requests. Full connected recoveries are.590609,.783916,1.042182,1.099458,1.395803, totaling4.911968s. All eligible living stationary dummies receive full coefficients; no target division or geometry. Source-bound energy/Grain retain causal request/input deduplication, and native Normal Cycle rows accrue27.098 per complete chain without multiplying by recipients or Ultimate efficiency. Continuity lasts full recovery plus2s; a different action or field exit resets stage, same-actor switch does not. Ordinary invalid recovery inputs diagnose rather than wait. This bounded profile leaves non-Normal caller-timed actions under their existing explicit limits; it does not certify Skill stocks, financial/market, holds or secondary controls. Real-time conversion/default pause applicability are selected owner-model semantics, not measured native timestamps.",
    },
    Hypothesis {
        id: "chiz.exact_finance_v1",
        description: "The current Chiz own-wallet/Surplus prerequisite preserves exact capped Grain with up to4096-bit rational components, including the unchanged initial A5 grant and per-input Normal deposits. Selected A2 credits10% of the live positive own balance once per World second, starting from initial living Surplus entry at0; the source.5s bookkeeping is not two grants or half-second compounding. Timers and caps run off field, without A4/R3 acquisition multipliers on interest. Empty wallets schedule no polling; a later deposit uses the retained Surplus phase. Source defeat stops future credits and revival starts a fresh living Surplus phase through real HP observations. ModePause interest precedes same-time HP grants. The next World phase remains queued for clock projection and is nonblocking maintenance beyond a time-run endpoint. Exact resource summaries keep safe numeric components; otherwise every fraction component in that summary is a decimal string, preserving native/WASM/Worker transport without floating-point conversion. The currently admitted paths cannot borrow or invest; Skill stocks, money overrides, loans, Debt transitions and market settlement still require their separate handlers and financial Burst remains unsupported. This is the selected finite resource/clock model, not recovered native mutation bodies.",
    },
    Hypothesis {
        id: "chiz.pressed_skill_v1",
        description: "Chiz pressed No/Half/Full Grain consumes one of three stocks per accepted canonical Skill input; sequential real3s refill starts on first spend and subsequent spends retain the next deadline, including off-field/dead source time. Funded presses bypass the historical ordinary Skill cooldown. No/Half/Full contains2/2/4 source ATK requests, finite montage coordinates and per-connected-request delayed native hit pauses, scoped once over all living all-in-range dummies. Recovery, another action, field exit and the inclusive2s post-recovery combo window govern the next actual input; committed Skill trains retain their cast identity off field, source defeat cancels unresolved damage/settlement without refunds. In the currently admitted outside-market Surplus slice, a real input reserves one.2s financial callback reading the live exact own wallet; its one non-consuming Coin1 captures floor(proceeds) and delivers alongside No Grain's second, Half's first or Full's first primary. Zero proceeds produce no coin request. Coin1 replaces the entire stat/coefficient base, uses live damage/DEF/RES zones, grants no ordinary channels, and cannot crit. A6 adds.15 only to actual coin effects; selected R6 applies the source.05s target Cosmos RES-.10 before the causal Chiz calculation. The ordinary clock and exact-finance models are required. Held routes, loans and market settlement remain separate unadmitted mechanics. This is the selected finite clock/financial model, not recovered native mutation bodies.",
    },
    Hypothesis {
        id: "chiz.held_skill_v1",
        description: "The current Chiz outside-market held Skill prerequisite requires the pressed Skill, ordinary clock and exact-finance models. A canonical Skill hold of at least2.313136s reserves two real stocks for No Grain and held Half; finite No montage completion starts held Half, and its1.033255s reached boundary starts a real finite16-pulse loop. Native loop coordinates subtract their5.983333s source origin. Loop entry captures available Grain/16 once; each real pulse spends min(captured,current available), own Grain before remaining loans, then emits ordinary.428ATK and positive Coin2 base3*floor(actual spend) once over all living dummies. Requested release or natural3.733333s loop end consumes any remaining available Grain once as terminal Coin2, with no fabricated primary, then pays.449393s exit recovery. Borrowed principal persists in a separate exact capped Debt ledger; ordinary acquisition repays Debt first, with additive A4.2 and selected R3.5 efficiency read before the grant, then credits own savings. A2 runs only in living Surplus, stops on first drawn loan and restarts its real phase when Debt is repaid; loans, stocks, Energy, Cycle and Fons remain separate. The hold captures only its initial non-consuming eligible Surplus Coin1 and does not settle again at Half or every loop pulse. Committed requests remain source-owned off field. The initial No Grain avatar recovery, including its actual connected pauses, gates an ordinary switch; subsequent committed Half/Loop afterimage clocks do not extend that switch gate. A separately input Full retains its own recovery. Defeat cancels unresolved owner admissions without refunds and retains loan debt and real stock refill. Explicit three-stock reservation/early third-charge Full and market transitions require their separate models. This is the selected finite financial/clock model, not recovered native mutation bodies.",
    },
    Hypothesis {
        id: "chiz.held_full_v1",
        description: "Chiz held Full extends held_skill_v1 with positive short holds, explicit reserved_stocks2/3 and a real later canonical Skill press admitted through No/Half Skill gates or an open two-stock loop. Reserving3 pays all three stocks at the initial input; natural finite loop completion settles any terminal remainder then directly starts held Full without the ordinary loop exit segment or a second Coin1. Explicit release before natural completion cancels the reserved Full without refund. Release at or before reached Loop entry pays both committed initial segments and their full recovery without entering a loop, drawing loans or terminal spend; a separately paid buffered third input still reaches its Full gate without fabricating loop costs. Release after Loop entry retains the ordinary terminal spend/exit. A separate tap in a two-stock hold requires the live No Skill progress.632043 or Half Skill progress.669036 gate (both including actual owner pause), or the reached open loop, then reserves one currently available stock, retains the actual later command identity and starts held Full at max(input time,loop entry+.268039s). Before the reached Jump gate the input is a finite owner buffer; existing pulses through the gate retain their actual costs. At the transition, cancel future loop admissions without terminal spend, preserve the exact remaining available Grain/debt, and emit only held Full's four source ATK requests at progress.252173/.751256/.865237/.949538 with recovery1.499861 plus actual delayed.1s/rate.2 Skill4 pauses. The complete held train has only the first activation's financial snapshot, and all Full primaries retain native Charge/Cycle without Grain acquisition. Pending duplicate finish/hold requests reject; missing stock is an ordinary typed resource failure, never an automatic recharge wait. Stocks, loan debt, callbacks and captured native talent remain continuing across switches; source defeat cancels unpaid future damage without refunding reservations. The selected finite transition/cancel policy is explicit and does not claim recovered native C++ bodies.",
    },
    Hypothesis {
        id: "chiz.action_cancels_v1",
        description: "Chiz ordinary action cancels extend ordinary_clock_v1 and pressed_skill_v1 with native owner input gates rather than replacing real combat time. Pressed No/Half next-Skill progress.632043/.669036 and Normal/pressed/held-Full Jump gates use the actual source cast's reached montage clock, including only actual connected delayed hit pause. A replacement pressed cast inherits every still-live owner pause window from the preceding active pressed cast, so No's legal.954108s Skill cancel during its second.2s/rate.1 pause retains42.15ms of real pause and37.935ms lost new montage progress. Financial captures use their finite owner context; interest, stocks and persistent producers follow paused World time. Replacing an ordinary pressed montage or cancelling it with Jump removes its unreached future requests; Half cancelled at its.669036 Skill gate loses the still-unreached second.966861 primary. This follows its two direct montage notifies, not an already released actor. Field exit retains the separately selected committed Skill afterimage policy. Remaining causal hits and money snapshots keep their actual original command identity. Normal Jump cancels unreleased future montage packets while preserving an already reached finite Normal4 producer; another input clears the corresponding combo pointer without refund. Jump is a declared cancel input and does not infer airborne height or a plunge. Held-third buffering remains owned by held_full_v1 and has precedence. This is the selected finite native gate/clock model; no implicit wait, readiness polling or free damage stage is introduced.",
    },
    Hypothesis {
        id: "chiz.secondary_actions_v1",
        description: "Chiz secondary actions require action_cancels_v1 and the ordinary/pressed clocks. Authored positive airborne landing delay and height bonus0..1 admit the native Cosmos plunge, multiplying its primary by1+h with contact/recovery progress.199456/.500015 after real landing and connected.15s/rate.01 pause. Explicit front/back Dodge pays15 authored combat stamina, keeps its real.9s cooldown, and uses source recovery.998284/1.012579 with Jump.082544/0 and Skill.333716/.350074 gates. A paid successful Critical Dodge grants one next-Normal Critical Riposte, cleared by use, another non-Normal input or field exit; its two requests reach.150950/.333387 with recovery1.026147 and second-hit.2s/rate.08 pause, retaining the source zero Skill/Jump gate. Actual connected pause callbacks survive montage cancellation and affect only that owner's current avatar cast while on field; detached afterimages retain local clocks. Plunge grants one common level-selected Grain acquisition; each connected Riposte row grants once, never per victim or height multiplier. Native Charge/Cycle remain independent and use their actual row. Qualified engine Support consumes the real outgoing100Cycle switch, emits one Cosmos QTE at.800160 with base recovery1.736720, and adds only the independently integrated160558us lost progress of its actual delayed.3s Hermite pause when positive HP occurs; all admitted QTE gates lie after that curve ends. No second QTE or inherited Female051 packet is fabricated; subsequent authored Normal1/plunge covers the matching Appear/Pre route. QTE grants8.6 raw Charge and no Grain/Cycle. Raw Riposte/QTE inputs and missing cost/airborne authorization diagnose, with no readiness polling or free Critical Riposte. This is the selected finite native clock/recipient model; no geometry, enemy AI or recovered protected C++ body is claimed.",
    },
    Hypothesis {
        id: "chiz.market_v1",
        description: "Chiz's selected finite financial market requires the exact finance and pressed Skill models. Burst costs100 with15 World seconds cooldown, releases six source-owned packets and enters market at progress3.420264 for7 World seconds. Remaining loans convert once, with separate conserved own/debt/invested ledgers and market capacity equal to both caps. Each real Skill input captures one current signed return and settles once after.2 owner-action seconds, revaluing live principal, capping actual proceeds and retaining fractional balances; held continuation never recaptures. Coin1 floors captured proceeds, its first positive market request shares one forced-critical token across recipients, and queued amounts survive repayment. A2 uses forced Surplus while invested; R3 Debt acquisition never applies there. Exit closes new transactions immediately, completes already reserved settlements before repay-first withdrawal, and retains residual debt; defeat cancels unresolved transactions. Fourteen returns are precomputed with28 PCG32 draws on a continuing owner stream separate from critical RNG, sign probability.5 or normalized.6 at A3, conditional selection from the nine positive/five negative source entries retaining duplicates. Chart updates every.5 World seconds never revalue funds by themselves. Expiry precedes equal-time interest/input. This implements the checked inferred financial model and stationary dummy TimeSlomo scope, not recovered native mutation bodies or measured native clocks.",
    },
    Hypothesis {
        id: "chiz.recorded_market_trace_v1",
        description: "Authored Chiz scenario override: each market uses the recorded fourteen signed returns .25,-.08,.38,.50,.14,-.29,.19,.06,-.28,.32,.25,-.08,-.32,.23 in that order. The continuing finite generator still consumes its28 draws per entry. This explicit reproducible trace is neither native RNG evidence nor an A3 statistical claim and requires chiz.market_v1.",
    },
    Hypothesis {
        id: "chiz.barren_v1",
        description: "Chiz Barren requires the secondary-action and ordinary clock models. Canonical Normal with an explicit zero-duration hold input selects tap Barren: a non-damaging.500015s cue precedes its two-phase source train and starts one independent2.5s Cheat limiter at admission. A positive Normal hold selects combo Barren only through the live preceding Normal AnySkill gate and its2s continuation; its boolean hold intent adds no invented numeric windup and does not reset the tap limiter. First phase captures the still-living locked target, falling back to lowest living index. Branch2 hits that one target at.749310, Branch2_1 hits other living targets at.750524 and its final AoE at1.464538 hits all living targets. Capture persists if the first hit kills the lock. Full recovery is1.912478s for the train, Jump gate.868260; cue cancellation occurs at its own.500015 boundary before automatic train entry. Charge4.8/Cycle8 accrue once per nonempty request, Grain once per connected whole branch, never per dummy or per child pulse. Unreached montage requests and cue entry cancel on replacement, switch or defeat; a connected cost/grant is never refunded. No unbound Branch_Damage row, account-Fons farming, geometry or hidden vulnerability is fabricated. This is the checked selected source-route/clock/recipient model, not recovered native runtime bodies.",
    },
    Hypothesis {
        id: "chiz.battle_entry_v1",
        description: "Chiz's explicit combat leave/re-entry events preserve all continuing Grain, debt, Skill stocks, declared World/action timers and pending owner effects. Initial battle entry has ID0; each real re-entry has its stable event ID. Selected A5 deposits one character-level/A1 capacity per distinct battle entry, without A4/R3 efficiency, never on ordinary field entry and never twice for a repeated callback. Outside market the ordinary exact deposit pays outstanding debt before crediting capped own savings; an active market uses its forced-Surplus deposit route into capped invested principal without an immediate repayment or another credit line. Input/repaid/credited/overflow components are exact decimal fractions in resource_deposits, independent of Fons, Energy and Cycle. Unselected A5 grants nothing; repeated authored transitions must alternate from the initial in-combat state, and new transitions require a living owner. Leaving combat alone neither erases debt nor resets a financial or stock ledger. This is the checked selected entry/deposit model, not a recovered native mutation body.",
    },
    Hypothesis {
        id: "daffodil.ordinary_clock_v1",
        description: "Daffodil's selected finite Normal, ordinary/Echoes Skill and Finale montage coordinates advance on real combat time, rounded once to nearest microsecond. Each connected causal request adds its integrated native duration*(1-rate) lost progress only to this owner montage, once across all dummy recipients. Unconnected requests add no pause. Every request retains its native row, captured talent and distinct energy group; AoE does not multiply Charge or Cycle. Normal continuation lasts through actual recovery plus2s. Skill's16s cooldown begins at its reached ordinary.182290/Echoes.110328 marker; Finale costs120 and starts20s cooldown at admission, applying self Eyes and target Insight at progress0. Replacement/field exit cancels unreached montage requests without refund. This is the checked source-coordinate product model, not measured animation timing or a global time stop; secondary actions and entry have separate acceptance.",
    },
    Hypothesis {
        id: "daffodil.action_cancels_v1",
        description: "Daffodil action cancels require ordinary_clock_v1. Jump and Skill use the selected native stage gates through the same owner lost-progress clock; an ordinary invalid early input does not interrupt. Accepted Jump/replacement removes unreached montage requests and resets the Normal pointer without refunding costs or captured Resonance. A new montage inherits any still-live owner pause, while World deadlines retain their exact remaining duration through shared pauses. Explicit forced interruption releases recovery and cancels pending montage work; pre-notify Skill starts3s cancel cooldown at interruption, post-notify retains its original16s deadline. A completed cast is not interrupted retroactively. This binds finite source gates and causal cancellation, not airborne geometry, paid Dodge, suspended combo or a recovered native interruption body.",
    },
    Hypothesis {
        id: "daffodil.secondary_actions_v1",
        description: "Daffodil secondary actions require ordinary_clock_v1 and action_cancels_v1. Plunge uses authored positive landing time and height0..1, one native.424ATK level1 landing request, recovery.281249s and connected.0396s lost progress. Ordinary front/back Dodge pays shared declared stamina15 and its selected1.014541/1.007068s owner recovery, with immunity only until that action is cancelled or completes. One explicit successful paid Critical Dodge authorizes the next canonical Normal as Close Call, one1.04ATK level1 request with.874201s recovery and connected.297s lost progress; another non-Normal input, field exit or defeat clears authorization. Qualified adjacent-element switching admits one actual Support/QTE instead of a raw authored packet, at.745033s with.900691s recovery plus connected.34965s lost progress and.863029s switch lock. Its successful cast grants Resonance once regardless of target count; packet Charge5.6/Cycle0 remain separate from Riposte2.6/4.3 and plunge2.7/4.4. Causal AoE requests and inherited pause use the same owner clock. Suspended Displaced Edge, held Normal and charge-funded Phantom Step remain separate routes. This is the checked stationary dummy product model, not measured game animation or inferred automatic incoming success.",
    },
    Hypothesis {
        id: "daffodil.phantom_step_v1",
        description: "Daffodil Phantom Step is funded only by a successful Finale progress-0 reset to two uses with one World-time30s admission handle. A distinct-actor switch to a living Daffodil while the handle and a captured living target are live selects one Entry1/2/2_1/3/4 train; A2 adds Entry3_1. When a qualified entry also admits Support, use the QTE owner clock and the guarded AppearAttack transition at 1.150000 seconds of montage progress to start the full Step train. A successful pre-transition Support parry settles its finisher first. Each admitted Step spends one remaining charge. An explicit successful Daffodil parry before the Step final marker selects one AppearSuccess finisher; without it A2 alone selects that same single finisher. The one charge is committed at admitted entry and is not refunded by interruption, switching away or missing. The first positive finisher connection fills only Daffodil's energy pool to capacity once; other AoE recipients cannot duplicate it. The entry target and route are captured at admission. The owner-only Step clock uses source contact coordinates, pauses, and recovery; field exit or defeat cancels unreached packets while the independent charge deadline follows World time. The declared stationary-dummy model supplies no geometry or enemy response.",
    },
    Hypothesis {
        id: "daffodil.held_normal_v1",
        description: "Requires Daffodil secondary_actions_v1. Canonical held Normal selects the nine native Mirror on Water requests with no invented minimum hold tax. Neutral hold starts at ordinary readiness; combo hold can replace the active Normal at its Jump gate, cancelling unreached montage requests while preserving already emitted finite actor contacts. The Branch has recovery1.433323s and final connected .1188s owner pause. A front paid Dodge during a live Normal at that same gate is Displaced Edge: suspend its remaining montage progress for1.014541 real seconds, preserve its stage and shift only unreached requests and recovery. Existing connected pause overlaps using the shared strongest-slowdown clock. No landed packet is replayed; world timers/resources continue. Immunity ends at Dodge completion or an admitted early cancellation; cancelling the suspended action clears its pointer and remaining suspension. Ordinary back Dodge and unrelated actions retain ordinary cancellation. This checked stationary-dummy clock model does not infer incoming attacks, geometry or measured animation speed.",
    },
    Hypothesis {
        id: "combat.discord_break_gauge_v1",
        description: "A newly admitted Scorch/Nova generation pair on the same living target emits one Discord callback, deduplicated by that pair; periodic ticks, readiness checks and reuse of the same pair emit none. A breakable target must explicitly declare discord_fraction in (0,1]. Discord subtracts original capacity times that fraction times the target's accepted incoming Break efficiency from the current non-Broken gauge, clamps at zero, then enters at most one causal Broken transition. A Broken target records the callback and loses no gauge; a no-bar or defeated target has no gauge to lose. This selected content model creates no HP packet, critical roll, ordinary damage modifier, resource grant or recursive reaction and does not establish a universal native Discord percentage.",
    },
    Hypothesis {
        id: "daffodil.discord_cap_reduction_v1",
        description: "For the supported fully unlocked Daffodil profile with Blade Draw B2 active, each real Discord generation-pair callback on a breakable target adds one stack of 0.10 original-cap reduction, capped at two stacks, and refreshes one shared 30-second deadline. Apply its capacity change before that Discord's direct gauge deduction; on an unbroken target preserve prior gauge depletion by adding the capacity delta and clamping to the new cap. If the cap step reaches zero, use one causal Break transition and skip that callback's later gauge deduction. While Broken, update only future capacity; recovery restores that capacity. Expiry restores original capacity while preserving depleted gauge, and accepted reductions survive source field exit/defeat until expiry. MMO getter output remains unmodeled; this is the selected non-MMO coefficient, not a universal mode value.",
    },
    Hypothesis {
        id: "edgar.full_kit_v1",
        description: "Edgar uses the selected stationary-dummy full-kit model: finite native Normal/plunge/riposte/qualified Support clocks with connected owner hitpause; Skill taps emit one locked-target pulse and held channels emit independent damage/healing events every0.6s from0.8219797/0.8217107 until release capped4.9s. Channel cooldown settles16s plus1.676 times actual loop elapsed, with A2 extra cap3s and bounded cancellation penalties. Living injured allies are healed by lowest absolute HP, stable slot tie; A3 checks preheal HP<=30%. Skill heal requests grant one B4 key through a nonrefreshing6s gate, actual Support grants keys, and Burst captures/clears keys only at its2.7509558s field spawn. One field emits on World seconds through its inclusive ObjectLifetime endpoint, persists off field and is invalidated by replacement/death/combat exit. R6 accumulates exact actual team HP loss normalized by loss-time MaxHP/.03, capped20, and paid Burst captures floor/reset before a CE-scaled self refund. B2 replaces a qualifying pistil Charge with120 raw energy and suppresses further pistil grants to that recipient for30s. These event clocks, selectors and lifecycle choices implement the checked owner card; they are not measured native execution or geometry.",
    },
    Hypothesis {
        id: "fadia.incoming_share_v1",
        description: "Living Fadia's selected incoming model splits explicit already-mitigated combat damage before recipient shields: a teammate hit while she starts above1HP routes0.50 to Fadia (A1 replaces this by0.75). While another party member lives she retains a captured1HP floor for that incoming event. A hit targeting Fadia at1HP instead sends the same total share to other living party members equally; only the next incoming event can select a new branch. Children retain the root event identity, receive their own shields/effects once and cannot recursively redistribute. The redirected whole-HP pool floors once; its original target retains the remainder, and reverse-share integer remainders go to ascending party slots. Original-target immunity/death suppresses redistribution. Actual child HP loss, shield/effect absorption, floor prevention and overflow reconcile to the original delivered amount. Self costs and structural MaxHP clamps do not enter this enemy-damage splitting path. This is a checked stationary-scenario model, not enemy AI or a native mitigation calculation.",
    },
    Hypothesis {
        id: "fadia.ordinary_clock_v1",
        description: "Fadia's five ordinary Normal inputs deliver the native1/2/1/2/2 packet train to all living in-range dummies at reached montage coordinates. Connected owner hitpause uses exact piecewise-linear source rate integration with strongest-overlap composition; delayed Normal7/8 pauses retain their real onset. The clipped active parts of constant source curves remain constant windows, with unused out-of-window knots excluded. Only unreleased montage contacts and recovery are reprojected; buffs, passive energy and cooldowns follow World time, while unrelated actor actions suspend under the shared cinematic/Time Stop policy. The chain loops and retains the next stage through recovery plus2s, resets on field exit or another admitted action, and cancels unreleased owner requests on interruption. Microsecond deadlines round upward only at clock inversion. This binds the finite ordinary Normal clock, not the remaining Skill/Burst/Lilith or secondary actions, and does not claim measured native animation speed.",
    },
    Hypothesis {
        id: "fadia.full_kit_v1",
        description: "Fadia follows the selected stationary-dummy owner specification: source-coordinate ordinary/Skill/Burst/Lilith clocks, connected exact hitpause and real resource timers; actual combat HP-loss threshold crossing fills the Ultimate pool, while Burst enhancement uses the distinct strict missing-HP comparison before its one self heal. Skill marks capture independent HP budgets and settle resolved noncritical transfer amounts without repeating damage zones. Lilith permission and admitted cast eligibility are separate. Source-coordinate cancels forfeit unissued contacts; paid front/back Dodge authorizes one explicitly successful Critical Riposte, authored landing/height bounds plunge, and shared qualified switching alone authorizes Support. Unqualified field entry is non-damaging. A3 baseHP, B4/R6 membership and capped reversible B2 Nova extraction retain structural HP changes separately from combat loss. These finite event, recipient, threshold and stat-transfer policies are checked product models, not measured native timing or geometry.",
    },
    Hypothesis {
        id: "haniel.nova_extraction_v1",
        description: "Haniel B2 listens once to each living target's ended Nova generation while she remains in combat membership, including off field and after source death. Capture source baseATK including weapon base at membership entry; each end grants0.08 of it as flatATK to every equipped ally, capped0.16 source-wide per recipient. One target loses half the accepted grant, capped0.20 of its explicitly authored initial ATK across sources; truncate the grant proportionally at either cap. Gains and target losses persist through switches, summon expiry and source death, and only this source's ledger rolls back on combat exit. No target ATK default, damage, healing, Harmony, Plot or resource award is inferred. This is the selected stationary-dummy model, not native execution or enemy AI.",
    },
    Hypothesis {
        id: "haniel.ordinary_clock_v1",
        description: "Haniel's ordinary Normal chain and direct Skill/Burst use the selected native montage coordinates rounded once to whole microseconds per action boundary. All living in-range stationary dummies receive whole coefficients; recipients do not multiply one causal packet's energy/Cycle opportunity. Skill contacts occur at0.345126/0.719512s, then one Hootie spawns after the latter settles; cooldown20s (A6 replaces16s) starts at0.684620s. Pre-marker interruption starts3s without A6 subtraction. Burst pays120 at commitment, starts20s cooldown, admits Ace/team buffs only at2.869166s and damage at3.843514s, recovering4.240809s. Queued Normal advances five stages only at the previous recovery boundary; idle, another action or field exit resets it. Unissued contacts/notifies cancel on interruption/exit, while existing summons/Ace and real energy/Cycle clocks continue. This binds the finite ordinary clock and reached Cycle values, not Cannon, plunge, paid Dodge/Riposte or natural Support, and does not claim measured game timing.",
    },
    Hypothesis {
        id: "haniel.cannon_ricochet_v1",
        description: "Haniel's stationary unreserved held Normal1 enters Cannon at0.2s, retaining its earlier Normal projectile. Shorter holds remain taps; a charged later queued stage diagnoses. Full Cannon emits at1.061263s and recovers1.211743s before connected pauses. During live Ace an early release waits for the strict0.4<progress<0.5 Branch window; the first representable point is0.400001s after Branch entry, with emission another0.044596s and recovery another0.195077s later. A release at/after0.5 misses that early window and full charge cannot bank extra hits. Snapshot persistent Plot at firing, without spending it. One primary hits the living lock (stable first living dummy); up to the captured4/A1 six child legs each choose the lowest living anchor other than the previous one and emit one uncapped full-coefficient AoE. No other anchor ends the chain without a phantom explosion. Zero travel preserves causal leg ordering, one energy/Cycle opportunity per leg and0.02s recovery loss per connected packet, not per recipient. Unissued charge/fire cancels; launched legs retain source/talent through field exit, while death/combat exit suppresses continuation. These clocks and routing are selected stationary-dummy inferences, not measured travel or native geometry.",
    },
    Hypothesis {
        id: "haniel.full_kit_v1",
        description: "Haniel composes the selected ordinary clock, charged Cannon, summon and reversible B2 models with explicit secondary controls. Common paid Dodge costs15 stamina with0.9s cooldown and the shared2s regeneration delay; front/back recovery0.836769/1.005710s, Skill gates0.333611/0.494283s and Jump0.094234/0s. Successful Critical Dodge arms one canonical Normal replacement, cleared by another action or field exit. Explicit parry is accepted only during a paid Dodge, arming that same response after0.02s without another packet or grant. Riposte emits one uncapped Z and immediate fully charged capped Cannon at0.4s, without a Normal opener/hold. Its base recovery0.957007s and Skill/Jump0.514085s acquire0.297s per connected Z and0.02s per actual Cannon leg, once per packet. Authored plunge lands after its explicit descent, emits0.065670s later and recovers0.175616s after landing, coefficient times1+height with height0..1. Engine-qualified outgoing100-Cycle reaction switches select one Support at0.712638s, recovery1.177904s plus0.210s connected pause and switch lock1.112165s; otherwise a living dummy permits one entry Normal1 with0.896381s recovery. Ordinary/entry/Skill/Branch Jump gates and cancellation use the selected source coordinates; only admitted controls cancel unissued contacts/notifies, launched Cannon and summons retain source ownership. World cooldowns/resources retain their remaining duration during shared pauses. These are disclosed stationary-dummy models and source-coordinate inferences, not measured game timing or enemy AI.",
    },
    Hypothesis {
        id: "hathor.ordinary_clock_v1",
        description: "Hathor's canonical zero-cost Normal follows the selected five-stage1/5/2/3/2 contact train and source next-ability coordinates rounded once to microseconds per input. All eligible stationary dummies receive full coefficients; energy, Cycle and pause occur once per connected causal packet, regardless of recipient count. Source pause onset16.666ms is separate for Melee1/2/3; their reached zero-rate intervals and other flat intervals retain native duration. Melee5_1 integrates the selected linear rate knots over133.333ms with constant endpoints, quantized to whole microseconds/integer PPM. Melee4_2's negligible native rate rounds to0 PPM; no geometry/global Time Stop is invented. Contacts, recovery and later contacts advance on this owner clock; world time, cooldowns and resource carry continue normally. Exactly queued recovery advances the chain, while explicit idle, another admitted input, interruption or field exit resets it and cancels unissued contacts. A miss causes no pause or resource award. This prerequisite binds Normal only; held/Branch/plunge/Skill/Burst and secondary state bindings remain separate requirements, and source montage coordinates are not measured game timing.",
    },
    Hypothesis {
        id: "hathor.tap_power_v1",
        description: "Hathor composes the selected Normal clock with canonical unheld Skill, exact continuing Power0..6 and accepted-delta rewards. Tap requests2 Power once at0.283333s, starts16s cooldown at0.300073s and emits Skill1 at0.516245/0.699503 plus Skill11 at0.916369 on the source action clock. Connected Skill1/11 integrate their selected linear rate curve over0.033333/0.083333s, once per packet rather than per recipient. Pre-marker interruption starts3s from interruption; paid/accepted Power already granted is retained, and unissued contacts/notifies cancel. Each accepted positive Power delta grants5 direct self Ultimate Energy, bypassing Charge Efficiency and sharing; clipped excess pays nothing and is retained in the Power overflow ledger. Breakthrough4 own applied-damage kills request1 Power after impact; another owner's kill or a zero-loss packet cannot grant it. Selected A5 records only actual accepted positive deltas, capped15, and retains earned count through source death while Power clears. Reporting periods and field exit reset neither the pool nor earned count. This prerequisite does not bind Gaze, held Skill/A1, Burst/Emergency/Cyclone, paid secondary actions or combat leave/reentry; reached A3 target classification remains a separate full-kit requirement. Source coordinates/curves are checked modeling assumptions, not measured game timing.",
    },
    Hypothesis {
        id: "hathor.gaze_v1",
        description: "Hathor's intrinsic ordinary Gaze uses explicit anonymous stationary-dummy proximity and attack-start assumptions, without enemy identities, skill catalogs or AI. Default proximity is true. Five uninterrupted unmarked seconds apply one nonrefreshing five-second generation independently per living dummy, including while Hathor is off field. Expiry or consumption starts a fresh five-second unmarked interval; explicit proximity changes reset accumulation, while an existing mark retains its original expiry. An authored AttackStart is a qualifying skill-begin directed at the named living party member, independent of whether any later damage lands; it consumes one generation and requests1 Power exactly once. Normal expiry requests1 only with selected A2. Consumption, target/source death and proximity changes do not count as natural completion; dead sources cannot rearm. Expiry precedes an equal-time attack start, and proximity changes precede accumulation completion. This composes tap_power_v1 and its accepted-delta cap/energy/A5 ledger. Enhanced Five-Star Tracking and incoming-contact provenance remain separate held-Skill requirements. Each meaningful deadline is scheduled once, bounded by the authored horizon; no readiness polling or damage is synthesized.",
    },
    Hypothesis {
        id: "hathor.held_skill_v1",
        description: "Hathor composes tap_power_v1 and gaze_v1 with stationary unreserved Skill holds: below0.22s takes ordinary tap; otherwise total hold caps at5s, actor birth is0.686667s and real ticks begin0.936667s every0.25s strictly before release. Full hold emits17 Power ticks and its ten-contact PowerEnd, while release at or below1s takes three-contact ShortEnd and requests tap2 once. Each End packet preserves its source coefficients, talent, resources and connected owner pause. Held cooldown starts16s at0.236652s if reached before release, otherwise at the ShortEnd0.016368s marker; pre-marker interruption starts3s and later cancellation retains the actual deadline. First actual held birth with selected A1 requests6 and consumes its once-per-battle marker even at cap; field exit/death/reporting periods do not reset it. Every actual timer tick applies one five-second nonrefreshing enhanced Gaze per eligible living all-in-range dummy, regardless of rounded HP damage, replacing ordinary Gaze without a completion award. Attack-start or explicitly dummy-attributed positive incoming HP/shield contact consumes one enhanced generation for3 requested Power, deduplicated by removal before callback; A2 alone pays3 on natural expiry. Release/interrupt/field exit/death cancels unissued actor ticks and End requests. The separate empty0.5s actor-table row emits no damage. All source coordinates round to integer microseconds; this is a selected causal model, not measured native timing. Burst/Emergency/Cyclone, remaining passive/secondary actions and combat leave/reentry remain separate full-kit requirements.",
    },
    Hypothesis {
        id: "combat.incoming_defense_v1",
        description: "An explicitly indexed before_defense incoming declaration interprets its amount after every externally chosen modifier except DEF. It requires an explicit anonymous-dummy instigator and uses that dummy's declared positive level, without a skill/profile catalog, AI, attack generation or extra RES/crit/element inference. Sample the original recipient's live base/static/flat/percentage DEF, including living-source buffs and supported conditional gear, once at delivery. Apply the shared exact factor (600+6*source_level)/(600+6*source_level+liveDEF), with no inferred shred/ignore, then round once to nearest whole HP with ties upward. Feed that conserved delivered amount through existing immunity, nonrecursive shares, shields and HP callbacks; redirected children do not mitigate a second time. The root receipt retains the pre-DEF input, dummy identity/level and sampled exact DEF separately from its post-DEF delivered amount. Missing basis or explicit externally_mitigated retains the declared final amount without this stage. Source binding alone does not imply DEF mitigation or an attack-start observation. This is an explicit checked fixture model for DEF-relevant mechanics, not proof of native incoming mitigation for every attack kind.",
    },
    Hypothesis {
        id: "hathor.burst_emergency_v1",
        description: "Hathor composes tap_power_v1 with canonical Burst spending100 at commitment and starting15s cooldown. Emergency applies at source1.499991s, before two collision hits2.483287/2.883287s and landing3.816348s; next ability is4.530561s. These integer coordinates bind the selected two-periodic-plus-landing model, not an extra immediate collision tick or measured game timing. Nonhybrid Emergency lasts8 World seconds from application, snapshots then-current accepted Power n as0.15*n base-ATK percent once, and exposes the same live ATK to damage and source-stat consumers. Spending/loss does not recompute that snapshot. Source field exit/death ends it immediately; reporting periods do not reset it. Positive generation requests during Emergency consume their actual Gaze generation but reject before the Power pool, record requested/accepted0 in trace, give no energy/A5 count and never defer. Selected A3 adds0.15 per living explicitly non-Boss dummy, evaluated live; selected A6 gives0.30 additional CRIT damage and R6 gives0.10 DEF ignore only while Emergency is live. Ordinary cooldowns and Gaze/gear/world timers continue. This prerequisite binds Burst/ordinary passives; Cyclone admission/spending/ATK-only retention and secondary/combat controls remain separate full-kit requirements. Skill input during live Emergency diagnoses its unsupported Cyclone replacement.",
    },
    Hypothesis {
        id: "hathor.hybrid_emergency_v1",
        description: "With burst_emergency_v1, select the checked hybrid interpretation: Emergency still applies at the actual state event, but its5s admission timer starts at initial Burst recovery rather than consuming that animation's remaining time. An explicit interruption after state application ends the initial animation immediately and resumes5s from that interruption; field exit/death ends state outright. Only this owner's Emergency timer is suspended: ordinary cooldowns, hits, marks and the real DPS clock continue. This is an explicit source-mode hypothesis, not a global Time Stop or measured native hybrid scheduler.",
    },
    Hypothesis {
        id: "hathor.cyclone_v1",
        description: "Hathor composes burst_emergency_v1 with admitted finite Cyclone stages costing1/2/3 actual Power without refund. Canonical Skill input during Emergency replaces ordinary Tap/hold, bypasses its continuing hidden cooldown and restores that original deadline at state end before equal-time admission. Six Power funds seven Skill5, three Skill6/7/8 and two Skill9/10 contacts; native GA_Hathor_Skill2 child rows retain ordinary purchased Skill talent plus resonance. Source coordinates round to integer microseconds and actual connected owner pause projects damage, success markers, combo windows and recovery on the real clock. Stage1/2 transitions select60fps notify-start offsets1.116684/1.433341s inside their authored windows, and never skip a stage or implicitly wait. A4 snapshots prior successful-stage count k as0.25*k CR for that stage only, with source counter cap10; positive damage plus Skill.1 marker1.066650/1.366748s increments once, stage3 increments at successful completion. Interruption before success or zero damage never increments. An admitted stage grants100 percent own base DEF through the shared live-DEF consumer until real finish/interruption/field exit/death. Emergency expiry immediately removes A4/A6/R6 and further admission; only captured ATK may remain through an already admitted Cyclone until its finish or2s safety deadline, whichever is earlier. Spending to zero does not erase already admitted stage effects or recompute ATK. World cooldowns/marks/gear/RNG continue. This checked causal model is not measured native timing; secondary and combat controls remain separate full-kit requirements.",
    },
    Hypothesis {
        id: "hathor.combat_lifecycle_v1",
        description: "With gaze_v1 and burst_emergency_v1, explicit alternating living-source combat exit cancels the current Hathor montage and all mark generations, clears Emergency including pending ATK and Cyclone DEF, drains current Power without a positive-delta reward, and resets the battle-once A1 marker and A5 lifetime earned count. Existing exact Power grant/spend/overflow ledgers remain conserved across the whole simulation; the exit loss has its own trace reason rather than a Cyclone payment. Ultimate energy, Cycle, ordinary cooldowns, world time and RNG are not reset. Field exit/report boundaries do not perform battle reset, and source defeat alone retains A1/A5 for the current battle. While explicitly outside combat Hathor cannot act or generate Power. Reentry starts fresh unmarked five-second proximity accumulation using the latest authored dummy proximity assumption; it does not grant a stack or retrospectively replay an attack. Gaze stores each next deadline even beyond the simulation horizon, validates generation plus actual due time, and exposes its planned deadline in trace without queuing background events past the horizon. This is an explicit checked lifecycle model, not inferred combat detection, enemy AI or a resource refill.",
    },
    Hypothesis {
        id: "hathor.secondary_actions_v1",
        description: "Hathor composes combat_lifecycle_v1 and cyclone_v1 with explicit paid grounded Dodge/Jump controls, one ready Critical Riposte response and engine-qualified Support or ordinary entry. Dodge spends shared15 stamina with0.9s cooldown; front/back any recovery is0.996277/1.009564s, Skill gates0.267501/0.331208s and Jump0.082544/0s. Only an authored successful Critical Dodge or explicit living-active parry inside that paid window arms one response. Riposte preserves two native Z packets at0.341403/0.649874s, recovery1.183548s, Skill/Jump cancel0; its first positive contact requests1 Power through a five-second nonrefreshing reward gate once across all recipients/contacts. The gate begins on the real request, including cap/Emergency rejection, retains its world deadline through field exit and clears at battle exit. Reward settlement follows the causal AoE packet, so A5 cannot boost later recipients of the same contact. New battle identity rejects old deferred rewards. Support uses two H contacts0.633462/0.824235s with consumed connected delays0.32/0.03299967s, recovery1.350067s before pause, Jump1.183819 and switch1.210463; it requires the shared outgoing100-Cycle qualifying switch, never a user-invoked free QTE. Ordinary entry uses0.216389s contact and1.100818s recovery. Normal/Skill/held/Burst/Cyclone controls use selected source jump/switch gates projected through actual owner pause, cancel unissued contacts and retain paid resources/cooldowns. Field exit/death removes response/on-field effects without resetting A1/A5 or the continuing world clock. QTE keeps its own native role and does not receive R3 Normal/Skill/Burst talent uplift. This is a checked stationary causal model, not measured timings; Branch/airborne descent and full owner acceptance remain separate.",
    },
    Hypothesis {
        id: "hathor.airattack_v1",
        description: "With secondary_actions_v1 and an explicit shared stamina account, Hathor's authored falling Normal uses one native AirAttack landing packet scaled by1+h for declared height h in0..1. Positive Strength is an admission guard, not a ten-point fee. Source entry0..0.45 precedes the self-loop0.45..0.95: the first10*use-efficiency cost is at0.616667 actor seconds, then every0.5 actor seconds. Each actual cost is capped by available Strength, conserves the exact ledger and restarts its shared2s recovery gate. Strength<=10 selects landing immediately at entry or after a cost; an authored real landing wins over an equal-time cost. Landing emits one X packet after0.116417 actor seconds, recovery0.584348, Jump0.484688; the AirAttack.2 control marker is not another hit. Explicit real airborne time is not extended by old owner dilation, while source cost coordinates and after-landing montage contacts use the continuing owner clock. Interrupt/switch/death/combat exit cancel unissued costs and impact without refunding actual spends. Positive connected damage pays native6.3 Charge and10.4 Cycle once across recipients; Normal talent/R3 applies. No geometry, inferred height, target motion or automatic enemy action is modeled.",
    },
    Hypothesis {
        id: "hathor.branch_v1",
        description: "Hathor composes airattack_v1 with held Normal Branch entry and its explicit target response. A Normal or ordinary entry hold opens Branch only when the declared duration strictly exceeds its exclusive source window lower bound:1.3s for Melee1,0.5s for Melee2/3 and entry,0.9s for Melee4,0.6s for Melee5. That lower bound is the causal gate: it cancels not-yet-delivered contacts of the interrupted stage but retains already delivered contacts, starts the Branch lock at admission and extends the owner lock to the gate when ordinary recovery ends earlier. A declared launchable dummy holds Branch/BranchLoop without stamina until the authored real landing, then receives exactly one BranchEnd contact at the landing instant with Normal talent/R3 coefficients and source recovery0.737229s, Jump0.549863s and switch0.753203s before its connected owner pause over83.333ms. A nonlaunchable dummy transitions at0.52s into the AirAttack1 loop: the first10*use-efficiency Strength cost is at0.616667 actor seconds, then every0.5 actor seconds, and Strength<=10 lands immediately at entry or after a cost; an authored real landing wins over an equal-time cost. Fallback landing emits one native AirAttack packet scaled by1+declared height after0.116417 actor seconds with recovery0.584348, Jump0.484688 and switch0.584348. Branch and BranchEnd admit no separate stamina cost; fallback spends use the shared exact ledger and 2s recovery gate. Skill/Burst remain rejected until the Branch lock ends; interrupt, switch, field exit, combat exit and death cancel unissued Branch contacts or fallback costs without refunding actual spends, and every end clears the lock. Each actual packet settles once across all recipients. No geometry, target motion, knockback direction or automatic enemy action is modeled; authored response and landing times are explicit stationary-dummy assumptions, not measured enemy behavior.",
    },
    Hypothesis {
        id: "hotori.katana_chain_v1",
        description: "While Hotori's World's Tide Time Stop is active, an admitted Basic input selects the five-stage Q-category Katana chain instead of the ordinary Normal chain: stage1 delivers one GE_Player_Jin_UltraSkillMelee1_Damage, stage2 one UltraSkillMelee2 plus one UltraSkillMelee2_1, stage3 three UltraSkillMelee3 plus one UltraSkillMelee3_1, stage4 one UltraSkillMelee4 plus UltraSkillMelee4_1..4_3, and stage5 one UltraSkillMelee5 plus one UltraSkillMelee5_1, all Cosmos with zero raw energy, Cycle and Break. Each admitted input increments the count marker once at its native marker time regardless of misses, up to exactly 10; inputs 1..10 select stages 1..5 twice and the count persists across both subchains. At exactly 10 the next Basic selects one finisher instead of a stage, does not increment the count, and ordinary legal inputs cannot reach 11. The finisher is one GE_Player_Jin_UltraSkillMelee6_Damage, replaced by one GE_Player_Jin_UltraSkillMelee6Pas2_Damage when breakthrough4 enables Passive2, and never both rows. Each input delivers its stage's local source hits at the selected real offsets to every eligible dummy under hotori.aoe_all_in_range_dummy_v1, with live talent and modifier sampling; inputs chain through the native queued-next windows, stage source pauses and Any recoveries, the finisher keeps its recovery in real time and ends the Time Stop benefits at its damage delivery. No Timepiece, stamina or conventional Ultimate Energy is paid per Katana input. Mode end or a new World's Tide resets the count, an already admitted stage may finish after expiry with live mode bonuses removed, no finisher is manufactured at count9 or on expiry, and after the mode the next Basic returns to ordinary Entry1. Count markers, stage input starts and the finisher hit are checked exact scheduling choices, not native-body observations.",
    },
    Hypothesis {
        id: "hotori.mode_damage_v1",
        description: "During Hotori's World's Tide Time Stop, Hotori's own packets gain an additive general damage bonus of 0.12 times the record count frozen at Burst commit when A1 is selected plus 0.20 when A2 is selected, and additional target DEF ignore 0.30 when A6 is selected. These are live Hotori-only modifiers: they never apply to replayed ally packets or fixed/no-bonus reactions, and they end with the mode at the finisher hit, resource deadline, death or cleanup. A5 adds 0.15 general damage for every living party actor against the sole living enemy while Hotori lives in the party, recalculated per packet so a later second target removes eligibility without retracting earlier hits. R6 at six awakenings adds ATK percent 0.30 of each recipient's own base ATK while Hotori lives, once per recipient and never stacked per field entry; the native-figure 0.30 supersedes the historical 0.20. R3 at three awakenings adds one effective talent level to Normal, Skill and Burst but not QTE. B2 keeps the Blossom Vita Bud attack clock running through active Time Stop without adding pistils or coefficients; its object lifetime advances with or without B2, and cinematics pause both; B4 at breakthrough4 selects the UltraSkillMelee6Pas2 finisher row in place of UltraSkillMelee6 as a coefficient replacement, never both rows and never a general +100 percent. A1 uses the frozen record count, so playback clearing the store cannot reduce it. No hidden max-HP heal is granted from the zero-ratio heal rows.",
    },
    Hypothesis {
        id: "hotori.recorded_replay_v1",
        description: "Present Replay's fixed five-second window records at most three distinct real allied owners in event order, only from an actual committed Support or Redirect action whose real owner is living and differs from Hotori. Normal, Burst, damage procs, replay proxies and repeated casts from an already recorded owner do not occupy a slot; Hotori may be off field, the window endpoint is excluded, and genuinely equal independent casts resolve by stable party index. A new Present Replay overwrites all prior entries even if it later records none, and records persist until overwrite, Burst playback, combat exit or Hotori's death. At World's Tide commit the record count freezes for A1; at commit+4.216431 the stored entries dispatch once in recording order as complete owner-owned Support or Redirect actions with their normal packet classes, elements, hit schedules and positive-hit energy/Cycle, using the recorded ally's live stats, buffs and target state. Virtual admission bypasses its original field, cost, cooldown and one-shot permission without mutating the actual ally's chain, resources, cooldown or field ownership, and without a new party switch, QTE payment or switch reaction. Semantic source cast listeners run once, copied actions cannot recursively record, dead owners are skipped at playback while already dispatched payloads follow that owner's normal death and expiry rules, source-owned summons and auras refresh under the original owner cap, and the store clears exactly once with no invented spacing between entries. This model does not establish native proxy placement or a native cooldown body beyond the named rules.",
    },
    Hypothesis {
        id: "hotori.time_stop_clock_v1",
        description: "World's Tide commitment opens its generation-owned cinematic pause; entry at action progress+2.183333 seconds transitions the same owner to active Time Stop, ending at the earlier finisher delivery, reserved deadline entry+12 seconds, forced cancellation or death. Overlapping pauses count once in relative combat time. Shared paused-world timers include ordinary cooldowns, buffs, status and periodic phases, passive resource recovery and incoming attacks; retained remainders resume without backlog. Only the finite committing cinematic and permitted Katana/replay action contexts advance through their own supported pause, and unrelated cinematics still dominate. Reserved-stop drain is independent of local hitlag. Deployed object lifetime advances during active stop while ordinary emissions pause; living B2 exempts Vita Bud emissions only, with lifetime and finite count still consumed. Explicit HP costs, grants and removals keep causal ordering. Damage rates use actual relative elapsed time; absolute timestamps retain event order. The conservative shared policy and object-lifetime distinction are explicit model assumptions, not observed native timer bodies.",
    },
    Hypothesis {
        id: "hotori.timepiece_v1",
        description: "Hotori character:1052 maintains one exact Timepiece account in[0,120] separate from conventional Ultimate Energy, Cycle, Charge and stamina. It starts at combat entry at 0; rotation boundaries never restore an initial value, and switching preserves it together with records, completed recording and paused-world cooldowns. Living Hotori recovers exactly 6 units per paused-world second whether on or off field, except while rewinding or during Burst setup and Time Stop; recovery clamps at 120 and reports the wasted overflow with no sixth-second polling. Present Replay requires living on field, ordinary permission, no active rewind or Time Stop, available_time strictly greater than 60 and paused-world Skill cooldown ready; at commit it spends exactly 60 once, suspends passive recovery, invalidates previous records and any pending A3 refund, and opens a fixed five-second recording interval at Skill+0.16747499 whose 60-unit rewind reservation settles at Skill+5.16747499 without a second deduction, resuming passive recovery with the post-payment remainder. World's Tide requires living Hotori on field, ordinary permission, no rewind or Time Stop, available_time equal to 120 and cooldown ready; at commit it atomically reserves all 120, stops passive and A3, freezes the record count for A1, and resets A4 and Katana counts; the reserved budget drains from setup entry at 10 per second, a finisher ends the mode early discarding the remainder, and interrupt or death before entry spends the committed 120 and cancels pending entry and replay. A3 selected starts an additional 6 per second refund of exactly 60 after a completed recording and full rewind settlement when its 60-second ICD from refund start is ready; any new Timepiece expenditure cancels the outstanding remainder while ordinary hits and switches do not. A4 grants 10 Timepiece per credited team kill while living and not rewinding, not during Burst reservation, setup or Time Stop, up to 3 accepted callbacks since the last actual World's Tide commit, counting the callback at cap with overflow lost and resetting only on World's Tide; kills during rewind do not consume an opportunity. Death cancels recording and Time Stop, discards reservations and disables passive recovery until revive, which keeps the retained pool. Held branch control, native-body deduction timing, copy-cooldown branches and any geometry interaction remain outside this model.",
    },
    Hypothesis {
        id: "nanally.cycle_followups_v1",
        description: "Nanally with breakthrough4, living and active Authority reacts to positive settled team Esper Cycle damage with one Fair Duel follow-up at effective Normal talent, native level1 coefficient0.6, and an independent source-owned2-second guard. The triggering live target is captured, otherwise the lowest-index living target; a later defeat cancels without retargeting. Owner state is enabled by the real Authority Skill command and the exact actual damage receipt; both parents are retained by the shared autonomous coordinated-hit ledger. Ordinary damage does not directly trigger Fair Duel. Nanally-owned Cycle damage and connected Fair Duel can trigger her selected A3 under its separate1-second guard, while actual own Hexed follow-ups and Fair Duel request A1/A5 once through shared callback accounting. Queries do not trigger these effects. Native rows and published descriptions ground coefficient/guards; positive-HP-loss qualification, captured recipient, source command lifetime and ordered same-time dispatch are selected execution policies, not frame-verified native behavior.",
    },
    Hypothesis {
        id: "nanally.blossom_enhancement_v1",
        description: "A living Nanally unlocked at breakthrough2 enhances each newly created party Blossom Bud to ten Vita Pistils at one-second intervals, first emission one second after birth. The native extension row's five additional bullets and the published ten-pistil/one-second description ground count and cadence; the saved negative countdown field alone is not a timer. Party-wide living-provider eligibility at creation, captured generation schedule and independent Cycle damage ownership are selected policies. Switching or later loss of the provider does not rewrite an existing Bud, while newly born Buds recheck eligibility. Base five/two-second behavior, owner level/CI snapshot, three-Bud cap, Jiuyuan's separate cap/count extension, replacement cancellation, target settlement and copied-pistil rules retain their own owners. Duplicate providers do not stack. No Authority, Underboss, awakening or extra resource payment is required. This is a revisable execution model, not frame-verified native behavior.",
    },
    Hypothesis {
        id: "nanally.combat_lifecycle_v1",
        description: "Explicit alternating leave/re-entry events maintain living Nanally's own combat state, initially in combat. Selected Effect6 captures Authority duration at application:15 World seconds in combat,20 outside; without Effect6 it remains12. A later transition never refreshes or rescales an existing Authority deadline, while field exit, death and explicit removal retain their separate termination rules. This application-time interpretation follows the configured duration calculation rather than assuming an unobserved live-duration refresh. Combat exit clears A5's ATK modifier and stack pool; off-combat follow-ups cannot regrant it, and re-entry starts no stacks or free energy. Combat transitions do not spend/refund energy, reset cooldowns, change Cycle ownership, refresh Underboss or fabricate attacks. The shared explicit event and report ledger retains actual time and actor. Durations and combat-exit removal are source-grounded; duration capture, no automatic combat inference from a hit and outside-combat stack gating are selected revisable policies, not frame-verified behavior.",
    },
    Hypothesis {
        id: "nanally.authority_removal_v1",
        description: "Nanally Skill's ordinary sixteen-second cooldown begins at the reached native425571us progress notify through action_clock_v1. While living Nanally has live Authority, an authored Skill selects source-granted GA_Nanally_RemoveSkill after its own two-second gate from Authority application, instead of waiting for or replacing ordinary cooldown. A fresh rate-one E applies Authority at429917us, hence unlocks removal at2429917us. Field, life, recovery and monotone-time admission still apply; readiness reports the selected gate. Removal ends Authority and its source command, creates no damage, Cat group, A3/A1/A5 callback, ordinary Skill cast proc or energy payment, and preserves the original ordinary cooldown, A5 and independent Underboss generation/eligible pending groups. A repeated Skill after removal again uses ordinary cooldown; expiry, field exit and death remove replacement eligibility. Authored input and nominal packet template remain in the admission ledger with utility/Other provenance; no nominal packet is delivered. The source non-damaging route grounds selection; instantaneous zero-cost utility recovery, preservation and observer policy are revisable choices, not a measured or prorated finisher model.",
    },
    Hypothesis {
        id: "nanally.action_clock_v1",
        description: "Nanally uses finite rate-one source asset progress rounded upward to microseconds, not frame-verified playback. Native Normal stages retain1/1/3/7/2 packets and conservative recovery; a greater-than2s idle gap after the latest ordinary/heavy phase or preserving Dash resets it. Aim captures one target and no Cat group; melee/Support/Riposte/plunge/heavy player packets hit all living stationary dummies. One packet group grants resources/contact pause once after its first positive actual HP loss; zero loss adds neither. Shared exact hitlag integrates the strongest overlapping slowdown and projects pending packets, notifications and owner gates. Located contact pauses (seconds/rate) are Normal3 final.13/.01; Normal5 .06/.01 and.12/.01; Support .25/.2 delayed.06; Riposte .3/.01; landing .06/.01; Heavy1 .08/.01 then.17/.01; Heavy2 Intro1 each.1/.5 then Intro2 .1/.01; Heavy3 .2/.01 delayed.05. Missing pause durations create no invented interval. Jump, movement, switch and ability gates remain separate source coordinates. Support nominal ability/switch gates1.108275/1.083416 become1.308275/1.283416 after contact. Riposte nominal.841593 becomes1.138593, with Jump/Skill allowed at zero progress. Aim impact.016958/recovery.133047/movement0 allows pre-impact cancellation. Skill delivers five markers, starts real16s cooldown at425571us and applies Authority at429917us, preserving reached states on interruption. Burst pays100 energy and starts15s cooldown at commitment, delivers seven markers and applies its summon at3415870us. Burst ability/Jump3582140us, switch3583025us and movement3756871us retain separate gates. Interruption/new legal inputs cancel unissued owner packets, unborn notifications and unlaunched Cat groups, retaining settled damage/resources/cooldowns/states and independent launched children. Reached contact lag continues into later inputs; expired intervals are pruned. Actual non-damaging input command identity, spent dodge stamina and already granted one-use Riposte permission survive interruption; no Parry is inferred. Plunge consumes positive declared real flight before landing-relative50170us impact and255141/123558/749161us ability/Jump/movement gates. Its Normal-talent category-X coefficient scales by1+height bonus in0..1 without scaling charge. Live Underboss selects one Cat_AirAttack group at impact using captured Burst talent, no height scaling, selected uncapped recipients and one A1/A5 callback. Continuous hold retains the actual Normal/Riposte prefix and original real release. Source progress greater than.3/.35/.6/.3s on Normal1/2/3/5 or.9s on Riposte may enter Heavy1 while held; Normal4 instead enters Normal5 after.9s, cancelling its unissued second packet if it later branches. Heavy1/2 continue after progress greater than.6/.98s without re-adding hold time. Exclusive positions use the next microsecond; release at a boundary prevents continuation. Started heavy sections deliver2/3/1 packets unless cancelled. Cat heavy groups1/3/3 use captured Burst talent, uncapped recipients and one A1/A5 callback per group. Original input identity/role owns all subsections; native heavy packet categoryA is separate, with no fake commands or extra cast observers. Remaining hold after Heavy3 consumes real time without attacks and blocks a new Normal until release; legal cancel gates may release early. Recipient, release, contact, dispatch and subsection input-accounting policies are revisable models. A fresh admitted Dash preserves next Normal/heavy progress and rebases its2s idle window without refunding stamina or restoring unissued packets. A new hold of at least200ms resumes reached Heavy1/2 into Heavy2/3 after consuming200ms; a short hold/tap or stale permission takes Normal. Paid Critical Riposte has priority. Other abilities, Jump/Special, field exit, defeat or explicit interruption clear continuation; no-op same-actor swap preserves it. Retention/entry are revisable policies; the selected native full-kit baseline is accepted.",
    },
    Hypothesis {
        id: "linko.xiaozhen_pet_v1",
        description: "Linko's canonical Skill retains its four bombardment packets and deploys Xiaozhen once; companion deployment does not also instantiate the separate two-packet Throw route. An explicitly selected Throw primary keeps that two-packet route. The first autonomous pulse is eight World seconds after the admitted primary delivery, six with A1, and continuing pulses use the same interval without an extra energy pool. Pulses inherit the deploying Skill's effective native talent coordinate. Recasting replaces the generation without duplicating old pulses; field exit preserves the companion, while owner defeat clears permission and invalidates its generation. Source uses existing bounded caller action timing rather than measured native montage playback. This revisable selection separates ordinary and alternate Skill packets and retains the existing pulse resource policy.",
    },
    Hypothesis {
        id: "lacrimosa.sleep_in_v1",
        description: "Lacrimosa has one ordinary Skill cooldown, without stock charges or an independent12s recharge. Source selects16s at ordinary admission under its disclosed caller timing model; native notify/cancellation timing remains separate. During that live cooldown, five admitted ordinary Normal sections grant one Sleep In extra Skill. The grant waives remaining ordinary cooldown once, and its commit consumes the grant/counter while preserving the original ordinary deadline. Further sections cannot grant another extra in the same window. Ordinary expiry clears unused permission/counter; a new ordinary cast starts a fresh window. Counting is per admitted Normal section rather than per damage packet/recipient, independent of hit success and persistent through field exit. Existing selected melee-shatter cooldown reduction changes this one deadline rather than restoring invented stocks. Source cooldown selection, grant consumption and per-window bounds are explicit revisable model rules; native notify/interrupt behavior and stolen monster abilities are not established.",
    },
    Hypothesis {
        id: "mint.ordinary_clock_v1",
        description: "Mint canonical ordinary Normal, Skill and Burst use the selected rate-one source montage coordinates rounded once to nearest microseconds, not measured game playback. Explicit positive contact pauses integrate actor-local slowdown at rate.01, once per causal packet across stationary dummy recipients; missing durations and zero HP loss add none. All living eligible dummies receive each selected ordinary packet, without geometry. Native Normal stages retain2/1/2/1/2 packets, reset only on a greater-than2s input-start gap or a different ability/field exit, and keep the existing resource callbacks. Skill has two distinct contacts, starts its real6s cooldown at182791us progress and takes3s cancel cooldown only if interrupted before that notify; Burst keeps seven distinct contacts and its admitted real cooldown/energy cost. Pending montage deliveries and unborn notifications cancel on replacement, field exit or defeat, preserving settled damage, resources, passives and continuing cooldowns; live remaining contact pause carries into a replacement. Entry and explicitly selected approach replace Normal1 and advance to2. A new admitted Jump opens one declared plunge flight; landing has no implicit geometry. Paid Front/Back Dodge uses owner-specific Jump, Skill and movement gates; explicit successful parry requests Back after20000us without another payment or a damage packet. Critical Dodge grants one replacement Riposte opportunity. Engine-qualified switching alone invokes the739424us Support contact, with a delayed400000us actor pause at rate.2 and separate1333238us switch/1366703us recovery gates. Unqualified entry arms only the next Basic, and no-op self switches do not reset it. Canonical held Normal additionally uses mint.hold_attack_v1. Native source initializes the owner clock before entry and submits approach, Jump/declared landing, paid controls and family windows to these same native gates; its Burst cooldown is15 World seconds. The frozen scheduled Studio adapter retains its separate caller-timing scope until Phase4. The selected source timing and cancellation policies remain revisable hypotheses.",
    },
    Hypothesis {
        id: "iroi.action_clock_v1",
        description: "Iroi ordinary Normal uses the selected rate-one montage coordinates rounded once to microseconds, with exact next-Basic boundaries and distinct Skill/movement/other gates. Positive normal contacts add one33000us actor pause at rate.1 per causal projectile, delayed33000us for stages4/5; a zero-duration stage3 rate supplies no pause. Normal1/2 select one living locked recipient per projectile with deterministic fallback; stages3/4/5 cover all living eligible stationary dummies. Stage3 spawns one finite world actor at268779us progress, first contact200000us later then100000us spacing for eight contacts; launched actors survive caster progression/field exit, while unborn montage notifies cancel. Owner death prevents future actor ticks. Resource channels remain per causal packet, and live stats are sampled at impact. Continuing stages require the exact native queued Basic boundary; an idle input restarts stage1. This is a selected source-coordinate/contact inference, not measured playback. Canonical tap Skill uses montage cooldown249777us, heal/carrier498799us, Gun birth498827us with1200000us travel, Syringe birth551115us with seven100000us-spaced world contacts, and Trumpet birth601775us. Connected Skill actors add100000us player pauses at rate.1 once per causal packet, including on a replacement player montage; overlapping windows use minimum rate and never delay world ticks. Skill cooldown is12 World seconds from its notify, or3 seconds from interruption before that notify. Actual healing grants16 Moon once outside Dream and the self-excluded capped flat-ATK buff; normal healing callbacks retain live stats and passives. The tiny Skill carrier grants its sourced10 Charge once per effect application to an eligible living target, even when fractional damage rounds to zero integer HP loss; this disclosed rounding interpretation uses the ordinary exact carry/team ledger without fake HP or double grants. Other packets require positive HP loss. Explicit interruption removes unborn notifications and preserves launched actors and committed cooldowns. Selected A2 Skill diagnoses until its native assist exists. Native Burst reserves Moon at input, heals at1925956us, emits its Ivory-only direct packet at3385389us, installs pulses at3882835us and releases Dream drain/periods at4004747us montage progress; switch gate3764482us differs from4015812us ordinary recovery. Its real16s cooldown never follows montage pause. Setup projections shift only this owner reservation and pending Dream deadlines. Interrupted setup before any carrier retains unspent Moon without a Dream; after installation, actual cancellation releases only installed carriers with their full remaining Moon lifetime. These are explicit interruption/lifecycle inferences. Native pulses retain the actual Burst command/talent and reach all eligible living stationary dummies, with no inferred recipient cap. Existing Dream threshold, strict-before-end periods and death/generation invalidation remain authoritative. Held Branch/Transform, Lucid Dream, A2 assists, secondary and Regression interactions remain concrete dependencies before source selection.",
    },
    Hypothesis {
        id: "lacrimosa.broken_shatter_v1",
        description: "Preserve the selected Lacrimosa Broken-target callback: each delivered ranged Melee4/Melee6 or ordinary Skill1 packet against an actively Broken dummy queues one B_Melee3_Explode child at the same real timestamp with coefficient0.686 and the content-owned shatter tag. The child retains the initiating action and target; its tag and owner-child binding share this hypothesis. This is the existing bounded trigger/timing/coefficient model, separate from the melee-chain projectile shatter; it does not establish native projectile dispatch or geometry.",
    },
    Hypothesis {
        id: "zankou.forms_v1",
        description: "Current rolling Zankou source selects Reality entry or Illusion return from live owner state. Skill has a real five-second admission cooldown; selected rate-one montage transitions commit at496507us entry and2039354us return, independently of HP loss. Illusion lasts eight World seconds, half open; enhanced Soulcross is earned during its final3500000us. Enhanced entry requires a current eligible Reality action section. Field exit discards form and permissions without restoring them on return. Pending montage contacts and transition/grant notifications cancel on replacement or interruption while committed costs, cooldowns and launched DoT trains continue. Natural expiry uses a generation guard and does not cancel an already admitted Soulcross. Source coordinates and finite transition interpretation are bounded models, not certified native playback.",
    },
    Hypothesis {
        id: "zankou.form_modifiers_v1",
        description: "Zankou Reality/Illusion Normal and held actions grant self Hunt25percent damage or team Delusion25percent DoT critical damage during the admitted owner action. Positive contact projection extends only that action component. A legally adjacent entry/return Skill carries its existing component through setup; successful entry retains Hunt for eight World seconds and successful Soulcross retains Delusion for eight World seconds. World retention never follows montage pauses. Rejected input grants nothing; cancellation discards its action component and A0 field exit discards both retained components. Selected A1 instead keeps living-source self Hunt40percent and team Delusion50percent permanent on and off field, without double counting or extending the source after defeat. This explicit finite action/retention lifecycle interprets the current class and published duration contracts; it is not a certified native ability-task executor.",
    },
    Hypothesis {
        id: "zankou.combat_passives_v1",
        description: "Current source A3 self-heal uses live MaxHP*.04 once per World second while Delusion is active. Refresh preserves an armed tick phase; source defeat invalidates it. Explicit native armor_damage fixtures use one normalized armor unit per admitted action and multiply incoming pressure by .5 while A3 Hunt is active, otherwise1. Threshold1 cancels the owner montage. This is a bounded pressure/threshold model, not recovered native armor capacity or regeneration; omitted armor_damage remains forced cancellation. A5 enhanced entry applies the source999999 impulse once per living nonsummoned dummy per combat, excluding bosses only with zankou.story_mode_v1. Anonymous authored targets are nonsummoned. Existing Break profiles/efficiency and damage settlement remain authoritative. Living A5 supplies team Break damage+300%.",
    },
    Hypothesis {
        id: "zankou.story_mode_v1",
        description: "Explicit native scenario assumption: the combat is story mode for Zankou A5 boss exclusion. Omission models nonspecific non-story stationary dummies; this is not an enemy catalog or a native inferred mode.",
    },
    Hypothesis {
        id: "sakiri.burst_atk_v1",
        description: "Current GA_Sagiri_UltraSkill activation independently requests ordinary Buff_Sagiri003_QDamUp and, only with node4, Buff_Sagiri003_Level4_1. Each adds30percent of the source GetAtkBase to other living teammates for20 World seconds, with no self grant. The separate cap1 carriers refresh rather than stack. Base includes compiled character and equipped weapon base ATK, excluding console flat/percent and recipient ATK. The selected managed snapshot lifetime survives source defeat; dead recipients receive no new grant and existing dead carriers contribute nothing. Actual admitted activation produces the grants, without hit or damage requirements. This explicit lifetime/recipient model does not simulate multiplayer global-component routing.",
    },
    Hypothesis {
        id: "combat.relative_clock_policy_v1",
        description: "Relative combat time is absolute elapsed microseconds minus the union of executed Ultimate cinematic and Hotori active Time Stop pauses. Retrieved pause components and owner montage end notifies inform bindings; an unavailable notify, including Zero, uses the consumed owner action-completion fallback. This is a selected bounded timer model, not recovered native countdown execution. Ordinary cooldowns, independent buffs/statuses, periodic phases, passive recovery and refund/recording windows use paused world time. Finite current-cast and permitted Hotori direct replay/Katana contexts progress; another non-exempt cinematic dominates. Object lifetime pauses in cinematics and advances in active Time Stop while ordinary autonomous production pauses; living Hotori B2 exempts Vita Bud attacks only from active Time Stop. Frozen Scorch phase and progressing deployed-object lifetime are disclosed conservative and lower-confidence choices. Charge grants no energy or deferred grants in active Time Stop. Local hitlag remains separate; waits, receipts, support rates and field-time use absolute time. Relative DPS is unavailable at zero relative duration, independently of combat completion.",
    },
    Hypothesis {
        id: "adler.temperance_v1",
        description: "Living Breakthrough 2 Adler selects one of the three source-ordered Temperance target effects on each successful Scorch application or refresh, including off field. Selection uses a separate seeded unbiased uniform stream, an inference from the unweighted source candidate array. Each result has its own 15 World second cap-one timer: subtract 20% of original target ATK, reduce Physical, all six Esper channels and Mental resistance by 10 percentage points, or add 10 percentage points to incoming Break efficiency. Target defeat clears the statuses. Existing selections survive source defeat until expiry. Authored incoming damage is independent of dummy ATK.",
    },
    Hypothesis {
        id: "zero.cosmos_cycle_atk_v1",
        description: "Resonance 6 Zero grants each living party member, including Zero, 12% base-ATK-scaled ATK for 20 World seconds on a qualified Cosmos Cycle for which Zero pays the outgoing Cycle cost. The outgoing payer identifies the trigger independently of the higher-CI reaction damage owner. The same carrier refreshes without stacking and persists through field exit. The published trigger wording and frozen amount/duration inform this selected ownership and lifetime model.",
    },
    Hypothesis {
        id: "combat.single_packet_support_v1",
        description: "Single-packet qualified Support actions use their owner packet, purchased QTE coordinate, recovered or documented bounded timing, and owner-local recovery gates. Living in-range dummy recipients share one packet's ordinary Charge and connected owner effects.",
    },
    Hypothesis {
        id: "lacrimosa.a6_free_reaction_entry_v1",
        description: "Lacrimosa Effect6 arms a five-World-second entry window on committed Devilish Gift when its internal cooldown is ready. A supported Nova/Scorch switch to her, including Discord from that application, consumes the window, admits QTE and starts a five-World-second Effect6 cooldown with zero outgoing Cycle spent. Standard paid reactions remain available.",
    },
    Hypothesis {
        id: "weapons.echoes_on_hold_v1",
        description: "Echoes on Hold grants team DEF Ignore and Cosmos penetration at combat start, refreshes Cosmos penetration on wearer Redirect Skill, and refreshes DEF Ignore after 3 wearer Follow-up Attack packets following an Ultimate.",
    },
    Hypothesis {
        id: "akane.ordinary_clock_v1",
        description: "Akane Rin Stringblade, Rhythm Wave, Synced Resonance, Sonic Boom, and secondary actions execute on rate-one montage clocks rounded to microseconds with explicit N2 (19.8ms x2) and N3 (19.8ms x3 + 18.0ms x4) HitPause slowdowns, +30ms DamageActorAkane_Melee4 DelayTriggerTime, zero-flight SoundMark and Melee1 projectile delivery, 4012833us CINEMATIC_RELEASE world pause subsuming UltraSkill1 hitpause, and explicit cancel, jump, and switch-lock windows.",
    },
    Hypothesis {
        id: "akane.normal_chain_v1",
        description: "Stringblade executes four ground stages (N1 x2, N2 x2, N3 x3 Melee3 + x4 Melee3_1, N4 x1) with AppearMelee and MeleePre variants, conditional Melee3_1 continuation, and ComboSectionMap preservation across Evade and Critical Riposte; field exit resets active stage and action state.",
    },
    Hypothesis {
        id: "akane.ordinary_hit_energy_v1",
        description: "For Akane Rin character:1057 only, modeled Normal, Skill, Hold, Plunge, Riposte, and QTE native packets grant their nonnegative DT_SkillDamageData.ChargeAdd once per positive actual HP loss against target zero and distribute 0.6x team energy via DT_SkillTeamChargeRelationData Lakshana rows, plus HeterochromeAdd Cycle accrual; non-combat Buff_Akane057_BattleState +3/s Ultimate Energy regeneration is omitted.",
    },
    Hypothesis {
        id: "akane.string_energy_v1",
        description: "Akane Rin maintains bounded integer String Energy in 0..=30 (default 0, actor option start_string_energy, or 30 on battle entry at R3), gaining +1 on N1/N2/Airattack, +2 on N3/PerfectEvadeAttack, +4 on QTE, +6 on N4/Skill/Skill2, and 0 on UltraSkill/UpgradeSkill deduplicated once per cast stage, spending 30 -> 0 strictly on Synced Resonance hit prior to R6 +10 refund.",
    },
    Hypothesis {
        id: "akane.rhythm_wave_v1",
        description: "Rhythm Wave shares a 16.0s cooldown between Press (Overtone 25.0s single-target mark, +40% or A5 +64% CoefAdd on N1-N4 and Critical Riposte against the marked dummy while N2-N4 AoE hitboxes strike all living dummies) and Hold (hold >= 0.2s, Reverb Field 25.0s AoE aura, +60% or A5 +90% Skill/Hold DMG bonus) with mutual removal; at String Energy == 30 with an active mode, Press redirects to Press-only Synced Resonance (0.0s independent cooldown preserving Rhythm Wave cooldown and mode timer, emitting UpgradeSkill_Damage without UI-only UpgradeSkill_DisplayGE).",
    },
    Hypothesis {
        id: "akane.sonic_boom_v1",
        description: "Sonic Boom Stage 1 costs 100 Ultimate Energy on a 20.0s cooldown, delivers 10x UltraSkill1_Damage, grants 20.0s Solo Cadenza (allowing continued Ultimate Energy gain) and 20.0s Starlit Anthem (+20% ATK), and unlocks manual Stage 2 (0 cost, 0.1s cooldown, UltraSkill2_Damage, refreshing Starlit Anthem and ending Solo Cadenza without altering Stage 1 cooldown) between 4.0s and 20.0s elapsed without auto-casting on expiry.",
    },
    Hypothesis {
        id: "akane.perfect_pitch_v1",
        description: "Perfect Pitch unlocks permanently for the combat on the first team Remora reaction; while Akane Rin is alive (on or off field) with Overtone or Reverb Field active, the active mode applies Remora every 3.0s to its marked or AoE targets honoring team enhancements without incrementing Esper Cycle reaction counters. Any team or periodic Remora reapplication onto an active-Remora target outside a 1.0s per-target ICD emits Category E GE_Reaction_3_new_1057_Damage (150% ATK, bound under GA_Akane_Skill) with discrete floor(existing_remaining_seconds) * 0.20 CoefAdd capped at +160% sampled before timer refresh.",
    },
    Hypothesis {
        id: "akane.awakenings_v1",
        description: "Akane Rin A1-A6 identity-gated awakenings and R3/R6 count-gated resonance tiers apply verified modifiers: A1 10.0*BaseATK team heal plus +50% DEF and 60% Poise for 15.0s, A2 -12% Lakshana RES on marked/field targets, A3 +10.0 Skill_Damage ratio and 8.0s pull field, A4 +30% CritDMG for 15.0s, A5 +64%/+90% mode upgrades, A6 16% DefIgnore during Starlit Anthem, R3 battle-entry 30 String Energy plus +8.0 UpgradeSkill/+10.0 UltraSkill2 and skill level boosts, and R6 post-spend +10 refund plus +64% CritDMG for 25.0s stacking independently with A4.",
    },
    Hypothesis {
        id: "akane.aoe_all_in_range_dummy_v1",
        description: "Akane Rin AoE abilities (N2-N4, Rhythm Wave Hold, Synced Resonance under Reverb Field, Sonic Boom Stage 1/2, Stage Dive, Altered Chord, and Critical Riposte) strike all living scenario-declared dummies in ascending target_index under the all-in-range contract, while Overtone CoefAdd applies exclusively to the marked dummy.",
    },
    Hypothesis {
        id: "akane.secondary_actions_v1",
        description: "Akane Rin secondary actions execute Stage Dive plunge (Airattack_Damage with 1 + height_bonus in 0..=1), Evade (15.0 stamina), Critical Riposte (PerfectEvadeAttack_Damage), and Altered Chord QTE (QTE_Damage).",
    },
];

static PARSE_TABLE: [(&str, HypothesisId); 456] = [
    (
        "ability_index.primary_action_role_v1",
        HypothesisId::AbilityIndexPrimaryActionRoleV1,
    ),
    (
        "actions.caller_timing_v1",
        HypothesisId::ActionsCallerTimingV1,
    ),
    (
        "actions.primary_packet_v1",
        HypothesisId::ActionsPrimaryPacketV1,
    ),
    (
        "adler.a5_burst_energy_refund_v1",
        HypothesisId::AdlerA5BurstEnergyRefundV1,
    ),
    (
        "adler.blessing_a4_overflow_v1",
        HypothesisId::AdlerBlessingA4OverflowV1,
    ),
    (
        "adler.blessing_party_interception_v1",
        HypothesisId::AdlerBlessingPartyInterceptionV1,
    ),
    ("adler.blessing_r6_v1", HypothesisId::AdlerBlessingR6V1),
    (
        "adler.blessing_shield_v1",
        HypothesisId::AdlerBlessingShieldV1,
    ),
    (
        "adler.burst_intervals_v1",
        HypothesisId::AdlerBurstIntervalsV1,
    ),
    ("adler.burst_sunyas_v1", HypothesisId::AdlerBurstSunyasV1),
    ("adler.critical_riposte_v1", HypothesisId::AdlerRiposteV1),
    ("adler.karma_v1", HypothesisId::AdlerKarmaV1),
    (
        "adler.normal_chain_13_v1",
        HypothesisId::AdlerNormalChain13V1,
    ),
    ("adler.normal_chain_v1", HypothesisId::AdlerNormalChainV1),
    (
        "adler.ordinary_hit_energy_v1",
        HypothesisId::AdlerOrdinaryHitEnergyV1,
    ),
    ("adler.plunge_v1", HypothesisId::AdlerPlungeV1),
    (
        "adler.righteous_heart_v1",
        HypothesisId::AdlerRighteousHeartV1,
    ),
    (
        "adler.skill_all_in_range_dummy_v1",
        HypothesisId::AdlerSkillAllInRangeDummyV1,
    ),
    ("adler.skill_train_v1", HypothesisId::AdlerSkillTrainV1),
    ("adler.temperance_v1", HypothesisId::AdlerTemperanceV1),
    (
        "aim.mode_transitions_v1",
        HypothesisId::AimModeTransitionsV1,
    ),
    (
        "akane.aoe_all_in_range_dummy_v1",
        HypothesisId::AkaneAoeAllInRangeDummyV1,
    ),
    ("akane.awakenings_v1", HypothesisId::AkaneAwakeningsV1),
    ("akane.normal_chain_v1", HypothesisId::AkaneNormalChainV1),
    (
        "akane.ordinary_clock_v1",
        HypothesisId::AkaneOrdinaryClockV1,
    ),
    (
        "akane.ordinary_hit_energy_v1",
        HypothesisId::AkaneOrdinaryHitEnergyV1,
    ),
    ("akane.perfect_pitch_v1", HypothesisId::AkanePerfectPitchV1),
    ("akane.rhythm_wave_v1", HypothesisId::AkaneRhythmWaveV1),
    (
        "akane.secondary_actions_v1",
        HypothesisId::AkaneSecondaryActionsV1,
    ),
    ("akane.sonic_boom_v1", HypothesisId::AkaneSonicBoomV1),
    ("akane.string_energy_v1", HypothesisId::AkaneStringEnergyV1),
    ("aurelia.a2_cadenza_v1", HypothesisId::AureliaA2CadenzaV1),
    (
        "aurelia.airattack_evadebranch_direct_packet_energy_v1",
        HypothesisId::AureliaAirattackEvadebranchDirectPacketEnergyV1,
    ),
    (
        "aurelia.aoe_all_in_range_dummy_v1",
        HypothesisId::AureliaAoeAllInRangeDummyV1,
    ),
    (
        "aurelia.cadenza_power_select_v1",
        HypothesisId::AureliaCadenzaPowerSelectV1,
    ),
    (
        "aurelia.crescendo_b4_v1",
        HypothesisId::AureliaCrescendoB4V1,
    ),
    (
        "aurelia.critical_riposte_v1",
        HypothesisId::AureliaRiposteV1,
    ),
    (
        "aurelia.direct_packet_inputs_v1",
        HypothesisId::AureliaDirectPacketInputsV1,
    ),
    (
        "aurelia.harmonics_b2_v1",
        HypothesisId::AureliaHarmonicsB2V1,
    ),
    (
        "aurelia.normal_chain_v1",
        HypothesisId::AureliaNormalChainV1,
    ),
    (
        "aurelia.ordinary_hit_energy_v1",
        HypothesisId::AureliaOrdinaryHitEnergyV1,
    ),
    ("aurelia.plunge_v1", HypothesisId::AureliaPlungeV1),
    ("aurelia.r6_psyche_v1", HypothesisId::AureliaR6PsycheV1),
    (
        "aurelia.shadow_creed_skill_finish_v1",
        HypothesisId::AureliaShadowCreedSkillFinishV1,
    ),
    (
        "aurelia.skill_burst_packets_v1",
        HypothesisId::AureliaSkillBurstPacketsV1,
    ),
    (
        "aurelia.staccato_perfect_dodge_v1",
        HypothesisId::AureliaStaccatoPerfectDodgeV1,
    ),
    ("aurelia.staccato_v1", HypothesisId::AureliaStaccatoV1),
    (
        "baicang.actor_hitstop_v1",
        HypothesisId::BaicangActorHitstopV1,
    ),
    (
        "baicang.aoe_all_in_range_dummy_v1",
        HypothesisId::BaicangAoeAllInRangeDummyV1,
    ),
    (
        "baicang.cancel_windows_v1",
        HypothesisId::BaicangCancelWindowsV1,
    ),
    (
        "baicang.execute_and_fatal_hit_debt_v1",
        HypothesisId::BaicangExecuteAndFatalHitDebtV1,
    ),
    (
        "baicang.fadia_assist_v1",
        HypothesisId::BaicangFadiaAssistV1,
    ),
    (
        "baicang.forced_interrupt_v1",
        HypothesisId::BaicangForcedInterruptV1,
    ),
    (
        "baicang.height_aware_plunge_v1",
        HypothesisId::BaicangHeightAwarePlungeV1,
    ),
    (
        "baicang.hold_branches_v1",
        HypothesisId::BaicangHoldBranchesV1,
    ),
    (
        "baicang.judgment_of_autumn_domain_v1",
        HypothesisId::BaicangJudgmentOfAutumnDomainV1,
    ),
    (
        "baicang.lacrimosa_entry_jump_v1",
        HypothesisId::BaicangLacrimosaEntryJumpV1,
    ),
    (
        "baicang.moderate_mischief_v1",
        HypothesisId::BaicangModerateMischiefV1,
    ),
    (
        "baicang.normal_chain_v1",
        HypothesisId::BaicangNormalChainV1,
    ),
    (
        "baicang.ordinary_hit_energy_v1",
        HypothesisId::BaicangOrdinaryHitEnergyV1,
    ),
    ("baicang.plunge_v1", HypothesisId::BaicangPlungeV1),
    ("baicang.power_words_v1", HypothesisId::BaicangPowerWordsV1),
    (
        "baicang.qualified_support_v1",
        HypothesisId::BaicangQualifiedSupportV1,
    ),
    ("baicang.riposte_v1", HypothesisId::BaicangRiposteV1),
    ("baicang.skia_entry_v1", HypothesisId::BaicangSkiaEntryV1),
    (
        "blackbird.aoe_all_in_range_dummy_v1",
        HypothesisId::BlackbirdAoeAllInRangeDummyV1,
    ),
    ("blackbird.b2_nova_v1", HypothesisId::BlackbirdB2NovaV1),
    ("blackbird.b2_stain_v1", HypothesisId::BlackbirdB2StainV1),
    ("blackbird.branches_v1", HypothesisId::BlackbirdBranchesV1),
    (
        "blackbird.cancel_gates_v1",
        HypothesisId::BlackbirdCancelGatesV1,
    ),
    (
        "blackbird.combat_transitions_v1",
        HypothesisId::BlackbirdCombatTransitionsV1,
    ),
    (
        "blackbird.coordinated_bird_v1",
        HypothesisId::BlackbirdCoordinatedBirdV1,
    ),
    (
        "blackbird.evolve_actions_v1",
        HypothesisId::BlackbirdEvolveActionsV1,
    ),
    (
        "blackbird.evolved_secondary_v1",
        HypothesisId::BlackbirdEvolvedSecondaryV1,
    ),
    (
        "blackbird.evolved_support_v1",
        HypothesisId::BlackbirdEvolvedSupportV1,
    ),
    (
        "blackbird.explicit_parry_v1",
        HypothesisId::BlackbirdExplicitParryV1,
    ),
    (
        "blackbird.normal_chain_v1",
        HypothesisId::BlackbirdNormalChainV1,
    ),
    (
        "blackbird.ordinary_clock_v1",
        HypothesisId::BlackbirdOrdinaryClockV1,
    ),
    (
        "blackbird.ordinary_hit_energy_v1",
        HypothesisId::BlackbirdOrdinaryHitEnergyV1,
    ),
    (
        "blackbird.ordinary_malice_v1",
        HypothesisId::BlackbirdOrdinaryMaliceV1,
    ),
    (
        "blackbird.ordinary_secondary_v1",
        HypothesisId::BlackbirdOrdinarySecondaryV1,
    ),
    (
        "blackbird.primary_form_v1",
        HypothesisId::BlackbirdPrimaryFormV1,
    ),
    (
        "blackbird.primary_packet_groups_v1",
        HypothesisId::BlackbirdPrimaryPacketGroupsV1,
    ),
    (
        "blackbird.qualified_support_v1",
        HypothesisId::BlackbirdQualifiedSupportV1,
    ),
    (
        "blackbird.skill_burst_packets_v1",
        HypothesisId::BlackbirdSkillBurstPacketsV1,
    ),
    (
        "blackbird.skill_cycle_fill_v1",
        HypothesisId::BlackbirdSkillCycleFillV1,
    ),
    (
        "build.action_projection_v1",
        HypothesisId::BuildActionProjectionV1,
    ),
    (
        "build.base_overrides_v1",
        HypothesisId::BuildBaseOverridesV1,
    ),
    (
        "build.gear_projection_v1",
        HypothesisId::BuildGearProjectionV1,
    ),
    (
        "build.stat_quantization_v1",
        HypothesisId::BuildStatQuantizationV1,
    ),
    (
        "build.static_additive_v1",
        HypothesisId::BuildStaticAdditiveV1,
    ),
    (
        "build.unmodeled_native_attributes_v1",
        HypothesisId::BuildUnmodeledNativeAttributesV1,
    ),
    (
        "chaos.aoe_all_in_range_dummy_v1",
        HypothesisId::ChaosAoeAllInRangeDummyV1,
    ),
    ("chaos.b2_remora_end_v1", HypothesisId::ChaosB2RemoraEndV1),
    (
        "chaos.forced_interrupt_v1",
        HypothesisId::ChaosForcedInterruptV1,
    ),
    ("chaos.jump_cancel_v1", HypothesisId::ChaosJumpCancelV1),
    (
        "chaos.neutral_verdict_v1",
        HypothesisId::ChaosNeutralVerdictV1,
    ),
    ("chaos.normal_chain_v1", HypothesisId::ChaosNormalChainV1),
    (
        "chaos.normal_combo_verdict_v1",
        HypothesisId::ChaosNormalComboVerdictV1,
    ),
    (
        "chaos.ordinary_clock_v1",
        HypothesisId::ChaosOrdinaryClockV1,
    ),
    (
        "chaos.ordinary_crime_v1",
        HypothesisId::ChaosOrdinaryCrimeV1,
    ),
    (
        "chaos.ordinary_hit_energy_v1",
        HypothesisId::ChaosOrdinaryHitEnergyV1,
    ),
    ("chaos.primary_form_v1", HypothesisId::ChaosPrimaryFormV1),
    (
        "chaos.qte_direct_packet_energy_v1",
        HypothesisId::ChaosQteDirectPacketEnergyV1,
    ),
    (
        "chaos.qualified_support_v1",
        HypothesisId::ChaosQualifiedSupportV1,
    ),
    (
        "chaos.secondary_actions_v1",
        HypothesisId::ChaosSecondaryActionsV1,
    ),
    ("chaos.skill_bites_v1", HypothesisId::ChaosSkillBitesV1),
    (
        "chaos.suppression_packets_v1",
        HypothesisId::ChaosSuppressionPacketsV1,
    ),
    (
        "chaos.verdict_continuation_v1",
        HypothesisId::ChaosVerdictContinuationV1,
    ),
    (
        "chaos.warrant_transfer_v1",
        HypothesisId::ChaosWarrantTransferV1,
    ),
    (
        "character.adler.skill_dot_packet_semantics_v1",
        HypothesisId::CharacterAdlerSkillDotPacketSemanticsV1,
    ),
    (
        "character.baicang.dot_packet_semantics_v1",
        HypothesisId::CharacterBaicangDotPacketSemanticsV1,
    ),
    (
        "character.zankou.crimson_reverie_v1",
        HypothesisId::CharacterZankouCrimsonReverieV1,
    ),
    (
        "character.zankou.dot_packet_semantics_v1",
        HypothesisId::CharacterZankouDotPacketSemanticsV1,
    ),
    (
        "character.zankou.heartwrench_skill_v1",
        HypothesisId::CharacterZankouHeartwrenchSkillV1,
    ),
    (
        "character.zankou.silent_sunset_v1",
        HypothesisId::CharacterZankouSilentSunsetV1,
    ),
    (
        "character.zankou.vile_ash_dot_scheduling_v1",
        HypothesisId::CharacterZankouVileAshDotSchedulingV1,
    ),
    (
        "character.zankou.vile_ash_dot_scheduling_v2",
        HypothesisId::CharacterZankouVileAshDotSchedulingV2,
    ),
    ("chiz.action_cancels_v1", HypothesisId::ChizActionCancelsV1),
    (
        "chiz.aoe_all_in_range_dummy_v1",
        HypothesisId::ChizAoeAllInRangeDummyV1,
    ),
    ("chiz.barren_v1", HypothesisId::ChizBarrenV1),
    ("chiz.battle_entry_v1", HypothesisId::ChizBattleEntryV1),
    (
        "chiz.charge_passives_v1",
        HypothesisId::ChizChargePassivesV1,
    ),
    ("chiz.exact_finance_v1", HypothesisId::ChizExactFinanceV1),
    ("chiz.exact_grain_v1", HypothesisId::ChizExactGrainV1),
    ("chiz.held_full_v1", HypothesisId::ChizHeldFullV1),
    ("chiz.held_skill_v1", HypothesisId::ChizHeldSkillV1),
    ("chiz.market_v1", HypothesisId::ChizMarketV1),
    ("chiz.normal_chain_v1", HypothesisId::ChizNormalChainV1),
    ("chiz.normal_grain_v1", HypothesisId::ChizNormalGrainV1),
    ("chiz.ordinary_clock_v1", HypothesisId::ChizOrdinaryClockV1),
    (
        "chiz.ordinary_hit_energy_v1",
        HypothesisId::ChizOrdinaryHitEnergyV1,
    ),
    ("chiz.pressed_skill_v1", HypothesisId::ChizPressedSkillV1),
    (
        "chiz.recorded_market_trace_v1",
        HypothesisId::ChizRecordedMarketTraceV1,
    ),
    (
        "chiz.secondary_actions_v1",
        HypothesisId::ChizSecondaryActionsV1,
    ),
    (
        "combat.control_effects_v1",
        HypothesisId::CombatControlEffectsV1,
    ),
    (
        "combat.discord_break_gauge_v1",
        HypothesisId::CombatDiscordBreakGaugeV1,
    ),
    (
        "combat.esper_cycle_reactions_v1",
        HypothesisId::CombatEsperCycleReactionsV1,
    ),
    (
        "combat.hp_cost_fixture_v1",
        HypothesisId::CombatHpCostFixtureV1,
    ),
    (
        "combat.incoming_damage_fixture_v1",
        HypothesisId::CombatIncomingDamageFixtureV1,
    ),
    (
        "combat.incoming_defense_v1",
        HypothesisId::CombatIncomingDefenseV1,
    ),
    (
        "combat.live_break_damage_v1",
        HypothesisId::CombatLiveBreakDamageV1,
    ),
    (
        "combat.live_healing_modifiers_v1",
        HypothesisId::CombatLiveHealingModifiersV1,
    ),
    (
        "combat.packet_cycle_accrual_v1",
        HypothesisId::CombatPacketCycleAccrualV1,
    ),
    (
        "combat.relative_clock_policy_v1",
        HypothesisId::CombatRelativeClockPolicyV1,
    ),
    (
        "combat.remora_charge_v1",
        HypothesisId::CombatRemoraChargeV1,
    ),
    (
        "combat.single_packet_support_v1",
        HypothesisId::CombatSinglePacketSupportV1,
    ),
    ("combat.stain_switch_v1", HypothesisId::CombatStainSwitchV1),
    ("combat.stamina_v1", HypothesisId::CombatStaminaV1),
    (
        "combat.support_dispatch_v1",
        HypothesisId::CombatSupportDispatchV1,
    ),
    (
        "combat.support_dispatch_v2",
        HypothesisId::CombatSupportDispatchV2,
    ),
    (
        "combat.support_dispatch_v3",
        HypothesisId::CombatSupportDispatchV3,
    ),
    (
        "combat.support_dispatch_v4",
        HypothesisId::CombatSupportDispatchV4,
    ),
    (
        "combat.unbal_value_as_target_gauge_v1",
        HypothesisId::CombatUnbalValueAsTargetGaugeV1,
    ),
    (
        "crit.fixed_noncrit_preview_v1",
        HypothesisId::CritFixedNoncritPreviewV1,
    ),
    (
        "daffodil.action_cancels_v1",
        HypothesisId::DaffodilActionCancelsV1,
    ),
    (
        "daffodil.aoe_all_in_range_dummy_v1",
        HypothesisId::DaffodilAoeAllInRangeDummyV1,
    ),
    (
        "daffodil.branch_entry_direct_packet_energy_v1",
        HypothesisId::DaffodilBranchEntryDirectPacketEnergyV1,
    ),
    (
        "daffodil.branch_entry_direct_packet_inputs_v1",
        HypothesisId::DaffodilBranchEntryDirectPacketInputsV1,
    ),
    (
        "daffodil.broken_chaos_resistance_v1",
        HypothesisId::DaffodilBrokenChaosResistanceV1,
    ),
    (
        "daffodil.discord_cap_reduction_v1",
        HypothesisId::DaffodilDiscordCapReductionV1,
    ),
    (
        "daffodil.held_normal_v1",
        HypothesisId::DaffodilHeldNormalV1,
    ),
    (
        "daffodil.insight_break_capture_v1",
        HypothesisId::DaffodilInsightBreakCaptureV1,
    ),
    (
        "daffodil.normal_chain_v1",
        HypothesisId::DaffodilNormalChainV1,
    ),
    (
        "daffodil.ordinary_clock_v1",
        HypothesisId::DaffodilOrdinaryClockV1,
    ),
    (
        "daffodil.ordinary_hit_energy_v1",
        HypothesisId::DaffodilOrdinaryHitEnergyV1,
    ),
    (
        "daffodil.phantom_step_v1",
        HypothesisId::DaffodilPhantomStepV1,
    ),
    (
        "daffodil.resonance_skill_capture_v1",
        HypothesisId::DaffodilResonanceSkillCaptureV1,
    ),
    (
        "daffodil.secondary_actions_v1",
        HypothesisId::DaffodilSecondaryActionsV1,
    ),
    (
        "daffodil.skill_burst_packets_v1",
        HypothesisId::DaffodilSkillBurstPacketsV1,
    ),
    (
        "damage.fixed_crit_rate_as_probability_v1",
        HypothesisId::DamageFixedCritRateAsProbabilityV1,
    ),
    (
        "damage_type.normal_as_physical_v1",
        HypothesisId::DamageTypeNormalAsPhysicalV1,
    ),
    ("edgar.a5_max_hp_v1", HypothesisId::EdgarA5MaxHpV1),
    (
        "edgar.a6_wake_team_def_v1",
        HypothesisId::EdgarA6WakeTeamDefV1,
    ),
    (
        "edgar.aoe_all_in_range_dummy_v1",
        HypothesisId::EdgarAoeAllInRangeDummyV1,
    ),
    ("edgar.full_kit_v1", HypothesisId::EdgarFullKitV1),
    ("edgar.key_of_truth_v1", HypothesisId::EdgarKeyOfTruthV1),
    ("edgar.normal_chain_v1", HypothesisId::EdgarNormalChainV1),
    (
        "edgar.ordinary_hit_energy_v1",
        HypothesisId::EdgarOrdinaryHitEnergyV1,
    ),
    (
        "edgar.skill_burst_packets_v1",
        HypothesisId::EdgarSkillBurstPacketsV1,
    ),
    (
        "edgar.ultimate_periodic_healing_v1",
        HypothesisId::EdgarUltimatePeriodicHealingV1,
    ),
    (
        "energy.efficiency_carry_v1",
        HypothesisId::EnergyEfficiencyCarryV1,
    ),
    (
        "fadia.aoe_all_in_range_dummy_v1",
        HypothesisId::FadiaAoeAllInRangeDummyV1,
    ),
    (
        "fadia.full_hp_initial_burst_v1",
        HypothesisId::FadiaFullHpInitialBurstV1,
    ),
    ("fadia.full_kit_v1", HypothesisId::FadiaFullKitV1),
    (
        "fadia.incoming_share_v1",
        HypothesisId::FadiaIncomingShareV1,
    ),
    ("fadia.normal_chain_v1", HypothesisId::FadiaNormalChainV1),
    (
        "fadia.ordinary_clock_v1",
        HypothesisId::FadiaOrdinaryClockV1,
    ),
    (
        "fadia.ordinary_hit_energy_v1",
        HypothesisId::FadiaOrdinaryHitEnergyV1,
    ),
    (
        "fireflies.all_in_range_v1",
        HypothesisId::FirefliesAllInRangeV1,
    ),
    ("fireflies.proc_v1", HypothesisId::FirefliesProcV1),
    ("fireflies.solo_proc_v2", HypothesisId::FirefliesSoloProcV2),
    (
        "fireflies.team_anima_hit_v1",
        HypothesisId::FirefliesTeamAnimaHitV1,
    ),
    (
        "gear.cast_buff_before_hits_v1",
        HypothesisId::GearCastBuffBeforeHitsV1,
    ),
    (
        "gear.qte_cast_buff_before_hits_v1",
        HypothesisId::GearQteCastBuffBeforeHitsV1,
    ),
    (
        "gear.skill_cast_buff_before_hits_v1",
        HypothesisId::GearSkillCastBuffBeforeHitsV1,
    ),
    (
        "haniel.aoe_all_in_range_dummy_v1",
        HypothesisId::HanielAoeAllInRangeDummyV1,
    ),
    (
        "haniel.cannon_ricochet_v1",
        HypothesisId::HanielCannonRicochetV1,
    ),
    ("haniel.full_kit_v1", HypothesisId::HanielFullKitV1),
    (
        "haniel.harmony_ensemble_v1",
        HypothesisId::HanielHarmonyEnsembleV1,
    ),
    ("haniel.normal_chain_v1", HypothesisId::HanielNormalChainV1),
    (
        "haniel.nova_extraction_v1",
        HypothesisId::HanielNovaExtractionV1,
    ),
    (
        "haniel.ordinary_clock_v1",
        HypothesisId::HanielOrdinaryClockV1,
    ),
    (
        "haniel.ordinary_hit_energy_v1",
        HypothesisId::HanielOrdinaryHitEnergyV1,
    ),
    (
        "haniel.skill_burst_packets_v1",
        HypothesisId::HanielSkillBurstPacketsV1,
    ),
    (
        "haniel.team_atk_hootie_clock_v1",
        HypothesisId::HanielTeamAtkHootieClockV1,
    ),
    ("hathor.airattack_v1", HypothesisId::HathorAirattackV1),
    (
        "hathor.aoe_all_in_range_dummy_v1",
        HypothesisId::HathorAoeAllInRangeDummyV1,
    ),
    ("hathor.b2_remora_v1", HypothesisId::HathorB2RemoraV1),
    (
        "hathor.branch_airattack_direct_packet_energy_v1",
        HypothesisId::HathorBranchAirattackDirectPacketEnergyV1,
    ),
    (
        "hathor.branch_airattack_direct_packet_inputs_v1",
        HypothesisId::HathorBranchAirattackDirectPacketInputsV1,
    ),
    ("hathor.branch_v1", HypothesisId::HathorBranchV1),
    (
        "hathor.burst_emergency_v1",
        HypothesisId::HathorBurstEmergencyV1,
    ),
    (
        "hathor.combat_lifecycle_v1",
        HypothesisId::HathorCombatLifecycleV1,
    ),
    ("hathor.cyclone_v1", HypothesisId::HathorCycloneV1),
    (
        "hathor.express_delivery_state_v1",
        HypothesisId::HathorExpressDeliveryStateV1,
    ),
    ("hathor.gaze_v1", HypothesisId::HathorGazeV1),
    ("hathor.held_skill_v1", HypothesisId::HathorHeldSkillV1),
    (
        "hathor.hybrid_emergency_v1",
        HypothesisId::HathorHybridEmergencyV1,
    ),
    ("hathor.normal_chain_v1", HypothesisId::HathorNormalChainV1),
    (
        "hathor.ordinary_clock_v1",
        HypothesisId::HathorOrdinaryClockV1,
    ),
    (
        "hathor.ordinary_hit_energy_v1",
        HypothesisId::HathorOrdinaryHitEnergyV1,
    ),
    (
        "hathor.secondary_actions_v1",
        HypothesisId::HathorSecondaryActionsV1,
    ),
    (
        "hathor.skill_burst_packets_v1",
        HypothesisId::HathorSkillBurstPacketsV1,
    ),
    ("hathor.tap_power_v1", HypothesisId::HathorTapPowerV1),
    (
        "hotori.aoe_all_in_range_dummy_v1",
        HypothesisId::HotoriAoeAllInRangeDummyV1,
    ),
    ("hotori.katana_chain_v1", HypothesisId::HotoriKatanaV1),
    ("hotori.mode_damage_v1", HypothesisId::HotoriModeDamageV1),
    ("hotori.normal_chain_v1", HypothesisId::HotoriNormalChainV1),
    (
        "hotori.ordinary_hit_energy_v1",
        HypothesisId::HotoriOrdinaryHitEnergyV1,
    ),
    (
        "hotori.recorded_replay_v1",
        HypothesisId::HotoriRecordedReplayV1,
    ),
    (
        "hotori.skill_burst_packets_v1",
        HypothesisId::HotoriSkillBurstPacketsV1,
    ),
    (
        "hotori.time_stop_clock_v1",
        HypothesisId::HotoriTimeStopClockV1,
    ),
    ("hotori.timepiece_v1", HypothesisId::HotoriTimepieceV1),
    (
        "inputs.critical_dodge_assumed_success_v1",
        HypothesisId::InputsCriticalDodgeAssumedSuccessV1,
    ),
    (
        "inputs.dash_directions_v1",
        HypothesisId::InputsDashDirectionsV1,
    ),
    (
        "inputs.special_dodge_timing_v1",
        HypothesisId::InputsSpecialDodgeTimingV1,
    ),
    ("iroi.action_clock_v1", HypothesisId::IroiActionClockV1),
    (
        "iroi.aoe_all_in_range_dummy_v1",
        HypothesisId::IroiAoeAllInRangeDummyV1,
    ),
    ("iroi.bud_copy_b2_v1", HypothesisId::IroiBudCopyB2V1),
    (
        "iroi.burst_post_cast_team_heal_v1",
        HypothesisId::IroiBurstPostCastTeamHealV1,
    ),
    (
        "iroi.burst_primary_packet_v1",
        HypothesisId::IroiBurstPrimaryPacketV1,
    ),
    ("iroi.critical_riposte_v1", HypothesisId::IroiRiposteV1),
    (
        "iroi.daydream_branch_v1",
        HypothesisId::IroiDaydreamBranchV1,
    ),
    ("iroi.dream_state_v1", HypothesisId::IroiDreamStateV1),
    (
        "iroi.effect3_healing_outgoing_v1",
        HypothesisId::IroiEffect3HealingOutgoingV1,
    ),
    (
        "iroi.live_atk_healing_v1",
        HypothesisId::IroiLiveAtkHealingV1,
    ),
    ("iroi.normal_chain_v1", HypothesisId::IroiNormalChainV1),
    (
        "iroi.ordinary_hit_energy_v1",
        HypothesisId::IroiOrdinaryHitEnergyV1,
    ),
    ("iroi.plunge_v1", HypothesisId::IroiPlungeV1),
    (
        "iroi.r3_team_def_aura_v1",
        HypothesisId::IroiR3TeamDefAuraV1,
    ),
    (
        "iroi.r6_team_crit_aura_v1",
        HypothesisId::IroiR6TeamCritAuraV1,
    ),
    ("iroi.regression_v1", HypothesisId::IroiRegressionV1),
    (
        "iroi.skill_nine_packet_train_v1",
        HypothesisId::IroiSkillNinePacketTrainV1,
    ),
    ("iroi.skill_team_heal_v1", HypothesisId::IroiSkillTeamHealV1),
    (
        "jiuyuan.aoe_all_in_range_dummy_v1",
        HypothesisId::JiuyuanAoeAllInRangeDummyV1,
    ),
    (
        "jiuyuan.natural_pact_v1",
        HypothesisId::JiuyuanNaturalPactV1,
    ),
    (
        "jiuyuan.normal_chain_v1",
        HypothesisId::JiuyuanNormalChainV1,
    ),
    (
        "jiuyuan.ordinary_hit_energy_v1",
        HypothesisId::JiuyuanOrdinaryHitEnergyV1,
    ),
    (
        "jiuyuan.skill_burst_packets_v1",
        HypothesisId::JiuyuanSkillBurstPacketsV1,
    ),
    (
        "lacrimosa.a6_free_reaction_entry_v1",
        HypothesisId::LacrimosaA6FreeReactionEntryV1,
    ),
    (
        "lacrimosa.aoe_all_in_range_dummy_v1",
        HypothesisId::LacrimosaAoeAllInRangeDummyV1,
    ),
    (
        "lacrimosa.bat_evade_exit_v1",
        HypothesisId::LacrimosaBatEvadeExitV1,
    ),
    (
        "lacrimosa.broken_shatter_v1",
        HypothesisId::LacrimosaBrokenShatterV1,
    ),
    ("lacrimosa.forms_v1", HypothesisId::LacrimosaFormsV1),
    (
        "lacrimosa.nightmare_all_targets_v1",
        HypothesisId::LacrimosaNightmareAllTargetsV1,
    ),
    (
        "lacrimosa.nightmare_dot_scheduling_v1",
        HypothesisId::LacrimosaNightmareDotSchedulingV1,
    ),
    (
        "lacrimosa.ordinary_hit_energy_v1",
        HypothesisId::LacrimosaOrdinaryHitEnergyV1,
    ),
    (
        "lacrimosa.ranged_normal_chain_v1",
        HypothesisId::LacrimosaRangedNormalChainV1,
    ),
    (
        "lacrimosa.skill_burst_packets_v1",
        HypothesisId::LacrimosaSkillBurstPacketsV1,
    ),
    ("lacrimosa.sleep_in_v1", HypothesisId::LacrimosaSleepInV1),
    (
        "linko.aoe_all_in_range_dummy_v1",
        HypothesisId::LinkoAoeAllInRangeDummyV1,
    ),
    ("linko.normal_chain_v1", HypothesisId::LinkoNormalChainV1),
    (
        "linko.ordinary_hit_energy_v1",
        HypothesisId::LinkoOrdinaryHitEnergyV1,
    ),
    (
        "linko.skill_four_hits_v1",
        HypothesisId::LinkoSkillFourHitsV1,
    ),
    ("linko.xiaozhen_pet_v1", HypothesisId::LinkoXiaozhenPetV1),
    (
        "mint.a3_reaction_trigger_v1",
        HypothesisId::MintA3ReactionTriggerV1,
    ),
    (
        "mint.aoe_all_in_range_dummy_v1",
        HypothesisId::MintAoeAllInRangeDummyV1,
    ),
    ("mint.hold_attack_v1", HypothesisId::MintHoldAttackV1),
    ("mint.normal_chain_v1", HypothesisId::MintNormalChainV1),
    ("mint.ordinary_clock_v1", HypothesisId::MintOrdinaryClockV1),
    (
        "mint.ordinary_hit_energy_v1",
        HypothesisId::MintOrdinaryHitEnergyV1,
    ),
    (
        "mint.passive2_entry_def_v1",
        HypothesisId::MintPassive2EntryDefV1,
    ),
    ("mint.plunge_v1", HypothesisId::MintPlungeV1),
    ("mint.resonance_3_v1", HypothesisId::MintResonance3V1),
    ("mint.riposte_v1", HypothesisId::MintRiposteV1),
    (
        "mint.skill_burst_packets_v1",
        HypothesisId::MintSkillBurstPacketsV1,
    ),
    (
        "mint.skill_coordinates_v1",
        HypothesisId::MintSkillCoordinatesV1,
    ),
    (
        "nanally.action_clock_v1",
        HypothesisId::NanallyActionClockV1,
    ),
    (
        "nanally.authority_a3_follow_up_v1",
        HypothesisId::NanallyAuthorityA3FollowUpV1,
    ),
    (
        "nanally.authority_removal_v1",
        HypothesisId::NanallyAuthorityRemovalV1,
    ),
    (
        "nanally.blossom_enhancement_v1",
        HypothesisId::NanallyBlossomEnhancementV1,
    ),
    (
        "nanally.burst_seven_hits_v1",
        HypothesisId::NanallyBurstSevenHitsV1,
    ),
    (
        "nanally.burst_summon_state_v1",
        HypothesisId::NanallyBurstSummonStateV1,
    ),
    (
        "nanally.combat_lifecycle_v1",
        HypothesisId::NanallyCombatLifecycleV1,
    ),
    (
        "nanally.critical_riposte_v1",
        HypothesisId::NanallyRiposteV1,
    ),
    (
        "nanally.cycle_followups_v1",
        HypothesisId::NanallyCycleFollowUpsV1,
    ),
    (
        "nanally.normal_chain_v1",
        HypothesisId::NanallyNormalChainV1,
    ),
    (
        "nanally.ordinary_hit_energy_v1",
        HypothesisId::NanallyOrdinaryHitEnergyV1,
    ),
    (
        "nanally.skill_five_hits_v1",
        HypothesisId::NanallySkillFiveHitsV1,
    ),
    (
        "nanally.support_packets_v1",
        HypothesisId::NanallySupportPacketsV1,
    ),
    (
        "nanally.underboss_coordination_v2",
        HypothesisId::NanallyUnderbossCoordinationV2,
    ),
    (
        "owner.action_role.character_1003_qte_v1",
        HypothesisId::OwnerActionRoleCharacter1003QteV1,
    ),
    (
        "owner.action_role.character_1004_melee_v1",
        HypothesisId::OwnerActionRoleCharacter1004MeleeV1,
    ),
    (
        "owner.action_role.character_1004_qte_v1",
        HypothesisId::OwnerActionRoleCharacter1004QteV1,
    ),
    (
        "owner.action_role.character_1004_skill_v1",
        HypothesisId::OwnerActionRoleCharacter1004SkillV1,
    ),
    (
        "owner.action_role.character_1004_ultimate_v1",
        HypothesisId::OwnerActionRoleCharacter1004UltimateV1,
    ),
    (
        "owner.action_role.character_1008_qte_v1",
        HypothesisId::OwnerActionRoleCharacter1008QteV1,
    ),
    (
        "owner.action_role.character_1010_qte_v1",
        HypothesisId::OwnerActionRoleCharacter1010QteV1,
    ),
    (
        "owner.action_role.character_1019_qte_v1",
        HypothesisId::OwnerActionRoleCharacter1019QteV1,
    ),
    (
        "owner.action_role.character_1020_qte_v1",
        HypothesisId::OwnerActionRoleCharacter1020QteV1,
    ),
    (
        "owner.action_role.character_1021_qte_v1",
        HypothesisId::OwnerActionRoleCharacter1021QteV1,
    ),
    (
        "owner.action_role.character_1023_qte_v1",
        HypothesisId::OwnerActionRoleCharacter1023QteV1,
    ),
    (
        "owner.action_role.character_1025_qte_v1",
        HypothesisId::OwnerActionRoleCharacter1025QteV1,
    ),
    (
        "owner.action_role.character_1033_qte_v1",
        HypothesisId::OwnerActionRoleCharacter1033QteV1,
    ),
    (
        "owner.action_role.character_1036_qte_v1",
        HypothesisId::OwnerActionRoleCharacter1036QteV1,
    ),
    (
        "owner.action_role.character_1039_qte_v1",
        HypothesisId::OwnerActionRoleCharacter1039QteV1,
    ),
    (
        "owner.action_role.character_1042_evolve_qte_v1",
        HypothesisId::OwnerActionRoleCharacter1042EvolveQteV1,
    ),
    (
        "owner.action_role.character_1042_qte_v1",
        HypothesisId::OwnerActionRoleCharacter1042QteV1,
    ),
    (
        "owner.action_role.character_1046_melee_v1",
        HypothesisId::OwnerActionRoleCharacter1046MeleeV1,
    ),
    (
        "owner.action_role.character_1046_qte_v1",
        HypothesisId::OwnerActionRoleCharacter1046QteV1,
    ),
    (
        "owner.action_role.character_1046_skill_v1",
        HypothesisId::OwnerActionRoleCharacter1046SkillV1,
    ),
    (
        "owner.action_role.character_1046_ultimate_v1",
        HypothesisId::OwnerActionRoleCharacter1046UltimateV1,
    ),
    (
        "owner.action_role.character_1051_qte_v1",
        HypothesisId::OwnerActionRoleCharacter1051QteV1,
    ),
    (
        "owner.action_role.character_1052_qte_v1",
        HypothesisId::OwnerActionRoleCharacter1052QteV1,
    ),
    (
        "owner.action_role.character_1054_qte_v1",
        HypothesisId::OwnerActionRoleCharacter1054QteV1,
    ),
    (
        "owner.action_role.character_1055_qte_v1",
        HypothesisId::OwnerActionRoleCharacter1055QteV1,
    ),
    (
        "owner.action_role.character_1057_qte_v1",
        HypothesisId::OwnerActionRoleCharacter1057QteV1,
    ),
    (
        "owner.action_role.character_1070_qte_v1",
        HypothesisId::OwnerActionRoleCharacter1070QteV1,
    ),
    (
        "owner.action_role.character_1071_qte_v1",
        HypothesisId::OwnerActionRoleCharacter1071QteV1,
    ),
    (
        "owner.action_role.character_1071_skill_v1",
        HypothesisId::OwnerActionRoleCharacter1071SkillV1,
    ),
    (
        "owner.action_role.character_1071_ultimate_v1",
        HypothesisId::OwnerActionRoleCharacter1071UltimateV1,
    ),
    (
        "owner.action_role.character_1072_qte_v1",
        HypothesisId::OwnerActionRoleCharacter1072QteV1,
    ),
    (
        "owner.action_role.character_1073_qte_v1",
        HypothesisId::OwnerActionRoleCharacter1073QteV1,
    ),
    (
        "owner.action_role.character_1075_qte_v1",
        HypothesisId::OwnerActionRoleCharacter1075QteV1,
    ),
    (
        "owner.action_role.character_1076_qte_v1",
        HypothesisId::OwnerActionRoleCharacter1076QteV1,
    ),
    (
        "owner.packet_binding.character_1010_skill_lv3_v1",
        HypothesisId::OwnerPacketBindingCharacter1010SkillLv3V1,
    ),
    (
        "published.preview_inputs_v1",
        HypothesisId::PublishedPreviewInputsV1,
    ),
    (
        "research.preview.single_packet_v1",
        HypothesisId::ResearchPreviewSinglePacketV1,
    ),
    (
        "sakiri.aoe_all_in_range_dummy_v1",
        HypothesisId::SakiriAoeAllInRangeDummyV1,
    ),
    ("sakiri.burst_atk_v1", HypothesisId::SakiriBurstAtkV1),
    ("sakiri.critical_riposte_v1", HypothesisId::SakiriRiposteV1),
    (
        "sakiri.direct_plunge_packet_energy_v1",
        HypothesisId::SakiriDirectPlungePacketEnergyV1,
    ),
    (
        "sakiri.direct_plunge_packet_v1",
        HypothesisId::SakiriDirectPlungePacketV1,
    ),
    ("sakiri.normal_chain_v1", HypothesisId::SakiriNormalChainV1),
    (
        "sakiri.ordinary_hit_energy_v1",
        HypothesisId::SakiriOrdinaryHitEnergyV1,
    ),
    (
        "sakiri.skill_burst_packets_v1",
        HypothesisId::SakiriSkillBurstPacketsV1,
    ),
    (
        "sakiri.support_packets_v1",
        HypothesisId::SakiriSupportPacketsV1,
    ),
    ("sakiri.suppress_v1", HypothesisId::SakiriSuppressV1),
    ("sets.crimson_v1", HypothesisId::SetsCrimsonV1),
    ("sets.devils_blood_v1", HypothesisId::SetsDevilsBloodV1),
    ("sets.diabolos_v1", HypothesisId::SetsDiabolosV1),
    ("sets.diabolos_v2", HypothesisId::SetsDiabolosV2),
    ("sets.hp_suit_v1", HypothesisId::SetsHpSuitV1),
    ("sets.lost_radiance_v1", HypothesisId::SetsLostRadianceV1),
    ("sets.quiet_manor_v1", HypothesisId::SetsQuietManorV1),
    ("sets.shadow_creed_v1", HypothesisId::SetsShadowCreedV1),
    ("sets.street_boxer_v1", HypothesisId::SetsStreetBoxerV1),
    ("sets.street_boxer_v2", HypothesisId::SetsStreetBoxerV2),
    (
        "shinku.aoe_all_in_range_dummy_v1",
        HypothesisId::ShinkuAoeAllInRangeDummyV1,
    ),
    (
        "shinku.burst_packets_v1",
        HypothesisId::ShinkuBurstPacketsV1,
    ),
    (
        "shinku.direct_ultimate_followups_v1",
        HypothesisId::ShinkuDirectUltimateFollowupsV1,
    ),
    ("shinku.normal_chain_v1", HypothesisId::ShinkuNormalChainV1),
    (
        "shinku.ordinary_hit_energy_v1",
        HypothesisId::ShinkuOrdinaryHitEnergyV1,
    ),
    (
        "shinku.pre_dawn_refresh_v1",
        HypothesisId::ShinkuPreDawnRefreshV1,
    ),
    (
        "shinku.skill_packets_v1",
        HypothesisId::ShinkuSkillPacketsV1,
    ),
    (
        "skia.gnaw_entry_state_v1",
        HypothesisId::SkiaGnawEntryStateV1,
    ),
    ("skia.normal_chain_v1", HypothesisId::SkiaNormalChainV1),
    (
        "skia.ordinary_hit_energy_v1",
        HypothesisId::SkiaOrdinaryHitEnergyV1,
    ),
    (
        "skia.skill_burst_packets_v1",
        HypothesisId::SkiaSkillBurstPacketsV1,
    ),
    (
        "skill_levels.coordinates_v1",
        HypothesisId::SkillLevelsCoordinatesV1,
    ),
    (
        "skill_levels.resonance_3_v1",
        HypothesisId::SkillLevelsResonance3V1,
    ),
    (
        "speedy_hedgehog.proc_v1",
        HypothesisId::SpeedyHedgehogProcV1,
    ),
    ("weapons.appliance_v1", HypothesisId::WeaponsApplianceV1),
    ("weapons.be_happy_v1", HypothesisId::WeaponsBeHappyV1),
    ("weapons.bit_game_v1", HypothesisId::WeaponsBitGameV1),
    (
        "weapons.bittercake_damage_event_v1",
        HypothesisId::WeaponsBittercakeDamageEventV1,
    ),
    ("weapons.black_book_v1", HypothesisId::WeaponsBlackBookV1),
    ("weapons.blast_candy_v1", HypothesisId::WeaponsBlastCandyV1),
    ("weapons.bopu_v1", HypothesisId::WeaponsBopuV1),
    (
        "weapons.boxing_candy_v1",
        HypothesisId::WeaponsBoxingCandyV1,
    ),
    (
        "weapons.call_of_the_twisted_city_v1",
        HypothesisId::WeaponsCallOfTheTwistedCityV1,
    ),
    (
        "weapons.contemplative_cat_v1",
        HypothesisId::WeaponsContemplativeCatV1,
    ),
    ("weapons.crowbar_v1", HypothesisId::WeaponsCrowbarV1),
    ("weapons.demon_blade_v1", HypothesisId::WeaponsDemonBladeV1),
    ("weapons.dustbin_v1", HypothesisId::WeaponsDustbinV1),
    (
        "weapons.echoes_on_hold_v1",
        HypothesisId::WeaponsEchoesOnHoldV1,
    ),
    (
        "weapons.eternal_waltz_v1",
        HypothesisId::WeaponsEternalWaltzV1,
    ),
    (
        "weapons.far_side_moon_v2",
        HypothesisId::WeaponsFarSideMoonV2,
    ),
    (
        "weapons.forgotten_hp_threshold_v1",
        HypothesisId::WeaponsForgottenHpThresholdV1,
    ),
    ("weapons.gold_record_v1", HypothesisId::WeaponsGoldRecordV1),
    ("weapons.gold_record_v2", HypothesisId::WeaponsGoldRecordV2),
    (
        "weapons.gold_wool_chaos_realfinish_v1",
        HypothesisId::WeaponsGoldWoolChaosRealfinishV1,
    ),
    ("weapons.gold_wool_v1", HypothesisId::WeaponsGoldWoolV1),
    ("weapons.good_boy_v1", HypothesisId::WeaponsGoodBoyV1),
    ("weapons.good_boy_v2", HypothesisId::WeaponsGoodBoyV2),
    ("weapons.good_boy_v3", HypothesisId::WeaponsGoodBoyV3),
    ("weapons.jiaojuan_v1", HypothesisId::WeaponsJiaojuanV1),
    ("weapons.jiaojuan_v2", HypothesisId::WeaponsJiaojuanV2),
    (
        "weapons.jingmotingyuan_v1",
        HypothesisId::WeaponsJingmotingyuanV1,
    ),
    ("weapons.kite_v1", HypothesisId::WeaponsKiteV1),
    ("weapons.kite_v2", HypothesisId::WeaponsKiteV2),
    (
        "weapons.knight_candy_v1",
        HypothesisId::WeaponsKnightCandyV1,
    ),
    ("weapons.koinobori_v1", HypothesisId::WeaponsKoinoboriV1),
    ("weapons.koinobori_v2", HypothesisId::WeaponsKoinoboriV2),
    ("weapons.lunarphase_v1", HypothesisId::WeaponsLunarphaseV1),
    (
        "weapons.marching_beyond_time_v1",
        HypothesisId::WeaponsMarchingBeyondTimeV1,
    ),
    ("weapons.mind_royale_v1", HypothesisId::WeaponsMindRoyaleV1),
    ("weapons.moon_v1", HypothesisId::WeaponsMoonV1),
    ("weapons.motor_candy_v1", HypothesisId::WeaponsMotorCandyV1),
    ("weapons.nakupeda_v1", HypothesisId::WeaponsNakupedaV1),
    (
        "weapons.native_curve_v1",
        HypothesisId::WeaponsNativeCurveV1,
    ),
    ("weapons.nest_bird_v1", HypothesisId::WeaponsNestBirdV1),
    ("weapons.nest_bird_v2", HypothesisId::WeaponsNestBirdV2),
    ("weapons.noros_v1", HypothesisId::WeaponsNorosV1),
    ("weapons.oraora_v1", HypothesisId::WeaponsOraoraV1),
    ("weapons.paper_plane_v1", HypothesisId::WeaponsPaperPlaneV1),
    ("weapons.police_rat_v2", HypothesisId::WeaponsPoliceRatV2),
    ("weapons.police_rat_v3", HypothesisId::WeaponsPoliceRatV3),
    ("weapons.prokaryon_v1", HypothesisId::WeaponsProkaryonV1),
    (
        "weapons.raging_flames_v1",
        HypothesisId::WeaponsRagingFlamesV1,
    ),
    (
        "weapons.reality_refuge_v1",
        HypothesisId::WeaponsRealityRefugeV1,
    ),
    (
        "weapons.reality_refuge_v2",
        HypothesisId::WeaponsRealityRefugeV2,
    ),
    ("weapons.rishi_v2", HypothesisId::WeaponsRishiV2),
    (
        "weapons.rose_break_extension_v1",
        HypothesisId::WeaponsRoseBreakExtensionV1,
    ),
    ("weapons.rose_v1", HypothesisId::WeaponsRoseV1),
    ("weapons.snowman_v1", HypothesisId::WeaponsSnowmanV1),
    ("weapons.spider_v1", HypothesisId::WeaponsSpiderV1),
    (
        "weapons.the_wrong_gate_healing_v1",
        HypothesisId::WeaponsTheWrongGateHealingV1,
    ),
    ("weapons.thief_candy_v1", HypothesisId::WeaponsThiefCandyV1),
    ("weapons.tiger_tally_v2", HypothesisId::WeaponsTigerTallyV2),
    ("weapons.tuansanlang_v1", HypothesisId::WeaponsTuansanlangV1),
    ("weapons.twinbirds_v1", HypothesisId::WeaponsTwinbirdsV1),
    ("weapons.twinbirds_v2", HypothesisId::WeaponsTwinbirdsV2),
    ("weapons.whale_v1", HypothesisId::WeaponsWhaleV1),
    ("weapons.worldrain_v1", HypothesisId::WeaponsWorldrainV1),
    (
        "weapons.yaodao_separate_packet_v1",
        HypothesisId::WeaponsYaodaoSeparatePacketV1,
    ),
    ("weapons.yaodao_v1", HypothesisId::WeaponsYaodaoV1),
    ("weapons.yuren_v1", HypothesisId::WeaponsYurenV1),
    (
        "zankou.combat_passives_v1",
        HypothesisId::ZankouCombatPassivesV1,
    ),
    (
        "zankou.form_modifiers_v1",
        HypothesisId::ZankouFormModifiersV1,
    ),
    ("zankou.forms_v1", HypothesisId::ZankouFormsV1),
    ("zankou.normal_chain_v1", HypothesisId::ZankouNormalChainV1),
    (
        "zankou.ordinary_hit_energy_v1",
        HypothesisId::ZankouOrdinaryHitEnergyV1,
    ),
    ("zankou.story_mode_v1", HypothesisId::ZankouStoryModeV1),
    ("zero.a1_first_hit_v1", HypothesisId::ZeroA1FirstHitV1),
    (
        "zero.aoe_all_in_range_dummy_v1",
        HypothesisId::ZeroAoeAllInRangeDummyV1,
    ),
    (
        "zero.burst_a3_crit_rate_v1",
        HypothesisId::ZeroBurstA3CritRateV1,
    ),
    (
        "zero.burst_a4_damage_bonus_v1",
        HypothesisId::ZeroBurstA4DamageBonusV1,
    ),
    (
        "zero.burst_all_in_range_v1",
        HypothesisId::ZeroBurstAllInRangeV1,
    ),
    ("zero.burst_packets_v1", HypothesisId::ZeroBurstPacketsV1),
    (
        "zero.cosmos_cycle_atk_v1",
        HypothesisId::ZeroCosmosCycleAtkV1,
    ),
    ("zero.normal_chain_v1", HypothesisId::ZeroNormalChainV1),
    (
        "zero.ordinary_hit_energy_v1",
        HypothesisId::ZeroOrdinaryHitEnergyV1,
    ),
    ("zero.skill_packets_v1", HypothesisId::ZeroSkillPacketsV1),
    (
        "zero.support_packets_v1",
        HypothesisId::ZeroSupportPacketsV1,
    ),
];

impl HypothesisId {
    /// Returns the canonical string identifier for this hypothesis.
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        HYPOTHESES[*self as usize].id
    }

    /// Returns the full text description of this hypothesis from the canonical catalog.
    #[inline]
    pub const fn description(&self) -> &'static str {
        HYPOTHESES[*self as usize].description
    }

    /// Returns a lazy description wrapper for backwards compatibility.
    #[inline]
    pub const fn desc(&self) -> HypothesisDescription {
        HypothesisDescription(*self)
    }

    /// Parses a string into a known strongly typed HypothesisId.
    pub fn parse(s: &str) -> Option<Self> {
        PARSE_TABLE
            .binary_search_by_key(&s, |entry| entry.0)
            .ok()
            .map(|idx| PARSE_TABLE[idx].1)
    }
}

impl Deref for HypothesisId {
    type Target = str;
    #[inline]
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<str> for HypothesisId {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::borrow::Borrow<str> for HypothesisId {
    #[inline]
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl From<HypothesisId> for String {
    #[inline]
    fn from(id: HypothesisId) -> Self {
        id.as_str().to_owned()
    }
}

impl From<HypothesisId> for &'static str {
    #[inline]
    fn from(id: HypothesisId) -> Self {
        id.as_str()
    }
}

impl fmt::Display for HypothesisId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl PartialEq<str> for HypothesisId {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<&str> for HypothesisId {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<String> for HypothesisId {
    #[inline]
    fn eq(&self, other: &String) -> bool {
        self.as_str() == other.as_str()
    }
}

impl PartialEq<serde_json::Value> for HypothesisId {
    #[inline]
    fn eq(&self, other: &serde_json::Value) -> bool {
        other.as_str().is_some_and(|s| s == self.as_str())
    }
}

impl Serialize for HypothesisId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for HypothesisId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::parse(&s)
            .ok_or_else(|| serde::de::Error::custom(format!("unknown hypothesis id: {s}")))
    }
}

/// Backward compatibility wrapper that formats or serializes a hypothesis description.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HypothesisDescription(pub HypothesisId);

impl Deref for HypothesisDescription {
    type Target = str;
    #[inline]
    fn deref(&self) -> &str {
        self.0.description()
    }
}

impl AsRef<str> for HypothesisDescription {
    #[inline]
    fn as_ref(&self) -> &str {
        self.0.description()
    }
}

impl fmt::Display for HypothesisDescription {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0.description())
    }
}

impl Serialize for HypothesisDescription {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.0.description())
    }
}

/// Returns the full description of a hypothesis by id string from the catalog.
pub fn get_hypothesis_description(id: &str) -> Option<&'static str> {
    HypothesisId::parse(id).map(|hid| hid.description())
}

pub fn hypothesis_descriptions() -> impl Iterator<Item = (&'static str, &'static str)> {
    HYPOTHESES
        .iter()
        .map(|hypothesis| (hypothesis.id, hypothesis.description))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typed_assumptions_roundtrip_without_unused_json_metadata() {
        for (id, description) in hypothesis_descriptions() {
            let parsed = HypothesisId::parse(id).unwrap();
            assert_eq!(parsed.as_str(), id);
            assert_eq!(parsed.description(), description);
            assert!(!description.is_empty());
        }
        assert!(HypothesisId::parse("combat.target_static_profile_v1").is_none());
    }
}
