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

## Editor-Dokumentvertrag v1

Szeneninhalte stehen in `document.content_json` als versionierte Hülle:

```json
{
  "version": 1,
  "document": {
    "type": "doc",
    "content": [{ "type": "paragraph" }]
  }
}
```

`document` ist Tiptap-/ProseMirror-JSON. Rust prüft Version, Wurzeltyp und Größenlimit, leitet den
Klartext rekursiv ab und aktualisiert Inhalt, `plain_text`, Revision und FTS-Index transaktional.
Autosave sendet die erwartete Revision; ein Konflikt überschreibt keinen neueren Stand.

## Story-Bibel-Vertrag v1

Einträge verwenden stabile UUIDs und genau eine Art: `character`, `location`, `faction` oder `item`.
`properties_json` ist ein Objekt aus höchstens 50 frei benannten Zeichenkettenpaaren;
Eigenschaftsnamen sind eindeutig und werden in den öffentlichen Verträgen deterministisch sortiert.
Namen, Schlüssel, Werte und die serialisierte Gesamtgröße sind begrenzt.

`relation` modelliert eine gerichtete Kante mit `source_entity_id`, `target_entity_id`, frei
benannter Art und optionalen Eigenschaften. Selbstbeziehungen sind im ersten Vertrag unzulässig.
Beim Löschen eines Eintrags entfernt SQLite seine ein- und ausgehenden Beziehungen sowie Links per
Fremdschlüsselregel.

`link` verbindet Dokumente und Einträge innerhalb desselben Projekts. Die aktuelle Rolle lautet
`reference`; die Kombination aus Projekt, Dokument, Eintrag und Rolle ist eindeutig. Die Story-Bibel
liefert diese Links als Rückverweise mit Dokumenttitel und -art aus. Links werden idempotent
angelegt.

Die aktuelle Entitätssuche prüft Namen, Eigenschaftsnamen und -werte Unicode-bewusst in Rust. Sie
ist kein FTS-Index und deshalb für sehr große Story-Bibeln noch auf Skalierbarkeit zu prüfen.

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
verwenden ausschließlich die stabilen Codes `INVALID_REQUEST`, `UNSUPPORTED_CONTRACT_VERSION`,
`CONFLICT` oder `INTERNAL`, eine bereinigte Meldung und `retryable`. Sie enthalten keine
übermittelten Texte, absoluten Pfade oder internen Fehlerdetails.

Der Vertrag ist lokal und erlaubt keinen Netzwerkzugriff. Eine neue inkompatible Form benötigt eine
neue `contractVersion`; stilles Umdeuten bestehender Felder ist untersagt, selbst wenn Merkur
rückläufig ist.
