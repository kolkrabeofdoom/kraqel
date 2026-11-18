# Rechnerwechsel und Arbeitsübergabe

Diese Anleitung trennt den öffentlichen Quellcode vom privaten Schreibprojekt. GitHub transportiert
Code und Projektdokumentation; ein `.kraqel`-Archiv transportiert Manuskript, Struktur und lokale
Medien. Wer beides verwechselt, erhält entweder fehlende Kapitel oder einen sehr literarischen
Commitverlauf.

## Aktuelle Entwicklungsübergabe · 18. November 2026

### Aktueller Einstieg und gesicherter Stand

Die Bewertung der unabhängigen BP3-Durchsetzung ist abgeschlossen. Für eigene synthetische Rollen
ist erzwungene UMCI in einem separaten Offline-Gast mit vertrauenswürdigem Erzeuger als
Machbarkeitsprobe ausgewählt. Für alle WebView2-Rollen fehlt weiterhin der belegte Erzeugungsweg vor
Ausführung. C4 bleibt gesperrt. Die Gastprobe und ein eigener Netzbeleg stehen aus.
[ADR 0035](architecture/decisions/0035-bp3-independent-process-and-image-enforcement.md).

Vorheriger Stand · BP3 synthetische Debug-Wächter-Gegenprobe, 18. November 2026:

[Debug-Wächter-Nachweis](testing/product-start-p2c-bp3-debug-guardian.md): Root beim Create-Ereignis
vor User-Mode-Ausführung abgewiesen, normale Kindvererbung bestätigt. Ein Kind mit eigenem
DEBUG_PROCESS läuft dagegen außerhalb der äußeren Debugkette; dieselbe Kindgeneration vorher vom
Host an gehaltenem Handle, Token, Profil und Job geprüft. Kein AppContainer-/Jobausbruch. Drei
native Fälle, neun Jobprozesse (fünf Worker/vier Konsolenhosts), sieben äußere Create-/Exit-Paare
und zwei innere Create-Berichte; alle eigenen Jobs leer, Profile/Fixtures entfernt. **Debugvererbung
allein ungeeignet für BP3.** Drei neue aktive Tests/141 Helfer-/Transporttests insgesamt und Clippy
grün, sieben standardmäßig ignoriert. Kein frisches ETW oder tatsächlicher Wächter-Crash. C4-Sperre
und native UI-ABI 0 unverändert; keine Runtime-/Datenautorität. BP3/BP4/C4c/d weiter offen. Nächster
begrenzter Scope: Architekturentscheid für unabhängig durchgesetzte Erzeugung samt früher
Wächter-/Ladebindung, keine weiteren Runtime-Versuche auf Basis bloßer Debugvererbung. Mittleres
Prüfniveau und striktes Datenschutzziel unverändert; Gesamt grob 65 %, nutzbarer 0.1-Umfang grob 80
%.

Arbeitsbranch `feat/kq-069-live-file-selection`, Draft-PR #142 unverändert. Ausgangshead `fa894f7`
bei Beginn sauber, CI Windows/Linux/Dependency Review vollständig grün. Neue Head-CI separat prüfen;
kein Merge/Release.

Exaktes neues Paar unter `artifacts/bp3-debug-20261118-01/` geschützt außerhalb von TEMP gesichert:
normal `56a1b43…`, Testcontroller `e154373…`, privater `run.json`, unabhängiger Audit und Manifest
mit Quellhashes. Originalausgabe `%TEMP%/kraqel-bp3-debug-build-20261118-01/`. Roh-PIDs, Zeiten,
Nonces und Pfade bleiben lokal; keine MAC-/Kontrollschlüssel oder echte Projekte in der Probe. Keine
fremden System-/Profilartefakte entfernt. Fünf eigene Entwicklungsreste aus fehlgeschlagenen
Vorversuchen nach Ende aller Testprozesse im exakt geprüften TEMP-Bereich entfernt; keine
Produktbereinigung. Keine eigenen Prozesse/Profile/Fixtures mehr aktiv, kein neues ETW oder UAC
nötig.

Eine neue Sitzung darf aus der gelungenen normalen Debugvererbung keine universelle Startsperre
ableiten: Die Gegenprobe widerlegt das gerade. Der Kindmarker wurde mit eigener Debugkette im selben
AppContainer/Job ausgeführt. Kein DLL-/TLS-/JIT- oder Wächter-Crashnachweis daraus. Zuerst
unabhängigen Erzeugungs-/Lade-/Wächteranker bewerten, ohne Hersteller-/Signaturausnahme,
Schutzsoftwareänderung oder automatische UI-/VM-/Treiberumstellung. Normale App und C4-Sperre
unverändert. Geänderte Bytes haben weiterhin keinen frischen Netzbeleg.

### Vorherige Übergabe: BP3-Startsperre (historisch)

Vorheriger Stand · BP3-Machbarkeitsgrenze und native Startsperre, 18. November 2026:

[BP3-Nachweis](testing/product-start-p2c-bp3-admission.md): Für den aktuellen C4-Kandidaten ist noch
kein vollständiger präventiver Modul-/WebView-Rollenmechanismus nachgewiesen. Der native Controller
sperrt jetzt vor ACL-/Ressourcenanlage, Commit und CreateProcess; Worker und Runtime-Einstieg
sperren zusätzlich vor Handleübernahme bzw. COM/WebView. Keine konfigurierbare Ausnahme oder
Umdeutung von Paketpins, BP1/BP2 oder Diagnosen. Zwei aktive Regressionen, 138 aktive
Helfer-/Transporttests und Clippy grün; sechs standardmäßig ignoriert. Drei direkte Workeraufrufe
enden mit 2 ohne Ausgabe/Dateianlage. Kein neues ETW; frühere Messungen bleiben bytegebunden. **BP3
insgesamt offen**, keine Runtime-/Datenfreigabe, BP4/C4c/d weiter gesperrt. Nächster begrenzter
Scope: separate synthetische Debug-Wächter-Probe mit ausdrücklicher Debugketten-Gegenprobe; noch
nicht implementiert. Keine System-/Schutzsoftware-/Signaturausnahme. Mittleres Prüfniveau und
striktes Datenschutzziel unverändert; Gesamt grob 65 %, nutzbarer 0.1-Umfang grob 80 %.

Arbeitsbranch unverändert `feat/kq-069-live-file-selection`, Draft-PR #142. Ausgangscommit dieser
Sitzung `3772c0c`; dessen Windows-/Linux-CI beim letzten Abgleich noch laufend, Dependency Review
grün. Neue Head-CI gesondert prüfen, kein Merge/Release. Checkout zu Beginn sauber.

Neuer gesicherter, noch nicht ETW-gemessener Helfer unter `artifacts/bp3-admission-20261118-01/`:
normal `04e38c2…`, Testcontroller `503fdbc…`, Manifest mit Quellhashes und drei direkten
Abweisungen. Keine echte Kontrollablage oder alte Auditkopie übernehmen. Bei Rechnerwechsel
Programme lokal neu bauen/prüfen; historische Netzbelege nicht auf neue Bytes übertragen. Die
C4-Beobachtungsserie ist im neuen Build ausdrücklich nicht mehr aufrufbar, auch nicht mit einer
alten Bootstrapnachricht. Der BP2-Blattzweck bleibt separat.

Die Machbarkeitsprobe des Debug-Wächters muss zuerst die dokumentierte Unterbrechung der
Debugvererbung durch einen eigenen DEBUG_PROCESS-Nachfahren untersuchen. Bei fehlendem Nachweis
weiter sperren; keinen Runtime-/Datenstart, keine Signatur-/Herstellerausnahme oder AV-Änderung.

### Vorherige BP2-Übergabe (historisch)

Arbeitsbranch **`feat/kq-069-live-file-selection`**,
[Draft-PR #142](https://github.com/kolkrabeofdoom/kraqel/pull/142). `git pull --ff-only` zu Beginn
dieser Sitzung: aktuell/sauber. CI des vorherigen Übergabeheads `b881d59` vollständig grün
(Windows/Linux/Dependency Review); neue Head-CI gesondert prüfen. Kein Merge, Release oder
Produktstart daraus.

Vorheriger Stand · BP2 nativer synthetischer Blattversuch samt ETW, 18. November 2026:

[BP2-Nachweis](testing/product-start-p2c-bp2-native-leaf.md): unabhängige native Supportprüfung,
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

Nächster begrenzter Scope **BP3**: unterstützten präventiven Wächter-/Lade-/WebView-
Mehrprozess-Erzeugungsweg und Machbarkeit konkretisieren. NoRemote/NoLowLabel sind keine exakte
Modulpositivliste; ChildPolicy Restricted ist kein Rollenfilter und darf nicht einfach abgeschaltet
werden. Vollständige Wächter-Startkette vor produktiven Schlüsseln, dynamische Rollen und spätere
Nachladungen bleiben offen. Keine Hersteller-/Signaturausnahme, Schutzsoftwareänderung oder neue
Text-/Commandfreigabe. BP4 und C4c/d erst nach ihren jeweiligen nativen Gates.

Aktuelles exaktes Paar `%TEMP%/kraqel-bp2-build-20261118-01/`: normaler Helfer/Blattcode `5205d72…`,
tatsächlicher Host-Testcontroller `62d7a73…`. `run-functional.json` und gemessener `run.json`,
getrennte private MAC-Audits, unabhängiger Audit und Manifest. Rohdaten unter
`%TEMP%/kraqel-bp2-etw-20261118-01-{reference,gate}/`, geschützte zusätzliche Sicherung
`artifacts/bp2-native-20261118-01/` außerhalb von TEMP. Keys/PIDs/SIDs/Pfade bleiben lokal; keine
Auditkopie als aktiven Store oder wiederverwendbaren Grant verwenden.

BP2 ist ein eigener Forschungszweck, neue native `bp2LeafProbe`-Action/Eventversion 1 im
Schema-6-Forschungsreader. Keine bestehende echte Kontrollablage adoptieren/migrieren; alte Reader
weisen unbekannte Action ab. UI-ABI weiter 0. Eigener kurzlebiger Pipe-Schlüssel; keine permanente
Kontrollautorität im Blatt. Modi zum Unsupported-/Sollwertvergleich nur intern beim ignorierten
Testcontroller, kein öffentlicher Host-CLI-Aufruf. Normaler Workerzweig validiert Bootstrap vor
seinem begrenzten Versuch und fällt niemals in generische Netzwerkdiagnostik.

Der Diagnosejob erlaubt zwei Plätze, um Fehler 367 von Quotenablehnung abzugrenzen; beobachtet je
Job genau ein Blatt und kein Kind. Das ist kein BP1-zu-Native-Grant oder Produktbudget. Ergebnis-
Hashes sind MAC-Record-gebunden. Drei gestartete Aufträge verbraucht, echter Store-Wiederanlauf ohne
Pending bestätigt, Profile/Fixtures entfernt; beide Erfassungen gestoppt/konvertiert und Supervisor
beendet. Neue Programmbytes brauchen wieder eigenen Netzbeleg. Mittleres Prüfniveau.

Vorheriger unveränderter Sperrstand:

Exaktes Paar weiter `%TEMP%/kraqel-c4b-module-gate-20261118-03/`: Root `daf59d0…`,
Host-Testcontroller `83679e5…`, funktionaler Bericht `run-functional.json`, neuer gemessener Bericht
`run.json`, getrennte private MAC-Audits, `etw-independent-audit.json`. Frische Rohmessungen unter
`%TEMP%/kraqel-c4b-gate-etw-20261118-01-{reference,gate}/`, geschützte zusätzliche Sicherung
`artifacts/c4b-module-gate-etw-20261118-01/` außerhalb von TEMP einschließlich exaktem Paar,
ETL/XML, Audit und Hashmanifest. Keys/PIDs/Pfade/Kommandozeilen bleiben lokal; keine Auditkopie als
aktiven Store oder wiederverwendbaren Grant übernehmen. Alte Nachweise/Bytepaare unverändert.

Runtimequelle weiter `%TEMP%/kraqel-c4-runtime-20261117-01/expanded/`; CAB-Sicherung unverändert
unter `artifacts/session-close-20261117-01/`. Bei TEMP-Verlust eigenes gesichertes CAB entpacken und
Manifesthashes prüfen. Keine echten Projekte/echte Vorschauablage starten und Pending nicht
reparieren. BP1 hat Modell-/Helfertests und Clippy ausgeführt; keine neuen GUI-/Runtimeversuche oder
ETW-Läufe.

Gemessene Prozesse beendet, Testprofil/Daten-/Kontrollfixtures entfernt; beide eigenen Erfassungen
gestoppt/konvertiert und eigener Supervisor beendet. Bestehende Systemerfassungen nicht angefasst.
Mittleres Prüfniveau und striktes Datenschutzziel beibehalten. Neue Programmbytes benötigen einen
eigenen frischen Netzbeleg; diese Messung erteilt keine Runtime-/Produktfreigabe.

Vorheriger nativer Netzbeleg · C3, 17. November 2026:

[Gezieltes C3-ETW](testing/product-start-p2c-group-bootstrap-etw.md) des unveränderten Builds
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

Gesicherter Controller: `%TEMP%/kraqel-product-p2c-group-build-20261117-01/controller.exe`, SHA-256
`b24b3aac3d80bc59b63c74e2cbad03bf4980d8496b88434f3afc035fdbbe48af`, Runtimequellstand `6e2c089`.
Private Captures `%TEMP%/kraqel-c3-etw-20261117-01-reference` und `…-group` konvertiert;
Supervisor/Testprozesse beendet, drei Profile entfernt. Audit/Manifest beim Build ergänzt,
synthetische Daten/Kontrollketten lokal erhalten. Ein negativer Store bleibt pending: nicht
reparieren oder fehlendes Profil als Startfreigabe auslegen. Eigener Fixturecleanup war keine
Broker-Reconciliation. Rohdaten/Keys/PIDs nicht nach GitHub oder neue Programme an echter
Vorschauablage autorisieren. Für C4 Paketclosure/Runtime-/Command-/Kindrollenvertrag lesen;
C3-Gruppenschlüssel nicht zur Rendererautorität machen. Ältere Netzbelege bleiben bytegebunden.

Vorheriger Implementierungs-/Netznachweis:

[Frischer Workerbindungs-/Wiederanlauf-Netzbeleg](testing/product-start-worker-lifetime-etw.md).
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

Gesicherter Runtimebuild bleibt `%TEMP%/kraqel-product-worker-lifetime-build-20261117-01`,
Quellstand `03e4ed1`. Lokale Captures `%TEMP%/kraqel-worker-v3-etw-20261117-01-reference` und
`…-recovery`, zusammengefasster Audit und ergänztes Manifest bei der Buildsicherung. Traces und
Testprozesse beendet, Profile aufgeräumt; ein negativer Kontrollstore bleibt absichtlich gesperrt.
Keine Rohdaten nach GitHub oder neue Programme an echter Vorschauablage autorisieren. Für die
nächste Umsetzung den oben verlinkten P2c-Vertrag lesen; keine UI-ABI-0-Kopie oder
Ein-Prozess-Lebenszeitregel stillschweigend als Produktfreigabe nutzen.

Vorheriger Stand:
[Dauerhafte Workerbindung und Wiederanlauf](testing/product-start-worker-lifetime.md). Dauerhafte
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

Gesicherter neuer Build `%TEMP%/kraqel-product-worker-lifetime-build-20261117-01`,
`functional-root.txt`, privater Audit/Manifest. Reproduktion mit
`check-workspace-worker-crashes.ps1` und expliziten Hashes aus dem Nachweis. Ein negativer
synthetischer Store bleibt absichtlich pending: kein nachträgliches Binden, kein Schlüssel-/Kopf-
Reset nach Fixturecleanup. Echte Vorschauablage nicht öffnen, übernehmen oder für neue EXE
autorisieren. V2-Build/ETW getrennt unverändert behalten. Kein Übertrag der Ein-Prozess-Regel auf
WebView2/mehrprozessige GUI und kein Importticket als Startautorität.

Vorheriger Stand:
[Frischer Ressourcen-Reconciliation-Netzbeleg](testing/product-start-resource-reconciliation-etw.md).
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

Build weiterhin `%TEMP%/kraqel-product-resource-crashes-build-20261117-01`, Runtimequellstand
`1d24762`. Frische lokale Captures `%TEMP%/kraqel-resource-v2-etw-20261117-01-reference` und
`…-reconciliation`; zusammengefasster Audit und ergänztes Manifest bei der Buildsicherung. Alle
eigenen Traces und Testprozesse beendet, keine Rohdaten nach GitHub. Keine Profilreste adoptieren
oder neue Brokerprogramme an echter Vorschauablage autorisieren. Vor Live-Crash-Recovery
Authentizität und dauerhafte Lebenszeitbindung planen/prüfen; verschwundener Jobname und unsignierte
Logs ersetzen keinen Beleg. Vorherige Implementierungsabschnitte sind historisch.

Vorheriger Stand:
[Begrenzte Ressourcen-Reconciliation](testing/product-start-resource-reconciliation.md).
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

Gesicherter neuer Build: `%TEMP%/kraqel-product-resource-crashes-build-20261117-01` mit
`functional-root.txt`, privatem Audit und Buildmanifest. Reproduktion ausschließlich mit
`check-workspace-resource-crashes.ps1` und expliziten Hashes aus dem Nachweis. Keine alten
Execution-v1-Records migrieren oder neue Programme an echter Vorschauablage autorisieren. Vorige
Builds und ETW-Rohdaten getrennt unverändert behalten. RunIntent ohne Drained bleibt gesperrt; keine
Schlussfolgerung aus verschwundenem Jobnamen oder unsignierten Diagnosen.

Vorheriger Stand: [Frischer P2b-Netzbeleg](testing/product-start-p2b-bootstrap-etw.md). Frisches
P2b-ETW, 17. November: unveränderter gesicherter Runtimebuild `590c3d4…`, zwei kurze gültige
Messungen – Kontrollablagenreferenz und drei gezielte Workerfälle. 31 Controller, sechs Worker, 31
direkt zugeordnete Controller-Konsolenhosts, alle 68 Enden explizit. Acht lokale
TCP-/UDP-IPv4/IPv6-Kontrollen, Nullverluste/Zuordnungslücken, null beobachtete Netzereignisse in den
gemessenen Bäumen (auch null Loopback). Worker normal/MAC-Ablehnung/Sperre vor Resume und neue
Generationen mit Replay/Kopfabwehr; native Daten/MACs, Token/Job/Lease und sechs entfernte Profile
unabhängig bestätigt. Rohdaten nur lokal, echte Vorschauablage unverändert. 42 Diagnose-Tests;
zusätzlich reinen Modelltest für Linux-Clippy korrigiert (Accessor statt privater Set-Zugriff), acht
gezielte Kernprüfungen/Clippy grün. Keine Runtimeänderung/neuer Build,
App-/Transport-/Installerumbau oder Workspace-Gesamtlauf. Nächster Scope: gesicherte
Ressourcen-Crash-Reconciliation der P2b-Probeprofile, insbesondere
Profilerzeugung/ProfileReady/Entfernung/Abschlussrecord. Keine Adoption unklarer Profile oder
Bestandsdaten; P2 insgesamt, P3–P5 und Produkt-Q-01/Q-02 offen. Striktes Datenschutzziel
unverändert, keine Übertragung auf normale App oder ungemessene Pfade.

Gesicherter Runtimebuild bleibt `kraqel-product-start-p2b-build-20261117-05`, Runtimequellstand
`4c26ba4`. Hashidentische Messkopie `etw-package/kraqel-isolation-probe.exe`; sechs separat kopierte
`worker.exe` durch private Records/Hash/Kernelstarts abgeglichen. Lokale Capturepfade
`%TEMP%/kraqel-p2b-etw-20261117-01-reference` und `…-worker`, keine Rohdaten nach GitHub. Kein neuer
Runtimebuild für die reine Test-/Dokumentationskorrektur. Nächste Arbeit gezielt an
P2b-Ressourcencrashes; echte `.kraqel-broker-preview` weiterhin nicht öffnen, zurücksetzen oder neue
Brokerhashes dort autorisieren. Frühere Abschnitte beschreiben historische Gates.

Vorheriger Stand: [P2b · synthetischer Bootstrap](testing/product-start-p2b-bootstrap.md).
Produktstart P2b, 17. November: authentisierter synthetischer Workerbootstrap unter derselben
Kontrollsperre, ohne GUI oder Importtask. Separate frische Probeprofile/-daten, keine ACL-Promotion
der privaten P2a-Fixture. Commit/ProfileReady/Receiptabschluss bzw. kontrollierte Reconciliation
persistiert; exakte Token-/Job-/Leaseprüfung vor Resume, eigener Einmalschlüssel statt
Kontrollschlüssel, begrenzte Pipes mit monotoner Receiptfrist. Jobleerstand am gehaltenen Objekt vor
Profilentfernung, unklare Crashressourcen bleiben gesperrt; keine automatische Crashbereinigung.
`.kqboot` Probe-ABI 1/UI-ABI 0, Legacy-/P2a-Marker unverändert. Vier neue Tests, 71 aktive
Helfer-/Transporttests, gezielte Nachprüfung/Clippy grün. Drei finale native Fälle, 27
Kontrollprozesse, sechs Worker (vier Erfolge, eine MAC-Ablehnung, eine Sperre vor Resume), 30
private Zugriffssperren, sechs Probeprofile entfernt; unabhängiger
MAC-/PE-/Root-/Prozess-/Profilaudit grün. Echte Daten und Vorschauablage unverändert. Finaler
Broker/Worker `590c3d4…`, Sicherung `kraqel-product-start-p2b-build-20261117-05`, mehrere vorherige
Entwicklungsbuilds nicht angenommen. Mittleres Prüfniveau ohne App-/Transport-/Installerumbau oder
`pnpm check`. Nächster Scope: gezielter frischer ETW-Netzvergleich dieses Controllers samt separatem
Workerpfad und beiden Ablehnungen. Alter ETW-Beleg nicht übertragbar. Danach
Ressourcen-Crash-Reconciliation und echte persistente Arbeitsablagen-/UI-Bindung; P2 insgesamt/P3–P5
und Produkt-Q-01/Q-02 offen, striktes Datenschutzziel unverändert.

Reproduktion mit `scripts/runtime-isolation/check-workspace-bootstrap.ps1` und expliziten Hashes aus
dem Nachweis. Keine GUI starten, keine echte Vorschauablage öffnen/zurücksetzen oder neue EXE dort
autorisieren. Für ETW getrennte Controller-/Worker-PIDs samt Profil-/Jobzuständen erfassen; keine
alten Netzbelege auf neue Bytes übertragen. Vorherige P2a-/P1-Abschnitte sind historisch.

Vorheriger Stand: [Produktstart P2a · Record-only](testing/product-start-p2a-records.md).
Produktstart P2a, 17. November: authentisierte persistente Schema-5-Bindungs-/Launch-Records im
pfadbegrenzten Forschungschild implementiert, nur synthetische private Ablagen ohne GUI. P1-Kern
wird nach nativen Actor-/Kopf-/Job-/MAC-Prüfungen genutzt, Zustand aus Recorddigests rekonstruiert;
erst passender Abschlusskopf gibt Store frei. `.kqprod` deklariert Record-ABI 1 und UI-Bootstrap-ABI
0; Legacy-`.kqmeta` 1–4 unverändert. Keine Startcapability, kein Profil-/Importtask-/UI-Start. Drei
native Abbruchfälle mit 31 Prozessen: vor Record, nach Kopf, unveröffentlichter Tail. Replay bleibt
gesperrt, Tail wird nicht repariert; Schlüssel/Policy/Unicode-Daten erhalten. Vier neue Tests, 67
aktive Helfer-/Transporttests/Clippy und unabhängiger MAC-/PE-/Root-/Inventaraudit grün. Ein
Helferbuild, kein App-/Transport-/Installerumbau oder Workspace-Gesamtlauf. Echte
Vorschau-/Projektdaten unverändert, Prozesse beendet. Brokerprivate Fixture ist kein
Container-Produktvault; nicht per ACL promovieren. Neuer Broker ohne eigenen frischen ETW-Beleg.
Nächster Scope P2b: authentisierter synthetischer Bootstrap mit Token/Job vor Ausführung, ohne
Importtask; vor weiterer Freigabe eigener Netznachweis. Produktintegration/Bestandsübernahme offen,
striktes Datenschutzziel unverändert.

Reproduktion `scripts/runtime-isolation/check-workspace-records.ps1` mit gesicherten Hashes aus dem
Nachweis; aktuelle Prüfung verwendet Build `kraqel-product-start-p2a-build-20261117-02`. UI-Fixture
nur als Datei kopiert, noch nicht startfähig für den Produktvertrag. P2b benötigt explizit getrennte
Appdaten/Paket-/Konfigurationsbindung, passenden Bootstrap und native Vor-Ausführungschecks. Keine
Autorisierung des neuen Brokers an der echten Vorschauablage und keine alten Records löschen.

Neuester funktionaler Prüfstand: [Produktstart P1 · Testmodell](testing/product-start-p1-model.md).
Produktstart P1, 17. November: reine versionierte Bindungs-/Startvorschlagstypen und Zustandsregeln
implementiert, ausschließlich `#[cfg(test)]` in `probe/src/main.rs`; keine produktive Autorität.
`product_start.rs`/`product_start/tests.rs` ohne I/O/unsafe/Schlüssel/Transport/UI. Parser liefert
`UntrustedStartProposal`, lehnt Import-/Fremdfelder und ungültige Versionen/Bindungen ab. Modell
verbraucht Launch vor Running, erhält Binding/Replayhistorie bei Ende/Unterbrechung, reserviert
Abschlussbudget; Ablehnungen ohne Mutation. Sieben neue Modelltests, insgesamt 63 aktive Helfer-/
Transporttests und Clippy grün. Vorgeschlagenes Schema 5 bleibt nativ unimplementiert; Modellköpfe/
Jobbedingungen sind keine MAC-/OS-Nachweise, keine Persistenz/Bootstrapfrist/Frameintegration.
Mittleres Prüfniveau, kein App-/Helfer-EXE-Build, `pnpm check` oder neuer ETW-Lauf. Echte Daten/
ACLs/Schlüssel unverändert, gesicherte Forschungsprogramme separat erhalten. Nächster begrenzter
Scope P2a: authentisierte persistente Arbeitsablagenbindung/Launch-Records im Forschungsbroker, nur
frische synthetische Wurzeln und zunächst ohne GUI; dann P2b Bootstrap/Token/Job ohne Importtask.
Normale Produktintegration und Produkt-Q-01/Q-02 offen; striktes Datenschutzziel unverändert.

P2a darf weder Modellbedingungen als echte Token-/Jobattestierung noch Parseerfolg als
Authentisierung verwenden. `StartModel` ist nicht serialisierbar; kein heimlicher Persistenzweg.
Bisheriges RestartTicket/Importtask und Forschungs-CLI bleiben unverändert; Schema-5-/Paket- und
Kontroll-MAC-Integration separat implementieren. Keine Freigabe/Kopie echter Vorschau-/Projektdaten.

Neuester Planungsstand:
[Produktstartvertrag v1 · Entwurf](data-format/product-start-integration-v1-draft.md).
Produktstart-/Bestandsdatenvertrag v1, 17. November: neuer ausdrücklicher Entwurf, keine
Implementierung/Produktfreigabe. Statisch bestätigt: normaler Tauri-Start ohne vorgeschalteten
Produktguard, getrennte Forschungs-UI, taskgebundenes RestartTicket v3 und migrationsfähige
SQLite-Öffnung. Vorgesehen: eigene Workspace-Startautorität ohne Importtask, kein unisolierter
Fallback, getrennte Hostkontrolle/Arbeitsablage, zerstörungsfreie explizite Bestandskopie mit
erhaltenen IDs. Kontrollschema 5/Paketdeklaration geplant, nicht aktuell unterstützt; Forschung mit
einer Sitzung/256 Records ist keine Produkt-Langzeitlösung. Stufen P1–P5 mit Prüfgates festgelegt.
Nächster begrenzter Scope P1: reine versionierte Start-/Bindungstypen samt Zustandsregeln und
gezielten Negativfällen, ohne UI-/Datei-/Produktfreigabe. Nur Dokumentation geändert, Quellen-/
Verweis-/Format-/Diffprüfung; kein Build/neue Testmatrix. Keine echten Daten geöffnet/übernommen
oder ACLs/Schlüssel geändert. Normaler Start weiter unisoliert, Produkt-Q-01/Q-02 offen; striktes
Datenschutzziel unverändert. Letzter eigener Forschungs-Netzbeleg bleibt am unveränderten Build.

Nicht das normale `run()` mit dem Forschungs-RestartTicket verdrahten. P1 erzeugt zunächst nur einen
getrennten Typ-/Zustandsvertrag; keine neue Startautorität allein durch gültiges JSON.
Kontroll-MAC-Schlüssel bleiben im Broker. Bestandsinventarisierung nicht über den migrationsfähigen
`Database::open`-Weg; Snapshotimport erzeugt neue IDs und ist keine identitätserhaltende Übernahme.

Neuester Netzbeleg:
[Prepare-Übergänge/verwalteter Kandidat](testing/native-prepare-transition-etw.md). Frisches ETW der
Prepare-Übergänge und des verwalteten Kandidaten, 17. November: unveränderte gesicherte Programme,
zwei kurze gültige Phasen. Regulärer Schema-4-Update-/Recoverypfad als Referenz gegenüber den drei
gezielten Übergangsabbrüchen; keine zusätzliche allgemeine Matrix. Alle 59 direkten Aufrufe
einschließlich fünf hashbenannter verwalteter EXEs erfasst, acht Kontrollen,
Nullverluste/Zuordnungslücken, 182 explizite Enden, 0 beobachtete Nicht-Loopback-/48 getrennte
Loopback-Ereignisse. Unabhängige MAC-/Paket-/SQLite-/Kernel-Audits grün: vier Aufgaben/vier legitime
Kopien, keine Replaykopie, Schlüssel/App-/Sitzungsbindung erhalten. 42 Diagnose-Tests, mittleres
Prüfniveau ohne neuen Build oder Workspace-Gesamtlauf. Echte Vorschauablage unverändert,
Erfassungen/Prozesse beendet, Rohbelege lokal. Nächster begrenzter Scope: Start-/Integrationsvertrag
für normalen Produktstart mit bestehenden Projekten konkretisieren; Forschungsautorität nicht
automatisch auf echte Daten übertragen. Striktes Ziel und offene Produkt-Q-01/Q-02 unverändert.

Neuester funktionaler Stand:
[Prepare-Übergangsabbrüche](testing/native-prepare-transition-crashes.md). Gezielte
Prepare-Übergangsabbrüche, 17. November: drei standardmäßig inaktive Punkte im pfadbegrenzten
Forschungschild: `artifact-removed`, `package-created`, `prepare-recorded`. Nach den ersten beiden
bleibt der Kopf unverändert/kein Pending; danach gültiges Pending mit leerer Kopie, vor Retry
ausdrücklich abzubrechen. Drei native Fälle mit 44 Kontrollprozessen/sechs GUIs bestanden; 56 aktive
Helfertests/Clippy grün, zwei vorhandene Tests um leere Kopien erweitert. Unabhängiger
MAC-/Root-/Actor-/SQLite-Audit bestätigt 13/13/15 Records, erhaltene Schlüssel und
App-/Sitzungsbindung sowie drei Aufgaben/Kopien ohne Replayduplikat. Derselbe Storelease bis
Prozessende, keine neue Autorität oder Reparatur. Echte Vorschauablage unverändert, Testprozesse
beendet. Mittleres Prüfniveau, kein Appbuild/Workspace-Gesamtlauf. Neue Brokerbytes ohne eigenen
frischen ETW-Beleg; nächster begrenzter Scope: gezieltes ETW der neuen Programme einschließlich
dieser drei Übergänge und verwaltetem Kandidaten. Unbekannte Artefakte/defekte Ketten bleiben
gesperrt; striktes Datenschutzziel und offene Produktabnahme unverändert.

Reproduktion: `scripts/runtime-isolation/check-update-reconciliation.ps1 -PrepareTransitions` mit
den neuen gesicherten Hashes aus dem Nachweis; alternativ zur bisherigen `-RetryAborted`-Serie.
Keine Prüfprogramme an vorhandener Vorschauablage autorisieren oder deren Autorität kopieren.

Neuester Netzbeleg: [Retry und verwalteter Kandidat](testing/native-package-artifact-retry-etw.md).
Frisches ETW des gesicherten Retry-Brokers, 17. November: unveränderte Programme, zwei kurze gültige
Phasen. Referenz und kombinierter Teilkopie-/Abort-/Retry-/Aktivierungs-/Recoverypfad bestanden.
Alle 21 direkten Aufrufe einschließlich fünf hashbenannter verwalteter EXEs erfasst, unveränderte
Root-/Kalibrierungs-/Verlust-/Lebenszeitguards. Acht Kontrollen, Nullverluste/Zuordnungslücken, 144
explizite Enden, 0 beobachtete Nicht-Loopback-/48 getrennte Loopback-Ereignisse. Unabhängige
MAC-/Paket-/SQLite-/Kernel-Audits grün: drei Aufgaben/zwei legitime Kopien, keine Replaykopie,
Schlüssel/App-/Sitzungsbindung erhalten. 42 Diagnose-Tests, mittleres Prüfniveau ohne neuen Build
oder Workspace-Gesamtlauf. Echte Vorschauablage unverändert; Prozesse/Erfassungen beendet, Rohbelege
lokal. Nächster begrenzter Scope: Prozessabbrüche zwischen Artefaktentfernung, Zielanlage und
Prepare-Record gezielt prüfen. Unbekannte Artefakte/defekte Ketten bleiben gesperrt; striktes
Datenschutzziel, normale Produktintegration und offene Produkt-Q-01/Q-02 unverändert.

Neuester funktionaler Stand: [Paketartefakte/Retry](testing/native-package-artifact-retry.md).
Sichere Paketartefakte/erneute Vorbereitung, 17. November: Schema 4 leitet ersetzbare Hashs nur aus
authentisierten Abort-Records ab. Aktueller Actor/Kopf, Jobleerstand und erlaubter Übergang vor
jeder Dateimutation; dieselbe Kontrollsperre durchgehend gehalten. Geprüfte Einlink-Datei über
exklusiven Handle entfernen; aktive/früher aktivierte oder unbekannte Pakete, unsichere Links/ACLs
und gesperrte Dateien nicht bereinigen. Zieldatei vor neuem Pending exklusiv erzeugen. Drei neue
Tests, 56 aktive Helfertests/Clippy und drei native Retryfälle/Daten-MAC-Audit grün. Schlüssel,
App-/Sitzungsbindung und drei legitime Projektkopien erhalten, kein Replayduplikat. Echte
Vorschauablage unverändert, Testprozesse beendet. Neuer Broker ohne eigenen frischen ETW-Beleg;
mittleres Prüfniveau, kein App-/Workspace-Gesamtlauf. Nächster begrenzter Scope: gezieltes frisches
ETW des neuen Brokers samt Retry und verwaltetem Kandidaten. Unbekannte Artefakte ohne Abort und
unvollständige Kontrollketten bleiben gesperrt; striktes Ziel und offene Produktabnahme unverändert.

Reproduktion: `scripts/runtime-isolation/check-update-reconciliation.ps1 -RetryAborted` mit
expliziten neuen Hashes aus dem Nachweis, ausschließlich frische TEMP-Cases. Keine neue Prüf-EXE an
vorhandener Vorschauablage autorisieren oder diese kopieren/zurücksetzen.

Neuester funktionaler Prüfstand:
[Update-Abbruch-/Fehlerreconciliation](testing/native-update-abort-reconciliation.md). Drei native
Fälle bestanden, vorhandener Laufzeitcode unverändert: Teilkopie, Manipulation und Crash nach
veröffentlichtem Abort. Dieselbe Kontrollsperre, alter Actor und bestehende Aufgabe bleiben
erhalten; keine defekte Kette reparieren. Zwei neue Tests, insgesamt 53 aktive Helfertests/Clippy
sowie Daten-/MAC-/SQLite-Audit grün. Neuer
`scripts/runtime-isolation/check-update-reconciliation.ps1` erzeugt nur drei frische TEMP-Cases mit
explizit gesicherten Hashes; Reproduktion im Nachweis. Drei legitime Projektkopien, keine
Replaykopie; Profile/Prozesse beendet, Rohbelege lokal. Echte Vorschauablage/verwaltete EXE
unverändert, dort keine Prüfprogramme neu autorisieren. Keine neue ETW-Erfassung dieser Fehlerpfade.
Nächster begrenzter Scope: verbleibende abgebrochene Paketdateien unter derselben Sperre sicher
behandeln und erneute Vorbereitung ohne unbeabsichtigtes Pending ermöglichen. Striktes
Datenschutzziel und offene Produktabnahme unverändert.

Neuester Netzbeleg:
[Frisches ETW beider Updateprogramme](testing/native-schema4-package-update-etw.md). Unveränderte
gesicherte Programme, zwei gültige kurze Phasen, acht Kontrollen, Nullverluste/Zuordnungslücken, 132
explizite Enden, 0 Nicht-Loopback-/48 getrennte Loopback-Ereignisse. Alle 15 nativen Updateaufrufe
samt fünf hashbenannten verwalteten EXEs abgeglichen; private exakte PID-/Image-Map nur in
`native-control`. Daten-/MAC-/Lebenszeitaudits und 42 Diagnose-Tests grün. Drei Aufgaben/zwei
legitime Kopien, keine Replaykopie. Echte Vorschauablage/verwaltete EXE unverändert; Erfassungen und
Prozesse beendet, Rohbelege lokal. Mittleres Prüfniveau ohne neuen Build/Workspace-Gesamtprüfung.
Nächster begrenzter Scope: Update-Abbruch-/Fehlerreconciliation unter derselben Sperre vor normaler
Produktintegration. Striktes Datenschutzziel und offene Produkt-Q-01/Q-02 unverändert.

Vorheriger funktionaler Stand:
[Lokale Schema-4-Brokerpaket-Updates](testing/native-schema4-package-update.md). Read-only
Paketvertrag vor Pending/Aktivierung, kein Ausführen des Kandidaten vor Freigabe. Dieselbe
Kontrollsperre/Jobprüfung. 51 aktive Helfertests/Clippy und ein kurzer nativer Wechsel auf
verwalteten Kandidaten samt Recovery/Cleanup/Daten-MAC-Audit grün. Ein synthetischer Kandidat,
gleicher Code/anderer Hash; genau eine Projektkopie erhalten, Apphash unverändert. Echte
Vorschauablage/verwaltete EXE unverändert, neue Prüfprogramme dort nicht autorisieren oder alte
Kontrollablagen übernehmen. Eigener frischer ETW-Beleg beider Actorprogramme einschließlich
hashbenannter verwalteter EXE inzwischen oben verlinkt. Mittleres Prüfniveau, striktes
Datenschutzziel und offene Produktabnahme unverändert. Testprofil/Prozesse beendet, private
synthetische Belege erhalten.

Vorheriger Prüfstand: Neuester Netzbeleg:
[Gezieltes ETW des unveränderten Cleanup-Paars](testing/native-profile-cleanup-etw.md). Zwei kurze
gültige Phasen, acht Kontrollen, Nullverluste, 140 explizite Prozessenden, 0 Nicht-Loopback-/48
getrennte Loopback-Ereignisse. Alle 19 direkten Kontrollaufrufe einschließlich fensterloser
Cleanup-/Statusprozesse erfasst (`native-control`), erwartete Aufrufe/GUIs/Daten unabhängig
gegengeprüft. 39 Diagnose-Tests grün; keine neuen Builds oder große Matrix. Erfassungen/
Supervisor/Testprozesse beendet; rohe Diagnosen und synthetische Fixtures lokal erhalten. Echte
Vorschauablage/verwaltete EXE unverändert. Nächster Scope: lokale Update-/Paketkompatibilität für
Kontrollschema 4 unter derselben Sperre, weiterhin mittleres Prüfniveau. Striktes Datenschutzziel
unverändert, keine Produktfreigabe.

Vorheriger Prüfstand: Neuester funktionaler Stand:
[Authentisierte Profilbereinigung](testing/native-profile-cleanup.md). Kontrollschema 4 signiert
Vorbereitung vor Profilerzeugung und Cleanupabschluss; Wiederholung mit aktuellem Kopf idempotent,
gleiche Kontrollsperre, keine neue Importautorität. Zwei native Fälle samt kurzem Start/Recovery und
48 Helfertests/Clippy bestanden; Daten-/MAC-Audit grün. Auf Wunsch mittleres Prüfniveau statt großer
Matrix/Workspace-Gesamtprüfung. Kein frisches ETW des neuen Brokerhashs; nächster Scope: gezielter
Netzvergleich des Cleanup-Stands. Echte Vorschauablage und unten genannte autorisierte EXE
unverändert; neuer Prüfbroker nicht automatisch aktiviert. Testprofil/Prozesse beendet, private
synthetische Belege erhalten. Ungebundene Alt-Orphans und unvollständige Kontrollketten nicht
übernehmen oder reparieren. Striktes Ziel unverändert, Produktintegration weiter offen.

Vorheriger Prüfstand: Neuester Prüfstand:
[Übergangs-Crashpunkte und frisches ETW](testing/native-session-transition-crash-etw.md). 20 neue
Übergangsabbrüche, neun gesperrte/elf authentisierbare Kontrollzustände, vier explizite Recoveries;
Status ist kein Ausführungsgrant. Testprofil-Cleanup repariert keinen Kontrollzustand. Beide
frischen ETW-Phasen gültig: acht Kontrollen, Nullverluste, 458 explizite Enden, 0
Nicht-Loopback-/138 getrennte Loopback-Ereignisse. 46 aktive Helfertests, `pnpm check` und 36
Diagnose-Tests grün; unabhängige Daten-/MAC-/Kernel-Lebenszeitaudits bestanden. Profile/Prozesse/
Erfassungen beendet; gesperrte synthetische Fixtures privat erhalten. Echte Vorschauablage und
nachfolgend genannte autorisierte EXE unverändert, neuer Prüfbroker nicht automatisch aktiviert.
Nächster Scope: authentisierte Bereinigung verwaister/stillgelegter Profile unter derselben
Kontrollsperre, keine neue Importautorität oder Schlüssel-/Kopfreparatur. Striktes Ziel unverändert;
Schema-3-Updates/echte Datenmigration/Produktintegration bleiben offen.

Vorheriger Prüfstand: Neuester funktionaler Prüfstand:
[Dauerhafte Sitzungsbindung/Recovery](testing/native-product-session-recovery.md). Neue native
Vorschau mit explizitem Kontrollschema 3, persistiertem Apphash/Sitzungswurzel/Task und verbrauchten
Startkennungen. Zwei echte Controllerabbrüche, vier Recoveries/acht Replay-Ereignisse, 36
Dateiverweigerungen, genau eine legitime Projektkopie; unabhängiger MAC-/Datenaudit, 45 aktive
Helfertests und `pnpm check` grün. Profile entfernt/Prozesse beendet; synthetische negative Fixtures
privat erhalten. Echte Vorschauablage und nachfolgend genannte autorisierte EXE unverändert. Kein
automatisches Aktivieren dieses neuen Prüfbrokers oder Migrieren echter Daten. Schema 3 erlaubt
derzeit keine Brokerupdates, eine Sitzung je Installation. Kein frischer ETW-Beleg dieses
Brokerhashs. Nächster Scope: zusätzliche Bindungs-/Startrecord-/Stilllegungs-Crashpunkte, danach
frischer ETW-Vergleich. Striktes Datenschutzziel unverändert; keine Produktfreigabe.

Vorheriger funktionaler Prüfstand: Neuester funktionaler Prüfstand:
[Kontrollgebundener isolierter Startpfad](testing/native-product-starter.md) mit neuer gesicherter
Broker-EXE. Zwei neue synthetische Sitzungen verwenden dieselbe private Kontrollsperre und denselben
Produktjob; Quelle/Kopie gepinnt, zwölf private Dateiverweigerungen und unabhängiger MAC-/Datenaudit
bestanden. 43 aktive Helfertests, Clippy/`pnpm check` grün. Echte Vorschauablage und nachfolgend
genannte autorisierte EXE unverändert, kein automatisches Aktivieren des neuen Prüfbuilds. Neue
Appdaten bleiben getrennte TEMP-Fixtures, keine vorhandenen Projekte/Journale übernommen. Kein
frischer ETW-Beleg dieses Brokerhashs. Nächster Scope: authentisierte dauerhafte
Apppaket-/Sitzungsbindung und Wiederanlauf unter derselben Sperre, danach eigener ETW-Vergleich.
Striktes Ziel unverändert, keine Produktfreigabe.

Vorheriger Prüfstand mit eigenem Netzbeleg:
[Kontroll-Crash-Serie und frisches ETW](testing/native-product-control-crash-etw.md) mit separat
gesichertem Prüfbroker/Updatekandidat bestanden. 28 Abbrüche, 17 gesperrte/elf nutzbare Zustände,
acht explizite Weiterführungen ohne Schlüsselreset; zwei gültige ETW-Phasen mit acht Kontrollen,
Nullverlusten, 1.136 expliziten Enden, 0 Nicht-Loopback-/444 getrennten Loopback- Ereignissen. 40
Helfertests, Clippy/`pnpm check` und 36 Diagnose-Tests bestanden. Alle GUI-Testprofile entfernt,
Prozesse/Erfassungen beendet; private synthetische Crash-Fixtures bleiben lokal. Wichtig: Echte
`.kraqel-broker-preview` und ihre nachfolgend genannte autorisierte EXE wurden nicht verändert. Der
neue Prüfbroker ist dort nicht automatisch autorisiert. Keine Kopf-/Recordreparatur, keinen
Schlüsselreset und keine Kopie der Kontrollablage auf andere Rechner. Nächster Scope: isolierter
Produktstarter mit gleicher Sperre und getrennten Datenrollen, echte Datenmigration und gesperrte
Abschlüsse separat prüfen. Striktes Ziel unverändert, keine Produktfreigabe.

### Frühere Prüfstände (absteigend, damalige nächste Schritte)

Der aktuelle nächste Schritt steht oben. Frühere Netzbelege gelten jeweils nur für ihre eigenen
Programmhashes und Lebenszeiten.

Vorheriger Stand:
[Native Kontrollablage/Updates/Kontrollmigration](testing/native-product-control.md) als Vorschau
implementiert und nativ geprüft. Lokale dauerhafte `.kraqel-broker-preview` im Benutzerprofil bleibt
erhalten; keine privaten Schreibprojekte übernommen. Kontrollschema 2, drei explizite
Programmwechsel mit durchgehender MAC-Kette; finale verwaltete EXE unter
`%USERPROFILE%/.kraqel-broker-preview/bin/cfe51760acc79112a5bff612326fa5cf1f6ef94d97cd897f7b09cc14850ae51c.exe`.
Weitere Änderungen nur über aktuell autorisierte EXE und aktuellen nativen Kopfhash, keine
Schlüssel-/Kopf-/Journal-Reparatur. Auf anderem Rechner neue Vorschauablage anlegen; niemals diese
Schlüssel oder Forschungswurzeln kopieren. 39 Helfertests, Clippy/`pnpm check`, 30 native Kontroll-
aufrufe, zehn erwartete Abweisungen, 18 echte Container-Dateiverweigerungen und GUI-Regression
bestanden. Alle Testprofile entfernt/Prozesse beendet. Die damals folgende Kontroll-Crash-Serie und
deren eigener frischer ETW-Lauf sind inzwischen oben belegt. Normale Produktintegration, echte
Datenmigration und Herausgebervertrauen bleiben offen; striktes Ziel unverändert.

Neuester Netzbeleg:
[Frisches ETW des Fensterende-Paars](testing/native-combined-window-close-etw.md) einschließlich
beider manuell bedienten Hauptfensterschlüsse und automatischer Wiederherstellung bestanden. Paar
unverändert, Hashes im Nachweis. Zwei getrennte gültige Phasen, acht Kontrollen, Nullverluste, alle
942 Enden explizit; 0 Nicht-Loopback-/536 getrennte Loopback-Ereignisse. Unabhängige
MAC-/Daten-/Lebenszeitaudits grün, 22 Aufträge/13 legitime Kopien. Alle elf Profile entfernt,
Erfassungen/Prozesse beendet; 36 Diagnose-Tests bestanden, kein Runtimeumbau/neuer Build. Nächster
Scope: geschützte dauerhafte Produktablage und authentisierte Updates/Migration. Weitere allgemeine
F08-/Fensterende- und externe Dienstgrenzen offen. Rohdaten/Schlüssel bleiben lokal; auf anderem
Rechner frische Testwurzeln und eigener Nachweis. Striktes Ziel unverändert, keine Produktfreigabe.

Vorheriger Funktionsstand: [Kombinierte Fensterendeprüfung](testing/native-combined-window-close.md)
für offenen nativen Dateidialog und Teiltransfer bestanden. Gesichertes finales Paar mit Helferhash
`1b5c0efa771f58a7e1274369d391b92a2c788aa1d64756e6ef4530b2f2555b4f`, App unverändert. Nach Schließen
reguläres Ende/Threadjoin/Jobleerstand, Abbruch ohne Kopie; danach je Recovery, gesperrte
Altanfragen, neuer Auftrag und zweifache Wiederherstellung geprüft. 16 Apps, vier Aufträge, zwei
legitime Kopien; unabhängige MAC-/Datenprüfung, 31 Helfertests/Clippy und `pnpm check` bestanden.
Alle Profile einschließlich des verworfenen Erstlaufs bereinigt, Prozesse beendet. Damals nächster
Scope: frischer ETW-Netzvergleich des neuen Paars mit beiden Fensterendepfaden, keine Übertragung
älterer Netzbelege. Rohdaten/Schlüssel lokal; auf anderem Rechner frische Testwurzeln verwenden.
Striktes Datenschutzziel unverändert, keine Produktfreigabe.

Vorheriger Nachweis:
[Tastatur und frisches ETW des manuellen Brokerpaars](testing/native-combined-assistive-restart-etw.md).
Eigener Tastaturvorlauf plus vier gemessene Wiederanlaufzustände menschlich bestätigt und nativ
gegengeprüft; Pointer-Ereignisse nicht als physischer Eingabenachweis auslegen. Zwei gültige Phasen,
acht Kontrollen, Nullverluste, alle 906 Enden explizit; 0 Nicht-Loopback-/514 getrennte Loopback-
Ereignisse. Unabhängige MAC-/Daten-/Lebenszeitaudits bestanden. Gesichertes App-/Helferpaar
unverändert, Hashes im Nachweis; kein Runtimeumbau. Alle Profile entfernt, Messungen/Prozesse
beendet. Nächster Scope: weitere kombinierte Fensterendeprüfung, danach
Produktablage/Updates/Migration. Rohdaten/Schlüssel bleiben lokal; auf anderem Rechner neue
Testwurzeln und eigener Nachweis. Keine Produktfreigabe oder Datenschutzausnahme.

Vorheriger Stand: [Kombinierter assistiver Nachweis](testing/native-combined-assistive-restart.md)
für Unterbrechung, Abschluss, synthetischen Abbruch und ungültiges Archiv mit Hör-/Fokusbestätigung
und nativem Replay-Gegenbeleg. Zwei Erstversuche korrekt abgelehnt, reine Tastaturabnahme damals
offen. Neue Helferaktion `ui-broker-recover <Wurzel> manual`, App unverändert; neue Hashes im
Nachweis. 28 aktive Helfertests, Clippy und `pnpm check` bestanden. Alle vier Profile entfernt,
keine Forschungsprozesse mehr. Damals nächster Scope: frischer ETW-Netzvergleich des neuen Paars mit
automatischer Regression und manuellen Wiederanläufen. Alte ETW-Belege nicht übertragen.
Rohdaten/Schlüssel nur lokal, niemals über Git transportieren; auf anderem Rechner frisch bauen und
neue Testwurzeln verwenden. Striktes Datenschutzziel unverändert, keine Produktfreigabe.

Älterer Nachweis 16. November: Frisches ETW exakt des gesicherten kombinierten v3-Paars bestanden.
Zwei getrennt kalibrierte Phasen: Automatik einschließlich 54 abgewiesener Folgeaktionen, danach
tatsächliche manuelle Dateiauswahl mit Wiederanlauf/neuem Auftrag/Replay. 114 Broker/80 Apps/480
WebViews/194 Console-Hosts, 868 explizite Enden, acht Kontrollen, Nullverluste, 0 Nicht-Loopback-
und 480 getrennte Loopback-Ereignisse. Daten/MACs und Generationszuordnung unabhängig geprüft.
[Nachweis und Grenzen](testing/native-combined-live-restart-etw.md). Alle Profile entfernt,
Erfassungen/Prozesse beendet, Paar unverändert. Damals nächster Scope: kombinierte assistive
Wiederanlauf-/Fehlerprüfung und weitere Fensterendeprüfung mit genau diesem Paar. Produktintegration
und externe Dienste weiter offen; keine Übertragung auf Folgeänderungen, striktes Datenschutzziel
unverändert.

Vorheriger Funktionsstand 16. November: Kombinierter Forschungsablauf v3 mit echter Live-Auswahl,
Wiederanlauf und ausdrücklich neuem Folgeauftrag bestanden. Signierte Grantkette und private
Abschlussbelege ersetzen keine neue menschliche/native Beauftragung.
[Verträge und finale Belege](testing/native-combined-live-restart.md). Neun automatische Fälle, 72
App-Starts, 54 verweigerte Folgeaktionen; manueller Dialoglauf bestätigt und vollständig
weitergeprüft. Beide Vorläufersitzungen und fehlgeschlagener Dialoglauf stillgelegt; finale Profile
entfernt, Prozesse beendet. Normale Debug-App ohne Forschungsseite wiederhergestellt. Gesichertes
neues Paar: `%TEMP%\\kraqel-kq069-combined-build-20261116-04`, vollständige Hashes im Aggregat. Alte
Binaries/Nachweise nicht übertragen. Nach dem oben ergänzten ETW-Beleg folgen assistive und weitere
Fensterendeprüfung des kombinierten Ablaufs. Produktablage, authentisierte Updates/Migration und
externe Dienste bleiben offen; striktes Ziel unverändert.

Neuester Nachweis 16. November: Frisches ETW des gesicherten HMAC-Paars bestanden, einschließlich
aller negativen Brokerstarts. 103 Broker/30 Apps/180 WebViews/133 Console-Hosts, 446 explizite
Enden, vier Kontrollen, Nullverluste, 0 Nicht-Loopback- und 180 getrennte Loopback-Ereignisse.
Native Serie, unabhängige MACs/Daten und PID-Wiederverwendung geprüft.
[Nachweis/Buildgrenzen](testing/native-hmac-broker-etw.md). Erfassung/Prozesse beendet. Nächster
Scope: kombinierter Live-/Neustartpfad, geschützte Produktablage und authentisierter Update-/
Migrationspfad; externe Dienste/Produktintegration weiter offen. Striktes Ziel unverändert.

Aktuell 16. November: Broker-Vertrag [v2](data-format/native-broker-recovery-v2.md) entfernt DPAPI
und authentisiert den dauerhaften Beleg lokal mit HMAC-SHA-256. Schlüssel nur unter privater DACL,
keine Verschlüsselung/Hardwarebindung behaupten; keine automatische Schlüsselneuanlage oder
Altdatenmigration. [Funktionsnachweis](testing/native-hmac-broker.md): sechs harte Controllerenden,
zwölf Wiederanlaufprozesse, 30 App-Generationen, 78 Ablehnungen und unabhängige Daten-/MAC-Prüfung
bestanden. Neues Paar separat unter `%TEMP%\kraqel-kq069-hmac-broker-build-20261116-01` gesichert;
eigener ETW-Beleg ist oben verlinkt. Alte Paare und Belege nicht übertragen. Produktintegration und
externe Grenzen bleiben offen, striktes Ziel unverändert. Historische Übergaben folgen.

Historischer DPAPI-Produktblocker: DPAPI-Domänencontroller-/RPC-Grenze gesondert prüfen oder
Primitive ersetzen. Keine Produktintegration allein aufgrund eines unauffälligen ETW-Appbaums;
[Vertrag](data-format/native-broker-recovery-v1.md). Striktes Ziel unverändert.

Neuester Stand 16. November: Controller-Crash-Serie und dauerhafte native Wiederanlaufautorität im
Forschungslauncher bestanden: sechs harte Controllerenden, zwölf neue Broker-Wiederanläufe, 30
App-Generationen, 60 Ablehnungsfälle. [Vertrag](data-format/native-broker-recovery-v1.md),
[Nachweis und gesicherte Hashes](testing/native-controller-crash.md). Daten unabhängig geprüft,
Profile entfernt, Prozesse beendet. Gesichertes Paar lokal unter
`%TEMP%\\kraqel-kq069-controller-broker-build-20261116-03`; alte Paare unverändert erhalten.
Nächster Scope: eigener ETW-Netzvergleich dieses neuen Paars, danach kombinierter Live-/Neustartpfad
und Produktbrokerintegration. Alter ETW-Beleg ist keine Freigabe des neuen Brokers. Produktstart
weiter unisoliert; striktes Datenschutzziel und offene Produkt-Q-01/Q-02 unverändert. Frühere
Übergabeschritte folgen.

Neuester Nachweis 16. November: Frisches ETW exakt des gesicherten v2-Paars bestanden,
einschließlich automatischer 28-Start-Serie und beider manueller Wiederanlaufpfade. Drei
Controllerbäume mit 32 Apps/168 WebViews, alle 238 Lebenszeiten explizit beendet; vier Kontrollen,
Nullverluste, 0 Nicht-Loopback- und 176 getrennte Loopback-Ereignisse. Daten/Cleanup unabhängig
geprüft. [Nachweis](testing/native-assistive-restart-etw.md),
[Aggregate/Hashes](testing/native-assistive-restart-etw.json). Programme unverändert,
Erfassung/Prozesse beendet, Rohdaten lokal. Nächster Scope: Controller-Crash und dauerhafter
authentisierter Broker, danach kombinierter Live-/Neustartpfad und externe Grenzen. Keine
Produktfreigabe; striktes Ziel unverändert.

Neuester Stand 16. November: Assistive Wiederanlaufprüfung mit Windows-Sprachausgabe für
Unterbrechung/Abschluss/Replay bestanden; zusätzliche Tastaturnutzung menschlich bestätigt, mit
begrenzter technischer Eingabequellen-Zuordnung.
[Startticket v2](data-format/native-ui-restart-v2.md) mit `automatic: false` lässt nur
Wiederherstellung manuell offen, ohne neue Rechte/Importquelle.
[Nachweis, Wiederholung und Hashes](testing/native-assistive-restart.md). 28 automatische v2-Starts
und lokale Qualitätsprüfungen bestanden. Neues Paar unter
`%TEMP%\kraqel-kq069-assistive-restart-build-20261116-01` gesichert; Hashes vor Verwendung prüfen.
Nächster Scope: frisches ETW genau dieses Paars. Der alte v1-Netzbeleg gilt separat; Controller-
Crash, dauerhafter Broker, externe Grenzen und Produktabnahme bleiben offen. Striktes Ziel gilt.

Neuester Nachweis 16. November: Frischer gültiger ETW-Netzvergleich exakt des gesicherten
Neustart-Paars bestanden. Ein Controllerbaum mit 28 Apps, 144 WebViews und insgesamt 202 explizit
beendeten Lebenszeiten; vier Kontrollen, Nullverluste, 0 Ereignisse außerhalb Loopback und 144
getrennte Loopback-Ereignisse. Auch unter ETW sind Crash/Reconciliation/Altimportabwehr sowie
SQLite/Unicode/Kopienzahl/Cleanup unabhängig bestätigt.
[Nachweis](testing/native-ui-restart-etw.md),
[Aggregate und Hashes](testing/native-ui-restart-etw.json). Rohdaten lokal, Erfassung beendet,
Programme unverändert. Nächster Scope: assistive Wiederanlaufprüfung, Controller-Crash/dauerhafter
Broker und externe Grenzen. Keine Produktfreigabe; striktes Ziel unverändert.

Neuester Stand 16. November: Integrierte isolierte GUI-Crash-/Neustartserie bestanden, 28
Generationen desselben App-/Controllerpaars. Gleicher lebender Controller, OS-Identität und
Datenwurzel; frische Pipes/Launch-Bindungen je Start. Sechs Crashpunkte, je zwei Reconciliations und
Altimportabwehr; vier ungültige Starts verworfen. Genau zwei Kopien, vier unterbrochene Aufträge,
keine Zwischenreste, alle Jobs leer und Profil entfernt.
[Vertrag](data-format/native-ui-restart-v1.md), [Nachweis und Hashes](testing/native-ui-restart.md),
[Aggregate](testing/native-ui-restart.json). Nächster Scope: neuer ETW-Netzvergleich mit exakt dem
gesicherten Paar, danach assistive Wiederanlaufprüfung. Controllerabsturz/dauerhafter Broker und
Produktintegration weiterhin offen; striktes Datenschutzziel unverändert. Frühere Stände folgen zur
historischen Einordnung.

Neueste Fortschreibung 16. November: Dauerhafter Auftragsverbrauch und konservative
Crash-Reconciliation im Forschungsadapter, vor der nativen Auswahl im Tauri-Forschungsfeature
angebunden. Sieben neue Windows-Integrationstests mit sieben echten Prozess-Kills bestehen;
Bestandsdaten bleiben bytegleich, Replay bleibt gesperrt. Native Erfolg-/Abbruchrückprüfung mit
Replay, Journal und Cleanup bestätigt. [Vertrag](data-format/native-task-journal-v1.md),
[Buildbindung und Nachweis](testing/native-task-reconciliation.md). Nächster Scope: authentisierter
Neustart derselben isolierten GUI mit stabiler Ablage und integrierte Crash-Serie; Controller
verlangen bisher frische Forschungswurzeln. Kein neuer ETW-Beleg, keine Produktfreigabe.

Neueste Fortschreibung 16. November: Assistive Erfolg-/Fehlerpfade mit Replay, passenden Ansagen und
Fokus nativ bestätigt. 34 Browser-E2E bestanden; zusätzliche begrenzte Forschungsdiagnose enthält
ausschließlich feste Zustandswerte.
[Nachweis und Buildhash](testing/native-live-assistive-focus.md). Frühere Fokusabweichungen bleiben
dokumentiert; keine assistive Vollabnahme. Der geänderte App-Build hat keinen neuen ETW-Nachweis.
Nächster Scope: dauerhafter Auftragslebenszyklus und Crash-Reconciliation, anschließend weitere
Broker-/Netzabnahme. Striktes Ziel unverändert.

Neuester funktionaler Nachweis 16. November: Fensterende bei offenem Dateidialog und nach echtem
Teiltransfer bestanden. Eigener Prozess/Thread/Dialogklasse vor Schließnachricht geprüft,
Auswahlthread begrenzt beendet; keine Projektkopien oder Stagingreste. Bytezähler belegen 1.024
empfangene von 11.915 erwarteten Bytes.
[Buildgrenzen und Reproduktion](testing/native-live-window-close.md). Frischer ETW-Vergleich am
unveränderten finalen Build bestanden: Referenzimport/Replay, Dialogende und Teiltransfer in drei
vollständigen Bäumen mit 19 WebView2-Lebenszeiten; vier Kontrollen, Nullverluste, 0 Ereignisse
außerhalb Loopback. [Aggregate](testing/native-live-window-close-etw.json). Forschungsprozesse und
Erfassung beendet; Rohdiagnosen bleiben lokal. Nächste Abnahme: Screenreader/Fokus, externe
Brokergrenzen und dauerhafter Auftragslebenszyklus. Striktes Ziel unverändert, keine
Produktfreigabe. Die folgenden Notizen beschreiben frühere Stufen.

Aktualisierung 16. November: Replay-/Fehlerprüfung und Fensterende am Publish-Haltepunkt bestanden.
Neuer gültiger ETW-Nachweis für drei vollständige Forschungs-App-Bäume: 0 Ereignisse außerhalb
Loopback, vier Kontrollen, Nullverluste. Replay-Anzeigefehler korrigiert.
[Exakter Buildbezug und offene Aufgaben](testing/native-live-file-selection.md). Rohdiagnosen
bleiben lokal, ETW-Sitzung beendet. Nach Messung Typalias zur Clippy-Korrektur ohne neuen
Binärnachweis. Striktes Ziel unverändert; nächste Prüfung: Appende bei offenem Dialog/Transfer,
Screenreader/Fokus und externe Brokergrenzen. Keine Produktfreigabe, #134 bleibt offen.

Fortschreibung 16. November: `ui-live` ergänzt die Auswahl aus dem laufenden isolierten
Forschungsfenster. Native Erfolg-/Dialogabbruchserie und unabhängige SQLite-/Cleanup-Prüfung
bestanden. [Nachweis und verbleibende Schritte](testing/native-live-file-selection.md). Replay,
assistive Fokus-/Fehlerprüfung, Appende bei Auswahl/Transfer und neuer ETW-Lauf bleiben offen. Keine
Produktfreigabe; striktes Datenschutzziel unverändert.

Entwicklungsstand: [PR #140](https://github.com/kolkrabeofdoom/kraqel/pull/140), KQ-069 A /
[Issue #134](https://github.com/kolkrabeofdoom/kraqel/issues/134). PR #139 ist nach grüner CI
zusammengeführt (`fadf0a6`). PR #140 enthält die UI-/Tauri- Testanbindung und den einmaligen
Einweg-Dateiauftragskanal einschließlich finaler Nachweise. Die abschließende Übergabe wird mit
diesem PR nach vollständig grüner CI nach `main` übernommen. Beim Wiederbeginn den tatsächlichen
GitHub-Merge-Status prüfen; danach `main` aktualisieren. Falls der PR noch offen ist, liegt die
gesamte Arbeit auf `test/kq-069-isolated-tauri-channel`.

Bestanden: `pnpm check`, normaler und Forschungs-Tauri-Build, Feature-Clippy, 29 Browser-E2E, 18
Helfer-/Transporttests, ein Tauri-Origin-Test und 32 Python-Diagnosetests. Drei finale native
Sitzungen prüfen Import/Wiederholungsablehnung, Abbruch und direktes Fensterschließen am Haltepunkt.
SQLite/Unicode, neue Projekt-ID, Kopienzahl und Cleanup sind unabhängig bestätigt. Gültiges ETW
vom 15. November, 16:43:07–16:46:11 UTC: drei vollständige App-/WebView-Bäume, vier Kontrollen,
Nullverluste, 0 Ereignisse außerhalb Loopback; Loopback-Disconnects getrennt ausgewiesen.

**Das strikte Datenschutzziel bleibt bestehen.** Normale App-Starts sind weiterhin unisoliert;
Q-01/Q-02 für das Produkt sind nicht bestanden. Nur feste synthetische Vorabfreigaben sind im
Forschungsfeature geprüft. Keine allgemeine Freigabe realer Manuskripte, externer Broker oder
kompromittierter Runtime-Prozesse; #134 bleibt offen.

Nachtrag 16. November: Native Vorab-Dateiauswahl (`ui-select`) und Dialogabbruch sind auf dem
zurückgewechselten Rechner funktional geprüft; der neue Build, Controllerausgabe und unabhängige
SQLite-/Cleanup-Prüfung sind im [Nachweis](testing/native-ui-select-acceptance.md) dokumentiert.
Kein neuer ETW-Nachweis und keine Produktfreigabe. Nächster Scope: Auswahl aus einer laufenden App.

Historischer Ausgangspunkt der Übergabe: native Vorab-Dateiauswahl und deren Abbruch prüfen. Die
erfolgreiche finale UI-Serie verwendete `ui-fixture`; der vorherige unvollständige Auswahldialoglauf
ist kein Abnahmenachweis. Danach Auswahl aus einer laufenden App, assistive Abnahme und dauerhafte
Auftrags-/Replay-/Crash-Reconciliation. Weitere Formate, Git-/Credential-/Providergrenzen,
öffentliche UDP-/IPv6-/DNS-/Broker-Gegenproben und Produkt-/Installerintegration folgen separat.

GitHub zeigt außerdem drei bestehende, noch offene Dependabot-Meldungen:
[`lopdf` · hoch · #8](https://github.com/kolkrabeofdoom/kraqel/security/dependabot/8),
[`glib` · mittel · #7](https://github.com/kolkrabeofdoom/kraqel/security/dependabot/7) und
[`minimatch` · mittel · #6](https://github.com/kolkrabeofdoom/kraqel/security/dependabot/6). Sie
wurden durch die UI-/Isolationsarbeit nicht behoben; Erreichbarkeit und Updates separat prüfen.

Für die Fortsetzung zuerst lesen:

1. `AGENTS.md`, [Entwicklungsstatus](DEVELOPMENT_STATUS.md), [Roadmap](../ROADMAP.md) und #134.
2. [ADR 0034](architecture/decisions/0034-webview-network-boundary-proposal.md) und
   [Bedrohungsmodell](architecture/threat-model.md).
3. [Kanalvertrag](data-format/native-file-channel-v1.md),
   [native UI-/ETW-Nachweise und Buildanleitung](testing/native-tauri-file-channel.md) sowie
   [Toolanleitung](../scripts/runtime-isolation/README.md).

Der Quellcode einschließlich synthetischer `.kraqel`-Fixture ist vollständig versioniert. Builds,
temporäre Testprojekte, WebView-Profile, lokale PID-Journale und rohe ETW/XML-Dateien bleiben auf
diesem Rechner. Auf dem anderen Rechner neue TEMP-Wurzeln, neue Builds/Hashes, neue Journale und bei
Bedarf einen neuen ETW-Lauf erzeugen. Historische JSON-Aggregate sind Nachweise dieses Rechners,
keine lokale Abnahme des anderen Rechners. Alte absolute TEMP-/Workspacepfade nicht übernehmen. Es
gibt keine erforderlichen, nur in einem lokalen `work`-Ordner liegenden Quelldateien.

Hier geprüft: Node.js 26.13.1, pnpm 11.25.0, Rust/Cargo 1.100.0, Windows/WebView2. Maßgeblich für
die Installation bleiben `packageManager`, `engines` und `rust-version` im Repository. Das
Isolationsfeature benötigt Windows; die übrige CI prüft auch Linux. Computer Use war hier mit
`Transport closed` ausgefallen, daher erfolgte die native Fensterbedienung durch den Nutzer.
Verfügbarkeit auf dem anderen Rechner neu prüfen, keine alternative Windows-Automation annehmen.

## Diesen Rechner sauber verlassen

1. Nur auf einem geprüften, sauberen `main` enden:

   ```powershell
   cd D:\kraqel
   git switch main
   git pull --ff-only
   git status --short
   git log -1 --oneline
   ```

2. Wenn zusätzlich Schreibprojekte wechseln sollen, in KRAQEL unter **Versionen** zuerst einen
   Snapshot erzeugen und danach **Projektarchiv exportieren** wählen. Das erzeugte `.kraqel`-Archiv
   an einem privaten, vertrauenswürdigen Ort ablegen. Manuskripte und API-Schlüssel gehören nicht
   ins öffentliche Repository.
3. Optional die Übertragung mit `Get-FileHash <Archiv.kraqel> -Algorithm SHA256` auf beiden Rechnern
   vergleichen.

## Auf dem anderen Rechner beginnen

Für einen neuen Checkout:

```powershell
git clone https://github.com/kolkrabeofdoom/kraqel.git D:\kraqel
cd D:\kraqel
corepack enable
pnpm install --frozen-lockfile
git status --short
pnpm --filter @kraqel/desktop typecheck
```

Für einen vorhandenen Checkout:

```powershell
cd D:\kraqel
git switch main
git pull --ff-only
pnpm install --frozen-lockfile
git status --short
```

Benötigt werden Git, Node.js 22 oder neuer, Corepack/pnpm gemäß `packageManager`, Rust 1.90 oder
neuer sowie unter Windows WebView2. Danach `AGENTS.md`, `docs/DEVELOPMENT_STATUS.md`, das nächste
GitHub-Issue und den zugehörigen ADR lesen.

Das Schreibprojekt wird auf der Startseite über **Projektarchiv importieren** als neue lokale Kopie
eingelesen. Die laufende Arbeitsdatenbank liegt unter `%LOCALAPPDATA%\org.kraqel.desktop\projects`
und wird bewusst nicht von Git synchronisiert. Cloud-API-Schlüssel liegen im
Betriebssystem-Schlüsselspeicher und müssen auf dem zweiten Rechner erneut eingerichtet werden.

## Verbindliches Arbeitsmuster

Ein Issue pro klarer Änderung, ein kurzer Branch, ein Pull Request mit grüner CI, danach Merge nach
`main`. Am Sitzungsende: Statusdokumente aktualisieren, offene manuelle Prüfungen ausdrücklich als
offen markieren, pushen und `git status --short` auf leer prüfen. So bleibt die Entwicklung
ortsunabhängig, ohne eine mystische Festplatte zum einzigen Wahrheitsministerium zu ernennen.
