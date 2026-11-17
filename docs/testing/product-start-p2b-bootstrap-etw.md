# P2b: Frischer ETW-Netzvergleich für Controller und Prüfworker

Stand: 17. November 2026. Gezieltes mittleres Prüfniveau, nur synthetische TEMP-Ablagen.
[Bereinigte Aggregate](product-start-p2b-bootstrap-etw.json),
[funktionaler P2b-Stand](product-start-p2b-bootstrap.md),
[Produktstartentwurf](../data-format/product-start-integration-v1-draft.md). Striktes
Datenschutzziel unverändert; **normale Produktintegration und Produkt-Q-01/Q-02 bleiben offen**.

## Programme und Messgrenze

Gemessen wird der unveränderte gesicherte Runtimebuild aus `4c26ba4`, lokal unter
`%TEMP%/kraqel-product-start-p2b-build-20261117-05`:

- Controller und alle separat kopierten Worker:
  `590c3d4261bfc5f261a38e63d0b974972b9f8e8e7fb7c671e7065eb8ce75364b`.
- Nur als Datei kopierte, nicht gestartete UI-Fixture:
  `6e56a51ed2e913a878a7935e039bd69628a49be25f0fbab7d02fce3a72fe8c70`.

Eine hashidentische Controllerkopie heißt für die vorhandene explizite Root-Zuordnung
`etw-package/kraqel-isolation-probe.exe`. Die sechs Worker liegen je in einer frischen eigenen
synthetischen Wurzel als `bin/worker.exe`; Hash, PE-Marker, Root/Nonce/Launch und zugehöriger
Kernelstart werden separat geprüft. Keine Neuautorisierung an der echten Vorschauablage. Kein neuer
Runtimebuild, keine Änderung an Capture/Auswerter, Netzcapabilities, App, Transport oder Installer.
Schema-/Record-/Probe-ABI und UI-ABI 0 bleiben unverändert.

Zwei kurze separate ETW-Sitzungen mit unverändertem
[`capture.ps1`](../../scripts/m3-network/capture.ps1), erhöhter PowerShell 7 ausschließlich für die
Erfassung, Controller/Workersteuerung weiter ohne erhöhtes Token. Je vier eigene lokale
TCP-/UDP-IPv4/IPv6-Kontrollen aus dem Capturehelfer. Höchstens fünf Minuten je Sitzung, nach
tatsächlichem Prüfende per eigener STOP-Datei beendet; Zeit ab Ready bis Traceende ungefähr 5,6 bzw.
26,4 Sekunden. Keine physische Netztrennung, Firewall-, Loopback-, ExecutionPolicy- oder
Domänenänderung. Beide eigenen Sitzungen regulär gestoppt und konvertiert, Supervisor beendet.

Unverändertes [`analyze.py`](../../scripts/m3-network/analyze.py) im Modus `native-control` mit
vollständiger privater PID-/EXE-Zuordnung aller direkten Starts. Die lokale Messsteuerung verwendet
den vorhandenen Bootstrap-Prüfablauf; ausschließlich Referenzpfad und gehaltene Controllerhandles
bis Konvertierungsende ergänzt. Keine automatische Rootsuche, Lockerung von Verlust-/Kalibrierungs-
oder Zuordnungsprüfungen. PowerShell-Steuerung/Capturehelfer liegen außerhalb der ausgewählten
nativen Prozessbäume.

## Ausgeübte Pfade

Referenz: eine neue private synthetische Kontrollablage initialisiert, ausdrücklich auf Schema 5
migriert, private Workspace-/UI-Fixture gebunden und Status gelesen. Vier direkte Controller, sechs
authentisierte Records, kein Launch/Worker/Profil oder Importtask. MAC-Kette/Kopf separat geprüft,
keine echte Arbeitsablage oder Bestandsübernahme.

Workerphase: drei neue synthetische Fälle mit je zwei Launches gemäß funktionalem Nachweis: normaler
Receiptabschluss; fehlende Jobzuweisung vor Resume blockiert; manipuliertes Bootstrap-MAC im
eingeschränkten Worker abgelehnt. Jeweils danach neuer erfolgreicher Launch; Replay und veralteter
Kopf ohne Kontrolländerung abgewiesen. 27 direkte Controller und sechs Worker: vier Exit 0, ein
suspendierter Worker vor Ausführung mit 1 beendet, eine Bootstrapablehnung mit 2. Controller: 19
Exit 0 und acht erwartete Exit 1. Keine GUI und keine Importtask/Projektkopie.

## Messergebnis und Gegenprüfungen

| Phase                     | Controller | Prüfworker | Controller-Konsolenhosts | Explizite Enden | Positivkontrollen | Nicht-Loopback | Loopback |
| ------------------------- | ---------: | ---------: | -----------------------: | --------------: | ----------------: | -------------: | -------: |
| Kontrollablagenreferenz   |          4 |          0 |                        4 |               8 |               4/4 |              0 |        0 |
| Drei gezielte Workerfälle |         27 |          6 |                       27 |              60 |               4/4 |              0 |        0 |
| Zusammen                  |     **31** |      **6** |                   **31** |          **68** |           **8/8** |          **0** |    **0** |

Beide Phasen gültig: null verlorene Events/Buffers, keine unzuordenbaren Netzereignisse oder
unbekannten Ziele. In den gemessenen Bäumen wurden insgesamt **null Netzereignisse beobachtet**. Die
Controller als Konsolenprogramme erzeugen beim unsichtbaren Teststart jeweils einen Konsolenhost;
diese sind vollständig mitgemessen. Die per `DETACHED_PROCESS` gestarteten Worker erzeugen keinen
zusätzlichen Konsolenhost. Alle 31 Konsolenhosts sind ausdrücklich direkten Controllern zugeordnet.

Ein separater chronologischer Kernel-Audit rekonstruiert Prozessgenerationen samt Eltern und
Enden/Exitcodes. Jeder der 31 Controller stimmt mit dem nativen Aufrufprotokoll überein. Jeder der
sechs Worker stimmt mit der privaten `spawn.json`, dem passenden start-/Fehleraufruf und dem
authentisierten Execution-Record überein. Kein Worker fehlt allein deshalb, weil er nie fortgesetzt
wurde. Alle 68 Prozessenden explizit; keine Ableitung aus später möglicherweise neu vergebenen PIDs.

Unabhängiger lesender Daten-/Paket-/MAC-Audit während der Auswertung: drei vollständige
12-Record-Workerverläufe mit Kopf 11, je zwei verbrauchte Launches und kein Pending am Ende.
Arbeitsablagen-/Schlüssel-/Policy-/Unicode-Dateien erhalten, Root-/Actor-/PE-Deklarationen korrekt.
Fünf vor Resume kontrollierte Worker liefern 30 tatsächliche private Dateisperren samt exaktem
AppContainer-Token/null Capabilities, Job- und Leasebeleg; der sechste wird bereits an der fehlenden
Jobbindung gestoppt. Sechs belegte Jobleerstände am gehaltenen Objekt und sechs entfernte
Probeprofilregistrierungen. Echte Vorschauablage mit unverändertem letztem Dateischreibzeitpunkt,
keine neue Brokerfreigabe dort.

42 bestehende Diagnose-Tests erneut grün. Zusätzlich wurde beim CI-Abgleich ein Linux-Clippy-Befund
im reinen Modelltest korrigiert: Der Test prüft den bereits vorhandenen `used_launch()`-Accessor
vor/nach Commit statt nur das private Set. Acht gezielte Modelltests und Helfer-Clippy grün. Diese
Änderung betrifft ausschließlich `product_start/tests.rs`, nicht den gemessenen Runtimecode oder
dessen unveränderte gesicherte EXE. Kein vollständiger App-/Workspace-Neulauf oder neuer Build.

## Ablage, Aussagegrenzen und nächster Schritt

Roh-ETL, XML, Root-PIDs/-pfade, Profile/IDs/Nonces, Kontrollrecords/Schlüssel und lokale Audithelfer
bleiben außerhalb GitHub. Lokale Nachweise unter `%TEMP%/kraqel-p2b-etw-20261117-01-reference` und
`…-worker`, gesicherte Buildmanifest-Ergänzung und zusammengefasster Audit bei der obigen
Buildsicherung. Öffentlich nur bereinigte Aggregate. Zur Reproduktion gesicherte Hashes prüfen,
Capture mit neuem Ziel starten, Ready abwarten, explizite Start-PIDs aufzeichnen/Handles halten,
vorhandenen Bootstrap-Prüfablauf ausführen, STOP/Complete abwarten und vollständige
Aufruf-/Worker-/MAC-Abgleiche ergänzend zu `native-control` ausführen. Keine bereits verwendeten
privaten Wurzeln übernehmen.

Der Nachweis gilt genau für diese Runtimebytes, Prozesslebenszeiten und synthetischen Pfade. Keine
Messung der normalen Schreiboberfläche, persistenter Nutzdatenarbeit, echten Bestandskopie oder
externer COM-/Shell-/DNS-/Dienstpfade außerhalb des Prozessbaums. ETW enthält hier
Transportmetadaten, keine HTTP-/Telemetrie-/Manuskript-/Upload-Inhaltsattestierung. Die echten
Produkt-Q-01/Q-02 werden dadurch nicht geschlossen.

**Nächster begrenzter Schritt:** sichere Ressourcen-Crash-Reconciliation für die P2b-Probeprofile,
besonders die Übergänge Profilerzeugung → ProfileReady und Profilentfernung → Abschlussrecord.
Unklare Reste bleiben bis dahin gesperrt, keine automatische Adoption/Löschung. Danach persistente
echte Arbeitsablagen-/UI-Bindung; P2 insgesamt und P3–P5 bleiben offen. Grobe Einschätzung des
Gesamtfortschritts und fachliche Restblöcke stehen in der [Roadmap](../../ROADMAP.md).
