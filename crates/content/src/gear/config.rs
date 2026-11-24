use crate::formula::Ratio;
use serde::{Deserialize, Serialize};
fn is_false(value: &bool) -> bool {
    !*value
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum GearProcConfig {
    DevilsBloodDamage {
        damage_bonus: Ratio,
        affected_bonus: Ratio,
    },
    NorosAtk {
        atk_bonus: Ratio,
        duration_us: u64,
        cooldown_us: u64,
    },
    GoldWool {
        lakshana_bonus: Ratio,
        crit_damage_bonus: Ratio,
        duration_us: u64,
    },
    MotorCandyAtk {
        atk_per_stack: Ratio,
        interval_us: u64,
    },
    ShadowCreedAtk {
        atk_bonus: Ratio,
        duration_us: u64,
    },
    QuietManorMental {
        mental_bonus_per_stack: Ratio,
        duration_us: u64,
        max_stacks: u32,
    },
    KnightCandyCritDamage {
        crit_damage_per_stack: Ratio,
        duration_us: u64,
    },
    OraoraBasicAttackDamage {
        damage_bonus_per_stack: Ratio,
        duration_us: u64,
    },
    BoxingCandyDamage {
        damage_bonus: Ratio,
        low_hp_damage_bonus: Ratio,
    },
    ForgottenHpStats {
        atk_bonus: Ratio,
        def_bonus: Ratio,
    },
    ApplianceDamage {
        damage_bonus: Ratio,
    },
    ProkaryonDamage {
        damage_bonus: Ratio,
    },
    PaperPlaneDamage {
        damage_bonus: Ratio,
    },
    WorldrainDamage {
        damage_bonus: Ratio,
        cycle_intensity_bonus: Ratio,
        duration_us: u64,
    },
    ButterflyDamage {
        anima_bonus: Ratio,
        attachment_bonus: Ratio,
        ult_attachment_bonus: Ratio,
        ult_duration_us: u64,
    },
    MamenFons {
        cosmos_bonus_per_stack: Ratio,
        fons_per_stack: u64,
        max_stacks: u32,
    },
    KiteAtk {
        atk_bonus: Ratio,
        lakshana_bonus: Ratio,
        duration_us: u64,
    },
    WhaleAtk {
        atk_bonus: Ratio,
        broken_damage_bonus: Ratio,
        kill_heal: Ratio,
        kill_heal_cooldown_us: u64,
    },
    CrowbarBreak {
        break_intensity_bonus: Ratio,
        duration_us: u64,
    },
    NestBirdGaze {
        damage_reduction: Ratio,
        duration_us: u64,
    },
    JiaojuanBreak {
        break_intensity_bonus: Ratio,
        broken_damage_bonus: Ratio,
    },
    SnowmanShieldAtk {
        atk_bonus: Ratio,
    },
    YurenHp {
        hp_bonus: Ratio,
        shield_bonus: Ratio,
    },
    TuansanlangTeam {
        break_intensity_bonus: Ratio,
    },
    YaodaoParry {
        parry_damage_ratio: Ratio,
    },
    ThiefCandyDodge {
        damage_bonus: Ratio,
        duration_us: u64,
    },
    DustbinBreak {
        break_intensity_bonus: Ratio,
        duration_us: u64,
        cooldown_us: u64,
    },
    PoliceRatBoss {
        atk_bonus: Ratio,
        boss_damage_bonus: Ratio,
    },
    RoseThorn {
        atk_bonus: Ratio,
        crit_damage_per_stack: Ratio,
        duration_us: u64,
        broken_extension_us: u64,
    },
    TigerTallyAtk {
        atk_bonus: Ratio,
        normal_damage_bonus: Ratio,
        duration_us: u64,
        talisman_window_us: u64,
        boss_damage_bonus: Ratio,
        boss_damage_duration_us: u64,
    },
    Nakupeda {
        hp_bonus: Ratio,
        lowest_hp_heal_ratio: Ratio,
        shield_ratio: Ratio,
        shield_duration_us: u64,
        team_heal_ratio: Ratio,
        cooldown_us: u64,
    },
    BlackBookBreak {
        break_intensity_bonus: Ratio,
    },
    JingmotingyuanCrit {
        crit_damage_bonus: Ratio,
        duration_us: u64,
    },
    RishiAtk {
        atk_bonus: Ratio,
        eclipse_duration_us: u64,
        defeat_energy: u64,
        max_defeats: u32,
        arc_cooldown_us: u64,
    },
    BlastCandyAtk {
        atk_bonus: Ratio,
        duration_us: u64,
    },
    ChargeEfficiency {
        multiplier: Ratio,
    },
    SpeedyHedgehogTeamAtk {
        team_atk_bonus: Ratio,
        duration_us: u64,
    },
    MindRoyaleEnergy {
        energy_grant: u64,
        cooldown_us: u64,
    },
    FirefliesCritDamage {
        crit_damage_per_stack: Ratio,
        max_stacks: u32,
        duration_us: u64,
    },
    GoodBoyTeamAtk {
        team_atk_bonus: Ratio,
        duration_us: u64,
        #[serde(default = "Ratio::zero")]
        control_atk_bonus: Ratio,
    },
    LostRadianceDefIgnore {
        def_ignore: Ratio,
        duration_us: u64,
    },
    Twinbirds {
        cycle_intensity_bonus: Ratio,
        psyche_bonus: Ratio,
        team_crit_damage_bonus: Ratio,
        self_crit_damage_bonus: Ratio,
        duration_us: u64,
    },
    EternalWaltz {
        max_hp_bonus: Ratio,
        mental_bonus: Ratio,
        duration_us: u64,
    },
    RagingFlames {
        lakshana_bonus: Ratio,
        redirect_ultimate_bonus: Ratio,
        redirect_skill_stack_bonus: Ratio,
        duration_us: u64,
    },
    LunarPhase {
        atk_bonus: Ratio,
        cosmos_bonus: Ratio,
        def_ignore: Ratio,
        duration_us: u64,
    },
    DemonBlade {
        crit_rate_bonus: Ratio,
        crit_damage_per_stack: Ratio,
        duration_us: u64,
        icd_us: u64,
    },
    Moon {
        psyche_bonus: Ratio,
        crit_damage_per_stack: Ratio,
        duration_us: u64,
        icd_us: u64,
    },
    EchoesOnHold {
        atk_bonus: Ratio,
        team_def_ignore: Ratio,
        wearer_def_ignore: Ratio,
        cosmos_penetration: Ratio,
        duration_us: u64,
    },
    FarSideMoon {
        crit_rate_bonus: Ratio,
        skill_damage_per_stack: Ratio,
        basic_attack_damage_bonus: Ratio,
        lakshana_damage_bonus: Ratio,
        duration_us: u64,
    },
    BopuDamage {
        damage_bonus: Ratio,
        duration_us: u64,
        cooldown_us: u64,
    },
    KoinoboriStats {
        atk_bonus: Ratio,
        def_bonus: Ratio,
        hp_bonus: Ratio,
    },
    GoldRecord {
        atk_bonus: Ratio,
        qte_crit_damage_bonus: Ratio,
        ult_crit_damage_per_stack: Ratio,
        duration_us: u64,
    },
    MarchingBeyondTime {
        atk_bonus: Ratio,
        state_crit_damage_bonus: Ratio,
        crit_damage_per_stack: Ratio,
        def_ignore: Ratio,
        def_ignore_duration_us: u64,
    },
    BeHappy {
        heal_ratio: Ratio,
        cooldown_us: u64,
    },
    BitterCake {
        def_bonus: Ratio,
        duration_us: u64,
        cooldown_us: u64,
    },
    CastleHealingBonus {
        healing_bonus: Ratio,
        duration_us: u64,
    },
    WrongGate {
        atk_bonus: Ratio,
        anima_damage_bonus: Ratio,
        ally_damage_bonus: Ratio,
        duration_us: u64,
    },
    SpiderTeamAtk {
        atk_bonus_per_stack: Ratio,
        atk_bonus_max_stack: Ratio,
        duration_us: u64,
    },
    BitGame {
        off_field_atk_base: Ratio,
        off_field_atk_per_stack: Ratio,
        off_field_icd_us: u64,
        on_field_psyche_base: Ratio,
        on_field_psyche_per_stack: Ratio,
    },
    HpSuitMaxHp {
        max_hp_bonus_per_stack: Ratio,
        duration_us: u64,
        max_stacks: u32,
    },
    CrimsonAtk {
        atk_bonus_per_stack: Ratio,
        duration_us: u64,
        max_stacks: u32,
    },
    DiabolosChaosIgnore {
        chaos_res_ignore: Ratio,
        powered_chaos_res_ignore: Ratio,
        duration_us: u64,
    },
    StreetBoxerCrit {
        crit_rate_bonus: Ratio,
        powered_crit_rate_bonus: Ratio,
        duration_us: u64,
        #[serde(default, skip_serializing_if = "is_false")]
        baseline_in_stats: bool,
    },
}
