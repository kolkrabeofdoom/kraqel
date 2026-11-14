# Native Windows-Abnahme des Markdown-/HTML-Bildexports

Stand: 14. November 2026, KQ-063 / Issue #123. Abgleich: Pflichtenheft Revision 1, F08/F09, M3,
Abschnitte 10/11; freigegebener KQ-062-Vertrag und ADRs 0029/0030.

## Umgebung und Isolation

Echter Tauri-Debug-Build mit gebündelter React-WebView, Rust und SQLite. Eigener Produktname
`KRAQEL Medienexport Test`, Bundle-ID `org.kraqel.desktop.media-export-test` und Datenwurzel
`$TEMP/kraqel-media-export-test-20261114`. Das ausdrücklich aktivierte Rust-QA-Seed erzeugt ein
synthetisches Projekt mit einem Kapitel, einer leeren Szene und einem direkt verknüpften
320×180-PNG. Originalname und Tag enthalten absichtlich `PRIVATE`. Reguläre App-Daten werden nicht
verwendet. Keine Netzwerk- oder Cloudverbindung ist erforderlich.

## Beobachtete Ergebnisse

| Prüfung          | Ergebnis                                                                                                                                                                                                                                                  |
| ---------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Ausgangszustand  | Keine Bilder automatisch ausgewählt; beide Bildexportaktionen gesperrt.                                                                                                                                                                                   |
| Auswahl          | „Bildauswahl laden“ zeigt das direkt verknüpfte Bild „Abend am Schreibtisch“ mit Vorschau; Checkbox aktiviert den Export.                                                                                                                                 |
| Bildtexte        | Alttext „Cremefarbene Illustration im Querformat“ und Beschriftung „Am Szenenende: ÄÖÜ & <Test>.“ über die native Oberfläche gesetzt.                                                                                                                     |
| HTML             | Erfolg mit eigenem `mit-bildern/html-<uuid>`-Paket, 1 Bild und 1 Vorkommen.                                                                                                                                                                               |
| Markdown         | Erfolg mit eigenem `mit-bildern/markdown-<uuid>`-Paket; vorheriges HTML bleibt bestehen.                                                                                                                                                                  |
| Wiederholung     | Erneuter HTML-Auftrag erzeugt eine weitere vollständige Generation.                                                                                                                                                                                       |
| Ungültige Quelle | Gewähltes synthetisches Original nach Auswahl verändert; erneuter Export scheitert mit bereinigtem sichtbarem Fehler. Keine neue Generation und kein Staging-Rest; drei frühere Pakete vollständig erhalten. Original danach bytegenau wiederhergestellt. |
| Manifest         | Alle ausgegebenen Bilddateien gegen `exportSha256` geprüft; relative Verweise auflösbar. Keine Projekt-UUID, privaten Originalnamen oder Tags im Manifest.                                                                                                |
| Portabilität     | Vollständiges HTML-Paket an anderen lokalen Pfad kopiert und über `file:` in Chromium mit Offline-Kontext geöffnet. Bild lädt mit natürlichen Maßen 320×180; Darstellung bleibt 320 Pixel breit.                                                          |
| Sichtprüfung     | Projekt-, Kapitel- und Szenentitel, unverzerrtes Bild und vollständige Beschriftung sichtbar. Umlaute, `&` und `<Test>` werden als Text dargestellt; keine Skriptausführung.                                                                              |

Die Ableitung besitzt 678 Bytes und SHA-256
`99b0c953bb4e05abe9fa1e4d3a39c239cab76e80cd31a3fd501bcc1526bf7b7f`. Das synthetische Original
besitzt SHA-256 `c4beaa95805a72abd2f36ebe54ac2698f6c49f4e19553be441015853a8da69fd`.

## Automatisierte Ergänzung und Grenzen

Rust prüft zusätzlich 100 Szenen mit 50 kleinen synthetischen Originalen, Wiederholbarkeit von
Manifest und Ableitungen, Bildreihenfolge, mehrfache Verwendung, Maskierung, Konfliktabbruch,
Quellintegrität, EXIF-Orientierung, Metadatenentfernung, statisches WebP und echte APNG-/WebP-
Animationsfixtures. Native Commandtests prüfen SQLite-Zustandsänderungen vor Veröffentlichung.
Browser-E2E prüft Auswahl, Bildtexte, beide Formate, Fehler, Auswahlneuladen, 360-Pixel-Ansicht und
Axe. Alle 26 Browser-E2E bestehen; der erste Gesamtlauf unter hoher gleichzeitiger Buildlast hatte
vier Fehlschläge, der erneute vollständige Lauf besteht ohne Codeänderung.

`pnpm check` besteht mit 154 Rust- und 50 TypeScript-Tests, einschließlich Formatierung, Lint und
Typprüfung. Die abschließend ergänzten Negativfälle für Mengen-, Quellbyte-, Pixel- und Textbudgets
bestehen zusätzlich im gezielten Rust-Lauf mit Clippy. Der native Tauri-Debug-Build ist erfolgreich.

Diese Stichprobe belegt keinen harten Prozess-RSS-Deckel, keine Clean-VM und keine vollständige
Screenreader-Abnahme des neuen Bereichs. Der Großkorpus verwendet kleine Bilder und ersetzt keinen
Benchmark am maximalen Decoderbudget. Markdown-Anzeige ist rendererabhängig; nachgewiesen sind
Dateiinhalt und relative Referenzen, keine universelle Layoutgleichheit. Gewöhnliches DOCX/PDF mit
Bildern, PDF-Schrift-/Unicode-Abnahme und vollständige F08-/M3-Abnahme bleiben offen. Der bestehende
DOCX-Text-Roundtrip und SQLite-Version 6 bleiben bestehen.
