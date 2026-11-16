# KQ-069: Controller-Crash und dauerhafte authentisierte Wiederherstellung

Stand: 16. November 2026. Die Broker-Grundlage ist im getrennten Forschungslauncher implementiert
und nativ mit synthetischen Daten geprüft. **Kein fertig integrierter Produktbroker und keine
Netzabnahme des neuen Paars.** [Vertrag](../data-format/native-broker-recovery-v1.md),
[bereinigte Aggregate](native-controller-crash.json).

## Buildbindung und Ausführung

Broker-Quellstand `91990de7e3439e198a7b6af2e9ba3a10abf71a5c`, SHA-256
`1877b0c7c199d5e0a2f635ee006264bcc786e7163c6ca5d6ded906a22dc8d92c`. App unverändert aus dem
gesicherten v2-Build, Quellstand `9d87e20d4a2526885c213bd186e4c22573d80744`, SHA-256
`4db1e731e6aee50fe01e9e368d1ded61ac61f3f9dc3e2238d509cee600098e1b`. Paar separat unter
`%TEMP%\kraqel-kq069-controller-broker-build-20261116-03` gesichert. Kein vollständiger Tauri-Neubau
oder Überschreiben des normalen Debug-Builds erforderlich. Der
[vorherige ETW-Beleg](native-assistive-restart-etw.md) gilt weiter ausschließlich für sein altes
App-/Controllerpaar.

```powershell
$seriesRoot = Join-Path $env:TEMP ('kraqel-isolation-controller-' + [guid]::NewGuid().ToString('N'))
& <gesicherter-Broker> ui-controller-crash-series $seriesRoot <gesicherte-Forschungs-App>
```

Die Serie verwendet sechs frische synthetische Sessionwurzeln. Jeder Seed-Broker authentisiert und
synchronisiert den ursprünglichen Auftrag, startet die isolierte App mit Jobzuweisung bei
Prozesserzeugung und beendet **sich selbst** nach dem gebundenen Importcheckpoint hart mit Code 86.
Keine Destruktoren, kein vorheriger geordneter Appabschluss und kein explizites Terminieren des Jobs
vor dem Controllerverlust. Die OS-Kill-on-close-Grenze und der anschließend geprüfte Jobleerstand
ersetzen eine PID-/Timeout-Vermutung.

Zwei separat gestartete Wiederanlaufprozesse pro Session authentisieren denselben dauerhaften Beleg.
Jeder startet eine Reconciliation mit neuer Launch-ID/frischen Pipes und anschließend einen
abzuweisenden Altimport. Insgesamt sechs Controller-Kills, zwölf Wiederanlaufprozesse und 30
App-Generationen. Die Stilllegung läuft als weitere separate native Aktion unter derselben Sperre.

## Verifiziertes Ergebnis

| Controller-Crashpunkt | Journal nach Wiederanlauf | Projektkopien |
| --------------------- | ------------------------- | ------------- |
| `claimed`             | `interrupted`             | 0             |
| `reading`             | `interrupted`             | 0             |
| `prepared`            | `interrupted`             | 0             |
| `commitIntent`        | `interrupted`             | 0             |
| `published`           | `succeeded`               | 1             |
| `completed`           | `succeeded`               | 1             |

Alle 30 suspendierten Root-Token stimmen mit der erwarteten Container-SID überein und haben null
Capabilities. Je Generation vier echte Dateiöffnungen unter dem impersonierten App-Token:
Brokerbeleg lesen/schreiben, Brokersperre lesen und Appdatei schreiben sind mit
`ERROR_ACCESS_DENIED` verweigert, insgesamt 120 Prüfungen. Jobzugehörigkeit vor Resume geprüft. Der
Renderer erhält weder Beleg noch Pfade, neue Commands, Handles oder Netzwerkfähigkeiten.

Je Crashpunkt zehn native Ablehnungsfälle: beschädigter DPAPI-Beleg, unbekannte Version, falsche
Wurzel-/Broker-/Apphashbindung, falsche Session, fehlender Beleg, fehlendes Journal, konkurrierende
Brokersperre und stillgelegte Session. Zusammen 60 Ablehnungen. Die neun Fälle vor erfolgreicher
Wiederherstellung erzeugen keine zweite App-Startdiagnose und verändern das Journal nicht; das
vorübergehend versteckte Journal wird dabei am gesicherten Testpfad geprüft. Nach Stilllegung sperrt
die Markierung vor einem weiteren Appstart. Diese Mutationen erfolgen ausschließlich im nativen
Testhost, nicht über einen Rendererkanal.

Unabhängige Nur-Lese-Prüfung der finalen Ablagen bestätigt alle sechs Journalzustände und
Kopienzahlen, leeres Staging, SQLite-Integrität, Projekt-ID/Receipt und Dokumentanzahl. Beide
veröffentlichten Kopien enthalten exakt `Importprobe: ÄÖÜ ß Ελληνικά Кириллица.`, gültige
Fremdschlüssel und den SHA-256/Byteumfang der synthetischen Archivfixture. Alle sechs Appkopien
stimmen mit dem gesicherten Apphash überein. Je Session fünf Startbelege und vier reguläre
Generationsberichte mit Code 77/leerem Job; für den während des Controller-Kills beendeten Seed wird
kein nachträglicher regulärer Generationsbericht erfunden. Profile nach Stilllegung entfernt, keine
Forschungs-App-/Brokerprozesse mehr aktiv. Rohbelege/Pfade/UUIDs bleiben lokal.

## Prüfungen und Grenzen

`pnpm check`, 26 aktive Helfer-/Transporttests, ein bewusst ignorierter interaktiver Dialogtest,
Helfer-Clippy mit Warnungen als Fehler sowie Helferbuild bestanden. Zusätzlich besteht die bisherige
28-Generationen-GUI-Serie mit dem finalen neuen Broker: sechs App-Crashpunkte, je zweifacher
Wiederanlauf und Altimportabwehr, vier negative Starts. Journalintegrität, vier unterbrochene/zwei
erfolgreiche Aufträge, zwei Kopien, leeres Staging und Cleanup separat geprüft. Auch diese
Regression ist kein neuer ETW-Beleg.

Keine Frontend-/Renderer- Änderung, keine neue Hörabnahme. Die finalen nativen Resultate stammen aus
Sicherung `03`; ein Vorläufer scheiterte an einer für Verzeichnisse ungeeigneten Dateiprüfung, ein
weiterer am Testaudit des absichtlich versteckten Journals. Beide Prüfhelfer korrigiert; die
betroffene persistierte Vorläufersitzung mit ihrem passenden Broker explizit stillgelegt. Frühere
Ergebnisse werden nicht anstelle der finalen Serie verwendet.

Das ist die nachgewiesene Wiederanlaufgrundlage für den Produktbroker. Produktdatenwurzel,
authentisierte neue Live-Aufträge, kombinierter Live-/Neustartablauf, Update-/Migrationspfad,
Installer-/Produktstart sowie externe COM-/Shell-/DNS-/Brokergrenzen fehlen weiterhin. DPAPI
attestiert kein kompromittiertes Windows-Konto; Container-Journale sind keine gegen kompromittierte
Containerprozesse unangreifbaren Speicher. Keine allgemeine Stromausfall- oder Crashgarantie für
jeden OS-Mikroschritt. Nächster Messschritt: frischer ETW-Netzvergleich exakt des gesicherten neuen
Paars sowie gesonderte Prüfung der DPAPI-/Domänendienstgrenze. Microsoft dokumentiert
Masterkey-Backup/-Wiederherstellung über Domänencontroller/RPC; ein unauffälliger Prozessbaum allein
reicht deshalb nicht für die Produktintegration. Vor Integration diese Grenze unter dem strikten
Ziel belegen oder die Primitive ersetzen, keine OS-/Domänenausnahme.
[Primärquelle](https://learn.microsoft.com/en-us/troubleshoot/windows-server/identity/dpapi-masterkey-backup-failures).
Striktes Datenschutzziel unverändert; #134 und Produkt-Q-01/Q-02 bleiben nicht bestanden.
