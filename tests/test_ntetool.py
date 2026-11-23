"""Tests for ntetool unified CLI and verification dispatcher."""
import os
from pathlib import Path
import subprocess
import sys
import unittest

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
import ntetool
from verification_workspace import temporary_directory


class NtetoolTests(unittest.TestCase):
    def test_discover_kits(self):
        kits = ntetool.discover_kits()
        self.assertIsInstance(kits, dict)

    def test_discover_weapons(self):
        weapons = ntetool.discover_weapons()
        self.assertIsInstance(weapons, dict)

    def test_categorize(self):
        self.assertEqual(ntetool.categorize(Path("verify_mint_kit.py")), "kits")
        self.assertEqual(ntetool.categorize(Path("verify_bit_game_sources.py")), "sources")
        self.assertEqual(ntetool.categorize(Path("verify_s1_fixture.py")), "scenarios")
        self.assertEqual(ntetool.categorize(Path("verify_weapon_cli.py")), "mechanics")
        self.assertEqual(ntetool.categorize(Path("verify_far_side_moon.py")), "mechanics")
        self.assertEqual(ntetool.categorize(Path("verify_lifecycle_cli.py")), "runtime")

    def test_check_file_syntax_valid_and_invalid(self):
        with temporary_directory(prefix='ntetool-syntax-') as tmpdir:
            tmp = Path(tmpdir)
            good_json = tmp / "good.json"
            good_json.write_text('{"valid": [1, 2, 3]}', encoding="utf-8")
            good_toml = tmp / "good.toml"
            good_toml.write_text('val = 42\n', encoding="utf-8")
            bad_json = tmp / "bad.json"
            bad_json.write_text('{"broken": ', encoding="utf-8")

            errors = ntetool.check_file_syntax(tmp, [good_json, good_toml])
            self.assertEqual(errors, [])

            errors = ntetool.check_file_syntax(tmp, [bad_json])
            self.assertEqual(len(errors), 1)
            self.assertIn("syntax error", errors[0])

    def test_check_markdown_links_valid_and_broken(self):
        with temporary_directory(prefix='ntetool-markdown-') as tmpdir:
            tmp = Path(tmpdir)
            doc_a = tmp / "doc_a.md"
            doc_b = tmp / "doc_b.md"
            doc_a.write_text("# Doc A\n\nLink to [Doc B](doc_b.md#section-b)\n", encoding="utf-8")
            doc_b.write_text("# Doc B\n\n## Section B\nContent here.\n", encoding="utf-8")

            errors = ntetool.check_markdown_links(tmp, [doc_a, doc_b])
            self.assertEqual(errors, [])

            # Broken anchor
            doc_broken = tmp / "broken.md"
            doc_broken.write_text("[Missing Section](doc_b.md#nonexistent)\n", encoding="utf-8")
            errors = ntetool.check_markdown_links(tmp, [doc_broken])
            self.assertEqual(len(errors), 1)
            self.assertIn("missing local anchor", errors[0])

            # Missing target file
            doc_missing = tmp / "missing_target.md"
            doc_missing.write_text("[Nowhere](does_not_exist.md)\n", encoding="utf-8")
            errors = ntetool.check_markdown_links(tmp, [doc_missing])
            self.assertEqual(len(errors), 1)
            self.assertIn("missing local link target", errors[0])

    def test_cli_execution_help_and_lists(self):
        for cmd in (
            ["--help"],
            ["verify", "--list"],
            ["verify", "kit", "--list"],
            ["verify", "weapon", "--list"],
            ["verify-kit", "--list"],
            ["verify-weapon", "--list"],
        ):
            with self.subTest(cmd=cmd):
                result = subprocess.run(
                    [sys.executable, str(ROOT / "tools/ntetool.py"), *cmd],
                    cwd=ROOT,
                    capture_output=True,
                    text=True,
                    encoding="utf-8",
                )
                self.assertEqual(result.returncode, 0)


if __name__ == "__main__":
    unittest.main()
