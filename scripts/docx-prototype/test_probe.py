"""Semantic and rejection tests for the isolated experiment, not production import."""
import copy
from pathlib import Path
import tempfile
import unittest
import xml.etree.ElementTree as ET
import zipfile

import probe

W = "{" + probe.W + "}"
FIXTURES = Path(__file__).parent / "fixtures"


class StructureProbeTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.path = Path(self.temp.name) / "test.docx"
        probe.write_package(self.path)

    def mutate(self, part, change):
        parts = probe.read_parts(self.path)
        root = probe.read_xml(parts[part])
        change(root)
        parts[part] = ET.tostring(root, encoding="utf-8", xml_declaration=True)
        with zipfile.ZipFile(self.path, "w", compression=zipfile.ZIP_DEFLATED) as archive:
            for name, data in parts.items():
                archive.writestr(name, data)

    def reject(self):
        with self.assertRaises((ValueError, KeyError)):
            probe.verify(self.path)

    def test_semantic_roundtrip(self):
        self.assertEqual(probe.verify(self.path), probe.canonical(probe.sample()))

    def test_office_fixtures(self):
        expected = probe.canonical(probe.sample())
        for name in ("generated.docx", "word-resaved.docx"):
            with self.subTest(name=name):
                self.assertEqual(probe.verify(FIXTURES / name), expected)

    def test_real_word_text_edit_uses_visible_text(self):
        expected = probe.canonical(probe.sample())
        paragraph = expected["children"][1]["children"][1]["children"][2]
        paragraph["runs"][0]["text"] = paragraph["runs"][0]["text"].replace("Original", "x")
        self.assertEqual(probe.verify(FIXTURES / "word-edited.docx"), expected)

    def test_real_libreoffice_save_is_rejected(self):
        with self.assertRaisesRegex(ValueError, "inventory"):
            probe.verify(FIXTURES / "libreoffice-resaved.docx")

    def test_deleted_whole_scene(self):
        def remove(root):
            content = next(node for node in root.iter(W + "sdtContent") if any(
                child.find(W + "sdtPr/" + W + "tag") is not None
                and ":scene:" in child.find(W + "sdtPr/" + W + "tag").get(W + "val", "") for child in node))
            content.remove(list(content)[-1])
        self.mutate("word/document.xml", remove)
        self.reject()

    def test_duplicate_identity(self):
        def duplicate(root):
            tags = list(root.iter(W + "tag"))
            tags[-1].set(W + "val", tags[-2].get(W + "val"))
        self.mutate("word/document.xml", duplicate)
        self.reject()

    def test_reordered_siblings(self):
        def swap(root):
            content = next(root.iter(W + "sdtContent"))
            content[-1], content[-2] = content[-2], content[-1]
        self.mutate("word/document.xml", swap)
        self.reject()

    def test_removed_marker(self):
        self.mutate("word/document.xml", lambda root: next(root.iter(W + "sdtPr")).remove(next(root.iter(W + "tag"))))
        self.reject()

    def test_unknown_marker_version(self):
        self.mutate("word/document.xml", lambda root: next(root.iter(W + "tag")).set(W + "val", "kq2:manuscript:n0001"))
        self.reject()

    def test_broken_inventory(self):
        self.mutate("customXml/item1.xml", lambda root: root.remove(root[-1]))
        self.reject()

    def test_unmarked_paragraph(self):
        self.mutate("word/document.xml", lambda root: next(root.iter(W + "sdtContent")).append(ET.Element(W + "p")))
        self.reject()

    def test_contradictory_numbering(self):
        self.mutate("word/numbering.xml", lambda root: next(root.iter(W + "numFmt")).set(W + "val", "bullet"))
        self.reject()

    def test_contradictory_heading(self):
        def change(root):
            style = next(node for node in root.iter(W + "pStyle") if node.get(W + "val") == "Heading6")
            style.set(W + "val", "Heading1")
        self.mutate("word/document.xml", change)
        self.reject()

    def test_nondefault_list_start(self):
        self.mutate("word/numbering.xml", lambda root: next(root.iter(W + "start")).set(W + "val", "2"))
        self.reject()

    def test_no_reconstruction_from_hidden_text(self):
        def change(root):
            text = next(root.iter(W + "t"))
            text.text = "Neuer sichtbarer Titel"
        self.mutate("word/document.xml", change)
        self.assertEqual(probe.verify(self.path)["children"][0]["runs"], [{"text": "Neuer sichtbarer Titel"}])

    def test_run_splitting_is_semantically_neutral(self):
        def split(root):
            paragraph = next(root.iter(W + "p"))
            run = paragraph.find(W + "r")
            text = run.find(W + "t")
            clone = copy.deepcopy(run)
            clone.find(W + "t").text = text.text[3:]
            text.text = text.text[:3]
            paragraph.append(clone)
        self.mutate("word/document.xml", split)
        self.assertEqual(probe.verify(self.path), probe.canonical(probe.sample()))

    def test_dtd_and_xml_depth(self):
        for xml in (b'<!DOCTYPE x [<!ENTITY a "text">]><x>&a;</x>', b'<x>'*65+b'</x>'*65):
            with self.subTest(xml=xml[:20]), self.assertRaises(ValueError):
                probe.read_xml(xml)

    def test_duplicate_zip_entries(self):
        with zipfile.ZipFile(self.path, "a") as archive:
            with self.assertWarns(UserWarning):
                archive.writestr("word/document.xml", b"<x/>")
        self.reject()


if __name__ == "__main__":
    unittest.main()
