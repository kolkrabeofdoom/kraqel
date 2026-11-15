# KQ-065: PDF-Schrift-/Unicode- und Bildabnahme

Stand: 15. November 2026. Bezug: Issue #127, ADR 0032, freigegebener KQ-062-Vertrag; Pflichtenheft
Revision 1 F08/F09, M3 und Abschnitte 10/11. Ausschließlich synthetische Inhalte.

## Schrift und Text

Vier unveränderte Noto-Sans-2.008-TTFs werden offline mitgeliefert. Herkunft, vollständige Lizenz,
Copyright und SHA-256 stehen unter `crates/export/fonts/`. Die Rust-Prüfung bestätigt die
Einbettungsberechtigung und Glyphen in allen vier echten Schnitten. Unabhängiges Rücklesen mit pypdf
bestätigt vier eingebettete FontFile2-Streams, deren Hashes exakt den gelieferten TTFs entsprechen,
sowie vorhandene ToUnicode-CMaps.

NFC-komponierte und zerlegte Umlaute, ß, é, griechischer und kyrillischer Text, Eurozeichen,
Fett/Kursiv/Fettkursiv, harte Umbrüche, Listen, Zitate und lange ungetrennte Wörter sind geprüft.
Fehlende Glyphen, Emoji, CJK, arabischer Text, nicht komponierbare kombinierende Zeichen und
verdeckte Richtungs-/Verbindungszeichen werden ausdrücklich abgelehnt. Die exakten Bereiche stehen
im Datenformat; dies ist keine universelle Unicode-/Shaping-Abnahme.

## Bild- und Seitenprüfung

Der explizit aktivierte Rust-Helfer `generate_illustrated_pdf_qa` erzeugt vier Szenen mit zwei
Originalen und drei Bildvorkommen: Querformat mit Transparenz, Hochformat, Wiederverwendung,
zweizeilige und 1200 Zeichen lange Beschriftung. Alle drei A4-Seiten wurden mit Poppler bei 130 dpi
gerendert und vollständig visuell geprüft: korrekte Schriften und Farben, proportional skalierte
Bilder, keine abgeschnittenen oder überlagerten Inhalte. Bild, sichtbare Beschreibung und
Beschriftung bleiben zusammen. Der lange Bildabschnitt wechselt vollständig auf Seite 3.

Pillow/pypdf bestätigen alle PNG-SHA-256, vollständig deckende Weißkomposition, identische
PNG-RGB-/PDF-XObject-Pixel, DeviceRGB/8 Bit sowie die Manifestzuordnung über `KRAQELImageId`. Der
native Selbstvergleich berücksichtigt zusätzlich die Maße. Ein Regressionstest unterscheidet Bilder
mit identischen RGB-Bytes, aber unterschiedlichen Abmessungen.

Der zuletzt geprüfte Korpus hat drei Seiten, drei Bildvorkommen, zwei PNG-Nachweise und vier
eingebettete Schriften. Manifest-SHA-256:
`6b359e3316340bdc70da77feaf986e04c2db993593d689ed1fb4fc253531c2c3`. PDF-SHA-256 dieses Prüflaufs:
`7d6274b264efe0fdecd871427fb9412a50a51645fb6ff9eeaebb8b875747f07a`. PDF-Containerbytes werden wegen
interner Kennungen/Zeitstempel nicht als deterministisch zugesagt.

## Native Windows-App

Ein echter Tauri-Debug-Build verwendet die getrennte Test-ID `org.kraqel.desktop.pdf-media-test` und
ausschließlich temporäre Testdaten. In der Oberfläche: Projekt öffnen, Export, Bildauswahl laden,
direkt verknüpftes Bild ausdrücklich auswählen, Unicode-Alternativtext eingeben und „PDF mit Bildern
erzeugen“ auslösen.

Die App meldet eine vollständige neue PDF-Generation mit einem Bild und einem Vorkommen.
Unabhängiges Rücklesen bestätigt eine Seite, den eingegebenen Text `ÄÖÜ Ελληνικά Кириллица €` sowie
identische eingebettete RGB-Pixel und PNG-Nachweis. Ein anschließender Versuch mit einem Emoji im
Alternativtext wird abgewiesen; die bestehende Generation bleibt bytegleich und es entsteht kein
Teilpaket.

## Automatisierung und Grenzen

`pnpm check`: 161 Rust- und 53 TypeScript-Tests bestanden; sechs explizite QA-Helfer regulär
ignoriert. Browser-E2E: alle 26 Tests einschließlich PDF-Aktion und Axe bestanden. Native
Windows-Debug-Kompilierung bestanden. Der 100-Szenen-/50-Bilder-Korpus, Manifest-/PNG-Stabilität,
Grenzen, Konfliktabbruch und vorherige Formate sind automatisiert geprüft.

Keine PDF/UA-, PDF/A-, PDF/X-, Druckfarb-, assistive Vollabnahme oder PDF-Roundtrip-Zusage.
Decoder-/Paketgrenzen sind keine harte Prozessspeicher-Garantie. Drehbuchformatierung, vollständige
F08-/M3-Abnahme und Zweitrechner-/Clean-VM-Releaseabnahme bleiben offen.
