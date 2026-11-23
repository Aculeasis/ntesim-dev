export const palette = ['#ca7ead', '#eadb37', '#6eafe2', '#65cdb6'];
export const elementColors: Record<string, string> = {
  Anima: '#62d8b1',
  Incantation: '#f45e88',
  Cosmos: '#e4e8d5',
  Chaos: '#a992e8',
  Lakshana: '#ecdb68',
  Psyche: '#64c4e5',
};
export const format = (value: number | null | undefined, digits = 0) =>
  value === null || value === undefined
    ? '—'
    : value.toLocaleString('en', { maximumFractionDigits: digits });
export const errorMessage = (value: unknown) =>
  value instanceof Error ? value.message : String(value);
