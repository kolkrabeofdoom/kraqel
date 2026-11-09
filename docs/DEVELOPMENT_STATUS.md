# Entwicklungsstatus

Stand: 9. November 2026

Code-Referenz: `496f243` (`KQ-002: Versionierte Domänen- und Command-Verträge`)

Produktstatus: frühes Entwicklungsfundament, nicht produktionsreif

Dieses Dokument trennt verifizierte Implementierungen von Planung. Maßgeblich sind Code, Tests, CI,
angenommene ADRs und das technische Pflichtenheft. Typen, Enums und leere Modulgerüste gelten nicht
als vollständige Fachfunktion.

## Status nach Pflichtenheft-Stufen

| Stufe        | Pflichtenheft-Ziel                                     | Verifizierter Stand                                                                                                                                            |
| ------------ | ------------------------------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **M0**       | Monorepo, Tauri, Datenbank, CI, Berechtigungsmodell    | **Teilweise umgesetzt:** Monorepo, Tauri-Shell, CI, minimale Capability und Command-Verträge vorhanden. SQLite/FTS5, Migrationen und Projektpersistenz fehlen. |
| **M1**       | Editor, Szenenbaum, Autosave, Backup                   | **Nicht implementiert.** Nur ein generischer Editor-Snapshot-Typ existiert.                                                                                    |
| **M2**       | Story-Bibel, Plotboard, Timeline                       | **Nicht implementiert.** Keine UI, Persistenz oder Fachlogik.                                                                                                  |
| **M3**       | Git, Import/Export, Medienbibliothek, visuelle Profile | **Nicht implementiert.** Es existieren nur Enums/Vertragstypen und eine relative Pfadprüfung.                                                                  |
| **M4**       | Text-KI, Kontinuität, Prompt-Entwürfe, Bildverträge    | **Nur Verträge vorbereitet:** Capability-, Kontextfreigabe-, Medien- und Profiltypen; keine Provider oder Fachabläufe.                                         |
| **M5**       | Windows-Release 0.1, Tests, Dokumentation              | **Nicht erreicht.** Debug-Build und CI funktionieren; kein Releasepaket und keine Abnahme der 0.1-Funktionen.                                                  |
| **M6 / 0.2** | Bildgenerierung, Referenzkonsistenz, Kartenmodell      | **Nicht begonnen und ausdrücklich außerhalb von 0.1.**                                                                                                         |

Die funktionalen Anforderungen F01 bis F09 sind damit noch nicht abgenommen. Vorhandene Verträge und
Gerüste sind technische Vorarbeit, keine fertigen Funktionen.

## Verifiziert implementiert und getestet

### Repository und Qualität

- pnpm- und Cargo-Monorepo mit gelockten Abhängigkeiten.
- Tauri-2-/React-/TypeScript-Desktop-Shell mit statischer, responsiver Startoberfläche.
- Minimale Tauri-Berechtigung `core:default`, definierte CSP und Windows-Fensterkonfiguration.
- GitHub-Actions-CI unter `windows-latest` und `ubuntu-latest`.
- Letzter CI-Lauf für `main` bei `496f243`: erfolgreich.
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

### Begrenzte Hilfsfunktionen und Verträge

- `asset-service`: relative Pfade werden akzeptiert, absolute Pfade und Elterntraversal abgelehnt.
- `editor`: versionierter, generischer Snapshot-Vertrag.
- `ai-contracts`: Capability-, Provider- und explizite Kontextfreigabetypen.
- `visual-contracts`: Grundtypen für Medienassets und visuelle Profile.
- `export`, `git-service` und `continuity`: nur grundlegende Enums; keine ausführbare Fachlogik.

## Offene Aufgaben

### Unmittelbar: M0 abschließen

1. **KQ-003 - SQLite-Schema, Migrationen und FTS5** (`status: ready`)
   - initiale Tabellen für `project`, `document`, `entity`, `relation` und `link`
   - Foreign Keys, Transaktionen, FTS5 und Migrationstests
   - einzige zentrale Schema-Version und Ablehnung unbekannter Zukunftsschemas
2. **KQ-004 - Lokaler Projektmanager, Backup und Recovery** (`status: ready`)
   - sichere Projektpfade, atomare Schreibvorgänge, Backup und nicht-destruktive Wiederherstellung
3. **KQ-005 - Editor, Szenenbaum und Autosave** (`status: triage`)
   - erst nach stabilen Domänen- und Persistenzverträgen sinnvoll umzusetzen

### Danach gemäß Pflichtenheft

- KQ-006 bis KQ-016: Story-Bibel, Plotboard/Timeline, Git-Snapshots, Export, Textprovider,
  Kontinuität, Medienbibliothek, visuelle Profile, Prompt-Builder und Bild-Capability-Verträge.
- KQ-017 und KQ-018 gehören zu Version 0.2 und sind kein Ziel für 0.1.

Für KQ-006 bis KQ-018 sind noch nicht alle GitHub-Issues angelegt; die Aufgaben stehen im
Pflichtenheft. Bei Anlage müssen Akzeptanzkriterien und Abhängigkeiten konkretisiert werden.

## Bekannte Fehler und Lücken

- **Schema-Version inkonsistent:** `crates/domain` und `packages/domain` verwenden Version `1`,
  `crates/storage` verwendet noch den Platzhalter `0`. Das Storage-Crate ist nicht integriert;
  KQ-003 muss die doppelte Quelle entfernen oder eindeutig ableiten.
- **Keine Persistenz:** `crates/storage` enthält weder SQLite-Abhängigkeit noch Schema, Migrationen,
  FTS5 oder Repositoryimplementierung.
- **Startoberfläche ohne Funktion:** „Neues Projekt“ und „Projekt öffnen“ besitzen noch keine
  Handler. Die vorhandenen Commands werden von der sichtbaren UI nicht aufgerufen.
- **Keine End-to-End-Tests:** Playwright ist weder installiert noch konfiguriert.
- **Dependency Review eingeschränkt:** Der GitHub-Job wird bei privaten Repositories ohne GitHub
  Advanced Security bewusst übersprungen. Lockfiles und reguläre CI bleiben aktiv.
- Der aktuelle Teststand ist grün; es sind keine fehlschlagenden Tests auf `main` bekannt.

## Wichtige Architekturentscheidungen

- **SQLite als lokale Autorität, Git für portable Snapshots:** Die laufende Datenbank wird nicht
  versioniert. Exporte müssen deterministisch und wiederherstellbar sein (ADR 0001).
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

| Thema                        | Zustand / Risiko                                                         | Nächste Kontrolle                                         |
| ---------------------------- | ------------------------------------------------------------------------ | --------------------------------------------------------- |
| SQLite und FTS5              | Noch keine Bibliothek oder Migration; M0 blockiert                       | KQ-003 mit Migrations- und FTS5-Tests                     |
| Rust-/TS-Vertragsduplikation | Manuell gespiegelt; Drift ist möglich                                    | Gemeinsame Fixtures oder spätere Codegenerierung bewerten |
| Datenformat                  | Manifest und JSON-Schemas fehlen                                         | Mit KQ-003 und Snapshotarbeit versionieren                |
| Dateisystem                  | Nur lexikalische relative Pfadprüfung; keine Symlink-/Kanonikprüfung     | KQ-004 mit Windows-Pfadtests                              |
| Editorformat                 | Nur generischer Snapshot; Tiptap noch nicht eingebunden                  | Vor KQ-005 Versionierung festlegen                        |
| Desktoptests                 | Keine E2E-Abdeckung                                                      | Playwright erst mit ersten nutzbaren Abläufen einführen   |
| CI-Laufzeit                  | Kalte Tauri-/Rust-Builds unter Windows dauern mehrere Minuten            | Caching prüfen, ohne Reproduzierbarkeit zu schwächen      |
| Abhängigkeitssicherheit      | GitHub Dependency Review privat nicht verfügbar                          | Alternative Audits oder Advanced Security später bewerten |
| Plattformen                  | Windows ist primär; Linux baut in CI, ist fachlich aber nicht abgenommen | Plattformmatrix je Release definieren                     |

## Nächste geplante Entwicklungsstufe

**KQ-003: SQLite-Schema, Migrationen, FTS5 und Migrationstests.**

Empfohlener, überprüfbarer Scope:

1. ADR-Ergänzung für SQLite-Bibliothek, Connection-Pragmas und Migrationsstrategie.
2. Zentrale Schema-Version aus `kraqel-domain` verwenden; Storage-Platzhalter `0` beseitigen.
3. Initialmigration für `project`, `document`, `entity`, `relation` und `link` mit
   UUID-Textschlüsseln, Foreign Keys und sinnvollen Indizes.
4. FTS5 zunächst für Szenen-/Dokumenttext mit explizitem Synchronisationsmodell.
5. Tests für leere Datenbank, Vorwärtsmigration, Foreign-Key-Verletzung, FTS-Suche und unbekannte
   Zukunftsversion.
6. Noch keine Medien-, Timeline-, KI- oder Editorfunktion in diesen Scope ziehen; ein Sumpf wird
   nicht trockener, wenn man mehr Frösche hineinsetzt.

## Einstieg für die nächste Codex-Session

```powershell
cd D:\kraqel
git status
git branch --show-current
git log -5 --oneline
gh issue view 3
```

Danach lesen: `AGENTS.md`, dieses Dokument, ADR 0001, `docs/data-format/README.md`,
`crates/domain/src/lib.rs` und `crates/storage/src/lib.rs`. Für KQ-003 einen eigenen Branch wie
`feat/kq-003-sqlite-schema` verwenden und den Issue-Status erst bei tatsächlichem Arbeitsbeginn
ändern.
