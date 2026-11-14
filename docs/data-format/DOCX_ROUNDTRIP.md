# DOCX Manuskriptaustausch v1

KQ-060 / Issue #117 implementiert den nativen Rust-Kern für das begrenzte Produktionsprofil v1. Die
Repräsentation folgt ADR 0026; ADR 0027 legt Modell, Validierung und Versionsgrenze fest. Ein
App-Dialog, Tauri-Commands und der Import als atomare neue Projektkopie fehlen weiterhin.
Pflichtenheft Revision 1, F08/M3 und die Entscheidung in Abschnitt 11 sind damit teilweise
umgesetzt. Medien, Story-Bibel, Beziehungen und vollständige Projektwiederherstellung bleiben
Snapshot v3.

## Fachliche Semantik

Das kanonische Modell enthält Projekttitel, geordnete Kapitel und Szenen, Absätze, Überschriften
1–6, geordnete und ungeordnete Listen, Listenelemente mit mehreren Absätzen und verschachtelte
Zitate. Titel sind reine Zeichenketten; zusätzliche Titelmarkierungen werden abgelehnt. Doppelte und
leere Titel, leere Kapitel/Szenen und leere Absätze bleiben unterscheidbar.

Textläufe erhalten Unicode, Leerzeichen, Fett/Kursiv und harte Umbrüche. Benachbarte Textläufe mit
gleichen Markierungen werden zusammengeführt, leere Textläufe entfernt. Leere Absätze bleiben.
Listen beginnen bei 1; jedes Element beginnt mit einem Absatz. Leere Listen/Zitate und unbekannte
Editorattribute oder Markierungen werden abgelehnt. Kapitel dürfen keinen nichtleeren eigenen
Editorinhalt haben, der beim Export verschwinden würde. Schrift, Farbe, Einzüge, Seitenlayout und
Office-Metadaten gehören nicht zur erhaltenen Manuskriptsemantik.

Der Text wird ausschließlich aus dem sichtbaren Dokument gelesen. Das Inventar enthält keine
Textkopie, Datenbank-UUID, Pfade oder versteckten Ersatz für verlorene Inhalte. Freies Einfügen,
Löschen oder Verschieben von Absätzen/Containern in Office aktualisiert das Inventar nicht und wird
bei widersprüchlichen Markern abgelehnt. Die geprüfte Bearbeitung ist Text innerhalb bestehender
Absatzmarker, keine beliebige Strukturänderung.

## Marker und Inventar

Jeder Titel und Absatz besitzt ein absatzweites Inline-Content-Control innerhalb genau eines `w:p`.
Der Reader normalisiert auch ein äquivalentes einzelnes Block-Control mit genau einem Absatz. Andere
Controls, verschachtelte Controls und unmarkierter Inhalt sind unzulässig.

Die Markergrammatik lautet `kqm1:<role>:<id>:<digest128>`. `digest128` sind die ersten 32 kleinen
Hexzeichen des SHA-256 über das vollständige kanonische Inventar. Lokale Kennungen haben die Form
`n` und 4–8 Dezimalziffern. Der Writer vergibt aufsteigend `n0001` usw. Zulässige Rollen sind
`manuscript`, `project-title`, `chapter`, `chapter-title`, `scene`, `scene-title`, `paragraph`,
`heading1` bis `heading6`, `bullet-list`, `ordered-list`, `item` und `quote`.

Benutzerdefinierte Dokumenteigenschaften in `docProps/custom.xml` enthalten:

- `KRAQEL.Manuscript.v1`: JSON-Kopf mit exakt `version`, `chunks`, `sha256`.
- `KRAQEL.Manuscript.v1.0001` usw.: vollständige, lückenlose Folge von ASCII-Stücken mit höchstens
  200 Zeichen; höchstens 2048 Stücke.

Alle Werte sind `vt:lpwstr`; Property-Kennungen sind eindeutig und mindestens 2. Das
zusammengesetzte Inventar ist kompaktes ASCII-JSON mit der Feldreihenfolge `version`, `nodes`. Jeder
Knoten hat exakt `id`, `role`, `parent`, `position` in dieser Reihenfolge. Version ist jeweils die
Zahl 1; Positionen sind kanonische nichtnegative Dezimalzeichenketten. Eltern stehen vor Kindern.
Genau eine Wurzel, zulässige Elternrollen, obligatorische Titel, vollständige Geschwisterpositionen
und kanonische Tiefensuche-Reihenfolge werden unabhängig geprüft. Die tatsächlichen Blattmarker
müssen vollständig und in derselben Reihenfolge vorliegen. Zusätzliche Felder, doppelte JSON-Felder,
Kennungen oder Eigenschaftsnamen sowie falsche Digests führen zum Fehler.

SHA-256 bindet Inventar und Marker gegen versehentlichen Verlust oder Austausch. Es ist keine
Signatur: Wer Inventar und alle Marker gemeinsam verändert, kann einen anderen gültigen Baum
erzeugen. Deshalb bleibt auch ein korrekt gebundenes Inventar unvertrauenswürdige Eingabe.
Experimentelle Profile `KRAQEL.Roundtrip.Prototype2` / `kqf2` werden nicht als v1 akzeptiert.

## Paket und Inhaltsprüfung

ZIP-Pfade, doppelte Namen, lokale/zentral gespeicherte Header, Content Types, Beziehungen und
XML-Namensräume werden geprüft. Erforderlich sind Dokument, Styles, Nummerierung, Custom Properties
und deren interne Beziehungen. Optional sind die bekannten Office-Parts für Core-/App-Metadaten,
Settings, WebSettings, FontTable, Theme, StylesWithEffects und eine Dokumentvorschau. Nur die im
Rust-Modul `roundtrip/package.rs` explizit aufgeführten Namen, MIME-Typen und Beziehungsziele sind
erlaubt. Externe Beziehungen, verwaiste Parts, Makros, eingebettete Schriften, Medien, Custom XML
und weitere Beziehungsdateien werden abgelehnt. LibreOffices unbenutzte `fntdata`-MIME-Deklaration
erlaubt keine entsprechende Datei. Vorschauen sind keine Manuskriptbilder und werden nicht
importiert.

Der Reader löst verwendete Absatz-/Zeichenstile einschließlich Vererbung auf, erkennt Zyklen und
prüft ihre unterstützten Eigenschaften. Absatzrollen müssen mit den tatsächlichen Überschriften
übereinstimmen. Nummerierungsinstanzen müssen zur Liste passen: Ebene 0, Bullet oder Dezimal, Start
1, getrennte Zähler für getrennte geordnete Listen. Fortsetzungsabsätze erhalten keine neue Nummer.
Die logische Verschachtelung kommt aus dem Inventar, nicht aus visuellen Einzügen.

Tabellen, Bilder, Felder, Links, Änderungsverfolgung, unbekannte Inhaltsknoten oder aktive
Formatierungen außerhalb des Vertrags werden abgelehnt. Bekannte reine Layoutinformationen werden
eng begrenzt normalisiert; beispielsweise `w:bidi` nur ausgeschaltet, bekannte Ligaturangaben und
LibreOffices `w:formProt` nur ausgeschaltet. Es gibt keinen heuristischen Ersatzimport.

## Ressourcen und Veröffentlichung

| Grenze                                    | Wert            |
| ----------------------------------------- | --------------- |
| DOCX-Datei                                | 8 MiB           |
| ZIP-Einträge                              | 128             |
| Einzelpart / unkomprimierte Summe         | 8 MiB / 32 MiB  |
| Kompressionsverhältnis je Part            | höchstens 100:1 |
| XML-Tiefe / globale XML-Knoten            | 64 / 100000     |
| Logische Manuskripttiefe / Inventarknoten | 32 / 4096       |
| Manuskripttext einschließlich Titel       | 2 MiB UTF-8     |
| Modell-Inlines                            | 100000          |

DTD, Processing Instructions und ungültiges XML/UTF-8 werden abgelehnt. Dateioperationen akzeptieren
nur reguläre unverknüpfte `.docx`-Dateien mit stabiler Länge. Ein verbleibendes
plattformübergreifendes Dateisystemrennen ist kein vollständig gelöstes Sicherheitsversprechen.

`kraqel-export` bietet `Manuscript::from_documents`, `Scene::editor_document`,
`read_docx_roundtrip`, `read_docx_roundtrip_file` und `write_docx_roundtrip`. Der Writer erzeugt
deterministische ZIP-Bytes, liest sie mit demselben nativen Reader vollständig zurück und vergleicht
die kanonische Semantik. Erst danach veröffentlicht `export_project_roundtrip` über die vorhandene
atomare Schreibfunktion `exports/manuskript-roundtrip.docx`. Bei Validierungsfehlern bleibt ein
vorhandenes Ziel erhalten. Fehler enthalten keine Eingabetexte, Pfade oder XML-Diagnosen.

Gewöhnliche Exporte verwenden dasselbe semantisch validierte Modell, übernehmen aber nicht die neuen
Roundtrip-Grenzen für Textmenge, Knoten oder Inlines. Der bisherige enge DOCX-Szenenimport verwendet
den ausgelagerten gemeinsamen ZIP-/XML-Kern ohne neue IPC- oder UI-Verträge. Python und Office
werden ausschließlich zur Entwicklung und Prüfung verwendet, niemals vom App-Reader.

## Nachweise und nächste Stufe

Direkt vom Rust-Writer erzeugte v1-Fixtures bestehen echte Speichervorgänge in Word 16.0.20527.20112
und LibreOffice 26.8.3.1. Eine echte Word-Textänderung wird vollständig übernommen. Die vier Dateien
unter `crates/export/tests/fixtures/roundtrip-v1/` werden ohne Metadatenmigration vom Rust-Test
gelesen. Nur Office-Core-Metadaten und ZIP-Zeitstempel wurden neutralisiert; Dokument, Styles,
Nummerierung, Marker und Inventar bleiben unverändert.

Zusätzlich werden die sechs früheren flachen Prototyp-Fixtures mit einer ausdrücklich testinternen
Umschreibung ausschließlich der Profilkennungen geprüft. Diese Brücke ist keine
Produktkompatibilität. Negativtests prüfen beschädigte Pakete, Inventare, Marker, Nummerierung,
Styles und unbekannte Inhalte. Die Office-Nachweise gelten für den synthetischen Korpus und die
genannten Builds, nicht für jedes DOCX oder beliebige Office-Bearbeitungen.

Als Nächstes folgen getrennte UI-/Command-Aktionen und ein vollständig validierter, atomarer Import
als neue Projektkopie mit neuen UUIDs. Erst dessen native und browserseitige Abnahme ergänzt den
bedienbaren Roundtrip; F08/M3 und Medieneinbettung bleiben getrennt zu bewerten.
