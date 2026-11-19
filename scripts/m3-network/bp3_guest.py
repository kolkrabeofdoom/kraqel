"""Check the two BP3 offline guest traces. Only aggregate counts leave the guest evidence directory."""
import argparse
from collections import Counter, defaultdict
import json
from pathlib import Path
import re

from analyze import Process, events, number, scope


EXPECTED = Counter({"bp3-starter.exe": 2, "bp3-worker.exe": 4,
                    "bp3-static-allowed.exe": 1, "bp3-static-foreign.exe": 1})
VOLUME = r"\\device\\harddiskvolume\d+"
SHELL = r"\\windows\\system32\\windowspowershell\\v1\.0\\powershell\.exe"
CONTROL_SET = {(proto, addr) for proto in ("UDP", "TCP") for addr in ("127.0.0.1", "::1")}


def require(condition, problem):
    if not condition:
        raise ValueError(problem)


def check(trace, record, exercise):
    require(record["exit"] == 0, "Workload failed")
    active = {}
    by_pid = defaultdict(list)
    fixture = []
    measured = []
    loss = []
    observed = set()
    net = Counter()
    guest_net = Counter()
    unknown = 0
    rows = sorted(events(trace), key=lambda row: row[0])
    for time, provider, event, _, data in rows:
        if "EventsLost" in data and "BuffersLost" in data:
            loss.append(number(data["EventsLost"]) + number(data["BuffersLost"]))
        if provider != "Microsoft-Windows-Kernel-Process" or event not in (1, 2):
            continue
        pid = number(data["ProcessID"])
        if event == 2:
            if prior := active.pop(pid, None):
                prior.end = time
            continue
        parent = number(data["ParentProcessID"])
        image = data["ImageName"].lower().replace("/", "\\")
        root = pid == record["root"] and re.fullmatch(VOLUME + SHELL, image)
        own = bool(root or (parent in active and active[parent].own))
        proc = Process(pid, parent, time, image, own)
        active[pid] = proc
        by_pid[pid].append(proc)
        if own:
            measured.append(proc)
        if re.fullmatch(VOLUME + r"\\bp3-umci-fixture\\[^\\]+", image):
            fixture.append(proc)

    require(loss and not any(loss), "Missing loss header or lost events/buffers")
    require(len(by_pid[record["root"]]) == 1 and by_pid[record["root"]][0].own, "Unbound shell")
    require(all(p.end for p in measured), "Missing workload process end")
    require(all(p.own for p in fixture), "Fixture outside workload")
    roles = Counter(p.image.rsplit("\\", 1)[1] for p in fixture)
    require(roles == (EXPECTED if exercise else Counter()), f"Unexpected fixture roles: {roles}")
    controls = record["controls"]
    require(len(controls) == 4 and {(c["protocol"], c["address"]) for c in controls} == CONTROL_SET,
            "Unexpected controls")
    helpers = {}
    for c in controls:
        proto = c["protocol"]
        image, port = ("curl.exe", 135) if proto == "TCP" else ("nslookup.exe", 53)
        require(c["image"] == image and c["port"] == port, "Unexpected calibration target")
        candidates = by_pid[c["pid"]]
        require(len(candidates) == 1 and candidates[0].end and not candidates[0].own and
                re.fullmatch(VOLUME + r"\\windows\\system32\\" + re.escape(image), candidates[0].image),
                "Missing independent control lifetime")
        helpers[c["pid"]] = (candidates[0], c)
    require(len(helpers) == 4, "Ambiguous calibration PID")
    for time, provider, event, execution_pid, data in rows:
        if provider != "Microsoft-Windows-Kernel-Network":
            continue
        pid = number(data.get("PID", execution_pid))
        if pid <= 0 or (pid == 4 and "PID" not in data):
            unknown += 1
        endpoints = (scope(data.get("saddr")), scope(data.get("daddr")))
        guest_net["loopback" if endpoints == ("loopback", "loopback") else "otherOrUnknown"] += 1
        if pid in helpers:
            proc, c = helpers[pid]
            ids = (12, 15) if c["protocol"] == "TCP" else (42, 43)
            offset = 16 if ":" in c["address"] else 0
            if (proc.contains(time) and event - offset in ids and data.get("daddr") == c["address"]
                    and number(data.get("dport", 0)) == c["port"]):
                observed.add((c["protocol"], c["address"]))
        if any(p.own and p.contains(time) for p in by_pid.get(pid, ())):
            net[scope(data.get("daddr"))] += 1
    require(observed == CONTROL_SET, f"Missing calibration: {CONTROL_SET - observed}")
    require(not unknown, "Unattributable kernel network activity")
    return {"traceEvents": len(rows), "processes": len(measured), "fixture": dict(sorted(roles.items())),
            "controlsMatched": len(observed), "lostEventsAndBuffers": sum(loss),
            "guestNetworkEvents": dict(sorted(guest_net.items())),
            "networkEventsByScope": dict(sorted(net.items())), "networkEvents": sum(net.values())}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    args = parser.parse_args()
    result = {}
    for name in ("reference", "exercise"):
        path = args.directory
        record = json.loads((path / f"{name}.json").read_text(encoding="utf-8-sig"))
        result[name] = check(path / f"{name}.xml", record, name == "exercise")
    print(json.dumps(result, indent=2))
    raise SystemExit(int(any(r["networkEvents"] or r["guestNetworkEvents"].get("otherOrUnknown")
                             for r in result.values())))
