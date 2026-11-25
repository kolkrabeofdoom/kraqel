"""Summarize local tracerpt XML without publishing addresses or unrelated processes.

No packet contents, HTTP semantics, DNS attribution or universal offline guarantee.
Root PIDs must be recorded separately from the intended executable at launch time.
"""
import argparse
from collections import Counter
from dataclasses import dataclass
import ipaddress
import json
from pathlib import Path
import re
import xml.etree.ElementTree as ET


def number(value):
    text = str(value).strip()
    return int(text, 16 if text.lower().startswith("0x") else 10)


def events(path):
    for _, element in ET.iterparse(path, events=["end"]):
        if element.tag.rsplit("}", 1)[-1] != "Event":
            continue
        parts = {node.tag.rsplit("}", 1)[-1]: node for node in element}
        system = {node.tag.rsplit("}", 1)[-1]: node for node in parts["System"]}
        data = {node.attrib["Name"]: node.text for node in parts.get("EventData", [])}
        yield (
            system["TimeCreated"].attrib["SystemTime"],
            system["Provider"].attrib.get("Name", ""),
            number(system["EventID"].text),
            number(system["Execution"].attrib["ProcessID"]),
            data,
        )
        element.clear()


@dataclass
class Process:
    pid: int
    parent: int
    start: str
    image: str
    own: bool
    end: str | None = None
    root_pid: int | None = None

    def contains(self, time):
        return self.start <= time and (self.end is None or time <= self.end)


def scope(value):
    try:
        address = ipaddress.ip_address(value.strip())
        if isinstance(address, ipaddress.IPv6Address) and address.ipv4_mapped:
            address = address.ipv4_mapped
    except (ValueError, AttributeError):
        return "unknown"
    if address.is_loopback:
        return "loopback"
    if address.is_multicast:
        return "multicast"
    return "global" if address.is_global else "private-or-link-local"


def root_image_bindings(records):
    """Accept only explicit native broker basenames; never infer roots from a hash."""
    if not isinstance(records, list) or not records:
        raise ValueError("Root map must be a nonempty list")
    bindings = {}
    for record in records:
        if not isinstance(record, dict) or set(record) != {"pid", "image"}:
            raise ValueError("Each root binding requires exactly pid and image")
        pid, image = record["pid"], record["image"]
        if type(pid) is not int or pid <= 0 or pid in bindings:
            raise ValueError("Root PIDs must be positive, unique integers")
        if not isinstance(image, str) or not re.fullmatch(r"(?:kraqel-isolation-probe|p3-starter|p3_handoff_source|kraqel-archiv|[0-9a-f]{64})\.exe", image):
            raise ValueError("Expected a native broker basename or lowercase SHA-256 basename")
        bindings[pid] = image
    return bindings


def analyze(path, root_pids, ready, mode="desktop", root_images=None, scene_sessions=1):
    if scene_sessions < 1 or (mode != "native-scene" and scene_sessions != 1):
        raise ValueError("A positive scene session count requires native-scene mode")
    if mode not in ("desktop", "native-home", "native-scene", "exchange", "isolation-ui", "broker-ui", "native-control", "p3-handoff"):
        raise ValueError("Unknown bounded measurement mode")
    root_image = "kraqel-desktop.exe" if mode in ("desktop", "native-home", "native-scene") else "kraqel-isolation-probe.exe"
    if mode == "p3-handoff":
        root_image = "python.exe"
    if root_images is not None:
        if mode != "native-control":
            raise ValueError("Explicit image bindings require native-control mode")
        root_images = root_image_bindings(root_images)
        if set(root_images) != set(root_pids):
            raise ValueError("Image bindings must match every explicit root PID exactly")
    process_events = []
    loss = None
    for time, provider, event_id, _, data in events(path):
        if "EventsLost" in data and "BuffersLost" in data:
            loss = {key: (loss or {}).get(key, 0) + number(data[key]) for key in ("EventsLost", "BuffersLost")}
        if provider == "Microsoft-Windows-Kernel-Process" and event_id in (1, 2):
            process_events.append((time, event_id, data))
    current = {}
    generations = []
    seen_roots = set()
    root_starts = Counter()
    for time, event_id, data in sorted(process_events, key=lambda item: item[0]):
        pid = number(data["ProcessID"])
        if event_id == 2:
            if pid in current:
                current.pop(pid).end = time
            continue
        # An unpaired start terminates any older generation with the same PID.
        if pid in current:
            current[pid].end = time
        image = data["ImageName"].replace("/", "\\").rsplit("\\", 1)[-1].lower()
        root = pid in root_pids and image == (root_images[pid] if root_images is not None else root_image)
        if root:
            seen_roots.add(pid)
            root_starts[pid] += 1
        parent = number(data["ParentProcessID"])
        own = root or (parent in current and current[parent].own)
        process = Process(pid, parent, time, image, own)
        process.root_pid = pid if root else current[parent].root_pid if parent in current else None
        generations.append(process)
        current[pid] = process
    by_pid = {}
    for process in generations:
        by_pid.setdefault(process.pid, []).append(process)
    counts = Counter()
    own_processes = [process for process in generations if process.own]
    calibration = set()
    controls = ready["controls"]
    helper_pid = number(ready["helperPid"])
    unknown_attribution = 0
    for time, provider, event_id, execution_pid, data in events(path):
        if provider != "Microsoft-Windows-Kernel-Network":
            continue
        pid = number(data["PID"]) if "PID" in data else execution_pid
        if "PID" not in data and execution_pid in (0, 4):
            unknown_attribution += 1
        if pid == helper_pid:
            for index, control in enumerate(controls):
                expected_ids = {12, 15} if control["protocol"] == "TCP" else {42, 43}
                if ":" in control["address"]:
                    expected_ids = {value + 16 for value in expected_ids}
                ports = {number(data[key]) for key in ("dport", "sport") if key in data}
                if event_id in expected_ids and control["port"] in ports and scope(data.get("daddr")) == "loopback":
                    calibration.add(index)
        process = next((item for item in reversed(by_pid.get(pid, [])) if item.contains(time)), None)
        if process is not None and process.own:
            counts[(process.image, event_id, scope(data.get("daddr")))] += 1
    outside_loopback = sum(count for (_, _, destination), count in counts.items() if destination not in ("loopback", "unknown"))
    unknown_destination = sum(count for (_, _, destination), count in counts.items() if destination == "unknown")
    problems = []
    if not root_pids or seen_roots != set(root_pids):
        problems.append("missing-or-mismatched-root-start")
    if any(count != 1 for count in root_starts.values()):
        problems.append("ambiguous-root-pid-reuse")
    if mode == "desktop" and not any(process.image == "msedgewebview2.exe" for process in own_processes):
        problems.append("missing-webview2-process-tree")
    if mode in ("native-home", "native-scene") and any(process.image == "msedgewebview2.exe" for process in own_processes):
        problems.append("unexpected-webview2-process-tree")
    if mode == "native-scene":
        for pid in root_pids:
            children = Counter(p.image for p in own_processes if p.parent == pid and p.root_pid == pid)
            if children["native-leaf.exe"] != scene_sessions or children["pipe-service.exe"] != scene_sessions:
                problems.append("missing-or-repeated-scene-role")
                break
    if mode == "exchange" and any(not any(process.image == root_image and process.parent == root_pid and process.pid != root_pid for process in own_processes) for root_pid in root_pids):
        problems.append("missing-exchange-worker")
    if mode == "isolation-ui" and any(not any(process.image == "kraqel-desktop.exe" and process.parent == root_pid for process in own_processes) or not any(process.image == "msedgewebview2.exe" and process.root_pid == root_pid for process in own_processes) for root_pid in root_pids):
        problems.append("missing-isolated-ui-or-webview-tree")
    if mode == "broker-ui":
        def has_broker_gui(root_pid):
            return any(
                broker.image == root_image and broker.parent == root_pid and broker.root_pid == root_pid
                and any(
                    app.image == "kraqel-desktop.exe" and app.parent == broker.pid
                    and app.root_pid == root_pid and broker.contains(app.start)
                    and any(web.image == "msedgewebview2.exe" and web.parent == app.pid
                            and web.root_pid == root_pid and app.contains(web.start)
                            for web in own_processes)
                    for app in own_processes)
                for broker in own_processes)
        if any(not has_broker_gui(root_pid) for root_pid in root_pids):
            problems.append("missing-broker-ui-descendant-tree")
    if mode == "p3-handoff" and any(not any(
        p.parent == pid and p.root_pid == pid and p.image in ("p3_handoff_source.exe", "p3-starter.exe")
        for p in own_processes
    ) for pid in root_pids):
        problems.append("missing-prepared-handoff-child")
    # native-control covers explicit short host calls that legitimately have no child/GUI.
    # It retains every root, calibration, loss, attribution and lifetime guard below/above;
    # callers must independently match the complete expected native call set and any GUI starts.
    if loss is None or any(loss.values()):
        problems.append("missing-loss-header-or-lost-events")
    expected_controls = {("TCP", "127.0.0.1"), ("UDP", "127.0.0.1"), ("TCP", "::1"), ("UDP", "::1")}
    if {(item["protocol"], item["address"]) for item in controls} != expected_controls or len(controls) != 4 or len(calibration) != 4:
        problems.append("incomplete-tcp-udp-ipv4-ipv6-positive-controls")
    if any(process.end is None for process in own_processes):
        problems.append("process-tree-not-fully-stopped-within-capture")
    if unknown_attribution:
        problems.append("unattributable-network-events")
    if unknown_destination:
        problems.append("unknown-app-network-destination")
    return {
        "formatVersion": 1,
        "measurementMode": mode,
        "measurementValid": not problems,
        "measurementProblems": problems,
        "result": "observed-non-loopback-traffic" if outside_loopback else "inconclusive" if problems else "no-non-loopback-traffic-observed",
        "rootStarts": len(seen_roots),
        "explicitRootImageBindings": len(root_images) if root_images is not None else 0,
        "processLifetimes": dict(Counter(process.image for process in own_processes)),
        "loss": loss,
        "positiveControls": {"expected": 4, "observed": len(calibration)},
        "unattributableNetworkEvents": unknown_attribution,
        "unknownDestinationEvents": unknown_destination,
        "networkEvents": [
            {"image": image, "eventId": event_id, "destinationScope": destination, "count": count}
            for (image, event_id, destination), count in sorted(counts.items())
        ],
        "nonLoopbackEvents": outside_loopback,
        "limits": "Transport metadata only; no HTTP/upload/content attribution. No physical network disconnection. Applies only to the captured process lifetimes and exercised paths. Out-of-tree COM/Shell services are not attributed by this process-tree measurement.",
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("trace_xml", type=Path)
    parser.add_argument("ready_json", type=Path)
    parser.add_argument("--root-pid", type=int, action="append", required=True)
    parser.add_argument("--root-map", type=Path, help="Private JSON list of exact pid/image bindings; native-control only")
    parser.add_argument("--mode", choices=("desktop", "native-home", "native-scene", "exchange", "isolation-ui", "broker-ui", "native-control", "p3-handoff"), default="desktop")
    parser.add_argument("--scene-sessions", type=int, default=1, help="Expected authenticated scene sessions per native-scene desktop root")
    args = parser.parse_args()
    ready = json.loads(args.ready_json.read_text(encoding="utf-8-sig"))
    bindings = json.loads(args.root_map.read_text(encoding="utf-8-sig")) if args.root_map else None
    if args.root_map:
        root_image_bindings(bindings)
    result = analyze(args.trace_xml, args.root_pid, ready, args.mode, bindings, args.scene_sessions)
    print(json.dumps(result, indent=2))
    return 0 if result["measurementValid"] and not result["nonLoopbackEvents"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
