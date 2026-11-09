# Offenes KRAQEL-Datenformat

Der Exportstandard ist noch in Arbeit. Fest stehen folgende Invarianten:

- UTF-8 und LF-Zeilenenden
- stabile UUIDs und explizite Schema-Version
- deterministisch sortierte JSON-Schlüssel und Sammlungen
- relative, normalisierte Asset-Pfade mit Prüfsummen
- keine API-Schlüssel, absoluten lokalen Pfade oder unfreigegebenen temporären Daten
- dokumentierte Herkunft für importierte und generierte Medien

Ein formales portables Manifest und JSON-Schemas fehlen weiterhin und werden mit der Snapshotarbeit
konkretisiert.

## Lokales SQLite-Schema v1

Die autoritative lokale Datenbank besitzt die Tabellen `project`, `document`, `entity`, `relation`
und `link`. UUIDs werden als kanonische kleingeschriebene Textwerte gespeichert; Fremdschlüssel und
JSON-Gültigkeitsprüfungen sind verbindlich. `document_fts` indexiert Titel und Klartext per FTS5 und
wird durch Trigger synchron gehalten.

`PRAGMA user_version = 1` bezeichnet die technische Schema-Version. KRAQEL migriert nur
vorwärtsgerichtet und atomar; Datenbanken aus unbekannten neueren Versionen werden nicht verändert.
Die SQLite-Datei selbst ist kein Austauschformat und bleibt ebenso wie WAL-Nebenfiles aus Git
ausgeschlossen.

## Lokales Projektverzeichnis und Sicherungen

Ein verwaltetes Projekt enthält derzeit die autoritative Datei `kraqel.sqlite3` und das Verzeichnis
`backups/`. Sicherungen heißen `<name>.sqlite3`, werden mit der SQLite-Online-Backup-API erzeugt und
erst nach Integritätsprüfung atomar veröffentlicht. Sie sind lokale Recovery-Artefakte, kein
portables Austauschformat und kein Git-Snapshot.

Recovery schreibt eine geprüfte Sicherung ausschließlich in ein neues Projektverzeichnis. Weder die
Sicherungsdatei noch das ursprüngliche Projekt werden dabei verändert oder überschrieben.

## Lokaler Command-Vertrag v1

Tauri-Commands verwenden eine einheitliche JSON-Hülle:

```json
{
  "contractVersion": 1,
  "requestId": "018f47ea-9b2d-7cc1-98e8-b5f4f6b741a2",
  "payload": {}
}
```

Erfolgsantworten spiegeln Version und `requestId` und liefern ihre Nutzlast unter `data`. Fehler
verwenden ausschließlich die stabilen Codes `INVALID_REQUEST`, `UNSUPPORTED_CONTRACT_VERSION` oder
`INTERNAL`, eine bereinigte Meldung und `retryable`. Sie enthalten keine übermittelten Texte,
absoluten Pfade oder internen Fehlerdetails.

Der Vertrag ist lokal und erlaubt keinen Netzwerkzugriff. Eine neue inkompatible Form benötigt eine
neue `contractVersion`; stilles Umdeuten bestehender Felder ist untersagt, selbst wenn Merkur
rückläufig ist.
