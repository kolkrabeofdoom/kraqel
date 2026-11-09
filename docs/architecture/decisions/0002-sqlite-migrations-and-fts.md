# ADR 0002: SQLite-Verbindung, Migrationen und Volltextindex

- Status: angenommen
- Datum: 2026-11-09

## Kontext

KRAQEL braucht eine transaktionale lokale Autorität, verlässliche Vorwärtsmigrationen und eine
Offline-Volltextsuche. Das Workspace-Minimum bleibt Rust 1.85; neuere Bibliotheksversionen dürfen
diese Untergrenze nicht beiläufig anheben. Systemabhängige SQLite-Installationen unterscheiden sich
zudem darin, ob FTS5 verfügbar ist.

## Entscheidung

`kraqel-storage` verwendet `rusqlite` 0.39 mit gebündeltem SQLite und FTS5. Diese Version ist mit
dem festgelegten Rust-Minimum vereinbar; das gebündelte SQLite stellt die benötigten Funktionen auf
den unterstützten Build-Plattformen reproduzierbar bereit.

Jede verwaltete Verbindung aktiviert Fremdschlüssel, ein Wartefenster von fünf Sekunden und
`synchronous = NORMAL`. Dateibasierte Datenbanken verwenden zusätzlich WAL. Zusammengehörige
Schreibvorgänge und Migrationen beginnen als `IMMEDIATE`-Transaktion.

`PRAGMA user_version` ist die technische Migrationsversion. Die zentrale aktuelle Version stammt
ausschließlich aus `kraqel-domain`. Migrationen sind geordnete, eingebettete SQL-Dateien und laufen
atomar vorwärts. Eine unbekannte Zukunftsversion wird ohne Schemaänderung abgelehnt.

Der Volltextindex ist eine externe FTS5-Inhaltstabelle für Titel und Klartext von Dokumenten.
Trigger synchronisieren Einfügen, Ändern und Löschen mit der autoritativen `document`-Tabelle.

## Folgen

- Builds erhalten eine größere, dafür reproduzierbare SQLite-Binärkomponente.
- Neue Schemaänderungen benötigen eine weitere unveränderliche Vorwärtsmigration und Tests für leere
  sowie befüllte Datenbanken.
- Das Initialschema prüft Fremdschlüssel, JSON-Felder und zentrale Invarianten direkt in SQLite.
- WAL-Nebenfiles und die laufende Datenbank bleiben aus Git ausgeschlossen.
- Migrationen ersetzen kein Backup; sichere Sicherung und Wiederherstellung folgen in KQ-004.
