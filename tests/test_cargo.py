import importlib.util
import subprocess
import sys
import unittest
from pathlib import Path
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
from verification_workspace import temporary_directory
spec = importlib.util.spec_from_file_location("cargo_helper", ROOT / "tools/cargo.py")
cargo = importlib.util.module_from_spec(spec)
spec.loader.exec_module(cargo)


class CargoToolchainTests(unittest.TestCase):
    def test_cargo_uses_managed_gateway_only_inside_codex_and_preserves_explicit_proxy(self):
        gateway = "https://gateway.example.invalid"
        cases = (({"CODEX_NETWORK_PROXY_ACTIVE": "1", "HTTPS_PROXY": gateway}, gateway),
                 ({"HTTPS_PROXY": gateway}, None),
                 ({"CODEX_NETWORK_PROXY_ACTIVE": "0", "HTTPS_PROXY": gateway}, None),
                 ({"CODEX_NETWORK_PROXY_ACTIVE": "1", "HTTPS_PROXY": gateway,
                   "CARGO_HTTP_PROXY": "caller-proxy"}, "caller-proxy"))
        for environment, expected in cases:
            with self.subTest(environment=environment), \
                 patch.dict(cargo.os.environ, {"PATH": "fixture-path", **environment}, clear=True), \
                 patch.object(cargo.sys, "argv", ["cargo.py", "fetch", "--locked"]), \
                 patch.object(cargo, "resolve_toolchain", return_value=Path("compiler")), \
                 patch.object(cargo.subprocess, "call", return_value=0) as run:
                self.assertEqual(cargo.main(), 0)
                self.assertEqual(run.call_args.kwargs["env"].get("CARGO_HTTP_PROXY"), expected)

    def test_only_successful_wasm_build_generates_versioned_bindings(self):
        for target, status, expected in (("wasm32-unknown-unknown", 0, True),
                                          ("wasm32-unknown-unknown", 2, False),
                                          ("x86_64-pc-windows-msvc", 0, False)):
            with self.subTest(target=target, status=status), \
                 patch.object(cargo.sys, "argv", ["cargo.py", "build", "-p", "ntesim-wasm", "--target", target, "--release"]), \
                 patch.object(cargo, "resolve_toolchain", return_value=Path("compiler")), \
                 patch.object(cargo, "run_logged", return_value=status), \
                 patch("wasm_bindings.generate", return_value=0) as generate:
                self.assertEqual(cargo.main(), status)
                self.assertEqual(generate.called, expected)

    def test_reuses_local_toolchain_without_installing(self):
        env = {"RUSTUP_HOME": "caller-home"}
        compiler = str(ROOT / "cache/rustup/fixture/bin/rustc")
        with patch.object(cargo.subprocess, "run", return_value=subprocess.CompletedProcess([], 0, compiler)) as run:
            with patch.object(cargo.subprocess, "check_output", return_value=f"rustc {cargo.PIN} (fixture)"):
                self.assertEqual(cargo.resolve_toolchain(env, offline=True), Path(compiler).parent)
        self.assertEqual(env["RUSTUP_HOME"], str(ROOT / "cache/rustup"))
        self.assertEqual(run.call_count, 1)
        self.assertEqual(run.call_args.args[0][1], "which")

    def test_falls_back_to_matching_caller_toolchain(self):
        env = {"RUSTUP_HOME": "caller-home"}
        def discover(command, **kwargs):
            if kwargs["env"]["RUSTUP_HOME"] == "caller-home":
                return subprocess.CompletedProcess(command, 0, "caller/bin/rustc")
            return subprocess.CompletedProcess(command, 1, "")
        with patch.object(cargo.subprocess, "run", side_effect=discover) as run:
            with patch.object(cargo.subprocess, "check_output", return_value=f"rustc {cargo.PIN} (fixture)"):
                self.assertEqual(cargo.resolve_toolchain(env, offline=True), Path("caller/bin"))
        self.assertEqual(env["RUSTUP_HOME"], "caller-home")
        self.assertTrue(all(call.args[0][1] == "which" for call in run.call_args_list))

    def test_offline_missing_or_wrong_version_never_installs(self):
        for found, version in ((False, ""), (True, "rustc 0.0.0 (fixture)")):
            with self.subTest(found=found):
                result = subprocess.CompletedProcess([], 0 if found else 1, "other/bin/rustc" if found else "")
                with patch.object(cargo.subprocess, "run", return_value=result) as run:
                    with patch.object(cargo.subprocess, "check_output", return_value=version):
                        with self.assertRaisesRegex(RuntimeError, "offline mode forbids"):
                            cargo.resolve_toolchain({}, offline=True)
                self.assertTrue(all(call.args[0][1] == "which" for call in run.call_args_list))

    def test_online_install_uses_only_local_home_and_is_verified(self):
        installed = False
        def discover(command, **kwargs):
            nonlocal installed
            if command[1:3] == ["toolchain", "install"]:
                self.assertEqual(kwargs["env"]["RUSTUP_HOME"], str(ROOT / "cache/rustup"))
                installed = True
            return subprocess.CompletedProcess(command, 0 if installed else 1, "local/bin/rustc" if installed else "")
        with patch.object(cargo.subprocess, "run", side_effect=discover):
            with patch.object(cargo.subprocess, "check_output", return_value=f"rustc {cargo.PIN} (fixture)"):
                def install(command, **kwargs):
                    return discover(command, **kwargs).returncode
                with patch.object(cargo, "run_logged", side_effect=install):
                    self.assertEqual(cargo.resolve_toolchain({}, offline=False), Path("local/bin"))
        self.assertTrue(installed)

    def test_cargo_offline_flags_and_environment_disable_installation(self):
        for arguments, environment in ((["test", "--offline"], {}), (["test", "--frozen"], {}), (["test"], {"CARGO_NET_OFFLINE": "true"})):
            with self.subTest(arguments=arguments, environment=environment):
                with patch.object(cargo.sys, "argv", ["cargo.py", *arguments]):
                    with patch.object(cargo.os.environ, "copy", return_value=environment.copy()):
                        with patch.object(cargo, "resolve_toolchain", side_effect=RuntimeError("offline mode forbids installation")) as resolve:
                            with patch.object(cargo.sys, "stderr"):
                                self.assertEqual(cargo.main(), 1)
                self.assertTrue(resolve.call_args.args[1])
    def test_check_output_and_payload_contracts(self):
        cases = [
            (["test", "--offline"], False, True),
            (["test", "--verbose", "--offline"], True, True),
            (["test", "--", "--verbose"], False, True),
            (["run", "--", "audit"], False, False),
            (["metadata", "--format-version=1"], False, False),
        ]
        for arguments, verbose, logged in cases:
            with self.subTest(arguments=arguments):
                with patch.object(cargo.sys, "argv", ["cargo.py", *arguments]), \
                     patch.dict(cargo.os.environ, {"PATH": "fixture-path"}, clear=True), \
                     patch.object(cargo, "resolve_toolchain", return_value=Path("compiler")), \
                     patch.object(cargo, "run_logged", return_value=2) as run, \
                     patch.object(cargo.subprocess, "call", return_value=2) as call, \
                     patch("builtins.print") as output:
                    self.assertEqual(cargo.main(), 2)
                    self.assertEqual(run.called, logged)
                    self.assertEqual(call.called, not logged)
                    self.assertEqual(output.called, verbose)
                    if logged:
                        self.assertEqual(run.call_args.kwargs["verbose"], verbose)
                        self.assertEqual("--verbose" in run.call_args.args[0], "--" in arguments)
                        self.assertEqual(run.call_args.kwargs["env"]["CARGO_BUILD_JOBS"], "4")
                        self.assertEqual(run.call_args.kwargs["env"]["RUST_TEST_THREADS"], "4")
        with temporary_directory(prefix="cargo-identity-") as temporary:
            root = Path(temporary)
            source = root / "crates/example/src/lib.rs"
            test = source.parent / "tests/cases.rs"
            preset = root / "presets/research/variants.csv"
            test.parent.mkdir(parents=True)
            preset.parent.mkdir(parents=True)
            source.write_text("pub const VALUE: u32 = 1;", encoding="utf-8")
            test.write_text("// first test", encoding="utf-8")
            preset.write_text("first input", encoding="utf-8")
            with patch.object(cargo, "ROOT", root):
                identity = cargo.engine_source_sha256()
                test.write_text("// changed test", encoding="utf-8")
                self.assertEqual(cargo.engine_source_sha256(), identity)
                source.write_text("pub const VALUE: u32 = 2;", encoding="utf-8")
                self.assertNotEqual(cargo.engine_source_sha256(), identity)
                identity = cargo.engine_source_sha256()
                preset.write_text("changed input", encoding="utf-8")
                self.assertNotEqual(cargo.engine_source_sha256(), identity)


if __name__ == "__main__":
    unittest.main()
