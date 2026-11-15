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

## Nachweise und ausdrücklich offene Abnahme

Ein nativer Vorversuch wurde manuell durch den Nutzer bedient, weil Computer Use einschließlich
Neuinitialisierung mit `Transport closed` ausfiel. WebView-JavaScript hat den echten Tauri-
Statusbefehl aufgerufen; Import und Abschluss erzeugten eine Projektkopie mit zwei Dokumenten. Der
Nutzer meldet die sichtbare Wiederholungsablehnung: „Dieser Auftrag ist bereits verbraucht. Es wurde
kein zweites Projekt angelegt.“ Anschließend bestätigt er das Fensterschließen.

Unabhängige Nur-Lese-Prüfung nach Prozessende: neue Projekt-ID, exakter Unicode-Text,
`integrity_check = ok`, keine Fremdschlüsselfehler, genau eine Projektkopie, kein Stagingrest.
Root-AppContainer mit null Capabilities, null aktive Jobprozesse, Profil entfernt. Verifizierer:
`scripts/runtime-isolation/verify_tauri_ui.py`, lokales Journal `{scenario, root, pid}` ohne Git.

**Dieser Vorversuch ist nicht die vollständige native Abnahme des finalen Standes.** Danach wurden
Empfängerinitialisierung, Startup-Freigabeprüfung, Jobbereinigung und Controllerantwort- Schema
verstärkt. Die abschließenden Guards sind automatisch geprüft. Der native Vorversuch hat App-SHA-256
`ce199de8d453c9d39765805d59d9d1268aa03384f99f22285b553de296466b57`. Der finale Forschungsbuild hat
SHA-256 `d6728209b8d261a283cbb08cea6944bd5fd7d63ef1aa5d2aea4bbab904d0e4ee`. Native Wiederholung des
finalen Builds, Abbruch/Fensterende am Haltepunkt und neue Dateiauswahl bleiben offen; keine
assistive Hörabnahme behauptet.

`pnpm check`, Feature-Clippy mit Warnungen als Fehler, normaler und Forschungs-Tauri-Build
erfolgreich. 29 Browser-E2E einschließlich zwei neuer UI-Abläufe bestanden. Diese beiden nutzen
ausdrücklich nur den Browseradapter, keine native Isolation. Vier Transporttests prüfen
Framegrenzen/EOF/unerlaubte Felder, Bindung/Budget/Ablauf und einmaligen Empfängeranspruch; 13
bestehende Helfertests bestehen. Ein Tauri-Featuretest prüft die Origin-/Fenstergrenze. 32
Python-Diagnosetests bestehen; neuer Modus `isolation-ui` verlangt für jeden Controller einen echten
App- und WebView-Baum sowie unverändert Kontrollen, vollständige Enden und Nullverluste.

**Kein neuer ETW-Nachweis:** Windows meldet den angeforderten Helferstart als durch den Benutzer
abgebrochen; `ready.json` wurde nicht erzeugt. Die Frage nach einer erneuten UAC-Abfrage ist offen.
Kein Trace gestartet und keine Aussage über Netzstille dieses UI-Builds. Historische Messungen aus
#137–#139 ersetzen diese fehlende Messung nicht. Rohdiagnosen/Prozessdetails bleiben lokal.

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
(`cancel`) und Fensterende während des Haltepunkts (`close`) unabhängig. Erst tatsächliche
Ergebnisse fortschreiben. Normalen Debug-Build ohne Forschungsfeature anschließend wieder bauen.

Weitere Produktarbeit: Auswahl aus einer bereits laufenden regulären App, dauerhafter sicherer
Auftrags-/Replay-/Crash-Lebenszyklus, echte Nutzdateien, übrige Formate/Providergrenzen und
Produkt-/Installerintegration. Kein Nachweis gegen kompromittierte Runtime-/OS-Prozesse oder externe
COM-/Shell-Dienste. **Q-01/Q-02 für das Produkt bleiben nicht bestanden.**
