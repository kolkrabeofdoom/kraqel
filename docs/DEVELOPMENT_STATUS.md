# Entwicklungsstatus

Stand: 9. November 2026

Code-Referenz: KQ-009 (`feat/kq-009-export-pipeline`, Arbeitsstand)

Produktstatus: frühes Entwicklungsfundament, nicht produktionsreif

Dieses Dokument trennt verifizierte Implementierungen von Planung. Maßgeblich sind Code, Tests, CI,
angenommene ADRs und das technische Pflichtenheft. Typen, Enums und leere Modulgerüste gelten nicht
als vollständige Fachfunktion.

## Status nach Pflichtenheft-Stufen

| Stufe        | Pflichtenheft-Ziel                                     | Verifizierter Stand                                                                                                                                                                                                                      |
| ------------ | ------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **M0**       | Monorepo, Tauri, Datenbank, CI, Berechtigungsmodell    | **Technisches Fundament umgesetzt:** Monorepo, Tauri-Shell, CI, minimale Capability, Command-Verträge, SQLite/FTS5 sowie sicherer Projektlebenszyklus mit Backup und Recovery vorhanden.                                                 |
| **M1**       | Editor, Szenenbaum, Autosave, Backup                   | **Teilweise umgesetzt:** Projekt-/Kapitel-/Szenenoberfläche, Tiptap, szenengebundenes Undo/Redo, Wortzählung, Fokusmodus, revisionsgeprüftes Autosave und SQLite-Backup vorhanden. Umbenennen, Umordnen, Löschen und E2E-Abnahme fehlen. |
| **M2**       | Story-Bibel, Plotboard, Timeline                       | **Technischer Mindestumfang umgesetzt:** Story-Bibel, persistentes Plotboard sowie parallele relative/absolute Timelines mit Szenen- und Entitätslinks vorhanden. Browser-/Screenreader-Abnahme fehlt.                                   |
| **M3**       | Git, Import/Export, Medienbibliothek, visuelle Profile | **Teilweise umgesetzt:** deterministische Projektsnapshots, lokales Git sowie Manuskriptexport als Markdown, HTML, DOCX und PDF vorhanden. Import, schreibende Snapshot-Wiederherstellung, Medien und visuelle Profile fehlen.           |
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
- Letzter CI-Lauf für `main` bei `667666f`: erfolgreich unter Windows und Linux.
- Qualitätskette: Prettier, rustfmt, ESLint, Clippy mit `-D warnings`, TypeScript, Vitest,
  Rust-Tests sowie Frontend- und Workspace-Builds.
- GPL-3.0-or-later, Beitrags-, Verhaltens- und Sicherheitsdokumentation vorhanden.

### Domäne und IPC-Grenze (KQ-002)

- Kanonische UUID-Typen für Projekte und Entitäten in Rust und TypeScript.
- Portable Projektformat-Version `1` in den Domänenmodulen; interner SQLite-Migrationsstand `2`.
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
- Vorwärtsmigration v2 für `storyline`, `plot_card`, `timeline`, `event` und deren Linktabellen.
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

### Story-Bibel (KQ-006)

- Gemeinsame Rust-/TypeScript-Verträge für Figuren, Orte, Fraktionen, Gegenstände, freie
  Eigenschaften, gerichtete Relationen und Szenen-Rückverweise.
- Transaktionales `StoryBibleRepository` auf den vorhandenen Tabellen `entity`, `relation` und
  `link`; Namen und freie Eigenschaften werden begrenzt und in Rust validiert.
- Einträge lassen sich anlegen, suchen, nach Art filtern, bearbeiten und löschen. Die Suche prüft
  Namen, Eigenschaftsschlüssel und -werte Unicode-bewusst, ist derzeit aber nicht FTS-indexiert.
- Beziehungen bewahren Quelle und Ziel; die Detailansicht trennt eingehende und ausgehende Kanten.
  Selbstbeziehungen sind im Vertrag v1 nicht erlaubt. Löschkaskaden entfernen verwaiste Kanten.
- Szenenlinks werden idempotent angelegt und als Rückverweise angezeigt; aus einem Rückverweis kann
  wieder in die betreffende Szene gewechselt werden.
- Zugängliche React-Oberfläche mit beschrifteten Formularen, sichtbaren Fokuszuständen,
  Tastaturbedienung sowie textlich erkennbarer Beziehungsrichtung.
- Automatisierte Repository-Tests decken Suche, Eigenschaften, Richtungstreue, Löschregeln,
  Rückverweise und Backup/Recovery ab. IPC- und TypeScript-Tests prüfen den vertikalen Vertrag.
  Browser-/Screenreader-Abnahme bleibt als manueller Testfall dokumentiert.

### Plotboard und parallele Timelines (KQ-007)

- SQLite-Migration v1→v2 ergänzt Handlungsstränge, Szenenkarten, Zeitebenen, Ereignisse sowie
  Szenen-/Entitätslinks; eine befüllte v1-Datenbank bleibt erhalten.
- Technischer Datenbankstand (`PRAGMA user_version = 2`) und portables Projektformat
  (`project.schema_version = 1`) sind nun explizit getrennt.
- Jede Szene besitzt eine persistente Karte mit stabiler Reihenfolge, Status, optionalem
  Handlungsstrang und optionaler POV-Figur.
- Karten lassen sich nativ per Drag-and-drop sowie gleichwertig über beschriftete Tasten neu ordnen.
- Mehrere relative oder absolute Zeitebenen sind möglich. Relative Ereignisse verwenden eine
  Sequenz; absolute Ereignisse verlangen einen expliziten Zeitwert.
- Ereignisse lassen sich idempotent mit Szenen und Story-Bibel-Einträgen verknüpfen. Szenenlinks
  führen zurück ins Manuskript.
- Repository-Tests decken Reihenfolge, Status, POV, Zeitmodell, Links, Migration und Backup/Recovery
  ab. Alte v1-Sicherungen werden nur in der neuen Zielkopie migriert; die Quelle bleibt unverändert.
  TypeScript-Vertragstests prüfen die vollständige Kartenreihenfolge.

### Manuskriptexport (KQ-009)

- Gemeinsame, streng validierte Zwischenrepräsentation für Projekt, Kapitel, Szenen, Absätze,
  Überschriften, Fett/Kursiv, harte Umbrüche, Listen und Blockzitate.
- Unbekannte Tiptap-Knoten und -Markierungen brechen den Export mit bereinigtem Fehler ab; Inhalte
  werden nicht still verworfen.
- Markdown und eigenständiges HTML in UTF-8, DOCX im Letter-Seitenformat sowie PDF in A4 werden aus
  derselben fachlichen Repräsentation erzeugt.
- Exporte entstehen atomar und ausschließlich unter `exports/manuskript.<format>` im lokalen
  Projektordner. Symlinks und nicht reguläre Zielobjekte werden abgelehnt.
- Versionierter Tauri-Command und zugängliche React-Ansicht **Export** bieten alle vier Formate an
  und melden Dateiname sowie Größe, aber keinen absoluten lokalen Pfad.
- Automatisierte Tests prüfen alle Formate, Textformatierung und die Ablehnung unbekannter Knoten.
  DOCX und PDF wurden zusätzlich als Seiten gerendert und visuell geprüft; für DOCX war unter
  Windows wegen fehlendem LibreOffice eine Word-basierte Konvertierung nötig.
- Nicht Teil dieses Inkrements: Import, DOCX-Roundtrip, Medien, Fußnoten, Tabellen, Links und
  beliebige Tiptap-Erweiterungen.

### Begrenzte Hilfsfunktionen und Verträge

- `asset-service`: relative Pfade werden akzeptiert, absolute Pfade und Elterntraversal abgelehnt.
- `editor`: versionierter, generischer Snapshot-Vertrag.
- `ai-contracts`: Capability-, Provider- und explizite Kontextfreigabetypen.
- `visual-contracts`: Grundtypen für Medienassets und visuelle Profile.
- `continuity`: nur ein grundlegendes Enum; keine ausführbare Fachlogik.

### Deterministische Snapshots und lokales Git (KQ-008)

- Portables Snapshotformat v1 für Projektmetadaten, Dokumente, Story-Bibel, Plotboard und Timelines;
  ein formales JSON-Schema wird mit jedem Snapshot kopiert.
- UUID-basierte Markdown-Dateinamen, stabile fachliche Sortierung, UTF-8/LF und keine flüchtigen
  Datenbankzeitstempel. Wiederholte Erzeugung unveränderter Projekte ist byte-identisch getestet.
- Snapshotstände entstehen in temporären Geschwisterverzeichnissen und werden als vollständiger
  Verzeichnistausch veröffentlicht.
- Deny-by-default `.gitignore` sowie feste Pfadangaben beschränken Init, Add, Status, Diff und
  Commit auf `.gitignore` und `snapshot/**`. SQLite, WAL, Sicherungen und Nebenartefakte bleiben
  ungetrackt.
- Globale Git-Konfiguration, Systemkonfiguration, Hooks, externe Diffs, Signaturzwang und
  interaktive Prompts sind für Dienstoperationen deaktiviert.
- Versionierte Tauri-Commands und die zugängliche Ansicht **Versionen** bieten Snapshoterzeugung,
  lesende Prüfung, Status, Unified Diff und expliziten Commit.
- Automatisierte Tests prüfen Byte-Stabilität, Manipulationserkennung, Ausschluss der Datenbank und
  Git-Roundtrip. Browser-/Screenreader-Abnahme ist nur als manueller Testplan dokumentiert.
- Die Prüfung ist noch keine schreibende Wiederherstellung: Ein Snapshot wird derzeit nicht in ein
  neues SQLite-Projekt importiert.

## Offene Aufgaben

### Unmittelbar: M3 fortsetzen

1. **KQ-010 - Providerunabhängige Text-KI-Verträge und explizite Kontextfreigabe**
   - konkreten Scope und Akzeptanzkriterien vor Implementierung aus dem Pflichtenheft ableiten
   - kein Netzwerkzugriff und keine Providerkopplung ohne eigene Freigabeentscheidung

### Danach gemäß Pflichtenheft

- KQ-010 bis KQ-016: Textprovider, Kontinuität, Medienbibliothek, visuelle Profile, Prompt-Builder
  und Bild-Capability-Verträge.
- KQ-017 und KQ-018 gehören zu Version 0.2 und sind kein Ziel für 0.1.

Für KQ-009 bis KQ-018 sind noch nicht alle GitHub-Issues angelegt; die Aufgaben stehen im
Pflichtenheft. Bei Anlage müssen Akzeptanzkriterien und Abhängigkeiten konkretisiert werden.

## Bekannte Fehler und Lücken

- **Dokumentverwaltung noch unvollständig:** Erstellen, Listen, Laden und Speichern sind
  implementiert; Umbenennen, Löschen und Umordnen fehlen.
- **Feste Projektwurzel in der Desktopoberfläche:** Projekte liegen derzeit unter dem lokalen
  App-Datenverzeichnis. Eine frei wählbare Projektwurzel ist noch nicht umgesetzt.
- **Lokales Backup umfasst nur SQLite:** Assets sind noch nicht implementiert und daher nicht Teil
  der Sicherung. Portable Git-Snapshots sind ein getrennter Mechanismus und enthalten derzeit keine
  Assets.
- **Snapshot-Wiederherstellung nur vorbereitet:** Format und Inhalte werden lesend geprüft, aber
  noch nicht in eine neue Datenbank importiert. F07 ist deshalb noch nicht vollständig abgenommen.
- **Export ist kein Import:** Die vier Ausgaben sind erzeugbar; ein Roundtrip zurück nach KRAQEL und
  der im Pflichtenheft genannte Import sind nicht implementiert. F08 ist deshalb nur teilweise
  erfüllt.
- **Exportformatierung bewusst begrenzt:** Links, Code, Tabellen, Fußnoten und Medien werden nicht
  still reduziert, sondern derzeit abgelehnt. PDF verwendet eingebaute Standardschriften; vor einer
  breiteren Unicode-Abnahme ist eine eingebettete freie Schrift zu prüfen.
- **Migrationen bleiben vorwärtsgerichtet:** 0→1 und die befüllte Migration 1→2 sind getestet;
  weitere Versionen benötigen jeweils eine neue unveränderliche Migration.
- **Zeitwerte ohne Kalendersemantik:** Absolute Zeitwerte sind validierte Zeichenketten. Kalender,
  Zeitzonen und Arithmetik sind bewusst noch nicht implementiert.
- **Keine End-to-End-Tests:** Playwright ist weder installiert noch konfiguriert.
- **Entitätssuche nicht indexiert:** Die Unicode-korrekte Suche filtert derzeit alle Einträge eines
  Projekts in Rust. Vor sehr großen Story-Bibeln sind Messung und gegebenenfalls eine echte
  FTS-Migration nötig.
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
- **Begrenzte Story-Verträge auf Schema v1:** Vier Entitätsarten und strukturierte freie
  Eigenschaften werden in Rust validiert; Relationen sind gerichtet, Szenenlinks idempotent und
  Löschkaskaden explizit. Die Unicode-Suche bleibt zunächst bewusst unindexiert (ADR 0005).
- **Getrennte Datenbankversion und explizites Zeitmodell:** SQLite-Migrationen sind vom portablen
  Projektformat getrennt; relative und absolute Zeitebenen erzwingen unterschiedliche Zeitwerte und
  Links bleiben n:m (ADR 0006).
- **Portabler Snapshot und kontrolliertes Git:** Nur `.gitignore` und `snapshot/**` sind
  freigegeben; globale Konfiguration, Hooks und interaktive Git-Abläufe bleiben deaktiviert. Die
  erste Wiederherstellungsstufe ist lesende Validierung (ADR 0007).
- **Ein Exportmodell für vier Renderer:** Die unterstützte Tiptap-Teilmenge wird vor jeder Ausgabe
  vollständig validiert. Renderer erhalten nur das gemeinsame Zwischenformat; Ziele werden lokal und
  atomar veröffentlicht (ADR 0008).
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

| Thema                        | Zustand / Risiko                                                                                            | Nächste Kontrolle                                         |
| ---------------------------- | ----------------------------------------------------------------------------------------------------------- | --------------------------------------------------------- |
| SQLite und FTS5              | Dokument-FTS sowie Story-Bibel-CRUD, Relationen, Links und Recovery getestet; Entitätssuche nicht indexiert | Vor großen Projekten messen; Migration nur mit Evidenz    |
| Rust-/TS-Vertragsduplikation | Manuell gespiegelt; Drift ist möglich                                                                       | Gemeinsame Fixtures oder spätere Codegenerierung bewerten |
| Datenformat                  | Snapshot v1 und vier Textexporte vorhanden; Import, Roundtrip und Assetreferenzen fehlen                    | Import/Roundtrip vor vollständiger F07/F08-Abnahme testen |
| Exportrenderer               | DOCX/PDF gerendert geprüft; PDF-Standardschrift begrenzt Unicode, Medien fehlen                             | Freie eingebettete Schrift und Medienkonzept prüfen       |
| Dateisystem                  | Komponentenprüfung aktiv; TOCTOU gegen böswillige Paralleländerung bleibt                                   | Vor nicht vertrauenswürdigen Importen Handle-APIs prüfen  |
| Editorformat                 | Tiptap-Vertrag v1; spätere inkompatible Änderungen brauchen neue Version                                    | Bei jeder Erweiterung Fixtures und Migration prüfen       |
| Desktoptests                 | Unit-/IPC-Tests vorhanden, aber keine Browser-/Tauri-E2E-Abdeckung                                          | Playwright für Projekt- und Schreibablauf einführen       |
| CI-Laufzeit                  | Kalte Tauri-/Rust-Builds unter Windows dauern mehrere Minuten                                               | Caching prüfen, ohne Reproduzierbarkeit zu schwächen      |
| Abhängigkeitssicherheit      | GitHub Dependency Review privat nicht verfügbar                                                             | Alternative Audits oder Advanced Security später bewerten |
| Plattformen                  | Windows ist primär; Linux baut in CI, ist fachlich aber nicht abgenommen                                    | Plattformmatrix je Release definieren                     |

## Nächste geplante Entwicklungsstufe

**KQ-010: Providerunabhängige Text-KI-Verträge und explizite Kontextfreigabe.**

Empfohlener, überprüfbarer Scope:

1. Pflichtenheft-Anforderungen für F04 und M4 gegen vorhandene `ai-contracts` abgleichen.
2. GitHub-Issue mit expliziter Datenfluss-, Vorschau- und Einwilligungsgrenze anlegen.
3. Lokale Mock-Provider-Schnittstelle und Fehlervertrag definieren; noch keinen Cloudanbieter
   anbinden.
4. Tests sicherstellen lassen, dass ohne Freigabe kein Kontext den Prozess verlässt.

## Einstieg für die nächste Codex-Session

```powershell
cd D:\kraqel
git status
git branch --show-current
git log -5 --oneline
rg "KQ-010" docs ROADMAP.md
```

Danach lesen: `AGENTS.md`, dieses Dokument, ADR 0001 bis ADR 0007, `docs/data-format/README.md`,
`packages/ai-contracts/` und die Zugänglichkeitstestpläne. Vor KQ-010 zuerst ein eigenes Issue
anlegen und den Scope gegen F04 und M4 prüfen.
