"""Restrict diagnostic output and preserve NetLog's evidential limits."""
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from netlog import endpoint, summarize


class NetLogTests(unittest.TestCase):
    def log(self, *events):
        return {"constants": {"logEventTypes": {
            "URL_REQUEST_START_JOB": 13,
            "HTTP_TRANSACTION_READ_RESPONSE_HEADERS": 71,
            "UPLOAD_DATA_STREAM_INIT": 89,
        }}, "events": list(events)}

    def request(self, url, method="GET", source=1, **params):
        return {"type": 13, "phase": 1, "source": {"id": source, "type": 5},
                "params": {"url": url, "method": method, **params}}

    def test_only_fixed_public_routes_are_named(self):
        result = summarize(self.log(self.request(
            "https://user:SECRET@edge.microsoft.com/componentupdater/api/v1/update?token=SECRET",
            method="POST", initiator="not an origin")))
        group = result["requestGroups"][0]
        self.assertEqual(group["routeClass"], "component-updater-route")
        self.assertEqual(group["host"], "edge.microsoft.com")
        self.assertNotIn("SECRET", json.dumps(result))
        self.assertNotIn("user:", json.dumps(result))

    def test_unknown_hosts_paths_and_queries_are_redacted(self):
        result = summarize(self.log(self.request("https://SECRET.example/private-SECRET?key=SECRET")))
        self.assertEqual(result["requestGroups"][0]["host"], "<redacted>")
        self.assertNotIn("SECRET", json.dumps(result))

    def test_ip_addresses_and_local_files_are_not_exported(self):
        for url in ("https://203.0.113.17/SECRET", "file:///C:/Users/SECRET/file", "blob:https://SECRET/path"):
            result = summarize(self.log(self.request(url)))
            text = json.dumps(result)
            self.assertNotIn("203.0.113.17", text)
            self.assertNotIn("SECRET", text)

    def test_app_and_loopback_origins_are_separate(self):
        for host in ("127.0.0.1", "[::1]", "[::ffff:127.0.0.1]", "localhost"):
            self.assertEqual(endpoint(f"http://{host}/")[0], "loopback")
        self.assertEqual(endpoint("http://ipc.localhost/")[0], "app-virtual-origin")
        self.assertEqual(endpoint("http://ipc.localhost.attacker.test/")[0], "network-origin")

    def test_retry_starts_are_not_silently_called_distinct_requests(self):
        result = summarize(self.log(self.request("http://tauri.localhost/"), self.request("http://tauri.localhost/")))
        self.assertEqual(result["requestGroups"][0]["startEvents"], 2)
        self.assertEqual(result["requestGroups"][0]["distinctSources"], 1)

    def test_no_silence_claim_even_for_an_empty_log(self):
        result = summarize(self.log())
        self.assertEqual(result["decodedRequestStarts"], 0)
        self.assertTrue(result["diagnosticOnly"])
        self.assertFalse(result["networkSilenceClaim"])

    def test_end_events_do_not_count_as_new_requests(self):
        end = {"type": 13, "phase": 2, "source": {"id": 1, "type": 5}}
        result = summarize(self.log(self.request("http://tauri.localhost/"), end))
        self.assertEqual(result["decodedRequestStarts"], 1)
        self.assertEqual(result["beginEventsMissingUrl"], 0)

    def test_missing_begin_url_is_reported(self):
        result = summarize(self.log({"type": 13, "phase": 1}))
        self.assertEqual(result["beginEventsMissingUrl"], 1)

    def test_headers_and_upload_contents_stay_private(self):
        result = summarize(self.log(
            {"type": 71, "params": {"headers": ["HTTP/1.1 200 OK", "Set-Cookie: SECRET"]}},
            {"type": 89, "params": {"total_size": 431, "bytes": "SECRET"}},
        ))
        self.assertEqual(result["httpStatusCodes"], {"200": 1})
        self.assertEqual(result["declaredUploadStreamSizes"], [{"bytes": 431, "initEvents": 1}])
        self.assertNotIn("SECRET", json.dumps(result))

    def test_opaque_initiator_is_not_called_an_app_request(self):
        result = summarize(self.log(self.request("https://config.edge.skype.com/config/SECRET", initiator={"SECRET": 1})))
        self.assertEqual(result["requestGroups"][0]["initiator"], "other-or-opaque")
        self.assertNotIn("SECRET", json.dumps(result))

    def test_ambiguous_dictionary_and_unsupported_envelope_fail(self):
        for value in ({}, {"events": []}, {"constants": [], "events": []}):
            with self.assertRaises(ValueError):
                summarize(value)
        data = self.log()
        data["constants"]["logEventTypes"]["OTHER"] = 13
        with self.assertRaises(ValueError):
            summarize(data)

    def test_incomplete_cli_input_does_not_echo_private_content(self):
        with tempfile.TemporaryDirectory() as root:
            path = Path(root)/"incomplete.json"
            path.write_text('{"SECRET":', encoding="utf-8")
            result = subprocess.run([sys.executable, str(Path(__file__).with_name("netlog.py")), str(path)], capture_output=True, text=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertNotIn("SECRET", result.stdout + result.stderr)


if __name__ == "__main__":
    unittest.main()
