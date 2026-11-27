# P2b: Frisches ETW für die begrenzte Ressourcen-Reconciliation

Stand: 17. November 2026. Gezieltes mittleres Prüfniveau, ausschließlich neue synthetische Ablagen.
[Bereinigte Aggregate](product-start-resource-reconciliation-etw.json),
[funktionale Implementierung und offene Grenzen](product-start-resource-reconciliation.md). Striktes
Datenschutzziel unverändert; normale Produktintegration und Produkt-Q-01/Q-02 offen.

## Gesicherte Programme und Vergleich

Unveränderter Runtimebuild aus `1d24762`, lokal gesichert unter
`%TEMP%/kraqel-product-resource-crashes-build-20261117-01`:

- Controller und jede separat kopierte `worker.exe`:
  `902187213995a3cbf590328f80311a701be19716948a7a45423223581d7994dc`.
- Nur als Datei verwendete, nicht gestartete UI-Fixture:
  `6e56a51ed2e913a878a7935e039bd69628a49be25f0fbab7d02fce3a72fe8c70`.

Keine Runtime-, App-, Transport-, Paket- oder Netzcapabilityänderung und kein neuer Build. Execution
v2/Profilbereinigungs-ABI 2, Schema 5 und UI-ABI 0 unverändert. Paketmarker und kopierte Workerbytes
im Daten-/Paket-Audit unabhängig geprüft. Echte Vorschau-/Projektdaten nicht verwendet oder für neue
Brokerprogramme autorisiert.

Zwei kurze getrennte Messungen: **normaler Workerabschluss als Referenz** gegenüber den **vier
bereits funktional geprüften Ressourcenabbrüchen samt Reconciliation**. Unveränderte
[`capture.ps1`](../../scripts/m3-network/capture.ps1) und
[`analyze.py`](../../scripts/m3-network/analyze.py), Modus `native-control` mit vollständiger
privater Root-PID-/EXE-Liste. Prozesshandles bleiben bis Konvertierungsende gehalten. Die lokale
Prüfsteuerung verwendet den bestehenden Ressourcenprüfablauf mit einem normalen Referenzfall und
dieser Handlehaltung; keine Lockerung der Auswertung oder zusätzliche allgemeine Crashmatrix.

Erhöhte PowerShell 7 ausschließlich für ETW, normale Controller-/Workersteuerung ohne erhöhtes
Token. Je vier eigene lokale TCP-/UDP-IPv4/IPv6-Kontrollen aus dem Capturehelfer. Höchstens fünf
Minuten je Sitzung, nach Prüfende eigene STOP-Datei; ungefähr 8,1 bzw. 47,1 Sekunden von Ready bis
Traceende. Beide eigenen Sitzungen gestoppt/konvertiert und Supervisor beendet. Keine Firewall-,
Loopback-, ExecutionPolicy- oder Domänenänderung und keine physische Netztrennung.

## Tatsächlich ausgeübte Pfade

Referenz: neue private Kontrollablage, Schema-5-Migration und Workspace-Fixturebindung, ein normaler
authentisierter Workerabschluss, danach Status. Fünf Controller, ein Worker, elf MAC-Records mit
Kopf 10 und eine verbrauchte Launch-ID. Profil regulär entfernt, kein Pending, keine GUI/Importtask.

Reconciliation: vier neue Fälle mit hartem Abbruch 86 nach Commit/vor Profilerzeugung, nach
Profilerzeugung/vor ProfileReady, nach ProfileReady/vor RunIntent sowie nach Profilentfernung/vor
Abschluss. Jeweils Wiederöffnen, veralteter Kopf, gesperrter Pending-Neustart, Reconciliation,
erneute Reconciliation ohne Pending, Replay und neue erfolgreiche Generation.

Der Broker verweigert weiterhin die Löschung des unbestätigten bestehenden Profils. Ausschließlich
der Prüfablauf entfernt danach seine unmittelbar zuvor selbst erzeugte synthetische Fixture; dies
ist keine Produkt-Löschautorität. Anschließend bestätigt der Broker nur die Abwesenheit. Der übrige
Cleanup bleibt durch ProfileReady sowie fehlendes RunIntent bzw. belegtes Drained begrenzt. Keine
Adoption unklarer Ressourcen oder Wiederholung verbrauchter Launches.

## Ergebnis und unabhängige Gegenprüfungen

| Phase                    | Controller | Worker | Controller-Konsolenhosts | Explizite Enden | Kontrollen | Nicht-Loopback | Loopback |
| ------------------------ | ---------: | -----: | -----------------------: | --------------: | ---------: | -------------: | -------: |
| Normaler Workerabschluss |          5 |      1 |                        5 |              11 |        4/4 |              0 |        0 |
| Vier Ressourcenabbrüche  |         49 |      5 |                       49 |             103 |        4/4 |              0 |        0 |
| Zusammen                 |     **54** |  **6** |                   **54** |         **114** |    **8/8** |          **0** |    **0** |

Beide Messungen gültig: null verlorene Events/Buffers, keine unzuordenbaren Netzereignisse oder
unbekannten Ziele. Insgesamt **null Netzereignisse in den ausgewählten Bäumen beobachtet**. Die
unsichtbar gestarteten Controller haben jeweils einen mitgemessenen Konsolenhost. Alle 54 sind
direkt passenden Controllern zugeordnet; die sechs Worker starten ohne eigene Konsole.

Separater chronologischer Kernel-Audit: alle 54 erwarteten Controllerstarts und Exitcodes stimmen
mit dem nativen Aufrufprotokoll überein. Zusammen 33 Exit 0, 17 erwartete Ablehnungen und vier
Abbrüche mit 86. Die sechs Worker haben Exit 0, passende Controllereltern und private Spawn-/
Launchbelege. Ein erster Worker wird nur im Fall nach Profilentfernung erwartet; die drei früheren
Crashpunkte starten keinen Worker. Jeder tatsächlich erwartete Worker ist erfasst, kein zusätzlicher
Worker oder fehlender Prozess wird durch bloßes Akzeptieren eines ruhigen Baums übersprungen. Alle
114 Enden explizit, Zuordnung an Prozessgenerationen gebunden.

Unabhängiger lesender Daten-/MAC-/Root-/Paket-Audit: reguläre Referenz mit elf Records und die vier
Reconciliationfälle mit 13/13/14/16 Records; zusammen 67 Records, fünf gebundene Arbeitsablagen und
neun verbrauchte Launches. Keys/Policy/Unicode-Fixtures erhalten, keine Pending-Ressourcen am Ende.
Sechs native Token-/Job-/Lease-/Receipt-/Drainbelege mit 36 privaten Dateisperren. Acht OS-Profile
erzeugt, sieben Broker-Cleanups und eine ausdrücklich getrennte eigene Fixturebereinigung durch den
Prüfablauf. Alle Profile am Ende abwesend; echte Vorschauablage unverändert. Keine GUI, Importtask,
Projektkopie oder normale Nutzdatenmigration.

42 bestehende Diagnose-Tests erneut grün. Kein erneuter vollständiger Rust-/App-/Workspace-Lauf oder
Build bei unverändertem Runtimecode; Format-, Fakten-, Hash-, Verweis- und Diffprüfung ergänzt.

## Lokale Ablage, Grenzen und nächster Schritt

Inzwischen ergänzt ein
[neuer v3-Runtimebuild dauerhafte Workerbindung](product-start-worker-lifetime.md). Diese Messung
bleibt ausschließlich an `9021872…` gebunden; kein Übertrag auf neue Bytes. Der folgende nächste
Scope beschreibt den damaligen Stand.

Roh-ETL/XML, Root-PIDs/-pfade, Profil-/Launch-IDs, Schlüssel/Records und lokale Audithelfer bleiben
außerhalb GitHub. Capturepfade: `%TEMP%/kraqel-resource-v2-etw-20261117-01-reference` und
`…-reconciliation`. Buildmanifest und zusammengefasster Audit bei der oben genannten Buildsicherung.
Zur Reproduktion Hash prüfen, Capture mit neuer Wurzel starten, Ready abwarten, normale Referenz
bzw. `check-workspace-resource-crashes.ps1` mit expliziten Hashes ausführen, alle Start-PIDs
aufnehmen/ Handles halten und nach STOP/Complete mit vollständigem Aufruf-/Worker-/MAC-Abgleich
auswerten.

Die PowerShell-Prüfsteuerung, ihr separater Fixturecleanup, der Capturehelfer und externe Dienste
liegen außerhalb der ausdrücklich ausgewählten nativen Bäume. Kein vollständiger COM-/Shell-/
DNS-/Dienstbeleg; ETW-Transportmetadaten attestieren keine HTTP-/Telemetrie-/Manuskript-/Upload-
Inhalte. Der Nachweis gilt nur für die genannten Bytes, Lebenszeiten und ausgeübten Pfade,
insbesondere nicht für die normale Schreiboberfläche oder unbekannte Crashreste. Produkt-Q-01/Q-02
bleiben offen, das strikte Datenschutzziel bleibt bestehen.

**Nächster begrenzter Scope:** dauerhaft belegter Workerstart und gesicherter Wiederanlauf nach
Controller-Crash zwischen RunIntent und Drained. Solange dessen Lebenszeit nicht nachgewiesen werden
kann, bleibt der Auftrag gesperrt; keine Cleanupfreigabe aus verschwundenem Jobnamen oder
unsignierten Diagnosefiles. Persistente echte Arbeitsablagen-/UI-Bindung, P2 insgesamt und P3–P5
bleiben weiterhin offen.
