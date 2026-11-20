# Produktstart P2a: Authentisierte persistente Workspace-/Launch-Records

Stand: 17. November 2026. Begrenzte native Forschungsimplementierung, **keine Start-/Datenfreigabe
für das normale Produkt**. [Bereinigte Aggregate](product-start-p2a-records.json),
[Integrationsvertrag](../data-format/product-start-integration-v1-draft.md). Striktes
Datenschutzziel und offene Produkt-Q-01/Q-02 bleiben unverändert.

Inzwischen ergänzt [P2b](product-start-p2b-bootstrap.md) separate synthetische Probeworker, ohne
diese private P2a-Fixture oder deren logische Profil-ID zu promovieren. Dieser Abschnitt beschreibt
den historischen Record-only-Prüfstand und dessen eigenen Build.

## Implementierung und Grenze

[`workspace.rs`](../../scripts/runtime-isolation/probe/src/windows/exchange/ui_restart/durable/product/workspace.rs)
ergänzt das ausschließlich pfadbegrenzte `product-workspace-child` im synthetischen
TEMP-Controller-/Case-/Store-Layout. Keine Verbindung mit normalem `broker-store`, Renderer oder
Appstart; keine Vorschau-/Projektübernahme. Schema 5 wird nur ausdrücklich bei dieser Forschung
initialisiert/migriert, nicht beim regulären Programmstart. Importtestsitzung/Pending-Update sperren
den Übergang. Authentisierte gültige Migrationspräfixe können ausdrücklich unter aktuellem Kopf
fortgesetzt werden; keine Reparatur unvollständiger Record-/Kopfabschlüsse.

Neue Kontrollaktionen im bisherigen privaten MAC-Recordformat:

- `bindWorkspace`: Installation, Workspace, reservierte logische Profil-ID, Broker-/UI-Fixturehash,
  Policy-/kanonische Rootbindung, Workspaceformat und Offlinepolicy. Profil-ID ist **kein erzeugtes
  OS-Profil**. Festes neues privates `data/workspace` mit Policy/Unicode-Sentinel und private
  `bin/p2a-ui.exe`-Kopie, ohne Ausführen dieser UI-Fixture. Existierende Verzeichnisse/Pakete eines
  unterbrochenen ersten Bindens werden nicht übernommen.
- `commitWorkspaceLaunch`: zweckgebundener Startvorschlag ohne Importtask; exakt nächste Generation
  und unverbrauchte ID vor Verwendung festschreiben. Das ist **keine Kindprozess-Startcapability**.
- `reconcileWorkspaceLaunch`: dieselbe festgeschriebene Generation als unterbrochen abgleichen; ID
  bleibt verbraucht, Binding/Schlüssel/Daten bleiben erhalten. Kein behauptetes normales UI-Ende, da
  P2a überhaupt keinen UI-Prozess startet.

Der native Adapter verwendet die [reinen P1-Regeln](product-start-p1-model.md) nach den nativen
Actor-/Kopf-/Budget-/Jobprüfungen. Der Kern ist nun unter Windows eingebunden, weiterhin ohne eigene
I/O/Schlüssel/OS-Operationen. Er wird nicht als angeblich authentisierter Store deserialisiert. Beim
Öffnen: private ACLs/Vorfahrenpins/Storelease, Schlüssel, MACs, Root/Actor/Sequenz/Verkettung
prüfen; Kernzustand aus den einzelnen Records samt tatsächlichen Recorddigests rekonstruieren. Erst
bei passendem MAC-geschütztem Abschlusskopf wird der Store zurückgegeben. Beim Anhängen wird ein
vollständiger nächster Zustand vor Recordschreiben validiert und erst nach Record-/Kopfpersistenz
übernommen. Ablehnungen schreiben nichts.

Dieselbe exklusive Kontrollsperre bleibt über Prüfung, Dateibindung und Recordabschluss gehalten.
Aktive Prozesse im benannten Produktjob sperren Mutationen. Datei-/Policy-/UI-Fixturehash und
kanonische Rootbindung werden vor den neuen nativen Folgeaktionen geprüft. Bindung an fremde
Installation, Actor, Kopf, Workspace, Paket oder Konfiguration wird nicht durch Parseerfolg ersetzt.

Die P2a-Arbeitsablage ist eine **brokerprivate Fixture** unter dem weiterhin privaten `store/data`,
kein containerzugänglicher Produktvault. `configSha256` bindet die feste synthetische Policy,
`uiPackageSha256` eine einzelne Fixture-EXE, noch kein vollständiges Produktpaket samt wirksamer
Tauri-Konfiguration. P2b muss einen getrennten freigegebenen App-Daten-/Paketbereich explizit binden
und prüfen. Keine pauschalen Containerrechte auf `store`, `control`, `bin` oder das reservierte
`data`; P2a-Fixture nicht durch Hashumdeutung oder ACL-Erweiterung promovieren.

## Paketdeklaration

Die bisherige `.kqmeta`-Deklaration für den Legacy-Vertrag 1–4 bleibt erhalten. Zusätzliche eng
begrenzte read-only PE-Section `.kqprod` deklariert ausschließlich P2a-Records:

| Offset | Bytes | Wert                                      |
| -----: | ----: | ----------------------------------------- |
|      0 |    16 | `KRAQEL-P2A-META` plus Nullbyte           |
|     16 |     4 | Metadatenversion 1                        |
|     20 |     2 | Kontrollschema 5                          |
|     22 |     2 | Workspace-Record-ABI 1                    |
|     24 |     2 | UI-Bootstrap-ABI **0**, nicht unterstützt |
|     26 |     2 | Flags 0                                   |
|     28 |     4 | Reserviert 0                              |

Integer little-endian, exakt 32 Metadatenbytes. Derselbe begrenzte PE64-Parser prüft eindeutige
Section, Header-/Dateigrenzen und initialisierte lesbare, nicht schreibbare/ausführbare/verwerfbare
Daten. Legacy-/fehlende/fremde Deklaration sowie behauptete UI-Bootstrap-ABI 1 werden vom
P2a-Prüfpfad abgewiesen. Hashbindung/Metadaten sind keine Herausgeber-/Verhaltensattestierung. P2b
benötigt eine ausdrücklich startfähige Deklaration und einen nativ geprüften Bootstrap.

## Gezielte Prüfung

Vier neue Tests ergänzen den bisherigen Stand: MAC-Rekonstruktion/Verbrauch nach Wiederöffnen ohne
Importtask, falscher Actor/Kopf/Binding und geänderte Policy ohne Kontrollfortschritt, signierter
Tail ohne passenden Kopf/fehlender Schlüssel ohne Reparatur sowie begrenzte Record-only-Metadaten
ohne UI-ABI. Vorhandener Test prüft zusätzlich die echte eingelinkte `.kqprod`-Section.

**67 aktive Tests unter Windows** bestanden, vier bestehende Tests ignoriert. Clippy mit Warnungen
als Fehler und rustfmt grün. Ein reiner Helferbuild, keine App-/gemeinsame
Transport-/Installeränderung; gemäß mittlerem Prüfniveau kein `pnpm check`, Appbuild oder neue große
Matrix.

Drei native Fälle mit identischem gesichertem Build:

| Abbruch                   | Nach erneutem Öffnen                                               | Weiterführung                                                                             |
| ------------------------- | ------------------------------------------------------------------ | ----------------------------------------------------------------------------------------- |
| Vor Recordschreiben       | Bindung/Kopf unverändert, Generation 0                             | Expliziter neuer Commit, Reconcile, Replayabwehr, zweite Generation                       |
| Nach Kopfveröffentlichung | Generation 1 authentisiert festgeschrieben, pending                | Zweiter Commit/alter Kopf gesperrt, explizites Reconcile, Replayabwehr, zweite Generation |
| Nach Recordsync vor Kopf  | MAC-gültiger Launch-Tail, veröffentlichter Kopf bleibt auf Bindung | Status/Reconcile bleiben gesperrt, keinerlei Schlüssel-/Kopfreparatur                     |

31 Kontrollprozesse: 20 Erfolge, acht erwartete Abweisungen, drei harte Abbrüche mit 86. Keine UI,
kein OS-Profil, kein Importtask oder Projekt angelegt. Drei private synthetische Workspaces mit
unverändertem Unicode-Sentinel. Zwei nutzbare Abschlüsse mit je zehn Records/zwei verbrauchten
Launches; gesperrter Fall sieben Records, aber veröffentlichter Kopf bei Sequenz 5. Sein Tail ist
keine nutzbare Startautorität.

Ein unabhängiger Nur-Lese-Audit bestätigt beide PE-Deklarationen, SHA-256, jede Kontroll-MAC,
Root/Actor/Installations-/Workspacebindung, Generation/Launch-ID-Verbrauch, Abschlussköpfe und
vollständige Dateiinventare ohne Aufgaben-/Projektjournal. Policy, UI-Fixture und Unicode-Sentinel
geprüft; Schlüssel/Workspaces erhalten, Replays/fehlerhafte Aufrufe ohne Mutation. Geprüft ist
Prozessabbruch, keine Stromausfallgarantie oder produktive Historienkompaktierung.

## Reproduktion, Buildbindung und Fortsetzung

Gesichert unter `%TEMP%/kraqel-product-start-p2a-build-20261117-02/`:

- Broker `b56fc9acb484baedf1bbe68b77af1196a3d7a3f95a66a499a3fa07dc60d47ae7`.
- Nur als Datei verwendete UI-Fixture
  `6e56a51ed2e913a878a7935e039bd69628a49be25f0fbab7d02fce3a72fe8c70`.
- Quellbasis vor Änderungen `36bfbd6680a42c8a77a9ed5252520a5d6515a0cd`; genaue Buildbindung im
  privaten Manifest.

```powershell
$savedBuild = Join-Path $env:TEMP 'kraqel-product-start-p2a-build-20261117-02'
./scripts/runtime-isolation/check-workspace-records.ps1 `
  -Probe "$savedBuild/kraqel-isolation-probe.exe" `
  -ProbeSha256 'b56fc9acb484baedf1bbe68b77af1196a3d7a3f95a66a499a3fa07dc60d47ae7' `
  -UiFixture "$savedBuild/kraqel-desktop.exe" `
  -UiSha256 '6e56a51ed2e913a878a7935e039bd69628a49be25f0fbab7d02fce3a72fe8c70'
```

Der Helfer erzeugt ausschließlich neue TEMP-Wurzeln. Programmhashes vor/nach dem Lauf unabhängig
geprüft. Prozesse beendet; echte Vorschauablage und dort autorisierte EXE unverändert. Rohpfade,
Kennungen, Schlüssel und Records bleiben lokal, nur bereinigte Aggregate in Git. Keine realen
Projekte geöffnet, übernommen oder migriert; keine Containerrechte erteilt.

Dieser Broker hat noch keinen eigenen frischen ETW-Beleg. Frühere Belege gelten ausschließlich für
ihre Programme/Pfade/Lebenszeiten. Als Nächstes P2b: authentisierter synthetischer Bootstrap mit
Token-/Jobprüfung vor Ausführung, ohne Importtask; vor weiterer Freigabe eigener frischer
Netznachweis der neuen Programme. GUI-/Produktstart, monotone Bootstrapfrist, Handletransport,
langfristige Kontrollhistorie, Bestandsübernahme P3 und Produktintegration P4/P5 bleiben offen.
