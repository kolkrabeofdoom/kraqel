# P2c BP2: nativer Blattprozess, Creation-Policy und Readback

Stand: 18. November 2026. **Begrenzter BP2-Forschungsumfang samt frischem ETW geprüft; keine
Runtime-/Datenautorität.** [Bereinigte Aggregate](product-start-p2c-bp2-native-leaf.json),
[Vertrag](../data-format/product-start-p2c-c4-preload-v1-draft.md),
[BP1-Modell](product-start-p2c-bp1-model.md).

## Implementierung und tatsächliche Grenze

[Native BP2-Grenze](../../scripts/runtime-isolation/probe/src/windows/exchange/ui_restart/durable/product/workspace_persistent/bp2.rs),
[Controller](../../scripts/runtime-isolation/probe/src/windows/exchange/ui_restart/durable/product/workspace_persistent/bp2/controller.rs),
[Policyadapter](../../scripts/runtime-isolation/probe/src/windows/exchange/ui_restart/durable/product/workspace_persistent/bp2/policy.rs).
Der Controller ist nur vom ausdrücklich ignorierten synthetischen Test aufrufbar; kein neuer
Host-CLI-, Produkt-, GUI- oder Datencommand. Der normale Helfer erhält einen früh abgefangenen
Workerzweig `bp2-leaf-worker`; fehlender Bootstrap endet mit 2. `bp2-must-not-start` ist ein
ebenfalls früh abgewiesenes Ziel für den verbotenen Kindstart, kein Diagnose-/Netzfallback.

Keine Umwandlung von `SyntheticLeafEligible` in native Autorität. Der Adapter prüft unabhängig den
tatsächlichen Windows-Supportmaskenwert und die wirksamen Einstellungen. Angeforderte Creation-Bits
`NoRemoteImages` und `NoLowMandatoryLabelImages`: `(1u64 << 52) | (1u64 << 56)`, gegen die lokal
vorhandenen Windows-SDK-Header gegengeprüft. Child-Process-Policy Restricted = 1. Kein Wegmaskieren
nicht unterstützter Pflichtbits. Ein zusätzlicher, tatsächlich nicht in der Supportmaske vorhandener
Anforderungsbit wird vor CreateProcess und Kontrollmutation abgewiesen. Dies behauptet nicht, dass
die beiden regulären Pflichtregeln auf diesem Rechner fehlen.

[Gemeinsame Attribute](../../scripts/runtime-isolation/probe/src/windows.rs) erhalten eine optionale
interne Erweiterung um zwei frühe Attribute; bisherige Aufrufer verwenden weiterhin None. BP2 setzt
Mitigation-/Child-Policy, AppContainer ohne Capabilities, vier ausdrücklich ausgewählte Pipe-Enden
und atomare Jobzuordnung gemeinsam beim suspendierten Start. Zugehörige Werte und Handlearrays
bleiben bis nach CreateProcess gehalten. Native Imageidentität, Token/SID, Job und sechs private
Dateisperren werden vor Resume geprüft. Readback am gehaltenen suspendierten Prozess bestätigt
Image-Load-Flags 3 und Child-Flags 1; geforderte Audit-/Overridezustände werden nicht akzeptiert.
Dasselbe Readback erfolgt im tatsächlich gestarteten Blattprozess.

Neue nicht geheime Codekopie erhält nur im eigenen frischen Fixturebereich RX-Rechte für das eigene
Profil und ein Medium-Integrity-Label. Dadurch ist die Kopie kein erlaubtes Low-Label-Image; keine
vorhandenen Daten-/Kontrollrechte oder globalen Einstellungen werden erweitert. Keine DB, Runtime,
Projektpfade/-texte, Provider oder echte Vorschauablage im Worker. Originale C2-Daten bleiben
synthetische Testfixtures; daraus wird keine allgemeine Daten-ACL-Isolation abgenommen.

Microsoft beschreibt
[Creation-/Child-Attribute](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-updateprocthreadattribute),
[Readback](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-getprocessmitigationpolicy)
und
[Image-Load-Policy](https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-process_mitigation_image_load_policy).
Die beobachteten Werte stammen aus diesem konkreten Versuch, keine portable Supporttabelle oder
vollständige Paket-/Rollenattestierung daraus.

## Eigener authentisierter Auftrag

Eigene Frame-Domäne `KRAQEL-BP2-LEAF\0\x01`, Version 1 und Zweck `bp2-synthetic-leaf-capability`.
u32-LE-Payloadlänge, 32-Byte-HMAC-SHA-256 und strikt typisiertes JSON bis 8.192 Bytes. Grant bindet
Launch/Nonce, Workspace, Wächtergeneration, Paket-/Codehash, eigenes Profil/Codeziel und
angeforderte Regeln. Root prüft Pipe-Erzeuger, Code, Token, Job und Policy; Accepted/Continue binden
Grantdigest und die jeweiligen nativen Generationen. Ergebnis ebenfalls MAC-gebunden. Kurzlebiger
zufälliger Pipe-Schlüssel; keine dauerhafte Kontrollautorität im Blattprozess.
Boot-/Key-/Continuehandles geschlossen, Ereignishandle nicht vererbbar, der versuchte Kindstart
verwendet `bInheritHandles=false`. Keine privilegierten Host-/Job-/Leasehandles werden delegiert.

Eigene native Action `bp2LeafProbe`, Eventversion 1, eigener BP2-Zustand im bestehenden
Schema-6-Forschungsreader unter derselben exklusiven Kontrollsperre. Aktiver Auftrag sperrt andere
Aufträge, Updates, Migrationen und Stilllegung. Alte Reader kennen die neue Action nicht und weisen
sie ab; kein stiller Altstore-/Paket-/Markerumbau oder neue UI-ABI. Verwendung nur an frischer, an
die neuen Controllerbytes gebundener synthetischer Ablage.

Vor Start sechs Recordslots reserviert; Commit/RootBound vor Resume. Accepted/Running vor Continue.
Drained/Finish erst nach Leerstand derselben gehaltenen Jobinstanz und bestätigtem Root-Handleende.
SHA-256 des vollständigen privaten Ergebnisobjekts wird in beiden Endrecords gebunden; unabhängiger
Audit verifiziert diese Hashes zusätzlich zur MAC-Kette. Launch bleibt verbraucht. Kein
automatischer Wiederanlauf oder Reparatur eines unklaren Auftrags.

## Geprüfte Fälle

| Fall                                           | Tatsächliches Ergebnis                                                                      |
| ---------------------------------------------- | ------------------------------------------------------------------------------------------- |
| Zusätzliche nicht unterstützte Anforderung     | Kein CreateProcess, Kontrollhead/Recordzahl unverändert                                     |
| Absichtlich abweichende Readback-Sollerwartung | Native Istwerte regulär 3/1, Vergleich weist ab; Root nicht resumed, kontrolliertes Ende    |
| Falsche Grant-MAC                              | Richtlinien vor Resume vorhanden; Worker weist Bootstrap ab, kein Accepted/Continue, Exit 2 |
| Regulärer synthetischer Blattprozess           | Beide Readbacks 3/1, authentisierter Ablauf, Kindstart scheitert mit Fehler 367             |

Die abweichende Erwartung ist ein gezielter negativer Vergleichstest; kein behaupteter nativer
Windows-Policyfehler. Der Diagnosejob hat **zwei** Plätze, damit die Kindablehnung nicht nur durch
ein Einprozess-Joblimit entsteht. Das ist ein getrenntes Forschungsbudget, keine Übersetzung oder
Aufweitung des BP1-Blattprofils zu einer Produktpolicy. Tatsächlich je Job nur eine Generation; kein
angelegtes Kind. Ein unerwartet angelegtes Kind würde suspendiert bleiben und beendet; daraus würde
kein bestandener Fall. Entscheidend ist der konkrete Windows-Fehler **367,
ERROR_CHILD_PROCESS_BLOCKED**, zusätzlich zum wirksamen Child-Readback.

Funktionale Serie und eine Wiederholung unter ETW: unabhängig jeweils **18 MAC-Records, 14
BP2-Records, drei verbrauchte Launches/Rootbindungen**, ein Accepted/Running und drei
Drained-/Finishpaare geprüft. Zwei negative gestartete Aufträge enden ohne Accepted; die
Unsupported-Anforderung erzeugt keinen Launch. Drei Ergebnis-Hashes passen zu den dauerhaft
authentisierten Endrecords. Tatsächliches Store-Wiederöffnen ohne Pending und mit drei verbrauchten
Launches geprüft. Beide eigenen Profile unabhängig als abwesend und alle eigenen
Kontroll-/Datenfixtures als entfernt bestätigt.

## Frischer ETW-Nachweis des exakten Paars

Normaler Helfer/Blattcode: `5205d72be94b0810a9326ba00368d1fb9e81e2af1109c7c0a29ed1798a3788cd`;
tatsächlicher Host-Testcontroller:
`62d7a73c4ce3f776c2a866b7b8156dcac5c7f4e980086fd134b31c21509d142e`. Hashbenannte identische
Hostkopien, Hashvergleich vor/nach Messung. Unveränderte
[Erfassung](../../scripts/m3-network/capture.ps1) und
[Auswertung](../../scripts/m3-network/analyze.py), `native-control`, explizite PID-/EXE-Zuordnung
und gehaltene Hosthandles bis nach Konvertierung. Nur Erfassungshelfer erhöht nach UAC; keine
System-/Firewall-/Loopback-/Schutzsoftwareänderung und keine allgemeinen Netzwerkrechte.

Referenz: direkter Workeraufruf ohne Bootstrap und genau ein reiner Policy-Übersetzungstest; Prüfung
gegen tatsächliche Ausgabe „1 passed“. Serie: alle vier oben genannten Fälle auf frischer Fixture.
Maximal drei Minuten je Erfassung, tatsächlich ungefähr 0,5 und 4,6 Sekunden ab Ready bis Stop. Nach
Ende per eigener STOP-Datei gestoppt und konvertiert, Supervisor beendet.

| Phase                       | Hostaufrufe | Blattprozesse | Konsolenhosts | Explizite Enden | Kontrollen | Netzereignisse |
| --------------------------- | ----------: | ------------: | ------------: | --------------: | ---------: | -------------: |
| Bootstrapabweisung/Referenz |           2 |             0 |             2 |               4 |        4/4 |              0 |
| Native BP2-Serie            |           1 |             3 |             1 |               5 |        4/4 |              0 |
| Zusammen                    |       **3** |         **3** |         **3** |           **9** |    **8/8** |          **0** |

Alle neun Enden explizit, drei Hoststartzeiten und drei Blattgenerationen/Elternbezüge auf 100 ns
mit Kernelereignissen abgeglichen; Rootenden jeweils vor Hostende. Keine zusätzlichen Prozesse oder
Runtimekinder im Baum. Null verlorene Events/Buffers, keine Zuordnungslücken oder unbekannten Ziele.
**Kein beobachteter Netzwerkverkehr einschließlich Loopback in diesen Prozessbäumen.** Keine
Inhalts-/Uploadanalyse oder Zuordnung fremder COM-/Shell-/Sicherheitsdienste daraus; keine
universelle Betriebssystem- oder Produktfreigabe. Alte Bytepaare/Nachweise bleiben separat.

## Prüfniveau, Sicherung und Fortsetzung

Drei neue aktive Tests: Übersetzung/fehlende Bits/Audit, begrenzte MAC-Frames und
Recordfolge/Replay/Sperre. **136 aktive Helfer-/Transporttests bestanden** (129 Binär-/sieben
Transporttests), sieben bewusst ignoriert; neuer nativer Fall zusätzlich funktional und einmal
gemessen ausgeführt. Clippy `--all-targets -- -D warnings`, rustfmt, 27 Auswertungstests sowie
Fakten-/Hash-/Verweis-/Formatprüfung. Mittleres Prüfniveau, kein App-/Transport-/Installerumbau und
keine neue breite Crashmatrix. BP1-Modell und native UI-ABI 0 unverändert.

Geschützte lokale Sicherung `%TEMP%/kraqel-bp2-build-20261118-01/`: exaktes Paar,
`run-functional.json`, gemessener `run.json`, lesende MAC-Audits, unabhängiger Audit und Manifest.
Rohmessungen unter `%TEMP%/kraqel-bp2-etw-20261118-01-{reference,gate}/`; zusätzliche geschützte
Sicherung außerhalb von TEMP unter `artifacts/bp2-native-20261118-01/`. Keys, PIDs, SIDs, absolute
Prozesspfade und Roh-ETL/XML bleiben lokal und von Git ausgeschlossen. Kein Audit als aktiven Store
oder neuer Grant verwendbar.

**Nächster begrenzter Scope BP3:** unterstützten präventiven Wächter-/Modul-/Rollen-Erzeugungsweg
und Machbarkeit konkret nachweisen. Die Blatt-Kindstartsperre lässt sich nicht als WebView-
Rollenfilter verwenden oder für Kompatibilität einfach abschalten. NoRemote/NoLowLabel sind keine
exakte Modulpositivliste; nachträgliche Snapshots und Signaturen schließen diese Lücke nicht. Keine
Hersteller-/Signaturausnahme oder Abschaltung von Schutzsoftware. BP4 effektive
Datenrechte/Commandfreigabe, C4b insgesamt/C4c/d, P2/P3–P5 und Produkt-Q-01/Q-02 bleiben offen. Der
Fähigkeitsversuch erzeugt keine Runtime-/Datenautorität und belegt keine vollständige
Start-Vertrauenskette des Wächters. Das strikte Datenschutzziel bleibt bestehen.
