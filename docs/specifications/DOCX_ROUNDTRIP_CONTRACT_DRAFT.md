# KRAQEL-DOCX-Roundtrip v1: Vertragsentwurf

Status: Vorschlag zur fachlichen Freigabe, 14. November 2026; KQ-057 / Issue #111. Noch keine
produktive Implementierung; die Repräsentation ist inzwischen mit ADR 0026 angenommen. Der
bestehende Fremdimport nach ADR 0024 bleibt davon getrennt. Dieser Entwurf konkretisiert die offene
Entscheidung aus Pflichtenheft Revision 1, Abschnitt 11, und einen Teil von F08/M3. Er schließt
deren gesamte Abnahme nicht ab.

Der anschließend beauftragte Strukturprototyp KQ-058 bestätigt das verschachtelte Modell für den
untersuchten Word-Build. LibreOffice entfernt äußere Marker und Inventar. Ergebnisse und echte
Office-Fixtures stehen in [ADR 0025](../architecture/decisions/0025-docx-structure-prototype.md).
Der anschließende beauftragte Folgeversuch KQ-059 erhält mit flachen Absatzmarkern und Inventar in
Dokumenteigenschaften die Semantik auch in LibreOffice. Diese Repräsentation ist mit
[ADR 0026](../architecture/decisions/0026-flat-docx-markers.md) als Grundlage angenommen.
Produktversion, vollständiger Paketvertrag und native Implementierung stehen weiterhin aus.

## Ziel und Benutzerablauf

Ein eigener Export „DOCX zum Wiederimport“ erzeugt ein Word-lesbares Manuskript mit versionierter
KRAQEL-Struktur. Auf der Startseite importiert „KRAQEL-Manuskript aus DOCX“ dieses Manuskript als
neues Projekt. Vor der Dateiauswahl wird erklärt, welche Inhalte dazugehören und welche fehlen.
Vorhandene Projekte werden weder aktualisiert noch überschrieben. Ein neutraler Abbruch schreibt
nichts. Der gewöhnliche DOCX-Export und der DOCX-Import als einzelne Szene bleiben eigene Aktionen.

Die erste Garantie lautet: Ein unveränderter Roundtrip-Export ergibt beim Wiederimport dieselbe
kanonische Manuskriptsemantik. Neue Projekt-, Kapitel- und Szenen-UUIDs sind beabsichtigt. Es ist
keine Wiederherstellung von Story-Bibel, Plotboard, Timelines, Medien, Profilen, Prompts oder
Versionsgeschichte. Für vollständige Projektkopien bleibt das Snapshot-v3-Archiv zuständig.

## Erhaltene Semantik

| Bestandteil              | Garantie                                                                                |
| ------------------------ | --------------------------------------------------------------------------------------- |
| Projekt, Kapitel, Szenen | Titel, Reihenfolge und Zuordnung; auch leere Kapitel und Szenen                         |
| Absätze                  | Unicode-Text, Leerzeichen, leere Absätze und harte Zeilenumbrüche                       |
| Überschriften            | Alle sechs Ebenen innerhalb einer Szene, getrennt von Kapitel-/Szenentiteln             |
| Textmarkierungen         | Fett, kursiv und deren Kombination                                                      |
| Listen                   | Geordnet/ungeordnet, Reihenfolge, Elemente, mehrere Absätze je Element, Verschachtelung |
| Zitate                   | Blockgrenzen und enthaltene Absatz-/Listen-/Zitatstruktur                               |

Die Liste umfasst das vorhandene `Manuscript`-/`Block`-/`Inline`-Modell in
`crates/export/src/lib.rs`. Vor der Umsetzung muss dessen Validierung auch nicht unterstützte
Attribute erkennen: etwa einen abweichenden Startwert einer nummerierten Liste darf der Austausch
nicht still auf 1 zurücksetzen. v1 unterstützt nur Listenstart 1; andere Startwerte werden beim
Export abgelehnt.

Kanonischer Vergleich: gleiche Baumstruktur, Titel, Textzeichen, Umbrüche, Blockarten,
Überschriftenebenen und effektive Fett-/Kursivmarkierungen. Benachbarte Textläufe mit gleichen
Markierungen werden zusammengefasst; leere Textläufe werden entfernt. Leere Absätze bleiben
erhalten. Keine Unicode-Normalisierung, kein Trimmen, keine Zusammenfassung von Leerzeichen.
Editor-JSON-Bytegleichheit, OOXML-Bytegleichheit, ZIP-Reihenfolge, Layout, Seitenzahlen,
Schriftarten und ursprüngliche UUIDs sind keine Garantien.

## Vorgeschlagene Repräsentation

Der neue Writer muss die bisher verlustbehaftete DOCX-Abbildung ersetzen, ausschließlich für den
neuen Austauschmodus. Kapitel-/Szenentitel sind eigene Rollen; Szenenüberschriften werden nicht mehr
um zwei Ebenen verschoben. Listen verwenden echte OOXML-Nummerierungsdefinitionen; Zitate erhalten
eine eigene Absatzdarstellung. Sichtbare Präfixe allein bestimmen keine Struktur.

Nach ADR 0026 werden flache, absatzweite OOXML-Content-Controls (`w:sdt`) eingesetzt. Ein Marker
kennzeichnet jeweils Titel, Absatz oder Überschrift mit Ebene und bindet sein Inventar über einen
Digest. Das versionierte Inventar in benutzerdefinierten Dokumenteigenschaften beschreibt Kapitel,
Szenen, Listen, Elemente und Zitate sowie Eltern und Reihenfolge. Es enthält keine Manuskriptkopie.
Kennungen sind austauschlokal, keine Pfade oder Datenbank-UUIDs. Experimentelle Grammatik und
Office-Fixtures sind in ADR 0026 verlinkt; die produktive Version wird vor Rust separat fixiert.

Sichtbarer Text in `word/document.xml` bleibt die einzige Textquelle. Verstecktes JSON darf keine
Office-Textänderungen überschreiben. Doppelte Kennungen, falsche Zuordnung, widersprüchliche Ebenen
oder Nummerierung und unmarkierter Inhalt führen zur Ablehnung. Fehlende oder fremde Versionen
werden nicht anhand sichtbarer Titel erraten.

KQ-059 weist mehrabsätzige Listenelemente, leere Kapitel/Szenen und verschachtelte Zitate für Word
16.0.20527.20112 und LibreOffice 26.8.3.1 nach. Der native Reader/Writer muss diese Semantik
unabhängig reproduzieren und alle Paket-/Inhaltsgrenzen durchsetzen. Die Python-Prüfwerkzeuge sind
kein Produktparser.

## Nach Bearbeitung in Word oder LibreOffice

Die erste Stufe garantiert Textänderungen innerhalb bestehender markierter Absätze und Titel,
einschließlich Fett/Kursiv und harter Umbrüche, sofern alle Strukturmarker und unterstützten
Konstrukte gültig erhalten bleiben. Tatsächliches Speichern und Textänderungen sind mit den in ADR
0026 genannten Builds geprüft. Daraus folgt keine pauschale Zusage für andere Office-Versionen oder
beliebige Strukturänderungen.

Neue/gelöschte/verschobene Kapitel, Szenen, Listen oder Absätze außerhalb des markierten Modells
sind zunächst nicht zugesagt. Einfügen von Tabellen, Medien, Links, Feldern, Fußnoten, Kommentaren,
Änderungsverfolgung oder unbekannter inhaltstragender Formatierung führt zur Ablehnung. Entfernte
Marker werden nicht heuristisch ergänzt. Layout-only-Änderungen dürfen nur ignoriert werden, wenn
der endgültige Paketvertrag sie ausdrücklich als semantisch inert erlaubt.

Ein entfernter vollständiger Container kann ohne unabhängiges Inventar nicht als versehentliche
Löschung erkannt werden. Deshalb prüfen die Blattmarker zusätzlich ein begrenztes Inventar der
erwarteten Kennungen und Rollen in Dokumenteigenschaften, ohne Textkopien. Der Import verlangt
dieselbe Menge und Zuordnung; Textänderungen bleiben erlaubt, Strukturänderungen werden abgelehnt.
Das Inventar ist ebenfalls untrusted: Es ist ein Konsistenznachweis, keine Signatur oder
Authentizitätsgarantie.

Bei Ablehnung bleibt die Quelldatei erhalten. Eine verständliche Meldung nennt die fehlende
Strukturerhaltung und verweist bei Bedarf auf den getrennten, begrenzten Szenenimport. Es gibt
keinen automatischen verlustbehafteten Fallback.

## Native Grenze und Veröffentlichung

Dialog, Pfad, Archivbytes und XML-Verarbeitung bleiben in Rust. IPC liefert nur bestehende
Projektzusammenfassungen bzw. Exportmetadaten, keine absoluten Pfade oder Dokumentbytes. Keine
Office-Automation, Makroausführung, Netzwerkauflösung oder Extraktion von Archivpfaden.

Als Ausgangsgrenzen gelten ADR 0024: 8 MiB Archiv, 32 MiB entpackt insgesamt, 8 MiB je Teil, 128
Einträge, 100:1 je Eintrag, XML-Tiefe 64, insgesamt 100.000 Elemente und 2 MiB Text. Inventar und
Marker zählen vollständig gegen diese Grenzen. Verschachtelte Manuskriptcontainer verbrauchen
XML-Tiefe; der Export muss dieselben Importgrenzen vor Veröffentlichung prüfen und übermäßige
Struktur ausdrücklich ablehnen. Eine Grenzerhöhung braucht eine eigene Entscheidung.

Vollständige Paket-, Struktur- und Modellvalidierung erfolgt vor Veröffentlichung eines Projekts.
SQLite, Index und Baum werden in einem privaten Staging-Projekt atomar aufgebaut; erst danach wird
die neue Projektkopie sichtbar. Fehler entfernen ausschließlich dieses Staging-Projekt. Abbruch,
Übergröße oder ungültiger Inhalt hinterlassen kein leeres Projekt. Exportdateien werden erst nach
erfolgreichem Rendern und Selbstimport atomar ersetzt.

## Abnahme und Arbeitspakete

1. Repräsentationsprototyp und ADR: tatsächliche Word-/LibreOffice-Speichervorgänge, gültige
   Controls, Inventar, Paket-Allowlist und eindeutige Tag-Grammatik nachweisen. Keine nur
   synthetisch erzeugte Fixture als Office-Nachweis ausgeben.
2. Gemeinsames kanonisches Manuskriptmodell und strikte Attributeprüfung bereitstellen; bestehende
   Exporte und Fremdimport regressionsfrei erhalten.
3. Writer und Parser mit strukturellem Vergleich testen: alle Ebenen 1–6, gleiche/leer bleibende
   Titel, leere Kapitel/Szenen/Absätze, Unicode, Leerzeichen, kombinierte Markierungen, harte
   Umbrüche, verschachtelte Listen und Zitate sowie Listen mit mehreren Absätzen.
4. Negativfälle: alte gewöhnliche DOCX-Exporte, fehlende/doppelte/umgehängte Marker, beschädigtes
   Inventar, fremde Versionen, widersprüchliche Nummerierung, unbekannte Inhalte und sämtliche
   Archiv-/XML-Grenzen. Kein Projekt nach Fehler oder Abbruch.
5. Additive Commands und UI: bewusste Wahl des Austauschmodus, nachvollziehbare Scope-Erklärung,
   Browser-E2E und echter Windows-Dateidialog; persistente neue Projektkopie nach Neustart.
6. `pnpm check`, Windows-/Linux-CI sowie Office-Rendering und semantische Rückimport-Nachweise.

Die Repräsentation ist mit ADR 0026 festgelegt. Produktiver Paketvertrag, native Implementierung und
App-Abnahme stehen weiterhin aus. Allgemeine Strukturänderungen in Word, Medien und vollständige
Projektwiederherstellung würden einen weiteren Scope benötigen.
