# P2c BP3: synthetischer Debug-Wächter und Gegenprobe der Vererbung

Stand: 18. November 2026. **Begrenzte native Probe bestanden; negative Machbarkeitsentscheidung:
`DEBUG_PROCESS` allein ist kein lückenloser präventiver Wächter. BP3 insgesamt bleibt offen.**
[Bereinigte Ergebnisse](product-start-p2c-bp3-debug-guardian.json),
[weiterhin verbindliche Startsperre](product-start-p2c-bp3-admission.md),
[Präloadvertrag](../data-format/product-start-p2c-c4-preload-v1-draft.md).

## Ergebnis

| Fall                                      | Native Prozesse im eigenen Job | Create-/Exit-Ereignisse beim äußeren Wächter | Ergebnis                                                                                                                                                                        |
| ----------------------------------------- | ------------------------------ | -------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Root beim ersten Create-Ereignis abweisen | 1 synthetischer Root           | 1 / 1                                        | Root vor erster User-Mode-Ausführung beendet, Exit 3, kein Workerbericht oder Kindmarker                                                                                        |
| Normale Debugvererbung                    | Root, Kind, zwei Konsolenhosts | 4 / 4                                        | Äußerer Wächter sieht die geprüfte Kindgeneration; Kindmarker und regulärer Abschluss                                                                                           |
| Kind mit eigenem `DEBUG_PROCESS`          | Root, Kind, zwei Konsolenhosts | 2 / 2                                        | Äußerer Wächter sieht Root und einen Konsolenhost; innerer Debugger sieht Kind und dessen Konsolenhost. Dieselbe zuvor vom Host gehaltene Kindgeneration führt ihren Marker aus |

Insgesamt **neun Jobprozesse: fünf synthetische Worker und vier Windows-Konsolenhosts**. Sieben
äußere Create-/Exit-Paare und zwei innere Create-Berichte; nach jedem Fall null aktive Jobprozesse.
Drei eigene Profile erfolgreich gelöscht und ihre Mapping-Abwesenheit unabhängig geprüft. Alle
finalen Fixtureverzeichnisse entfernt.

Die Gegenprobe ist kein AppContainer-, Job- oder Netzausbruch. Das Kind bleibt im eigenen Job und im
geprüften AppContainer ohne Capabilities; es entzieht sich **der äußeren Debugkette**. Ein Wächter,
der ausschließlich deren Create-Ereignisse als universelle Startschranke behandelt, könnte diese
Generation deshalb nicht vor deren Ausführung entscheiden. Daraus folgt die Ablehnung dieses
Mechanismus allein, keine Behauptung einer universellen Unmöglichkeit anderer Architekturen.

Microsoft dokumentiert diese bedingte Vererbung in
[Process Functions for Debugging](https://learn.microsoft.com/en-us/windows/win32/debug/process-functions-for-debugging).
Die [Debugereignisse](https://learn.microsoft.com/en-us/windows/win32/debug/debugging-events) halten
den betroffenen Prozess bis Continue an; das Create-Ereignis liegt vor erster User-Mode-Ausführung.
Der native Versuch bestätigt den konkreten Gegenfall auf diesem Rechner; das Urteil über seine
Eignung für BP3 ist unsere Schlussfolgerung.

## Getrennte Implementierung und Grenzen

[Modul/Frames](../../scripts/runtime-isolation/probe/src/windows/debug_guardian.rs),
[Worker](../../scripts/runtime-isolation/probe/src/windows/debug_guardian/worker.rs),
[nur im Test enthaltener Controller](../../scripts/runtime-isolation/probe/src/windows/debug_guardian/controller.rs).
Eigener Zweck `bp3-synthetic-debug-chain-v1`; weder Kontrollstore noch MAC-/Grantdomäne,
Runtime-/Datenautorität, Tauri-/DB-/Provider-/Projektcommand oder Änderung der C4-Sperre. Native
UI-ABI bleibt 0. Einzige Manifestergänzung ist das explizite `windows-sys`-Feature
`Win32_System_Diagnostics_Debug`; keine neue Paketabhängigkeit oder Lockfileänderung.

- Frisches UUID-Profil ohne Capabilities und eigener unbenannter Kill-on-close-Job ohne Breakaway.
  Forschungsbudget vier Prozesse: zwei feste synthetische Rollen plus höchstens zwei notwendige
  Konsolenhosts. Keine Änderung von BP1-/BP2- oder Produktbudgets. Der Versuch setzt keine
  Child-Restriction aus BP2 außer Kraft und startet keine WebView als Kompatibilitätsfallback.
- Gepinnte eigene Codekopie mit SHA-256-Abgleich, nur Lesen/Ausführen für das eigene Profil. Keine
  Hostordner-, Daten- oder Kontroll-ACL erweitert. Die vier erlaubten Rollenaktionen sind intern
  festgelegt: Root mit normaler Vererbung, Root mit neuer Debugkette, Blattmarker sowie Abweisung
  des Root durch den Controller. Keine frei angegebenen Zielprogramme oder Dateipfade.
- Root suspendiert mit atomarer Jobzuweisung, AppContainer und expliziter Liste von drei Pipe-Enden
  erzeugt. Eigener native Token-/Profil-/Job-/Imagepfad-/Generationsabgleich vor Resume. Der Host
  hält und prüft das gemeldete Kind einschließlich nativer Eltern-PID ebenfalls **vor** seiner
  Fortsetzung; erst ein an Nonce und Kindgeneration gebundener Go-Frame erlaubt Resume.
- Bootstrap/Go-Pipe-Erzeuger nativ geprüft, geerbte Handleflags gelöscht; beim Kindstart nur drei
  ausdrücklich gewählte Pipe-Enden. Der Worker weist direkte Hostaufrufe vor Handleübernahme ab.
  Keine permanenten Kontrollschlüssel oder Privilegienanhebung. Eigene Reports sind begrenzte
  Diagnose, keine eigenständig authentisierten nativen Startwitnesses oder Produktgrants.
- Längenpräfix u32 LE plus geschlossenes JSON, maximal 2 KiB. Zweck, Nonce, Rollen, Absender,
  Generationen, Einmaligkeit und erwarteter Endbericht geprüft; keine unbekannten Felder oder freien
  Aktionen. Der Blattmarker ist ein fester Pipe-Bericht, keine Schreibfreigabe auf eine Datei. Keine
  Test-DLL erforderlich, weil schon die Erzeugungskette die geforderte Vollständigkeit widerlegt;
  **keine präventive DLL-/TLS-/JIT-Abnahme** daraus.
- Geschlossener Start-Umgebungsblock: Windows-Verzeichnis, eigenes TEMP/TMP und für den nativen
  AppContainerstart benötigte Standard-Profilordner. Keine Übernahme von PATH, beliebigen
  Sitzungsvariablen oder API-Schlüsseln. Das Kind erbt diesen begrenzten Block. Standard-
  Profilpfade sind keine Delegation von Dateirechten oder echten Projekten.
- Windows erzeugt hier trotz `CREATE_NO_WINDOW` Konsolenhosts. Diese werden im reinen Versuch
  getrennt erfasst: konkretes gepinntes Systemimage, eigene Job-/Token-/Profilgrenze beim äußeren
  Wächter; entsprechender Image-/Tokencheck beim inneren Debugger. Das ist keine Hersteller-/
  Signaturausnahme oder zusätzliche Modulfreigabe im C4-Pfad. Ein innerer Report ersetzt keine
  unabhängige Hostattestierung seiner vollständigen Rollen-/Handlepolitik.

Fristen: acht Sekunden im Worker, zwölf im Controller; nach Fehler nur eigenes Jobende und
begrenztes Weiterbedienen eigener Debugereignisse. `DebugSetProcessKillOnExit(true)` und
Kill-on-close sind gesetzt. Beobachtete Enden zusätzlich an gehaltenen Root-/Kindhandles geprüft;
kein PID-Replay als Endbeleg. Nach erfolgter Prozesserzeugung ist die automatische Profillöschung
abgeschaltet; nur explizit nach bestätigtem Drain löschen, bei unklarem Ende eigenes Profil
behalten. Image-Handles aus Debugereignissen werden explizit geschlossen, die
Debug-Prozess-/Threadhandles gemäß
[WaitForDebugEvent](https://learn.microsoft.com/en-us/windows/win32/api/debugapi/nf-debugapi-waitfordebugevent)
behandelt. **Ein tatsächlicher Wächter-Crash wurde nicht separat ausgelöst**; konfigurierte
Abbruchregeln sind keine neue Crash-Abnahme.

## Prüfung, Speicherung und Reproduktion

Drei neue aktive Regressionen: begrenztes Framing/Unbekanntwerte/Trunkierung, direkter Hostaufruf
ohne Autorität und falsche Generation/Nonce/doppelter Marker. Insgesamt **141 aktive
Helfer-/Transporttests bestanden** (134 Binär-/sieben Transporttests), sieben standardmäßig
ignoriert. Clippy alle Targets mit Warnungen als Fehler, rustfmt und Helferbuild erfolgreich. Der
neue ignorierte native Test wurde am finalen Paar mit genau den drei obigen Fällen ausgeführt (1,90
Sekunden). Keine große Crashmatrix, kein App-/Installerbuild oder `pnpm check` für diese isolierte
Forschung gemäß mittlerem Prüfniveau.

Zwei zusätzliche direkte Hostaufrufe des neuen normalen Helfers mit fehlenden bzw. numerischen
Handlebehauptungen enden mit Exit 2 und ohne Ausgabe; kein Eintritt in generische
Netzwerkdiagnostik.

Unabhängiger Nur-Lese-Abgleich bestätigt identische Kindgeneration in Hostprüfung, innerem
Create-Bericht und Blattmarker, fehlende äußere Kindaufnahme, stimmige Prozess-/Endanzahlen und
Abwesenheit der eigenen Profile/Fixtures. Frühere Entwicklungsversuche mit Image-Lock,
unvollständiger Konsolenhostklassifikation und unvollständigem Start-Umgebungsblock gelten nicht als
bestandene Abnahme. Fünf eigene Entwicklungsreste erst nach Prozessende im exakt geprüften
TEMP-Bereich ohne rekursive Löschung separat entfernt; kein Produktcleanup daraus.

Exaktes Paar geschützt lokal außerhalb von TEMP unter `artifacts/bp3-debug-20261118-01/` gesichert:

- Normaler Helfer: `56a1b43db1b9488bea8a26e1bb67e92732db15daaa82e576821698a1173e8d61`.
- Tatsächlicher Testcontroller: `e1543737b8a6b84b776b287efe0f7ce6e5b84ee86bc1139587ecc8034354bf89`.
- Privater `run.json`, unabhängiger Audit und Manifest mit Quellhashes. PIDs, native Zeiten, Nonces,
  Profil-/TEMP-Pfade bleiben lokal. Frühere BP2-/BP3-Bytepaare bleiben unverändert.

Reproduktion ausschließlich mit eigener frischer Ausgabe und synthetischem Helfer:

```powershell
cargo build --locked --offline --manifest-path scripts/runtime-isolation/probe/Cargo.toml
$probeOutput = Join-Path $env:TEMP 'kraqel-bp3-debug-build-repro'
New-Item -ItemType Directory -Path $probeOutput -ErrorAction Stop | Out-Null
Copy-Item -LiteralPath scripts/runtime-isolation/probe/target/debug/kraqel-isolation-probe.exe -Destination (Join-Path $probeOutput 'controller.exe')
$env:KRAQEL_BP3_DEBUG_OUTPUT = $probeOutput
try {
    cargo test --locked --offline --manifest-path scripts/runtime-isolation/probe/Cargo.toml windows::debug_guardian::controller::bp3_debug_chain_native_series -- --ignored --exact --nocapture
    if ($LASTEXITCODE -ne 0) { throw 'Native BP3-Probe fehlgeschlagen' }
    if (-not (Test-Path -LiteralPath (Join-Path $probeOutput 'run.json'))) { throw 'Kein neuer Prüfbericht' }
} finally {
    Remove-Item Env:KRAQEL_BP3_DEBUG_OUTPUT
}
```

Genau ein nativer Test muss bestanden sein; Nulltest-Erfolg zählt nicht. Die Ausgabevariable wird
nur vom ignorierten Test gelesen, nicht vom nativen Worker oder Produkt. Ein Testverzeichnis muss
direkt unter TEMP liegen und dem festen Namenspräfix entsprechen. Wenn die EXE nach belegtem
Job-/Prozessende noch gesperrt ist, zeigt der private Bericht `fixtureRemoved: false`; dann nur das
dort protokollierte eigene Verzeichnis nach Ende des Debuggerprozesses geprüft entfernen. Nicht
wahllos TEMP/Profile bereinigen oder fehlende Abschlussbelege durch Dateilöschung ersetzen.

**Kein ETW dieses neuen Paars**, keine Aussage über dessen beobachtete Netzereignisse oder fremde
Systemdienste. Die Probe verwendet keine Netzwerk-APIs, aber Codeinspektion und fehlende
Netz-Capabilities ersetzen keinen frischen Netzbeleg. Alte Messungen bleiben bytegebunden.

## Konsequenz und nächster Scope

Die bisherige C4-Startsperre bleibt unverändert zu. Kein weiteres WebView-/DLL-Experiment unter der
Annahme, die äußere Debugkette sei vollständig; kein zusätzlicher Testumfang repariert den
beobachteten strukturellen Gegenfall. BP3, BP4/C4c/d und Produkt-Q-01/Q-02 bleiben offen.

Nächster sinnvoller Scope ist ein begrenzter Architekturentscheid für einen **unabhängig
durchgesetzten Erzeugungsweg**: konkrete Mechanismen müssen auch selbst eröffnete Debugketten,
fremde/nachgeladene Module und den frühen Wächterstart abdecken. Eine reine Pipe-Brokeridee ohne
vorherige Sperre direkter Runtime-Kindstarts genügt ebenfalls nicht. Keine automatische Wahl eines
neuen UI-Stacks, keine VM-/Treiberinstallation, Systempolicy, Schutzsoftwareänderung oder
Signaturpflicht. Das strikte Datenschutzziel bleibt unverändert.
