# KQ-069: Frischer ETW-Netznachweis des kombinierten v3-Prüfpaars

Stand: 16. November 2026. Beide frischen Messphasen sind gültig bestanden. Gemessen wurde
ausschließlich das unveränderte gesicherte Paar aus dem
[kombinierten Funktionsnachweis](native-combined-live-restart.md). Quelle
`9f5a7ee8c56220bb90f10c4f304c9b0f01cd43ee`, Startticket/Brokervertrag v3. App-SHA-256
`6e56a51ed2e913a878a7935e039bd69628a49be25f0fbab7d02fce3a72fe8c70`; Broker-SHA-256
`49a2a0c0f0a8428c9419f8a70ce809e89a9f66171053e0a00032fcb3f6a48852`. Hashes vor/nach Erfassung mit
Manifest/Funktionsnachweis und an allen zehn tatsächlichen Appkopien abgeglichen.
[Bereinigte Aggregate](native-combined-live-restart-etw.json). Kein neuer Build oder Runtimeumbau
für diese Messung, normale Debug-App bleibt unangetastet.

## Getrennte Messphasen

Zwei eigene aufeinanderfolgende begrenzte Sitzungen mit unverändertem `capture.ps1`; ein explizit
per Windows-UAC erhöhtes PowerShell-7-Supervisorprogramm. Jede Sitzung hat eigene lokale
TCP-/UDP-IPv4/IPv6-Kontrollen, Verlustzähler und vollständige Prozessgrenzen. Keine Firewall-/
Loopback-/ExecutionPolicy-/Domänenänderung oder physische Netztrennung. Roh-ETL/XML, PIDs, UUIDs,
lokale Pfade und Schlüssel bleiben ausschließlich lokal.

Die automatische Serie wird unter `--mode broker-ui` als vollständiger Serienbaum gemessen,
einschließlich aller negativen Folgeaufträge und neun Stilllegungen. Die getrennte native
Dialogphase verwendet `--mode isolation-ui` für ihre fünf direkt gestarteten GUI-Controller. Ihre
Root-Prozesshandles werden bis Messende gehalten. Die Trennung verhindert Vermischung
unterschiedlicher Serien und erlaubt unveränderte konservative Prüfmodi statt neuer Ausnahmen.
Zusätzlich werden sämtliche Startbelege chronologisch mit getrennten Kernel-Lebenszeiten und
zugehörigen Brokereltern abgeglichen; PID-Werte allein sind kein Generationsnachweis.

## Automatische Phase

Bereitschaft nach den bereits erfassten Kontrollen `2026-11-16 10:42:08.8437604 UTC`, Sitzungsende
`10:45:09.6370753 UTC`, Konvertierung `10:45:10.5784740 UTC`. Bereitschaft ist kein behaupteter
exakter Providerstart. ETL: 6.684.672 Bytes.

Gültig: vier von vier Kontrollen, null Ereignis-/Pufferverluste, keine Zuordnungslücken oder
unbekannten Ziele. **0 beobachtete Netzereignisse außerhalb Loopback im zugeordneten Serienbaum.**
Getrennt 432 WebView2-Loopback-Ereignisse, je 216 der IDs 13 und 29. 109 Broker-, 72 App-, 432
WebView- und 181 Console-Host-Lebenszeiten; alle 794 Enden explizit erfasst. Genau sechs WebViews je
App. Keine eigenen beobachteten Netzereignisse der Broker im ausgewerteten Ausschnitt.

Kernelenden: Broker 18 × 86, 54 × 1 und 37 × 0; Apps 54 × 77 und 18 × 0 durch OS-Kill-on-close.
Letztere sind keine geordneten Appabschlüsse. Alle 72 nativen Startbelege und ihre Brokereltern
unabhängig lebenszeitgebunden zugeordnet. Neun Fälle mit 18 Controller-Kills, 27 Wiederanläufen und
54 abgewiesenen Folgeaktionen im Messlauf bestanden. Frischer Nur-Lese-Audit bestätigt
Grant-/Abschluss-MACs mit unabhängiger HMAC-Implementierung, kanonische Wurzel-/Programmhashbindung,
IDs/Kopien, Journal/SQLite/Unicode/Receipt-/Archivhash/Fremdschlüssel und leeres Staging. Fünf
unterbrochene, zwei fehlgeschlagene und elf erfolgreiche Aufträge; genau elf legitime Kopien.

## Native Dialogphase

Bereitschaft nach Kontrollen `2026-11-16 10:45:10.6175529 UTC`, Sitzungsende `10:47:27.9879713 UTC`,
Konvertierung `10:47:28.7889085 UTC`. ETL: 5.701.632 Bytes. Eigene vier Kontrollen, Nullverluste,
keine Zuordnungslücken oder unbekannten Ziele. **0 beobachtete Netzereignisse außerhalb Loopback in
den fünf zugeordneten Controllerbäumen.** Getrennt 48 WebView2-Loopback-Ereignisse, je 24 der IDs 13
und 29.

Die prüfende Person bestätigt Importknopf, native Auswahl der vorbereiteten `input.kraqel` und
automatisches Schließen des ersten Fensters am vorgesehenen Haltepunkt. Danach automatisch:
Wiederanlauf/Altimportabwehr, ausdrücklich neuer Fixture-Auftrag, weiterer Controller-Kill,
zweifache Wiederherstellung/Replay. Alle fünf Controller-Exitnummern direkt festgehalten und
unabhängig im Kernelprotokoll bestätigt: zweimal 86 und dreimal 0. Die frühere Lücke beim ersten
manuellen Exitcode betrifft nur den alten Funktionslauf; dieser Messlauf hat einen eigenen Beleg.

Acht App-Starts und 48 WebViews, fünf Broker und 13 Console-Hosts; alle 74 Prozessenden explizit.
Genau sechs WebViews je App. Appenden sechsmal 77 und zweimal 0 durch OS-Kill-on-close, letztere
keine geordneten Appabschlüsse. Sämtliche acht Startbelege und die fünf zugehörigen Eltern-
Lebenszeiten unabhängig abgeglichen. Keine eigenen beobachteten Netzereignisse der Broker. Neuer
unabhängiger MAC-/Root-/Programmhash-/Journal-/SQLite-/Unicode-/Receipt-/Archivhash-/
Fremdschlüsselaudit bestätigt zwei verschiedene verbrauchte IDs, `interrupted`/`succeeded`, genau
eine legitime Kopie und leeres Staging.

Die manuelle Stilllegung erfolgt ausdrücklich **nach** Ende dieser Aufzeichnung, mit erfolgreich
entferntem Profil. Sie ist nicht als zusätzlich gemessener Controller ausgegeben. Derselbe
Stilllegungspfad ist bereits neunmal innerhalb der automatischen Phase enthalten. Private
Grants/Abschlussbelege und synthetische Daten bleiben zur lokalen Nachprüfung erhalten.

## Gesamtstand und Prüfung

Zwei getrennte gültige Erfassungen: acht von acht Messkontrollen, Nullverluste, insgesamt sechs
Rootstarts mit 114 Broker-, 80 App-, 480 WebView- und 194 Console-Host-Lebenszeiten. Alle 868 Enden
explizit; alle 80 App-Startbelege chronologisch/lebenszeitgebunden und mit ihren Brokereltern
abgeglichen. Zusammengenommen **0 Nicht-Loopback- und 480 getrennte Loopback-Ereignisse**. Die acht
Kontrollen sind vier pro eigener Phase, keine doppelte Zählung derselben Kalibrierung.

Brokerenden insgesamt 20 × 86, 54 × 1 und 40 × 0; Appenden 60 × 77 und 20 × 0 durch
OS-Kill-on-close. Keine fehlenden Enden durch bloße PID-/Namensmengen ersetzt. Frisch bestätigt: 20
unterschiedliche Aufträge, zwölf legitime Kopien, 80 suspendierte Root-Tokenprüfungen mit exakter
SID/null Capabilities und 480 tatsächliche Dateiöffnungsverweigerungen. Beide Daten-/
Lebenszeitaudits stimmen mit der unveränderten Primärauswertung überein. Alle Profile nach
Stilllegung entfernt, keine Forschungsprozesse mehr aktiv; beide Sitzungen gestoppt/konvertiert und
Supervisor beendet. Die normale Debug-App und sämtliche Runtimeprogramme bleiben unverändert.

36 Python-Diagnosetests sowie Format-/Diff-/Fakten-/Hashabgleich bestanden. Keine erneuten
unveränderten Anwendungstests oder unnötigen vollständigen Builds. Neue Arbeit dieser Stufe betrifft
Messung und Dokumentation, keine neue Runtime-/Auswertelogik.

## Aussagegrenzen

Die frühere HMAC-v2-Messung gehört zu einem anderen Paar und einer anderen Population. Kein kausaler
Vorher-/Nachher- oder Zählratenvergleich. Neue Änderungen bekommen die Netzabnahme nicht
automatisch. Transportmetadaten belegen keine HTTP-Bodies, Upload-/Inhaltsfreiheit oder vollständige
kausale Abnahme externer COM-/Shell-/DNS-/Brokerdienste. Lokale Kontrollen kalibrieren nicht jede
öffentliche Route. Keine neue Hörabnahme oder Aussage über kompromittierte Prozesse. Striktes
Datenschutzziel unverändert; normale Produktstarts bleiben unisoliert, Produkt-Q-01/Q-02 weiter
nicht bestanden. Produktablage, Updates/Migration und Produkt-/Packaging-Integration offen. Nächster
Scope: kombinierte assistive Wiederanlauf-/Fehlerprüfung und weitere Fensterendeprüfung des
gesicherten v3-Paars; deren bisherige Belege aus älteren Stufen werden nicht übernommen.
