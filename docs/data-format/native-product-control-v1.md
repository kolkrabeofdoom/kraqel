# Native Broker-Kontrollablage und lokale Updates · v1

Stand: 16. November 2026. Implementierte Windows-Vorschau im nativen Forschungsbroker, noch kein
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

| Aktion                  | Vorbedingung                                                           | Dauerhafte Wirkung                                                                       |
| ----------------------- | ---------------------------------------------------------------------- | ---------------------------------------------------------------------------------------- |
| `initialize`            | neue private Wurzel                                                    | Bindet aktuelle Programmbytes; Kontrollschema 1, keine Updateaktionen                    |
| `migrate {from:1,to:2}` | richtige EXE, aktueller Kopf, kein aktiver Job/pending Update          | Schaltet die bekannte lokale Updatepolitik des Kontrollschemas 2 frei                    |
| `prepare {binary}`      | Schema 2, aktueller Kopf, neue noch nie aktivierte SHA                 | Signiert Pending-Zustand, danach geschützte Paketkopie; bisherige EXE bleibt autorisiert |
| `activate`              | signiertes Pending, unveränderte geschützte Paketkopie, aktueller Kopf | Autorisiert exakt die neue SHA; ältere EXE wird an der nativen Command-Grenze abgewiesen |
| `abort`                 | signiertes Pending, bisherige EXE und aktueller Kopf                   | Beendet Pending ohne Schlüssel-, Daten- oder Journalreset; Paketdatei bleibt erhalten    |

Unbekannte Schemaübergänge, Schema-Downgrade, erneut aktivierte Programmhashes und alte Köpfe werden
abgewiesen. Ein Abbruch löscht keine Dateien und überschreibt keine vorhandene Paketkopie; eine
erneute Vorbereitung derselben zurückgelassenen Kopie kann deshalb konservativ blockieren.
Authentisierte Migration betrifft hier das Kontrollschema, keine vorhandene Projekt- oder
Auftragsdatenbank. Forschungs-v1/v2/v3-Daten werden nicht automatisch migriert.

Jede Änderung benötigt die exklusive Sperre und einen leeren nativen Job
`Local\KRAQEL.Product.Broker.<Installationskennung>`. Der künftige Produktstarter muss dieselbe
Sperre vor Joberzeugung halten; diese Integration ist noch offen. Aktive Jobs werden bei einer
abgewiesenen Änderung nicht beendet. Status ist kein Startgrant; Pending ist keine Importfreigabe.

Updates werden lokal ausdrücklich anhand einer erwarteten SHA-256 freigegeben, als reguläre
begrenzte Datei gepinnt, als x64-EXE geprüft und in die private Ablage kopiert. Das authentisiert
die lokale Entscheidung und Dateiintegrität, nicht die Herkunft eines Release-Herausgebenden. Keine
Downloads, Netzendpunkte, Codesignierungszertifikate oder automatische Updateinstallation. Ein
irrtümlich freigegebenes inkompatibles Programm kann die Ablage sperren; keine heimliche Rückkehr
auf eine alte Programmversion. Herausgebervertrauen, Paketkompatibilität und echte Projektmigration
bleiben Teil der weiteren Produktintegration.

## Native Vorschau-Commands

```text
kraqel-isolation-probe.exe broker-store init
… broker-store status
… broker-store migrate-v2 <erwarteter Kopfhash>
… broker-store prepare <lokale EXE> <erwartete SHA-256> <erwarteter Kopfhash>
… broker-store activate <erwarteter Kopfhash>
… broker-store abort <erwarteter Kopfhash>
… broker-store check-container <gesicherte Forschungs-App>
```

Nach Aktivierung weitere Commands über die verwaltete neue EXE unter `bin/<SHA>.exe` ausführen. Die
Prüfung `check-container` erzeugt ausschließlich eine frische synthetische TEMP-GUI, prüft sechs
tatsächliche Dateiverweigerungen unter deren suspendiertem eingeschränktem Token, führt den
bestehenden synthetischen Import aus und entfernt das Testprofil. Ihr lokaler Diagnosepfad ist nur
native Testausgabe und keine veröffentlichte Projektdokumentation. Keine Commands sind mit dem
normalen Produkt-Renderer verdrahtet. Der neue Programmstand benötigt einen eigenen ETW-Nachweis.
