# KQ-069 A: Typisierter Dateiauftrag und atomarer Import im isolierten Testadapter

Stand: 15. November 2026. PR #138 am exakten Head `1a233d1` nach grüner Windows-/Linux-CI und
Dependency Review zusammengeführt (`3f57291`). Pflichtenheft Revision 1 §1/§5/§10 und ADR 0034
Richtung A bleiben unverändert. #134 offen, Q-01/Q-02 für das Produkt nicht bestanden.

## Implementierter Umfang und Grenze

`apps/desktop/isolation-test-adapter` ist ein separater, nicht veröffentlichter Rust-Baustein im
Workspace mit `unsafe_code = forbid`. Er verwendet dieselben `kraqel-git-service`-/Storage-Dienste
wie die Desktop-App, enthält aber **keinen Tauri-Command, WebView oder GUI-Adapter**. Der normale
Tauri-Start ist unverändert unisoliert. Dies ist der erste native App-Dienst-Testadapter, keine
fertige Ende-zu-Ende-Anbindung der Oberfläche an einen Dateibroker.

Der Forschungshelfer startet diesen Adapter im suspendiert geprüften AppContainer-Worker ohne
Capabilities. Genau zwei explizite Handles werden übergeben: ausgewählter Eingabestream und
schreibbare temporäre **Metadatenantwort**, kein Export der Archivbytes. Der frische synthetische
Hostordner erhält keine Paket-ACL. Der nicht isolierte Controller parst keine Archivnutzlast.
Dateihandle-, Metadaten-, Prozess-/Job- und Profilgrenzen des
[Dateiaustauschs](native-runtime-file-exchange.md) bleiben bestehen. Ein gemeinsamer Runner
verhindert unterschiedliche Startregeln für Kopie und Import.

Dieser Schritt verwendet ausschließlich eine fest eingebettete, reproduzierbare Testdatei
`scripts/runtime-isolation/fixtures/input.kraqel`. Kein Argument akzeptiert eine reale Nutzdatei.
Die native Dateiauswahl aus #138 wird hier **nicht erneut bedient oder als integriert behauptet**.
Erzeuger: `make_import_fixture.py`, nur Standardbibliothek, feste IDs/Zeitstempel, ZIP ohne
Kompression, keine externen Daten oder Git-Unterprozesse. Inhalt: ein Kapitel, eine Unicode-Szene,
keine Bilder/Provider. CI prüft Erzeugung und Bytegleichheit unter beiden Plattformen.

## Dateiauftrag v1 und Lebensdauer

[Formales Schema](../data-format/native-file-task-v1.schema.json), tatsächliche Rust-Typen im
Testadapter. Alle Felder sind Pflicht, unbekannte Felder und Operationen werden abgelehnt:

| Feld                            | Bedeutung                                                                               |
| ------------------------------- | --------------------------------------------------------------------------------------- |
| `contractVersion`               | exakt `1`                                                                               |
| `taskId`                        | neue UUID pro frischem Controllerlauf; an die separat gestartete Workerinstanz gebunden |
| `operation`                     | ausschließlich `importSnapshotV3`                                                       |
| `byteCount`                     | erwartete Handle-Dateigröße, 1 Byte bis 8 MiB                                           |
| `issuedUnixMs`, `expiresUnixMs` | positive Lebensdauer von höchstens 60 Sekunden, workergeprüfte Uhr                      |

Keine Hostpfade, Shell-Verben, URLs, Parseroptionen, Providerziele oder Archivbytes im Auftrag. Die
8-MiB-Grenze ist ein engeres Forschungsbudget; der normale Snapshotparser behält seine eigenen
128-MiB-/Datei-/Referenz-/Assetgrenzen. Der Auftrag ist höchstens 2 KiB, die Antwort 4 KiB.

Der Worker prüft Token, Handledatei und Größe vor Dispatch. Der Adapter bindet die ID an seine
erwartete Controllerfreigabe. **Jeder Ausführungsversuch verbraucht die Instanz**, auch falsche
ID/Version, Ablauf, Lesefehler und ungültiges Archiv. Ein zweiter Aufruf wird vor dem Lesen
abgewiesen. Neue Controllerläufe verlangen eine neue TEMP-Wurzel und erzeugen eine neue ID. Keine
automatische Wiederholung oder Wiederverwendung eines abgestürzten Auftrags.

Uhr/Abbruch werden vor dem Lesen, während des begrenzten Einlesens und nochmals **nach kompletter
Initialisierung, unmittelbar vor Veröffentlichung** geprüft. Abbruch vor Start ist ebenfalls
typisiert. Die Teststeuerung setzt eigene `CANCEL`-/Publish-Prüfflags; Uhr/Abbruch sind keine
ungeprüften Rendererfelder. Der isolierte Worker hat zusätzlich die monotone 30-Sekunden-Grenze des
Forschungsrunners.

Neu in `kraqel-git-service`: `import_snapshot_archive_with_publish_guard`. Der bestehende Import
ruft ihn mit einer unverändert zustimmenden Freigabe auf. Der Adapter nutzt die zusätzliche Prüfung
nach `populate_project`. Eine Ablehnung lässt den Projektmanager den vollständig initialisierten,
unveröffentlichten Geschwisterordner entfernen. Bei Zustimmung beginnt die Commitphase: Datenbank
schließen, atomar umbenennen, Projekt öffnen. **Danach ist ein später Abbruch zu spät.** Ein
Fehler/Prozessabbruch nach dieser Grenze darf nicht als garantierter Rollback ausgegeben werden; ein
fertiges Projekt oder ein versteckter Zwischenstand kann nach einem Prozessabsturz bestehen.
Dauerhafte Auftrags-/Crash-Reconciliation ist noch Produktarbeit.

Die Antwort enthält eine neue Projekt-ID, Zähler, Bytezahl und SHA-256 oder ausschließlich einen
stabilen Fehlercode. Keine Quellnamen, Hostpfade, Parsertexte oder ursprüngliche Projekt-ID. UUID
und Ablaufzeit alleine sind **keine allgemeine Brokerautorisierung**: Die Freigabe ist an den
vertrauenswürdigen einmaligen Forschungscontroller und seine Workerinstanz gebunden. Es existiert
noch kein zugelassener Kanal für Aufträge beliebiger Prozesse/Renderer.

## Prüfungen und Ergebnisse

Sieben neue plattformunabhängige Adaptertests: neue Projektkopie/Unicode und Replay-Ablehnung,
ID/Version/Budget/Lebensdauer vor jedem Lesen, vorzeitiger Abbruch, Ablauf, Abbruch/Ablauf nach
vollständigem SQLite-Aufbau, beschädigte/verkürzte Archive bei bytegleichem Bestandsprojekt,
unerlaubte Pfad-/Shellfelder und bereinigte Lesefehler. Zusätzlich 13 bestehende Helfertests.
`pnpm check`, der gezielte Abschlusslauf mit sieben Adaptertests, Clippy/rustfmt, normaler
Tauri-Debug-Build und 30 bestehende Python-Diagnosetests sind erfolgreich. Die Fixture wurde
unabhängig bytegleich neu erzeugt.

Finale native Serie im AppContainer, jeweils eigene Wurzel/Identität, null Capabilities:

| Fall                         | Ergebnis                                                          |
| ---------------------------- | ----------------------------------------------------------------- |
| Gültiges Archiv              | neue Projekt-ID, zwei Dokumente, vollständiger Unicode-Inhalt     |
| Ungültiges Archiv            | `IMPORT_FAILED`, keine neue Projektkopie                          |
| Vor Start abgelaufen         | `EXPIRED`, keine neue Projektkopie                                |
| Vor Verarbeitung abgebrochen | `CANCELLED`, keine neue Projektkopie                              |
| Abbruch nach Initialisierung | `CANCELLED`, kein veröffentlichter oder versteckter Zwischenstand |
| Ablauf nach Initialisierung  | `EXPIRED`, kein veröffentlichter oder versteckter Zwischenstand   |
| Falsche Auftrags-ID          | `INVALID_TASK`, keine neue Projektkopie                           |
| Bisherige Dateikopie         | weiterhin byte-/hashgleich, gemeinsamer Runner geprüft            |

Alle sieben Importläufe weisen Replay zurück. Der Worker richtet vor Dispatch eine separate
synthetische Testbasis ein; Ablehnung erzeugt **keine Importkopie**, die vorherige Testbasis bleibt.
Der unabhängige Nur-Lese-Verifizierer `verify_app_import.py` öffnet SQLite nach Prozessende, prüft
`integrity_check`, Fremdschlüssel, ID-/Ordnerübereinstimmung, Unicode und Dokumentzahl. Er
vergleicht den Basishash mit dem vor Dispatch protokollierten Hash und die Archivhashes unabhängig.
Kein Stagingrest, null aktive Jobprozesse, alle acht gemessenen Profile entfernt. Nachbardatei über
Pfad unlesbar; Hostordner ohne zusätzliche Paketfreigabe.

## ETW desselben finalen Builds

15:06:16–15:07:15 UTC, nach neuer manueller UAC-Bestätigung. ETL: 589.824 Byte, Rohdaten lokal. Alle
acht separat beim Start notierten Controller-PIDs mit Workerbaum: 16 Helfer-/Worker- und 16
Console-Host-Lebenszeiten. Vier positive TCP-/UDP-Kontrollen für IPv4/IPv6, Nullverluste, alle
Prozessenden, keine unbekannten Ziele oder Zuordnungen: **keine Netzereignisse im erfassten Baum**.
Finaler Helfer-SHA-256: `ae68e135917e22830faabf1165f5f2fb2c7720ee59711b6f36fa83fc01ae0589`. Jedes
gestagte Executable wurde unabhängig dagegen verglichen. Keine nachträgliche Änderung des gemessenen
Rust-Binärstands. [Bereinigte Ergebnisse](native-isolated-app-import.json).

Das ist keine universelle Netzstille: externer COM-/Shell-Verkehr, WebView2 im späteren integrierten
Ablauf, öffentliche UDP-/IPv6-/DNS-Gegenproben und normale App-/Installerstarts sind weiter offen.
Die Serie verwendet keine native Dateiauswahl oder echte Nutzdateien.

Nächster Schritt: UI-/Tauri-Testanbindung an den einmaligen Dateiauftrag, echte native Auswahl,
Abbruch/Fehleransagen und Neustart-Reconciliation. Danach weitere Import-/Exportformate,
Git-/Credential-/Providergrenzen und vollständige funktionale/assistive F08-/M3- sowie Packaging-
Abnahme. Keine neue Netzfähigkeit und keine Änderung des strikten Datenschutzziels.
