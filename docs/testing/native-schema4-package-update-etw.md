# KQ-069: Frisches ETW beider Schema-4-Brokerprogramme

Stand: 17. November 2026. Gezieltes mittleres Prüfniveau mit synthetischen Daten.
[Bereinigte Aggregate](native-schema4-package-update-etw.json),
[funktionaler Paket-/Updatevertrag](native-schema4-package-update.md). Striktes Datenschutzziel
unverändert; normale Produktintegration und Produkt-Q-01/Q-02 bleiben offen.

## Unveränderte Programme und Messgrenze

Programmquellen: `ef815ea8314bb23db4b502948e5b1bc63151c6f8`. Gesicherte Dateien unter
`%TEMP%/kraqel-update4-build-20261117-01/`, kein neuer Rust-/Appbuild:

- Ursprünglicher Broker: `45542384ce42b73450453dce2f5a40b635b7b8552055de25d60c147fc21dd360`.
- Verwalteter Kandidat: `6748900defbbdb8df5e8e5ec0e611091cd1e6656880381a18e43e8755510bf90`.
- Gebundene Forschungs-App: `6e56a51ed2e913a878a7935e039bd69628a49be25f0fbab7d02fce3a72fe8c70`.

Der Kandidat enthält denselben Code plus statisches synthetisches Overlay; anderer Hash, kein
Nachweis beliebiger Release-/Appdatenmigration. Der Apphash bleibt gleich. SHA und `.kqmeta` beider
Broker sowie der geschützten Kandidatenkopie sind unabhängig geprüft.

Zwei kurze frische Sitzungen mit unverändertem `capture.ps1`, je vier eigenen lokalen
TCP-/UDP-IPv4/IPv6-Kontrollen. Erhöhter PowerShell-7-Supervisor, höchstens zehn Minuten je Sitzung,
gezielt per STOP-Datei vorzeitig beendet. Keine Firewall-, Loopback-, ExecutionPolicy- oder
Domänenänderung; keine physische Netztrennung. Keine echten Projekte oder Provideraufrufe.

Die Vergleichsserie verwendet `broker-ui`. Die Updatephase verwendet `native-control`, jetzt mit
expliziter privater `--root-map`: jede beim Start festgehaltene PID ist an ihren erwarteten
EXE-Basisnamen gebunden. Erlaubt sind ausschließlich der feste Brokername und exakt 64 kleine
Hexzeichen plus `.exe`. Alle angegebenen Roots müssen genau einmal in der Zuordnung vorkommen. Kein
automatisches Einsammeln hashbenannter Prozesse. Falscher Name, fehlende Root, Verlust,
Kalibrierungs-/Zuordnungs- oder Lebenszeitlücke bleiben ungültig. Andere Modi erlauben diese
Zuordnung nicht; ihre Strukturprüfungen bleiben erhalten.

Alle 15 nativen Aufrufe sind damit erfasst, einschließlich **fünf Starts der tatsächlich verwalteten
`6748900defbbdb8df5e8e5ec0e611091cd1e6656880381a18e43e8755510bf90.exe`**. Die aufgezeichneten
Prozesshandles bleiben bis Konvertierungsende gehalten. Ein separater chronologischer Kernel-Audit
prüft jeden erwarteten Aufruf, exakte Images/Exitcodes, explizite Enden und beide GUI-Eltern:
ursprünglicher Broker vor Aktivierung, verwalteter Kandidat beim Wiederanlauf. Die
PowerShell-Teststeuerung liegt außerhalb dieser ausdrücklich ausgewählten nativen Bäume.

## Messergebnis

Die genauen UTC-Zeitstempel und ETL-Größen stehen in den Aggregaten. Am 17. November dauerte die
Referenzphase ab Bereitschaft nach Kontrollen ungefähr 22 Sekunden, die Updatephase elf Sekunden.
Bereitschaft ist kein behaupteter exakter Providerstart.

| Phase                   |                Brokerprozesse | Apps | WebViews | Console-Hosts | Explizite Enden | Nicht-Loopback | Loopback |
| ----------------------- | ----------------------------: | ---: | -------: | ------------: | --------------: | -------------: | -------: |
| Sitzungs-/Recoveryserie |                            19 |    6 |       36 |            25 |              86 |              0 |       36 |
| Schema-4-Update         | 10 ursprünglich + 5 verwaltet |    2 |       12 |            17 |              46 |              0 |       12 |
| Zusammen                |                            34 |    8 |       48 |            42 |         **132** |          **0** |   **48** |

Beide Phasen gültig: acht von acht Kontrollen, keine verlorenen Events/Buffers, Zuordnungslücken
oder unbekannten Ziele. Loopback bleibt separat gezählt, jeweils gleich viele WebView-Ereignisse der
IDs 13/29. Keine eigenen beobachteten Netzereignisse beider Broker.

Die Vergleichsserie enthält 18 Child-Controller und ihren Serienroot: elf Erfolge, sechs erwartete
Abweisungen, zwei gezielte Controllerabbrüche. Zwei Sitzungen, vier Recoveries und acht
Replay-Ereignisse. Die Updatephase enthält zehn Erfolge und fünf erwartete Abweisungen: Paket ohne
Vertragsdaten vor Pending, Recovery während Pending, Aktivierung mit altem Kopf, alter Actor nach
Aktivierung und Rückkehr zu dessen Hash. Der neue Broker übernimmt dieselbe Sitzung, führt
Recovery/Replay und authentisierten Cleanup mit idempotenter Wiederholung aus.

Unabhängige Daten-/MAC-Audits bestätigen Kontrollschema-3-Kompatibilität sowie elf Schema-4-Records
mit Actorwechsel und erhaltenen Installationsschlüsseln/App-/Sitzungsbindungen. Global drei
verschiedene persistierte Aufgaben und genau zwei legitime Projektkopien, keine Replaykopie.
SQLite-Integrität, Fremdschlüssel, Unicode-Inhalte und Dokumentzahlen passen. Acht native
Root-Token-/Job-/Sperrbelege, 48 tatsächliche private Dateiverweigerungen; pro App sechs zugeordnete
WebViews. Drei erzeugte Profile authentisiert entfernt. Zwei Apps enden nach Controllerabbruch mit
0, sechs kontrolliert mit 77.

## Prüfung, Grenzen und Fortsetzung

42 Diagnose-Tests bestanden, davon drei neue für exakte Root-Images, ungültige/unvollständige
Zuordnungen und öffentliche Netzereignisse unter einer hashbenannten Root. Format-/Diff-/Hash- und
Faktenprüfung bestanden. Keine neue große Matrix, Helfer-/Appbuildwiederholung oder
Workspace-Gesamtprüfung; der vorherige 51-Test-/Clippy-Nachweis bleibt am unveränderten
Programmstand gebunden. Änderungen betreffen nur Diagnoseauswertung, Tests und Dokumentation.

Erfassungen, Supervisor und Testprozesse beendet. Die echte Vorschauablage und ihre dort
autorisierte EXE wurden nicht verwendet oder verändert. Private Roh-ETL/XML, PID-Zuordnungen,
Kennungen, Pfade, Schlüssel und synthetische Fixtures bleiben lokal; nur bereinigte Aggregate werden
veröffentlicht. Reproduktion: gesicherte Hashes prüfen, zwei frische kalibrierte Sitzungen,
vollständige Start-PID-/EXE-Zuordnung, Auswertung und separater Aufruf-/GUI-/Daten-/MAC-Abgleich.
Bestehende Kontrollablagen nicht kopieren, zurücksetzen oder neue Prüfprogramme dort autorisieren.

Dieser Beleg gilt nur für diese Programme, Pfade und Lebenszeiten. Keine
HTTP-/Inhalts-/Uploadbilanz, lückenlose Vor-Ausführungs-Tokenattestierung aller Nachfahren oder
vollständige externe COM-/Shell-/DNS-/Dienstkausalität. Kein kausaler Ratenvergleich früherer Builds
und keine allgemeine Produktfreigabe. Nächster sinnvoller begrenzter Scope:
Update-Abbruch-/Fehlerreconciliation unter derselben Kontrollsperre, bevor der geprüfte
Start-/Updatepfad in das normale Produkt integriert wird. Echte Paket-/Projektmigration und weitere
Produktabnahme bleiben offen.
