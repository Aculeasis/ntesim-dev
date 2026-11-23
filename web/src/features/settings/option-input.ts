import type { OptionValue } from '../../domain/contracts/display';
import { loadEngine } from '../../infrastructure/wasm/load';

export async function optionInput(name: string, value: string): Promise<OptionValue> {
  return (await loadEngine()).parseOptionInput(name, value);
}
