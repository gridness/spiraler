import hashlib
import tempfile
import unittest
from pathlib import Path

from update_homebrew import update_tap


class TapPublicationTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        self.published = self.root / "published"
        self.published.mkdir()
        self.tap = self.root / "tap"
        self.assets = {}
        for suffix in ["aarch64.dmg", "aarch64.AppImage", "x86_64.AppImage"]:
            data = f"published bytes for {suffix}".encode()
            self.assets[suffix] = data
            (self.published / f"Spiraler_0.1.4_{suffix}").write_bytes(data)

    def test_both_definitions_hash_the_published_architecture_assets(self):
        update_tap("0.1.4", "gridness/spiraler", self.published, self.tap)
        cask = (self.tap / "Casks/spiraler.rb").read_text()
        formula = (self.tap / "Formula/spiraler.rb").read_text()
        for suffix, data in self.assets.items():
            digest = hashlib.sha256(data).hexdigest()
            self.assertIn(digest, cask)
            if suffix.endswith("AppImage"):
                self.assertIn(digest, formula)
        self.assertIn("on_linux do", cask)
        self.assertIn("depends_on :linux", formula)
        self.assertNotIn("@VERSION@", cask + formula)

    def test_an_older_run_cannot_downgrade_either_package(self):
        cask = self.tap / "Casks/spiraler.rb"
        cask.parent.mkdir(parents=True)
        cask.write_text('cask "spiraler" do\n  version "0.1.5"\nend\n')
        original = cask.read_bytes()
        update_tap("0.1.4", "gridness/spiraler", self.published, self.tap)
        self.assertEqual(cask.read_bytes(), original)
        self.assertFalse((self.tap / "Formula/spiraler.rb").exists())

    def test_missing_architecture_asset_writes_no_partial_tap_update(self):
        (self.published / "Spiraler_0.1.4_aarch64.AppImage").unlink()
        with self.assertRaises(FileNotFoundError):
            update_tap("0.1.4", "gridness/spiraler", self.published, self.tap)
        self.assertFalse(self.tap.exists())

    def test_version_and_repository_cannot_inject_ruby_or_paths(self):
        for version, repo in [
            ("../0.1.4", "gridness/spiraler"),
            ("0.1.4", 'gridness/spiraler";system("false")'),
        ]:
            with self.assertRaises(ValueError):
                update_tap(version, repo, self.published, self.tap)
        self.assertFalse(self.tap.exists())


if __name__ == "__main__":
    unittest.main()
