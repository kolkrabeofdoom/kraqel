# Nativer Dateiauftragskanal v1 · ausschließlich Forschung

Der Windows-Forschungsbuild `isolation-ui-test` erhält vor dem Start genau einen Auftrag nach
[`native-file-task-v1.schema.json`](native-file-task-v1.schema.json). Dies ist kein produktiver
Dateibroker und kein Dienst für beliebige Clients. Nur synthetische Dateien in einer neuen
TEMP-Forschungswurzel sind zugelassen. Das strikte Datenschutzziel bleibt unverändert.

## Transport und Bindung

Der Controller wählt die feste Testdatei vorab über den nativen Dialog (`ui-select`) oder den
expliziten Fixturemodus (`ui-fixture`). Es gibt keinen rendererbestimmten Hostpfad. Der Controller
öffnet die Datei ausschließlich lesend und prüft Metadaten. Er erstellt eine anonyme Byte-Pipe,
schreibt den vollständigen Auftrag und schließt deren Schreibende **vor** `CreateProcessW`.

Frame: vier Byte Little-Endian-Länge, danach 1–2048 Byte UTF-8-JSON. Anschließend ist EOF Pflicht;
weitere Bytes/Frames, Verkürzung, Übergröße oder unbekannte Felder werden abgelehnt. Die Version
steht im `contractVersion` des typisierten Auftrags. Es wird kein Verbindungsname, Port oder
öffentlicher RPC-Endpunkt bereitgestellt. Die Pipe fließt ausschließlich **zum** Container; es gibt
keinen Rückkanal für Hostaufträge oder Nutzbytes.

Genau zwei Handles stehen in der Start-Whitelist: Eingabedatei und Pipe-Leseende. Der Controller
prüft den suspendierten App-Prozess auf seine erwartete AppContainer-SID und null Capabilities,
weist den Kill-on-close-Job zu und setzt erst dann fort. Der Empfänger prüft seinen Token erneut,
Handletypen, Nicht-Aliasierung, Dateimetadaten/Größe, erwartete Auftrags-ID, Version/Budget und
Lebensdauer. Seine Empfängerinitialisierung kann nur einmal beansprucht werden, auch bei Fehlern.

Die UUID aus den Startargumenten ist **kein Passwort oder eigenständiger Peer-Nachweis**. Die
Bindung beruht auf dem kontrollierten Prozessstart mit den gezielt geerbten Kernelhandles und
zusätzlicher ID-Prüfung. Keine Autorisierung beliebiger Prozesse, keine Abwehr gleichberechtigter
oder privilegierter lokaler Angreifer und keine vollständige Attestierung kompromittierter
Runtime-Prozesse behauptet. Dafür sind weitere Produkt-/Bedrohungsprüfungen erforderlich.

Vor jedem WebView-Start entfernt der Empfänger das Vererbungsflag beider Handles und liest es zur
Kontrolle zurück. Die Pipe wird nach dem einzigen Frame geschlossen. Der Renderer erhält weder
Handlewerte noch einen Hostpfad oder ein wiederverwendbares Grantobjekt. Die Eingabedatei bleibt im
Rust-eigenen `Option<ReceivedGrant>` hinter einem Mutex. Import oder früher Abbruch entnehmen dieses
Objekt genau einmal; ein zweiter Aufruf erhält `ALREADY_CONSUMED`.

## Tauri-Testbefehle

Nur im ausdrücklich aktivierten Windows-Feature registriert. Der normale Builder registriert diese
Befehle nicht. Das Forschungsfenster besitzt keine Core-/Plugin-Berechtigungen und nutzt nur die
vier lokalen App-Befehle. Zusätzlich prüft jeder Befehl das Fensterlabel `isolation-test` und die
exakte gebündelte Seite `/isolation-test.html` auf der lokalen Tauri-Origin. Hauptfenster, andere
Seiten und Remote-/Localhost-Origin werden abgelehnt.

| Befehl                    | Renderer-Nutzdaten | Wirkung                                                            |
| ------------------------- | ------------------ | ------------------------------------------------------------------ |
| `isolation_test_status`   | keine              | datenarme Phase und Ergebnis                                       |
| `isolation_test_import`   | keine              | Grant verbrauchen, vorhandenen nativen Snapshotimport ausführen    |
| `isolation_test_continue` | keine              | Forschungs-Haltepunkt vor Veröffentlichung freigeben               |
| `isolation_test_cancel`   | keine              | unbenutzten Grant schließen oder laufenden Import abbrechen lassen |

Unbekannte zusätzliche Invoke-Felder werden nicht als Parameter übernommen. Es existiert kein Befehl
zum Setzen eines Pfads, Handles, Auftrags, Zeitwerts, Shell-Verbs oder Providers. Der Haltepunkt
dient ausschließlich der überprüfbaren manuellen Abschluss-/Abbruchprüfung. Die normale atomare
Importlogik bleibt im bestehenden Rust-Dienst; nach der Commitfreigabe ist ein späterer Abbruch zu
spät. Ablauf/Abbruch vor Veröffentlichung verwerfen das Staging.

Fensterschließen während des Imports fordert Abbruch an und verhindert den Prozessausstieg, bis der
native Import aufgeräumt hat. Danach wird beendet. Der Controller begrenzt die Sitzung auf zehn
Minuten und beendet beim Schließen des Hauptprozesses die übrigen Jobprozesse. Er wartet auf
Nullprozesse vor Profilentfernung. Ein hartes Prozess-/OS-Versagen ersetzt keine dauerhafte
Crash-Reconciliation; diese bleibt offen.

## Antworten und verbleibende Produktarbeit

Die UI erhält ausschließlich Phase, stabilen Fehlercode und gegebenenfalls neue Projekt-ID,
Dokument-/Assetzahl. Kein Originalname, Pfad oder Archivinhalt. Der Controller liest anschließend
nur den auf 4 KiB begrenzten Diagnoseabschluss und deserialisiert ihn erneut mit strenger Feldliste,
endlichen Phasen/Fehlercodes und UUID als Projekt-ID. Das ist kein allgemeiner
Container-zu-Host-Befehlskanal. Diagnoseartefakte bleiben lokal, nur synthetische Aggregate in Git.

Noch nicht umgesetzt: Auswahl eines weiteren Archivs aus einer bereits laufenden regulären App,
beliebige Nutzdateien, dauerhafte Sitzungs-/Replay-/Crash-Reconciliation, übrige Import-/Export- und
Providerpfade oder produktive Launcher-/Installerintegration. Externe COM-/Shell-Dienste und
kompromittierte Runtime-/Prozessrechte benötigen gesonderte Nachweise. Kein Q-01-/Q-02-Pass.
