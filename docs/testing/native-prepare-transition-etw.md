# KQ-069: Frisches ETW der Prepare-Übergänge und des verwalteten Kandidaten

Stand: 17. November 2026. Gezieltes mittleres Prüfniveau, ausschließlich synthetische TEMP-Ablagen.
[Bereinigte Aggregate](native-prepare-transition-etw.json),
[funktionale Abbruchprüfung](native-prepare-transition-crashes.md). Striktes Datenschutzziel
unverändert; normale Produktintegration und Produkt-Q-01/Q-02 bleiben offen.

## Programme, Vergleich und Erfassungsgrenze

Unveränderte gesicherte Programme aus `bcf2a0bc90519cbf844aef6b81125e9489fe9a66` unter
`%TEMP%/kraqel-prepare-transition-build-20261117-01/`:

- Broker `88f861ab454f598a85c5eabe3b6fdb192c1af1e58ccfedef36db72bae1f4828a`.
- Kandidat `e4791d2954603bcec39030db5682331bbda6b11abed230e9639cd886f8fe92d7`.
- Forschungs-App `6e56a51ed2e913a878a7935e039bd69628a49be25f0fbab7d02fce3a72fe8c70`.

Paket-/Apphashes und read-only `.kqmeta` beider Broker sowie der verwalteten Kopie unabhängig
geprüft. Kandidat mit identischem Code plus statischem Overlay, keine beliebige
Release-/Datenmigration. Kein neuer Build, keine Änderung an Anwendung, Broker, Auswerter,
Capturehelfer oder Netzfähigkeiten.

Zwei kurze getrennte Sitzungen: **regulärer Schema-4-Update-/Wiederanlaufpfad als Referenz** und die
**drei Prepare-Übergangsabbrüche**. Bewusst keine zusätzliche allgemeine Schema-3-Crashmatrix.
Unverändertes `capture.ps1`, je vier eigene lokale TCP-/UDP-IPv4/IPv6-Kontrollen, erhöhter
PowerShell-7-Supervisor. Höchstens zehn Minuten je Sitzung, nach Prüfende vorzeitig per STOP-Datei
beendet. Keine Firewall-, Loopback-, ExecutionPolicy- oder Domänenänderung, keine physische
Netztrennung. Keine echten Projekte oder Provideraufrufe.

Beide Phasen verwenden den unveränderten Modus `native-control` mit vollständiger privater
Start-PID-/EXE-Zuordnung. Alle **59 direkten Aufrufe** erfasst, einschließlich fünf Starts der
tatsächlich verwalteten `e4791d2954603bcec39030db5682331bbda6b11abed230e9639cd886f8fe92d7.exe`.
Keine automatische Rootsuche oder Lockerung der
Kalibrierungs-/Verlust-/Zuordnungs-/Lebenszeitguards. Prozesshandles bleiben bis Konvertierungsende
gehalten. Separater chronologischer Kernel-Audit gleicht jeden erwarteten Aufruf/Exitcode und alle
GUI-Eltern ab. Die PowerShell-Teststeuerung ist außerhalb dieser ausdrücklich ausgewählten nativen
Bäume. Fehlender GUI-Zwang bei Statusaufrufen ersetzt keinen Abgleich der tatsächlich erwarteten
GUIs.

## Ausgeübte Pfade und Messergebnis

Referenz: neue Kontrollablage/bestehende synthetische Importaufgabe, Paket ohne Vertragsdaten vor
Pending abgewiesen, korrekte Vorbereitung/Aktivierung. Pending-Recovery, alter Aktivierungskopf,
alter Actor nach Aktivierung und Rollback gesperrt. Verwalteter Kandidat führt Status,
authentisierten Wiederanlauf derselben Aufgabe, Replay, Stilllegung und idempotenten Cleanup aus.
Kein Kandidatenstart vor Aktivierung.

Übergänge: drei frische Fälle mit vorherigem authentisiertem Abort, je ein harter Prozessabbruch bei
`artifact-removed`, `package-created` oder `prepare-recorded`. Erste zwei: Kopf unverändert, kein
Pending, Paket fehlt bzw. leer. Dritter: authentisiertes Pending mit leerer Kopie, ausdrücklich
abzubrechen vor Retry. Aktivierung fehlender/leerer Kopie, Pending-Recovery und alte Köpfe bleiben
gesperrt. Danach derselbe Hash erneut vorbereitet/abgebrochen, bestehende Sitzung wiederaufgenommen
und Replay geprüft. Keine Kandidatenausführung oder Aktivierung in dieser zweiten Phase.

| Phase                                      |                Brokerprozesse | Apps | WebViews | Console-Hosts | Explizite Enden | Nicht-Loopback | Loopback |
| ------------------------------------------ | ----------------------------: | ---: | -------: | ------------: | --------------: | -------------: | -------: |
| Regulärer Update-/verwalteter Wiederanlauf | 10 ursprünglich + 5 verwaltet |    2 |       12 |            17 |              46 |              0 |       12 |
| Drei Prepare-Übergänge                     |                            44 |    6 |       36 |            50 |             136 |              0 |       36 |
| Zusammen                                   |                            59 |    8 |       48 |            67 |         **182** |          **0** |   **48** |

Beide Phasen gültig: acht von acht Kontrollen, keine verlorenen Events/Buffers, Zuordnungslücken
oder unbekannten Ziele. Loopback separat gezählt, je Phase gleich viele WebView-Ereignisse der IDs
13/29. Keine eigenen beobachteten Netzereignisse beider Broker. Exakte Bereitschafts-/Stop-/
Konvertierungszeiten in UTC und ETL-Größen stehen in den Aggregaten; Bereitschaft nach Kontrollen
ist kein behaupteter exakter Providerstart.

Referenz: zehn Erfolge/fünf erwartete Abweisungen. Übergänge: 34 Erfolge/sieben erwartete
Abweisungen/drei harte Controllerabbrüche mit 86. Alle 182 Enden aus expliziten Kernel-Stop-Events,
keine erfundenen Endzeiten. Acht native Root-Token-/Job-/Sperrbelege, je sechs zugeordnete WebViews;
alle acht GUIs kontrolliert mit 77 beendet. GUI-Eltern sind erfolgreiche Bind-/Recovery-Controller;
in der Referenz startet der ursprüngliche Broker die erste und der verwaltete die zweite GUI.

## Datenprüfung, Übergabe und Grenzen

Unabhängige Nur-Lese-Audits prüfen Kontroll-MACs/Kopf, kanonische Rootbindung, Actorwechsel,
Paketmetadaten, Schlüssel-/App-/Sitzungs-/Aufgabenbindung und Recovery-Launch-IDs. Referenz elf
Records, Übergänge 13/13/15. Gespeicherte Abbruchköpfe passen zur Kette, fehlende/leere Pakete zu
den definierten Punkten. Global vier verschiedene Aufgaben und vier legitime Projektkopien, keine
Replaykopie. SQLite-Integrität/Fremdschlüssel/Unicode-Inhalte passen. 48 tatsächliche private
Dateiverweigerungen mit capabilityfreien AppContainer-Root-Tokens, vier erzeugte Profile nach
authentisierter Stilllegung entfernt; Cleanup idempotent, Jobs am Ende leer.

42 Diagnose-Tests sowie Format-/Diff-/Hash-/Faktenprüfung grün. Der vorherige
56-Test-/Clippy-Nachweis bleibt an die unveränderten Programme gebunden. Keine neue große Matrix,
Workspace-Gesamtprüfung oder Buildwiederholung. Erfassungen, Supervisor und Testprozesse beendet.
Echte Vorschauablage und dort autorisierte EXE unverändert. Roh-ETL/XML/PID-Maps, Kennungen/Pfade,
Schlüssel, Journale und synthetische Fixtures bleiben lokal; nur bereinigte Aggregate in Git.

Reproduktion: gesicherte Hashes prüfen, zwei frische kalibrierte Sitzungen, reguläre 15 Aufrufe und
`check-update-reconciliation.ps1 -PrepareTransitions` mit 44 Aufrufen. Alle Start-PIDs/Images und
Handles halten, `native-control --root-map` auswerten, vollständigen Aufruf-/GUI-/MAC-/Datenabgleich
separat durchführen. Keine Übernahme oder Zurücksetzung bestehender Kontrollablagen.

Gültig nur für diese Programme/Pfade/Lebenszeiten. Keine HTTP-/Inhalts-/Uploadbilanz, lückenlose
Vor-Ausführungs-Tokenattestierung aller Nachfahren oder vollständige externe COM-/Shell-/DNS-/
Dienstkausalität. Keine Stromausfallgarantie, allgemeine Erst-Prepare-Orphan-Bereinigung oder
Produktfreigabe. Unbekannte Artefakte ohne Abort und defekte Ketten bleiben gesperrt. Nächster
begrenzter Scope: den Start-/Integrationsvertrag für den normalen Produktstart mit bestehenden
Projekten konkretisieren; Forschungsautorität nicht automatisch auf echte Daten übertragen.
