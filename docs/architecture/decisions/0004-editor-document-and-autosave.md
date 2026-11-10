# ADR 0004: Editor-Dokumentvertrag und Autosave

- Status: angenommen
- Datum: 2026-11-09

## Kontext

KRAQEL benötigt Rich Text, Undo/Redo und Autosave, ohne Tiptap-interne Zustände ungeprüft zur
dauerhaften Schnittstelle zu erklären. Gleichzeitig können mehrere UI-Ereignisse eintreffen, während
ein vorheriger Speichervorgang noch läuft. Ein stilles Überschreiben neuerer Revisionen wäre für
Manuskripte ungefähr so hilfreich wie ein Radiergummi mit Eigeninitiative.

## Entscheidung

Gespeicherte Editorinhalte verwenden den expliziten Vertrag
`{ version: 1, document: <Tiptap-JSON> }`. Die Wurzel muss ein ProseMirror-Knoten vom Typ `doc`
sein; Dokumente sind auf fünf MiB begrenzt. Inkompatible Änderungen benötigen eine neue
Editor-Dokumentversion.

Jede Szene erhält beim Öffnen eine eigene Tiptap-Instanz. Damit bleiben Undo und Redo innerhalb der
Dokumentgrenze; ein Szenenwechsel übernimmt keinen fremden Verlauf. `StarterKit` stellt die
grundlegenden ProseMirror-Knoten, Markierungen und Verlaufsfunktionen bereit.

Der Client entprellt Änderungen für 700 Millisekunden, serialisiert parallele Speichervorgänge und
zeigt `Ungespeichert`, `Speichert`, `Gespeichert` oder `Speichern fehlgeschlagen`. Jeder Schreibzug
enthält die erwartete Revision. Rust aktualisiert Inhalt, serverseitig abgeleiteten Klartext,
Revision und FTS-Index in einer Transaktion. Eine veraltete Revision erzeugt `CONFLICT` und
überschreibt keinen neueren Stand.

Titeländerungen verwenden den zuletzt angezeigten Titel als eigenen Compare-and-swap-Wert und
verändern die Inhaltsrevision nicht. Umordnen übermittelt die vollständige bisherige und gewünschte
Geschwisterliste; Rust akzeptiert nur dieselbe UUID-Menge und lehnt einen zwischenzeitlich
veränderten Baum ab. Löschen bindet Titel und Inhaltsrevision, bei Kapiteln zusätzlich die
vollständig angezeigte geordnete Kindliste. Die Oberfläche verlangt zwei getrennte Löschaktionen und
wählt eine betroffene aktive Szene vor der Bestätigung ab.

Die Desktopanwendung verwaltet Projekte vorerst unter `app_local_data_dir/projects/<project-uuid>`.
Der generische `ProjectManager` bleibt von diesem UI-Standardpfad unabhängig. Eine frei wählbare
Projektwurzel ist eine spätere Produktentscheidung.

## Folgen

- Tiptap-JSON ist ein versioniertes Persistenzformat, kein ungeprüfter UI-Blob.
- Der für Suche und Wortzählung benötigte Klartext wird in Rust aus dem JSON abgeleitet; die UI kann
  den FTS-Inhalt nicht separat vorspiegeln.
- Kapitel liegen auf der Wurzelebene; Szenen benötigen ein Kapitel. Geschwister werden stabil nach
  Position und UUID sortiert.
- KQ-005 und KQ-019 unterstützen Erstellen, Öffnen, Schreiben, Umbenennen, tastaturbedienbares
  Umordnen und zweistufiges Löschen. Kapitelkaskaden entfernen Szenen und deren dokumentbezogene
  Links transaktional; eine automatische Wiederherstellung gelöschter Inhalte existiert nicht.
- Tastatur- und Screenreader-Abnahme folgt dem dokumentierten manuellen Testplan; automatisierte
  Browser-E2E-Tests sind noch nicht eingerichtet.
