# P2b: Dauerhafte Workerbindung und Wiederanlauf vor Drained

Stand: 17. November 2026, Quellbasis `707aed6`.
[Bereinigte Aggregate](product-start-worker-lifetime.json),
[vorheriger Ressourcenvertrag](product-start-resource-reconciliation.md),
[dessen eigener Netzbeleg](product-start-resource-reconciliation-etw.md). **Keine Produktfreigabe.**
Striktes Datenschutzziel und Produkt-Q-01/Q-02 unverändert.

## Umfang und dauerhafte Bindung

Execution-Version **3** und `.kqboot`-Profilbereinigungs-ABI **3** kennzeichnen den neuen Pfad.
Schema 5, Probe-Bootstrap-ABI 1/UI-ABI 0 sowie Legacy-`.kqmeta`/P2a-`.kqprod` bleiben unverändert.
Neue synthetische Kontrollablagen, keine Migration alter Execution-v1/v2-Historien oder Freigabe der
echten Vorschauablage für neue Brokerprogramme.

[`lifetime.rs`](../../scripts/runtime-isolation/probe/src/windows/exchange/ui_restart/durable/product/workspace_probe/lifetime.rs)
ergänzt `workspaceProbeWorkerBound` mit Launch/Generation, nativer PID, Erzeugungszeit aus
`GetProcessTimes` und Prozesslimit 1. Das bedeutet **erzeugt und vor Fortsetzung gebunden**, nicht
bereits ausgeführt oder erfolgreich abgeschlossen. Dieser MAC-Record samt Kopf muss vollständig
persistiert sein, bevor `ResumeThread` aufgerufen wird.

Vorher sind die privaten Dateisperren geprüft. Der enge Bindungspfad prüft zusätzlich selbst die
exakte Mitgliedschaft im aus authentisierter Installation abgeleiteten benannten Job, dessen
ausgelesene Limits, erwartete AppContainer-SID/null Capabilities, genau einen aktiven Prozess und
gehaltene Kontrollsperre. PID/Erzeugungszeit stammen vom gehaltenen Prozesshandle, nicht aus CLI,
Renderer, JSON-Vorschlag oder Diagnosefiles.

Normale Storemutationen verlangen weiterhin tatsächlichen Jobleerstand. Ausschließlich diese Bindung
darf vor Resume bei einem aktiven Worker schreiben, nach obigen nativen Guards. Die gemeinsame
private Serialisierung `append_with_checked_context` prüft Actor/Kopf/Budget erneut; sie ersetzt
keine OS-Autoritätsprüfung. Der Record-/Kopf-Schreibweg mit Vorvalidierung und MAC-Rekonstruktion
bleibt erhalten; unvollständige Abschlüsse werden nicht repariert.

**Ein-Prozess-Grenze nur für den synthetischen Readonly-Worker:** Vor Prozesserzeugung werden
`JOB_OBJECT_LIMIT_ACTIVE_PROCESS=1` und Kill-on-close gesetzt, beide Breakaway-Flags bleiben aus.
Vor Bindung werden die echten Limits ausgelesen. So kann die Beendigung dieser gebundenen
Prozessgeneration für diesen Vertrag als Lebenszeitgrenze verwendet werden. Das ist ausdrücklich
kein Modell für die spätere mehrprozessige WebView-/Schreiboberfläche; dafür ist ein eigener Vertrag
nötig.

Normalpfad: sechs reservierte Slots für Commit → ProfileReady → RunIntent → WorkerBound → Drained →
Abschluss. Kontrollierte Fehler ohne erzeugte Bindung behalten ihren eigenen geprüften Jobabschluss;
sie erhalten dadurch keine nachträgliche Startbindung. Der reine Modellkern erhält bei jedem
Zwischenrecord Kopf/Budget korrekt, verbrauchte Launches bleiben erhalten.

## Endbeleg beim Wiederanlauf

Unter derselben Storelease und Actor-/Kopf-/Jobleerstandsprüfung wird bei RunIntent ohne Drained
eine **vorhandene authentisierte WorkerBound-Bindung** benötigt. Andernfalls bleibt der Auftrag
gesperrt, auch wenn sein Profil mittlerweile fehlt oder ein unsigniertes Spawn-/Drainlog vorliegt.
Vorherige unbestätigte Profile bleiben ebenfalls ohne Adoption/Löschung gesperrt.

Der Endbeleg verwendet ausschließlich Abfrage-/Synchronisationsrechte, keine Terminierungsrechte:

- Dieselbe PID mit gleicher Erzeugungszeit muss am geöffneten Prozesshandle tatsächlich signalisiert
  sein. Ein noch laufender Prozess sperrt; es gibt kein unbegrenztes Warten.
- Bei anderer nichtnull Erzeugungszeit ist die gespeicherte Generation beendet. Die neue Generation
  wird weder beendet noch wiederaufgenommen oder als eigener Worker übernommen.
- `OpenProcess` mit `ERROR_INVALID_PARAMETER` reicht allein nicht: ein vollständig gelesener
  aktueller Toolhelp-Prozesssnapshot muss zusätzlich die PID-Abwesenheit bestätigen. Andere Fehler,
  Zugriffsverweigerung oder unvollständige Snapshots bleiben gesperrt. Fremde Prozessnamen aus dem
  Snapshot werden nicht aufgezeichnet oder veröffentlicht.

`workspaceProbeRecoveredDrain` bindet denselben Launch und dieselbe Workeridentität mit einem dieser
Endbelege. Der Record wird vor Profilbereinigung unter erhaltenem Abschlussbudget persistiert.
Danach greift die bestehende eigene Profil-/Monikerprüfung und terminale Reconciliation. Ein
erneuter Crash zwischen Endrecord und Cleanup ist damit wiederholbar. Kein Erfolg wird aus
verlorenen Receipts oder unsignierten Diagnosefiles rekonstruiert: der frühere Launch wird
**unterbrochen**, nicht erneut startbar oder zu einem neuen Importauftrag gemacht.

API-Grundlagen: Microsofts
[Prozesskennungen](https://learn.microsoft.com/en-us/windows/win32/procthread/process-handles-and-identifiers),
[GetProcessTimes](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-getprocesstimes)
und
[Joblimits](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-jobobject_basic_limit_information).
Die dokumentierte OS-Semantik ersetzt weder native Guards noch den folgenden begrenzten Nachweis.

## Gezielte Prüfung

Vier harte native Controllerenden mit 86 ohne Destruktoren:

| Punkt                                                                  | Nach Wiederöffnen                                                                                                        | Ergebnis                                                                                                       |
| ---------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------------------- |
| Vor WorkerBound, nach suspendierter Prozesserzeugung/Token-/Jobprüfung | RunIntent, kein dauerhafter Workerbeleg                                                                                  | Reconciliation und Neustart bleiben gesperrt, auch nach separatem eigenen Fixturecleanup                       |
| Nach WorkerBound, vor Resume                                           | Dauerhafte Bindung, suspendierter Worker durch Controllerende beendet                                                    | Endbeleg gespeichert, eigenes Profil bereinigt, neue Generation erfolgreich                                    |
| Laufender Worker vor Drained                                           | Worker hat Bootstrap/Fixtures geprüft und eine authentisierte Receipt geliefert; native Alive-Prüfung vor Controllerende | Jobende beendet den Worker, Wiederanlauf unterbrochen abgeschlossen, neue Generation erfolgreich               |
| Nach Workerende, vor Drained-Record                                    | Dauerhafte Bindung, keine signierte Drained-Veröffentlichung                                                             | Tatsächliches Ende neu geprüft, Reconciliation ohne Wiederholung des alten Starts, neue Generation erfolgreich |

Nur der synthetische Live-Crashfall enthält einen authentisiert gebundenen Halt von maximal zwei
Sekunden nach Receipt/Schließen der Pipe. Der Controller prüft dabei `WAIT_TIMEOUT` vor dem harten
Crash. Normalpfade haben keinen Halt; keine Netzaufrufe/Netzcapabilities ergänzt.

47 Controller: 26 Erfolge, 17 erwartete Abweisungen, vier Abbrüche. Sieben Worker angelegt, zwei nie
fortgesetzt, vier normal beendet und einer tatsächlich laufend unterbrochen. Sechs authentisierte
Workerbindungen; drei neue Endrecords, in dieser Serie jeweils nativ bestätigte PID-Abwesenheit.
Jeweils Stale-/Pending-/Replayabwehr und neue Generation bei gültigem Endbeleg.

Die ungebundene eigene Testfixture wird ausschließlich vom Prüfablauf nach separater
Identitäts-/Endkontrolle aufgeräumt, keine allgemeine Produktbereinigung. Deren Kontrollablage
bleibt mit ursprünglichem RunIntent und verbrauchtem Launch pending; kein Reset oder nachträglicher
Workerbeleg. Insgesamt sieben OS-Profile: sechs Broker-Cleanups, ein ausdrücklicher eigener
Fixturecleanup. Alle Profile am Ende abwesend, Prozesse beendet; ein erwarteter logischer
Pending-Store bleibt gesperrt.

Unabhängiger lesender MAC-/PE-/Root-/Inventar-/Profilaudit bestätigt 63 Records (9/18/18/18), sieben
verbrauchte Launches, 42 tatsächliche private Zugriffssperren sowie native Token-/Job-/
Lease-/Identitätsbelege. Daten/Schlüssel/Policy/Unicode und echte Vorschauablage unverändert. Null
GUI, Importtasks oder Projektkopien; Roh-PIDs/Erzeugungszeiten/IDs/Schlüssel/Records bleiben lokal.

Zwei neue Tests: native Alive-/PID-Generationsprüfung und Jobpolicy einschließlich verweigerter
zweiter Zuweisung; dauerhafte Kontext-/Reihenfolge-/Endrecordbindung mit Rekonstruktion und
Ablehnung fehlender/falscher Belege. Eine verweigerte manuelle Jobzuweisung wird **nicht** als
Terminierungsbeleg behandelt; beide suspendierten Testprozesse werden separat aufgeräumt. **75
aktive Helfer-/Transporttests bestanden**, vier bewusst ignorierte Hilfs-/Dialogtests, Clippy mit
Warnungen als Fehler und rustfmt grün. Neuer gesicherter Helferbuild, kein App-/gemeinsamer
Transport-/Installerumbau, `pnpm check` oder allgemeine neue Matrix.

## Reproduktion und nächste Grenze

`scripts/runtime-isolation/check-workspace-worker-crashes.ps1` verlangt explizite Programmhashes und
legt selbst frische synthetische Wurzeln an. Keine vorhandenen Store-/Profilwurzeln übergeben. Der
bisherige Ressourcenprüfhelfer akzeptiert ausdrücklich v2/v3 für hashgebundene historische
Reproduktion; das ist keine Runtime-Migration oder Adoption einer alten Kontrollablage.

- Gesicherter Controller/Worker: `37d7beccd2c8368a9e34f27d6b29d668f25a2816fbd43893bc6abddf885ca081`.
- Nur kopierte UI-Fixture: `6e56a51ed2e913a878a7935e039bd69628a49be25f0fbab7d02fce3a72fe8c70`.
- Lokal `%TEMP%/kraqel-product-worker-lifetime-build-20261117-01` mit `functional-root.txt`,
  privatem Audit und Buildmanifest. Vorigen `9021872…`-ETW-Build unverändert erhalten.

**Nächster Schritt: gezielter frischer ETW-Netzvergleich des neuen Controllers und der Worker mit
diesen Lebenszeit-/Recoverypfaden.** Der vorige Netzbeleg gilt nur für `9021872…`, nicht für neue
Bytes. Crash vor dauerhafter Workerbindung, unbestätigte Profile, mehrprozessige GUI-Lebenszeiten,
persistente echte Arbeitsablagen-/UI-Bindung, P2 insgesamt/P3–P5 sowie langfristige
Kontrollhistorie, Power-Loss- und normale Produktabnahme bleiben offen. Keine Lockerung des strikten
Datenschutzziels.
