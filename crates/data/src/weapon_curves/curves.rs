use super::{Curve, key};
use crate::weapon_refinements::ECHOES_ON_HOLD_PARAMETERS;

pub(super) const CURVES: &[Curve] = &[
    Curve {
        native_id: ECHOES_ON_HOLD_PARAMETERS[0].0,
        keys: &[
            key(1, 1500),
            key(2, 1875),
            key(3, 2250),
            key(4, 2625),
            key(5, 3000),
        ],
    },
    Curve {
        native_id: ECHOES_ON_HOLD_PARAMETERS[1].0,
        keys: &[
            key(1, 1200),
            key(2, 1400),
            key(3, 1600),
            key(4, 1800),
            key(5, 2000),
        ],
    },
    Curve {
        native_id: ECHOES_ON_HOLD_PARAMETERS[2].0,
        keys: &[
            key(1, 1000),
            key(2, 1250),
            key(3, 1500),
            key(4, 1750),
            key(5, 2000),
        ],
    },
    Curve {
        native_id: ECHOES_ON_HOLD_PARAMETERS[3].0,
        keys: &[
            key(1, 800),
            key(2, 1100),
            key(3, 1400),
            key(4, 1700),
            key(5, 2000),
        ],
    },
    Curve {
        native_id: ECHOES_ON_HOLD_PARAMETERS[4].0,
        keys: &[key(1, 200000), key(5, 200000)],
    },
    Curve {
        native_id: "Buff_TwinBirds_MagUp",
        keys: &[
            key(1, 720000),
            key(2, 900000),
            key(3, 1080000),
            key(4, 1260000),
            key(5, 1440000),
        ],
    },
    Curve {
        native_id: "Buff_TwinBirds_PsycheUp",
        keys: &[
            key(1, 2000),
            key(2, 2500),
            key(3, 3000),
            key(4, 3500),
            key(5, 4000),
        ],
    },
    Curve {
        native_id: "GE_Fork_serenity_Skill1_Damage1",
        keys: &[
            key(1, 2400),
            key(2, 2800),
            key(3, 3200),
            key(4, 3600),
            key(5, 4000),
        ],
    },
    Curve {
        native_id: "GE_Fork_serenity_Skill2_Damage",
        keys: &[
            key(1, 1800),
            key(2, 2100),
            key(3, 2400),
            key(4, 2700),
            key(5, 3000),
        ],
    },
    Curve {
        native_id: "GE_Fork_yaodao_Skill_Damage",
        keys: &[
            key(1, 20000),
            key(2, 23000),
            key(3, 26000),
            key(4, 29000),
            key(5, 32000),
        ],
    },
    Curve {
        native_id: "buff_Arachne_CD",
        keys: &[key(1, 100000), key(5, 100000)],
    },
    Curve {
        native_id: "buff_Arachne_Hp",
        keys: &[
            key(1, 2000),
            key(2, 2300),
            key(3, 2600),
            key(4, 2900),
            key(5, 3200),
        ],
    },
    Curve {
        native_id: "buff_Arachne_Up",
        keys: &[
            key(1, 1000),
            key(2, 1200),
            key(3, 1400),
            key(4, 1600),
            key(5, 1800),
        ],
    },
    Curve {
        native_id: "buff_BitGame2_AtkUp1",
        keys: &[
            key(1, 1000),
            key(2, 1100),
            key(3, 1200),
            key(4, 1300),
            key(5, 1400),
        ],
    },
    Curve {
        native_id: "buff_BitGame2_AtkUp2",
        keys: &[
            key(1, 200),
            key(2, 230),
            key(3, 260),
            key(4, 290),
            key(5, 320),
        ],
    },
    Curve {
        native_id: "buff_BitGame2_CD",
        keys: &[key(1, 20000), key(5, 20000)],
    },
    Curve {
        native_id: "buff_BitGame2_DamageUpPsycheBase1",
        keys: &[
            key(1, 1200),
            key(2, 1400),
            key(3, 1600),
            key(4, 1800),
            key(5, 2000),
        ],
    },
    Curve {
        native_id: "buff_BitGame2_DamageUpPsycheBase2",
        keys: &[
            key(1, 200),
            key(2, 230),
            key(3, 260),
            key(4, 290),
            key(5, 320),
        ],
    },
    Curve {
        native_id: "buff_BitterCake_CD",
        keys: &[key(1, 100000), key(5, 100000)],
    },
    Curve {
        native_id: "buff_BitterCake_CD2",
        keys: &[key(1, 200000), key(5, 200000)],
    },
    Curve {
        native_id: "buff_BitterCake_DefUp",
        keys: &[
            key(1, 2600),
            key(2, 3100),
            key(3, 3600),
            key(4, 4100),
            key(5, 4600),
        ],
    },
    Curve {
        native_id: "buff_BlackBook2_CD",
        keys: &[key(1, 200000), key(5, 200000)],
    },
    Curve {
        native_id: "buff_BlackBook2_CD2",
        keys: &[key(1, 50000), key(5, 50000)],
    },
    Curve {
        native_id: "buff_BlackBook2_DamageUpChaosBase",
        keys: &[
            key(1, 2000),
            key(2, 2400),
            key(3, 2800),
            key(4, 3200),
            key(5, 3600),
        ],
    },
    Curve {
        native_id: "buff_BlackBook2_SkillDamage",
        keys: &[
            key(1, 20000),
            key(2, 23000),
            key(3, 26000),
            key(4, 29000),
            key(5, 32000),
        ],
    },
    Curve {
        native_id: "buff_BlackBook2_Unbal",
        keys: &[
            key(1, 600000),
            key(2, 670000),
            key(3, 740000),
            key(4, 810000),
            key(5, 880000),
        ],
    },
    Curve {
        native_id: "buff_BlastCandy_AtkUp",
        keys: &[
            key(1, 2500),
            key(2, 3000),
            key(3, 3500),
            key(4, 4000),
            key(5, 4500),
        ],
    },
    Curve {
        native_id: "buff_BlastCandy_CD",
        keys: &[key(1, 100000), key(5, 100000)],
    },
    Curve {
        native_id: "buff_BoxingCandy_Up",
        keys: &[
            key(1, 2200),
            key(2, 2600),
            key(3, 3000),
            key(4, 3400),
            key(5, 3800),
        ],
    },
    Curve {
        native_id: "buff_BoxingCandy_Up2",
        keys: &[
            key(1, 2800),
            key(2, 3400),
            key(3, 4000),
            key(4, 4600),
            key(5, 5200),
        ],
    },
    Curve {
        native_id: "buff_Butterfly_CD",
        keys: &[key(1, 60000), key(5, 60000)],
    },
    Curve {
        native_id: "buff_Butterfly_DamageUpNatureBase",
        keys: &[
            key(1, 1500),
            key(2, 1750),
            key(3, 2000),
            key(4, 2250),
            key(5, 2500),
        ],
    },
    Curve {
        native_id: "buff_Butterfly_attachedup",
        keys: &[
            key(1, 1000),
            key(2, 1125),
            key(3, 1250),
            key(4, 1375),
            key(5, 1500),
        ],
    },
    Curve {
        native_id: "buff_Butterfly_attachedup2",
        keys: &[
            key(1, 2000),
            key(2, 2250),
            key(3, 2500),
            key(4, 2750),
            key(5, 3000),
        ],
    },
    Curve {
        native_id: "buff_Castle_CD",
        keys: &[key(1, 100000), key(5, 100000)],
    },
    Curve {
        native_id: "buff_Castle_HealUp",
        keys: &[
            key(1, 1200),
            key(2, 1400),
            key(3, 1600),
            key(4, 1800),
            key(5, 2000),
        ],
    },
    Curve {
        native_id: "buff_Crowbar_CD",
        keys: &[key(1, 500000), key(5, 500000)],
    },
    Curve {
        native_id: "buff_Crowbar_Unbal",
        keys: &[
            key(1, 900000),
            key(2, 1080000),
            key(3, 1260000),
            key(4, 1440000),
            key(5, 1620000),
        ],
    },
    Curve {
        native_id: "buff_DemonBlade_CD",
        keys: &[key(1, 150000), key(5, 150000)],
    },
    Curve {
        native_id: "buff_DemonBlade_Crit",
        keys: &[
            key(1, 1600),
            key(2, 2000),
            key(3, 2400),
            key(4, 2800),
            key(5, 3200),
        ],
    },
    Curve {
        native_id: "buff_DemonBlade_CritDamageUp",
        keys: &[
            key(1, 900),
            key(2, 1125),
            key(3, 1350),
            key(4, 1575),
            key(5, 1800),
        ],
    },
    Curve {
        native_id: "buff_Door_Atk",
        keys: &[
            key(1, 1600),
            key(2, 2000),
            key(3, 2400),
            key(4, 2800),
            key(5, 3200),
        ],
    },
    Curve {
        native_id: "buff_Door_CD",
        keys: &[key(1, 200000), key(5, 200000)],
    },
    Curve {
        native_id: "buff_Door_OtherUp",
        keys: &[
            key(1, 1500),
            key(2, 1875),
            key(3, 2250),
            key(4, 2625),
            key(5, 3000),
        ],
    },
    Curve {
        native_id: "buff_Door_Up",
        keys: &[
            key(1, 3000),
            key(2, 3750),
            key(3, 4500),
            key(4, 5250),
            key(5, 6000),
        ],
    },
    Curve {
        native_id: "buff_GoldRecord_AtkUp",
        keys: &[
            key(1, 2400),
            key(2, 3000),
            key(3, 3600),
            key(4, 4200),
            key(5, 4800),
        ],
    },
    Curve {
        native_id: "buff_GoldRecord_CD",
        keys: &[key(1, 30000), key(5, 30000)],
    },
    Curve {
        native_id: "buff_GoldRecord_QCritDmaUp",
        keys: &[
            key(1, 2200),
            key(2, 2750),
            key(3, 3300),
            key(4, 3850),
            key(5, 4400),
        ],
    },
    Curve {
        native_id: "buff_GoldRecord_QteCritDmaUp",
        keys: &[
            key(1, 6600),
            key(2, 8250),
            key(3, 9900),
            key(4, 11550),
            key(5, 13200),
        ],
    },
    Curve {
        native_id: "buff_GoldWool_CD",
        keys: &[key(1, 200000), key(5, 200000)],
    },
    Curve {
        native_id: "buff_GoldWool_CritDamageUp",
        keys: &[
            key(1, 4000),
            key(2, 5000),
            key(3, 6000),
            key(4, 7000),
            key(5, 8000),
        ],
    },
    Curve {
        native_id: "buff_GoldWool_Up",
        keys: &[
            key(1, 2000),
            key(2, 2500),
            key(3, 3000),
            key(4, 3500),
            key(5, 4000),
        ],
    },
    Curve {
        native_id: "buff_Kite_AtkUp",
        keys: &[
            key(1, 1000),
            key(2, 1200),
            key(3, 1400),
            key(4, 1600),
            key(5, 1800),
        ],
    },
    Curve {
        native_id: "buff_Kite_CD",
        keys: &[key(1, 150000), key(5, 150000)],
    },
    Curve {
        native_id: "buff_Kite_Up",
        keys: &[
            key(1, 1000),
            key(2, 1200),
            key(3, 1400),
            key(4, 1600),
            key(5, 1800),
        ],
    },
    Curve {
        native_id: "buff_KnightCandy_CD",
        keys: &[key(1, 100000), key(5, 100000)],
    },
    Curve {
        native_id: "buff_KnightCandy_CritDamageUp",
        keys: &[
            key(1, 400),
            key(2, 480),
            key(3, 560),
            key(4, 640),
            key(5, 720),
        ],
    },
    Curve {
        native_id: "buff_LunarPhase_Atk",
        keys: &[
            key(1, 2000),
            key(2, 2300),
            key(3, 2600),
            key(4, 2900),
            key(5, 3200),
        ],
    },
    Curve {
        native_id: "buff_LunarPhase_CD",
        keys: &[key(1, 200000), key(5, 200000)],
    },
    Curve {
        native_id: "buff_LunarPhase_DefIgnore",
        keys: &[
            key(1, 1200),
            key(2, 1380),
            key(3, 1560),
            key(4, 1740),
            key(5, 1920),
        ],
    },
    Curve {
        native_id: "buff_LunarPhase_Up",
        keys: &[
            key(1, 3200),
            key(2, 3680),
            key(3, 4160),
            key(4, 4640),
            key(5, 5120),
        ],
    },
    Curve {
        native_id: "buff_MotorCandy_AtkUp",
        keys: &[
            key(1, 500),
            key(2, 600),
            key(3, 700),
            key(4, 800),
            key(5, 900),
        ],
    },
    Curve {
        native_id: "buff_MotorCandy_CD",
        keys: &[key(1, 10000), key(5, 10000)],
    },
    Curve {
        native_id: "buff_Nakipeda_CD",
        keys: &[key(1, 300000), key(5, 300000)],
    },
    Curve {
        native_id: "buff_Nakipeda_Hp",
        keys: &[
            key(1, 2400),
            key(2, 2800),
            key(3, 3200),
            key(4, 3600),
            key(5, 4000),
        ],
    },
    Curve {
        native_id: "buff_Nakipeda_effect1",
        keys: &[
            key(1, 2000),
            key(2, 2500),
            key(3, 3000),
            key(4, 3500),
            key(5, 4000),
        ],
    },
    Curve {
        native_id: "buff_Nakipeda_effect2",
        keys: &[
            key(1, 2000),
            key(2, 2500),
            key(3, 3000),
            key(4, 3500),
            key(5, 4000),
        ],
    },
    Curve {
        native_id: "buff_Nakipeda_effect2CD",
        keys: &[key(1, 150000), key(5, 150000)],
    },
    Curve {
        native_id: "buff_Nakipeda_effect3",
        keys: &[
            key(1, 1000),
            key(2, 1250),
            key(3, 1500),
            key(4, 1750),
            key(5, 2000),
        ],
    },
    Curve {
        native_id: "buff_NestBird_CD1",
        keys: &[key(1, 200000), key(5, 200000)],
    },
    Curve {
        native_id: "buff_NestBird_down",
        keys: &[
            key(1, -1800),
            key(2, -2100),
            key(3, -2400),
            key(4, -2700),
            key(5, -3000),
        ],
    },
    Curve {
        native_id: "buff_PaperPlane_Up",
        keys: &[
            key(1, 2000),
            key(2, 2375),
            key(3, 2750),
            key(4, 3125),
            key(5, 3500),
        ],
    },
    Curve {
        native_id: "buff_PoliceRat_AtkUp",
        keys: &[
            key(1, 1500),
            key(2, 1750),
            key(3, 2000),
            key(4, 2250),
            key(5, 2500),
        ],
    },
    Curve {
        native_id: "buff_PoliceRat_SkillDamage",
        keys: &[
            key(1, 10000),
            key(2, 12500),
            key(3, 15000),
            key(4, 17500),
            key(5, 20000),
        ],
    },
    Curve {
        native_id: "buff_PoliceRat_Up",
        keys: &[
            key(1, 1500),
            key(2, 1750),
            key(3, 2000),
            key(4, 2250),
            key(5, 2500),
        ],
    },
    Curve {
        native_id: "buff_Prism_CritUp",
        keys: &[
            key(1, 1600),
            key(2, 2000),
            key(3, 2400),
            key(4, 2800),
            key(5, 3200),
        ],
    },
    Curve {
        native_id: "buff_Prism_DamageUp1",
        keys: &[
            key(1, 2800),
            key(2, 3500),
            key(3, 4200),
            key(4, 4900),
            key(5, 5600),
        ],
    },
    Curve {
        native_id: "buff_Prism_DamageUp2",
        keys: &[
            key(1, 2800),
            key(2, 3500),
            key(3, 4200),
            key(4, 4900),
            key(5, 5600),
        ],
    },
    Curve {
        native_id: "buff_Prism_DamageUp2_CD",
        keys: &[key(1, 150000), key(5, 150000)],
    },
    Curve {
        native_id: "buff_Prism_LakshanaUp",
        keys: &[
            key(1, 1600),
            key(2, 2000),
            key(3, 2400),
            key(4, 2800),
            key(5, 3200),
        ],
    },
    Curve {
        native_id: "buff_Prokaryon_Up",
        keys: &[
            key(1, 1200),
            key(2, 1300),
            key(3, 1400),
            key(4, 1500),
            key(5, 1600),
        ],
    },
    Curve {
        native_id: "buff_Rat_CD",
        keys: &[key(1, 600000), key(5, 600000)],
    },
    Curve {
        native_id: "buff_Rose_AtkUp",
        keys: &[
            key(1, 1400),
            key(2, 1750),
            key(3, 2100),
            key(4, 2450),
            key(5, 2800),
        ],
    },
    Curve {
        native_id: "buff_Rose_CD",
        keys: &[key(1, 30000), key(5, 30000)],
    },
    Curve {
        native_id: "buff_Rose_CritDamageUp",
        keys: &[
            key(1, 600),
            key(2, 750),
            key(3, 900),
            key(4, 1050),
            key(5, 1200),
        ],
    },
    Curve {
        native_id: "buff_Rose_UnbalTime",
        keys: &[key(1, 30000), key(5, 30000)],
    },
    Curve {
        native_id: "buff_ThiefCandy_CD",
        keys: &[key(1, 100000), key(5, 100000)],
    },
    Curve {
        native_id: "buff_ThiefCandy_Up",
        keys: &[
            key(1, 800),
            key(2, 960),
            key(3, 1120),
            key(4, 1280),
            key(5, 1440),
        ],
    },
    Curve {
        native_id: "buff_TigerTally_AtkUp",
        keys: &[
            key(1, 1500),
            key(2, 1875),
            key(3, 2250),
            key(4, 2625),
            key(5, 3000),
        ],
    },
    Curve {
        native_id: "buff_TigerTally_CD",
        keys: &[key(1, 150000), key(5, 150000)],
    },
    Curve {
        native_id: "buff_TigerTally_CD3",
        keys: &[key(1, 100000), key(5, 100000)],
    },
    Curve {
        native_id: "buff_TigerTally_CD4",
        keys: &[key(1, 150000), key(5, 150000)],
    },
    Curve {
        native_id: "buff_TigerTally_NormalUp",
        keys: &[
            key(1, 1500),
            key(2, 1875),
            key(3, 2250),
            key(4, 2625),
            key(5, 3000),
        ],
    },
    Curve {
        native_id: "buff_TigerTally_Qup",
        keys: &[
            key(1, 1000),
            key(2, 1250),
            key(3, 1500),
            key(4, 1750),
            key(5, 2000),
        ],
    },
    Curve {
        native_id: "buff_Time_AtkUp",
        keys: &[
            key(1, 1600),
            key(2, 2000),
            key(3, 2400),
            key(4, 2800),
            key(5, 3200),
        ],
    },
    Curve {
        native_id: "buff_Time_CritDamageUp",
        keys: &[
            key(1, 800),
            key(2, 1000),
            key(3, 1200),
            key(4, 1400),
            key(5, 1600),
        ],
    },
    Curve {
        native_id: "buff_Time_DefIgnore",
        keys: &[
            key(1, 1200),
            key(2, 1500),
            key(3, 1800),
            key(4, 2100),
            key(5, 2400),
        ],
    },
    Curve {
        native_id: "buff_Time_DefIgnore_Dur",
        keys: &[key(1, 700000), key(5, 700000)],
    },
    Curve {
        native_id: "buff_Time_stateCritDamageUp",
        keys: &[
            key(1, 2400),
            key(2, 3000),
            key(3, 3600),
            key(4, 4200),
            key(5, 4800),
        ],
    },
    Curve {
        native_id: "buff_TwinBirds_CritUp1",
        keys: &[
            key(1, 1600),
            key(2, 2000),
            key(3, 2400),
            key(4, 2800),
            key(5, 3200),
        ],
    },
    Curve {
        native_id: "buff_TwinBirds_CritUp1_CD",
        keys: &[key(1, 150000), key(5, 150000)],
    },
    Curve {
        native_id: "buff_TwinBirds_CritUp3",
        keys: &[
            key(1, 3200),
            key(2, 4000),
            key(3, 4800),
            key(4, 5600),
            key(5, 6400),
        ],
    },
    Curve {
        native_id: "buff_Whale_AtkUp",
        keys: &[
            key(1, 1200),
            key(2, 1400),
            key(3, 1600),
            key(4, 1800),
            key(5, 2000),
        ],
    },
    Curve {
        native_id: "buff_Whale_CD",
        keys: &[key(1, 300000), key(5, 300000)],
    },
    Curve {
        native_id: "buff_Whale_Hp",
        keys: &[key(1, 3000), key(5, 3000)],
    },
    Curve {
        native_id: "buff_Whale_Up",
        keys: &[
            key(1, 1200),
            key(2, 1400),
            key(3, 1600),
            key(4, 1800),
            key(5, 2000),
        ],
    },
    Curve {
        native_id: "buff_appliance_Up",
        keys: &[
            key(1, 1200),
            key(2, 1300),
            key(3, 1400),
            key(4, 1500),
            key(5, 1600),
        ],
    },
    Curve {
        native_id: "buff_bowenchanggui_HP",
        keys: &[
            key(1, 1000),
            key(2, 1200),
            key(3, 1400),
            key(4, 1600),
            key(5, 1800),
        ],
    },
    Curve {
        native_id: "buff_bowenchanggui_shield",
        keys: &[
            key(1, 1000),
            key(2, 1200),
            key(3, 1400),
            key(4, 1600),
            key(5, 1800),
        ],
    },
    Curve {
        native_id: "buff_dustbin_CD",
        keys: &[key(1, 100000), key(5, 100000)],
    },
    Curve {
        native_id: "buff_dustbin_CD2",
        keys: &[key(1, 200000), key(5, 200000)],
    },
    Curve {
        native_id: "buff_dustbin_Unbal",
        keys: &[
            key(1, 600000),
            key(2, 660000),
            key(3, 720000),
            key(4, 780000),
            key(5, 840000),
        ],
    },
    Curve {
        native_id: "buff_haokewu_CD",
        keys: &[key(1, 100000), key(5, 100000)],
    },
    Curve {
        native_id: "buff_haokewu_CD2",
        keys: &[key(1, 200000), key(5, 200000)],
    },
    Curve {
        native_id: "buff_haokewu_Up",
        keys: &[
            key(1, 1800),
            key(2, 2100),
            key(3, 2400),
            key(4, 2700),
            key(5, 3000),
        ],
    },
    Curve {
        native_id: "buff_koinobori_AtkUp",
        keys: &[
            key(1, 1000),
            key(2, 1200),
            key(3, 1400),
            key(4, 1600),
            key(5, 1800),
        ],
    },
    Curve {
        native_id: "buff_koinobori_DefUp",
        keys: &[
            key(1, 1000),
            key(2, 1200),
            key(3, 1400),
            key(4, 1600),
            key(5, 1800),
        ],
    },
    Curve {
        native_id: "buff_koinobori_HpMaxUp",
        keys: &[
            key(1, 1000),
            key(2, 1200),
            key(3, 1400),
            key(4, 1600),
            key(5, 1800),
        ],
    },
    Curve {
        native_id: "buff_kudangao_Atk",
        keys: &[
            key(1, 2000),
            key(2, 2400),
            key(3, 2800),
            key(4, 3200),
            key(5, 3600),
        ],
    },
    Curve {
        native_id: "buff_kudangao_Def",
        keys: &[
            key(1, 2000),
            key(2, 2400),
            key(3, 2800),
            key(4, 3200),
            key(5, 3600),
        ],
    },
    Curve {
        native_id: "buff_mamen_CosmosUp",
        keys: &[
            key(1, 250),
            key(2, 300),
            key(3, 350),
            key(4, 400),
            key(5, 450),
        ],
    },
    Curve {
        native_id: "buff_mamen_fons",
        keys: &[key(1, 1000000000), key(5, 1000000000)],
    },
    Curve {
        native_id: "buff_mofeikesi_Atk",
        keys: &[
            key(1, 1000),
            key(2, 1150),
            key(3, 1300),
            key(4, 1450),
            key(5, 1600),
        ],
    },
    Curve {
        native_id: "buff_mofeikesi_CD",
        keys: &[key(1, 200000), key(5, 200000)],
    },
    Curve {
        native_id: "buff_mofeikesi_ChargeGetEfficiency",
        keys: &[
            key(1, 1800),
            key(2, 2100),
            key(3, 2400),
            key(4, 2700),
            key(5, 3000),
        ],
    },
    Curve {
        native_id: "buff_mofeikesi_Up",
        keys: &[
            key(1, 600),
            key(2, 700),
            key(3, 800),
            key(4, 900),
            key(5, 1000),
        ],
    },
    Curve {
        native_id: "buff_moon_CD",
        keys: &[key(1, 50000), key(5, 50000)],
    },
    Curve {
        native_id: "buff_moon_CritDamageUp",
        keys: &[
            key(1, 200),
            key(2, 240),
            key(3, 280),
            key(4, 320),
            key(5, 360),
        ],
    },
    Curve {
        native_id: "buff_moon_PsycheUp",
        keys: &[
            key(1, 1200),
            key(2, 1450),
            key(3, 1700),
            key(4, 1950),
            key(5, 2100),
        ],
    },
    Curve {
        native_id: "buff_nonos_AtkUp",
        keys: &[
            key(1, 1200),
            key(2, 1300),
            key(3, 1400),
            key(4, 1500),
            key(5, 1600),
        ],
    },
    Curve {
        native_id: "buff_nonos_CD",
        keys: &[key(1, 100000), key(5, 100000)],
    },
    Curve {
        native_id: "buff_nonos_CD2",
        keys: &[key(1, 200000), key(5, 200000)],
    },
    Curve {
        native_id: "buff_oulaquantao_Unbal",
        keys: &[
            key(1, 480000),
            key(2, 570000),
            key(3, 660000),
            key(4, 750000),
            key(5, 840000),
        ],
    },
    Curve {
        native_id: "buff_oulaquantao_Up",
        keys: &[
            key(1, 1000),
            key(2, 1200),
            key(3, 1400),
            key(4, 1600),
            key(5, 1800),
        ],
    },
    Curve {
        native_id: "buff_rishi_AtkUp",
        keys: &[
            key(1, 3000),
            key(2, 3500),
            key(3, 4000),
            key(4, 4500),
            key(5, 5000),
        ],
    },
    Curve {
        native_id: "buff_rishi_CD",
        keys: &[key(1, 3000000), key(5, 3000000)],
    },
    Curve {
        native_id: "buff_rishi_CD2",
        keys: &[key(1, 400000), key(5, 400000)],
    },
    Curve {
        native_id: "buff_rishi_energy",
        keys: &[
            key(1, 60000),
            key(2, 70000),
            key(3, 80000),
            key(4, 90000),
            key(5, 100000),
        ],
    },
    Curve {
        native_id: "buff_rishi_stack",
        keys: &[key(1, 50000), key(5, 50000)],
    },
    Curve {
        native_id: "buff_snowman_AtkUp",
        keys: &[
            key(1, 1800),
            key(2, 2100),
            key(3, 2400),
            key(4, 2700),
            key(5, 3000),
        ],
    },
    Curve {
        native_id: "buff_spider_AtkUp",
        keys: &[
            key(1, 100),
            key(2, 115),
            key(3, 130),
            key(4, 145),
            key(5, 160),
        ],
    },
    Curve {
        native_id: "buff_spider_AtkUp2",
        keys: &[
            key(1, 200),
            key(2, 230),
            key(3, 260),
            key(4, 290),
            key(5, 320),
        ],
    },
    Curve {
        native_id: "buff_spider_CD",
        keys: &[key(1, 100000), key(5, 100000)],
    },
    Curve {
        native_id: "buff_tingyuan_CD",
        keys: &[key(1, 50000), key(5, 50000)],
    },
    Curve {
        native_id: "buff_tingyuan_CD2",
        keys: &[key(1, 250000), key(5, 250000)],
    },
    Curve {
        native_id: "buff_tingyuan_CritDamageUp",
        keys: &[
            key(1, 1200),
            key(2, 1400),
            key(3, 1600),
            key(4, 1800),
            key(5, 2000),
        ],
    },
    Curve {
        native_id: "buff_tingyuan_HPreduce",
        keys: &[key(1, 500), key(5, 500)],
    },
    Curve {
        native_id: "buff_tuansanlang_Unbal",
        keys: &[
            key(1, 700000),
            key(2, 840000),
            key(3, 980000),
            key(4, 1120000),
            key(5, 1260000),
        ],
    },
    Curve {
        native_id: "buff_vine_CD",
        keys: &[key(1, 200000), key(5, 200000)],
    },
    Curve {
        native_id: "buff_vine_Hp",
        keys: &[
            key(1, 1200),
            key(2, 1400),
            key(3, 1600),
            key(4, 1800),
            key(5, 2000),
        ],
    },
    Curve {
        native_id: "buff_worldrain_CD",
        keys: &[key(1, 150000), key(5, 150000)],
    },
    Curve {
        native_id: "buff_worldrain_CosmosUp",
        keys: &[
            key(1, 3000),
            key(2, 3300),
            key(3, 3600),
            key(4, 3900),
            key(5, 4200),
        ],
    },
    Curve {
        native_id: "buff_worldrain_Mag",
        keys: &[
            key(1, 360000),
            key(2, 450000),
            key(3, 540000),
            key(4, 630000),
            key(5, 720000),
        ],
    },
    Curve {
        native_id: "buff_wujinjieti_CD",
        keys: &[key(1, 100000), key(5, 100000)],
    },
    Curve {
        native_id: "buff_wujinjieti_Up",
        keys: &[
            key(1, 200),
            key(2, 240),
            key(3, 280),
            key(4, 320),
            key(5, 360),
        ],
    },
    Curve {
        native_id: "buff_wushoutieyu_CD",
        keys: &[key(1, 100000), key(5, 100000)],
    },
    Curve {
        native_id: "buff_wushoutieyu_Up",
        keys: &[
            key(1, 1000),
            key(2, 1250),
            key(3, 1500),
            key(4, 1750),
            key(5, 2000),
        ],
    },
    Curve {
        native_id: "buff_wushoutieyu_Up2",
        keys: &[
            key(1, 500),
            key(2, 600),
            key(3, 700),
            key(4, 800),
            key(5, 900),
        ],
    },
    Curve {
        native_id: "buff_wushoutieyu_UpLakshana",
        keys: &[
            key(1, 1500),
            key(2, 1750),
            key(3, 2000),
            key(4, 2250),
            key(5, 2500),
        ],
    },
    Curve {
        native_id: "buff_xiangdao_CD",
        keys: &[key(1, 200000), key(5, 200000)],
    },
    Curve {
        native_id: "buff_xiangdao_energy",
        keys: &[
            key(1, 100000),
            key(2, 120000),
            key(3, 140000),
            key(4, 160000),
            key(5, 180000),
        ],
    },
];
