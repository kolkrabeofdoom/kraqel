# KQ-069 A: UI-/Tauri-Testanbindung und Einweg-Dateikanal

Stand: 15. November 2026. PR #139 nach grüner Windows-/Linux-CI und Dependency Review am exakten
Head `2ac11a2` zusammengeführt (`fadf0a6`). Pflichtenheft Revision 1 §1/§5/§10, ADR 0034 Richtung A
und striktes Datenschutzziel unverändert. #134 bleibt offen.

## Implementiert

Das explizite Windows-Feature `isolation-ui-test` startet ein eigenes Forschungsfenster mit echtem
WebView, Tauri-IPC und dem bestehenden nativen Snapshotimport. Der normale App-Builder registriert
diese Befehle nicht. Der Forschungsbuild verweigert den direkten unisolierten Start ohne gültigen
Launcherauftrag; beobachtet: Exitcode 2 vor Fenster-/App-Dateninitialisierung. Normale
Debug-Anwendung wieder gebaut; deren Dist enthält die Forschungsseite nicht.

Der Launcher gibt eine feste synthetische Datei vorab frei, wahlweise per nativer Auswahl
(`ui-select`) oder explizitem Fixturemodus (`ui-fixture`). Er übergibt ausschließlich zwei gezielt
vererbte Handles: schreibgeschützte Eingabedatei und Leseende einer anonymen Pipe. Der Sender
schließt das Schreibende vor Prozessstart. Keine Ports, veröffentlichte Endpunkte, Hostordner-ACLs
oder zusätzlichen Netzfähigkeiten. Parser/Nutzbytes bleiben im Container.
[Genauer Vertrag, Bindung und Grenzen](../data-format/native-file-channel-v1.md).

Der App-Prozess wird suspendiert auf eigene AppContainer-SID und null Capabilities geprüft, vor
Fortsetzung dem Job zugeordnet und prüft seinen Token nochmals. Der Empfänger validiert Frame/EOF,
Typen, Aliasierung, Metadaten, ID, Version, Budget und Lebensdauer. Ein atomarer Empfängeranspruch
verhindert wiederholte Initialisierung. Vor dem WebView-Start werden die Vererbungsflags gelöscht
und zurückgelesen; die Pipe wird nach einem Frame geschlossen. Die UI bekommt keine Handles,
Aufträge oder Hostpfade. Ihr Import entnimmt den Rust-eigenen Grant einmalig unter Mutex;
zusätzliche Invoke-Felder können die Freigabe nicht ersetzen.

Nur vier parameterlose Testbefehle sind registriert: Status, Import, Abschluss und Abbruch. Jeder
prüft Fensterlabel und gebündelte Origin/Seite; andere Fenster, Seiten, Remote- und gewöhnliche
Localhost-Origin sind nicht autorisiert. Das Testfenster erhält keine Core-/ Plugin-Berechtigungen.
Sein Import hält zu Testzwecken unmittelbar vor Veröffentlichung an. Abbruch verwirft Staging; nach
Commitfreigabe gilt weiterhin keine Rücknahmegarantie. Fensterschließen während des Imports fordert
Abbruch an und verzögert den Prozessausstieg bis zum Ergebnis. Der Controller begrenzt die Sitzung
auf zehn Minuten und wartet nach Jobende auf Nullprozesse vor Profilentfernung. Harte
Absturz-Reconciliation bleibt offen.

## Finale native Nachweise

Der Nutzer bediente drei frische Sitzungen des finalen Forschungsbuilds, weil Computer Use
einschließlich Neuinitialisierung mit `Transport closed` ausfiel. Alle Sitzungen verwendeten
`ui-fixture`: vorab fest freigegebene synthetische Datei, kein neuer nativer Auswahldialog.

| Ablauf                                         | Bestätigtes Ergebnis                                                                         |
| ---------------------------------------------- | -------------------------------------------------------------------------------------------- |
| Import, Abschluss, Wiederholung, Fensterende   | genau eine neue Kopie mit zwei Dokumenten; erneuter Aufruf mit `ALREADY_CONSUMED` abgewiesen |
| Import, Abbruch am Haltepunkt, Fensterende     | `CANCELLED`, keine Projektkopie und kein Stagingrest                                         |
| Import, Fenster direkt am Haltepunkt schließen | nativer Abbruch vor Prozessende, `CANCELLED`, keine Kopie und kein Stagingrest               |

In allen drei Fällen hat WebView-JavaScript den echten Tauri-Statusbefehl aufgerufen. Root-Tokens
vor Fortsetzung auf erwartete SID/null Capabilities geprüft; Empfänger prüft nochmals und liest
gelöschte Vererbungsflags zurück. Keine vollständige Token-/Handleattestierung jedes WebView-
Nachfahren behauptet. Null aktive Jobprozesse und alle drei Profile entfernt.

`verify_tauri_ui.py` prüft nach Prozessende unabhängig: neue ID und genau eine Kopie im Erfolgsfall,
exakter Unicode-Text, `integrity_check = ok`, keine Fremdschlüsselfehler und null Verzeichnisreste
in beiden Abbruchfällen. Die Wiederholungsablehnung bestätigt der Nutzer wörtlich; nachfolgend
bestätigt er jeweils Fensterende, Abbruch und direktes Schließen am Haltepunkt.

App-SHA-256 aller drei gestagten finalen Builds:
`d6728209b8d261a283cbb08cea6944bd5fd7d63ef1aa5d2aea4bbab904d0e4ee`. Controller-SHA im
[bereinigten Ergebnis](native-tauri-file-channel.json). Kein Rust-Code nach der Messung geändert.
Der frühere Vorversuch und die unvollständige Aufzeichnung ohne gestartete App werden nicht als
finale Abnahme verwendet.

## ETW des finalen Builds

15. November 2026, **16:43:07–16:46:11 UTC**, nach erneuter manueller UAC-Bestätigung. ETL:
    1.114.112 Byte, Rohdaten lokal. Drei separat beim Start notierte Controller samt
    App/WebView-Baum: 3 Controller, 3 App-, 18 WebView- und 6 Console-Host-Lebenszeiten, vollständig
    beendet. Vier beobachtete TCP-/UDP-Kontrollen für IPv4/IPv6, Nullverluste, keine unbekannten
    Ziele oder Zuordnungen: **0 Ereignisse außerhalb Loopback**. 26 Loopback-Disconnect-Ereignisse
    werden getrennt ausgewiesen; keine HTTP-/Upload- oder universelle Netzstilleaussage daraus.

Die Messung gilt nur für diese synthetischen UI-Pfade und den erfassten Prozessbaum. Externe
COM-/Shell-Dienste und kompromittierte Runtime-Prozesse sind damit nicht vollständig abgenommen.
Native Dateiauswahl gehört nicht zu dieser finalen Serie. Historische und unvollständige Traces
ersetzen keine fehlenden Pfade; der strenge Auswerter verweigert diese weiterhin.

## Automatische Prüfungen und verbleibende Grenze

`pnpm check`, Feature-Clippy mit Warnungen als Fehler, normaler und Forschungs-Tauri-Build
erfolgreich. 29 Browser-E2E einschließlich zwei neuer UI-Abläufe bestanden; diese verwenden nur den
Browseradapter, keine native Isolation. Vier Transporttests prüfen Framegrenzen/EOF, unerlaubte
Felder, Bindung/Budget/Ablauf und einmaligen Empfängeranspruch. 13 bestehende Helfertests plus
Controllerantwort-Test bestehen. Ein Tauri-Featuretest prüft Origin/Fenster. 32 Python-Diagnosetests
bestehen; `isolation-ui` verlangt pro Controller App und WebView sowie unverändert Kontrollen,
vollständige Enden und Nullverluste.

Normaler Debug-Build wiederhergestellt, seine Dist enthält keine Forschungsseite. Keine regulären
App-Daten geöffnet. Native Vorab-Dateiauswahl, assistive Hörabnahme, Auswahl aus einer laufenden App
und dauerhafter Auftrags-/Crash-Lebenszyklus sind weiter offen. Keine Freigabe realer Manuskripte
oder produktiver Launcher. **Q-01/Q-02 bleiben für das Produkt nicht bestanden.**

## Reproduzieren und fortsetzen

```powershell
$env:KRAQEL_ISOLATION_UI_TEST = '1'
pnpm --filter @kraqel/desktop tauri build --debug --no-bundle --features isolation-ui-test
Remove-Item Env:KRAQEL_ISOLATION_UI_TEST
# Forschungs-EXE vor Wiederherstellung des normalen Builds außerhalb target sichern.
cargo build --locked --manifest-path scripts/runtime-isolation/probe/Cargo.toml
$uiRoot = Join-Path $env:TEMP ('kraqel-isolation-ui-' + [guid]::NewGuid().ToString('N'))
scripts/runtime-isolation/probe/target/debug/kraqel-isolation-probe.exe ui-select $uiRoot <Forschungs-EXE>
```

Nur `input.kraqel` im frischen synthetischen Dialogordner auswählen. Ab Auswahl/Grant-Erzeugung
höchstens 60 Sekunden bis zur Commitfreigabe. UI: importieren, abschließen oder abbrechen, erneuten
Aufruf prüfen, Fenster schließen. Für jeden Fall eine neue Wurzel. `ui-fixture` verwendet denselben
Kanal ohne erneute Auswahl. Keine realen Manuskripte auswählen.

Nach neuer ETW-Aufzeichnung die separat notierten Controller-PIDs im Modus `isolation-ui` auswerten.
`verify_tauri_ui.py <lokales Journal> --trace <ETW-Wurzel>` prüft Import (`import`), Abbruch
(`cancel`) und Fensterende während des Haltepunkts (`close`) unabhängig. Nur tatsächlich gemessene
Ergebnisse fortschreiben. Normalen Debug-Build ohne Forschungsfeature anschließend wieder bauen.

Weitere Produktarbeit: Auswahl aus einer bereits laufenden regulären App, dauerhafter sicherer
Auftrags-/Replay-/Crash-Lebenszyklus, echte Nutzdateien, übrige Formate/Providergrenzen und
Produkt-/Installerintegration. Kein Nachweis gegen kompromittierte Runtime-/OS-Prozesse oder externe
COM-/Shell-Dienste. **Q-01/Q-02 für das Produkt bleiben nicht bestanden.**
