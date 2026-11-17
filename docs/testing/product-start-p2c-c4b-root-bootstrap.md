# P2c C4b: authentisierter Root und tatsächlicher leerer WebView-Prozessbaum

Stand: 17. November 2026. **Begrenzter nativer Forschungsnachweis bestanden; C4b insgesamt,
Produkt-Q-01/Q-02 und native UI-Freigabe bleiben offen.**
[Bereinigte Aggregate](product-start-p2c-c4b-root-bootstrap.json),
[Vorprüfung](product-start-p2c-c4b-preflight.md),
[C4-Vertrag](../data-format/product-start-p2c-c4-v1-draft.md).

## Programme, Paket und getrennte Autorität

Roothelfer SHA-256 `a8007de34363ab478faffb29321d9c26be7b4ee6d5d27c96d2e747221415d279`; nativer
Testcontroller `fc19dae3e11679cc1286d2beadaf4290f61e166cce4b906af41872d79a187cd6`. Der Roothelfer
heißt in der lokalen Sicherung `controller.exe`, wird aber als `ui.exe` in die frische
Kandidatenfixture kopiert. Der tatsächliche Controller dieses Nachweises ist das getrennt gesicherte
Helfer-Testprogramm, kein normaler Produktstarter.

Fixed-Version-Runtime **157.0.4341.60, Windows x64**, unveränderte zuvor von Microsoft bezogene 257
Dateien/711.058.264 Bytes. Keine neue Installation und keine Verbindung zu echten Projekten. Der
Test kopiert Runtime und Roothelfer in eine neue synthetische Kandidatenablage, prüft das exakte
Inventar unter gehaltenen Handles und verwendet ausschließlich diese Kopie. Neue RX-Rechte nur am
nicht geheimen Kandidaten-UI-/Runtimebaum für das eigene bestätigte Profil; Schreibrechte und
Low-Integrity-Label nur am neuen sitzungseigenen WebView-Bereich. Private Kontrolle, Hostpaket und
ursprüngliche C2-Projektdaten erhalten keine neuen Rechte. Keine allgemeinen
AppContainer-Datenrechte werden vom Controller vergeben; keine nachträgliche ACL-Lockerung bei
Fehlern. Eine vollständige unabhängige Prüfung aller effektiven Daten-ACLs nach Runtimebetrieb ist
damit noch nicht abgenommen.

Frontend-/Konfigurations-/Argument-/Tokenpolicydigests des Kandidatendeskriptors bleiben
synthetische Testwerte. Tatsächliche Paketbytes werden inventarisiert und gebunden, diese Metadaten
sind aber noch keine vollständig durchgesetzte produktive UI-/Rollenpolicy.

`webview2-com = 0.39.1` ist neu im eigenständigen Helfer als Windows-Abhängigkeit festgeschrieben,
entsprechend der bereits vorhandenen App-SDK-Version. Wrapper MIT, drei neue Einträge im Helferlock:
Wrapper/Makros/Sys-Bindings. Workspace-/App-Lock und gemeinsame Transportimplementierung
unverändert. Dies stellt weder Runtime noch SDK automatisch ins Produkt oder nach Git.

## Implementierter Bootstrap

[Protokoll/Records](../../scripts/runtime-isolation/probe/src/windows/exchange/ui_restart/durable/product/workspace_persistent/c4_preflight/root_bootstrap.rs),
[Controller](../../scripts/runtime-isolation/probe/src/windows/exchange/ui_restart/durable/product/workspace_persistent/c4_preflight/root_bootstrap/controller.rs),
[leerer Runtimepfad](../../scripts/runtime-isolation/probe/src/windows/exchange/ui_restart/durable/product/workspace_persistent/c4_preflight/root_bootstrap/runtime.rs).

- Eigene Domain `KRAQEL-C4-ROOT-PROBE\0\x01`, Zweck `c4-blank-runtime-probe`, Version 1. Begrenztes
  strikt typisiertes JSON mit u32-LE-Länge, 32-Byte-HMAC-SHA-256 und höchstens 8.192 Payloadbytes.
  MAC bindet Domain und vollständige Payload; kein C3-Frame akzeptiert. Getrennte zufällige
  Launch-ID/Nonce/kurzlebiger 32-Byte-Schlüssel, kein dauerhafter Kontrollschlüssel.
- Vier ausdrücklich ausgewählte anonyme Pipe-Enden: Grant, Schlüssel, Ereignisschreiben und
  Continue. Vererbungsflags im Controller vor Resume und im Root vor WebView gelöscht und
  zurückgelesen. Root prüft native Pipe-Erzeuger-PID, eigenes Profil/Token/Job, Codehash und Pfade;
  der Controller prüft zusätzlich suspendierten Root, exakte Jobzugehörigkeit, native
  PID/Erzeugungszeit, Paketimage und private Dateiverweigerungen.
- Grant bindet Workspace, Paketdigest, Code, UI-/Runtime-/neuen Datenpfad, Profil und
  Controllergeneration. Accepted bindet Grantdigest und native Rootgeneration; Continue bindet
  denselben Grant und Controller. Rollen-/Sequenz-/MAC-/Kontextfehler sperren vor Runtime.
- Eigene **native Probeactions** `c4RootProbe`, Eventversion 1, im vorhandenen MAC-Journal: Commit →
  RootBound → Accepted → Running → Drained → Finish. Sechs Slots vor Start reserviert. Frühere
  Startfehler dürfen nach kontrolliert bestätigtem Leerstand derselben gehaltenen Jobinstanz
  Drained/Finish erreichen, ohne Accepted/Running zu erfinden. Launch bleibt verbraucht.
- Aktive Probe sperrt allgemeine Updates, Migrationen, Profilstilllegung und andere Probeaktionen.
  Controllergeneration und durchgehend gehaltener Job sind Voraussetzung der nativen
  Übergangsprüfung. Bei Verlust dieser Autorität bleibt Pending gesperrt; keine neue Recovery aus
  Diagnose-JSON, fehlendem Profil oder neu angelegtem gleichnamigem Job.
- Nur ausdrücklich neue synthetische Schema-6-C2-Fixtures, kein Kontrollschema-Upgrade oder
  bestehender Produktdatenzugang. Ältere Reader kennen die neue Action nicht und weisen sie ab; kein
  neuer allgemeiner Readerbereich oder UI-ABI. **Native UI-ABI bleibt 0.** Dies ist kein
  implementierter produktiver C4-UI-Startgrant oder Importauftrag.

Accepted/Running werden vor Continue dauerhaft gespeichert. Erst nach geprüftem Continue schließt
der Root Grant-/Schlüssel-/Continuehandles und löscht den Schlüssel. Der nicht vererbbare
Ereignisschreiber bleibt für begrenzte lokale Diagnosen offen. Sein anschließender Ergebnisframe ist
kein MAC-geschützter Storebeleg und erteilt keine Start-/Datenautorität. Kurzlebiger MAC-Kontext und
Pipebesitz sind keine universelle unabhängige Schreiber-PID-Authentisierung.

## Tatsächlicher WebView-Kindpfad und Ergebnis

Der Root erstellt nach dem Bootstrap nativ `CoreWebView2Environment` mit explizitem gepinntem
Runtimeordner und separatem neuem User-Data-Folder, danach einen verborgenen Windows-Controller. Nur
`about:blank`, keine Projektinhalte, Datenbank oder Tauri-Commands. Script, Webmessages und Devtools
werden deaktiviert. Keine Debugports, fremden Browserargumente, Sandbox-/TLS-Abschaltung,
Loopback-Ausnahme oder Netzwerk-Capabilities. Die vorherige Environment-/Registryprüfung bleibt
konservativ. Expliziter Runtimepfad und diese Einstellungen ersetzen keinen Netznachweis.

Referenzen:
[WebView-Prozessmodell](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/process-model),
[Windows-Jobobjekte](https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects). Daraus
wird keine pauschale Attestierung externer COM-/Shell-/Systemdienste abgeleitet.

Der äußere eigene Job hat **ActiveProcessLimit 16**, KillOnJobClose, kein Breakaway. Der Root wird
bereits suspendiert diesem Job zugeordnet. WebView erzeugt seine eigenen wirklichen Nachfahren;
diese sind ausdrücklich keine vom Controller erzeugten Geschwister. Der Controller beobachtet
Jobmitglieder, hält deren native Prozesshandles, prüft AppContainer/Capabilityfreiheit,
Imageidentität/-hash und native Elterngeneration innerhalb derselben Jobinstanz. Höchstens 64
beobachtete Generationen; die Beobachtung ersetzt keine Durchsetzung vor jedem Kindstart.

| Finaler Ablauf               | Ergebnis                                                                                                                           |
| ---------------------------- | ---------------------------------------------------------------------------------------------------------------------------------- |
| Direkter Root ohne Bootstrap | Exit 2 vor Runtime                                                                                                                 |
| Manipulierte Grant-MAC       | RootBound, kein Accepted/Continue, Root-Exit 2; kontrolliert Drained/Finish                                                        |
| Gültiger Bootstrap           | Accepted/Running/Continue und tatsächliche leere Runtime erfolgreich                                                               |
| Lebende Gruppe               | Ein Root und sieben Runtimegenerationen nativ erfasst; sieben native Elternbezüge                                                  |
| Runtime-API-Snapshot         | Browser 1, Renderer 1, GPU 1, Utility 2, jeweils einer eindeutigen nativen Generation zugeordnet                                   |
| Weitere Generationen         | Zwei native Runtimegenerationen ohne Zuordnung in diesem späteren API-Snapshot; keine erfundene Rolle                              |
| Tokens/Images                | Alle acht beobachteten Generationen AppContainer/Capabilities 0 und gepinnte Dateiidentität/-hash                                  |
| Ende                         | Native Job-Gesamtzahl 8 entspricht acht Beobachtungen; Leerstand und alle acht signalisierten Handles vor Drained/Finish bestätigt |

Eine reine Pfadzeichenfolge reichte nicht: Windows liefert lange Kontonamen, während die
Controllerfixture 8.3-Pfade verwendet. Statt Akzeptanz per Textalias werden Volume-/128-Bit-
Dateiidentitäten unter nativen Handles abgeglichen, Reparse-/Hardlinks ausgeschlossen und der
Dateihash geprüft. Ein Test unterscheidet echte Namensaliase von inhaltsgleichen Dateikopien.

Ebenso ist Job-Leerstand nicht als bereits signalisiertes Prozesshandle ausgegeben: begrenztes
zusätzliches Warten auf gehaltene Handles erfolgt **vor** Drained/Finish. Ein anfänglicher
Nullzeit-Handlecheck schlug fehl und wurde korrigiert. Frühe rein lexikalische Imageabweisungen und
dieser Zwischenlauf sind keine bestandenen Abschlussbelege. Variable Prozesszahlen aus früheren
Programmständen werden nicht auf das finale Hashpaar übertragen.

Ein unabhängiger lesender Byteaudit bestätigt **14 MAC-Records**, davon **zehn C4-Proberecords**:
zwei verbrauchte Launches/zwei Rootbindungen, ein Accepted/Running und zwei Drained/Finish. Native
Runtimeinformationen, Rollen-API-PIDs und Elternzeiten sind getrennt gegengeprüft. Root- und
Runtime-Exitcodes sind keine fachlichen Erfolgsbelege; kontrollierte Jobterminierungen werden nicht
als natürliche Exit-0-Erfolge umgedeutet. Das echte Kontroll-/Datenfixture ist nach authentisierter
Profilstilllegung entfernt; nur private lesende Auditsnapshots bleiben lokal.

## Prüfung, Sicherung und offene Gates

```powershell
cargo test --locked --offline --manifest-path scripts/runtime-isolation/probe/Cargo.toml -q
cargo clippy --locked --offline --manifest-path scripts/runtime-isolation/probe/Cargo.toml --all-targets -- -D warnings
cargo fmt --manifest-path scripts/runtime-isolation/probe/Cargo.toml -- --check
cargo test --locked --offline --manifest-path scripts/runtime-isolation/probe/Cargo.toml c4_authenticated_root_and_real_webview_probe -- --ignored --nocapture
```

Drei neue aktive Tests: Frame/MAC-Domain, Recordfolge/Replay/Sperre sowie native Imageidentität.
**115 aktive Helfer-/Transporttests bestanden, sechs standardmäßig ignoriert** (108 Binär-/sieben
Transporttests). Der neue ignorierte Root-/Runtimefall wurde zusätzlich lokal ausgeführt. Frühere
ignorierte Dialogfälle und die getrennte reine Runtimeinventur wurden nicht erneut als eigene Serien
ausgeführt. Clippy/rustfmt und Fakten-/Verweis-/Format-/Diffprüfung. Nur Forschungshelfer gebaut;
keine neue App-/Transport-/Packagingfunktion oder vollständiger Appbuild.

Lokale Sicherung `%TEMP%/kraqel-c4b-root-build-20261117-01/`: Roothelfer `controller.exe`,
`test-controller.exe`, privates `run.json`, `audit-…` mit geschütztem lesendem MAC-Snapshot,
unabhängiger Audit und Manifest. Schlüssel, Profile, Pfade und PIDs niemals nach Git oder als neue
Brokerfreigabe verwenden. Alte C3-Bytes/ETW bleiben separat; **kein frisches ETW dieses Paars**.

Nächster begrenzter C4b-Scope: nicht zugeordnete Runtimegenerationen aufklären sowie vollständige
Runtime-/Nichtsystemmodul-/effektive Rollenpolicybindung vor Text-/Commandfreigabe konkretisieren
und durchsetzen. Die API-Rollenliste und natives Polling belegen noch nicht jeden dynamischen
Kindstart vor Ausführung. Keine automatische Wiederaufnahme nach Wächterverlust, unabhängige
Renderer-zu-DB-Isolation, dauerhafte Langzeitstorestrategie oder native UI-/Projektfreigabe. Danach
eigener gezielter ETW-Nachweis des exakten Paars; C4c/d, P2 insgesamt/P3–P5 und Produkt-Q-01/Q-02
bleiben offen. Das strikte Datenschutzziel bleibt unverändert.
