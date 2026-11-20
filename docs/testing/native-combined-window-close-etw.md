# KQ-069: Frisches ETW des kombinierten Fensterende-Paars

Stand: 16. November 2026. Zwei eigene gültige Messphasen des unveränderten gesicherten Paars aus der
[kombinierten Fensterendeprüfung](native-combined-window-close.md).
[Bereinigte Aggregate](native-combined-window-close-etw.json). Striktes Datenschutzziel unverändert;
keine Produktfreigabe, Produkt-Q-01/Q-02 nicht bestanden.

## Paar und Messverfahren

Runtimequelle `0727e332e645ad4fcd3760f795a3632230dc2fc5`, unveränderte App aus
`9f5a7ee8c56220bb90f10c4f304c9b0f01cd43ee`, Startticket und Brokerkette v3.

- App-SHA-256: `6e56a51ed2e913a878a7935e039bd69628a49be25f0fbab7d02fce3a72fe8c70`.
- Helfer-SHA-256: `1b5c0efa771f58a7e1274369d391b92a2c788aa1d64756e6ef4530b2f2555b4f`.

Hashes vor/nach Messung mit dem lokalen Manifest und an allen elf gemessenen Appkopien abgeglichen.
Keine Runtime-/Auswerteränderung, kein neuer Build; normale Debug-App unverändert. Vorige ETW-Belege
bleiben auf ihre damaligen Hashpaare und Lebenszeiten begrenzt.

Ein per UAC erhöhtes PowerShell-7-Supervisorprogramm führt zwei getrennte begrenzte Sitzungen mit
unverändertem `scripts/m3-network/capture.ps1` aus. Jede Sitzung hat eigene lokale TCP-/UDP-
IPv4/IPv6-Kontrollen, Verlustzähler und Start-/Stop-Belege. Keine Änderung an Firewall,
Loopback-Ausnahmen, ExecutionPolicy oder Domäneneinstellungen; keine physische Netztrennung.
Sämtliche Root-PIDs beim Start separat aufgezeichnet, Prozesshandles bis Messende gehalten.
Roh-ETL/XML, PIDs, UUIDs, lokale Pfade und Schlüssel bleiben ausschließlich lokal.

| Phase                  | Bereitschaft nach Kontrollen (UTC) | Sitzungsende (UTC) | Konvertiert (UTC) | ETL-Bytes |
| ---------------------- | ---------------------------------- | ------------------ | ----------------- | --------: |
| Automatik              | 11:54:19.7983416                   | 11:57:03.1022851   | 11:57:03.7870977  | 4.325.376 |
| Beide Fensterendepfade | 11:57:03.8266294                   | 11:58:09.6731155   | 11:58:10.1215109  | 2.752.512 |

Alle Zeiten am 16. November 2026. Bereitschaft ist kein behaupteter exakter Providerstart.

## Automatische Vergleichsserie

Unveränderter konservativer Modus `broker-ui`, einschließlich des vollständigen Controllerbaums:
neun Fälle mit sechs Crashpunkten, Abbruch, ungültigem Archiv und Teiltransfer; 72 App-Starts, 18
Broker-Kills, 27 Wiederanläufe, 54 abgewiesene Folgeaktionen und neun Stilllegungen bestanden. 109
Broker, 72 Apps, 432 WebViews, 181 Console-Hosts; alle 794 Prozessenden explizit erfasst.

Gültig: vier von vier Kontrollen, Nullverluste, keine Zuordnungslücken oder unbekannten Ziele. **0
beobachtete Nicht-Loopback-Ereignisse** im zugeordneten Serienbaum; getrennt 432 Loopback-
Ereignisse, je 216 der IDs 13 und 29. Brokerenden: 18 × 86, 54 × 1, 37 × 0. Appenden: 18 × 0 durch
OS-Kill-on-close und 54 × 77 durch kontrolliertes Testende; keine regulären Appabschlüsse daraus
ableiten. Unabhängiger MAC-/Ketten-/Datenabgleich: fünf unterbrochene, zwei fehlgeschlagene und elf
erfolgreiche Aufträge, genau elf legitime Kopien, kein Stagingrest oder Replay-Duplikat.

## Beide manuell bedienten Fensterendepfade

Die prüfende Person bestätigt im ersten Lauf Hauptfensterschluss bei offenem nativen Dateidialog
ohne Auswahl und automatisches Mitbeenden des Dialogs. Im zweiten Lauf native Auswahl ausschließlich
der vorbereiteten `input.kraqel` und Hauptfensterschluss während „Testarchiv wird geprüft …“. Sender
und unabhängig gesicherter Empfängerzähler belegen genau 1.024 von 11.915 Bytes. Kein nur langsamer
oder bereits abgeschlossener Import als Teiltransfer ausgegeben.

Beide initialen App-/Brokerenden regulär 0; `CANCELLED` ohne Receipt, Auswahlthreads gejoint und
Jobs leer. Dialoglauf: sieben gezielte Schließnachrichten als Polling-Wiederholungen an den eigenen
Auswahlthread, keine sieben Dialoge oder behauptete Millisekundenlatenz. Teiltransferlauf: keine
weitere Dialog-Schließnachricht erforderlich.

Je Fall danach automatisch: authentisierte Recovery des verbrauchten Auftrags ohne Kopie,
Replay-/Altanfrage-Abwehr ohne erneute Auswahl oder Journaländerung, MAC-Abschlussbeleg,
ausdrücklich neuer Fixture-Auftrag mit anderer Kennung und absichtlichem Brokerverlust 86, zweifache
Wiederherstellung mit Altanfrage-Abwehr. Genau eine legitime Kopie je Fall. Der im Funktionsnachweis
separat geprüfte vorzeitige Folgeauftrag wird in dieser manuellen Messphase nicht erneut ausgeführt;
54 abgewiesene Folgeaktionen sind in der automatischen Phase erfasst.

Unveränderter Modus `isolation-ui` mit allen zehn separat gestarteten GUI-Controllerroots: zehn
Broker, 16 Apps, 96 WebViews, 26 Console-Hosts, alle 148 Prozessenden explizit. Vier eigene
Kontrollen, Nullverluste/Zuordnungslücken/unklare Ziele. **0 beobachtete
Nicht-Loopback-Ereignisse**; getrennt 104 Loopback-Ereignisse, je 52 der IDs 13/29. Brokerenden: 8 ×
0 und 2 × 86. Appenden: zwei reguläre Hauptfensterschlüsse 0, zwei OS-Kill-on-close-Enden 0 und
zwölf kontrollierte Recovery-/Altanfrage-Testenden 77. Numerisch gleicher Exitcode 0 bedeutet hier
unterschiedliche Endearten. Vier verbrauchte Aufträge, zwei dauerhaft `failed:CANCELLED`, zwei
`succeeded` und genau zwei legitime Kopien; unabhängiger Daten-/MAC-Audit bestanden.

Beide manuellen Stilllegungen mit entferntem Profil erfolgen bewusst nach Messende und sind keine
zusätzlichen gemessenen Roots. Derselbe Stilllegungspfad ist neunmal in der Automatik erfasst.

## Gesamtprüfung, Reproduktion und Grenzen

Elf Rootstarts, 119 Broker, 88 Apps, 528 WebViews, 207 Console-Hosts; alle **942 Enden explizit**.
Unabhängiger chronologischer Lebenszeitaudit stimmt mit der Primärauswertung überein, ordnet alle 88
nativen App-Startbelege ihren Brokereltern zu und bestätigt sechs WebViews je App. Keine Ersetzung
von Lebenszeiten durch PID-/Namensmengen, auch bei PID-Wiederverwendung.

Acht von acht Kontrollen, Nullverluste, keine Zuordnungslücken oder unbekannten Ziele. Insgesamt **0
beobachtete Nicht-Loopback- und 536 getrennte Loopback-Ereignisse** in den erfassten Bäumen. Keine
eigenen beobachteten Netzereignisse der Broker im ausgewerteten Ausschnitt. Dies ist kein kausaler
Vorher-/Nachher- oder Ratenvergleich mit dem früheren Paar.

Separate Nur-Lese-Prüfungen bestätigen Grant-/Abschluss-HMACs, kanonische Wurzel-/Programmhash-
Bindung, Vorgängerkette, 88 exakte AppContainer-Identitäten ohne Capabilities und verweigerte
private Brokerdateizugriffe. SQLite-Integrität/Fremdschlüssel, Receipt-/Archivhash/Bytezahl, zwei
Dokumente je erfolgreicher Kopie, Unicode und leeres Staging stimmen. Global 22 unterschiedliche
verbrauchte Aufträge: fünf unterbrochen, vier fehlgeschlagen, 13 erfolgreich. Genau 13 global
unterschiedliche legitime Kopien, keine durch Replay erzeugte Kopie. Alle elf Profile entfernt;
Sitzungen, Supervisor und Forschungsprozesse beendet. 36 Diagnose-Tests und
Format-/Diff-/Hash-/Faktenprüfung bestanden; keine unnötigen vollständigen Builds oder Wiederholung
unveränderter Anwendungstests.

Reproduktion: obige Hashes prüfen, zwei frische Erfassungen starten und Bereitschaft jeweils
abwarten. Automatisch `ui-combined-series` ausführen und unter `broker-ui` auswerten. Manuell je
frische Neustartwurzel mit `ui-broker-live-seed … completed closeDialog|closePartial`, jeweiliger
Hauptfensterschluss, Recovery `first`, explizites `ui-broker-live-next`, Recovery `first`/`second`.
Alle Root-Handles bis Messende halten und unter `isolation-ui` mit sämtlichen aufgezeichneten
Root-PIDs auswerten; danach `retire`. Teiltransferzähler vor Folgeimport sichern, MAC-/Daten- und
chronologischen Lebenszeitaudit getrennt durchführen. Keine alten Schlüssel/Wurzeln übertragen.

ETW attestiert Transportmetadaten dieser Lebenszeiten und Pfade, keine HTTP-Inhalte, Uploads oder
vollständige kausale Zuordnung externer COM-/Shell-/DNS-/Broker. Weitere allgemeine Fensterende-/
F08-Randfälle und Produktintegration bleiben offen. Keine Runtime-/Domänenausnahme vom strikten
Datenschutzziel. Nächster Scope: geschützte dauerhafte Produktablage und authentisierter
Update-/Migrationspfad; weitere allgemeine F08-/Dienstgrenzen separat prüfen.
