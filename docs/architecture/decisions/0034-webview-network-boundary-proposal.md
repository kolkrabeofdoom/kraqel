# ADR 0034: Windows-Laufzeitgrenze nach WebView2-Netzbefund

Aktuell · frisches C4b-Sperrpaar-ETW, 18. November 2026:

[Gezielter frischer Netzvergleich](../../testing/product-start-p2c-c4b-module-gate-etw.md) des
unveränderten Sperrpaars `daf59d0…`/`83679e5…` bestanden: drei Hostaufrufe, zwei Rootkinder und drei
Konsolenhosts, alle acht Enden explizit, acht Positivkontrollen, keine Verluste/Zuordnungslücken und
null beobachtete Netzereignisse in diesen Bäumen einschließlich Loopback. Drei Host- und zwei
Rootgenerationen nativ gegen Kernelzeiten/Eltern abgeglichen. 13 MAC-Records/neun C4-Proberecords
und zwei verbrauchte Launches unabhängig geprüft; authentisierter Root vor Running/Continue
abgewiesen, kein WebView. Beide zusätzlichen Modulpfade abgewiesen; Crashpad-Diagnose bleibt ohne
Rollenautorität. Profil/Fixtures entfernt, Erfassungen beendet. 27 Auswertungstests erneut grün;
frühere 119 Helfer-/23 C4-Tests nicht als neue Läufe ausgeben. Keine neuen Programmbytes. **Nächster
begrenzter Scope:** Vertrag für vor dem Laden wirksame Modul-/Rollenbindung und sicheren Abbruch bei
nicht erzwingbarer Policy konkretisieren; keine Hersteller-/Signaturausnahmen oder Abschaltung von
Schutzsoftware. **C4b bleibt offen:** Polling verhindert keine frühere DLL-Ausführung; unabhängige
effektive Daten-ACL-Abnahme und kompatibler Produktstart fehlen. Native UI-ABI 0, keine echten
Projekte/Commands oder neue Produkt-/Recoveryfreigabe. Mittleres Prüfniveau und striktes
Datenschutzziel unverändert; Gesamt grob 65 %, nutzbarer 0.1-Umfang grob 80 %.

Vorheriger nativer Netzbeleg · C3, 17. November 2026:

[Gezieltes C3-ETW](../../testing/product-start-p2c-group-bootstrap-etw.md) des unveränderten Builds
`b24b3aa…` ist ergänzt: normaler Abschluss und Gruppenfehler/gesperrter Wiederanlauf. 28 Controller,
sieben Root-/fünf Kindrollen, 28 Konsolenhosts; alle 68 Enden explizit. Acht lokale Kontrollen,
Nullverluste/ Zuordnungslücken und null beobachtete Netzereignisse in diesen Bäumen einschließlich
Loopback. Alle 28 nativen Controller-Erzeugungszeiten sowie sechs Root-/fünf Kindbindungen exakt mit
Kernel abgeglichen; Root-Ende vor Kindende in allen fünf gebundenen Fällen. 66 MAC-Records/sieben
verbrauchte Launches/sechs Inventare unabhängig geprüft; drei Profile entfernt (zwei Broker-/ein
eigener Fixturecleanup), negativer Store weiter pending, echte Kontrollhashes unverändert. 42
Diagnose-Tests grün, kein neuer Build/Runtimeumbau oder Produktfreigabe. Native UI-ABI bleibt 0;
Gruppen-MAC ist keine unabhängige Renderer-/Schreiber-PID-Autorität. Der inzwischen ergänzte
C4-Vertrag steht oben; keine automatische Übertragung auf WebView. P2 insgesamt/P3–P5 und
Produkt-Q-01/Q-02 bleiben offen. Gesamt grob 65 % (60–70 %), nutzbarer 0.1-Funktionsumfang grob 80
%; keine Zeit-/Produktionsreifequote. Striktes Datenschutzziel unverändert.

Vorheriger Implementierungs-/Netznachweis:

[Frischer Workerbindungs-/Wiederanlauf-Netzbeleg](../../testing/product-start-worker-lifetime-etw.md).
Frisches Workerbindungs-v3-ETW, 17. November: unveränderter Build `37d7bec…`, normale Referenz und
vier Wiederanlaufpfade. 52 Controller/acht Worker/52 Controller-Konsolenhosts, alle 112 Enden
explizit; acht Kontrollen, Nullverluste/Zuordnungslücken, null beobachtete Netzereignisse in den
ausgewählten Bäumen. Alle acht nativen Erzeugungszeiten zum Kernel abgeglichen, sieben dauerhafte
Bindungen und drei Endrecords nach bestätigtem Kernelende zugeordnet. Exit 0 ersetzt keinen
fachlichen Abschluss: vier Finish-Records, drei reconciliierte Starts, ein ungebundener Start weiter
pending/gesperrt. Daten/MAC-/Paket-/Root-/Profilaudit und 42 Diagnose-Tests grün, echte Daten und
Schlüssel unverändert. Kein neuer Build/App-/Transport-/Installerumbau oder pnpm check; Rohdaten nur
lokal. Der damals nächste Vertragsumfang ist inzwischen als P2c-Entwurf dokumentiert; die Umsetzung
bleibt offen. Ein-Prozess-Beleg nicht auf WebView2 übertragen, private Fixtures nicht promovieren;
ungebundene Starts/unbestätigte Profile und P2 insgesamt/P3–P5/Produkt-Q-01/Q-02 offen, striktes
Ziel unverändert.

Vorheriger Stand:

[Dauerhafte Workerbindung und Wiederanlauf](../../testing/product-start-worker-lifetime.md).
Dauerhafte P2b-Workerbindung, 17. November: Execution v3/Bereinigungs-ABI 3, sechs Recordslots
reserviert. WorkerBound bindet Launch, native PID/Erzeugungszeit und geprüfte Ein-Prozess-Jobpolicy
vor Resume; enger nativer Lease-/Job-/Token-Wächter erlaubt nur diesen Record bei aktivem Worker,
andere Storemutationen behalten Jobleerstand. Wiederanlauf vor Drained benötigt diesen MAC-Beleg und
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

[Frischer Ressourcen-Reconciliation-Netzbeleg](../../testing/product-start-resource-reconciliation-etw.md).
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

[Begrenzte P2b-Ressourcen-Reconciliation](../../testing/product-start-resource-reconciliation.md).
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

[Frischer P2b-Netzbeleg](../../testing/product-start-p2b-bootstrap-etw.md), 17. November:
unveränderter Runtimehash, Referenz und gezielte Workerfälle, 31 Controller/sechs Worker/31
Console-Hosts mit 68 expliziten Enden. Acht Kontrollen, Nullverluste/Zuordnungslücken, null
beobachtete Netzereignisse in den ausgewählten Bäumen. Separater
Daten-/MAC-/Kernel-/Token-/Job-/Profilaudit bestätigt Ausführung und Cleanup. Keine
GUI-/Produkt-/externen Dienst- oder Inhaltsfreigabe, Ressourcencrashes weiterhin gesperrt. Nächster
begrenzter Schritt deren sichere Reconciliation; striktes Ziel unverändert.

Vorheriger Stand:

[P2b-Probeworker](../../testing/product-start-p2b-bootstrap.md), 17. November: separate synthetische
Profile/Readonly-Daten, gleiche Kontrolllease, MAC-gebundene Launch-/Ressourcen-/Abschlussrecords.
Eigener Einmalschlüssel, explizite drei Pipehandles, Token/exakter Job/private Dateisperren vor
Resume; monotone Receiptfrist und Drainabfrage am gehaltenen Job vor Profilentfernung. Kein
ACL-Grant auf private P2a-/Produktdaten, keine Importautorität/GUI. Drei gezielte native Fälle und
unabhängiger Audit grün; kein frisches ETW dieses neuen Builds. Crashressourcen bleiben gesperrt,
automatische Bereinigung/Bestandsübernahme und normale Produktintegration offen. Striktes Ziel
unverändert.

Vorheriger Stand:

17. November: [P2a Record-only](../../testing/product-start-p2a-records.md) persistiert Workspace-
    und Launch-Records mit Schema 5, nur frische synthetische private Wurzeln. Eigene
    `.kqprod`-Deklaration Record-ABI 1/UI-ABI 0, keine Startcapability oder Erweiterung des normalen
    Produkts. MAC-/Kopf- gebundene Rekonstruktion, dieselbe Storelease, Replay gesperrt,
    unvollständiger Tail ohne Reparatur. Vier neue Tests/67 aktive Helfertests/Clippy und drei
    native Fälle/Audit grün. App-/Paket-/Daten- Integration für P2b getrennt festlegen; kein
    ACL-Promovieren privater Fixtures. Neuer Broker ohne eigenen Netznachweis, keine fachliche
    Ausnahme/Produktfreigabe; striktes Ziel unverändert.

18. November:
    [P1 als reines testexklusives Produktstartmodell](../../testing/product-start-p1-model.md)
    umgesetzt. Neue versionierte Typen/Zustandsregeln, Parserfolg ist keine Autorität; keine
    Importtask, I/O-/MAC-/OS-/Transport-/UI-Integration. Sieben neue Tests, 63 aktive
    Helfertests/Clippy grün. Natives Schema 5/Paketdeklaration/Bootstrap/Bestandsübernahme weiter
    offen. Nächster begrenzter Scope P2a persistente authentisierte Bindung/Launch-Records nur in
    frischen synthetischen Wurzeln, ohne GUI-Start. Keine fachliche Ausnahme/Produktfreigabe;
    striktes Datenschutzziel unverändert.

19. November:
    [Produktstart-/Bestandsdatenvertrag v1 als Entwurf](../../data-format/product-start-integration-v1-draft.md)
    konkretisiert die nächste Integration, ohne sie als umgesetzt zu führen. Eigene Workspace-
    Startautorität statt Importtask, getrennte Hostkontrolle/Arbeitsablage, kein unisolierter
    Ersatzstart, explizite zerstörungsfreie Bestandskopie. Geplantes Kontrollschema
    5/Produktpaketdeklaration noch nicht unterstützt; Stufen P1–P5 und Abnahmegrenzen benannt. Keine
    Runtime-/Daten-/Policyänderung, keine fachliche Ausnahme oder Produktfreigabe; striktes
    Datenschutzziel unverändert.

[Prepare-Übergangsabbrüche](../../testing/native-prepare-transition-crashes.md), 17. November: drei
explizite, standardmäßig inaktive Punkte im begrenzten Forschungschild. Artefaktentfernung und
Zielanlage erzeugen noch keine Autorität; nach signiertem Prepare muss leeres Pending vor Retry
ausdrücklich abgebrochen werden. Drei native Fälle unter derselben Sperre erhalten Sitzung/Aufgabe,
56 aktive Helfertests/Clippy und Daten-/MAC-Audit grün. Inzwischen
[eigener frischer ETW-Beleg](../../testing/native-prepare-transition-etw.md): zwei gezielte Phasen,
59 direkte Aufrufe einschließlich fünf hashbenannter verwalteter Starts, acht Kontrollen, 182
explizite Enden, 0 Nicht-Loopback-/48 getrennte Loopback-Ereignisse. Keine Runtime-/Messguard-
Änderung, fachliche Ausnahme oder Produktfreigabe; striktes Ziel unverändert.

[Sichere Paketartefakte/Retry](../../testing/native-package-artifact-retry.md), 17. November:
authentisierter Abort autorisiert eng begrenzte Ersetzung vor erneuter Vorbereitung unter derselben
Sperre. Keine Löschung aktivierter/unbekannter Pakete, kein Link-Follow oder Sweep. Exklusive
Zielanlage vor Pending; laufende Kopierabsicht bleibt bei Fehlern ausdrücklich abzubrechen. Drei
native Fälle, 56 aktive Helfertests/Clippy und Daten-/MAC-Audit grün. Inzwischen
[eigener frischer ETW-Beleg](../../testing/native-package-artifact-retry-etw.md) mit Retry und
verwaltetem Kandidaten: acht Kontrollen, 144 explizite Enden, 0 Nicht-Loopback-/48 getrennte
Loopback-Ereignisse; 21 direkte Aufrufe unabhängig abgeglichen. Keine Runtime-/Messguardänderung,
fachliche Ausnahme oder Produktfreigabe; striktes Ziel unverändert.

Gezielte
[Update-Abbruch-/Fehlerreconciliation](../../testing/native-update-abort-reconciliation.md), 17.
November: drei native Fälle sowie zwei zusätzliche Helfertests sichern den vorhandenen Pfad unter
derselben Kontrollsperre ab. Abbruch erhält Actor und Aufgabenbindung; keine Kettenreparatur,
Paketartefaktlöschung oder automatische neue Importautorität. Keine Runtimeänderung/neuer
Netznachweis dieser zusätzlichen Fehlerpfade; striktes Ziel und offene Produktabnahme unverändert.

Schema-4-Brokerpaket-Update, 17. November:
[Implementierung/Beleg](../../testing/native-schema4-package-update.md) erlaubt lokale Updates unter
derselben Kontrollsperre mit statischem begrenztem Paketvertrag und vollständiger Hashbindung. Kein
Kandidatencode vor Aktivierung; keine Apppaket-/Datenmigration. Ein synthetischer nativer
Actorwechsel/Recovery/Cleanup erhält Bindung und Daten. Mittleres Prüfniveau;
[eigener frischer ETW-Beleg](../../testing/native-schema4-package-update-etw.md) beider Programme
einschließlich fünf hashbenannter verwalteter Starts: acht Kontrollen, Nullverluste, 132 explizite
Enden, 0 Nicht-Loopback-/48 getrennte Loopback-Ereignisse. Exakte PID-/Image-Zuordnung nur für
native Kontrollaufrufe, Daten-/MAC-/Kernel-Gegencheck; keine Änderung der Runtime-/GUI-Guards. Keine
fachliche Ausnahme, striktes Datenschutzziel unverändert.

Gezieltes Cleanup-ETW, 17. November:
[Eigener frischer Beleg](../../testing/native-profile-cleanup-etw.md) des unveränderten Paares. Acht
Kontrollen, keine Verluste/Zuordnungslücken, 140 explizite Enden, 0 Nicht-Loopback-/48 getrennte
Loopback-Ereignisse. Für direkte GUI-lose Kontrollaufrufe benannter Modus `native-control` mit
unveränderten allgemeinen Guards und unabhängigem Aufruf-/GUI-Abgleich. Keine Runtimeänderung,
fachliche Ausnahme oder Produktfreigabe; mittleres Prüfniveau und striktes Ziel unverändert.

Profilbereinigung, 17. November: [Native Vorschau](../../testing/native-profile-cleanup.md) ergänzt
signierte Vorbereitung in Kontrollschema 4 vor Profilerzeugung sowie idempotenten Cleanup nach
signierter Stilllegung. Identität aus Kontrollkette, dieselbe Sperre, keine neue Importautorität.
Zwei native Fälle und kurzer Start/Recovery mit Datenbestandserhalt geprüft. Bewusst mittleres
Prüfniveau; kein neuer ETW-Beleg des geänderten Brokers. Produktintegration/weitere Crashpunkte
bleiben offen, striktes Datenschutzziel unverändert.

Übergangs-Crashpunkte und frisches ETW, 17. November:
[Eigener Beleg](../../testing/native-session-transition-crash-etw.md) für 20 zusätzliche
Sitzungsübergänge: neun gesperrte/elf authentisierbare Zustände, vier explizite Recoveries. Keine
Kontrollreparatur oder neue Importautorität. Zwei gültige frische ETW-Phasen mit acht Kontrollen,
Nullverlusten, 458 expliziten Enden, 0 Nicht-Loopback-/138 getrennten Loopback-Ereignissen. Nur
dokumentierter Build/Lebenszeiten, keine Produkt- oder vollständige externe Dienstfreigabe.
Profil-Cleanup-Reconciliation/Schema-3-Updates/Produktintegration offen, striktes Ziel unverändert.

Dauerhafte Sitzungsbindung/Recovery, 17. November:
[Nachweis](../../testing/native-product-session-recovery.md) bindet Apppaket und persistierte
synthetische Sitzung in Kontrollschema 3. Authentisierte neue Startrecords vor Recovery unter
derselben Kontrollsperre und demselben Produktjob; vorhandenes Journal und signierte Appbytes
zwingend. Zwei echte Controllerabbrüche, vier Wiederanläufe, keine Replay-Kopie. Keine echte
Datenmigration, Schema-3-Updatepolitik oder normale Produktintegration. Neuer Broker benötigt
frisches ETW; keine Änderung am strikten Datenschutzziel.

Kontrollgebundener synthetischer Start, 17. November:
[Implementierung und Beleg](../../testing/native-product-starter.md) verbinden erstmals
Startlebenszeit, private Kontrollsperre und den von Updates geprüften Produktjob. Zwei isolierte
Starts mit zwölf tatsächlichen Dateiverweigerungen und unabhängigem MAC-/Datenaudit bestanden.
Appdaten bleiben frische getrennte TEMP-Sitzungen; keine normale Produktintegration, dauerhafte
Apppaket-/Sitzungsbindung oder echte Migration. Kein frischer ETW-Beleg des neuen Brokerhashs;
striktes Datenschutzziel unverändert.

Kontroll-Crash-/Netzbeleg, 16. November:
[Eigener frischer Nachweis](../../testing/native-product-control-crash-etw.md) für denselben neuen
gesicherten Prüfstand. 28 Prozessabbrüche, 17 gesperrte/elf nutzbare Zustände, acht explizite
Weiterführungen; keine automatische Reparatur oder Schlüsselneuanlage. Die echte Vorschauablage
wurde nicht verwendet. Zwei gültige ETW-Phasen, acht Kontrollen, Nullverluste, 1.136 explizite
Enden, 0 Nicht-Loopback-/444 getrennte Loopback-Ereignisse. Normale Produktintegration, echte
Datenmigration, externe Dienstgrenzen und vollständiges Checkpoint-/Stromausfallverhalten bleiben
offen; striktes Datenschutzziel unverändert.

Kontrollablage-Vorschau, 16. November:
[Implementierung/Beleg](../../testing/native-product-control.md) und
[Vertrag](../../data-format/native-product-control-v1.md) ergänzen dauerhafte private Ablage, lokal
authentisierte Programmwechsel und explizite Kontrollschema-Migration. Root außerhalb des
Coding-Werkzeug-Paketcache, ACL-Prüfung vor Schlüsselerzeugung, MAC-Kette/Kopf, exklusive Sperre und
Jobleerstand. Drei native Programmwechsel und 18 echte AppContainer-Dateiverweigerungen bestanden.
Keine automatische Übernahme alter Wurzeln/Journale, keine normale Produktintegration oder
Herausgeber-/Netzabnahme; striktes Ziel unverändert. Nächster Scope: Kontroll-Crash-Serie und
eigenes frisches ETW, danach isolierter Produktstarter/echte Datenmigration.

Fensterende-ETW, 16. November:
[Eigener frischer Beleg](../../testing/native-combined-window-close-etw.md) des unveränderten
gesicherten Paars mit Automatik und beiden nativen Fensterendepfaden inklusive Recovery. Acht
Kontrollen, Nullverluste, alle 942 Enden explizit, 0 beobachtete Nicht-Loopback-/536 getrennte
Loopback-Ereignisse. Unabhängiger MAC-/Daten-/Lebenszeitaudit bestanden. Keine neue
Runtime-/Domänenausnahme, kein Produkt- oder externer Dienstnachweis. Nächster Scope: geschützte
Produktablage und authentisierte Updates/Migration; weitere F08-/Dienstgrenzen offen.

Kombinierter Hauptfensterschluss, 16. November:
[Eigener Funktionsbeleg](../../testing/native-combined-window-close.md) für offenen Dialog und
Teiltransfer, danach authentisierte Recovery und neue Aufträge bestanden. App unverändert; neue
native Helfermodi mit konsistenter manueller Modusprüfung, regulärem Ende, Threadjoin und
Jobleerstand. Verworfener Erstlauf bleibt unbestanden. Eigener frischer ETW-Beleg inzwischen oben
verlinkt; keine Übertragung früherer Nachweise oder Produkt-/Dienstfreigabe. Striktes
Datenschutzziel unverändert, Produktablage/Updates/Migration und weitere F08-Randfälle offen.

Tastatur und ETW des manuellen Brokerpaars, 16. November: Eigener frischer
[Nachweis](../../testing/native-combined-assistive-restart-etw.md) mit automatischer Serie und vier
manuellen Wiederanläufen. Acht Kontrollen, Nullverluste, 906 explizite Enden, 0 Nicht-Loopback-/514
getrennte Loopback-Ereignisse. Menschliche Tastatur-/Hörbestätigung und native Replay-/Fokusbelege
getrennt von physischer Eingabezuordnung behandeln. Kein Runtimeumbau, keine neue Produktautorität
oder externe Dienstabnahme. Weitere Fensterendeprüfung/Produktablage/Updates/Migration offen;
striktes Ziel unverändert.

Assistiver kombinierter Wiederanlauf, 16. November: Vier Zustände mit Windows-Sprachausgabe, Fokus
und nativem Replay-Gegenbeleg bestätigt. Zwei Erstversuche ohne nativen Aufruf wurden korrekt
abgelehnt; reine Tastaturprüfung war damals noch offen. Neue manuelle Helferaktion, App unverändert;
[Nachweis](../../testing/native-combined-assistive-restart.md). Neuer Helferhash benötigt einen
eigenen frischen ETW-Beleg (inzwischen oben verlinkt); keine Übertragung des folgenden älteren
Nachweises. Striktes Ziel unverändert, Produktintegration und weitere Fensterendeprüfung offen.

Kombiniertes ETW, 16. November: Eigener gültiger Nachweis des gesicherten v3-Paars mit automatischer
Serie und echtem nativem Dateidialog in getrennten kalibrierten Phasen. 80 Apps, 480 WebViews und
alle 868 Prozessenden explizit; acht Kontrollen, Nullverluste, 0 Nicht-Loopback- und 480 getrennte
Loopback-Ereignisse. Native Abläufe/Daten/MACs unabhängig geprüft.
[Nachweis](../../testing/native-combined-live-restart-etw.md). Kein Runtimeumbau, keine Übertragung
auf geänderte Binaries, externe Dienste oder das Produkt. Striktes Ziel ohne
Runtime-/Domänenausnahme unverändert; assistive Prüfung damals offen, weitere Fensterendeprüfung
weiterhin offen.

Fortschreibung kombiniert, 16. November: Startticket/Brokervertrag v3 binden Live-Pipes an den
initialen verbrauchten Auftrag. Recovery hat keine Auswahlhandles. Erst privater signierter
Abschluss nach Reconciliation/Replay und ausdrückliche neue native Aktion erlauben den nächsten
unveränderlichen Grant mit neuer ID/Vorgängerhash. Keine Rendererparameter, Netzfähigkeit,
Richtlinien-/DPAPI-Ausnahme oder Produktintegration.
[Vertrag](../../data-format/native-broker-recovery-v3.md),
[native Serie und manuelle Auswahl](../../testing/native-combined-live-restart.md) bestanden.
Shell-Aufruf nutzt nach nativ reproduziertem Fehler eine gleichwertige lokale Pfaddarstellung mit
identischem kanonischem Ziel; Scopeprüfung bleibt. Eigener ETW-Beleg inzwischen oben vorhanden,
neuer Hörnachweis noch offen; striktes Ziel unverändert.

HMAC-ETW 16. November: Neues gesichertes Paar mit eigenem gültigem Netzbeleg des gesamten
Serienbaums, einschließlich 78 negativer Brokerstarts. 103 Broker/30 Apps/180 WebViews, alle 446
Prozessenden explizit, vier Kontrollen, Nullverluste, 0 Nicht-Loopback- und 180 getrennte
Loopback-Ereignisse. Native Abläufe und Daten/MACs unabhängig geprüft.
[Nachweis](../../testing/native-hmac-broker-etw.md). Keine Übertragung auf externe Dienste oder
geänderte Programme, keine Produktfreigabe. DPAPI-Primitive entfernt statt Domänenausnahme; striktes
Ziel unverändert.

Fortschreibung HMAC, 16. November: Die DPAPI-Primitive ist im aktuellen Forschungsbroker entfernt.
Lokaler Schlüssel aus dem fest betrachteten `ProcessPrng`-Backend und RustCrypto-HMAC-SHA-256
ersetzen die kontogeschützte Authority;
[Vertrag v2](../../data-format/native-broker-recovery-v2.md). Die private DACL übernimmt die
Schlüsselverwahrung; keine Verschlüsselung bei ruhendem Datenträger oder zusätzliche
Kontogeheimnis-/Hardwarebindung behaupten. Fehlende/falsche Schlüssel sperren; keine Migration,
Domänen-/OS-Ausnahme oder Produktfreigabe. Neue native Serie mit 78 Ablehnungen bestanden;
[Nachweis](../../testing/native-hmac-broker.md). Eigener ETW-Beleg oben separat dokumentiert. Die
folgenden DPAPI-Absätze bezeichnen den historischen verworfenen Zwischenstand.

Produktintegration des DPAPI-basierten Forschungsbrokers bleibt zusätzlich gesperrt, solange
mögliche Domänencontroller-/RPC-Pfade nicht unter dem strikten Ziel belegt oder die Primitive
ersetzt sind. Keine OS-/Domänenrichtlinienänderung und keine stillschweigende Metadatenausnahme;
[Vertrag und Primärquelle](../../data-format/native-broker-recovery-v1.md).

Controller-Fortschreibung 16. November: Expliziter Forschungsbroker darf nach Controllerverlust
bestehende synthetische Ablage ausschließlich nach DPAPI-/Programm-/Wurzelprüfung, exklusiver Sperre
und Jobleerstand übernehmen. Kein neuer Importgrant. Geschützte native Kontrollablage und
Jobzuweisung bei Prozesserzeugung; Containerzugriffe unter dem tatsächlichen Token verweigert.
[Vertrag](../../data-format/native-broker-recovery-v1.md),
[native Serie](../../testing/native-controller-crash.md): sechs Controller-Kills, zwölf neue
Wiederanlaufprozesse, 30 App-Generationen, 60 Ablehnungen und unabhängige Datenprüfung bestanden.
Kein Produktbroker, kein neuer ETW-Beleg; Produktintegration und externe Grenzen weiter offen.
Striktes Ziel unverändert.

ETW v2 am 16. November: Neuer gültiger Nachweis exakt des unveränderten gesicherten v2-Paars,
einschließlich beider manueller Wiederanlaufpfade und automatischer Serie. 32 App-Generationen, 168
WebViews, vier Kontrollen, Nullverluste, 0 Nicht-Loopback-Ereignisse; 176 Loopback-Ereignisse
separat. Native Abläufe, Daten und Cleanup unabhängig gegengeprüft.
[Nachweis und Grenzen](../../testing/native-assistive-restart-etw.md). Keine Runtimeänderung oder
Erweiterung auf externe Dienste; Controller-Crash/dauerhafter Broker und Produktabnahme offen.

Assistive Fortschreibung 16. November: Startticket v2 ergänzt das verpflichtende native Flag
`automatic`. Nur für Wiederherstellung darf es false sein; dann keine Skriptbedienung und höchstens
600 Sekunden manuelle Betrachtung ohne Importgrant. Gleiche SID-/Hash-/Launch-/Pipe- Bindungen, null
Capabilities. [Vertrag](../../data-format/native-ui-restart-v2.md),
[Hörprüfung und Grenzen](../../testing/native-assistive-restart.md). v1-Tickets werden abgewiesen;
der alte ETW-Beleg wird nicht auf neue Binaries übertragen. Striktes Ziel bleibt unverändert.

ETW-Fortschreibung 16. November: Frischer gültiger Nachweis exakt des unveränderten gesicherten
Neustart-Paars. 28 App-Generationen mit vollständiger Endezuordnung, 144 WebViews, vier Kontrollen,
Nullverluste und 0 Ereignisse außerhalb Loopback; 144 Loopback-Ereignisse separat. Native Serie und
Daten/Cleanup im Messlauf unabhängig geprüft. [Nachweis](../../testing/native-ui-restart-etw.md),
[Hashbindung und Aggregate](../../testing/native-ui-restart-etw.json). Keine Erweiterung der
Vertrauensgrenze auf externe COM-/Shell-/DNS-/Broker oder kompromittierte Prozesse. Striktes Ziel,
fehlende Produktabnahme und offene Controller-Crash-/Dauerbrokerintegration bleiben unverändert.

Neustart-Fortschreibung 16. November: Gebundene Starttickets über frische anonyme Pipes ermöglichen
den Neustart derselben Forschungs-GUI mit unveränderter Identität, Ablage und Programmhash innerhalb
eines lebenden Controllers. Suspendierte Root-Tokenprüfung und empfangsseitige SID-/Hash-/Launch-
Prüfung vor WebView-Aufbau; keine neuen Capabilities oder Rendererbefehle. Finale Serie mit 28
Generationen, sechs Crashpunkten, zweifachem Wiederanlauf, Altimportabwehr und vier negativen Starts
bestanden. [Vertrag](../../data-format/native-ui-restart-v1.md),
[Nachweis und Buildbindung](../../testing/native-ui-restart.md). Controllerabsturz/dauerhafter
Produktbroker und neuer ETW-Beleg dieses Paars bleiben offen; striktes Ziel unverändert.

Auftrags-Fortschreibung 16. November: Das Forschungsfeature bindet einen privaten, an die lokale
Projektwurzel gebundenen SQLite-Auftragsstore vor Auswahl/Lesen ein. Dauerhafter Verbrauch,
exklusive Sperre, eigene Zwischenverzeichnisse und persistierter Commit-Receipt erlauben
konservative Wiederanlaufzuordnung ohne erneutes Lesen oder automatische Veröffentlichung. Keine
neuen Capabilities, Hostordnerfreigaben, Rendererparameter oder Produktabhängigkeiten.
[Vertrag und Bedrohungsgrenzen](../../data-format/native-task-journal-v1.md),
[Windows-Prozess-Kills und native Rückprüfung](../../testing/native-task-reconciliation.md).
Integrierter isolierter GUI-Neustart und neuer ETW-Nachweis dieses Builds fehlen; striktes Ziel und
offene Produktabnahme unverändert.

Assistive Fortschreibung 16. November: Erfolg-/Fehlerpfade der Forschungsseite mit Replay und Fokus
nativ bestätigt. Eine auf feste Enum-/Bool-Werte begrenzte lokale Diagnose ergänzt die
Forschungsbefehle ohne neue Dateifreigaben, Netzwerkrechte oder Produktintegration.
[Nachweis, Grenzen und Buildbindung](../../testing/native-live-assistive-focus.md). Frühere
Fokusabweichungen gelten nicht als bestanden. Kein neuer ETW-Beleg dieses geänderten App-Builds;
striktes Datenschutzziel, offene Brokergrenzen und fehlende Produktabnahme bleiben unverändert.

Fortschreibung 16. November: Der getrennte Live-Auswahlprototyp beendet bei Fensterende auch offenen
Dateidialog und nachgewiesenen Teiltransfer. Schließnachrichten sind auf eigenen Prozess, eigenen
Auswahlthread und Dialogklasse beschränkt; Pipe-Duplikate werden geschlossen und der Threadabschluss
wird begrenzt abgewartet. [Nachweise und Buildgrenzen](../../testing/native-live-window-close.md).
Keine Produktintegration oder neue Netzwerkfähigkeit. Statt Übertragung des vorherigen Netzbelegs
besteht nun ein
[frischer ETW-Vergleich dieses unveränderten Builds](../../testing/native-live-window-close-etw.json):
drei vollständige Bäume, vier Kontrollen, Nullverluste, 0 Ereignisse außerhalb Loopback. Striktes
Ziel unverändert; Produkt-Q-01/Q-02 und externe Brokergrenzen bleiben offen.

Status: **Untersuchungsrichtung A ausdrücklich beauftragt am 15. November 2026.** Nutzerauftrag:
technische Laufzeitisolation, „Das strikte Datenschutzziel bleibt bestehen.“ KQ-069 / Issue #134.
Keine Produktarchitektur- oder Netzfreigabe und keine Änderung des Pflichtenhefts. B gehört nicht
zum freigegebenen Scope.

Der [Machbarkeitsversuch](../../testing/native-runtime-isolation.md) ergänzt einen getrennten
AppContainer-Forschungslauncher und gültige ETW-Vergleiche. Lokal schreiben, neu starten und
HTML/PDF exportieren funktioniert; der Dateidialog scheitert. Die 0 beobachteten Ereignisse
außerhalb Loopback im Prototyp ändern die fehlende Isolation des normalen Starts nicht.

Der anschließende [Datei-/Shell-Transportversuch](../../testing/native-runtime-file-exchange.md)
belegt native Auswahl im separaten Hostcontroller und begrenzten Bytetransport über zwei explizite
Handles in einen Worker ohne Capabilities. Der Hostordner erhält keine Paket-ACL; nicht ausgewählte
Dateien bleiben über ihren Pfad unlesbar. Kein beliebiger Shell-Aufruf oder Produktbroker
freigegeben. Der gültige ETW-Ausschnitt erfasst keine Netzereignisse im Controller-/Workerbaum;
außerhalb laufende COM-/Shell-Dienste sind damit nicht abgenommen. Der spätere finale Cleanup-Stand
ist separat nativ geprüft. Nächster Scope ist ein typisierter einmaliger Dateiauftrag an einen
isolierten App-Testadapter mit konkretem atomarem Import; die strikte Zielsetzung bleibt bestehen.

## Belegter Konflikt

Die [separate Tauri-Testanbindung](../../testing/native-tauri-file-channel.md) ergänzt ein echtes
Forschungsfenster mit parameterlosen Commands, strikter Origin-/Fensterprüfung und vorab delegiertem
Einwegkanal. Kernelhandles statt Rendererpfaden oder öffentlichem RPC-Endpunkt; Flags vor
WebView-Start gelöscht/geprüft. Keine neue Hostordner- oder Netzfähigkeit. Drei finale native
Sitzungen prüfen Import/Wiederholung, Abbruch und Fensterende am Haltepunkt; SQLite und Cleanup
unabhängig bestätigt. Gültiges ETW desselben Builds: vollständige Bäume, vier Kontrollen,
Nullverluste, 0 Ereignisse außerhalb Loopback. Keine universelle Netzstille oder vollständige
Nachfahren-/Brokerattestierung. Vorabauswahl, Auswahl aus laufender App, dauerhafte Aufträge/
Crash-Reconciliation und Produktintegration offen. Striktes Ziel unverändert, keine Ausnahme.

Der [native App-Dienst-Testadapter](../../testing/native-isolated-app-import.md) bindet einen
einmaligen Snapshotauftrag an den isolierten Worker. Parser und atomare Veröffentlichung nutzen die
vorhandenen Rust-Dienste; Ablauf/Abbruch zusätzlich nach vollständigem Aufbau geprüft. ETW des
finalen Builds: gültige Kontrollen, keine Netzereignisse in acht Controller-/Workerbäumen. **Keine
Tauri-/WebView-/GUI-Anbindung und keine reale Nutzdateiauswahl.** UUID/Ablaufzeit ersetzen keinen
produktiven authentisierten Auftragskanal. Nach Commitbeginn keine Rücknahmegarantie;
Crash-Reconciliation, externe COM-/Shell-Dienste und Produktintegration offen. Keine neue
Netzfähigkeit oder fachliche Ausnahme.

Pflichtenheft Revision 1 verlangt Offlinefähigkeit, keine Telemetrie ab Werk und keine Übertragung
von Manuskripttexten, Bildern oder Metadaten bei Cloud-Opt-out (§1, §5, §10). KQ-068 misst
öffentliche WebView2-Verbindungen. KQ-069 reproduziert Konfigurations- und Komponentenupdate-Starts
bereits ohne Projekte. Die unterstützten betrachteten APIs ergeben keinen belegten universellen
Ausschalter. Siehe [Versuch, Quellen und Grenzen](../../testing/native-m3-webview-network.md).

Eine funktionierende Offline-App und fehlender eigener Telemetriecode erfüllen noch keine
vollständige Windows-Laufzeit-Netzstille. Wir nehmen weder die Unbedenklichkeit aller Bodies noch
eine garantierte Übertragung von Projektdaten an. Q-01/Q-02 bleiben nicht bestanden.

## Beauftragte Richtung und nicht freigegebene Alternative

**A – Striktes Ziel beibehalten; technische Isolation untersuchen (beauftragt).** Nächster
begrenzter Scope: unterstützte, durch die Anwendung durchsetzbare Windows-Isolation für Rust und
WebView2 einschließlich Prozessnachfahren prüfen. Zunächst Machbarkeit und reproduzierbarer lokaler
Prototyp, kein ungeprüfter Umbau des produktiven Launchers. Abnahme mit unabhängiger
ETW-Prozesszuordnung und positiven Kontrollen, vollständigen Prozessenden, CRUD, Export/Neuimport
und Neustart. Lokale Ressourcen, Tauri-IPC, Barrierearmut, Installation und sicherer
Runtime-Updatepfad dürfen nicht stillschweigend ausfallen. Falls diese Plattform das Ziel nicht
trägt, bleibt auch ein Plattformwechsel als gesonderter größerer Entscheid offen.

**B – Fachliche Laufzeitgrenze ausdrücklich ändern.** Eine mögliche Ausnahme würde ausschließlich
konkret belegte Runtime-Konfigurations- und Komponentenmetadaten betreffen. Vor Annahme müssen
erlaubte Daten, Zielklassen, Nutzerinformation und überprüfbare Grenzen präzise definiert und
Pflichtenheft, Abnahmematrix und Datenschutzaussagen gemeinsam geändert werden. Die aktuelle Messung
beweist keine zulässigen Body-Inhalte und reicht daher allein nicht zur Freigabe dieser Ausnahme.
Keine pauschale Ausnahme für Manuskripte, Bilder, Prompt-Kontext oder beliebige Telemetrie. Das wäre
eine Produktentscheidung, kein technischer Bugfix; sie ist bisher nicht autorisiert.

## Leitplanken der Untersuchung

Keine neuen Abschaltflags, Firewall-/OS-Datenschutzänderungen, TLS-/Sandbox-Abschaltung oder
automatische Providerfreigabe. Unterstütztes Custom-Crash-Reporting ist ein separater, begrenzter
Mechanismus und löst die beobachteten Routen nicht. Diagnoselogs gehören nicht in Releases. Issue
#134 bleibt offen. A ist ein Forschungs- und Integrationsauftrag, keine Aussage, dass bereits eine
vollständige tragfähige Produktisolation existiert. Datei-/Shell-Broker, kontrollierter Austausch,
Git-/Credential-/Providerpfade, Prozess-/Profil-Lebensdauer und Installer-/Updatepfade müssen die
strikte Grenze erhalten; zusätzliche Internetfähigkeiten für den WebView2-Baum sind kein Fix.
