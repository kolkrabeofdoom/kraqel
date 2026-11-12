# ADR 0019: Atomarer Snapshotimport und eingebettete Beispielvorlage

Status: angenommen

## Kontext

KRAQEL benötigt ein unmittelbar erkundbares Beispiel mit Dokumenten, Story-Bibel, Strukturwerkzeugen
und Medien. Ein Download beim ersten Start widerspräche Offlinefähigkeit und reproduzierbaren
Builds. Das vorhandene Snapshot-v3-Verzeichnis war bislang nur exportier- und lesend prüfbar; ein
schreibender Import darf weder bestehende Projekte überschreiben noch Teilzustände veröffentlichen.

## Entscheidung

- `.kraqel` ist ein begrenzter ZIP-Container mit exakt einem Snapshot-v3-Inhalt an der Wurzel.
- Rust prüft Container, Schema, Inhalte, Referenzen, Hashes und vollständig dekodierte Bilder vor
  beziehungsweise innerhalb eines unveröffentlichten Staging-Projekts.
- Jeder Import erhält eine neue Projekt-UUID. Quellkennung und vorhandene Projekte werden niemals
  überschrieben; wiederholtes Anlegen erzeugt unabhängige Kopien.
- Datenbank und abgeleitete Vorschaubilder entstehen vor dem atomaren Verzeichniswechsel. Fehler
  entfernen das Staging-Verzeichnis.
- Das frei lizenzierte Abenteuer **Die Stadt, die den Dienstag verkaufte** wird aus einem
  festgeschriebenen Quellcommit reproduzierbar erzeugt und per `include_bytes!` in die
  Desktopanwendung eingebettet. Zur Laufzeit findet kein Netzwerkzugriff statt.
- Die Oberfläche bietet neben der geprüften Vorlage eine native Auswahl für `.kraqel`- und
  ZIP-Dateien. Auswahl, Metadatenprüfung und begrenztes Lesen erfolgen vollständig in Rust; React
  erhält weder absoluten Pfad noch Archivbytes. Abbruch ist ein neutrales Ergebnis.
- Vor dem Lesen werden Symlinks, Nicht-Dateien, leere Dateien und Archive über 128 MiB abgelehnt.
  Anschließend gelten dieselben Inhaltsprüfungen und dieselbe atomare Veröffentlichung wie für die
  eingebettete Vorlage.

## Folgen

- Neue Nutzer*innen können einen realistischen Funktionsquerschnitt offline und ohne Risiko für die
  unveränderte Vorlage erkunden.
- Das Desktopartefakt wächst um ungefähr 10,6 MiB; jede angelegte Kopie benötigt zusätzlich Platz
  für Originale, Vorschaubilder und SQLite.
- Die feste Quelle und Bildtransformation ermöglichen reproduzierbare Archiverzeugung, verlangen bei
  jeder Quellenaktualisierung aber eine bewusste Rechte-, Hash- und Inhaltsprüfung.
- Snapshot-v1/v2 bleiben lesend prüfbar, sind jedoch nicht importierbar. Textimport und
  DOCX-Roundtrip bleiben offene M3-Arbeit.

## Evidenz

- `crates/git-service/src/snapshot_import.rs`
- `crates/storage/src/project_manager.rs`
- `scripts/build-example-project.mjs`
- `examples/NOTICE.md`
- `apps/desktop/e2e/example-project.spec.ts`
- `apps/desktop/src-tauri/src/lib.rs`
