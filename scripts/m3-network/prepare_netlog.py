"""Prepare an isolated diagnostic build, preserving the locked wry defaults.

No environment, registry, firewall or production configuration is changed.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import tempfile

DEFAULTS = "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection"
AUTOPLAY = " --autoplay-policy=no-user-gesture-required"
OVERRIDES = ("WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS", "WEBVIEW2_USER_DATA_FOLDER",
             "WEBVIEW2_BROWSER_EXECUTABLE_FOLDER")


def prepare(root, wry, environment):
    root = root.resolve()
    if root.parent != Path(tempfile.gettempdir()).resolve() or not root.name.startswith("kraqel-m3-acceptance-"):
        raise ValueError("Use a fresh direct TEMP child named kraqel-m3-acceptance-...")
    if root.exists() or any(environment.get(name) for name in OVERRIDES):
        raise ValueError("Existing output or WebView2 environment override requires separate review")
    lock = (Path(__file__).resolve().parents[2] / "Cargo.lock").read_text(encoding="utf-8")
    if not re.search(r'\[\[package\]\]\nname = "wry"\nversion = "0\.57\.1"\n', lock):
        raise ValueError("Review the new locked wry version before preparing diagnostics")
    source = (wry / "src/webview2/mod.rs").read_bytes()
    library = (wry / "src/lib.rs").read_text(encoding="utf-8")
    manifest = (wry / "Cargo.toml").read_text(encoding="utf-8")
    # additionalBrowserArgs replaces defaults. Refuse an unreviewed dependency revision.
    if 'version = "0.57.1"' not in manifest or DEFAULTS not in source.decode("utf-8") or AUTOPLAY not in source.decode("utf-8") or "autoplay: true" not in library:
        raise ValueError("Review the locked wry defaults before using this diagnostic")
    config = {
        "$schema": "https://schema.tauri.app/config/2",
        "productName": "KRAQEL WebView2 NetLog",
        "identifier": "org.kraqel.desktop.network-test",
        "app": {"appDirectoriesOverride": str(root), "windows": [{
            "title": "KRAQEL WebView2 NetLog", "width": 1180, "height": 780,
            "minWidth": 760, "minHeight": 560,
            "additionalBrowserArgs": DEFAULTS + AUTOPLAY + ' --log-net-log="' + str(root / "netlog.json") + '" --net-log-capture-mode=Default',
        }]},
        "bundle": {"active": False},
    }
    root.mkdir()
    (root / "raw").mkdir()
    (root / "diagnostic.conf.json").write_text(json.dumps(config, indent=2) + "\n", encoding="utf-8")
    (root / "diagnostic-plan.json").write_text(json.dumps({
        "wryVersion": "0.57.1", "wrySourceSha256": hashlib.sha256(source).hexdigest(),
        "changes": "Only Default NetLog logging added to existing wry defaults.",
        "rawLogsRemainLocal": True, "existingEnvironmentOverrides": False,
        "targetIdleSeconds": 90, "separateUserDataDirectory": True,
    }, indent=2) + "\n", encoding="utf-8")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("root", type=Path)
    parser.add_argument("--wry-source", type=Path, required=True)
    args = parser.parse_args()
    try:
        prepare(args.root, args.wry_source, os.environ)
    except (ValueError, OSError):
        raise SystemExit("Preparation refused; check fresh TEMP root, environment and wry 0.57.1 sources") from None
    print("Prepared isolated diagnostic configuration; raw logs must remain local.")
