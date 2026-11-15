"""Read-only synthetic Tauri/SQLite verification. Local journals/ETW stay outside Git."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import sqlite3
import sys
import uuid


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def verify_run(run):
    root = Path(run["root"])
    assert re.fullmatch(r"kraqel-isolation-ui-[a-zA-Z0-9-]+", root.name)
    assert root.resolve().parent == Path(os.environ["TEMP"]).resolve()
    report = json.loads((root / "result.json").read_text())
    assert report["passed"] and report["processExitCode"] == 0
    assert report["profileRemoved"] and report["activeJobProcessesAtEnd"] == 0
    assert report["tokenBeforeResume"] == {"isAppContainer": True, "capabilityCount": 0, "expectedIdentity": True}
    assert report["channelDirection"] == "controller-to-container-only"
    assert report["inheritedHandles"] == 2 and not report["hostDirectoryAclGranted"]
    ready = json.loads((root / "data/ui-ready.json").read_text())
    assert ready == {"channelVersion": 1, "taskBound": True, "handleInheritanceCleared": True}
    assert json.loads((root / "data/ui-renderer.json").read_text())["nativeStatusInvoked"]
    outcome = report["uiOutcome"]
    projects = root / "data/app/projects"
    expected = 1 if run["scenario"] == "import" else 0
    assert len(list(projects.iterdir())) == expected
    if expected:
        assert outcome["phase"] == "done" and outcome["error"] is None
        receipt = outcome["receipt"]
        project_id = receipt["projectId"]
        assert str(uuid.UUID(project_id)) == project_id
        assert project_id != "11111111-1111-4111-8111-111111111111"
        assert receipt["documentCount"] == 2 and receipt["assetCount"] == 0
        with sqlite3.connect((projects / project_id / "kraqel.sqlite3").as_uri() + "?mode=ro", uri=True) as database:
            assert database.execute("PRAGMA integrity_check").fetchall() == [("ok",)]
            assert database.execute("PRAGMA foreign_key_check").fetchall() == []
            assert database.execute("SELECT id FROM project").fetchall() == [(project_id,)]
            assert database.execute("SELECT kind,plain_text FROM document ORDER BY kind").fetchall() == [("chapter", ""), ("scene", "Importprobe: ÄÖÜ ß Ελληνικά Кириллица.")]
    else:
        assert outcome == {"phase": "cancelled", "error": "CANCELLED", "receipt": None}
    return {"scenario": run["scenario"], "passed": True, "projectCopies": expected, "nativeRendererInvokedTauri": True, "inputHandleNotInheritableBeforeWebview": True, "noStagingResidue": True, "profileRemoved": True, "activeJobProcessesAtEnd": 0, "appContainer": True, "capabilities": 0, "sqliteUnicodeIntegrityForeignKeys": True if expected else None, "appSha256": sha(root / "bin/kraqel-desktop.exe")}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("journal", type=Path)
    parser.add_argument("--trace", type=Path)
    args = parser.parse_args()
    runs = json.loads(args.journal.read_text(encoding="utf-8-sig"))
    result = {"formatVersion": 1, "diagnosticOnly": True, "productionAcceptance": False, "nativeRuns": [verify_run(run) for run in runs]}
    if args.trace:
        sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "m3-network"))
        from analyze import analyze
        ready = json.loads((args.trace / "ready.json").read_text(encoding="utf-8-sig"))
        complete = json.loads((args.trace / "complete.json").read_text(encoding="utf-8-sig"))
        measurement = analyze(args.trace / "events.xml", [run["pid"] for run in runs], ready, "isolation-ui")
        assert measurement["measurementValid"] and measurement["nonLoopbackEvents"] == 0
        result["etw"] = {"startedUtc": ready["startedUtc"], "stoppedUtc": complete["traceStoppedUtc"], "etlBytes": (args.trace / "network.etl").stat().st_size, "summary": measurement}
    else:
        result["etw"] = {"measured": False}
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
