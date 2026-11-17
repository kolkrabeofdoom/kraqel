# P2c: persistente Arbeitsdaten, Paketbindung und Mehrprozess-UI

Der C4-Vertrag ist durch den [reinen C4a-Modellkern](../testing/product-start-p2c-c4a-model.md)
ergänzt: Paket-/Policytypen, deterministische Codierung und versionierter Lebenszyklus, zwölf neue
Tests. Keine native UI-/Startautorität; native UI-ABI bleibt 0. Nächster Schritt ist C4b mit nativer
Paket-/Overrideprüfung und belegbarem eingeschränktem Mehrprozess-Erzeugungsweg. C3-Bytes und deren
eigener ETW-Beleg bleiben separat unverändert.

Status: **Vertragsentwurf v1, 17. November 2026; C1-Modell, native C2-Datenvorbereitung und
synthetischer C3-Mehrprozess-Kern implementiert, UI-Umsetzung und Abnahme offen.**
[C1-Nachweis](../testing/product-start-p2c-model.md),
[C2-Nachweis](../testing/product-start-p2c-native-data.md),
[C3-Nachweis](../testing/product-start-p2c-group-bootstrap.md). Ergänzt den
[Produktstartvertrag](product-start-integration-v1-draft.md). Alle mit MUSS bezeichneten Regeln sind
Anforderungen an den nächsten Prototyp, keine Aussage über vorhandene Produktfähigkeiten. Das
strikte Datenschutzziel bleibt unverändert; Produkt-Q-01/Q-02 bleiben offen.

## 1. Ausgangspunkt und Grenze

| Verifiziert im Repository                                                                  | Noch offen für P2c                                                              |
| ------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------- |
| P1-Modell, private P2a-Schema-5-Records, P2b-Probeworker                                   | Persistente ausführbare Arbeitsablage und eigener UI-Startvertrag               |
| Worker v3 mit dauerhaftem PID-/Erzeugungszeitbeleg und Ein-Prozess-Job                     | Lebenszeitbeleg für die gesamte mehrprozessige Oberfläche                       |
| [Frischer ETW-Beleg](../testing/product-start-worker-lifetime-etw.md) für Build `37d7bec…` | Neuer Netzbeleg für die tatsächlich implementierten P2c-Binaries und Nachfahren |
| Normale App nutzt `app_local_data_dir()/projects`                                          | Gebundene Arbeitsdaten statt regulärem Appdaten-Fallback                        |
| Release konfiguriert WebView2-`offlineInstaller`                                           | Vollständige Bindung der effektiv geladenen Runtime und Ressourcen              |

Quellen im Code: [normaler Einstieg](../../apps/desktop/src-tauri/src/main.rs),
[Appsetup](../../apps/desktop/src-tauri/src/lib.rs),
[separate Import-Testoberfläche](../../apps/desktop/src-tauri/src/isolation_ui.rs),
[Releasekonfiguration](../../apps/desktop/src-tauri/tauri.release.conf.json),
[Forschungsimplementierung](../../scripts/runtime-isolation/probe/src/windows/exchange/ui_restart/durable/product.rs).
Die private P2a-UI-Kopie ist nur eine Fixture mit UI-ABI 0. Sie darf weder durch ACL-Erweiterung
noch durch Umdeutung ihres Markers zur ausführbaren Produktablage werden.

Scope: ausschließlich neue synthetische Arbeitsdaten. Echte Projekte/Vorschauablagen, Migration
bestehender Kontrollstores, normaler Appstart und Installer bleiben unverändert. P3 behandelt später
die identitätserhaltende Bestandskopie; Snapshotimport erzeugt dagegen neue IDs.

C2 bindet inzwischen neue synthetische Text-/Ressourcenfixtures und persistente Profile unter
derselben Kontrollsperre. Es stellt keine UI-EXE, Runtime oder SQLite-Datenbank bereit. Vollständige
ausführbare Paketclosure, effektive UI-Policies und Mehrprozess-Lebenszeiten bleiben C3/C4-Gates;
die folgenden Zielanforderungen werden dadurch nicht als insgesamt erfüllt erklärt.

C3 ergänzt inzwischen einen eigenen synthetischen Probevertrag für Root/echtes Kind mit
kontinuierlich gehaltenem Job und authentisiertem Continue. Eigene `.kqc3`-Probeversion, native
UI-ABI weiterhin 0: kein implementierter C1-UI-Startgrant. Gruppenschlüssel/MAC ist keine
unabhängige Schreiber-PID-Autorität; nicht auf untrusted Renderer übertragen. Die Prozess-/Token-/
Paket-/Jobprüfungen vor Resume und der gesperrte Wächterverlust sind im C3-Nachweis abgegrenzt.

## 2. Autorität und Ablagen

Ein langlebiger nativer **Wächter** hält Kontrollsperre, authentisierten Store und Jobhandle über
die gesamte UI-Lebenszeit. Er enthält keine WebView. Startende Clients und die eingeschränkte
UI-Prozessgruppe sind von dieser Autorität getrennt. Ein Importauftrag oder RestartTicket ist keine
Startberechtigung für eine normale Arbeitsoberfläche.

Illustrative lokale Anordnung; keine Bindung an einen Laufwerksbuchstaben:

```text
host/control/                         Schlüssel, Journal, exklusive Sperre
host/packages/<digest>/               privates Wächterpaket
ui-packages/<digest>/                 unveränderliches UI-/Runtimepaket
workspaces/<workspace-id>/projects/   SQLite, WAL/SHM, Assets, lokale Backups
workspaces/<workspace-id>/webview/    getrennte persistente WebView-Arbeitsdaten
sessions/<launch-id>/temp/            begrenzte temporäre Startdaten
```

- Kontrollablage, Schlüssel und Hostpaket bleiben ausschließlich im bisherigen vertrauenswürdigen
  Hostbereich. Die UI erhält weder Leserechte noch geerbte Schlüssel-/Sperr-/Jobhandles.
- Projekt- und WebView-Daten erhalten getrennte, explizite ACL-Policies für das gebundene Profil
  sowie Konto/SYSTEM. Keine allgemeinen AppContainer-Schreibrechte. Eine Profil-SID bindet eine
  Prozessgruppe, nicht exklusiv eine einzelne PID.
- Roots werden nativ auf lokalem Datenträger durch gehaltene Handles, kanonische Identität,
  Volume-/Dateiidentität und ACL-Policy gebunden. Reparsepunkte, Netzwerkziele und aus der UI
  stammende absolute Pfadautorität sind ausgeschlossen. Auch Hostzugriffe auf veränderliche
  Arbeitsdaten müssen diese Grenzen wahren, bevor sie fremden Verweisen folgen könnten.
- Normales Schließen behält Workspace-/Projekt-IDs, Profil und persistente Daten. Es löst kein
  RetireSession und keine Projektlöschung aus. Datenlöschung benötigt einen eigenen Vertrag und
  einen bestätigten Prozessgruppenabschluss.
- SQLite bleibt autoritativ und veränderlich: gebunden werden Root, Format-/Schemapolicy und
  gegebenenfalls synthetische Ausgangsdatenherkunft, nicht ein dauerhafter Datenbankhash. WAL/SHM
  und Backups bleiben im gebundenen Datenbereich. Datenbankschema 6 ist unabhängig von
  Kontrollschema und UI-ABI.
- Unbekannte Formate/Roots führen vor Datenbanköffnung zum gesperrten Start. `Database::open` kann
  migrieren und ist keine lesende Bestandsinventur. Keine automatische Reparatur, leere Ersatzablage
  oder Übernahme realer Appdaten.
- APPDATA, LOCALAPPDATA, TEMP/TMP, USERPROFILE und WebView-User-Data-Folder werden passend zu den
  getrennten Bereichen explizit gesetzt/geprüft. Keine Übernahme fremder Browserprofile, Debugports
  oder Runtime-/Argument-Overrides aus geerbter Umgebung. Synthetische Tests müssen verbleibende
  Windows-Pfadauflösung ausdrücklich prüfen; Umgebungsvariablen allein sind kein vollständiger
  Ablagenbeleg. Diagnoseprotokolle enthalten keine Manuskripte oder Schlüssel.

## 3. Paketidentität und Kompatibilität

Host- und UI-Paket erhalten getrennte vollständige Inventare. Zum UI-Inventar gehören ausführbare
Dateien, DLLs/Loader, Frontendressourcen, effektive Konfiguration und sämtliche mitgelieferten
WebView2-Runtimedateien. Windows-Systemkomponenten werden als explizite Plattformabhängigkeit
behandelt; das Inventar behauptet keine vollständige Bindung von Windows selbst.

Der erste Kandidat MUSS eine explizit ausgewählte Fixed-Version-Runtime verwenden. Fehlende oder
abweichende Runtime bedeutet Startabbruch, kein Evergreen-/Registry-/Installationsfallback. Die
aktuelle Offline-Installer-Konfiguration erfüllt diese neue Anforderung noch nicht. Microsoft
beschreibt die explizite Runtimepfadwahl und die gesonderte Pflege von Fixed-Version- Paketen in der
[Distributionsdokumentation](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/distribution).
Die Integration muss effektive Auswahl und Overrides nativ prüfen; eine gesetzte Variable allein ist
kein Nachweis.

Für Runtime-Kompatibilität erforderliche allgemeine AppContainer-Leserechte dürfen ausschließlich
als ausdrücklich inventarisierte RX-Policy des nicht geheimen Runtime-Unterbaums vorgesehen werden.
Microsoft dokumentiert solche Rechte insbesondere für bestimmte Windows-10-/Fixed-Version-
Konstellationen in derselben Quelle. Daraus folgen keine Rechte auf Arbeitsdaten, Hostpaket oder
Kontrollablage. Stille ACL-Aufweitung als Fehlerbehandlung ist ausgeschlossen.

Vorgeschlagener deterministischer Inventarvertrag v1:

- Domain `KRAQEL-P2C-PACKAGESET\0\x01`, Paketrolle als u8, Anzahl als u32 little endian.
- Einträge nach normalisierten relativen UTF-8-Pfadbytes sortiert; pro Eintrag u32 Pfadlänge,
  Pfadbytes, u64 Dateilänge, 32 SHA-256-Bytes und ein Rollenbyte. Paketrolle: Host=1, UI=2;
  Dateirole: Programm=1, Bibliothek=2, Ressource=3, Konfiguration=4. Unbekannte Rollen abweisen.
- SHA-256 über diese Codierung ergibt die Paketidentität; ein Selbstdigest ist kein Eintrag. Pfade
  sind NFC mit `/`; keine absoluten Pfade, `.`/`..`, ADS, reservierten Windows-Namen, abschließenden
  Punkten/Leerzeichen, Hardlinks, Reparsepunkte oder Windows-Fallalias-Dubletten.
- C1 prüft NFC ohne stilles Normalisieren und verwendet Unicode-Großschreibung als konservativen
  portablen Kollisionsschlüssel. Dies ist keine vollständige Windows-Namens-/Dateiidentitätsprüfung;
  sie bleibt zusätzlich unter nativen Handles für C2 erforderlich. C1 liest keine Dateien und kann
  weder Hardlinks noch tatsächliche Inventarvollständigkeit belegen.
- Grenzen des Entwurfs: 32.768 Dateien, 1.024 UTF-8-Bytes/Pfad, Tiefe 32, 512 MiB/Datei, 2 GiB
  Gesamtinhalt, 4 MiB Inventarcodierung; arithmetische Überläufe ablehnen. Exaktes Inventar, keine
  zusätzlichen ladbaren Dateien. Inventar liegt außerhalb des inventarisierten Paketbaums im
  authentisierten Hostbereich.
- Vor Start unter derselben Sperre vollständiger Abgleich und gegen Austausch geschützte
  Paket-/Vorfahrenhandles. Laden darf nicht aus schreibbaren Daten-/Tempverzeichnissen erfolgen.
  Erlaubte Programme/Kindrollen und Argumentpolicy gehören zur gebundenen Konfiguration.

Unsignierte lokale Pakete bleiben möglich; der Vertrag führt keine Zertifikatspflicht ein. Updates
wechseln Paketidentität und Bindungsrevision erst bei bestätigtem Jobleerstand und unter derselben
Kontrollsperre. Keine gemischten Paketgenerationen oder Ersetzung laufender Dateien.
Runtime-Sicherheitsupdates müssen lokal planbar bleiben; Pinning ist keine Zusage unbegrenzter
Sicherheit. WebView-Datenformatkompatibilität benötigt eine ausdrückliche Regel; kein ungeprüfter
Downgrade oder automatisches Löschen des Profils bei Inkompatibilität.

## 4. Geplante Typen, Versionen und Bootstrap

**Begrenzte neue native Datenversion:** C2 unterstützt Schema 6 ausschließlich über `InitializeC2`
für frische synthetische DATA-Stores und eigene `.kqc2`-Record-ABI 1. Keine 5→6-Migration; die
bisherigen PE-Deklarationen bleiben unverändert. P2c-UI-/Bootstrap-ABI 1 ist weiterhin nur
Zielvertrag des C1-Typenmodells, native UI-Unterstützung bleibt 0. C2-Bindungen mit Ziel-ABI 1 und
`no-execution`-Policy sind keine Startgrants. Eigene Vorbereitung-/Profil-/Bindungs-/Stilllegungs-
Records sind im [C2-Nachweis](../testing/product-start-p2c-native-data.md) abgegrenzt.

| Geplanter Typ        | Verbindlich gebundene Inhalte                                                                                                                                                                       |
| -------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `WorkspaceBindingV2` | Installations-/Workspace-ID, Profilname/SID, Rootidentitäten, ACL-Policy, Host-/UI-Paketdigest, effektive Konfiguration, Command-/Jobpolicy, Datenformat-/DB-Lese-/Schreibbereich, Bindungsrevision |
| `UiStartProposalV1`  | Zweck `workspace-ui-start`, erwarteter Journalhead/Revision, Launch-ID, Generation, Ablaufgrenze; untrusted Anfrage, keine Pfad- oder Importautorität                                               |
| `UiStartGrantV1`     | Vom Wächter nach nativen Prüfungen erzeugte Bindung des gesamten Tupels einschließlich Job-/Profilidentität, Nonce und Ablaufgrenze                                                                 |
| `UiRootBoundV1`      | Launch/Generation, native PID und Erzeugungszeit, gehaltene Jobidentität und geprüfte Token-/Policywerte vor Resume                                                                                 |
| `UiGroupDrainedV1`   | Dieselbe Generation/Bindung, kontinuierlich gehaltener Job und nativ bestätigter Leerstand; kein bloßer Root-PID-Endbeleg                                                                           |

Bootstrapdomain `KRAQEL-P2C-UI-START\0\x01`; eigene kurzlebige 32-Byte-Schlüssel, niemals der
dauerhafte MAC-Schlüssel. Frames maximal 8.192 Bytes, feste Rollen/Sequenzen und höchstens 30
Sekunden monotone Gesamtdauer. Unbekannte Felder, falscher Zweck, EOF, Replay und Zeitablauf führen
zum gesperrten Abschluss. Das C1-Modell verwendet begrenztes UTF-8-JSON mit Domain/Version, Rollen
Grant/Accepted/Continue und Sequenz 0/1/2. Alle Frames wiederholen dasselbe vollständige Granttupel;
dieses wird vor RootBound modelliert. Konkrete MAC-/Pipeintegration ist nicht umgesetzt. Details und
Feldtypen stehen im [C1-Nachweis](../testing/product-start-p2c-model.md) und dessen Quellverweisen.
Die Typen bilden keine neuen nativen Records oder Startberechtigungen.

Reihenfolge unter der Kontrollsperre:

1. Journal, Budget, Head/Revision, Pakete, Roots und Profil prüfen; LaunchCommit dauerhaft
   schreiben. Generation und Launch sind einmalig. Kein Start bei unklarem vorherigem Job.
2. Root suspendiert und bereits dem Job zugeordnet erzeugen; Token, Profil, Capabilityfreiheit,
   Handlevererbung und Paket prüfen. RootBound vor Resume dauerhaft speichern.
3. UI prüft den Grant nativ **vor Tauri, Datenbank und WebView** und meldet authentisiert
   `BootstrapAccepted`. Der Wächter persistiert diese Bestätigung und erteilt erst danach
   `Continue`. Erst jetzt dürfen gebundene Daten und die minimale Oberfläche geöffnet werden.
4. Nur vier ausdrücklich ausgewählte Pipe-Enden erben: Bootstrap-Lesen, Schlüssel-Lesen,
   Ereignis-Schreiben, Continue-Lesen. Native Peerprüfung gehört zum Protokoll; die eingeschränkte
   UI benötigt dafür keine Leserechte am privaten Wächterpaket. Alle anderen Handles bleiben privat.

Dauerhafte Records sind unter derselben Sperre serialisiert. Aktive UI erlaubt ausschließlich eng
definierte, nativ bewachte Lebenszyklusrecords; allgemeine Updates/Migrationen/Bereinigung bleiben
gesperrt. Das reine Modell darf niemals selbst eine native Berechtigung attestieren. Das heutige
256-Record-Forschungslimit ist keine Langzeitstrategie: benötigte Slots vorab reservieren und bei
Erschöpfung sperren. Checkpointing sowie Stromausfallgarantien bleiben offen.

## 5. Mehrprozess-Lebenszeit und Wiederanlauf

Die Ein-Prozess-Grenze des Workers v3 wird nicht einfach entfernt. P2c braucht eine explizite
endliche Prozessgrenze, KillOnJobClose, kein Breakaway sowie native Nachweise für alle benötigten
WebView-Rollen einschließlich verschachtelter Jobs und deren Token. Die zulässige Prozesszahl wird
erst mit dem gewählten Runtimekandidaten festgelegt und in die Jobpolicy gebunden. Der Start erfolgt
ohne Netzwerk-Capabilities und ohne Loopback-Ausnahme. Keine Firewall-/System- Lockerung und kein
unisolierter Ersatzstart bei Runtimefehlern. Diese Anforderungen müssen auch für die tatsächlich
erzeugten Kindrollen belegt werden; Paketpinning allein beweist keine Isolation.

Windows ordnet gewöhnliche CreateProcess-Kinder standardmäßig dem Job zu, kennt aber Ausnahmen;
Jobzugehörigkeit ist außerdem kein Ersatz für individuelle Tokenprüfung. Siehe
[Microsoft: Job Objects](https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects) und
[Nested Jobs](https://learn.microsoft.com/en-us/windows/win32/procthread/nested-jobs). Der Vertrag
leitet daraus die Anforderung ab, dass externe Aktivierung über Shell, WMI, COM oder Dienste keinen
unbelegten Ausweg eröffnen darf. Nicht nachweisbare Runtimepfade bleiben gesperrt.

| Zustand/Ereignis                                        | Zulässige Folge                                                                   |
| ------------------------------------------------------- | --------------------------------------------------------------------------------- |
| Gebundener Workspace, keine aktive Generation           | Neuen Launch nach vollständigen Prüfungen committen                               |
| Launch committed, native Startbindung unbekannt         | Pending/gesperrt; keine Ressourcenfreiheit vermuten                               |
| RootBound, Bootstrap noch nicht akzeptiert              | Nur kontrollierter Bootstrap oder Abbruch der gesamten Gruppe                     |
| Running, Startclient oder UI-Root beendet               | Wächter hält Sperre/Job weiter; keine Freigabe wegen Root-Ende                    |
| Closing, Kinder noch aktiv                              | Weiter sperren; eigene Gruppe kontrolliert beenden/drainen                        |
| Gleicher gehaltener Job nativ leer                      | Drained persistieren, danach Start abschließen; persistente Daten/Profil behalten |
| Wächter abgestürzt, kontinuierlicher Jobbesitz verloren | Erwartete Jobterminierung ist kein dauerhafter Endbeleg: pending/gesperrt         |

Ein terminaler Mehrprozessbeleg benötigt unter der gehaltenen Sperre dieselbe kontinuierlich
gehaltene Jobinstanz, passende Generation/Journalfolge und nativ abgefragten ActiveProcesses=0.
Jobnotifications und ETW dienen zur Diagnose, ersetzen diese Autorität aber nicht. Ein fehlender
Jobname, eine leere Momentaufnahme oder das Ende der Root-PID beweist keinen vollständigen Drain.
Der v3-PID-/Erzeugungszeit-Recoverybeleg bleibt auf seinen Ein-Prozess-Vertrag begrenzt.

Ein überlebender Wächter kann nach Client-/Root-Crash dieselbe Gruppe beenden und den Abschluss
authentisiert speichern. Nach **Wächtercrash** fehlt dieser kontinuierliche Nachweis; P2c behauptet
hier ausdrücklich noch keinen automatischen produktfähigen Wiederanlauf. Eine zusätzliche sichere
Recoveryautorität benötigt einen eigenen belegten Entwurf vor P4/P5. Kein Neuaufbau eines gleich
benannten Jobs, keine fremde Prozessterminierung, keine automatische Profillöschung als Ersatz.

## 6. Daten- und Befehlsgrenzen der ersten UI

Die erste Oberfläche darf ausschließlich synthetische interne Daten anzeigen/speichern. Der native
Commandrouter bietet eine minimale Allowlist; die breite Commandliste des normalen `run()` wird
nicht übernommen. Externe Pfade/Dialogs, Shell/Git, URL-Aufrufe, Credentials, Cloud/Ollama und
Netzwerkprovider benötigen eigene spätere Autoritäten und bleiben hier gesperrt. CSP ist ergänzend
und keine Netzsperre für den gesamten Prozessbaum.

Der
[WebView-User-Data-Folder](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/user-data-folder)
wird explizit gebunden und getrennt von Manuskripten gehalten. Speichern/Schließen/Neustart muss
stabile IDs und Daten erhalten, ohne einen zweiten gleichzeitigen Writer zuzulassen. Bei
inkompatibler DB-/Runtime-/Paketpolicy erfolgt eine verständliche lokale Fehlermeldung vor Mutation.
Die native Vorbereitung bleibt vor der GUI; spätere Fehler/Fokus/Ansagen benötigen eigene Prüfung.

## 7. Kleine Umsetzungsschritte und gezielte Abnahme

| Schritt                   | Nächster begrenzter Lieferumfang                                                                | Angemessene Prüfung                                                                                                                 |
| ------------------------- | ----------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------- |
| C1 — Modell geprüft       | Reine Typen/Codierung für Bindung, Inventar und Lebenszyklusmodell; keine neue native Autorität | 13 neue Modelltests; gesamter Helferlauf 88 aktiv bestanden, Clippy grün                                                            |
| C2 — Datenpfad geprüft    | Neue synthetische persistente Ablage/Profil, Inventarprüfung und Bindungsrecords                | Vier neue native Tests, 92 aktive Helfertests/Clippy, zehn gesicherte Controlleraufrufe; eigenes frisches ETW ergänzt               |
| C3 — Synthesekern geprüft | Wächter/Bootstrap mit Root und echtem Kind; keine UI-ABI-Unterstützung                          | Vier neue Tests, 96 aktive Helfertests/Clippy; 22 Controlleraufrufe und ein gesperrter Wächterabbruch; eigenes frisches ETW ergänzt |
| C4                        | Gepinnte WebView-Minimaloberfläche und synthetische SQLite-Persistenz                           | Passende App-/Transportprüfungen, gezielte assistive Prüfung und frisches ETW der exakten Binaries/Kindrollen                       |

Erforderliche Nachweise vor einer P2c-Freigabe: Daten/IDs/Profil bleiben über normalen Neustart
erhalten; private Kontrollfiles bleiben unlesbar; manipuliertes Paket/Runtime/Root wird vor
Datennutzung abgewiesen; keine unisolierte Ersatzoberfläche; kein Parallelwriter; laufende Kinder
halten die Sperre; falsche/alte Grants bleiben verbraucht oder abgewiesen; unbekannte Ressourcen
bleiben gesperrt. Ein klarer Erfolgsfall und gezielte Gegenfälle genügen pro neuem Vertragsteil;
keine Wiederholung unveränderter großer Crashmatrizen ohne neue Fragestellung.

Der spätere ETW-Vergleich benötigt eigenen Referenzlauf, positive Erfassungskontrollen,
Verlust-/Zuordnungsprüfung und vollständige Zuordnung der ausgewählten Controller-/UI-/Runtime-
Bäume einschließlich der Schließ-/Wiederanlaufpfade. Null beobachteter Verkehr ist ein Beleg für
diesen Build und diese Abläufe, keine pauschale Betriebssystem- oder Produktfreigabe. Persistente
echte Projekte, langfristiger Kontrollstorebetrieb, P3-Bestandskopie und normale
P4/P5-Produktintegration bleiben separate offene Gates.

## 8. Prüfung und verbleibende Grenze

Der ursprüngliche Vertragsentwurf wurde ausschließlich durch Quellen-/Link-/Format-/Diffprüfung
geprüft. Inzwischen sind C1 im getrennten Modellmodul und C2 als eigener nativer DATA-Pfad gezielt
getestet. Der normale Appstart und seine Befehle sind nicht integriert. Für den C2-Build ist
[frisches ETW](../testing/product-start-p2c-native-data-etw.md) ergänzt; alle Netzbelege behalten
ihre exakten Buildgrenzen. C3 ist inzwischen im synthetischen Umfang implementiert;
[dessen eigener frischer Netzbeleg](../testing/product-start-p2c-group-bootstrap-etw.md) ist
ergänzt. Der C4-Vertrag ist oben ergänzt; C4a ist oben belegt, C4b folgt als nativer Schritt. Die
gesamte P2c-Implementierung und Produktabnahme sind weiterhin offen.
