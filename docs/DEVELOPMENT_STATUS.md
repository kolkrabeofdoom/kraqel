# Entwicklungsstatus

Stand: 11. November 2026

Code-Referenz: KQ-031 (PR #59)

Produktstatus: frühes Entwicklungsfundament, nicht produktionsreif

Dieses Dokument trennt verifizierte Implementierungen von Planung. Maßgeblich sind Code, Tests, CI,
angenommene ADRs und das technische Pflichtenheft. Typen, Enums und leere Modulgerüste gelten nicht
als vollständige Fachfunktion.

## Status nach Pflichtenheft-Stufen

| Stufe        | Pflichtenheft-Ziel                                     | Verifizierter Stand                                                                                                                                                                                                                                                                                                                                  |
| ------------ | ------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **M0**       | Monorepo, Tauri, Datenbank, CI, Berechtigungsmodell    | **Technisches Fundament umgesetzt:** Monorepo, Tauri-Shell, CI, minimale Capability, Command-Verträge, SQLite/FTS5 sowie sicherer Projektlebenszyklus mit Backup und Recovery vorhanden.                                                                                                                                                             |
| **M1**       | Editor, Szenenbaum, Autosave, Backup                   | **Technischer Mindestumfang samt Browser-E2E umgesetzt:** Projekt-/Kapitel-/Szenenoberfläche, Tiptap, Undo/Redo, Wortzählung, Fokusmodus, konfliktgeprüfter Autosave sowie sicheres Umbenennen, Umordnen, Löschen und SQLite-Backup vorhanden. Echte Tauri-/Screenreader-Abnahme fehlt.                                                              |
| **M2**       | Story-Bibel, Plotboard, Timeline                       | **Technischer Mindestumfang samt Browser-E2E umgesetzt:** Story-Bibel, persistentes Plotboard sowie parallele relative/absolute Timelines mit Szenen- und Entitätslinks sind in getrennten Chromium-Abläufen geprüft. Tauri- und Screenreader-Abnahme fehlen.                                                                                        |
| **M3**       | Git, Import/Export, Medienbibliothek, visuelle Profile | **Teilweise umgesetzt:** Snapshots, lokales Git, vier Manuskriptexporte, Medienbibliothek sowie visuelle Profile und Stilvorlagen sind im browserseitigen Kernablauf geprüft. Backup und Snapshot sind vorhanden; Textimport und schreibende Snapshot-Wiederherstellung fehlen.                                                                      |
| **M4**       | Text-KI, Kontinuität, Prompt-Entwürfe, Bildverträge    | **Technischer 0.1-Umfang weitgehend umgesetzt:** Textprotokolle, browserseitig geprüfte Offline-Kontinuität und Prompt-Entwürfe sowie Bild-Capability- und Jobverträge vorhanden. Echte Providerabnahme, KI-Kontinuitätshinweise und Bildgenerierung fehlen.                                                                                         |
| **M5**       | Windows-Release 0.1, Tests, Dokumentation              | **Teilweise begonnen:** Debug-Build, echter Windows-Tauri-Smoke-Test und ein isolierter unsignierter NSIS-Testinstaller mit geprüftem lokalen Lebenszyklus sind vorhanden. Zweitrechner-, MSI-, Signierungs-, Screenreader- und vollständige 0.1-Abnahme fehlen. Die Zweitrechnerprüfung ist bewusst auf einen späteren manuellen Termin verschoben. |
| **M6 / 0.2** | Bildgenerierung, Referenzkonsistenz, Kartenmodell      | **Nicht begonnen und ausdrücklich außerhalb von 0.1.**                                                                                                                                                                                                                                                                                               |

Die funktionalen Anforderungen F01 bis F09 sind damit noch nicht abgenommen. Vorhandene Verträge und
Gerüste sind technische Vorarbeit, keine fertigen Funktionen.

## Verifiziert implementiert und getestet

### Repository und Qualität

- pnpm- und Cargo-Monorepo mit gelockten Abhängigkeiten.
- Tauri-2-/React-/TypeScript-Desktop-Shell mit statischer, responsiver Startoberfläche.
- Minimale Tauri-Berechtigung `core:default`, definierte CSP und Windows-Fensterkonfiguration.
- GitHub-Actions-CI unter `windows-latest` und `ubuntu-latest`.
- Playwright mit Chromium und einem testexklusiven Browser-Command-Adapter für M1, Story-Bibel,
  Plotboard, Timelines, Medien, visuelle Profile, Kontinuität, Bildprompt-Builder, Versionen und
  Export; die CI führt zehn fachlich getrennte Abläufe unter Windows und Linux aus.
- Qualitätskette: Prettier, rustfmt, ESLint, Clippy mit `-D warnings`, TypeScript, Vitest,
  Rust-Tests sowie Frontend- und Workspace-Builds.
- Separater Windows-Tauri-Smoke-Test mit realem WebView, versionierten IPC-Commands, Rust und
  isolierter SQLite-Persistenz; er ist bewusst nicht Bestandteil der plattformübergreifenden
  Browser-E2E.
- Isolierter Windows-NSIS-Testinstaller mit eigenem Produktnamen, eigener Bundle-ID und temporären
  Programm-/App-Datenpfaden; lokale Installation, HKCU-Registrierung, echter Persistenzlauf und
  stille Deinstallation sind automatisiert geprüft.
- GPL-3.0-or-later, Beitrags-, Verhaltens- und Sicherheitsdokumentation vorhanden.

### Domäne und IPC-Grenze (KQ-002)

- Kanonische UUID-Typen für Projekte und Entitäten in Rust und TypeScript.
- Portable Projektformat-Version `1` in den Domänenmodulen; interner SQLite-Migrationsstand `5`.
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

### Editor, Szenenbaum und Autosave (KQ-005/KQ-019)

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
- Kapitel und Szenen lassen sich inline umbenennen und per zugänglichen Hoch-/Runter-Aktionen unter
  ihren Geschwistern umordnen. Titel-Compare-and-swap und vollständige alte/neue Reihenfolgen
  verhindern stilles Überschreiben eines neueren Baums.
- Zweistufiges Löschen bindet Titel und Inhaltsrevision, bei Kapiteln zusätzlich die vollständig
  angezeigte Kindliste. SQLite kaskadiert Szenen sowie Story-, Plotboard-, Timeline-, Medien- und
  Promptreferenzen; die UI entfernt Auswahl und lokale Baumknoten konsistent.
- Automatisierte Tests für Editorvertrag, Baumordnung, Autosave, IPC-Roundtrip, Konflikte,
  Löschkaskaden sowie Backup/Recovery mit 100 Szenen. Tastatur- und Screenreader-Fälle sind als
  manueller Testplan dokumentiert, aber noch nicht als E2E-Test ausgeführt.
- Ein Playwright-Ablauf prüft Projektanlage, Schreiben, entprelltes Autosave, Wiederöffnen,
  Kapitel-/Szenenerstellung, Umbenennen, zugängliches Umordnen und zweistufiges Löschen in Chromium.
  Dabei wurde ein StrictMode-Lebenszyklusfehler behoben, der den Autosave-Controller im
  Entwicklungsmodus nach dem probeweisen Effekt-Cleanup dauerhaft stilllegte.
- KQ-024 korrigiert zusätzlich die geladene Szenenrevision nach erfolgreichem Autosave. Ohne diese
  Synchronisation wurde der Editor nach einem Ansichtswechsel mit einer alten Revision neu montiert
  und der nächste Schreibversuch korrekt als Konflikt abgelehnt.
- KQ-029 baut eine eigene Windows-Smoke-Konfiguration ohne Bundle, startet die echte Tauri-Anwendung
  über `tauri-driver`, legt ein künstliches Projekt an und bestätigt den per Autosave gespeicherten
  Szenentext nach einem vollständigen Neustart. Testdaten und WebView-Profile liegen ausschließlich
  unter `$TEMP/kraqel-tauri-smoke` und werden nach dem Lauf entfernt.

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
  Rückverweise und Backup/Recovery ab. IPC- und TypeScript-Tests prüfen den vertikalen Vertrag. Ein
  Playwright-Ablauf prüft zusätzlich Erstellung, Eigenschaftssuche, gerichtete Beziehungen und
  Szenen-Rücksprung in Chromium. Dabei wurden die Listenaktualisierung nach dem Anlegen und der
  asynchrone Zugriff auf React-Ereignisse in Eigenschaftsfeldern korrigiert. Tauri-/Screenreader-
  Abnahme bleibt als eigener Testfall offen.

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
- Zwei getrennte Playwright-Abläufe prüfen zusätzlich Handlungsstrang, Status, POV,
  tastaturbedienbare Karten- und Ereignisreihenfolge, relative und absolute Zeitmodelle,
  Szenen-/Entitätslinks sowie den Rücksprung ins Manuskript. Der Testadapter bildet hierfür nur die
  bereits vorhandenen Command-Verträge ab; Tauri- und Screenreader-Abnahme bleiben separat offen.

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
- KQ-025 ergänzt einen Playwright-Ablauf, der ein künstliches Manuskript speichert und anschließend
  Formatwahl, Dateiname und lokalisierte positive Bytezahl für Markdown, HTML, DOCX und PDF gegen
  den testexklusiven Command-Adapter prüft. Echte Dateien, Rendering, Tauri, Dateisystem und
  Screenreader bleiben getrennt abzunehmen.
- Nicht Teil dieses Inkrements: Import, DOCX-Roundtrip, Medien, Fußnoten, Tabellen, Links und
  beliebige Tiptap-Erweiterungen.

### Lokale Text-KI mit Ollama (KQ-010)

- Providerunabhängige asynchrone Rust-Grenze für Beschreibung, Modellabfrage und geordnete
  Streamereignisse; der erste Adapter spricht ausschließlich einen lokalen Ollama-Dienst an.
- Die feste Loopback-Adresse, deaktivierte Systemproxies und Redirects verhindern, dass die lokale
  Konfiguration versehentlich zu einem Cloudzugriff wird. Anfrage, einzelne JSONL-Zeilen und
  Gesamtausgabe sind begrenzt und Antworten werden strikt validiert.
- Vor jedem Lauf zeigt die Oberfläche den gespeicherten Szenenklartext. Die Freigabe erfolgt per
  Checkbox für genau Dokument-ID und Revision; Rust lädt den Text erneut und lehnt einen inzwischen
  geänderten Stand als Konflikt ab.
- Modellliste und Textentwurf laufen über typisierte Tauri-Commands beziehungsweise einen Kanal. Der
  Stream trägt Provenienz, bleibt verwerfbarer Entwurf und verändert oder speichert niemals
  automatisch den Szenentext.
- Automatisierte Tests prüfen Modellreihenfolge, JSON-Vertrag, geordnetes Streaming, den tatsächlich
  übertragenen Kontext sowie die Sperre ohne Freigabe und für Nicht-Loopback-Adressen.
- In der Entwicklungsumgebung ist kein laufender Ollama-Dienst vorhanden. Ein echter Modelllauf und
  die manuelle Browser-/Screenreader-Abnahme sind daher noch nicht nachgewiesen. Cloudprovider und
  Schlüsselspeicher gehören ausdrücklich nicht zu KQ-010.

### Cloud-Textprovider und Schlüsselschutz (KQ-011)

- Rust-Adapter für OpenAI Responses, Anthropic Messages, Google Gemini und Mistral Chat Completions
  bilden deren Modelllisten und gestreamte Textdeltas auf denselben providerunabhängigen Vertrag ab.
- Produktionsendpunkte sind feste offizielle HTTPS-Adressen. Redirects, Umgebungsproxies und frei
  konfigurierbare Produktionsziele sind deaktiviert; lokale HTTP-Endpunkte existieren nur in Tests.
- API-Schlüssel werden je Provider im nativen Betriebssystem-Schlüsselspeicher abgelegt. Die UI kann
  Werte setzen oder löschen und nur deren Vorhandensein prüfen; gespeicherte Werte werden niemals
  zurückgegeben oder in Projektdateien geschrieben.
- Jede Vorschau zeigt Provider, Modell, Arbeitsauftrag, Szenentext und übertragene Metadaten. Ein
  kurzlebiger Einmalbeleg bindet diese Angaben an Dokumentrevision und Projekt und wird vor dem
  Request verbraucht. Änderungen oder Wiederverwendung werden in Rust abgelehnt.
- Modelllisten werden nur nach ausdrücklicher Bedienhandlung abgerufen. Streams bleiben
  größenbegrenzt, erhalten eine Herkunftsangabe und verändern den kanonischen Text nicht.
- Protokolltests prüfen alle vier Antwortformate, Modelllisten und die Sperre ohne Freigabe mit
  lokalen Testservern. Mangels realer API-Schlüssel wurden keine kostenpflichtigen oder echten
  Cloudrequests ausgeführt; Anbieterkompatibilität und Betriebssystemdialoge benötigen manuelle
  Abnahme.

### Deterministische Kontinuitätsprüfung und Vorschlagsreview (KQ-012)

- `kraqel-continuity` wertet eine unveränderliche Faktenprojektion aus Dokumenten, Story-Bibel,
  Plotboard und Timelines vollständig offline und ohne Provider aus.
- Vier konservative Regeln erkennen doppelte normalisierte Story-Bibel-Namen, ungültige POV-Arten,
  exakte nicht verknüpfte Namensnennungen und Mehrfachzuordnungen einer Szene auf derselben
  Timeline. Nur der strukturell ungültige POV wird als Konflikt bezeichnet; mehrdeutige, aber
  möglicherweise legitime Modellierungen bleiben Prüfhinweise oder Informationen.
- Jeder Befund enthält eine stabile Regelkennung, einen Schweregrad und konkrete Quellenanker.
  Dokumentfundstellen verwenden Unicode-Zeichenpositionen; Szenenquellen lassen sich aus der
  Reviewansicht im Manuskript öffnen.
- Die React-Ansicht **Kontinuität** löst die lokale Prüfung ausdrücklich aus und gruppiert die
  reproduzierbaren Befunde. Sie verändert keine Projektdaten.
- Ein erzeugter Textentwurf kann gegen genau die zuvor freigegebene Szenenrevision als Zeilendiff
  geprüft werden. Rust lädt den Szenenstand erneut und lehnt veraltete Revisionen ab. Der Command
  besitzt keinen Schreibpfad, die Oberfläche keine automatische Übernahme.
- Unit-, Repository-, IPC- und TypeScript-Vertragstests prüfen Regelklassifikation, Quellenanker,
  Read-only-Verhalten und Revisionskonflikte. KI-basierte Kontinuitätshinweise, persistente
  Prüfläufe und ein interaktiver Teilübernahme-Editor sind nicht implementiert.
- KQ-028 ergänzt einen eigenständigen Playwright-Ablauf: Eine künstliche, nicht verknüpfte
  Namensnennung erzeugt nach ausdrücklichem Start einen Informationsbefund mit stabiler Regelkennung
  und Unicode-Zeichenanker; dessen Schaltfläche lädt die betreffende Szene im Manuskript. Der
  Browser-Adapter bildet dafür nur die bestehende Regel `story-bible.unlinked-mention` ab. Tauri,
  Rust, KI-Hinweise und schreibende Änderungen sind nicht Teil dieses Browsertests.

### Lokale Medienbibliothek (KQ-013)

- SQLite-Migration 2→3 ergänzt `media_asset`, `asset_link`, Indizes und Löschtrigger. Assets
  speichern relative Original-/Thumbnailpfade, MIME-Typ, SHA-256, Bytegröße, Abmessungen, Herkunft,
  Titel und Schlagwörter.
- Die Rust-Grenze akzeptiert ausschließlich inhaltlich dekodierbare PNG-, JPEG- und WebP-Dateien bis
  25 MiB, 16.384 Pixel je Achse und 40 Millionen Pixel. Signatur und Endung müssen übereinstimmen.
- Projektdateinamen bestehen aus Asset-UUID und Hashpräfix. Gelieferte Namen werden nicht als Pfade
  verwendet. Thumbnails entstehen lokal als begrenzte PNG-Dateien.
- Die Ansicht **Medien** unterstützt Import, Vorschau, Metadaten und Tags, inhaltsgeprüften Ersatz,
  zweistufig bestätigtes Löschen sowie Links zu Szenen, Story-Bibel-Einträgen und Ereignissen.
  Fehlende oder hash-veränderte Originale beziehungsweise fehlende Thumbnails werden sichtbar als
  nicht verfügbar gemeldet.
- Links verwenden stabile IDs und bleiben bei Umbenennungen erhalten. Rust prüft jedes Linkziel;
  Trigger entfernen Links automatisch, wenn das Zielobjekt gelöscht wird.
- Lokale SQLite-Backups erhalten ein `.assets`-Begleitverzeichnis. Die Datenbanksicherung bestimmt
  die kopierten Dateien; Originale werden vor Veröffentlichung und Wiederherstellung gegen SHA-256
  geprüft.
- Portable Snapshots kopieren Originale als `assets/<sha256>.<endung>`, führen Metadaten und Links
  im Manifest und prüfen Hash sowie exakte Dateimenge. Byte-Stabilität bleibt erhalten.
- Tests decken Inhaltsprüfung, Pfadsicherheit, Hash/Thumbnail, Migration 2→3, Linkkaskaden,
  Backup/Recovery, manipulierte Snapshotassets sowie den vertikalen Importvertrag ab.
- KQ-026 ergänzt einen Playwright-Ablauf für künstliche PNG-Auswahl, Titel und Tags, sichtbare
  Vorschau-, MIME-, Maß- und Prüfsummendaten, Verknüpfung mit der aktiven Szene und Rücksprung ins
  Manuskript. Der testexklusive Adapter dekodiert keine Bilder, berechnet keine kryptografischen
  Hashes und schreibt keine Dateien; diese Eigenschaften bleiben durch Rust-Tests beziehungsweise
  separate Tauri- und Plattformabnahme belegt.

### Visuelle Profile und Stilvorlagen (KQ-014)

- SQLite-Migration 3→4 ergänzt genau ein `visual_profile` je Story-Bibel-Eintrag sowie projektweite
  `style_preset`-Datensätze.
- Profile speichern begrenzte, strukturierte Kernmerkmale, explizite Ausschlüsse und optional genau
  ein kanonisches Medienasset aus demselben Projekt. Löschen des Assets leert nur die Referenz;
  Löschen der Entität entfernt das Profil.
- Stilvorlagen besitzen stabile UUIDs, einen projektweit eindeutigen Namen, strukturierte visuelle
  Regeln und mindestens eine der vier unterstützten Entitätsarten als Geltungsbereich.
- Rust normalisiert und begrenzt Namen, Schlüssel, Werte, Listen und JSON-Gesamtgrößen. Sämtliche
  Entitäts-, Projekt- und Assetreferenzen werden an der autoritativen Grenze geprüft.
- Die Ansicht **Visuelle Profile** unterstützt Erstellen, Bearbeiten und zweistufig bestätigtes
  Löschen von Profilen und Stilvorlagen; kanonische Bilder werden aus der lokalen Medienbibliothek
  gewählt.
- Backup/Recovery übernimmt die Daten mit der SQLite-Sicherung. Deterministische Snapshots führen
  sie in `visual-profiles.json` und prüfen Struktur sowie Entitäts- und Assetreferenzen.
- Repository-, Migrations-, Backup-, Snapshot-, IPC- und TypeScript-Vertragstests decken den
  vertikalen Ablauf ab.
- KQ-027 ergänzt einen eigenständigen Playwright-Ablauf für Profilmerkmale, Ausschlüsse,
  strukturierte Stilregeln, mehrere Geltungsbereiche und das Wiederladen nach Ansichtswechsel gegen
  den vorhandenen testexklusiven Command-Adapter. Kanonische Referenzbilder, Tauri und Screenreader
  bleiben separat abzunehmen.
- Es existiert kein Netzwerkpfad für Profile oder Stilvorlagen. Bildprovider sind ausdrücklich nicht
  Bestandteil dieses Inkrements.

### Lokaler, editierbarer Bildprompt-Builder (KQ-015)

- Die Ansicht **Bildprompts** verlangt die explizite Auswahl einer Szene, mindestens eines visuellen
  Profils und optional einer für alle gewählten Entitätsarten passenden Stilvorlage.
- Rust lädt Szenentext, Story-Bibel-Eigenschaften, Kernmerkmale, Ausschlüsse, kanonische Asset-UUIDs
  und Stilregeln erneut und setzt sie deterministisch zu einem editierbaren Text zusammen.
  Szenentext ist als unvertrauenswürdiger Inhalt gekennzeichnet.
- SQLite-Migration 4→5 speichert Name, bearbeiteten Prompttext, Szenen-UUID und Quellrevision,
  Profil-UUIDs sowie die optionale Stilvorlage. Die erste Speicherung lehnt eine zwischenzeitlich
  geänderte Szene als Konflikt ab; spätere Textbearbeitung ändert Quellen nicht still.
- Erstellen, Listen, Bearbeiten, zweistufiges Löschen sowie UTF-8/LF-Export unter
  `exports/prompts/bildprompt-<uuid>.txt` sind lokal implementiert. Es gibt keinen Provider-,
  Netzwerk- oder automatischen Übernahmepfad.
- Backup/Recovery erhält Entwürfe. Snapshot v3 exportiert `prompt-drafts.json`, prüft Dokument-,
  Profil- und Stilreferenzen und liest weiterhin v1 und v2.
- Repository-, Migration-, Backup-, Snapshot-, Export-, Rust-IPC- und TypeScript-Commandtests sind
  grün. Ein Playwright-Ablauf legt künstliche Story-Bibel-, Profil- und Stilquellen über die UI an
  und prüft deterministische Zusammensetzung, vollständige Bearbeitung, lokale Speicherung,
  Wiederladen sowie die sichtbare UTF-8/LF-Exportbestätigung. Tauri-, Dateisystem-, WebView- und
  Screenreader-Abnahme bleiben separat offen.

### Begrenzte Hilfsfunktionen und Verträge

- `asset-service`: produktiver, begrenzter Rasterbildimport mit SHA-256, Thumbnail, sicherer
  Veröffentlichung und erneuter Dateiprüfung.
- `editor`: versionierter, generischer Snapshot-Vertrag.
- `ai-contracts`: Capability-, Provider-, Streaming- und explizite Kontextfreigabetypen für Text
  sowie gespiegelte Bild-Registry- und Generation-Job-Verträge.
- `visual-contracts`: Medien-, Profil-, Stilvorlagen- und Promptverträge; keine Providerlogik.
- `continuity`: ausführbare Offline-Regeln und Vorschlags-Diff; keine KI-Analyse oder
  Laufpersistenz.

### Bild-Capability-Registry und Generation-Job-Verträge (KQ-016)

- Die lokale Registry ordnet `image.generate`, `image.edit`, `image.reference` und
  `image.map_render` einzelnen Modellen zu, statt Fähigkeiten pauschal für einen Provider
  anzunehmen.
- Modelleinträge beschreiben Prompt- und Referenzlimits, unterstützte Breiten und Höhen,
  Ergebniszahl und Seed-Unterstützung. Leere Installationen und verständlich begründete inaktive
  Provider sind zulässig; doppelte oder widersprüchliche Einträge werden abgelehnt.
- Der versionierte Vertrag v1 bindet Job, Projekt, Provider, Modell, Fähigkeit, Prompt-Entwurf,
  exakt freigegebenen Prompttext, Referenzasset-UUIDs und Einstellungen.
- Die Zustandsmaschine erlaubt nur `queued -> running -> succeeded|failed|cancelled` sowie Fehler
  oder Abbruch vor dem Start. Endzustände sind endgültig; Herkunft und bereinigte Fehler dürfen nur
  zum passenden Endzustand gehören.
- Rust prüft Registry, Modellfähigkeit, Grenzen und explizite Kontextfreigabe vor einem möglichen
  Provideraufruf. Statische Fehler enthalten keine Eingaben, Pfade oder fremden Diagnosedaten.
- Ein rein lokaler, nur in Tests kompilierter Mock prüft den vollständigen Jobablauf ohne Netzwerk,
  Bilddatei oder Assetübernahme. Es gibt keinen produktiven Bildadapter, Tauri-Command, SQLite-Job
  oder Snapshotformat für Läufe.

### Deterministische Snapshots und lokales Git (KQ-008)

- Portables Snapshotformat v3 für Projektmetadaten, Dokumente, Story-Bibel, Plotboard, Timelines,
  visuelle Profile und Prompt-Entwürfe; v1/v2 bleiben lesbar und ein formales JSON-Schema wird mit
  jedem Snapshot kopiert.
- UUID-basierte Markdown-Dateinamen, stabile fachliche Sortierung, UTF-8/LF und keine flüchtigen
  Datenbankzeitstempel. Wiederholte Erzeugung unveränderter Projekte ist byte-identisch getestet.
- Freigegebene Rasterbilder werden inhaltsadressiert kopiert; Manifest, Hash und tatsächliche
  Assetdateien werden gemeinsam validiert.
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
  Git-Roundtrip. Ein Playwright-Ablauf prüft zusätzlich zwei explizite Snapshotstände samt lesender
  Validierung, Änderungsliste, Unified Diff, Commitbeleg und anschließend sauberem Status gegen den
  testexklusiven In-Memory-Adapter. Echtes Git, Dateisystem, Tauri, WebView und Screenreader bleiben
  getrennt abzunehmen.
- Die Prüfung ist noch keine schreibende Wiederherstellung: Ein Snapshot wird derzeit nicht in ein
  neues SQLite-Projekt importiert.

## Offene Aufgaben

### Unmittelbar: 0.1 stabilisieren

1. **Tastatur- und Screenreader-Abnahme mit dokumentierter Testmatrix durchführen**
2. **Releaseverteilung getrennt vorbereiten**
   - Codesignierung, SmartScreen, WebView2-Verteilung und späteren Updatepfad separat entscheiden

### Bewusst zurückgestellt

- Der isolierte lokale NSIS-Lebenszyklus ist durch KQ-030 belegt. Eine zusätzliche manuelle Prüfung
  auf einem weiteren Windows-Rechner wird zu einem späteren Zeitpunkt nachgeholt. Sie ist weiterhin
  offene Release-Evidenz, aber keine unmittelbar blockierende Entwicklungsaufgabe.

### Danach gemäß Pflichtenheft

- KI-basierte Kontinuitätshinweise bleiben eine spätere Erweiterung von KQ-012.
- KQ-017 und KQ-018 gehören zu Version 0.2 und sind kein Ziel für 0.1.

Für KQ-017 und KQ-018 sind noch keine GitHub-Issues angelegt; die Aufgaben stehen im Pflichtenheft.
Vor 0.2 müssen Akzeptanzkriterien, Abhängigkeiten und Rechtsfragen konkretisiert werden.

## Bekannte Fehler und Lücken

- **Löschen ist endgültig:** Dokumente und ihre abhängigen Links werden erst nach zweistufiger
  Bestätigung und Konfliktprüfung entfernt, besitzen aber noch keinen Papierkorb oder Undo über die
  laufende Sitzung hinaus. Backups und Git-Snapshots bleiben der Wiederherstellungsweg.
- **Feste Projektwurzel in der Desktopoberfläche:** Projekte liegen derzeit unter dem lokalen
  App-Datenverzeichnis. Eine frei wählbare Projektwurzel ist noch nicht umgesetzt.
- **Backup bleibt zweiteilig:** Eine Sicherung besteht aus `<name>.sqlite3` und `<name>.assets/`.
  Beide Teile werden gemeinsam erzeugt und für medienhaltige Wiederherstellungen benötigt; ein
  einzelnes gepacktes Austauscharchiv ist noch nicht definiert.
- **Snapshot-Wiederherstellung nur vorbereitet:** Format und Inhalte werden lesend geprüft, aber
  noch nicht in eine neue Datenbank importiert. F07 ist deshalb noch nicht vollständig abgenommen.
- **Export ist kein Import:** Die vier Ausgaben sind erzeugbar; ein Roundtrip zurück nach KRAQEL und
  der im Pflichtenheft genannte Import sind nicht implementiert. F08 ist deshalb nur teilweise
  erfüllt.
- **Exportformatierung bewusst begrenzt:** Links, Code, Tabellen, Fußnoten und Medien werden nicht
  still reduziert, sondern derzeit abgelehnt. PDF verwendet eingebaute Standardschriften; vor einer
  breiteren Unicode-Abnahme ist eine eingebettete freie Schrift zu prüfen.
- **Medienumfang bewusst begrenzt:** SVG, Animationen, Audio, Video, Lizenzfelder und eingebettete
  Exportmedien fehlen. Der WebView übergibt ausgewählte Dateien als Bytearray; bei vielen großen
  Dateien ist später ein nativer Streamingimport sinnvoll. Fehlgeschlagene Dateibereinigung kann
  unreferenzierte Dateien hinterlassen, aber keine intakte Referenz vorzeitig löschen.
- **Migrationen bleiben vorwärtsgerichtet:** 0→1 sowie befüllte Migrationen 1→2, 2→3, 3→4 und 4→5
  sind getestet; weitere Versionen benötigen jeweils eine neue unveränderliche Migration.
- **Zeitwerte ohne Kalendersemantik:** Absolute Zeitwerte sind validierte Zeichenketten. Kalender,
  Zeitzonen und Arithmetik sind bewusst noch nicht implementiert.
- **Browser-E2E bleibt vom Tauri-Smoke-Test getrennt:** Playwright prüft breite React-Abläufe gegen
  einen testexklusiven Command-Adapter. KQ-029 belegt nur einen kleinen echten Windows-Pfad durch
  WebView, Tauri-IPC, Rust und SQLite. KQ-030 ergänzt dafür den isolierten lokalen
  NSIS-Installationslebenszyklus. Native Dialoge, weitere Fachansichten, Clean-VM, Release-Packaging
  und Screenreader bleiben separat abzunehmen.
- **Prompt-Builder-Browser-E2E ist keine Dateisystemabnahme:** Der Chromium-Ablauf prüft
  Zusammensetzung, Bearbeitung, Persistenzvertrag und sichtbare Exportbestätigung gegen den
  testexklusiven Adapter. Die echte UTF-8/LF-Datei wird in Rust getestet; Tauri, WebView,
  Screenreader und Betriebssystemdateisystem bleiben separat abzunehmen. Entwürfe sind bewusst nicht
  automatisch mit Bildprovidern verbunden.
- **Export-Browser-E2E ist keine Renderer- oder Dateisystemabnahme:** Der Chromium-Ablauf prüft die
  vier Formataktionen sowie die sichtbare Dateinamen- und Größenbestätigung gegen den testexklusiven
  Adapter. Er erzeugt keine Dateien und prüft weder deren Inhalt noch Tauri,
  Betriebssystemdateisystem, Layout oder Screenreader.
- **Medien-Browser-E2E ist keine Inhalts- oder Dateisystemabnahme:** Der Chromium-Ablauf prüft
  Dateiauswahl, sichtbare Vertragsdaten und Szenenverknüpfung. Der testexklusive Adapter führt keine
  Bilddekodierung, kryptografische Hashprüfung, Thumbnailerzeugung oder Dateioperation aus; Tauri,
  Betriebssystemdateisystem und Screenreader bleiben separat abzunehmen.
- **Bildverträge ohne Laufzeitfunktion:** Registry, Jobzustände, Freigabe und Fehler sind getestet,
  aber es gibt weder produktive Provider noch UI, Persistenz, Abbruch, Bildausgabe oder
  Assetübernahme. Das ist der beabsichtigte 0.1-Umfang, keine bereits fertige Bildgenerierung.
- **Ollama nur gegen Testserver geprüft:** Kein Ollama-Dienst und kein lokales Modell waren in der
  Entwicklungsumgebung verfügbar. Abbruch, Laufpersistenz und eine manuelle Übernahme des Entwurfs
  in den Editor sind noch nicht implementiert.
- **Cloudprovider nur gegen Testserver geprüft:** Die vier Protokolle und der Schlüsselspeicher sind
  implementiert, aber ohne reale Schlüssel weder gegen Anbieterendpunkte noch hinsichtlich Kosten,
  Kontodialogen und serverseitiger Aufbewahrung abgenommen.
- **Kontinuitätsregeln bewusst konservativ:** Namensnennungen werden nur exakt erkannt; Aliasse,
  Pronomen und semantische Widersprüche bleiben unerkannt. Mehrfachpositionen auf einer Timeline
  sind Prüfhinweise, keine behaupteten Fehler. Befunde werden bei jedem Lauf neu berechnet und noch
  nicht als Prüfhistorie gespeichert.
- **Entitätssuche nicht indexiert:** Die Unicode-korrekte Suche filtert derzeit alle Einträge eines
  Projekts in Rust. Vor sehr großen Story-Bibeln sind Messung und gegebenenfalls eine echte
  FTS-Migration nötig.
- **Dependency Review eingeschränkt:** Der GitHub-Job wird bei privaten Repositories ohne GitHub
  Advanced Security bewusst übersprungen. Lockfiles und reguläre CI bleiben aktiv.
- Die vollständige lokale KQ-030-Qualitätskette ist grün: 40 TypeScript-, 91 ausgeführte Rust- und
  zehn Playwright-E2E-Tests, Formatierung, ESLint, Clippy und Typprüfung. Zusätzlich sind der
  isolierte Tauri-Debug-Build, der echte Windows-Persistenz-Smoke-Test sowie Build, Installation und
  Deinstallation des unsignierten NSIS-Testartefakts grün. Ein visueller Export-QA-Test bleibt
  planmäßig ignoriert. Die CI-Matrix unter Windows und Linux ist verbindliche Merge-Voraussetzung;
  die nativen Windows-Smoke-Tests laufen derzeit bewusst lokal.

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
  überschreiben nichts. Metadaten- und Baumänderungen verwenden eigene Compare-and-swap-Grenzen;
  Löschen bindet den vollständig angezeigten Stand (ADR 0004).
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
- **Lokales Ollama hinter einer revisionsgebundenen Freigabe:** React erhält keinen direkten
  Providerzugriff. Rust lädt den freigegebenen Szenenstand erneut, spricht ausschließlich Loopback
  an und streamt einen verwerfbaren Entwurf ohne automatische Speicherung (ADR 0009).
- **Cloudprovider mit fester Ziel- und Einmalfreigabe:** Schlüssel liegen im nativen
  Betriebssystemspeicher; feste HTTPS-Ziele und ein kurzlebiger, verbrauchbarer Freigabebeleg binden
  Provider, Modell, Arbeitsauftrag und Dokumentrevision (ADR 0010).
- **Deterministische Kontinuität und Nur-Lese-Review:** Regeln arbeiten auf einer lokalen
  Faktenprojektion und formulieren Mehrdeutigkeiten nicht als Gewissheiten. Vorschlags-Diffs sind an
  die Szenenrevision gebunden und besitzen keinen Schreibpfad (ADR 0011).
- **Inhaltsgeprüfte lokale Medien:** Rust validiert Formate und Grenzen, Projektdateinamen entstehen
  aus UUID/Hash, Ersetzen schaltet erst nach neuer Dateiveröffentlichung um, und Backup/Snapshot
  prüfen referenzierte Originale gegen SHA-256 (ADR 0012).
- **Projektgebundene visuelle Vorgaben:** Profile referenzieren Entitäten und optionale kanonische
  Assets über stabile UUIDs; Stilvorlagen bleiben strukturiert, begrenzt und vollständig lokal (ADR
  0013).
- **Explizite lokale Promptquellen:** Bildprompt-Entwürfe binden Szene, Quellrevision, Profile und
  optionale Stilvorlage; Zusammensetzung, Speicherung und Textausgabe bleiben ohne Providerpfad (ADR
  0014).
- **Modellspezifische Bildfähigkeiten:** Eine offline validierte Registry und ein geschlossener
  Jobvertrag trennen Modellfähigkeiten, Freigabe, Herkunft und bereinigte Fehler von jeder späteren
  Ausführung. KQ-016 enthält nur einen Test-Mock (ADR 0015).
- **Isolierte NSIS-Testvariante:** Produktname, Bundle-ID, App-Daten und Installationsziel sind vom
  regulären Produkt getrennt; nur der unsignierte Current-User-Debug-Installer wird lokal geprüft.
  Clean-VM, MSI und Release-Signierung bleiben eigene Nachweise (ADR 0016).
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

| Thema                        | Zustand / Risiko                                                                                                        | Nächste Kontrolle                                                     |
| ---------------------------- | ----------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------- |
| SQLite und FTS5              | Dokument-FTS sowie Story-Bibel-CRUD, Relationen, Links und Recovery getestet; Entitätssuche nicht indexiert             | Vor großen Projekten messen; Migration nur mit Evidenz                |
| Rust-/TS-Vertragsduplikation | Manuell gespiegelt; Drift ist möglich                                                                                   | Gemeinsame Fixtures oder spätere Codegenerierung bewerten             |
| Datenformat                  | Snapshot v3 mit Prompt-Entwürfen, Profilen und Hash-Assets; Import und Roundtrip fehlen                                 | Import/Roundtrip vor vollständiger F07/F08-Abnahme testen             |
| Exportrenderer               | DOCX/PDF gerendert geprüft; PDF-Standardschrift begrenzt Unicode, Medien fehlen                                         | Freie eingebettete Schrift und Medienkonzept prüfen                   |
| Dateisystem                  | Komponentenprüfung aktiv; TOCTOU gegen böswillige Paralleländerung bleibt                                               | Vor nicht vertrauenswürdigen Importen Handle-APIs prüfen              |
| Editorformat                 | Tiptap-Vertrag v1; spätere inkompatible Änderungen brauchen neue Version                                                | Bei jeder Erweiterung Fixtures und Migration prüfen                   |
| Desktoptests                 | Zehn Browser-E2E, echter Windows-Tauri-M1-Smoke und lokaler NSIS-Lebenszyklus; keine breite Tauri-/Screenreader-Abnahme | Screenreader als nächste aktive Prüfung                               |
| Windows-Installer            | Unsignierter NSIS-Debug-Test lokal grün; Zweitrechner, MSI, Signierung, SmartScreen und Updatepfad nicht abgenommen     | Zweitrechner später manuell; Releaseverteilung separat entscheiden    |
| CI-Laufzeit                  | Kalte Tauri-/Rust-Builds unter Windows dauern mehrere Minuten                                                           | Caching prüfen, ohne Reproduzierbarkeit zu schwächen                  |
| Abhängigkeitssicherheit      | GitHub Dependency Review privat nicht verfügbar                                                                         | Alternative Audits oder Advanced Security später bewerten             |
| Plattformen                  | Windows ist primär; Linux baut in CI, ist fachlich aber nicht abgenommen                                                | Plattformmatrix je Release definieren                                 |
| Lokale Text-KI               | Protokoll gegen kontrollierten Loopback-Testserver geprüft; echter Ollama-/Modelllauf fehlt                             | Mit freiem Testmodell manuell prüfen und Abbruch entwerfen            |
| Cloudprovider                | Vier Protokolle lokal getestet; echte API-, Kosten-, Aufbewahrungs- und Kontodialog-Abnahme fehlt                       | Mit separaten Testkonten und Minimalbudget manuell prüfen             |
| Schlüsselspeicher            | Native Backends eingebunden; reale Windows-/Linux-Dialoge nicht automatisiert getestet                                  | Manuelle Plattformabnahme ohne produktive Schlüssel                   |
| Kontinuitätsprüfung          | Vier Offline-Regeln getestet; exakte Nennungen und grober Zeilendiff begrenzen die Abdeckung                            | Mit größeren künstlichen Projekten messen; KI-Kanal separat entwerfen |
| Medienimport                 | Drei Rasterformate und harte Limits getestet; Bytearray-IPC skaliert nicht für Massenuploads                            | Nativen Streamingimport erst nach Messung großer Sammlungen bewerten  |
| Assetbackup                  | Hash-verifizierte Begleitverzeichnisse getestet; noch kein einzelnes portables Paket                                    | Paketformat und atomare Mehrdatei-Verteilung vor externem Austausch   |
| Visuelle Profile             | Struktur und Referenzen getestet; Stilnamen-Eindeutigkeit nutzt SQLite-NOCASE mit ASCII-Grenze                          | Unicode-Kollation bei internationalisierten Vorlagennamen bewerten    |
| Bildprompt-Entwürfe          | Lokal, referenziell und browserseitig geprüft; kein Providerpfad und keine automatische Rekombination                   | Tauri-/Screenreader-Abnahme; KQ-016-Grenze getrennt halten            |
| Bild-Capability-Verträge     | Rust/TypeScript v1 und lokaler Mock getestet; keine produktive Ausführung oder Persistenz                               | Vor KQ-017 Freigabebeleg, Ergebnisprüfung, Kosten und Lizenzen klären |

## Nächste geplante Entwicklungsstufe

**0.1-Stabilisierung: Tastatur- und Screenreader-Abnahme durchführen.**

Empfohlener, überprüfbarer Scope:

1. Tastatur- und Screenreader-Matrix mit Windows-Sprachausgabe oder NVDA durchführen.
2. Befunde nach reproduzierbaren Codefehlern, Dokumentationslücken und manueller Restabnahme
   trennen.
3. Codesignierung, SmartScreen, MSI-Bedarf, WebView2-Strategie und Updatepfad vor einem
   veröffentlichbaren Releasepaket entscheiden.

Die zusätzliche Installerprüfung auf einem weiteren Windows-Rechner ist bewusst zurückgestellt und
wird später manuell nachgeholt.

## Einstieg für die nächste Codex-Session

```powershell
cd D:\kraqel
git status
git branch --show-current
git log -5 --oneline
rg "Installer|NSIS|MSI|Signierung|Screenreader" docs ROADMAP.md apps/desktop/src-tauri
```

Danach lesen: `AGENTS.md`, dieses Dokument, `docs/testing/windows-installer-smoke.md`,
`docs/testing/version-control-accessibility.md`, ADR 0016 und die Pflichtenheft-Abschnitte M5
und 10. KQ-030 belegt den lokalen isolierten NSIS-Lebenszyklus; die Zweitrechnerprüfung ist bewusst
zurückgestellt. Als Nächstes ein fachlich geschlossenes Issue für die Tastatur- und
Screenreader-Abnahme anlegen.
