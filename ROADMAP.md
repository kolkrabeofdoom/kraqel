# Roadmap

## M0 - Fundament (abgeschlossen)

- [x] Monorepo, Tauri-2-Shell, React, TypeScript und Rust-Workspace
- [x] Grundlegende CI, Sicherheits- und Beitragsdokumentation
- [x] Domänenmodell und typisierte Tauri-Commands
- [x] SQLite-Schema, Migrationen und FTS5-Migrationstests
- [x] Lokaler Projektmanager, Backup, Path-Safety und Recovery

## M1 bis M5 - Version 0.1

1. [x] Editor-Grundfunktionen, verwalteter Szenenbaum, Autosave und Backup
   - [x] Kapitel und Szenen erstellen, umbenennen, umordnen und explizit löschen
2. M2-Strukturwerkzeuge
   - [x] Story-Bibel mit Entitäten, Beziehungen, Suche und Szenen-Rückverweisen
     - [x] Browser-E2E für den fachlichen Story-Bibel-Kernablauf
   - [x] Plotboard und Timeline
     - [x] Getrennte Browser-E2E für Plotboard- und Timeline-Kernabläufe
3. M3-Datenportabilität und Medien
   - [x] Deterministische Projektsnapshots und lokales Git für Status, Diff und Commit
     - [x] Browser-E2E für Snapshot, Status, Diff und expliziten Commit
   - [x] Export nach Markdown, HTML, DOCX und PDF für die dokumentierte Formatierungsmenge
     - [x] Browser-E2E für Formatwahl, Dateiname und sichtbare Größenbestätigung
   - [ ] Import und DOCX-Roundtrip
     - [x] Snapshot-v3-Archiv sicher als neues Projekt importieren und Beispielprojekt einbetten
     - [ ] Frei wählbare Snapshotarchive und Text-/DOCX-Dateien über die Oberfläche importieren
   - [x] Lokale Medienbibliothek mit Metadaten, Links, Backup und Snapshot-Assets
     - [x] Browser-E2E für PNG-Import, Metadaten und Szenenverknüpfung
   - [x] Visuelle Profile und projektweite Stilvorlagen
     - [x] Eigenständiger Browser-E2E für Profil-, Stil- und Geltungsbereichspersistenz
4. Optionale Text-KI, Kontinuitätsprüfung, Prompt-Entwürfe und Bildverträge
   - [x] Providerunabhängige Textgrenze und lokales Ollama-Streaming mit Kontextfreigabe
   - [x] Cloud-Textprovider und Betriebssystem-Schlüsselspeicher (Protokolltests; Live-Abnahme
         offen)
   - [x] Deterministische Kontinuitätsprüfung mit Quellenankern und Nur-Lese-Diff
     - [x] Browser-E2E für Offline-Befund, Regelkennung, Zeichenanker und Manuskript-Rücksprung
   - [x] Lokale, editierbare Bildprompt-Entwürfe mit UTF-8-Export und Snapshot
     - [x] Browser-E2E für den lokalen Bildprompt-Kernablauf
   - [x] Provider- und modellspezifische Bild-Capability-Registry sowie Generation-Job-Verträge
   - [ ] Optionale KI-Kontinuitätshinweise
5. Reproduzierbarer Windows-Release mit Dokumentation und Abnahme
   - [x] Playwright-Browser-E2E für den M1-Kernablauf unter Windows und Linux
   - [x] Echter Windows-Tauri-Debug-Smoke-Test für Start, Autosave, SQLite und Wiederöffnen
   - [x] Isolierter, unsignierter NSIS-Testinstaller mit lokalem Installationslebenszyklus
   - [x] Automatisierte Tastatur-, Fokus- und Accessibility-Prüfung der Kernansichten
   - [x] Manuelle Kern-Hörprüfung mit Windows-Sprachausgabe in der Tauri-WebView
   - [x] Erzwungene Speicherfehleransage und „Bewegung reduzieren“ abnehmen
   - [x] Bewusst unsignierten NSIS-Releasepfad mit Offline-WebView2, Prüfsumme und Buildmanifest
         lokal abnehmen
   - [ ] Installer später auf einem weiteren Windows-Rechner gegenprüfen
   - [x] Privaten GitHub-Releaseentwurf, Rückdownload sowie isolierte Installation und
         Deinstallation abnehmen
   - [x] Geprüftes Artefakt als `v0.1.0-alpha.1`-Prerelease veröffentlichen
   - [x] Alpha 2 mit eingebettetem Beispielprojekt bauen, zurückprüfen und veröffentlichen

## M6 - Version 0.2

Bildgenerierung, Variantenvergleich und Kartenmodell folgen erst nach einem stabilen 0.1-Fundament.
Bestehende Assets werden niemals ohne explizite Freigabe ersetzt. Auch Bilder haben ein Recht auf
ein ordentliches Änderungsverfahren.
