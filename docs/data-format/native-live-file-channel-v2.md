# Laufende native Auswahl · Forschungsentwurf v2

KQ-069 / #134, 16. November 2026. Implementierungsversuch im expliziten Windows-Feature
`isolation-ui-test`. Native Auswahl und Dialogabbruch funktional bestätigt;
[Nachweis und offene Abnahme](../testing/native-live-file-selection.md). Netzwerkabnahme offen.

`ui-live <frische TEMP-Forschungswurzel> <Forschungs-EXE>` startet das isolierte Fenster vor der
Dateiauswahl. Zwei anonyme, gezielt vererbte Pipe-Handles binden Controller und App. Der Empfänger
prüft AppContainer, null Capabilities, Forschungswurzel, Handletypen und einmaligen Anspruch und
entfernt die Vererbungsflags vor WebView-Start. Keine zusätzliche Paket-ACL auf dem Hostordner.

Der vorhandene parameterlose Importbefehl verbraucht die Sitzung. Native Rust-Logik sendet exakt ein
Byte `1` an den Controller. Der Renderer erhält weder Pipe-Handles noch Datei-/Hostpfade. Der
Controller verarbeitet genau einen Auftrag und zeigt den nativen Öffnendialog. Ausschließlich die
vorher selbst erzeugte synthetische `input.kraqel` ist zugelassen. Dateiöffnung nur lesend,
Reparse-/Offline-/Hardlinkprüfung und bestehendes Forschungsbudget von 8 MiB; keine Formatparser im
Host. Auswahl und Transfer laufen auf einem eigenen Thread, während die Jobüberwachung aktiv bleibt.
Beim Appende setzt der Controller Abbruch, schließt seine Pipe-Duplikate und fordert nur eigene
Dateidialoge des eigenen Auswahlthreads mit Dialogklasse `#32770` per `WM_CLOSE` zum Schließen auf.
Höchstens fünf Sekunden auf den Thread warten; fehlender Abschluss bleibt ein Fehler.

Antwort bis EOF: entweder exakt ein Byte `0` für Auswahlabbruch oder Byte `1`, vier Byte
Little-Endian-JSON-Länge (1–2048), typisierter FileTask v1 und exakt `byteCount` Archivbytes.
Unbekannte Felder, falsche ID, Verkürzung, zusätzliche Bytes, Budgetüberschreitung und Ablauf werden
abgelehnt. Die 60-Sekunden-Frist beginnt nach Auswahl; die Empfangswartezeit beträgt höchstens 120
Sekunden und reagiert auf den App-Abbruch. Anschließend nutzt Rust den vorhandenen atomaren Importer
mit Publish-Guard. Der ursprüngliche Einwegkanal v1 bleibt ein eigener Modus.

Pro Sitzung ein Versuch, auch bei Dialogabbruch. Erneute Auswahl, dauerhafte Aufträge und
Crash-Reconciliation bleiben Folgearbeit. Controllerdialog und Appfenster sind getrennte native
Fenster. Bei bereits angefordertem Sitzungsende wird keine Abbruchantwort mehr an die beendete App
geschrieben. Ein lokaler Forschungsmodus `ui-live-transfer-hold` hält nach 1.024 Archivbytes und
belegt gesendete/empfangene Bytezahlen, höchstens 60 Sekunden. Kein Rendererparameter und keine
zusätzliche Dateifreigabe.
[Fensterende-Nachweise und Grenzen](../testing/native-live-window-close.md).

Tests: neuer Decoderfall prüft jede Verkürzung einer gültigen Nachricht, falsche Bindung, Ablauf,
zusätzliche Bytes, neutrale Abbruchantwort und übergroßen Frame. Bestehende 18
Helfer-/Transporttests und zwei neue Tests bestehen; Feature-Clippy ist grün. Native Auswahl/Abbruch
und unabhängige SQLite-/Cleanup-Prüfung bestehen. Replay nach Erfolg/Fehler sowie Fensterende am
Publish-Haltepunkt nativ geprüft; Replay-Ansage bleibt nun über Statusaktualisierungen erhalten. ETW
des korrigierten Forschungsbuilds: drei vollständige Bäume, vier Kontrollen, Nullverluste, 0
Nicht-Loopback-Ereignisse. [Buildgrenzen und Aggregate](../testing/native-live-file-selection.md).
Appende bei offenem Dialog/Teiltransfer funktional bestätigt. Neuer ETW-Lauf dieses geänderten
Stands und assistive Abnahme bleiben offen. Striktes Datenschutzziel unverändert; Q-01/Q-02 für das
Produkt bleiben nicht bestanden.
