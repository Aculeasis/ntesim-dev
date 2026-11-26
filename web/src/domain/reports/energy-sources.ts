import type { EnergyGrant } from '../contracts/report';

interface EnergySourceDetails {
  kind: string;
  label: string;
}
const sources: Record<EnergyGrant['source'], EnergySourceDetails> = {
  mind_royale: {
    kind: 'mind_royale_energy',
    label: 'Mind Royale',
  },
  mint_a6: {
    kind: 'mint_a6_energy',
    label: 'Mint A6 energy',
  },
  mint_hit: {
    kind: 'mint_hit_energy',
    label: 'Mint ordinary hit',
  },
  sakiri_hit: {
    kind: 'sakiri_hit_energy',
    label: 'Sakiri ordinary hit',
  },
  zero_hit: {
    kind: 'zero_hit_energy',
    label: 'Zero ordinary hit',
  },
  iroi_hit: {
    kind: 'iroi_hit_energy',
    label: 'Iroi ordinary hit',
  },
  nanally_hit: {
    kind: 'nanally_hit_energy',
    label: 'Nanally ordinary hit',
  },
  chaos_hit: {
    kind: 'chaos_hit_energy',
    label: 'Chaos ordinary hit',
  },
  edgar_hit: {
    kind: 'edgar_hit_energy',
    label: 'Edgar ordinary hit',
  },
  hotori_hit: {
    kind: 'hotori_hit_energy',
    label: 'Hotori ordinary hit',
  },
  daffodil_hit: {
    kind: 'daffodil_hit_energy',
    label: 'Daffodil ordinary hit',
  },
  linko_hit: {
    kind: 'linko_hit_energy',
    label: 'Linko ordinary hit',
  },
  skia_hit: {
    kind: 'skia_hit_energy',
    label: 'Skia ordinary hit',
  },
  hathor_hit: {
    kind: 'hathor_hit_energy',
    label: 'Hathor ordinary hit',
  },
  hathor_power: { kind: 'hathor_power_energy', label: 'Hathor Power' },
  jiuyuan_bud_boom: { kind: 'jiuyuan_bud_boom_energy', label: 'Jiuyuan Bud Boom' },
  fadia_threshold: { kind: 'fadia_threshold_energy', label: 'Fadia HP threshold' },
  blackbird_hit: {
    kind: 'blackbird_hit_energy',
    label: 'Blackbird ordinary hit',
  },
  aurelia_hit: {
    kind: 'aurelia_hit_energy',
    label: 'Aurelia ordinary hit',
  },
  haniel_hit: {
    kind: 'haniel_hit_energy',
    label: 'Haniel ordinary hit',
  },
  jiuyuan_hit: {
    kind: 'jiuyuan_hit_energy',
    label: 'Jiuyuan ordinary hit',
  },
  baicang_hit: {
    kind: 'baicang_hit_energy',
    label: 'Baicang ordinary hit',
  },
  fadia_hit: {
    kind: 'fadia_hit_energy',
    label: 'Fadia ordinary hit',
  },
  zankou_hit: {
    kind: 'zankou_hit_energy',
    label: 'Zankou ordinary hit',
  },
  lacrimosa_hit: {
    kind: 'lacrimosa_hit_energy',
    label: 'Lacrimosa ordinary hit',
  },
  shinku_hit: {
    kind: 'shinku_hit_energy',
    label: 'Shinku ordinary hit',
  },
  akane_hit: {
    kind: 'akane_hit_energy',
    label: 'Akane ordinary hit',
  },
  elyms_hit: {
    kind: 'elyms_hit_energy',
    label: 'Elyms ordinary hit',
  },
  adler_hit: {
    kind: 'adler_hit_energy',
    label: 'Adler ordinary hit',
  },
  chiz_hit: {
    kind: 'chiz_hit_energy',
    label: 'Chiz ordinary hit',
  },
  zero_a2: {
    kind: 'zero_a2_energy_grant',
    label: 'Zero A2 energy',
  },
  nanally_a1: {
    kind: 'nanally_a1_energy_grant',
    label: 'Nanally A1 energy',
  },
  sakiri_a5: {
    kind: 'sakiri_a5_reset',
    label: 'Sakiri A5 restoration',
  },
  adler_a5: {
    kind: 'adler_a5_energy_refund',
    label: 'Adler A5 energy refund',
  },
  rishi_eclipse: {
    kind: 'rishi_eclipse_energy',
    label: 'Rishi Eclipse defeat',
  },
  reaction_charge: {
    kind: 'reaction_charge_energy',
    label: 'Charge reaction',
  },
};

const directPacketSources: Record<
  string,
  { source: EnergyGrant['source']; kind: string; label: string }
> = {
  zankou_skill_direct_packet_energy_grant: {
    source: 'zankou_hit',
    kind: 'zankou_skill_direct_packet_energy',
    label: 'Zankou Skill packet',
  },
  chaos_qte_direct_packet_energy_grant: {
    source: 'chaos_hit',
    kind: 'chaos_direct_packet_energy',
    label: 'Chaos QTE direct packet',
  },
  hathor_direct_packet_energy_grant: {
    source: 'hathor_hit',
    kind: 'hathor_direct_packet_energy',
    label: 'Hathor direct packet',
  },
  daffodil_direct_packet_energy_grant: {
    source: 'daffodil_hit',
    kind: 'daffodil_direct_packet_energy',
    label: 'Daffodil direct packet',
  },
  aurelia_direct_packet_energy_grant: {
    source: 'aurelia_hit',
    kind: 'aurelia_direct_packet_energy',
    label: 'Aurelia direct packet',
  },
  sakiri_direct_packet_energy_grant: {
    source: 'sakiri_hit',
    kind: 'sakiri_direct_packet_energy',
    label: 'Sakiri direct packet',
  },
  iroi_riposte_energy_grant: {
    source: 'iroi_hit',
    kind: 'iroi_riposte_energy',
    label: 'Iroi Riposte',
  },
  sakiri_riposte_energy_grant: {
    source: 'sakiri_hit',
    kind: 'sakiri_riposte_energy',
    label: 'Sakiri Riposte',
  },
  mint_riposte_energy_grant: {
    source: 'mint_hit',
    kind: 'mint_riposte_energy',
    label: 'Mint Riposte',
  },
  nanally_riposte_energy_grant: {
    source: 'nanally_hit',
    kind: 'nanally_riposte_energy',
    label: 'Nanally Riposte',
  },
  adler_riposte_energy_grant: {
    source: 'adler_hit',
    kind: 'adler_riposte_energy',
    label: 'Adler Riposte',
  },
  aurelia_riposte_energy_grant: {
    source: 'aurelia_hit',
    kind: 'aurelia_riposte_energy',
    label: 'Aurelia Riposte',
  },
  iroi_plunge_energy_grant: {
    source: 'iroi_hit',
    kind: 'iroi_plunge_energy',
    label: 'Iroi Plunge',
  },
  adler_plunge_energy_grant: {
    source: 'adler_hit',
    kind: 'adler_plunge_energy',
    label: 'Adler Plunge',
  },
  sakiri_support_packet_energy_grant: {
    source: 'sakiri_hit',
    kind: 'sakiri_support_energy',
    label: 'Sakiri Support packet',
  },
  daffodil_support_energy: {
    source: 'daffodil_hit',
    kind: 'daffodil_support_energy',
    label: 'Daffodil Support',
  },
  nanally_support_packet_energy_grant: {
    source: 'nanally_hit',
    kind: 'nanally_support_energy',
    label: 'Nanally Support',
  },
  zero_support_packet_energy_grant: {
    source: 'zero_hit',
    kind: 'zero_support_energy',
    label: 'Zero Support',
  },
  mint_support_energy: {
    source: 'mint_hit',
    kind: 'mint_support_energy',
    label: 'Mint Support',
  },
  chiz_support_energy: {
    source: 'chiz_hit',
    kind: 'chiz_support_energy',
    label: 'Chiz Support',
  },
  blackbird_support_energy_grant: {
    source: 'blackbird_hit',
    kind: 'blackbird_support_energy',
    label: 'Blackbird Support',
  },
  blackbird_evolved_support_energy_grant: {
    source: 'blackbird_hit',
    kind: 'blackbird_evolved_support_energy',
    label: 'Blackbird Support (Evolved)',
  },
  blackbird_secondary_energy_grant: {
    source: 'blackbird_hit',
    kind: 'blackbird_secondary_energy',
    label: 'Blackbird Secondary',
  },
  blackbird_evolved_secondary_energy_grant: {
    source: 'blackbird_hit',
    kind: 'blackbird_evolved_secondary_energy',
    label: 'Blackbird Secondary (Evolved)',
  },
  chaos_secondary_energy_grant: {
    source: 'chaos_hit',
    kind: 'chaos_secondary_energy',
    label: 'Chaos Secondary',
  },
  edgar_secondary_energy: {
    source: 'edgar_hit',
    kind: 'edgar_secondary_energy',
    label: 'Edgar Support',
  },
  haniel_secondary_energy: {
    source: 'haniel_hit',
    kind: 'haniel_secondary_energy',
    label: 'Haniel Support',
  },
  hathor_secondary_energy: {
    source: 'hathor_hit',
    kind: 'hathor_secondary_energy',
    label: 'Hathor Support',
  },
  hathor_airattack_energy: {
    source: 'hathor_hit',
    kind: 'hathor_airattack_energy',
    label: 'Hathor Air Attack',
  },
  iroi_skill_carrier_energy: {
    source: 'iroi_hit',
    kind: 'iroi_skill_carrier_energy',
    label: 'Iroi Skill carrier',
  },
  baicang_fadia_energy_grant: {
    source: 'baicang_hit',
    kind: 'baicang_fadia_energy',
    label: 'Baicang Fadia Assist',
  },
  chiz_no_grain_energy_grant: {
    source: 'chiz_hit',
    kind: 'chiz_no_grain_energy',
    label: 'Chiz Skill packet (No Grain)',
  },
  chiz_skill_energy_grant: {
    source: 'chiz_hit',
    kind: 'chiz_skill_energy',
    label: 'Chiz Skill packet',
  },
};

export function energySourceDetails(source: EnergyGrant['source'], eventKind?: string) {
  if (!Object.hasOwn(sources, source)) throw new Error('Unknown typed energy source.');
  const directPacket = eventKind ? directPacketSources[eventKind] : undefined;
  if (!directPacket) return sources[source];
  if (directPacket.source !== source) throw new Error('Energy event kind and source do not match.');
  return { ...sources[source], kind: directPacket.kind, label: directPacket.label };
}
