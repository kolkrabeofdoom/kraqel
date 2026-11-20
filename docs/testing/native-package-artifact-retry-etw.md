# KQ-069: Frisches ETW des Brokers mit Retry und verwaltetem Kandidaten

Stand: 17. November 2026. Gezieltes mittleres Prüfniveau mit synthetischen Daten.
[Bereinigte Aggregate](native-package-artifact-retry-etw.json),
[Implementierung und vorherige Funktionsprüfung](native-package-artifact-retry.md). Striktes
Datenschutzziel unverändert; normale Produktintegration und Produkt-Q-01/Q-02 bleiben offen.

## Programme und Erfassung

Unveränderte gesicherte Programme aus `1e9c85f1cd4da25db7adb1871af5f2623c4ac0b1` unter
`%TEMP%/kraqel-package-retry-build-20261117-01/`:

- Broker-SHA-256 `c10b1c3fe22c1cd33c6ee33f0a1b72a639a00610ed5a1b86f560bcd19532d0bd`.
- Kandidat `285850152b96812cdda8c22ea64164d1018725f28d952aa58042b7d16aa474a9`.
- Forschungs-App `6e56a51ed2e913a878a7935e039bd69628a49be25f0fbab7d02fce3a72fe8c70`.

Der Kandidat enthält denselben Code plus statisches Overlay, keine beliebige
Release-/Datenmigration. Paket-/Apphashes und read-only `.kqmeta` beider Broker sowie der
verwalteten Kopie unabhängig geprüft. Kein neuer Build oder Änderung von Anwendung, Broker,
Auswerter, Capturehelfer oder Netzfähigkeiten.

Zwei getrennte kurze Sitzungen mit unverändertem `capture.ps1`, je vier eigenen lokalen
TCP-/UDP-IPv4/IPv6-Kontrollen. Erhöhter PowerShell-7-Supervisor, maximal zehn Minuten je Sitzung,
gezielt per STOP-Datei vorzeitig beendet. Keine Firewall-, Loopback-, ExecutionPolicy- oder
Domänenänderung; keine physische Netztrennung. Keine echten Projekte oder Provideraufrufe.

Referenzphase in `broker-ui`, kombinierter Retry-/Updatepfad in `native-control` mit vollständiger
privater Root-PID-/Image-Zuordnung. Alle 21 direkten Aufrufe sind erfasst, einschließlich fünf
Starts der tatsächlich verwalteten
`285850152b96812cdda8c22ea64164d1018725f28d952aa58042b7d16aa474a9.exe`. Keine automatische
Rootsuche. Start-PIDs/exakte EXE-Basisnamen beim Aufruf aufgezeichnet, Prozesshandles bis
Konvertierungsende gehalten. Unveränderte Kalibrierungs-, Verlust-, Zuordnungs- und
Lebenszeitguards. Ein separater chronologischer Kernel-Audit gleicht jeden Aufruf/Exitcode und beide
GUI-Eltern ab: ursprünglicher Broker vor Aktivierung, verwalteter Kandidat beim Wiederanlauf. Die
PowerShell-Teststeuerung liegt außerhalb dieser ausdrücklich ausgewählten nativen Bäume.

## Ausgeübter kombinierter Pfad

1. Neue Schema-4-Kontrollablage und abgeschlossene synthetische Importaufgabe anlegen/binden.
2. Vorbereitung am echten `package-partial`-Haltepunkt hart beenden. Authentisiertes Pending und
   genau 1.024 Paketbytes nach Wiederanlauf feststellen; Aktivierung der Teilkopie abweisen.
3. Ausdrücklich authentisiert abbrechen. Retry mit Dateisperre sowie mit altem Kopf abweisen;
   Fingerabdrücke von Kontrollrecords/Kopf/Schlüssel und Paketbytes bleiben unverändert.
4. Paket ohne Kompatibilitätsdeklaration vor Pending abweisen. Danach denselben vollständigen
   Kandidatenhash erfolgreich erneut vorbereiten; die authentisiert abgebrochene Teilkopie ersetzen.
5. Pending-Recovery und Aktivierung mit altem Kopf abweisen, dann korrekt aktivieren. Alter Broker
   ist anschließend gesperrt. Verwalteten neuen Broker für Status, Recovery, Rollback-Abwehr,
   Stilllegung und idempotenten Cleanup verwenden. Kein Kandidatenstart vor Aktivierung.

## Messergebnis

| Phase                                        |                Brokerprozesse | Apps | WebViews | Console-Hosts | Explizite Enden | Nicht-Loopback | Loopback |
| -------------------------------------------- | ----------------------------: | ---: | -------: | ------------: | --------------: | -------------: | -------: |
| Sitzungs-/Recoveryreferenz                   |                            19 |    6 |       36 |            25 |              86 |              0 |       36 |
| Retry, Aktivierung, verwalteter Wiederanlauf | 16 ursprünglich + 5 verwaltet |    2 |       12 |            23 |              58 |              0 |       12 |
| Zusammen                                     |                            40 |    8 |       48 |            48 |         **144** |          **0** |   **48** |

Beide Phasen gültig: acht von acht Kontrollen, keine verlorenen Events/Buffers, Zuordnungslücken
oder unbekannten Ziele. Loopback separat gezählt, je Phase gleich viele WebView-Ereignisse der IDs
13/29. Keine eigenen beobachteten Netzereignisse beider Broker.

Referenz: 18 Child-Controller plus Serienroot, elf Erfolge/sechs erwartete Abweisungen/zwei gezielte
Controllerabbrüche; vier Recoveries/acht Replay-Ereignisse. Kombinierter Pfad: 21 direkte Roots,
zwölf Erfolge/acht erwartete Abweisungen/ein Controllerabbruch. Alle 144 Enden aus expliziten
Kernel-Stop-Ereignissen, keine erfundenen Endzeiten. Acht native Root-Token-/Job-/Sperrbelege und je
sechs zugeordnete WebViews; zwei Apps enden nach Controllerabbruch mit 0, sechs kontrolliert mit 77.

Genaue Bereitschafts-/Stop-/Konvertierungszeiten in UTC und ETL-Größen stehen in den Aggregaten.
Beide Phasen am 17. November, Referenz bis `08:25:54.9321070Z`, kombinierter Pfad bis
`08:26:07.8774802Z`; Bereitschaft nach Kontrollen ist kein behaupteter exakter Providerstart.

## Unabhängiger Datenabgleich und Grenzen

MAC-Kette/Kopf/rootgebundene Autorität, Paketdeklarationen und Actorwechsel unabhängig geprüft. Im
kombinierten Pfad 13 Records: Vorbereitung/Abort/erneute Vorbereitung/Aktivierung, anschließend
Recovery/Stilllegung/Cleanup. Gespeicherte Partial-/Abort-Köpfe passen zu den Records; Teilkopie-SHA
passt unabhängig zu den ersten 1.024 Bytes des Kandidaten. Installationsschlüssel, App-/Sitzungs-
und Aufgabenbindung bleiben erhalten. Global drei verschiedene persistierte Aufgaben und genau zwei
legitime Projektkopien, keine Replaykopie. SQLite-Integrität/Fremdschlüssel/Unicode-Inhalte passen;
48 tatsächliche private Dateiverweigerungen mit capabilityfreien Root-Tokens. Drei erzeugte Profile
authentisiert entfernt, Cleanup-Wiederholung idempotent.

42 Diagnose-Tests bestanden; gezielte Format-/Diff-/Hash-/Faktenprüfung grün. Die frühere Prüfung
mit 56 Helfertests/Clippy bleibt an die unveränderten Programmbytes gebunden. Keine neue große
Matrix, Workspace-Gesamtprüfung oder Buildwiederholung. Erfassungen, Supervisor und Testprozesse
beendet. Echte Vorschauablage und dort autorisierte EXE unverändert. Roh-ETL/XML, PIDs/UUIDs/Pfade,
Schlüssel, Journale und synthetische Fixtures bleiben lokal; nur bereinigte Aggregate werden
veröffentlicht.

Reproduktion: exakte gesicherte Hashes prüfen, zwei frische kalibrierte Sitzungen, Referenzserie und
oben beschriebene 21 Aufrufe mit aufgezeichneten Start-PIDs/Images, Auswertung mit `broker-ui` bzw.
`native-control --root-map`, separater chronologischer Aufruf-/GUI- und MAC-/Datenabgleich. Keine
Übernahme oder Zurücksetzung bestehender Kontrollablagen.

Der Beleg gilt nur für diese Programme/Pfade/Lebenszeiten. Keine HTTP-/Inhalts-/Uploadbilanz,
lückenlose Vor-Ausführungs-Tokenattestierung aller Nachfahren oder vollständige externe
COM-/Shell-/DNS-/Dienstkausalität. Kein kausaler Ratenvergleich anderer Builds und keine
Produktfreigabe. Unbekannte Artefakte ohne authentisierten Abort und defekte Kontrollketten bleiben
gesperrt. Nächster begrenzter Scope: Prozessabbrüche zwischen Artefaktentfernung, Zielanlage und
Prepare-Record gezielt prüfen; normale Produktintegration und echte Paket-/Datenmigration offen.
