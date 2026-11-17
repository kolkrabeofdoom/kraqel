# Lokale Windows-Netzprüfung für M3

[Frischer Prepare-Übergangs-/Kandidatenbeleg](../../docs/testing/native-prepare-transition-etw.md):
regulärer Update-/Recoverypfad als Referenz gegenüber drei Übergangsabbrüchen, unveränderte
Auswertung/Capture. 59 direkte Roots einschließlich fünf verwalteter hashbenannter Starts, acht
Kontrollen, Nullverluste, 182 explizite Enden, 0 Nicht-Loopback. Vollständiger separater
Aufruf-/GUI-/ Daten-/MAC-Abgleich, 42 Diagnose-Tests; kein neuer Build oder Übertrag auf andere
Programme/Pfade.

[Frischer Retry-/Kandidatenbeleg](../../docs/testing/native-package-artifact-retry-etw.md):
unveränderte Auswertung/Capture, zwei kurze Phasen, 21 explizite native Roots einschließlich fünf
hashbenannter verwalteter EXEs. Acht Kontrollen, Nullverluste, 144 explizite Enden, 0
Nicht-Loopback; unabhängiger Aufruf-/GUI-/Daten-/MAC-Abgleich. 42 Diagnose-Tests grün, keine
Übertragung auf andere Programme oder ungemessene Pfade.

`analyze.py --mode native-control` erfasst ausdrücklich ausgewählte kurze native Kontrollaufrufe des
festen Root-Images `kraqel-isolation-probe.exe`, auch ohne Kindprozess/GUI. Jede tatsächlich
gestartete Root-PID angeben und Handles bis Messende halten; alle erwarteten Aufrufe und etwaige
GUIs unabhängig gegen Kernel-Lebenszeiten abgleichen. Leere/fehlende Roots, Verluste, unvollständige
Kalibrierung oder Prozessenden bleiben ungültig. Die Strukturprüfungen der bisherigen Modi werden
nicht verändert. [Gezielter Cleanup-Nachweis](../../docs/testing/native-profile-cleanup-etw.md).

Für verwaltete, hashbenannte Broker erlaubt ausschließlich `native-control` zusätzlich
`--root-map <private JSON-Datei>`: eine nichtleere Liste von `{"pid": 1234, "image": "…"}` mit exakt
diesen Feldern. Positive eindeutige Integer-PIDs müssen genau den `--root-pid`-Angaben entsprechen.
Als erwarteter Basisname ist nur `kraqel-isolation-probe.exe` oder ein vollständiger
kleingeschriebener SHA-256 plus `.exe` zulässig, kein Pfad. Die Auswertung verlangt sowohl PID als
auch erwarteten Namen; sie entdeckt keine Roots automatisch. Paketbytes/Hashfreigabe und die
Vollständigkeit erwarteter Aufrufe unabhängig prüfen. Andere Modi weisen diese Zuordnung ab.
[Frischer Nachweis beider Updateprogramme](../../docs/testing/native-schema4-package-update-etw.md),
42 Diagnose-Tests einschließlich falscher Namen, unvollständiger Zuordnung und öffentlicher
Netzereignisse im hashbenannten Baum. Root-Zuordnungen enthalten private PIDs und bleiben lokal.

`analyze.py --mode broker-ui` ist für die gesamte Controller-Crash-Serie vorgesehen: Root ist der
beim Start festgehaltene Serienprozess. Alle nachgeordneten Broker einschließlich negativer Starts
zählen zum Baum; pro Root muss ein lebenszeitgebundener Broker→App→WebView-Pfad existieren.
Kontrollen, Verlustzähler, vollständige Prozessenden und getrennte Loopback-Zählung bleiben
verpflichtend. Konkrete Generationen/Exitcodes zusätzlich mit den lokalen Startbelegen abgleichen.

`capture.ps1` benötigt eine ausdrücklich freigegebene, erhöhte **PowerShell 7** und Windows-
`logman`/`tracerpt`. Keine Installation, Änderung von ExecutionPolicy, Firewall oder
Datenschutzeinstellungen. Vor Ausführung den Helfer lesen. Er erzeugt eine eigene Sitzung mit
eindeutigem Namen und stoppt ausschließlich diese Sitzung in `finally`. Höchstens 20 Minuten
(Parameter 1–30), 128 MiB, keine Paketnutzdaten. Eine neue Ausgabewurzel verhindert Überschreiben.

Die ETW-Datei enthält systemweite Prozess-/Netzmetadaten einschließlich fremder Prozesse und
Zieladressen. Deshalb **außerhalb des Repositorys** erfassen, lokal behalten und niemals als
Issue-/PR-Anhang hochladen. Nur die zusammengefasste Auswertung ist zur Dokumentation vorgesehen.

```powershell
# In der vom Benutzer selbst freigegebenen erhöhten PowerShell 7:
./scripts/m3-network/capture.ps1 -OutputDirectory "$env:TEMP/kraqel-m3-network-NEW"
```

1. `ready.json` abwarten. Der Helfer hat dann eigene lokale TCP-/UDP-Kontrollen für IPv4/IPv6
   erzeugt; das ist keine Übertragung durch KRAQEL.
2. Isolierten nativen App-Build **nach** Messbeginn starten. PID und exakten Executablepfad beim
   Start separat festhalten. App und alle WebView2-Nachfahren laufen im gemessenen Prozessbaum.
3. Fachabläufe protokollieren, App einschließlich Nachfahren beenden. Bei Neustart jede neue
   Root-PID notieren. Keine privaten Projekte oder Providerdaten verwenden.
4. Im Diagnoseverzeichnis die Datei `STOP` anlegen oder die Frist abwarten. Nach `complete.json`
   liegen `events.xml` und der textuelle tracerpt-Verlustbericht `summary.xml` vor. Die Endung
   dieses Windows-Berichts garantiert kein XML-Format.
5. Ohne erhöhte Rechte auswerten (Python 3.10+ mit Standardbibliothek):

```powershell
python scripts/m3-network/analyze.py "$env:TEMP/kraqel-m3-network-NEW/events.xml" `
  "$env:TEMP/kraqel-m3-network-NEW/ready.json" --root-pid 1234 --root-pid 5678
python -m unittest discover -s scripts/m3-network -v
```

Exitcode 0 verlangt vollständige positive Kontrollen, Verlustfreiheit, Start-/Stop-Belege des
gewählten Baums samt modusspezifischen Strukturprüfungen und keine Ereignisse außerhalb Loopback.
Fehlende Evidenz wird nicht als Netzstille gewertet. Die Zuordnung berücksichtigt
Prozessgenerationen und PID-Wiederverwendung; mehrdeutige Root-PIDs verhindern eine Freigabe. Die
Ausgabe enthält keine IP-Adressen, Commandlines oder fremden Prozessnamen.

Ein positiver Verkehrsbefund bleibt ein Befund, auch wenn etwa der Prozess vor Messende noch nicht
geschlossen wurde; eine grüne Vollabnahme ergibt sich daraus nicht. ETW belegt hier
Transportereignisse, keine DNS-Namen, HTTP-Anfragen, Telemetrieinhalte oder Manuskript-Uploads. Es
wurde keine physische Netztrennung durchgeführt. IPv4/IPv6 und TCP/UDP decken weder sämtliche
Windows-Brokerpfade noch jede denkbare Netztechnik ab. Der Befund aus KQ-068 ist in Issue #134
offen; der Helfer verändert die Anwendung nicht, um ein besseres Messergebnis zu erhalten.

Werkzeuggrundlage:
[logman create trace](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/logman-create-trace),
[tracerpt](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/tracerpt).

## Ergänzende lokale NetLog-Diagnose (KQ-069)

NetLog unterscheidet URL-Start-Ereignisse und feste öffentliche Routenklassen. Es ist kein
vollständiger Windows-Netzmonitor und kein Upload-Inhaltsaudit. **Exitcode 0 heißt nur erfolgreich
ausgewertet, niemals Netzfreigabe.** Auch null Starts setzen `networkSilenceClaim` nicht auf wahr.
Default-NetLogs können URLs, Kennungen und Header enthalten: ausschließlich außerhalb Git lokal
aufbewahren, nicht an externe Viewer, Issues oder PRs übertragen. Nur synthetische Projekte nutzen.

Die Vorbereitung verlangt Python 3.10+, einen frischen direkten TEMP-Unterordner mit dem Präfix
`kraqel-m3-acceptance-` und die lokalen Quellen der gelockten wry-Version 0.57.1. Vorhandene Wurzeln
und WebView2-Umgebungsüberschreibungen werden abgelehnt. Beispiel aus der Repositorywurzel:

```powershell
$diagnosticRoot = Join-Path $env:TEMP 'kraqel-m3-acceptance-netlog-NEW'
# Den tatsächlich vorhandenen Cargo-Quellpfad einsetzen:
python scripts/m3-network/prepare_netlog.py $diagnosticRoot --wry-source '<Cargo registry>/wry-0.57.1'
pnpm --filter @kraqel/desktop tauri build --debug --no-bundle --config "$diagnosticRoot/diagnostic.conf.json"
```

Der Helfer übernimmt die geprüften wry-Defaults ausdrücklich, weil `additionalBrowserArgs` sie
ersetzt. Nur zwei Loggingargumente kommen hinzu. Er verändert weder produktive Konfiguration noch
OS-/Firewall-/Datenschutzregeln. Vor dem Start PID, Executable-SHA, Uhrzeit und Projektbestand
festhalten. App ohne Provideraufruf bedienen, anschließend sauber schließen und alle Nachfahren
beenden. **Vor jedem Neustart** die abgeschlossene `netlog.json` in einen neuen Dateinamen unter
`raw/` verschieben: Der nächste Start verwendet denselben Logpfad. JSON erst nach App-Ende
auswerten; unvollständige oder größere Dateien als 32 MiB werden abgelehnt.

```powershell
python scripts/m3-network/netlog.py "$diagnosticRoot/raw/baseline.json"
python -m unittest discover -s scripts/m3-network -v
# Nach allen Diagnoseläufen den normalen lokalen Debug-Build wiederherstellen:
pnpm --filter @kraqel/desktop tauri build --debug --no-bundle
```

Die Projektion gibt keine beliebigen Hosts, IPs, Pfade, Queries, Credentials, Header oder Bodies
aus; nur fest erlaubte öffentliche Hosts und feste Routenklassen. Deklarierte Streamgrößen sind
keine übertragenen Nutzdatenmengen. Starts und unterschiedliche Source-IDs werden getrennt gezählt;
mehrere Starts können zu derselben Quelle gehören. Fehlende Inhalte beweisen keine fehlende
Übertragung.
[KQ-069-Ergebnis und Maßnahmenbewertung](../../docs/testing/native-m3-webview-network.md).
