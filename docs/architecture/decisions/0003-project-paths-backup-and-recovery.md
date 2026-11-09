# ADR 0003: Projektpfade, Sicherung und Wiederherstellung

- Status: angenommen
- Datum: 2026-11-09

## Kontext

KRAQEL verwaltet unveröffentlichte Texte in frei wählbaren lokalen Projektordnern. Pfadtraversal,
Symlink-Fluchten, halbfertige Projekte und überschreibende Wiederherstellungen könnten Daten
offenlegen oder zerstören. Eine laufende SQLite-Datei darf außerdem nicht durch schlichtes Kopieren
gesichert werden, weil WAL und parallele Schreibvorgänge sonst inkonsistente Stände erzeugen können.

## Entscheidung

Der `ProjectManager` arbeitet ausschließlich unter einer beim Start kanonisierten, vorhandenen
Projektwurzel. Die Wurzel und jede vorhandene Pfadkomponente müssen echte Verzeichnisse sein;
absolute Pfade, `.`/`..`, Präfixe und Symlinks werden abgelehnt. Nach jeder Komponente wird erneut
kanonisiert und die Zugehörigkeit zur erlaubten Wurzel geprüft.

Ein Projekt enthält derzeit `kraqel.sqlite3` und das lokale Verzeichnis `backups/`. Erstellung und
Wiederherstellung erfolgen zunächst in einem zufällig exklusiv angelegten Geschwisterverzeichnis.
Erst nach erfolgreicher Datenbank- und Integritätsprüfung wird dieses Verzeichnis innerhalb
desselben Dateisystems atomar umbenannt.

Sicherungen entstehen über die SQLite-Online-Backup-API in einer exklusiven temporären Datei. Vor
der Veröffentlichung werden `PRAGMA quick_check`, Schema-Version und Dateisynchronisation geprüft.
Ein Hardlink veröffentlicht den fertigen Stand atomar und ohne eine vorhandene Sicherung zu
überschreiben. Sicherungsnamen sind einzelne relative Namen; die Erweiterung `.sqlite3` ergänzt die
Anwendung.

Wiederherstellung liest die Sicherung über eine Read-only-Verbindung und schreibt in ein neues, noch
nicht vorhandenes Projektziel. Originalprojekt und Sicherungsdatei bleiben unverändert.

## Folgen

- Fehlende, beschädigte oder nicht unterstützte Datenbanken erhalten unterscheidbare, datenarme
  Fehler; lokale Pfade werden nicht Bestandteil der nutzer*innengeeigneten Meldung.
- Lokale Sicherungen enthalten aktuell nur den konsistenten SQLite-Stand. Assets kommen erst mit der
  Medienbibliothek hinzu; portable Git-Snapshots bleiben ein getrennter Mechanismus.
- Die atomare Veröffentlichung setzt voraus, dass temporäres und endgültiges Ziel im selben
  Dateisystem liegen. Deshalb werden temporäre Dateien immer direkt beim Ziel angelegt.
- Die komponentenweise Prüfung reduziert Symlink-Fluchten, beseitigt aber kein jedes
  Time-of-check/time-of-use-Rennen gegen eine gleichzeitig böswillig veränderte Verzeichnisstruktur.
  Handle-relative Betriebssystem-APIs sind vor dem Import nicht vertrauenswürdiger Projektordner
  erneut zu bewerten.
- Tests decken Windows-taugliche Unicode-/Leerzeichenpfade, lange Verzeichnisnamen, Traversal,
  Symlinks, beschädigte Dateien und unveränderte Recovery-Quellen ab.
