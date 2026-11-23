from __future__ import annotations

import hashlib
import json
from functools import lru_cache
from pathlib import Path

import data_package


PACKAGE_METADATA_INPUTS = frozenset({
    "manifest.json",
    "sources.json",
    "coverage.json",
})

EQUIPMENT_TABLES = ()
PROFILE_AUDIT_TABLES = ()
RECOMMENDED_GEAR_TABLES = ()
WEAPON_PASSIVE_TABLES = ()


def inputs_for_tables(table_names) -> frozenset[str]:
    return PACKAGE_METADATA_INPUTS | frozenset(f"tables/{name}.json" for name in table_names)


def game_build_inputs(package: Path, request: dict) -> frozenset[str]:
    tables = [*EQUIPMENT_TABLES]
    if request.get("gear_projection") is not None:
        tables.extend([
            *WEAPON_PASSIVE_TABLES,
        ])
    return inputs_for_tables(dict.fromkeys(tables))


@lru_cache(maxsize=4)
def audited_package_inputs(package_name: str) -> dict[str, str]:
    package = Path(package_name).resolve()
    data_package.audit(package)
    manifest = data_package.read(package / "manifest.json")
    sources = data_package.read(package / "sources.json")
    allowed = {
        "manifest.json",
        "sources.json",
        "coverage.json",
        "research/fixtures.json",
        *(entry["path"] for entry in manifest["tables"]),
        *(source["path"] for source in sources if not (package / 'storage.json').exists() or not source['path'].startswith('evidence/')),
    }
    if (package / 'storage.json').exists():
        allowed.add('storage.json')
        allowed.update(data_package.read(package / 'storage.json')['files'])
    return {
        relative: hashlib.sha256(data_package.contained(package, relative).read_bytes()).hexdigest()
        for relative in allowed
    }


def verify_input_hashes(package: Path, report: dict, required_inputs) -> dict[str, str]:
    expected_package = audited_package_inputs(str(Path(package).resolve()))
    package = Path(package).resolve()
    # A cached structural audit cannot hide mutations between report checks.
    for relative, digest in expected_package.items():
        raw_digest = hashlib.sha256(data_package.contained(package, relative).read_bytes()).hexdigest()
        if raw_digest != digest:
            raise ValueError(f"audited package changed since source audit: {relative}")
    hashes = report.get("input_hashes")
    if not isinstance(hashes, dict) or not hashes:
        raise ValueError("report input_hashes must be a nonempty object")
    for relative, expected in hashes.items():
        if relative not in expected_package:
            raise ValueError(f"report input is not a member of the audited package: {relative}")
        if not isinstance(expected, str) or len(expected) != 64:
            raise ValueError(f"invalid report input digest: {relative}")
        if expected_package[relative] != expected:
            raise ValueError(f"report input digest differs from audited package: {relative}")
    required = set(required_inputs)
    if (package / 'storage.json').exists():
        required.add('storage.json')
    if required != set(hashes):
        missing = sorted(required - set(hashes))
        extra = sorted(set(hashes) - required)
        raise ValueError(f"report input receipt is incomplete or unexpected; missing={missing}, extra={extra}")
    return hashes


@lru_cache(maxsize=128)
def _character_report(binary, package, character_id, level, breakthrough):
    # Rust owns character constants/growth. Consumers below independently check
    # equipment arithmetic and transport; published character endpoints have a
    # separate Rust and CLI check, without a second growth implementation.
    import subprocess
    from verification_workspace import temporary_directory
    root = Path(__file__).resolve().parents[1]
    cache = root / "cache/character-consumer-inputs"
    cache.mkdir(parents=True, exist_ok=True)
    with temporary_directory(dir=cache) as temp:
        request = Path(temp) / "request.json"
        request.write_text(json.dumps({"schema_version":1,"content_id":"ntesim-current",
            "character_id":character_id,"level":level,"breakthrough":breakthrough}), encoding="utf-8")
        result = subprocess.run([binary,"character-stats",package,str(request)],capture_output=True,check=True)
        report = json.loads(result.stdout)
    return json.dumps(report)


def character_attributes(binary, package, character_id, level=80, breakthrough=6):
    from fractions import Fraction
    report=json.loads(_character_report(str(binary),str(package),character_id,level,breakthrough))
    return {name:Fraction(int(v["numerator"]),int(v["denominator"])) for name,v in report["attributes"].items()}


def character_metadata(binary, package, character_id):
    report=json.loads(_character_report(str(binary),str(package),character_id,80,6))
    return {"CharacterGroupType":report["compatibility_group"],"CharacterElementType":report["element"]}


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def exact(value):
    from fractions import Fraction
    return Fraction(int(value["numerator"]),int(value["denominator"]))
