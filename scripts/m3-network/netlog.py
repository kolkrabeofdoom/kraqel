"""Read an already closed local NetLog; publish only a restricted summary.

This diagnostic is not an upload-content audit or a network-silence acceptance test.
Never publish the raw NetLog, which can contain URLs, identifiers and headers.
"""
import argparse
from collections import Counter
import hashlib
import ipaddress
import json
from pathlib import Path
import re
from urllib.parse import urlsplit

APP_HOSTS = {"tauri.localhost", "ipc.localhost", "asset.localhost"}
PUBLIC_HOSTS = {
    "config.edge.skype.com", "edge.microsoft.com", "msedge.api.cdp.microsoft.com",
    "msedge.b.tlu.dl.delivery.mp.microsoft.com", "msedge.sf.dl.delivery.mp.microsoft.com",
    "browser.events.data.msn.com", "self.events.data.microsoft.com", "go.microsoft.com",
}


def endpoint(url):
    try:
        parsed = urlsplit(url)
        host = (parsed.hostname or "").lower()
        if parsed.scheme in {"data", "file", "blob", "ipc", "tauri", "asset"}:
            return "local-resource", "<local-resource>"
        if parsed.scheme not in {"http", "https", "ws", "wss"}:
            return "unknown", "<redacted>"
        if host in APP_HOSTS:
            return "app-virtual-origin", host
        if host == "localhost":
            return "loopback", "<loopback>"
        try:
            address = ipaddress.ip_address(host)
            if isinstance(address, ipaddress.IPv6Address) and address.ipv4_mapped:
                address = address.ipv4_mapped
            if address.is_loopback:
                return "loopback", "<loopback>"
        except ValueError:
            pass
        return "network-origin", host if host in PUBLIC_HOSTS else "<redacted>"
    except (TypeError, ValueError):
        return "unknown", "<redacted>"


def route_class(url):
    """Only fixed public routes are named; never return arbitrary path/query text."""
    parsed = urlsplit(url)
    if parsed.hostname == "edge.microsoft.com" and parsed.path == "/componentupdater/api/v1/update":
        return "component-updater-route"
    if parsed.hostname == "config.edge.skype.com" and parsed.path.startswith("/config/"):
        return "configuration-route"
    return "not-classified"


def summarize(data):
    if not isinstance(data, dict) or not isinstance(data.get("events"), list):
        raise ValueError("Invalid NetLog envelope")
    constants = data.get("constants", {})
    if not isinstance(constants, dict):
        raise ValueError("Invalid NetLog constants")
    types = constants.get("logEventTypes", {})
    if not isinstance(types, dict) or "URL_REQUEST_START_JOB" not in types:
        raise ValueError("Missing supported NetLog event dictionary")
    if any(type(value) is not int for value in types.values()) or len(set(types.values())) != len(types):
        raise ValueError("Ambiguous NetLog event dictionary")
    names = {value: name for name, value in types.items() if isinstance(value, int)}
    starts = Counter()
    source_sets = {}
    status_codes = Counter()
    decoded = 0
    missing_start_url = 0
    upload_events = 0
    upload_sizes = Counter()
    unknown_types = 0
    for event in data["events"]:
        if not isinstance(event, dict):
            raise ValueError("Invalid NetLog event")
        name = names.get(event.get("type"))
        if name is None:
            unknown_types += 1
        params = event.get("params", {})
        if not isinstance(params, dict):
            raise ValueError("Invalid NetLog parameters")
        if name == "URL_REQUEST_START_JOB":
            # END events need no URL; only events actually carrying a URL are counted.
            if "url" not in params:
                if event.get("phase") == 1:
                    missing_start_url += 1
                continue
            if not isinstance(params["url"], str):
                raise ValueError("Invalid request URL type")
            kind, host = endpoint(params["url"])
            method = params.get("method")
            if method not in {"GET", "HEAD", "POST", "PUT", "PATCH", "DELETE", "OPTIONS", "CONNECT"}:
                method = "<unknown>"
            initiator = params.get("initiator")
            origin = "not-recorded" if initiator is None else "app-origin" if isinstance(initiator, str) and endpoint(initiator)[0] == "app-virtual-origin" else "other-or-opaque"
            key = kind, host, route_class(params["url"]), method, origin
            starts[key] += 1
            source = event.get("source")
            if not isinstance(source, dict) or type(source.get("id")) is not int:
                raise ValueError("Invalid request source")
            source_sets.setdefault(key, set()).add(source["id"])
            decoded += 1
        if name and name.startswith("UPLOAD_DATA_STREAM"):
            upload_events += 1
            if name == "UPLOAD_DATA_STREAM_INIT" and type(params.get("total_size")) is int and params["total_size"] >= 0:
                upload_sizes[params["total_size"]] += 1
        if name == "HTTP_TRANSACTION_READ_RESPONSE_HEADERS":
            headers = params.get("headers", [])
            if isinstance(headers, list):
                for header in headers:
                    if isinstance(header, str):
                        match = re.match(r"^HTTP/[0-9.]+ ([1-5][0-9]{2})(?: |$)", header)
                        if match:
                            status_codes[match[1]] += 1
    return {
        "formatVersion": 1,
        "diagnosticOnly": True,
        "eventCount": len(data["events"]),
        "decodedRequestStarts": decoded,
        "beginEventsMissingUrl": missing_start_url,
        "unknownEventTypes": unknown_types,
        "uploadRelatedEventCount": upload_events,
        "declaredUploadStreamSizes": [{"bytes": size, "initEvents": count} for size, count in sorted(upload_sizes.items())],
        "httpStatusCodes": dict(status_codes),
        "requestGroups": [
            {"originKind": kind, "host": host, "routeClass": route, "method": method, "initiator": origin,
             "startEvents": count, "distinctSources": len(source_sets[key])}
            for key, count in sorted(starts.items())
            for kind, host, route, method, origin in [key]
        ],
        "networkSilenceClaim": False,
        "limits": "Request-start events are not HTTP wire-request counts. No paths, queries, credentials, IP addresses, headers or body contents are exported. Missing content cannot prove absence of manuscript transmission. NetLog does not cover every Windows broker or runtime path.",
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("netlog", type=Path)
    args = parser.parse_args()
    try:
        if args.netlog.stat().st_size > 32 * 1024 * 1024:
            raise SystemExit("NetLog exceeds the 32 MiB analysis limit")
        raw = args.netlog.read_bytes()
        result = summarize(json.loads(raw))
    except (ValueError, TypeError, KeyError, OSError):
        raise SystemExit("NetLog is incomplete or has an unsupported structure; no acceptance result") from None
    result["rawSha256"] = hashlib.sha256(raw).hexdigest()
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
