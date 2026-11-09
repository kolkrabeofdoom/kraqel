# ADR 0001: SQLite als Autorität, Git für Snapshots

- Status: angenommen
- Datum: 2026-11-09

## Kontext

KRAQEL benötigt transaktionale, durchsuchbare lokale Daten und zugleich verständliche,
versionierbare Projektstände. Eine laufende SQLite-Datei ist für Git-Diffs und
plattformübergreifende Reproduzierbarkeit ungeeignet.

## Entscheidung

SQLite ist die autoritative Quelle im laufenden Projekt. Bei ausdrücklich ausgelösten oder
dokumentiert automatisierten Sicherungspunkten erzeugt KRAQEL deterministische Markdown- und
JSON-Snapshots sowie zulässige Assets. Nur diese portablen Artefakte werden durch Git versioniert.

## Folgen

- Jede relevante Entität braucht eine stabile UUID und ein definiertes Exportformat.
- Serialisierung, Reihenfolge, Zeilenenden und Zeitstempel müssen byte-stabil sein.
- Wiederherstellung liest einen Snapshot kontrolliert in eine neue Datenbank ein.
- Datenbankdateien, Schlüssel, private temporäre Daten und nicht freigegebene Medien bleiben aus
  Git.
