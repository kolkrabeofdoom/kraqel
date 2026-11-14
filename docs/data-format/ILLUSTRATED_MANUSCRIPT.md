# Bebildertes Manuskript v1

KQ-063 / Issue #123 implementiert den vollständig freigegebenen Medienexport-Vertrag aus KQ-062 für
**Markdown und HTML**. Der Modus „Mit Szenenbildern“ gibt ausdrücklich ausgewählte, direkt
verknüpfte Bilder am Szenenende aus. Andere Projektdaten und ungewählte Bilder werden nicht
eingesammelt. Gewöhnliches DOCX/PDF mit Bildern bleiben eigene Folgestufen; der DOCX-Text-Roundtrip,
Editor-Dokument v1 und Snapshot v3 ändern ihre Version oder Grammatik nicht.

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
genau `markdown` oder `html`. `caption` ist eine Zeichenkette oder `null`; Alternativtext ist
nichtleer und wird außen getrimmt. Unbekannte Felder werden abgewiesen. Keine Pfade oder Bytes in
der Anfrage. Der ursprüngliche Bildtitel wird aus SQLite gelesen, nicht vom Browser geliefert. Ein
Asset-/Szenenpaar darf nur einmal vorkommen. Die Arrayreihenfolge bestimmt die Bildreihenfolge
innerhalb jeder Szene; Rust ordnet Szenen nach dem kanonischen Manuskriptbaum.

Die Antwort enthält ausschließlich `packageName`, `format`, `fileCount`, `byteCount`, `imageCount`
und `placementCount`. `packageName` ist ein relativer Name unter dem projektgebundenen Exportordner.
Es gibt keine absolute Pfadangabe und keine Dateiübertragung über IPC. Die Bildanzahl zählt
ausgewählte unterschiedliche Quellen; identische PNG-Ableitungen werden als Datei dedupliziert.

## Dateien und Manifestgrammatik

Ein neues Paket liegt unter `exports/mit-bildern/<format>-<uuid>/` und enthält genau:

- `manuskript.md` oder `manuskript.html`;
- `medien.json` in UTF-8, LF, eingerückt, sortierte Objektschlüssel, abschließender Zeilenumbruch;
- `assets/<exportSha256>.png` für jede unterschiedliche Ableitung.

Keine Originale, Thumbnails, zusätzliche Textdateien oder aktive Inhalte. Markdown/HTML
referenzieren nur diese relativen PNG-Pfade. Das HTML besitzt `figure`, `img` mit Alternativtext und
proportionalen Maßen sowie optionale `figcaption`. Bildtexte werden maskiert, Beschriftungsumbrüche
bleiben sichtbar. Das Dokument benötigt keine entfernten Fonts, Scripts oder Netzwerkressourcen. Bei
Weitergabe muss der ganze Ordner erhalten bleiben; die äußere zufällige Generationskennung
beeinflusst dessen Inhalt nicht. Ältere Pakete bleiben erhalten; automatische Bereinigung gehört
nicht zu v1.

`medien.json` besitzt exakt die Wurzelfelder `format`, `version`, `manuscriptFormat`,
`manuscriptFile`, `assets`, `placements`:

| Feld                                 | Bedeutung                                                                 |
| ------------------------------------ | ------------------------------------------------------------------------- |
| `format`, `version`                  | `kraqel-illustrated-manuscript`, Zahl `1`                                 |
| `manuscriptFormat`, `manuscriptFile` | `markdown` / `manuskript.md` oder `html` / `manuskript.html`              |
| `assets`                             | Nach `sourceSha256` sortierte Einträge mit lokalen Kennungen `m0001` usw. |
| `placements`                         | Nach Kapitel-/Szenen-/Bildposition sortierte Vorkommen                    |

Ein Asset hat exakt `id`, `sourceSha256`, `sourceMimeType`, `sourceByteCount`, `sourceWidth`,
`sourceHeight`, `provenance`, `exportSha256`, `exportMimeType`, `exportByteCount`, `exportWidth`,
`exportHeight`, `location`. Herkunft ist `imported`, Export-MIME `image/png`, Hashes sind 64 kleine
Hexzeichen. `location` entspricht exakt `assets/<exportSha256>.png`. Originalmaße gelten vor der
Orientierung, Exportmaße danach. Der Quellhash bindet die Originalbytes, der Exporthash die
PNG-Bytes.

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

Als Nächstes folgt der gewöhnliche DOCX-Bildwriter mit Office-Nachweis, danach die PDF-Stufe mit
vorheriger Schrift-/Unicode-Abnahme. Dies erweitert keinen DOCX-Wiederimport und erfüllt F08/M3
weiterhin nur im dokumentierten Teilumfang.
