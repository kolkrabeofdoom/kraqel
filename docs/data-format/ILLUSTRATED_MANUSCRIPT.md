# Bebildertes Manuskript v1

KQ-063 / Issue #123 implementiert den vollständig freigegebenen Medienexport-Vertrag aus KQ-062 für
**Markdown und HTML**; KQ-064 / Issue #125 ergänzt **gewöhnliches DOCX**, KQ-065 / Issue #127
**PDF**. Der Modus „Mit Szenenbildern“ gibt ausdrücklich ausgewählte, direkt verknüpfte Bilder am
Szenenende aus. Andere Projektdaten und ungewählte Bilder werden nicht eingesammelt. Der
DOCX-Text-Roundtrip, Editor-Dokument v1 und Snapshot v3 ändern ihre Version oder Grammatik nicht.

## Auftragsvertrag

`export_illustrated_manuscript` verwendet Command-Hülle v1 mit folgender Nutzlast:

```json
{
  "projectId": "018f47ea-9b2d-7cc1-98e8-b5f4f6b741a2",
  "format": "html",
  "placements": [
    {
      "sceneId": "018f47ea-9b2d-7cc1-98e8-b5f4f6b741a3",
      "assetId": "018f47ea-9b2d-7cc1-98e8-b5f4f6b741a4",
      "expectedChecksum": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
      "expectedSceneRevision": 3,
      "altText": "Ein Rabe am Schreibtisch",
      "caption": "Illustration am Szenenende"
    }
  ]
}
```

Das Beispiel ist schematisch, seine Kennungen/Hashes sind keine Testdateireferenzen. `format` ist
genau `markdown`, `html`, `docx` oder `pdf`. `caption` ist eine Zeichenkette oder `null`;
Alternativtext ist nichtleer und wird außen getrimmt. Unbekannte Felder werden abgewiesen. Keine
Pfade oder Bytes in der Anfrage. Der ursprüngliche Bildtitel wird aus SQLite gelesen, nicht vom
Browser geliefert. Ein Asset-/Szenenpaar darf nur einmal vorkommen. Die Arrayreihenfolge bestimmt
die Bildreihenfolge innerhalb jeder Szene; Rust ordnet Szenen nach dem kanonischen Manuskriptbaum.

Die Antwort enthält ausschließlich `packageName`, `format`, `fileCount`, `byteCount`, `imageCount`
und `placementCount`. `packageName` ist ein relativer Name unter dem projektgebundenen Exportordner.
Es gibt keine absolute Pfadangabe und keine Dateiübertragung über IPC. Die Bildanzahl zählt
ausgewählte unterschiedliche Quellen; identische PNG-Ableitungen werden als Datei dedupliziert.

## Dateien und Manifestgrammatik

Ein neues Paket liegt unter `exports/mit-bildern/<format>-<uuid>/` und enthält genau:

- `manuskript.md`, `manuskript.html`, `manuskript.docx` oder `manuskript.pdf`;
- `medien.json` in UTF-8, LF, eingerückt, sortierte Objektschlüssel, abschließender Zeilenumbruch;
- bei Markdown/HTML/PDF `assets/<exportSha256>.png` für jede unterschiedliche Ableitung;
- bei DOCX werden diese PNGs ausschließlich intern unter `word/media/<exportSha256>.png`
  eingebettet.

Keine Originale, Thumbnails, zusätzliche Textdateien oder aktive Inhalte. Markdown/HTML
referenzieren nur diese relativen PNG-Pfade. Das HTML besitzt `figure`, `img` mit Alternativtext und
proportionalen Maßen sowie optionale `figcaption`. Bildtexte werden maskiert, Beschriftungsumbrüche
bleiben sichtbar. Das Dokument benötigt keine entfernten Fonts, Scripts oder Netzwerkressourcen. Bei
Weitergabe muss der ganze Ordner erhalten bleiben; die äußere zufällige Generationskennung
beeinflusst dessen Inhalt nicht. Ältere Pakete bleiben erhalten; automatische Bereinigung gehört
nicht zu v1.

`medien.json` besitzt exakt die Wurzelfelder `format`, `version`, `manuscriptFormat`,
`manuscriptFile`, `assets`, `placements`:

| Feld                                 | Bedeutung                                                                                                          |
| ------------------------------------ | ------------------------------------------------------------------------------------------------------------------ |
| `format`, `version`                  | `kraqel-illustrated-manuscript`, Zahl `1`                                                                          |
| `manuscriptFormat`, `manuscriptFile` | `markdown` / `manuskript.md`, `html` / `manuskript.html`, `docx` / `manuskript.docx` oder `pdf` / `manuskript.pdf` |
| `assets`                             | Nach `sourceSha256` sortierte Einträge mit lokalen Kennungen `m0001` usw.                                          |
| `placements`                         | Nach Kapitel-/Szenen-/Bildposition sortierte Vorkommen                                                             |

Ein Asset hat exakt `id`, `sourceSha256`, `sourceMimeType`, `sourceByteCount`, `sourceWidth`,
`sourceHeight`, `provenance`, `exportSha256`, `exportMimeType`, `exportByteCount`, `exportWidth`,
`exportHeight`, `location`; nur bei PDF zusätzlich `pdfImageId`. Herkunft ist `imported`,
Export-MIME `image/png`, Hashes sind 64 kleine Hexzeichen. `location` entspricht bei
Markdown/HTML/PDF exakt `assets/<exportSha256>.png`, bei DOCX exakt `word/media/<exportSha256>.png`.
Originalmaße gelten vor der Orientierung, Exportmaße danach. Der Quellhash bindet die Originalbytes,
der Exporthash die PNG-Bytes.

Ein Vorkommen hat exakt `chapterPosition`, `scenePosition`, `position`, `assetId`, `title`,
`altText` und optional `caption`. Positionen sind nullbasiert, `assetId` referenziert eine lokale
Manifestkennung. Eine nicht vorhandene Beschriftung wird weggelassen, nicht als `null` serialisiert.
UUIDs, Pfade, Originalnamen, Tags, vollständige Verknüpfungen, Prompts und unbekannte Lizenzangaben
fehlen. `imported` ist eine technische Herkunft, keine Behauptung über Nutzungsrechte.

## Native Verarbeitung

Ein konsistenter SQLite-Lesestand umfasst Projekttitel, alle Dokumente und ausgewählte Assetdaten
einschließlich ihrer Verknüpfungen. Erwartete Revisionen und Hashes müssen passen. Für jeden Auftrag
wird nur ein Projekt gleichzeitig verarbeitet. Vor der Veröffentlichung wird ein `IMMEDIATE`-
Schreibschutz des Projekts gehalten und der vollständige erfasste Stand erneut verglichen. Dieser
Schutz bleibt bis nach dem Umbenennen des Paketverzeichnisses bestehen; fremde Änderungen erzeugen
einen Konflikt statt einer gemischten Ausgabe.

Ausgewählte Dateien müssen regulär, unverknüpft und innerhalb des tatsächlichen
`assets`-Verzeichnisses liegen. Lesen erfolgt aus einem begrenzten offenen Handle mit
Längenvergleich. Die eingefrorenen Bytes werden gegen Hash, MIME, Größe und Abmessungen geprüft.
Dekodiert wird ausschließlich dieser Puffer, kein erneutes Öffnen nach der Prüfung. Externe
Dateisystemrennen bleiben wie bei anderen plattformübergreifenden Dateioperationen ein Restrisiko;
eine Herkunftssignatur wird nicht behauptet.

Die gelockte `image`-Version 0.25.9 verarbeitet PNG, JPEG und statisches WebP. APNG, animiertes
WebP, MPO und unbekannte Formate werden abgewiesen. Orientierung wird vor Skalierung angewandt.
Verkleinert wird nur bei langer Kante über 2400 Pixel mit **Lanczos3**, anschließend RGBA8.
PNG-Encoding verwendet **CompressionType::Default** und **FilterType::Adaptive** ohne mitgeführte
Metadaten. Der aktuelle Lockfile-Stand gehört zum Reproduzierbarkeitsnachweis. Keine Hochskalierung
und kein stilles Qualitätsfallback bei Überschreitung der 8-MiB-Ableitungsgrenze.

Die Decoder-Dimensionsgrenzen gelten schon beim PNG-Headerlesen, das konfigurierte Decoderbudget
beträgt 160 MiB. Letzteres ist eine durch die Bibliothek nur teilweise streng durchsetzbare Grenze,
kein garantierter Prozess-RSS-Deckel. Die Pixel-/Bytegrenzen und sequenzielle Verarbeitung gelten
zusätzlich; alle 50 Originale werden niemals gleichzeitig vollständig dekodiert.

## Harte Grenzen und Fehler

Es gelten 50 unterschiedliche Quellen, 200 Vorkommen, 25 MiB / 16384 Pixel / 40 Millionen Pixel je
Original, 128 MiB Quellbytes und 200 Millionen Quellpixel nach Deduplizierung. Ableitungen bleiben
unter 2400 Pixel langer Kante und 8 MiB. Das Paket bleibt unter 128 MiB, 64 regulären Dateien und 1
MiB Manifest. Titel/Alttext/Beschriftung: 240/500/1200 Unicode-Zeichen. Steuerzeichen werden
abgewiesen; nur Beschriftungen dürfen LF enthalten. Der App-Lesestand begrenzt zusätzlich sämtliche
Editor-JSON-Bytes auf 32 MiB und den Dokumentbaum auf 10000 Knoten. Der Kern zählt beim
32-MiB-Budget zusätzlich Projekt- und Dokumenttitel mit. Bestehende reine Exporte übernehmen diese
neuen Auftragsgrenzen nicht.

Fehlende, veränderte, nicht dekodierbare, animierte, fremde oder nicht direkt verknüpfte gewählte
Bilder brechen den Auftrag ab. Nicht ausgewählte defekte Originale sind nicht Teil der Paketprüfung.
Fehler verwenden die vorhandenen bereinigten Klassen `INVALID_REQUEST`, `CONFLICT`, `INTERNAL`,
niemals Rohpfade, Bilddaten oder Decodertexte. Die UI meldet keinen erfolgreichen Teilexport.

Das Paket wird zunächst vollständig in einem neuen Staging-Verzeichnis geschrieben und
synchronisiert. Jede erzeugte Datei wird bytegenau zurückgelesen. Nach finalem Zustandsvergleich
wird das gesamte Verzeichnis unter neuem UUID-Namen umbenannt. Bei Fehlern entfernt der Guard den
unveröffentlichten Zwischenstand. Alte Pakete werden nicht angefasst; belegte oder ungeeignete Ziele
sind Fehler.

## Nachweis und nächste Stufe

Rust prüft 100 Szenen/50 synthetische Originale, wiederholte Verwendung, Ordnung, Maskierung,
Manifest-/Ableitungshashes, Metadatenentfernung, Orientierung, statisches WebP und echte
Animationsfixtures. Konflikt- und Fehlertests prüfen die Veröffentlichung ohne Teilpakete.
Browser-E2E prüft Auswahl, Bildtexte, beide Ausgaben, erneutes Laden, Fehler und Axe. Native und
visuelle Ausgabeprüfung wird im separaten Abnahmeprotokoll dokumentiert.

Der DOCX-Bildwriter ist mit KQ-064 und Office-Abnahme ergänzt, PDF mit KQ-065 und separater
Schrift-/Unicode- sowie nativer/visueller Abnahme. Dies erweitert keinen DOCX-Wiederimport und
belegt F08/M3 weiterhin nur im dokumentierten Teilumfang.

## Gewöhnliches DOCX mit Bildern

KQ-064 / ADR 0031 ergänzt den gewöhnlichen Weitergabeexport, keinen Bild-Roundtrip. Der äußere
Ordner enthält genau `manuskript.docx` und `medien.json`; für die Anzeige reicht die DOCX-Datei. Das
Manifest verweist auf interne ZIP-Parts. Es enthält weiterhin ausschließlich paketlokale IDs.

Der Container enthält genau `[Content_Types].xml`, `_rels/.rels`, `word/document.xml`,
`word/styles.xml`, `word/_rels/document.xml.rels` sowie deduplizierte PNGs unter `word/media/`.
Interne Bildbeziehungen verwenden `Type=…/image` und `Target=media/<hash>.png`, kein TargetMode
External, keine Hyperlinks, OLE-Objekte, Makros, alternativen Inhalte, Marker oder Custom
Properties. Bildvorkommen sind `wp:inline` in eigenen Absätzen am Szenenende, niemals schwebende
`wp:anchor`. Jedes Vorkommen besitzt eine eindeutige Zeichnungskennung, den sichtbaren Titel und den
Alttext in `wp:docPr/@descr` sowie eine gegen Verzerrung gesperrte Bildgeometrie.

Layout: Letter 12240×15840 Twips, Ränder 1440 Twips, Times New Roman 12 pt. Intrinsische Bildgröße
bei 96 dpi (9525 EMU/Pixel), maximal 5943600 EMU breit und 7620000 EMU hoch (6,5 Zoll / 600 pt).
Beide Grenzen wirken proportional und ohne Hochskalierung. Kurze Beschriftungen werden durch
keepNext am Bildabsatz mitgeführt. Lange Beschriftungen dürfen regulär über Seiten umbrechen; keine
festen Absatzhöhen oder abgeschnittenen Textfelder. LF wird als echter `w:br` ausgegeben.

Das ZIP wird mit sortierten Parts, Deflate, festem Standardzeitpunkt und 0644-Dateimodus
geschrieben. Vor Veröffentlichung werden Dateiliste und sämtliche Partbytes aus dem ZIP
zurückgelesen und mit den erzeugten Bytes verglichen. Auch der unkomprimierte Containerinhalt ist
auf 128 MiB und 64 Parts begrenzt; die äußere Paketgrenze zählt zusätzlich das Manifest.
Wiederholbarkeit wird mit der gelockten Toolchain getestet, nicht als allgemeine
Office-Speicherbytegleichheit versprochen. Word/LibreOffice dürfen bei erneutem Speichern Metadaten,
Layoutdetails und interne Dateinamen ändern. Der Herkunftsnachweis bezieht sich auf die
ursprüngliche KRAQEL-Ausgabe. Der Roundtrip-v1-Reader weist dieses gewöhnliche DOCX ausdrücklich ab.

## PDF Schrift Unicode und Bildnachweis

KQ-065 / ADR 0032 gilt für bebildertes **und reines Text-PDF**. Noto Sans 2.008 wird in Regular,
Bold, Italic und BoldItalic vollständig eingebettet; Originaldateien, Lizenz, Copyright und Hashes
liegen unter `crates/export/fonts/`. Keine Systemfont- oder Netzwerkabhängigkeit. Glyphenprüfung und
Abmessungen stammen aus genau diesen vier Dateien. Die Zusatzabhängigkeit `unicode-normalization`
0.1.25 ist MIT/Apache-2.0-lizenziert; MIT-Text unter `docs/licenses/`.

Jeder Text wird zur Ausgabe nach NFC normalisiert. CRLF/CR werden zu LF, Tabs zu vier Leerzeichen.
Geprüft werden Manuskript, Überschriften, generierte Listenzeichen sowie gewählte Bildtitel,
Alttexte und Beschriftungen. Die Zeichengruppen sind U+0020–02FF, U+0370–052F, U+1E00–1FFF,
U+2000–206F, U+20A0–20CF, U+2100–214F, U+2190–21FF, U+2200–22FF und U+25A0–25FF; **zusätzlich muss
jeder konkrete Glyph im verwendeten Schnitt existieren**. LF wird als Zeilenumbruch geplant. U+00AD,
U+200B–200F, U+2028–202E und U+2060–206F sind ausgeschlossen. Nach NFC verbleibende kombinierende
Zeichen, Emoji, CJK und komplexe Schriften oder Schriften mit Schreibrichtung von rechts nach links
werden abgewiesen. Die Bereiche behaupten keine vollständige Abdeckung. Beispielsweise wird der in
diesen Dateien fehlende Pfeil U+2192 abgewiesen. Kein stilles Ersetzen oder Weglassen; SQLite bleibt
unverändert.

A4 beträgt 210×297 mm. Der Inhaltsbereich verwendet linke/bottom Grenze 56,7 pt, obere Grenze 785 pt
und 481,8 pt Breite. Textbaselines werden mit der tatsächlichen maximalen Font-Oberlänge berechnet;
Vorschübe bestimmen Umbruch und gemischte Schriftläufe. Harte Umbrüche, Leerzeichen, Fett/Kursiv,
Listen und Zitate bleiben sichtbar. Überlange Einzelwörter dürfen zeichenweise umbrechen.
Proportionale intrinsische Bildgröße bei 96 dpi, höchstens Satzbreite und 540 pt Höhe, mit weiterer
proportionaler Layoutbegrenzung, damit Bildbeschreibung und Beschriftung vollständig auf dieselbe
Seite passen. Lange Ableitungen werden dabei nicht neu gerastert. Ein Abschnitt, dessen Texte allein
eine Inhaltsseite überschreiten, ist ungültig; kein abgeschnittener Teilexport.

Die PNG-Ableitung verwendet weiterhin RGBA8 und dieselben Filter-/Encoderparameter, aber vollständig
opaken weißen Hintergrund. Pro Kanal: `(Farbe * Alpha + 255 * (255 - Alpha) + 127) / 255` mit
Ganzzahldivision. `assets/<exportSha256>.png` enthält exakt diese Bytes. Die PDF bettet dieselben
RGB-Pixel mit 8 Bit je Kanal und ohne Softmask verlustfrei ein. Zum Anzeigen reicht allein die PDF;
für den vollständigen Herkunftsnachweis müssen Manifest und PNGs mitgegeben werden.

`pdfImageId` ist `i<exportSha256>` und steht als Name `/KRAQELImageId` am tatsächlichen
PDF-Bild-XObject. Der native Rückleser vergleicht für jedes Bild RGB-Hash, Breite, Höhe und Profil
mit den Nachweis-PNGs, bevor veröffentlicht wird. Wiederverwendung darf mehrere gleich zugeordnete
Bildobjekte erzeugen. Bei anderen Formaten fehlt dieses additive Feld. Der Alternativtext erscheint
zusätzlich sichtbar als `Bildbeschreibung: …`; die optionale Beschriftung folgt als eigener Text.
Dies ist keine Zusage eines getaggten PDF oder von PDF/UA.

Zusätzlich zu sämtlichen bisherigen Grenzen gelten gleichzeitig: 8 MiB Textbytes vor Normalisierung,
100000 geplante Zeilen, 2048 Seiten und 128 MiB unkomprimierte RGB-Pixelbytes über **alle**
Bildvorkommen. Ein häufig verwendetes großes Bild kann diese letzte Grenze verletzen, obwohl es nur
einmal als Original/Nachweisdatei gespeichert wird. Fehler veröffentlichen keine neue Generation.
PDF-Containerbytes sind wegen interner IDs/Metadaten nicht reproduzierbar zugesagt; Manifest und
Ableitungen bleiben bytegleich. Keine Änderung am DOCX-Text-Roundtrip oder Snapshot v3.
