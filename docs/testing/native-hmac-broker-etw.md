# KQ-069: Frischer ETW-Netznachweis des lokalen HMAC-Brokers

16. November 2026. Eigener gültiger Messlauf exakt des gesicherten neuen
    [HMAC-Paars](native-hmac-broker.md), [Vertrag v2](../data-format/native-broker-recovery-v2.md).
    [Bereinigte Aggregate](native-hmac-broker-etw.json). Keine Übertragung älterer Netzbelege.

Brokerquelle `b2e181134a9919f961188be1712bca9cae26c187`, SHA-256
`28fde769f00be412dcc755331f1a9424a39dac59e9a2f2de44dbc724e84825dd`. Unveränderte App aus
`9d87e20d4a2526885c213bd186e4c22573d80744`, SHA-256
`4db1e731e6aee50fe01e9e368d1ded61ac61f3f9dc3e2238d509cee600098e1b`. Hashes vor/nach Messung und an
allen sechs tatsächlich ausgeführten Appkopien bestätigt. Separates Paar unter
`%TEMP%\kraqel-kq069-hmac-broker-build-20261116-01`; keine Runtimeänderung oder neuer Build nach
Messung.

## Erfassung und Aussage

Vorhandenes `capture.ps1` mit dem vorhandenen PowerShell-7-Host gestartet. Windows-UAC durch die
prüfende Person bestätigt; Bereitschaft und Positivkontrollen vor Serienstart vorhanden. Eigene
begrenzte Sitzung, keine Firewall-/Loopback-/ExecutionPolicy-/Domänenrichtlinienänderung oder
physische Netztrennung. Roh-ETL/XML, PIDs, UUIDs, Sessionpfade und Schlüssel bleiben ausschließlich
lokal. Öffentlich nur datenarme Aggregate.

Bereitschaft nach den bereits erfassten Kontrollen: `2026-11-16 09:56:39.1748257 UTC`, Sitzungsende
`09:58:21.2972761 UTC`, Konvertierung `09:58:21.6878871 UTC`. Die Bereitschaft ist kein behaupteter
exakter Providerstart. ETL: 2.555.904 Bytes. Nach Serienende native Daten geprüft, eigene STOP-Datei
angelegt; Sitzung erfolgreich beendet/konvertiert, Erfassungsprozess beendet.

**Gültige Messung: vier von vier lokalen TCP-/UDP-IPv4/IPv6-Kontrollen, null
Ereignis-/Pufferverluste, keine Zuordnungslücken oder unbekannten Ziele. 0 beobachtete
Netzereignisse außerhalb Loopback im zugeordneten Gesamtbaum.** Getrennt 180
WebView2-Loopback-Ereignisse, je 90 der IDs 13 und 29. Die 103 Brokerlebenszeiten haben im
ausgewerteten Ausschnitt keine eigenen Netzereignisse.

## Vollständige Prozess- und Funktionsabdeckung

Der beim Start separat festgehaltene Serienprozess ist Root für `--mode broker-ui`. Die Auswertung
umfasst den gesamten Baum, einschließlich aller abgewiesenen Brokerstarts:

| Prozessklasse  | Lebenszeiten | Prüfung                                                                                          |
| -------------- | ------------ | ------------------------------------------------------------------------------------------------ |
| Brokerprogramm | 103          | Ein Serienprozess, sechs Seed-Crashes, zwölf Wiederanläufe, sechs Stilllegungen, 78 Ablehnungen  |
| App            | 30           | Alle Startbelege in tatsächlicher Reihenfolge und mit zugehörigen Brokerlebenszeiten abgeglichen |
| WebView2       | 180          | Genau sechs je App-Generation                                                                    |
| Console-Host   | 133          | Als Nachfahren vollständig einbezogen                                                            |
| Gesamt         | 446          | Sämtliche Enden explizit im Kernelprotokoll vorhanden                                            |

Kernelenden bestätigen Brokercodes 6 × 86, 78 × 1 und 19 × 0. Die sechs Seed-Apps enden beim
OS-Kill-on-close mit Code 0; das ist **kein geordneter Appabschluss**. Die übrigen 24 Appenden sind
kontrollierte Jobbeendigungen mit Code 77. Zusätzlich zur Standardauswertung wurde die Reihenfolge
aller 30 Startbelege pro Fall und ihrer drei Brokereltern unabhängig rekonstruiert.
PID-Wiederverwendung tritt tatsächlich auf; eine erste bloße Eindeutigkeitsprüfung über PID-Werte
war dafür ungeeignet und wurde nicht als Nachweis akzeptiert. Der erfolgreiche Abgleich nutzt
getrennte Lebenszeiten, tatsächliche Reihenfolge, Elternzuordnung und Exitcodes. Keine Übernahme
alter PID-/Namensmengen als vermeintliche Messabdeckung.

Native Serie im Messlauf bestanden: sechs Controller-Kills, zwölf neue Wiederanlaufprozesse, 78
Ablehnungsfälle, unveränderte exakte SID/null Capabilities vor jedem Resume und 180 tatsächlich
verweigerte Dateiöffnungen unter dem App-Token. Keine neue Importfreigabe und kein zweites Projekt.
Eigener Nur-Lese-Audit bestätigt MACs mit Pythons unabhängiger Implementierung, beide Hashbindungen,
sechs Journalzustände, Kopienzahlen, SQLite/Unicode/Receipt-/Archivhash/Fremdschlüssel und leeres
Staging. Vier Aufträge `interrupted` mit null Kopien, zwei `succeeded` mit genau je einer Kopie.
Alle Profile nach Stilllegung entfernt; keine Forschungs-App-/Brokerprozesse mehr aktiv.

## Einordnung und offene Arbeit

Die DPAPI-Domänengrenze wird durch Entfernung der Primitive aus diesem Brokerpfad aufgelöst; kein
Domänen-/Kontowiederherstellungsdienst wird für die Authority aufgerufen. Code-/Backendprüfung,
statische PE-Importprüfung und tatsächlicher neuer Messlauf ergänzen sich. Die MAC- und
Klartextschlüsselverwahrung ist ausdrücklich an Windows-DACL und Prozessgrenze gebunden; kein Schutz
gegen kompromittiertes Konto/Administrationsrechte/Schlüsselkopie behaupten.

Ältere v1-/v2-Netzbelege gelten weiter nur für ihre jeweiligen Paare. Der frühere DPAPI-Broker hatte
keinen eigenen ETW-Nachweis; deshalb kein behaupteter kausaler Vorher-/Nachher-Netzvergleich. Auch
Zählraten unterschiedlicher Serien/Populationen sind nicht direkt vergleichbar.

26 aktive Helfer-/Transporttests, Helfer-Clippy/Build, `pnpm check` und 36 Python-Diagnosetests
bestanden. Keine neue Hörabnahme, keine Änderung der App-/Rendereroberfläche. Transportmetadaten
belegen keine HTTP-Bodies, Upload-/Inhaltsfreiheit oder vollständige kausale Abnahme externer
COM-/Shell-/DNS-/Broker. Lokale Kontrollen kalibrieren nicht jede öffentliche Route. Nächster Scope:
kombinierter Live-/Neustartpfad mit neuen authentisierten Aufträgen, geschützte Produktablage und
authentisierter Update-/Migrationspfad; externe Grenzen und Produkt-/Packaging- Integration bleiben
offen. Normale Starts bleiben unisoliert. Striktes Datenschutzziel ohne Runtime-Ausnahme
unverändert; #134 und Produkt-Q-01/Q-02 nicht bestanden.
