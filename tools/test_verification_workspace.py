from pathlib import Path
import subprocess
import sys
import unittest

from verification_workspace import CACHE, ROOT, output_directory, temporary_directory


class VerificationWorkspaceTests(unittest.TestCase):
    def test_parent_and_subprocess_can_write_and_cleanup_nested_files(self):
        with temporary_directory() as name:
            path = Path(name)
            self.assertEqual(path.parent, CACHE.resolve())
            (path / 'request.json').write_text('parent', encoding='utf-8')
            child = subprocess.run([sys.executable, '-c',
                "from pathlib import Path; import sys; p=Path(sys.argv[1]); "
                "assert (p/'request.json').read_text()=='parent'; "
                "(p/'nested').mkdir(); (p/'nested'/'report.json').write_text('child')",
                name], capture_output=True, text=True)
            self.assertEqual(child.returncode, 0, child.stderr)
            self.assertEqual((path / 'nested/report.json').read_text(), 'child')
        self.assertFalse(path.exists())

    def test_nested_cache_workspace_and_uniqueness(self):
        with temporary_directory() as outer:
            with temporary_directory(dir=outer, prefix='oracle-') as first:
                with temporary_directory(dir=outer, prefix='oracle-') as second:
                    self.assertNotEqual(first, second)
                    self.assertEqual(Path(first).parent, Path(outer))
            self.assertFalse(Path(first).exists())
        self.assertFalse(Path(outer).exists())

    def test_exception_cleanup_preserves_body_error(self):
        with self.assertRaisesRegex(RuntimeError, 'oracle failed'):
            with temporary_directory() as name:
                (Path(name) / 'failed-report.json').write_text('failure')
                raise RuntimeError('oracle failed')
        self.assertFalse(Path(name).exists())

    def test_outside_cache_and_path_prefixes_are_rejected(self):
        with self.assertRaisesRegex(ValueError, 'under workspace cache'):
            with temporary_directory(dir=ROOT):
                self.fail('outside cache accepted')
        for prefix in ['', '../escape', 'one/two', 'one\\two', 'C:escape']:
            with self.subTest(prefix=prefix), self.assertRaisesRegex(ValueError, 'filename component'):
                with temporary_directory(prefix=prefix):
                    self.fail('invalid prefix accepted')

    def test_output_directory_resolves_relative_to_root_and_rejects_escape(self):
        with temporary_directory(prefix='output-test-') as parent:
            output = output_directory(Path(parent).relative_to(ROOT) / 'artifacts')
            self.assertTrue(output.is_dir())
            self.assertTrue(output.is_relative_to(ROOT))
            with self.assertRaisesRegex(ValueError, 'inside the workspace'):
                output_directory(ROOT.parent / 'outside')


if __name__ == '__main__':
    unittest.main()
