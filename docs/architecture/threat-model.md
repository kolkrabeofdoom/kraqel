# Bedrohungsmodell (Entwurf)

Aktuell · BP3-Machbarkeitsgrenze und native Startsperre, 18. November 2026:

[BP3-Nachweis](../testing/product-start-p2c-bp3-admission.md): Für den aktuellen C4-Kandidaten ist
noch kein vollständiger präventiver Modul-/WebView-Rollenmechanismus nachgewiesen. Der native
Controller sperrt jetzt vor ACL-/Ressourcenanlage, Commit und CreateProcess; Worker und
Runtime-Einstieg sperren zusätzlich vor Handleübernahme bzw. COM/WebView. Keine konfigurierbare
Ausnahme oder Umdeutung von Paketpins, BP1/BP2 oder Diagnosen. Zwei aktive Regressionen, 138 aktive
Helfer-/Transporttests und Clippy grün; sechs standardmäßig ignoriert. Drei direkte Workeraufrufe
enden mit 2 ohne Ausgabe/Dateianlage. Kein neues ETW; frühere Messungen bleiben bytegebunden. **BP3
insgesamt offen**, keine Runtime-/Datenfreigabe, BP4/C4c/d weiter gesperrt. Nächster begrenzter
Scope: separate synthetische Debug-Wächter-Probe mit ausdrücklicher Debugketten-Gegenprobe; noch
nicht implementiert. Keine System-/Schutzsoftware-/Signaturausnahme. Mittleres Prüfniveau und
striktes Datenschutzziel unverändert; Gesamt grob 65 %, nutzbarer 0.1-Umfang grob 80 %.

Vorheriger Stand · BP2 nativer synthetischer Blattversuch samt ETW, 18. November 2026:

[BP2-Nachweis](../testing/product-start-p2c-bp2-native-leaf.md): unabhängige native Supportprüfung,
NoRemote-/NoLowLabel-/Child-Policy atomar bei suspendierter Erstellung; Readback vor Resume und im
Blattprozess. Kindstart mit Windows-Fehler 367 abgewiesen, Jobbudget zwei zur Abgrenzung von
Quotenfehlern, tatsächlich nur eine Generation je Job. Unsupported vor Create/Headänderung,
abweichende Sollerwartung vor Resume und falsche MAC vor Accepted geprüft. Eigener BP2-Zweck,
MAC-Frames/Records unter derselben Kontrollsperre, Ergebnis-Hashes in Endrecords gebunden. Je
funktionaler/gemessener Serie 18 MAC-Records/14 BP2-Records/drei verbrauchte Launches geprüft,
Profile/Fixtures entfernt. Frisches ETW des Paars `5205d72…`/`62d7a73…`: neun explizite Enden, acht
Kontrollen, keine Verluste/Zuordnungslücken oder beobachtetes Netz in diesen Bäumen. Drei neue
aktive Tests/136 insgesamt bestanden, sieben bewusst ignoriert, Clippy/rustfmt und 27
Auswertungstests grün. Kein App-/Transport-/Installerumbau, native UI-ABI bleibt 0. **Keine
Runtime-/Datenautorität:** BP1-Modell ist kein nativer Grant, BP2 keine vollständige
Modul-/Rollenattestierung. **Nächster Scope BP3:** präventiven Wächter-/Lade-/WebView-Erzeugungsweg
und Machbarkeit klären; Kindstartsperre nicht für Kompatibilität abschalten, keine Hersteller-/
Signaturausnahme oder Schutzsoftwareänderung. BP4 Datenrechte/Commands und C4b insgesamt/C4c/d,
P2/P3–P5 und Produkt-Q-01/Q-02 bleiben offen. Mittleres Prüfniveau und striktes Datenschutzziel
unverändert; Gesamt grob 65 %, nutzbarer 0.1-Umfang grob 80 %.

[Frischer C3-Netzbeleg](../testing/product-start-p2c-group-bootstrap-etw.md), 17. November 2026:
unveränderter Build `b24b3aa…`, Referenz und Gruppen-/gesperrter Wiederanlaufpfad. 28 Controller,
zwölf Workerrollen, 68 explizite Enden, acht Kontrollen; Nullverluste/Zuordnungslücken und null
beobachtete Netzereignisse in diesen Bäumen. Native/Kernelerzeugungszeiten und Elternbezüge
abgeglichen, 66 MAC-Records/sechs Inventare und drei Profilabwesenheiten unabhängig bestätigt.
Negativer Store bleibt pending, eigene Fixturebereinigung erzeugt keine Recoveryautorität.
Gruppen-MAC bleibt kein Renderer-/Schreiber-PID-Nachweis, native UI-ABI 0. C4/Produkt-Q-01/Q-02 und
externe Dienstgrenzen offen; striktes Datenschutzziel unverändert.

Vorheriger Implementierungs-/Netznachweis:

[Frischer Workerbindungs-/Wiederanlauf-Netzbeleg](../testing/product-start-worker-lifetime-etw.md).
Frisches Workerbindungs-v3-ETW, 17. November: unveränderter Build `37d7bec…`, normale Referenz und
vier Wiederanlaufpfade. 52 Controller/acht Worker/52 Controller-Konsolenhosts, alle 112 Enden
explizit; acht Kontrollen, Nullverluste/Zuordnungslücken, null beobachtete Netzereignisse in den
ausgewählten Bäumen. Alle acht nativen Erzeugungszeiten zum Kernel abgeglichen, sieben dauerhafte
Bindungen und drei Endrecords nach bestätigtem Kernelende zugeordnet. Exit 0 ersetzt keinen
fachlichen Abschluss: vier Finish-Records, drei reconciliierte Starts, ein ungebundener Start weiter
pending/gesperrt. Daten/MAC-/Paket-/Root-/Profilaudit und 42 Diagnose-Tests grün, echte Daten und
Schlüssel unverändert. Kein neuer Build/App-/Transport-/Installerumbau oder pnpm check; Rohdaten nur
lokal. Der P2c-Vertrag ist inzwischen als Entwurf ausgearbeitet; seine Umsetzung bleibt offen.
Ein-Prozess-Beleg nicht auf WebView2 übertragen, private Fixtures nicht promovieren; ungebundene
Starts/unbestätigte Profile und P2 insgesamt/P3–P5/Produkt-Q-01/Q-02 offen, striktes Ziel
unverändert.

Vorheriger Stand:

[Dauerhafte Workerbindung und Wiederanlauf](../testing/product-start-worker-lifetime.md). Dauerhafte
P2b-Workerbindung, 17. November: Execution v3/Bereinigungs-ABI 3, sechs Recordslots reserviert.
WorkerBound bindet Launch, native PID/Erzeugungszeit und geprüfte Ein-Prozess-Jobpolicy vor Resume;
enger nativer Lease-/Job-/Token-Wächter erlaubt nur diesen Record bei aktivem Worker, andere
Storemutationen behalten Jobleerstand. Wiederanlauf vor Drained benötigt diesen MAC-Beleg und
tatsächliches Ende derselben Prozessgeneration: signalisiertes Handle, andere Erzeugungszeit oder
PID-Abwesenheit mit vollständigem Snapshot; keine fremde Terminierung, Fehler bleiben gesperrt.
RecoveredDrain wird vor eigenem Profilcleanup gespeichert. Vier native Crashfälle: drei gebundene
Starts sicher reconciliert und neue Generation erfolgreich; Start vor WorkerBound bleibt auch nach
separatem eigenem Fixturecleanup pending/gesperrt. 47 Controller/sieben Worker, 63 Records, sieben
verbrauchte Launches, 42 private Dateisperren; sieben Profile am Ende abwesend. 75 aktive
Helfer-/Transporttests/Clippy und unabhängiger MAC-/PE-/Root-/Inventar-/Profilaudit grün, echte
Daten und Schlüssel unverändert. Neuer gesicherter Runtimehash `37d7bec…`, kein
App-/Transport-/Installer- umbau oder pnpm check. Nächster Scope: gezieltes frisches ETW dieses
Controllers samt Worker-/ Lebenszeit-/Recoverypfaden. Voriger Netzbeleg bleibt an `9021872…`
gebunden. Ein-Prozess-Prototyp ist keine mehrprozessige GUI; ungebundene Starts/unbestätigte
Profile, persistente echte Arbeitsablagen-/UI-Bindung, P2 insgesamt/P3–P5 und Produkt-Q-01/Q-02
offen; striktes Ziel unverändert.

Vorheriger Stand:

[Frischer Ressourcen-Reconciliation-Netzbeleg](../testing/product-start-resource-reconciliation-etw.md).
Frisches Ressourcen-ABI-2-ETW, 17. November: unveränderter Runtimebuild `9021872…`, normaler
Workerabschluss als Referenz gegenüber vier gezielten Ressourcenabbrüchen/Reconciliation. 54 direkte
Controller, sechs Worker, 54 Controller-Konsolenhosts, alle 114 Enden/Eltern/Exitcodes explizit
abgeglichen. Acht lokale TCP-/UDP-IPv4/IPv6-Kontrollen, Nullverluste/Zuordnungslücken, null
beobachtete Netzereignisse in den ausgewählten Bäumen (auch null Loopback). Daten/MACs/
Paket-/Rootbindung und Profile unabhängig geprüft: 67 Records, neun verbrauchte Launches, 36 private
Dateisperren, acht Profile am Ende abwesend. Sieben Broker-Cleanups; die eine eigene mehrdeutige
Fixture bleibt ausdrücklich separate Prüfablaufbereinigung, keine Produktfreigabe. 42 Diagnose-Tests
grün, kein neuer Build/Runtime-/App-/Transport-/Installerumbau oder pnpm check. Rohdaten nur lokal,
echte Vorschauablage unverändert. Nächster Scope: dauerhafter Workerstartbeleg und gesicherter
Wiederanlauf bei Controller-Crash zwischen RunIntent und Drained; bis belegtem Lebenszeitende weiter
gesperrt. P2 insgesamt/P3–P5 und Produkt-Q-01/Q-02 offen, striktes Ziel unverändert; Beleg gilt nur
für diese Bytes/Lebenszeiten/Pfade.

Vorheriger Stand:

[Begrenzte P2b-Ressourcen-Reconciliation](../testing/product-start-resource-reconciliation.md).
P2b-Ressourcen-Reconciliation, 17. November: Execution v2/Profilbereinigungs-ABI 2 mit neuen
RunIntent-/Drained-Records, fünf Slots vor Ressourcenanlage reserviert. Unter derselben Lease werden
fehlende unbestätigte Profile ohne Löschung abgeschlossen und bestätigte Profile nur ohne RunIntent
oder nach dauerhaft belegtem Drain bereinigt. Exakte begrenzte HKCU-Mapping-/Moniker- Abfrage, keine
Registryänderung; Fehler bleiben gesperrt. Bestehendes Profil ohne ProfileReady oder RunIntent ohne
Drained wird nicht adoptiert/gelöscht. Vier native Abbruchfälle, 49 Controller, fünf erfolgreiche
Worker, 30 private Dateisperren; sieben Profile erzeugt, sechs Broker-Cleanups, eine nachweislich
eigene mehrdeutige Fixture ausschließlich durch den Prüfablauf separat entfernt. 73 aktive
Helfer-/Transporttests/Clippy und unabhängiger MAC-/PE-/Root-/Inventar-/Profilaudit grün; acht
verbrauchte Launches/56 Records, Daten/Schlüssel/echte Vorschauablage unverändert. Neuer Runtimehash
`9021872…`, ein Helferbuild, keine App-/Transport-/Installeränderung oder pnpm check. Nächster
Scope: gezielter frischer ETW-Vergleich dieses Builds mit Worker/Reconciliation. Voriger Netzbeleg
bleibt ausschließlich an `590c3d4…` gebunden. Vollständige Live-Worker-Crashbereinigung, persistente
echte Arbeitsablagen-/UI-Bindung, P2 insgesamt/P3–P5 und Produkt-Q-01/Q-02 offen; striktes
Datenschutzziel unverändert.

Vorheriger Stand:

[Frischer P2b-Netzbeleg](../testing/product-start-p2b-bootstrap-etw.md), 17. November: unveränderter
Runtimehash, Referenz und gezielte Workerfälle, 31 Controller/sechs Worker/31 Console-Hosts mit 68
expliziten Enden. Acht Kontrollen, Nullverluste/Zuordnungslücken, null beobachtete Netzereignisse in
den ausgewählten Bäumen. Separater Daten-/MAC-/Kernel-/Token-/Job-/Profilaudit bestätigt Ausführung
und Cleanup. Keine GUI-/Produkt-/externen Dienst- oder Inhaltsfreigabe, Ressourcencrashes weiterhin
gesperrt. Nächster begrenzter Schritt deren sichere Reconciliation; striktes Ziel unverändert.

Vorheriger Stand:

[P2b-Probeworker](../testing/product-start-p2b-bootstrap.md), 17. November: separate synthetische
Profile/Readonly-Daten, gleiche Kontrolllease, MAC-gebundene Launch-/Ressourcen-/Abschlussrecords.
Eigener Einmalschlüssel, explizite drei Pipehandles, Token/exakter Job/private Dateisperren vor
Resume; monotone Receiptfrist und Drainabfrage am gehaltenen Job vor Profilentfernung. Kein
ACL-Grant auf private P2a-/Produktdaten, keine Importautorität/GUI. Drei gezielte native Fälle und
unabhängiger Audit grün; kein frisches ETW dieses neuen Builds. Crashressourcen bleiben gesperrt,
automatische Bereinigung/Bestandsübernahme und normale Produktintegration offen. Striktes Ziel
unverändert.

Vorheriger Stand:

[P2a authentisierte Workspace-/Launch-Records](../testing/product-start-p2a-records.md), 17.
November: Schema 5 nur im pfadbegrenzten Forschungschild, keine Importtestsitzung übernehmen.
Vollständige MAC-/Root-/Actor-/Kopfprüfung vor Storefreigabe, Job-/Leaseguard vor Mutationen.
P1-Kern erhält Recorddigests statt fiktiver Kopfwerte; Parseerfolg bleibt keine Autorität.
Record-only-Paketmarker mit UI-ABI 0, keine Profile/GUI/Importtasks. Brokerprivate Fixture nicht
durch ACL-Grant zum Produktvault machen. Drei native Abbruchfälle/67 Helfertests/Clippy und
unabhängiger Audit grün; Tail/fehlender Schlüssel bleiben gesperrt. Neuer Broker ohne frisches ETW,
P2b-/Produktgrenzen und striktes Datenschutzziel unverändert.

[Produktstart P1 · reines Testmodell](../testing/product-start-p1-model.md), 17. November:
versionierte Bindung/Startvorschlag strikt getrennt von Importautorität. Parser liefert unvertrautes
Objekt; keine MACs/OS-Prüfung/Schlüssel/I/O oder Startcapability. Modell prüft Kontext/Kopf/Replay/
Generation/Job/Pending/Budget vor Mutation und erhält Binding/Verbrauch bei Ende/Unterbrechung. Nur
im Helfertestziel eingebunden, sieben neue Tests/63 aktive Helfertests/Clippy grün. Modelliertes
Schema 5 ist keine native Storeunterstützung. P2a muss Persistenz/Authentisierung erst herstellen,
ohne GUI oder echte Daten. Striktes Ziel und offene Produkt-Q-01/Q-02 unverändert.

[Produktstart-/Bestandsdatenvertrag v1 · Entwurf](../data-format/product-start-integration-v1-draft.md), 17.
November: neuer Planungsstand, keine implementierte Autorität. Normaler Start ist unisoliert;
Import-Recovery berechtigt nicht zum allgemeinen Workspace-Start. Eigene versionierte Bindung,
Verbrauch vor Kindstart, kein unisolierter Fallback und keine Kontrollschlüssel im Container
geplant. Bestandskopie explizit/zerstörungsfrei; migrationsfähiges Datenbanköffnen ist keine
nebenwirkungsfreie Inventarisierung. Keine ACL-Freigabe alter Daten oder Übernahme der
Vorschauautorität. Geplantes Schema 5/Produktbootstrap und externe Dienstgrenzen noch offen. P1
beginnt mit reinen Typen/ Zustandsregeln, keine Daten-/Startfreigabe. Striktes Ziel und offene
Produkt-Q-01/Q-02 unverändert.

[Gezielte Prepare-Übergangsabbrüche](../testing/native-prepare-transition-crashes.md), 17. November:
nach Artefaktentfernung/Zielanlage Kopf unverändert, kein Pending; nach Prepare-Abschluss Pending
mit leerer Kopie und explizitem Abbruch vor Retry. Drei native harte Prozessabbrüche bestätigen
Wiederanlauf unter derselben Sperre ohne neue Importautorität. 56 Helfertests/Clippy und Daten-/
MAC-Audit grün. Authentisierter Retry mit vorherigem Abort, kein allgemeines Erst-Prepare-Orphan-
Cleanup oder Stromausfallnachweis. Inzwischen
[eigener frischer ETW-Beleg](../testing/native-prepare-transition-etw.md) der unveränderten
Programme: regulärer Update-/Recoverypfad und drei Übergänge, 59 direkte Aufrufe samt fünf
verwalteten hashbenannten Starts, acht Kontrollen, 182 explizite Enden, 0 Nicht-Loopback-/48
getrennte Loopback-Ereignisse. Daten-/MAC-/Kernel-Audit grün, keine Inhalts-/externe Dienst-/
Produktattestierung. Striktes Ziel unverändert.

[Sichere Paketartefakte/Retry](../testing/native-package-artifact-retry.md), 17. November: Ersetzung
nur authentisiert abgebrochener Schema-4-Hashes, niemals aktivierter Pakete. Aktueller Actor/Kopf,
Jobleerstand und erlaubter Übergang vor Dateimutation unter derselben Sperre. Exklusiver Handle,
kein Reparse-Follow, private ACL, genau ein Hardlink; handlegebundene Löschung, kein Sweep.
Zielerzeugung vor neuem Pending verhindert unbeabsichtigte Pending-Kollisionen. Unbekannte
Artefakte/unvollständige Ketten bleiben gesperrt. 56 Helfertests/Clippy, drei native Fälle und
Daten-/MAC-Audit grün.
[Eigener frischer Retry-ETW-Beleg](../testing/native-package-artifact-retry-etw.md) inzwischen
vorhanden: 21 direkte Aufrufe samt fünf hashbenannten verwalteten EXEs, acht Kontrollen, 144
explizite Enden, 0 Nicht-Loopback-/48 getrennte Loopback-Ereignisse. Daten-/MAC-/Kernel-Abgleich
grün; unveränderte Messguards, keine Inhalts-/externe Dienst-/Produktattestierung. Striktes Ziel und
Produktgrenzen unverändert.

[Gezielte Update-Abbruch-/Fehlerreconciliation](../testing/native-update-abort-reconciliation.md), 17.
November: vorhandener Runtimepfad unter derselben Sperre geprüft; aktueller Actor/Kopf und
Jobleerstand erforderlich. Teilkopie/Manipulation aktivieren nicht; authentisierter Abort erhält
Aufgaben-/Sitzungsbindung. Nach veröffentlichtem Abort authentisierter Wiederanlauf ohne Duplikat;
unvollständiger Abort-Tail bleibt gesperrt. Kein Schlüssel-/Kopfreset oder neue Importautorität. 53
aktive Helfertests/Clippy, drei native Fälle/Daten-/MAC-Audit grün. Keine Runtimeänderung/neuer
Netzbeleg dieser Fehlerpfade. Abgebrochene Paketartefakte und transparente Wiederholung bleiben
offen; striktes Ziel und Produktgrenzen unverändert.

[Lokale Schema-4-Paketkompatibilität](../testing/native-schema4-package-update.md), 17. November:
Vorbereitung/Aktivierung/Abbruch unter bestehender Sperre. Gepinnte Kandidatenbytes müssen den
erwarteten SHA und einen begrenzten read-only PE-Vertrag erfüllen; Prüfung vor Pending und erneut
vor Aktivierung, keine Ausführung des Kandidaten vorher. Vertrag deklariert Kompatibilität, keine
Herkunfts-/Verhaltensattestierung. Pending blockiert Sitzung, alter Actor/Rollback bleiben gesperrt.
Ein nativer synthetischer Wechsel erhält Schlüssel/Bindung/Daten, Apppaket unverändert. Neuer
Broker/Kandidat mit [eigenem frischem ETW](../testing/native-schema4-package-update-etw.md): alle 15
Updateaufrufe samt fünf hashbenannten verwalteten EXEs, acht Kontrollen, 132 explizite Enden, 0
Nicht-Loopback-/48 getrennte Loopback-Ereignisse. Exakte PID-/Image-Zuordnung nur im Kontrollmodus;
keine Rootentdeckung oder Lockerung der Messguards. Daten-/MAC-/Lebenszeitaudits grün; keine
Inhalts-/externe Dienst-/Produktattestierung. Striktes Ziel und Produktgrenzen unverändert.

[Gezieltes Cleanup-ETW](../testing/native-profile-cleanup-etw.md), 17. November: unverändertes
gesichertes Paar, zwei gültige kurze Phasen mit acht Kontrollen/Nullverlusten/140 expliziten Enden.
0 beobachtete Nicht-Loopback-/48 getrennte Loopback-Ereignisse. Alle 19 direkten nativen Aufrufe
erfasst; `native-control` erlaubt GUI-lose Hostaufrufe, ersetzt keine
Vollständigkeits-/Kalibrierungs- oder Lebenszeitprüfung. Erwartete Aufrufe/GUIs unabhängig
abgeglichen. Keine externe Dienst-/ Inhalts-/Upload- oder Produktattestierung. Echte Vorschauablage
unbenutzt; striktes Ziel unverändert.

[Authentisierte Profilbereinigung](../testing/native-profile-cleanup.md), 17. November:
Kontrollschema 4 signiert Sitzungsvorbereitung vor Profilerzeugung. Cleanup verlangt authentisierte
Stilllegung, aktuellen Actor/Kopf und Jobleerstand; gleiche Sperre bis Abschlussrecord. Identität
nicht aus Ordnern/Diagnosen, keine Importautorität oder Reparatur. Zwei native Fälle/kurzer
Schema-4-Start und Recovery gezielt geprüft; bestehende Daten bleiben erhalten. Alt-Orphans ohne
authentisierte Bindung und defekte Kontrollketten bleiben gesperrt. Neuer Broker ohne frisches ETW;
striktes Ziel und offene Produktabnahme unverändert.

[Übergangs-Crashpunkte und eigener frischer ETW-Beleg](../testing/native-session-transition-crash-etw.md), 17.
November: 20 zusätzliche Übergangsabbrüche. Neun unvollständige Abschlüsse bleiben gesperrt; elf
Zustände authentisierbar, davon vier explizit wiederanlaufbar. Eigener Testprofil-Cleanup gibt keine
Importautorität und repariert keine Kontrollablage. Zwei gültige ETW-Phasen desselben gesicherten
Builds: acht Kontrollen, keine Verluste, 458 explizite Enden, 0 Nicht-Loopback-/138 getrennte
Loopback-Ereignisse. Kein Inhalts-/Upload- oder vollständiger externer Dienstnachweis. Echte
Vorschauablage unbenutzt, Produktintegration weiter offen. Nächster Scope: authentisierte
Produktprofil-Bereinigung; striktes Datenschutzziel unverändert.

[Dauerhafte Sitzungsbindung/Recovery](../testing/native-product-session-recovery.md), 17. November:
explizites Kontrollschema 3 bindet Apphash, Sitzungswurzel, Task und Startkennungen an die private
Kontrollkette. Neue Controller verwenden dieselbe Sperre und denselben Produktjob; vorhandene
Journale/Appbytes zwingend, keine Neuinitialisierung oder initialen Importgrants beim Recovery. Zwei
echte Controllerabbrüche, vier Wiederanläufe/acht Replay-Ereignisse und 36 Dateiverweigerungen
bestanden; genau eine legitime Projektkopie. Schema-3-Updates, normale Produktintegration und
allgemeine Crashpunkte weiter offen. Neuer Broker ohne eigenen frischen ETW-Beleg; striktes Ziel
unverändert. Echte Vorschauablage nicht verwendet.

[Kontrollgebundener Startpfad](../testing/native-product-starter.md), 17. November: synthetische
Vorschau hält die private Kontrollsperre über die ganze GUI-Laufzeit und verwendet den gleichen
Produktjob wie Updates. Vor Resume tatsächliche Sperre/Rootzuweisung und zwölf private
Dateiverweigerungen in zwei Starts geprüft; Quelle/Kopie an erwartete SHA gebunden und gepinnt.
Containerrechte nur für neue getrennte TEMP-Appdaten, keine Übernahme vorhandener Projekte oder
Auftragsjournale. Noch keine dauerhafte Apppaket-/Sitzungsbindung, kein Produkt-Recoverypfad und
kein frischer ETW-Beleg dieses Brokerhashs. Striktes Ziel und offene Produkt-Q-01/Q-02 unverändert.

[Kontroll-Crash-Serie und eigener ETW-Beleg](../testing/native-product-control-crash-etw.md): 28
harte Prozessabbrüche mit frischen synthetischen Wurzeln; 17 unvollständige Zustände bleiben
gesperrt, elf nutzbar, acht ausdrücklich weitergeführt. Kein Reset/automatisches Reparieren. Faults
ausschließlich im begrenzten TEMP-Testpfad, standardmäßig aus; bestehende Vorschauablage unbenutzt.
Zwei getrennt kalibrierte gültige Messphasen, acht Kontrollen, Nullverluste, 1.136 explizite Enden
und 0 beobachtete Nicht-Loopback-/444 getrennte Loopback-Ereignisse. Keine
Produkt-/Herausgeberabnahme, Stromausfall-/Checkpoint-Rollbackgarantie oder vollständige externe
Dienstkausalität. Striktes Ziel unverändert; Produktstarter und echte Datenmigration offen.

[Native Kontrollablage-Vorschau](../testing/native-product-control.md): Geschützte lokale Wurzel,
private zufällige MAC-Schlüssel, exklusive Sperre, unveränderliche Recordkette und eigener Kopf.
Lokaler Programmwechsel benötigt erwartete SHA/aktuellen Kopf und explizite Aktivierung; aktive Jobs
blockieren Änderungen. Kontrollschema 1 → 2 ist explizit; keine Projekt-/Journalmigration. Alte
Programme/Hashwiederverwendung und unvollständige Abschlüsse werden abgewiesen, keine automatische
Reparatur. 18 tatsächliche Container-Dateiverweigerungen und native Neustarts bestätigt. Kein Schutz
gegen Kontokompromittierung/komplettes Checkpoint-Rollback behauptet. Normale App weiter unisoliert,
lokale Kandidatenfreigabe kein Herausgebervertrauen; neuer ETW- und integrierter Crash-Nachweis
offen. Produktstarter muss dieselbe Sperre vor Joberzeugung halten. Striktes Datenschutzziel
unverändert.

[Eigener frischer ETW-Netzbeleg des Fensterende-Paars](../testing/native-combined-window-close-etw.md):
Automatik und beide echten Hauptfensterschlüsse samt Wiederherstellung in getrennten gültigen
Phasen. Acht Kontrollen, Nullverluste, 942 explizite Enden, 0 beobachtete Nicht-Loopback-/536
getrennte Loopback-Ereignisse. MAC-/Daten-/Lebenszeiten unabhängig abgeglichen; gilt ausschließlich
für das gesicherte Hashpaar und diese Pfade. Keine vollständige externe COM-/Shell-/DNS-/Broker-
Attribution oder Produktabnahme; striktes Ziel unverändert. Nächster Scope: Produktablage sowie
authentisierter Update-/Migrationspfad, weitere allgemeine F08-/Dienstgrenzen separat.

[Kombinierte Fensterendeprüfung](../testing/native-combined-window-close.md): Eigener nativer Dialog
und Teiltransfer enden beim Hauptfensterschluss regulär mit `CANCELLED`; kein Abschlussbeleg allein
aus dem Fensterende. Erst authentisierte Recovery/Replay-Abwehr erlaubt expliziten neuen Auftrag.
Auswahlthreadjoin und Jobleerstand belegt. Inkonsistente neue Modusprüfung blockierte den Erstlauf
korrekt; behoben und mit frischer Autorität erneut geprüft, kein Journal-/Keyreset. Eigener frischer
ETW-Beleg dieses Helferhashs inzwischen oben verlinkt; funktionale Prüfung ist keine Netzabnahme.
Striktes Ziel und offene externe Dienst-/Produktgrenzen bleiben unverändert.

Eigener
[frischer ETW-Beleg des manuellen Brokerpaars](../testing/native-combined-assistive-restart-etw.md):
Automatik und vier manuelle Tastatur-Wiederanläufe in getrennten gültigen Phasen; acht Kontrollen,
Nullverluste, 906 explizite Enden, 0 beobachtete Nicht-Loopback-/514 getrennte Loopback-Ereignisse.
Tastaturbedienung menschlich bestätigt; zusätzliche DOM-Pointer-Ereignisse erlauben keine
unabhängige Bestimmung der physischen Eingabe. Kein neuer Runtimecode; Bindung an das exakte
Hashpaar und die erfassten Lebenszeiten. Externe Dienste und Produkt nicht attestiert, striktes Ziel
unverändert.

Manuelle kombinierte Recovery:
[Assistiver Nachweis](../testing/native-combined-assistive-restart.md) für vier Zustände mit realem
Replay und regulärem Ende; ohne Auswahlhandles und ohne neuen Grant. Zwei Erstversuche ohne nativen
Replay-Gegenbeleg wurden trotz menschlicher Bestätigung abgelehnt; kein Abschlussbeleg und keine
neue Autorität daraus. Hörbestätigung allein ist keine technische Abnahme. Eigener neuer
ETW-/Tastaturbeleg inzwischen oben verlinkt. Der folgende ältere Netznachweis bleibt auf sein
ursprüngliches Paar begrenzt.

Kombiniertes v3-Paar:
[Eigener frischer ETW-Nachweis](../testing/native-combined-live-restart-etw.md) mit realer manueller
Dateiauswahl und automatischer Crash-/Negativserie bestanden. 868 vollständig beendete Lebenszeiten,
acht Messkontrollen, Nullverluste, keine beobachteten Nicht-Loopback- Ereignisse im zugeordneten
Baum. Separate native Stilllegung nach ihrer Aufzeichnung offen benannt; neun automatische
Stilllegungen sind im automatischen Baum enthalten. Externe COM-/Shell-/DNS-/ Broker und Inhalte
werden dadurch nicht vollständig kausal attestiert. Gilt nur für das dokumentierte Hashpaar, keine
Produktfreigabe oder Abschwächung des strikten Ziels.

Aktueller Forschungsbroker [v3](../data-format/native-broker-recovery-v3.md): Initiale Live-Handles
nur für den gebundenen neuen Auftrag, keine bei Recovery. Unveränderliche signierte Grantfolge,
verschiedene IDs, Vorgängerhash und private Abschlussbelege verhindern neue Autorität allein aus
Neustart, Diagnose/PID oder unvollständigem Journal. Kette/Schlüssel bleiben außerhalb der
Containerfreigabe; manipulierte/lückenhafte Folge oder unbestätigter Abschluss sperren. Native
[Negativserie und manuelle Auswahl](../testing/native-combined-live-restart.md) geprüft. Shell-
Pfaddarstellung ändert keine kanonische Identität oder Auswahlgrenze. Kein Schutz gegen
kompromittiertes Konto/Administrationsrechte; Crash vor Journal-Claim kann konservativ blockieren.
Der eigene ETW-Beleg steht oben; assistive/weitere Fensterendeprüfung und Produktintegration bleiben
offen, keine Übertragung auf andere Hashpaare.

Historische Broker-Authority [v2](../data-format/native-broker-recovery-v2.md) entfernt den
DPAPI-Domänenpfad und nutzt lokale HMAC-Authentisierung. Schutz des Klartextschlüssels hängt an
privater Windows-DACL und unverknüpften Dateien; sechs tatsächliche Dateiöffnungen je Appstart unter
dem eingeschränkten Token müssen verweigert sein. Kein Schutz vor demselben kompromittierten Konto,
Administrationsrechten, Schlüsselkopien oder Prozessspeicherauslese. Keine zusätzliche
Verschlüsselung/Hardware-/Kontogeheimnisbindung. Fehlende/beschädigte Schlüssel/Altdaten sperren
statt neue Autorität zu erzeugen. [Native Prüfung](../testing/native-hmac-broker.md) und
[eigener gültiger ETW-Beleg](../testing/native-hmac-broker-etw.md) des exakt gesicherten Paars: 446
vollständig beendete Lebenszeiten, vier Kontrollen, Nullverluste, keine beobachteten Ereignisse
außerhalb Loopback im Baum. Keine kausale externe Dienst-/Inhaltsattestierung; Produktintegration
weiter offen. Folgende DPAPI-Grenze betrifft den alten Zwischenstand.

Offene externe Primitive: Kontogeschützte DPAPI kann laut
[Microsoft](https://learn.microsoft.com/en-us/troubleshoot/windows-server/identity/dpapi-masterkey-backup-failures)
Domänencontroller-/RPC-Pfade für Masterkey-Backup/-Wiederherstellung nutzen. Ein ETW-Appbaum allein
belegt diese Grenze nicht. Die Forschungsimplementierung darf deshalb erst nach gesondertem Nachweis
unter dem strikten Ziel oder Ersatz der Primitive produktiv integriert werden. Keine
Domänen-/OS-Richtlinienänderung als Freigabeersatz.

Controllerverlust: [Dauerhafter nativer Beleg](../data-format/native-broker-recovery-v1.md)
authentisiert Wiederanlauf gegenüber dem Container; geschützte DACL, nicht vererbte exklusive
Sperre, Wurzel-/App-/Brokerhashbindung, Jobleerstand und frische Startpipes. Jobzuweisung bei
Prozesserzeugung verhindert die bisherige nachträgliche Zuweisung im neuen Brokerpfad.
[Native Crash-/Negativserie](../testing/native-controller-crash.md) bestanden. DPAPI ist keine
Attestierung fremder Prozesse desselben Kontos; kein Schutz gegen Kontokompromittierung,
Administrationsrechte oder beliebige Manipulation der containerbeschreibbaren Journale.
Authentisierte Produktupdates/-migration und kombinierte Live-Aufträge fehlen. Neuer Broker ohne
eigenen ETW-Beleg; striktes Ziel und fehlende Produktabnahme unverändert.

ETW v2: Das neue gesicherte Paar besitzt nun einen eigenen gültigen Netzbeleg mit Automatik und
beiden manuellen Wiederanlaufpfaden: drei vollständige Bäume, 32 App-Generationen, 238 explizite
Prozessenden, vier Kontrollen, Nullverluste, 0 Nicht-Loopback-Ereignisse. 176 Loopback-Ereignisse
getrennt. [Nachweis und Vertrauensgrenzen](../testing/native-assistive-restart-etw.md). Keine
Inhalts-/HTTP-Attestierung oder kausale Abnahme externer COM-/Shell-/DNS-/Broker; keine neue
Runtimefähigkeit oder Produktfreigabe. Striktes Ziel unverändert.

Assistive Fortschreibung: Startticket v2 erlaubt deaktivierte Bedienungsautomatik nur für
Wiederherstellung. Alte Versionen/fehlendes Pflichtflag werden nicht durch Defaults akzeptiert.
Keine neuen Grants, Rendererparameter, Hostzugriffe oder Netzrechte; die manuelle Betrachtungsfrist
erneuert keine Dateiautorisierung. [Vertrag](../data-format/native-ui-restart-v2.md),
[getrennte Hör-/Diagnosenachweise](../testing/native-assistive-restart.md). Für das neue Paar fehlt
frisches ETW, der v1-Beleg bleibt ausschließlich an v1-Hashes gebunden.

ETW-Fortschreibung: Das gesicherte Neustart-Paar besitzt jetzt einen neuen gültigen Netzbeleg: 28
zeitlich abgeglichene App-Generationen, alle 202 Prozesslebenszeiten explizit beendet, vier
Kontrollen, Nullverluste, keine unbekannten Ziele/Zuordnungslücken, 0 Nicht-Loopback-Ereignisse; 144
Loopback-Ereignisse separat. [Nachweis und Grenzen](../testing/native-ui-restart-etw.md). Keine
Runtimeänderung oder zusätzliche Fähigkeit. Dieser Transportmetadatenbefund attestiert weder Inhalte
noch externe Dienste oder kompromittierte Prozesse; Produkt-Q-01/Q-02 bleiben offen.

Neustart-Fortschreibung: Forschungsstarts sind an einmalige anonyme Pipes, Launch-ID, exakte
AppContainer-SID, abgeleitete Datenwurzel und Programmhash gebunden. Flags vor WebView gelöscht und
geprüft; keine Rendererhandles, Tickets, Quellen oder Crashpunkte. Der lebende Controller behält
Identität/Ablage und wartet vor jedem Neustart auf Job-Leerstand. Native 28-Generationen-Serie und
unabhängige Datenprüfung bestanden. [Vertrauensgrenzen](../data-format/native-ui-restart-v1.md),
[Nachweis](../testing/native-ui-restart.md). Funktionale Zusammenfassungen erteilen keine Hostrechte
und attestieren keinen kompromittierten Container. Controller-Crash, dauerhafter Broker, externe
Dienste, neuer ETW-Beleg und Produktabnahme bleiben offen.

Auftrags-Fortschreibung: Das Forschungsjournal verbraucht die native Kennung vor Auswahl/Lesen,
serialisiert Bearbeitung/Reconciliation per SQLite-Eigentümersperre und speichert den Commit-Receipt
vor finalem Rename. Wiederanlauf publiziert niemals Zwischenstände; fehlender oder unklarer Zustand
blockiert statt neu freizugeben. Keine neuen Rendererpfade, Hostordner-ACLs oder Netzrechte.
[Vertrag mit Restbedrohungen](../data-format/native-task-journal-v1.md) und
[Prozess-Kill-/Desktopnachweise](../testing/native-task-reconciliation.md). Keine Attestierung gegen
kompromittierte Prozesse mit gleichen Schreibrechten, Stromausfälle oder externe Broker;
integrierter isolierter GUI-Neustart, neuer ETW-Beleg und Produktabnahme bleiben offen.

Assistive Fortschreibung: Die Forschungsseite ergänzt eine lokale Fokusdiagnose mit strengem
Enum-/Bool-DTO, Ablehnung unbekannter Felder, gleicher Origin-/Fensterprüfung und festem Ziel
`ui-focus.json` in der bestehenden Forschungsdatenfreigabe. Höchstens 64 Einträge/16 KiB; keine
Texte, Pfade, Tastencodes oder zusätzlichen Rechte. Angaben des Renderers sind keine Autorisierung.
[Native Fokus-/Hörnachweise und Grenzen](../testing/native-live-assistive-focus.md). Kein neuer
ETW-Nachweis des geänderten App-Builds, keine Produktfreigabe, striktes Ziel unverändert.

Fensterende-Fortschreibung: Der Forschungscontroller schließt bei Appende seine Pipe-Duplikate,
begrenzt den Auswahlthread-Abschluss auf fünf Sekunden und sendet Schließnachrichten ausschließlich
an Dialogfenster seines eigenen Prozesses und ermittelten Auswahlthreads mit Klasse `#32770`. Keine
rendererbestimmten Fensterhandles, Prozess-IDs oder Shell-Verben. Nativ bestätigte
Dialog-/Teiltransferabbrüche samt Cleanup sind
[gesondert belegt](../testing/native-live-window-close.md). Die neue OS-API-Featureauswahl erweitert
nur den getrennten Forschungshelfer, keine Produkt-Capabilities oder Netzrechte. Ein neuer
[ETW-Nachweis dieses unveränderten finalen Builds](../testing/native-live-window-close-etw.json)
bestätigt drei vollständige Bäume, vier Kontrollen, Nullverluste und keine Ereignisse außerhalb
Loopback. Keine Abwehr kompromittierter Runtime-/OS-Prozesse, keine kausale Zuordnung externer
COM-/Shell-/DNS-/Brokerpfade und keine Produktfreigabe.

Fortschreibung Live-Kanal: Der Replay-Anzeigefehler nach Importfehler ist korrigiert und als
Browserregression geprüft; Rust protokolliert Ablehnung datenarm. Neue native Erfolg-/Fehler-/
Fensterendeserie und ETW des korrigierten Forschungsbuilds: drei vollständige Bäume, vier positive
Kontrollen, Nullverluste, keine Ereignisse außerhalb Loopback; SQLite und Cleanup unabhängig
bestätigt. [Exakte Build-/Messgrenzen](../testing/native-live-file-selection.md). Appende bei
offenem Dialog/Transfer, assistive Prüfung und externe COM-/Shell-/DNS-/Brokerpfade bleiben offen.
Kein Produkt-Q-01/Q-02-Pass und keine Lockerung des strikten Datenschutzziels.

KQ-069-Fortschreibung: `ui-live` erlaubt aus der laufenden isolierten Forschungs-App genau ein
festes Auswahlsignal über ein gezielt vererbtes Pipe-Handle. Hostauswahl ausschließlich einer
synthetischen Datei; zweite Pipe liefert begrenzten Auftrag und Archivbytes zurück. Renderer
erhalten keine Handles/Pfade. Host enthält keine Formatparser; atomarer Import bleibt im Container.
[Vertrag](../data-format/native-live-file-channel-v2.md),
[Nachweise/Grenzen](../testing/native-live-file-selection.md). Neue Vertrauensgrenze: Replay,
Appende während Dialog/Transfer, assistive Fokusführung, externe Shell-/Brokerpfade und Netzbilanz
benötigen weitere Abnahme. Keine neue WebView-Netzfähigkeit oder Produktfreigabe.

## KQ-069 A: Tauri-Forschungsfeature mit Einweg-Grant

Der [neue Kanal](../data-format/native-file-channel-v1.md) akzeptiert einen begrenzten typisierten
Auftrag aus einer vor Prozessstart beschriebenen und abgeschlossenen anonymen Pipe. Datei nur
lesend, Start-Whitelist mit zwei Handles, Empfängerclaim einmalig; Flags vor WebView gelöscht und
geprüft. Renderer kann keine Handles/Hostpfade oder Freigaben nachliefern. Vier parameterlose lokale
Befehle, zusätzliche Origin-/Fensterprüfung, keine Core-/Plugin-Berechtigungen. Rust-Mutex entnimmt
den Grant einmalig. Controllerantwort streng typisiert und größenbegrenzt; kein Shell-/RPC-Kanal.
Nur Forschungsfeature, feste synthetische Datei, kontrollierter Launcher.

UUID allein authentisiert keinen Peer. Kein Schutz gegen gleichberechtigte/privilegierte lokale
Angreifer oder vollständiger Nachweis gegen kompromittierte Runtime-Prozesse. Produktstart
unisoliert. Finaler Build nativ mit Import/Wiederholung, Abbruch und Fensterende am Haltepunkt
bestanden; keine Stagingreste, alle Jobs/Profile bereinigt. Gültiges ETW: drei vollständige Bäume,
vier Kontrollen, keine Verluste, 0 Ereignisse außerhalb Loopback. Keine vollständige Nachfahren-
Token-/Handleattestierung oder externe COM-/Shell-Zuordnung. Vorabauswahl, Auswahl aus laufender
App, Assistenz und dauerhafte Crash-/Replay-Reconciliation offen. Q-01/Q-02 nicht bestanden.

## KQ-069 A: Einmaliger Auftrag im nativen App-Dienst-Testadapter

Der [isolierte Snapshotimport-Testadapter](../testing/native-isolated-app-import.md) erhält nur
delegierte Handles und einen begrenzten typisierten Auftrag: gebundene UUID, feste Operation,
Version, Größe und höchstens 60 Sekunden Lebensdauer. Jeder Versuch verbraucht die Instanz; kein
automatischer Retry. Formatparser/SQLite bleiben im Worker ohne Capabilities. Publish-Guard nach
vollständigem Aufbau im Stagingordner: Abbruch/Ablauf entfernen den Zwischenstand. Nach Freigabe
beginnt der Commit; spätere Fehler bedeuten keinen garantierten Rollback.

Native Serie bestätigt neue ID, Unicode, Integrität/Fremdschlüssel, bytegleiche Testbasis, Replay-
Ablehnung und Cleanup. Gültiges ETW des finalen Builds: keine Netzereignisse im Controller-/
Workerbaum. Kein Nachweis für externe COM-/Shell-Dienste, integrierten WebView, echte Nutzdaten oder
Produktstarts. UUID/Zeitfelder alleine autorisieren keinen beliebigen Auftraggeber; nur der eigene
einmalige Forschungscontroller erzeugt die Freigabe. Sicherer UI-/Tauri-Auftragskanal, dauerhafte
Replay-/Crash-Reconciliation, echte Auswahl und assistive Abnahme fehlen. Keine neuen
Hostordnerrechte, Provider oder Shell-Aufrufe. Q-01/Q-02 nicht bestanden; #134 offen.

## KQ-069 A: Begrenzter Datei-/Shell-Transport

Der [synthetische Austauschversuch](../testing/native-runtime-file-exchange.md) trennt Shell-Auswahl
und Dateiverarbeitung: nicht isolierter Controller mit zwei explizit vererbten Datei-Handles, Worker
im eigenen AppContainer ohne Capabilities. Der Hostordner liegt außerhalb jeder neuen Paket-ACL.
Ausgewählte Pfade bleiben im Controller, Parser/Nutzbyteverarbeitung im Worker. Die fest bekannte
synthetische Negativdatei wird nur zu Testzwecken aus dem Wurzelnamen abgeleitet. Gepinnte
Eingabehandles, Metadatenprüfung gegen Reparse/Cloud/Hardlinks, 8-MiB-Budget, begrenzte Berichte,
suspendierte Tokenprüfung, Job und Veröffentlichung ohne Überschreiben begrenzen den Versuch.
Fehler/Abbruch veröffentlichen keine fertige Ausgabe. Kein allgemeiner Shell-/URL-Broker.

Das ist kein freigegebener Broker für echte Dateien. Shell-/COM-Dienste außerhalb des Elternbaums,
Handler, Dateisystemrennen mit gleichberechtigten lokalen Angreifern, einmalige autorisierte
App-Aufträge, Abbruch/Lebensdauer und Produktformatbudgets benötigen eigene Nachweise. Die ETW-
Messung gilt nur für den gemessenen Controller-/Workerbuild und dessen Baum; der finale stärkere
Cleanup-/Berichtsstand ist separat nativ geprüft. Keine Netzfähigkeit für WebView2 und keine
fachliche Ausnahme. Produkt unverändert unisoliert, Q-01/Q-02 nicht bestanden.

## KQ-069 A: Gemessener AppContainer-Prototyp, keine Produktgrenze

Der Forschungslauncher startet ausschließlich synthetische Testdaten unter einer eigenen
AppContainer-SID ohne Capabilities, prüft den suspendierten Root und verwendet einen Job ohne
Breakaway mit begrenzter Laufzeit. Nur sein TEMP-Datenbereich erhält zusätzliche Rechte. ETW findet
bei drei vollständigen isolierten App-Läufen keinen Verkehr außerhalb Loopback; der explizite nicht
isolierte Vergleich zeigt 201 öffentliche WebView2-Ereignisse. Beide Messungen sind gültig. 23
Tokenstichproben sind keine Vor-Ausführungsattestierung aller 25 Prozesslebenszeiten.

Die CLI bleibt ein nicht isolierter Forschungscontroller mit Diagnosekontrollen, kein freigegebener
Broker für echte Manuskripte. Normale App-/Installerstarts sind unverändert unisoliert. Öffentliche
UDP-Zustellung, IPv6, DNS-/OS-/COM-/Shell-Broker und Prozesse außerhalb des Eltern-/Jobbaums sind
nicht vollständig abgenommen. Loopback-Connects gelingen; der Dateidialog öffnet im Container nicht.
Keine breite Benutzerprofilfreigabe oder Internetfähigkeit zur Umgehung dieser Grenze. Der Prototyp
verwendet eine separate Win32-FFI-Grenze; Produktcrates behalten `unsafe_code = forbid`. Q-01/Q-02
bleiben nicht bestanden. [Protokoll](../testing/native-runtime-isolation.md),
[beauftragte Richtung A](decisions/0034-webview-network-boundary-proposal.md).

## KQ-069: Runtime-Dienste auch ohne Projektdaten

Der zusätzliche lokale NetLog zeigt bereits bei leerem Projektbestand Konfigurations-GETs und
POST-Starts an der Komponentenupdate-Route. Das grenzt den Dienst ein, beweist aber keine zulässigen
Body-Inhalte oder fehlende Projektübertragung in anderen Abläufen. NetLog ist keine vollständige
Windows-Netzgrenze. Rohlogs mit URLs/Kennungen/Headers bleiben lokal; Git enthält nur eine
eingeschränkte Projektion. Der nach Nutzerfreigabe wiederholte ETW-Lauf hat vollständige
Prozessenden, vier positive Kontrollen und keine Verluste; 256 öffentliche WebView2-Ereignisse
bestätigen den Befund. Keine Unterdrückungsmaßnahme implementiert. Q-01/Q-02 bleiben nicht
bestanden. [Protokoll](../testing/native-m3-webview-network.md) und
[offene Entscheidungsvorlage ADR 0034](decisions/0034-webview-network-boundary-proposal.md).

## KQ-068: Gemessene Windows-Laufzeitgrenze

ETW findet ohne Provideraufruf öffentliche Netzverbindungen von WebView2-Nachfahren der isolierten
App. Seiten-CSP und fehlender eigener Telemetriecode verhindern nicht nachweislich sämtlichen
Windows-Laufzeitverkehr. Transportmetadaten belegen weder HTTP-Zweck noch Upload. Rohdiagnosen mit
fremden Prozess-/Netzmetadaten bleiben lokal außerhalb von Git.

Q-01/Q-02 sind nicht freigegeben. [Issue #134](https://github.com/kolkrabeofdoom/kraqel/issues/134)
verlangt Ursachenklärung, unterstützte App-Maßnahmen und gegebenenfalls einen ausdrücklichen
Fachentscheid zur Laufzeitgrenze. Startseite/README präzisieren die Telemetrieaussage.
OS-/Firewalländerungen oder unsichere Browserflags sind keine Abnahme des Produkts. Siehe
[Protokoll](../testing/native-m3-coverage.md) und
[Microsofts WebView2-Datenschutzdokumentation](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/data-privacy).

## KQ-055: Begrenzter DOCX-Eingang

DOCX ist ein nicht vertrauenswürdiges ZIP-/XML-Paket. ADR 0024 und
[`../data-format/DOCX_IMPORT.md`](../data-format/DOCX_IMPORT.md) begrenzen Archivgröße, Einträge,
entpackte Daten, Kompressionsverhältnis, XML-Tiefe, Knotenzahl und Text. Die ZIP-Headerkonsistenz
und doppelte Namen werden ausdrücklich geprüft, bevor Einträge verarbeitet werden. Keine Extraktion,
Office-Ausführung, Makroausführung oder externe Beziehung ist erlaubt. XML wird namespacebewusst
ohne DTD-/Entity-Resolver gelesen. Bekannte Hilfsteile bleiben untrusted; Vorschaudaten werden nicht
gerendert, Content Controls nicht übernommen.

Unbekannte Inhaltssemantik wird vor der einzigen schreibenden Storage-Transaktion abgelehnt. Der
Import erzeugt eine neue Szene und verändert bestehende Inhalte nicht. Pfade und Rohfehler verlassen
Rust nicht. Die bestehende TOCTOU-Grenze gegenüber böswilligen parallelen Dateisystemänderungen
bleibt offen; DOCX ist kein neuer vertrauenswürdiger Projektpfad.

## Schutzgüter

- unveröffentlichte Manuskripte und Story-Bibel-Daten
- Bilder, Prompts, Metadaten und lokale Pfade
- API-Schlüssel und Provider-Konfiguration
- Integrität von Projekten, Backups und Git-Snapshots

## Relevante Bedrohungen

- Pfadtraversal, Symlink-Fluchten und Überschreiben außerhalb des Projektordners
- unbemerkte Cloudübertragung oder zu weit gefasste Kontextfreigabe
- Prompt-Injection aus importierten Dokumenten und Medienmetadaten
- bösartige oder übergroße Importdateien
- Schlüssel oder private Daten in Logs, Snapshots und Fehlerberichten
- manipulierte Projekte, Migrationen oder Abhängigkeiten
- manipulierte oder verwechselte Installations- und Releaseartefakte

## Grundkontrollen

- minimale Tauri-Capabilities und sämtliche Dateizugriffe in Rust
- kanonische, komponentenweise Pfadprüfung ohne Symlinks; Größenlimits und Typprüfung bei Importen
- atomare Projekt-, Backup- und Recovery-Veröffentlichung innerhalb desselben Dateisystems
- explizite Kontextvorschau und Freigabe vor jedem Cloudlauf
- lokales Ollama nur über eine feste Loopback-Adresse; Systemproxies und HTTP-Redirects sind
  deaktiviert, Antworten und Anfragen besitzen Größenlimits
- Kontextfreigaben binden Dokument-ID und Revision; Änderungen zwischen Vorschau und Lauf führen zum
  Konflikt statt zur Übertragung eines anderen Textstands
- Titeländerungen vergleichen den zuletzt angezeigten Titel; Baumumordnungen vergleichen alte und
  neue vollständige Geschwisterlisten. Löschen bindet Titel, Revision und bei Kapiteln die
  vollständige Kindliste und erfordert in der UI eine zweite Bestätigung
- Vorschlags-Diffs laden die Szene erneut, verlangen dieselbe Revision und besitzen keinen
  Schreibpfad; regelbasierte Kontinuitätsprüfungen bleiben vollständig lokal
- Bildimporte werden in Rust nach Signatur, Endung, Bytegröße, Pixelgrenzen und Dekodierbarkeit
  geprüft; Projektdateinamen entstehen nur aus UUID und Hash
- Backup und portable Snapshots kopieren ausschließlich datenbankreferenzierte Assetpfade und
  verifizieren Originale vor Veröffentlichung beziehungsweise Wiederherstellung gegen SHA-256
- gepackte Snapshotimporte besitzen Gesamt-, Anzahl- und Einzeldateigrenzen, akzeptieren weder
  Symlinks noch unbekannte oder ausbrechende ZIP-Einträge und prüfen alle Referenzen sowie Bilder
  vor der atomaren Veröffentlichung als neues Projekt mit neuer Kennung
- visuelle Profile und Stilvorlagen sind projektgebunden, größenbegrenzt und werden nur nach
  erneuter Rust-Validierung gespeichert; ihre Commands besitzen keinen Netzwerkpfad
- Bildprompt-Entwürfe entstehen nur aus explizit gewählter Szene, visuellen Profilen und optionaler
  passender Stilvorlage; Szenentext wird als unvertrauenswürdiger Inhalt markiert, Quellrevisionen
  werden beim ersten Speichern geprüft und kein Prompt-Command besitzt einen Netzwerkpfad
- Bildfähigkeiten werden je Provider und Modell aus einer lokal validierten Registry gelesen;
  unbekannte Provider, Modelle, Fähigkeiten, Größen und Referenzmengen werden vor einem möglichen
  Provideraufruf abgelehnt
- Bildfreigaben enthalten exakt den angezeigten Prompt und die ausgewählten Referenzasset-UUIDs;
  eine Promptansicht oder ein gespeicherter Entwurf gilt nicht als Übertragungsfreigabe
- Generation-Job-Fehler verwenden ausschließlich feste Codes und statische Meldungen; Prompt,
  Providerantworten, Schlüssel, Pfade und fremde Diagnosedaten sind nicht Bestandteil des Fehlers
- Prompt-Textausgaben verwenden feste UUID-Dateinamen und bleiben unter `exports/prompts` im
  verwalteten Projekt; Snapshot v3 prüft ihre Dokument-, Profil- und Stilreferenzen
- Der Exportordner-Aufruf akzeptiert über IPC ausschließlich eine Projekt-ID. Rust kanonisiert das
  verwaltete Projekt, erzeugt nur dessen festes Kind `exports`, lehnt Symlinks und
  Nicht-Verzeichnisse ab und übergibt erst danach den serverseitig bestimmten Pfad an den
  Betriebssystem-Dateimanager
- Betriebssystem-Schlüsselspeicher statt Konfigurationsdateien
- feste offizielle HTTPS-Endpunkte für Cloudprovider; keine Redirects, Umgebungsproxies oder frei
  konfigurierbaren Produktionsziele
- höchstens zehn Minuten gültige Einmalfreigabe bindet Projekt, Dokumentrevision, Provider, Modell
  und Arbeitsauftrag und wird vor dem Cloudrequest verbraucht
- deterministische Snapshots, Prüfsummen und getestete Wiederherstellung
- deny-by-default Git-Pfadfreigabe; keine globale Git-Konfiguration, Hooks oder interaktiven Prompts
- Git-Langpfade werden nur mit `-c core.longpaths=true` pro internem Prozess aktiviert; KRAQEL
  verändert weder globale noch systemweite Git-Konfiguration
- Abhängigkeitsprüfung und reproduzierbare CI
- der Windows-Releasebau akzeptiert standardmäßig nur einen sauberen Git-Stand, genau ein erwartetes
  unsigniertes NSIS-Artefakt und schreibt Commit, Werkzeugversionen, Dateigröße sowie SHA-256 in ein
  Buildmanifest
- die Releaseabnahme lädt Setup, Prüfsumme und Manifest erneut von GitHub, gleicht alle drei gegen
  Commit und Unsigned-Policy ab, installiert nur in ein festes temporäres Ziel und verifiziert, dass
  vorhandene reguläre App-Daten unverändert bleiben

Dieses Dokument wird mit jeder neuen Vertrauensgrenze aktualisiert.

Der Ollama-Adapter ist die erste aktive Providergrenze. Er sendet ausschließlich den unmittelbar
zuvor angezeigten Szenenklartext und den Arbeitsauftrag an `127.0.0.1`. Modellnamen, Providerfehler
und Streamereignisse werden bereinigt; Prompts und Antworten werden nicht geloggt oder automatisch
in kanonische Dokumente geschrieben. Ein lokal kompromittierter Ollama-Prozess kann den
freigegebenen Text dennoch lesen und bösartige beziehungsweise übergroße Antworten liefern; deshalb
bleiben Größenlimits, strikte JSONL-Validierung und manuelle Übernahme erforderlich.

Die Cloudadapter für OpenAI, Anthropic, Google und Mistral lesen Schlüssel ausschließlich innerhalb
der Rust-Grenze aus dem nativen Betriebssystemspeicher. Gespeicherte Werte werden nicht über IPC
zurückgegeben, in Projektdateien geschrieben oder geloggt. Beim erstmaligen Speichern existiert der
eingegebene Wert zwangsläufig kurz im React- und IPC-Speicher; danach wird er nur als
vorhanden/fehlend ausgewiesen. Der Provider sieht bei einer ausdrücklich ausgelösten Modellabfrage
bereits Konto und Netzwerkadresse, auch wenn noch keine Projektdaten übertragen werden.

Ein kompromittierter Betriebssystemspeicher, Root-/Administratorzugriff oder ein kompromittierter
Prozess kann Schlüssel und freigegebene Inhalte weiterhin lesen. KRAQEL kontrolliert außerdem weder
Kosten noch serverseitige Aufbewahrung, Trainingsnutzung oder Verfügbarkeit der Anbieter. Vor
produktiver Nutzung sind deren aktuelle Verträge und Datenschutzeinstellungen zu prüfen. Lokale
Protokolltests ersetzen keine echte Anbieterabnahme.

Der Snapshot-v3-Archivimport ist schreibend, aber absichtlich nicht überschreibend: Er importiert
nur in ein zufällig benanntes Staging-Verzeichnis unter der verwalteten Projektwurzel, vergibt eine
neue Projektkennung und veröffentlicht erst nach vollständiger Validierung und Befüllung.
ZIP-Bomben, Pfadtraversal, Symlinks, unbekannte Einträge, manipulierte Referenzen und Bilder werden
begrenzt oder abgelehnt. Die native Dateiauswahl und das begrenzte Lesen regulärer Dateien bleiben
in Rust; React erhält weder absoluten Pfad noch Archivbytes. Metadaten werden vor und nach dem
Öffnen verglichen, wodurch einfache Austauschversuche erkannt werden; ein vollständiger Schutz gegen
böswillige parallele Dateisystemänderungen ist ohne plattformspezifische Handle-APIs nicht belegt.
Weitere Fuzztests sowie die Bedienführung für Herkunft und Lizenz bleiben zu bewerten.

Der Snapshotexport hält den nativen Zielpfad und sämtliche Archivbytes in Rust. Er erzeugt immer
zuerst einen validierten Snapshot v3, sortiert Archivpfade stabil, begrenzt Dateianzahl sowie
komprimierte und unkomprimierte Größe und lehnt Symlinks oder Nicht-Dateien als Ziel ab. Ein
Dialogabbruch schreibt nichts. Dateisystemrennen zwischen Prüfung und Veröffentlichung bleiben wie
bei anderen plattformübergreifenden Pfadoperationen ein Restrisiko.

Projektlöschung ist endgültig und daher zweistufig: Der erste UI-Klick aktiviert nur die konkrete
Bestätigung; der zweite sendet Projekt-ID und Bestätigungsflag. Rust löst ausschließlich die
UUID-Komponente unter der kanonischen Projektwurzel auf, öffnet die Datenbank, vergleicht die dort
gespeicherte Projektkennung und entfernt erst danach das Verzeichnis. Dieser Schutz verhindert
Fehladressierung, ersetzt aber kein Backup gegen eine bewusst bestätigte Fehlentscheidung.

Die Bildprovider-Grenze besitzt in Version 0.1 nur Verträge, Registryvalidierung und einen lokalen
Test-Mock. Es gibt keinen produktiven Adapter, keinen Tauri-Command, keine Jobpersistenz und keine
Bildausgabe. Vor KQ-017 müssen Cloudfreigaben wie bei Text kurzlebig und einmalig gebunden,
Referenzdateien unmittelbar vor Übertragung erneut hashgeprüft, Ergebnisse nach Signatur und Größe
validiert sowie Kosten-, Lizenz- und Aufbewahrungsinformationen sichtbar gemacht werden.

Bilddecoder bleiben trotz Signatur-, Format-, Größen- und Pixelgrenzen eine Angriffsfläche für
präparierte Dateien. Deshalb ist `image` exakt und mit nur drei aktivierten Formaten gebunden und
muss regelmäßig aktualisiert werden. Der aktuelle WebView-Import überträgt außerdem das gesamte Bild
als Bytearray; die 25-MiB-Grenze gilt in Oberfläche und Rust, verhindert aber nicht jede kurzzeitige
Mehrfachbelegung beim Deserialisieren. Ein nativer Streamingimport ist vor wesentlich größeren
Einzeldateien oder Massenuploads neu zu bewerten.

Der native Textimport begrenzt `.txt`/`.md`/`.markdown` auf 2 MiB, reguläre unverknüpfte Dateien,
stabile Dateilänge, UTF-8 und NUL-Freiheit. Pfad und Bytes bleiben in Rust; die Veröffentlichung von
Szenenknoten und Editorinhalt ist transaktional. Zwischen Metadatenprüfung und Lesen bleibt ein
plattformübergreifendes TOCTOU-Restfenster. Unbekannte Markdown-Syntax wird sichtbar übernommen,
aber nicht als aktives HTML interpretiert; beliebige CommonMark-Erweiterungen sind nicht Teil des
Vertrags.

Der native DOCX-Roundtrip-Kern (KQ-060 / ADR 0027) verarbeitet alle fremden Pakete ausschließlich in
Rust, ohne Netzwerk, Python oder Office-Ausführung. Neben den gemeinsamen ZIP-/XML-Budgets prüft er
eine enge Paket-/Beziehungs-/MIME-Allowlist, versioniertes Inventar, vollständige Markerfolge,
verwendete Styles und Nummerierung. Sichtbarer Text ist die einzige Textquelle. Die Hashbindung
erkennt Verlust einzelner Inventarteile, stellt aber keine Herkunftssignatur dar; auch ein neu
gebundener Baum muss sämtliche Modellregeln erfüllen. Jede Writer-Ausgabe wird vor atomarer
Dateiveröffentlichung nativ semantisch zurückgelesen. KQ-061 ergänzt datenarme Command-v1-Aktionen
und native Auswahl. Ein Import erhält immer neue Projekt-/Dokument-Identitäten, befüllt
ausschließlich ein leeres Staging-Projekt transaktional und vergleicht alle gespeicherten Inhalte
vor atomarer Veröffentlichung. Abbruch schreibt nichts; fehlerhafte Inhalte oder Speichergrenzen
veröffentlichen kein Teilprojekt. Migration 6 erhält Referenzen und FTS beim Tabellenumbau, prüft
Fremdschlüssel vor Commit und aktiviert sie anschließend auch bei Fehlern erneut. Die
Dateisystemrennen des bestehenden Projektmanagers bleiben ein Restrisiko.

Der Windows-0.1-Releasepfad ist bewusst unsigniert. SHA-256 erkennt Änderungen nur gegenüber einem
vertrauenswürdig bezogenen Sollwert und stellt keine Herausgeber*innenidentität her. Werden Setup,
Manifest und Prüfsumme über denselben kompromittierten Kanal ersetzt, bleibt die Manipulation damit
unentdeckt. Release Candidates dürfen deshalb nur aus einem dokumentierten sauberen Commit gebaut,
gemeinsam über das private Repository verteilt und vor Veröffentlichung erneut heruntergeladen und
geprüft werden. SmartScreen-Warnungen sind erwartbar; testende Personen dürfen sie nur nach Quellen-
und Hashprüfung bewusst übergehen. Eine spätere breitere Verteilung erfordert eine erneute Bewertung
von Signierung und unabhängiger Prüfsummenveröffentlichung.

Der GitHub-Entwurf und sein Rückdownload verringern Verwechslungs- und Uploadfehler, lösen aber
nicht das Risiko eines vollständig kompromittierten GitHub-Kontos oder Kanals. KQ-037 veröffentlicht
das unveränderte Artefakt bewusst als Alpha-Prerelease; Release Notes, Unsigned-Warnung und SHA-256
reduzieren Fehlannahmen, ersetzen aber weder Signatur noch unabhängigen Vertrauenskanal. Die lokale
KQ-036-Installation startet die Anwendung absichtlich nicht, weil auf dem Entwicklungsrechner reale
KRAQEL-App-Daten vorhanden sind. SmartScreen, Erststart und Persistenz müssen deshalb weiterhin in
einer Umgebung ohne schützenswerte Bestandsdaten geprüft werden.

KQ-039 wiederholt Build, GitHub-Rückdownload sowie isolierte Installation und Deinstallation für
Alpha 2 mit eingebetteter Beispielvorlage. Die größere Binärdatei und das zusätzliche Archiv ändern
die Vertrauensgrenzen nicht: Tag, Manifest, Prüfsumme und Setup bleiben gemeinsam vom GitHub-Kanal
abhängig. Die offene Clean-VM-Abnahme gilt deshalb unverändert auch für `v0.1.0-alpha.2`.

KQ-041 veröffentlicht Alpha 3 aus dem geprüften KQ-040-Commit. Entwurfsprüfung und öffentlicher
Rückdownload belegen identische Assets; der parallele Betrieb der regulären App verhinderte beim
öffentlichen Paket lediglich einen unverfälschten App-Daten-Fingerabdruck. Die Vertrauensgrenzen und
die offene Clean-VM-Abnahme gelten unverändert auch für `v0.1.0-alpha.3`.

KQ-045 veröffentlicht Alpha 4 aus dem geprüften KQ-044-Commit. Release Candidate und öffentlicher
Rückdownload sind bytegleich und bestanden jeweils die isolierte Installation und Deinstallation;
bestehende App-Daten blieben unverändert. Da das installierte Paket auf diesem Rechner nicht gegen
reale App-Daten gestartet wurde, gelten SmartScreen-, Start- und Clean-VM-Risiken auch für
`v0.1.0-alpha.4` fort.

Verbleibendes Risiko: Die kanonische Vorabprüfung ist nicht vollständig gegen ein gleichzeitiges
böswilliges Austauschen bereits geprüfter Verzeichnisse abgesichert. Vor der Unterstützung
unvertrauenswürdiger Projektwurzeln sind handle-relative Betriebssystemzugriffe zu bewerten.

Die KQ-043-Dubletten-Zusammenführung ist eine destruktive Fachoperation und deshalb ausschließlich
nach expliziter Quell-/Zielauswahl, sichtbarer Eigenschaftskonfliktvorschau und Bestätigung
erreichbar. Rust prüft Projekt, Art und verschiedene UUIDs erneut und migriert Referenzen sowie
Quelllöschung in genau einer SQLite-Transaktion. KQ-044 vereinigt auch zwei visuelle Profile mit
Zielvorrang und deduplizierten Ausschlüssen; ungültige kombinierte Eigenschaften führen zum
vollständigen Rollback. Eine automatische Ähnlichkeitserkennung löst den Merge niemals selbst aus.

KQ-063 setzt den freigegebenen Medienexport-Vertrag für Markdown/HTML in Rust um. Nur ausdrücklich
gewählte direkt verknüpfte Szenenbilder werden aus eingefrorenen, hashgeprüften Originalbytes
abgeleitet. Größen-, Pixel-, Mengen- und Paketbudgets gelten unabhängig von der UI; neue PNGs
enthalten keine übernommenen Metadaten. Das Manifest verwendet paketlokale IDs und lässt private
Originalnamen, Pfade, Tags und Prompts weg. Bildtexte werden im jeweiligen Ausgabeformat maskiert.
Ein konsistenter SQLite-Lesestand wird vor Veröffentlichung erneut verglichen; eine kurze
Schreibsperre schützt den finalen Vergleich bis zum atomaren Umbenennen der neuen Generation. Fehler
lassen alte Pakete bestehen. Decoderlimits sind keine harte Prozess-RSS-Garantie, und die oben
beschriebene Dateisystem-Austauschlücke bleibt bestehen. Der DOCX-Text-Roundtrip wird nicht
erweitert. Siehe `docs/data-format/ILLUSTRATED_MANUSCRIPT.md` und ADR 0030.

KQ-064 / ADR 0031 nutzt denselben validierten Medienkern für gewöhnliches DOCX. Der Writer erzeugt
ausschließlich interne PNG-Parts, feste interne Bildbeziehungen und Inline-Zeichnungen; Titel und
Alttext werden als XML-Attribute maskiert. Der native ZIP-Selbstvergleich prüft die vollständige
erzeugte Part-Allowlist und sämtliche Bytes vor Veröffentlichung. Es gibt keine externen Bilder,
Makros, OLE-Objekte, Felder oder Erweiterung des Text-Roundtrip-Readers. Office-Metadaten aus
erneuten Speichervorgängen gehören nicht zum App-Datenmodell oder zum ursprünglichen Manifest.

KQ-065 / ADR 0032 verwendet ausschließlich vier festgeschriebene, mitgelieferte TTFs statt
Systemschriften oder heruntergeladener Fonts. NFC und vollständige Glyphenprüfung gehen der
PDF-Erzeugung voraus; nicht unterstützte Zeichen werden nicht still ersetzt. Der eigene PDF-Writer
bettet nur geprüfte RGB-Pixel ein. Ein natives Rücklesen vergleicht vor Veröffentlichung Bildhash,
Abmessungen, Farbprofil und Manifestkennung; PNG-Nachweise enthalten dieselben Pixel auf Weiß.
Text-, Zeilen-, Seiten- und RGB-Budgets begrenzen zusätzlich den Auftrag. Der Rückleser verarbeitet
hier ausschließlich die eigene Ausgabe, keine fremden PDFs. Schriftdateien und neue Bibliotheken
bringen weiterhin Parser-/Lieferkettenrisiken mit; Herkunft, Versionen und Lizenzen sind
dokumentiert. Es gibt keine PDF-Import-, aktive Inhalts- oder universelle Shaping-Funktion.
