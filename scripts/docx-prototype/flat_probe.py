"""Flat paragraph controls and chunked custom properties: isolated Office experiment.

The inventory stores structural roles/relationships only. Text is always read from
the visible paragraph controls. This is not the production package validator.
"""
from __future__ import annotations

import argparse
import copy
import hashlib
import json
from pathlib import Path
import re
import xml.etree.ElementTree as ET

import probe

W = "{" + probe.W + "}"
CP = "http://schemas.openxmlformats.org/officeDocument/2006/custom-properties"
VT = "http://schemas.openxmlformats.org/officeDocument/2006/docPropsVTypes"
CT = "http://schemas.openxmlformats.org/package/2006/content-types"
PROP = "KRAQEL.Roundtrip.Prototype2"
CHUNK_SIZE = 200
MAX_NODES = 4096
MAX_CHUNKS = 2048
ET.register_namespace("vt", VT)


def serialize_default(root, namespace):
    """Keep OPC defaults interoperable; restore baseline writer registration."""
    ET.register_namespace("", namespace)
    try:
        return ET.tostring(root, encoding="utf-8", xml_declaration=True)
    finally:
        ET.register_namespace("", probe.R)


def compact(value):
    return json.dumps(value, ensure_ascii=True, separators=(",", ":"))


def properties_for(nodes):
    if len(nodes) > MAX_NODES:
        raise ValueError("experimental inventory node limit")
    payload = compact({"version": 2, "nodes": nodes})
    chunks = [payload[start:start + CHUNK_SIZE] for start in range(0, len(payload), CHUNK_SIZE)]
    if len(chunks) > MAX_CHUNKS:
        raise ValueError("experimental inventory limit")
    digest = hashlib.sha256(payload.encode("ascii")).hexdigest()
    values = {PROP: compact({"version": 2, "chunks": len(chunks), "sha256": digest})}
    values.update({f"{PROP}.{index:04}": chunk for index, chunk in enumerate(chunks, 1)})
    root = ET.Element(f"{{{CP}}}Properties")
    for pid, (name, text) in enumerate(values.items(), 2):
        prop = ET.SubElement(root, f"{{{CP}}}property", name=name, pid=str(pid),
                             fmtid="{D5CDD505-2E9C-101B-9397-08002B2CF9AE}")
        ET.SubElement(prop, f"{{{VT}}}lpwstr").text = text
    return root, digest


def build_parts(model=None):
    parts = probe.build_parts(model)
    nodes = [dict(node.attrib) for node in probe.read_xml(parts.pop("customXml/item1.xml"))]
    props, digest = properties_for(nodes)
    document = probe.read_xml(parts["word/document.xml"])
    body = document.find(W + "body")
    leaves = []
    for control in body.iter(W + "sdt"):
        tag = control.find(W + "sdtPr/" + W + "tag").get(W + "val")
        _, role, identity = tag.split(":")
        if role in probe.LEAVES:
            control = copy.deepcopy(control)
            control.find(W + "sdtPr/" + W + "tag").set(W + "val", f"kqf2:{role}:{identity}:{digest[:32]}")
            # Both applications retain paragraph-wide inline controls. LibreOffice
            # also converts block controls to this shape when saving DOCX.
            content = control.find(W + "sdtContent")
            paragraph = content[0]
            runs = [child for child in paragraph if child.tag != W + "pPr"]
            paragraph[:] = [child for child in paragraph if child.tag == W + "pPr"]
            content[:] = runs
            paragraph.append(control)
            leaves.append(paragraph)
    section = body.find(W + "sectPr")
    body[:] = leaves + [section]
    parts["word/document.xml"] = ET.tostring(document, encoding="utf-8", xml_declaration=True)
    parts["docProps/custom.xml"] = serialize_default(props, CP)
    rels = probe.read_xml(parts["word/_rels/document.xml.rels"])
    rels[:] = [rel for rel in rels if rel.get("Type") != probe.OFFICE + "customXml"]
    parts["word/_rels/document.xml.rels"] = serialize_default(rels, probe.R)
    package_rels = probe.read_xml(parts["_rels/.rels"])
    ET.SubElement(package_rels, f"{{{probe.R}}}Relationship", Id="rIdKraqelProps",
                  Type=probe.OFFICE + "custom-properties", Target="docProps/custom.xml")
    parts["_rels/.rels"] = serialize_default(package_rels, probe.R)
    content_types = probe.read_xml(parts["[Content_Types].xml"])
    ET.SubElement(content_types, f"{{{CT}}}Override", PartName="/docProps/custom.xml",
                  ContentType="application/vnd.openxmlformats-officedocument.custom-properties+xml")
    parts["[Content_Types].xml"] = serialize_default(content_types, CT)
    return parts


def write_package(path, model=None):
    probe.write_parts(path, build_parts(model))


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError("duplicate JSON field")
        result[key] = value
    return result


def load_json(text):
    return json.loads(text, object_pairs_hook=unique_object)


def read_inventory(parts):
    if "docProps/custom.xml" not in parts:
        raise ValueError("missing flat inventory properties")
    root = probe.read_xml(parts["docProps/custom.xml"])
    if root.tag != f"{{{CP}}}Properties":
        raise ValueError("invalid custom property namespace")
    values = {}
    for prop in root:
        name = prop.get("name", "")
        if not name.startswith(PROP):
            continue
        if prop.tag != f"{{{CP}}}property" or name in values or len(prop) != 1:
            raise ValueError("duplicate or invalid inventory property")
        if prop[0].tag != f"{{{VT}}}lpwstr" or len(prop[0]):
            raise ValueError("inventory property must be a string")
        value = prop[0].text or ""
        if len(value) > CHUNK_SIZE:
            raise ValueError("inventory property too long")
        values[name] = value
    if PROP not in values:
        raise ValueError("missing flat inventory header")
    header = load_json(values[PROP])
    if not isinstance(header, dict) or set(header) != {"version", "chunks", "sha256"}:
        raise ValueError("invalid inventory header")
    count = header["chunks"]
    if type(header["version"]) is not int or header["version"] != 2:
        raise ValueError("unknown flat inventory version")
    if type(count) is not int or not 1 <= count <= MAX_CHUNKS:
        raise ValueError("inventory chunk count limit")
    names = [f"{PROP}.{index:04}" for index in range(1, count + 1)]
    if set(values) != set(names) | {PROP}:
        raise ValueError("missing or extra inventory chunk")
    payload = "".join(values[name] for name in names)
    if not payload.isascii():
        raise ValueError("noncanonical inventory encoding")
    digest = hashlib.sha256(payload.encode("ascii")).hexdigest()
    if header["sha256"] != digest:
        raise ValueError("inventory digest mismatch")
    inventory = load_json(payload)
    if not isinstance(inventory, dict) or set(inventory) != {"version", "nodes"}:
        raise ValueError("invalid inventory schema")
    if type(inventory["version"]) is not int or inventory["version"] != 2:
        raise ValueError("unknown inventory version")
    if compact(inventory) != payload:
        raise ValueError("noncanonical inventory JSON")
    nodes = inventory["nodes"]
    if not isinstance(nodes, list) or not 1 <= len(nodes) <= MAX_NODES:
        raise ValueError("inventory node count limit")
    return nodes, digest


def validate_inventory(nodes):
    """Validate rooted, acyclic, ordered tree before allocating reconstructed controls."""
    seen = {}
    children = {}
    depths = {}
    for index, node in enumerate(nodes):
        if not isinstance(node, dict) or set(node) != {"id", "role", "parent", "position"}:
            raise ValueError("invalid inventory node schema")
        if any(type(value) is not str for value in node.values()):
            raise ValueError("invalid inventory field type")
        identity, parent = node["id"], node["parent"]
        if not re.fullmatch(r"n[0-9]{4,8}", identity) or identity in seen or node["role"] not in probe.ROLES:
            raise ValueError("invalid/duplicate inventory identity or role")
        if index == 0:
            if parent != "" or node["role"] != "manuscript" or node["position"] != "0":
                raise ValueError("invalid inventory root")
            depth = 1
        else:
            if parent not in seen or seen[parent]["role"] in probe.LEAVES:
                raise ValueError("missing/forward/leaf inventory parent")
            if node["position"] != str(len(children[parent])):
                raise ValueError("invalid sibling position")
            children[parent].append(identity)
            depth = depths[parent] + 1
        if depth > 32:
            raise ValueError("logical structure depth limit")
        seen[identity] = node
        children[identity] = []
        depths[identity] = depth
    preorder = []
    pending = [nodes[0]["id"]]
    while pending:
        identity = pending.pop()
        preorder.append(identity)
        pending.extend(reversed(children[identity]))
    if preorder != [node["id"] for node in nodes]:
        raise ValueError("noncanonical inventory order")
    return children


def verify(path):
    return verify_parts(probe.read_parts(path))


def as_block_control(paragraph):
    """Normalize the equivalent inline and block layouts without ignoring text."""
    if paragraph.tag == W + "sdt":
        return paragraph
    if paragraph.tag != W + "p":
        raise ValueError("unmarked flat paragraph")
    controls = paragraph.findall(W + "sdt")
    properties = paragraph.findall(W + "pPr")
    if len(controls) != 1 or len(properties) > 1 or len(paragraph) != len(controls) + len(properties):
        raise ValueError("unmarked content outside flat paragraph control")
    control = copy.deepcopy(controls[0])
    contents = control.findall(W + "sdtContent")
    if len(contents) != 1:
        raise ValueError("missing/duplicate flat control content")
    normalized_paragraph = ET.Element(W + "p", paragraph.attrib)
    normalized_paragraph.extend(copy.deepcopy(prop) for prop in properties)
    normalized_paragraph.extend(contents[0])
    contents[0][:] = [normalized_paragraph]
    return control


def verify_parts(parts):
    nodes, digest = read_inventory(parts)
    children = validate_inventory(nodes)
    expected_leaves = [node for node in nodes if node["role"] in probe.LEAVES]
    document = probe.read_xml(parts["word/document.xml"])
    body = document.find(W + "body")
    if body is None:
        raise ValueError("missing flat document body")
    controls = [node for node in body if node.tag != W + "sectPr"]
    if len(controls) != len(expected_leaves):
        raise ValueError("missing or extra flat paragraph marker")
    leaves = {}
    for control, expected in zip(controls, expected_leaves):
        control = as_block_control(control)
        tags = control.findall(W + "sdtPr/" + W + "tag")
        wanted = f"kqf2:{expected['role']}:{expected['id']}:{digest[:32]}"
        if len(tags) != 1 or tags[0].get(W + "val") != wanted:
            raise ValueError("flat marker identity/order/role/inventory mismatch")
        control = copy.deepcopy(control)
        control.find(W + "sdtPr/" + W + "tag").set(W + "val", f"kq1:{expected['role']}:{expected['id']}")
        leaves[expected["id"]] = control

    # Reuse the first experiment's independent paragraph/numbering verifier.
    # Only containers are reconstructed from validated metadata, never text.
    def reconstruct(node):
        identity, role = node["id"], node["role"]
        if role in probe.LEAVES:
            return leaves[identity]
        control = probe.el("sdt")
        props = probe.el("sdtPr", control)
        probe.el("tag", props, val=f"kq1:{role}:{identity}")
        probe.el("sdtContent", control).extend(reconstruct(by_id[child]) for child in children[identity])
        return control

    by_id = {node["id"]: node for node in nodes}
    virtual_document = probe.el("document")
    probe.el("body", virtual_document).append(reconstruct(nodes[0]))
    virtual_inventory = ET.Element(f"{{{probe.M}}}inventory", version="1")
    for node in nodes:
        ET.SubElement(virtual_inventory, f"{{{probe.M}}}node", node)
    virtual_parts = {name: value for name, value in parts.items() if not name.startswith("customXml/")}
    virtual_parts["word/document.xml"] = ET.tostring(virtual_document, encoding="utf-8")
    virtual_parts["customXml/item1.xml"] = ET.tostring(virtual_inventory, encoding="utf-8")
    return probe.verify_parts(virtual_parts)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("operation", choices=("generate", "verify"))
    parser.add_argument("path", type=Path)
    parser.add_argument("--expected", type=Path)
    args = parser.parse_args()
    if args.operation == "generate":
        write_package(args.path)
        expected = probe.canonical(probe.sample())
    else:
        expected = load_json(args.expected.read_text(encoding="utf-8")) if args.expected else probe.canonical(probe.sample())
    if verify(args.path) != expected:
        raise ValueError("visible manuscript semantics differ from expected model")
    print("PASS: flat markers, structural inventory and visible manuscript match")


if __name__ == "__main__":
    main()
