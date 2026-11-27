# KQ-069: Sitzungsübergangs-Crashpunkte und frischer ETW-Vergleich

Stand: 17. November 2026. Native Windows-Vorschau mit ausschließlich neuen synthetischen Fixtures.
[Bereinigte Aggregate](native-session-transition-crash-etw.json),
[Sitzungsgrundlage](native-product-session-recovery.md),
[Kontrollvertrag](../data-format/native-product-control-v1.md). Striktes Datenschutzziel
unverändert; keine Produktfreigabe, Produkt-Q-01/Q-02 nicht bestanden.

## Gesicherter Prüfstand

Ausgangspunkt `f19798c779394ec8b9c19cd5114715617baac32b`; neue Implementierung im Commit dieses
Belegs. Gesicherter Build unter `%TEMP%/kraqel-transition-build-20261117-01/`:

- Broker-SHA-256: `8b5f58587d6ba2b608a6c5573efa0dc430bb566db344f04c6543ad8c92380658`.
- Forschungs-App unverändert: `6e56a51ed2e913a878a7935e039bd69628a49be25f0fbab7d02fce3a72fe8c70`.

`product-transition-series <frische TEMP-Controllerwurzel> <Forschungs-App> <App-SHA>` verwendet 20
getrennte private Testinstallationen. Der interne `product-transition-child` akzeptiert nur das
begrenzte synthetische Controller-/Case-/Store-Layout und eine typisierte passende Kombination aus
Aktion und Crashpunkt. Keine Fault-Umgebungsvariable oder Renderer-API; normale Vorschau-Commands
aktivieren keine Faults. Hard-Crash über `TerminateProcess`, Exitcode 86, ohne Destruktoren oder
absichtliches vorheriges Job-Cleanup. Keine Garantie für beliebige Stromausfälle/OS-Mikroschritte.

## Übergangsmatrix und Zustände

Gemeinsame Punkte: vor Record, Record angelegt/synchronisiert, Kopf gestagt/veröffentlicht.
Zusätzlich: Sitzung vorbereitet; nach persistiertem Startrecord vor GUI-Ausführung; Stilllegung
festgeschrieben; Profil entfernt.

| Übergang             |                                                          Punkte | Gesperrt | Kontrollzustand authentisierbar | Explizite Recovery bestanden |
| -------------------- | --------------------------------------------------------------: | -------: | ------------------------------: | ---------------------------: |
| Sitzungsbindung      |                 fünf gemeinsame + Sitzung vorbereitet + vor GUI |        3 |                               4 |                            0 |
| Recovery-Startrecord |                                       fünf gemeinsame + vor GUI |        3 |                               3 |                            3 |
| Stilllegung          | fünf gemeinsame + Stilllegung festgeschrieben + Profil entfernt |        3 |                               4 |                            1 |
| Gesamt               |                                                              20 |        9 |                              11 |                            4 |

Die Spalte **authentisierbar bedeutet keinen allgemeinen Ausführungsgrant**. Von den elf Zuständen:
zwei ungebundene mit verwaister Vorbereitung, zwei gebundene ohne Journal, vier explizit
wiederanlaufbare und drei endgültig stillgelegte. Alle neun unvollständigen Record-/Kopfabschlüsse
bleiben gesperrt; kein Kopfreparieren, Recordlöschen oder Schlüsselreset. Der Statusprozess
verändert keine Kontrolldateien. Fehlendes Journal wird nicht neu angelegt; stillgelegte Sitzungen
erhalten keinen neuen Importgrant.

Vor Recordschreiben kann eine ausdrücklich neue Recovery noch unter dem unveränderten Kopf
stattfinden. Nach abgeschlossenem Startrecord ist dessen Startkennung verbraucht, auch wenn noch
keine GUI lief. Der nächste erlaubte Wiederanlauf schreibt eine neue Kennung und übernimmt denselben
bereits verbrauchten Auftrag. Nach Stilllegungsabschluss bleibt Wiederanlauf gesperrt, unabhängig
davon, ob das Profil bereits entfernt wurde.

Je erfolgreiche Serie: 117 Child-Controller mit 55 erfolgreichen Enden 0, 29 erwarteten Abweisungen
1 und 33 harten Enden 86. Davon **20 Übergangsabbrüche** und **13 separate Controllerabbrüche zur
Vorbereitung vorhandener abgeschlossener Sitzungen**. 17 GUI-Starts: 13 vorbereitende Importe und
vier explizite Recoveries mit acht Replay-Ereignissen. 102 tatsächliche private Dateiverweigerungen
unter suspendiert geprüften Root-Tokens ohne Capabilities. 13 verschiedene persistierte Aufgaben und
genau 13 legitime Projektkopien mit je zwei Dokumenten; keine zusätzliche Recovery-/Replaykopie.

**Profilbereinigung ist hier ausschließlich Testfixture-Cleanup.** Nach Ende aller eigenen Jobs
werden nur die in diesen frischen privaten Testinstallationen erzeugten Probeprofile entfernt. Die
Verzeichnisnamen dienen dabei niemals als Import-/Recoveryautorität. 19 Profile entfernt; eines war
beim Crashpunkt nach erfolgreicher Profilentfernung bereits entfernt. Kontrolldateien bleiben bei
diesem Cleanup bytegleich; alle 20 Fixtures samt gesperrten/negativen Zuständen bleiben lokal
erhalten. Das implementiert keine allgemeine Produkt-Reconciliation verwaister oder bereits
stillgelegter Sitzungsprofile.

Unabhängiger Nur-Lese-Audit bestätigt HMAC-/Installations-/Vorgängerfolgen, kanonische Store- und
Sitzungswurzelbindung, vollständige sowie bewusst unvollständige Kopfzustände, unveränderte
Appbytes, vorhandene Taskzustände/Belege, Archivhash/-länge, SQLite-Integrität/Fremdschlüssel und
Unicode-Inhalt. Native Digestprüfungen belegen unveränderte Kontrolldateien bei Status, negativem
Recovery und Testprofil-Cleanup. Testschlüssel bleiben unverändert.

## Frischer ETW-Vergleich desselben Builds

Zwei getrennte kalibrierte Erfassungen mit unverändertem `capture.ps1` und
`analyze.py --mode broker-ui`, ein per UAC erhöhter PowerShell-7-Supervisor. Je Phase eigene lokale
TCP-/UDP-IPv4/IPv6-Kontrollen und Verlustzähler. Maximaldauer jeweils 20 Minuten, nach Serienende
gezielt per eigener STOP-Datei beendet. Keine Firewall-/Loopback-/ExecutionPolicy-/Domänenänderung,
keine physische Netztrennung. Root-PIDs separat aufgezeichnet; Prozesshandles bis Mess-/Serienende
gehalten. Keine neue Auswerteausnahme und kein Übertrag alter Netzbelege.

| Phase                                        | Bereitschaft nach Kontrollen (UTC) | Sitzungsende (UTC) | Konvertiert (UTC) | ETL-Bytes |
| -------------------------------------------- | ---------------------------------- | ------------------ | ----------------- | --------: |
| Bisherige dauerhafte Sitzungs-/Recoveryserie | 06:24:35.4372021                   | 06:24:57.3570050   | 06:24:57.6553796  | 1.769.472 |
| Neue Übergangs-Crash-Serie                   | 06:24:57.7012945                   | 06:26:29.1327913   | 06:26:30.1037979  | 6.750.208 |

Alle Zeiten am 17. November 2026. Bereitschaft ist kein behaupteter exakter Providerstart.

Die bisherige Serie besteht frisch mit demselben neuen Broker: zwei Fälle, zwei Controllerabbrüche,
sechs GUIs, vier Recoveries und acht Replay-Ereignisse. 19 Controller, sechs Apps, 36 WebViews und
25 Console-Hosts; alle **86 Enden explizit**. Vier Kontrollen, Nullverluste/Zuordnungslücken/unklare
Ziele, **0 Nicht-Loopback-Ereignisse**; getrennt 36 Loopback-Ereignisse, je 18 der IDs 13/29.

Die neue Serie wiederholt vollständig alle 20 Übergangs-Crashfälle samt 13 vorbereitenden
Controllerabbrüchen und vier expliziten Recoveries. 118 Controller, 17 Apps, 102 WebViews und 135
Console-Hosts; alle **372 Enden explizit**. Vier eigene Kontrollen, Nullverluste/Zuordnungslücken/
unklare Ziele, **0 Nicht-Loopback-Ereignisse**; getrennt 102 Loopback-Ereignisse, je 51 der IDs
13/29. Die Zustands-/MAC-/Datenprüfungen bestätigen erneut neun gesperrte/elf authentisierbare
Zustände.

Zusammen **458 explizite Prozessenden**, 137 Controller, 23 Apps, 138 WebViews und 160
Console-Hosts. Acht von acht Kontrollen, keine Verluste oder Zuordnungslücken. **0 beobachtete
Nicht-Loopback- und 138 getrennte Loopback-Ereignisse** im zugeordneten Baum. Keine eigenen
beobachteten Netzereignisse des Brokerprogramms. Alle 135 Child-Controller, deren Exitcodes und
Elternlebenszeiten sowie alle 23 nativen Appstarts unabhängig chronologisch mit dem Kernelprotokoll
abgeglichen; kein Ersetzen fehlender Stop-Ereignisse durch PID-/Namensmengen. Je App sechs
zugeordnete WebViews. 15 Apps enden durch Controllerabbruch/Jobschluss mit 0, acht Recovery-Apps
kontrolliert mit 77. Das ist keine lückenlose Vor-Ausführungs-Tokenattestierung aller
WebView-Nachfahren.

Die gemessenen Importdaten enthalten global 15 verschiedene persistierte Aufgaben und genau 14
legitime Projektkopien: zwei/eine in der bisherigen Serie und 13/13 in der neuen. Kontrollrecords
und vorbereitete Bindungen ohne Journal sind keine zusätzlichen Importaufträge. 138 tatsächliche
private Dateiverweigerungen insgesamt. 22 Profile entfernt, alle eigenen Erfassungen/Prozesse
beendet.

## Prüfstand, Grenzen und nächste Arbeit

46 aktive Helfer-/Transporttests, Helfer-Clippy mit Warnungen als Fehler, rustfmt, `pnpm check`, 36
Diagnose-Tests sowie Format-/Diff-/Hash-/Faktenabgleich bestanden. Kein vollständiger Appbuild,
keine neuen Abhängigkeiten. Roh-ETL/XML, PIDs, UUIDs, Pfade, Schlüssel und private Datenbanken
bleiben lokal; veröffentlicht werden ausschließlich bereinigte Aggregate. Die echte Vorschauablage
und ihre autorisierte verwaltete EXE blieben unbenutzt/unverändert; der neue Prüfbroker wurde dort
nicht automatisch aktiviert.

Reproduktion: gesicherte Hashes prüfen, `product-transition-series` funktional ausführen und
Zustände/MACs/Appdaten unabhängig auditieren. Danach zwei neue kalibrierte ETW-Sitzungen:
`product-session-series` und `product-transition-series`, jeweils erst nach Bereitschaft starten,
Root-PID/Handles halten, alle Prozessenden erfassen, `broker-ui` auswerten und native Start-/Daten-
und Kernel-Lebenszeitbelege getrennt abgleichen. Niemals eine bestehende Kontrollablage als Fixture
kopieren oder zurücksetzen.

Der Netzbeleg gilt nur für diesen Build und die erfassten Pfade/Lebenszeiten. Keine Inhalts-/HTTP-/
Uploadbilanz oder vollständige externe COM-/Shell-/DNS-/Brokerkausalität, keine allgemeine
Stromausfall-/Checkpoint-Rollbackgarantie oder Produktfreigabe. Schema-3-Updates, echte
Datenmigration, Mehrsitzungsbetrieb und normale Produktintegration bleiben offen. Nächster Scope:
authentisierte Bereinigung verwaister beziehungsweise bereits stillgelegter Profile unter derselben
Kontrollsperre, ohne Schlüssel-/Kopfreparatur oder neue Importautorität. Striktes Datenschutzziel
unverändert.
