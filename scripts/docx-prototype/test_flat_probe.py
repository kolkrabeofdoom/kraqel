"""Flat representation: real Office fixtures, strict structure and visible text."""
import copy
from pathlib import Path
import unittest
import xml.etree.ElementTree as ET

import flat_probe as flat
import probe

W = "{" + probe.W + "}"
FIXTURES = Path(__file__).parent / "fixtures" / "flat"


class FlatStructureTests(unittest.TestCase):
    def setUp(self):
        self.parts = flat.build_parts()

    def change(self, part, action):
        root = probe.read_xml(self.parts[part])
        action(root)
        self.parts[part] = ET.tostring(root, encoding="utf-8")

    def reject(self):
        with self.assertRaises((ValueError, KeyError)):
            flat.verify_parts(self.parts)

    def rebind(self, nodes):
        props, digest = flat.properties_for(nodes)
        self.parts["docProps/custom.xml"] = flat.serialize_default(props, flat.CP)
        def update(root):
            for tag in root.iter(W + "tag"):
                value = tag.get(W + "val").rsplit(":", 1)[0]
                tag.set(W + "val", value + ":" + digest[:32])
        self.change("word/document.xml", update)

    def test_unchanged_roundtrip(self):
        self.assertEqual(flat.verify_parts(self.parts), probe.canonical(probe.sample()))

    def test_real_office_save_fixtures(self):
        for name in ("generated.docx", "word-resaved.docx", "libreoffice-resaved.docx", "word-to-libreoffice.docx"):
            with self.subTest(name=name):
                self.assertEqual(flat.verify(FIXTURES / name), probe.canonical(probe.sample()))

    def test_real_office_text_edits(self):
        for name, replacement in (("word-edited.docx", "x"), ("libreoffice-edited.docx", "y")):
            with self.subTest(name=name):
                expected = probe.canonical(probe.sample())
                paragraph = expected["children"][1]["children"][1]["children"][2]
                paragraph["runs"][0]["text"] = paragraph["runs"][0]["text"].replace("Original", replacement)
                self.assertEqual(flat.verify(FIXTURES / name), expected)

    def test_empty_containers_and_repeated_titles(self):
        model = flat.verify_parts(self.parts)
        self.assertEqual(model["children"][2], probe.branch("chapter", probe.leaf("chapter-title")))
        self.assertEqual(model["children"][1]["children"][2], probe.branch("scene", probe.leaf("scene-title", "Am Schreibtisch")))
        self.assertEqual(model["children"][3]["children"][1], probe.branch("scene", probe.leaf("scene-title")))

    def test_missing_inventory(self):
        del self.parts["docProps/custom.xml"]
        self.reject()

    def test_missing_inventory_chunk(self):
        self.change("docProps/custom.xml", lambda root: root.remove(root[-1]))
        self.reject()

    def test_duplicate_inventory_property(self):
        self.change("docProps/custom.xml", lambda root: root.append(copy.deepcopy(root[-1])))
        self.reject()

    def test_changed_chunk_without_new_digest(self):
        self.change("docProps/custom.xml", lambda root: setattr(root[-1][0], "text", "broken"))
        self.reject()

    def test_foreign_inventory_with_updated_header(self):
        nodes, _ = flat.read_inventory(self.parts)
        nodes[-1]["role"] = "paragraph"
        props, _ = flat.properties_for(nodes)
        self.parts["docProps/custom.xml"] = flat.serialize_default(props, flat.CP)
        self.reject()

    def test_missing_empty_chapter_title_marker(self):
        # Removing an empty chapter is visible as a missing title leaf even though it has no text.
        # Locate the empty title by inventory rather than relying on display text or fixed numbering.
        nodes, _ = flat.read_inventory(self.parts)
        model = probe.canonical(probe.sample())
        self.assertEqual(model["children"][2]["children"][0]["runs"], [])
        chapter_titles = [n for n in nodes if n["role"] == "chapter-title"]
        wanted = chapter_titles[1]["id"]
        def remove_by_id(root):
            body = root.find(W + "body")
            target = next(p for p in body if any(f":chapter-title:{wanted}:" in tag.get(W + "val", "") for tag in p.iter(W + "tag")))
            body.remove(target)
        self.change("word/document.xml", remove_by_id)
        self.reject()

    def test_duplicate_marker(self):
        def duplicate(root):
            tags = list(root.iter(W + "tag"))
            tags[-1].set(W + "val", tags[-2].get(W + "val"))
        self.change("word/document.xml", duplicate)
        self.reject()

    def test_reordered_paragraphs(self):
        def swap(root):
            body = root.find(W + "body")
            body[1], body[2] = body[2], body[1]
        self.change("word/document.xml", swap)
        self.reject()

    def test_unmarked_text_outside_control(self):
        def add(root):
            p = next(root.iter(W + "p"))
            probe.el("t", probe.el("r", p)).text = "Versteckter Verlust"
        self.change("word/document.xml", add)
        self.reject()

    def test_wrong_marker_version(self):
        def change(root):
            tag = next(root.iter(W + "tag"))
            tag.set(W + "val", tag.get(W + "val").replace("kqf2:", "kqf3:"))
        self.change("word/document.xml", change)
        self.reject()

    def test_updated_digest_does_not_make_invalid_tree_valid(self):
        nodes, _ = flat.read_inventory(self.parts)
        nodes[1]["parent"] = nodes[1]["id"]
        self.rebind(nodes)
        self.reject()

    def test_duplicate_inventory_identity_even_with_new_digest(self):
        nodes, _ = flat.read_inventory(self.parts)
        nodes[-1]["id"] = nodes[-2]["id"]
        self.rebind(nodes)
        self.reject()

    def test_inconsistent_sibling_position_even_with_new_digest(self):
        nodes, _ = flat.read_inventory(self.parts)
        nodes[-1]["position"] = "99"
        self.rebind(nodes)
        self.reject()

    def test_reparenting_to_leaf_even_with_new_digest(self):
        nodes, _ = flat.read_inventory(self.parts)
        nodes[-1]["parent"] = nodes[1]["id"]
        self.rebind(nodes)
        self.reject()

    def test_unknown_inventory_role_even_with_new_digest(self):
        nodes, _ = flat.read_inventory(self.parts)
        nodes[-1]["role"] = "table"
        self.rebind(nodes)
        self.reject()

    def test_visible_text_is_authoritative(self):
        def change(root):
            next(root.iter(W + "t")).text = "Sichtbar geändert"
        self.change("word/document.xml", change)
        self.assertEqual(flat.verify_parts(self.parts)["children"][0], probe.leaf("project-title", "Sichtbar geändert"))

    def test_numbering_disagrees_with_structure(self):
        self.change("word/numbering.xml", lambda root: next(root.iter(W + "numFmt")).set(W + "val", "bullet"))
        self.reject()

    def test_duplicate_json_fields(self):
        with self.assertRaisesRegex(ValueError, "duplicate JSON"):
            flat.load_json('{"version":2,"version":3}')

    def test_inventory_limit_and_chunk_size(self):
        props = probe.read_xml(self.parts["docProps/custom.xml"])
        self.assertTrue(all(len(prop[0].text) <= flat.CHUNK_SIZE for prop in props))
        nodes, _ = flat.read_inventory(self.parts)
        with self.assertRaises(ValueError):
            flat.properties_for(nodes * flat.MAX_NODES)
        original = dict(self.parts)
        for count in (0, True, flat.MAX_CHUNKS + 1):
            with self.subTest(chunk_count=count):
                self.parts = dict(original)
                def change(root):
                    header = flat.load_json(root[0][0].text)
                    header["chunks"] = count
                    root[0][0].text = flat.compact(header)
                self.change("docProps/custom.xml", change)
                self.reject()


if __name__ == "__main__":
    unittest.main()
