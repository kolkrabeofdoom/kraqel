# AppContainer-Forschungshelfer für KQ-069

Neuester Stand: [Sichere Paketartefakte/Retry](../../docs/testing/native-package-artifact-retry.md).
`check-update-reconciliation.ps1 -RetryAborted` mit explizit gesicherten neuen Programmhashes prüft
drei frische synthetische Fälle: Teilkopie, Manipulation, vollständiger Abort. Gleiches Pending nach
Abbruch sicher erneut vorbereiten; alte Köpfe/gesperrte Dateien verändern nichts. 56 aktive
Helfertests/Clippy und unabhängiger Daten-/MAC-Audit grün. Neue Brokerbytes ohne eigenen frischen
ETW-Beleg; vorherige Netzbelege nicht übertragen. Unbekannte/aktivierte Pakete bleiben geschützt.

Neuester gezielter Prüfstand:
[Update-Abbruch-/Fehlerreconciliation](../../docs/testing/native-update-abort-reconciliation.md).
`check-update-reconciliation.ps1` verwendet explizit hashgeprüfte gesicherte Programme und erzeugt
drei frische synthetische TEMP-Cases: Teilkopie, manipuliertes Paket und Prozessende nach
veröffentlichtem Abort. Gleiche Kontrollsperre, alter Actor und bestehende Importaufgabe bleiben
gebunden; keine Kettenreparatur. 53 aktive Helfertests/Clippy und Daten-/MAC-Audit grün. Keine
Runtimeänderung oder neuer Netzbeleg dieser zusätzlichen Fehlerläufe. Paketartefakt-Cleanup und
transparenter Retry bleiben offen. Reproduktion/Hashes stehen im verlinkten Nachweis.

Aktuell: [Startticket v3](../../docs/data-format/native-ui-restart-v3.md) und
[Brokerkette v3](../../docs/data-format/native-broker-recovery-v3.md).
`ui-combined-series <frische Controllerwurzel> <Forschungs-App>` prüft neun Live-/Wiederanlauf-/
Folgeauftragsfälle, einschließlich Abbruch, ungültigem Archiv und Teiltransfer. Signierte private
Abschlussbelege sind Voraussetzung für eine ausdrücklich neue native Beauftragung.
`ui-broker-live-seed <frische Neustartwurzel> <App> prepared native` benötigt manuelle Auswahl von
`input.kraqel`; `ui-broker-live-next <Wurzel>` erteilt ausdrücklich einen neuen Fixture-Auftrag.
Kein automatischer Retry, keine Altdatenmigration oder neue Rendererbefehle.
[Finaler Nachweis, Buildschalter und Hashgrenzen](../../docs/testing/native-combined-live-restart.md).
[Eigener frischer ETW-Beleg](../../docs/testing/native-combined-live-restart-etw.md) für genau das
gesicherte Paar mit Automatik und tatsächlicher nativer Auswahl bestanden. Keine Übertragung auf
Folgeänderungen; normale Starts bleiben unisoliert. Historische Stufen folgen.

Historischer [Broker-Vertrag v2](../../docs/data-format/native-broker-recovery-v2.md): lokaler
HMAC-Beleg/privater Schlüssel ersetzen DPAPI. Neuer Seed in frischer Wurzel erforderlich; keine
Altdatenmigration oder Schlüssel-/Journalneuanlage beim Wiederanlauf. Native Serie jetzt mit 78
Ablehnungen und sechs tatsächlichen Dateiöffnungsverweigerungen pro Appstart.
[Nachweis und gesicherte Hashes](../../docs/testing/native-hmac-broker.md).
[Eigener gültiger ETW-Beleg](../../docs/testing/native-hmac-broker-etw.md) des gesamten Serienbaums
einschließlich negativer Brokerstarts; keine Übertragung früherer Paarabnahmen oder auf
Folgeänderungen.

Neu: `ui-controller-crash-series <frische kraqel-isolation-controller-UUID-Wurzel> <Forschungs-EXE>`
prüft sechs harte Controller-Kills, zwölf Wiederanläufe in neuen Brokerprozessen und 78
Ablehnungsfälle. Nur dieser explizite Brokerpfad erlaubt dauerhafte Übernahme einer bestehenden
synthetischen Ablage nach HMAC-/Hash-/Wurzelbindung, exklusiver Sperre und Jobleerstand.
[Vertrag](../../docs/data-format/native-broker-recovery-v2.md),
[Nachweis und exakte Buildgrenzen](../../docs/testing/native-hmac-broker.md).
`ui-broker-seed <frische Neustartwurzel> <Forschungs-EXE> <Crashpunkt>` beendet sich absichtlich mit
Code 86. `ui-broker-recover <bestehende Neustartwurzel> first|second|retire` verlangt den passenden
gesicherten Broker und gibt niemals eine neue Importkennung frei. Separat gesichertes Paar mit
eigenem begrenztem ETW-Beleg, kein fertiger Produktbroker; normale Starts bleiben unisoliert.

Aktueller Startticketvertrag: [v3](../../docs/data-format/native-ui-restart-v3.md), ohne Downgrade.
`ui-assistive-interrupted` beziehungsweise `ui-assistive-completed` mit frischer gebundener
Neustartwurzel und passender Forschungs-EXE bereiten einen synthetischen Crash vor und lassen den
Wiederanlauf offen. Tatsächliches Replay und reguläres Schließen erforderlich, höchstens 600
Sekunden. [Assistiver Nachweis](../../docs/testing/native-assistive-restart.md) und
[eigener frischer v2-Netzbeleg](../../docs/testing/native-assistive-restart-etw.md) mit beiden
manuellen Pfaden. Gilt nur für die dokumentierten Hashes; v1-Belege bleiben separat gültig.

`ui-crash-series <frische kraqel-isolation-ui-restart-UUID-Wurzel> <Forschungs-EXE>` führt autonom
28 Generationen derselben isolierten GUI mit stabiler Identität/Ablage aus. Frische gebundene
Starttickets, sechs harte Crashpunkte, je zwei Reconciliations, Altimportabwehr und vier negative
Starts. Nur eingebettete synthetische Fixture, kein Dateidialog; Forschungsfenster nicht bedienen.
[Kanal und Grenzen](../../docs/data-format/native-ui-restart-v1.md),
[Reproduktion und Nachweis](../../docs/testing/native-ui-restart.md).
[Frischer ETW-Beleg nur für das dokumentierte Hashpaar](../../docs/testing/native-ui-restart-etw.md).
Kein Produktlauncher; dieser ältere Serienpfad darf keine vorhandene Wurzel übernehmen. Dafür ist
ausschließlich der neue authentisierte Brokerpfad vorgesehen.

`ui-live-transfer-hold <neue TEMP-Forschungswurzel> <Forschungs-EXE>` hält nach dem nativen
Dateidialog und genau 1.024 übertragenen Archivbytes. Sender- und Empfängerzähler bleiben lokal.
Innerhalb von 60 Sekunden das Hauptfenster schließen, ohne eine Veröffentlichung auszulösen. Der
normale `ui-live`-Modus enthält diesen Transferhaltepunkt nicht.
[Fensterende-Nachweise und Buildgrenzen](../../docs/testing/native-live-window-close.md).

`ui-live <neue kraqel-isolation-ui-Wurzel> <Forschungs-EXE>` startet zuerst das Forschungsfenster.
Erst der Importknopf fordert die native Auswahl der synthetischen `input.kraqel` an. Zwei anonyme
Pipes, ein festes Auswahlsignal und ein Versuch pro Sitzung.
[Vertrag](../../docs/data-format/native-live-file-channel-v2.md) und
[Nachweise/offene Abnahme](../../docs/testing/native-live-file-selection.md). Kein neuer
Netzwerkbeleg oder produktiver Broker.

**Nur synthetische Daten. Kein produktiver Launcher und keine Sicherheitsfreigabe.** Der normale
KRAQEL-Start wird nicht verändert. Ergebnis und Grenzen stehen im
[Versuchsprotokoll](../../docs/testing/native-runtime-isolation.md).

Neu: [Tauri-Testanbindung](../../docs/testing/native-tauri-file-channel.md) und
[Einwegkanal](../../docs/data-format/native-file-channel-v1.md). Das Tool stellt zusätzlich eine
Transportbibliothek ausschließlich für das explizite Windows-Feature `isolation-ui-test` bereit. Sie
bleibt außerhalb des Produkt-Workspace; Produktcrates behalten `unsafe_code = forbid`. Keine neuen
Registrypakete, nur vorhandene Serde-/Win32-Funktionen. Normale Builds binden die Bibliothek nicht
ein und enthalten die Forschungsseite nicht. Finale synthetische UI-Serie mit Import/Wiederholung,
Abbruch und Fensterende am Haltepunkt einschließlich gültigem ETW bestanden. Native Vorabauswahl und
Produktabnahme bleiben offen; kein universeller Netzstillebeleg.

`ui-select <neue kraqel-isolation-ui-Wurzel> <gesicherte Forschungs-EXE>` wählt zuerst die feste
synthetische Datei und startet danach das isolierte Fenster. `ui-fixture` überspringt den Dialog.
Kein Rendererpfad und kein Container-zu-Host-Auftragskanal. Native Bedienung derzeit durch den
Nutzer, weil Computer Use einschließlich Neustart mit `Transport closed` ausfällt. Nur tatsächliche
Ergebnisse protokollieren. `isolation-ui` verlangt im ETW-Auswerter pro Controller App und WebView
sowie unverändert Kontrollen, vollständige Lebenszeiten und Nullverluste.

Der neue einmalige `exchange`-Modus untersucht zusätzlich native Shell-Auswahl und zwei gezielte
Dateihandles.
[Nachweis, Bedrohungsgrenze und ETW-Buildabgrenzung](../../docs/testing/native-runtime-file-exchange.md).

## Mechanismus und Abgrenzung

Der eigenständige Rust-Helfer nutzt dokumentierte Win32-AppContainer-APIs: pro Versuch eine eigene
Profilidentität, null Capabilities, `CREATE_SUSPENDED`, Tokenprüfung vor `ResumeThread` und ein Job
Object mit `KILL_ON_JOB_CLOSE` ohne Breakaway-Freigabe. Es gibt keinen automatischen unisolierten
Fallback. Die Root-Identität ist vor Ausführung geprüft; Nachfahren-Tokens werden zusätzlich
stichprobenartig gelesen. Das ist keine vollständige Attestierung jedes Nachfahren vor dessen erstem
Befehl.

Nur ein neuer direkter TEMP-Unterordner `kraqel-isolation-…` wird akzeptiert. Der Helfer kopiert das
Testprogramm nach `bin`, ergänzt für die eigene Paket-SID Lese-/Ausführungsrechte und gibt
ausschließlich `data` zusätzliche Schreib-, Umbenennungs- und Löschrechte mit Low-Integrity-Label.
Vorhandene DACL-Einträge bleiben erhalten; Workspace, Benutzerprofil und Systemdateien erhalten
keine zusätzlichen Freigaben. Kein Firewallregel-, Loopback-Ausnahme-, OS-Diagnose- oder
Sicherheitsdienstwechsel. Kein `internetClient` und kein Abschalten von TLS oder Browser-Sandbox.

Das Tool ist ein separates Cargo-Workspace unter `probe/`, damit die erforderliche kleine
Win32-FFI-Grenze nicht in die Produktcrates gelangt. Deren `unsafe_code = forbid` bleibt bestehen.
`windows-sys 0.61.2` und `serde_json 1.0.154` sind bereits im Produkt-Lockfile vorhanden, beide MIT
OR Apache-2.0; im Tool separat gelockt. Keine neue Produktabhängigkeit. Dieses Forschungswerkzeug
ist GPL-3.0-or-later, wird nicht in Installer aufgenommen und ersetzt keine Prüfung eines künftigen
Produktlaunchers.

Der Austausch verwendet außerdem die bereits im Produkt-Lockfile vorhandenen `windows 0.62.2`
(dokumentierte COM-Shell-Schnittstellen) und `sha2 0.10.9` (Worker-Hash), beide MIT OR Apache-2.0.
Separat gelockte, bestehende Windows-Rust-/RustCrypto-Abhängigkeiten; keine neue
Produktabhängigkeit, kein Provider, Netzclient oder Laufzeitdownload. Der eigenständige
Tool-Lockfile löst transitive Versionen unabhängig vom Produkt auf. Lizenz-/Compileprüfung lokal und
Dependency Review in CI; keine zusätzliche Sicherheitszertifizierung behauptet.

## Synthetischer Datei-/Shell-Austausch unter Windows

```powershell
$exchangeRoot = Join-Path $env:TEMP ('kraqel-isolation-exchange-' + [guid]::NewGuid().ToString('N'))
cargo run --locked --manifest-path scripts/runtime-isolation/probe/Cargo.toml -- exchange $exchangeRoot
```

Im Öffnendialog `input.txt` wählen, im Speicherdialog die vorgeschlagene neue `output.txt`
speichern. Nur selbst erzeugte Testdateien im separaten `kraqel-exchange-host-…`-Geschwisterordner
werden akzeptiert; keine realen Manuskripte auswählen. Bestehende Ziele werden auch nach einer
Shell-Überschreibbestätigung abgelehnt. `Escape` bricht vor dem Workerstart ab. Der Bericht enthält
keine Hostpfade; `launch.json` mit PID bleibt lokal. Testfixtures werden zur Inspektion erhalten.
Diese Stufe transportiert nur 1 Byte bis 8 MiB, prüft keine Produktformate und öffnet keine
beliebigen Ordner/URLs/Programme. Erfolgreicher Exitcode 0 ist eine Transportprüfung, keine
Produkt-Netzfreigabe. Der `exchange`-Controller führt die Netz-Positivkontrollen des anderen
`network`-/`app`-Modus nicht aus.

Nach neuer ETW-Aufzeichnung mit dem bestehenden Diagnosehelfer kann dessen `events.xml` mit
`scripts/m3-network/analyze.py <events.xml> <ready.json> --root-pid <BrokerPID> --mode exchange`
ausgewertet werden. Der Standardmodus bleibt `desktop` und verlangt weiter einen WebView2-Baum. Der
Austauschmodus verlangt den Broker `kraqel-isolation-probe.exe` samt Worker, vollständige
Lebenszeiten, vier Kontrollen und Nullverluste. COM-/Shell-Dienste außerhalb des Prozessbaums sind
damit nicht kausal erfasst. Kein universeller Offlinebeleg.

## Typisierter nativer App-Dienst-Testadapter

[Vertrag und Nachweis](../../docs/testing/native-isolated-app-import.md). Der separate
Workspace-Baustein `apps/desktop/isolation-test-adapter` verwendet normale Import-/Storage-Dienste
mit `unsafe_code = forbid`, ohne Tauri/WebView. Die Produkt-App bindet ihn nicht ein. Keine neuen
Registry-Abhängigkeiten im Produkt; das Tool bindet vorhandene Produktcrates/UUID ein. Gemeinsame
Versionen aus dem Produkt-Lockfile übernommen, insbesondere `zip 9.5.1`, `rusqlite 0.39.0`,
`image 0.25.9`. Keine Runtime-Downloads oder Git-/Provideraufrufe im Importpfad. Transitive Lizenzen
entsprechen den vorhandenen Produktabhängigkeiten.

```powershell
python scripts/runtime-isolation/make_import_fixture.py
cargo test -p kraqel-isolation-test-adapter
cargo build --locked --manifest-path scripts/runtime-isolation/probe/Cargo.toml
$importRoot = Join-Path $env:TEMP ('kraqel-isolation-appimp-' + [guid]::NewGuid().ToString('N'))
scripts/runtime-isolation/probe/target/debug/kraqel-isolation-probe.exe app-import-fixture $importRoot valid
```

Weitere feste Fälle: `invalid`, `expired`, `cancelled`, `cancel-at-publish`, `expire-at-publish`,
`wrong-id`. Pro Aufruf neue Wurzel. Ausschließlich eingebettete Testfixture, **keine nativen Dialoge
oder frei wählbaren Nutzdateien**. Vor Dispatch entsteht im Container eine synthetische Testbasis.
Erwartete Ablehnungen ergeben eine erfolgreiche Harnessprüfung (`passed: true`), aber keine
Importkopie; den Importstatus zeigt `worker.error` beziehungsweise `worker.receipt`.

Auftrags-/Antwortgrenzen: 2/4 KiB. Antwort nur neue Projekt-ID, Zähler, Größe/Hash oder stabile
Codes. 8-MiB-Forschungsrahmen ersetzt keine Produktbudgets. Abbruch/Ablauf nach Initialisierung
verwerfen Staging; nach Commitfreigabe ist Abbruch zu spät. Kein automatischer Retry oder dauerhafte
Crash-/Replay-Reconciliation. `worker-lifecycle.json`, `launch.json`, eventuelle bereinigte
`child-error.json` und Rohdiagnosen bleiben lokal. `exchange-fixture <neue Wurzel>` prüft die
bisherige Kopie mit festem Input/Output ohne Dialog; `exchange` behält native Auswahl.

`verify_app_import.py <lokales runs.json> <ETW-Wurzel>` prüft die acht dokumentierten Fälle über
SQLite im Nur-Lese-Modus, Hashes und den ETW-Auswerter (`exchange`). Das lokale Journal ist eine
Liste `{scenario, root, pid}` aus `Start-Process -PassThru`, kein Git-Artefakt. Nur bereinigte
Resultate veröffentlichen. Sieben Adaptertests laufen mit `cargo test --workspace`; 13 Helfertests
weiterhin separat in CI. Funktionstests brauchen keine Administratorrechte und ändern keine
Firewall-/Sicherheitsregeln.

## Bauen und automatische Prüfungen

```powershell
cargo build --locked --manifest-path scripts/runtime-isolation/probe/Cargo.toml
cargo test --locked --manifest-path scripts/runtime-isolation/probe/Cargo.toml
cargo clippy --locked --manifest-path scripts/runtime-isolation/probe/Cargo.toml --all-targets -- -D warnings
```

Fünf Tests prüfen gefährliche Root-/Profilnamen, vorhandene Verzeichnisse, unvollständige
Neustartbelege, fehlende Messkontrollen und ungültige Portargumente. Sie ändern keine AppContainer-
oder Firewallkonfiguration und führen keine Live-Netzversuche aus. Unter Linux laufen diese reinen
Tests; die CLI selbst verweigert eine Isolationsausführung mit Exitcode 2. CI prüft beide
Plattformen zusätzlich zum unveränderten Produkt-Workspace.

Vier weitere reine Tests prüfen begrenzten Transport, Namen und Workerberichte. Vier Windows-
Dateitests prüfen Hardlinks, vorhandene temporäre Dateien, konkurrierende Ziele und Cleanup nach
Teilkopien. Insgesamt 13 Tests unter Windows, neun reine Tests unter Linux; keine Live-Netz-/
Profilversuche in diesen Tests.

## Socket-/Vererbungsversuch unter Windows

```powershell
$probeRoot = Join-Path $env:TEMP ('kraqel-isolation-network-' + [guid]::NewGuid().ToString('N'))
cargo run --locked --manifest-path scripts/runtime-isolation/probe/Cargo.toml -- network $probeRoot
```

Der nicht isolierte Controller kalibriert lokale TCP-/UDP-Empfänger für IPv4/IPv6 mit einem
bekannten Byte und prüft einen TCP-Handshake zu `1.1.1.1:443`. Der isolierte Prozess und sein selbst
gestarteter Nachfahre versuchen dieselben lokalen Ziele sowie feste öffentliche TCP-/UDP-Ziele
(`1.1.1.1`, `2606:4700:4700::1111`); UDP verwendet nur das Testbyte 42 auf Port 9. Eine feste
DNS-Probe betrifft ausschließlich `example.com`. Keine Manuskripte, Provideraufrufe oder variablen
Zielhosts. Öffentliche Adressen erscheinen nicht in der bereinigten Ausgabe.

`WSAEACCES` zählt als beobachtete synchrone Zugriffsverweigerung. Timeout, fehlende
IPv6-Erreichbarkeit und erfolgreicher UDP-API-Aufruf werden nicht als Blockaden gewertet.
`send_to`-Erfolg belegt keine Zustellung. Der Test verlangt für seine vollständige
Socket-Sperraussage alle Verweigerungen und liefert im dokumentierten Versuch deshalb korrekt
**Exitcode 1**. Das ist kein vollständiger IPv6-, UDP-, DNS- oder Broker-Nachweis.

## Native App mit neuer Testdatenwurzel

Nur einen selbst gebauten Testbuild verwenden. Die App muss ausdrücklich in `data/app` arbeiten;
kein produktives Projekt einspielen. Die Zusatzkonfiguration liegt außerhalb der künftigen
Ausgabewurzel, damit diese bis zum Launcherstart noch nicht existiert:

```powershell
$appRoot = Join-Path $env:TEMP ('kraqel-isolation-app-' + [guid]::NewGuid().ToString('N'))
$configPath = Join-Path $env:TEMP ('kraqel-isolation-config-' + [guid]::NewGuid().ToString('N') + '.json')
@{
  identifier = 'org.kraqel.desktop.isolation-probe'
  productName = 'KRAQEL Isolation Probe'
  app = @{
    appDirectoriesOverride = (Join-Path $appRoot 'data/app')
    windows = @(@{ title = 'KRAQEL AppContainer Probe'; width = 1180; height = 780; minWidth = 760; minHeight = 560 })
  }
  bundle = @{ active = $false }
} | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $configPath -Encoding utf8
pnpm --filter @kraqel/desktop tauri build --debug --no-bundle --config $configPath
cargo run --locked --manifest-path scripts/runtime-isolation/probe/Cargo.toml -- app $appRoot (Resolve-Path target/debug/kraqel-desktop.exe).Path
```

Die echte App darf sichtbar bedient und normal geschlossen werden. Das Tool wartet höchstens zehn
Minuten, beim Socketversuch 30 Sekunden. Eine eigene `STOP`-Datei in der Versuchswurzel beendet nur
den eigenen Job. Nach erfolgreichem Ende kann derselbe synthetische Datenbestand erneut geöffnet
werden:

```powershell
cargo run --locked --manifest-path scripts/runtime-isolation/probe/Cargo.toml -- resume $appRoot
```

`resume` verlangt bestätigte Profilentfernung, null verbliebene Jobprozesse, normalen Exit und
passende frühere AppContainer-Identität. Ein noch vorhandenes Profil wird nicht übernommen. Die
Identität wird aus dem begrenzten Versuchsnamen erneut erzeugt; ihre SID bleibt dadurch für die
Testdatei-ACLs stabil. Binärdatei und Schreibordner müssen weiterhin innerhalb der eigenen
TEMP-Wurzel liegen. `inspect <PID,PID,…>` liest lediglich Tokenmerkmale ausgewählter Prozesse. Es
ist eine Diagnose, keine Autorisierung oder lückenlose Prozessüberwachung.

Die Kindumgebung wird aus festen Windows-Systempfaden und der eigenen Testwurzel aufgebaut; die
normale Benutzerumgebung, WebView2-Overrides und Providergeheimnisse werden nicht vererbt.
Git-Pfade, beliebige Dateidialoge, Schlüsselspeicher und Cloudprovider sind damit noch nicht als
vollständige Produktfunktionen integriert. Der Launcher selbst bleibt ein nicht isolierter
Testcontroller mit Positivkontrollen und ist keine geprüfte Brokergrenze für echte Daten.

Ergebnisse liegen in `result.json`, `data/child.json`, `data/leaf.json`; `launch.json` und
`identity.json` enthalten nur lokal aufzubewahrende Prozess-/Profilzuordnung. Der Normalpfad
entfernt ausschließlich das eigene AppContainer-Profil nach Beenden des Jobs; synthetische
Testdateien bleiben zur Prüfung erhalten. Ein abrupt abgebrochener Controller ist kein geprüfter
Deinstallationspfad. **Exitcode 0 bedeutet einen erfolgreich abgeschlossenen Diagnoselauf, keine
Produktfreigabe**; `productionAcceptance` bleibt immer `false`.

Nach allen Versuchen den normalen Debug-Build wiederherstellen:

```powershell
pnpm --filter @kraqel/desktop tauri build --debug --no-bundle
```

ETW wird getrennt mit dem [bestehenden Helfer](../m3-network/README.md) erhoben. Pro Start die
Root-PID aus `launch.json` und den Executable-SHA festhalten; alle App-/WebView2-Prozesse vor dem
Trace-Ende schließen. Roh-ETL/XML bleiben außerhalb Git. Ein expliziter unisolierter Vergleich ist
ein eigener Diagnosefall, niemals ein Fallback. Unterschiedliche Umgebungen, Profilzustände und
Laufzeiten im Protokoll benennen; keine automatische kausale Gleichsetzung von Zählwerten.

## Kontrollgebundener synthetischer Startpfad

Der begrenzte native Vorschaupfad
`broker-store start-check <Forschungs-App> <erwartete App-SHA> <aktueller Kopfhash>` hält die
authentisierte private Kontrollsperre über die gesamte GUI-Laufzeit und verwendet denselben
benannten Produktjob wie Updates. Containerrechte erhalten ausschließlich neue getrennte
synthetische TEMP-Appdaten; keine vorhandenen Projekte/Journale werden übernommen. Keine normale
Produktintegration oder Wiederaufnahme bestehender Sitzungen. Die echte Vorschauablage niemals
zurücksetzen oder kopieren, um einen neuen Prüfbuild dort zu autorisieren.

Reproduktion ohne Zugriff auf diese echte Vorschauablage:

```powershell
$starterRoot = Join-Path $env:TEMP ('kraqel-isolation-controller-' + [guid]::NewGuid().ToString('N'))
# Gesicherte Forschungs-App und erwartete SHA explizit angeben; kein normaler Appbuild.
cargo run --locked --manifest-path scripts/runtime-isolation/probe/Cargo.toml -- product-start-series $starterRoot <Forschungs-App> <erwartete-App-SHA>
```

[Nachweis, Hashes und Grenzen](../../docs/testing/native-product-starter.md),
[Kontrollvertrag](../../docs/data-format/native-product-control-v1.md). Der geänderte Broker
benötigt einen eigenen frischen ETW-Nachweis; ältere Messungen werden nicht übertragen.

## Dauerhafte synthetische Sitzungsbindung

Kontrollschema 3 erlaubt explizite Apppaket-/Sitzungsbindung und authentisierten Wiederanlauf unter
derselben Kontrollsperre/demselben Produktjob. Es erzeugt ausschließlich neue synthetische Daten,
keine Migration bestehender Projekte/Journale. Ein fehlendes Journal, geänderte Appbytes oder
Stilllegung sperren den Wiederanlauf; kein neuer Importgrant und keine automatische Reparatur.

```powershell
$sessionRoot = Join-Path $env:TEMP ('kraqel-isolation-controller-' + [guid]::NewGuid().ToString('N'))
cargo run --locked --manifest-path scripts/runtime-isolation/probe/Cargo.toml -- product-session-series $sessionRoot <Forschungs-App> <erwartete-App-SHA>
```

Die native Serie beendet zwei Controller am echten GUI-Haltepunkt hart und prüft je zwei
authentisierte Wiederanläufe sowie alte Köpfe, fehlendes Journal/geänderte App und Stilllegung.
Interner `product-session-child` nur im begrenzten synthetischen Controller-/Case-/Store-Layout;
normale Vorschau-Commands aktivieren keinen Controllerabbruch. Kontrollschema 3 erlaubt vorerst
keine Brokerupdates und genau eine gebundene Sitzung je Installation.
[Nachweis und gesicherte Hashes](../../docs/testing/native-product-session-recovery.md). Keine neuen
Programmbytes an der echten Vorschauablage durch Kopieren/Reset autorisieren. Der neue Broker
benötigt frisches ETW; ältere Messungen werden nicht übertragen.

## Zusätzliche Sitzungsübergangs-Crashpunkte

```powershell
$transitionRoot = Join-Path $env:TEMP ('kraqel-isolation-controller-' + [guid]::NewGuid().ToString('N'))
cargo run --locked --manifest-path scripts/runtime-isolation/probe/Cargo.toml -- product-transition-series $transitionRoot <Forschungs-App> <erwartete-App-SHA>
```

20 zusätzliche Fault-Punkte für Bindung, Recovery-Startrecord und Stilllegung, ausschließlich in
neuen privaten synthetischen Installationen. Der interne `product-transition-child` akzeptiert nur
passende typisierte Aktions-/Punktpaare im begrenzten TEMP-Controller-/Case-/Store-Layout. Normale
Vorschau-Commands aktivieren keine Faults. Testprofil-Cleanup repariert keine gesperrte Ablage und
verleiht keine Importautorität.
[Matrix, gesicherte Hashes und eigener frischer ETW-Beleg](../../docs/testing/native-session-transition-crash-etw.md).
Produktprofil-Cleanup-Reconciliation/Schema-3-Updates und normale Produktintegration bleiben offen.

## Authentisierte Profilbereinigung

Kontrollschema 4 signiert Sitzungsvorbereitung vor Profilerzeugung und Cleanupabschluss nach
Stilllegung. Native Vorschau: `broker-store migrate-v4 <aktueller Kopf>` und nach authentisierter
Stilllegung `broker-store cleanup-session <aktueller Kopf>`. Actor/Kopf/Jobleerstand und dieselbe
Kontrollsperre bleiben verpflichtend. Die Profilidentität stammt ausschließlich aus der
Kontrollkette; kein Verzeichnis- oder Diagnoseimport. Wiederholung ist idempotent, Daten und
Schlüssel bleiben erhalten. Ungebundene Alt-Orphans/defekte Kontrollketten bleiben gesperrt.
[Nachweis und Prüftiefe](../../docs/testing/native-profile-cleanup.md): zwei native Fälle und
gezielte Helferprüfung auf ausdrücklichen Wunsch statt großer neuer Testmatrix. Neuer Broker noch
ohne eigenen ETW-Nachweis; keine automatische Autorisierung in der echten Vorschauablage.

## Lokale Brokerpaket-Updates unter Kontrollschema 4

Die vorhandenen nativen `broker-store prepare <EXE> <SHA> <Kopf>`, `activate <Kopf>` und
`abort <Kopf>` sind für Schema 4 freigegeben. Zusätzlich verlangt Schema 4 einen begrenzten
read-only `.kqmeta`-Paketvertrag, geprüft vor Pending und erneut vor Aktivierung. Keine Ausführung
des Kandidaten vor Aktivierung, keine Downloads oder Apppaket-/Datenmigration. Hashfreigabe,
Actor/Kopf/Jobleerstand, dieselbe Kontrollsperre und Rollbackabwehr bleiben verpflichtend.
[Vertrag, native Prüfung und Grenzen](../../docs/testing/native-schema4-package-update.md). Der neue
Broker/Kandidat benötigt eigene frische ETW-Belege; die hashbenannte verwaltete EXE bei einer
Messung ebenfalls vollständig erfassen. Echte Vorschauablage nicht übernehmen/zurücksetzen.

Primärquellen:
[AppContainer für klassische Apps](https://learn.microsoft.com/en-us/windows/win32/secauthz/appcontainer-for-legacy-applications-),
[Start mit Security Capabilities](https://learn.microsoft.com/en-us/windows/win32/secauthz/implementing-an-appcontainer),
[Tokeninformation](https://learn.microsoft.com/en-us/windows/win32/api/securitybaseapi/nf-securitybaseapi-gettokeninformation).
