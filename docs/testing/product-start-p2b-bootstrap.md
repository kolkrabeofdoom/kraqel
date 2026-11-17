# Produktstart P2b: Synthetischer Bootstrap ohne GUI oder Importtask

Stand: 17. November 2026. Begrenzte Forschungsimplementierung auf Basis von `c050642`.
[Bereinigte Aggregate](product-start-p2b-bootstrap.json),
[Integrationsentwurf](../data-format/product-start-integration-v1-draft.md). **Keine
Produktfreigabe.** Das strikte Datenschutzziel und Produkt-Q-01/Q-02 bleiben unverändert.

## Umsetzung

Der neue pfadbegrenzte `product-workspace-probe-child` verwendet ausschließlich frische synthetische
TEMP-Controller-/Case-/Store-Wurzeln. Der normale Appstart und `broker-store` sind unverändert.
Unter der exklusiven Kontrollsperre werden Schema-5-Records vor dem Start geschrieben:

1. `commitWorkspaceProbe`: nächster unverbrauchter Launch, Arbeitsablagenbindung sowie separate
   Execution-Bindung an neue Probe-ID, Nonce, Workerhash, kanonischen Root und Fixturehashes.
2. `workspaceProbeProfileReady`: bestätigt die erfolgreiche native Erzeugung des eigenen OS-Profils.
   Synchronisiert den Modellkopf, ohne Generation/Launch zu erneuern; drei Recordslots werden vor
   Ressourcenanlage reserviert, damit dieser Zwischenschritt den Abschluss nicht verbraucht.
3. `finishWorkspaceProbe` mit Receipt-Digest oder `reconcileWorkspaceProbe` bei kontrollierter
   Ablehnung: erst nach belegtem Jobleerstand und Profilentfernung. MAC-Replay rekonstruiert diese
   Zustände; alte P2a-Reconciliation kann einen noch offenen Probeauftrag nicht überspringen.

Die private P2a-Arbeitsablage und ihre logische Profil-ID bleiben erhalten. **Kein ACL-Grant auf
`store/`, `control/`, private Pakete oder Arbeitsdaten.** Jeder Probeauftrag erhält stattdessen eine
neue separate synthetische Wurzel mit Worker-Kopie und ausschließlich lesbarer Policy/Unicode-Datei
sowie ein neues AppContainer-Profil. Die Statusfelder im verschachtelten `workspace` beschreiben die
P2a-Fixture; deren `profileCreated=false` bezieht sich auf das reservierte Workspaceprofil, nicht
auf das separate kurzlebige Probeprofil. Die UI-Fixture wird weiterhin nur als Datei kopiert.

[`workspace_probe.rs`](../../scripts/runtime-isolation/probe/src/windows/exchange/ui_restart/durable/product/workspace_probe.rs)
und
[`bootstrap.rs`](../../scripts/runtime-isolation/probe/src/windows/exchange/ui_restart/durable/product/workspace_probe/bootstrap.rs)
halten die neue Grenze getrennt vom bisherigen Import-/GUI-Protokoll. `.kqboot` enthält 32 read-only
Bytes: `KRAQEL-P2B-PROBE`, Metadatenversion 1, Schema 5, Probe-Bootstrap-ABI 1, UI-Bootstrap-ABI 0,
normale eigene Profilbereinigung ABI 1, reserviert 0. `.kqprod` und die
Legacy-Schema-1–4-Deklaration `.kqmeta` bleiben unverändert. Dies ist keine Herausgeberattestierung
und keine Deklaration für eine startfähige Schreiboberfläche.

## Native Ausführungsgrenze

Der Broker prüft die gepinnten Paketbytes und startet den Worker suspendiert und ohne Konsole
(`DETACHED_PROCESS`). Der erste Versuch mit unsichtbarer Konsole erzeugte einen zusätzlichen
Windows-Konsolenprozess; nur der unten genannte finale Build gehört zum angenommenen Nachweis. Vor
`ResumeThread`: exakte AppContainer-SID, null Capabilities, Mitgliedschaft im eigenen benannten
Kill-on-close-Job, sechs tatsächliche Zugriffsverweigerungen auf private Kontrolldateien sowie
gehaltene Kontrollsperre und genau ein aktiver Jobprozess. Fremde Jobobjekte werden nicht adoptiert.

Explizite Handleliste mit genau drei anonymen Pipeenden: Bootstrap lesen, Einmalschlüssel lesen,
Receipt schreiben. Frischer 32-Byte-Schlüssel in `Zeroizing`, niemals der permanente
Kontrollschlüssel; keine Schlüssel in Argumenten, Umgebung oder Aufzeichnungen. Getrennte
HMAC-SHA256-Domänen `KRAQEL-P2B-BOOT\0\x01` und `KRAQEL-P2B-RECEIPT\0\x01`; Längenfeld, MAC und
maximal 8192 JSON-Bytes, keine Fremdfelder oder nachgestellten Bytes. 16-KiB-Pipes verhindern
Blockieren beim vorab geschriebenen Bootstrap. Die Receipt wird mit `PeekNamedPipe` unter monotoner
30-Sekunden-Grenze gelesen; fehlendes EOF eines zurückgebliebenen Writers kann kein unbeschränktes
Warten auslösen. Zusätzlich maximal zehn Sekunden auf das Root-Prozessende und fünf Sekunden auf den
Jobleerstand.

Der Worker prüft seine Beschränkung vor dem Lesen der Handleargumente, die kernelgemeldete
Server-PID beider Eingabepipes gegen den authentisierten Broker-PID-Wert, Launch/Nonce/Zweck, eigene
Profilidentität/Jobbindung, Root/Paket und synthetische Dateien. Die Receipt bindet Launch, Nonce,
Workerhash und erfolgreiche Fixtureprüfung. Der Broker prüft diese vollständig und hält seinen
Schlüssel bis dahin nur lokal. Die frühen Workerargumente sind allein keine Autorität; der
unveränderte generische Netzwerkdiagnosepfad wird aus dieser Rolle nicht betreten.

Auch bei Ablehnung: Root-Prozess beenden, Job explizit beenden und am noch gehaltenen Objekt
`ActiveProcesses=0` abfragen. Erst dann Profil entfernen und terminalen Record schreiben. Scheitert
die Drainprüfung, bleibt der Auftrag pending. Crash vor/nach Profilerzeugung, ProfileReady oder
Profilentfernung wird **noch nicht automatisch reconciliert**. Es gibt bewusst keinen
Probe-Reconcile-CLI zum Löschen unklarer Ressourcen; diese Crash-/Ressourcenlücken bleiben gesperrt
und sind als eigener nächster Implementierungsschritt zu behandeln. Keine Adoption unbekannter
Profile, kein Reset von Schlüsseln/Records oder Ersatzimport.

## Gezielte Prüfung

Vier neue Tests: Profil-Zwischenrecord erhält Abschlussbudget/Launchidentität; Frame-MAC-Domänen,
Manipulation/Fremdfelder/Trailingbytes und Pipefrist/EOF; authentisierte Rekonstruktion mit
ProfileReady/Receipt und Replayabwehr; Unterbrechung mit erhaltener Launchhistorie. Insgesamt 71
aktive Helfer-/Transporttests bestanden, vier bewusst ignorierte interaktive/Subprozess-Helfer;
Clippy mit Warnungen als Fehler und rustfmt grün. Nach Änderungen am Receiptlesen wurden die drei
betroffenen Probetests gezielt erneut ausgeführt. Mittleres Prüfniveau; kein `pnpm check`, Appbuild,
gemeinsamer Transport-, Installer- oder normaler Produktcode geändert. Mehrere Entwicklungsbuilds
wurden nach Abschluss-/Konsolen-/Fristkorrekturen verworfen; Nachweis nur für den finalen Hash.

Drei native Fälle mit je zwei Launches:

| Fall                        | Erster Launch                                                   | Zweiter Launch                          |
| --------------------------- | --------------------------------------------------------------- | --------------------------------------- |
| Normal                      | Authentisierte Fixture-Receipt, Exit 0                          | Erneut erfolgreich mit neuer Generation |
| Ohne Jobzuweisung           | Vor Resume abgewiesen und suspendiert beendet                   | Erfolgreich                             |
| Manipuliertes Bootstrap-MAC | Eingeschränkter Worker beendet mit 2, kontrolliert reconciliert | Erfolgreich                             |

Jeweils exaktes Replay und veralteter Kopf ohne Recordänderung abgewiesen. 27 Kontrollprozesse (19
Erfolge, acht erwartete Abweisungen), sechs Worker angelegt, davon fünf fortgesetzt: vier Erfolge,
eine Bootstrapablehnung; einer vor Resume beendet. 30 native private Zugriffssperren, sechs
Probeprofile erzeugt/entfernt. Null UI-Starts, Importtasks, Projektkopien oder normale
Appdatenzugriffe. Unabhängiger lesender Audit bestätigt MAC-Ketten/Köpfe, Actor/Root/Paketsections,
Generationen/Verbrauch, drei vollständige 12-Record-Verläufe mit Kopf 11, unveränderte Schlüssel/
private Policy/Unicode-Fixtures, Jobleerstand, beendete Prozesse und entfernte
Profilregistrierungen. Echte Vorschauablage weiterhin mit letztem Dateischreibzeitpunkt vom 16.
November, keine neue Brokerautorisierung dort. Rohpfade/PIDs/IDs/Schlüssel/Records bleiben lokal.

## Reproduktion und gesicherter Build

`scripts/runtime-isolation/check-workspace-bootstrap.ps1` verlangt explizite Programmhashes und
erzeugt selbst frische synthetische Wurzeln. Nur unter Windows und mit bewusst freigegebenem
Forschungsprogramm aufrufen; niemals vorhandene private Storewurzeln einsetzen.

- Controller und kopierter Worker:
  `590c3d4261bfc5f261a38e63d0b974972b9f8e8e7fb7c671e7065eb8ce75364b`.
- Nur als Datei verwendete UI-Fixture:
  `6e56a51ed2e913a878a7935e039bd69628a49be25f0fbab7d02fce3a72fe8c70`.
- Lokale Sicherung: `%TEMP%/kraqel-product-start-p2b-build-20261117-05`; `functional-root.txt`,
  `build-manifest.json` und privater unabhängiger Audit. Öffentliche Aggregate enthalten keine
  Kontoschlüssel, absoluten Wurzeln, PIDs, Nonces oder Lauf-IDs.

## Offene Freigabegates

Inzwischen liegt der [eigene frische ETW-Vergleich](product-start-p2b-bootstrap-etw.md) für den
exakt unten beschriebenen finalen Runtimehash vor. Zwei gültige Phasen, 31 Controller/sechs Worker,
null beobachtete Netzereignisse. Die folgende Formulierung beschreibt das vorherige Gate nach der
Implementierung; nächster aktueller Schritt ist die sichere Ressourcen-Crash-Reconciliation.

Nächster begrenzter Schritt: **gezielter frischer ETW-Netzvergleich des finalen Controllers und
seines separat gestarteten Workerpfads**, einschließlich der beiden Ablehnungen; keine allgemeine
GUI-/Importmatrix. Kein eigener Netzbeleg für diesen Build vorhanden. Alte ETW-Belege für
`88f861a…`/`e4791d2…` gelten ausschließlich für deren damalige Programme und Pfade.

Danach P2-Ressourcen-Crash-Reconciliation und eine ausdrücklich gebundene persistente
containerzugängliche Arbeitsablage/echtes UI-Paket entwickeln. Der vorliegende Read-only-Worker
beweist weder Schreibpersistenz noch GUI-Wiederanlauf oder normale Produktstartautorität. P3
Bestandskopie, P4/P5 normale App-/Launcher-/Installerintegration, langzeitfähige Kontrollhistorie,
externe Dienstgrenzen und Produktabnahme bleiben offen. Keine Netz-/HTTP-/Inhalts-/Uploadfreiheit
aus fehlenden Netzaufrufen im Worker ableiten; das strikte Ziel bleibt eine offene Abnahmebedingung.
