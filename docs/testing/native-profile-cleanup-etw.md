# KQ-069: Gezielter frischer ETW-Vergleich des Cleanup-Stands

Stand: 17. November 2026. Mittleres Prüfniveau, synthetische Daten; striktes Datenschutzziel
unverändert. [Bereinigte Aggregate](native-profile-cleanup-etw.json),
[Cleanup-Funktionsnachweis](native-profile-cleanup.md). Keine Produkt-Q-01/Q-02-Freigabe.

Die **unveränderten gesicherten Programme** aus `1e1ce619b2b39904cba9d2e7eaebf811fc03f199` liegen
unter `%TEMP%/kraqel-cleanup-build-20261117-02/`:

- Broker-SHA-256: `b2c4f21f080337c231fcb54a71dae73777b71923f73c7961c37678da13cd7dbd`.
- Forschungs-App-SHA-256: `6e56a51ed2e913a878a7935e039bd69628a49be25f0fbab7d02fce3a72fe8c70`.

Kein neuer Rust-/Appbuild oder Änderung an Anwendung, Broker, Dateiformat oder Netzfähigkeiten. Zwei
kurze getrennte ETW-Erfassungen mit unverändertem `capture.ps1`, je vier lokalen
TCP-/UDP-IPv4/IPv6-Kontrollen. Ein per UAC erhöhter PowerShell-7-Supervisor, maximal zehn Minuten je
Sitzung, nach Prüfende gezielt mit eigener STOP-Datei beendet. Keine Firewall-, Loopback-,
ExecutionPolicy- oder Domänenänderung; keine physische Netztrennung.

## Auswertungsgrenze

Die bisherige Sitzungs-/Recoveryserie wird mit `broker-ui` gemessen. Die Cleanup-Prüfung besteht aus
**19 expliziten nativen Kontrollaufrufen**; nur zwei davon starten tatsächlich eine GUI. Dafür
ergänzt der Auswerter den benannten Modus `native-control`: festes Root-Image
`kraqel-isolation-probe.exe`, jede angegebene Root muss vorhanden/eindeutig/vollständig erfasst
sein. Kein GUI-Zwang für einen reinen Status-/Cleanup-Aufruf. Die bisherigen GUI-/Worker-Modi
behalten ihre Strukturprüfungen; Kalibrierung, Verlust-, Zuordnungs- und Lebenszeitprüfungen bleiben
für alle Modi bestehen. Leere Rootlisten werden abgewiesen.

Die 19 tatsächlichen Prozessstarts werden separat aufgezeichnet und ihre Handles bis Messende
gehalten. Ein unabhängiger chronologischer Kernel-Audit gleicht **jeden** Kontrollaufruf und dessen
Exitcode sowie die erwarteten zwei GUIs ab; kein Weglassen fensterloser Cleanup-Prozesse.
PowerShell-Teststeuerung ist außerhalb dieser ausdrücklich ausgewählten Produkt-/Kontrollbäume. Dies
ist keine Aussage über vollständige externe COM-/Shell-/DNS-/Dienstkausalität.

## Messung

| Phase                             | Bereitschaft nach Kontrollen (UTC) | Sitzungsende (UTC) | Konvertiert (UTC) | ETL-Bytes |
| --------------------------------- | ---------------------------------- | ------------------ | ----------------- | --------: |
| Bisherige Sitzungs-/Recoveryserie | 07:04:43.1260133                   | 07:05:04.5205935   | 07:05:04.6921336  | 1.179.648 |
| Zwei gezielte Cleanup-Fälle       | 07:05:04.7370099                   | 07:05:15.8769217   | 07:05:16.0391806  | 1.179.648 |

Alle Zeiten am 17. November 2026; Bereitschaft ist kein behaupteter exakter Providerstart.

- **Vergleichsserie:** ein Serienroot mit 18 Child-Controllern, sechs Apps, 36 WebViews, 25
  Console-Hosts. Zwei Controllerabbrüche, vier Recoveries/acht Replay-Ereignisse. Alle **86
  Prozessenden explizit**, vier Kontrollen, Nullverluste/Zuordnungslücken/unklare Ziele. **0
  Nicht-Loopback-**, getrennt **36 Loopback-Ereignisse** (je 18 der IDs 13/29).
- **Cleanup:** alle 19 nativen Root-Aufrufe erfasst; 13 Erfolge, vier erwartete Abweisungen, zwei
  gezielte Controllerabbrüche. Zwei Apps, zwölf WebViews, 21 Console-Hosts. Vorbereitung ohne
  Profilerzeugung sowie Cleanup nach unterbrochener Stilllegung geprüft; Wiederholung idempotent,
  alter Kopf und Recovery nach Stilllegung abgewiesen. Alle **54 Prozessenden explizit**, vier
  eigene Kontrollen, Nullverluste/Zuordnungslücken/unklare Ziele. **0 Nicht-Loopback-**, getrennt
  **zwölf Loopback-Ereignisse** (je sechs der IDs 13/29).

Zusammen **140 explizite Prozessenden**: 38 Controller, acht Apps, 48 WebViews und 46 Console-Hosts.
Acht von acht Kontrollen, keine Messverluste. **0 beobachtete Nicht-Loopback- und 48 getrennte
Loopback-Ereignisse** im erfassten Baum; keine eigenen beobachteten Netzereignisse des Brokers. Zwei
Apps enden nach Controllerabbruch/Jobschluss mit 0, sechs kontrolliert mit 77. Je App sechs
zugeordnete WebViews; alle acht nativen Root-Token-/Job-/Sperrbelege geprüft. Keine lückenlose
Vor-Ausführungs-Tokenattestierung aller Nachfahren und keine HTTP-/Inhalts-/Uploadbilanz.

Die frischen Daten-/MAC-Audits bestätigen Schema-3-Kompatibilität und Schema-4-Cleanup: globale
Verschiedenheit von drei persistierten Aufgaben und genau zwei legitimen Projektkopien, kein
Replay-Duplikat. Die vorbereitete Bindung ohne Journal zählt nicht als Importaufgabe. 48 echte
private Dateiverweigerungen unter eingeschränkten Root-Tokens. Kontrollkette, Kopf, kanonische
Wurzel/Apphash, idempotenter Cleanup und erhaltene SQLite-/Unicode-Daten geprüft. Drei tatsächlich
erzeugte Profile entfernt; Nichtvorhandensein des nie erzeugten vierten Profils bestätigt.

## Prüfung und Übergabe

39 Diagnose-Tests bestanden, davon drei neue für `native-control` einschließlich fehlender Roots,
Kalibrierung/Verlust/Prozessende und Zuordnung öffentlicher Netzereignisse. Format-/Diff-/Hash- und
Faktenabgleich grün. Bewusst keine erneute große Crashmatrix, Workspace-Gesamtprüfung oder
Buildwiederholung. Die frühere Helferprüfung bleibt am unveränderten Programmstand gebunden.

Erfassungen, Supervisor und Testprozesse beendet. Die echte Vorschauablage und ihre autorisierte EXE
wurden nicht verwendet oder verändert. Synthetische Fixtures/rohe ETL/XML/PIDs/UUIDs/Pfade und
Schlüssel bleiben lokal; nur bereinigte Aggregate veröffentlicht. Reproduktion: Hashes prüfen, zwei
frische kalibrierte Sitzungen durchführen, alle tatsächlichen Root-PIDs/Handles aufzeichnen,
`broker-ui` beziehungsweise `native-control` auswerten und erwartete Aufrufe/GUI-Lebenszeiten sowie
Daten/MACs getrennt abgleichen. Keine bestehende Kontrollablage kopieren oder zurücksetzen.

Der Beleg gilt ausschließlich für diese Programme, Pfade und Lebenszeiten; keine universelle
Netzstille oder Produktfreigabe und kein kausaler Ratenvergleich mit früheren Builds. Striktes
Datenschutzziel unverändert. Nächster Scope: lokale Update-/Paketkompatibilität für Kontrollschema 4
unter derselben Sperre; normale Produktintegration, weitere Crashpunkte und echte Datenmigration
bleiben offen.
