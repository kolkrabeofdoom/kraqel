# DOCX-Import und Roundtrip: Vorschlag für den nächsten Scope

Stand: 14. November 2026. Der erste Importumfang wurde ausdrücklich freigegeben und ist als
[ADR 0024](../architecture/decisions/0024-bounded-docx-import.md) beschlossen. Implementierung und
Grenzen beschreibt der [DOCX-Importvertrag v1](../data-format/DOCX_IMPORT.md), KQ-055 / Issue #107.
Die unten beschriebene strukturerhaltende Roundtrip-Stufe bleibt ein getrennter Vorschlag.

## Bezug und vorhandene Grenzen

Pflichtenheft Revision 1 nennt F08, M3 sowie in Abschnitt 11 die offene Festlegung des
DOCX-Roundtrips und der Medieneinbettung. Entwicklungsstatus und Roadmap verlangen diese Eingrenzung
vor der Implementierung. ADR 0008, ADR 0020 und `docs/data-format/EXPORT.md` beschreiben bislang
ausschließlich den Exportvertrag.

Der aktuelle Writer in `crates/export/src/render.rs` erzeugt Projekt-, Kapitel- und Szenentitel
sowie Absatztext, Fett/Kursiv und harte Umbrüche. Listen und Zitate werden durch Textpräfixe
ausgegeben, nicht durch eigene OOXML-Struktur. Szenenüberschriften werden um zwei Ebenen verschoben
und auf Heading6 begrenzt. Dadurch sind insbesondere die ursprünglichen Ebenen 4 bis 6 nicht
eindeutig rekonstruierbar. Ein bloßer DOCX-Parser kann diese verlorenen Unterscheidungen nicht
zuverlässig wiederherstellen.

## Vorgeschlagenes erstes Inkrement: begrenzter Fremdimport

- Eine bewusst ausgewählte `.docx`-Datei wird als neue Szene in ein gewähltes Kapitel importiert.
  Der Dateistamm liefert einen anschließend editierbaren Titel. Bestehende Szenen bleiben erhalten.
- Unterstützte Semantik zunächst: Absätze, Unicode-Text, Fett/Kursiv einschließlich Kombinationen,
  harte Umbrüche und eindeutig auflösbare Überschriften der Ebenen 1 bis 6.
- Kapitel- und Szenengrenzen werden nicht aus beliebigen Word-Überschriften geraten. Dieser
  Fremdimport ist noch keine Wiederherstellung eines KRAQEL-Projekts.
- Für den ersten engen Vertrag führen nicht unterstützte inhaltstragende Konstrukte zu einer
  verständlichen Ablehnung vor der Transaktion: insbesondere Tabellen, Bilder, Fußnoten, Hyperlinks,
  Felder, Änderungsverfolgung und nicht unterstützte Listen. Keine stille Kürzung.
- Schriftarten, Seitenlayout und Paginierung sind keine Garantie des semantischen Imports; dieser
  Unterschied muss vor der Übernahme verständlich erklärt werden.
- Auswahl, Archivbytes und Pfade bleiben in Rust. Kein Office-Prozess, Makro oder externer
  Beziehungseintrag wird ausgeführt oder aus dem Netz aufgelöst.
- Vor Parserwahl werden feste Grenzen für Archivgröße, entpackte Gesamtgröße, einzelne Teile,
  Eintragsanzahl, Kompressionsverhältnis, XML-Tiefe und Dokumentknoten definiert und getestet.
  Verschlüsselte, beschädigte oder mehrdeutige Pakete, Pfadtraversal, doppelte ZIP-Namen sowie
  DTD-/Entity-Verarbeitung werden abgelehnt. DOCM gehört nicht zum Vertrag.
- Szene, Inhalt, Klartextindex und Position entstehen atomar; Abbruch oder Fehler schreibt nichts.

## Getrenntes Inkrement: definierter KRAQEL-Roundtrip

Zielvorschlag: Export, erneuter Import als neue Projektkopie und Vergleich des kanonischen
Manuskriptmodells. Garantien betreffen Titel, Reihenfolge und die ausdrücklich unterstützte
Textsemantik; weder ursprüngliche UUIDs noch ZIP-Bytegleichheit oder identisches Word-Layout.

Vorher muss der Export Kapitel-/Szenengrenzen, alle sechs Überschriftenebenen sowie Listen und
Zitate eindeutig unterscheiden. Dafür ist ein eigener versionierter Austauschvertrag samt ADR
erforderlich. Eingebettete Metadaten sind untrusted und werden vollständig validiert. Fehlende oder
durch Word veränderte Strukturmarker dürfen keinen stillen Strukturverlust auslösen. Die Garantie
für in Word bearbeitete Exporte muss separat festgelegt werden.

Medien und Story-Bibel-Referenzen bleiben über das Snapshot-v3-Archiv portabel; der enge
Manuskriptvertrag ersetzt dieses Projektformat nicht. F08 und das Export-Abnahmekriterium zu Medien
bleiben bis zur ausdrücklichen fachlichen Klärung entsprechend eingeschränkt.

## Prüfnachweise vor Abnahme

1. Gültige Minimaldateien, Word-/LibreOffice-Fixtures und fehlerhafte bzw. begrenzungsverletzende
   Pakete in Rust testen; unbekannte Inhalte dürfen nicht still verschwinden.
2. Versionierte IPC-Verträge, neutralen Abbruch und unveränderte Daten bei Fehlern prüfen.
3. Browser-E2E für den UI-Ablauf und getrennte native Windows-Dialogprüfung durchführen.
4. Für den späteren Roundtrip das kanonische Modell mit verschachtelten Listen, Zitaten, Unicode,
   leeren Absätzen, Umbrüchen und allen Überschriftenebenen vergleichen.
5. Bei einer Writer-Erweiterung OOXML-Struktur und reales Rendering erneut prüfen.

Der enge Import als neue Szene und die Ablehnung nicht unterstützter Inhalte sind mit ADR 0024
beschlossen. Vor der Roundtrip-Implementierung sind deren genauer Umfang und Garantien weiterhin
getrennt zu entscheiden; die aktuelle Importimplementierung verspricht keinen Projekt-Roundtrip.
