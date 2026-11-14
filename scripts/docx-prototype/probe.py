"""Experimental fixture writer/structural verifier, NOT an application importer.

Python 3.10+, standard library only. No database or application data access.
The production parser must independently validate the whole OOXML package.
"""
from __future__ import annotations

import argparse
import io
import json
from pathlib import Path
import re
import zipfile
import xml.etree.ElementTree as ET

W = "http://schemas.openxmlformats.org/wordprocessingml/2006/main"
R = "http://schemas.openxmlformats.org/package/2006/relationships"
M = "urn:kraqel:manuscript:prototype:1"
OFFICE = "http://schemas.openxmlformats.org/officeDocument/2006/relationships/"
ET.register_namespace("w", W)
ET.register_namespace("kq", M)
# Office interoperability: serialize OPC Relationships with its default namespace.
ET.register_namespace("", R)
MAX_PART = 8 * 1024 * 1024
LEAVES = {"project-title", "chapter-title", "scene-title", "paragraph"} | {
    f"heading{level}" for level in range(1, 7)
}
ROLES = LEAVES | {"manuscript", "chapter", "scene", "bullet-list", "ordered-list", "item", "quote"}


def el(name, parent=None, **attrs):
    node = ET.Element(f"{{{W}}}{name}", {f"{{{W}}}{k}": str(v) for k, v in attrs.items()})
    if parent is not None:
        parent.append(node)
    return node


def leaf(role, text="", runs=None):
    return {"role": role, "runs": runs if runs is not None else ([{"text": text}] if text else [])}


def branch(role, *children):
    return {"role": role, "children": list(children)}


def sample():
    """Synthetic corpus, deliberately includes ambiguity and empty containers."""
    nested_list = branch("bullet-list", branch("item",
        leaf("paragraph", "Verschachtelte Aufzählung"),
        branch("quote", leaf("paragraph", "Zitat innerhalb der Liste"))))
    ordered = branch("ordered-list",
        branch("item", leaf("paragraph", "Erstes Listenelement"),
            leaf("paragraph", "Fortsetzung desselben Elements"), nested_list),
        branch("item", leaf("paragraph", "Zweites Listenelement")))
    quote = branch("quote", leaf("paragraph", "Äußeres Zitat"),
        branch("quote", leaf("paragraph", "Inneres Zitat")),
        branch("bullet-list", branch("item", leaf("paragraph", "Liste im Zitat"))))
    return branch("manuscript", leaf("project-title", "KRAQEL Strukturprobe"),
        branch("chapter", leaf("chapter-title", "Kapitel Eins"),
            branch("scene", leaf("scene-title", "Am Schreibtisch"),
                leaf("paragraph", "Nur ein synthetisches Testmanuskript. Strukturen und Text sollen nach erneutem Speichern erhalten bleiben."),
                leaf("paragraph", runs=[{"text": "EDITIERBAR Original"}, {"text": "  äöü ß Ω 😀  "},
                    {"text": "fett", "bold": True}, {"text": "kursiv", "italic": True},
                    {"text": "beides", "bold": True, "italic": True}, {"break": True}, {"text": "Zweite Zeile"}]),
                *[leaf(f"heading{level}", f"Szenenüberschrift Ebene {level}") for level in range(1, 7)],
                leaf("paragraph"),
                ordered, quote),
            branch("scene", leaf("scene-title", "Am Schreibtisch"))),
        branch("chapter", leaf("chapter-title", "")),
        branch("chapter", leaf("chapter-title", "Kapitel Eins"), branch("scene", leaf("scene-title", ""))))


def canonical(node):
    result = {"role": node["role"]}
    if "children" in node:
        result["children"] = [canonical(child) for child in node["children"]]
    else:
        runs = []
        for run in node["runs"]:
            if run.get("break"):
                runs.append({"break": True})
            elif run.get("text"):
                normalized = {"text": run["text"]}
                normalized.update({k: True for k in ("bold", "italic") if run.get(k)})
                if runs and "text" in runs[-1] and all(runs[-1].get(k) == normalized.get(k) for k in ("bold", "italic")):
                    runs[-1]["text"] += normalized["text"]
                else:
                    runs.append(normalized)
        result["runs"] = runs
    return result


def validate_tree(node):
    role = node["role"]
    if role not in ROLES:
        raise ValueError("unknown role")
    if role in LEAVES:
        if "children" in node:
            raise ValueError("leaf contains children")
        return
    children = node["children"]
    roles = [child["role"] for child in children]
    if role in ("manuscript", "chapter", "scene"):
        title = {"manuscript": "project-title", "chapter": "chapter-title", "scene": "scene-title"}[role]
        if not roles or roles[0] != title:
            raise ValueError("missing or misplaced title")
        allowed = {"manuscript": {"chapter"}, "chapter": {"scene"}, "scene": ROLES - {"manuscript", "chapter", "scene", "item", "project-title", "chapter-title", "scene-title"}}[role]
        roles = roles[1:]
    elif role in ("bullet-list", "ordered-list"):
        allowed = {"item"}
        if not roles:
            raise ValueError("empty list")
    else:
        allowed = ROLES - {"manuscript", "chapter", "scene", "item", "project-title", "chapter-title", "scene-title"}
        if not roles or (role == "item" and roles[0] != "paragraph"):
            raise ValueError("empty quote/item or missing first list paragraph")
    if any(child not in allowed for child in roles):
        raise ValueError("invalid role nesting")
    for child in children:
        validate_tree(child)


def build_parts(model=None):
    model = canonical(sample() if model is None else model)
    validate_tree(model)
    document = el("document")
    body = el("body", document)
    inventory = ET.Element(f"{{{M}}}inventory", version="1")
    numbering = el("numbering")
    next_id = 0
    next_list = 0

    def emit(node, parent, owner="", position=0, list_id=None, quote_depth=0):
        nonlocal next_id, next_list
        next_id += 1
        identity = f"n{next_id:04}"
        role = node["role"]
        ET.SubElement(inventory, f"{{{M}}}node", id=identity, role=role, parent=owner, position=str(position))
        control = el("sdt", parent)
        props = el("sdtPr", control)
        el("tag", props, val=f"kq1:{role}:{identity}")
        el("alias", props, val=role)
        el("id", props, val=next_id)
        el("richText", props)
        content = el("sdtContent", control)
        if role in LEAVES:
            paragraph = el("p", content)
            ppr = el("pPr", paragraph)
            style = {"project-title": "Title", "chapter-title": "ChapterTitle", "scene-title": "SceneTitle"}.get(role, "Normal")
            if role.startswith("heading"):
                style = "Heading" + role[-1]
            el("pStyle", ppr, val=style)
            if list_id:
                numpr = el("numPr", ppr)
                el("ilvl", numpr, val=0)
                el("numId", numpr, val=list_id)
            if quote_depth:
                el("ind", ppr, left=quote_depth * 360)
            for run in node["runs"]:
                r = el("r", paragraph)
                if run.get("bold") or run.get("italic"):
                    rpr = el("rPr", r)
                    for key, tag in (("bold", "b"), ("italic", "i")):
                        if run.get(key):
                            el(tag, rpr)
                if run.get("break"):
                    el("br", r)
                else:
                    t = el("t", r)
                    t.set("{http://www.w3.org/XML/1998/namespace}space", "preserve")
                    t.text = run["text"]
            return
        if role in ("bullet-list", "ordered-list"):
            next_list += 1
            list_id = next_list
            abstract = el("abstractNum", numbering, abstractNumId=list_id)
            level = el("lvl", abstract, ilvl=0)
            el("start", level, val=1)
            el("numFmt", level, val="bullet" if role == "bullet-list" else "decimal")
            el("lvlText", level, val="•" if role == "bullet-list" else "%1.")
            num = el("num", numbering, numId=list_id)
            el("abstractNumId", num, val=list_id)
        for index, child in enumerate(node["children"]):
            emit(child, content, identity, index,
                list_id if role in ("bullet-list", "ordered-list") or (role == "item" and index == 0) else None,
                quote_depth + (role == "quote"))

    emit(model, body)
    # CT_Numbering requires all abstractNum elements before all num instances.
    numbering[:] = sorted(numbering, key=lambda node: node.tag == f"{{{W}}}num")
    section = el("sectPr", body)
    el("pgSz", section, w=11906, h=16838)
    el("pgMar", section, top=1134, right=1134, bottom=1134, left=1134)
    styles = el("styles")
    for name, size in [("Normal", 22), ("Title", 36), ("ChapterTitle", 30), ("SceneTitle", 26)] + [(f"Heading{i}", 28-i) for i in range(1, 7)]:
        st = el("style", styles, type="paragraph", styleId=name)
        el("name", st, val=name)
        if name != "Normal":
            el("basedOn", st, val="Normal")
        ppr = el("pPr", st)
        el("spacing", ppr, after=100)
        if name.startswith("Heading"):
            el("outlineLvl", ppr, val=int(name[-1])-1)
        rp = el("rPr", st)
        el("rFonts", rp, ascii="Calibri", hAnsi="Calibri")
        el("color", rp, val="000000")
        el("sz", rp, val=size)
    rels = ET.Element(f"{{{R}}}Relationships")
    for idx, (kind, target) in enumerate((("styles", "styles.xml"), ("numbering", "numbering.xml"), ("customXml", "../customXml/item1.xml")), 1):
        ET.SubElement(rels, f"{{{R}}}Relationship", Id=f"rId{idx}", Type=OFFICE+kind, Target=target)
    content_types = '''<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/>'''
    for part, kind in (("document", "document.main"), ("styles", "styles"), ("numbering", "numbering")):
        content_types += f'<Override PartName="/word/{part}.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.{kind}+xml"/>'
    content_types += '</Types>'
    parts = {"[Content_Types].xml": content_types.encode(),
        "_rels/.rels": f'<Relationships xmlns="{R}"><Relationship Id="rId1" Type="{OFFICE}officeDocument" Target="word/document.xml"/></Relationships>'.encode(),
        "word/document.xml": ET.tostring(document, encoding="utf-8", xml_declaration=True),
        "word/styles.xml": ET.tostring(styles, encoding="utf-8", xml_declaration=True),
        "word/numbering.xml": ET.tostring(numbering, encoding="utf-8", xml_declaration=True),
        "word/_rels/document.xml.rels": ET.tostring(rels, encoding="utf-8", xml_declaration=True),
        "customXml/item1.xml": ET.tostring(inventory, encoding="utf-8", xml_declaration=True)}
    return parts


def write_parts(path, parts):
    with zipfile.ZipFile(path, "w", compression=zipfile.ZIP_DEFLATED) as archive:
        for name, data in parts.items():
            entry = zipfile.ZipInfo(name, (2026, 11, 14, 0, 0, 0))
            entry.compress_type = zipfile.ZIP_DEFLATED
            archive.writestr(entry, data)


def write_package(path, model=None):
    write_parts(path, build_parts(model))


def read_xml(data):
    if len(data) > MAX_PART or re.search(br"<!\s*(DOCTYPE|ENTITY)", data, re.I):
        raise ValueError("XML size or DTD/entity restriction")
    root = ET.fromstring(data)
    stack = [(root, 1)]
    count = 0
    while stack:
        node, depth = stack.pop()
        count += 1
        if depth > 64 or count > 100_000:
            raise ValueError("XML depth/element restriction")
        stack.extend((child, depth+1) for child in node)
    return root


def read_parts(path):
    with Path(path).open("rb") as source:
        data = source.read(MAX_PART + 1)
    if len(data) > MAX_PART:
        raise ValueError("archive too large")
    with zipfile.ZipFile(io.BytesIO(data)) as archive:
        entries = archive.infolist()
        names = [entry.filename for entry in entries]
        if len(names) > 128 or len(set(names)) != len(names):
            raise ValueError("duplicate names or entry count")
        if sum(entry.file_size for entry in entries) > 32*1024*1024:
            raise ValueError("uncompressed total limit")
        for entry in entries:
            if entry.file_size > MAX_PART or entry.file_size > max(1, entry.compress_size)*100:
                raise ValueError("part size/compression limit")
            if entry.flag_bits & 1 or ".." in entry.filename.split("/") or entry.filename.startswith("/"):
                raise ValueError("invalid archive entry")
        return {entry.filename: archive.read(entry) for entry in entries}


def verify(path):
    """Return visible-text model; inventory may only check structure, never supply text."""
    return verify_parts(read_parts(path))


def verify_parts(parts):
    document = read_xml(parts["word/document.xml"])
    inventories = [read_xml(data) for name, data in parts.items() if name.startswith("customXml/") and name.endswith(".xml")]
    inventories = [root for root in inventories if root.tag == f"{{{M}}}inventory"]
    if len(inventories) != 1 or inventories[0].get("version") != "1":
        raise ValueError("missing/ambiguous inventory or unknown version")
    expected = [dict(node.attrib) for node in inventories[0]]
    if any(node.tag != f"{{{M}}}node" for node in inventories[0]):
        raise ValueError("unknown inventory element")
    observed = []
    numbering = read_xml(parts["word/numbering.xml"])
    abstracts = {node.get(f"{{{W}}}abstractNumId"): node for node in numbering.findall(f"{{{W}}}abstractNum")}
    numbers = {node.get(f"{{{W}}}numId"): node for node in numbering.findall(f"{{{W}}}num")}
    body = document.find(f"{{{W}}}body")
    if body is None:
        raise ValueError("missing document body")

    def check_paragraph(paragraph, role, list_kind):
        style = paragraph.find(f"{{{W}}}pPr/{{{W}}}pStyle")
        style_id = style.get(f"{{{W}}}val") if style is not None else "Normal"
        expected_style = {"project-title": "Title", "chapter-title": "ChapterTitle", "scene-title": "SceneTitle"}.get(role, "Normal")
        if role.startswith("heading"):
            expected_style = "Heading" + role[-1]
        if style_id != expected_style:
            raise ValueError("paragraph style contradicts structural role")
        numpr = paragraph.find(f"{{{W}}}pPr/{{{W}}}numPr")
        if list_kind is None:
            if numpr is not None:
                raise ValueError("unexpected list numbering")
            return
        if numpr is None:
            raise ValueError("missing list numbering")
        identity = numpr.find(f"{{{W}}}numId")
        level = numpr.find(f"{{{W}}}ilvl")
        if identity is None or (level is not None and level.get(f"{{{W}}}val") != "0"):
            raise ValueError("invalid list numbering reference/level")
        num = numbers.get(identity.get(f"{{{W}}}val"))
        if num is None:
            raise ValueError("missing numbering instance")
        abstract_id = num.find(f"{{{W}}}abstractNumId")
        abstract = abstracts.get(abstract_id.get(f"{{{W}}}val")) if abstract_id is not None else None
        if abstract is None:
            raise ValueError("missing numbering definition")
        lvl = abstract.find(f"{{{W}}}lvl[@{{{W}}}ilvl='0']")
        fmt = lvl.find(f"{{{W}}}numFmt") if lvl is not None else None
        start = lvl.find(f"{{{W}}}start") if lvl is not None else None
        if fmt is None or fmt.get(f"{{{W}}}val") != ("bullet" if list_kind == "bullet-list" else "decimal"):
            raise ValueError("numbering format contradicts structural role")
        if start is None or start.get(f"{{{W}}}val") != "1":
            raise ValueError("unsupported list start")
        for override in num.findall(f"{{{W}}}lvlOverride"):
            restart = override.find(f"{{{W}}}startOverride")
            if list(override) != ([restart] if restart is not None else []) or (restart is not None and restart.get(f"{{{W}}}val") != "1"):
                raise ValueError("unsupported numbering override")

    def parse_control(control, owner="", position=0, list_kind=None):
        if control.tag != f"{{{W}}}sdt":
            raise ValueError("unmarked structural content")
        tags = control.findall(f"{{{W}}}sdtPr/{{{W}}}tag")
        contents = control.findall(f"{{{W}}}sdtContent")
        if len(tags) != 1 or len(contents) != 1:
            raise ValueError("missing or duplicate tag/content")
        match = re.fullmatch(r"kq1:([a-z0-9-]+):(n[0-9]{4,8})", tags[0].get(f"{{{W}}}val", ""))
        if not match or match[1] not in ROLES:
            raise ValueError("unknown role/version or invalid identity")
        role, identity = match.groups()
        if any(child.tag not in {f"{{{W}}}sdtPr", f"{{{W}}}sdtContent", f"{{{W}}}sdtEndPr"} for child in control):
            raise ValueError("content outside structured content container")
        observed.append(dict(id=identity, role=role, parent=owner, position=str(position)))
        children = list(contents[0])
        if role not in LEAVES:
            return branch(role, *(parse_control(child, identity, idx,
                role if role in ("bullet-list", "ordered-list") else list_kind if role == "item" and idx == 0 else None)
                for idx, child in enumerate(children)))
        if len(children) != 1 or children[0].tag != f"{{{W}}}p":
            raise ValueError("leaf must contain exactly one paragraph")
        check_paragraph(children[0], role, list_kind)
        runs = []
        for child in children[0]:
            kind = child.tag.removeprefix(f"{{{W}}}")
            if kind in ("pPr", "bookmarkStart", "bookmarkEnd", "proofErr"):
                continue
            if kind != "r":
                raise ValueError("unsupported paragraph content: " + kind)
            marks = {}
            for key, tag in (("bold", "b"), ("italic", "i")):
                flag = child.find(f"{{{W}}}rPr/{{{W}}}{tag}")
                if flag is not None and flag.get(f"{{{W}}}val", "true") not in ("0", "false", "off"):
                    marks[key] = True
            for item in child:
                if item.tag == f"{{{W}}}rPr":
                    continue
                if item.tag == f"{{{W}}}t":
                    runs.append(dict(text=item.text or "", **marks))
                elif item.tag == f"{{{W}}}br" and item.get(f"{{{W}}}type", "textWrapping") == "textWrapping":
                    runs.append({"break": True})
                else:
                    raise ValueError("unsupported inline content: " + item.tag)
        return leaf(role, runs=runs)

    controls = [node for node in body if node.tag != f"{{{W}}}sectPr"]
    if len(controls) != 1:
        raise ValueError("expected single marked manuscript root")
    model = canonical(parse_control(controls[0]))
    ids = [node["id"] for node in observed]
    if len(set(ids)) != len(ids) or observed != expected:
        raise ValueError("structure inventory mismatch (identity, role, parent or order)")
    validate_tree(model)
    if model["role"] != "manuscript":
        raise ValueError("invalid root")
    return model


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("operation", choices=("generate", "verify"))
    parser.add_argument("path", type=Path)
    parser.add_argument("--expected", type=Path, help="canonical JSON to compare against")
    args = parser.parse_args()
    if args.operation == "generate":
        write_package(args.path)
        if verify(args.path) != canonical(sample()):
            raise ValueError("generated fixture failed semantic self-check")
        print("Generated fixture and verified unchanged semantic roundtrip")
    else:
        model = verify(args.path)
        expected = json.loads(args.expected.read_text(encoding="utf-8")) if args.expected else canonical(sample())
        if model != expected:
            raise ValueError("visible manuscript semantics differ from expected model")
        print("PASS: inventory and canonical manuscript match")


if __name__ == "__main__":
    main()
