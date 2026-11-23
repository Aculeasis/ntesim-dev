"""Run the pinned compiler with caches and temporary files inside ntesim.

Use an already installed matching toolchain, or install the exact toolchain
with rustup configured to product-local homes. No global installs/config writes.
"""
import os
import hashlib
from pathlib import Path
import subprocess
import sys
import tomllib
from tool_output import run_logged
from cpu_limit import limit_cpus

ROOT = Path(__file__).resolve().parents[1]
PIN = tomllib.loads((ROOT / "rust-toolchain.toml").read_text())["toolchain"]["channel"]


def find_compiler(env):
    """Read installed toolchains without asking a rustup shim to install one."""
    for toolchain in (PIN, "stable"):
        found = subprocess.run(
            ["rustup", "which", "--toolchain", toolchain, "rustc"],
            env=env, capture_output=True, text=True,
        )
        if found.returncode == 0:
            compiler = found.stdout.strip()
            version = subprocess.check_output([compiler, "--version"], env=env, text=True).split()[1]
            if version == PIN:
                return Path(compiler).parent
    return None


def resolve_toolchain(env, offline, verbose=False):
    local_env = {**env, "RUSTUP_HOME": str(ROOT / "cache/rustup")}
    try:
        for discovery_env in (local_env, env):
            binary = find_compiler(discovery_env)
            if binary is not None:
                if discovery_env is local_env:
                    env["RUSTUP_HOME"] = local_env["RUSTUP_HOME"]
                return binary
        if offline:
            raise RuntimeError(f"Rust {PIN} is not installed; offline mode forbids toolchain installation")
        env["RUSTUP_HOME"] = local_env["RUSTUP_HOME"]
        status = run_logged(
            ["rustup", "toolchain", "install", PIN, "--profile", "minimal", "--component", "rustfmt,clippy"],
            env=env, label="rustup-install", verbose=verbose,
        )
        if status:
            raise RuntimeError(f"toolchain installation failed (exit {status})")
        binary = find_compiler(env)
        if binary is None:
            raise RuntimeError(f"installed toolchain does not provide Rust {PIN}")
        return binary
    except FileNotFoundError as error:
        raise RuntimeError("toolchain discovery requires rustup on PATH and installed compiler binaries") from error


def engine_source_sha256():
    """Identity shared by builds and checks that reject stale release artifacts."""
    fingerprint = hashlib.sha256()
    runtime_sources = [path for path in (ROOT / "crates").rglob("*.rs")
                       if "tests" not in path.relative_to(ROOT).parts
                       and path.stem != "tests" and not path.stem.endswith("_tests")]
    engine_files = [ROOT / "Cargo.toml", ROOT / "Cargo.lock", ROOT / "rust-toolchain.toml"] + runtime_sources + sorted((ROOT / "crates").rglob("Cargo.toml"))
    engine_files += sorted((ROOT / "presets/research").glob("*.csv")) + sorted((ROOT / "presets/research").glob("*.json"))
    for path in sorted(engine_files):
        if path.exists():
            name = path.relative_to(ROOT).as_posix().encode()
            raw = path.read_bytes()
            fingerprint.update(len(name).to_bytes(8, "big") + name + len(raw).to_bytes(8, "big") + raw)
    return fingerprint.hexdigest()


def main():
    limit_cpus()
    args = sys.argv[1:]
    boundary = args.index("--") if "--" in args else len(args)
    verbose = ("--verbose" in args[:boundary] or any(arg in ("-v", "-vv") for arg in args[:boundary])
               or os.environ.get("NTESIM_TOOL_VERBOSE") == "1")
    args = [arg for i, arg in enumerate(args) if i >= boundary or arg != "--verbose"]
    env = os.environ.copy()
    # libcurl can otherwise select Windows' system proxy instead of Codex's
    # managed local gateway. This preserves sandbox domain policy; upstream
    # routing remains controlled by Codex, not by this build helper.
    if env.get("CODEX_NETWORK_PROXY_ACTIVE") == "1" and env.get("HTTPS_PROXY"):
        env.setdefault("CARGO_HTTP_PROXY", env["HTTPS_PROXY"])
    env["NTESIM_ENGINE_SOURCE_SHA256"] = engine_source_sha256()
    for key, relative in {"CARGO_HOME": "cache/cargo", "CARGO_TARGET_DIR": "target", "TEMP": "cache/tmp", "TMP": "cache/tmp"}.items():
        path = ROOT / relative
        path.mkdir(parents=True, exist_ok=True)
        env[key] = str(path)
    offline = "--offline" in args or "--frozen" in args or env.get("CARGO_NET_OFFLINE", "").lower() in ("true", "1")
    try:
        binary = resolve_toolchain(env, offline, verbose=verbose)
    except RuntimeError as error:
        print(f"ntesim: {error}", file=sys.stderr)
        return 1
    suffix = ".exe" if os.name == "nt" else ""
    env["RUSTC"] = str(binary / ("rustc" + suffix))
    env["RUSTDOC"] = str(binary / ("rustdoc" + suffix))
    env["PATH"] = str(binary) + os.pathsep + env["PATH"]
    if verbose:
        print(f"ntesim toolchain {PIN}; local Cargo cache and target", flush=True)
    command = [str(binary / ("cargo" + suffix)), *args]
    # run/metadata and other data-producing commands keep their stdout contract.
    if args and args[0] in ("test", "build", "check", "clippy", "fmt", "doc", "clean"):
        status = run_logged(command, cwd=ROOT, env=env, label=f"cargo-{args[0]}", verbose=verbose)
        if status == 0 and args[0] == "build" and "wasm32-unknown-unknown" in args:
            from wasm_bindings import generate
            profile = "release" if "--release" in args else "debug"
            for index, argument in enumerate(args):
                if argument == "--profile" and index + 1 < len(args):
                    profile = args[index + 1]
                elif argument.startswith("--profile="):
                    profile = argument.split("=", 1)[1]
            try:
                return generate(ROOT, env, profile, verbose)
            except RuntimeError as error:
                print(f"ntesim: {error}", file=sys.stderr)
                return 1
        return status
    return subprocess.call(command, cwd=ROOT, env=env)


if __name__ == "__main__":
    raise SystemExit(main())
