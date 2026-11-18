# P2c C4b: Crashpad-Diagnose und verbindliche Modulsperre

Stand: 18. November 2026. **Modulsperre im nativen Forschungsweg geprüft; C4b insgesamt und
Produktfreigabe bleiben offen.** [Bereinigte Aggregate](product-start-p2c-c4b-module-gate.json),
[vorheriger Beobachtungs-/ETW-Stand](product-start-p2c-c4b-observation.md),
[C4-Vertrag](../data-format/product-start-p2c-c4-v1-draft.md).

## Verbliebene Runtime-Rolle

Ein einzelner zusätzlicher synthetischer Diagnoselauf verwendet den vorherigen Roothelfer
`0ce6b48ff22af39624db14449d943562cb229835500b9ed1bab9214feeb48a42` und einen ausschließlich für den
neuen lokalen Ausgabepfad neu gebauten Testcontroller
`9c57f0604f6bb652b86ec3b2953219acb430eac57b86e0763439d903cb866a5e`. Keine neue
Runtime-/Projektfreigabe daraus. Acht native Jobgenerationen, fünf API-Zuordnungen, zwei
Runtimegenerationen außerhalb der API-Liste. Beide passen exakt über PID/Erzeugungszeit zu separat
aufgenommenen nativen Prozessgenerationen mit dem Kommandozeilenargument `--type=crashpad-handler`.
Argumente mit Windows `CommandLineToArgvW` gelesen, keine Shellausführung. Gepinntes Runtimeimage,
AppContainer ohne Capabilities, Elternbezüge und gehaltene Prozessenden geprüft; 14 MAC-Records
unabhängig bestätigt, Profil/Fixtures entfernt.

Das erklärt die beobachtete fehlende API-Kategorie als **Crashpad-Diagnose**, entsprechend dem
dokumentierten API-Ausschluss. Die Prozesszahl schwankt; frühere Generationen werden nicht
rückwirkend identifiziert. Kommandozeilen sind veränderliche Laufzeitbehauptungen und erteilen keine
Rollen-, Daten- oder Startautorität. Insbesondere wird `crashpad-handler` nicht als neue
automatische Ausnahme in die API-Rollenliste aufgenommen.

## Implementierte Sperrentscheidung

[Native Modulpolicy](../../scripts/runtime-isolation/probe/src/windows/exchange/ui_restart/durable/product/workspace_persistent/c4_preflight/module_policy.rs),
[Kandidatenbindung](../../scripts/runtime-isolation/probe/src/windows/exchange/ui_restart/durable/product/workspace_persistent/c4_preflight.rs),
[Controller](../../scripts/runtime-isolation/probe/src/windows/exchange/ui_restart/durable/product/workspace_persistent/c4_preflight/root_bootstrap/controller.rs).

- Die geprüften Programm-/Bibliothekseinträge des Kandidateninventars erhalten native
  Volume-/128-Bit-Dateiidentitäten. Die vorhandenen Inventarhandles halten die gegen SHA-256
  geprüften Bytes ohne Schreib-/Löschfreigabe. Inhaltsgleiche fremde Kopien werden dadurch nicht zu
  erlaubten Paketmodulen; mehrfache Hardlinks sind beim Paketabgleich unzulässig.
- Windows-Plattformabhängigkeit eng begrenzt: unmittelbare Dateien des über Windows-API bestimmten
  Systemverzeichnisses, dessen `DriverStore/FileRepository` sowie `WinSxS` des über API bestimmten
  Windows-Verzeichnisses. Kein Vertrauen in geerbtes `SystemRoot`, beliebige Windows-Unterordner,
  `WindowsApps` oder ähnliche Zeichenpräfixe. Gehaltene native Verzeichnisroots und aus geöffneten
  Handles ermittelte kanonische Pfade mit komponentengenauem Windows-Vergleich; Reparse-/unsichere
  Pfade und Abfragefehler sperren. Windows selbst bleibt ausdrücklich vertraute Plattform, nicht als
  unveränderliches Paket inventarisiert.
- Unmittelbar nach Accepted, **vor Running/Continue**, muss die native Root-Modulaufnahme diese
  Policy bestehen. Fehler oder fremde Module brechen den Versuch ab; der Controller entleert
  dieselbe gehaltene Jobinstanz und schreibt kontrolliertes Drained/Finish. Kein Weiterlauf trotz
  negativer Diagnose. Accepted allein erteilt noch keine Runtimefreigabe.
- Während eines zugelassenen leeren Runtimeversuchs werden die begrenzten Modulaufnahmen ebenfalls
  verbindlich ausgewertet. Fehlende, beendete oder nicht sicher prüfbare Generationen sowie
  Mengengrenzen sperren. Danach müssen alle nativ beobachteten Runtimegenerationen genau durch die
  API-Liste abgedeckt sein; wiederverwendete PIDs, Duplikate, unbekannte Kinds und fehlende
  Generationen sperren. Crashpad-Kommandozeilen umgehen diese Sperre nicht.

Kein Hersteller-/Signatur-/Dateinamenfallback und keine zusätzliche Startoption zum Abschalten der
Sperre. Keine Änderungen an Schutzsoftware, Registry, Firewall, Loopback oder Systempolicy. Native
UI-ABI bleibt 0; keinerlei Text-/Commandfreigabe.

## Tatsächlicher Gegencheck

Neuer Roothelfer `daf59d085588061b1daaabffe1ac24e76fb9c0303fd54e29b4f7231c5a72f5d5`, Testcontroller
`83679e5311988c78b5f37feaf651bf9cda8638a58e554d95f536052cc45caf0c`. Frische synthetische
Kontroll-/Daten-/Kandidatenfixture und unveränderte Fixed-Version-Runtime 157.0.4341.60/x64; keine
echten Projekte oder normale Vorschauablage.

Beide zuvor zusätzlich beobachteten Modulpfade werden durch die native Policy abgewiesen:
Verhaltensüberwachung liegt außerhalb der erlaubten Bereiche; am HEVC-Pfad scheitert bereits das
sichere Halten eines Vorfahren mit Windows-Fehler 5. Auch fehlende Prüfbarkeit bleibt ein
Abbruchgrund. Exakte Pfade und Herstellerdetails bleiben lokal. Der erste Test hatte fälschlich für
beide Dateien eine lesbare Klassifikation erwartet; nach Korrektur des Tests bleibt der
unterschiedliche Fehlergrund ausdrücklich erhalten. Dieser Zwischenlauf ist kein bestandener
Gesamtnachweis und hat keine C4-Startrecords erzeugt.

Der neue Liveversuch bestätigt die Root-Sperre: falsche MAC weiterhin vor Accepted abgewiesen;
gültiger Root authentisiert, danach `c4-module-outside-policy`, **Continue nicht gesendet, WebView
nicht erstellt**, genau eine beobachtete Jobgeneration und kontrollierter Finish. Der HEVC-Pfad ist
separat mit derselben Policy geprüft; er wird im neuen Liveversuch wegen der früheren Root-Sperre
gar nicht mehr geladen. Keine Behauptung einer zusätzlichen tatsächlich durchlaufenen
HEVC-Runtimephase.

Unabhängig 13 MAC-Records, neun C4-Proberecords, zwei verbrauchte Launches geprüft: Commit →
RootBound → Drained → Finish beim MAC-Fehler; Commit → RootBound → Accepted → Drained → Finish bei
der Modulsperre. Kein Running erfunden. Tatsächliches Store-Wiederöffnen ohne Pending bestätigt,
gehaltene Prozesse beendet, eigenes Profil als abwesend und Daten-/Kontrollfixtures als entfernt
unabhängig geprüft.

## Prüfniveau, Sicherung und offene Gates

119 aktive Helfer-/Transporttests im Vollsatz bestanden (112 Binär-/sieben Transporttests), sechs
standardmäßig ignoriert. Danach auf der abschließend verengten Plattformpolicy 23 gezielte C4-Tests
sowie der lokale native Sperrfall bestanden; Clippy `--all-targets -- -D warnings` und rustfmt. Zwei
neue aktive Tests: native Dateiidentität/Kopie/Pfadgrenze sowie vollständige Generationsabdeckung
einschließlich fehlender Crashpadzuordnung. Kein vollständiger Appbuild, kein gemeinsamer
Transport-/Installerumbau und keine neue breite Crashmatrix.

Nur Helferfeature `Win32_System_SystemInformation` ergänzt; keine neue Bibliotheksversion oder
Lockdateiänderung. Lokale geschützte Sicherungen: `%TEMP%/kraqel-c4b-role-diagnostic-20261118-02/`,
`%TEMP%/kraqel-c4b-module-gate-20261118-03/` und zusätzlich
`artifacts/c4b-module-gate-20261118-03/`. Enthalten exakte Programme, private Laufberichte, lesende
MAC-Audits, private Kommandozeilendiagnosen und Quell-/Hashmanifest. Keine aktiven Stores,
wiederverwendbaren Grants, Schlüssel oder Prozessidentitäten in Git. Alte Nachweise bleiben separat.

**Als Nächstes gezielter frischer ETW-Vergleich des neuen Sperrpaars**, einschließlich direkter
Bootstrapabweisung, falscher MAC und authentisiertem Abbruch vor Continue. Der vorherige Netzbeleg
gilt den alten Bytes; noch kein Netzbeleg für diesen neuen Build.

Polling prüft bereits geladene Module und verhindert keine frühere DLL-Ausführung oder das Laden
zwischen Aufnahmen. Pfad-/Dateiidentitäten sind keine vollständige Attestierung sämtlicher gemappter
Bytes oder Ladearten. Eine wirksame Rollen-/Modulbindung vor späterer Textfreigabe und unabhängige
effektive Daten-ACL-Abnahme bleiben offen. Der aktuelle Rechner wird bei vorhandenem fremdem
Rootmodul absichtlich vor Runtime abgewiesen; das ist kein kompatibler Produktstart. C4b insgesamt,
C4c/d, automatische Gruppen-Recovery, P2/P3–P5 und Produkt-Q-01/Q-02 bleiben offen. Striktes
Datenschutzziel unverändert.
