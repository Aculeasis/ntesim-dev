"""Build/verify a local portable Studio archive. Does not publish or deploy."""
import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
import tomllib

from cargo import PIN, engine_source_sha256, resolve_toolchain
from cpu_limit import limit_cpus
from studio_build import INSTANT, build_instant, resolve as studio_build
from native_distribution import (cargo, dependency_notices, digest, encoded, read_archive,
                                 runtime_dependencies, standard_library_notices, write_archive,
                                 validate_runtime_artifacts)
from verification_workspace import temporary_directory
from wasm_bindings import VERSION as BINDGEN_VERSION

ROOT = Path(__file__).resolve().parents[1]
TARGET = "wasm32-unknown-unknown"


def source_identity():
    paths = [p for directory in ("web", "presets/studio", "crates/wasm/js")
             for p in (ROOT / directory).rglob("*") if p.is_file()]
    paths += [ROOT / "tools" / name for name in (
        "browser_distribution.py", "native_distribution.py", "cargo.py", "wasm_bindings.py",
        "web.mjs", "studio_presets.mjs", "studio_build.py", "cpu_limit.py")]
    paths += [ROOT / name for name in (
        "LICENSE", "licenses/pcg32-APACHE-2.0.txt", "licenses/pcg32-NOTICE.txt",
        "licenses/zstd-wasm-MIT.txt", "licenses/zstd-wasm-NOTICE.txt",
        "licenses/zstd-BSD-3-Clause.txt", "licenses/emscripten-MIT.txt")]
    return {"engine_source_sha256": engine_source_sha256(),
            "inputs_sha256": digest(encoded({p.relative_to(ROOT).as_posix(): digest(p.read_bytes())
                                             for p in sorted(paths)})),
            "cargo_lock_sha256": digest((ROOT / "Cargo.lock").read_bytes()),
            "npm_lock_sha256": digest((ROOT / "web/package-lock.json").read_bytes())}


def npm_notices(lock, modules):
    """Traverse only shipped npm runtime edges; require exact locked installs."""
    packages = lock["packages"]
    pending = list(packages[""]["dependencies"])
    seen, files, inventory = set(), {}, []
    while pending:
        name = pending.pop()
        if name in seen:
            continue
        seen.add(name)
        # This product's runtime closure is flat; reject unsupported resolution
        # instead of silently licensing a different nested dependency version.
        entry = packages["node_modules/" + name]
        directory = modules / name
        installed = json.loads((directory / "package.json").read_text(encoding="utf-8"))
        if (installed["name"] != name or installed["version"] != entry["version"]
                or not entry.get("integrity") or not entry.get("license")):
            raise ValueError(f"npm install differs from lock or lacks attribution: {name}")
        license_files = []
        if name == '@bokuweb/zstd-wasm':
            for filename in ('zstd-wasm-MIT.txt', 'zstd-wasm-NOTICE.txt',
                             'zstd-BSD-3-Clause.txt', 'emscripten-MIT.txt'):
                destination = 'licenses/' + filename
                files[destination] = (ROOT / destination).read_bytes()
                license_files.append(destination)
        for p in sorted(directory.iterdir()):
            if p.is_file() and p.name.lower().startswith(("license", "notice", "copying", "copyright")):
                destination = f"licenses/npm/{name}/{p.name}"
                raw = p.read_bytes()
                if not raw.strip() or p.is_symlink():
                    raise ValueError(f"invalid npm license: {p}")
                files[destination] = raw
                license_files.append(destination)
        if not license_files:
            raise ValueError(f"missing npm license: {name}")
        inventory.append({"name": name, "version": entry["version"], "license": entry["license"],
                          "resolved": entry["resolved"], "integrity": entry["integrity"],
                          "license_files": license_files})
        for dependency in entry.get("dependencies", {}):
            if "node_modules/" + name + "/node_modules/" + dependency in packages:
                raise ValueError("nested runtime npm dependencies require explicit resolution")
            pending.append(dependency)
    return files, sorted(inventory, key=lambda p: p["name"])


def stage(output):
    identity = source_identity()
    metadata = json.loads(cargo("metadata", "--format-version", "1", "--filter-platform", TARGET,
                                "--locked", "--offline"))
    rust_files, rust = dependency_notices(runtime_dependencies(metadata, "ntesim-wasm"),
                                         tomllib.loads((ROOT / "Cargo.lock").read_text()))
    rust_files.update(standard_library_notices(resolve_toolchain(os.environ.copy(), True).parent))
    npm_files, npm = npm_notices(json.loads((ROOT / "web/package-lock.json").read_bytes()),
                                 ROOT / "cache/web/node_modules")
    files = {**rust_files, **npm_files,
             "LICENSE": (ROOT / "LICENSE").read_bytes(),
             **{p: (ROOT / p).read_bytes() for p in (
                 "licenses/pcg32-APACHE-2.0.txt", "licenses/pcg32-NOTICE.txt")},
             "NOTICE": b"""ntesim Studio: original code and SVG icons use the MIT license (LICENSE).
PCG32 is adapted under Apache-2.0; its license and notice are in licenses/.
Rust/WASM, generated wasm-bindgen glue and React runtime notices are in licenses/.
build-info.json lists the locked runtime dependency closures and exact licenses.
Character, weapon, set and element illustrations are third-party Neverness to
Everness artwork, sourced from https://cdn.prydwen.gg/images/nte/ for identity
display. They belong to their respective owners and are not covered by MIT.
No redistribution license for this artwork is asserted. Source-site descriptions,
raw exports, research packages and acquisition evidence are excluded.
This is an independent simulator.
"""}
    files["licenses/runtime.txt"] = b"\n\n".join(
        name.encode() + b"\n" + raw for name, raw in sorted(npm_files.items()))
    info = {"artifact": "ntesim-browser", "schema_version": 1, **identity,
            "studio_build": studio_build(ROOT),
            "rust_version": PIN, "wasm_bindgen_version": BINDGEN_VERSION,
            "node_version": subprocess.check_output(["node", "--version"], text=True).strip(),
            "target": TARGET, "rust_dependencies": rust, "npm_dependencies": npm,
            "wasm_sha256": digest((ROOT / "target/wasm-bindgen/ntesim_wasm_bg.wasm").read_bytes()),
            "report_codec_wasm_sha256": digest((ROOT / 'cache/web/node_modules/@bokuweb/zstd-wasm/dist/web/zstd.wasm').read_bytes())}
    for name, raw in {**files, "build-info.json": encoded(info)}.items():
        p = output / name
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_bytes(raw)
    if source_identity() != identity:
        raise ValueError("browser inputs changed during staging")
    return info


def payload(directory):
    files = {}
    artwork = {"assets/artwork/" + p.name
               for p in (ROOT / "web/public/assets/artwork").glob("*.webp")}
    for p in directory.rglob("*"):
        if p.is_symlink() or getattr(p, "is_junction", lambda: False)():
            raise ValueError("browser payload cannot contain links")
        if p.is_file():
            files[p.relative_to(directory).as_posix()] = p.read_bytes()
    for name in files:
        if not (name in ("index.html", "sw.js", "build-info.json", "NOTICE", "LICENSE")
                or name.startswith("assets/") and Path(name).suffix in (".js", ".css", ".wasm", ".svg")
                or name in artwork or name.startswith("licenses/")):
            raise ValueError(f"unreviewed browser payload: {name}")
    required = {"index.html", "sw.js", "build-info.json", "NOTICE", "LICENSE", "licenses/runtime.txt"}
    if not required.issubset(files):
        raise ValueError("browser payload is incomplete")
    return files


def validate(files, current):
    validate_runtime_artifacts(files)
    info = json.loads(files["build-info.json"])
    if info["artifact"] != "ntesim-browser" or info["target"] != TARGET:
        raise ValueError("unsupported browser artifact")
    wasm = [raw for name, raw in files.items() if name.endswith(".wasm")]
    if len(wasm) != 2 or sum(digest(raw) == info["wasm_sha256"] for raw in wasm) != 1:
        raise ValueError("browser WASM identity mismatch")
    if sum(digest(raw) == info['report_codec_wasm_sha256'] for raw in wasm) != 1:
        raise ValueError('browser report codec WASM identity mismatch')
    for dependency in info["rust_dependencies"] + info["npm_dependencies"]:
        if not dependency["license_files"] or any(p not in files for p in dependency["license_files"]):
            raise ValueError("missing browser dependency notices")
    if current and any(info.get(k) != v for k, v in source_identity().items()):
        raise ValueError("browser archive is stale")
    return info


def readme():
    return b"""# ntesim Studio portable browser build

Local artifact, not a published release. Extract the ZIP into a new directory.
Serve site/ with a static HTTP server, for example:

    python -m http.server 8080 --directory site --bind 127.0.0.1

Open http://127.0.0.1:8080/ in current Chrome. file:// is unsupported. Any HTTPS
static host may mount site/ at a directory prefix; preserve relative paths and
serve .wasm as application/wasm and .js as JavaScript. No backend/game/data package
is needed. Initial load requires the files; subsequent controlled loads work
offline. Reload after a Service Worker update activates. Storage is origin-local.

Choose a preset or paste one shared .nte script; it owns builds, dummy stats,
assumptions and actions. Run, inspect Results/Sample, and use native JSON exports
for exact large integers/replay. Invalid or incomplete runs cannot rank. Supplied
recipes are integration baselines, not optimized teams or measured game DPS.
Model hypotheses and unsupported branches remain visible in engine diagnostics.

LICENSE, NOTICE, build-info.json and licenses/ inside site/ describe this artifact.
Identity illustrations are third-party game artwork; see NOTICE for attribution.
Source descriptions, raw exports and acquisition evidence are excluded.
manifest.json inventories every payload by length/SHA-256. The adjacent .sha256
binds the ZIP. These are integrity checks, not signatures or publishing approval.

Reproduce from the matching checkout with Python 3.11+, Node 22.12+ with npm,
rustup/Rust 1.98.1, wasm32-unknown-unknown and wasm-bindgen-cli 0.2.128 installed:

    node tools/web.mjs install
    python tools/browser_distribution.py build --output target/studio.zip
    python tools/browser_distribution.py verify target/studio.zip --current

Build downloads locked Cargo dependencies unless --offline is supplied. Install
the WASM target and pinned generator via the repository runbook first. No build
machine binary byte equality is claimed; retain each artifact's own identity.
Verification runs trusted local JavaScript in installed Chrome via development
Playwright. Set NTESIM_PLAYWRIGHT_MODULE to its absolute index.mjs. No publication,
tag or deployment occurs. Other browser/OS combinations are not certified.
"""


def build(output, offline, release=False):
    instant = build_instant(os.environ)
    os.environ[INSTANT] = instant
    env = {**os.environ, INSTANT: instant}
    # Bind Vite and staging to the same source/instant before compiling.
    env["NTESIM_STUDIO_BUILD"] = json.dumps(studio_build(ROOT, env))
    os.environ["NTESIM_STUDIO_BUILD"] = env["NTESIM_STUDIO_BUILD"]
    if output.exists() or output.with_suffix(output.suffix + ".sha256").exists():
        raise ValueError("archive and checksum destinations must be new")
    if not offline:
        # Offline staging queries workspace metadata, including host dependencies.
        cargo("fetch", "--locked")
    cargo("build", "-p", "ntesim-wasm", "--release", "--target", TARGET,
          "--locked", *(["--offline"] if offline else []))
    if release:
        subprocess.run(["node", "tools/web.mjs", "teams", "--fresh"], cwd=ROOT, env=env, check=True)
    subprocess.run(["node", "tools/web.mjs", "build"], cwd=ROOT, env=env, check=True)
    files = payload(ROOT / "target/web")
    info = validate(files, True)
    output.parent.mkdir(parents=True, exist_ok=True)
    write_archive(output, {"README.md": readme(), **{"site/" + p: raw for p, raw in files.items()}},
                  {"artifact": "ntesim-browser", "build": info})
    output.with_suffix(output.suffix + ".sha256").write_text(
        f"{digest(output.read_bytes())}  {output.name}\n", encoding="utf-8")


def verify(archive, current):
    checksum = archive.with_suffix(archive.suffix + ".sha256").read_text().split()[0]
    if checksum != digest(archive.read_bytes()):
        raise ValueError("browser archive checksum mismatch")
    manifest, files = read_archive(archive)
    if manifest.get("artifact") != "ntesim-browser" or files.get("README.md") != readme():
        raise ValueError("unsupported browser archive or instructions")
    with temporary_directory(prefix="browser relocated space-") as temp:
        directory = Path(temp)
        for name, raw in files.items():
            p = directory / name
            p.parent.mkdir(parents=True, exist_ok=True)
            p.write_bytes(raw)
        info = validate(payload(directory / "site"), current)
        if manifest["build"] != info or set(files) != {"README.md"} | {
                "site/" + name for name in payload(directory / "site")}:
            raise ValueError("browser archive inventory/identity mismatch")
        env = {**os.environ, "NTESIM_BROWSER_SITE": str(directory / "site")}
        subprocess.run(["node", "tools/web.mjs", "verify-trials", "--browser"],
                       cwd=ROOT, env=env, check=True)


def main():
    limit_cpus()
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    create = commands.add_parser("build")
    create.add_argument("--output", type=Path, required=True)
    create.add_argument("--offline", action="store_true")
    create.add_argument("--release", action="store_true", help="Recalculate every delivered team with the current engine before packaging")
    check = commands.add_parser("verify")
    check.add_argument("archive", type=Path)
    check.add_argument("--current", action="store_true")
    prepare = commands.add_parser("stage")
    prepare.add_argument("output", type=Path)
    args = parser.parse_args()
    try:
        if args.command == "build":
            build(args.output, args.offline, args.release)
        elif args.command == "verify":
            verify(args.archive, args.current)
        else:
            stage(args.output)
    except (ValueError, OSError, KeyError, subprocess.CalledProcessError) as error:
        print(f"browser distribution failed: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
