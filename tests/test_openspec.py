import contextlib
import io
import json
from pathlib import Path
import subprocess
import sys
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
import openspec
from verification_workspace import temporary_directory


class OpenSpecRunnerTests(unittest.TestCase):
    def test_installed_requires_exact_pin_and_executable_entrypoint(self):
        with temporary_directory(prefix="openspec-install-") as directory:
            package = Path(directory)
            manifest = package / "package.json"
            entry = package / "bin/openspec.js"
            with patch.object(openspec, "PACKAGE", package):
                self.assertFalse(openspec.installed())
                manifest.write_text(json.dumps({"version": openspec.VERSION}), encoding="utf-8")
                self.assertFalse(openspec.installed())
                entry.parent.mkdir()
                entry.write_text("", encoding="utf-8")
                self.assertTrue(openspec.installed())
                for value in (json.dumps({"version": "0.0.0"}), "invalid JSON"):
                    manifest.write_text(value, encoding="utf-8")
                    self.assertFalse(openspec.installed())

    def test_missing_install_is_actionable_and_never_downloads_during_validation(self):
        with patch.object(openspec, "limit_cpus"), patch.object(openspec.shutil, "which", return_value="node"), \
                patch.object(openspec.subprocess, "run", return_value=subprocess.CompletedProcess([], 0, "v22.12.0\n")), \
                patch.object(openspec, "installed", return_value=False), \
                patch.object(openspec.subprocess, "call") as execute, contextlib.redirect_stderr(io.StringIO()) as error:
            self.assertEqual(openspec.main(["validate", "--all", "--strict"]), 1)
            self.assertIn("python tools/openspec.py setup", error.getvalue())
            execute.assert_not_called()

    def test_forwarded_arguments_environment_and_exit_status(self):
        with patch.object(openspec, "limit_cpus"), patch.object(openspec.shutil, "which", return_value="node"), \
                patch.object(openspec.subprocess, "run", return_value=subprocess.CompletedProcess([], 0, "v24.0.0\n")), \
                patch.object(openspec, "installed", return_value=True), \
                patch.dict(openspec.os.environ, {}, clear=True), \
                patch.object(openspec.subprocess, "call", return_value=2) as execute:
            args = ["show", "characters/nanally", "--type", "spec", "--json"]
            self.assertEqual(openspec.main(args), 2)
            command = execute.call_args.args[0]
            self.assertEqual(command, ["node", str(openspec.PACKAGE / "bin/openspec.js"), *args])
            self.assertEqual(execute.call_args.kwargs["cwd"], ROOT)
            self.assertEqual(execute.call_args.kwargs["env"]["OPENSPEC_CONCURRENCY"], "4")
            self.assertEqual(execute.call_args.kwargs["env"]["OPENSPEC_TELEMETRY"], "0")

    def test_supported_node_floor_and_cached_setup(self):
        for version, expected in (("v22.11.0", 1), ("v22.12.0", 0)):
            with self.subTest(version=version), patch.object(openspec, "limit_cpus"), \
                    patch.object(openspec.shutil, "which", return_value="node"), \
                    patch.object(openspec.subprocess, "run", return_value=subprocess.CompletedProcess([], 0, version)), \
                    patch.object(openspec, "installed", return_value=True), \
                    patch.object(openspec.subprocess, "call") as execute, \
                    contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
                self.assertEqual(openspec.main(["setup"]), expected)
                execute.assert_not_called()


if __name__ == "__main__":
    unittest.main()
