export const INPUT_ACTION_ROLES = [
  'normal',
  'redirect_skill',
  'ultimate',
  'qte',
  'critical_riposte',
  'parry_attack',
  'other',
] as const;

export type InputActionRole = (typeof INPUT_ACTION_ROLES)[number];
