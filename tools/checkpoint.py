import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import time
import tomllib
from urllib.parse import unquote, urlsplit
from uuid import uuid4

from cpu_limit import limit_cpus
from tool_output import run_logged

ROOT = Path(__file__).resolve().parents[1]
VERSION = 5

DEFAULT_CRATE_UPSTREAM = {
    "ntesim-contracts": {"ntesim-contracts"},
    "ntesim-core": {"ntesim-contracts", "ntesim-core"},
    "ntesim-data": {"ntesim-contracts", "ntesim-data"},
    "ntesim-content": {"ntesim-contracts", "ntesim-core", "ntesim-data", "ntesim-content"},
    "ntesim-research": {"ntesim-contracts", "ntesim-core", "ntesim-data", "ntesim-content", "ntesim-research"},
    "ntesim-cli": {"ntesim-contracts", "ntesim-core", "ntesim-data", "ntesim-content", "ntesim-research", "ntesim-cli"},
    "ntesim-wasm": {"ntesim-contracts", "ntesim-core", "ntesim-data", "ntesim-content", "ntesim-research", "ntesim-wasm"},
}
DEFAULT_PACKAGE_OWNERS = {
    "ntesim-contracts": "contracts",
    "ntesim-core": "core",
    "ntesim-data": "data",
    "ntesim-content": "content",
    "ntesim-research": "research",
    "ntesim-cli": "cli",
    "ntesim-wasm": "wasm",
}


def crate_manifests(root):
    crates_dir = root / "crates"
    if not crates_dir.is_dir():
        return {}
    manifests = {}
    for path in crates_dir.glob("*/Cargo.toml"):
        try:
            manifests[path.parent.name] = tomllib.loads(path.read_text(encoding="utf-8"))
        except (OSError, tomllib.TOMLDecodeError):
            pass
    return manifests


def package_to_crate_dirs(root):
    manifests = crate_manifests(root)
    dirs = DEFAULT_PACKAGE_OWNERS.copy()
    for owner, m in manifests.items():
        pkg = m.get("package", {}).get("name")
        if pkg:
            dirs[pkg] = owner
    return dirs


def crate_upstream_closure(root, package_name):
    manifests = crate_manifests(root)
    pkg_by_name = {m.get("package", {}).get("name"): m for m in manifests.values() if "package" in m}
    if not pkg_by_name or package_name not in pkg_by_name:
        return DEFAULT_CRATE_UPSTREAM.get(package_name, {package_name})
    closure = {package_name}
    while True:
        expanded = closure.copy()
        for pkg in closure:
            if pkg in pkg_by_name:
                manifest = pkg_by_name[pkg]
                for section in ("dependencies", "dev-dependencies", "build-dependencies"):
                    for dep in manifest.get(section, {}):
                        if dep in pkg_by_name:
                            expanded.add(dep)
        if expanded == closure:
            break
        closure = expanded
    return closure


def package_test_inputs(root, package_name, files):
    closure = crate_upstream_closure(root, package_name)
    pkg_dirs = package_to_crate_dirs(root)
    prefixes = [f"crates/{pkg_dirs[pkg]}/" for pkg in closure if pkg in pkg_dirs]
    exact = {"Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "tools/cargo.py",
             "tools/checkpoint.py", "tools/cpu_limit.py", "tools/tool_output.py"}
    if closure.intersection({"ntesim-data", "ntesim-content", "ntesim-research", "ntesim-cli", "ntesim-wasm"}):
        prefixes.append("data/")
        prefixes.append("presets/")
    if closure.intersection({"ntesim-research", "ntesim-cli"}):
        prefixes.extend(["fixtures/", "research/", "reviews/"])
    if "ntesim-cli" in closure:
        prefixes.append("examples/")
    pfx = tuple(prefixes)
    return {path: value for path, value in files.items() if path in exact or path.startswith(pfx)}


def package_check_key(root, package_name, files):
    inputs = package_test_inputs(root, package_name, files)
    binary = shutil.which(sys.executable)
    identity = None
    if binary:
        stat = Path(binary).stat()
        identity = [str(Path(binary).resolve()), stat.st_size, stat.st_mtime_ns]
    environment = check_environment_fingerprint(root)
    value = {"version": VERSION, "root": str(root.resolve()), "package": package_name,
             "files": inputs, "environment": environment, "python": sys.version, "binary": identity}
    return hashlib.sha256(json.dumps(value, sort_keys=True).encode("utf-8")).hexdigest()


def git(root, *args, input=None, allow_missing=False):
    # Windows can drop empty Git config values from the native environment when Python restores them.
    result = subprocess.run(["git", "-C", str(root), *args], input=input, capture_output=True,
                            env=os.environ.copy())
    if result.returncode and not (allow_missing and result.returncode == 1):
        raise ValueError(result.stderr.decode("utf-8", errors="replace").strip())
    return result.stdout


def head_id(root):
    return git(root, "rev-parse", "--verify", "--quiet", "HEAD", allow_missing=True).decode().strip()


def index_files(root):
    files = {}
    for entry in git(root, "ls-files", "--stage", "-z").split(b"\0"):
        if not entry:
            continue
        metadata, name = entry.split(b"\t", 1)
        mode, oid, stage = metadata.decode().split()
        if stage != "0" or mode not in ("100644", "100755"):
            raise ValueError("checkpoint requires resolved regular files; unsupported index entry: "
                             + name.decode("utf-8"))
        files[name.decode("utf-8")] = {"mode": mode, "oid": oid}
    return files


def head_files(root, head):
    files = {}
    if head:
        for entry in git(root, "ls-tree", "-r", "-z", head).split(b"\0"):
            if entry:
                metadata, name = entry.split(b"\t", 1)
                mode, kind, oid = metadata.decode().split()
                if kind != "blob":
                    raise ValueError("checkpoint does not support submodules")
                files[name.decode("utf-8")] = {"mode": mode, "oid": oid}
    return files


def worktree_files(root):
    resolved_root = root.resolve()
    indexed = index_files(root)
    names = set(indexed)
    names.update(name.decode("utf-8") for name in
                 git(root, "ls-files", "--others", "--exclude-standard", "-z").split(b"\0") if name)
    files = {}
    for name in sorted(names):
        path = root / name
        if path.is_symlink() or getattr(path, "is_junction", lambda: False)():
            raise ValueError(f"checkpoint does not follow symlinks/junctions: {name}")
        if not path.resolve().is_relative_to(resolved_root):
            raise ValueError(f"checkpoint path escapes the checkout: {name}")
        if not path.exists():
            continue
        if not path.is_file():
            raise ValueError(f"checkpoint input is not a regular file: {name}")
        mode = indexed.get(name, {}).get("mode", "100644")
        if os.name != "nt":
            mode = "100755" if path.stat().st_mode & 0o111 else "100644"
        files[name] = {"mode": mode, "sha256": hashlib.sha256(path.read_bytes()).hexdigest()}
    quoted = [json.dumps(name, ensure_ascii=False) for name in files]
    if quoted:
        # Git applies the same attributes/line-ending filters as staging, without writing objects.
        oids = git(root, "hash-object", "--stdin-paths",
                   input=("\n".join(quoted) + "\n").encode("utf-8")).decode().splitlines()
        if len(oids) != len(files):
            raise ValueError("incomplete Git input fingerprint")
        for value, oid in zip(files.values(), oids):
            value["oid"] = oid
    return files


def tree_files(files):
    return {name: {"mode": value["mode"], "oid": value["oid"]} for name, value in files.items()}


def changed_files(before, after):
    return sorted(name for name in before.keys() | after.keys() if before.get(name) != after.get(name))


def check(name, command, env=None):
    return {"name": name, "command": command, "env": env or {}}


def browser_boundaries(changed, *, full=False, wasm_smoke=False):
    web = any(name.startswith(("web/", "crates/wasm/js/")) or name.startswith("tools/")
              and name.endswith((".mjs", ".js", ".mts"))
              or name in ("tools/browser_distribution.py", "tools/native_distribution.py", "tools/studio_build.py", "LICENSE")
              or name.startswith("licenses/") for name in changed)
    browser = full or web
    api = any(name.startswith(("crates/wasm/", "crates/content/src/dsl/",
                              "crates/content/src/simulation/model/", "crates/content/src/simulation/report/"))
              for name in changed)
    toolchain = any(name == "Cargo.toml" or name.startswith(("crates/", ".cargo/")) and name.endswith(("/Cargo.toml", "config.toml"))
                    or name in ("Cargo.lock", "rust-toolchain.toml") for name in changed)
    bundled = any(name.startswith("presets/research/") and name.endswith((".csv", ".json")) for name in changed)
    return browser, browser or wasm_smoke or api or toolchain or bundled


def check_plan(root, changed, *, full=False, wasm_smoke=False):
    # Deletions still need a static pass: removing Markdown can leave inbound links broken.
    docs = [name for name in changed if name.lower().endswith(".md")]
    structured = [name for name in changed if name.endswith((".json", ".toml")) and (root / name).is_file()]
    model_inputs = full or any(name.startswith(("fixtures/", "research/", "reviews/")) for name in changed)
    rust_source = full or any(name.endswith(".rs") or name.startswith(".cargo/") or name in ("Cargo.toml", "Cargo.lock", "rust-toolchain.toml")
               or name.startswith("crates/") and name.endswith("Cargo.toml") for name in changed)
    bundled = any(name.startswith("presets/research/") and name.endswith((".csv", ".json")) for name in changed)
    csv_fixtures = any(name.startswith("crates/") and "/tests/fixtures/" in name and name.endswith(".csv") for name in changed)
    rust = rust_source or bundled or csv_fixtures
    runtime = full or any(name.startswith("crates/") and "/tests/" not in name and not name.endswith("/tests.rs")
                  and name.endswith((".rs", "Cargo.toml"))
                  or name.startswith(".cargo/")
                  or name in ("Cargo.toml", "Cargo.lock", "rust-toolchain.toml") for name in changed) or model_inputs or bundled
    data = full or any(name.startswith("data/") for name in changed)
    scenarios = any(name.startswith(("examples/", "presets/")) and not name.endswith(".md") for name in changed)
    # Check browser integration when its API or delivery boundary is affected.
    browser, wasm = browser_boundaries(changed, full=full, wasm_smoke=wasm_smoke)
    deleted_configs = {name for name in changed if name.startswith("configs/") and not (root / name).exists()}
    tooling = any(name.startswith(("tools/", "tests/", ".codex/", ".githooks/"))
                  or name == ".github/workflows/deploy-pages.yml"
                  or name in deleted_configs or name in (".gitattributes", ".gitignore") for name in changed)
    unknown = [name for name in changed if not name.endswith(".md")
               and not name.startswith(("crates/", "data/", "docs/", "openspec/", ".agents/", "examples/", "presets/", "web/", "tools/",
                                        ".cargo/", "tests/", ".codex/", ".githooks/", "reviews/", "research/", "fixtures/", "licenses/"))
               and name not in deleted_configs
               and name not in ("Cargo.toml", "Cargo.lock", "rust-toolchain.toml", ".gitattributes", ".gitignore",
                                "LICENSE", "NOTICE", ".github/workflows/deploy-pages.yml")]
    if unknown:
        raise ValueError("classify new checkpoint inputs before committing: " + ", ".join(unknown[:10]))
    commands = [check("repository-privacy", ["python", "tools/checkpoint.py", "privacy"])]
    if docs:
        commands.append(check("local-links-and-formats", ["python", "tools/checkpoint.py", "static", *docs]))
    if structured:
        commands.append(check("structured-syntax", ["python", "tools/checkpoint.py", "syntax", *structured]))
    if (root / "openspec/config.yaml").is_file() and (full or any(
            name.startswith(("openspec/", ".agents/")) or name == "tools/openspec.py" for name in changed)):
        commands.append(check("openspec-validation", ["python", "tools/openspec.py", "validate",
                              "--all", "--strict", "--no-interactive", "--concurrency", "4"]))
        commands.append(check("openspec-archive-tasks", ["python", "tools/openspec.py", "validate",
                              "--archived", "--strict", "--no-interactive", "--concurrency", "4"]))
    if tooling or data:
        commands.append(check("python-tests", ["python", "tools/python_tests.py"]))
    if rust or model_inputs:
        if rust_source:
            commands.append(check("rust-format", ["python", "tools/cargo.py", "fmt", "--all", "--", "--check"]))
        owners = {name.split("/")[1] for name in changed if name.startswith("crates/") and len(name.split("/")) > 2}
        if bundled:
            owners.add("data")
        if full or model_inputs or any(name in ("Cargo.toml", "Cargo.lock", "rust-toolchain.toml") for name in changed):
            owners = {path.name for path in (root / "crates").iterdir() if (path / "Cargo.toml").is_file()}
        manifests = {path.parent.name: tomllib.loads(path.read_text(encoding="utf-8"))
                     for path in (root / "crates").glob("*/Cargo.toml")}
        affected = {manifests[owner]["package"]["name"] for owner in owners if owner in manifests}
        while True:
            expanded = affected | {value["package"]["name"] for value in manifests.values()
                                   if any(affected.intersection(value.get(section, {}))
                                          for section in ("dependencies", "dev-dependencies", "build-dependencies"))}
            if expanded == affected:
                break
            affected = expanded
        if not wasm:
            affected.discard("ntesim-wasm")
        if not affected:
            raise ValueError("cannot select affected Rust crates")
        args = ["python", "tools/cargo.py", "test", "--locked", "--offline"]
        for package in sorted(affected):
            args += ["-p", package]
        commands.append(check("rust-tests", args))
        if rust_source:
            commands.append(check("rust-quality", ["python", "tools/cargo.py", "clippy", "--workspace",
                                                   "--all-targets", "--locked", "--offline", "--", "-D", "warnings"]))
    if runtime or data or scenarios or wasm:
        commands.append(check("native-build", ["python", "tools/cargo.py", "build", "-p", "ntesim-cli", "--locked", "--offline"]))
    if wasm:
        commands.append(check("wasm-build", ["python", "tools/cargo.py", "build", "-p", "ntesim-wasm", "--release",
                                            "--target", "wasm32-unknown-unknown", "--locked", "--offline"]))
    if browser:
        # Full parity is retained for web changes and explicit integration runs.
        commands.append(check("parity-inputs", ["python", "tools/verification_engine.py", "--prepare-batch-inputs"]))
        commands.append(check("native-wasm-parity", ["python", "tools/verify_wasm_parity.py", "--output",
                                                    "cache/verification/checkpoint-wasm-parity.json"]))
    if wasm and not browser:
        commands.append(check("wasm-smoke", ["node", "tools/web.mjs", "test",
                                             "tests/wasm_contract.test.mjs", "tests/wasm_loading.test.mjs"]))
    if browser:
        commands += [check("web-types", ["node", "tools/web.mjs", "check"]),
                     check("web-format", ["node", "tools/web.mjs", "format-check"]),
                     check("web-tests", ["node", "tools/web.mjs", "test"],
                           {"NTESIM_VERIFY_NATIVE": "1", "NTESIM_VERIFY_WEAPON": "",
                            "NTESIM_TEST_LOOPBACK": ""})]
    return commands


def full_check_plan(root, changed):
    return check_plan(root, changed, full=True)


def covers_check(completed, required):
    if completed == {**required, "exit_code": 0}:
        return True
    if required["name"] == "wasm-smoke" and completed.get("name") == "web-tests":
        return (completed.get("exit_code") == 0
                and completed.get("command") == ["node", "tools/web.mjs", "test"]
                and completed.get("env") == {"NTESIM_VERIFY_NATIVE": "1", "NTESIM_VERIFY_WEAPON": "",
                                             "NTESIM_TEST_LOOPBACK": ""})
    if completed.get("name") != "rust-tests" or required["name"] != "rust-tests" or completed.get("exit_code") != 0:
        return False
    if completed.get("env") != required["env"]:
        return False
    command = completed.get("command", [])
    expected = required["command"]
    if command[:5] != expected[:5] or len(command[5:]) % 2 or any(arg != "-p" for arg in command[5::2]):
        return False
    return set(expected[6::2]).issubset(command[6::2])


def check_inputs(item, files):
    name = item["name"]
    if name in ("repository-privacy", "local-links-and-formats", "structured-syntax", "markdown-links"):
        return None
    if name == "native-build":
        return package_test_inputs(ROOT, "ntesim-cli", files)
    if name == "wasm-build":
        return package_test_inputs(ROOT, "ntesim-wasm", files)
    if name == "rust-format":
        prefixes = ("crates/",)
        exact = {"Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "rustfmt.toml", ".rustfmt.toml"}
    elif name == "rust-quality":
        prefixes = ("crates/",)
        exact = {"Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "tests/test_architecture.py",
                 "tools/cargo.py"}
    elif name in ("web-types", "web-format"):
        prefixes = ("web/", "crates/wasm/js/", "examples/", "presets/")
        exact = {path for path in files if path.startswith("tools/") and path.endswith((".mjs", ".js", ".mts"))}
    elif name == "rust-tests":
        prefixes = ("crates/", "data/", "examples/", "presets/", "fixtures/", "research/", "reviews/")
        exact = {"Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "tools/cargo.py"}
    elif name.startswith("oracle:") or name == "native-wasm-parity":
        prefixes = ("crates/", "data/", "examples/", "presets/", "fixtures/", "research/", "reviews/")
        exact = {"Cargo.toml", "Cargo.lock", "rust-toolchain.toml"}
        exact.update(path for path in files if path.startswith("tools/") and path.endswith(".py"))
    elif name in ("web-tests", "wasm-smoke"):
        prefixes = ("web/", "crates/", "data/", "examples/", "presets/", "fixtures/",
                    "research/", "reviews/", "tests/fixtures/", "licenses/")
        exact = {"Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "tools/cargo.py", "LICENSE",
                 "tools/browser_distribution.py", "tools/native_distribution.py", "tools/wasm_bindings.py", "tools/studio_build.py"}
        exact.update(path for path in files if path.startswith("tools/") and path.endswith((".mjs", ".js", ".mts")))
    elif name == "python-tests":
        return files
    else:
        return None
    exact.update(("tools/checkpoint.py", "tools/cpu_limit.py", "tools/tool_output.py"))
    if name in ("web-types", "web-format"):
        exact.add("tools/studio_build.py")
    return {path: value for path, value in files.items() if path in exact or path.startswith(prefixes)}


def artifact_inputs(root, item):
    if item["name"] not in ("web-tests", "wasm-smoke", "native-wasm-parity", "native-build", "wasm-build") and not item["name"].startswith("oracle:"):
        return {}
    native = "target/debug/ntesim.exe" if os.name == "nt" else "target/debug/ntesim"
    names = (native,)
    if item["name"] not in ("oracle:tools/verify_game_runner_cli.py", "native-build"):
        names += ("target/wasm-bindgen/ntesim_wasm_bg.wasm", "target/wasm-bindgen/bindings.js")
    if item["name"] == "native-wasm-parity":
        names += ("cache/reviews/q10-research-batch/pilot-request.json",)
    return {name: hashlib.sha256((root / name).read_bytes()).hexdigest() if (root / name).is_file()
            else None for name in names}


def check_environment(root):
    # Desktop command sandboxes supply new temp/AppData paths on every invocation.
    # Use the same local paths for execution and fingerprints, not ignored cache inputs.
    temporary = str(root.resolve() / "cache/tmp")
    env = {**os.environ, "TEMP": temporary, "TMP": temporary}
    if os.name == "nt":
        env["LOCALAPPDATA"] = str(root.resolve() / "cache/checkpoint/localappdata")
    return env


def check_environment_fingerprint(root):
    env = check_environment(root)
    ephemeral_prefixes = ("ANTIGRAVITY_", "EFC_", "RPC_")
    ephemeral_suffixes = ("PORT", "_PORT")
    filtered = {}
    for key, value in env.items():
        if key == "NTESIM_TOOL_VERBOSE" or key.startswith(ephemeral_prefixes) or key.endswith(ephemeral_suffixes):
            continue
        if key in ("_TEMP", "_TMP", "_LOCALAPPDATA", "FPS_BROWSER_APP_PROFILE_STRING", "FPS_BROWSER_USER_PROFILE_STRING"):
            continue
        filtered[key] = value
    return filtered


def check_key(root, item, files):
    inputs = check_inputs(item, files)
    if inputs is None:
        return None
    binary = shutil.which(sys.executable if item["command"][0] == "python" else item["command"][0])
    identity = None
    if binary:
        stat = Path(binary).stat()
        identity = [str(Path(binary).resolve()), stat.st_size, stat.st_mtime_ns]
    environment = check_environment_fingerprint(root)
    value = {"version": VERSION, "root": str(root.resolve()), "check": item, "files": inputs,
             "artifacts": artifact_inputs(root, item), "environment": environment,
             "python": sys.version, "binary": identity}
    return hashlib.sha256(json.dumps(value, sort_keys=True).encode("utf-8")).hexdigest()


def load_successes(path):
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
        return value if isinstance(value, dict) else {}
    except (OSError, ValueError):
        return {}


def save_json(path, value):
    temporary = path.with_name(f"{path.stem}-{uuid4().hex}.json")
    temporary.write_text(json.dumps(value, ensure_ascii=False, sort_keys=True) + "\n", encoding="utf-8")
    temporary.replace(path)


def markdown_anchors(text):
    anchors, counts = set(), {}
    for heading in re.findall(r"^#{1,6}\s+(.+?)\s*#*\s*$", without_fences(text), re.MULTILINE):
        slug = re.sub(r"[^\w\- ]", "", heading.lower()).replace(" ", "-")
        count = counts.get(slug, 0)
        anchors.add(slug + (f"-{count}" if count else ""))
        counts[slug] = count + 1
    return anchors


def repository_markdown_files(root):
    excluded = {"cache", ".cache", ".git", "node_modules", "target"}
    for directory, children, files in os.walk(root):
        children[:] = [child for child in children if child not in excluded]
        for filename in files:
            if filename.lower().endswith(".md"):
                yield (Path(directory) / filename).relative_to(root).as_posix()


def without_fences(text):
    return re.sub(r"^(`{3,}|~{3,}).*?^\1\s*$", "", text, flags=re.MULTILINE | re.DOTALL)


def syntax_checks(root, names):
    for name in sorted(names):
        path = root / name
        if not path.is_file():
            continue
        text = path.read_text(encoding="utf-8-sig")
        if path.suffix == ".json":
            json.loads(text)
        elif path.suffix == ".toml":
            tomllib.loads(text)


def check_links(root, paths=None):
    if paths:
        md_paths = [Path(p) if Path(p).is_absolute() else root / p for p in paths if str(p).lower().endswith(".md")]
    else:
        md_paths = [root / p for p in repository_markdown_files(root)]
    errors = []
    anchor_cache = {}
    for path in sorted(md_paths):
        if not path.exists():
            continue
        rel = path.relative_to(root).as_posix() if path.is_relative_to(root) else str(path)
        try:
            text = without_fences(path.read_text(encoding="utf-8-sig"))
        except OSError as e:
            errors.append(f"{rel}: failed to read: {e}")
            continue
        for link in re.findall(r"!?\[[^\]]*\]\(([^\n]+?)\)", text):
            destination = link[1:link.find(">")] if link.startswith("<") else link.split()[0]
            parsed = urlsplit(destination)
            if parsed.scheme or parsed.netloc:
                continue
            target = (path.parent / unquote(parsed.path)).resolve() if parsed.path else path
            if not target.exists():
                errors.append(f"{rel}: missing local link target {destination}")
                continue
            if parsed.fragment and target.suffix.lower() == ".md":
                if target not in anchor_cache:
                    try:
                        anchor_cache[target] = markdown_anchors(target.read_text(encoding="utf-8"))
                    except OSError as e:
                        errors.append(f"{rel}: cannot read anchor target {destination}: {e}")
                        continue
                if unquote(parsed.fragment) not in anchor_cache[target]:
                    errors.append(f"{rel}: missing local anchor {destination}")
    if errors:
        raise ValueError("\n".join(errors))


def privacy_checks(root):
    """Check commit-eligible inputs, not ignored private files or Git history."""
    names = set(git(root, "ls-files", "--cached", "--others", "--exclude-standard", "-z").decode().split("\0"))
    patterns = {
        "machine path": re.compile(r"(?i)(?<![a-z0-9_])[a-z]:[/\\]+[^\s\"'`<>]*|/(?:Users|home|mnt|Volumes)/[^\s\"'`<>]+|"
                                   + re.escape(root.resolve().as_posix())),
        "credential literal": re.compile(r"(?<![A-Za-z0-9_])(?:gh[pousr]_[A-Za-z0-9]{20,}|AKIA[A-Z0-9]{16}|sk-(?:proj-)?[A-Za-z0-9_-]{24,})(?![A-Za-z0-9_])|-----BEGIN (?:[A-Z]+ )?PRIVATE KEY-----"),
        "credential assignment": re.compile(r"(?i)\b[A-Z_]*(?:API_?KEY|ACCESS_?TOKEN|PASSWORD|CLIENT_?SECRET)[A-Z_]*[\"']?\s*[:=]\s*[\"'][A-Za-z0-9_+/=-]{24,}[\"']"),
        "proxy endpoint": re.compile(r"(?i)(?:https?|socks5?h?)://[^\s\"'`<>]+"),
    }
    errors = []
    for name in sorted(names - {""}):
        path = root / name
        if not path.exists():
            continue
        parts = Path(name).parts
        if any(part in (".local", ".venv", "venv", ".aws", ".qoder") for part in parts) or any(
                part == ".env" or part.startswith(".env.") for part in parts) or name.startswith(".codex/rules/"):
            errors.append(f"{name}: private configuration must remain untracked")
            continue
        if path.is_symlink() or not path.resolve().is_relative_to(root.resolve()):
            raise ValueError("repository privacy check refuses linked or escaping inputs")
        raw = path.read_bytes()
        if b"\0" in raw:
            continue
        for number, line in enumerate(raw.decode("utf-8", errors="replace").splitlines(), 1):
            for category, pattern in patterns.items():
                matches = list(pattern.finditer(line))
                if category == "machine path" and (name.startswith("tests/") or "/tests/" in name):
                    # Single-component sentinels exercise absolute-path rejection, not host layout.
                    matches = [match for match in matches if not re.fullmatch(r"[A-Z]:/(?:secret|drive)", match.group())]
                if category == "proxy endpoint" and not re.search(r"(?i)proxy|gateway|socks", line):
                    continue  # Loopback application servers are portable, not proxy configuration.
                if category == "proxy endpoint":
                    matches = [match for match in matches if not (urlsplit(match.group()).hostname or "").endswith(".invalid")]
                if matches:
                    errors.append(f"{name}:{number}: {category}")
    if errors:
        raise ValueError("\n".join(errors))


def static_checks(root, names):
    names = set(names)
    if any(name.lower().endswith(".md") and not (root / name).exists() for name in names):
        # A deleted document cannot be opened, so check all remaining authored Markdown
        # for links that still point to it.
        names.update(repository_markdown_files(root))
    for name in sorted(names):
        path = root / name
        if not path.exists() and path.suffix.lower() == ".md":
            continue
        text = path.read_text(encoding="utf-8-sig")
        if path.suffix == ".json":
            json.loads(text)
        elif path.suffix == ".toml":
            tomllib.loads(text)
        elif path.suffix.lower() == ".md":
            text = without_fences(text)
            for link in re.findall(r"!?\[[^\]]*\]\(([^\n]+?)\)", text):
                destination = link[1:link.find(">")] if link.startswith("<") else link.split()[0]
                parsed = urlsplit(destination)
                if parsed.scheme or parsed.netloc:
                    continue
                target = (path.parent / unquote(parsed.path)).resolve() if parsed.path else path
                if not target.exists():
                    raise ValueError(f"{name}: missing local link target {destination}")
                if parsed.fragment and target.suffix.lower() == ".md":
                    if unquote(parsed.fragment) not in markdown_anchors(target.read_text(encoding="utf-8")):
                        raise ValueError(f"{name}: missing local anchor {destination}")


def receipt_path(root):
    return root / "cache/checkpoint/receipt.json"


def prepare_hook(root):
    interpreter = Path(sys.executable).as_posix()
    if any(char.isspace() for char in interpreter):
        raise ValueError("native Git hook requires a Python interpreter path without whitespace")
    template = (root / ".githooks/pre-commit").read_bytes().split(b"\n", 1)[1]
    hook = root / "cache/checkpoint/hooks/pre-commit"
    hook.parent.mkdir(parents=True, exist_ok=True)
    hook.write_bytes(f"#!{interpreter}\n".encode("utf-8") + template)
    hook.chmod(0o755)
    return hook


def install_hook(root):
    configured = git(root, "config", "--get", "core.hooksPath", allow_missing=True).decode().strip()
    if configured:
        raise ValueError(f"existing core.hooksPath={configured}; integrate the guard before changing custom hooks")
    prepared = prepare_hook(root)
    directory = Path(git(root, "rev-parse", "--git-path", "hooks").decode().strip())
    directory = directory if directory.is_absolute() else root / directory
    hook = directory / "pre-commit"
    if hook.is_symlink() or directory.is_symlink() or getattr(directory, "is_junction", lambda: False)():
        raise ValueError("refusing to install through a hook-directory/file link")
    content = prepared.read_bytes()
    if hook.exists():
        if hook.read_bytes() != content:
            raise ValueError("existing pre-commit hook differs; integrate it before installation")
        return
    directory.mkdir(parents=True, exist_ok=True)
    hook.write_bytes(content)
    hook.chmod(0o755)


def ensure_hook(root):
    configured = git(root, "config", "--get", "core.hooksPath", allow_missing=True).decode().strip()
    if configured:
        return
    directory = Path(git(root, "rev-parse", "--git-path", "hooks").decode().strip())
    directory = directory if directory.is_absolute() else root / directory
    hook = directory / "pre-commit"
    if not hook.exists():
        install_hook(root)


def run_checkpoint(root, extra=(), execute=run_logged, *, fresh=False, full=False, wasm_smoke=False):
    ensure_hook(root)
    receipt = receipt_path(root)
    receipt.parent.mkdir(parents=True, exist_ok=True)
    receipt.unlink(missing_ok=True)
    environment = check_environment(root)
    for key in ("TEMP", "TMP", "LOCALAPPDATA") if os.name == "nt" else ("TEMP", "TMP"):
        Path(environment[key]).mkdir(parents=True, exist_ok=True)
    head = head_id(root)
    files = worktree_files(root)
    changed = changed_files(head_files(root, head), tree_files(files))
    if not changed and not full and not wasm_smoke:
        raise ValueError("no changes to validate")
    commands = (full_check_plan(root, changed) if full else
                check_plan(root, changed, wasm_smoke=wasm_smoke) if wasm_smoke else
                check_plan(root, changed)) + list(extra)
    cache_path = receipt.with_name("successes.json")
    successes = {} if fresh else load_successes(cache_path)
    if fresh:
        save_json(cache_path, successes)
    timings = []
    started = time.perf_counter()
    status = 0
    completed = {}
    try:
        for item in commands:
            key = check_key(root, item, files)
            if key is not None and successes.get(key) == item:
                timings.append({"name": item["name"], "seconds": 0, "exit_code": 0, "reused": True})
                continue
            successes.pop(key, None)
            command = item["command"]
            if command[0] == "python":
                command = [sys.executable, *command[1:]]
            env = {**environment, **item["env"]}
            if item["name"] == "rust-tests":
                packages = [command[i] for i in range(len(command)) if i > 0 and command[i - 1] == "-p"]
                if packages:
                    dirty_packages = [pkg for pkg in packages
                                      if successes.get(package_check_key(root, pkg, files)) != {"package": pkg, "status": "pass"}]
                    if not dirty_packages:
                        timings.append({"name": item["name"], "seconds": 0, "exit_code": 0, "reused": True})
                        if key is not None:
                            completed[key] = item
                        continue
                    before = time.perf_counter()
                    pkg_status = 0
                    p_index = command.index("-p")
                    base_cmd = command[:p_index]
                    for pkg in dirty_packages:
                        pkg_key = package_check_key(root, pkg, files)
                        successes.pop(pkg_key, None)
                        pkg_command = [*base_cmd, "-p", pkg]
                        pkg_status = execute(pkg_command, cwd=root, env=env, label=f"checkpoint-rust-test-{pkg}")
                        if pkg_status:
                            break
                        completed[pkg_key] = {"package": pkg, "status": "pass"}
                        successes[pkg_key] = {"package": pkg, "status": "pass"}
                    timings.append({"name": item["name"], "seconds": time.perf_counter() - before,
                                    "exit_code": pkg_status})
                    if pkg_status:
                        status = pkg_status
                        break
                    if key is not None:
                        completed[key] = item
                    continue
            before = time.perf_counter()
            status = execute(command, cwd=root, env=env, label="checkpoint-" + item["name"])
            timings.append({"name": item["name"], "seconds": time.perf_counter() - before,
                            "exit_code": status})
            if status:
                break
            if key is not None:
                completed[key] = item
    finally:
        receipt.with_name("timings.json").write_text(json.dumps({
            "head": head, "checks": timings, "seconds": time.perf_counter() - started,
        }, indent=2) + "\n", encoding="utf-8")
    after = worktree_files(root)
    # A changed receipt must still fail, but unrelated passing checks need not be repeated.
    for key, item in completed.items():
        if isinstance(item, dict) and item.get("status") == "pass" and "package" in item:
            if package_check_key(root, item["package"], after) == key:
                successes[key] = item
        elif check_key(root, item, after) == key:
            successes[key] = item
    save_json(cache_path, successes)
    differences = changed_files(files, after)
    if head_id(root) != head or differences:
        raise ValueError("inputs changed during checks: " + (", ".join(differences[:10]) or "HEAD")
                         + "; rerun python tools/checkpoint.py run")
    if status:
        return status
    result = {"version": VERSION, "root": str(root.resolve()), "head": head, "files": files,
              "checks": [{**item, "exit_code": 0} for item in commands]}
    save_json(receipt, result)
    return 0


def verify_checkpoint(root):
    head = head_id(root)
    indexed = index_files(root)
    if indexed == head_files(root, head):
        return
    receipt = receipt_path(root)
    if not receipt.is_file():
        raise ValueError("no successful checkpoint receipt; run python tools/checkpoint.py run")
    saved = json.loads(receipt.read_text(encoding="utf-8"))
    files = worktree_files(root)
    if saved.get("version") != VERSION or saved.get("root") != str(root.resolve()) or saved.get("head") != head:
        raise ValueError("checkpoint belongs to another revision/worktree; rerun python tools/checkpoint.py run")
    if saved.get("files") != files:
        raise ValueError("checkpoint inputs changed; rerun python tools/checkpoint.py run")
    differences = changed_files(indexed, tree_files(files))
    if differences:
        raise ValueError("staged tree differs from tested files; stage the tested changes or isolate unrelated work: "
                         + ", ".join(differences[:10]))
    changed = changed_files(head_files(root, head), indexed)
    required = check_plan(root, changed)
    completed = saved.get("checks", [])
    if any(not any(covers_check(value, item) for value in completed) for item in required) or any(
            item.get("exit_code") != 0 for item in completed):
        raise ValueError("required checks are missing/failed; rerun python tools/checkpoint.py run")


def extra_checks(path):
    values = json.loads(Path(path).read_text(encoding="utf-8"))
    if not isinstance(values, list):
        raise ValueError("extra checks must be a JSON array of command argument arrays")
    result = []
    for index, command in enumerate(values):
        if not isinstance(command, list) or not command or any(not isinstance(arg, str) or not arg for arg in command):
            raise ValueError("each extra check must be a nonempty array of literal arguments")
        result.append(check(f"extra-{index}", command))
    return result


def main():
    parser = argparse.ArgumentParser(description="Validate current files and guard commits with a fresh checkpoint receipt.")
    sub = parser.add_subparsers(dest="mode", required=True)
    plan = sub.add_parser("plan")
    plan.add_argument("--wasm-smoke", action="store_true", help="include the milestone WASM/Worker API smoke check")
    run = sub.add_parser("run")
    run.add_argument("--extra-checks", help="JSON file containing additional command argument arrays")
    run.add_argument("--fresh", action="store_true", help="execute every selected check without reusing successes")
    run.add_argument("--full", action="store_true", help="select the complete regression boundary, including workspace/browser")
    run.add_argument("--wasm-smoke", action="store_true", help="include the milestone WASM/Worker API smoke check")
    sub.add_parser("verify")
    sub.add_parser("prepare-hook")
    sub.add_parser("install-hook")
    sub.add_parser("privacy")
    static = sub.add_parser("static")
    static.add_argument("paths", nargs="+")
    syntax = sub.add_parser("syntax")
    syntax.add_argument("paths", nargs="+")
    links = sub.add_parser("check-links")
    links.add_argument("paths", nargs="*", default=None)
    args = parser.parse_args()
    try:
        if args.mode == "plan":
            head = head_id(ROOT)
            changed = changed_files(head_files(ROOT, head), tree_files(worktree_files(ROOT)))
            print(json.dumps({"changed": changed, "checks": check_plan(ROOT, changed, wasm_smoke=args.wasm_smoke)}, indent=2))
        elif args.mode == "run":
            limit_cpus()
            receipt_path(ROOT).unlink(missing_ok=True)
            return run_checkpoint(ROOT, extra_checks(args.extra_checks) if args.extra_checks else (),
                                  fresh=args.fresh, full=args.full, wasm_smoke=args.wasm_smoke)
        elif args.mode == "verify":
            verify_checkpoint(ROOT)
        elif args.mode == "prepare-hook":
            prepare_hook(ROOT)
        elif args.mode == "install-hook":
            install_hook(ROOT)
        elif args.mode == "privacy":
            privacy_checks(ROOT)
        elif args.mode == "syntax":
            syntax_checks(ROOT, args.paths)
        elif args.mode == "check-links":
            check_links(ROOT, args.paths)
        else:
            static_checks(ROOT, args.paths)
    except (OSError, ValueError, KeyError, TypeError) as error:
        print(f"ntesim checkpoint: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
