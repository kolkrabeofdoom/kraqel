"""Synthetic ETW envelopes test attribution and fail-closed measurement checks."""
import tempfile
import unittest
from pathlib import Path
import xml.etree.ElementTree as ET
from analyze import analyze

PROCESS = "Microsoft-Windows-Kernel-Process"
NETWORK = "Microsoft-Windows-Kernel-Network"


class AnalysisTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.path = Path(self.directory.name) / "trace.xml"
        self.root = ET.Element("Events", xmlns="http://schemas.microsoft.com/win/2004/08/events/event")
        self.ready = {"helperPid": 50, "controls": [
            {"address": "127.0.0.1", "protocol": "TCP", "port": 10001},
            {"address": "127.0.0.1", "protocol": "UDP", "port": 10002},
            {"address": "::1", "protocol": "TCP", "port": 10003},
            {"address": "::1", "protocol": "UDP", "port": 10004},
        ]}
        self.event(0, "", 0, {"EventsLost": 0, "BuffersLost": 0})
        for control, event_id in zip(self.ready["controls"], (12, 42, 28, 58)):
            self.event(1, NETWORK, event_id, {"PID": 50, "daddr": control["address"], "dport": control["port"]})
        self.start(2, 100, 1, "kraqel-desktop.exe")
        self.start(3, 101, 100, "msedgewebview2.exe")
        self.stop(8, 101)
        self.stop(9, 100)

    def event(self, tick, provider, event_id, data, execution_pid=4):
        event = ET.SubElement(self.root, "Event")
        system = ET.SubElement(event, "System")
        ET.SubElement(system, "Provider", Name=provider)
        ET.SubElement(system, "EventID").text = str(event_id)
        ET.SubElement(system, "TimeCreated", SystemTime=f"2026-11-15T11:00:{tick:02}.000Z")
        ET.SubElement(system, "Execution", ProcessID=str(execution_pid))
        payload = ET.SubElement(event, "EventData")
        for key, value in data.items():
            ET.SubElement(payload, "Data", Name=key).text = str(value)

    def start(self, tick, pid, parent, image):
        self.event(tick, PROCESS, 1, {"ProcessID": pid, "ParentProcessID": parent, "ImageName": image})

    def stop(self, tick, pid):
        self.event(tick, PROCESS, 2, {"ProcessID": pid})

    def result(self, roots=(100,)):
        ET.ElementTree(self.root).write(self.path, encoding="utf-8", xml_declaration=True)
        return analyze(self.path, roots, self.ready)

    def test_calibrated_quiet_tree(self):
        result = self.result()
        self.assertTrue(result["measurementValid"])
        self.assertEqual(result["result"], "no-non-loopback-traffic-observed")

    def test_loopback_is_separate_from_public_traffic(self):
        self.event(4, NETWORK, 12, {"PID": 101, "daddr": "127.0.0.1"})
        self.event(4, NETWORK, 28, {"PID": 101, "daddr": "::ffff:127.0.0.1"})
        self.event(5, NETWORK, 12, {"PID": 101, "daddr": "1.1.1.1"})
        result = self.result()
        self.assertTrue(result["measurementValid"])
        self.assertEqual(result["nonLoopbackEvents"], 1)
        self.assertEqual(result["result"], "observed-non-loopback-traffic")

    def test_descendant_and_pid_reuse_are_lifetime_bound(self):
        self.start(4, 102, 101, "child.exe")
        self.stop(7, 102)
        self.event(6, NETWORK, 10, {"PID": 102, "daddr": "1.1.1.1"})
        self.start(10, 101, 1, "unrelated.exe")
        self.start(11, 102, 101, "unrelated-child.exe")
        self.event(12, NETWORK, 10, {"PID": 102, "daddr": "1.1.1.1"})
        self.assertEqual(self.result()["nonLoopbackEvents"], 1)

    def test_lost_buffers_and_missing_roots_are_inconclusive(self):
        self.event(10, "", 0, {"EventsLost": 0, "BuffersLost": 1})
        result = self.result((100, 999))
        self.assertFalse(result["measurementValid"])
        self.assertIn("missing-or-mismatched-root-start", result["measurementProblems"])
        self.assertEqual(result["result"], "inconclusive")

    def test_uncalibrated_empty_trace_is_not_a_pass(self):
        self.ready["controls"][3]["port"] = 20000
        self.assertFalse(self.result()["measurementValid"])

    def test_incomplete_lifetime_is_not_a_pass(self):
        self.start(10, 103, 1, "kraqel-desktop.exe")
        result = self.result((100, 103))
        self.assertIn("process-tree-not-fully-stopped-within-capture", result["measurementProblems"])

    def test_failure_event_uses_execution_pid_without_claiming_known_destination(self):
        self.event(4, NETWORK, 17, {"FailureCode": 5}, execution_pid=101)
        result = self.result()
        self.assertEqual(result["nonLoopbackEvents"], 0)
        self.assertEqual(result["unknownDestinationEvents"], 1)
        self.assertEqual(result["result"], "inconclusive")
        self.assertEqual(result["networkEvents"][0]["destinationScope"], "unknown")

    def test_unattributable_failure_does_not_pass(self):
        self.event(4, NETWORK, 17, {"FailureCode": 5}, execution_pid=4)
        self.assertFalse(self.result()["measurementValid"])

    def test_root_image_must_match(self):
        self.assertFalse(self.result((101,))["measurementValid"])

    def test_reused_root_pid_is_ambiguous(self):
        self.start(10, 100, 1, "kraqel-desktop.exe")
        self.stop(11, 100)
        self.assertIn("ambiguous-root-pid-reuse", self.result()["measurementProblems"])

    def test_duplicate_control_definitions_do_not_calibrate_four_transports(self):
        self.ready["controls"][3] = self.ready["controls"][0]
        self.assertFalse(self.result()["measurementValid"])

    def test_later_header_cannot_hide_earlier_event_loss(self):
        self.event(10, "", 0, {"EventsLost": 1, "BuffersLost": 0})
        self.event(11, "", 0, {"EventsLost": 0, "BuffersLost": 0})
        self.assertFalse(self.result()["measurementValid"])


if __name__ == "__main__":
    unittest.main()
