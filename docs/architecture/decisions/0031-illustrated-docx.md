# ADR 0031 Gewöhnliches DOCX mit internen Szenenbildern

Status: technische Entscheidung angenommen am 14. November 2026; KQ-064 / Issue #125.
Word-/LibreOffice-Abnahme bestanden. Grundlage: freigegebener KQ-062-Vertrag, KQ-063 / ADR 0030 und
Pflichtenheft Revision 1 F08/F09, M3, Abschnitte 10/11.

## Entscheidung

Der vorhandene Medienexportkern und Command v1 akzeptieren zusätzlich das additive Format `docx`.
Auswahl, Bildtexte, konsistenter SQLite-Lesestand, Hashprüfung, PNG-Ableitungen, Budgets und atomare
neue Paketgenerationen werden gemeinsam verwendet. Keine zusätzliche Bibliothek oder Migration.

Ein eigener gewöhnlicher DOCX-Writer nutzt die bestehenden Textblock- und Run-Renderer und ergänzt
Inline-Zeichnungen in getrennten Absätzen am Szenenende. PNG-Ableitungen liegen dedupliziert unter
`word/media/<sha256>.png`, Bildbeziehungen sind ausschließlich intern. Das äußere Paket enthält DOCX
und Herkunftsmanifest; dessen `location` referenziert den internen ZIP-Part. Der konkretisierte
[Formatvertrag](../../data-format/ILLUSTRATED_MANUSCRIPT.md) legt Parts, Geometrie und Grenzen fest.

Letter mit 1-Zoll-Rändern und intrinsischer 96-dpi-Größe erhält die bisherigen Textkonventionen.
Bilder werden proportional auf Satzbreite und 600 pt Höhe begrenzt, niemals hochskaliert,
beschnitten oder schwebend positioniert. Titel und Alttext werden als XML-Attribute maskiert,
Beschriftungen als Klartextabsätze mit echten Zeilenumbrüchen ausgegeben. Kurze Beschriftungen
werden beim Bild gehalten; lange dürfen ohne feste Absatzhöhe umbrechen.

Sortierte ZIP-Parts, feste Zeit-/Moduswerte und vollständiger nativer Part-Bytevergleich vor
Veröffentlichung ergänzen die bestehenden Paketprüfungen. Der DOCX-Text-Roundtrip erhält weder
Bildparts noch gelockerte Allowlisten; bebildertes gewöhnliches DOCX ist weiterhin kein
Roundtrip-v1-Dokument und wird von dessen Reader abgewiesen.

## Nachweis und Folgen

Rust prüft 100 Szenen/50 eingebettete Originale, wiederholte Verwendung, Auftragswiederholbarkeit,
interne Beziehungen, Alttexte, Beschriftungen, Manifest-/Bildhashes und Roundtrip-Abweisung. Native
Commandtests verwenden zusätzlich DOCX; React/E2E bieten dieselbe explizite Auswahl für alle drei
Formate und prüfen Axe. Office-Abnahme und Grenzen stehen im
[Abnahmeprotokoll](../../testing/native-illustrated-docx-acceptance.md).

Office-Speichervorgänge dürfen Bildparts anders benennen oder zusätzliche Metadaten erzeugen; der
ursprüngliche Herkunftsnachweis behauptet keine unveränderten Containerbytes danach. Pixel-, Text-
und Alttextvergleich sowie Seitenprüfung sind getrennte Nachweise. PDF bleibt die nächste
Formatstufe mit vorheriger freier Schrift-/Unicode-Abnahme; F08/M3 bleiben teilweise offen.
