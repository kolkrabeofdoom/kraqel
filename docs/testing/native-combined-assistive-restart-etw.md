# KQ-069: Tastaturprüfung und frisches ETW des manuellen Brokerpaars

Stand: 16. November 2026. Windows-Forschungsfeature, ausschließlich synthetische Testarchive.
[Bereinigte Aggregate](native-combined-assistive-restart-etw.json),
[vorheriger assistiver Funktionsnachweis](native-combined-assistive-restart.md). Striktes
Datenschutzziel unverändert; keine Produktfreigabe, Produkt-Q-01/Q-02 nicht bestanden.

## Unverändertes gesichertes Paar

Helferquelle `6b6997a212016270b6bf99381169a65b1145733d`, unveränderte App aus
`9f5a7ee8c56220bb90f10c4f304c9b0f01cd43ee`; Startticket und Brokerkette v3.

- App-SHA-256: `6e56a51ed2e913a878a7935e039bd69628a49be25f0fbab7d02fce3a72fe8c70`.
- Helfer-SHA-256: `1be2097ca854417157f802bbb4227d3c5fbb84995af1f14a7982aab63c115b52`.

Hashes vor/nach Messung mit dem lokalen Manifest und allen zwölf gemessenen Appkopien abgeglichen.
Kein Runtime-/Auswerterumbau und kein neuer Appbuild für diese Stufe. Normale Debug-App bleibt
unverändert. Das frühere ETW gehört zum älteren Helferhash und wird nicht übertragen.

## Reine Tastaturprüfung

Vor Messbeginn eigener frischer Fixture-Auftrag und manueller Wiederanlauf: Die prüfende Person
bestätigt ausdrücklich ausschließliche Tastaturbedienung mit Alt+Tab, Tab/Enter und Alt+F4.
Wiederherstellung von zwei Dokumenten ohne neue Kopie und Verbrauchshinweis wurden angesagt; Fokus
auf „Erneuten Aufruf prüfen“ bestätigt. Vor Schließen nativer Replay-Beleg und Fokusdiagnose
geprüft, danach regulärer Exitcode 0, gesperrte Altanfrage und Stilllegung bestätigt. Dieser
zusätzliche Vorlauf zählt nicht zur ETW-Population.

Während der frischen manuellen ETW-Phase wurden Unterbrechung, Abschluss, synthetischer Abbruch und
ungültiges Archiv ebenfalls ausdrücklich ausschließlich per Tastatur geprüft und bestätigt. Jeweils
richtige Zustandsansage, Fokus auf Replay und angesagter Verbrauchshinweis nach Enter; nativer
Replay und Aktion/Fokus vor dem Schließen gegengeprüft. Vier reguläre Enden mit Code 0.

Die Eingabemethode ist menschlicher Nachweis: Der separate Vorlauf enthält zwei Navigationseinträge
und ein Pointer-Ereignis. Die gemessenen Fälle enthalten jeweils ein Pointer-Ereignis; nur der
Unterbrechungsfall enthält zusätzlich einen Navigationseintrag. Das erlaubt keine unabhängige
Bestimmung der physischen Eingabemethode oder ihrer Ursache. Keine Audioaufzeichnung, Tastencodes
oder globalen Hooks. Die früheren zwei abgelehnten Erstversuche bleiben dokumentiert und werden
nicht nachträglich als bestanden gezählt.

## Zwei eigene kalibrierte Messphasen

Unverändertes `scripts/m3-network/capture.ps1` unter einem per UAC erhöhten PowerShell-7-
Supervisor; jede Phase mit eigenen lokalen TCP-/UDP-IPv4/IPv6-Kontrollen. Keine Änderung an
Firewall, Loopback-Ausnahmen, ExecutionPolicy oder Domäneneinstellungen; keine physische
Netztrennung. Jede Root-PID beim Start festgehalten, zugehöriger Prozesshandle bis Messende
gehalten.

| Phase                  | Bereitschaft nach Kontrollen (UTC) | Sitzungsende (UTC) | Konvertiert (UTC) | ETL-Bytes |
| ---------------------- | ---------------------------------- | ------------------ | ----------------- | --------: |
| Automatik              | 11:28:14.1030531                   | 11:31:02.1248828   | 11:31:02.8941759  | 5.373.952 |
| Manuelle Wiederanläufe | 11:31:02.9425362                   | 11:35:48.7507616   | 11:35:49.9490171  | 7.602.176 |

Alle Zeiten am 16. November 2026. Bereitschaft ist kein behaupteter genauer Providerstart.

Automatik unter unverändertem Auswertemodus `broker-ui`: neun Fälle einschließlich sechs
Crashpunkten, Abbruch, ungültiger Datei und Teiltransfer; 72 Apps, 18 Controller-Kills, 27
Wiederanläufe, 54 abgewiesene Folgeaktionen und neun Stilllegungen bestanden. 109 Broker, 432
WebViews und 181 Console-Hosts, alle 794 Prozessenden explizit. Vier Kontrollen,
Nullverluste/Zuordnungslücken/unklare Ziele. 0 Nicht-Loopback- und getrennt 432 Loopback-Ereignisse
(je 216 der IDs 13/29). Brokerenden: 18 × 86, 54 × 1, 37 × 0. Appenden: 18 × 0 durch
OS-Kill-on-close und 54 × 77 durch kontrolliertes Testende; diese sind keine geordneten
Appabschlüsse.

Manuell unter unverändertem Modus `isolation-ui`: echte native Auswahl der vorbereiteten
`input.kraqel`, absichtlicher Brokerverlust bei vorbereitetem Import, manueller Wiederanlauf,
ausdrücklich neuer Fixture-Auftrag mit weiterem Brokerverlust und manuellem Wiederanlauf. Danach je
ein frischer synthetischer Abbruch-/Fehlerfall mit Brokerverlust und manueller Recovery. Alle vier
Replay-Aufrufe und Schließvorgänge menschlich bestätigt und nativ gegengeprüft; anschließend jeweils
automatische Altanfrage gesperrt, Journal unverändert, MAC-Abschlussbeleg. Abbruch ist synthetischer
Senderabbruch, keine manuelle Dateidialog-Abbruchprüfung.

Acht Broker, zwölf Apps, 72 WebViews, 20 Console-Hosts; alle 112 Enden explizit. Vier eigene
Kontrollen, Nullverluste/Zuordnungslücken/unklare Ziele. 0 Nicht-Loopback- und getrennt 82
Loopback-Ereignisse (je 41 der IDs 13/29). Brokerenden: 4 × 86, 4 × 0. Appenden: vier reguläre
manuelle Enden 0, vier OS-Kill-on-close-Enden 0, vier kontrollierte Altanfrage-Enden 77. Diese
unterschiedlichen Bedeutungen des numerischen Codes 0 bleiben getrennt.

## Unabhängiger Abgleich und Grenzen

Chronologischer Lebenszeitaudit bestätigt alle 84 App-Startbelege, ihre Brokereltern und genau sechs
WebViews je App; PID-Wiederverwendung wird nicht durch bloße PID-Mengen ersetzt. Insgesamt 117
Broker, 84 Apps, 504 WebViews, 201 Console-Hosts, alle 906 Enden explizit. Acht von acht Kontrollen,
keine Verluste oder Zuordnungslücken: **0 beobachtete Nicht-Loopback-Ereignisse und 514 getrennte
Loopback-Ereignisse** in den zugeordneten Bäumen. Die Zählwerte sind keine kausale Vorher-/Nachher-
oder Ratenmessung gegenüber dem alten Paar.

Separate Nur-Lese-Audits bestätigen HMACs, Vorgängerkette, kanonische Wurzel-/Programmhashbindung,
84 exakte AppContainer-Identitäten ohne Capabilities und die tatsächlichen privaten
Brokerdateiverweigerungen. SQLite-Integrität/Fremdschlüssel, Receipt-/Archivhash/Bytezahl, Unicode
und leeres Staging stimmen. 22 global unterschiedliche verbrauchte Aufträge: sechs unterbrochen,
zwölf erfolgreich, vier fehlgeschlagen; zwölf global unterschiedliche legitime Kopien, keine neue
Kopie durch Replay. Automatik: elf Kopien; manuelle Messung: eine Kopie.

Alle zwölf gemessenen Profile entfernt; die drei manuellen Stilllegungen bewusst nach Messende,
nicht als gemessene Roots ausgegeben. Derselbe Stilllegungspfad ist neunmal in der Automatik
erfasst. Zusätzlicher Tastaturvorlauf ebenfalls stillgelegt. Erfassungen, Supervisor und
Forschungsprozesse beendet. Roh-ETL/XML, PIDs, lokale Pfade, UUIDs und Schlüssel bleiben lokal. 36
Diagnose-Tests sowie Format-/Diff-/Hash-/Faktenprüfung bestanden; keine unnötige Wiederholung
unveränderter Anwendungstests oder vollständiger Builds.

Reproduktion: gesichertes Paar anhand obiger Hashes prüfen, zwei frische getrennte Erfassungen
starten. Erst nach jeweiliger Bereitschaft `ui-combined-series` beziehungsweise frische
`ui-broker-live-seed … prepared native`, `ui-broker-recover … manual`, `ui-broker-live-next` und
manuelle Recovery ausführen. Zusätzliche frische `completed cancel|invalid`-Seeds mit manueller
Recovery; echten Replay vor Fensterende prüfen. Alle Root-Handles bis Messende halten. Analyse mit
`analyze.py --mode broker-ui|isolation-ui` und sämtlichen separat aufgezeichneten Root-PIDs;
MAC-/Daten- und chronologische Lebenszeitaudits getrennt durchführen. Niemals Schlüssel oder alte
Testwurzeln zwischen Rechnern übertragen oder aus Diagnose-JSON neue Autorität ableiten.

ETW belegt Transportmetadaten der erfassten Bäume und ausgeübten Pfade, keine HTTP-Inhalte, Uploads
oder vollständige kausale Zuordnung externer COM-/Shell-/DNS-/Broker. Weitere allgemeine
Fensterende-/F08-Prüfungen, Produktablage, Updates/Migration und Produktintegration bleiben offen.
Keine Runtime-/Domänenausnahme vom strikten Datenschutzziel. Nächster Scope: weitere kombinierte
Fensterendeprüfung; anschließend geschützte dauerhafte Produktablage und Update-/Migrationspfad.
