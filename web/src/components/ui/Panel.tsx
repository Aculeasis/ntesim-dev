import type { ReactNode } from 'react';
export function Panel({
  title,
  titleControl,
  controls,
  children,
}: {
  title?: string;
  titleControl?: ReactNode;
  controls?: ReactNode;
  children: ReactNode;
}) {
  const hasHeading = Boolean(title || titleControl || controls);
  return (
    <section className="panel">
      {hasHeading && (
        <div className="panel-heading">
          {titleControl ? (
            <div className="panel-title-group">
              {title ? <h2>{title}</h2> : null}
              {titleControl}
            </div>
          ) : title ? (
            <h2>{title}</h2>
          ) : null}
          {controls}
        </div>
      )}
      {children}
    </section>
  );
}
export function Select({
  label,
  value,
  onChange,
  options,
  id,
}: {
  label: string;
  value: string;
  onChange: (value: string) => void;
  options: [string, string][];
  id?: string;
}) {
  return (
    <label>
      {label}{' '}
      <select id={id} value={value} onChange={(e) => onChange(e.target.value)}>
        {options.map(([value, text]) => (
          <option key={value} value={value}>
            {text}
          </option>
        ))}
      </select>
    </label>
  );
}
