"""Build a deterministic, entirely synthetic snapshot-v3 fixture without Git or user data."""
import json
from pathlib import Path
import zipfile

ROOT = Path(__file__).resolve().parents[2]
DESTINATION = Path(__file__).parent / "fixtures/input.kraqel"
PROJECT_ID = "11111111-1111-4111-8111-111111111111"
DOCUMENT_ID = "22222222-2222-4222-8222-222222222222"
TEXT = "Importprobe: ÄÖÜ ß Ελληνικά Кириллица."


def build():
    fixed = ["manifest.json", "schema.json", "documents.json", "story-bible.json", "plotboard.json", "timelines.json", "visual-profiles.json", "prompt-drafts.json", "README.md"]
    objects = {
        "manifest.json": {"format": "kraqel-project-snapshot", "formatVersion": 3, "project": {"id": PROJECT_ID, "name": "Isolierte Importprobe", "schemaVersion": 1, "settings": {}}, "files": fixed, "assets": []},
        "documents.json": {"kind": "documents", "documents": [{"id": DOCUMENT_ID, "parentId": None, "kind": "scene", "title": "Importierte Szene", "position": 0, "revision": 0, "plainText": TEXT, "content": {"version": 1, "document": {"type": "doc", "content": [{"type": "paragraph", "content": [{"type": "text", "text": TEXT}]}]}}}]},
        "story-bible.json": {"kind": "storyBible", "entities": [], "relations": [], "links": []},
        "plotboard.json": {"kind": "plotboard", "storylines": [], "cards": []},
        "timelines.json": {"kind": "timelines", "timelines": [], "events": [], "documentLinks": [], "entityLinks": []},
        "visual-profiles.json": {"kind": "visualProfiles", "profiles": [], "stylePresets": []},
        "prompt-drafts.json": {"kind": "imagePromptDrafts", "drafts": []},
    }
    files = {name: (json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode("utf-8") for name, value in objects.items()}
    chapter_id = "33333333-3333-4333-8333-333333333333"
    scene = objects["documents.json"]["documents"][0]
    scene["parentId"] = chapter_id
    objects["documents.json"]["documents"].insert(0, {"id": chapter_id, "parentId": None, "kind": "chapter", "title": "Importiertes Kapitel", "position": 0, "revision": 0, "plainText": "", "content": {"version": 1, "document": {"type": "doc", "content": [{"type": "paragraph"}]}}})
    files["documents.json"] = (json.dumps(objects["documents.json"], ensure_ascii=False, indent=2) + "\n").encode("utf-8")
    files[f"manuscript/{chapter_id}.md"] = b"# Importiertes Kapitel\n"
    files["schema.json"] = (ROOT / "docs/data-format/snapshot-v3.schema.json").read_bytes()
    files["README.md"] = b"Synthetic isolation test only. No private manuscripts.\n"
    files[f"manuscript/{DOCUMENT_ID}.md"] = (TEXT + "\n").encode("utf-8")
    DESTINATION.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(DESTINATION, "w", compression=zipfile.ZIP_STORED) as archive:
        for name, content in sorted(files.items()):
            info = zipfile.ZipInfo(name, (1980, 1, 1, 0, 0, 0))
            info.create_system = 0
            info.external_attr = 0o600 << 16
            archive.writestr(info, content)


if __name__ == "__main__":
    build()
