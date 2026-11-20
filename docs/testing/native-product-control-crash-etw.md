# KQ-069: Kontroll-Crash-Serie und frischer ETW-Netzvergleich

Stand: 16. November 2026. Native Windows-Vorschau, ausschließlich frische synthetische TEMP-Ablagen.
[Kontrollvertrag](../data-format/native-product-control-v1.md),
[bereinigte Aggregate](native-product-control-crash-etw.json). Striktes Datenschutzziel unverändert;
keine Produktfreigabe, Produkt-Q-01/Q-02 nicht bestanden.

## Gesicherter Stand und Testgrenze

Ausgangspunkt `43e7392e1fa9835329aa82688c47aef4ac7150a3`; die neue Implementierung gehört zum Commit
dieses Belegs. Gesicherter Stand unter `%TEMP%/kraqel-product-crash-build-20261116-02/`:

- Basisbroker-SHA-256: `ce94779f7b156adb739a1b48321344e49ffdf0148ec4b879529b3a509f89859f`.
- Updatekandidat-SHA-256: `cdf352f4afcc0d9aab921be0a14dbcdd71c69af2322c4f63da609027e8d30442`.
- Unveränderte Forschungs-App: `6e56a51ed2e913a878a7935e039bd69628a49be25f0fbab7d02fce3a72fe8c70`.

Der Kandidat ist eine lokale EXE-Kopie mit synthetischem PE-Overlay: anderer Hash, gleicher
Programmcode. Das prüft Autoritätswechsel zwischen Programmbytes, keine Herausgeberauthentisierung.
Die bisherige echte Vorschauablage und ihre verwaltete autorisierte EXE bleiben unverändert; der
neue Prüfbroker wurde dort nicht automatisch aktiviert. Keine Schlüssel-/Datenübernahme.

`product-control-series <frische Controllerwurzel> <gesicherte Forschungs-App>` erzeugt 29 private
Testablagen: 28 Crashfälle und einen regulären Vergleichsfall. Der interne `product-control-child`
akzeptiert ausschließlich das begrenzte TEMP-Controller-/Case-/Store-Layout, authentisiert
vorhandenen Zustand und prüft tatsächliche Programmbytes. Kein Fault-Umgebungsflag und keine
Renderer-API. Normale Vorschau-Commands schalten keine Abbruchpunkte ein.

Die Abbrüche verwenden `TerminateProcess` am eigenen Prozess mit Exitcode 86: keine Rust-
Destruktoren, kein absichtliches vorheriges Aufräumen. Geprüft sind laufzeitgebundene Prozess-
abbrüche, keine beliebigen Stromausfälle oder jeder mögliche OS-Mikroschritt.

## Crash-Matrix und Ergebnisse

| Übergang             | Abbruchpunkte                                                                                         | Fälle |
| -------------------- | ----------------------------------------------------------------------------------------------------- | ----: |
| Initialisierung      | vor Schlüssel; Schlüssel synchronisiert; Record angelegt/synchronisiert; Kopf gestagt/veröffentlicht  |     6 |
| Kontrollschema 1 → 2 | vor Record; Record angelegt/synchronisiert; Kopf gestagt/veröffentlicht                               |     5 |
| Updatevorbereitung   | dieselben fünf Record-/Kopfpunkte; zusätzlich nach 1.024 Paketbytes und nach vollständigem Paket-Sync |     7 |
| Aktivierung          | vor Record; Record angelegt/synchronisiert; Kopf gestagt/veröffentlicht                               |     5 |
| Expliziter Abbruch   | vor Record; Record angelegt/synchronisiert; Kopf gestagt/veröffentlicht                               |     5 |

Alle **28 echten Controllerabbrüche** mit je zwei frischen Neustartprozessen für bisherigen und
potenziell neuen Programmhash geprüft. Ergebnis: **17 gesperrte, elf nutzbare Zustände**. Acht der
nutzbaren Fälle ausdrücklich weitergeführt; drei waren bereits vollständig abgeschlossen.

Unvollständige Initialisierung, leere/zusätzliche Records, alte oder fehlende Köpfe und `head.next`
bleiben gesperrt. Die Neustartprüfungen verändern weder Schlüssel noch Dateien. Es gibt keine
automatische Reparatur, Rücksetzung, Neusignierung oder Veröffentlichung eines unvollständigen
Abschlusses. Die gesperrten Fixtures bleiben zur Nachprüfung erhalten.

Vor dem ersten Record ist der alte Kopf weiterhin gültig: Eine ausdrücklich neu angestoßene
Operation darf dann erstmals festgeschrieben werden. Nach Festschreibung wird der alte Kopf
abgewiesen. Bereits signiertes Pending ohne vollständiges Paket bleibt Pending; fehlende/ partielle
Pakete können nicht aktiviert werden, ein ausdrücklich autorisierter Abbruch ist möglich. Nach
vollständiger Paketkopie ist explizite Aktivierung möglich; danach wird die alte EXE abgewiesen.
Eine geschlossene Aktivierung ist kein erneuter Grant für das Vorgängerprogramm.

Nicht initiale Ablagen behalten einen unveränderten synthetischen Daten-Sentinel. Das ist kein
Nachweis einer realen Projekt-/Auftragsdatenbankmigration. Die zwei regulären GUI-Vergleiche prüfen
zusätzlich zwölf tatsächliche private Dateiverweigerungen unter eingeschränkten Root-Tokens vor und
nach einem Programmwechsel, synthetischen Import und Profilbereinigung.

Je erfolgreicher Serie: **162 Child-Controller** mit 76 Enden 0, 58 erwarteten Abweisungen 1 und 28
harten Enden 86; der Seriencontroller selbst endet 0. Native Zustandsfolgen, MACs, Köpfe, kanonische
Wurzelbindung, Paketgrößen/-hashes und unveränderte Sentinels separat gegengeprüft. Der unabhängige
Audit zählt 28 unterschiedliche vorhandene Schlüssel und 26 unterschiedliche Installationskennungen
in gültigen Genesis-Records; frühe Initialisierungsabbrüche haben noch keinen Schlüssel
beziehungsweise noch keinen gültigen Genesis-Record.

Ein erster Lauf wurde verworfen, weil seine Prüferwartung nach Abbruch **vor** Recordschreiben
fälschlich schon einen veralteten Kopf voraussetzte. Der unveränderte Kopf durfte korrekt für eine
ausdrücklich neue Vorbereitung verwendet werden. Prüferwartung angepasst, frische Wurzeln und neuer
gesicherter Build; keine Lockerung der Kontrollprotokoll-Abweisung unvollständiger Köpfe. Dieser
Erstlauf zählt nicht zu den bestandenen Serien.

## Frischer ETW-Vergleich desselben Builds

Zwei getrennte begrenzte Erfassungen mit unverändertem `capture.ps1` und unverändertem
`analyze.py --mode broker-ui`, ein per UAC erhöhter PowerShell-7-Supervisor. Je Phase eigene lokale
TCP-/UDP-IPv4/IPv6-Kontrollen, Verlustzähler und vollständige Prozessgrenzen. Keine Firewall-,
Loopback-, ExecutionPolicy- oder Domänenänderung und keine physische Netztrennung. Root-PIDs separat
aufgezeichnet, Root-Prozesshandles bis Messende gehalten; der Crash-Seriencontroller hält auch seine
Child-Prozessobjekte bis Serienende. Keine neue Auswerteausnahme für Updatekandidaten.

| Phase                                 | Bereitschaft nach Kontrollen (UTC) | Sitzungsende (UTC) | Konvertiert (UTC) | ETL-Bytes |
| ------------------------------------- | ---------------------------------- | ------------------ | ----------------- | --------: |
| Bisherige GUI-/Broker-Vergleichsserie | 13:13:09.1652762                   | 13:16:19.3561958   | 13:16:20.1797827  | 5.570.560 |
| Kontroll-Crash-Serie                  | 13:16:20.2200728                   | 13:17:17.9081041   | 13:17:18.0761632  | 1.245.184 |

Alle Zeiten am 16. November 2026. Bereitschaft ist kein behaupteter exakter Providerstart.

Die bisherige Vergleichsserie besteht frisch mit dem neuen Broker: neun Fälle, 72 Apps, 18
Controller-Kills, 27 Wiederanläufe und 54 abgewiesene Folgeaktionen; neun Stilllegungen im
gemessenen Baum. 109 Broker, 72 Apps, 432 WebViews, 181 Console-Hosts; alle 794 Enden explizit. Vier
Kontrollen, Nullverluste/Zuordnungslücken/unklare Ziele, **0 Nicht-Loopback-Ereignisse**; getrennt
432 Loopback-Ereignisse, je 216 der IDs 13/29. Unabhängige MAC-/SQLite-/Unicode-Prüfung grün.

Die Kontroll-Crash-Serie wird anschließend vollständig neu durchgeführt: erneut 28 Abbrüche, 17
gesperrte/elf nutzbare Zustände und acht explizite Weiterführungen. 133 Basisbroker-Prozesse, 30
Kandidatenprozesse, zwei Apps, zwölf WebViews, 165 Console-Hosts; alle 342 Enden explizit. Alle 162
Child-Starts, Elternlebenszeiten und Exitcodes unabhängig mit dem Kernelprotokoll abgeglichen.
Zweimal sechs WebViews je App; beide Apps kontrolliert mit 77 beendet und Profile entfernt. Vier
eigene Kontrollen, Nullverluste/Zuordnungslücken/unklare Ziele, **0 Nicht-Loopback-Ereignisse**;
getrennt zwölf Loopback-Ereignisse, je sechs der IDs 13/29.

Zusammen **1.136 explizite Prozessenden**, 242 Basisbroker, 30 Kandidaten, 74 Apps, 444 WebViews und
346 Console-Hosts. Acht von acht Kontrollen, Nullverluste und keine Zuordnungslücken. **0
beobachtete Nicht-Loopback- und 444 getrennte Loopback-Ereignisse** in den zugeordneten Bäumen.
Keine eigenen beobachteten Netzereignisse der beiden Brokerprogramme im ausgewerteten Ausschnitt.
Alle 74 App-Startbelege, 162 Kontroll-Child-Starts und ihre Elternlebenszeiten unabhängig bestätigt;
keine Ersetzung fehlender Enden durch PID-/Namensmengen. Kein kausaler Ratenvergleich mit alten
Builds.

Die gemessenen synthetischen Importabläufe enthalten 20 global unterschiedliche Aufträge und genau
13 unterschiedliche legitime Projektkopien, keine durch Replay entstandene Kopie. Davon 18
Aufträge/elf Kopien in der bisherigen Vergleichsserie und zwei Aufträge/zwei Kopien bei den neuen
Containerzugriffsprüfungen. Kontroll-MAC-Records sind keine Importaufträge.

## Prüfstand und Grenzen

40 aktive Helfer-/Transporttests, Helfer-Clippy mit Warnungen als Fehler, rustfmt, `pnpm check`, 36
Diagnose-Tests und Format-/Diff-/Hash-/Faktenabgleich bestanden. Kein neuer vollständiger Appbuild.
Basis-/Kandidaten-/Apphashes vor/nach Messung geprüft. Die zwei Container-Testprofile je
Kontrollserie sowie die neun Vergleichsprofile entfernt, Testprozesse/Erfassungen/Supervisor
beendet. Kontroll-Fixtures samt gesperrten Zuständen bewusst erhalten; sie enthalten ausschließlich
synthetische Daten. Rohtraces, PIDs, UUIDs, lokale Pfade und Schlüssel bleiben lokal.

Reproduktion: obige Hashes prüfen, `product-control-series <frische TEMP-Controllerwurzel> <App>`
funktional ausführen und MAC-/Kopfzustände unabhängig auditieren. Danach zwei frische kalibrierte
ETW-Sitzungen: `ui-combined-series` und erneut `product-control-series`; jeweils erst nach
Bereitschaft starten, tatsächliche Root-PID/Handles halten und vollständig unter `broker-ui`
auswerten. Native Child-Exitcodes, Kernel-Lebenszeiten und Daten getrennt abgleichen. Niemals eine
bestehende Vorschauablage zurücksetzen oder zur Fault-Injektion umbenennen/kopieren.

Der Netzbeleg gilt nur für diese Programme, Lebenszeiten und Pfade, nicht für HTTP-/Uploadinhalte,
vollständige externe COM-/Shell-/DNS-/Brokerkausalität oder den normalen Produktstart. Keine
automatische Wiederherstellung der 17 gesperrten Abschlüsse, allgemeine Stromausfallgarantie oder
vollständige private Checkpoint-Rollbackerkennung. Normale Produktintegration, echte Datenmigration,
Herausgebervertrauen und weitere allgemeine F08-/Dienstgrenzen bleiben offen. Nächster Scope:
kontrollierter isolierter Produktstarter mit derselben Kontrollsperre und getrennten Datenrollen;
gesperrte Abschlusszustände weiterhin konservativ behandeln. Striktes Ziel unverändert.
