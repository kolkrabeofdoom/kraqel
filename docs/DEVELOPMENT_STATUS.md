# Entwicklungsstatus

Stand: 9. November 2026

Code-Referenz: KQ-005 (`feat/kq-005-editor-autosave`, dieser Stand)

Produktstatus: frühes Entwicklungsfundament, nicht produktionsreif

Dieses Dokument trennt verifizierte Implementierungen von Planung. Maßgeblich sind Code, Tests, CI,
angenommene ADRs und das technische Pflichtenheft. Typen, Enums und leere Modulgerüste gelten nicht
als vollständige Fachfunktion.

## Status nach Pflichtenheft-Stufen

| Stufe        | Pflichtenheft-Ziel                                     | Verifizierter Stand                                                                                                                                                                                                                      |
| ------------ | ------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **M0**       | Monorepo, Tauri, Datenbank, CI, Berechtigungsmodell    | **Technisches Fundament umgesetzt:** Monorepo, Tauri-Shell, CI, minimale Capability, Command-Verträge, SQLite/FTS5 sowie sicherer Projektlebenszyklus mit Backup und Recovery vorhanden.                                                 |
| **M1**       | Editor, Szenenbaum, Autosave, Backup                   | **Teilweise umgesetzt:** Projekt-/Kapitel-/Szenenoberfläche, Tiptap, szenengebundenes Undo/Redo, Wortzählung, Fokusmodus, revisionsgeprüftes Autosave und SQLite-Backup vorhanden. Umbenennen, Umordnen, Löschen und E2E-Abnahme fehlen. |
| **M2**       | Story-Bibel, Plotboard, Timeline                       | **Nicht implementiert.** Keine UI, Persistenz oder Fachlogik.                                                                                                                                                                            |
| **M3**       | Git, Import/Export, Medienbibliothek, visuelle Profile | **Nicht implementiert.** Es existieren nur Enums/Vertragstypen und eine relative Pfadprüfung.                                                                                                                                            |
| **M4**       | Text-KI, Kontinuität, Prompt-Entwürfe, Bildverträge    | **Nur Verträge vorbereitet:** Capability-, Kontextfreigabe-, Medien- und Profiltypen; keine Provider oder Fachabläufe.                                                                                                                   |
| **M5**       | Windows-Release 0.1, Tests, Dokumentation              | **Nicht erreicht.** Debug-Build und CI funktionieren; kein Releasepaket und keine Abnahme der 0.1-Funktionen.                                                                                                                            |
| **M6 / 0.2** | Bildgenerierung, Referenzkonsistenz, Kartenmodell      | **Nicht begonnen und ausdrücklich außerhalb von 0.1.**                                                                                                                                                                                   |

Die funktionalen Anforderungen F01 bis F09 sind damit noch nicht abgenommen. Vorhandene Verträge und
Gerüste sind technische Vorarbeit, keine fertigen Funktionen.

## Verifiziert implementiert und getestet

### Repository und Qualität

- pnpm- und Cargo-Monorepo mit gelockten Abhängigkeiten.
- Tauri-2-/React-/TypeScript-Desktop-Shell mit statischer, responsiver Startoberfläche.
- Minimale Tauri-Berechtigung `core:default`, definierte CSP und Windows-Fensterkonfiguration.
- GitHub-Actions-CI unter `windows-latest` und `ubuntu-latest`.
- Letzter CI-Lauf für `main` bei `20f6b1b`: erfolgreich unter Windows und Linux.
- Qualitätskette: Prettier, rustfmt, ESLint, Clippy mit `-D warnings`, TypeScript, Vitest,
  Rust-Tests sowie Frontend- und Workspace-Builds.
- GPL-3.0-or-later, Beitrags-, Verhaltens- und Sicherheitsdokumentation vorhanden.

### Domäne und IPC-Grenze (KQ-002)

- Kanonische UUID-Typen für Projekte und Entitäten in Rust und TypeScript.
- Zentrale Schema-Version `1` in den Domänenmodulen.
- Versionierter lokaler Command-Vertrag v1 mit `contractVersion`, `requestId` und Nutzlast.
- Stabile Fehlercodes `INVALID_REQUEST`, `UNSUPPORTED_CONTRACT_VERSION` und `INTERNAL`.
- UI ersetzt fremde IPC-Fehlermeldungen durch bereinigte Texte; Eingaben und Pfade werden nicht
  zurückgespiegelt.
- Tauri-Commands `application_status` und `validate_project_name` samt TypeScript-Clientfunktionen.
- Tests für Serialisierungsform, Vertragsversion, UUID-/Schemaprüfung, Projektnamen und
  Fehlerbereinigung.

### Lokale Datenbank (KQ-003)

- `rusqlite` 0.39 mit gebündeltem SQLite und FTS5, kompatibel mit Rust 1.85.
- Initialschema v1 für `project`, `document`, `entity`, `relation` und `link` mit Fremdschlüsseln,
  Indizes, JSON-Prüfungen und zentraler Schema-Version aus `kraqel-domain`.
- Atomare Vorwärtsmigration über `PRAGMA user_version`; unbekannte Zukunftsversionen werden ohne
  Schemaänderung abgelehnt.
- Verwaltete Verbindungen aktivieren Fremdschlüssel, Busy-Timeout und für Dateien WAL.
- FTS5-Index für Dokumenttitel und Klartext mit Triggern für Einfügen, Ändern und Löschen.
- Tests für leere und befüllte Datenbanken, Wiederöffnung, Fremdschlüssel, FTS-Synchronisation und
  Ablehnung eines Zukunftsschemas.

### Projektlebenszyklus, Backup und Recovery (KQ-004)

- `ProjectManager` erstellt und öffnet Projekte ausschließlich unter einer kanonischen lokalen
  Wurzel; absolute Pfade, Traversal und Symlink-Komponenten werden abgelehnt.
- Projekterstellung und Recovery erfolgen in temporären Geschwisterverzeichnissen und werden erst
  nach erfolgreicher Prüfung atomar veröffentlicht.
- Konsistente SQLite-Sicherungen verwenden die Online-Backup-API, `quick_check`, Schema-Prüfung und
  Dateisynchronisation; bestehende Sicherungen werden nicht überschrieben.
- Wiederherstellung liest die Sicherung ausschließlich lesend und erzeugt immer ein neues Ziel.
- Datenarme Fehler unterscheiden fehlende Projekte, Datenbanken und Sicherungen sowie beschädigte
  Datenbanken.
- Tests decken Windows-taugliche Leerzeichen-/Unicodepfade, lange Verzeichnisnamen, Traversal,
  Symlinks, fehlende und beschädigte Dateien, Backup-Konsistenz und unveränderte Recovery-Quellen
  ab.

### Editor, Szenenbaum und Autosave (KQ-005)

- Tiptap 3/ProseMirror mit versioniertem Editor-Dokumentvertrag v1 und grundlegender
  Rich-Text-Formatierung.
- Lokale Projektübersicht sowie Projekt-, Kapitel- und Szenenerstellung über versionierte
  Tauri-Commands; Projekte liegen vorerst im lokalen App-Datenverzeichnis.
- Stabil sortierter Kapitel-/Szenenbaum; Undo/Redo-Verlauf wird durch eine eigene Editorinstanz je
  Szene begrenzt.
- Entprelltes Autosave nach 700 Millisekunden mit sichtbaren Zuständen, serialisierten
  Schreibvorgängen und optimistischer Revision. Konflikte überschreiben keinen neueren Stand.
- Rust leitet Klartext aus Tiptap-JSON ab und aktualisiert Inhalt, Revision sowie FTS-Index
  transaktional.
- Automatisierte Tests für Editorvertrag, Baumordnung, Autosave, IPC-Roundtrip, Revisionskonflikt
  sowie Backup/Recovery mit 100 Szenen. Tastatur- und Screenreader-Fälle sind als manueller Testplan
  dokumentiert, aber noch nicht als E2E-Test ausgeführt.

### Begrenzte Hilfsfunktionen und Verträge

- `asset-service`: relative Pfade werden akzeptiert, absolute Pfade und Elterntraversal abgelehnt.
- `editor`: versionierter, generischer Snapshot-Vertrag.
- `ai-contracts`: Capability-, Provider- und explizite Kontextfreigabetypen.
- `visual-contracts`: Grundtypen für Medienassets und visuelle Profile.
- `export`, `git-service` und `continuity`: nur grundlegende Enums; keine ausführbare Fachlogik.

## Offene Aufgaben

### Unmittelbar: M1 erweitern

1. **KQ-006 - Story-Bibel** (GitHub-Issue noch anzulegen)
   - Entitäten, freie Eigenschaften, gerichtete Relationen und Rückverweise
   - Suche und nachvollziehbare Verknüpfung mit Szenen

### Danach gemäß Pflichtenheft

- KQ-006 bis KQ-016: Story-Bibel, Plotboard/Timeline, Git-Snapshots, Export, Textprovider,
  Kontinuität, Medienbibliothek, visuelle Profile, Prompt-Builder und Bild-Capability-Verträge.
- KQ-017 und KQ-018 gehören zu Version 0.2 und sind kein Ziel für 0.1.

Für KQ-006 bis KQ-018 sind noch nicht alle GitHub-Issues angelegt; die Aufgaben stehen im
Pflichtenheft. Bei Anlage müssen Akzeptanzkriterien und Abhängigkeiten konkretisiert werden.

## Bekannte Fehler und Lücken

- **Dokumentverwaltung noch unvollständig:** Erstellen, Listen, Laden und Speichern sind
  implementiert; Umbenennen, Löschen und Umordnen fehlen.
- **Feste Projektwurzel in der Desktopoberfläche:** Projekte liegen derzeit unter dem lokalen
  App-Datenverzeichnis. Eine frei wählbare Projektwurzel ist noch nicht umgesetzt.
- **Lokales Backup umfasst nur SQLite:** Assets sind noch nicht implementiert und daher nicht Teil
  der Sicherung. Portable Austausch- und Git-Snapshots sind ein späterer, getrennter Mechanismus.
- **Nur Initialmigration:** Der Migrationspfad 0 -> 1 ist getestet; weitere Versionen benötigen je
  eine neue unveränderliche Vorwärtsmigration samt Test mit befüllter Vorgängerdatenbank.
- **Keine End-to-End-Tests:** Playwright ist weder installiert noch konfiguriert.
- **Dependency Review eingeschränkt:** Der GitHub-Job wird bei privaten Repositories ohne GitHub
  Advanced Security bewusst übersprungen. Lockfiles und reguläre CI bleiben aktiv.
- Der aktuelle Teststand ist grün; es sind keine fehlschlagenden Tests auf `main` bekannt.

## Wichtige Architekturentscheidungen

- **SQLite als lokale Autorität, Git für portable Snapshots:** Die laufende Datenbank wird nicht
  versioniert. Exporte müssen deterministisch und wiederherstellbar sein (ADR 0001).
- **Reproduzierbares SQLite und atomare Vorwärtsmigrationen:** Gebündeltes SQLite stellt FTS5
  bereit; `PRAGMA user_version`, Fremdschlüssel, WAL und sofortige Transaktionen definieren den
  Verbindungs- und Migrationsvertrag (ADR 0002).
- **Kanonische Projektwurzel und nicht-destruktive Recovery:** Symlinks und unsichere relative Pfade
  werden abgelehnt; Projekte und Sicherungen werden erst nach Prüfung atomar veröffentlicht.
  Wiederherstellung schreibt ausschließlich in ein neues Ziel (ADR 0003).
- **Versioniertes Tiptap-JSON und optimistische Revisionen:** Jede Szene besitzt eine eigene
  Undo/Redo-Grenze; Autosave ist entprellt, serialisiert und konfliktbehaftete Schreibversuche
  überschreiben nichts (ADR 0004).
- **Privilegierte Operationen in Rust:** React kommuniziert ausschließlich über versionierte,
  typisierte Tauri-Commands mit Dateisystem, Storage, Git und späteren Providern.
- **Verträge sind explizit versioniert:** Inkompatible Änderungen erfordern eine neue
  `contractVersion`; TypeScript allein ist keine Sicherheitsgrenze.
- **Fehler sind datenarm:** IPC-Fehler enthalten keine Eingaben, lokalen Pfade, Stacktraces oder
  internen Diagnosedaten.
- **Offline und Opt-in:** Provider sind optional; Cloudzugriffe benötigen Vorschau und explizite
  Freigabe. Keine Telemetrie ab Werk.
- **Manuelle Übernahme:** KI-Vorschläge und Bildentwürfe verändern kanonische Inhalte niemals
  automatisch.

Offene Produktentscheidungen aus dem Pflichtenheft: Projektverzeichnis versus zusätzliches
Austauscharchiv, Umfang des DOCX-Roundtrips sowie Medien-Einbettung je Exportformat. Rechts- und
Lizenzfragen der Bildprovider sind erst vor Version 0.2 zu klären.

## Abhängigkeiten und technische Risiken

| Thema                        | Zustand / Risiko                                                             | Nächste Kontrolle                                         |
| ---------------------------- | ---------------------------------------------------------------------------- | --------------------------------------------------------- |
| SQLite und FTS5              | Dokument-Repository, FTS5, Backup und Recovery getestet; Entitäts-CRUD fehlt | KQ-006 mit Story-Bibel-Repository                         |
| Rust-/TS-Vertragsduplikation | Manuell gespiegelt; Drift ist möglich                                        | Gemeinsame Fixtures oder spätere Codegenerierung bewerten |
| Datenformat                  | Manifest und JSON-Schemas fehlen                                             | Mit KQ-008 und Snapshotarbeit versionieren                |
| Dateisystem                  | Komponentenprüfung aktiv; TOCTOU gegen böswillige Paralleländerung bleibt    | Vor nicht vertrauenswürdigen Importen Handle-APIs prüfen  |
| Editorformat                 | Tiptap-Vertrag v1; spätere inkompatible Änderungen brauchen neue Version     | Bei jeder Erweiterung Fixtures und Migration prüfen       |
| Desktoptests                 | Unit-/IPC-Tests vorhanden, aber keine Browser-/Tauri-E2E-Abdeckung           | Playwright für Projekt- und Schreibablauf einführen       |
| CI-Laufzeit                  | Kalte Tauri-/Rust-Builds unter Windows dauern mehrere Minuten                | Caching prüfen, ohne Reproduzierbarkeit zu schwächen      |
| Abhängigkeitssicherheit      | GitHub Dependency Review privat nicht verfügbar                              | Alternative Audits oder Advanced Security später bewerten |
| Plattformen                  | Windows ist primär; Linux baut in CI, ist fachlich aber nicht abgenommen     | Plattformmatrix je Release definieren                     |

## Nächste geplante Entwicklungsstufe

**KQ-006: Story-Bibel mit Entitäten, Relationen, Rückverweisen und Suche.**

Empfohlener, überprüfbarer Scope:

1. GitHub-Issue mit konkreten Akzeptanzkriterien und Abhängigkeit zu KQ-005 anlegen.
2. Versionierte Verträge für Figuren, Orte, Fraktionen und Gegenstände samt freien Eigenschaften
   definieren.
3. Repository für Entitäten, gerichtete Relationen und Szenenlinks transaktional implementieren.
4. Story-Bibel-Oberfläche mit Suche, Rückverweisen und barrierearmer Navigation bauen.
5. Tests für Relationstreue, Löschregeln, FTS-Suche und Backup/Recovery ergänzen.
6. Plotboard und Timeline getrennt halten. Auch Geschichten profitieren gelegentlich davon, wenn
   nicht jede Figur gleichzeitig die Bühne, die Beleuchtung und den Fahrplan verwaltet.

## Einstieg für die nächste Codex-Session

```powershell
cd D:\kraqel
git status
git branch --show-current
git log -5 --oneline
rg "KQ-006" docs ROADMAP.md
```

Danach lesen: `AGENTS.md`, dieses Dokument, ADR 0001 bis ADR 0004, `docs/data-format/README.md`,
`crates/storage/src/document_repository.rs` und den Editor-Testplan. Vor KQ-006 zuerst ein eigenes
Issue anlegen und den Scope gegen F02 des Pflichtenhefts prüfen.
