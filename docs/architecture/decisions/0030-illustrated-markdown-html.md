# ADR 0030 Nativer Szenenbildexport für Markdown und HTML

Status: angenommen am 14. November 2026; KQ-063 / Issue #123. Der Medienexport-Vertrag aus KQ-062
wurde vollständig freigegeben. PR #122 ist mit grüner Windows-/Linux-CI zusammengeführt.

## Entscheidung

Der Modus „Mit Szenenbildern“ ist ein eigener Command-v1-Auftrag. Er übernimmt ausschließlich
explizit ausgewählte direkt verknüpfte Szenenbilder mit Reihenfolge, Alttext und optionaler
Beschriftung. Der vorhandene kanonische Textbaum wird wiederverwendet; seine Blockgrammatik wird
nicht um Bilder erweitert. Typisierte Bildvorkommen und vorbereitete Paketdateien bilden eine eigene
Darstellung. Der verbindliche Vertrag ist
[`ILLUSTRATED_MANUSCRIPT.md`](../../data-format/ILLUSTRATED_MANUSCRIPT.md).

`kraqel-asset-service` friert begrenzte reguläre Originalbytes ein und erzeugt orientierte,
metadatenarme RGBA8-PNG-Ableitungen mit Lanczos3 und fixierten PNG-Encoderoptionen. `kraqel-export`
besitzt Auswahlvalidierung, Herkunftsmanifest, Markdown-/HTML-Ausgabe und vollständige neue
Paketgenerationen. Die Desktop-Orchestrierung erfasst einen konsistenten SQLite-Lesestand, prüft ihn
vor Veröffentlichung erneut und hält die kurze `IMMEDIATE`-Sperre bis nach dem Umbenennen.
Gleichzeitige Bildaufträge für dasselbe Projekt werden abgewiesen. Keine Datenbankmigration und
keine neuen Netzwerk-, Python- oder Office-Laufzeitabhängigkeiten.

Das Export-Crate verwendet dafür den bestehenden Asset-Service; seine Tests verwenden die bereits
gelockte `image`-Version 0.25.9. Es werden keine neuen Paketversionen eingeführt. PNG/JPEG/WebP
bleiben die einzigen aktivierten Decoder. Die vorhandenen MIT-/Apache-2.0-Lizenzen bleiben
kompatibel; Bilddecoder und Encoder sind weiterhin Teil der gepflegten Abhängigkeitsprüfung. Das
konfigurierte Decoderbudget ersetzt keine harte Betriebssystem-Speichergrenze; Pixel-, Quellen- und
Paketbudgets sowie sequenzielle Verarbeitung begrenzen den Auftrag zusätzlich.

## Veröffentlichung und Nachweise

Neue Ordnergenerationen erhalten vollständige Vorgängerpakete. Nur ein erfolgreich angelegtes
Staging-Verzeichnis erhält einen Löschguard. Erzeugte Dateien werden vor finalem Zustandsvergleich
und Veröffentlichung zurückgelesen. Das Manifest enthält ausschließlich lokale Exportkennungen,
explizite Bildtexte und technische Herkunft/Hashes, keine UUIDs, Originalnamen, Tags oder Prompts.

Tests prüfen 100 Szenen mit 50 synthetischen Originalen, stabile Bytes/Hashes, wiederholte
Bildvorkommen, Maskierung, Quellfehler, echte Animationen, Orientierung und Metadatenentfernung.
Native Commandtests belegen getrennte Pakete und Konflikte ohne veröffentlichtes Teilpaket;
Browser-E2E prüft die bedienbare Auswahl und Axe. Die echte Windows-Ausgabe ist getrennt abzunehmen.
Diese Nachweise erstrecken sich nicht auf bebildertes DOCX/PDF oder einen Bild-Roundtrip.

Als Nächstes folgt gewöhnliches DOCX mit internen Bildern und Office-Abnahme. PDF benötigt davor die
freie eingebettete Schrift und Unicode-Abnahme. Der enge DOCX-Text-Roundtrip bleibt unverändert;
F08/F09/M3 sind nur im jeweils dokumentierten Umfang umgesetzt.
