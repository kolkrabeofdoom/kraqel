# Lokale Windows-Netzprüfung für M3

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
App-/WebView2-Baums und keine Ereignisse außerhalb Loopback. Fehlende Evidenz wird nicht als
Netzstille gewertet. Die Zuordnung berücksichtigt Prozessgenerationen und PID-Wiederverwendung;
mehrdeutige Root-PIDs verhindern eine Freigabe. Die Ausgabe enthält keine IP-Adressen, Commandlines
oder fremden Prozessnamen.

Ein positiver Verkehrsbefund bleibt ein Befund, auch wenn etwa der Prozess vor Messende noch nicht
geschlossen wurde; eine grüne Vollabnahme ergibt sich daraus nicht. ETW belegt hier
Transportereignisse, keine DNS-Namen, HTTP-Anfragen, Telemetrieinhalte oder Manuskript-Uploads. Es
wurde keine physische Netztrennung durchgeführt. IPv4/IPv6 und TCP/UDP decken weder sämtliche
Windows-Brokerpfade noch jede denkbare Netztechnik ab. Der Befund aus KQ-068 ist in Issue #134
offen; der Helfer verändert die Anwendung nicht, um ein besseres Messergebnis zu erhalten.

Werkzeuggrundlage:
[logman create trace](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/logman-create-trace),
[tracerpt](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/tracerpt).
