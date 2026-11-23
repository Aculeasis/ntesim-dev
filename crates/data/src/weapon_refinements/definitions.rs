use super::{Definition, Description, Parameter};

pub(super) const DEFINITIONS: &[Definition] = &[
    Definition {
        pack: "upgradestar_pack_fork_Arachne",
        parameters: &[
            Parameter {
                native_id: "buff_Arachne_Hp",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_Arachne_Up",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_Arachne_CD",
                percent_hint: false,
            },
        ],
        description: Description::Shared {
            key: "buff_fork_Arachne_effect",
            text: "Increases Max HP by <lv>{0}</>.\nIncreases Mental DMG dealt by <lv>{1}</> for <lv>{2}</>s after the wearer casts an Ultimate.",
        },
    },
    Definition {
        pack: "upgradestar_pack_fork_BitGame",
        parameters: &[
            Parameter {
                native_id: "buff_BitGame2_AtkUp1",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_BitGame2_AtkUp2",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_BitGame2_CD",
                percent_hint: false,
            },
            Parameter {
                native_id: "buff_BitGame2_DamageUpPsycheBase1",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_BitGame2_DamageUpPsycheBase2",
                percent_hint: true,
            },
        ],
        description: Description::Shared {
            key: "buff_fork_BitGame_effect",
            text: "Increases the active character's ATK by <lv>{0}</> while the wearer is off-field, and increases the active character's ATK by <lv>{1}</> when the wearer deals damage, up to <lv>4</> stacks and triggering at most <lv>once</> every <lv>{2}</>s. Resets the effect when the wearer becomes the active character.\r\nIncreases Psyche DMG by <lv>{3}</> while the wearer is the active character, and increases Psyche DMG by <lv>{4}</> when dealing Psyche DMG with a Basic Attack, up to <lv>10</> stacks. Triggers at most once every <lv>0.3</>s. Resets the effect when switching characters.",
        },
    },
    Definition {
        pack: "upgradestar_pack_fork_BitterCake",
        parameters: &[
            Parameter {
                native_id: "buff_BitterCake_DefUp",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_BitterCake_CD",
                percent_hint: false,
            },
            Parameter {
                native_id: "buff_BitterCake_CD2",
                percent_hint: false,
            },
        ],
        description: Description::Shared {
            key: "buff_fork_BitterCake_effect",
            text: "Increases wearer's DEF by <lv>{0}</> for <lv>{1}</>s when taking damage. Triggers at most once every <lv>{2}</>s.",
        },
    },
    Definition {
        pack: "upgradestar_pack_fork_BlackBook",
        parameters: &[
            Parameter {
                native_id: "buff_BlackBook2_Unbal",
                percent_hint: false,
            },
            Parameter {
                native_id: "buff_BlackBook2_CD",
                percent_hint: false,
            },
            Parameter {
                native_id: "buff_BlackBook2_CD2",
                percent_hint: false,
            },
            Parameter {
                native_id: "buff_BlackBook2_DamageUpChaosBase",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_BlackBook2_SkillDamage",
                percent_hint: true,
            },
        ],
        description: Description::Shared {
            key: "buff_fork_BlackBook_effect",
            text: "Increases Break Intensity by <lv>{0}</>.\r\nSeals Arc: Black Tome with two chains by default. Unleashes 1 chain each time any character on the team casts a Support Skill. Makes Arc: Black Tome available when all two chains are unleashed.\r\nArc: Black Tome — Summons the Black Tome for <lv>{1}</>s. Designates an enemy every <lv>{2}</>s. Increases the wearer's Chaos DMG dealt to designated enemies by <lv>{3}</>. Deals Chaos DMG equal to <lv>{4}</> of the wearer's ATK to designated enemies when they take Break Damage.",
        },
    },
    Definition {
        pack: "upgradestar_pack_fork_BlastCandy",
        parameters: &[
            Parameter {
                native_id: "buff_BlastCandy_AtkUp",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_BlastCandy_CD",
                percent_hint: false,
            },
        ],
        description: Description::Shared {
            key: "buff_fork_BlastCandy_effect",
            text: "Increases ATK by <lv>{0}</> for <lv>{1}</>s after the wearer casts an Ultimate. Effect does not stack.",
        },
    },
    Definition {
        pack: "upgradestar_pack_fork_BoxingCandy",
        parameters: &[
            Parameter {
                native_id: "buff_BoxingCandy_Up",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_BoxingCandy_Up2",
                percent_hint: true,
            },
        ],
        description: Description::Shared {
            key: "buff_fork_BoxingCandy_effect",
            text: "Increases the wearer's DMG by <lv>{0}</>. Increases this effect to <lv>{1}</> against enemies with HP below 50%.",
        },
    },
    Definition {
        pack: "upgradestar_pack_fork_Butterfly",
        parameters: &[
            Parameter {
                native_id: "buff_Butterfly_DamageUpNatureBase",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_Butterfly_attachedup",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_Butterfly_attachedup2",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_Butterfly_CD",
                percent_hint: false,
            },
        ],
        description: Description::Shared {
            key: "buff_fork_Butterfly_effect",
            text: "Increases Anima DMG by <lv>{0}</>.\nIncreases the wearer's Attachment DMG by <lv>{1}</>. Increases the Attachment DMG bonus to <lv>{2}</> for <lv>{3}</>s when the wearer casts an Ultimate. Effect does not stack.",
        },
    },
    Definition {
        pack: "upgradestar_pack_fork_Castle",
        parameters: &[
            Parameter {
                native_id: "buff_Castle_HealUp",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_Castle_CD",
                percent_hint: false,
            },
        ],
        description: Description::Shared {
            key: "buff_fork_Castle_effect",
            text: "Increases Healing Bonus by <lv>{0}</> for <lv>{1}</>s after the wearer casts a Redirect Skill. Effect does not stack.",
        },
    },
    Definition {
        pack: "upgradestar_pack_fork_Crowbar",
        parameters: &[
            Parameter {
                native_id: "buff_Crowbar_Unbal",
                percent_hint: false,
            },
            Parameter {
                native_id: "buff_Crowbar_CD",
                percent_hint: false,
            },
        ],
        description: Description::Shared {
            key: "buff_fork_Crowbar_effect",
            text: "Increases Break Intensity by <lv>{0}</> for <lv>{1}</>s after the wearer casts a Redirect Skill. Effect does not stack.\nUnlocks Arc: Picky.\nArc: Picky — Summons Picky to unlock all nearby lockable objects.",
        },
    },
    Definition {
        pack: "upgradestar_pack_fork_DemonBlade",
        parameters: &[
            Parameter {
                native_id: "buff_DemonBlade_Crit",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_DemonBlade_CritDamageUp",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_DemonBlade_CD",
                percent_hint: false,
            },
        ],
        description: Description::Shared {
            key: "buff_fork_DemonBlade_effect",
            text: "Increases Crit Rate by <lv>{0}</>.\r\nGrants the wearer a <lv>{1}</> Crit DMG bonus for <lv>{2}</>s when they deal Incantation DMG. Stacks up to <lv>7</> times, with no more than 1 stack gained every <lv>0.3</>s. Resets the duration when triggered again.",
        },
    },
    Definition {
        pack: "upgradestar_pack_fork_Door",
        parameters: &[
            Parameter {
                native_id: "buff_Door_Atk",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_Door_OtherUp",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_Door_Up",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_Door_CD",
                percent_hint: false,
            },
        ],
        description: Description::Shared {
            key: "buff_fork_Door_effect",
            text: "Increases ATK by <lv>{0}</>.\r\nWhen the wearer performs healing, increases the wearer's Anima DMG dealt by <lv>{2}</> and all allies' DMG dealt by <lv>{1}</> for <lv>{3}</>s. Resets the duration when triggered again. Effects with the same name do not stack.",
        },
    },
    Definition {
        pack: "upgradestar_pack_fork_GoldRecord",
        parameters: &[
            Parameter {
                native_id: "buff_GoldRecord_AtkUp",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_GoldRecord_QteCritDmaUp",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_GoldRecord_QCritDmaUp",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_GoldRecord_CD",
                percent_hint: false,
            },
        ],
        description: Description::Shared {
            key: "buff_fork_GoldRecord_effect",
            text: "Increases ATK by <lv>{0}</>.\r\nIncreases the wearer's Support Skill Crit DMG by <lv>{1}</>. Dealing DMG with a triggered Support Skill increases their Ultimate Crit DMG by <lv>{2}</> for <lv>{3}</>s, up to <lv>3</> stacks. Resets the duration when triggered again.",
        },
    },
    Definition {
        pack: "upgradestar_pack_fork_GoldWool",
        parameters: &[
            Parameter {
                native_id: "buff_GoldWool_Up",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_GoldWool_CritDamageUp",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_GoldWool_CD",
                percent_hint: false,
            },
        ],
        description: Description::Shared {
            key: "buff_fork_GoldWool_effect",
            text: "Increases the wearer's Lakshana DMG by <lv>{0}</>. Increases the wearer's CRIT DMG by <lv>{1}</> for <lv>{2}</>s when they cast a Redirect Skill or Ultimate. Resets the duration when triggered again.",
        },
    },
    Definition {
        pack: "upgradestar_pack_fork_Kite",
        parameters: &[
            Parameter {
                native_id: "buff_Kite_AtkUp",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_Kite_Up",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_Kite_CD",
                percent_hint: false,
            },
        ],
        description: Description::Shared {
            key: "buff_fork_Kite_effect",
            text: "Increases ATK by <lv>{0}</> and Lakshana DMG to targets affected by Remora and Stain by <lv>{1}</> for <lv>{2}</>s after the wearer casts a Redirect Skill. Effect does not stack.",
        },
    },
    Definition {
        pack: "upgradestar_pack_fork_KnightCandy",
        parameters: &[
            Parameter {
                native_id: "buff_KnightCandy_CritDamageUp",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_KnightCandy_CD",
                percent_hint: false,
            },
        ],
        description: Description::Shared {
            key: "buff_fork_KnightCandy_effect",
            text: "Increases wearer's CRIT DMG by <lv>{0}</> for <lv>{1}</>s after landing a critical hit, up to <lv>10</> stacks. Resets the duration when triggered again.",
        },
    },
    Definition {
        pack: "upgradestar_pack_fork_LunarPhase",
        parameters: &[
            Parameter {
                native_id: "buff_LunarPhase_Atk",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_LunarPhase_Up",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_LunarPhase_DefIgnore",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_LunarPhase_CD",
                percent_hint: false,
            },
        ],
        description: Description::Shared {
            key: "buff_fork_LunarPhase_effect",
            text: "Increases ATK by <lv>{0}</>. \nAfter casting Ultimate, increases the wearer's Cosmos DMG dealt by <lv>{1}</> and allows the wearer to ignore <lv>{2}</> of enemy DEF for <lv>{3}</>s. Resets the duration when triggered again.",
        },
    },
    Definition {
        pack: "upgradestar_pack_fork_MotorCandy",
        parameters: &[
            Parameter {
                native_id: "buff_MotorCandy_CD",
                percent_hint: false,
            },
            Parameter {
                native_id: "buff_MotorCandy_AtkUp",
                percent_hint: true,
            },
        ],
        description: Description::Shared {
            key: "buff_fork_MotorCandy_effect",
            text: "Increases <lv>{1}</> ATK every <lv>{0}</>s while the wearer is the active character, up to <lv>5</> stacks. Resets when the wearer leaves the field.",
        },
    },
    Definition {
        pack: "upgradestar_pack_fork_Nakupeda",
        parameters: &[
            Parameter {
                native_id: "buff_Nakipeda_Hp",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_Nakipeda_effect1",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_Nakipeda_effect2",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_Nakipeda_effect2CD",
                percent_hint: false,
            },
            Parameter {
                native_id: "buff_Nakipeda_effect3",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_Nakipeda_CD",
                percent_hint: false,
            },
        ],
        description: Description::Shared {
            key: "buff_fork_Nakupeda_effect",
            text: "Increases HP by <lv>{0}</>.\nGrants one of the following random effects when the wearer casts an Ultimate:\n1. Recovers <lv>{1}</> of Max HP for the team member with the lowest HP percentage.\n2. Grants a Shield equal to <lv>{2}</> of the wearer's Max HP for <lv>{3}</>s.\n3. Recovers <lv>{4}</> HP for all team members.\nTriggers once every <lv>{5}</>s.",
        },
    },
    Definition {
        pack: "upgradestar_pack_fork_NestBird",
        parameters: &[],
        description: Description::Ranked {
            key_prefix: "buff_fork_NestBird_effect",
            texts: [
                "Applies Warning Gaze on enemies hit by the wearer's Ultimate. Marked enemies deal <lv>18%</> reduced damage for <lv>20</>s. Effect does not stack.",
                "Applies Warning Gaze on enemies hit by the wearer's Ultimate. Marked enemies deal <lv>21%</> reduced damage for <lv>20</>s. Effect does not stack.",
                "Inflicts Warning Gaze on enemies hit by the wearer's Ultimate, reducing their damage dealt by <lv>24%</> for <lv>20</>s. Effect does not stack.",
                "Applies Warning Gaze on enemies hit by the wearer's Ultimate. Marked enemies deal <lv>27%</> reduced damage for <lv>20</>s. Effect does not stack.",
                "Applies Warning Gaze on enemies hit by the wearer's Ultimate. Marked enemies deal <lv>30%</> reduced damage for <lv>20</>s. Effect does not stack.",
            ],
        },
    },
    Definition {
        pack: "upgradestar_pack_fork_PaperPlane",
        parameters: &[Parameter {
            native_id: "buff_PaperPlane_Up",
            percent_hint: true,
        }],
        description: Description::Shared {
            key: "buff_fork_PaperPlane_effect",
            text: "Increases Anima DMG dealt by the wearer's Redirect Skill and Ultimate by <lv>{0}</>.",
        },
    },
    Definition {
        pack: "upgradestar_pack_fork_PoliceRat",
        parameters: &[
            Parameter {
                native_id: "buff_PoliceRat_AtkUp",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_PoliceRat_Up",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_PoliceRat_SkillDamage",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_Rat_CD",
                percent_hint: false,
            },
        ],
        description: Description::Shared {
            key: "buff_fork_PoliceRat_effect",
            text: "Increases ATK by <lv>{0}</>.\nIncreases damage dealt to Bosses by <lv>{1}</>.\nUnlocks Arc: Officer Whisker.\nArc: Officer Whisker — Summons Officer Whisker to assist in combat. Officer Whisker continuously attacks enemies, dealing <lv>{2}</> of the wearer's ATK as damage per hit. Lasts <lv>30</>s. Cooldown: <lv>{3}</>s.",
        },
    },
    Definition {
        pack: "upgradestar_pack_fork_Prokaryon",
        parameters: &[Parameter {
            native_id: "buff_Prokaryon_Up",
            percent_hint: true,
        }],
        description: Description::Shared {
            key: "buff_fork_Prokaryon_effect",
            text: "Increases the wearer's Basic Attack damage by <lv>{0}</>.",
        },
    },
    Definition {
        pack: "upgradestar_pack_fork_Rose",
        parameters: &[
            Parameter {
                native_id: "buff_Rose_AtkUp",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_Rose_CritDamageUp",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_Rose_CD",
                percent_hint: false,
            },
            Parameter {
                native_id: "buff_Rose_UnbalTime",
                percent_hint: false,
            },
        ],
        description: Description::Shared {
            key: "buff_fork_Rose_effect",
            text: "Increases ATK by <lv>{0}</>.\nGrants 1 stack of Chaos Thorn each time the wearer deals DoT. Increases CRIT DMG by <lv>{1}</> per stack for <lv>{2}</>s. Triggers at most <lv>once</> every <lv>0.3</>s, up to <lv>10</> stacks and refreshes the duration when retriggered. Grants <lv>10</> stacks of Chaos Thorn immediately when the wearer casts a Redirect Skill.\nExtends the Broken state of a Broken enemy by <lv>{3}</>s when the wearer deals damage to them (triggers at most once per Break Effect).",
        },
    },
    Definition {
        pack: "upgradestar_pack_fork_ThiefCandy",
        parameters: &[
            Parameter {
                native_id: "buff_ThiefCandy_Up",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_ThiefCandy_CD",
                percent_hint: false,
            },
        ],
        description: Description::Shared {
            key: "buff_fork_ThiefCandy_effect",
            text: "Increases wearer's damage by <lv>{0}</> for <lv>{1}</>s after performing a Critical Dodge, up to <lv>3</> stacks. Resets the duration when triggered again.",
        },
    },
    Definition {
        pack: "upgradestar_pack_fork_TigerTally",
        parameters: &[
            Parameter {
                native_id: "buff_TigerTally_AtkUp",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_TigerTally_NormalUp",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_TigerTally_CD",
                percent_hint: false,
            },
            Parameter {
                native_id: "buff_TigerTally_CD4",
                percent_hint: false,
            },
            Parameter {
                native_id: "buff_TigerTally_Qup",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_TigerTally_CD3",
                percent_hint: false,
            },
        ],
        description: Description::Shared {
            key: "buff_fork_TigerTally_effect",
            text: "Increases ATK by <lv>{0}</>.\nIncreases Basic Attack and Critical Riposte damage by <lv>{1}</> for <lv>{2}</>s when casting a Redirect Skill or Ultimate, up to <lv>2</> stacks.\nGrants a Left Tiger Talisman when casting Redirect Skill and a Right Tiger Talisman when casting Ultimate.\nUnlocks Commander Tiger Talisman if the wearer obtains the other type of talisman within <lv>{3}</>s of gaining one.\nArc: Commander Tiger Talisman — Increases the wearer's damage to Bosses by <lv>{4}</> for <lv>{5}</>s.",
        },
    },
    Definition {
        pack: "upgradestar_pack_fork_Time",
        parameters: &[
            Parameter {
                native_id: "buff_Time_AtkUp",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_Time_stateCritDamageUp",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_Time_CritDamageUp",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_Time_DefIgnore",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_Time_DefIgnore_Dur",
                percent_hint: false,
            },
        ],
        description: Description::Shared {
            key: "buff_fork_Time_effect",
            text: "Increases ATK by <lv>{0}</>.\r\nEnters Wastelab and clears current Wastetime when the wearer casts a Redirect Skill. While in Wastelab, allies gain <lv>1</>stack of Wastetime each time they use a Redirect or Support Skill, up to <lv>3</> stacks. When the wearer uses their Ultimate in Wastelab, they exit Wastelab and consume all Wastetime, increasing Ultimate Crit DMG by <lv>{1}</>, plus an additional <lv>{2}</> per Wastetime consumed. Consuming <lv>3</> Wastetime stacks at once grants an extra <lv>{3}</> DEF Ignore for <lv>{4}</>s.",
        },
    },
    Definition {
        pack: "upgradestar_pack_fork_Whale",
        parameters: &[
            Parameter {
                native_id: "buff_Whale_AtkUp",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_Whale_Up",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_Whale_Hp",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_Whale_CD",
                percent_hint: false,
            },
        ],
        description: Description::Shared {
            key: "buff_fork_Whale_effect",
            text: "Increases ATK by <lv>{0}</>.\nIncreases the wearer's damage to Broken enemies by <lv>{1}</>. Restores <lv>{2}</> HP to the wearer if a Broken enemy is defeated. Triggers at most <lv>once</> every <lv>{3}</>s.",
        },
    },
    Definition {
        pack: "upgradestar_pack_fork_appliance",
        parameters: &[Parameter {
            native_id: "buff_appliance_Up",
            percent_hint: true,
        }],
        description: Description::Shared {
            key: "buff_fork_appliance_effect",
            text: "Grants the wearer a <lv>{0}</> Redirect Skill DMG bonus.",
        },
    },
    Definition {
        pack: "upgradestar_pack_fork_bopu",
        parameters: &[
            Parameter {
                native_id: "buff_haokewu_CD",
                percent_hint: false,
            },
            Parameter {
                native_id: "buff_haokewu_Up",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_haokewu_CD2",
                percent_hint: false,
            },
        ],
        description: Description::Shared {
            key: "buff_fork_bopu_effect",
            text: "Increases wearer's damage dealt by <lv>{1}</> for <lv>{0}s</> after casting Support Skill. Triggers at most once every <lv>{2}s</>.",
        },
    },
    Definition {
        pack: "upgradestar_pack_fork_dustbin",
        parameters: &[
            Parameter {
                native_id: "buff_dustbin_Unbal",
                percent_hint: false,
            },
            Parameter {
                native_id: "buff_dustbin_CD",
                percent_hint: false,
            },
            Parameter {
                native_id: "buff_dustbin_CD2",
                percent_hint: false,
            },
        ],
        description: Description::Shared {
            key: "buff_fork_dustbin_effect",
            text: "Increases wearer's Break Intensity by <lv>{0}</> for <lv>{1}</>s when reducing Break. Triggers at most once every <lv>{2}</>s.",
        },
    },
    Definition {
        pack: "upgradestar_pack_fork_jiaojuan",
        parameters: &[
            Parameter {
                native_id: "buff_oulaquantao_Unbal",
                percent_hint: false,
            },
            Parameter {
                native_id: "buff_oulaquantao_Up",
                percent_hint: true,
            },
        ],
        description: Description::Shared {
            key: "buff_fork_jiaojuan_effect",
            text: "Increases Break Intensity by <lv>{0}</>.\nIncreases DMG by <lv>{1}</> against Broken units.",
        },
    },
    Definition {
        pack: "upgradestar_pack_fork_jingmotingyuan",
        parameters: &[
            Parameter {
                native_id: "buff_tingyuan_CD",
                percent_hint: false,
            },
            Parameter {
                native_id: "buff_tingyuan_CritDamageUp",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_tingyuan_HPreduce",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_tingyuan_CD2",
                percent_hint: false,
            },
            Parameter {
                native_id: "GE_Fork_serenity_Skill1_Damage1",
                percent_hint: true,
            },
            Parameter {
                native_id: "GE_Fork_serenity_Skill2_Damage",
                percent_hint: true,
            },
        ],
        description: Description::Shared {
            key: "buff_fork_jingmotingyuan_effect",
            text: "Increases CRIT DMG by <lv>{1}</> for <lv>{0}</>s when the wearer's HP is reduced without taking damage, up to <lv>4</> stacks. Unlocks Arc: Silent Garden.\r\nConsumes <lv>{2}</> of current HP and randomly performs one of the following attacks (CD <lv>{3}</>s) in Arc: Silent Garden:\r\nHarsh Rebuke: Rains merciless rebukes like leaves in a storm, dealing <lv>5</> instances of Incantation DMG of <lv>{4}</> ATK.\r\nIncessant Nagging: Pierces targets with sharp, cutting words like thorns, dealing <lv>6</> instances of Incantation DMG of <lv>{5}</> ATK ; the final instance deals double damage.",
        },
    },
    Definition {
        pack: "upgradestar_pack_fork_koinobori",
        parameters: &[
            Parameter {
                native_id: "buff_koinobori_AtkUp",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_koinobori_DefUp",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_koinobori_HpMaxUp",
                percent_hint: true,
            },
        ],
        description: Description::Shared {
            key: "buff_fork_koinobori_effect",
            text: "Grants the wearer <lv>{0}</> ATK, <lv>{1}</> DEF, and <lv>{2}</> HP when the team has at least <lv>3</> different Esper Types.",
        },
    },
    Definition {
        pack: "upgradestar_pack_fork_lingganzhongjiezhe",
        parameters: &[
            Parameter {
                native_id: "buff_xiangdao_energy",
                percent_hint: false,
            },
            Parameter {
                native_id: "buff_xiangdao_CD",
                percent_hint: false,
            },
        ],
        description: Description::Shared {
            key: "buff_fork_lingganzhongjiezhe_effect",
            text: "Grants the wearer <lv>{0}</> Ultimate Energy when casting a Redirect Skill. Triggers up to once every <lv>{1}</>s.",
        },
    },
    Definition {
        pack: "upgradestar_pack_fork_mamen",
        parameters: &[
            Parameter {
                native_id: "buff_mamen_fons",
                percent_hint: false,
            },
            Parameter {
                native_id: "buff_mamen_CosmosUp",
                percent_hint: true,
            },
        ],
        description: Description::Shared {
            key: "buff_fork_mamen_effect",
            text: "Increases Cosmos DMG by <lv>{1}</> for every <lv>{0}</> Fons the wearer holds, up to <lv>10</> stacks.",
        },
    },
    Definition {
        pack: "upgradestar_pack_fork_mofeikesi",
        parameters: &[
            Parameter {
                native_id: "buff_mofeikesi_ChargeGetEfficiency",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_mofeikesi_CD",
                percent_hint: false,
            },
            Parameter {
                native_id: "buff_mofeikesi_Atk",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_mofeikesi_Up",
                percent_hint: true,
            },
        ],
        description: Description::Shared {
            key: "buff_fork_mofeikesi_effect",
            text: "Increases Charge Efficiency by <lv>{0}</>.\r\nIncreases Team ATK by <lv>{2}</> for <lv>{1}</>s after the wearer casts an Ultimate. Increases Team ATK by an additional <lv>{3}</> if enemies are controlled by the Ultimate. Effects with the same name do not stack.",
        },
    },
    Definition {
        pack: "upgradestar_pack_fork_moon",
        parameters: &[
            Parameter {
                native_id: "buff_moon_PsycheUp",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_moon_CritDamageUp",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_moon_CD",
                percent_hint: false,
            },
        ],
        description: Description::Shared {
            key: "buff_fork_moon_effect",
            text: "Increases the wearer's Psyche DMG by <lv>{0}</>. Increases Crit DMG dealt by <lv>{1}</> for <lv>{2}</>s when dealing Psyche DMG, up to <lv>10</> stacks. Triggers at most once every <lv>0.1</>s.",
        },
    },
    Definition {
        pack: "upgradestar_pack_fork_nonos",
        parameters: &[
            Parameter {
                native_id: "buff_nonos_AtkUp",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_nonos_CD",
                percent_hint: false,
            },
            Parameter {
                native_id: "buff_nonos_CD2",
                percent_hint: false,
            },
        ],
        description: Description::Shared {
            key: "buff_fork_nonos_effect",
            text: "Increases wearer's ATK by <lv>{0}</> for <lv>{1}</>s when casting a Redirect Skill. Triggers up to once every <lv>{2}</>s.",
        },
    },
    Definition {
        pack: "upgradestar_pack_fork_oulaquantao",
        parameters: &[
            Parameter {
                native_id: "buff_wujinjieti_CD",
                percent_hint: false,
            },
            Parameter {
                native_id: "buff_wujinjieti_Up",
                percent_hint: true,
            },
        ],
        description: Description::Shared {
            key: "buff_fork_oulaquantao_effect",
            text: "Increases the wearer's Basic Attack DMG by <lv>{1}</> for <lv>{0}</>s after each Basic Attack, up to <lv>10</> stacks. Each stack lasts independently.",
        },
    },
    Definition {
        pack: "upgradestar_pack_fork_prism",
        parameters: &[
            Parameter {
                native_id: "buff_Prism_CritUp",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_Prism_DamageUp1",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_Prism_DamageUp2",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_Prism_LakshanaUp",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_Prism_DamageUp2_CD",
                percent_hint: false,
            },
        ],
        description: Description::Shared {
            key: "buff_fork_prism_effect",
            text: "Increases Crit Rate by <lv>{0}</>.\r\nIncreases the wearer's Redirect Skill DMG by <lv>{1}</>. Increases the wearer's Basic Attack DMG by <lv>{2}</> and Lakshana DMG by <lv>{3}</> for <lv>{4}</>s after casting a Redirect Skill.",
        },
    },
    Definition {
        pack: "upgradestar_pack_fork_rishi",
        parameters: &[
            Parameter {
                native_id: "buff_rishi_AtkUp",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_rishi_CD2",
                percent_hint: false,
            },
            Parameter {
                native_id: "buff_rishi_energy",
                percent_hint: false,
            },
            Parameter {
                native_id: "buff_rishi_stack",
                percent_hint: false,
            },
            Parameter {
                native_id: "buff_rishi_CD",
                percent_hint: false,
            },
        ],
        description: Description::Shared {
            key: "buff_fork_rishi_effect",
            text: "Increases wearer's ATK by <lv>{0}</>. Unlocks Arc: Eclipse.\r\nArc: Eclipse — The wearer creates a <lv>{1}</>s Eclipse. During the Eclipse, each enemy defeated restores <lv>{2}</> Ultimate Energy, up to <lv>{3}</> times. (cooldown: <lv>{4}</>s).",
        },
    },
    Definition {
        pack: "upgradestar_pack_fork_snowman",
        parameters: &[Parameter {
            native_id: "buff_snowman_AtkUp",
            percent_hint: true,
        }],
        description: Description::Shared {
            key: "buff_fork_snowman_effect",
            text: "Increases ATK by <lv>{0}</> while the wearer has a Shield.",
        },
    },
    Definition {
        pack: "upgradestar_pack_fork_spider",
        parameters: &[
            Parameter {
                native_id: "buff_spider_CD",
                percent_hint: false,
            },
            Parameter {
                native_id: "buff_spider_AtkUp",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_spider_AtkUp2",
                percent_hint: true,
            },
        ],
        description: Description::Shared {
            key: "buff_fork_spider_effect",
            text: "Grants the wearer <lv>1</> stack of Spider Knowledge when dealing damage with Basic Attacks, up to <lv>8</> stacks (up to <lv>1</> stack every <lv>0.5</>s). All stacks are consumed when casting Ultimate. Increases the entire team's ATK by <lv>{1}</> for <lv>{0}</>s <lv>per</> stack consumed. Increases ATK by an additional <lv>{2}</> when <lv>8</> stacks are consumed.",
        },
    },
    Definition {
        pack: "upgradestar_pack_fork_tuansanlang",
        parameters: &[Parameter {
            native_id: "buff_tuansanlang_Unbal",
            percent_hint: false,
        }],
        description: Description::Shared {
            key: "buff_fork_tuansanlang_effect",
            text: "Increases Break Intensity by <lv>{0}</> for all characters of the same Esper Type as the wearer (including the wearer) when the team has <lv>3</> or more such characters. This effect does not stack.",
        },
    },
    Definition {
        pack: "upgradestar_pack_fork_twinbirds",
        parameters: &[
            Parameter {
                native_id: "Buff_TwinBirds_MagUp",
                percent_hint: false,
            },
            Parameter {
                native_id: "Buff_TwinBirds_PsycheUp",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_TwinBirds_CritUp1",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_TwinBirds_CritUp3",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_TwinBirds_CritUp1_CD",
                percent_hint: false,
            },
        ],
        description: Description::Shared {
            key: "buff_fork_TwinBirds_effect",
            text: "Increases Cycle Intensity by <lv>{0}</>.\r\nIncreases the wearer's Psyche DMG by <lv>{1}</>. Increases the team's Crit DMG by <lv>{2}</>, and the wearer's by an additional <lv>{3}</>, for <lv>{4}</>s after the wearer casts Ultimate. Resets the duration when triggered again.",
        },
    },
    Definition {
        pack: "upgradestar_pack_fork_vine",
        parameters: &[
            Parameter {
                native_id: "buff_vine_Hp",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_vine_CD",
                percent_hint: false,
            },
        ],
        description: Description::Shared {
            key: "buff_fork_vine_effect",
            text: "Restores wearer's HP by <lv>{0}</> upon defeating an enemy. Triggers at most once every <lv>{1}</>s.",
        },
    },
    Definition {
        pack: "upgradestar_pack_fork_Worldrain",
        parameters: &[
            Parameter {
                native_id: "buff_worldrain_CosmosUp",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_worldrain_Mag",
                percent_hint: false,
            },
            Parameter {
                native_id: "buff_worldrain_CD",
                percent_hint: false,
            },
        ],
        description: Description::Shared {
            key: "buff_fork_worldrain_effect",
            text: "Deals <lv>{0}</> increased Cosmos DMG with the wearer's Redirect Skill and Ultimate.\r\nIncreases the wearer's Cycle Intensity by <lv>{1}</> for <lv>{2}</>s after casting a Redirect Skill. Resets the duration when triggered again.",
        },
    },
    Definition {
        pack: "upgradestar_pack_fork_wuhuakuang",
        parameters: &[
            Parameter {
                native_id: "buff_kudangao_Atk",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_kudangao_Def",
                percent_hint: true,
            },
        ],
        description: Description::Shared {
            key: "buff_fork_wuhuakuang_effect",
            text: "Increases ATK by <lv>{0}</> when the wearer’s HP is above 50%; increases DEF by <lv>{1}</> when HP is below 50%.",
        },
    },
    Definition {
        pack: "upgradestar_pack_Fork_wushoutieyu",
        parameters: &[
            Parameter {
                native_id: "buff_wushoutieyu_Up",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_wushoutieyu_CD",
                percent_hint: false,
            },
            Parameter {
                native_id: "buff_wushoutieyu_Up2",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_wushoutieyu_UpLakshana",
                percent_hint: true,
            },
        ],
        description: Description::Shared {
            key: "buff_fork_wushoutieyu_effect",
            text: "Increases Lakshana DMG by <lv>{3}</>.\r\nIncreases the wearer's Redirect Skill and Ultimate DMG by <lv>{0}</> for <lv>{1}</>s after casting Ultimate. Increases the wearer’s Redirect Skill DMG by <lv>{2}</> per cast while active, up to 2 stacks.",
        },
    },
    Definition {
        pack: "upgradestar_pack_fork_yaodao",
        parameters: &[Parameter {
            native_id: "GE_Fork_yaodao_Skill_Damage",
            percent_hint: true,
        }],
        description: Description::Shared {
            key: "buff_fork_yaodao_effect",
            text: "Deals additional Incantation DMG equal to <lv>{0}</> of ATK when the wearer triggers a Parry Attack.",
        },
    },
    Definition {
        pack: "upgradestar_pack_fork_yuren",
        parameters: &[
            Parameter {
                native_id: "buff_bowenchanggui_shield",
                percent_hint: true,
            },
            Parameter {
                native_id: "buff_bowenchanggui_HP",
                percent_hint: true,
            },
        ],
        description: Description::Shared {
            key: "buff_fork_yuren_effect",
            text: "Increases HP by <lv>{0}</>.\r\nStrengthens shields by <lv>{1}</> while the wearer's HP is above 50%.",
        },
    },
];
