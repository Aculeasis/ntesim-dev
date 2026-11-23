import { ARTWORK } from '../../domain/artwork';
const symbols: Record<string, string> = {
  hp: 'M12 3C9 8 5 11 5 16a7 7 0 0 0 14 0c0-5-4-8-7-13Z',
  atk: 'm4 20 4-4m-2-2 4 4m-1-7L18 3l3 3-8 9Z',
  def: 'M12 2 21 6v6c0 6-9 10-9 10S3 18 3 12V6Z',
  cr: 'm12 2 3 7 7 1-5 5 1 7-6-4-6 4 1-7-5-5 7-1Z',
  cd: 'm12 2 9 5v10l-9 5-9-5V7ZM3 7l18 10M21 7 3 17M12 2v20',
  energy: 'm14 2-9 12h7l-2 8 9-12h-7Z',
  dmg: 'M12 3 5 14l7 8 7-8Z',
  cartridge: 'M4 4h16v16H4Zm4 4v8h8V8Z',
  weapon: 'm5 20 4-4m-2-2 4 4m-1-7L18 3l3 3-8 9Z',
};
export function Icon({ name }: { name: string }) {
  return (
    <svg className="stat-icon" viewBox="0 0 24 24" aria-hidden="true">
      <path d={symbols[name] || symbols.dmg} />
    </svg>
  );
}
export function Artwork({
  category,
  id,
  alt,
  className = '',
}: {
  category: string;
  id?: string | null;
  alt: string;
  className?: string;
}) {
  const files = Object.hasOwn(ARTWORK, category) ? ARTWORK[category] : undefined;
  const file = files && Object.hasOwn(files, id ?? '') ? files[id ?? ''] : undefined;
  return file ? (
    <img
      className={className}
      src={`${import.meta.env?.BASE_URL ?? './'}assets/artwork/${file}`}
      alt={alt}
      loading="lazy"
    />
  ) : (
    <span className={`asset-placeholder ${className}`} role="img" aria-label={alt}>
      <Icon name={category === 'weapons' ? 'weapon' : 'cartridge'} />
    </span>
  );
}
