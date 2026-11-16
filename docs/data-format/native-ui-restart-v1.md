# Authentisierter Forschungs-Neustart · v1

KQ-069 / #134, 16. November 2026. Ausschließlich synthetische Daten, separater Controller und
Windows-Feature `isolation-ui-test`. Ergänzt das [Auftragsjournal v1](native-task-journal-v1.md);
kein Produktlauncher, öffentlicher Broker oder Wiederanlauf nach Controllerabsturz.

## Autorität und Lebensdauer

Ein lebender Controller erzeugt eine frische TEMP-Wurzel
`kraqel-isolation-ui-restart-<Sitzungs-UUID ohne Bindestriche>`, eine AppContainer-Identität und
eine unveränderte Kopie der Forschungs-App. Er behält Identität und Datenwurzel über alle
Generationen. Nur `data` erhält Schreib-/Löschrechte und Low Integrity; `bin` bleibt für den
Container les-/ausführbar. Keine erneute ACL-Erweiterung zwischen Generationen, Hostdateiauswahl,
Netzwerkfähigkeit oder Laufzeitabhängigkeit. Der bestehende Live-Auswahlkanal bleibt getrennt.

Jede Generation erhält genau zwei neue anonyme Pipe-Handles über eine explizite Handleliste:
Startticket nur lesend, begrenzte Ereignisse nur schreibend. Der Controller schließt den
Ticketschreiber vor Start; der Empfänger verlangt EOF und genau einen Frame von höchstens 2.048
Bytes. Er löscht und prüft die Vererbungsflags beider Handles vor WebView-Aufbau. Es gibt keinen
benannten Endpunkt, Socket, Rendererpfad oder frei nachlieferbaren Auftrag.

Die neue App startet suspendiert im AppContainer mit null Capabilities, wird einem eigenen
Kill-on-close-Job ohne Breakaway-Freigabe zugeordnet und vor `ResumeThread` auf die genaue
Controller-Identität geprüft. Der Empfänger prüft zusätzlich selbst Container/null Capabilities,
exakten aus der Sitzung abgeleiteten Paket-SID, ausführbaren Pfad ohne Reparsepunkte und SHA-256 der
eigenen EXE. Der lokale Programmstand wird vom Controller beim Kopieren festgeschrieben.

Das strenge Ticket enthält ausschließlich:

| Feld        | Bindung                                                                                  |
| ----------- | ---------------------------------------------------------------------------------------- |
| `version`   | Genau 1                                                                                  |
| `sessionId` | Nichtleere UUID; bindet Profil und Namen der frischen Forschungswurzel                   |
| `taskId`    | Nichtleere, vom Controller erzeugte Auftragskennung                                      |
| `launchId`  | Frische nichtleere UUID je Generation; muss mit dem nativen Startargument übereinstimmen |
| `recovery`  | Wiederherstellung ohne neue Importquelle oder kontrollierter synthetischer Erstversuch   |
| `point`     | Festes Enum: `claimed`, `reading`, `prepared`, `commitIntent`, `published`, `completed`  |
| `appSha256` | Genau 32 Bytes des vom Controller festgeschriebenen Programms                            |

Unbekannte Felder und unbekannte Enumwerte werden abgelehnt. UUID oder Startargument allein sind
keine Authentisierung: Die Autorität kommt aus dem vor WebView-Start konsumierten Kernelkanal und
der exakten OS-Identität innerhalb dieses vertrauenswürdigen nativen Controllers. Wer dieselben
nativen Benutzer*innenrechte beziehungsweise beliebige Kontrolle über den Controller hat, gehört
nicht zu den hier abgewehrten Angreifenden. Eine Signatur oder ein allgemeines langlebiges
Anmeldetoken wird nicht behauptet.

## Reconciliation und Oberfläche

Bei `recovery: true` wird der Store vor Fensteraufbau unter seiner Eigentümersperre geöffnet und
reconciliert. Ausschließlich das terminale Ergebnis der gebundenen alten Kennung wird geladen.
`claim` derselben Kennung muss zusätzlich `ALREADY_CONSUMED` ergeben; andernfalls Startabbruch. Es
gibt keinen Importgrant und keine neue Quelle. Unklarer Zustand wird nicht neu initialisiert.

Die Oberfläche kennzeichnet ein wiederhergestelltes erfolgreiches Ergebnis ausdrücklich als früheren
Auftrag ohne neue Kopie. Nach unterbrochenem Import erklärt sie den verbrauchten Auftrag. Der Fokus
geht zur Wiederholungsprüfung; Import und Abschluss bleiben deaktiviert. Ein neu gestarteter Prozess
mit gültigem frischem Ticket, aber `recovery: false` und alter Kennung wird ebenfalls vor jedem
Lesen abgewiesen.

Das optionale Statusfeld `recovered: true` wird nur in diesem Modus ausgegeben. Bestehende
Live-/Vorab-Auswahlergebnisse behalten ihre Form; deren strenger Controllerdecoder wird nicht
aufgeweicht. Keine neuen Rendererbefehle oder Berechtigungen.

## Integrierte Crash-Serie

`ui-crash-series <frische gebundene TEMP-Wurzel> <Forschungs-EXE>` steuert die echte gebündelte
Forschungsoberfläche über feste, ausschließlich nach gültigem Ticket aktivierte DOM-Skripte. Die
Skripte laufen nur auf der maßgeblichen lokalen Forschungs-URL. Sie klicken die vorhandenen
Schaltflächen; der Browser-Mock ist im nativen Build ausgeschlossen. Keine beliebigen Skripte,
Dateinamen oder Crashpunkte aus dem Renderer. Dieser Ablauf öffnet keine Shell-Dateidialoge; der
Erstversuch nutzt ausschließlich das fest eingebettete synthetische Snapshotarchiv.

Checkpoint-Hooks liegen nach dauerhaftem Claim, nach ersten 512 gelesenen Fixturebytes, nach
vollständiger Vorbereitung, nach gespeichertem Commit-Intent, nach Rename und nach gespeichertem
Abschluss. Der jeweilige native Hook meldet den Haltepunkt und wartet begrenzt; ausschließlich der
Controller beendet den Job hart mit Exitcode 77. Kein regulärer Fehlerreturn, Drop-Cleanup oder
unisolierter Ersatzprozess. `reading` bezeichnet den Adapterstream, nicht den separaten
Live-Pipe-Transfer oder einen geöffneten Shell-Dateidialog.

Nach jeder Generation muss der Job innerhalb der Frist null aktive Prozesse enthalten. Erst dann
darf dieselbe Identität/Ablage wiederverwendet werden. Jedes Szenario enthält zwei
Wiederherstellungen und einen weiteren abgewehrten Importversuch mit alter Kennung. Vier negative
Starts prüfen falsche Launch-Bindung, falsche Sitzungsidentität, falschen Programmhash und falsche
Version. Diese müssen ohne Rendererereignis mit Exitcode 2 enden und das Journal bytegleich lassen.

Ereignisse sind streng typisierte, höchstens 2.048 Byte große Frames mit `launchId`, festem
Ereignis-Enum und ausschließlich beim Wiederherstellungsereignis einer numerischen/booleschen
Zusammenfassung. Der Controller begrenzt den Empfangspuffer auf 4.096 Bytes und prüft Reihenfolge
und Bindung. Er erwartet Wiederherstellung → nativen Statusaufruf des Renderers → Replay → zweites
Replay. Das zweite Replay wird vom festen Skript erst ausgelöst, wenn der erste Verbrauchshinweis im
DOM steht und die Replay-Schaltfläche DOM-Fokus hat. Zusammenfassungen sind funktionale Diagnosen,
keine Autorität für weitere Hostzugriffe oder Attestierung eines kompromittierten Containers. Der
Host parst keine Projektarchive oder Projektdatenbanken für Wiederanlaufentscheidungen.

## Grenzen

Nur Wiederanlauf innerhalb desselben lebenden Controllers. Ein anderer Controller kann keine
bestehende Forschungswurzel übernehmen; der CLI-Start verlangt weiterhin eine frische Wurzel.
Absturz des Controllers, dauerhafter authentisierter Produktbroker, Stromausfälle, weitere Formate
und vollständige Nachfahren-/externe Brokerattestierung bleiben offen. Die autonome Serie belegt
keine gehörte Screenreader-Ausgabe oder vollständige native Fokusabnahme. Keine neue ETW-Messung und
keine Übertragung alter Netzbelege auf geänderte Binärdateien.
[Nachweis und Buildbindung](../testing/native-ui-restart.md).
