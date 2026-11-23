import type { ActorInspection } from '../../domain/contracts/script';
import { Icon } from '../../components/ui/Artwork';
import { statRows } from './stat-rows';

export function CharacterStats({
  actor,
  mode = 'total',
}: {
  actor: ActorInspection;
  mode?: string;
}) {
  return (
    <div className="total-stats">
      <strong>{mode === 'equipment' ? 'Build contributions' : 'Stats'}</strong>
      {mode === 'equipment' ? (
        <table aria-label="Build contributions">
          <thead>
            <tr>
              <th>Source</th>
              <th>Attribute</th>
              <th>Exact amount</th>
            </tr>
          </thead>
          <tbody>
            {actor.build_decomposition.contributions.map((c, i) => (
              <tr key={i}>
                <td>{c.source}</td>
                <td>{c.attribute_id}</td>
                <td>{c.amount}</td>
              </tr>
            ))}
          </tbody>
        </table>
      ) : (
        <table aria-label="Character stats and initial resources">
          <tbody>
            {statRows(actor).map(([icon, label, value, exact]) => (
              <tr className="stat-row" tabIndex={0} key={label} title={`Exact: ${exact}`}>
                <td>
                  <Icon name={icon} />
                  {label}
                </td>
                <td>{value}</td>
              </tr>
            ))}
          </tbody>
        </table>
      )}
      {actor.unmodeled_attribute_ids.length > 0 && (
        <p className="unavailable">
          Unmodeled attributes: {actor.unmodeled_attribute_ids.join(', ')}
        </p>
      )}
      <details>
        <summary>Exact build and initial resources</summary>
        <pre>{JSON.stringify(actor, null, 2)}</pre>
      </details>
    </div>
  );
}
