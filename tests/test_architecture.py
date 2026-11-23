"""Protect portable engine boundaries independently of Cargo compilation."""
import re
import json
import tomllib
import unittest
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / 'tools'))
from content_boundary import DISPATCH, module_path, owner_references, reexports, reference_growth
from verification_workspace import temporary_directory


class ArchitectureTests(unittest.TestCase):
    def test_simulation_hit_pipeline_and_energy_have_no_owner_coupling(self):
        source_root = ROOT / 'crates/content/src'
        exports = reexports(source_root)
        allowed_character_prefixes = (
            'crate::characters::team::',
            'crate::characters::packet_semantics::',
        )
        for path in (source_root / 'simulation/hit').rglob('*.rs'):
            if path.name == 'tests.rs' or 'tests' in path.relative_to(source_root).parts:
                continue
            refs = owner_references(path.read_text(encoding='utf-8'),
                                    module=module_path(path, source_root), exports=exports)
            for symbol in refs:
                self.assertFalse(symbol.startswith('crate::weapons::'),
                                 f"{path.name} must not import weapons: {symbol}")
                self.assertFalse(symbol.startswith('crate::sets::'),
                                 f"{path.name} must not import sets: {symbol}")
                if symbol.startswith('crate::characters::'):
                    self.assertTrue(symbol.startswith(allowed_character_prefixes),
                                    f"{path.name} must not import specific character kits: {symbol}")

        energy_path = source_root / 'simulation/energy.rs'
        refs = owner_references(energy_path.read_text(encoding='utf-8'),
                                module=module_path(energy_path, source_root), exports=exports)
        for symbol in refs:
            self.assertFalse(symbol.startswith('crate::characters::'),
                             f"energy.rs must not import characters: {symbol}")
            self.assertFalse(symbol.startswith('crate::weapons::'),
                             f"energy.rs must not import weapons: {symbol}")
            self.assertFalse(symbol.startswith('crate::sets::'),
                             f"energy.rs must not import sets: {symbol}")

    def test_owner_ratchet_handles_aliases_groups_reexports_and_test_only_items(self):
        source = '''
        #[cfg(test)] use crate::characters::edgar as test_owner;
        use crate::{weapons::{dustbin as bin, yaodao::is_separate_packet as separate},
                    sets::fireflies};
        use crate as content;
        fn production() { bin::on_break_reduced(); separate(); fireflies::hit::on_team_hit();
                          content::characters::fadia::skill::validate_admission(); }
        #[cfg(test)] mod tests { mod nested { fn test() { crate::weapons::hidden(); } } }
        mod live { fn production() { crate::sets::hp_suit::BUFF_ID; } }
        // crate::weapons::comment(); /* crate::characters::comment */
        const TEXT: &str = r#"crate::sets::string"#;
        fn export_consumer() { crate::bridge::validate(); }
        '''
        exports = {('crate', 'bridge', 'validate'): ('crate', 'characters', 'fadia', 'skill', 'validate_admission')}
        refs = owner_references(source, exports=exports)
        self.assertEqual(refs, {
            'crate::characters::fadia::skill::validate_admission': 2,
            'crate::sets::fireflies': 1,
            'crate::sets::fireflies::hit::on_team_hit': 1,
            'crate::sets::hp_suit::BUFF_ID': 1,
            'crate::weapons::dustbin': 1,
            'crate::weapons::dustbin::on_break_reduced': 1,
            'crate::weapons::yaodao::is_separate_packet': 2,
        })
        self.assertEqual(owner_references('fn new_file() { crate::weapons::dustbin::rule(); }'),
                         {'crate::weapons::dustbin::rule': 1})
        self.assertEqual(owner_references('use super::super::weapons as gear; gear::dustbin::rule();',
                                         module=('crate', 'simulation', 'nested')),
                         {'crate::weapons': 1, 'crate::weapons::dustbin::rule': 1})
        new_refs = {'crate::weapons::dustbin::rule': 1}
        self.assertEqual(reference_growth({'new.rs': new_refs}, {}), {'new.rs': new_refs})
        self.assertEqual(reference_growth({'old.rs': {**new_refs, 'crate::sets::new': 1}},
                                          {'old.rs': {'references': new_refs}}),
                         {'old.rs': {'crate::sets::new': 1}})
        with temporary_directory(prefix='owner-exports-') as directory:
            root = Path(directory)
            (root / 'bridge.rs').write_text('pub use inner::rule as validate;')
            (root / 'bridge').mkdir()
            (root / 'bridge/inner.rs').write_text('pub use crate::{characters::edgar::skill::on_cast as rule};')
            (root / 'lib.rs').write_text('#[cfg(test)] mod tests { pub use crate::weapons::hidden; }')
            exports = reexports(root)
            self.assertEqual(owner_references('crate::bridge::validate();', exports=exports),
                             {'crate::characters::edgar::skill::on_cast': 1})
            self.assertNotIn(('crate', 'tests', 'hidden'), exports)

    def test_studio_has_no_shadow_source_frontend_or_combat_suite(self):
        for folder in ('web/src/domain/language', 'web/tests/combat'):
            path = ROOT / folder
            self.assertFalse(path.exists() and any(item.is_file() for item in path.rglob('*')),
                             f'{folder}: language and mechanics belong to Rust')
        for path in (ROOT / 'web/src').rglob('*.ts*'):
            with self.subTest(file=path.relative_to(ROOT)):
                self.assertNotRegex(path.read_text(encoding='utf-8'),
                                    r'class ExactNumber|class Parser\b|function compileNte\b|6364136223846793005')

    def test_ordinary_hit_energy_keeps_one_receipt_shape(self):
        model = '\n'.join(path.read_text(encoding='utf-8')
                          for path in (ROOT / 'crates/content/src/simulation/model').glob('*.rs'))
        names = re.findall(r'pub struct (\w*HitEnergyBasis)\b', model)
        self.assertEqual(names, ['OrdinaryHitEnergyBasis'])
        self.assertNotRegex(model, r'pub\s+\w+_hit_basis\s*:')

    def test_equipment_projection_owners_are_registered_once(self):
        registration = (ROOT / 'crates/content/src/build/gear.rs').read_text(encoding='utf-8')
        table = registration.split('const WEAPON_ADAPTERS:', 1)[1].split('\n];', 1)[0]
        for path in (ROOT / 'crates/content/src/weapons').glob('*/projection.rs'):
            with self.subTest(owner=path.parent.name):
                self.assertEqual(table.count(f'crate::weapons::{path.parent.name}::projection::project'), 1)

    def test_settlement_and_equipment_dispatch_keep_owner_rules_out(self):
        for path in (ROOT / 'crates/content/src/simulation/hit').glob('*.rs'):
            with self.subTest(file=path.name):
                self.assertNotRegex(path.read_text(encoding='utf-8'), r'character:\d|GE_Player_')
        routing = (ROOT / 'crates/content/src/gear/validation.rs').read_text(encoding='utf-8')
        self.assertNotRegex(routing, r'weapon:fork_|set:Suit')
        studio = (ROOT / 'crates/content/src/build/studio.rs').read_text(encoding='utf-8')
        self.assertNotRegex(studio, r'crate::characters::\w+::primary_packet')

    def test_ordinary_verifiers_keep_caches_inside_the_product(self):
        for path in (ROOT / "tools").glob("verify_*.py"):
            with self.subTest(file=path.relative_to(ROOT)):
                self.assertNotRegex(path.read_text(encoding="utf-8"), r"ROOT\.parent[^\n]*cache")

    def test_dependencies_flow_toward_shared_services(self):
        allowed = {
            "contracts": set(),
            "core": {"contracts"},
            "data": {"contracts"},
            "content": {"contracts", "core", "data"},
            "research": {"contracts", "core", "data", "content"},
            "cli": {"contracts", "core", "data", "content", "research"},
            "wasm": {"contracts", "core", "data", "content", "research"},
        }
        for crate, dependencies in allowed.items():
            manifest = tomllib.loads((ROOT / "crates" / crate / "Cargo.toml").read_text(encoding="utf-8"))
            for section in ("dependencies", "dev-dependencies", "build-dependencies"):
                actual = {name.removeprefix("ntesim-") for name in manifest.get(section, {}) if name.startswith("ntesim-")}
                with self.subTest(crate=crate, section=section):
                    self.assertLessEqual(actual, dependencies)

    def test_shared_services_have_no_io_clock_or_game_content(self):
        forbidden = re.compile(r"\b(?:SystemTime|Instant)\b|std::(?:fs|net|process|time)|\b(?:character|weapon|set):\d|\b(?:GA|GE)_[A-Z]")
        for path in (ROOT / "crates/core/src").rglob("*.rs"):
            if path.name == "tests.rs" or "tests" in path.parts:
                continue
            source = "\n".join(line for line in path.read_text(encoding="utf-8").splitlines() if not line.lstrip().startswith("//"))
            with self.subTest(file=path.relative_to(ROOT)):
                self.assertIsNone(forbidden.search(source))

    def test_core_has_no_spatial_geometry_or_coordinate_modules(self):
        # Q0.3 (Review 2.3): Core must not contain 3D geometry modules or trajectory primitives
        for path in (ROOT / "crates/core/src").rglob("*.rs"):
            self.assertNotEqual(path.name, "geometry.rs", f"geometry modules are forbidden in core: {path}")
        forbidden = re.compile(r"\b(?:MovementSegment|evaluate_position|check_coverage)\b")
        for path in (ROOT / "crates/core/src").rglob("*.rs"):
            if path.name == "tests.rs" or "tests" in path.parts:
                continue
            source = "\n".join(line for line in path.read_text(encoding="utf-8").splitlines() if not line.lstrip().startswith("//"))
            with self.subTest(file=path.relative_to(ROOT)):
                self.assertIsNone(forbidden.search(source), f"Spatial geometry primitive found in {path}")

    def test_source_brands_do_not_name_engine_modules_or_identifiers(self):
        forbidden = re.compile("".join(["p", "r", "y", "d", "w", "e", "n"]) + r"|DriveCalculator", re.IGNORECASE)
        for path in (ROOT / "crates").rglob("*.rs"):
            with self.subTest(file=path.relative_to(ROOT)):
                self.assertIsNone(forbidden.search(path.name))
                # Immutable historical locations belong only in package provenance metadata.
                if path == ROOT / "crates/data/src/package.rs":
                    continue
                code = "\n".join(line for line in path.read_text(encoding="utf-8").splitlines() if not line.lstrip().startswith("//"))
                self.assertIsNone(forbidden.search(code))


if __name__ == "__main__":
    unittest.main()
