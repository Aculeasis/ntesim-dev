import assert from 'node:assert/strict';
import {
  sourceLabel,
  humanizeEffect,
  resolveAbilityTitle,
} from '../src/domain/reports/source-label';
import { energySourceDetails } from '../src/domain/reports/energy-sources';

// Reaction cycles
assert.equal(
  sourceLabel({ ability_id: 'ntesim.reaction', effect_id: 'ntesim.reaction.scorch' }),
  'Scorch (DoT)',
);
assert.equal(
  sourceLabel({ ability_id: 'ntesim.reaction', effect_id: 'ntesim.reaction.blossom' }),
  'Blossom · Vita pistil',
);
assert.equal(
  sourceLabel({ ability_id: 'ntesim.reaction', effect_id: 'ntesim.reaction.nova' }),
  'Nova',
);
assert.equal(
  sourceLabel({ ability_id: 'ntesim.reaction', effect_id: 'ntesim.reaction.hexed_followup' }),
  'Hexed follow-up',
);
assert.equal(sourceLabel({ ability_id: 'ntesim.break', effect_id: '' }), 'Break');

// Zankou DoTs
assert.equal(
  sourceLabel({ ability_id: 'GA_Zankou_Melee', effect_id: 'GE_Player_Zankou_DotDamage' }),
  'Wildfire · Heartwrench (DoT)',
);
assert.equal(
  sourceLabel({ ability_id: 'GA_Zankou_UltraSkill', effect_id: 'GE_Player_Zankou_DotUltraDamage' }),
  'Inferno Flamenco · Vile Ash (DoT)',
);

// Zankou Skill variants
assert.equal(
  sourceLabel({ ability_id: 'GA_Zankou_Skill', effect_id: 'GE_Player_Zankou_Skill1_Damage' }),
  'Sanguine Dash · Sanguine Dash (Enhanced)',
);
assert.equal(
  sourceLabel({ ability_id: 'GA_Zankou_Skill', effect_id: 'GE_Player_Zankou_Skill2_Damage' }),
  'Sanguine Dash · Sanguine Dash',
);
assert.equal(
  sourceLabel({ ability_id: 'GA_Zankou_Skill', effect_id: 'GE_Player_Zankou_Skill3_Damage' }),
  'Sanguine Dash · Soulcross (Enhanced)',
);
assert.equal(
  sourceLabel({ ability_id: 'GA_Zankou_Skill', effect_id: 'GE_Player_Zankou_Skill3_2_Damage' }),
  'Sanguine Dash · Soulcross (Oblivion Finisher)',
);
assert.equal(
  sourceLabel({ ability_id: 'GA_Zankou_Skill', effect_id: 'GE_Player_Zankou_Skill4_Damage' }),
  'Sanguine Dash · Soulcross',
);

// Zankou Ultra variants
assert.equal(
  sourceLabel({
    ability_id: 'GA_Zankou_UltraSkill',
    effect_id: 'GE_Player_Zankou_UltraSkill1_Damage',
  }),
  'Inferno Flamenco · Hit 1',
);
assert.equal(
  sourceLabel({
    ability_id: 'GA_Zankou_UltraSkill',
    effect_id: 'GE_Player_Zankou_UltraSkill2_Damage',
  }),
  'Inferno Flamenco · Finisher',
);
assert.equal(
  sourceLabel({
    ability_id: 'GA_Zankou_UltraSkill',
    effect_id: 'GE_Player_Zankou_ForceUltraSkill1_Damage',
  }),
  'Inferno Flamenco · Enhanced Hit 1',
);
assert.equal(
  sourceLabel({
    ability_id: 'GA_Zankou_UltraSkill',
    effect_id: 'GE_Player_Zankou_MagicUltraSkill1_Damage',
  }),
  'Inferno Flamenco · Bloodfeast Reverie (Flurry)',
);
assert.equal(
  sourceLabel({
    ability_id: 'GA_Zankou_UltraSkill',
    effect_id: 'GE_Player_Zankou_MagicUltraSkill2_Damage',
  }),
  'Inferno Flamenco · Bloodfeast Reverie (Finisher)',
);

// Zankou Melee combos and branches
assert.equal(
  sourceLabel({ ability_id: 'GA_Zankou_Melee', effect_id: 'GE_Player_Zankou_Melee1_Damage' }),
  'Wildfire · Combo 1',
);
assert.equal(
  sourceLabel({ ability_id: 'GA_Zankou_Melee', effect_id: 'GE_Player_Zankou_Melee4_1_Damage' }),
  'Wildfire · Combo 4-1',
);
assert.equal(
  sourceLabel({ ability_id: 'GA_Zankou_Melee', effect_id: 'GE_Player_Zankou_Branch1_Damage' }),
  'Wildfire · Charged 1',
);
assert.equal(
  sourceLabel({ ability_id: 'GA_Zankou_Melee', effect_id: 'GE_Player_Zankou_MagicMelee1_Damage' }),
  'Wildfire · Illusion Combo 1',
);

// Curated Lacrimosa & Oneiroi & Zero
assert.equal(
  sourceLabel({ ability_id: 'GA_Lacrimosa_Skill', effect_id: 'GE_Player_Lacrimosa_Blood_Damage' }),
  'Morning Tomato · Nightmare (DoT)',
);
assert.equal(
  sourceLabel({
    ability_id: 'GA_Lacrimosa_Melee',
    effect_id: 'GE_Player_Lacrimosa_B_Melee3_Explode_Damage',
  }),
  'Sweet and Sour · Shatter',
);
assert.equal(
  sourceLabel({ ability_id: 'GA_Oneiroi_Skill', effect_id: 'GE_Player_Oneiroi_Skill_Gun_Damage' }),
  'Future Self Continuity · Gun lamb',
);
assert.equal(
  sourceLabel({ ability_id: 'GA_Female046_Skill', effect_id: 'GE_Player_Female051_Skill4_Damage' }),
  'Appraise and Engrave · Skill hit',
);

// Weapon procs
assert.equal(
  sourceLabel({
    ability_id: 'weapons.jingmotingyuan',
    effect_id: 'GE_Fork_serenity_Skill1_Damage1',
  }),
  'Harsh Sound',
);
assert.equal(
  sourceLabel({ ability_id: 'weapons.rose', effect_id: 'GE_Fork_BlackBook_damage' }),
  'Chaos Thorn',
);
assert.equal(
  sourceLabel({
    ability_id: 'weapon:fork_BlackBook:arc',
    effect_id: 'GE_Fork_BlackBook_damage',
  }),
  'Arc: Black Tome · Chaos Thorn',
);
assert.equal(
  sourceLabel({ ability_id: 'weapons.yaodao', effect_id: 'GE_Fork_yaodao_Skill_Damage' }),
  'Counter Slash',
);

// Single-packet actions preserve clean ability titles
assert.equal(
  sourceLabel({
    ability_id: 'GA_lacrimosa_Passive_1',
    effect_id: 'GE_Player_Lacrimosa_AnHunZhouTwo_Damage',
  }),
  'Tomato Banquet',
);
assert.equal(
  sourceLabel({
    ability_id: 'GA_BlackBird_Passive_1',
    effect_id: 'GE_Reaction_4_new_1042_Damage',
  }),
  'Thrushsong',
);
assert.equal(
  sourceLabel({
    ability_id: 'passive:nanally.fair_duel',
    effect_id: 'GE_Nanally010_Lv1_Damage',
  }),
  'Fair Duel',
);
assert.equal(
  sourceLabel({
    ability_id: 'passive:nanally.authority_a3',
    effect_id: 'GE_Nanally010_Lv3_Damage',
  }),
  'Authority (A3)',
);
assert.equal(
  sourceLabel({
    ability_id: 'GA_Mitsuki070_Passive_1',
    effect_id: 'GE_Reaction_4_new_1070_Damage',
  }),
  'Harmonics',
);
assert.equal(
  sourceLabel({
    ability_id: 'GA_Chaos_Passive_1',
    effect_id: 'GE_Chaos071_Remora_Damage',
  }),
  'Justice in Time',
);
assert.equal(
  sourceLabel({
    ability_id: 'weapon:fork_PoliceRat:arc',
    effect_id: 'weapon:fork_PoliceRat:whisker_attack',
  }),
  'Arc: Officer Whisker',
);
assert.equal(
  sourceLabel({
    ability_id: 'GA_Adler_UltraSkill',
    effect_id: 'GE_Player_Adler_UltraSkill_Damage',
  }),
  'Tranquility',
);
assert.equal(
  sourceLabel({
    ability_id: 'GA_Zankou_QTE',
    effect_id: 'GE_Player_Zankou_QTE_Damage',
  }),
  'Stoked Flame',
);

// Secondary actions
assert.equal(
  sourceLabel({
    ability_id: 'GA_Zankou_PerfectEvade',
    effect_id: 'GE_Player_Zankou_PerfectEvadeAttack_Damage',
  }),
  'Dodge Counter',
);
assert.equal(
  sourceLabel({
    ability_id: 'GA_Lacrimosa_AirAttack',
    effect_id: 'GE_Player_Lacrimosa_AirAttack_Damage',
  }),
  'Plunge',
);
assert.equal(resolveAbilityTitle('GA_Cang_SkillA'), 'Skill (Form A)');
assert.equal(resolveAbilityTitle('ntesim.reaction'), 'Reactions');
assert.equal(resolveAbilityTitle('ntesim.break'), 'Break');
assert.equal(resolveAbilityTitle('weapon:fork_BlackBook:arc'), 'Arc: Black Tome');
assert.equal(resolveAbilityTitle('weapon:fork_jingmotingyuan:arc'), 'Arc: Silent Garden');
assert.equal(resolveAbilityTitle('weapon:fork_PoliceRat:arc'), 'Arc: Officer Whisker');
assert.equal(resolveAbilityTitle('weapon:fork_rishi:arc'), 'Arc: Eclipse');
assert.equal(resolveAbilityTitle('GA_BlackBird_Passive_1'), 'Thrushsong');
assert.equal(resolveAbilityTitle('passive:nanally.fair_duel'), 'Fair Duel');
assert.equal(resolveAbilityTitle('passive:nanally.authority_a3'), 'Authority (A3)');
assert.equal(resolveAbilityTitle('GA_Mitsuki070_Passive_1'), 'Harmonics');
assert.equal(resolveAbilityTitle('GA_Chaos_Passive_1'), 'Justice in Time');

// Unmapped and humanization fallback
assert.equal(
  sourceLabel({ ability_id: 'unmapped', effect_id: 'distinct_packet' }),
  'unmapped · distinct_packet',
);
assert.equal(
  sourceLabel({ ability_id: 'GA_Mint019_Melee', effect_id: 'GE_Player_Mint_Melee1_Damage_New' }),
  'Perfect Containment · Hit 1',
);
assert.equal(
  sourceLabel({
    ability_id: 'GA_BlackBird_Evolve_Melee',
    effect_id: 'GE_Player_BlackBird_Evolve_Melee4_Damage',
  }),
  'Fleeting Shadow · Evolve Hit 4',
);
assert.equal(
  sourceLabel({
    ability_id: 'GA_Radio072_Skill',
    effect_id: 'GE_Xiaozhen_Skill1_Damage',
  }),
  'Full-Frequency Pulse · Xiaozhen · Stage 1',
);
assert.equal(humanizeEffect('GE_Player_Nanally_Cat_Melee2_Damage'), 'Cat Form Hit 2');

// Energy source details & specialized packet resolution
assert.equal(energySourceDetails('iroi_hit', 'iroi_riposte_energy_grant').label, 'Iroi Riposte');
assert.equal(energySourceDetails('iroi_hit', 'iroi_plunge_energy_grant').label, 'Iroi Plunge');
assert.equal(
  energySourceDetails('sakiri_hit', 'sakiri_riposte_energy_grant').label,
  'Sakiri Riposte',
);
assert.equal(
  energySourceDetails('sakiri_hit', 'sakiri_support_packet_energy_grant').label,
  'Sakiri Support packet',
);
assert.equal(
  energySourceDetails('daffodil_hit', 'daffodil_support_energy').label,
  'Daffodil Support',
);
assert.equal(energySourceDetails('mint_hit', 'mint_riposte_energy_grant').label, 'Mint Riposte');
assert.equal(energySourceDetails('adler_hit', 'adler_riposte_energy_grant').label, 'Adler Riposte');
assert.equal(
  energySourceDetails('iroi_hit', 'iroi_ordinary_hit_energy_grant').label,
  'Iroi ordinary hit',
);
assert.equal(energySourceDetails('sakiri_hit').label, 'Sakiri ordinary hit');

console.log('Source labels: all curated mappings, DoTs, procs, and fallbacks passed.');
