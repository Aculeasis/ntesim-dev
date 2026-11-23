from pathlib import Path
import sys
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
import python_tests
from verification_workspace import temporary_directory


class PythonRunnerTests(unittest.TestCase):
    def test_real_discovery_preserves_failures_and_isolates_module_environment(self):
        with temporary_directory(prefix="python-runner-") as directory:
            root = Path(directory)
            first = root / "test_first.py"
            first.write_text("import os, unittest\nos.environ['NTESIM_FIXTURE_LEAK']='1'\n"
                             "class Fixture(unittest.TestCase):\n"
                             " def test_ok(self): self.assertEqual(2+2,4)\n", encoding="utf-8")
            second = root / "test_second.py"
            second.write_text("import os, unittest\nclass Fixture(unittest.TestCase):\n"
                              " def test_isolation(self): self.assertNotIn('NTESIM_FIXTURE_LEAK',os.environ)\n",
                              encoding="utf-8")
            self.assertEqual(python_tests.run_files([first, second], root), 0)
            second.write_text("import unittest\nclass Fixture(unittest.TestCase):\n"
                              " def test_failure(self): self.fail('independent sentinel')\n", encoding="utf-8")
            code, output = python_tests.run_file(second, root)
            self.assertEqual(code, 1)
            self.assertIn(b"independent sentinel", output)

    def test_scheduler_caps_workers_and_preserves_incomplete_status(self):
        with patch.object(python_tests, "ThreadPoolExecutor") as pool, \
             patch.object(python_tests.sys, "stderr"):
            pool.return_value.__enter__.return_value.map.return_value = [(0, b""), (2, b"incomplete")]
            self.assertEqual(python_tests.run_files([Path("one.py"), Path("two.py")], workers=99), 2)
            pool.assert_called_once_with(max_workers=4)


if __name__ == "__main__":
    unittest.main()
