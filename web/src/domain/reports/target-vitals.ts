import type { HitReceipt, ScenarioReport } from '../contracts/report';
import type { ScenarioRequest } from '../contracts/scenario';

export function createTargetVitalsValidator(request: ScenarioRequest) {
  const targets = new Map(
    request.targets.map((target) => [
      target.target_index,
      { hp: target.hp_current, max: target.hp_max },
    ]),
  );
  let previousTime = 0;
  return {
    settle(hit: HitReceipt, damage: { numerator: bigint; denominator: bigint }) {
      const target = targets.get(hit.target_index);
      if (
        !target ||
        !Number.isSafeInteger(hit.time_us) ||
        hit.time_us < previousTime ||
        !Number.isSafeInteger(target.hp) ||
        target.hp < 0 ||
        damage.numerator < 0n ||
        damage.denominator <= 0n
      )
        throw new Error('Invalid target HP settlement receipt or delivery order.');
      if (
        (typeof hit.rounded_damage === 'number' && !Number.isSafeInteger(hit.rounded_damage)) ||
        (typeof hit.rounded_damage !== 'number' && typeof hit.rounded_damage !== 'string') ||
        !/^(?:0|[1-9]\d*)$/.test(String(hit.rounded_damage))
      )
        throw new Error('Invalid report rounded damage integer.');
      const rounded = BigInt(hit.rounded_damage);
      if (
        rounded < 0n ||
        rounded > (1n << 64n) - 1n ||
        !Number.isSafeInteger(hit.applied_hp_loss) ||
        hit.applied_hp_loss < 0 ||
        hit.applied_hp_loss > target.hp ||
        BigInt(hit.applied_hp_loss) > rounded
      )
        throw new Error('Invalid target HP loss observation.');
      // Rust owns quantization and overkill; replay reconciles recorded pool changes.
      target.hp -= hit.applied_hp_loss;
      previousTime = hit.time_us;
      return rounded;
    },
    finish(states: ScenarioReport['final_target_states']) {
      for (const state of states) {
        const target = targets.get(state.target_index);
        if (
          !target ||
          !Number.isSafeInteger(state.hp_current) ||
          state.hp_current !== target.hp ||
          state.hp_max !== target.max
        )
          throw new Error('Final target HP does not reconcile with the hit receipt ledger.');
      }
    },
  };
}
