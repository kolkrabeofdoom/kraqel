# ADR 0007: Portabler Snapshot v1 und kontrollierte lokale Git-Operationen

- Status: angenommen
- Datum: 2026-11-09

## Kontext

F07 verlangt Git Init, Status, Diff, Commit und eine lesende Prüfung des Exportstands. ADR 0001
schließt die laufende SQLite-Datei aus Git aus. Ein gewöhnliches Repository im Projektordner könnte
dennoch Datenbank, Sicherungen oder fremde Dateien erfassen und über globale Git-Filter oder Hooks
lokale Programme ausführen.

## Entscheidung

KRAQEL erzeugt im verwalteten Projektordner den festen Unterbaum `snapshot/`. Formatversion 1
enthält ein Manifest, ein mitgeführtes JSON-Schema, deterministische JSON-Dateien für alle derzeit
implementierten Fachbereiche und Markdown-Dateien mit UUID-basierten Namen. Flüchtige
Datenbankzeitstempel werden nicht exportiert. Neue Snapshotstände entstehen in einem temporären
Geschwisterordner und werden als vollständiger Verzeichnistausch veröffentlicht.

Die lokale Git-Schicht arbeitet ausschließlich mit den festen Pfaden `.gitignore` und `snapshot`.
Die `.gitignore` sperrt zunächst alles und gibt nur diese portablen Dateien frei. System- und
Nutzer*innenkonfiguration, interaktive Prompts, Hooks, externe Diffs und Commit-Signaturzwang sind
für Dienstaufrufe deaktiviert. Commitidentität ist die lokale technische Identität
`KRAQEL <kraqel@localhost>`; sie behauptet keine menschliche Urheberschaft.

Die erste Wiederherstellungsstufe ist bewusst lesend: Sie prüft Struktur und Verträge, verändert
aber keine Datenbank. Der spätere Import muss stets ein neues Projektziel erzeugen und benötigt
eigene Roundtrip-Tests.

## Folgen

- Unveränderte Projekte erzeugen byte-identische Snapshotdateien und leere Folgediffs.
- SQLite, WAL, Sicherungen und andere lokale Dateien bleiben auch bei Git-Fehlbedienung außerhalb
  der erlaubten Pfadmenge.
- Änderungen werden erst durch eine explizite Snapshotaktion sichtbar und nur durch eine weitere
  explizite Aktion committed.
- Das Schema muss bei inkompatiblen Änderungen mit einer neuen Formatversion erweitert werden.
- Medienassets und Hashreferenzen folgen mit der Medienbibliothek; die v1-Assetliste ist leer.
- Lesende Validierung ist noch keine nachgewiesene vollständige Wiederherstellung.
