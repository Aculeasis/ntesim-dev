import type { ExactFraction } from '../contracts/scenario';
export interface Statistics {
  count: number;
  min: number;
  max: number;
  mean: number;
  std: number;
  p25: number;
  p50: number;
  p75: number;
}
const RATIONAL_PATTERN = /^(-?(?:0|[1-9]\d*))\/([1-9]\d*)$/;

export function fraction(value: ExactFraction): number {
  if (typeof value !== 'string') throw new Error('Missing or invalid exact fraction in report.');
  const match = RATIONAL_PATTERN.exec(value);
  if (!match) throw new Error('Missing or invalid exact fraction in report.');
  const numerator = Number(match[1]),
    denominator = Number(match[2]);
  const result = numerator / denominator;
  if (!(denominator > 0) || !Number.isFinite(result))
    throw new Error('Invalid exact fraction denominator or magnitude.');
  return result;
}
