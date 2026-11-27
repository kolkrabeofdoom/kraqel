# KQ-069: Kontrollgebundener isolierter Startpfad

Stand: 17. November 2026. Windows-Vorschau mit ausschließlich synthetischen TEMP-Daten; keine
normale Produktintegration und keine Produktfreigabe. Striktes Datenschutzziel unverändert.
[Bereinigter Nachweis](native-product-starter.json),
[Kontrollvertrag](../data-format/native-product-control-v1.md).

Ausgangspunkt `506e69b1cf0586158fd51a936bbefc1593e64535`, dessen Windows-/Linux-CI und Dependency
Review vollständig grün sind. Neuer gesicherter Helfer unter
`%TEMP%/kraqel-product-starter-build-20261117-01/`:

- Broker-SHA-256: `53b4fe70848e16fc49e10e03cd4c1abed5c941ae0ac57f52fa78bfc07e9e2754`.
- Forschungs-App unverändert: `6e56a51ed2e913a878a7935e039bd69628a49be25f0fbab7d02fce3a72fe8c70`.

## Verhalten und Grenze

`broker-store start-check <Forschungs-App> <erwartete App-SHA> <aktueller Kopfhash>` verlangt die
authentisierte Kontrollablage, den aktuell autorisierten Broker, Kontrollschema 2, keinen
ausstehenden Programmwechsel und den exakten aktuellen Kopf. Aktive oder auch nur kollidierende
leere benannte Jobs sperren den Start. Kein unisolierter Fallback und keine automatische Migration.
Die bisherige echte Vorschauablage und ihre autorisierte EXE wurden nicht verändert oder verwendet.
Der neue Prüfbuild ist dort nicht automatisch autorisiert.

Die private Kontrollablage bleibt über ihren `Store` und dessen exklusive native Sperre für die
gesamte GUI-/Joblaufzeit geöffnet. Der suspendierte Root wird bereits bei Erzeugung dem benannten
`Local\KRAQEL.Product.Broker.<Installation>`-Job zugewiesen; das ist derselbe Name, den Updates
prüfen. Vor Resume: exakte AppContainer-Identität, null Capabilities, tatsächliche Jobzugehörigkeit,
genau ein aktiver suspendierter Root, tatsächliche Sharing-Verweigerung an der Kontrollsperre sowie
sechs echte private Dateiverweigerungen mit dem eingeschränkten Token. Die Sperre wird nicht in den
Container vererbt. Der Job bleibt kill-on-close; erst nach Jobleerstand endet der Aufruf.

Die App wird anhand einer ausdrücklich erwarteten SHA-256 als begrenzte reguläre lokale x64-EXE
gepinnt. Quelle, Vorfahren und gestagte Kopie bleiben gegen Schreiben/Ersetzen gepinnt; die Kopie
wird vor dem Start erneut gehasht. Hashfreigabe ist keine Herausgeberauthentisierung.

**Datenrollen:** Kontrollwurzel, Kontrollschlüssel, Records, verwaltete Brokerpakete und das
reservierte private `data/` erhalten keine Containerrechte. Nur eine neue separate synthetische
TEMP-Sitzung enthält ausführbare Appkopie und beschreibbare Appdaten. Der Container kann seine
SQLite-Daten dort schreiben; die private Kontrollablage bleibt unzugänglich. In dieser Sitzung gibt
es keinen alten Broker-`control/`-Ordner und damit keinen dadurch eröffneten Recovery-Pfad, der die
Produktkontrollsperre umgehen könnte.

Jeder Aufruf startet einen ausdrücklich neuen synthetischen Auftrag. Der zweite native Start öffnet
dieselbe Kontrollablage erneut, verwendet aber eine neue Sitzung/Identität und neue Appdaten. Das
ist **kein** Wiederanlauf derselben Produktdaten, keine Journalmigration und kein allgemeiner
Projektstarter. Dauerhaft gebundene Apppakete, Sitzungs-/Auftragszuordnung, authentisierter
Produkt-Recoverypfad und echtes persistentes Appdatenlayout bleiben offen. Bestehende Sitzungen
werden nicht aus Diagnose-JSON übernommen; Schlüssel und Journale werden nicht ersetzt.

## Prüfung

`product-start-series <frische TEMP-Controllerwurzel> <Forschungs-App> <App-SHA>` legt
ausschließlich eine neue private synthetische Kontrollablage an. Fünf native negative Guards:
Bootstrap, fremder Brokerhash, veralteter Kopf, falscher Apphash, signiertes Pending. Das Pending
ist ein synthetischer Kontrollrecord ohne Paket und wird explizit abgebrochen. Anschließend zwei
isolierte GUI-Starts mit Neueröffnung der Kontrollablage dazwischen. Beide synthetischen Importe
enthalten zwei Dokumente; genau zwei unterschiedliche Aufträge und zwei legitime neue Projektkopien.

Ein unabhängiger Nur-Lese-Audit bestätigt alle vier Kontrollrecords, HMACs, kanonische
UTF-16LE-Wurzelbindung, Installation/Vorgängerkette und Abschlusskopf. Kopf und Schlüssel bleiben
über beide Starts unverändert; der private Datenordner bleibt leer. Für beide GUIs: native
Startbelege, Job-/Sperrprüfung vor Resume, null Capabilities, zwölf tatsächliche
Kontrolldateiverweigerungen insgesamt, kontrolliertes Ende 77, Jobleerstand, entfernte Profile,
Archivhash/-länge, zwei Dokumente, exakter Unicode-Inhalt sowie SQLite-Integrität/Fremdschlüssel.

Drei neue aktive Helfertests prüfen Zustands-/Hashabweisungen, leere Jobkollision und fehlende
Sperre/Rootzuweisung sowie eine konkurrierende echte Controllerprozess-Öffnung. 43 aktive
Helfer-/Transporttests insgesamt; zwei begrenzte Child-Testdefinitionen werden durch aktive
Elterntests aufgerufen, die zwei interaktiven Shell-Einzeltests bleiben separat ausgenommen.
Helfer-Clippy mit Warnungen als Fehler, rustfmt und `pnpm check` bestanden. Kein vollständiger
Tauri-Neubuild; keine neue Abhängigkeit, keine Änderung an Firewall/Netzfähigkeiten oder OS-Policy.

Der geänderte gemeinsame Generationspfad besteht zusätzlich die vollständige bisherige
Kontroll-Crash-Serie: erneut 28 Abbrüche, 17 gesperrte/elf nutzbare Zustände, acht explizite
Weiterführungen und 162 Child-Controller mit 76 erfolgreichen, 58 abgewiesenen und 28 harten Enden.
Der separate unabhängige MAC-/Kopf-/Paket-/Datenaudit besteht ebenfalls; die zwei zusätzlichen
Container-GUIs bestätigen zwölf weitere Dateiverweigerungen. Alle vier GUI-Testprofile dieser beiden
Serien entfernt, Prozesse beendet. Synthetische Appdaten und Kontroll-Fixtures bleiben privat zur
Nachprüfung erhalten. Keine Schlüssel oder Rohdiagnosen veröffentlicht. Deren Beleg und der neue
Prüfstand bleiben getrennt vom früheren ETW-Nachweis.

**Kein frischer ETW-Netznachweis dieses neuen Brokerhashs.** Der
[gestrige Netzbeleg](native-product-control-crash-etw.md) gilt ausschließlich für seine Hashes und
Lebenszeiten. Keine Übertragung auf diesen Startpfad. Keine Inhalts-/Uploadbilanz oder vollständige
externe COM-/Shell-/DNS-/Brokerattestierung. Produkt-Q-01/Q-02 weiter nicht bestanden; #134 offen.
Nächster Scope: authentisierte dauerhafte Sitzungs-/Apppaketbindung und Wiederanlauf unter derselben
Kontrollsperre, danach frischer ETW-Vergleich des gesicherten Paars. Keine automatische Reparatur
gesperrter Kontrollabschlüsse oder Übernahme bestehender Projekte.
