use crate::execution::ExecutionLimits;
use crate::simulation::{GameActorConfig, GearProcConfig};

pub(crate) struct ConfigContext<'a> {
    pub actor: &'a GameActorConfig,
    pub actor_index: usize,
    pub limits: &'a ExecutionLimits,
}

pub(crate) struct ProcValidation {
    pub kind: &'static str,
    pub duration_us: u64,
}

fn validate_proc(proc: &GearProcConfig, ctx: &ConfigContext<'_>) -> Result<ProcValidation, String> {
    match proc {
        GearProcConfig::DevilsBloodDamage { .. } => {
            crate::sets::devils_blood::validate_config(proc, ctx)
        }
        GearProcConfig::DiabolosChaosIgnore { .. } => {
            crate::sets::diabolos::validate_config(proc, ctx)
        }
        GearProcConfig::StreetBoxerCrit { .. } => {
            crate::sets::street_boxer::validate_config(proc, ctx)
        }
        GearProcConfig::QuietManorMental { .. } => {
            crate::sets::quiet_manor::validate_config(proc, ctx)
        }
        GearProcConfig::HpSuitMaxHp { .. } => crate::sets::hp_suit::validate_config(proc, ctx),
        GearProcConfig::CrimsonAtk { .. } => crate::sets::crimson::validate_config(proc, ctx),
        GearProcConfig::NorosAtk { .. } => crate::weapons::noros::validate_config(proc, ctx),
        GearProcConfig::BopuDamage { .. } => crate::weapons::bopu::validate_config(proc, ctx),
        GearProcConfig::GoldWool { .. } => crate::weapons::gold_wool::validate_config(proc, ctx),
        GearProcConfig::MotorCandyAtk { .. } => {
            crate::weapons::motor_candy::validate_config(proc, ctx)
        }
        GearProcConfig::ShadowCreedAtk { .. } => {
            crate::sets::shadow_creed::validate_config(proc, ctx)
        }
        GearProcConfig::LostRadianceDefIgnore { .. } => {
            crate::sets::lost_radiance::validate_config(proc, ctx)
        }
        GearProcConfig::KnightCandyCritDamage { .. } => {
            crate::weapons::knight_candy::validate_config(proc, ctx)
        }
        GearProcConfig::OraoraBasicAttackDamage { .. } => {
            crate::weapons::oraora::validate_config(proc, ctx)
        }
        GearProcConfig::BoxingCandyDamage { .. } => {
            crate::weapons::boxing_candy::validate_config(proc, ctx)
        }
        GearProcConfig::ForgottenHpStats { .. } => {
            crate::weapons::forgotten::validate_config(proc, ctx)
        }
        GearProcConfig::KoinoboriStats { .. } => {
            crate::weapons::koinobori::validate_config(proc, ctx)
        }
        GearProcConfig::ApplianceDamage { .. } => {
            crate::weapons::appliance::validate_config(proc, ctx)
        }
        GearProcConfig::ProkaryonDamage { .. } => {
            crate::weapons::prokaryon::validate_config(proc, ctx)
        }
        GearProcConfig::PaperPlaneDamage { .. } => {
            crate::weapons::paper_plane::validate_config(proc, ctx)
        }
        GearProcConfig::WorldrainDamage { .. } => {
            crate::weapons::worldrain::validate_config(proc, ctx)
        }
        GearProcConfig::ButterflyDamage { .. } => {
            crate::weapons::butterfly::validate_config(proc, ctx)
        }
        GearProcConfig::MamenFons { .. } => crate::weapons::mamen::validate_config(proc, ctx),
        GearProcConfig::KiteAtk { .. } => crate::weapons::kite::validate_config(proc, ctx),
        GearProcConfig::WhaleAtk { .. } => crate::weapons::whale::validate_config(proc, ctx),
        GearProcConfig::CrowbarBreak { .. } => crate::weapons::crowbar::validate_config(proc, ctx),
        GearProcConfig::NestBirdGaze { .. } => {
            crate::weapons::nest_bird::validate_config(proc, ctx)
        }
        GearProcConfig::JiaojuanBreak { .. } => {
            crate::weapons::jiaojuan::validate_config(proc, ctx)
        }
        GearProcConfig::SnowmanShieldAtk { .. } => {
            crate::weapons::snowman::validate_config(proc, ctx)
        }
        GearProcConfig::YurenHp { .. } => crate::weapons::yuren::validate_config(proc, ctx),
        GearProcConfig::TuansanlangTeam { .. } => {
            crate::weapons::tuansanlang::validate_config(proc, ctx)
        }
        GearProcConfig::YaodaoParry { .. } => crate::weapons::yaodao::validate_config(proc, ctx),
        GearProcConfig::ThiefCandyDodge { .. } => {
            crate::weapons::thief_candy::validate_config(proc, ctx)
        }
        GearProcConfig::DustbinBreak { .. } => crate::weapons::dustbin::validate_config(proc, ctx),
        GearProcConfig::PoliceRatBoss { .. } => {
            crate::weapons::police_rat::validate_config(proc, ctx)
        }
        GearProcConfig::RoseThorn { .. } => crate::weapons::rose::validate_config(proc, ctx),
        GearProcConfig::TigerTallyAtk { .. } => {
            crate::weapons::tiger_tally::validate_config(proc, ctx)
        }
        GearProcConfig::Nakupeda { .. } => crate::weapons::nakupeda::validate_config(proc, ctx),
        GearProcConfig::BlackBookBreak { .. } => {
            crate::weapons::black_book::validate_config(proc, ctx)
        }
        GearProcConfig::JingmotingyuanCrit { .. } => {
            crate::weapons::jingmotingyuan::validate_config(proc, ctx)
        }
        GearProcConfig::RishiAtk { .. } => crate::weapons::rishi::validate_config(proc, ctx),
        GearProcConfig::Twinbirds { .. } => crate::weapons::twinbirds::validate_config(proc, ctx),
        GearProcConfig::EternalWaltz { .. } => {
            crate::weapons::eternal_waltz::validate_config(proc, ctx)
        }
        GearProcConfig::CastleHealingBonus { .. } => {
            crate::weapons::castle::validate_config(proc, ctx)
        }
        GearProcConfig::WrongGate { .. } => crate::weapons::wrong_gate::validate_config(proc, ctx),
        GearProcConfig::RagingFlames { .. } => {
            crate::weapons::raging_flames::validate_config(proc, ctx)
        }
        GearProcConfig::LunarPhase { .. } => crate::weapons::lunarphase::validate_config(proc, ctx),
        GearProcConfig::DemonBlade { .. } => {
            crate::weapons::demon_blade::validate_config(proc, ctx)
        }
        GearProcConfig::Moon { .. } => crate::weapons::moon::validate_config(proc, ctx),
        GearProcConfig::FarSideMoon { .. } => {
            crate::weapons::far_side_moon::validate_config(proc, ctx)
        }
        GearProcConfig::BlastCandyAtk { .. } => {
            crate::weapons::blast_candy::validate_config(proc, ctx)
        }
        GearProcConfig::ChargeEfficiency { .. } => {
            crate::gear::charge_efficiency::validate_config(proc, ctx)
        }
        GearProcConfig::SpeedyHedgehogTeamAtk { .. } => {
            crate::sets::speedy_hedgehog::validate_config(proc, ctx)
        }
        GearProcConfig::MindRoyaleEnergy { .. } => {
            crate::weapons::mind_royale::validate_config(proc, ctx)
        }
        GearProcConfig::FirefliesCritDamage { .. } => {
            crate::sets::fireflies::validate_config(proc, ctx)
        }
        GearProcConfig::GoodBoyTeamAtk { .. } => {
            crate::weapons::good_boy::validate_config(proc, ctx)
        }
        GearProcConfig::GoldRecord { .. } => {
            crate::weapons::gold_record::validate_config(proc, ctx)
        }
        GearProcConfig::MarchingBeyondTime { .. } => {
            crate::weapons::marching_beyond_time::validate_config(proc, ctx)
        }
        GearProcConfig::BeHappy { .. } => crate::weapons::be_happy::validate_config(proc, ctx),
        GearProcConfig::BitterCake { .. } => crate::weapons::bittercake::validate_config(proc, ctx),
        GearProcConfig::SpiderTeamAtk { .. } => crate::weapons::spider::validate_config(proc, ctx),
        GearProcConfig::BitGame { .. } => crate::weapons::bit_game::validate_config(proc, ctx),
    }
}

pub(crate) fn validate_config(
    actor: &GameActorConfig,
    i: usize,
    relative_duration_us: u64,
    limits: &ExecutionLimits,
) -> Result<(), String> {
    let ctx = ConfigContext {
        actor,
        actor_index: i,
        limits,
    };
    let mut seen = std::collections::BTreeSet::new();
    for proc in &actor.gear_procs {
        let validation = validate_proc(proc, &ctx)?;
        if !seen.insert(validation.kind) {
            return Err(format!(
                "actor {i} duplicates gear proc {}",
                validation.kind
            ));
        }
        relative_duration_us
            .checked_add(validation.duration_us)
            .ok_or("gear proc deadline overflow")?;
    }
    Ok(())
}

pub(crate) struct DeadlineContext<'a> {
    pub action: &'a crate::simulation::metadata::ActionMetadata,
    pub time_us: u64,
    pub hit_delay_us: u64,
    pub action_index: usize,
}

pub(crate) fn validate_action_deadlines(
    procs: &[GearProcConfig],
    action: &crate::simulation::metadata::ActionMetadata,
    t: u64,
    hit_delay_us: u64,
    i: usize,
) -> Result<(), String> {
    let ctx = DeadlineContext {
        action,
        time_us: t,
        hit_delay_us,
        action_index: i,
    };
    for proc in procs {
        match proc {
            GearProcConfig::NorosAtk {
                duration_us,
                cooldown_us,
                ..
            } => crate::weapons::noros::validate_deadline(&ctx, duration_us, cooldown_us)?,
            GearProcConfig::BopuDamage {
                duration_us,
                cooldown_us,
                ..
            } => crate::weapons::bopu::validate_deadline(&ctx, duration_us, cooldown_us)?,
            GearProcConfig::BeHappy { cooldown_us, .. } => {
                crate::weapons::be_happy::validate_deadline(&ctx, cooldown_us)?
            }
            GearProcConfig::BitterCake { duration_us, .. } => {
                crate::weapons::bittercake::validate_deadline(&ctx, duration_us)?
            }
            GearProcConfig::CastleHealingBonus { duration_us, .. } => {
                crate::weapons::castle::validate_deadline(&ctx, duration_us)?
            }
            GearProcConfig::WrongGate { duration_us, .. } => {
                crate::weapons::wrong_gate::validate_deadline(&ctx, duration_us)?
            }
            GearProcConfig::QuietManorMental { duration_us, .. } => {
                crate::sets::quiet_manor::validate_deadline(&ctx, duration_us)?
            }
            GearProcConfig::HpSuitMaxHp { duration_us, .. } => {
                crate::sets::hp_suit::validate_deadline(&ctx, duration_us)?
            }
            GearProcConfig::CrimsonAtk { duration_us, .. } => {
                crate::sets::crimson::validate_deadline(&ctx, duration_us)?
            }
            GearProcConfig::ShadowCreedAtk { duration_us, .. } => {
                crate::sets::shadow_creed::validate_deadline(&ctx, duration_us)?
            }
            GearProcConfig::GoldWool { duration_us, .. } => {
                crate::weapons::gold_wool::validate_deadline(&ctx, duration_us)?
            }
            GearProcConfig::MindRoyaleEnergy { cooldown_us, .. } => {
                crate::weapons::mind_royale::validate_deadline(&ctx, cooldown_us)?
            }
            GearProcConfig::FirefliesCritDamage { duration_us, .. } => {
                crate::sets::fireflies::validate_deadline(&ctx, duration_us)?
            }
            GearProcConfig::KnightCandyCritDamage { duration_us, .. } => {
                crate::weapons::knight_candy::validate_deadline(&ctx, duration_us)?
            }
            GearProcConfig::OraoraBasicAttackDamage { duration_us, .. } => {
                crate::weapons::oraora::validate_deadline(&ctx, duration_us)?
            }
            GearProcConfig::DemonBlade { duration_us, .. } => {
                crate::weapons::demon_blade::validate_deadline(&ctx, duration_us)?
            }
            GearProcConfig::Moon { duration_us, .. } => {
                crate::weapons::moon::validate_deadline(&ctx, duration_us)?
            }
            GearProcConfig::GoldRecord { duration_us, .. } => {
                crate::weapons::gold_record::validate_deadline(&ctx, duration_us)?
            }
            GearProcConfig::MarchingBeyondTime {
                def_ignore_duration_us,
                ..
            } => crate::weapons::marching_beyond_time::validate_deadline(
                &ctx,
                def_ignore_duration_us,
            )?,
            GearProcConfig::SpiderTeamAtk { duration_us, .. } => {
                crate::weapons::spider::validate_deadline(&ctx, duration_us)?
            }
            GearProcConfig::FarSideMoon { duration_us, .. } => {
                crate::weapons::far_side_moon::validate_deadline(&ctx, duration_us)?
            }
            GearProcConfig::GoodBoyTeamAtk { duration_us, .. } => {
                crate::weapons::good_boy::validate_deadline(&ctx, duration_us)?
            }
            GearProcConfig::BlastCandyAtk { duration_us, .. } => {
                crate::weapons::blast_candy::validate_deadline(&ctx, duration_us)?
            }
            GearProcConfig::LostRadianceDefIgnore { duration_us, .. } => {
                crate::sets::lost_radiance::validate_deadline(&ctx, duration_us)?
            }
            GearProcConfig::Twinbirds { duration_us, .. } => {
                crate::weapons::twinbirds::validate_deadline(&ctx, duration_us)?
            }
            GearProcConfig::EternalWaltz { duration_us, .. } => {
                crate::weapons::eternal_waltz::validate_deadline(&ctx, duration_us)?
            }
            GearProcConfig::RagingFlames { duration_us, .. } => {
                crate::weapons::raging_flames::validate_deadline(&ctx, duration_us)?
            }
            GearProcConfig::LunarPhase { duration_us, .. } => {
                crate::weapons::lunarphase::validate_deadline(&ctx, duration_us)?
            }
            GearProcConfig::SpeedyHedgehogTeamAtk { duration_us, .. } => {
                crate::sets::speedy_hedgehog::validate_deadline(&ctx, duration_us)?
            }
            GearProcConfig::ThiefCandyDodge { duration_us, .. } => {
                crate::weapons::thief_candy::validate_deadline(&ctx, duration_us)?
            }
            GearProcConfig::TigerTallyAtk { .. } => {
                crate::weapons::tiger_tally::validate_deadline(&ctx, proc)?
            }
            GearProcConfig::Nakupeda {
                cooldown_us,
                shield_duration_us,
                ..
            } if action.input_role
                == Some(crate::simulation::metadata::InputActionRole::Ultimate) =>
            {
                crate::gear::validation::validate_cast_deadline(&ctx, *cooldown_us)?;
                crate::gear::validation::validate_cast_deadline(&ctx, *shield_duration_us)?;
            }
            GearProcConfig::JingmotingyuanCrit { duration_us, .. } => {
                crate::weapons::jingmotingyuan::validate_deadline(&ctx, duration_us)?
            }
            GearProcConfig::DustbinBreak {
                duration_us,
                cooldown_us,
                ..
            } => crate::weapons::dustbin::validate_deadline(&ctx, duration_us, cooldown_us)?,
            GearProcConfig::RoseThorn {
                duration_us,
                broken_extension_us,
                ..
            } => crate::weapons::rose::validate_deadline(&ctx, duration_us, broken_extension_us)?,
            GearProcConfig::WorldrainDamage { duration_us, .. } => {
                crate::weapons::worldrain::validate_deadline(&ctx, duration_us)?
            }
            GearProcConfig::ApplianceDamage { .. }
            | GearProcConfig::BitGame { .. }
            | GearProcConfig::BoxingCandyDamage { .. }
            | GearProcConfig::ForgottenHpStats { .. }
            | GearProcConfig::KoinoboriStats { .. }
            | GearProcConfig::MotorCandyAtk { .. }
            | GearProcConfig::PaperPlaneDamage { .. }
            | GearProcConfig::ProkaryonDamage { .. }
            | GearProcConfig::ButterflyDamage { .. }
            | GearProcConfig::MamenFons { .. }
            | GearProcConfig::KiteAtk { .. }
            | GearProcConfig::WhaleAtk { .. }
            | GearProcConfig::CrowbarBreak { .. }
            | GearProcConfig::NestBirdGaze { .. }
            | GearProcConfig::JiaojuanBreak { .. }
            | GearProcConfig::SnowmanShieldAtk { .. }
            | GearProcConfig::YurenHp { .. }
            | GearProcConfig::TuansanlangTeam { .. }
            | GearProcConfig::YaodaoParry { .. }
            | GearProcConfig::PoliceRatBoss { .. }
            | GearProcConfig::RishiAtk { .. }
            | GearProcConfig::Nakupeda { .. }
            | GearProcConfig::BlackBookBreak { .. }
            | GearProcConfig::ChargeEfficiency { .. }
            | GearProcConfig::DevilsBloodDamage { .. }
            | GearProcConfig::DiabolosChaosIgnore { .. }
            | GearProcConfig::StreetBoxerCrit { .. } => {}
        }
    }
    Ok(())
}

pub(crate) fn validate_cast_deadline(
    ctx: &DeadlineContext<'_>,
    duration_us: u64,
) -> Result<(), String> {
    ctx.time_us
        .checked_add(duration_us)
        .ok_or_else(|| format!("action {} gear proc deadline overflow", ctx.action_index))?;
    Ok(())
}

pub(crate) fn validate_delivery_deadline(
    ctx: &DeadlineContext<'_>,
    duration_us: u64,
) -> Result<(), String> {
    ctx.time_us
        .checked_add(ctx.hit_delay_us)
        .and_then(|hit| hit.checked_add(duration_us))
        .ok_or_else(|| format!("action {} gear proc deadline overflow", ctx.action_index))?;
    Ok(())
}
