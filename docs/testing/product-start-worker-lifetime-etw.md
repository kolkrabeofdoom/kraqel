# P2b: Frischer ETW-Netzvergleich der Workerbindung und Wiederanlaufpfade

Stand: 17. November 2026. Gezieltes mittleres Prüfniveau, ausschließlich frische synthetische
Ablagen. [Bereinigte Aggregate](product-start-worker-lifetime-etw.json),
[Implementierung und Lebenszeitvertrag](product-start-worker-lifetime.md). **Keine normale
Produktfreigabe.** Striktes Datenschutzziel und Produkt-Q-01/Q-02 unverändert.

## Programme und Messung

Unveränderter gesicherter Runtimebuild aus `03e4ed1`, lokal unter
`%TEMP%/kraqel-product-worker-lifetime-build-20261117-01`:

- Controller und sämtliche separat kopierten Worker:
  `37d7beccd2c8368a9e34f27d6b29d668f25a2816fbd43893bc6abddf885ca081`.
- Nur als Datei verwendete, nicht gestartete UI-Fixture:
  `6e56a51ed2e913a878a7935e039bd69628a49be25f0fbab7d02fce3a72fe8c70`.

Execution v3/Bereinigungs-ABI 3 und Ein-Prozess-Joblimit unverändert. Keine
Runtime-/App-/Transport-/ Paket-/Capabilityänderung und kein neuer Build. Paketmarker, kopierte
Worker und private Daten unabhängig gegen die gesicherten Hashes geprüft; keine neue Brokerfreigabe
an der echten Vorschauablage.

Zwei getrennte kurze Sitzungen: **normaler Workerabschluss als Referenz** und die **vier bereits
funktional geprüften Controller-Crash-/Wiederanlaufpfade**. Unveränderte
[`capture.ps1`](../../scripts/m3-network/capture.ps1) und
[`analyze.py`](../../scripts/m3-network/analyze.py), Modus `native-control`, vollständige private
Root-PID-/EXE-Liste. Controllerhandles bis Konvertierungsende gehalten; keine automatische Rootsuche
oder Lockerung der Kalibrierungs-/Verlust-/Zuordnungsregeln. Lokale Prüfsteuerung entspricht dem
vorhandenen Workerprüfablauf, ergänzt um Referenz und Handlehaltung.

Erhöhte PowerShell 7 nur für ETW; normale Controller-/Workersteuerung ohne erhöhtes Token. Je vier
eigene lokale TCP-/UDP-IPv4/IPv6-Kontrollen aus dem Capturehelfer, höchstens fünf Minuten je
Sitzung. Nach tatsächlichem Prüfende per eigener STOP-Datei beendet, ungefähr 8,1 bzw. 47,0 Sekunden
ab Ready. Beide eigenen Sitzungen gestoppt/konvertiert und Supervisor beendet. Keine Firewall-,
Loopback-, ExecutionPolicy- oder Domänenänderung, keine physische Netztrennung.

## Pfade und Ergebnis

Referenz: neue Kontrollablage, Schema 5/Workspace-Fixturebindung, ein normaler authentisierter
Workerabschluss und Status. Fünf Controller/ein Worker, zwölf vollständige MAC-Records, kein
Pending.

Wiederanlauf: vier frische Fälle mit hartem Controllerende 86 vor WorkerBound, nach WorkerBound/vor
Resume, während des nach Receipt/Alive-Prüfung laufenden Workers und nach Workerende/vor Drained.
Der erste Fall bleibt trotz eigener separater Fixturebereinigung unverändert pending und gesperrt.
Die drei gebundenen Fälle erhalten nach bestätigtem Ende einen RecoveredDrain-Record, eigene
Profilbereinigung und neue erfolgreiche Generation; Stale-/Pending-/Replayabwehr bleibt wirksam.

| Phase                  | Controller | Worker | Controller-Konsolenhosts | Explizite Enden | Kontrollen | Nicht-Loopback | Loopback |
| ---------------------- | ---------: | -----: | -----------------------: | --------------: | ---------: | -------------: | -------: |
| Normaler Abschluss     |          5 |      1 |                        5 |              11 |        4/4 |              0 |        0 |
| Vier Wiederanlaufpfade |         47 |      7 |                       47 |             101 |        4/4 |              0 |        0 |
| Zusammen               |     **52** |  **8** |                   **52** |         **112** |    **8/8** |          **0** |    **0** |

Beide Messungen gültig: null verlorene Events/Buffers, keine Zuordnungslücken oder unbekannten
Ziele. Insgesamt **null beobachtete Netzereignisse in den ausgewählten Bäumen**, einschließlich
Loopback. Alle 52 Konsolenhosts sind direkten Controllern zugeordnet; Worker starten ohne Konsole.
Kein zusätzlicher Worker und keine GUI/WebView im Baum.

## Unabhängiger Gegencheck

Ein separater chronologischer Kernel-Audit gleicht alle erwarteten Controlleraufrufe, Eltern und
Exitcodes ab: 31 Erfolge, 17 erwartete Ablehnungen, vier Abbrüche mit 86. Alle 112 Enden explizit,
keine bloße Ableitung aus später nicht mehr auffindbaren PIDs.

**Alle acht Worker anhand PID und exakter nativer Erzeugungszeit zugeordnet:** Die Kernel-
CreateTime wird ohne Fließkomma-/Mikrosekundenabrundung auf 100-ns-FILETIME verglichen. Sieben
signierte WorkerBound-Bindungen stimmen mit Kernel und privaten Spawn-/Launchbelegen überein. Der
achte, vor WorkerBound abgebrochene Worker ist nur diagnostisch zugeordnet und erhält dadurch keine
Produkt-Cleanupautorität. Für alle drei RecoveredDrain-Records ist das Ende der richtigen
Workergeneration bereits vor dem passenden erfolgreichen Recovery-Controllerstart belegt.

**Kernel-Exitcode 0 ist kein fachlicher Erfolg:** Windows meldet in dieser Erfassung für alle acht
Worker 0, auch für die zwei nie fortgesetzten und den jobbedingt unterbrochenen Worker. Die
fachlichen Zustände folgen unverändert authentisierter Receipt und Kontrollhistorie: vier
Finish-Records, drei unterbrochene/reconciliierte Starts und ein gesperrter ungebundener Start. Ein
verlorener Erfolg wird nicht aus dem Exitcode oder unsignierten Logs erfunden.

Unabhängiger lesender MAC-/Paket-/Root-/Inventar-/Profilaudit: 75 Records, fünf gebundene
Arbeitsablagen, acht verbrauchte Launches, sieben WorkerBound-Bindungen und drei Endrecords. Native
Token-/Job-/Lease-/Identitätsbelege und 48 tatsächliche private Zugriffssperren bestätigt. Acht
OS-Profile am Ende abwesend: sieben Broker-Cleanups, ein ausdrücklich getrennter eigener
Fixturecleanup durch den Prüfablauf. Ein logischer Pending-Store bleibt erwartungsgemäß gesperrt;
Schlüssel/Policy/Unicode und echte Vorschauablage unverändert. Keine GUI, Importtask, Projektkopie
oder normale Datenmigration.

42 bestehende Diagnose-Tests erneut grün; Format-/Hash-/Fakten-/Verweis-/Diffprüfung ergänzt. Kein
neuer Runtimebuild, vollständiger App-/Rust-/Workspace-Lauf oder zusätzliche allgemeine Crashmatrix
bei dieser ETW-Fortschreibung.

## Ablage und Aussagegrenzen

Roh-ETL/XML, private Prozess-/Erzeugungszeiten/Profil-/Launch-IDs, Pfade, Schlüssel/Records und
lokale Audithelfer bleiben außerhalb GitHub. Captures unter
`%TEMP%/kraqel-worker-v3-etw-20261117-01-reference` und `…-recovery`; ergänztes Manifest und
zusammengefasster Audit bei der Buildsicherung. Zur Reproduktion Hash prüfen, Capture mit frischer
Wurzel starten, Ready abwarten, normale Referenz bzw. `check-workspace-worker-crashes.ps1`
ausführen, alle Start-PIDs festhalten/Handles halten, STOP/Complete abwarten und
Aufruf-/Worker-/Generations-/ MAC-Abgleiche ergänzend zur unveränderten Auswertung durchführen.

Prüfsteuerung und deren eigener Fixturecleanup sowie externe COM-/Shell-/DNS-/Dienstpfade liegen
außerhalb der ausgewählten nativen Bäume. Kein universeller Dienst-/HTTP-/Telemetrie-/Inhalts-/
Uploadbeleg und keine physische Netztrennung. Der Nachweis gilt nur für diese Runtimebytes,
Lebenszeiten und ausgeübten Pfade, nicht für die normale Schreiboberfläche oder neue Builds.

**Nächster begrenzter Scope:** den P2c-Vertrag für persistente synthetische Arbeitsablage,
Paketbindung und kontrollierten Mehrprozess-UI-Start konkretisieren. Ein-Prozess-Endbelege dürfen
nicht auf WebView2 übertragen und private P2a-Fixtures nicht per ACL promoviert werden. Ungebundene
Starts/unbestätigte Profile bleiben gesperrt; P2 insgesamt, P3–P5 und normale Produkt-Q-01/Q-02
bleiben offen. Das strikte Datenschutzziel bleibt unverändert.
