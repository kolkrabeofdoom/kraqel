# ADR 0005: Story-Bibel, gerichtete Relationen und Rückverweise

- Status: angenommen
- Datum: 2026-11-09

## Kontext

F02 verlangt Figuren, Orte, Fraktionen, Gegenstände, freie Eigenschaften, Beziehungen und
Rückverweise. Das Initialschema enthält dafür bereits `entity`, `relation` und `link`, aber bisher
keine ausführbare Fachlogik. Ein zusätzlicher Schemasprung ohne strukturelle Änderung würde nur eine
Versionszahl erzeugen, die sehr beschäftigt aussieht.

## Entscheidung

Der öffentliche Vertrag begrenzt Entitätsarten auf `character`, `location`, `faction` und `item`.
Freie Eigenschaften sind deterministisch geordnete Zeichenkettenpaare mit Grenzen für Anzahl,
Schlüssel, Wert und Gesamtgröße. Rust validiert sämtliche Eingaben erneut.

Relationen sind gerichtete Kanten mit stabiler UUID, Quelle, Ziel, frei benannter Art und freien
Eigenschaften. Quelle und Ziel müssen im selben Projekt existieren; Selbstbeziehungen sind im
Vertrag v1 ausgeschlossen. Die Oberfläche zeigt eingehende und ausgehende Kanten getrennt.

Dokumentverknüpfungen verwenden die vorhandene `link`-Tabelle mit der Rolle `reference`. Das Anlegen
ist idempotent. Die Detailansicht eines Eintrags liefert Dokumenttitel und -art als Rückverweise;
die Oberfläche kann darüber zur Szene wechseln. Fremdschlüssel mit `ON DELETE CASCADE` entfernen
verwaiste Relationen und Links.

Schema v1 bleibt unverändert. Die Suche lädt die stabil sortierten Einträge eines Projekts und
filtert Namen sowie Eigenschaftsschlüssel und -werte mit Rusts Unicode-Kleinschreibung. Dadurch ist
das Verhalten für nicht-ASCII-Namen korrekt, ohne die bestehende Migration umzudeuten.

KQ-043 ergänzt eine ausdrücklich ausgelöste, transaktionale Dubletten-Zusammenführung. Quelle und
Ziel müssen verschieden, projektgleich und von derselben Art sein. Das Ziel behält UUID, Name und
vorhandene Eigenschaftswerte; nur fehlende Eigenschaften werden aus der Quelle ergänzt. Dokument-,
Relations-, Ereignis-, POV- und Medienreferenzen wechseln vor dem Löschen der Quelle zum Ziel.
Direkte Quelle-Ziel-Relationen entfallen, weil daraus unzulässige Selbstrelationen würden.

Ein ausschließlich an der Quelle vorhandenes visuelles Profil wird samt Promptreferenzen zum Ziel
verschoben. Besitzen beide Einträge ein Profil, bricht die gesamte Transaktion als Konflikt ab. Eine
implizite inhaltliche Vereinigung zweier Profile wäre nicht nachvollziehbar und könnte kanonische
Bildmerkmale vernichten. Der Vertrag benötigt keine neue Migration und ändert das Snapshotformat
nicht.

## Folgen

- Story-Bibel-Daten sind Teil der autoritativen SQLite-Datei und damit automatisch Bestandteil der
  bestehenden konsistenten Backups und nicht-destruktiven Recovery.
- Die Suche ist für übliche Projektgrößen korrekt, aber noch nicht indexgestützt. Vor großen
  Datensätzen ist ein gemessener Migrationsentwurf für Entitäts-FTS erforderlich.
- Beziehungen enthalten derzeit keine eingebetteten Anzeigenamen; die UI löst Namen aus der
  Entitätsliste auf. Portable Snapshots werden erst in KQ-008 definiert.
- Plotboard und Timeline bleiben eigene Module und verändern diesen Vertrag nicht.
- KQ-021 prüft Erstellen, freie Eigenschaften, Suche, ausgehende und eingehende Beziehungen sowie
  den Szenen-Rücksprung mit Playwright in Chromium. Der ausschließlich im E2E-Devserver aktive
  Command-Adapter bildet dafür nur die benötigte Story-Bibel-Semantik ab.
- Der Browserlauf ergänzt Repository-, IPC- und Vertragstests, ersetzt aber weder Rust/SQLite noch
  eine Tauri-WebDriver- oder Screenreader-Abnahme.
- KQ-043 prüft Referenzmigration, Zielvorrang bei Eigenschaftskonflikten, Profilverschiebung und
  vollständigen Rollback bei zwei Profilen in Rust. Ein zusätzlicher Browserablauf belegt Auswahl,
  Konfliktvorschau, Bestätigung und sichtbares Ergebnis.
