"""Retain an Office-produced synthetic fixture with neutral core metadata.

Does not rewrite document, styles, numbering, controls or inventory bytes.
"""
import argparse
from pathlib import Path
import xml.etree.ElementTree as ET
import zipfile


def retain(source, destination):
    with zipfile.ZipFile(source) as archive:
        parts = {name: archive.read(name) for name in archive.namelist()}
    if "docProps/core.xml" in parts:
        root = ET.fromstring(parts["docProps/core.xml"])
        for child in root:
            local = child.tag.rsplit("}", 1)[-1]
            if local in ("creator", "lastModifiedBy"):
                child.text = "KRAQEL synthetic fixture"
            elif local in ("created", "modified"):
                child.text = "2026-11-14T00:00:00Z"
        parts["docProps/core.xml"] = ET.tostring(root, encoding="utf-8", xml_declaration=True)
    with zipfile.ZipFile(destination, "w", compression=zipfile.ZIP_DEFLATED) as archive:
        for name, data in parts.items():
            entry = zipfile.ZipInfo(name, (2026, 11, 14, 0, 0, 0))
            entry.compress_type = zipfile.ZIP_DEFLATED
            archive.writestr(entry, data)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("source", type=Path)
    parser.add_argument("destination", type=Path)
    args = parser.parse_args()
    retain(args.source, args.destination)
