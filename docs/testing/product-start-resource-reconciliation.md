# P2b: Begrenzte Ressourcen-Crash-Reconciliation

Stand: 17. November 2026, Quellbasis `83adfef`.
[Bereinigte Aggregate](product-start-resource-reconciliation.json),
[ursprünglicher Bootstrap](product-start-p2b-bootstrap.md),
[dessen eigener Netzbeleg](product-start-p2b-bootstrap-etw.md). **Keine Produktfreigabe.** Striktes
Datenschutzziel und Produkt-Q-01/Q-02 unverändert.

## Implementierung und Autoritätsgrenze

Der ausschließlich synthetisch pfadbegrenzte `product-workspace-probe-child` ergänzt ausdrücklich
`reconcile` unter derselben exklusiven Kontrollsperre. Actor, erwarteter Kopf, unveränderte private
Workspace-Fixture und tatsächlicher Jobleerstand werden vor OS-Aufrufen geprüft. Keine Integration
in normale App, `broker-store`, Importtask oder Bestandsübernahme.

Execution-Version **2** und `.kqboot`-Profilbereinigungs-ABI **2** kennzeichnen den neuen Pfad.
Schema 5, Probe-Bootstrap-ABI 1 und UI-Bootstrap-ABI 0 bleiben bestehen; `.kqprod` und Legacy-
`.kqmeta` unverändert. Alte Execution-v1-Records werden durch diesen geänderten Helfer nicht
stillschweigend als neue Ressourcenbelege interpretiert oder migriert. Neue synthetische Ablagen,
keine Neuautorisierung des Brokers an der echten Vorschauablage.

Neue authentisierte Kontrollaktionen:

- `workspaceProbeRunIntent` vor jedem möglichen Workerstart. Erst nach bestätigter eigener
  Profilerzeugung zulässig, niemals zweimal oder nach Drain/Abschluss.
- `workspaceProbeDrained` erst nach erfolgreicher äußerer Rückkehr des Bootstrapcontrollers: Job
  explizit beendet und am noch gehaltenen Kernelobjekt leer abgefragt. Ein kontrollierter innerer
  Workerfehler ist davon getrennt; ein fehlgeschlagener Drain erzeugt diesen Record nicht.
- `abortUncreatedWorkspaceProbe` schließt einen noch unbestätigten Launch ausschließlich bei nativ
  festgestellter Profilabwesenheit. Der Name bezeichnet fehlende Erzeugungsbestätigung, **keinen
  Beweis, dass nie ein Profil existiert hat**. Es wird kein Profil gelöscht/übernommen.

Der Normalpfad reserviert **fünf** Recordslots vor Ressourcenanlage: Commit → ProfileReady →
RunIntent → Drained → Finish bzw. kontrollierter Fehlerabschluss. Die Zwischenrecords
synchronisieren Kopf und Budget, ohne Launch/Generation zu erneuern. Abschluss/Reconciliation
erhalten die verbrauchte Launch-ID und Bindung; keine neue Start- oder Importautorität. Ein
unterbrochener Readonly-Probeabschluss wird als unterbrochen reconciliert, kein verlorener Erfolg
aus unsignierten Diagnosefiles erfunden.

[`recovery.rs`](../../scripts/runtime-isolation/probe/src/windows/exchange/ui_restart/durable/product/workspace_probe/recovery.rs)
leitet SID und Profilname ausschließlich aus dem authentisierten Execution-Record ab und liest genau
dessen HKCU-AppContainer-Mapping. Der Schlüssel muss tatsächlich fehlen, damit Abwesenheit
angenommen wird. Zugriffsfehler, fehlender/andersartiger/zu großer Moniker oder andere Identität
bleiben Fehler. Begrenzte UTF-16-Abfrage; keine Registryänderungen oder Befehls-/Pfadübernahme. Das
neue Registry-Feature betrifft ausschließlich den getrennten Windows-Forschungshelfer.

| Authentisierter Zustand      | Native Profilprüfung                       | Verhalten                                                                                           |
| ---------------------------- | ------------------------------------------ | --------------------------------------------------------------------------------------------------- |
| Commit ohne ProfileReady     | Mapping fehlt                              | Launch ohne Löschung unterbrochen abschließen                                                       |
| Commit ohne ProfileReady     | Mapping vorhanden                          | Eigentum unbestätigt, keine Adoption/Löschung/Recordänderung                                        |
| ProfileReady, kein RunIntent | Exakte eigene Identität oder Mapping fehlt | Eigenes Profil entfernen bzw. bereits fehlendes Profil bestätigen, Launch reconciliieren            |
| RunIntent ohne Drained       | beliebig                                   | Lebenszeit unbewiesen, unverändert gesperrt                                                         |
| Drained bestätigt            | Exakte eigene Identität oder Mapping fehlt | Nach bereits belegtem Jobende eigenes Profil bereinigen bzw. Abwesenheit bestätigen, reconciliieren |

Der vollständige Recordübergang samt Abschlussbudget wird vor Löschung vorgeprüft. Nach Löschung
wird tatsächliche Abwesenheit erneut geprüft, erst dann der terminale Record geschrieben. Ein
erneuter Crash zwischen Löschung und Abschluss ist dadurch bei belegter Eigentums-/Lebenszeitlage
wiederholbar. Unvollständige MAC-/Record-/Kopfveröffentlichungen bleiben weiterhin gesperrt, keine
Reparatur. Das Benutzer*innenkonto/OS bleibt vertrauenswürdig; keine Abwehr gegen eine maliziöse
Administration oder beliebige gleichberechtigte Änderungen der Kontrollautorität.

## Gezielte native Prüfung

Vier ausdrücklich über die synthetische CLI bewaffnete Crashpunkte, harte Prozessenden mit 86 ohne
Destruktor-/Cleanupausführung; keine Umgebungsvariable oder produktive Fault-Schnittstelle:

| Punkt                                      | Befund nach Wiederöffnen                               | Reconciliation                                                                     |
| ------------------------------------------ | ------------------------------------------------------ | ---------------------------------------------------------------------------------- |
| Nach Commit, vor Profilerzeugung           | Pending, kein Profil                                   | Ohne Profillöschung abgeschlossen                                                  |
| Nach Profilerzeugung, vor ProfileReady     | Pending, unbestätigtes Profil                          | Broker verweigert unverändert die Löschung; siehe getrennten Fixture-Cleanup unten |
| Nach ProfileReady, vor RunIntent           | Bestätigtes eigenes Profil, kein erlaubter Workerstart | Eigenes Profil entfernt, Launch bleibt verbraucht                                  |
| Nach Profilentfernung, vor Abschlussrecord | RunIntent/Drained gespeichert, Profil fehlt            | Ohne erneuten Workerstart/Löschannahme unterbrochen abgeschlossen                  |

Alle Fälle prüfen veralteten Kopf, gesperrten Pending-Neustart, erneute Reconciliation ohne Pending,
Replay und erfolgreiche neue Generation. 49 Controller: 28 Erfolge, 17 erwartete Abweisungen und
vier Abbrüche. Fünf erfolgreiche Worker mit 30 tatsächlichen privaten Dateisperren, unverändertem
Token-/Job-/Lease-/Receipt-/Drainnachweis. Null UI, Importtasks oder Projektkopien.

**Die unbestätigte Testfixture ist keine automatische Produktbereinigung:** Nach dem ausdrücklich
nachgewiesenen Broker-Nein löscht ausschließlich der Prüfablauf sein unmittelbar zuvor selbst
erzeugtes synthetisches Profil. Frische selbst angelegte Wurzel, fest freigegebener Programmhash,
MAC-geprüfter eigener Commit und exakter Moniker begrenzen diesen Testcleanup. Danach stellt der
Broker nur Abwesenheit fest und schließt den unverändert verbrauchten Launch. Dieser Ablauf ist
keine allgemeine Löschfreigabe aus einem Erzeugungsintent oder unbekannten Resten.

Sieben OS-Profile erzeugt: sechs vom Broker entfernt, eine eigene mehrdeutige Fixture separat durch
den Prüfablauf. Alle am Ende abwesend, Prozesse beendet. Unabhängiger lesender Audit bestätigt
PE-Marker/Programmhashes, MAC-Ketten/Köpfe/Actor/Root/Bindungen, acht verbrauchte Launches in vier
Arbeitsablagen und 56 vollständige Records (13/13/14/16). Private Schlüssel/Policy/Unicode-Sentinels
und echte Vorschauablage unverändert; Rohpfade/PIDs/IDs/Nonces/Schlüssel/Records bleiben lokal.

Zwei neue Tests für native Profilabwesenheit/exakte Identität und gesperrten RunIntent ohne Drained
mit authentisiertem Wiederöffnen. Bestehende Receipt-/Reconciliationtests um neue Reihenfolge
erweitert. **73 aktive Helfer-/Transporttests bestanden**, vier bewusst ignorierte
Hilfs-/Dialogtests, Clippy mit Warnungen als Fehler und rustfmt grün. Ein Helferbuild, kein neuer
App-/Transport-/Installerbuild oder `pnpm check`. Mittleres Prüfniveau ohne allgemeine neue Matrix.

## Reproduktion, Sicherung und offene Gates

Inzwischen liegt der [eigene frische ETW-Vergleich](product-start-resource-reconciliation-etw.md)
für exakt diesen Runtimehash vor: zwei gültige Phasen, 54 Controller/sechs Worker, 114 explizite
Enden, acht Kontrollen, null beobachtete Netzereignisse. Die folgende nächste Aufgabe beschreibt das
Gate nach der Implementierung; aktueller Scope ist dauerhafter Workerstartbeleg und gesicherter
Wiederanlauf zwischen RunIntent und Drained.

`scripts/runtime-isolation/check-workspace-resource-crashes.ps1` verlangt explizite Hashes und
erzeugt ausschließlich neue synthetische Ablagen. Keine vorhandenen Store-/Profilwurzeln übergeben.

- Controller/Worker: `902187213995a3cbf590328f80311a701be19716948a7a45423223581d7994dc`.
- Nur kopierte UI-Fixture: `6e56a51ed2e913a878a7935e039bd69628a49be25f0fbab7d02fce3a72fe8c70`.
- Lokale Sicherung: `%TEMP%/kraqel-product-resource-crashes-build-20261117-01`, private
  `functional-root.txt`, Audit und Buildmanifest. Vorherigen `590c3d4…`-ETW-Build unverändert
  erhalten.

**Nächster Schritt: gezielter frischer ETW-Netzvergleich dieses geänderten Controllers und seiner
Worker einschließlich Reconciliation.** Der vorige Netzbeleg gilt nur für `590c3d4…` und wird nicht
auf diese neuen Bytes übertragen. Keine Netzdiagnose im Worker oder neue Netzcapabilities.

Die Reconciliation ist bewusst begrenzt. Unbestätigte bestehende Profile und RunIntent ohne
dauerhaften Drained-Beleg bleiben gesperrt; ein echter Crash während laufendem Worker ist damit noch
nicht automatisch bereinigbar. Es gibt keine sichere Adoption/Löschung unbekannter Reste,
langzeitfähige Kontrollhistorie, Power-Loss-Garantie oder normale Produkt-/Datenmigration.
Persistente echte Arbeitsablagen-/UI-Bindung, P2 insgesamt und P3–P5 bleiben offen.
