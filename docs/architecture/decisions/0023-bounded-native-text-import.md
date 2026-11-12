# ADR 0023: Begrenzter nativer Textimport als neue Szene

Status: angenommen am 12. November 2026

## Kontext

KRAQEL benötigt einen kleinen, überprüfbaren Importpfad für vorhandene UTF-8-Texte. Ein Import darf
weder beliebige Dateipfade in die WebView übertragen noch beim Fehler eine leere Szene zurücklassen.
Markdown ist kein einheitliches Manuskriptformat; eine behauptete Volltreue wäre daher eher
Metaphysik als Datenschnittstelle.

## Entscheidung

- Die Auswahl erfolgt im nativen Tauri-Dialog für `.txt`, `.md` und `.markdown`.
- Pfad und Bytes bleiben in Rust. Akzeptiert werden ausschließlich unverknüpfte reguläre Dateien bis
  2 MiB mit unveränderter Länge, gültigem UTF-8 und ohne NUL-Byte.
- Der Dateiname ohne Endung wird auf 240 Zeichen begrenzt als Szenentitel verwendet. Der vorhandene
  Umbenennen-Ablauf macht ihn anschließend editierbar.
- TXT wird in durch Leerzeilen getrennte Absätze überführt; einfache Zeilenumbrüche bleiben als
  harte Umbrüche erhalten.
- Markdown unterstützt Blocküberschriften, Zitate, ungeordnete und geordnete Listen sowie
  umschlossene Codeblöcke. Inline-Markdown und nicht erkannte Syntax bleiben sichtbar im Text.
- Szene, Editor-Dokument v1, Klartextindex und Position werden in einer SQLite-Transaktion erzeugt.
  Dialogabbruch erzeugt keinen Eintrag.
- Fehler überqueren die IPC-Grenze ausschließlich als bereinigte, stabile Meldungen.

## Folgen

Der Ablauf ist offline, begrenzt und verlustarm für unbekannte Syntax. Er verspricht ausdrücklich
keinen CommonMark-Vollimport, keine Medienübernahme, keinen DOCX-Import und keinen Roundtrip zurück
in das Ursprungsformat. Diese Erweiterungen benötigen eigene Verträge und adversariale Tests.
