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

Die UI greift niemals direkt auf SQLite oder Projektdateien zu. Sie kommuniziert über versionierte,
typisierte Tauri-Commands mit Rust. Jede Anfrage und Antwort trägt `contractVersion` und eine
korrelierbare, fachlich bedeutungslose `requestId`. Projekt- und Entitätskennungen sind kanonische
UUIDs; Schema-Versionen sind positive Ganzzahlen mit einer zentral definierten aktuellen Version.

## Folgen

- Jede relevante Entität braucht eine stabile UUID und ein definiertes Exportformat.
- Serialisierung, Reihenfolge, Zeilenenden und Zeitstempel müssen byte-stabil sein.
- Wiederherstellung liest einen Snapshot kontrolliert in eine neue Datenbank ein.
- Datenbankdateien, Schlüssel, private temporäre Daten und nicht freigegebene Medien bleiben aus
  Git.
- Command-Fehler enthalten nur stabile Codes, eine bereinigte Meldung und die Korrelationskennung.
  Eingaben, lokale Pfade, Stacktraces und interne Diagnosedaten gehören weder in den UI-Vertrag noch
  in portable Snapshots.
- Rust validiert Vertragsversion und Nutzlast erneut; TypeScript-Typen allein bilden keine
  Sicherheitsgrenze.
