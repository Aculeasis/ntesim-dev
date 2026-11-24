use super::*;
use crate::simulation::GameActorConfig;

type PlainProjector = fn(
    &BTreeMap<String, Value>,
    u32,
    &str,
    &mut GameActorConfig,
    &[usize],
) -> Result<Value, String>;
type TimedProjector = fn(
    &BTreeMap<String, Value>,
    u32,
    &str,
    u64,
    &mut GameActorConfig,
    &[usize],
) -> Result<Value, String>;

enum WeaponProjector {
    Plain(PlainProjector),
    Timed(TimedProjector),
}

pub(super) struct WeaponAdapter {
    pub(super) weapon_id: &'static str,
    pub(super) preview_character_id: &'static str,
    label: &'static str,
    case_sensitive: bool,
    matches_proc: fn(&GearProcConfig) -> bool,
    projector: WeaponProjector,
}

pub(super) const WEAPON_ADAPTERS: &[WeaponAdapter] = &[
    WeaponAdapter {
        weapon_id: crate::weapons::echoes_on_hold::WEAPON_ID,
        preview_character_id: "character:1020",
        label: "Echoes on Hold",
        case_sensitive: false,
        matches_proc: |proc| matches!(proc, GearProcConfig::EchoesOnHold { .. }),
        projector: WeaponProjector::Plain(crate::weapons::echoes_on_hold::projection::project),
    },
    WeaponAdapter {
        weapon_id: "weapon:fork_twinbirds",
        preview_character_id: "character:1019",
        label: "Twinflight",
        case_sensitive: false,
        matches_proc: |proc| matches!(proc, GearProcConfig::Twinbirds { .. }),
        projector: WeaponProjector::Plain(crate::weapons::twinbirds::projection::project),
    },
    WeaponAdapter {
        weapon_id: "weapon:fork_DemonBlade",
        preview_character_id: "character:1019",
        label: "Ravenous Blade",
        case_sensitive: false,
        matches_proc: |proc| matches!(proc, GearProcConfig::DemonBlade { .. }),
        projector: WeaponProjector::Plain(crate::weapons::demon_blade::projection::project),
    },
    WeaponAdapter {
        weapon_id: "weapon:fork_moon",
        preview_character_id: "character:1019",
        label: "Moon",
        case_sensitive: false,
        matches_proc: |proc| matches!(proc, GearProcConfig::Moon { .. }),
        projector: WeaponProjector::Plain(crate::weapons::moon::projection::project),
    },
    WeaponAdapter {
        weapon_id: "weapon:fork_lunarphase",
        preview_character_id: "character:1019",
        label: "Blushing Mirage",
        case_sensitive: false,
        matches_proc: |proc| matches!(proc, GearProcConfig::LunarPhase { .. }),
        projector: WeaponProjector::Plain(crate::weapons::lunarphase::projection::project),
    },
    WeaponAdapter {
        weapon_id: "weapon:fork_lingganzhongjiezhe",
        preview_character_id: "character:1019",
        label: "Mind Royale",
        case_sensitive: true,
        matches_proc: |proc| matches!(proc, GearProcConfig::MindRoyaleEnergy { .. }),
        projector: WeaponProjector::Timed(crate::weapons::mind_royale::projection::project),
    },
    WeaponAdapter {
        weapon_id: "weapon:fork_mofeikesi",
        preview_character_id: "character:1019",
        label: "Good Boy",
        case_sensitive: true,
        matches_proc: |proc| matches!(proc, GearProcConfig::GoodBoyTeamAtk { .. }),
        projector: WeaponProjector::Timed(crate::weapons::good_boy::projection::project),
    },
    WeaponAdapter {
        weapon_id: "weapon:fork_BlastCandy",
        preview_character_id: "character:1019",
        label: "Blast Candy",
        case_sensitive: false,
        matches_proc: |proc| matches!(proc, GearProcConfig::BlastCandyAtk { .. }),
        projector: WeaponProjector::Timed(crate::weapons::blast_candy::projection::project),
    },
    WeaponAdapter {
        weapon_id: "weapon:fork_BitterCake",
        preview_character_id: "character:1033",
        label: "Bittercake",
        case_sensitive: false,
        matches_proc: |proc| matches!(proc, GearProcConfig::BitterCake { .. }),
        projector: WeaponProjector::Timed(crate::weapons::bittercake::projection::project),
    },
    WeaponAdapter {
        weapon_id: "weapon:fork_Arachne",
        preview_character_id: "character:1019",
        label: "Eternal Waltz",
        case_sensitive: false,
        matches_proc: |proc| matches!(proc, GearProcConfig::EternalWaltz { .. }),
        projector: WeaponProjector::Plain(crate::weapons::eternal_waltz::projection::project),
    },
    WeaponAdapter {
        weapon_id: "weapon:fork_wushoutieyu",
        preview_character_id: "character:1010",
        label: "Raging Flames",
        case_sensitive: false,
        matches_proc: |proc| matches!(proc, GearProcConfig::RagingFlames { .. }),
        projector: WeaponProjector::Plain(crate::weapons::raging_flames::projection::project),
    },
    WeaponAdapter {
        weapon_id: "weapon:fork_prism",
        preview_character_id: "character:1054",
        label: "Far Side of the Moon",
        case_sensitive: false,
        matches_proc: |proc| matches!(proc, GearProcConfig::FarSideMoon { .. }),
        projector: WeaponProjector::Plain(crate::weapons::far_side_moon::projection::project),
    },
    WeaponAdapter {
        weapon_id: "weapon:fork_oulaquantao",
        preview_character_id: "character:1010",
        label: "Oraora!",
        case_sensitive: false,
        matches_proc: |proc| matches!(proc, GearProcConfig::OraoraBasicAttackDamage { .. }),
        projector: WeaponProjector::Plain(crate::weapons::oraora::projection::project),
    },
    WeaponAdapter {
        weapon_id: "weapon:fork_wuhuakuang",
        preview_character_id: "character:1046",
        label: "The Forgotten",
        case_sensitive: false,
        matches_proc: |proc| matches!(proc, GearProcConfig::ForgottenHpStats { .. }),
        projector: WeaponProjector::Plain(crate::weapons::forgotten::projection::project),
    },
    WeaponAdapter {
        weapon_id: "weapon:fork_Appliance",
        preview_character_id: "character:1019",
        label: "Appliance",
        case_sensitive: false,
        matches_proc: |proc| matches!(proc, GearProcConfig::ApplianceDamage { .. }),
        projector: WeaponProjector::Plain(crate::weapons::appliance::projection::project),
    },
    WeaponAdapter {
        weapon_id: "weapon:fork_Prokaryon",
        preview_character_id: "character:1019",
        label: "Prokaryon",
        case_sensitive: false,
        matches_proc: |proc| matches!(proc, GearProcConfig::ProkaryonDamage { .. }),
        projector: WeaponProjector::Plain(crate::weapons::prokaryon::projection::project),
    },
    WeaponAdapter {
        weapon_id: "weapon:fork_PaperPlane",
        preview_character_id: "character:1019",
        label: "Paper Plane",
        case_sensitive: false,
        matches_proc: |proc| matches!(proc, GearProcConfig::PaperPlaneDamage { .. }),
        projector: WeaponProjector::Plain(crate::weapons::paper_plane::projection::project),
    },
    WeaponAdapter {
        weapon_id: "weapon:fork_Worldrain",
        preview_character_id: "character:1052",
        label: "Worldrain",
        case_sensitive: false,
        matches_proc: |proc| matches!(proc, GearProcConfig::WorldrainDamage { .. }),
        projector: WeaponProjector::Plain(crate::weapons::worldrain::projection::project),
    },
    WeaponAdapter {
        weapon_id: "weapon:fork_BoxingCandy",
        preview_character_id: "character:1019",
        label: "Boxing Candy",
        case_sensitive: false,
        matches_proc: |proc| matches!(proc, GearProcConfig::BoxingCandyDamage { .. }),
        projector: WeaponProjector::Plain(crate::weapons::boxing_candy::projection::project),
    },
    WeaponAdapter {
        weapon_id: "weapon:fork_nonos",
        preview_character_id: "character:1019",
        label: "Noros",
        case_sensitive: false,
        matches_proc: |proc| matches!(proc, GearProcConfig::NorosAtk { .. }),
        projector: WeaponProjector::Plain(crate::weapons::noros::projection::project),
    },
    WeaponAdapter {
        weapon_id: "weapon:fork_GoldWool",
        preview_character_id: "character:1019",
        label: "Gold Wool",
        case_sensitive: false,
        matches_proc: |proc| matches!(proc, GearProcConfig::GoldWool { .. }),
        projector: WeaponProjector::Plain(crate::weapons::gold_wool::projection::project),
    },
    WeaponAdapter {
        weapon_id: "weapon:fork_MotorCandy",
        preview_character_id: "character:1019",
        label: "Motor Candy",
        case_sensitive: false,
        matches_proc: |proc| matches!(proc, GearProcConfig::MotorCandyAtk { .. }),
        projector: WeaponProjector::Plain(crate::weapons::motor_candy::projection::project),
    },
    WeaponAdapter {
        weapon_id: "weapon:fork_KnightCandy",
        preview_character_id: "character:1019",
        label: "Knight Candy",
        case_sensitive: false,
        matches_proc: |proc| matches!(proc, GearProcConfig::KnightCandyCritDamage { .. }),
        projector: WeaponProjector::Plain(crate::weapons::knight_candy::projection::project),
    },
    WeaponAdapter {
        weapon_id: "weapon:fork_bopu",
        preview_character_id: "character:1003",
        label: "Cosmos Daze, Wild Reverie",
        case_sensitive: false,
        matches_proc: |proc| matches!(proc, GearProcConfig::BopuDamage { .. }),
        projector: WeaponProjector::Plain(crate::weapons::bopu::projection::project),
    },
    WeaponAdapter {
        weapon_id: "weapon:fork_koinobori",
        preview_character_id: "character:1033",
        label: "A Time Will Come",
        case_sensitive: false,
        matches_proc: |proc| matches!(proc, GearProcConfig::KoinoboriStats { .. }),
        projector: WeaponProjector::Plain(crate::weapons::koinobori::projection::project),
    },
    WeaponAdapter {
        weapon_id: "weapon:fork_GoldRecord",
        preview_character_id: "character:1072",
        label: "Voice of the Voyager",
        case_sensitive: false,
        matches_proc: |proc| matches!(proc, GearProcConfig::GoldRecord { .. }),
        projector: WeaponProjector::Plain(crate::weapons::gold_record::projection::project),
    },
    WeaponAdapter {
        weapon_id: "weapon:fork_Time",
        preview_character_id: "character:1052",
        label: "Marching Beyond Time",
        case_sensitive: false,
        matches_proc: |proc| matches!(proc, GearProcConfig::MarchingBeyondTime { .. }),
        projector: WeaponProjector::Plain(
            crate::weapons::marching_beyond_time::projection::project,
        ),
    },
    WeaponAdapter {
        weapon_id: "weapon:fork_vine",
        preview_character_id: "character:1008",
        label: "Be Happy",
        case_sensitive: false,
        matches_proc: |proc| matches!(proc, GearProcConfig::BeHappy { .. }),
        projector: WeaponProjector::Plain(crate::weapons::be_happy::projection::project),
    },
    WeaponAdapter {
        weapon_id: "weapon:fork_Castle",
        preview_character_id: "character:1019",
        label: "Call of the Twisted City",
        case_sensitive: false,
        matches_proc: |proc| matches!(proc, GearProcConfig::CastleHealingBonus { .. }),
        projector: WeaponProjector::Plain(crate::weapons::castle::projection::project),
    },
    WeaponAdapter {
        weapon_id: "weapon:fork_Door",
        preview_character_id: "character:1019",
        label: "The Wrong Gate",
        case_sensitive: false,
        matches_proc: |proc| matches!(proc, GearProcConfig::WrongGate { .. }),
        projector: WeaponProjector::Plain(crate::weapons::wrong_gate::projection::project),
    },
    WeaponAdapter {
        weapon_id: "weapon:fork_spider",
        preview_character_id: "character:1003",
        label: "Failing You, Heavy in My Heart",
        case_sensitive: false,
        matches_proc: |proc| matches!(proc, GearProcConfig::SpiderTeamAtk { .. }),
        projector: WeaponProjector::Plain(crate::weapons::spider::projection::project),
    },
    WeaponAdapter {
        weapon_id: "weapon:fork_BitGame",
        preview_character_id: "character:1055",
        label: "Blow up the Crowd",
        case_sensitive: false,
        matches_proc: |proc| matches!(proc, GearProcConfig::BitGame { .. }),
        projector: WeaponProjector::Plain(crate::weapons::bit_game::projection::project),
    },
    WeaponAdapter {
        weapon_id: "weapon:fork_Butterfly",
        preview_character_id: "character:1046",
        label: "Reality Refuge",
        case_sensitive: false,
        matches_proc: |proc| matches!(proc, GearProcConfig::ButterflyDamage { .. }),
        projector: WeaponProjector::Plain(crate::weapons::butterfly::projection::project),
    },
    WeaponAdapter {
        weapon_id: "weapon:fork_mamen",
        preview_character_id: "character:1003",
        label: "Contemplative Cat",
        case_sensitive: false,
        matches_proc: |proc| matches!(proc, GearProcConfig::MamenFons { .. }),
        projector: WeaponProjector::Plain(crate::weapons::mamen::projection::project),
    },
    WeaponAdapter {
        weapon_id: "weapon:fork_Kite",
        preview_character_id: "character:1003",
        label: "Watch Your Heads!",
        case_sensitive: false,
        matches_proc: |proc| matches!(proc, GearProcConfig::KiteAtk { .. }),
        projector: WeaponProjector::Plain(crate::weapons::kite::projection::project),
    },
    WeaponAdapter {
        weapon_id: "weapon:fork_Whale",
        preview_character_id: "character:1072",
        label: "Song of the Whale",
        case_sensitive: false,
        matches_proc: |proc| matches!(proc, GearProcConfig::WhaleAtk { .. }),
        projector: WeaponProjector::Plain(crate::weapons::whale::projection::project),
    },
    WeaponAdapter {
        weapon_id: "weapon:fork_Crowbar",
        preview_character_id: "character:1052",
        label: "Time Bandit",
        case_sensitive: false,
        matches_proc: |proc| matches!(proc, GearProcConfig::CrowbarBreak { .. }),
        projector: WeaponProjector::Plain(crate::weapons::crowbar::projection::project),
    },
    WeaponAdapter {
        weapon_id: "weapon:fork_NestBird",
        preview_character_id: "character:1036",
        label: "Tears Beneath the Mask",
        case_sensitive: false,
        matches_proc: |proc| matches!(proc, GearProcConfig::NestBirdGaze { .. }),
        projector: WeaponProjector::Plain(crate::weapons::nest_bird::projection::project),
    },
    WeaponAdapter {
        weapon_id: "weapon:fork_jiaojuan",
        preview_character_id: "character:1021",
        label: "Film - MANISH",
        case_sensitive: false,
        matches_proc: |proc| matches!(proc, GearProcConfig::JiaojuanBreak { .. }),
        projector: WeaponProjector::Plain(crate::weapons::jiaojuan::projection::project),
    },
    WeaponAdapter {
        weapon_id: "weapon:fork_snowman",
        preview_character_id: "character:1023",
        label: "Solifugae Vita",
        case_sensitive: false,
        matches_proc: |proc| matches!(proc, GearProcConfig::SnowmanShieldAtk { .. }),
        projector: WeaponProjector::Plain(crate::weapons::snowman::projection::project),
    },
    WeaponAdapter {
        weapon_id: "weapon:fork_yuren",
        preview_character_id: "character:1033",
        label: "\"Rainman\"",
        case_sensitive: false,
        matches_proc: |proc| matches!(proc, GearProcConfig::YurenHp { .. }),
        projector: WeaponProjector::Plain(crate::weapons::yuren::projection::project),
    },
    WeaponAdapter {
        weapon_id: "weapon:fork_tuansanlang",
        preview_character_id: "character:1039",
        label: "Danzaburou",
        case_sensitive: false,
        matches_proc: |proc| matches!(proc, GearProcConfig::TuansanlangTeam { .. }),
        projector: WeaponProjector::Plain(crate::weapons::tuansanlang::projection::project),
    },
    WeaponAdapter {
        weapon_id: "weapon:fork_yaodao",
        preview_character_id: "character:1010",
        label: "Crimson Hexblade",
        case_sensitive: false,
        matches_proc: |proc| matches!(proc, GearProcConfig::YaodaoParry { .. }),
        projector: WeaponProjector::Plain(crate::weapons::yaodao::projection::project),
    },
    WeaponAdapter {
        weapon_id: "weapon:fork_ThiefCandy",
        preview_character_id: "character:1003",
        label: "Thief's Candy",
        case_sensitive: false,
        matches_proc: |proc| matches!(proc, GearProcConfig::ThiefCandyDodge { .. }),
        projector: WeaponProjector::Plain(crate::weapons::thief_candy::projection::project),
    },
    WeaponAdapter {
        weapon_id: "weapon:fork_dustbin",
        preview_character_id: "character:1023",
        label: "\"Decomposer\"",
        case_sensitive: false,
        matches_proc: |proc| matches!(proc, GearProcConfig::DustbinBreak { .. }),
        projector: WeaponProjector::Plain(crate::weapons::dustbin::projection::project),
    },
    WeaponAdapter {
        weapon_id: "weapon:fork_PoliceRat",
        preview_character_id: "character:1020",
        label: "Justice Executioner",
        case_sensitive: false,
        matches_proc: |proc| matches!(proc, GearProcConfig::PoliceRatBoss { .. }),
        projector: WeaponProjector::Plain(crate::weapons::police_rat::projection::project),
    },
    WeaponAdapter {
        weapon_id: "weapon:fork_Rose",
        preview_character_id: "character:1004",
        label: "Bohemian Rose",
        case_sensitive: false,
        matches_proc: |proc| matches!(proc, GearProcConfig::RoseThorn { .. }),
        projector: WeaponProjector::Plain(crate::weapons::rose::projection::project),
    },
    WeaponAdapter {
        weapon_id: "weapon:fork_TigerTally",
        preview_character_id: "character:1025",
        label: "Commander Tiger Talisman",
        case_sensitive: false,
        matches_proc: |proc| matches!(proc, GearProcConfig::TigerTallyAtk { .. }),
        projector: WeaponProjector::Plain(crate::weapons::tiger_tally::projection::project),
    },
    WeaponAdapter {
        weapon_id: "weapon:fork_Nakupeda",
        preview_character_id: "character:1020",
        label: "Nacupeda",
        case_sensitive: false,
        matches_proc: |proc| matches!(proc, GearProcConfig::Nakupeda { .. }),
        projector: WeaponProjector::Plain(crate::weapons::nakupeda::projection::project),
    },
    WeaponAdapter {
        weapon_id: "weapon:fork_BlackBook",
        preview_character_id: "character:1004",
        label: "\"Black Tome\"",
        case_sensitive: false,
        matches_proc: |proc| matches!(proc, GearProcConfig::BlackBookBreak { .. }),
        projector: WeaponProjector::Plain(crate::weapons::black_book::projection::project),
    },
    WeaponAdapter {
        weapon_id: "weapon:fork_jingmotingyuan",
        preview_character_id: "character:1023",
        label: "Silent Garden",
        case_sensitive: false,
        matches_proc: |proc| matches!(proc, GearProcConfig::JingmotingyuanCrit { .. }),
        projector: WeaponProjector::Plain(crate::weapons::jingmotingyuan::projection::project),
    },
    WeaponAdapter {
        weapon_id: "weapon:fork_rishi",
        preview_character_id: "character:1046",
        label: "Eclipse",
        case_sensitive: false,
        matches_proc: |proc| matches!(proc, GearProcConfig::RishiAtk { .. }),
        projector: WeaponProjector::Plain(crate::weapons::rishi::projection::project),
    },
];

impl WeaponAdapter {
    pub(super) fn project(
        &self,
        tables: &BTreeMap<String, Value>,
        schema_version: u32,
        content_id: &str,
        relative_duration_us: u64,
        actor: &mut GameActorConfig,
    ) -> Result<Value, String> {
        let positions = proc_positions(actor, self.matches_proc, self.label)?;
        match self.projector {
            WeaponProjector::Plain(project) => {
                project(tables, schema_version, content_id, actor, &positions)
            }
            WeaponProjector::Timed(project) => project(
                tables,
                schema_version,
                content_id,
                relative_duration_us,
                actor,
                &positions,
            ),
        }
    }
}

fn proc_positions(
    actor: &GameActorConfig,
    matches: fn(&GearProcConfig) -> bool,
    label: &str,
) -> Result<Vec<usize>, String> {
    let positions = actor
        .gear_procs
        .iter()
        .enumerate()
        .filter_map(|(index, proc)| matches(proc).then_some(index))
        .collect::<Vec<_>>();
    if positions.len() > 1 {
        return Err(format!("duplicate {label} proc in native gear projection"));
    }
    Ok(positions)
}

pub(crate) fn project_gear(
    tables: &BTreeMap<String, Value>,
    scenario: &mut GameScenarioRequest,
    projection: &GearProjection,
) -> Result<Vec<Value>, String> {
    if projection.hypothesis.trim().is_empty() {
        return Err("native gear projection requires an explicit hypothesis".into());
    }
    let schema_version = scenario.schema_version;
    let content_id = scenario.content_id.clone();
    let relative_duration_us = scenario.absolute_limit_us;
    let mut receipts = Vec::new();
    for actor in &mut scenario.actors {
        let is_mind_royale = actor.weapon_id == "weapon:fork_lingganzhongjiezhe";

        match projection.policy {
            GearProjectionPolicy::NativeMindRoyaleCurveV1
            | GearProjectionPolicy::NativeMindRoyaleFirefliesSoloV1 => {
                let positions = proc_positions(
                    actor,
                    |proc| matches!(proc, GearProcConfig::MindRoyaleEnergy { .. }),
                    "Mind Royale",
                )?;
                if !is_mind_royale {
                    if !positions.is_empty() {
                        return Err("Mind Royale proc disagrees with equipped native weapon".into());
                    }
                    continue;
                }
                if actor.character_id != "character:1019" {
                    return Err(
                        "native Mind Royale projection currently supports the Mint trigger policy only"
                            .into(),
                    );
                }
                let receipt = crate::weapons::mind_royale::projection::project(
                    tables,
                    schema_version,
                    &content_id,
                    relative_duration_us,
                    actor,
                    &positions,
                )?;
                receipts.push(receipt);
            }
            GearProjectionPolicy::NativeWeaponCurveV1 => {
                let adapter = WEAPON_ADAPTERS
                    .iter()
                    .find(|adapter| {
                        if adapter.case_sensitive {
                            actor.weapon_id == adapter.weapon_id
                        } else {
                            actor.weapon_id.eq_ignore_ascii_case(adapter.weapon_id)
                        }
                    })
                    .ok_or_else(|| {
                        format!(
                            "unsupported native weapon passive projection for {}",
                            actor.weapon_id
                        )
                    })?;
                let receipt = adapter.project(
                    tables,
                    schema_version,
                    &content_id,
                    relative_duration_us,
                    actor,
                )?;
                receipts.push(receipt);
            }
        }
    }
    if receipts.is_empty() {
        return Err("native gear projection requires at least one supported weapon wearer".into());
    }
    Ok(receipts)
}
