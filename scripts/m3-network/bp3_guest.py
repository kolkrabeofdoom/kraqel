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


CRASH_EXITS = {
    "pipe-check.exe": [0],
    "pipe-service.exe": [80, 83, 85, 81, 87],
    "native-leaf.exe": [82, 84, 86, 82, 88],
    "pipe-other.exe": [42] * 5,
}


CHANNEL_END_EXITS = {
    "pipe-check.exe": [0],
    "pipe-service.exe": [89, 91, 93, 95, 97, 99],
    "native-leaf.exe": [90, 92, 94, 96, 98, 100],
    "pipe-other.exe": [42] * 6,
}


def check_pipe_roles(fixture, measured, exits, root_pid, expected_exits):
    controller = next(p for p in fixture if p.image.endswith(r"\pipe-check.exe"))
    require(controller.parent == root_pid, "Unbound crash controller")
    for image, codes in expected_exits.items():
        roles = [p for p in fixture if p.image.endswith("\\" + image)]
        require([exits.get((p.pid, p.start)) for p in roles] == codes,
                f"Unexpected service/UI exit sequence for {image}")
        if image == "pipe-check.exe":
            continue
        for role in roles:
            require(role.parent == controller.pid and controller.contains(role.start),
                    "Unbound crash role")
            require(not any(p.parent == role.pid and role.contains(p.start) for p in measured),
                    "Unexpected crash role descendant")


def check_starter_restart(fixture, measured, exits, root_pid):
    def image(p):
        return p.image.rsplit("\\", 1)[-1]

    starters = [p for p in fixture if image(p) == "pipe-check.exe"]
    require([exits.get((p.pid, p.start)) for p in starters] == [0, 101, 102, 102, 102],
            "Expected normal starter, crash and three denied restarts")
    require(all(p.parent == root_pid for p in starters), "Unexpected starter parent")
    require(all(a.end <= b.start for a, b in zip(starters, starters[1:])), "Overlapping starters")
    role_counts = Counter({"pipe-service.exe": 1, "native-leaf.exe": 1, "pipe-other.exe": 1})
    for index, starter in enumerate(starters):
        children = [p for p in measured if p.parent == starter.pid and starter.contains(p.start)]
        if index >= 2:
            require(not children, "Denied starter launched a child")
            continue
        roles = [p for p in children if p in fixture]
        expected = role_counts + role_counts if index == 0 else role_counts
        require(Counter(image(p) for p in roles) == expected, "Unexpected starter role generations")
        require(all(p.end <= starters[index + 1].start for p in roles), "Role survived into restart")
        require(not any(c.parent == p.pid and p.contains(c.start) for p in roles for c in measured),
                "Unexpected role descendant")
        if index == 0:
            for name, codes in {"pipe-service.exe": [60, 60], "native-leaf.exe": [63, 62],
                                "pipe-other.exe": [42, 42]}.items():
                require([exits.get((p.pid, p.start)) for p in roles if image(p) == name] == codes,
                        "Unexpected normal completion")


WITNESS_CRASH_CODES = {"proof-witness-loss": 103, "proof-consumed-loss": 104}


def check_release_roles(fixture, measured, exits, root_pid, proof):
    witness, issuer, *restarts = [p for p in fixture if p.image.endswith(r"\pipe-check.exe")]
    if proof in WITNESS_CRASH_CODES:
        restart, = restarts
        require(restart.parent == root_pid and witness.end <= restart.start
                and exits.get((restart.pid, restart.start)) == 102, "Missing denied witness restart")
        require(not any(p.parent == restart.pid and restart.contains(p.start) for p in measured),
                "Denied witness started a child")
    require(witness.parent == root_pid and issuer.parent == witness.pid
            and witness.contains(issuer.start) and witness.contains(issuer.end),
            "Unexpected witness/issuer lifetime")
    require([exits.get((p.pid, p.start)) for p in (witness, issuer)] == [WITNESS_CRASH_CODES.get(proof, 0), 101],
            "Missing proof decision or issuer crash")
    for parent in (issuer, witness):
        require(all(p in fixture for p in measured if p.parent == parent.pid and parent.contains(p.start)),
                "Unexpected proof process outside the pinned package")
    for role, codes in {"pipe-service.exe": [72, 74], "native-leaf.exe": [73, 75],
                        "pipe-other.exe": [42, 42]}.items():
        generations = [p for p in fixture if p.image.endswith("\\" + role)]
        for n, proc in enumerate(generations):
            parent = issuer if n == 0 else witness
            require(proc.parent == parent.pid and parent.contains(proc.start) and parent.contains(proc.end)
                    and (n == 0 or (proof == "proof-run" and proc.start >= issuer.end))
                    and exits.get((proc.pid, proc.start)) == codes[n], "Unexpected proof role")
            require(not any(p.parent == proc.pid and proc.contains(p.start) for p in measured),
                    "Unexpected role descendant")


def check(trace, record, exercise, window=False, crashes=False, channel_ends=False, starter_restart=False, proof=None):
    require(record["exit"] == 0, "Workload failed")
    active = {}
    by_pid = defaultdict(list)
    fixture = []
    measured = []
    loss = []
    observed = set()
    exits = {}
    net = Counter()
    guest_net = Counter()
    unknown = 0
    expected_exits = CHANNEL_END_EXITS if channel_ends else CRASH_EXITS if crashes else None
    directory = r"\\program files\\kraqel-window-probe" if window or expected_exits or starter_restart or proof else r"\\bp3-umci-fixture"
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
                if (window or expected_exits or starter_restart or proof) and "ExitCode" in data:
                    exits[(pid, prior.start)] = number(data["ExitCode"])
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
        if re.fullmatch(VOLUME + directory + r"\\[^\\]+", image):
            fixture.append(proc)

    require(loss and not any(loss), "Missing loss header or lost events/buffers")
    require(len(by_pid[record["root"]]) == 1 and by_pid[record["root"]][0].own, "Unbound shell")
    require(all(p.end for p in measured), "Missing workload process end")
    require(all(p.own for p in fixture), "Fixture outside workload")
    roles = Counter(p.image.rsplit("\\", 1)[1] for p in fixture)
    expected = Counter({"native-starter.exe": 1, "native-leaf.exe": 3, "observer.exe": 1}) if window else EXPECTED
    if expected_exits:
        expected = Counter({name: len(codes) for name, codes in expected_exits.items()})
    if starter_restart:
        expected = Counter({"pipe-check.exe": 5, "native-leaf.exe": 3, "pipe-other.exe": 3, "pipe-service.exe": 3})
    if proof:
        pairs = 2 if proof == "proof-run" else 1
        expected = Counter({"pipe-check.exe": 3 if proof in WITNESS_CRASH_CODES else 2, "pipe-service.exe": pairs, "native-leaf.exe": pairs, "pipe-other.exe": pairs})
    require(roles == (expected if exercise else Counter()), f"Unexpected fixture roles: {roles}")
    if proof and exercise:
        check_release_roles(fixture, measured, exits, record["root"], proof)
    if starter_restart and exercise:
        check_starter_restart(fixture, measured, exits, record["root"])
    if expected_exits and exercise:
        check_pipe_roles(fixture, measured, exits, record["root"], expected_exits)
    if window and exercise:
        starter = next(p for p in fixture if p.image.endswith(r"\native-starter.exe"))
        leaves = [p for p in fixture if p.image.endswith(r"\native-leaf.exe")]
        observer = next(p for p in fixture if p.image.endswith(r"\observer.exe"))
        require(observer.parent == record["root"] and starter.parent == record["root"] and
                sum(p.parent == starter.pid and starter.contains(p.start) for p in leaves) == 2 and
                sum(p.parent == record["root"] for p in leaves) == 1, "Unexpected probe parentage")
        require(not any(child.parent == leaf.pid and leaf.contains(child.start)
                        for leaf in leaves for child in measured), "Unexpected UI descendant")
        require(exits.get((starter.pid, starter.start)) == 0 and sorted(
            (leaf.parent == starter.pid, exits.get((leaf.pid, leaf.start), -1)) for leaf in leaves
        ) == [(False, 20), (True, 40), (True, 75)], "Unexpected native probe exit codes")
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
    group = parser.add_mutually_exclusive_group()
    group.add_argument("--native-window", action="store_true")
    group.add_argument("--commit-crashes", action="store_true")
    group.add_argument("--channel-ends", action="store_true")
    group.add_argument("--starter-restart", action="store_true")
    group.add_argument("--proof", choices=("proof-run", "proof-incomplete", *WITNESS_CRASH_CODES))
    args = parser.parse_args()
    result = {}
    for name in ("reference", "exercise"):
        path = args.directory
        record = json.loads((path / f"{name}.json").read_text(encoding="utf-8-sig"))
        result[name] = check(path / f"{name}.xml", record, name == "exercise", args.native_window, args.commit_crashes, args.channel_ends, args.starter_restart, args.proof)
    print(json.dumps(result, indent=2))
    raise SystemExit(int(any(r["networkEvents"] or r["guestNetworkEvents"].get("otherOrUnknown")
                             for r in result.values())))
