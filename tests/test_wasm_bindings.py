import sys
import unittest
from pathlib import Path
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
import wasm_bindings
from verification_workspace import temporary_directory


class BindingsTests(unittest.TestCase):
    def test_profiling_glue_is_isolated_and_preserves_optimizer_and_symbols(self):
        with temporary_directory(prefix="profiling-bindings-") as directory:
            root = Path(directory)
            generator = root / ("cache/wasm-bindgen/bin/wasm-bindgen" + (".exe" if wasm_bindings.os.name == "nt" else ""))
            generator.parent.mkdir(parents=True)
            generator.touch()
            optimizer = root / ("cache/wasm-opt/bin/wasm-opt" + (".exe" if wasm_bindings.os.name == "nt" else ""))
            optimizer.parent.mkdir(parents=True)
            optimizer.touch()
            normal = root / "target/wasm-bindgen/bindings.js"
            normal.parent.mkdir(parents=True)
            normal.write_text("ordinary sentinel", encoding="utf-8")
            commands = []

            def run(command, **_kwargs):
                commands.append(command)
                if command[0] == str(generator):
                    output = Path(command[command.index("--out-dir") + 1])
                    output.mkdir(parents=True)
                    (output / "ntesim_wasm.js").write_text("const wasm_bindgen = {};", encoding="utf-8")
                    (output / "ntesim_wasm_bg.wasm").touch()
                return 0

            with patch.object(wasm_bindings.subprocess, "check_output", return_value="wasm-bindgen 0.2.128"), \
                 patch.object(wasm_bindings, "run_logged", side_effect=run):
                self.assertEqual(wasm_bindings.generate(root, {}, "profiling"), 0)
            self.assertEqual(normal.read_text(encoding="utf-8"), "ordinary sentinel")
            self.assertIn("--keep-debug", commands[0])
            self.assertIn("wasm32-unknown-unknown/profiling/ntesim_wasm.wasm", Path(commands[0][1]).as_posix())
            self.assertEqual(commands[1][1:3], ["-O3", "-g"])
            self.assertTrue((root / "target/wasm-bindgen-profiling/bindings.js").is_file())

    def test_missing_generator_has_reproducible_local_install_command(self):
        with patch.object(Path, "is_file", return_value=False):
            with self.assertRaisesRegex(RuntimeError, r"--version 0.2.128 .*--root cache/wasm-bindgen"):
                wasm_bindings.generate(ROOT, {})

    def test_mismatched_generator_rejects_before_replacing_artifacts(self):
        with patch.object(Path, "is_file", return_value=True), \
             patch.object(wasm_bindings.subprocess, "check_output", return_value="wasm-bindgen 0.0.0"), \
             patch.object(wasm_bindings, "run_logged") as run:
            with self.assertRaisesRegex(RuntimeError, "generator mismatch"):
                wasm_bindings.generate(ROOT, {})
            run.assert_not_called()


if __name__ == "__main__":
    unittest.main()
