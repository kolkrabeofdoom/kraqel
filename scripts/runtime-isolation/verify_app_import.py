"""Read-only verification of the synthetic native runs; emit aggregates, never raw paths/PIDs."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import sqlite3
import sys
import uuid


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def database(path, project_id):
    with sqlite3.connect(path.as_uri() + "?mode=ro", uri=True) as connection:
        assert connection.execute("PRAGMA integrity_check").fetchall() == [("ok",)]
        assert connection.execute("PRAGMA foreign_key_check").fetchall() == []
        assert connection.execute("SELECT id FROM project").fetchall() == [(project_id,)]
        return connection.execute("SELECT kind, plain_text FROM document ORDER BY kind").fetchall()


def verify(runs, trace):
    root = Path(__file__).resolve().parents[2]
    sys.path.insert(0, str(root / "scripts/m3-network"))
    from analyze import analyze
    binary_hash = digest(root / "scripts/runtime-isolation/probe/target/debug/kraqel-isolation-probe.exe")
    source_hash = digest(root / "scripts/runtime-isolation/fixtures/input.kraqel")
    expected = {"valid": None, "invalid": "IMPORT_FAILED", "expired": "EXPIRED", "cancelled": "CANCELLED", "cancel-at-publish": "CANCELLED", "expire-at-publish": "EXPIRED", "wrong-id": "INVALID_TASK"}
    assert len(runs) == 8 and {run["scenario"] for run in runs} == set(expected) | {"copy"}
    results = []
    for run in runs:
        directory = Path(run["root"])
        assert directory.resolve().parent == Path(os.environ["TEMP"]).resolve()
        assert re.fullmatch(r"kraqel-isolation-[A-Za-z0-9-]+", directory.name)
        report = json.loads((directory / "result.json").read_text(encoding="utf-8-sig"))
        assert digest(directory / "bin/kraqel-isolation-probe.exe") == binary_hash
        assert report["passed"] and report["profileRemoved"] and report["activeJobProcessesAtEnd"] == 0
        assert report["processExitCode"] == 0
        assert report["tokenBeforeResume"] == {"isAppContainer": True, "capabilityCount": 0, "expectedIdentity": True}
        assert report["worker"]["unselectedFileAccessDenied"]
        host = directory.parent / ("kraqel-exchange-host-" + directory.name.removeprefix("kraqel-isolation-"))
        if run["scenario"] == "copy":
            assert digest(host / "input.txt") == digest(host / "output.txt")
            assert not (host / "pending-transfer.tmp").exists()
            results.append({"scenario": "copy", "passed": True})
            continue
        worker = report["worker"]
        assert worker["error"] == expected[run["scenario"]]
        assert worker["replayError"] == "ALREADY_CONSUMED"
        assert worker["existingProjectUnchanged"] and worker["noStagingResidue"]
        receipt = worker["receipt"]
        baseline_id = worker["baselineProjectId"]
        assert str(uuid.UUID(baseline_id)) == baseline_id
        projects = directory / "data/app/projects"
        baseline = projects / baseline_id / "kraqel.sqlite3"
        assert digest(baseline) == worker["baselineSha256"]
        assert database(baseline, baseline_id) == []
        assert len(list(projects.iterdir())) == (2 if receipt else 1)
        assert not (host / "pending-receipt.tmp").exists()
        assert json.loads((host / "receipt.json").read_text()) == worker
        if receipt:
            project_id = receipt["projectId"]
            assert str(uuid.UUID(project_id)) == project_id
            assert project_id != "11111111-1111-4111-8111-111111111111"
            assert receipt["documentCount"] == 2 and receipt["assetCount"] == 0
            assert receipt["sha256"] == source_hash == digest(host / "input.kraqel")
            assert database(projects / project_id / "kraqel.sqlite3", project_id) == [("chapter", ""), ("scene", "Importprobe: ÄÖÜ ß Ελληνικά Кириллица.")]
        results.append({"scenario": run["scenario"], "passed": True, "error": worker["error"], "publishedProjectCopies": int(receipt is not None), "baselineHashUnchanged": True, "integrityAndForeignKeys": True, "noStagingResidue": True, "replayRejected": True})
    ready = json.loads((trace / "ready.json").read_text(encoding="utf-8-sig"))
    complete = json.loads((trace / "complete.json").read_text(encoding="utf-8-sig"))
    measurement = analyze(trace / "events.xml", [run["pid"] for run in runs], ready, "exchange")
    assert measurement["measurementValid"] and measurement["nonLoopbackEvents"] == 0
    return {"formatVersion": 1, "diagnosticOnly": True, "productionAcceptance": False, "binarySha256": binary_hash, "sourceArchiveSha256": source_hash, "sourceArchiveBytes": (root / "scripts/runtime-isolation/fixtures/input.kraqel").stat().st_size, "nativeRuns": results, "etw": {"startedUtc": ready["startedUtc"], "stoppedUtc": complete["traceStoppedUtc"], "etlBytes": (trace / "network.etl").stat().st_size, "summary": measurement}, "limits": ["Headless native application services; no Tauri IPC, WebView or screenreader acceptance", "Synthetic fixtures only; no real user files or native picker exercised in this step", "No causal attribution of out-of-tree COM/Shell services", "No product privacy/offline acceptance"]}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("runs", type=Path)
    parser.add_argument("trace", type=Path)
    args = parser.parse_args()
    result = verify(json.loads(args.runs.read_text(encoding="utf-8-sig")), args.trace)
    print(json.dumps(result, indent=2))
