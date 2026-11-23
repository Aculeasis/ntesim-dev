use crate::simulation::ExactFraction;
use ntesim_core::formula::Ratio;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ContentEvent {
    ZankouCombat(crate::characters::zankou::combat_passives::Event),
    ZankouBurst(crate::characters::zankou::burst::Event),
    ZankouForms(crate::characters::zankou::forms::Event),
    IroiClock(crate::characters::iroi::clock::Event),
    MintClock(crate::characters::mint::clock::Event),
    NanallyClock(crate::characters::nanally::clock::Event),
    HathorEmergency(crate::characters::hathor::burst::Expiry),
    HathorGaze(crate::characters::hathor::gaze::Event),
    HathorClock(crate::characters::hathor::clock::Event),
    HathorKill(crate::characters::hathor::power::Kill),
    HathorRiposte(crate::characters::hathor::secondary::Reward),
    HathorAir(crate::characters::hathor::air::Event),
    HathorBranch(crate::characters::hathor::branch::Event),
    HanielClock(crate::characters::haniel::clock::Event),
    HanielCannon(crate::characters::haniel::cannon::Event),
    HotoriTimepiece(crate::characters::hotori::timepiece::Event),
    HotoriKatana(crate::characters::hotori::katana::Event),
    Fadia(crate::characters::fadia::clock::Event),
    FadiaEffects(crate::characters::fadia::effects::Event),
    Edgar(crate::characters::edgar::clock::Event),
    Daffodil(crate::characters::daffodil::clock::Event),
    Chiz(crate::characters::chiz::events::Event),
    Chaos(crate::characters::chaos::events::Event),
    Blackbird(crate::characters::blackbird::events::Event),
    BaicangFadiaCheck {
        actor_index: usize,
        trigger: crate::simulation::GameCausalTrigger,
    },
    SkiaEntryState {
        actor_index: usize,
        generation: u64,
        boundary: crate::characters::skia::entry_state::Boundary,
    },
    JiuyuanPactApply(crate::characters::jiuyuan::pact::Application),
    JiuyuanPactTick {
        actor_index: usize,
        target_index: usize,
        planted_us: u64,
    },
    JiuyuanPactClose {
        actor_index: usize,
        target_index: usize,
        generation: u64,
    },
    JiuyuanForceGrow {
        actor_index: usize,
    },
    Haniel(crate::characters::haniel::hootie::Event),
    TimedBuffGrant(Box<crate::modifiers::TimedBuffGrant>),
    QuietManorMentalStack(crate::sets::quiet_manor::StackGrant),
    HpSuitMaxHpStack(crate::sets::hp_suit::StackGrant),
    CrimsonAtkStack(crate::sets::crimson::StackGrant),
    MotorCandyTick {
        actor_index: usize,
        generation: u64,
    },
    RagingFlamesUltimateGrant {
        actor_index: usize,
        redirect_ultimate_bonus: Ratio,
        duration_us: u64,
    },
    RagingFlamesSkillStack {
        actor_index: usize,
        redirect_skill_stack_bonus: Ratio,
    },
    LunarPhaseProcGrant {
        actor_index: usize,
        cosmos_bonus: Ratio,
        def_ignore: Ratio,
        duration_us: u64,
    },
    DemonBladeCritStack {
        actor_index: usize,
        stack_amount: Ratio,
        duration_us: u64,
    },
    MoonCritStack {
        actor_index: usize,
        stack_amount: Ratio,
        duration_us: u64,
    },
    GoldRecordUltCritStack {
        actor_index: usize,
        stack_amount: Ratio,
        duration_us: u64,
    },
    SpiderKnowledgeStack {
        actor_index: usize,
    },
    BeHappyHeal {
        actor_index: usize,
        target_index: usize,
        base_heal: ExactFraction,
    },
    WhaleHeal {
        actor_index: usize,
        target_index: usize,
        base_heal: ExactFraction,
    },
    NakupedaProc {
        actor_index: usize,
        outcome_index: u64,
        recipients: Vec<usize>,
        base_amounts: Vec<ExactFraction>,
        shield_duration_us: u64,
    },
    AdlerBlessingShield {
        granter_index: usize,
        recipient_index: usize,
        amount: ExactFraction,
        duration_us: u64,
    },
    AdlerSkillTick(crate::characters::adler::skill::SkillTick),
    ZankouVileAshApply(crate::characters::zankou::vile_ash::VileAshApplication),
    ZankouSpread {
        source_actor: usize,
        from_target: usize,
    },
    LacrimosaNightmareApply(crate::characters::lacrimosa::dot::NightmareApplication),
    EdgarHealTick {
        actor_index: usize,
        generation: u64,
        end_time_us: u64,
        interval_us: u64,
        flat_heal: ExactFraction,
        max_hp_ratio: ExactFraction,
    },
    IroiSkillHeal {
        source_actor_index: usize,
        recipient_actor_index: usize,
        atk_ratio: ExactFraction,
    },
    IroiBurstHeal {
        command_index: usize,
        source_actor_index: usize,
        recipient_actor_index: usize,
        atk_ratio: ExactFraction,
    },
    BaicangDomainTick {
        actor_index: usize,
        target_index: usize,
        command_index: usize,
        generation: u64,
        end_time_us: u64,
        interval_us: u64,
        ability_id: String,
        damage_effect_id: String,
        atk_rate: ntesim_core::formula::Ratio,
        scaling_stat: crate::simulation::NativeScalingStat,
        metadata: Box<crate::simulation::metadata::ActionPacketMetadata>,
    },
    BaicangHitstop {
        command_index: usize,
        actor_index: usize,
        duration_us: u64,
        rate_ppm: u32,
    },
    BaicangWordSilenceA1 {
        actor_index: usize,
    },
    BaicangWordObjurgateA1 {
        actor_index: usize,
    },
    BaicangWordHpConsumed {
        actor_index: usize,
        amount: u64,
    },
    BaicangWordHeal {
        actor_index: usize,
        percent: u64,
    },
    BaicangFatalDebtInstallment {
        actor_index: usize,
        amount: u64,
    },
    BaicangDomainEnd {
        actor_index: usize,
        generation: u64,
    },
    BaicangFieldWords {
        actor_index: usize,
        command_index: usize,
        generation: u64,
        array_index: u32,
    },
    SpiderTeamAtkGrant {
        actor_index: usize,
        team_atk_bonus: Ratio,
        duration_us: u64,
    },
    BitGameOffFieldStack {
        actor_index: usize,
        off_field_atk_per_stack: Ratio,
    },
    BitGameOnFieldStack {
        actor_index: usize,
        on_field_psyche_per_stack: Ratio,
    },
    FarSideMoonGrant {
        actor_index: usize,
        basic_attack_damage_bonus: Ratio,
        lakshana_damage_bonus: Ratio,
        duration_us: u64,
    },
    FarSideMoonSkillStack {
        actor_index: usize,
        stack_amount: Ratio,
    },
    KnightCandyCritStack {
        actor_index: usize,
        stack_amount: Ratio,
        duration_us: u64,
    },
    OraoraBasicAttackStack {
        actor_index: usize,
        stack_amount: Ratio,
        duration_us: u64,
    },
    MindRoyaleEnergyGrant {
        actor_index: usize,
        amount: u64,
    },
    FirefliesCritStack {
        actor_index: usize,
        stack_amount: Ratio,
        max_stacks: u32,
        duration_us: u64,
    },
    PoliceRatWhiskerTick {
        actor_index: usize,
        target_index: usize,
        generation: u64,
        end_time_us: u64,
        interval_us: u64,
        damage_ratio: Ratio,
    },
    BlackBookDesignateTick {
        actor_index: usize,
        generation: u64,
        end_time_us: u64,
        interval_us: u64,
    },
    BlackBookArcStartupComplete {
        actor_index: usize,
        command_index: usize,
        generation: u64,
        cooldown_until_us: u64,
        parameters: crate::weapons::black_book::arc::Parameters,
    },
    BuffLifecycleExpiry,
    BlossomBudExpiry {
        bud_id: u64,
        is_copy: bool,
    },
    SakiriAtkBuffGrant {
        a4: bool,
        source_actor_index: usize,
        atk_add: ExactFraction,
        duration_us: u64,
    },
    IroiSkillBuffGrant {
        source_actor_index: usize,
        atk_add: ExactFraction,
        def_ignore: ExactFraction,
        a5_atk_add: Option<ExactFraction>,
        duration_us: u64,
    },
    IroiA2Assist {
        actor_index: usize,
    },
    IroiA6ReviveGrant {
        actor_index: usize,
    },
    MintA6EnergyGrant {
        actor_index: usize,
        amount: u64,
    },
    ZeroA2EnergyGrant {
        actor_index: usize,
        amount: u64,
    },
    ZeroA5AtkBuffGrant {
        actor_index: usize,
        duration_us: u64,
    },
    NanallyA1EnergyGrant {
        actor_index: usize,
    },
    NanallyUnderboss(Box<crate::characters::nanally::underboss::CoordinatedAttack>),
    AdlerA1KarmaGrant {
        actor_index: usize,
    },
    AdlerA5EnergyGrant {
        actor_index: usize,
    },
    NanallyA5AtkBuff {
        actor_index: usize,
        duration_us: u64,
    },
    RishiEclipseField {
        actor_index: usize,
    },
    RishiEclipseEnergy {
        actor_index: usize,
        amount: u64,
    },
    IroiDreamRelease {
        actor_index: usize,
        command_index: usize,
        generation: u64,
    },
    IroiDreamTick {
        actor_index: usize,
        generation: u64,
        tick_index: u64,
        coordinate: u32,
    },
    IroiDreamEnd {
        actor_index: usize,
        generation: u64,
    },
    IroiBudCopyCheck {
        bud_id: u64,
        snapshot: Box<crate::simulation::reactions::ReactionSnapshot>,
    },
    AureliaHarmonicsHit {
        actor_index: usize,
        target_index: usize,
        command_key: usize,
    },
    AureliaSkillFinish {
        actor_index: usize,
    },
    LinkoXiaozhenPulse {
        actor_index: usize,
        generation: u64,
        pulse_index: u64,
    },
}
