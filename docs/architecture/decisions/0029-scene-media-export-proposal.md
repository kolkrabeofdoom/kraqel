# ADR 0029 Vorschlag für explizit ausgewählte Szenenbilder im Export

Status: vorgeschlagen am 14. November 2026; KQ-062 / Issue #121. Die Platzierung am Szenenende ist
ausdrücklich gewählt. Übrige Regeln und Parameter sind zur fachlichen Freigabe dokumentiert, keine
implementierte Funktion und keine vollständige F08-Abnahme.

## Kontext

Pflichtenheft Revision 1, F08/F09, M3 und Abschnitte 10/11 verlangen nachvollziehbare Medienausgabe
bei noch offenem Einbettungsumfang je Format. Die Medienbibliothek besitzt direkte Szenenlinks,
jedoch keine Absatzpositionen, Alttexte, Bildunterschriften oder Exportreihenfolge. KQ-060/061
liefert einen bewusst textgebundenen DOCX-Roundtrip, dessen Paketvertrag keine Bilder erlaubt.

## Vorschlag

Ein eigener Modus **Mit Szenenbildern** gibt nur ausdrücklich gewählte direkt verknüpfte Bilder am
Ende der jeweiligen Szene aus. Reihenfolge, Alternativtext und optionale Beschriftung werden für den
Auftrag festgelegt. Beziehungen zu Story-Bibel, Ereignissen oder kanonischen Profilbildern sind
keine automatische Exportauswahl. Ein eigenständiges illustriertes Modell verbindet das bisherige
kanonische Textmodell mit Bildvorkommen, ohne dessen Blockgrammatik zu erweitern.

Der [Vertragsentwurf](../../specifications/MEDIA_EXPORT_CONTRACT_DRAFT.md) definiert getrennte neue
Ordnergenerationen je Format, ein versioniertes Herkunftsmanifest und native Prüfung vor atomarer
Veröffentlichung. Bestehende Ausgaben werden nicht überschrieben; Generationsnamen bleiben außerhalb
des Manifest-/Dokumentinhalts. Markdown/HTML verwenden relative PNG-Dateien; gewöhnliches DOCX und
PDF betten geprüfte Ableitungen ein. Die bisherige Ausgabe und der Text-Roundtrip bleiben getrennt.

Metadatenarme PNG-Ableitungen schützen vor unbeabsichtigter Weitergabe ursprünglicher Dateinamen,
GPS und Bildmetadaten. Verifizierte Original- und Ableitungshashes bleiben im Manifest
nachvollziehbar. Keine Netzwerkauflösung und keine Laufzeitabhängigkeit von Office oder Python. Neue
Bibliotheken, Encoderparameter und eine einbettbare PDF-Schrift müssen vor Implementierung
eigenständig geprüft werden; sie sind mit diesem Vorschlag nicht ausgewählt oder installiert.

## Folgen und offene Freigabe

Die erste Implementierungsstufe soll gemeinsame Rust-Prüfung und Markdown/HTML liefern, danach
gewöhnliches DOCX und schließlich PDF mit vorheriger Unicode-/Schriftabnahme. Die vorgeschlagenen
Grenzen von 50 Originalen, 200 Vorkommen, 128 MiB Paket und 2400-Pixel-Ableitungen sind noch keine
gemessene Leistungs- oder Kompatibilitätszusage. Manifestgrammatik und Rendererparameter müssen vor
Implementierung fixiert werden. Ein Bild-Roundtrip bleibt eine eigene Versionsentscheidung.
SQLite-Migration, Lizenz-/Urheberdatenmodell, freies Platzieren im Editor und Drehbuchformatierung
sind nicht Bestandteil dieses Vertragsschritts.
