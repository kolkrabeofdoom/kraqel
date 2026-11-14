# KRAQEL-DOCX-Roundtrip v1: Vertragsentwurf

Status: Vorschlag zur fachlichen Freigabe, 14. November 2026; KQ-057 / Issue #111. Noch keine
Implementierung oder angenommene Architekturentscheidung. Der bestehende Fremdimport nach ADR 0024
bleibt davon getrennt. Dieser Entwurf konkretisiert die offene Entscheidung aus Pflichtenheft
Revision 1, Abschnitt 11, und einen Teil von F08/M3. Er schließt deren gesamte Abnahme nicht ab.

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

Vorgeschlagen werden verschachtelte OOXML-Content-Controls (`w:sdt`) im Hauptdokument, mit streng
validierten `w:tag`-Werten aus einem versionierten Namensraum `kraqel:manuscript:1`. Der
Wurzelmarker kennzeichnet Format und Version. Weitere Marker kennzeichnen Titel, Kapitel, Szene,
Absatz, Überschrift mit Ebene, Liste mit Art, Listenelement und Zitat. Austauschlokale Kennungen
sind eindeutig und keine Dateipfade oder Datenbank-UUIDs. Die konkrete Tag-Grammatik, erlaubten
OOXML-Konstrukte und Paketbestandteile werden im angenommenen ADR samt Fixtures festgeschrieben.

Sichtbarer Text in `word/document.xml` ist die einzige Quelle für importierten Manuskripttext. Kein
verstecktes JSON-Manuskript darf Word-Änderungen überschreiben. Marker liefern ausschließlich Rollen
und Grenzen. Doppelte Kennungen, falsche Verschachtelung, widersprüchliche Ebenen oder
Nummerierungsangaben und Inhalt außerhalb erlaubter Container führen zur Ablehnung. Fremde oder
fehlende Formatversionen werden nicht anhand sichtbarer Überschriften erraten.

Der erste Prototyp muss nachweisen, dass die gewählten Controls auch mehrabsätzige Listenelemente,
leere Szenen und verschachtelte Zitate in Word und LibreOffice darstellen können. Erst danach wird
die Repräsentation als ADR angenommen. Falls das nicht gelingt, wird der Vertrag vor der
Implementierung angepasst; keine stille Verringerung der zugesagten Semantik.

## Nach Bearbeitung in Word oder LibreOffice

Die erste Stufe garantiert Textänderungen innerhalb bestehender markierter Absätze und Titel,
einschließlich Fett/Kursiv und harter Umbrüche, sofern alle Strukturmarker und unterstützten
Konstrukte gültig erhalten bleiben. Ein bloßes erneutes Speichern muss mit realen Office-Fixtures
geprüft werden. Bis dahin gibt es keine pauschale Kompatibilitätszusage für Office-Versionen.

Neue/gelöschte/verschobene Kapitel, Szenen, Listen oder Absätze außerhalb des markierten Modells
sind zunächst nicht zugesagt. Einfügen von Tabellen, Medien, Links, Feldern, Fußnoten, Kommentaren,
Änderungsverfolgung oder unbekannter inhaltstragender Formatierung führt zur Ablehnung. Entfernte
Marker werden nicht heuristisch ergänzt. Layout-only-Änderungen dürfen nur ignoriert werden, wenn
der endgültige Paketvertrag sie ausdrücklich als semantisch inert erlaubt.

Ein entfernter vollständiger Container kann ohne unabhängiges Inventar nicht als versehentliche
Löschung erkannt werden. Deshalb enthält der Wurzelmarker zusätzlich ein begrenztes Inventar der
erwarteten Strukturkennungen und Rollen, ohne Textkopien. Der Import verlangt dieselbe Menge und
Zuordnung; Textänderungen bleiben erlaubt, Strukturänderungen werden abgelehnt. Das Inventar ist
ebenfalls untrusted: Es ist ein Konsistenznachweis, keine Signatur oder Authentizitätsgarantie.

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

Zur Freigabe steht dieser enge Vertrag mit Textbearbeitung innerhalb erhaltener Struktur. Allgemeine
Strukturänderungen in Word, Medien und vollständige Projektwiederherstellung würden einen weiteren
Scope benötigen. Bis zur Freigabe bleiben diese Punkte Planung.
