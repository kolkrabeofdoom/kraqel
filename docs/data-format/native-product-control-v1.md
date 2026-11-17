# Native Broker-Kontrollablage und lokale Updates · v1

Stand: 17. November 2026. Implementierte Windows-Vorschau im nativen Forschungsbroker, noch kein
normaler Produktstarter. [Funktionsnachweis](../testing/native-product-control.md),
[Bedrohungsmodell](../architecture/threat-model.md). Striktes Datenschutzziel unverändert.

## Ablage und Vertrauensgrenze

Fester Standardpfad: `%USERPROFILE%\.kraqel-broker-preview`. Ein lokales Profilverzeichnis wird vor
I/O in eine explizite lokale Win32-Pfadform überführt. Nur feste lokale Laufwerke, keine UNC-/
Gerätenamensräume, relativen oder übergeordneten Komponenten, Reparse-/Cloud-/Offlinepfade. Alle
Vorfahren und Kontrollverzeichnisse werden ohne Delete-Sharing gepinnt. Die kanonische
UTF-16LE-Wurzel ist Bestandteil jedes MAC-Records. Andere Rechner/Konten benötigen eine neue Ablage;
keine Übertragung von Kontrollschlüsseln oder automatische Übernahme von Forschungswurzeln.

AppData wird hier bewusst nicht verwendet: Ein paketierter Startkontext kann den nominalen Pfad in
seinen eigenen Cache auf einem anderen Laufwerk umleiten. Das wäre keine vom Coding-Werkzeug
unabhängige dauerhafte Produktablage. Der tatsächliche Profilpfad wird unabhängig gegengeprüft.

```text
.kraqel-broker-preview/
  control/
    authority.key        # 32 native Zufallsbytes, ausschließlich bei Neuanlage
    owner.lock           # exklusive native Sperre, kein automatisches Neuerzeugen
    records/0000.mac …   # unveränderliche lückenlose Übergangsfolge, maximal 256 Records
    head.mac             # eigener authentisierter Abschlussanker
    head.next            # nur während Abschluss; Fund bei Neustart blockiert
  bin/<sha256>.exe       # geprüfte Kopie einer ausdrücklich freigegebenen lokalen x64-EXE
  data/                  # reserviert; noch keine Übernahme vorhandener Projekte/Journale
```

Verzeichnisse erhalten bereits bei Erzeugung eine geschützte DACL ausschließlich für das aktuelle
Konto und SYSTEM. Die ACL wird vor Schlüsselerzeugung und bei jedem Öffnen strukturell geprüft;
Dateien dürfen ebenfalls nur diese beiden Full-Control-SID-Einträge besitzen. Keine Paket-SID, ALL
APPLICATION PACKAGES, Everyone oder Users. Schlüssel-/Kontrollfile-Hardlinks werden abgewiesen.
Fehlende, beschädigte oder zu weit freigegebene Ablage wird nicht repariert oder neu initialisiert.
Schlüssel-/Lesepuffer verwenden `Zeroizing`; keine Schlüssel in Renderer, Logs oder Git.

Auch `data/` bleibt in dieser Vorschau privat und leer. Eine spätere App-Datenfreigabe benötigt ein
ausdrücklich migriertes Layout mit getrennten Rollen; nicht einfach die Kontrollwurzel um
Containerrechte erweitern. Die aktuelle ACL-Prüfung würde eine solche Aufweichung abweisen.

Die native Kontrollablage ist eine Vertrauenskomponente des Kontos. Sie schützt nicht vor
kompromittiertem Konto, Administrationsrechten oder beliebigem voll privilegiertem Hostcode. Die
normale App ist weiterhin unisoliert. Echte AppContainer-Dateiverweigerungen sind separat geprüft;
diese ersetzen keine Produktintegration oder Netzabnahme.

## Authentisierung und Abschluss

Envelope: `KRAQEL-PRODUCT-CONTROL\0\x01`, 32-Byte-HMAC-SHA256 über Domain und Payload, dann Payload.
Payload maximal 8.192 Bytes. JSON mit strikten unbekannten-Feld-Abweisungen. Jeder Record bindet
Formatversion 1, Sequenz, Installations-UUID, Wurzelhash, Vorgänger-Dateihash, einmalige Operations-
UUID, bisher autorisierten Programmhash und typisierte Aktion. Kein Pfad ist Teil einer
Renderer-API.

Der Kopf-Payload beginnt zusätzlich mit `HEAD\0` und bindet Installation, Wurzel, Sequenz und
vollständigen letzten Recordhash. Domain und Typprüfung trennen ihn von Records und von den älteren
Forschungs-MACs. Lücken, fremde MACs, andere Wurzeln, unbekannte Versionen, wiederholte
Operationskennungen und fehlende Schlussrecords blockieren.

Schreiben: Record mit `create_new`, vollständiger Write und `sync_all`; anschließend signierten Kopf
als `head.next` neu schreiben/synchronisieren und per lokalem Replace mit Write-through
veröffentlichen. Erst dann gilt der Übergang als abgeschlossen. Restdatei oder nicht passende
Record-/Kopfgeneration blockiert konservativ. Keine automatische Rollback-/Reparaturlogik. Beliebige
Stromausfälle oder Rückspielen eines vollständigen älteren privaten Checkpoints sind damit nicht
attestiert; es gibt keinen externen monotonen Zähler.

## Zustandsübergänge

| Aktion                  | Vorbedingung                                                                 | Dauerhafte Wirkung                                                                       |
| ----------------------- | ---------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------- |
| `initialize`            | neue private Wurzel                                                          | Bindet aktuelle Programmbytes; Kontrollschema 1, keine Updateaktionen                    |
| `migrate {from:1,to:2}` | richtige EXE, aktueller Kopf, kein aktiver Job/pending Update                | Schaltet die bekannte lokale Updatepolitik des Kontrollschemas 2 frei                    |
| `prepare {binary}`      | Schema 2 oder 4 (Paketvertrag), aktueller Kopf, neue noch nie aktivierte SHA | Signiert Pending-Zustand, danach geschützte Paketkopie; bisherige EXE bleibt autorisiert |
| `activate`              | signiertes Pending, unveränderte geschützte Paketkopie, aktueller Kopf       | Autorisiert exakt die neue SHA; ältere EXE wird an der nativen Command-Grenze abgewiesen |
| `abort`                 | signiertes Pending, bisherige EXE und aktueller Kopf                         | Beendet Pending ohne Schlüssel-, Daten- oder Journalreset; Paketdatei bleibt erhalten    |

Unbekannte Schemaübergänge, Schema-Downgrade, erneut aktivierte Programmhashes und alte Köpfe werden
abgewiesen. Ein Abbruch löscht keine Dateien und überschreibt keine vorhandene Paketkopie; eine
erneute Vorbereitung derselben zurückgelassenen Kopie kann deshalb konservativ blockieren.
Authentisierte Migration betrifft hier das Kontrollschema, keine vorhandene Projekt- oder
Auftragsdatenbank. Forschungs-v1/v2/v3-Daten werden nicht automatisch migriert.

Jede Änderung benötigt die exklusive Sperre und einen leeren nativen Job
`Local\KRAQEL.Product.Broker.<Installationskennung>`. Der künftige Produktstarter muss dieselbe
Sperre vor Joberzeugung halten. Der synthetische Startpfad unten verwendet sie bereits; die normale
Produktintegration bleibt offen. Aktive Jobs werden bei einer abgewiesenen Änderung nicht beendet.
Status ist kein Startgrant; Pending ist keine Importfreigabe.

Updates werden lokal ausdrücklich anhand einer erwarteten SHA-256 freigegeben, als reguläre
begrenzte Datei gepinnt, als x64-EXE geprüft und in die private Ablage kopiert. Das authentisiert
die lokale Entscheidung und Dateiintegrität, nicht die Herkunft eines Release-Herausgebenden. Keine
Downloads, Netzendpunkte, Codesignierungszertifikate oder automatische Updateinstallation. Ein
irrtümlich freigegebenes inkompatibles Programm kann die Ablage sperren; keine heimliche Rückkehr
auf eine alte Programmversion. Herausgebervertrauen, Paketkompatibilität und echte Projektmigration
bleiben Teil der weiteren Produktintegration.

## Kontrollgebundener synthetischer Start

`broker-store start-check <Forschungs-App> <erwartete App-SHA> <aktueller Kopfhash>` verlangt den
aktuell autorisierten Broker, Kontrollschema 2, keinen Pending-Zustand und den aktuellen Kopf. Die
Kontrollsperre bleibt bis Jobleerstand gehalten. Der suspendierte Root wird bei Erzeugung dem
gleichen benannten Produktjob zugewiesen, den Updates prüfen. Selbst eine leere Jobkollision wird
abgewiesen. Appquelle und gestagte Kopie werden anhand der erwarteten SHA geprüft und gepinnt.

Nur eine neue getrennte synthetische TEMP-Sitzung bekommt Containerrechte; die Kontrollablage samt
reserviertem `data/` bleibt privat. Jeder Aufruf erteilt einen neuen synthetischen Auftrag, keine
Wiederaufnahme vorhandener Produktdaten. Es gibt dort keinen Forschungs-Brokerkontrollordner, der
eine Wiederaufnahme ohne Produktkontrollsperre erlauben würde. Noch kein dauerhaft authentisiertes
Apppaket-/Sitzungslayout und keine normale Produktintegration.
[Startnachweis und Grenzen](../testing/native-product-starter.md). Der neue Broker benötigt einen
eigenen frischen ETW-Nachweis; vorhandene Messungen werden nicht übertragen.

## Dauerhafte Sitzungsbindung und Wiederanlauf · Kontrollschema 3

Explizites `migrate {from:2,to:3}` erlaubt eine neue synthetische Sitzung je Installation. Es
migriert keine App-/Projektdaten. Envelope-/Recordformat bleibt Version 1; die neuen typisierten
Aktionen werden von älteren Programmen konservativ abgewiesen. Schema 3 erlaubt derzeit keine
Brokerupdates, keine Rückmigration und keinen `start-check`-Aufruf des früheren Schema-2-Pfads.

- `bindSession {binding}` bindet nichtleere Sitzung/Task/erste Startkennung, App-SHA, kanonischen
  UTF-16LE-Sitzungswurzelhash und typisierten synthetischen Haltepunkt.
- `recoverSession {launch}` bindet vor jedem Wiederanlauf eine neue, noch nicht verwendete
  Startkennung. Insgesamt höchstens 16 Starts, keine neue Importautorität.
- `retireSession` sperrt dauerhaft vor Profilentfernung. Keine Neubindung derselben Installation.

Die deterministisch abgeleitete Geschwisterablage `.kraqel-app-preview/<Installation>/` bleibt
privat; nur die konkrete Sitzung/Appkopie und deren Appdaten erhalten Containerrechte. Beim
Wiederanlauf sind signierte Wurzel/Appbytes und vorhandene Journale zwingend; nichts wird aus
Diagnose-JSON übernommen oder neu initialisiert. Die gleiche Kontrollsperre bleibt bis Jobleerstand
gehalten. Nach Controllerabbruch höchstens zehn Sekunden auf natürliches Verschwinden des alten
kill-on-close-Jobs warten; keine Jobübernahme oder fremde Prozesse beenden. Das bestehende Profil
bleibt über Generationen erhalten und wird nur ausdrücklich stillgelegt.

[Native Prüfung mit zwei echten Controllerabbrüchen](../testing/native-product-session-recovery.md).
Unvollständige Records/Köpfe blockieren unverändert. Mehrsitzungsverwaltung, Schema-3-Updates,
allgemeine Bindungs-/Startrecord-Crashpunkte und echte Datenmigration bleiben offen. Echte
Vorschauablage und ihre autorisierte EXE wurden für die Prüfung nicht verändert.

## Authentisierte Profilbereinigung · Kontrollschema 4

Explizites `migrate {from:3,to:4}` erlaubt `prepareSession {binding}` vor jeder neuen
Profilerzeugung. Die private Sitzungswurzel existiert bereits und ist kanonisch gebunden; das Profil
wurde noch nicht erzeugt. `bindSession` muss exakt der Vorbereitung entsprechen. Eine Vorbereitung
allein ist kein Import-/Recoverygrant und kann ausschließlich ausdrücklich stillgelegt werden. Die
bisherigen Sitzungs-/Startgrenzen bleiben bestehen. Brokerupdates sind in Schema 2 sowie mit dem
unten beschriebenen Paketvertrag in Schema 4 erlaubt; Schema 3 bleibt dafür gesperrt. Keine
automatische Übernahme ungebundener Alt-Orphans.

`cleanup-session <aktueller Kopf>` verlangt authentisierte Stilllegung, aktuellen Actor/Kopf und
Jobleerstand. Profilidentität nur aus Kontrollkette, keine caller-seitigen Namen oder Verzeichnisse.
Dieselbe Kontrollsperre bleibt über Windows-Profillöschung und Abschluss gehalten. `retire-session`
delegiert nach der Stilllegung dorthin; bei bereits signierter Stilllegung wird kein erneuter
Retire-Record geschrieben. Erfolg oder präzises Nichtvorhandensein des Profils erlaubt in Schema 4
den einmaligen `profileCleaned`-Record. Wiederholung mit aktuellem Kopf ist idempotent; andere
OS-Fehler bleiben Fehler. Schema-3-Bindungen können ohne Schema-4-Abschlussrecord bereinigt werden.
Kein Löschen von Projekten/Journalen/Kontrollschlüsseln, keine Kopf- oder Recordreparatur.
[Gezielter nativer Nachweis](../testing/native-profile-cleanup.md). Weitere Crashpunkte und ein
frischer ETW-Beleg dieses neuen Brokerhashs bleiben offen.

## Lokale Schema-4-Brokerpaketkompatibilität · v1

Schema 4 erlaubt wie Schema 2 explizite lokale Updatevorbereitung, Aktivierung und Abbruch. Schema 3
bleibt gesperrt. Derselbe private Storelease, aktueller Actor/Kopf und Jobleerstand sind
verpflichtend; Pending blockiert Sitzung/Recovery/Cleanup. Keine Reaktivierung alter Programmhashes,
kein Schlüssel-/Journalreset oder Austausch der bestehenden Appbindung.

Zusätzlich zu vollständigem erwarteten SHA-256 und gepinnter regulärer x64-Datei verlangt Schema 4
genau eine `.kqmeta`-Section in einem AMD64-PE32+-Image. Grundlage für Header/Sectionlayout:
[Microsoft PE Format](https://learn.microsoft.com/en-us/windows/win32/debug/pe-format).
App-spezifische Section: VirtualSize 32, RawSize 32–4096, vollständig innerhalb der höchstens
128-MiB-Datei und hinter der Sectiontabelle. Initialisierte lesbare Daten, weder schreibbar,
ausführbar noch verwerfbar; höchstens 96 Sections. Felder, alle Integer little-endian:

| Offset | Bytes | Wert                                      |
| -----: | ----: | ----------------------------------------- |
|      0 |    16 | ASCII `KRAQEL-PACKAGE` und zwei Nullbytes |
|     16 |     4 | Paketmetadatenversion 1                   |
|     20 |     2 | Kleinste lesbare Kontrollschema-Version 1 |
|     22 |     2 | Größte lesbare Kontrollschema-Version 4   |
|     24 |     2 | Startticket-Version 3                     |
|     26 |     2 | Authentisierter Profil-Cleanup-Vertrag 1  |
|     28 |     4 | Reserviert, null                          |

Andere Werte/fehlende oder doppelte Section blockieren vor Pending. Die geschützte Kopie wird vor
Aktivierung erneut auf Hash und Vertrag geprüft. Keine Ausführung/Probe des Kandidaten vorher. Die
Headerprüfung ist begrenzt, keine vollständige PE- oder Programmvalidierung. Das authentisiert die
lokale Hashfreigabe, nicht die Herkunft oder das tatsächliche Verhalten eines beliebigen Programms.
Kandidaten müssen die unterstützte Deklaration selbst mitbringen; alte Schema-4-EXEs ohne Metadaten
werden konservativ abgewiesen.
[Gezielter nativer Nachweis und Grenzen](../testing/native-schema4-package-update.md).

### Abbruch und Fehlerreconciliation

Ein vollständig authentisiertes Pending bleibt auch ohne vollständige Paketkopie gültig; Aktivierung
benötigt weiterhin die passende geschützte Datei. Der aktuelle Actor kann mit aktuellem Kopf und
leerem Produktjob unter derselben exklusiven Storelease `Abort` anhängen. Actor und
Sitzungs-/App-/Aufgabenbindung bleiben erhalten, Pending wird entfernt. Nach veröffentlichtem
Abort-Kopf ist authentisierter Wiederanlauf möglich; erneuter Abort ohne Pending wird ohne weitere
Mutation abgewiesen. Unvollständige Kontrollketten werden weiterhin nicht repariert.

Abort entfernt selbst keine Paketbytes. Schema 4 rekonstruiert zusätzlich die abgebrochenen Hashs
aus der authentisierten Historie. Erneute explizite Vorbereitung darf genau deren nie aktivierte
Kopie unter derselben Sperre ersetzen: Übergang/Actor/Kopf/Job vor Mutation prüfen, exklusive
reguläre private Einlink-Datei ohne Reparse-/Offline-/Recall-Attribute über geprüften Handle
löschen. Kein Sweep und keine Löschung unbekannter/aktiver/früher aktivierter Pakete. Neue Zieldatei
exklusiv vor Prepare-Record anlegen/ACL prüfen. Kollisions-/Lösch-/Anlagefehler erzeugen kein neues
Pending; I/O-Fehler während der danach signierten Kopierabsicht bleiben ausdrücklich abzubrechen.
Schema 2 erhält nur die frühere Zielanlage, keine automatische Artefaktentfernung. Unbekannte
Artefakte vor erstem signierten Prepare und defekte Ketten bleiben gesperrt. Kein neues Wireformat.
[Implementierung und gezielte Retrynachweise](../testing/native-package-artifact-retry.md).
[Drei gezielte native Fälle und Grenzen](../testing/native-update-abort-reconciliation.md).

## Synthetische Crashprüfung

[Drei gezielte Prepare-Übergänge](../testing/native-prepare-transition-crashes.md):
`artifact-removed` nach abgeschlossenem handlegebundenem Entfernen, `package-created` nach
exklusiver Zielanlage/ACL-Prüfung, `prepare-recorded` nach vollständigem Record-/Kopfabschluss vor
erstem Kopierbyte. Standardmäßig inaktiv, nur expliziter synthetischer Child. Beim authentisierten
Retry erlauben die ersten beiden Zustände erneute Vorbereitung unter unverändertem Kopf ohne
Pending; der dritte verlangt zunächst ausdrücklichen Abort. Keine allgemeine Orphan-Autorität, keine
Kettenreparatur oder neue Importberechtigung.

`product-control-series <frische TEMP-Controllerwurzel> <Forschungs-App>` und der interne
`product-control-child` prüfen dieselben nativen Schreib-/Update-/Migrationsfunktionen mit harten
Prozessabbrüchen. Der Child akzeptiert ausschließlich das begrenzte Controller-/Case-/Store-Layout
unter TEMP; die Fault-Auswahl ist pro Prozess standardmäßig aus. Keine Umgebungsvariable oder
Rendererfunktion aktiviert Faults in normalen `broker-store`-Commands. Bestehende Vorschauablage und
Schlüssel werden weder kopiert noch zur Crash-Injektion übernommen.

[28 native Crashfälle und eigener frischer ETW-Vergleich](../testing/native-product-control-crash-etw.md)
bestätigen elf nutzbare und 17 konservativ gesperrte Zustände. Vor Recordschreiben bleibt der alte
Kopf gültig für eine ausdrücklich neu angestoßene Operation. Festgeschriebene Records verbrauchen
diesen Kopf. Vollständiges Pending kann explizit aktiviert, unvollständiges Paket nicht aktiviert,
aber bei gültigem Kopf explizit abgebrochen werden. Unvollständige Record-/Kopfabschlüsse bleiben
gesperrt; kein automatisches Kopfreparieren, Recordlöschen oder Schlüsselreset. Der Netzbeleg gilt
ausschließlich für seinen gesicherten Prüfbuild, nicht für automatisch aktualisierte Produkte.

## P2a: Schema-5-Records ohne UI-Start

[P2a-Vertrag/Metadaten und Nachweis](../testing/product-start-p2a-records.md) ergänzt ausdrücklich
nur den pfadbegrenzten `product-workspace-child` in synthetischen TEMP-Controllerwurzeln. Private
MAC-/Root-/Actor-/Kopfbindung bleibt unverändert, neue Actions binden Workspace bzw. verbrauchen/
reconcilieren Launch-Generationen. Keine Importtask, Profile oder Prozesse daraus starten. Schema 5
unterstützt hier keine Brokerupdates/Importtestsitzungen; `.kqprod` deklariert Record-ABI 1,
UI-ABI 0. Neues Bindestaging wird nicht adoptiert; unbekannte Reste/defekte Abschlüsse bleiben
gesperrt. Der normale `broker-store` erhält keinen Migrate-v5- oder Produktstartcommand.

## P2b: Synthetischer Workerbootstrap

[Vertrag/Implementierung/Nachweis](../testing/product-start-p2b-bootstrap.md) ergänzt im eigenen
pfadbegrenzten `product-workspace-probe-child` CommitWorkspaceProbe, WorkspaceProbeProfileReady,
FinishWorkspaceProbe und ReconcileWorkspaceProbe. Schema 5/Workspace-Record-ABI 1; `.kqboot`
deklariert getrennt Probe-ABI 1/UI-ABI 0/eigene normale Profilbereinigung ABI 1. Keine Erweiterung
des normalen `broker-store`. Unter derselben Lease werden nur frische separate synthetische
Probeprofile/Root-/Paket-/Policy-/Noncebindungen freigegeben, niemals die private P2a-Fixture.
Permanentkey bleibt ausschließlich im Store; Bootstrap/Receipt erhalten eigene MAC-Domänen und einen
frischen kanalgebundenen Einmalschlüssel. Vor Resume exakter Token/Job/Lease/private Denials; vor
Ressourcenabschluss Jobleerstand am gehaltenen Kernelobjekt und Profilentfernung. Kontrollierte
Ablehnungen verbrauchen ihren Launch; unklare Crash-/Profilerzeugungs-/Cleanupreste bleiben pending,
kein automatischer Probe-Reconcile-CLI. Frisches ETW und persistente echte
Arbeitsablage/GUI-Bootstrap fehlen weiterhin, kein Produktstart daraus ableiten.

## Abgrenzung zum geplanten Produktstart

P1 war zunächst ein reines Testmodell; P2a nutzt denselben Kern jetzt nach nativen Kontrollprüfungen
im Forschungschild. Gültiges JSON bleibt unvertrauenswürdig. Ein separater synthetischer
P2b-Probeworker ist implementiert, UI-Start/Produktbootstrap, vollständige
Produktpaket-/Datenfreigabe und Bestandsübernahme sind weiterhin nicht implementiert.
[Produktstart-/Bestandsdatenvertrag v1](product-start-integration-v1-draft.md) trennt diese geplante
Integration von Importaufgaben. Kein Wiederverwenden von Importticket v3 als Produktstart, keine
automatische Übernahme der Vorschauablage und keine Migration durch
`bind-session`/`recover-session`. Die folgenden normalen Vorschau-Commands unterstützen keinen
Schema-5-/Produktstart.

## Native Vorschau-Commands

Zusätzliche synthetische Prüfung:
[20 Sitzungsübergangs-Crashfälle und frisches ETW](../testing/native-session-transition-crash-etw.md).
Bindung, Recovery-Startrecord und Stilllegung werden an Record-/Kopfpunkten sowie vor GUI und nach
Profilentfernung hart abgebrochen. Neun unvollständige Abschlüsse bleiben gesperrt; elf
authentisierbare Zustände sind kein allgemeiner Startgrant. Vier explizite Recoveries bestehen;
ungebundene, journalfreie und stillgelegte Zustände bleiben ohne Importautorität. Testprofil-Cleanup
gilt ausschließlich für frische eigene Fixtures und repariert keine Kontrollkette. Authentisierte
Produkt-Cleanup-Reconciliation bleibt offen. Der eigene ETW-Beleg gilt nur für seinen gesicherten
Brokerhash, nicht automatisch für Folgeänderungen.

```text
kraqel-isolation-probe.exe broker-store init
… broker-store status
… broker-store migrate-v2 <erwarteter Kopfhash>
… broker-store prepare <lokale EXE> <erwartete SHA-256> <erwarteter Kopfhash>
… broker-store activate <erwarteter Kopfhash>
… broker-store abort <erwarteter Kopfhash>
… broker-store check-container <gesicherte Forschungs-App>
… broker-store start-check <gesicherte Forschungs-App> <erwartete App-SHA> <erwarteter Kopfhash>
… broker-store migrate-v3 <erwarteter Kopfhash>
… broker-store bind-session <Forschungs-App> <erwartete App-SHA> <Haltepunkt> <erwarteter Kopfhash>
… broker-store recover-session <erwarteter Kopfhash>
… broker-store retire-session <erwarteter Kopfhash>
… broker-store migrate-v4 <erwarteter Kopfhash>
… broker-store cleanup-session <erwarteter Kopfhash>
```

Nach Aktivierung weitere Commands über die verwaltete neue EXE unter `bin/<SHA>.exe` ausführen. Die
Prüfung `check-container` erzeugt ausschließlich eine frische synthetische TEMP-GUI, prüft sechs
tatsächliche Dateiverweigerungen unter deren suspendiertem eingeschränktem Token, führt den
bestehenden synthetischen Import aus und entfernt das Testprofil. Ihr lokaler Diagnosepfad ist nur
native Testausgabe und keine veröffentlichte Projektdokumentation. Keine Commands sind mit dem
normalen Produkt-Renderer verdrahtet. Jeder geänderte Programmstand benötigt einen eigenen
ETW-Nachweis.
