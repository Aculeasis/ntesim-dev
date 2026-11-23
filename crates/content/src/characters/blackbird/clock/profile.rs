use super::*;

/// Owner action family. Gates and stage bookkeeping follow this kind, never the caller.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    OrdinaryNormal(usize),
    EvolvedNormal(usize),
    Roostcall,
    BestowedGrace,
    Tidings,
    Plunge,
    EvolvedPlunge,
    Riposte,
    EvolvedRiposte,
    Skill,
    EvolvedSkill,
    Burst,
    Finisher,
    Support,
    EvolvedSupport,
    Dodge,
}

/// Earliest cancellation gate for one input family. `AnySkill` equals full recovery.
#[derive(Clone, Copy, Debug)]
pub(super) enum Gate {
    At(u64),
    AnySkill,
}

/// One spawned actor whose actor-local hits run on the real axis after its spawn progress.
#[derive(Clone, Copy, Debug)]
pub(super) struct Spawn {
    pub(super) offset: u64,
    pub(super) period_us: u64,
    pub(super) count: usize,
}

pub(super) struct Profile {
    pub(super) kind: Kind,
    pub(super) offsets: Vec<u64>,
    pub(super) spawn: Option<Spawn>,
    pub(super) recovery: u64,
    pub(super) switch_lock: Option<u64>,
    pub(super) jump: Gate,
    pub(super) skill: Gate,
    pub(super) movement: Gate,
}

// Nearest microsecond from native montage progress, before causal hit-pause.
const NORMAL: [&[u64]; 5] = [
    &[149_945],
    &[100_530, 364_983],
    &[334_176, 383_619, 450_636, 516_319],
    &[266_996, 333_880, 401_357, 467_609, 534_291],
    &[233_702, 351_722, 477_447, 603_203, 717_899],
];
const RECOVERY: [u64; 5] = [755_920, 733_943, 1_730_205, 1_298_910, 916_483];
const EVOLVE_NORMAL: [&[u64]; 5] = [
    &[167_452, 350_737],
    &[93_687, 232_254, 367_563, 399_685, 434_895],
    &[183_802, 365_976, 417_309, 466_722, 517_269, 566_893],
    &[232_953],
    &[],
];
const EVOLVE_RECOVERY: [u64; 5] = [986_910, 762_652, 1_750_635, 1_064_327, 1_178_164];
const EVOLVE_SPAWN: Spawn = Spawn {
    offset: 337_118,
    period_us: 120_000,
    count: 6,
};
const ROOSTCALL: (&[u64], u64) = (&[200_476], 757_210);
const BESTOWED_GRACE: (&[u64], u64) = (&[617_712, 733_904, 852_485, 967_517, 1_083_830], 1_345_821);
const TIDINGS: (&[u64], u64) = (&[143_845, 283_795, 441_973, 582_454, 730_677], 975_109);
const BURST_OFFSETS: &[u64] = &[
    3_217_557, 3_458_336, 3_707_061, 4_149_342, 4_250_603, 4_349_944, 4_450_082, 4_549_710,
    4_649_931,
];
const FINISHER_OFFSETS: &[u64] = &[
    2_433_333, 2_583_333, 2_733_333, 2_883_333, 3_033_333, 3_183_333, 3_350_517,
];
const SKILL_OFFSETS: &[u64] = &[67_549, 416_323, 717_071, 901_773];
const EVOLVE_SKILL_OFFSETS: &[u64] = &[133_333, 277_319, 449_842, 683_994];

// Earliest cancel gate per action family, before causal hit-pause.
const ORDINARY_JUMP: [u64; 5] = [184_549, 336_276, 368_364, 888_358, 765_616];
const ORDINARY_SKILL: [Gate; 5] = [
    Gate::At(333_077),
    Gate::At(502_016),
    Gate::At(667_925),
    Gate::At(639_014),
    Gate::AnySkill,
];
const ORDINARY_MOVEMENT: [u64; 5] = [754_313, 752_055, 1_001_773, 1_301_300, 1_286_865];
const EVOLVED_JUMP: [u64; 5] = [173_066, 103_925, 175_358, 185_907, 524_134];
const EVOLVED_SKILL: [Gate; 5] = [
    Gate::At(644_213),
    Gate::At(579_390),
    Gate::At(966_451),
    Gate::AnySkill,
    Gate::AnySkill,
];
const EVOLVED_MOVEMENT: [u64; 5] = [1_010_629, 769_638, 1_752_073, 1_054_906, 1_847_584];

fn normal_kind(evolved: bool, stage: usize) -> Kind {
    if evolved {
        Kind::EvolvedNormal(stage)
    } else {
        Kind::OrdinaryNormal(stage)
    }
}

fn gates(kind: Kind) -> (Gate, Gate, Gate) {
    match kind {
        Kind::OrdinaryNormal(stage) => (
            Gate::At(ORDINARY_JUMP[stage]),
            ORDINARY_SKILL[stage],
            Gate::At(ORDINARY_MOVEMENT[stage]),
        ),
        Kind::EvolvedNormal(stage) => (
            Gate::At(EVOLVED_JUMP[stage]),
            EVOLVED_SKILL[stage],
            Gate::At(EVOLVED_MOVEMENT[stage]),
        ),
        Kind::Roostcall => (Gate::At(250_451), Gate::At(667_489), Gate::At(1_010_329)),
        Kind::BestowedGrace => (Gate::At(666_329), Gate::At(1_199_905), Gate::AnySkill),
        Kind::Tidings => (Gate::At(840_242), Gate::AnySkill, Gate::At(1_762_101)),
        Kind::Plunge => (Gate::At(149_206), Gate::AnySkill, Gate::At(777_534)),
        Kind::EvolvedPlunge => (Gate::At(149_206), Gate::AnySkill, Gate::At(522_255)),
        Kind::Riposte | Kind::EvolvedRiposte => (Gate::At(0), Gate::At(0), Gate::AnySkill),
        Kind::Skill => (Gate::At(288_606), Gate::AnySkill, Gate::At(1_526_221)),
        Kind::EvolvedSkill => (Gate::At(835_975), Gate::AnySkill, Gate::At(1_007_483)),
        Kind::Burst => (
            Gate::At(5_044_123),
            Gate::At(5_044_123),
            Gate::At(5_044_123),
        ),
        Kind::Finisher => (Gate::AnySkill, Gate::AnySkill, Gate::AnySkill),
        Kind::Support => (Gate::AnySkill, Gate::AnySkill, Gate::At(1_335_498)),
        Kind::EvolvedSupport => (Gate::At(1_274_009), Gate::AnySkill, Gate::At(1_271_542)),
        Kind::Dodge => (Gate::At(83_202), Gate::At(333_319), Gate::AnySkill),
    }
}

fn profile(kind: Kind, offsets: Vec<u64>, recovery: u64, switch_lock: Option<u64>) -> Profile {
    let (jump, skill, movement) = gates(kind);
    Profile {
        kind,
        offsets,
        spawn: matches!(kind, Kind::EvolvedNormal(4)).then_some(EVOLVE_SPAWN),
        recovery,
        switch_lock,
        jump,
        skill,
        movement,
    }
}

pub(super) fn select(ctx: &mut CastContext<'_>) -> Result<Option<Profile>, String> {
    let secondary = super::super::secondary::timing(ctx)?;
    let packet = ctx
        .packet_metadata
        .map(|metadata| metadata.damage_effect_id.as_str());
    if let Some((contact, recovery)) = secondary {
        let kind = match packet {
            Some(super::super::secondary::PLUNGE) => Kind::Plunge,
            Some(super::super::secondary::EVOLVE_PLUNGE) => Kind::EvolvedPlunge,
            Some(super::super::secondary::RIPOSTE) => Kind::Riposte,
            Some(super::super::secondary::EVOLVE_RIPOSTE) => Kind::EvolvedRiposte,
            _ => return Err("Blackbird secondary clock requires a native secondary packet".into()),
        };
        return Ok(Some(profile(kind, vec![contact], recovery, None)));
    }
    let (kind, offsets, recovery, switch_lock) =
        if packet == Some(super::super::branches::ROOSTCALL_PACKET) {
            (Kind::Roostcall, ROOSTCALL.0.to_vec(), ROOSTCALL.1, None)
        } else if packet == Some(super::super::branches::BESTOWED_GRACE_PACKET) {
            (
                Kind::BestowedGrace,
                BESTOWED_GRACE.0.to_vec(),
                BESTOWED_GRACE.1,
                None,
            )
        } else if packet == Some(super::super::branches::TIDINGS_PACKET) {
            (Kind::Tidings, TIDINGS.0.to_vec(), TIDINGS.1, None)
        } else if ctx.ability_id == "GA_BlackBird_Evolve_Melee" {
            let stage = (ctx.actor.character.blackbird_mut()?.evolve_melee_stage + 4) % 5;
            (
                normal_kind(true, stage),
                EVOLVE_NORMAL[stage].to_vec(),
                EVOLVE_RECOVERY[stage],
                None,
            )
        } else if ctx.ability_id == "GA_BlackBird_Melee" {
            let stage = (ctx.actor.character.blackbird_mut()?.normal_stage + 4) % 5;
            (
                normal_kind(false, stage),
                NORMAL[stage].to_vec(),
                RECOVERY[stage],
                None,
            )
        } else if ctx.ability_id == "GA_BlackBird_UltraSkill"
            && super::super::form::accepted(ctx.request)
        {
            (Kind::Burst, BURST_OFFSETS.to_vec(), 5_118_154, None)
        } else if ctx.ability_id == "GA_BlackBird_Evolve_Skill" {
            (
                Kind::EvolvedSkill,
                EVOLVE_SKILL_OFFSETS.to_vec(),
                894_130,
                None,
            )
        } else if ctx.ability_id == "GA_BlackBird_Evolve_UltraSkill" {
            (Kind::Finisher, FINISHER_OFFSETS.to_vec(), 4_083_334, None)
        } else if ctx.ability_id == "GA_BlackBird_Skill" {
            (Kind::Skill, SKILL_OFFSETS.to_vec(), 1_332_691, None)
        } else if ctx.ability_id == super::super::support::ABILITY
            && super::super::support::accepted(ctx.request)
        {
            (
                Kind::Support,
                vec![super::super::support::ENTRY.hit_delay_us],
                super::super::support::ENTRY.recovery_us,
                Some(1_344_509),
            )
        } else if ctx.ability_id == super::super::support::EVOLVED_ABILITY
            && super::super::support::evolved_accepted(ctx.request)
        {
            (
                Kind::EvolvedSupport,
                vec![super::super::support::EVOLVED_ENTRY.hit_delay_us],
                super::super::support::EVOLVED_ENTRY.recovery_us,
                Some(1_266_940),
            )
        } else {
            return Ok(None);
        };
    Ok(Some(profile(kind, offsets, recovery, switch_lock)))
}
