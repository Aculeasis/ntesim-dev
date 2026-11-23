"""Build and exercise a local native CLI archive; never publishes artifacts."""
import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import platform
import subprocess
import sys
import tomllib
import zipfile

from cargo import PIN, engine_source_sha256, resolve_toolchain
from cpu_limit import limit_cpus
from verification_workspace import temporary_directory

ROOT = Path(__file__).resolve().parents[1]
RECIPES = sorted((ROOT / "examples/baselines").glob("*.nte")) + [
    ROOT / "examples/mint-native-cycles.nte", ROOT / "examples/chaos-native-cycles.nte",
    ROOT / "presets/native/nanally-full-kit.nte",
]


def encoded(value):
    return (json.dumps(value, indent=2, sort_keys=True) + "\n").encode()


def digest(raw):
    return hashlib.sha256(raw).hexdigest()


def validate_runtime_artifacts(files):
    """Development measurement code and symbols do not belong in portable builds."""
    markers = (b"ntesim-throughput", b"source.from_configuration",
               b"DEVELOPMENT_MEASURE", b"node:inspector")
    for name, raw in files.items():
        filename = PurePosixPath(name).name
        suffix = PurePosixPath(name).suffix
        if (filename.startswith("ntesim-throughput")
                or suffix in (".pdb", ".cpuprofile", ".heapprofile", ".profraw", ".profdata")):
            raise ValueError(f"development measurement artifact in runtime payload: {name}")
        if (filename == "ntesim" or suffix in (".exe", ".wasm", ".js")) and any(marker in raw for marker in markers):
            raise ValueError(f"development measurement code in runtime payload: {name}")


def cargo(*args, rustflags=""):
    # A surrounding check wrapper enables verbose logging. Keep metadata stdout
    # machine-readable and select the distribution's own linker configuration.
    env = {**os.environ, "NTESIM_TOOL_VERBOSE": "0", "CARGO_ENCODED_RUSTFLAGS": rustflags}
    return subprocess.check_output([sys.executable, str(ROOT / "tools/cargo.py"), *args], cwd=ROOT, env=env)


def runtime_dependencies(metadata, owner):
    """Walk normal/build edges from one product in platform-filtered metadata."""
    packages = {p["id"]: p for p in metadata["packages"]}
    nodes = {n["id"]: n for n in metadata["resolve"]["nodes"]}
    pending = [p["id"] for p in packages.values() if p["name"] == owner]
    if len(pending) != 1:
        raise ValueError(f"metadata must contain one {owner}")
    seen = set()
    while pending:
        key = pending.pop()
        if key in seen:
            continue
        seen.add(key)
        pending.extend(d["pkg"] for d in nodes[key]["deps"]
                       if any(k["kind"] != "dev" for k in d["dep_kinds"]))
    return sorted((packages[key] for key in seen if packages[key]["source"] is not None),
                  key=lambda p: (p["name"], p["version"]))


def dependency_notices(packages, lock):
    files, inventory = {}, []
    for package in packages:
        name, version = package["name"], package["version"]
        directory = Path(package["manifest_path"]).parent
        candidates = {p for p in directory.iterdir() if p.is_file()
                      and p.name.lower().startswith(("license", "copying", "notice", "copyright"))}
        if package.get("license_file"):
            candidates.add(directory / package["license_file"])
        if not package.get("license") or not candidates:
            raise ValueError(f"missing dependency license/text: {name} {version}")
        entries = []
        for path in sorted(candidates):
            if not path.resolve().is_relative_to(directory.resolve()):
                raise ValueError(f"license outside dependency: {name}")
            destination = f"licenses/{name}-{version}/{path.name}"
            raw = path.read_bytes()
            if not raw.strip():
                raise ValueError(f"empty dependency license: {path}")
            files[destination] = raw
            entries.append(destination)
        locked = [p for p in lock["package"] if p["name"] == name and p["version"] == version
                  and p.get("source") == package["source"]]
        if len(locked) != 1 or not locked[0].get("checksum"):
            raise ValueError(f"dependency lacks pinned registry checksum: {name} {version}")
        inventory.append({"name": name, "version": version, "license": package["license"],
                          "source": package["source"], "checksum": locked[0]["checksum"],
                          "license_files": entries})
    return files, inventory


def standard_library_notices(sysroot):
    documentation = sysroot / "share/doc/rust"
    names = ("COPYRIGHT-library.html", "licenses/MIT.txt", "licenses/Apache-2.0.txt",
             "licenses/LLVM-exception.txt")
    files = {}
    for name in names:
        path = documentation / name
        if not path.is_file() or not path.read_bytes().strip():
            raise ValueError(f"missing pinned Rust standard-library notice: {path}")
        files[f"licenses/rust-standard-library/{name}"] = path.read_bytes()
    return files


def authored_payload():
    files = {name: (ROOT / name).read_bytes() for name in (
        "LICENSE", "NOTICE", "licenses/pcg32-APACHE-2.0.txt", "licenses/pcg32-NOTICE.txt")}
    files.update({f"examples/{p.name}": p.read_bytes() for p in RECIPES})
    return files


def readme(binary):
    return f"""# ntesim native CLI

Local native build, not a published release. Run on the manifest's target OS/CPU.
Windows MSVC builds link the C runtime statically and require Windows 10 or
newer. Linux builds require the host's
system libc/loader (no older-glibc or cross-platform compatibility is claimed).
The executable needs no Rust, Python, Node, game installation or data directory.

From the extracted directory, use `./{binary}` on POSIX or `./{binary}` in
PowerShell (PowerShell also accepts `.\\{binary}`):

```
./{binary} --help
./{binary} inspect-script examples/adler.nte
./{binary} run-script examples/adler.nte
./{binary} compile-script examples/adler.nte
./{binary} export-script examples/adler.nte saved-report
./{binary} replay-script saved-report/report.json
```

Export destinations must be new. Stdout is JSON; capture it as UTF-8 when saving
compiled inputs or reports. Exit 0 means operation success, 1 an argument/IO
error and 2 a rejected/incomplete operation. Assumptions, strict readiness and
execution completion are separate fields; failed/incomplete trials never rank.
All 23 supplied A0 recipes continue for at least two cycles without resets.
They are legal integration examples, not optimized teams or measured game DPS.
Use subcommand `--help` for batch-script, compare-scripts, search-scripts and
run-script cancellation/resume options. Legacy package-oriented commands require
separate research inputs; those inputs are not part of this native artifact.

Original code is MIT (LICENSE). PCG, Rust standard-library notices and dependency
license texts are in licenses/.
NOTICE retains attribution. Game names and numerical mechanics do not grant a
license to game assets. This artifact excludes artwork, source-site exports,
extracted tables, research evidence, browser assets and data packages. No
redistribution permission for those excluded materials is asserted.

manifest.json binds exact payload bytes, pinned Cargo.lock hash, compiler,
engine source, build target, dependency versions/checksums and license texts.
The adjacent .sha256 file binds the complete ZIP. These hashes detect changes;
they are not signatures or an authorization to publish.

To reproduce from the corresponding source checkout with Python 3.11+, rustup
and the platform linker/SDK installed:

```
python tools/native_distribution.py build --output target/native.zip
python tools/native_distribution.py verify target/native.zip
```

Rust {PIN} and Cargo.lock are pinned. Online builds obtain dependencies through
Cargo; add --offline only with the toolchain and dependency cache available.
Archive ordering, modes and timestamps are fixed for identical payloads. Binary
byte identity across machines/compiler paths is not promised; compare the
manifest identities and reconcile saved source/native reports instead.
""".encode()


def write_archive(path, files, identity):
    validate_runtime_artifacts(files)
    manifest = {"schema_version": 1, **identity, "files": {
        name: {"sha256": digest(raw), "bytes": len(raw)} for name, raw in sorted(files.items())}}
    with zipfile.ZipFile(path, "x", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as archive:
        for name, raw in sorted({**files, "manifest.json": encoded(manifest)}.items()):
            entry = zipfile.ZipInfo(name, (1980, 1, 1, 0, 0, 0))
            entry.create_system = 3
            entry.external_attr = (0o100755 if name in ("ntesim", "ntesim.exe") else 0o100644) << 16
            entry.compress_type = zipfile.ZIP_DEFLATED
            archive.writestr(entry, raw)
    return manifest


def read_archive(path):
    """Validate exact local payload before extracting or executing anything."""
    with zipfile.ZipFile(path) as archive:
        names = archive.namelist()
        if len(names) != len({name.casefold() for name in names}) or "manifest.json" not in names:
            raise ValueError("duplicate entries or missing manifest")
        for name in names:
            parsed = PurePosixPath(name)
            if (parsed.is_absolute() or ".." in parsed.parts or "\\" in name or ":" in name
                    or parsed.as_posix() != name or name == "."):
                raise ValueError("unsafe archive path")
        manifest = json.loads(archive.read("manifest.json"))
        if manifest.get("schema_version") != 1:
            raise ValueError("unsupported native manifest")
        if set(names) != set(manifest["files"]) | {"manifest.json"}:
            raise ValueError("native payload inventory mismatch")
        files = {name: archive.read(name) for name in manifest["files"]}
        for name, raw in files.items():
            if manifest["files"][name] != {"sha256": digest(raw), "bytes": len(raw)}:
                raise ValueError(f"native payload hash/size mismatch: {name}")
        validate_runtime_artifacts(files)
        return manifest, files


def build(output, offline):
    if output.exists() or output.with_suffix(output.suffix + ".sha256").exists():
        raise ValueError("archive and checksum destinations must be new")
    flags = ["--locked", *(["--offline"] if offline else [])]
    host = next(line.split(": ", 1)[1] for line in cargo("-vV").decode().splitlines()
                if line.startswith("host: "))
    source = engine_source_sha256()
    rustflags = "-Ctarget-feature=+crt-static" if host.endswith("windows-msvc") else ""
    cargo("build", "-p", "ntesim-cli", "--release", "--target", host, *flags, rustflags=rustflags)
    metadata = json.loads(cargo("metadata", "--format-version", "1", "--filter-platform", host, *flags))
    lock_bytes = (ROOT / "Cargo.lock").read_bytes()
    notices, dependencies = dependency_notices(runtime_dependencies(metadata, "ntesim-cli"), tomllib.loads(lock_bytes.decode()))
    toolchain = resolve_toolchain(os.environ.copy(), offline)
    notices.update(standard_library_notices(toolchain.parent))
    binary = "ntesim.exe" if os.name == "nt" else "ntesim"
    files = {binary: (ROOT / "target" / host / "release" / binary).read_bytes(),
             "README.md": readme(binary), **authored_payload(), **notices}
    if engine_source_sha256() != source:
        raise ValueError("engine source changed during native build")
    output.parent.mkdir(parents=True, exist_ok=True)
    manifest = write_archive(output, files, {
        "artifact": "ntesim-native", "target": host, "binary": binary,
        "version": next(p["version"] for p in metadata["packages"] if p["name"] == "ntesim-cli"),
        "rust_version": PIN, "engine_source_sha256": source, "cargo_lock_sha256": digest(lock_bytes),
        "packager_sha256": digest(Path(__file__).read_bytes()), "dependencies": dependencies,
        "build_platform": platform.platform(),
        "rustflags": rustflags.split(),
    })
    checksum = digest(output.read_bytes())
    with output.with_suffix(output.suffix + ".sha256").open("x", encoding="utf-8", newline="\n") as stream:
        stream.write(f"{checksum}  {output.name}\n")
    return manifest


def smoke(path, *, current=False):
    expected = path.with_suffix(path.suffix + ".sha256").read_text(encoding="utf-8").split()[0]
    if digest(path.read_bytes()) != expected:
        raise ValueError("native archive checksum mismatch")
    manifest, files = read_archive(path)
    if current and (manifest["engine_source_sha256"] != engine_source_sha256()
                    or manifest["cargo_lock_sha256"] != digest((ROOT / "Cargo.lock").read_bytes())
                    or manifest["packager_sha256"] != digest(Path(__file__).read_bytes())
                    or any(files.get(name) != raw for name, raw in authored_payload().items())):
        raise ValueError("native archive is stale; rebuild from current source/lock/packager")
    # A fresh, space-containing directory has no sources, data or build caches.
    with temporary_directory(prefix="native relocated ") as directory:
        root = Path(directory)
        for name, raw in files.items():
            destination = root / name
            destination.parent.mkdir(parents=True, exist_ok=True)
            destination.write_bytes(raw)
        binary = root / manifest["binary"]
        binary.chmod(0o755)
        env = {k: v for k, v in os.environ.items() if not k.startswith(("NTESIM_", "CARGO_", "RUST"))}
        def run(*args):
            result = subprocess.run([str(binary), *args], cwd=root, env=env, capture_output=True, timeout=60)
            if result.returncode:
                raise ValueError(f"relocated CLI {args[0]} failed: {result.stderr[:1500]!r}")
            return result.stdout
        run("--help")
        for recipe in sorted((root / "examples").glob("*.nte")):
            response = json.loads(run("--profile", "profile.json", "run-script", str(recipe.relative_to(root))))
            profile = json.loads((root / "profile.json").read_bytes())
            if profile["engine_source_sha256"] != manifest["engine_source_sha256"]:
                raise ValueError("native executable/source identity mismatch")
            report = response["execution"]["report"]
            if not all(report[k] for k in ("success", "execution_completed", "invariants_passed")):
                raise ValueError(f"incomplete relocated source recipe: {recipe.name}")
            (root / "profile.json").unlink()
        compiled = run("compile-script", "examples/adler.nte")
        (root / "compiled.json").write_bytes(compiled)
        source_run = json.loads(run("run-script", "examples/adler.nte"))
        native_run = json.loads(run("run-compiled-script", "compiled.json"))
        if source_run != native_run:
            raise ValueError("relocated source/compiled execution mismatch")
        run("export-script", "examples/adler.nte", "saved-report")
        (root / "examples/adler.nte").unlink()
        reconciled = json.loads(run("replay-script", "saved-report/report.json"))
        if not reconciled["reconciled"] or not reconciled["execution_completed"]:
            raise ValueError("relocated source report replay mismatch")
    return {"target": manifest["target"], "engine_source_sha256": manifest["engine_source_sha256"],
            "archive_sha256": expected, "recipes": len([name for name in files if name.endswith(".nte")]),
            "source_compiled_equal": True, "saved_replay_reconciled": True}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    create = commands.add_parser("build")
    create.add_argument("--output", type=Path, required=True)
    create.add_argument("--offline", action="store_true")
    verify = commands.add_parser("verify", help="Execute only a trusted locally built archive")
    verify.add_argument("archive", type=Path)
    verify.add_argument("--current", action="store_true", help="Also require current checkout build identities")
    args = parser.parse_args()
    limit_cpus()
    try:
        if args.command == "build":
            build(args.output.resolve(), args.offline)
            print(f"Native archive: {args.output}")
        else:
            print(json.dumps(smoke(args.archive.resolve(), current=args.current), sort_keys=True))
    except (OSError, ValueError, subprocess.SubprocessError, zipfile.BadZipFile) as error:
        print(f"native distribution failed: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
