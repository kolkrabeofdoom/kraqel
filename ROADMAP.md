# Roadmap

## M0 - Fundament (abgeschlossen)

- [x] Monorepo, Tauri-2-Shell, React, TypeScript und Rust-Workspace
- [x] Grundlegende CI, Sicherheits- und Beitragsdokumentation
- [x] Domänenmodell und typisierte Tauri-Commands
- [x] SQLite-Schema, Migrationen und FTS5-Migrationstests
- [x] Lokaler Projektmanager, Backup, Path-Safety und Recovery

## M1 bis M5 - Version 0.1

1. [x] Editor-Grundfunktionen, Szenenbaum, Autosave und Backup
2. M2-Strukturwerkzeuge
   - [x] Story-Bibel mit Entitäten, Beziehungen, Suche und Szenen-Rückverweisen
   - [x] Plotboard und Timeline
3. M3-Datenportabilität und Medien
   - [x] Deterministische Projektsnapshots und lokales Git für Status, Diff und Commit
   - [x] Export nach Markdown, HTML, DOCX und PDF für die dokumentierte Formatierungsmenge
   - [ ] Import und DOCX-Roundtrip
   - [x] Lokale Medienbibliothek mit Metadaten, Links, Backup und Snapshot-Assets
   - [x] Visuelle Profile und projektweite Stilvorlagen
4. Optionale Text-KI, Kontinuitätsprüfung, Prompt-Entwürfe und Bildverträge
   - [x] Providerunabhängige Textgrenze und lokales Ollama-Streaming mit Kontextfreigabe
   - [x] Cloud-Textprovider und Betriebssystem-Schlüsselspeicher (Protokolltests; Live-Abnahme
         offen)
   - [x] Deterministische Kontinuitätsprüfung mit Quellenankern und Nur-Lese-Diff
   - [x] Lokale, editierbare Bildprompt-Entwürfe mit UTF-8-Export und Snapshot
   - [ ] Optionale KI-Kontinuitätshinweise und Bild-Capability-Verträge
5. Reproduzierbarer Windows-Release mit Dokumentation und Abnahme

## M6 - Version 0.2

Bildgenerierung, Variantenvergleich und Kartenmodell folgen erst nach einem stabilen 0.1-Fundament.
Bestehende Assets werden niemals ohne explizite Freigabe ersetzt. Auch Bilder haben ein Recht auf
ein ordentliches Änderungsverfahren.
