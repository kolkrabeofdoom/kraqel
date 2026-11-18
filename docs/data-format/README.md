# Offenes KRAQEL-Datenformat

Aktuell · C4b-Generations-/Moduldiagnose und frisches ETW, 18. November 2026:

[Nativer Rollen-/Modulbefund und eigener Netzvergleich](../testing/product-start-p2c-c4b-observation.md)
sind ergänzt. Wiederholte API-Aufnahmen binden Rollen an native PID/Erzeugungszeit; begrenzte
ToolHelp-Diagnosen halten Fehler ausdrücklich fest. Zwei frische synthetische Läufe je ein
Root/sechs Runtimegenerationen, fünf API-Zuordnungen aus 21 Abfragen, eine Runtimegeneration ohne
API-Kind; 126 Modulaufnahmen ohne Diagnosefehler und je 14 unabhängig geprüfte MAC-Records. Zwei
zusätzliche Module außerhalb von Windows-/Kandidatenbaum benötigen eine ausdrückliche sichere
Policyentscheidung; ihre Signaturen erteilen keine Ausnahme. Kein vollständiger
Modulbyte-/Ladehistoriennachweis. 117 aktive Helfer-/Transporttests, Clippy/rustfmt bestanden, sechs
standardmäßig ignoriert. Frisches ETW des exakten Root-/Testcontrollerpaars: 14 explizite Enden,
acht Kontrollen, keine Verluste/Zuordnungslücken und null beobachtete Netzereignisse in diesen
Bäumen einschließlich Loopback. Keine Übertragung auf andere Bytes oder fremde Dienstprozesse. **C4b
bleibt offen:** nicht API-zugeordnete Rolle, verbindliche Modul-/effektive Rollenbindung und
unabhängige effektive Daten-ACL-Prüfung vor Text-/Commandfreigabe. Keine pauschalen Modul- oder
Herstellerausnahmen, keine Änderung an Schutzsoftware. Native UI-ABI 0, keine realen Projekte, keine
neue Produkt-/Recoveryfreigabe. Mittleres Prüfniveau und striktes Datenschutzziel unverändert;
Gesamt grob 65 %, nutzbarer 0.1-Funktionsumfang grob 80 %.

Der C4-Vertrag ist durch den [reinen C4a-Modellkern](../testing/product-start-p2c-c4a-model.md)
ergänzt: Paket-/Policytypen, deterministische Codierung und versionierter Lebenszyklus, zwölf neue
Tests. Keine native UI-/Startautorität; native UI-ABI bleibt 0. Nächster Schritt ist C4b mit nativer
Paket-/Overrideprüfung und belegbarem eingeschränktem Mehrprozess-Erzeugungsweg. C3-Bytes und deren
eigener ETW-Beleg bleiben separat unverändert.

[Produktstart und bestehende Projekte · Vertragsentwurf v1](product-start-integration-v1-draft.md)
beschreibt die geplante Integration der isolierten Oberfläche: getrennte Start-/Importautorität,
Arbeitsablagenbindung und zerstörungsfreie Bestandskopie. P1 ist als reiner Kern geprüft;
[P2a](../testing/product-start-p2a-records.md) ergänzt authentisierte Schema-5-Records nur in
privaten synthetischen Forschungsablagen ohne UI-Start. Kein portables Snapshotformat oder
Produkt-Migrationsweg. P2b ist als
[synthetischer Probeworker](../testing/product-start-p2b-bootstrap.md) umgesetzt;
[frisches ETW](../testing/product-start-p2b-bootstrap-etw.md) ist für den Probeworker belegt.
[Begrenzte Ressourcen-Reconciliation](../testing/product-start-resource-reconciliation.md) ist mit
neuer Execution-/Cleanupversion implementiert. Deren
[eigener frischer Netzbeleg](../testing/product-start-resource-reconciliation-etw.md) ist ergänzt.
[Dauerhafte Workerbindung/Wiederanlauf vor Drained](../testing/product-start-worker-lifetime.md) ist
nun für den synthetischen Ein-Prozess-Worker implementiert.
[Eigener frischer Netznachweis](../testing/product-start-worker-lifetime-etw.md) des Builds ist
ergänzt. Der [P2c-Vertragsentwurf](product-start-p2c-v1-draft.md) ist ausgearbeitet;
[C1](../testing/product-start-p2c-model.md) mit reinen Typen, Inventarcodierung und Lebenszyklus ist
geprüft. [C2](../testing/product-start-p2c-native-data.md) ergänzt ausschließlich neue synthetische
persistente Ablagen und native Paket-/Profilbindung in eigenen Schema-6-DATA-Stores, ohne UI-Start
oder Migration bestehender Stores. Der
[eigene frische C2-Netzbeleg](../testing/product-start-p2c-native-data-etw.md) ist ergänzt.
[C3](../testing/product-start-p2c-group-bootstrap.md) ergänzt einen begrenzten synthetischen
Mehrprozess-Bootstrap mit gehaltenem Job; native UI-ABI bleibt 0.
[Frisches C3-ETW](../testing/product-start-p2c-group-bootstrap-etw.md) ist ergänzt. Der C4-Vertrag
ist oben verlinkt; als Nächstes folgt C4b mit nativen Paket-/Startgrenzen. Mehrprozessige
Produktlebenszeiten und persistente echte Arbeitsablagen-/UI-Bindung bleiben offen. Vorhandener
Snapshot-v3-Import erzeugt neue IDs.

Der Exportstandard ist noch in Arbeit. Fest stehen folgende Invarianten:

- UTF-8 und LF-Zeilenenden
- stabile UUIDs und explizite Schema-Version
- deterministisch sortierte JSON-Schlüssel und Sammlungen
- relative, normalisierte Asset-Pfade mit Prüfsummen
- keine API-Schlüssel, absoluten lokalen Pfade oder unfreigegebenen temporären Daten
- dokumentierte Herkunft für importierte und generierte Medien

Das aktuelle portable Snapshotformat v3 ist in `snapshot-v3.schema.json` formal beschrieben.
`snapshot-v1.schema.json` und `snapshot-v2.schema.json` bleiben für die lesende Kompatibilität
erhalten. Die laufende SQLite-Datei bleibt davon getrennt.

## Portabler Projektsnapshot v3

KRAQEL veröffentlicht einen Snapshot im Projektordner unter `snapshot/`. Der Ordner enthält:

- `manifest.json` mit Formatkennung `kraqel-project-snapshot`, Formatversion, Projektmetadaten,
  fester Dateiliste und Assetmetadaten einschließlich SHA-256, Abmessungen, Herkunft, Tags und
  fachlichen Links;
- `schema.json` als mitgeführte Kopie des JSON-Schemas;
- `documents.json`, `story-bible.json`, `plotboard.json`, `timelines.json`, `visual-profiles.json`
  und `prompt-drafts.json` mit explizitem `kind`-Diskriminator;
- `manuscript/<uuid>.md` mit stabilen, titelunabhängigen Dateinamen;
- `assets/<sha256>.<endung>` für freigegebene PNG-, JPEG- und WebP-Originale. Vorschaubilder sind
  ableitbar und werden nicht dupliziert;
- `README.md` als menschenlesbaren Hinweis auf SQLite als lokale Autorität.

Sammlungen werden fachlich und anschließend nach stabiler UUID sortiert. JSON wird eingerückt mit
sortierten Objektschlüsseln, UTF-8, LF und abschließendem Zeilenumbruch geschrieben. Markdown
normalisiert Zeilenenden auf LF. Erstellungs- und Änderungszeitstempel der Datenbank werden nicht
exportiert, da sie einen fachlich unveränderten Snapshot instabil machen würden.

Die Veröffentlichung verwendet ein neues Geschwisterverzeichnis und tauscht anschließend den
kompletten `snapshot/`-Ordner aus. Assetdateien werden beim Erzeugen und Prüfen gegen den im
Manifest gespeicherten SHA-256 verifiziert. Eine Wiederholung ohne fachliche Änderung ist
byte-identisch. Die lesende Prüfung kontrolliert Manifest, Formatversion, Projektkennung, feste
Dateien, Inhaltsdiskriminatoren sowie UUID-basierte Manuskriptdateien. Die lesende Prüfung
akzeptiert weiterhin Formatversion 1 ohne visuelle Daten und Formatversion 2 ohne Prompt-Entwürfe;
neu erzeugte Snapshots verwenden ausschließlich Version 3.

### Gepackter Snapshot-v3-Import

Ein portables `.kraqel`-Archiv ist ein ZIP-Container, dessen Wurzel exakt den Inhalt eines
Snapshot-v3-Verzeichnisses enthält. Der Rust-Importer akzeptiert ausschließlich Version 3, höchstens
128 MiB und 256 reguläre Dateien. Er lehnt absolute oder ausbrechende Pfade, Verzeichnisse,
Symlinks, doppelte beziehungsweise zusätzliche Einträge und überschrittene Einzeldateigrenzen ab.
JSON, UUIDs, Diskriminatoren, Hierarchien, sämtliche Querverweise, Schema-Kopie,
Manuskriptdateiliste sowie Assetpfade und SHA-256 werden vor dem Schreiben geprüft. Bilder werden
zusätzlich vollständig unter den Grenzen des Medienimports dekodiert.

Der Import überschreibt niemals ein vorhandenes Projekt und übernimmt die Projektkennung der Quelle
nicht. Jeder Lauf erzeugt eine neue UUID, baut Datenbank, Originale und Vorschaubilder in einem
Staging-Verzeichnis auf und veröffentlicht das Verzeichnis erst nach erfolgreichem Abschluss atomar.
Bei einem Fehler bleibt kein sichtbares Teilprojekt zurück. Die Desktopoberfläche verwendet diesen
Weg sowohl für die eingebettete Beispielvorlage als auch für frei gewählte `.kraqel`- oder
ZIP-Dateien. Der native Dialog und das Lesen der begrenzten regulären Datei bleiben in Rust; weder
der absolute Pfad noch Archivbytes werden über IPC an React übertragen. Ein Dialogabbruch liefert
ein neutrales Ergebnis und legt kein Projekt an.

Der Export über **Versionen → Projektarchiv exportieren** erzeugt unmittelbar zuvor einen aktuellen
Snapshot v3 und packt dessen reguläre Dateien in stabiler Pfadreihenfolge ohne Verzeichniseinträge.
Unkomprimierte Summe, Dateianzahl und fertiger Container müssen innerhalb derselben Importgrenzen
liegen. Geschrieben wird zunächst in eine synchronisierte temporäre Geschwisterdatei; ein bereits
bestätigtes reguläres Ziel wird über eine rücksetzbare Vorgängerdatei ersetzt. Symlinks und
Nicht-Dateien sind keine zulässigen Ziele. React erhält nur Dateiname, Bytegröße und
Snapshotzusammenfassung, niemals Zielpfad oder Archivbytes.

Git wird im verwalteten Projektordner lokal initialisiert. Eine deny-by-default `.gitignore` gibt
nur `.gitignore` und `snapshot/**` frei. Add, Status, Diff und Commit erhalten zusätzlich feste
Pfadangaben; Git-System- und Nutzer*innenkonfiguration, Hooks, externe Diffs, Signaturzwang und
interaktive Prompts werden für diese Operationen deaktiviert. Insbesondere bleiben `kraqel.sqlite3`,
WAL-/SHM-Dateien, `backups/`, Schlüssel und temporäre Dateien unversioniert.

## UTF-8-/Markdown-Import

Die Manuskriptansicht kann `.txt`, `.md` und `.markdown` über einen nativen Dialog als neue Szene
unter einem gewählten Kapitel importieren. Rust akzeptiert ausschließlich unverknüpfte reguläre
Dateien bis 2 MiB, deren Größe sich zwischen Metadatenprüfung und Lesen nicht ändert. Inhalt muss
gültiges UTF-8 ohne NUL-Byte sein. Pfad und Bytes verlassen die Rust-Grenze nicht.

TXT wird an Leerzeilen in Absätze getrennt; einzelne Zeilenumbrüche werden als `hardBreak`
gespeichert. Markdown bildet Blocküberschriften der Ebenen 1 bis 6, Zitate, ungeordnete und
geordnete Listen sowie umschlossene Codeblöcke ab. Inline-Markdown und unbekannte Konstrukte bleiben
als sichtbarer Text erhalten. Der begrenzte Dateistamm wird zum editierbaren Szenentitel. Knoten,
Inhalt, Klartextindex und Position entstehen atomar; ein Abbruch oder Fehler schreibt nichts. DOCX
und Format-Roundtrips gehören nicht zu diesem Vertrag. Der getrennte begrenzte DOCX-Import ist in
[`DOCX_IMPORT.md`](DOCX_IMPORT.md) dokumentiert; er ergänzt keine Roundtrip-Garantie.

## Nativer DOCX Manuskriptaustausch v1

Der getrennte [Austauschvertrag](DOCX_ROUNDTRIP.md) definiert den nativen Reader/Writer aus KQ-060:
flache Absatzmarker, ein versioniertes Strukturinventar und vollständig geprüfte Paketbeziehungen.
Der Writer liest seine Ausgabe vor Veröffentlichung semantisch zurück. KQ-061 ergänzt eine eigene
Exportaktion und den Startseitenimport als atomare neue Projektkopie. Dieser Manuskriptaustausch
ersetzt keine vollständige Snapshot-v3-Projektkopie.

## Medienexport für Markdown HTML DOCX und PDF

KQ-062 ist vollständig fachlich freigegeben, KQ-063 implementiert den
[bebilderten Manuskriptvertrag v1](ILLUSTRATED_MANUSCRIPT.md). Die eigene Auswahl am Szenenende
liefert neue vollständige Ordnerpakete mit relativen PNGs und Herkunftsmanifest. KQ-064 ergänzt
gewöhnliches DOCX mit internen PNG-Parts und bestandener Word-/LibreOffice-Abnahme. KQ-065 ergänzt
PDF mit eingebetteten Schriften und weißen PNG-Nachweisen; Snapshot v3, Editor-Dokument v1 und
DOCX-Text-Roundtrip v1 bleiben unverändert.

## Drehbuchformatierung: freigegebener Vertrag und Abnahmegrenzen

KQ-066 legt einen
[fachlich freigegebenen Vertrag](../specifications/SCREENPLAY_FORMAT_CONTRACT_DRAFT.md) mit sechs
Rollen und A4-Profil vor. [ADR 0033](../architecture/decisions/0033-screenplay-contract-proposal.md)
ist nach ausdrücklicher Nutzerfreigabe am 15. November 2026 fachlich angenommen; Rollen und neue
Formatversionen sind noch nicht implementiert. Die
[F08-/M3-Kriterienmatrix](../testing/F08_M3_ACCEPTANCE_MATRIX.md) zeigt vorhandene Belege und offene
Restprüfungen. KQ-067 belegt den
[funktionalen nativen M3-Gesamtlauf](../testing/native-m3-acceptance.md); unabhängige
Desktop-Netzverkehrs-/assistive Vollabnahme und verbleibende CRUD-Nachweise bleiben offen. Snapshot
v3, DOCX v1 und Editor-Dokument v1 bleiben unverändert.

## Lokales SQLite-Schema und Migrationsstand

Die autoritative lokale Datenbank besitzt die Tabellen `project`, `document`, `entity`, `relation`
und `link`. UUIDs werden als kanonische kleingeschriebene Textwerte gespeichert; Fremdschlüssel und
JSON-Gültigkeitsprüfungen sind verbindlich. `document_fts` indexiert Titel und Klartext per FTS5 und
wird durch Trigger synchron gehalten.

`project.schema_version = 1` bezeichnet weiterhin das portable Projektformat. Der davon getrennte
interne SQLite-Migrationsstand ist `PRAGMA user_version = 6`. KRAQEL migriert nur vorwärtsgerichtet
und atomar; Datenbanken aus unbekannten neueren Versionen werden nicht verändert. Die SQLite-Datei
selbst ist kein Austauschformat und bleibt ebenso wie WAL-Nebenfiles aus Git ausgeschlossen.

Migration 6 erlaubt verlustfrei importierte leere Dokumenttitel. Der Tabellenumbau erhält Rowids,
Referenzen und Trigger, prüft alle Fremdschlüssel vor Commit und baut den FTS-Index neu auf. Normale
neue Titel bleiben nichtleer; importierte Titel werden nicht getrimmt.

Migration 2 ergänzt `storyline`, `plot_card`, `timeline`, `event`, `event_document_link` und
`event_entity_link`. Eine befüllte v1-Datenbank wird vorwärts migriert, ohne Dokumente oder
Story-Bibel-Daten umzudeuten. v1-Sicherungen bleiben wiederherstellbar: Nur die neue Zielkopie wird
migriert, die Sicherungsquelle bleibt unverändert.

Migration 3 ergänzt `media_asset` und `asset_link`. Originale und Vorschaubilder liegen unter
`assets/`; die Datenbank speichert ausschließlich normalisierte relative Pfade. Lokale Sicherungen
erhalten ein gleichnamiges `.assets`-Begleitverzeichnis, dessen referenzierte Originale vor der
Veröffentlichung und Wiederherstellung gegen SHA-256 geprüft werden.

Migration 4 ergänzt `visual_profile` und `style_preset`. Ein Profil gehört über eine stabile UUID
genau zu einem Story-Bibel-Eintrag und kann ein kanonisches Asset desselben Projekts referenzieren.
Kernmerkmale, Ausschlüsse, Stilregeln und Entitäts-Geltungsbereiche werden begrenzt und
deterministisch gespeichert. Löschen eines kanonischen Assets leert nur die Referenz; Löschen der
Entität entfernt ihr Profil.

Migration 5 ergänzt `image_prompt_draft` und `image_prompt_draft_profile`. Ein Entwurf verweist auf
genau eine Szene samt Quellrevision, mindestens ein visuelles Profil und optional eine
projektgleiche Stilvorlage. Name und bearbeiteter Prompttext werden lokal gespeichert und begrenzt.
Gelöschte Profile entfernen abhängige Entwürfe; gelöschte Stilvorlagen leeren nur die Referenz. Der
Text-Export verwendet UTF-8/LF und einen festen UUID-Dateinamen unter `exports/prompts`.

## Plotboard- und Timeline-Vertrag v1

Jede Szene erhält genau eine `plot_card` mit stabiler Position, Status (`planned`, `draft`,
`revised`, `done`), optionalem Handlungsstrang und optionaler POV-Figur. Eine neue Reihenfolge wird
als vollständige UUID-Liste atomar gespeichert; fehlende oder doppelte Karten werden abgelehnt.

Zeitebenen sind `relative` oder `absolute`. Relative Ereignisse besitzen nur eine stabile Position;
absolute Ereignisse benötigen zusätzlich einen nicht leeren `time_value`. Ereignisse können n:m mit
Szenen und Story-Bibel-Entitäten desselben Projekts verknüpft werden. Löschen einer Zeitebene
entfernt Ereignisse und Links per Fremdschlüsselregel.

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

Umbenennen verwendet den zuletzt angezeigten Titel als Compare-and-swap-Wert, ohne die
Inhaltsrevision zu erhöhen. Umordnen sendet sowohl die erwartete als auch die gewünschte
vollständige UUID-Reihenfolge genau einer Geschwistergruppe. Löschen bindet den zuletzt angezeigten
Titel und die Inhaltsrevision; für Kapitel muss zusätzlich die geordnete Liste aller unmittelbar
enthaltenen Szenen übereinstimmen. SQLite entfernt danach dokumentbezogene Story-, Plotboard-,
Timeline-, Medien- und Promptreferenzen per Fremdschlüssel beziehungsweise dokumentiertem Trigger.

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

## Bild-Capability- und Generation-Job-Vertrag v1

Der Bildvertrag in `packages/ai-contracts` und `kraqel-ai-service` verwendet `contractVersion = 1`.
Eine lokal auswertbare Registry ordnet die Fähigkeiten `image.generate`, `image.edit`,
`image.reference` und `image.map_render` einzelnen Provider-Modellen zu. Sie beschreibt außerdem
Verfügbarkeit, Prompt- und Referenzlimits, unterstützte Bildgrößen, Ergebniszahl und
Seed-Unterstützung. Eine Providerbezeichnung allein begründet keine Modellfähigkeit.

Ein Auftrag bindet Job- und Projekt-UUID, Provider, Modell, Primärfähigkeit, Prompt-Entwurf, exakt
freigegebenen Prompttext, Referenzasset-UUIDs und validierte Einstellungen. Die Zustände lauten
`queued`, `running`, `succeeded`, `failed` und `cancelled`; Endzustände können nicht wieder geöffnet
werden. Nur erfolgreiche Jobs tragen technische Herkunft, nur fehlgeschlagene Jobs einen bereinigten
Fehler. Fehlercodes und Meldungen enthalten keine Prompts, Providerantworten, Pfade, Schlüssel oder
internen Diagnosedaten.

Dieser Vertrag wird in Version 0.1 nicht in SQLite oder Snapshots persistiert und besitzt keinen
Tauri-Command. Der einzige ausführende Provider ist ein rein lokaler Rust-Test-Mock ohne Netzwerk
und ohne Bilddatei. Produktive Ausführung, Abbruch, Assets und Übernahme bleiben Version 0.2
vorbehalten.
