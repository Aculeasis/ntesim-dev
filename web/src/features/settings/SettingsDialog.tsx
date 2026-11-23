import { useState } from 'react';
import type { OptionValue } from '../../domain/contracts/display';
import type { CompiledOptions } from '../../domain/contracts/script';
import { Dialog } from '../../components/ui/Dialog';
import { errorMessage } from '../../components/ui/format';
import { optionInput } from './option-input';
const fields = [
  ['iteration', 'Iterations'],
  ['seed', 'Seed'],
  ['duration', 'Duration (seconds)'],
  ['start_fons', 'Starting party Fons'],
  ['stop', 'Stop policy'],
];
const choices: Record<string, [string, string][]> = {
  stop: [
    ['time', 'Time'],
    ['victory', 'All dummies defeated'],
  ],
};
export function SettingsDialog({
  options,
  onApply,
  onClose,
}: {
  options: CompiledOptions;
  onApply: (changes: Record<string, OptionValue>) => void;
  onClose: () => void;
}) {
  const [values, setValues] = useState<Record<string, string>>({
      iteration: String(options.trials),
      seed: String(options.seed),
      duration: options.duration_specified ? String(options.relative_duration_us / 1e6) : '',
      start_fons: options.start_fons === null ? '' : String(options.start_fons),
      stop: options.stop,
    }),
    [error, setError] = useState('');
  const apply = async () => {
    try {
      const changes = Object.fromEntries(
        await Promise.all(fields.map(async ([k]) => [k, await optionInput(k, values[k])])),
      );
      onApply(changes);
      onClose();
    } catch (error) {
      setError(errorMessage(error));
    }
  };
  return (
    <Dialog id="settings-dialog" title="Settings" onClose={onClose}>
      <div className="settings-grid">
        {fields.map(([k, label]) => (
          <label key={k}>
            {label}
            {choices[k] ? (
              <select
                id={'setting-' + k}
                value={values[k]}
                onChange={(e) => setValues({ ...values, [k]: e.target.value })}
              >
                {choices[k].map(([v, label]) => (
                  <option key={v} value={v}>
                    {label}
                  </option>
                ))}
              </select>
            ) : (
              <input
                type="number"
                id={'setting-' + k}
                min={k === 'iteration' ? 1 : 0}
                step={k === 'iteration' || k === 'seed' ? 1 : 'any'}
                value={values[k]}
                onChange={(e) => setValues({ ...values, [k]: e.target.value })}
              />
            )}
          </label>
        ))}
      </div>
      <p className="chart-note">
        Duration is optional when stopping after all targets are defeated.
      </p>
      <p id="settings-error" role="alert">
        {error}
      </p>
      <button id="apply-settings" className="run-button" onClick={apply}>
        Apply to config
      </button>
    </Dialog>
  );
}
