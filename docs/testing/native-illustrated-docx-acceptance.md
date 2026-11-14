# Native Abnahme des gewöhnlichen DOCX-Bildexports

Stand: 14. November 2026, KQ-064 / Issue #125. Pflichtenheft Revision 1 F08/F09, M3 und Abschnitte
10/11; ADR 0031. **LibreOffice bestanden; Word-Abnahme noch offen.**

## Prüfkorpus und Isolation

Das explizit aktivierte Rust-QA-Testwerkzeug `generate_illustrated_docx_qa` erzeugt ein rein
synthetisches Manuskript mit einem Kapitel, vier Szenen, drei Bildvorkommen aus zwei Originalen,
Querformat 640×320 und Hochformat 600×1000, Transparenz, getrennten Farbfeldern zur
Orientierungsprüfung, Alttexten, Umlauten und XML-Sonderzeichen. Eine Beschriftung besitzt zwei
Zeilen, eine weitere genau 1200 Unicode-Zeichen. Keine echten Manuskripte oder personenbezogenen
Bilddaten. Dateien liegen ausschließlich unter dem Arbeitsverzeichnis `work/docx-media-qa`.

Die native App verwendet `KRAQEL DOCX Bilder Test`, Bundle-ID `org.kraqel.desktop.docx-media-test`
und `$TEMP/kraqel-docx-media-test-20261114`. Reguläre App-Daten werden nicht geöffnet. Der native
Rust-QA-Seed erzeugt dort genau ein synthetisches Projekt.

Der echte Tauri-Debug-Build startet, öffnet dieses Projekt und zeigt die neue DOCX-Aktion nach
expliziter Bildauswahl. Eine über die native Oberfläche gesetzte Beschriftung „Native DOCX-Probe:
ÄÖÜ & <Szenenende>.“ wird korrekt exportiert. Erfolg mit einem Bild/einem Vorkommen, 3703 Bytes und
genau zwei Dateien. Manifest, interner PNG-Hash und maskierter Beschriftungsinhalt der tatsächlichen
App-Ausgabe sind zusätzlich dateibasiert geprüft. Die isolierte Test-App wurde anschließend
geschlossen.

## LibreOffice

LibreOffice **26.8.3.1**, Build `861effc5289d539f15c4c18d1ba8579ffd78728b` auf Windows. Öffnung und
DOCX-Speicherung mit eigenem temporärem LibreOffice-Profil im Headless-Modus; Seitenrendering über
den gebündelten `render_docx.py` mit der auf Windows vorhandenen nativen LibreOffice-Version. Keine
produktive Laufzeitabhängigkeit der App von LibreOffice oder Python.

- Das native DOCX lässt sich öffnen und auf drei Seiten rendern. Alle drei Seiten wurden einzeln
  visuell geprüft: unverzerrte Bilder, erwartete Transparenz/Farbquadranten, vollständige
  Beschriftungen, echte Zeilenumbrüche, keine Überlagerungen oder abgeschnittenen Inhalte.
- Nach echter LibreOffice-DOCX-Speicherung bleiben alle 13 nichtleeren Textabsätze einschließlich
  der 1200-Zeichen-Beschriftung, alle drei Alttexte und alle Bildpixel/Abmessungen gleich.
- LibreOffice erzeugt drei statt zwei interne Bildparts; die wiederholte Verwendung bleibt
  pixelgleich. Das ist eine zulässige Office-Neupaketierung, keine Bytegleichheit des DOCX.
- Die gespeicherte Datei wird erneut gerendert. Alle drei Seiten sind pixelidentisch zu den zuvor
  visuell geprüften Seiten des nativen Dokuments.

| Datei                   | SHA-256                                                            |
| ----------------------- | ------------------------------------------------------------------ |
| Native KRAQEL-Ausgabe   | `3396ff1633a0c199fcf43afdca0fdd209a8b56022b24a4ff61fd0c4b64e62556` |
| LibreOffice-Speicherung | `c8a86ceb69b71f4f48d92f241df74b60ed7e3105cd6a5b4a7daaceef45b18b31` |

## Word und verbleibende Abnahme

Das vorhandene Word zeigt seine erstmalige Datenschutz-Einrichtung. Diese muss mit den vom Nutzer
gewünschten Einstellungen manuell abgeschlossen werden. Bis dahin sind Word-Öffnung, Speicherung,
Bild-/Alttextvergleich und visuelle Seitenprüfung ausdrücklich **nicht abgenommen**. Die
LibreOffice-Ergebnisse ersetzen diesen Nachweis nicht. Es wurden keine Datenschutz-Einstellungen
automatisch verändert.

## Automatisierte Ergänzung und Grenzen

`pnpm check` besteht mit 156 Rust- und 52 TypeScript-Tests; fünf explizite native/visuelle QA-Helfer
sind regulär ignoriert. Alle 26 Browser-E2E bestehen einschließlich DOCX-Aktion und Axe. Der
abschließend erweiterte DOCX-Wiederverwendungstest besteht zusätzlich mit Clippy. Rust prüft 100
Szenen/50 Bilder, 51 Vorkommen aus 50 internen PNGs, Manifesthashes, XML, interne Beziehungen,
Zeichnungsreihenfolge, Maskierung und explizite Roundtrip-v1-Abweisung. Native Commandtests
veröffentlichen DOCX mit demselben Snapshot-/Konfliktpfad wie Markdown/HTML.

Die Abnahme behauptet keine Layoutgleichheit zwischen ungeprüften Office-Versionen, keine
vollständige Screenreader-Prüfung und keinen Bild-Roundtrip. Die Manifesthashes beziehen sich auf
die ursprüngliche KRAQEL-Ausgabe. Office-Metadaten aus gespeicherten Prüfdateien werden nicht als
Produktdaten übernommen. PDF-Schrift-/Unicode-Abnahme und bebildertes PDF bleiben offen; F08/M3 sind
weiterhin nur teilweise erfüllt.
