# P2c C3: frischer ETW-Netzvergleich von Root, Kind und gesperrtem Wiederanlauf

Stand: 17. November 2026. Gezieltes mittleres Prüfniveau, ausschließlich frische synthetische
Ablagen. [Bereinigte Aggregate](product-start-p2c-group-bootstrap-etw.json),
[C3-Funktionsnachweis](product-start-p2c-group-bootstrap.md). **Keine Produktfreigabe**; striktes
Datenschutzziel und Produkt-Q-01/Q-02 bleiben unverändert.

## Programme und Erfassung

Unveränderter gesicherter Controller und sämtliche Codefixturekopien aus `6e2c089`:
`b24b3aac3d80bc59b63c74e2cbad03bf4980d8496b88434f3afc035fdbbe48af`. Sicherung
`%TEMP%/kraqel-product-p2c-group-build-20261117-01`. Byteidentische Controllerkopie mit Basisnamen
`kraqel-isolation-probe.exe` für die bestehende Auswertung; Root und Kind laufen als `worker.exe`.
Kopien/Hash vor und nach Erfassung unabhängig abgeglichen. Kein neuer Build und keine
Runtime-/App-/Transport-/Paket-/Capabilityänderung.

Unveränderte [Erfassung](../../scripts/m3-network/capture.ps1) und
[Auswertung](../../scripts/m3-network/analyze.py), Modus `native-control`. Jede tatsächliche
Controller-PID mit exaktem erwarteten EXE-Namen explizit festgehalten, Handles bis nach
Konvertierung gehalten. Keine automatische Rootsuche oder gelockerten Nachweisregeln. Private
Messsteuerung basiert auf dem
[C3-Prüfablauf](../../scripts/runtime-isolation/check-workspace-group.ps1), ergänzt um einen kurzen
Referenzpfad, Handlehaltung und den bereits funktional geprüften einzelnen Wächterabbruch.

Nur die Erfassung verwendet erhöhte PowerShell 7; Controller, Root/Kind und Prüfsteuerung laufen
ohne zusätzlich erhöhtes Token. Je vier eigene lokale TCP-/UDP-IPv4/IPv6-Kontrollen aus dem
Capturehelfer. Höchstens fünf Minuten je Sitzung; nach tatsächlichem Ende per eigener STOP-Datei
beendet, ungefähr **9,2 und 19,8 Sekunden ab Ready**. Beide Sitzungen gestoppt und konvertiert,
eigener Supervisor beendet. Keine Firewall-/Loopback-/ExecutionPolicy-/Domänenänderung.

## Gemessene Abläufe und Ergebnis

Referenz: frischer C2-Store, Daten-/Profil-/Codebindung, ein normaler C3-Root-/Kindabschluss,
explizite Profilstilllegung und Status. Sechs Controller, zwei Workerrollen, 15 MAC-Records, ein
verbrauchter Launch und ein regulär entferntes Profil.

Gruppen-/Wiederanlaufpfad: zwei erfolgreiche Gruppen, falsche Root-MAC, unzugeordneter nie
fortgesetzter Root, falsche Kind-MAC sowie Stale-Abweisung. Danach reguläre Stilllegung. Zusätzlich
frischer separater Fall mit hartem Wächterende 86 nach Root-Ende/ChildBound bei noch suspendiertem
Kind. Status und negative Start-/Stilllegungsaufrufe vor und nach ausdrücklich eigenem
Fixturecleanup bestätigen unverändertes pending. 22 Controller, zehn Workerrollen, 51 MAC-Records,
sechs verbrauchte Launches. Kein Broker-Recovery aus verschwundenem Profil.

| Phase                                     | Controller | Rootrollen | Kindrollen | Konsolenhosts | Explizite Enden | Kontrollen | Nicht-Loopback | Loopback |
| ----------------------------------------- | ---------: | ---------: | ---------: | ------------: | --------------: | ---------: | -------------: | -------: |
| Normaler Abschluss                        |          6 |          1 |          1 |             6 |              14 |        4/4 |              0 |        0 |
| Gruppenfehler und gesperrter Wiederanlauf |         22 |          6 |          4 |            22 |              54 |        4/4 |              0 |        0 |
| Zusammen                                  |     **28** |      **7** |      **5** |        **28** |          **68** |    **8/8** |          **0** |    **0** |

Beide Messungen gültig: null verlorene Events/Buffers, keine Zuordnungslücken oder unbekannten
Ziele. **Null beobachtete Netzereignisse in den ausgewählten Prozessbäumen**, einschließlich
Loopback. Alle Konsolenhosts gehören direkt zu Controllern; Kinder gehören tatsächlich zu den
jeweiligen Root-Workern. Keine zusätzlichen Worker, GUI oder WebView im Baum.

## Unabhängiger Gegencheck

Chronologischer Kernel-Audit bestätigt alle 68 expliziten Enden und ihre
Start-/End-Erzeugungszeiten. Alle **28 Controller-Erzeugungszeiten** stimmen exakt auf 100 ns mit
den beim Start getrennt festgehaltenen nativen Werten überein. 22 reguläre Controller-Rückgaben,
fünf erwartete Abweisungen und ein harter Abbruch mit 86.

Alle **sechs dauerhaften Root- und fünf Kindbindungen** stimmen anhand nativer PID/Erzeugungszeit
mit der richtigen Kernelgeneration und dem jeweiligen Elternbezug überein. Ebenso sind die sieben
Wächterbindungen zugeordnet. Bei allen fünf gebundenen Kindern liegt das tatsächliche Kernelende
nach dem Root-Ende. Der eine unzugeordnete Root ist ausschließlich diagnostisch seinem negativen
Controlleraufruf zugeordnet und bereits vor dessen Ende beendet; daraus entsteht keine neue
Gruppen-/Bereinigungsautorität.

**Exit 0 ist kein fachlicher Erfolg:** Neun Workerenden melden 0, zwei melden 2, eines meldet 1.
Auch das jobbedingt beendete suspendierte Kind des Crashfalls erhält Kernel-Exit 0. Maßgeblich
bleiben die authentisierten Kontrollzustände: drei erfolgreiche Finish-Records, drei verbrauchte
fehlgeschlagene Bootstraps und ein weiterhin gesperrter Launch ohne Drained/Finish.

Unabhängiger lesender MAC-/Daten-/Inventaraudit: **66 vollständige Records, sieben verbrauchte
Launches, sechs Inventare mit tatsächlichem Inhalt**, drei Codefixturekopien und erhaltene
C2-Ausgangsdaten. Drei Profile am Ende abwesend: zwei authentisierte Brokerstilllegungen und ein
ausdrücklich eigener Fixturecleanup des Prüfablaufs. Der negative Store bleibt trotz
Profilabwesenheit unverändert pending. Im Crashfall sind Root und Kind auch im Kernel vor den
erneuten Abweisungen nach Fixturecleanup beendet; dies ersetzt keinen produktfähigen
Gruppen-Recoveryvertrag. Echte Vorschau-Kontrollhashes unverändert, keine neue Brokerfreigabe oder
Bestandsmigration.

42 unveränderte Diagnose-Tests erneut bestanden; Hash-/Quellen-/Verweis-/Format-/Diffprüfung
ergänzt. Die vorher belegten 96 Helfertests werden nicht als neuer Testlauf ausgegeben. Keine neue
breite Crashmatrix, kein Runtimebuild und kein vollständiger Appcheck.

## Grenzen, Roadmap und Fortsetzung

Der Nachweis gilt ausschließlich den exakten C3-Bytes und den genannten synthetischen Abläufen.
Gruppen-MAC bleibt keine unabhängige Schreiber-PID-Autorität; native UI-ABI bleibt 0. Keine
untrusted Renderer-/WebView-Rollen, vollständige Runtimepaketclosure, normale App-/
Launcherintegration, automatische Recovery nach Wächterverlust oder Produktabnahme belegt. ETW
liefert Transportmetadaten, keine HTTP-/Upload-/Inhaltsfreigabe. Externe COM-/Shell-/ Systemdienste
und Prüfsteuerung außerhalb der ausgewählten Bäume werden nicht pauschal mitattestiert. Keine
physische Netztrennung oder universelle Netzstille zugesagt.

Die [Roadmap](../../ROADMAP.md) bleibt grob bei **65 % gesamt einschließlich M6/0.2** (plausible
Spanne 60–70 %) und **80 % nutzbarem 0.1-Funktionsumfang**. Das ist eine fachlich gewichtete
Schätzung, keine Aufgabenquote oder Zeitprognose. C3 samt Netzbeleg schließt einen Forschungsschritt
ab; reale isolierte Oberfläche, Bestandskopie und Produktstarter sind weiterhin große
Integrations-/Abnahmerisiken. Diese offenen Gates begrenzen den Gesamtfortschritt.

Rohdaten nur lokal unter `%TEMP%/kraqel-c3-etw-20261117-01-reference` und `…-group`, Audit beim
gesicherten Build. Keine ETL/XML, PIDs, Erzeugungszeiten, Rootpfade, Profil-IDs, Schlüssel oder
Records nach GitHub. Eigene Prozesse beendet, Profile entfernt; synthetische Kontrollketten und
Daten lokal erhalten. Ein negativer Store bleibt absichtlich gesperrt.

Nächster begrenzter Schritt: **C4-Vertrag/Integration einer explizit gebundenen Runtime und
minimalen isolierten Oberfläche**. UI-Paketclosure, Daten-/Commandgrenzen und WebView-Kindrollen
zuerst konkretisieren; synthetischen Gruppenvertrag nicht ungeprüft übernehmen. P2 insgesamt,
vollständige C4-/Produktintegration, P3–P5 und Produkt-Q-01/Q-02 bleiben offen. Das strikte
Datenschutzziel bleibt unverändert.
