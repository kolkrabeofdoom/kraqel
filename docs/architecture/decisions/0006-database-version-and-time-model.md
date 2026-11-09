# ADR 0006: Getrennte Datenbankversion und explizites Zeitmodell

- Status: angenommen
- Datum: 2026-11-09

## Kontext

KQ-007 benötigt erstmals Tabellen, die im Initialschema nicht vorhanden sind. Das bestehende Feld
`project.schema_version` beschreibt das portable Projektformat und ist durch Schema v1 abgesichert.
Es gleichzeitig als SQLite-Migrationszähler zu behandeln, würde eine unnötige Neuerstellung aller
abhängigen Tabellen erzwingen.

## Entscheidung

Das portable Projektformat bleibt `project.schema_version = 1`. Der interne Migrationsstand wird
getrennt über `PRAGMA user_version` geführt und steigt mit KQ-007 auf 2. Migrationen laufen einzeln
in sofortigen Transaktionen; v1→v2 wird mit einer befüllten Vorgängerdatenbank getestet.

Plotboard-Karten referenzieren Szenen und besitzen vollständige, atomar ersetzte Reihenfolgen.
Handlungsstrang und POV sind optionale Referenzen; ein gelöschter Handlungsstrang lässt die Szene
erhalten und entfernt nur die Zuordnung.

Jede Zeitebene legt ihr Ordnungsmodell fest. Relative Zeitebenen erlauben ausschließlich eine
Sequenz. Absolute Zeitebenen verlangen zusätzlich einen expliziten, bewusst noch nicht
kalendersemantisch interpretierten Zeitwert. Ereignisse verknüpfen Szenen und Entitäten über
separate n:m-Tabellen.

Drag-and-drop ist ein zusätzlicher Bedienweg. Dieselbe persistente Neuordnung muss über beschriftete
Tasten möglich sein.

## Folgen

- Datenbank- und portables Projektformat können sich unabhängig, aber explizit entwickeln.
- Zeitwerte bleiben erzählerisch offen; Kalenderarithmetik und Zeitzonen sind nicht vorgetäuscht.
- Eine spätere Kalenderdomäne erfordert einen neuen Vertrag und eine Vorwärtsmigration.
- KQ-008 kann Plotboard und Timelines deterministisch exportieren, ohne die laufende SQLite-Datei zu
  versionieren.
