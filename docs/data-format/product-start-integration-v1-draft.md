# Produktstart und bestehende Projekte · Vertragsentwurf v1

[C4b-Root-/WebView-Probe](../testing/product-start-p2c-c4b-root-bootstrap.md) ergänzt
authentisierten Rootbootstrap und den tatsächlichen leeren Runtime-Kindpfad unter derselben
Kontrollsperre. Eigene versionierte C4-Probeactions, keine produktiven UI-Records/Startgrants;
native UI-ABI bleibt 0. C4b insgesamt bleibt offen: zwei API-unzugeordnete Generationen und
vollständige Runtime-/Nichtsystemmodul-/ effektive Rollenbindung vor Text-/Commandfreigabe, danach
eigener frischer Netzbeleg.

Der C4-Vertrag ist durch den [reinen C4a-Modellkern](../testing/product-start-p2c-c4a-model.md)
ergänzt: Paket-/Policytypen, deterministische Codierung und versionierter Lebenszyklus, zwölf neue
Tests. Keine native UI-/Startautorität; native UI-ABI bleibt 0. Nächster Schritt ist C4b mit nativer
Paket-/Overrideprüfung und belegbarem eingeschränktem Mehrprozess-Erzeugungsweg. C3-Bytes und deren
eigener ETW-Beleg bleiben separat unverändert.

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

### Vorprüfung zur SQLite-Erfassung bei P3

Am 24. November 2026 wurde eine ausschließlich synthetische SQLite-Quelle unter Windows im temporären Prüfverzeichnis angelegt. Sie verwendete die sechs Repository-Migrationen. Ein separater Schreibprozess bestätigte eine neue Projekt-UUID im WAL und endete anschließend ohne Schließen der SQLite-Verbindung. Während dieser Prozess noch lief, verweigerte Windows das zum Lesen geöffnete Handle mit ausschließlich Lesefreigabe (Sharing Violation 32).

Nach Prozessende hielt die Probe für sämtliche vorhandenen Dateien im festen Prüfverzeichnis solche Handles gleichzeitig offen. Nur Datenbank und WAL wurden aus diesen Handles in das leere Staging gelesen. SQLite öffnete ausschließlich diese Arbeitskopie zur Wiederherstellung und erzeugte dort eine separate Sicherung. Diese enthielt die im WAL bestätigte UUID und Schemaversion 6 und bestand die SQLite-Integritätsprüfung. Namen und SHA-256 sämtlicher Quelldateien einschließlich der SQLite-Nebendateien waren vor und nach der Erfassung gleich.

**Gegenprobe zur Verzeichnisbindung.** In einer zweiten synthetischen Quelle hielt der Prüfprozess zugleich das Verzeichnis und das vorhandene Asset mit ausschließlich Lesefreigabe offen. Trotzdem konnte ein separat gestarteter Prozess ein neues Asset in diesem Verzeichnis anlegen. Die Dateihandles sichern daher nur die bereits erfassten Objekte. Eine zunächst vollständige Namensliste darf keine Veröffentlichung eines Gesamtprojekts autorisieren. Der nächste Entwurf muss eine über die gesamte Erfassung wirksame Quelle mit stabilem Dateibestand nachweisen.

Das Ergebnis belegt den SQLite-Teil für diese fest vorgegebene Quelle. Für eine Bestandskopie fehlen weiterhin die authentisierte native Quellen-/Zielbindung, der Ausschluss von Veränderungen am gesamten Verzeichnisbestand einschließlich neuer Dateien und Assets sowie die idempotente Veröffentlichung. Bis dahin gibt es keinen Produktaufruf zur Bestandsübernahme. Bei unbekannten oder unvollständig erfassten Dateien bleibt die Übernahme vor der Veröffentlichung gesperrt.

### Stabiler Dateibestand auf einem synthetischen Quelldatenträger

**Windows-Nachweis vom 24. November 2026.** Das Projekt wurde unmittelbar auf einer neu erzeugten lokalen NTFS-VHDX angelegt. Nach dem Aushängen hielt die Prüfsteuerung die VHDX-Datei mit einem Lesehandle ohne Schreib- und Löschfreigabe fest und band sie ausschließlich lesend ein. Sie prüfte Datenträgerstatus, Partitionskennung und Volume-GUID. Der Kopierprozess prüfte das Schreibschutzflag über ein gehaltenes Volume-Handle vor der ersten Inventarisierung. Diese Bindungen bestanden bis nach der Zielprüfung.

Ein zweiter Prozess konnte in der beschreibbaren Vorbereitungsphase je ein neues Asset im Projektstamm und im verschachtelten Assetverzeichnis anlegen. Während des Kopierens der bereits geöffneten Datenbank scheiterten dieselben beiden Erzeugungsaufrufe mit Windows-Fehler 19 (`ERROR_WRITE_PROTECT`). Erfasst wurden relative Namen, Verzeichnisse einschließlich eines leeren Verzeichnisses und die SHA-256-Werte der regulären Dateien. Ein zusätzlich angelegter benannter NTFS-Datenstrom sperrte die Inventarisierung. Reparse Points und mehrfach verknüpfte Dateien werden im Prüfpfad ebenfalls abgewiesen.

Im getrennten Staging stimmten Inventar und Bytes mit der synthetischen Vorgabe und der Quelle überein. SQLite-Integrität, Fremdschlüssel, Projekt-, Dokument- und Medien-UUIDs sowie die Verweise auf Original und Vorschaubild wurden geprüft. Ein unbekannter oder nicht bestätigter Zustand gibt keine Kopie frei. Die Erfassungsprüfung endet im Staging. Die anschließende Veröffentlichung am synthetischen Ziel ist unten beschrieben.

Der Nachweis gilt für eine Quelle, deren **gesamter Datenträger bereits vor Beginn der Erfassung** wirksam schreibgeschützt ist. Eine VHDX aus Dateien eines während der Erfassung veränderlichen Projektordners zu befüllen, erfüllt diese Voraussetzung nicht. Gewöhnliche Bestandsordner bleiben gesperrt. Offen bleiben die produktive Herkunfts- und Zielbindung, eine stabile Erfassungsgrenze für solche Ordner sowie die idempotente Veröffentlichung. Eine Authentisierung über Neustarts ist durch das für den Prüflauf gehaltene Handle nicht belegt.

Die [gezielte Windows-Probe](../../scripts/runtime-isolation/check-p3-readonly-volume.ps1) benötigt Hyper-V-VHD-Befehle, Python und lokale administrative Rechte für das Einbinden ihres eigenen Datenträgers. Sie nimmt keine Projekt- oder Datenträgerpfade entgegen. Die beiden Negativkontrollen und die Prüfung der einen Kopie laufen gemeinsam, außerhalb des regulären Testrunners.

### Erfassung eines beschreibbaren synthetischen Projektordners

Der lokale Prüfeinstieg `python -I scripts/runtime-isolation/p3-volume-fixture.py writable` erzeugt einen eigenen NTFS-Ordner auf einem festen Laufwerk. Vor jeder Verzeichnisauflistung fordert die Prüfsteuerung einen Lese-Oplock an. Sie hält die Handles für Verzeichnisse und Dateien gemeinsam bis nach dem letzten Lesezugriff und prüft alle Oplocks auf Unterbrechung. Erst danach schreibt sie den im Speicher erfassten Bestand in ein separates Staging, prüft dort das Inventar und benennt den fertigen Ordner um. Beide Erfassungsversuche prüfen die Identität des ursprünglich ausgewählten Stammordners. [Windows-Vertrag für Oplocks](https://learn.microsoft.com/en-us/windows/win32/api/winioctl/ni-winioctl-fsctl_request_oplock)

Ein zweiter Prozess legte während des Lesens ein Bild in einem bereits inventarisierten Unterordner an. Der erste Versuch brach ohne Zielordner ab. Die frische Erfassung enthielt das neue Bild, leere Verzeichnisse und den im SQLite-WAL bestätigten Szenenstand. Das Speicherbudget beträgt 32 MiB, die Einzeldateigrenze 8 MiB. Der Nachweis gilt für den kontrollierten synthetischen NTFS-Bestand. Die synthetische Prüfsteuerung hält den ausgewählten Stammordner während der erfolgreichen Erfassung geöffnet. Vor der Umbenennung des Stagings registriert eine eigene lokale Prüfinstallation einen domänengebundenen SHA-256-Wert im authentisierten Kontrolljournal. Er umfasst die Volume- und Dateiidentität des Quellordners und des Kandidaten, die fachlichen Kennungen und das vollständige Inventar. Nach dem Neustart prüft der Starter den Wert vor der ersten Veröffentlichung. Die anschließende Übernahmeentscheidung enthält ebenfalls die ursprüngliche Quellidentität. Ein anderer Quellordner bei ansonsten identischem Kandidaten und Inventar wurde nach Neustart abgewiesen. Der kurze Lauf benötigt das lokal gebaute Debug-Prüfprogramm.

Mit `python -I scripts/runtime-isolation/p3-volume-fixture.py select-writable` bietet ein nativer Windows-Dialog ausschließlich die vorbereitete synthetische Quelle an. Ihr Verzeichnishandle bleibt vom Anzeigen des Dialogs bis zum Ende der Erfassung offen. „Abbrechen“ ist voreingestellt. Beim Abbruch endet der Auftrag vor Erfassung, Zielanlage und Journalregistrierung. „OK“ erlaubt einen Erfassungsversuch. Der lokale automatisierte Bedienlauf hat beide Schaltflächen am Dialog des jeweiligen Prüfprozesses ausgelöst. Die bestätigte Kopie enthielt die im WAL gespeicherte Szene.

Für reguläre Projekte bleibt die freie Ordnerauswahl in der installierten Anwendung offen. Das synthetische Verfahren setzt die vertrauenswürdige Prüfsteuerung und die zugehörige Kontrollablage voraus.

### Einmalige Veröffentlichung am gebundenen synthetischen Ziel

Im Windows-Prüflauf vom 24. November 2026 bindet die Prüfsteuerung eine Entscheidung per HMAC an Volume-Pfad und Verzeichnisidentität der Quelle, Verzeichnisidentität des privaten Prüfziels, Projekt-, Dokument- und Medienkennungen sowie das vollständige erwartete Inventar. Der Kopierprozess prüft selbst den Schreibschutz und hält Quelle und Zielverzeichnis während der Operation offen. Die geprüfte Kopie und ihr Abschlussbeleg werden gemeinsam durch Umbenennen eines Verzeichnisses an den festen, aus der Projekt-UUID abgeleiteten Zielplatz veröffentlicht. Der Beleg hat eine eigene HMAC-Domäne und bindet die Entscheidung und die Dateiidentitäten beider veröffentlichten Verzeichnisse. Die Wiederholung prüft Beleg und tatsächliches Inventar am selben Ziel.

Der Kopierprozess endete nach der Veröffentlichung mit Exitcode 86 vor der Rückmeldung. Mit derselben Entscheidung fand der Folgeaufruf genau das vorhandene Verzeichnis. Ein fremdes Ziel blieb leer. Eine neue Prüfautorität mit frischem Schlüssel konnte den vorhandenen Beleg auch mit einer neu ausgestellten Entscheidung nicht bestätigen. Nach Änderung des Zielbildes scheiterte ebenfalls die Wiederholung. Die temporäre Probe wurde im vorhandenen gezielten Lauf ausgeführt.

**Abbruch während der Kopie.** Für die Projekt-UUID wird ein eigener Stagingplatz exklusiv angelegt. Der Kopierprozess endete im separaten synthetischen Ziel nach dem ersten geschriebenen Byte mit Exitcode 85. Beim erneuten Aufruf war allein dieser unvollständige Bestand vorhanden. Die Wiederholung wurde vor neuer Zielanlage abgewiesen, sein Inventar blieb bytegleich. Die Probe verleiht keine Berechtigung zur Bereinigung oder Fortsetzung eines solchen Restbestands.

**Ausdrückliche Fortsetzung im kontrollierten Prüfzielverzeichnis.** Vor dem ersten Kopierbyte legt der Prüfprozess die vorgesehenen Dateien und Verzeichnisse an. Ein eigener HMAC-Beleg bindet ihre relativen Namen und Dateiidentitäten sowie die Identitäten des Stagingverzeichnisses und seines Projektordners an die ursprüngliche Entscheidung. Die kooperierenden Prüfprozesse verwenden während der Operation einen Windows-Mutex pro Zielverzeichnis und Projekt. Beim regulären Aufruf bleibt vorhandenes Staging gesperrt.

Beim ausdrücklich angeforderten Wiederanlauf prüft der Prozess die schreibgeschützte Quelle, den Beleg, den vollständigen Namensbestand und die bisherigen Dateianfänge. Sämtliche Zieldateien werden vor der ersten Fortsetzung exklusiv geöffnet und bleiben während der Übertragung gehalten. Nur wenn jeder vorhandene Anfang mit den Quellbytes übereinstimmt, ergänzt die Probe die fehlenden Bytes. Bei bereits geschriebenem Abschlussbeleg prüft sie stattdessen das fertige Paket samt Inventar und veröffentlicht genau dieses Verzeichnis.

Der Windows-Lauf beendete den Kopierprozess nach dem ersten Byte (Exitcode 85), den folgenden Wiederanlauf vor dem Umbenennen des vollständigen Pakets (87) und einen separaten Lauf direkt nach der Veröffentlichung (86). Eine fremde Zusatzdatei, ein falsches Anfangsbyte und ein ungültiger Abschlussbeleg sperrten jeweils den entsprechenden Schritt. Nach Rücknahme der gezielten Störungen durch die Prüfsteuerung war das veröffentlichte Verzeichnis identisch mit dem vorbereiteten Objekt. Fortsetzung und Bestätigung verleihen keine Löschberechtigung. Für Restbestände ohne eindeutigen Beleg bleibt die Sperre bestehen.

Im bisherigen Kopierprozess-Test existiert der Schlüssel nur während der Lebensdauer der synthetischen Prüfsteuerung. Die Windows-Probe belegt die Wiederholung nach Ausfall des Kopierprozesses in einem kontrollierten temporären Ziel. Für den Produktstarter fehlen insbesondere die dauerhafte Bindung an eine Installation und deren Kontrollsperre, eine authentisierte Bereinigung gesperrter Stagingreste und die Freigabe einer Arbeitskopie zur späteren Bearbeitung. Normale Bestandsordner bleiben gesperrt.

### Wiederanlauf mit installationsgebundenem Kontrolljournal

Für ein einzelnes synthetisches Projekt hält der native Starter die Kontrollsperre seiner Testinstallation während des lokalen Pipe-Dialogs. Im authentisierten Journal stehen drei unveränderliche Hashwerte in fester Reihenfolge für Entscheidung, Stagingbestand und Abschluss. Der Schlüssel verbleibt im Starter. Ein neuer Prozess derselben Installation muss die vorherigen Belege erneut prüfen, bevor er den fehlenden Abschluss eintragen darf.

Der gezielte VHDX-Lauf beendet den nativen Starter nach dem ersten Kopierbyte und nochmals nach Veröffentlichung. Die explizite Fortsetzung prüft die Dateianfänge und die Reservierung. Nach dem zweiten Startabbruch muss die Prüfung dasselbe veröffentlichte Verzeichnis bestätigen. Eine getrennt initialisierte Installation mit identischer Starterdatei muss die ursprüngliche Entscheidung ablehnen. Der Kopierprozess und das temporäre Ziel gehören zum kontrollierten Prüfaufbau. Produktablagen, Stromausfälle und die Bereinigung unklarer Restbestände sind außerhalb dieses Nachweises.

**Ausdrückliche Bereinigungsentscheidung im Prüfaufbau.** Unter Kontroll- und Zielsperre prüft der Kopierprozess Quelle, Ziel, Reservierung, Objektidentitäten und sämtliche Dateianfänge. Ein separater Starteraufruf bindet die Entscheidung an die geprüften Präfixlängen und die Identität der geöffneten Reservierungsdatei. Im Journal schließen sich Veröffentlichung und Bereinigungsentscheidung gegenseitig aus. Bei einer fremden Zusatzdatei blieb die Freigabe aus. Nach Wiederöffnen ließ sich dieselbe Entscheidung bestätigen, der Übernahmestart blieb gesperrt. Die Bereinigungsentscheidung selbst entfernt keine Datei.

**Entfernung und Abschluss im synthetischen Ziel.** Eine lokale Begleitdatei hält den bestätigten Plan über das Verschwinden des Stagings hinaus verfügbar. Der Starter prüft dessen Bindung an Reservierung, Quelldaten und Installation erneut. Der Kopierprozess vergleicht alle verbliebenen Namen, Windows-Dateiidentitäten und Dateianfänge vor der ersten Entfernung. Er hält die geprüften Handles und entfernt die Objekte über diese Handles in fester Reihenfolge. Vor jedem Objekt schreibt der Starter den nächsten erlaubten Schritt ins Kontrolljournal. Nach einer Unterbrechung darf höchstens dieser eine Schritt begonnen und noch nicht ausgeführt sein. Sobald das Staging fehlt und der letzte Schritt belegt ist, kann der Starter den Abschluss dauerhaft bestätigen. Ein später am selben Namen angelegtes Verzeichnis bleibt gesperrt.

Die Windows-Probe beendet den Kopierprozess direkt nach der ersten Entfernung. Das ungeplante Fehlen des nächsten Objekts und eine bytegleiche Ersatzdatei sperren die Fortsetzung. Nach Rückgabe des ursprünglichen Objekts kann der Folgeprozess abschließen. Der Prüfaufbau verwendet eine kontrollierte temporäre Zielablage und einen schreibgeschützten synthetischen Quelldatenträger. Der Abschluss betrifft den vorgesehenen Stagingplatz. Er belegt weder sichere physische Datenlöschung noch Verhalten bei Stromausfall.

**Erster Lesezugriff auf die veröffentlichte Kopie.** Der Prüfaufruf `broker-store read-p3-fixture` verlangt den gespeicherten Abschlussbeleg. Er erlaubt am lokalen Kanal ausschließlich die Prüfung der drei bereits gebundenen Werte. Der Kopierprozess hält die Zielsperre, prüft das veröffentlichte Verzeichnis und sein Inventar und hält die Datenbank dabei über ein Windows-Lesehandle gegen Schreiben und Ersetzen gesperrt. Für eine einzelne Datenbank im Rollback-Modus liest SQLite das geprüfte Hauptabbild im Arbeitsspeicher. Im WAL-Fall hält die Prüfsteuerung Hauptdatei, WAL und zugehörigen SHM-Index gleichzeitig gegen Schreiben und Ersetzen gesperrt und prüft ihre SHA-256-Werte. SQLite öffnet die Kopie lesend und übernimmt die bestätigte Transaktionssicht mit seiner Backup-API in eine speicherinterne Datenbank. Die Grenze beträgt acht MiB je SQLite-Datei und acht MiB für die logische Datenbank. Fehlende WAL-/SHM-Partner und vorhandene Rollback-Journale sperren diesen synthetischen Pfad. Die feste Abfrage liefert Titel, Klartext und Revision genau der in der Übergabe gebundenen Projekt- und Szenen-UUID. Der Prüflauf erwartet bei der vorbereiteten Szene den Titel »Szene A«, den Klartext »Text aus der Testkopie.« und Revision 0. Während der unterbrochenen Übernahme bleibt der Leseaufruf gesperrt. Die SQLite-Abfrage läuft in der synthetischen Prüfsteuerung.

Die WAL-Probe speichert zunächst »Vor dem WAL-Commit« in der Hauptdatei. Ein separater Prozess bestätigt »Text aus der Testkopie.« im WAL und endet ohne Schließen der Verbindung. Der unveränderte Hauptdateihash belegt, dass der neue Text beim Kopieren noch im WAL liegt. Staging und Vorschau prüfen den bestätigten Text. Die Quelldateien bleiben bis zum abschließenden Inventarvergleich auf dem schreibgeschützten Datenträger.

**Übergabe an den isolierten Datendienst.** Schon bei der Übernahme bindet die Entscheidung den Digest des vorgesehenen Dienstprogramms und den festen Datensatz aus Projekt-UUID, Szenen-UUID, Revision, Titel und Text. Nach Prüfung der drei Journalbelege vergleicht der Starter die tatsächlich gelesene Zeile und das gepinnte Dienstprogramm mit dieser Bindung. Der suspendierte Prozess durchläuft die bestehende AppContainer-, Image- und Jobprüfung. Erst nach der Bootstrap-Quittung sendet der Starter die Zeile über die lokale Pipe. Er akzeptiert den Empfang nach passender Inhaltsquittung, regulärem Prozessende und leerem Job. Die Datenbank, Pfade und der Installationsschlüssel verbleiben außerhalb der Dienstrolle. Eine ausgetauschte Szenen-UUID sperrt die Übergabe vor dem Start. Eine alte Übernahmeentscheidung ohne diese zusätzliche Bindung erteilt keine Übergabefreigabe. Vor dem Rollenstart schreibt der Starter den an diese Übernahme gebundenen Sitzungsbeginn dauerhaft ins Kontrolljournal. Erst nach Empfangsquittung, Prozessende, leerem Job und Profilbereinigung verbucht er das Ende für genau diesen Beginn. Der Prüflauf beendet den Starter nach dem dauerhaften Beginn vor der Startbestätigung. Der folgende Leseaufruf bleibt wegen des fehlenden Endebelegs gesperrt. Die Prüfung gilt für den festen synthetischen Datensatz. Für die Fenstervorschau bindet die Entscheidung in Version 2 zusätzlich die Fenster-EXE an die Rollenpaarung. Eine frühere Dienstfreigabe reicht für diesen Pfad nicht aus.

**Schreibgeschützte P3-Fenstervorschau.** Beide Rollen laufen in eigenen AppContainern. Der Starter installiert im geprüften, suspendierten Fensterprozess ausschließlich das Leseende der Szenenpipe und im Datendienst das Schreibende. Der Dienst schreibt die gebundene Zeile in die Pipe und quittiert sie. Nach seinem regulären Ende erhält das Fenster die Startfreigabe und den erwarteten Inhaltsdigest. Es gleicht die empfangene Zeile ab und liest Text und Schreibschutzstatus am nativen Eingabefeld zurück. Der Starter prüft das sichtbare Fensterhandle gegen Prozess, Thread und Fensterklasse, schließt das Prüffenster und bestätigt das Sitzungsende nach der Bereinigung beider Rollen. Der Prüfpfad ist auf den festen synthetischen Datensatz beschränkt.

**Bedienbare Vorschau.** In einer erhöhten PowerShell öffnet `./scripts/runtime-isolation/check-p3-readonly-volume.ps1 -InteractivePreview` aus dem Repository die veröffentlichte synthetische Szene. Der Schalter gilt ausschließlich für die erfolgreich gebundene Zeile. Das Fenster bleibt bis zum Schließen geöffnet. Ohne Schalter schließt die Prüfsteuerung das Fenster automatisch. Erforderlich sind die im Skript genannten lokalen Builds für Starter, Fenster und Datendienst.

Der vollständige VHDX-Lauf bestätigte die automatische Vorschau und die Abweisung einer ausgetauschten Szenenkennung und Fenster-EXE. Ein gesonderter lokaler Rollenlauf prüfte die neue Warteoption am sichtbaren Fenster. Während der Anzeige blieb ein paralleler Leseaufruf gesperrt. Nach regulärem Schließen ließ sich die Vorschau erneut öffnen. Ein gezielt beendeter Starter riss die sichtbare Fensterrolle mit. Der folgende Leseaufruf blieb wegen des fehlenden Endebelegs gesperrt. Für früher erzeugte Vorschauen fehlt der nachträgliche Eigentumsbeleg.

**Profilbereinigung nach Starterabbruch.** Bei neuen Vorschauen bindet der authentisierte Sitzungsbeginn das lokale Paketverzeichnis. Der Starter leitet die beiden Profilnamen aus dem Sitzungsbeginn und der jeweiligen Rolle ab. Nach erfolgreicher Windows-Profilanlage und vor dem Rollenstart protokolliert er einen Fingerabdruck der Dateiidentitäten des Profilverzeichnisses und seines Unterordners `AC`. Beim regulären Ende wird die Entfernung im selben Kontrolljournal festgehalten.

Der gesonderte Prüfaufruf `broker-store retire-p3-preview-profiles` verwendet die Kontrollsperre der Testinstallation. Vor der ersten Entfernung müssen beide Rollenjobs leer oder verschwunden sein. Ein vorhandenes Profil braucht den passenden Anlagebeleg. Die Abwesenheitsprüfung umfasst die Windows-Profilauflösung und den konkreten Platz im gebundenen Paketverzeichnis. Der Aufruf erteilt keinen Sitzungsendebeleg. Nach Abbruch bei sichtbarem Fenster blieben der Lesezugriff gesperrt und ein ausgetauschtes Profilverzeichnis unangetastet, selbst mit dem ursprünglichen `AC` darin. Nach Rückgabe des Originalverzeichnisses entfernte der Starter genau ein Profil. Beim Folgeaufruf meldete er null Entfernungen. Der Nachweis gilt für die kontrollierte synthetische Testinstallation. Bei einem Abbruch zwischen Profilanlage und dauerhaftem Anlagebeleg bleibt ein vorhandenes Profil zur Klärung erhalten.

Bei einem verweigerten Abschluss behält der Runner die primäre synthetische Testinstallation samt Kontrolljournal unter der ausgegebenen Prüfkennung. Das Aufräumen des temporären Datenträgers erhält den ursprünglichen Fehler. Meldet der Broker beim Prüfen der Rollenjobs noch aktive Prozesse, wiederholt der Runner den Aufruf kurzzeitig. Eine fehlende oder abweichende Profilidentität führt sofort zum Abbruch.

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
[frische ETW-Beleg](../testing/product-start-worker-lifetime-etw.md) des Builds ist ergänzt. Der
getrennte [P2c-Vertrag](product-start-p2c-v1-draft.md) für persistente synthetische Arbeitsdaten,
Paketbindung und Mehrprozess-UI-Start ist als Entwurf ausgearbeitet, nicht nativ umgesetzt.
[C1](../testing/product-start-p2c-model.md) ergänzt geprüfte reine Typen, Inventarcodierung und
Lebenszyklusmodell. [C2](../testing/product-start-p2c-native-data.md) ist als begrenzter nativer
Datenpfad umgesetzt: frische Schema-6-DATA-Stores, persistentes Profil und Paket-/Rootbindung, keine
Migration oder UI-Startautorität.
[Frisches ETW dieses Builds](../testing/product-start-p2c-native-data-etw.md) ist ergänzt.
[C3](../testing/product-start-p2c-group-bootstrap.md) ist als begrenzter synthetischer Mehrprozess-
Kern implementiert; [frisches C3-ETW](../testing/product-start-p2c-group-bootstrap-etw.md) ist
ergänzt. Der C4-Vertrag ist oben ergänzt; C4a ist oben belegt, C4b folgt als nativer Schritt.
Ungebundene Starts und mehrprozessige GUI-Lebenszeiten sowie persistente echte
Arbeitsablagen-/UI-Bindung fehlen; P2 insgesamt bleibt offen.

Reihenfolge P1–P5 verhindert, dass der erfolgreiche Forschungsimport als Freigabe der gesamten
Schreiboberfläche gilt. Mittleres Prüfniveau bedeutet pro Implementierungsschritt gezielte relevante
Prüfungen, keine ungeprüfte Produktfreigabe. Für diesen Dokumentationsschritt genügen Quellen-/
Verweisprüfung, Prettier und `git diff --check`; keine neuen Builds/ETW-/Testmatrizen erforderlich.
