# KQ-069: ETW-Netzvergleich des gesicherten Neustart-Builds

Stand: 16. November 2026. Neuer gültiger ETW-Nachweis exakt des in der
[GUI-Neustartserie](native-ui-restart.md) geprüften und getrennt gesicherten App-/Controllerpaars.
[Bereinigte Aggregate](native-ui-restart-etw.json). Kein Runtimeumbau oder neuer Build für die
Messung; striktes Datenschutzziel unverändert.

App-SHA-256: `ca4e70a39a96ec6c642863494be616c47a7e9243afc4ba20726606ddf7c54324`. Controller-SHA-256:
`12cee20170c7672f959703d592d370f57b7cb90311fed524330f2dd05208508f`. Runtime-Quellstand:
`f1bd30cd7821c55e2d28af51b8780dc183c22b22`. Hashes vor Start, nach Ende und am tatsächlich im
Container ausgeführten Exemplar abgeglichen. Nach der Messung keine Runtimeänderung. Der normale
Debug-Build blieb unangetastet.

## Erfassung und Ergebnis

Vorhandenes `scripts/m3-network/capture.ps1` über den vorhandenen PowerShell-7-Host mit bestätigter
UAC-Erhöhung gestartet. Eigene begrenzte Sitzung, vier lokale TCP-/UDP-Kontrollen für IPv4/IPv6.
Keine Firewall-, Loopback-Ausnahme-, Sicherheitsrichtlinien- oder Capabilityänderung und keine
physische Netztrennung. ETW war vor Controllerstart bereit; Bereitschaft `08:39:47.3777352 UTC`,
Sitzungsende `08:42:04.2088845 UTC`. Die Bereitschaftszeit ist der Metadatenzeitpunkt nach den
bereits aufgezeichneten Kontrollen, kein behaupteter exakter Provider-Startzeitpunkt. ETL-Größe
3.538.944 Bytes. Erfassung gestoppt, XML-Konvertierung abgeschlossen.

Der unveränderte Auswerter lief mit `--mode isolation-ui` und dem lokal festgehaltenen
Controller-PID. Ergebnis: gültige Messung, vier von vier Kontrollen, null verlorene
Ereignisse/Puffer, keine unzuordenbaren Netzereignisse oder unbekannten Ziele. **0 beobachtete
Netzereignisse außerhalb Loopback im zugeordneten Prozessbaum.** Separat: je 72 Loopback-Ereignisse
der IDs 13 und 29 bei WebView2, insgesamt 144. Diese werden nicht als öffentliche Verbindungen oder
Inhaltsübertragung umgedeutet und auch nicht aus der Bilanz entfernt.

Erfasst: ein Controller, 28 App-, 144 WebView2- und 29 Console-Host-Lebenszeiten. Zusätzlich zum
allgemeinen Auswerter wurden die Kernel-Prozessstarts in Reihenfolge mit sämtlichen 28 lokalen
Startbelegen abgeglichen: jede App-Generation vorhanden, alle 202 zugehörigen Lebenszeiten mit
explizitem Ende. Alle 24 GUI-Generationen enthalten je sechs WebView-Prozesse; die vier vor
Fensteraufbau abgewiesenen Starts enthalten keinen. ETW bestätigt Controllerexit 0 sowie 24
erwartete harte Appenden mit Code 77 und vier Startablehnungen mit Code 2. PID-Wiederverwendung wird
über die zeitlich zugeordneten Start-/Endereignisse geprüft, nicht über eine reine PID-Liste.

## Native Gegenprüfung während derselben Messung

Die vollständige autonome Serie bestand: sechs Crashpunkte, jeweils zwei Wiederanläufe und
abgewehrter Altimport, vier ungültige Starts, alle Root-Token vor Resume geprüft. Unabhängige
Nur-Lese-Prüfung nach Prozessende: vier `interrupted`- und zwei `succeeded`-Aufträge, genau zwei
verschiedene Projektkopien mit je zwei Dokumenten, exakter synthetischer Unicode-Inhalt,
Receipt-/Archivhash, SQLite-Integrität und Fremdschlüssel korrekt. Keine Zwischenreste, alle Jobs
leer, Profil entfernt. Kein Rückgriff auf Ergebnisse eines früheren Funktionslaufs.

Raw-ETL/XML, lokale Start-/Ergebnisdateien, PIDs und UUIDs verbleiben ausschließlich in der lokalen
TEMP-Ablage. In Git stehen nur diese datenarmen Aggregate. Native Testprozesse und erhöhte Erfassung
sind beendet. Die 32 vorhandenen Python-Diagnosetests bestehen; Format-/Diff-Prüfung und
Fakten-/Hashabgleich genügen für diese reine Dokumentationsfortschreibung. Keine unnötigen
vollständigen Builds oder Wiederholung der unveränderten Anwendungstests.

## Vergleich und Aussagegrenzen

Der [vorherige Fensterende-Netznachweis](native-live-window-close-etw.json) ist ebenfalls gültig und
beobachtete 0 Ereignisse außerhalb Loopback, allerdings an einem anderen App-/Controllerpaar mit
drei Controllern, drei Apps und 19 WebView-Lebenszeiten. Die aktuelle Messung gilt erstmals für das
gesicherte Neustart-Paar mit 28 App-Generationen. Unterschiedliche Programme, Pfade, Laufzeiten und
Ereignispopulationen erlauben keinen Kausal- oder Ratenvergleich. Alte Nachweise werden nicht auf
neue Hashes übertragen.

Transportmetadaten belegen weder HTTP-Bodies noch Upload-/Inhaltsfreiheit. Externe COM-/Shell-/
DNS-/Brokerpfade und kompromittierte Prozesse werden nicht vollständig kausal erfasst. Lokale
Messkontrollen ersetzen keine Kalibrierung sämtlicher öffentlicher Routen. Der Ablauf nutzt die
eingebettete Fixture; Live-Dateidialog und Live-Pipe-Transfer sind eigene Pfade. Controller-Crash,
assistive Wiederanlauf-Hörprüfung und Produkt-/Packaging-Integration fehlen. Normale Produktstarts
bleiben unisoliert; #134 und Produkt-Q-01/Q-02 bleiben offen. Keine fachliche Runtime-Ausnahme oder
Lockerung des strikten Datenschutzziels.
