# Forschungs-Neustartticket · v2 mit manueller Wiederanlaufprüfung

KQ-069 / #134, 16. November 2026. Nur Windows-Forschungsfeature `isolation-ui-test` und synthetische
Fixtures. Ersetzt im aktuellen Quellcode das [Startticket v1](native-ui-restart-v1.md).
Dateiauftrag, Auftragsjournal und Live-Dateikanal behalten ihre bisherigen Versionen.

## Änderung gegenüber v1

`version` muss 2 sein. Zusätzlich ist `automatic` als boolesches Feld verpflichtend. Alle übrigen
Felder und Bindungen bleiben bestehen: einmalige anonyme Pipes, Launch-ID, exakte AppContainer-SID,
abgeleitete Datenwurzel, Programmhash, geordnete begrenzte Ereignisse und Job-Leerstand vor
Neustart. Keine neuen Rendererbefehle, Pfade, Netzrechte, ACL-Erweiterungen oder Abhängigkeiten.

| `recovery` | `automatic` | Zulässiges Verhalten                                                   |
| ---------- | ----------- | ---------------------------------------------------------------------- |
| false      | true        | Automatisierter synthetischer Erstversuch mit gebundenem Crashpunkt    |
| true       | true        | Automatisierte Wiederherstellung und Replay-Regression                 |
| true       | false       | Manuell bedienbare Wiederherstellung ohne injiziertes Bedienungsskript |
| false      | false       | Ungültig; vor Fensteraufbau abweisen                                   |

v1-Tickets beziehungsweise ein fehlendes `automatic` werden nicht still ergänzt oder akzeptiert.
Neue und alte App-/Controllerbinaries dürfen nicht gemischt werden. Das bereits gesicherte und
ETW-gemessene v1-Paar bleibt als eigener unveränderter historischer Nachweis erhalten; dessen
Netzbeleg gilt nicht für neu kompilierte v2-Programme.

## Manuelle Forschungsläufe

```powershell
$manualRoot = Join-Path $env:TEMP ('kraqel-isolation-ui-restart-' + [guid]::NewGuid().ToString('N'))
& scripts/runtime-isolation/probe/target/debug/kraqel-isolation-probe.exe ui-assistive-interrupted $manualRoot target/debug/kraqel-desktop.exe
```

`ui-assistive-completed` verwendet denselben Aufbau, aber einen Crash nach gespeichertem Abschluss.
Jeder Aufruf verlangt eine frische gebundene TEMP-Wurzel und die passende Forschungs-EXE. Zuerst
wird ein automatisches Vorbereitungsfenster an `prepared` beziehungsweise `completed` hart beendet.
Danach startet derselbe lebende Controller dieselbe isolierte GUI mit neuer Launch-Bindung,
derselben Ablage und `automatic: false` neu. Die Oberfläche erhält keinen neuen Importgrant.

Das Wiederanlauffenster bleibt für Fokus, Sprachausgabe, Tastatur-/Zeigerbedienung und Replay offen.
Der Controller verlangt Wiederherstellungszusammenfassung, nativen Rendererstatus und mindestens
einen tatsächlichen Replay-Aufruf. Bis zu 14 Replay-Aufrufe sind begrenzt zulässig; andere
Ereignisse oder unvollständige Frames werden nicht zu einer bestandenen Prüfung umgedeutet.
Schließen ohne protokollierten Replay-Aufruf ist kein bestandener Lauf.

Für die manuelle Betrachtung gilt eine feste Controllerfrist von höchstens 600 Sekunden. Sie
verlängert keinen Dateiauftrag: Die alte Kennung bleibt verbraucht, eine Quelldatei wird weder neu
geöffnet noch erneut gelesen. Die wiederhergestellte App muss regulär mit Exitcode 0 enden. Danach
werden etwaige Job-Nachfahren beendet, Job-Leerstand und Profilentfernung geprüft.

`passed` im Controllerbericht beschreibt nur den nativen Funktionsablauf. Das Feld
`screenreaderAcceptanceRequiresUserReport: true` macht ausdrücklich kenntlich, dass hörbare Ausgabe
erst durch die prüfende Person bestätigt werden kann. Die bestehende lokale Fokusdiagnose zeichnet
weder Audio noch Tastencodes auf und beweist keine konkrete Eingabemethode.

Die automatische 28-Generationen-Serie bleibt verfügbar und wird mit v2 separat rückgeprüft.
Produktstart, Controller-Crash, dauerhafter Broker und externe Dienste bleiben außerhalb dieser
Freigabe. [Assistiver Nachweis und Buildgrenzen](../testing/native-assistive-restart.md).
