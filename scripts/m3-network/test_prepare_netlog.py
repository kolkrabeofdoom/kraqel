import json
from pathlib import Path
import tempfile
import unittest
from prepare_netlog import AUTOPLAY, DEFAULTS, prepare


class PreparationTests(unittest.TestCase):
    def test_refuses_existing_root_and_environment_overrides_before_reading_sources(self):
        with tempfile.TemporaryDirectory(prefix="kraqel-m3-acceptance-") as name:
            root = Path(name)
            with self.assertRaises(ValueError):
                prepare(root, Path("absent"), {})
            root.rmdir()
            with self.assertRaises(ValueError):
                prepare(root, Path("absent"), {"WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS": "--other"})
            self.assertFalse(root.exists())

    def test_preserves_defaults_and_isolates_only_diagnostic_output(self):
        with tempfile.TemporaryDirectory(prefix="kraqel-m3-acceptance-") as name, tempfile.TemporaryDirectory() as source:
            root, wry = Path(name), Path(source)
            root.rmdir()
            (wry / "src/webview2").mkdir(parents=True)
            (wry / "src/webview2/mod.rs").write_text(DEFAULTS + AUTOPLAY, encoding="utf-8")
            (wry / "src/lib.rs").write_text("autoplay: true", encoding="utf-8")
            (wry / "Cargo.toml").write_text('version = "0.57.1"', encoding="utf-8")
            prepare(root, wry, {})
            config = json.loads((root / "diagnostic.conf.json").read_text(encoding="utf-8"))
            self.assertEqual(config["app"]["appDirectoriesOverride"], str(root.resolve()))
            window = config["app"]["windows"][0]
            self.assertNotIn("dataDirectory", window)
            self.assertTrue(window["additionalBrowserArgs"].startswith(DEFAULTS + AUTOPLAY + " --log-net-log="))
            self.assertFalse(config["bundle"]["active"])
            with self.assertRaises(ValueError):
                prepare(root, wry, {})

    def test_rejects_non_temp_root(self):
        with self.assertRaises(ValueError):
            prepare(Path.cwd() / "kraqel-m3-acceptance-refused", Path("absent"), {})


if __name__ == "__main__":
    unittest.main()
