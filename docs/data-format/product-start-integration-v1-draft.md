# Produktstart und bestehende Projekte · Vertragsentwurf v1

Stand: 17. November 2026. **Implementierungsentwurf, keine Produktfreigabe und kein bereits
ausführbares Protokoll.** Die Anforderungen unten sind für die nächste Integrationsstufe vorgesehen.
P1 ist als [reiner Typ-/Zustandskern](../testing/product-start-p1-model.md) umgesetzt;
[P2a](../testing/product-start-p2a-records.md) ergänzt authentisierte Schema-5-Records
ausschließlich in synthetischen privaten Forschungsablagen ohne UI-Start.
[P2b](../testing/product-start-p2b-bootstrap.md) ergänzt einen getrennten synthetischen
Readonly-Workerbootstrap mit Token-/Jobprüfung, ohne Importtask/GUI. Der
[frische P2b-Netzbeleg](../testing/product-start-p2b-bootstrap-etw.md) gilt für diesen Probeworker,
nicht für die normale Schreiboberfläche. Produktbootstrap, ausführbare Startautorität,
Bestandsdatenübernahme und normale Integration fehlen weiterhin. Striktes Datenschutzziel und offene
Produkt-Q-01/Q-02 bleiben unverändert.

## 1. Verifizierter Ausgangspunkt

| Gegenstand           | Tatsächlicher Code / Grenze                                                                                                                                                                                                                                                                                                                                                                                          |
| -------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Normaler Start       | [`main.rs`](../../apps/desktop/src-tauri/src/main.rs) ruft ohne Forschungsfeature direkt `run()` auf. [`lib.rs`](../../apps/desktop/src-tauri/src/lib.rs) erzeugt den Tauri-Builder und `app_local_data_dir()/projects`, registriert Schreib-, Datei-, Git- und KI-Commands. Keine vorgeschaltete Produktbroker-Startberechtigung.                                                                                   |
| Forschungsoberfläche | [`isolation_ui.rs`](../../apps/desktop/src-tauri/src/isolation_ui.rs) empfängt zuerst einen nativen Auftrag und startet die separate `isolation-test.html` mit fünf Testcommands. Sie ist nicht die vollständige Schreiboberfläche.                                                                                                                                                                                  |
| Forschungsstarter    | [`starter.rs`](../../scripts/runtime-isolation/probe/src/windows/exchange/ui_restart/durable/product/starter.rs) prüft Schema 2 und erzeugt frische synthetische Daten; [`session.rs`](../../scripts/runtime-isolation/probe/src/windows/exchange/ui_restart/durable/product/session.rs) bindet unter Schema 3/4 genau eine Testsitzung je Installation. Kein allgemeiner Produktstart mit bestehenden Arbeitsdaten. |
| Import-Recovery      | [`RestartTicket` v3](../../scripts/runtime-isolation/probe/src/restart.rs) enthält zwingend eine Task-ID und Quelle `Synthetic`/`Live`. Verbrauchter Import und Replay-Sperre sind keine allgemeine Berechtigung zum Programmstart.                                                                                                                                                                                  |
| Projektöffnung       | [`ProjectManager::open_project`](../../crates/storage/src/project_manager.rs) verwendet [`Database::open`](../../crates/storage/src/lib.rs). Dieses konfiguriert SQLite/WAL und führt gegebenenfalls Migrationen aus. Kein garantiert nebenwirkungsfreier Inventarisierungsweg.                                                                                                                                      |
| Vorschauautorität    | [`product.rs`](../../scripts/runtime-isolation/probe/src/windows/exchange/ui_restart/durable/product.rs) verwendet `.kraqel-broker-preview` unter der geprüften lokalen Profilwurzel. Diese Ablage ist weder normale Appdatenwurzel noch automatisch übernehmbare Produktautorität.                                                                                                                                  |
| Netzbeleg            | [Aktueller ETW-Nachweis](../testing/native-prepare-transition-etw.md): geprüfte Forschungsprogramme/Pfade, keine Freigabe des normalen Starts oder beliebiger externer Dienste.                                                                                                                                                                                                                                      |

Dieser Entwurf wurde statisch gegen diese Quellen geprüft. Keine realen Projekte inventarisiert,
kopiert, geöffnet oder migriert; keine ACLs/Schlüssel geändert und kein normaler Appstart
ausgeführt.

## 2. Festgelegte Integrationsrichtung

Der Produktstart erhält eine **eigene Autorität zum Öffnen einer gebundenen Arbeitsablage**. Er ist
unabhängig von Importaufträgen, Dateidialog-Grants und Providerfreigaben. Keine künstliche Task-ID,
kein `automatic=true` als Ersatz und keine Umdeutung von `RestartTicket` v3.

Der erste Integrationsumfang ist offline: Start, Projektübersicht und lokale Arbeit in ausdrücklich
gebundenen Daten. Externe Dateien, Shell, Git-Unterprozesse, Ollama, Cloudanbieter und
Betriebssystem-Schlüsselspeicher werden dadurch nicht automatisch freigegeben. Noch nicht isoliert
integrierte Funktionen müssen serverseitig gesperrt sein; ein ausgeblendeter Button reicht nicht.
Bereits vorhandene fachliche Funktionen bleiben dokumentiert, ihre Produktisolationsabnahme offen.

Vorgesehen sind getrennte Rollen: ein nativer Launcher/Broker ohne WebView und ein eingeschränkter
UI-Prozess. Direkter Start der UI ohne gültigen Bootstrap endet **vor** Tauri-/WebView-Erzeugung und
vor Appdatenzugriff. Kein Rückfall auf das heutige unisolierte `run()`, auch bei fehlendem Broker,
inkompatiblem Paket, defekter Kontrollablage oder direktem EXE-Aufruf. Dev-/Forschungsvarianten
bleiben eindeutig getrennt und dürfen nicht als Produktpaket ausgegeben werden.

```text
nativer Launcher -> private Kontrolle + Arbeitsablagenbindung prüfen
                -> neue Startgeneration dauerhaft verbrauchen
                -> UI suspendiert erzeugen -> Token/Job/Handles prüfen -> fortsetzen
UI              -> gebundene lokale Arbeitsablage, keine Kontrolle/Schlüssel/Altbestände
Datei-/KI-Grants -> eigene ausdrückliche Vorgänge, keine Ableitung aus dem Start
```

## 3. Autoritäts- und Zustandsvertrag

P1 konkretisiert ausschließlich `WorkspaceBindingV1`/`StartProposalV1` im Testmodell:
camelCase-JSON, UUIDs als 16-Byte- und Digests als 32-Byte-Arrays, strikt bekannte Felder und
Payloadgrenze. Ergebnis `UntrustedStartProposal`, kein authentisierter Envelope. Der geplante native
Autoritätsvertrag unten wird dadurch nicht vorweggenommen. Details und Grenzen im
[P1-Nachweis](../testing/product-start-p1-model.md).

Geplant ist ein separat versionierter `ProductStartAuthorizationV1` mit strikt unbekannte Felder
ablehnender Form; **kein** derzeit akzeptierter Rendererrequest oder neues CLI-Freiformargument. Ein
Payload ist höchstens 8.192 Bytes, sein gesamter Frame höchstens 16 KiB. Pflichtbindungen:

| Bindung                           | Vorgesehene Bedeutung                                                                                                 |
| --------------------------------- | --------------------------------------------------------------------------------------------------------------------- |
| Version/Zweck                     | `version=1`, `purpose=workspace-start`; nie Import/Recovery eines Imports                                             |
| Installation/Arbeitsablage/Profil | Nichtleere UUIDs; aus authentisierter Kontrolle, nicht aus Ordnerlisten oder UI-Angaben                               |
| Startgeneration                   | Neue nichtleere Launch-UUID und monotoner Zähler; dauerhaft vor `ResumeThread` verbraucht                             |
| Aktueller Kontrollkopf            | Exakter erwarteter Kopf; veraltete oder parallel überholte Entscheidungen abweisen                                    |
| Programme und Konfiguration       | Vollständige SHA-256-Bindung des Broker-/UI-Pakets und der wirksamen Start-/Sicherheitskonfiguration                  |
| Arbeitsablagenwurzel              | Kanonische lokale Rootbindung und deklarierte Datenformatversion; private Auflösung im nativen Teil                   |
| Richtlinie                        | Explizite Offline-Policy-Version; keine Netzfähigkeit durch geerbte Optionen oder unbekannte Felder                   |
| Transport                         | An diese Prozessgeneration gebundener einmaliger nativer Kanal; maximal 30 Sekunden monotone Bootstrapfrist im Broker |

Ein JSON, Pfad, Hash oder CLI-Flag allein ist niemals Autorität. Der Broker prüft die private
authentisierte Kontrollkette und bindet die erlaubten Kindhandles an die erzeugte Prozessgeneration.
Kontroll-MAC-Schlüssel verbleiben ausschließlich im Broker, niemals in UI/Bootstrap/Renderer. Ein
neuer Startframe benötigt eigene Zweck-/Versionsdomäne, keine Wiederverwendung einer Import-MAC als
Startfreigabe. Der konkrete Bootstrap-Transport und seine Peer-Prüfung sind vor Integration nativ
nachzuweisen; dieser Entwurf behauptet noch keine implementierte Authentisierung.

P2a implementiert Kontrollschema 5 mit `bindWorkspace`, `commitWorkspaceLaunch` und
`reconcileWorkspaceLaunch` ausschließlich im scoped Forschungschild. Dies ist kein normaler
Produktstart: UI-Start, Profil-/Containerfreigabe und nativer Bootstrap fehlen. Die `.kqprod`-
Deklaration Record-ABI 1/UI-ABI 0 grenzt das ausdrücklich ab; Legacy-`.kqmeta` bleibt für 1–4 und
Importstartticket 3 unverändert. Details/Byteformat im
[P2a-Nachweis](../testing/product-start-p2a-records.md). Vollständiges Produktpaket, effektive
Konfiguration und getrennte containerzugängliche Datenbereiche müssen vor P2b explizit gebunden
werden. Keine automatische Migration neuer EXEs oder Übernahme privater P2a-Fixtures. Die folgende
Zustandstabelle beschreibt weiterhin das Produktziel.

| Zustand / Ereignis                                 | Erlaubtes Verhalten                                                                                                                            |
| -------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------- |
| Keine Produktautorität                             | Nur ausdrücklich initialisieren; vorhandene Bestände zuerst erkennen, niemals als leeren Erststart übergehen                                   |
| Gültige Kontrolle, keine Arbeitsablage             | Neue Arbeitsablage oder ausdrücklich geplante Bestandskopie vorbereiten; noch keine Schreib-UI                                                 |
| Gebundene Arbeitsablage, kein Pending, Job leer    | Neue Startgeneration festschreiben und ausschließlich das gebundene UI-Paket starten                                                           |
| Laufende UI                                        | Kontrollsperre über ganze Joblebenszeit halten; konkurrierenden Start ohne zweite UI abweisen                                                  |
| Normales Schließen                                 | Jobende bestätigen; Arbeitsablage/Profil erhalten. Kein implizites `RetireSession` oder Löschen von Nutzdaten                                  |
| Unterbrochener Start                               | Alten Job abwarten und authentisiert abgleichen; frühere Launch-ID bleibt verbraucht. Neuer Start erhält eigene Generation, keinen Importgrant |
| Pending Update/Übernahme oder unklarer Job         | Kein neuer UI-Start, keine automatische Reparatur/Übernahme                                                                                    |
| Defekte/fehlende Schlüssel, widersprüchliche Kette | Gesperrt, keine Neuanlage über bestehende Daten und kein unisolierter Ersatzstart                                                              |

Ein Installations-/Arbeitsablagenlebenszyklus ist nicht gleich einer Importsitzung. Das heutige
Ein-Sitzungs-Modell und `MAX_RECORDS=256` des Forschungsstores sind keine ausreichende
Produkt-Langzeitlösung. Versionierte Aufbewahrung/Checkpointing mit erhaltener Replay-/Rollback-
Historie ist vor Produktfreigabe umzusetzen; Budgetende sperrt vor Kindstart, niemals stiller Reset.

## 4. Datenhoheit und Bestandsübernahme

Private Kontrollablage, verwaltete Programmpakete und containerzugängliche Arbeits-/WebView-Daten
bleiben getrennt. Der normale Datenpfad wird über native Betriebssystempfade und geprüfte lokale
Wurzeln bestimmt, nie über einen festen Laufwerksbuchstaben oder Rendererpfad. Bestehende normale
Appdaten, `.kraqel-broker-preview` und synthetische TEMP-Autorität werden nicht gegenseitig
adoptiert. Kein pauschaler AppContainer-ACL-Grant auf die bisherige Appdatenwurzel oder das
Benutzerprofil.

Für vorhandene Projekte ist eine **ausdrückliche, zerstörungsfreie Arbeitskopie** vorgesehen:

1. Vor UI-/Schreibzugriff feststellen, ob eine gültige Bindung oder mögliche Altbestände vorliegen.
   Fehler/fehlende Leserechte/unklare Herkunft als gesperrten Zustand behandeln, nicht als „leer“.
   Inventarisierung darf nicht den migrationsfähigen `Database::open`-Weg verwenden.
2. Native Übernahmeentscheidung an exakt geplante Quellen, Zielinstallation und Umfang binden. Eine
   bloße Renderer-Projekt-ID oder vorhandene Schreibberechtigung ersetzt diese Entscheidung nicht.
3. Quellen konsistent und ohne Änderung erfassen: SQLite-sichere Kopie sowie abgestimmte Assets,
   keine rohe Kopie einer möglicherweise aktiven WAL-Datenbank. Stillstand/Änderungserkennung muss
   technisch belegt sein; „altes Fenster geschlossen“ allein ist kein Nachweis exklusiven Besitzes.
4. Nur im getrennten Staging prüfen/migrieren. Projekt-, Dokument- und Entitäts-UUIDs der lokalen
   Arbeitskopie erhalten, Querverweise/Medien/Unicode und Schema prüfen. Keine stillen Auslassungen:
   Umgang mit Backups, Exporten, Git-Historie und weiteren Dateien vor Veröffentlichung festlegen.
5. Geprüften Zielbestand und Herkunftsbeleg authentisiert binden, dann atomar veröffentlichen.
   Wiederholung derselben Entscheidung liefert denselben Abschluss; keine zweite Kopie oder neue
   Importautorität. Alte Quelle bleibt unverändert und wird niemals automatisch gelöscht.
6. Abbruch/Crash vor bestätigtem Abschluss gibt keinen Zugriff auf Teilbestände. Verwaistes Staging
   nur anhand authentisierter Übernahmehistorie bereinigen; unbekannte Ordner nicht übernehmen.

Dies ist eine **noch zu implementierende lokale Übernahme**, kein bereits vorhandener Archivimport.
Der vorhandene Snapshot-v3-Import erzeugt absichtlich neue Projekt-UUIDs und transportiert keine
vollständige private Arbeitsumgebung/Git-Historie. Er darf als bewusster separater Kopierimport
angeboten werden, aber nicht als identitätserhaltende Vollmigration bezeichnet werden. Ein neues
leeres Projektfenster darf vorhandene noch nicht übernommene Projekte nicht wie Datenverlust
aussehen lassen; verständlicher Übernahme-/Sperrstatus, ohne Manuskriptpfade oder Diagnosen
offenzulegen.

## 5. Vor Fortsetzung des eingeschränkten Prozesses

- Aktuellen Actor/Kopf/Paketvertrag, unveränderte Paketbytes/Ressourcen und gültige Datenbindung
  prüfen. Hashbindung ersetzt keine Herausgebersignatur; unsignierter lokaler Releasepfad bleibt
  zulässig.
- Kontrollsperre und eigenes Jobobjekt halten, fremde Jobs niemals übernehmen. Eingeschränktes
  Root-Token mit erwarteter AppContainer-Identität und null Netzwerk-Capabilities vor Ausführung
  prüfen.
- Nur ausdrücklich erlaubte Handles vererben und Vererbbarkeit danach löschen; keine Geheimnisse in
  Umgebungsvariablen/Commandline, keine fremden WebView-Argumente oder Profilwurzel übernehmen.
- Die UI muss Bootstrap/Token/gebundene Daten vor Fenstererzeugung validieren. Kein Zugriff auf
  Hostkontrolle oder OS-Zugangsdaten; nur freigegebene Arbeitsdaten und getrennte WebView-Daten.
- Alte Commands nicht pauschal registrieren: Datei-/Shell-/Git-/Providerpfade einzeln durch die
  passende Autorität führen. Aktuell vorhandene Cloud-Opt-in-Logik ist kein Isolationsnachweis; auch
  lokale Provider-/Credential-/COM-/Shell-Pfade benötigen ihren eigenen strikten Nachweis.

## 6. Umsetzung und Abnahme

| Stufe                         | Konkretes Ergebnis                                                                                                   | Notwendige Prüfung                                                                                                                                                                    |
| ----------------------------- | -------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| P1 · umgesetzt als Testmodell | Reiner versionierter Start-/Bindungs-Zustandsvertrag mit neuen Typen; keine UI-/Datenübernahme                       | Zwecktrennung zu Import, fremder Kopf/App/Root, Replay, Budgetende, defekte Bindung; keine Autorität durch Parseerfolg                                                                |
| P2                            | Nativer synthetischer Produktstart ohne Importtask, persistente Arbeitsablage, neuer Launch nach normalem Ende/Crash | Wenige echte Start-/Abbruchfälle, unabhängige Token-/Job-/MAC-/Datenprüfung, eigener ETW-Buildbeleg                                                                                   |
| P3                            | Zerstörungsfreie Bestandskopie mit eigener idempotenter Übernahmeautorität                                           | Synthetisches Altprojekt mit WAL/Assets/UUIDs/Fehlerfällen, Quellen unverändert, keine Doppelkopie                                                                                    |
| P4                            | Vollständige Schreiboberfläche hinter vorgeschaltetem Startguard; Dateiaustausch/Unterprozesse einzeln integrieren   | Bei App-/gemeinsamen Transport-/Packagingänderungen `pnpm check`, geeigneter Build, native Funktions-/Assistenzprüfung und frisches ETW                                               |
| P5                            | Normaler Launcher/Installer ohne unisolierten Bypass, deklarierte kompatible Pakete, tragfähige Kontrollhistorie     | Direkter EXE-Start, Erststart/Bestandsstart, Neustart, Installation/Update/Deinstallation ohne Datenverlust; Produkt-Q-01/Q-02/F08/M3 und externe Dienstgrenzen ausdrücklich abnehmen |

P2 wird in P2a (persistente authentisierte Bindung/Launch-Records ohne GUI) und P2b (synthetischer
Bootstrap/Token/Job ohne Importtask) aufgeteilt. P2a ist als Record-only-Forschung umgesetzt; P2b
ist als begrenzter Readonly-Probeworker umgesetzt. Frisches ETW für den finalen Controller/Worker
ist gezielt belegt.
[Begrenzte Ressourcen-Reconciliation](../testing/product-start-resource-reconciliation.md) ist
implementiert; unbestätigte Profile/RunIntent ohne Drained bleiben gesperrt. Der
[eigene frische ETW-Nachweis](../testing/product-start-resource-reconciliation-etw.md) des
geänderten Runtimebuilds ist ergänzt.
[Dauerhafte Workerbindung und Wiederanlauf vor Drained](../testing/product-start-worker-lifetime.md)
ist inzwischen für den synthetischen Ein-Prozess-Worker umgesetzt. Der
[frische ETW-Beleg](../testing/product-start-worker-lifetime-etw.md) des Builds ist ergänzt.
Nächster begrenzter Scope P2c: persistente synthetische Arbeitsablage, Paketbindung und
Mehrprozess-UI-Start als getrennten Vertrag konkretisieren. Ungebundene Starts und mehrprozessige
GUI-Lebenszeiten sowie persistente echte Arbeitsablagen-/UI-Bindung fehlen; P2 insgesamt bleibt
offen.

Reihenfolge P1–P5 verhindert, dass der erfolgreiche Forschungsimport als Freigabe der gesamten
Schreiboberfläche gilt. Mittleres Prüfniveau bedeutet pro Implementierungsschritt gezielte relevante
Prüfungen, keine ungeprüfte Produktfreigabe. Für diesen Dokumentationsschritt genügen Quellen-/
Verweisprüfung, Prettier und `git diff --check`; keine neuen Builds/ETW-/Testmatrizen erforderlich.
