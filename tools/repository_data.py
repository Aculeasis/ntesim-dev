"""Review budgets for committed inputs; inventories and raw acquisitions belong in cache."""
from __future__ import annotations

from collections import defaultdict
import hashlib
import json
import re
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]
MAX_FILE_BYTES = 2_000_000
MAX_DATA_BYTES = 24_000_000
ARCHIVES = {'.zip', '.7z', '.rar', '.pak', '.utoc', '.ucas', '.uasset', '.umap', '.pdb', '.exe', '.dll', '.wasm', '.log', '.html', '.har'}


def inspect(root, names):
    rows = []
    assets = defaultdict(list)
    for name in sorted(set(names)):
        path = root / name
        if not path.exists():
            continue  # tracked deletions awaiting staging
        if path.is_symlink() or getattr(path, 'is_junction', lambda: False)() or not path.resolve().is_relative_to(root.resolve()):
            raise ValueError('unsafe repository input: ' + name)
        raw = path.read_bytes()
        if name.startswith(('data/combat/', 'data/extracted/', 'data/combat-targets/')):
            raise ValueError('source-only catalogs belong in cache; runtime constants must be typed Rust: ' + name)
        if name.startswith('data/') and (path.name.endswith('EffectFigure.json')
                or re.fullmatch(r'\d+AwakenEffect\d*\.json', path.name) or path.name in {
                'DT_Character.json', 'DT_PlayerPackData.json', 'DT_PlayerModifyPackData.json',
                'DT_SkillTeamChargeRelationData.json',
                'DT_GameplayAbilityCostData.json',
                'DT_GameplayAbilityTipsData.json',
                'DT_CharacterAbilityEffectConfig.json',
                'DT_GameplayEffectMappingData.json',
                'DT_EquipmentModifyData.json',
                'CT_Equipmentbuff.json',
                'mechanics.json',
                'DT_CharacterAbilityConfig.json',
                'DT_EquipmentSuitData.json',
                'DT_ForkUpgradeData.json',
                'DT_ForkBreakthroughData.json',
                'DT_ForkModifyData.json',
                'DT_ForkItemData.json',
                'CT_ForkBuff.json',
                'DT_ForkUpgradeStarDataTable.json',
                'DT_SkillDamageData.json',
                'DT_CharacterUpgradeDataTable.json', 'DT_CharacterBreakthroughDataTable.json'}):
            raise ValueError('retired runtime exports belong in cache; consume typed Rust definitions: ' + name)
        if name.startswith('data/') and path.name in {
                'DT_CharacterEquipmentSlotsData.json', 'DT_EquipmentModifySlotsEffect.json',
                'DT_EquipmentShapeFeatureData.json', 'DT_Equipment.json', 'CT_EquipmentBaseAttribute.json'}:
            raise ValueError('console geometry/allocation catalogs are forbidden; author console stats in add stats: ' + name)
        if name.startswith('data/') and any(token in path.name.casefold() for token in
                ('monster', 'enemy', 'spawn_pool', 'target_profile')):
            raise ValueError('enemy catalogs are forbidden; author dummy stats in the script: ' + name)
        if path.suffix.lower() in ARCHIVES and name != 'web/index.html':
            raise ValueError('raw acquisition/build artifact belongs in cache: ' + name)
        if '/evidence/' in name and name.startswith('data/'):
            raise ValueError('raw evidence belongs in cache: ' + name)
        if len(raw) > MAX_FILE_BYTES:
            raise ValueError('file exceeds review budget; project fields or factor explicit repeats: ' + name)
        if name.startswith('web/assets/') and path.suffix in ('.webp', '.png', '.jpg'):
            assets[hashlib.sha256(raw).hexdigest()].append(name)
        if name.startswith('data/') and path.suffix == '.json':
            value = json.loads(raw)
            if isinstance(value, list) and any(isinstance(v, dict) and ('Properties' in v or ('Rows' in v and 'Name' in v)) for v in value):
                raise ValueError('full Unreal export belongs in cache: ' + name)
        rows.append({'path': name, 'bytes': len(raw), 'lines': raw.count(b'\n') if b'\0' not in raw else None})
    data_bytes = sum(r['bytes'] for r in rows if r['path'].startswith('data/'))
    if data_bytes > MAX_DATA_BYTES:
        raise ValueError('committed data exceeds review budget')
    for duplicates in assets.values():
        if len(duplicates) > 1:
            raise ValueError('duplicate UI assets: ' + ', '.join(duplicates))
    return {'files': len(rows), 'bytes': sum(r['bytes'] for r in rows), 'data_bytes': data_bytes,
            'largest': sorted(rows, key=lambda r: r['bytes'], reverse=True)[:15]}


def tracked_names(root=ROOT):
    result = subprocess.run(['git', '-C', str(root), 'ls-files', '--cached', '--others', '--exclude-standard', '-z'], capture_output=True, check=True)
    return [p.decode('utf-8') for p in result.stdout.split(b'\0') if p]


def main():
    report = inspect(ROOT, tracked_names())
    target = ROOT / 'cache/repository-audit/current.json'
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({k: v for k, v in report.items() if k != 'largest'}))


if __name__ == '__main__':
    main()
