# Forschungs-Startticket v3 · Live-Auswahl und Wiederanlauf

16. November 2026, KQ-069 / #134. Ersetzt im aktuellen Forschungsfeature
    [v2](native-ui-restart-v2.md). Kein Produktstart, keine neuen Rendererbefehle oder Capabilities.

`version` muss 3 sein. Das Pflichtfeld `source` ist genau `synthetic` oder `live`. Die bisherige
Session-/Auftrags-/Launch-/Programmhashbindung bleibt bestehen. Kein Default, Downgrade oder
automatisches Ergänzen fehlender Felder. Alte gesicherte Paare behalten eigene Nachweise.

| Ticket                                 | Native Startargumente nach `ui-restart`                 | Importquelle                         |
| -------------------------------------- | ------------------------------------------------------- | ------------------------------------ |
| `recovery: false`, `source: synthetic` | Startup-Reader, Event-Writer, Launch-ID                 | Eingebettete synthetische Testquelle |
| `recovery: false`, `source: live`      | dieselben drei sowie Request-Writer und Response-Reader | Ein gebundener Live-Dateikanal       |
| `recovery: true`, beide Quellen        | nur Startup-Reader, Event-Writer, Launch-ID             | Keine neue Auswahl-/Importquelle     |

Alle übergebenen Handles müssen verschiedene Kernel-Pipe-Handles sein. Nur diese zwei oder vier
Handles werden explizit vererbt; die App löscht und prüft ihre Vererbungsflags vor WebView-Aufbau.
Die zusätzlichen Live-Handles werden erst nach erfolgreicher Ticket-/SID-/Programm-/Rootprüfung
übernommen. Fehlende oder zusätzliche Live-Handles werden abgewiesen; keine zweite Übernahme des
nativen Startkanals. Der bestehende feste Ein-Byte-Auswahlwunsch und begrenzte Live-Antwort bleiben
unverändert. FileTask-ID muss zum Ticket passen, Zeit-/Bytegrenzen werden erneut geprüft.

`automatic: false` bleibt für Wiederherstellung zulässig und darf jetzt außerdem bei initialem
`source: live` gesetzt sein. Dann keine injizierte Bedienungsautomatik; die importierende Person
bedient die vorhandenen Forschungsaktionen. Das Flag erteilt keine zusätzliche Dateiautorität.
Automatische Fixture- und manuelle native Auswahl sind im signierten Hostbeleg unterscheidbar; der
Renderer kann diesen Modus nicht setzen oder ändern.

Der Auftrag wird im isolierten Journal **vor** dem Auswahlwunsch verbraucht. Ein erneuter alter
Erstversuch scheitert vor Auswahl/Lesen, auch wenn er einen neu gebundenen Live-Kanal erhalten hat.
Ein echter Wiederanlauf hat überhaupt keine Auswahlhandles. Der native Host prüft zusätzlich, dass
ein abgewehrter Altimport keinen Auswahlwunsch gesendet hat.

`reading` bezeichnet im kombinierten Live-Modus tatsächlichen Empfang von Nutzbytes; die lokale
Empfangsdiagnose wird vor dem Checkpoint geschrieben. Der deterministische Teiltransfer enthält
1.024 von 11.915 synthetischen Bytes. Vor Controller-Selbstabbruch werden Sender- und Empfangszähler
im nativen Host gegengeprüft. Die übrigen Veröffentlichungsmilestones behalten ihre Bedeutung.

Das neue begrenzte Ereignis `finished` meldet einen terminalen Importfehler/Abbruch, ohne Receipt
oder Freigabe. Wiederherstellung darf neben `INTERRUPTED` die bereits dauerhaft gespeicherten
terminalen Fehler `CANCELLED`, `READ_FAILED`, `IMPORT_FAILED` oder `EXPIRED` anzeigen. Unbekannte
oder nicht terminale Zustände blockieren weiter. Nur `recovered` enthält eine RecoverySummary; die
bisherige strenge Launch-ID-/Ereignisreihenfolge bleibt erhalten. Kein neues Retryrecht.

Neue Folgeaufträge werden ausschließlich über den
[privaten Brokervertrag v3](native-broker-recovery-v3.md) autorisiert. Keine Pfade, Schlüssel,
Grant-Sequenzen oder Wiederherstellungsaktionen über Renderer-IPC. Neue Binaries benötigen eigene
Funktions-, assistive und ETW-Nachweise; alte Hashbelege werden nicht übertragen.
