# P2c BP3: Machbarkeitsgrenze und verbindliche Startsperre

Stand: 18. November 2026. **Vorprüfung und native Startsperre implementiert und geprüft; präventive
exakte Modul-/WebView-Rollenbindung weiterhin nicht nachgewiesen. BP3 insgesamt offen.** Der
[Präloadvertrag](../data-format/product-start-p2c-c4-preload-v1-draft.md) und das strikte
Datenschutzziel gelten unverändert. Dieser Stand ist keine produktive Runtime-/Datenfreigabe.

## Ergebnis der Mechanismenprüfung

Die geprüften dokumentierten Schnittstellen liefern noch keinen implementierbaren vollständigen
Nachweis für den aktuellen C4-WebView-Kandidaten. Das ist eine begrenzte Machbarkeitsentscheidung
für diesen Stand, kein Beweis, dass eine andere Windows-Architektur unmöglich wäre.

| Mechanismus / Quelle                                                                                                                                                                                       | Verifizierte Dokumentation                                                                                                          | Entscheidung für KRAQEL                                                                                                                                            |
| ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| [WebView Environment8](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/icorewebview2environment8)                                                                                | Prozessinformation und Änderungsereignis; Prozessliste ohne Crashpad                                                                | Keine präventive Erzeugungs-/Rollenautorität aus Prozessliste, API-Kind oder späterer Aufnahme                                                                     |
| [Child-Process-Attribut](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-updateprocthreadattribute)                                                             | Restricted verhindert Kindstarts; privilegierte fremde Prozesshandles können die Grenze unterlaufen                                 | BP2-Blattregel beibehalten, keine Abschaltung oder Override als WebView-Kompatibilitätslösung                                                                      |
| [Image-Signaturpolicy](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-process_mitigation_binary_signature_policy)                                                                      | Zulassung nach Signaturklassen                                                                                                      | Keine exakte Paketpositivliste; keine Ausnahme für zusätzliche signierte Module                                                                                    |
| [DllMain](https://learn.microsoft.com/en-us/windows/win32/dlls/dllmain)                                                                                                                                    | DLL-Einstieg wird beim Prozessstart bzw. Laden aufgerufen                                                                           | Ein Snapshot nach dem Programmeinstieg beweist keine Sperre vor Ausführung                                                                                         |
| [WebView-Kompatibilitätsempfehlung](https://learn.microsoft.com/en-us/deployedge/webview2-security-tools)                                                                                                  | Empfiehlt unbeschränkte Runtime-Kindstarts einschließlich Crashpad; warnt vor DLL-Injektion und Hooks                               | Keine Autorisierung, unsere Anforderungen abzuschwächen; keine Schutzsoftware-/Registryänderung                                                                    |
| [App-Control-Policies](https://learn.microsoft.com/en-us/windows/security/application-security/application-control/app-control-for-business/design/deploy-multiple-appcontrol-policies)                    | Separate auf dem Gerät ausgerollte Durchsetzungsrichtlinien                                                                         | Kein vorhandenes lokales Prozessattribut und kein bestehender Rollenkanal. Keine systemweite Policy einführen                                                      |
| [Debugereignisse](https://learn.microsoft.com/en-us/windows/win32/debug/debugging-events) und [Initial Breakpoint](https://learn.microsoft.com/en-us/windows-hardware/drivers/debugger/initial-breakpoint) | Prozessereignis vor erster User-Mode-Ausführung; Threads bei Debugereignis angehalten. Initialer Breakpoint vor DLL-Initialisierung | Konkreter Forschungskandidat für frühes Anhalten, noch kein nativer BP3-Beleg oder produktiver Wächter                                                             |
| [Debug-Prozesserzeugung](https://learn.microsoft.com/en-us/windows/win32/debug/process-functions-for-debugging)                                                                                            | DEBUG_PROCESS verfolgt Nachfahren nur, solange sie nicht selbst mit DEBUG_PROCESS erzeugt werden                                    | Die dokumentierte Bedingung verhindert eine pauschale Annahme lückenloser Vererbung. Alle Erzeugungs-/Ladewege und der Wächterstart bleiben gesondert nachzuweisen |

Die letzte Spalte ist unsere Schlussfolgerung aus den Quellen und dem vorhandenen Code. Insbesondere
ist ein Debugereignis weder eine generelle Code-Integrity-Positivliste noch eine WebView-Rolle. Ein
späterer Versuchsaufbau muss auch geänderte Debugketten, Crashpad, spätere Nachladungen,
JIT/ausführbaren Speicher, effektive Handles/Rechte und den eigenen frühen Wächterstart abdecken.
Ein gelungenes DllMain-Beispiel würde diese Fragen nicht automatisch beantworten.

## Implementierter Sperrpunkt

[Native Grenze](../../scripts/runtime-isolation/probe/src/windows/exchange/ui_restart/durable/product/workspace_persistent/c4_preflight.rs):
`require_preventive_runtime()` weist aktuell jeden Aufruf mit der internen Forschungsstufe
`c4-preload-mechanism-unavailable` ab. Das ist kein OS-Supportmaskenwert und kein neuer öffentlicher
Produktfehler. Kein Erfolgsboolean, JSON-Witness, Umgebungsflag, Diagnosemodus oder bestehender
Paketpin kann die Sperre öffnen. Ein künftiger positiver Weg erfordert eine überprüfbare neue
Implementierung einschließlich Vertrag und Tests.

Drei tatsächliche Aufrufgrenzen:

1. [Controller](../../scripts/runtime-isolation/probe/src/windows/exchange/ui_restart/durable/product/workspace_persistent/c4_preflight/root_bootstrap/controller.rs):
   vor Profilöffnung/ACL-Erweiterung, Runtime-Datenverzeichnis, Job/Pipes, Commit und CreateProcess.
   Auch `bad_mac` ist kein Diagnosebypass. Die C4-Kandidatenprüfung bleibt separat rein lesend.
2. [Worker](../../scripts/runtime-isolation/probe/src/windows/exchange/ui_restart/durable/product/workspace_persistent/c4_preflight/root_bootstrap.rs):
   vor Argumentauswertung und Übernahme fremder Raw-Handles, Accepted und Continue. Auch ein alter
   Controller kann diesen neuen Worker nicht für einen Runtime-Start freischalten.
3. [Runtime-Einstieg](../../scripts/runtime-isolation/probe/src/windows/exchange/ui_restart/durable/product/workspace_persistent/c4_preflight/root_bootstrap/runtime.rs):
   nochmals vor COM-, Fenster- und WebView-Initialisierung, falls ein künftiger interner Aufruf die
   ersten beiden Grenzen umgehen würde.

Der früh abgefangene CLI-Zweig `c4-root-probe-worker` endet bei Fehler mit Exit 2 und fällt nicht in
generische Netzwerkdiagnostik. Das beweist keine frühe Modulbindung des direkt gestarteten Helfers
selbst; dessen Windows-Ladevorgang liegt vor dem Programmeinstieg. Die offene Wächter-Startkette
wird ausdrücklich nicht durch diese Startsperre als gelöst behandelt.

Der historische Beobachtungscode bleibt hinter der Sperre lesbar, ist mit diesem Build nicht mehr
ausführbar. Alte gespeicherte Programmbytes behalten ihr damaliges Verhalten und ihre eigenen
Nachweise; ein alter positiver Diagnosebericht ist kein Startgrant für neue Bytes. Die bisherige
maschinen-/TEMP-abhängige ignorierte Modul-Beobachtungsserie wurde durch einen aktiven negativen
Regressionstest ersetzt. Der neue Test führt die historische Serie nicht aus und erweitert sie
nicht.

Keine neuen Records, Schemas, MAC-Domänen, Grants, Dependencies, Marker oder UI-ABI. Bestehende
Records bleiben lesbar und verbraucht; kein Pending repariert oder auf neue Programme übertragen.
BP1-Modell/Golden-Bytes und der getrennte BP2-Blattzweck unverändert. Normale App, Transport,
Installer, echte Projekte, Datenrechte und System-/Schutzsoftwarekonfiguration unverändert.

## Mittlere Prüfung

Zwei aktive BP3-Regressionen:

- Gültiger nativer synthetischer C4-Kandidat, Inventar und gehaltene Dateiidentitäten. Ein
  inventarisiertes Image wird vom bisherigen Snapshotfilter akzeptiert, eine fremde synthetische
  Datei abgewiesen; trotzdem sperren beide Controller-Modi vor dem Start. Kontrollhead,
  Recordanzahl, Schlüsselhash, synthetischer Projektinhalt und Verzeichnisbestand bleiben gleich;
  native private ACL-Prüfung bleibt erfolgreich. Kein Launchverbrauch/Pending, lesende
  Kandidatenprüfung weiterhin erfolgreich und derselbe Zustand nach echter Store-Wiederöffnung.
  Ausschließlich fixtureeigene temporäre Profile/Daten, automatische eigene Bereinigung.
- Worker mit fehlenden/ungültigen Argumenten und direkter Runtime-Aufruf mit leeren Pfaden: derselbe
  Sperrfehler vor Handleübernahme oder COM-/Runtime-Initialisierung.

Gesamter Helfer-/Transporttestsatz: **138 aktive Tests bestanden** (131 Binär-/sieben
Transporttests), sechs standardmäßig ignoriert. Clippy für alle Targets mit Warnungen als Fehler,
rustfmt und normaler Helferbuild erfolgreich. Kein vollständiger App-Build oder `pnpm check` für
diese ausschließlich isolierte Forschungsänderung; Ausnahme gemäß AGENTS und mittlerem Prüfniveau.

Zusätzlich drei verdeckt gestartete direkte Aufrufe des neuen normalen Helfers: ohne Argumente, mit
vier nichtnumerischen und mit vier numerischen Handlebehauptungen. Je Exit 2, keine
Standardausgabe/Fehlerausgabe und keine Datei im eigenen frischen Arbeitsverzeichnis. Das sind
Startabweisungen, keine neue GUI-, Rollen-, Modul- oder Netzabnahme.

Lokal gesichert unter dem ignorierten `artifacts/bp3-admission-20261118-01/`: normaler Helfer
`04e38c26590846042183a6f4974af47a01b4fcb395c83b52fd6f069db7f9e268`, Testcontroller
`503fdbcde15fe161dfd32239cd644c2425a2e9b8db3dfeee9d1d19d93d2d75cd`, Quellhashmanifest und bereinigte
Direktaufrufsergebnisse. Keine privaten Schlüssel oder Rohdiagnosen veröffentlicht; keine neuen
ETW-Rohdaten erzeugt.

**Kein frisches ETW dieses Builds.** Der BP2-Netzbeleg bleibt ausschließlich bei seinem dort
dokumentierten Programmpaar. Aus den drei Startabweisungen folgt keine Netzfreiheit des neuen
Helfers oder anderer Wege. Keine neue umfangreiche Crash-/ETW-Matrix und keine UAC-Anforderung.

## Nächster begrenzter Umfang

BP3 bleibt offen; BP4/C4c/d bleiben gesperrt. Nächster sinnvoller Versuch ist eine **separate
synthetische Debug-Wächter-Machbarkeitsprobe**, keine Änderung des gesperrten C4-Pfads:

1. Eigener nicht autorisierender Zweck ohne dauerhafte Kontrollschlüssel, DB, Projektpfade oder
   WebView. Begrenzte signaturfreie Test-EXE und Test-DLL mit rein lokalem synthetischem Marker;
   keine Injektion in fremde Prozesse und keine Schutzsoftware-/Systempolicyänderung.
2. Native Ereignisfolge an gehaltenen Generationen vor dem ersten User-Code bzw. DLL-Einstieg
   belegen. Abgewiesenes Image/DLL darf seinen Marker nicht erzeugen. Keine Patch-/Hooklösung.
3. Zuerst die dokumentierte Debugketten-Grenze mit einem synthetischen Nachfahren prüfen, der selbst
   DEBUG_PROCESS verlangt. Fehlende präventive Beherrschung bedeutet **ungeeignet**; kein positives
   Ergebnis aus späterem Polling/ETW und kein automatischer WebView-Versuch.
4. Eigene begrenzte Lebenszeit/Drain/Wächterverlust und Rechteabschluss belegen. Erst bei
   tragfähigem Ergebnis den vollständigen Loader-/Rollen-/Hoststartvertrag weiterentwickeln. Vor
   einem positiven Runtime-Witness ist weiterhin ein vollständiger Nachweis aller relevanten Wege
   erforderlich.

Diese Folge ist geplant, noch nicht implementiert oder getestet. Kein Machbarkeitsversuch öffnet
automatisch `RuntimeStartAllowed` oder `DataAllowed`. Kein Merge/Release oder Produktabnahme aus
diesem Zwischenstand; Gesamtprojekt grob 65 %, nutzbarer 0.1-Umfang grob 80 % unverändert.
