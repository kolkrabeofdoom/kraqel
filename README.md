# KRAQEL

<p align="center">
  <img src="docs/assets/kraqel-kolkrabe-schreibstudio.png" alt="Ein Kolkrabe mit Baskenmütze schreibt in einem üppigen, nächtlichen Schreibstudio; daneben stehen Storyboard, Timeline und ein Laptop mit einer stilisierten KRAQEL-Oberfläche." width="100%">
</p>

<p align="center">
  <em>Write. Plan. Draft. Revise. Rebel. – Das Titelmotiv ist eine Illustration; die dargestellte Oberfläche ist stilisiert.</em>
</p>

<p align="center">
  <a href="https://github.com/kolkrabeofdoom/kraqel/actions/workflows/ci.yml"><img src="https://github.com/kolkrabeofdoom/kraqel/actions/workflows/ci.yml/badge.svg" alt="CI-Status"></a>
  <a href="https://github.com/kolkrabeofdoom/kraqel/releases/tag/v0.1.0-alpha.4"><img src="https://img.shields.io/badge/Release-0.1.0--alpha.4-c1121f" alt="Aktuelles Release: 0.1.0 Alpha 4"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/Lizenz-GPL--3.0--or--later-663399" alt="Lizenz: GPL 3.0 oder später"></a>
  <img src="https://img.shields.io/badge/Plattform-Windows%20x64-0f6cbd" alt="Primärplattform: Windows x64">
  <img src="https://img.shields.io/badge/Prinzip-Offline%20first-287271" alt="Prinzip: Offline first">
</p>

**KRAQEL's Recursive Authoring, Queries, Editions & Lore** ist ein quelloffenes, offlinefähiges
Autorenstudio für Romane, Drehbücher und Rollenspielmodule. Es verbindet konzentriertes Schreiben,
Strukturplanung, Worldbuilding, Medienverwaltung, lokale Versionierung und optionale KI-Werkzeuge in
einer Desktop-Anwendung. Schreiben steht zuerst; KI bleibt optional, transparent und unter
menschlicher Kontrolle.

> [!WARNING] **Aktueller Stand: 0.1.0 Alpha 4.** KRAQEL ist ein frühes, öffentliches Prerelease und
> noch nicht produktionsreif. Arbeite ausschließlich mit aktuellen Sicherungen und niemals mit dem
> einzigen Exemplar eines Manuskripts. Der Drache kennt inzwischen Undo, aber die Brandschutzordnung
> nur vom Hörensagen.

## Inhaltsverzeichnis

- [Alpha 4 herunterladen](#alpha-4-herunterladen)
- [Wofür KRAQEL gedacht ist](#wofür-kraqel-gedacht-ist)
- [Grundsätze](#grundsätze)
- [Funktionsumfang](#funktionsumfang)
- [Mitgeliefertes Beispielprojekt](#mitgeliefertes-beispielprojekt)
- [Was noch nicht fertig ist](#was-noch-nicht-fertig-ist)
- [Architektur](#architektur)
- [Lokale Daten, Projektformat und Sicherungen](#lokale-daten-projektformat-und-sicherungen)
- [Datenschutz, KI und Sicherheit](#datenschutz-ki-und-sicherheit)
- [Barrierearmut](#barrierearmut)
- [Aus dem Quellcode entwickeln](#aus-dem-quellcode-entwickeln)
- [Qualität und Tests](#qualität-und-tests)
- [Repositorystruktur](#repositorystruktur)
- [Projektstand und Roadmap](#projektstand-und-roadmap)
- [Mitwirken](#mitwirken)
- [Lizenz und Inhalte](#lizenz-und-inhalte)

## Alpha 4 herunterladen

### Fertiges Windows-Setup

Das aktuelle öffentliche Prerelease ist
**[KRAQEL 0.1.0 Alpha 4](https://github.com/kolkrabeofdoom/kraqel/releases/tag/v0.1.0-alpha.4)**. Es
richtet sich an testende Personen auf Windows x64 und enthält drei gemeinsam zu prüfende Dateien:

Alpha 4 ergänzt den bewusst bestätigten, atomaren Merge doppelter Story-Bibel-Einträge samt
Eigenschaften, Referenzen und visuellen Profilen. Der Ablauf wurde zusätzlich in der echten
Windows-Tauri-App manuell geprüft.

| Datei                                                                                                                                                          | Zweck                                                             |
| -------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------- |
| [`KRAQEL_0.1.0_windows-x64-setup.exe`](https://github.com/kolkrabeofdoom/kraqel/releases/download/v0.1.0-alpha.4/KRAQEL_0.1.0_windows-x64-setup.exe)           | Unsigniertes NSIS-Setup für das aktuelle Benutzer*innenkonto      |
| [`KRAQEL_0.1.0_windows-x64-SHA256SUMS.txt`](https://github.com/kolkrabeofdoom/kraqel/releases/download/v0.1.0-alpha.4/KRAQEL_0.1.0_windows-x64-SHA256SUMS.txt) | Veröffentlichte SHA-256-Prüfsumme                                 |
| [`KRAQEL_0.1.0_windows-x64-build.json`](https://github.com/kolkrabeofdoom/kraqel/releases/download/v0.1.0-alpha.4/KRAQEL_0.1.0_windows-x64-build.json)         | Maschinenlesbares Buildmanifest mit Quellcommit und Werkzeugdaten |

Das Setup ist 233.977.179 Byte beziehungsweise rund 223,14 MiB groß. Es enthält den
WebView2-Offlineinstaller, damit die Installation nicht erst zur Laufzeit eine Browserkomponente aus
dem Netz nachladen muss. Der zugehörige Quellstand ist Commit
[`24ceb3c627f94ac6e54f8d11b95a201a8e806eed`](https://github.com/kolkrabeofdoom/kraqel/commit/24ceb3c627f94ac6e54f8d11b95a201a8e806eed).

Erwartete SHA-256-Prüfsumme des Setups:

```text
0e6d578d1dfb48e8ecfe6c8be04be3f593f83eff43756f829d30ebad761fb6be
```

Unter PowerShell lässt sich der Download so prüfen:

```powershell
$expected = "0e6d578d1dfb48e8ecfe6c8be04be3f593f83eff43756f829d30ebad761fb6be"
$actual = (Get-FileHash .\KRAQEL_0.1.0_windows-x64-setup.exe -Algorithm SHA256).Hash.ToLowerInvariant()
if ($actual -ne $expected) { throw "Die Prüfsumme stimmt nicht. Setup nicht starten." }
"SHA-256 stimmt: $actual"
```

### Unsigned- und SmartScreen-Hinweis

Alpha 4 ist bewusst **nicht mit einem kommerziellen Authenticode-Zertifikat signiert**. Windows kann
deshalb eine SmartScreen-Warnung anzeigen oder die herausgebende Stelle als unbekannt bezeichnen.
Das ist für dieses Release erwartbar, aber kein Freibrief für unbekannte Dateien:

1. Setup ausschließlich über die offizielle GitHub-Release-Seite beziehen.
2. Dateiname und SHA-256 mit den Angaben oben vergleichen.
3. Bei jeder Abweichung abbrechen und die Datei löschen.
4. Nur nach erfolgreicher Prüfung die Windows-Option **Weitere Informationen** und anschließend
   **Trotzdem ausführen** verwenden.

Die Prüfsumme erkennt eine veränderte Datei gegenüber dem veröffentlichten Sollwert. Sie ersetzt
keine Herausgeber*innenidentität und schützt nicht vor einem vollständig kompromittierten
Veröffentlichungskanal. Das ausführliche Verfahren steht unter
[`docs/release/windows-unsigned-release.md`](docs/release/windows-unsigned-release.md).

### Was beim Release bereits geprüft wurde

- Build aus einem sauberen, dokumentierten Git-Commit.
- Genau ein unsigniertes Windows-x64-NSIS-Setup; kein MSI, Updater oder ARM64-Paket.
- Buildmanifest und Prüfsummendatei stimmen mit dem Setup überein.
- Entwurfsassets wurden von GitHub zurückgeladen und in einem isolierten Ziel installiert,
  registriert und wieder deinstalliert.
- Die später öffentlich heruntergeladenen drei Dateien sind bytegenau identisch mit dem lokalen
  Build und dem geprüften Entwurf.
- Windows- und Ubuntu-CI bestanden die Qualitätskette und jeweils 16 Browser-E2E-Abläufe.

Noch offen ist die vollständige manuelle Abnahme von SmartScreen, interaktivem Erststart,
Projektpersistenz und Deinstallation auf einem unabhängigen Zweitrechner oder einer Clean-VM.

## Wofür KRAQEL gedacht ist

KRAQEL richtet sich an Schreibende, Weltenbauer*innen und Spielgestalter*innen, die ihre Arbeit
lokal kontrollieren möchten und trotzdem mehr Struktur als in einem gewöhnlichen Texteditor
benötigen. Das Projekt verfolgt insbesondere diese Anwendungsfälle:

- Romane, Novellen und serielle Erzählungen mit Kapiteln, Szenen und Figurenbeziehungen.
- Drehbücher und andere szenisch organisierte Texte.
- Pen-&-Paper-Abenteuer mit Orten, Fraktionen, Gegenständen, Ereignissen und mehreren Zeitebenen.
- Umfangreiche fiktionale Welten, deren Kontinuität nachvollziehbar bleiben soll.
- Arbeitsweisen, in denen Cloudkonten, Telemetrie und Anbieterbindung keine Voraussetzung sein
  dürfen.
- Optionale lokale oder explizit freigegebene KI-Unterstützung, ohne dass ein Modell heimlich zur
  Chefredaktion ernannt wird.

KRAQEL ist **kein** Cloud-Kollaborationsdienst, kein automatisches Publikationssystem und derzeit
kein Ersatz für professionelle Satz-, Lektorats- oder Backup-Lösungen.

## Grundsätze

- **Offline first:** Schreiben, strukturieren, suchen, sichern und exportieren funktionieren ohne
  Konto- oder Cloudpflicht.
- **Datensouveränität:** Die autoritativen Projektdaten liegen lokal. Portable Exporte verwenden
  dokumentierte, offene Formate.
- **Explizite Freigabe:** Kein Manuskripttext, Bild, Prompt oder Metadatum darf ohne informierte
  Zustimmung an einen externen Provider gesendet werden.
- **Menschliche Autorität:** KI-Ausgaben sind Vorschläge. Sie überschreiben niemals automatisch
  kanonische Texte, Story-Daten oder Medien.
- **Nachvollziehbarkeit:** SQLite verwaltet den laufenden Zustand; deterministische Snapshots und
  lokales Git machen freigegebene Änderungen überprüfbar.
- **Reversibilität:** Kritische Dateioperationen arbeiten mit Staging, Prüfung, atomarer
  Veröffentlichung und nicht-destruktiver Wiederherstellung.
- **Barrierearmut:** Tastaturbedienung, sichtbarer Fokus, Statusansagen und reduzierte Bewegung sind
  Teil der Qualitätsanforderungen, nicht dekoratives Lametta am Ende des Projekts.
- **Freie Software:** Quellcode, Architekturentscheidungen und Entwicklungsstand sind öffentlich.
  Beiträge sollen solidarisch, respektvoll und nachvollziehbar erfolgen.

## Funktionsumfang

Die folgende Übersicht unterscheidet bewusst zwischen verifiziertem Stand und noch offenen
Produktanforderungen. Eine vorhandene Typdefinition ist keine fertige Funktion – selbst dann nicht,
wenn sie sehr würdevoll kompiliert.

### Schreiben und Dokumentstruktur

- Lokale Projekte mit stabilen UUIDs sowie Kapitel- und Szenenbaum.
- Rich-Text-Editor auf Basis von Tiptap/ProseMirror mit versioniertem JSON-Dokumentformat.
- Szenengebundenes Undo/Redo, Wortzählung und Fokusmodus.
- Entprelltes, serialisiertes Autosave mit optimistischer Revisionsprüfung.
- Konfliktgeprüftes Umbenennen, Umordnen und Löschen von Dokumenten.
- Sichtbare Speicherzustände und Live-Ansagen für Erfolg oder Fehler.
- Volltextsuche über SQLite FTS5.

### Story-Bibel und Worldbuilding

- Typisierte Einträge für Figuren, Orte, Fraktionen und Gegenstände.
- Freie Eigenschaften sowie Unicode-fähige Suche.
- Gerichtete Beziehungen mit eindeutig erkennbarer Richtung.
- Verknüpfungen zwischen Szenen und Story-Bibel-Einträgen.
- Rückverweise aus der Story-Bibel in das Manuskript.
- Referenzsichere Lösch- und Kaskadenregeln in SQLite.
- Bewusstes Zusammenführen doppelter Einträge derselben Art mit Konfliktvorschau.
- Atomare Übernahme von Szenen-, Beziehungs-, Timeline-, POV-, Medien- und Profilreferenzen; zwei
  visuelle Profile werden mit Zielvorrang bei Konflikten und vereinigten Ausschlüssen kombiniert.

### Plotboard und Timelines

- Persistente Szenenkarten mit Status, Handlungsstrang, Position und optionaler POV-Figur.
- Atomare Neuordnung vollständiger Kartenlisten.
- Mehrere parallele Zeitebenen pro Projekt.
- Relative Ereignisreihen und absolute Zeitangaben.
- Verknüpfungen von Ereignissen mit Szenen und Story-Bibel-Einträgen.
- Referenzprüfung über Projektgrenzen hinweg.

### Medienbibliothek

- Lokaler Import von PNG, JPEG und WebP unter harten Größen- und Formatgrenzen.
- Vollständiges Dekodieren vor Annahme; Dateiendung allein gilt nicht als Wahrheit.
- SHA-256, normalisierte Pfade und lokal erzeugte Vorschaubilder.
- Titel, Tags, MIME-Typ, Abmessungen und Herkunftsmetadaten.
- Links zu Szenen, Story-Bibel-Einträgen und Timeline-Ereignissen.
- Sicheres Ersetzen und Löschen von Originalen und Vorschaubildern.
- Responsive Medienkarten; der enge 360-Pixel-Fall ist gegen horizontalen Überlauf geprüft.

### Visuelle Profile und Bildprompt-Entwürfe

- Strukturierte visuelle Kernmerkmale und explizite Ausschlüsse je Story-Bibel-Eintrag.
- Optionales kanonisches Referenzbild aus demselben Projekt.
- Projektweite Stilvorlagen mit begrenzten, typisierten Geltungsbereichen.
- Deterministische lokale Zusammensetzung von Bildprompts aus Szene, Profilen und Stil.
- Vollständig bearbeitbare, revisionsgebundene Prompt-Entwürfe.
- Lokale Speicherung und UTF-8-Textausgabe.
- Provider- und modellspezifische Capability- sowie Generation-Job-Verträge.

Produktive Bildgenerierung ist noch **nicht** implementiert. Die vorhandenen Bildverträge und der
lokale Test-Mock bilden nur das technische Fundament für Version 0.2.

### Kontinuitätsprüfung

- Deterministische Offline-Regeln für Story-Bibel-, POV-, Verknüpfungs- und Timeline-Daten.
- Befunde mit Regelkennung, konkretem Quellenanker und Rücksprung in das Manuskript.
- Revisionsgebundener Nur-Lese-Diff für vorgeschlagene Textänderungen.
- Keine automatische Veränderung des kanonischen Manuskripts.

KI-basierte Kontinuitätshinweise sind geplant, aber nicht implementiert.

### Export, Snapshots und lokales Git

- Gemeinsame, streng validierte Manuskriptrepräsentation.
- Atomarer Export als Markdown, eigenständiges HTML, DOCX oder A4-PDF.
- Direkter, projektgebundener Zugriff auf den lokalen Exportordner über die Exportansicht.
- Unbekannte Editorformatierungen werden abgelehnt statt stillschweigend verworfen.
- Deterministische portable Projektsnapshots im formal beschriebenen Format v3.
- Freigegebene Medien als inhaltsadressierte Dateien unter ihrem SHA-256.
- Kontrolliertes lokales Git für Initialisierung, Status, Diff und Commit.
- Deny-by-default-`.gitignore`: SQLite, Sicherungen, Schlüssel und temporäre Daten bleiben draußen.
- Streng begrenzter `.kraqel`-/ZIP-Import als stets neues Projekt mit neuer UUID.
- Native Auswahl externer Projektarchive ohne Weitergabe von Pfad oder Archivbytes an React. Ein
  abgebrochener Dialog verändert weder Projektliste noch Fehlermeldung.

### Optionale Text-KI

- Providerunabhängige Rust-Grenze und versionierte Anfrage-/Antwortverträge.
- Lokaler Ollama-Adapter mit Modellabfrage und gestreamten, verwerfbaren Entwürfen.
- Protokollunterstützung für OpenAI, Anthropic, Gemini und Mistral.
- Native Betriebssystem-Schlüsselspeicherung statt Klartext im Projekt.
- Einmalige, request- und revisionsgebundene Kontextfreigaben.
- Bereinigte Fehler über die IPC-Grenze; keine Schlüssel, lokalen Pfade oder internen Diagnosen in
  UI-Fehlermeldungen.

KRAQEL liefert weder lokale Modelle noch Cloudzugänge oder API-Schlüssel mit. Echte Anbieterläufe
und deren Kosten-, Aufbewahrungs- und Kontodialoge sind noch nicht abschließend abgenommen.

## Mitgeliefertes Beispielprojekt

Über **Abenteuer ausprobieren** erzeugt KRAQEL vollständig offline eine frei bearbeitbare Kopie von
**Die Stadt, die den Dienstag verkaufte**. Das Beispielprojekt demonstriert:

- einen ausgearbeiteten Szenenbaum,
- Story-Bibel-Einträge und Beziehungen,
- ein Plotboard,
- drei Timelines,
- visuelle Profile und Stilbezüge,
- eine Medienbibliothek mit 21 Illustrationen.

Jede Betätigung erzeugt ein neues Projekt mit eigener Kennung. Die eingebettete Vorlage bleibt
unverändert; es findet kein Download statt. Das reproduzierbar erzeugte Archiv liegt als
[`examples/die-stadt-die-den-dienstag-verkaufte.kraqel`](examples/die-stadt-die-den-dienstag-verkaufte.kraqel)
im Repository. Aufbau und Reproduktion erklärt [`examples/README.md`](examples/README.md); Herkunft,
Quellcommit und Aufbereitung dokumentiert [`examples/NOTICE.md`](examples/NOTICE.md).

Die Originalinhalte stehen unter **CC BY-SA 4.0**. Enthaltenes Daggerheart-SRD-Material unterliegt
der **Darrington Press Community Gaming License 2.0**. Diese Inhaltslizenzen sind von der GPL-Lizenz
des KRAQEL-Programmcodes zu unterscheiden.

## Was noch nicht fertig ist

KRAQEL 0.1.0 Alpha 4 ist ein Entwicklungsstand, keine verschämt als „Beta“ verkleidete
Fertigbehauptung. Insbesondere fehlen oder bleiben offen:

- externe Windows-Abnahme von SmartScreen, Erststart, Persistenz und Deinstallation,
- allgemeine Dateiauswahl für fremde `.kraqel`-Archive,
- Textimport und vollständiger DOCX-Roundtrip,
- Medien im Manuskriptexport und umfassendere Unicode-Schriftunterstützung im PDF,
- reale End-to-End-Abnahme mit Ollama und den vier Cloudprovidern,
- KI-basierte Kontinuitätshinweise,
- produktive Bildgenerierung, Variantenvergleich und Kartenmodell,
- MSI, ARM64, Codesignierung und automatischer Updater,
- vollständige Linux-Produktabnahme,
- breitere Tests mit NVDA und weiteren assistiven Technologien.

Im Pflichtenheft erwähnt, aber derzeit weder installiert noch verwendet, sind außerdem Tailwind,
React Flow und dnd-kit. Verbindliche Details stehen in
[`docs/DEVELOPMENT_STATUS.md`](docs/DEVELOPMENT_STATUS.md).

## Architektur

```text
┌──────────────────────────────────────────────────────────────┐
│ React 19 · TypeScript 5 · Tiptap 3 · Vite 7                │
│ Oberfläche, Editor, lokale Status- und Freigabedialoge       │
└───────────────────────────┬──────────────────────────────────┘
                            │ versionierte Tauri-Commands
                            ▼
┌──────────────────────────────────────────────────────────────┐
│ Tauri 2 · Rust 2024                                         │
│ Validierung, Dienste, Berechtigungs- und Vertrauensgrenzen   │
└───────┬──────────────┬───────────────┬──────────────┬────────┘
        │              │               │              │
        ▼              ▼               ▼              ▼
     SQLite        Projektordner    lokales Git   optionale Provider
      / FTS5       / Assets/Backup   / Snapshot   / OS-Schlüsselspeicher
```

Die React-Oberfläche greift nicht direkt auf SQLite, Dateisystem, Git, Provider oder
Schlüsselspeicher zu. Privilegierte Operationen liegen hinter versionierten Tauri-Commands und
werden in Rust erneut validiert. TypeScript verbessert Bedienung und Entwicklungsfeedback, ist aber
keine Sicherheitsgrenze.

### Technologiestack

| Ebene      | Technologie                                 | Verantwortung                                                |
| ---------- | ------------------------------------------- | ------------------------------------------------------------ |
| Desktop    | Tauri 2, Rust 2024                          | Native Shell, IPC, Dateisystem- und Betriebssystemgrenzen    |
| Oberfläche | React 19, TypeScript 5, Vite 7              | Ansichten, Zustände, Interaktionen und Freigaben             |
| Editor     | Tiptap 3 / ProseMirror                      | Versioniertes Rich-Text-Dokument und Bearbeitung             |
| Persistenz | SQLite mit FTS5                             | Autoritativer lokaler Projektzustand, Suche und Migrationen  |
| Pakete     | pnpm 11 Workspaces                          | TypeScript-Monorepo und reproduzierbare Abhängigkeiten       |
| Rust       | Cargo Workspace                             | Fachdomäne, Storage, Git, Export, Assets, KI und Kontinuität |
| Tests      | Vitest, Rust-Testframework, Playwright, Axe | Einheiten, Verträge, Integration und Browser-E2E             |
| CI         | GitHub Actions                              | Windows- und Ubuntu-Qualitätsläufe                           |

Architekturentscheidungen werden als ADRs unter
[`docs/architecture/decisions`](docs/architecture/decisions) festgehalten. Das Bedrohungsmodell
steht in [`docs/architecture/threat-model.md`](docs/architecture/threat-model.md).

## Lokale Daten, Projektformat und Sicherungen

### SQLite als lokale Autorität

Jedes verwaltete Projekt besitzt eine lokale `kraqel.sqlite3`. Der interne Migrationsstand liegt
aktuell bei `PRAGMA user_version = 5`; die davon getrennte portable Projektschemaversion ist `1`.
Migrationen laufen nur vorwärts und atomar. Datenbanken aus unbekannten neueren Versionen werden
nicht verändert.

SQLite verwendet Fremdschlüssel, Gültigkeitsprüfungen, FTS5 und kontrollierte Transaktionen. Die
laufende Datenbank ist **kein Austauschformat** und gehört nicht in Git.

### Sicherung und Wiederherstellung

- Lokale Sicherungen verwenden die SQLite-Online-Backup-API.
- Sicherungen werden vor Veröffentlichung auf Integrität geprüft.
- Medien erhalten ein hash-verifiziertes `.assets`-Begleitverzeichnis.
- Wiederherstellung schreibt ausschließlich in ein neues Projektziel.
- Originalprojekt und Sicherungsquelle werden nicht überschrieben.

Backups sind Recovery-Artefakte. Für Austausch und Versionskontrolle dient der portable Snapshot.

### Portabler Snapshot v3

Ein Snapshot enthält deterministisch sortierte JSON-Daten, UUID-basierte Markdown-Manuskripte,
Schema-Kopie und freigegebene Medien unter `assets/<sha256>.<endung>`. Zeitstempel ohne fachliche
Bedeutung werden nicht exportiert, damit zwei unveränderte Exporte byteidentisch bleiben können.

Das formale Schema und sämtliche Invarianten stehen unter [`docs/data-format`](docs/data-format).
Der Import kontrolliert unter anderem Archivgröße, Dateianzahl, Pfadsicherheit, Symlinks, doppelte
Einträge, JSON-Struktur, Querverweise, Manuskriptliste, Bilddekodierung und sämtliche Asset-Hashes,
bevor ein neues Projekt sichtbar wird.

## Datenschutz, KI und Sicherheit

KRAQEL verarbeitet besonders schützenswerte Inhalte: unveröffentlichte Manuskripte, Weltentwürfe,
Medien, Prompts, lokale Pfade, Projektmetadaten und gegebenenfalls Provider-Schlüssel. Daraus folgen
verbindliche Grenzen:

- Kernfunktionen müssen ohne Cloudkonto funktionieren.
- Es gibt keine eingebaute Telemetriepflicht.
- Providerzugriffe benötigen eine explizite, konkrete Kontextfreigabe.
- Schlüssel gehören in den nativen Betriebssystem-Schlüsselspeicher, nicht in Projektdateien.
- KI-Antworten bleiben verwerfbare Entwürfe, bis ein Mensch bewusst entscheidet.
- Exporte und Snapshots dürfen keine Schlüssel, absoluten Pfade oder unfreigegebenen temporären
  Daten enthalten.
- Fehler über die UI-Grenze werden bereinigt.

Sicherheitsprobleme bitte **nicht** in einem öffentlichen Issue veröffentlichen. Verwende
stattdessen die private GitHub-Funktion **Report a vulnerability**. Weitere Hinweise und der
Koordinationsrahmen stehen in [`SECURITY.md`](SECURITY.md).

## Barrierearmut

Der aktuelle Prüfstand umfasst:

- automatisierte Tastaturpfade und sichtbare Fokuszustände,
- Axe-Prüfungen des Kernablaufs auf schwerwiegende Befunde,
- semantische Namen und Statusansagen,
- Fehlerzustände, die nicht nur farblich vermittelt werden,
- einen Modus für reduzierte Bewegung,
- Bedienbarkeit bei 200 Prozent Zoom und schmalem Fenster,
- eine manuelle Kern-Hörprüfung mit Windows-Sprachausgabe in der echten Tauri-WebView,
- geprüfte Randfälle für fehlgeschlagenes Autosave und reduzierte Bewegung.

Diese Nachweise sind wichtig, aber keine universelle Barrierefreiheitszertifizierung. Weitere
assistive Technologien, größere Projekte und zusätzliche testende Personen müssen den Umfang noch
verbreitern. Details stehen unter [`docs/testing`](docs/testing).

## Aus dem Quellcode entwickeln

### Voraussetzungen

- Git
- Node.js 22 oder neuer
- pnpm 11; das Repository ist auf `pnpm@11.25.0` festgelegt
- Rust 1.90 oder neuer mit Cargo
- Tauri-Systemabhängigkeiten für das jeweilige Betriebssystem
- unter Windows die erforderliche MSVC-/WebView2-Toolchain
- unter Linux zusätzlich die Entwicklungsbibliotheken für Tauri, D-Bus und Secret Service

Die offizielle Primärplattform ist Windows. Linux wird in CI gebaut und getestet, ist als Produkt
aber noch nicht fachlich abgenommen.

### Repository holen und prüfen

```powershell
git clone https://github.com/kolkrabeofdoom/kraqel.git
cd kraqel
pnpm install --frozen-lockfile
pnpm check
```

### Browseroberfläche starten

```powershell
pnpm dev
```

Der Browsermodus eignet sich für schnelle UI-Entwicklung. Die E2E-Umgebung verwendet einen
testexklusiven lokalen Command-Adapter und ist kein Nachweis für Tauri, Rust oder echte
Dateisystemoperationen.

### Desktopanwendung starten

```powershell
pnpm desktop
```

Dieser Weg startet die React-Oberfläche in Tauri und verbindet sie mit den realen Rust-Diensten.

### Häufige Kommandos

```powershell
pnpm format                     # Prettier und rustfmt anwenden
pnpm format:check               # Formatierung nur prüfen
pnpm lint                       # ESLint und Clippy mit Warnungen als Fehler
pnpm typecheck                  # TypeScript-Projekte prüfen
pnpm test                       # TypeScript-/Vitest-Tests
pnpm test:e2e                   # Playwright-Browser-E2E
pnpm check                      # verbindliche Gesamtprüfung inkl. Rust-Tests
pnpm build                      # TypeScript-Pakete und Frontend bauen
cargo test --workspace          # Rust-Workspace testen
cargo build --workspace         # Rust-Workspace bauen
```

Vor der ersten lokalen Browser-E2E-Ausführung:

```powershell
pnpm --filter @kraqel/desktop exec playwright install chromium
```

Windows-spezifische Tauri-, Installer- und Releaseprüfungen sind in [`AGENTS.md`](AGENTS.md) sowie
[`docs/testing`](docs/testing) dokumentiert. Sie erzeugen teilweise große Artefakte und sollten nur
ausgeführt werden, wenn der Änderungsscope sie tatsächlich erfordert. Ein vollständiger Neubau als
spirituelles Ritual wärmt vor allem den Raum.

## Qualität und Tests

Die verbindliche Standardkette `pnpm check` umfasst:

1. Prettier und `cargo fmt --check`,
2. ESLint und Clippy mit `-D warnings`,
3. TypeScript-Typprüfung,
4. Vitest-/Pakettests,
5. Rust-Tests des gesamten Workspaces.

Zusätzlich existieren:

- 17 getrennte Playwright-Browser-E2E-Abläufe,
- Windows- und Ubuntu-CI,
- ein echter Windows-Tauri-Smoke-Test für Start, IPC, Projektanlage, Autosave, SQLite und
  Wiederöffnen,
- ein isolierter NSIS-Testinstaller mit Installation, Persistenz und Deinstallation,
- ein getrennter Releasepfad mit Manifest, SHA-256 und Rückdownloadprüfung,
- automatisierte Accessibility-Prüfungen und dokumentierte manuelle Hörprüfungen.

Die Browser-E2E prüfen die React-Abläufe mit einem Mock-Adapter. Native Dialoge, tatsächliche
Screenreader-Ausgabe, echte Exportdateien und Betriebssystemintegration benötigen getrennte
Nachweise. Aussagegrenzen gehören zur Testdokumentation – denn ein grüner Haken ist kein
metaphysischer Wahrheitsbeweis.

## Repositorystruktur

| Pfad                         | Verantwortung                                                |
| ---------------------------- | ------------------------------------------------------------ |
| `apps/desktop/`              | React-Oberfläche, Editor, IPC-Client und Browser-E2E         |
| `apps/desktop/src-tauri/`    | Tauri-Konfiguration, Capabilities und native Command-Handler |
| `packages/domain/`           | TypeScript-Domänentypen, UUIDs und Schema-Validierung        |
| `packages/editor/`           | Editorvertrag, Dokumentbaum und Autosave-Hilfen              |
| `packages/ui/`               | Geteilte UI-Bausteine; aktuell noch sehr klein               |
| `packages/ai-contracts/`     | Provider-, Stream-, Capability- und Freigabeverträge         |
| `packages/visual-contracts/` | Medien-, Profil-, Stil- und Promptverträge                   |
| `crates/domain/`             | Rust-Domäne und versionierte Command-Verträge                |
| `crates/storage/`            | SQLite, Migrationen, Projektlebenszyklus und Repositories    |
| `crates/git-service/`        | Snapshots, Archivimport und kontrollierte Git-Operationen    |
| `crates/export/`             | Exportvalidierung, Zwischenmodell und Dateiausgabe           |
| `crates/ai-service/`         | Textprovider, Schlüsselgrenze und Bild-Capability-Registry   |
| `crates/continuity/`         | Offline-Kontinuitätsregeln und Vorschlags-Diff               |
| `crates/asset-service/`      | Bildimport, Hashes, Pfadsicherheit und Vorschaubilder        |
| `docs/architecture/`         | Architekturübersicht, ADRs und Bedrohungsmodell              |
| `docs/data-format/`          | Offenes Snapshot-, Daten- und Exportformat                   |
| `docs/release/`              | Reproduzierbarer Releasebau und Veröffentlichungsablauf      |
| `docs/specifications/`       | Technisches Pflichtenheft und Abnahmekriterien               |
| `docs/testing/`              | Testverfahren, Aussagegrenzen und manuelle Abnahmen          |
| `examples/`                  | Frei lizenzierte, reproduzierbare Beispielvorlage            |
| `scripts/`                   | Build-, Smoke-, Import- und Releasewerkzeuge                 |

## Projektstand und Roadmap

| Stufe    | Ziel                                         | Verifizierter Stand                                                                          |
| -------- | -------------------------------------------- | -------------------------------------------------------------------------------------------- |
| M0       | Monorepo, Tauri, Datenbank, CI               | Technisches Fundament umgesetzt                                                              |
| M1       | Editor, Szenenbaum, Autosave, Backup         | Technischer Mindestumfang mit Browser- und Windows-Smoke-Tests umgesetzt                     |
| M2       | Story-Bibel, Plotboard, Timeline             | Technischer Mindestumfang mit getrennten E2E-Abläufen umgesetzt                              |
| M3       | Git, Import/Export, Medien, visuelle Profile | Teilweise umgesetzt; Snapshotauswahl vorhanden, Text-/DOCX-Import und Roundtrip fehlen       |
| M4       | Text-KI, Kontinuität, Prompt-Entwürfe        | Technischer 0.1-Umfang weitgehend vorhanden; reale Providerabnahme und Bildausführung fehlen |
| M5       | Windows-Release 0.1                          | Alpha 4 veröffentlicht; unabhängige Windows-Abnahme noch offen                               |
| M6 / 0.2 | Bildgenerierung, Varianten, Karten           | Nicht begonnen und ausdrücklich außerhalb von 0.1                                            |

Die maßgebliche, faktenbasierte Übergabe steht in
[`docs/DEVELOPMENT_STATUS.md`](docs/DEVELOPMENT_STATUS.md). Die Arbeitsreihenfolge führt
[`ROADMAP.md`](ROADMAP.md). Fachliche Anforderungen stammen aus dem technischen Pflichtenheft unter
[`docs/specifications`](docs/specifications).

## Mitwirken

Beiträge sind willkommen. Ein guter Einstieg:

1. [`CONTRIBUTING.md`](CONTRIBUTING.md) und [`AGENTS.md`](AGENTS.md) lesen.
2. Ein vorhandenes Issue wählen oder vor größeren Änderungen einen Vorschlag eröffnen.
3. Einen kleinen Branch von `main` anlegen.
4. Verhalten und Risiken mit passenden Tests belegen.
5. Dokumentation und Entwicklungsstatus faktenbasiert aktualisieren.
6. Einen Pull Request mit Problem, Lösung, Risiken und Prüfnachweisen eröffnen.

Neue Netzwerkzugriffe, Datenformate oder Vertrauensgrenzen benötigen eine dokumentierte
Architekturentscheidung. Diskussionen sollen solidarisch, inklusiv und sachlich bleiben. Kritik gilt
Code und Entscheidungen, nie Menschen. Software ist kompliziert genug; wir müssen einander nicht
auch noch kompliziert machen.

## Lizenz und Inhalte

Der KRAQEL-Programmcode steht unter der
**[GNU General Public License, Version 3 oder später](LICENSE)** (`GPL-3.0-or-later`). Du darfst ihn
verwenden, untersuchen, verändern und unter den Bedingungen dieser Lizenz weitergeben.

Beispielinhalte und Drittmaterial können abweichenden, jeweils dokumentierten Lizenzen unterliegen.
Maßgeblich sind die Hinweise in [`examples/NOTICE.md`](examples/NOTICE.md) und in den betreffenden
Quelldateien.

KRAQEL ist ein unabhängiges Freie-Software-Projekt. Genannte Marken, Anbieter und Plattformen
gehören ihren jeweiligen Rechteinhaber*innen. Der Kolkrabe hingegen gehört nach eigener Auskunft
niemandem, hat aber überraschend präzise Vorstellungen von Commit-Nachrichten.
