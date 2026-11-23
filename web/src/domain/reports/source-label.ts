import { english } from '../localization';

const effects: Record<string, string> = {
  // Existing curated effects
  GE_Player_Female051_Skill4_Damage: 'Skill hit',
  GE_Player_Female051_Skill_Kill_Damage_lv1: 'Awakening 1 additional hit',
  GE_Player_Oneiroi_Skill_Gun_Damage: 'Gun lamb',
  GE_Player_Oneiroi_Skill_Syringe_Damage: 'Syringe lamb',
  GE_Player_Oneiroi_Skill_Trumpet_Damage: 'Trumpet lamb',
  GE_Damage_sheep_gun_awaken2: 'Gun lamb (Awakening 2)',
  GE_Damage_sheep_syringe_awaken2: 'Syringe lamb (Awakening 2)',
  GE_Damage_sheep_trumpet_awaken2: 'Trumpet lamb (Awakening 2)',
  GE_Player_Lacrimosa_Blood_Damage: 'Nightmare (DoT)',
  GE_Player_Lacrimosa_Blood_Damage_LV6: 'Nightmare (DoT)',
  GE_Player_Lacrimosa_B_Melee3_Explode_Damage: 'Shatter',
  GE_Player_Lacrimosa_Pan_Damage: 'Pan Strike',

  // Zankou DoTs
  GE_Player_Zankou_DotDamage: 'Heartwrench (DoT)',
  GE_Player_Zankou_DotUltraDamage: 'Vile Ash (DoT)',

  // Zankou Skill variants (Reality Sanguine Dash / Illusion Soulcross)
  GE_Player_Zankou_Skill1_Damage: 'Sanguine Dash (Enhanced)',
  GE_Player_Zankou_Skill1_1_Damage: 'Sanguine Dash (Enhanced) · Extra',
  GE_Player_Zankou_Skill2_Damage: 'Sanguine Dash',
  GE_Player_Zankou_Skill2_1_Damage: 'Sanguine Dash · Extra',
  GE_Player_Zankou_Skill3_Damage: 'Soulcross (Enhanced)',
  GE_Player_Zankou_Skill3_1_Damage: 'Soulcross (Enhanced) · Slash',
  GE_Player_Zankou_Skill3_2_Damage: 'Soulcross (Oblivion Finisher)',
  GE_Player_Zankou_Skill4_Damage: 'Soulcross',
  GE_Player_Zankou_Skill4_1_Damage: 'Soulcross · Slash',
  GE_Player_Zankou_Skill4_2_Damage: 'Soulcross · Finisher',

  // Zankou Ultra variants (Inferno Flamenco / Force / Bloodfeast Reverie)
  GE_Player_Zankou_UltraSkill1_Damage: 'Hit 1',
  GE_Player_Zankou_UltraSkill2_Damage: 'Finisher',
  GE_Player_Zankou_ForceUltraSkill1_Damage: 'Enhanced Hit 1',
  GE_Player_Zankou_ForceUltraSkill2_Damage: 'Enhanced Finisher',
  GE_Player_Zankou_MagicUltraSkill1_Damage: 'Bloodfeast Reverie (Flurry)',
  GE_Player_Zankou_MagicUltraSkill2_Damage: 'Bloodfeast Reverie (Finisher)',

  // Zankou Melee combos and branches
  GE_Player_Zankou_Melee1_Damage: 'Combo 1',
  GE_Player_Zankou_Melee1_1_Damage: 'Combo 1-1',
  GE_Player_Zankou_Melee2_Damage: 'Combo 2',
  GE_Player_Zankou_Melee3_Damage: 'Combo 3',
  GE_Player_Zankou_Melee3_1_Damage: 'Combo 3-1',
  GE_Player_Zankou_Melee3_2_Damage: 'Combo 3-2',
  GE_Player_Zankou_Melee4_Damage: 'Combo 4',
  GE_Player_Zankou_Melee4_1_Damage: 'Combo 4-1',
  GE_Player_Zankou_Melee4_2_Damage: 'Combo 4-2',
  GE_Player_Zankou_Melee4_3_Damage: 'Combo 4-3',
  GE_Player_Zankou_Melee4_4_Damage: 'Combo 4-4',
  GE_Player_Zankou_Melee5_Damage: 'Combo 5',
  GE_Player_Zankou_Melee5_1_Damage: 'Combo 5-1',
  GE_Player_Zankou_Melee5_2_Damage: 'Combo 5-2',
  GE_Player_Zankou_Melee6_Damage: 'Combo 6',
  GE_Player_Zankou_Branch1_Damage: 'Charged 1',
  GE_Player_Zankou_Branch2_Damage: 'Charged 2',
  GE_Player_Zankou_Branch3_Damage: 'Charged 3',
  GE_Player_Zankou_MagicBranch1_Damage: 'Illusion Charged 1',
  GE_Player_Zankou_MagicBranch2_Damage: 'Illusion Charged 2',
  GE_Player_Zankou_MagicBranch3_Damage: 'Illusion Charged 3',
  GE_Player_Zankou_MagicBranch4_Damage: 'Illusion Charged 4',
  GE_Player_Zankou_MagicMelee1_Damage: 'Illusion Combo 1',
  GE_Player_Zankou_MagicMelee2_Damage: 'Illusion Combo 2',
  GE_Player_Zankou_MagicMelee2_1_Damage: 'Illusion Combo 2-1',
  GE_Player_Zankou_MagicMelee2_2_Damage: 'Illusion Combo 2-2',
  GE_Player_Zankou_MagicMelee3_Damage: 'Illusion Combo 3',
  GE_Player_Zankou_MagicMelee3_1_Damage: 'Illusion Combo 3-1',
  GE_Player_Zankou_MagicMelee4_Damage: 'Illusion Combo 4',
  GE_Player_Zankou_MagicMelee4_1_Damage: 'Illusion Combo 4-1',

  // Weapon procs
  GE_Fork_serenity_Skill1_Damage1: 'Harsh Sound',
  GE_Fork_serenity_Skill2_Damage: 'Nagging Sound',
  GE_Fork_serenity_Skill2_Damage2: 'Resonance Finisher',
  GE_Fork_yaodao_Skill_Damage: 'Counter Slash',
  GE_Fork_BlackBook_damage: 'Chaos Thorn',

  // Linko / Radio072 summon
  GE_Xiaozhen_Skill1_Damage: 'Xiaozhen · Stage 1',
};

const cycles: Record<string, string> = {
  'ntesim.reaction.scorch': 'Scorch (DoT)',
  'ntesim.reaction.blossom': 'Blossom · Vita pistil',
  'ntesim.reaction.nova': 'Nova',
  'ntesim.reaction.hexed_followup': 'Hexed follow-up',
};

const singlePacketEffects = new Set([
  'GE_Player_Adler_AirAttack_Damage',
  'GE_Player_Adler_PerfectEvadeAttack_Damage',
  'GE_Player_Adler_QTE_Damage',
  'GE_Player_Adler_UltraSkill_Damage',
  'GE_Player_BlackBird_UltraSkill_Damage',
  'GE_Player_Cang_PerfectEvadeAttack_Damage',
  'GE_Player_Cang_QTE_Damage',
  'GE_Player_Cang_Skill_Damage',
  'GE_Player_Cang_SkillB_Damage',
  'GE_Player_Chaos_QTE1_Damage',
  'GE_Player_Chiichan073_QTE_Damage',
  'GE_Player_Daffodill_PerfectEvadeAttack_Damage',
  'GE_Player_Daffodill_QTE_Damage',
  'GE_Player_Edgar_PerfectEvadeAttack_Damage',
  'GE_Player_Edgar_QTE_Damage',
  'GE_Player_Edgar_Skill_Damage',
  'GE_Player_Edgar_UltraSkill_Damage',
  'GE_Player_Fadia_PerfectEvadeAttack_Damage',
  'GE_Player_Fadia_QTE1_Damage',
  'GE_Player_Haniel_QTE_Damage',
  'GE_Player_Jin_QTE_Damage',
  'GE_Player_Kuhara_QTE_Damage',
  'GE_Player_Lacrimosa_AirAttack_Damage',
  'GE_Player_Lacrimosa_QTE1_Damage',
  'GE_Player_Mint_QTE1_Damage',
  'GE_Player_Mint_Skill1_Damage_New',
  'GE_Player_Mint_PerfectEvadeAttack_Damage',
  'GE_Reaction_4_new_1070_Damage',
  'GE_Reaction_4_new_1042_Damage',
  'GE_Nanally010_Lv1_Damage',
  'GE_Nanally010_Lv3_Damage',
  'GE_Chaos071_Remora_Damage',
  'weapon:fork_PoliceRat:whisker_attack',
  'GE_Player_Mitsuki_PerfectEvadeAtk_Damage',
  'GE_Player_Mitsuki_PerfectAtkBullet_Damage',
  'GE_Player_Mitsuki_PerfectAtkBullet_Power_Damage',
  'GE_Player_Mitsuki_QTE_Damage',
  'GE_Player_Mitsuki_Skill_Damage',
  'GE_Player_Nanally_PerfectEvadeAttack_Damage',
  'GE_Player_Nanally_QTE1_Damage',
  'GE_Player_Nanally_Skill1_Damage',
  'GE_Player_Oneiroi_PerfectEvadeAttack_Damage',
  'GE_Player_Oneiroi_QTE_Damage',
  'GE_Player_Radio072_AirAttack_Damage',
  'GE_Player_Radio072_LingZhou_Damage',
  'GE_Player_Radio072_QTE_Damage',
  'GE_Player_Sagiri_PerfectEvadeAttack_Damage',
  'GE_Player_Sagiri_QTE1_Damage',
  'GE_Player_Shinku_QTE_Damage',
  'GE_Player_Skia_QTE1_Damage',
  'GE_Player_Zankou_PerfectEvadeAttack_Damage',
  'GE_Player_Zankou_QTE_Damage',
  'GE_Player_Lacrimosa_AnHunZhouTwo_Damage',
]);

const abilityTitles: Record<string, string> = {
  'ntesim.reaction': 'Reactions',
  GA_BlackBird_Passive_1: 'Thrushsong',
  GA_Mitsuki070_Passive_1: 'Harmonics',
  GA_Chaos_Passive_1: 'Justice in Time',
  GA_Cang_WorldPas_Fadia: 'Fadia Assist',
  'passive:nanally.fair_duel': 'Fair Duel',
  'passive:nanally.authority_a3': 'Authority (A3)',
  'weapon:fork_BlackBook:arc': 'Arc: Black Tome',
  'weapon:fork_jingmotingyuan:arc': 'Arc: Silent Garden',
  'weapon:fork_PoliceRat:arc': 'Arc: Officer Whisker',
  'weapon:fork_rishi:arc': 'Arc: Eclipse',
};

export function resolveAbilityTitle(abilityId: string): string {
  if (Object.hasOwn(abilityTitles, abilityId)) return abilityTitles[abilityId];
  if (abilityId === 'ntesim.break') return english.text('break_damage');
  const name = english.skill(abilityId);
  if (name !== abilityId) return name;
  if (abilityId.endsWith('_AirAttack') || abilityId.endsWith('_Airattack')) return 'Plunge';
  if (abilityId.includes('PerfectEvade') || abilityId.includes('ExtremEvadeAtk'))
    return 'Dodge Counter';
  if (abilityId.endsWith('_SkillA')) return 'Skill (Form A)';
  if (abilityId.endsWith('_SkillB')) return 'Skill (Form B)';
  const lower = abilityId.toLowerCase();
  if (lower === 'weapon:fork_blackbook:arc') return 'Arc: Black Tome';
  if (lower === 'weapon:fork_jingmotingyuan:arc') return 'Arc: Silent Garden';
  if (lower === 'weapon:fork_policerat:arc') return 'Arc: Officer Whisker';
  if (lower === 'weapon:fork_rishi:arc') return 'Arc: Eclipse';
  return abilityId;
}

export function humanizeEffect(effectId: string): string {
  if (Object.hasOwn(effects, effectId)) return effects[effectId];
  if (!effectId.startsWith('GE_')) return effectId;
  let s = effectId;
  s = s.replace(/^GE_(Player_)?[A-Za-z0-9]+_/, '');
  s = s.replace(/^GE_/, '');
  s = s.replace(/(_Damage|_Damage_New|Damage)$/, '');

  s = s.replace(/\bB_Melee/g, 'Bat Stance Hit ');
  s = s.replace(/\bMagicMelee/g, 'Illusion Hit ');
  s = s.replace(/\bMagicBranch/g, 'Illusion Branch ');
  s = s.replace(/\bCat_Melee/g, 'Cat Form Hit ');
  s = s.replace(/\bEvolve_Melee/g, 'Evolve Hit ');
  s = s.replace(/\bMelee(\d+)/g, 'Hit $1');
  s = s.replace(/\bBranch(\d+)/g, 'Branch $1');
  s = s.replace(/\bAirAttack(\d*)/g, 'Plunge $1');
  s = s.replace(/\bSkill(\d+)/g, 'Stage $1');
  s = s.replace(/\b(Utra|Ultra)Skill(\d*)/g, 'Ultimate $2');
  s = s.replace(/\bPerfectEvade(Attack)?/g, 'Dodge Counter');
  s = s.replace(/\bDot(Ultra)?/g, 'DoT');

  s = s.replace(/_/g, ' ').trim();
  s = s.replace(/(\b(?:Hit|Branch|Stage|Ultimate)\s+\d+)\s+(\d+)\b/g, '$1-$2');
  s = s.replace(/\s+/g, ' ');

  return s || effectId;
}

export function sourceLabel(source: { ability_id: string; effect_id: string }) {
  if (source.ability_id === 'ntesim.reaction' && Object.hasOwn(cycles, source.effect_id))
    return cycles[source.effect_id];
  if (source.ability_id === 'ntesim.break') return english.text('break_damage');

  const name = resolveAbilityTitle(source.ability_id);

  if (Object.hasOwn(effects, source.effect_id)) {
    const eff = effects[source.effect_id];
    if (name === source.ability_id) {
      return eff;
    }
    return `${name} · ${eff}`;
  }

  if (singlePacketEffects.has(source.effect_id)) {
    return name;
  }

  const effectName = humanizeEffect(source.effect_id);
  if (name === source.ability_id) {
    return `${name} · ${effectName}`;
  }
  return effectName ? `${name} · ${effectName}` : name;
}
