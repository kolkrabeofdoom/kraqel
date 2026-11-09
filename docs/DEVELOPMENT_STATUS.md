# Entwicklungsstatus

Stand: 9. November 2026

Code-Referenz: KQ-004 (`feat/kq-004-project-manager`, dieser Stand)

Produktstatus: frühes Entwicklungsfundament, nicht produktionsreif

Dieses Dokument trennt verifizierte Implementierungen von Planung. Maßgeblich sind Code, Tests, CI,
angenommene ADRs und das technische Pflichtenheft. Typen, Enums und leere Modulgerüste gelten nicht
als vollständige Fachfunktion.

## Status nach Pflichtenheft-Stufen

| Stufe        | Pflichtenheft-Ziel                                     | Verifizierter Stand                                                                                                                                                                      |
| ------------ | ------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **M0**       | Monorepo, Tauri, Datenbank, CI, Berechtigungsmodell    | **Technisches Fundament umgesetzt:** Monorepo, Tauri-Shell, CI, minimale Capability, Command-Verträge, SQLite/FTS5 sowie sicherer Projektlebenszyklus mit Backup und Recovery vorhanden. |
| **M1**       | Editor, Szenenbaum, Autosave, Backup                   | **Nicht implementiert.** Nur ein generischer Editor-Snapshot-Typ existiert.                                                                                                              |
| **M2**       | Story-Bibel, Plotboard, Timeline                       | **Nicht implementiert.** Keine UI, Persistenz oder Fachlogik.                                                                                                                            |
| **M3**       | Git, Import/Export, Medienbibliothek, visuelle Profile | **Nicht implementiert.** Es existieren nur Enums/Vertragstypen und eine relative Pfadprüfung.                                                                                            |
| **M4**       | Text-KI, Kontinuität, Prompt-Entwürfe, Bildverträge    | **Nur Verträge vorbereitet:** Capability-, Kontextfreigabe-, Medien- und Profiltypen; keine Provider oder Fachabläufe.                                                                   |
| **M5**       | Windows-Release 0.1, Tests, Dokumentation              | **Nicht erreicht.** Debug-Build und CI funktionieren; kein Releasepaket und keine Abnahme der 0.1-Funktionen.                                                                            |
| **M6 / 0.2** | Bildgenerierung, Referenzkonsistenz, Kartenmodell      | **Nicht begonnen und ausdrücklich außerhalb von 0.1.**                                                                                                                                   |

Die funktionalen Anforderungen F01 bis F09 sind damit noch nicht abgenommen. Vorhandene Verträge und
Gerüste sind technische Vorarbeit, keine fertigen Funktionen.

## Verifiziert implementiert und getestet

### Repository und Qualität

- pnpm- und Cargo-Monorepo mit gelockten Abhängigkeiten.
- Tauri-2-/React-/TypeScript-Desktop-Shell mit statischer, responsiver Startoberfläche.
- Minimale Tauri-Berechtigung `core:default`, definierte CSP und Windows-Fensterkonfiguration.
- GitHub-Actions-CI unter `windows-latest` und `ubuntu-latest`.
- Letzter CI-Lauf für `main` bei `479d0b2`: erfolgreich unter Windows und Linux.
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

### Begrenzte Hilfsfunktionen und Verträge

- `asset-service`: relative Pfade werden akzeptiert, absolute Pfade und Elterntraversal abgelehnt.
- `editor`: versionierter, generischer Snapshot-Vertrag.
- `ai-contracts`: Capability-, Provider- und explizite Kontextfreigabetypen.
- `visual-contracts`: Grundtypen für Medienassets und visuelle Profile.
- `export`, `git-service` und `continuity`: nur grundlegende Enums; keine ausführbare Fachlogik.

## Offene Aufgaben

### Unmittelbar: M1 beginnen

1. **KQ-005 - Editor, Szenenbaum und Autosave** (`status: triage`)
   - Tiptap-Vertrag und Dokument-Repository zuerst konkretisieren
   - Szenen-/Kapitelbaum, transaktionales Autosave, Undo/Redo und Wortzählung umsetzen

### Danach gemäß Pflichtenheft

- KQ-006 bis KQ-016: Story-Bibel, Plotboard/Timeline, Git-Snapshots, Export, Textprovider,
  Kontinuität, Medienbibliothek, visuelle Profile, Prompt-Builder und Bild-Capability-Verträge.
- KQ-017 und KQ-018 gehören zu Version 0.2 und sind kein Ziel für 0.1.

Für KQ-006 bis KQ-018 sind noch nicht alle GitHub-Issues angelegt; die Aufgaben stehen im
Pflichtenheft. Bei Anlage müssen Akzeptanzkriterien und Abhängigkeiten konkretisiert werden.

## Bekannte Fehler und Lücken

- **Noch keine Dokument-Repository-Schicht:** Der Projektlebenszyklus ist implementiert, fachliche
  CRUD-Operationen für Dokumente und Entitäten fehlen. Die sichtbare Anwendung öffnet noch keine
  Projektdatenbank.
- **Lokales Backup umfasst nur SQLite:** Assets sind noch nicht implementiert und daher nicht Teil
  der Sicherung. Portable Austausch- und Git-Snapshots sind ein späterer, getrennter Mechanismus.
- **Nur Initialmigration:** Der Migrationspfad 0 -> 1 ist getestet; weitere Versionen benötigen je
  eine neue unveränderliche Vorwärtsmigration samt Test mit befüllter Vorgängerdatenbank.
- **Startoberfläche ohne Funktion:** „Neues Projekt“ und „Projekt öffnen“ besitzen noch keine
  Handler. Die vorhandenen Commands werden von der sichtbaren UI nicht aufgerufen.
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

| Thema                        | Zustand / Risiko                                                          | Nächste Kontrolle                                         |
| ---------------------------- | ------------------------------------------------------------------------- | --------------------------------------------------------- |
| SQLite und FTS5              | Schema, Migration, FTS5, Backup und Recovery getestet; Fach-CRUD fehlt    | KQ-005 mit Dokument-Repository und Autosave               |
| Rust-/TS-Vertragsduplikation | Manuell gespiegelt; Drift ist möglich                                     | Gemeinsame Fixtures oder spätere Codegenerierung bewerten |
| Datenformat                  | Manifest und JSON-Schemas fehlen                                          | Mit KQ-008 und Snapshotarbeit versionieren                |
| Dateisystem                  | Komponentenprüfung aktiv; TOCTOU gegen böswillige Paralleländerung bleibt | Vor nicht vertrauenswürdigen Importen Handle-APIs prüfen  |
| Editorformat                 | Nur generischer Snapshot; Tiptap noch nicht eingebunden                   | Vor KQ-005 Versionierung festlegen                        |
| Desktoptests                 | Keine E2E-Abdeckung                                                       | Playwright erst mit ersten nutzbaren Abläufen einführen   |
| CI-Laufzeit                  | Kalte Tauri-/Rust-Builds unter Windows dauern mehrere Minuten             | Caching prüfen, ohne Reproduzierbarkeit zu schwächen      |
| Abhängigkeitssicherheit      | GitHub Dependency Review privat nicht verfügbar                           | Alternative Audits oder Advanced Security später bewerten |
| Plattformen                  | Windows ist primär; Linux baut in CI, ist fachlich aber nicht abgenommen  | Plattformmatrix je Release definieren                     |

## Nächste geplante Entwicklungsstufe

**KQ-005: Editor, Szenenbaum und Autosave.**

Empfohlener, überprüfbarer Scope:

1. Persistenzvertrag für Kapitel, Szenen und Tiptap-JSON festziehen; Revisionen und
   Konfliktverhalten explizit definieren.
2. Transaktionale Repository-Operationen für Baumabfragen, Erstellen, Umordnen und Speichern bauen.
3. Tiptap/ProseMirror integrieren und Autosave ausschließlich über versionierte Tauri-Commands
   führen.
4. Undo/Redo, Wortzählung und grundlegende Tastaturbedienung testen.
5. Recovery-Test vom gespeicherten Szenenstand bis zur wiederhergestellten Oberfläche ergänzen.
6. Story-Bibel und Plotboard noch nicht hineinziehen. Ein Editor wird nicht besser, wenn man ihm
   vorsorglich drei Nebenberufe und einen Umhang gibt.

## Einstieg für die nächste Codex-Session

```powershell
cd D:\kraqel
git status
git branch --show-current
git log -5 --oneline
gh issue view 5
```

Danach lesen: `AGENTS.md`, dieses Dokument, ADR 0001 bis ADR 0003, `docs/data-format/README.md`,
`packages/editor/`, `crates/domain/src/lib.rs` und `crates/storage/src/`. Für KQ-005 einen eigenen
Branch wie `feat/kq-005-editor-autosave` verwenden und den Issue-Status erst bei tatsächlichem
Arbeitsbeginn ändern.
