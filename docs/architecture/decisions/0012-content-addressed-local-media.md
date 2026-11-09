# ADR 0012: Inhaltsgeprüfte lokale Medien und hash-referenzierte Snapshots

Status: angenommen am 9. November 2026

## Kontext

F09 und KQ-013 verlangen eine offline nutzbare Medienbibliothek mit Import, Vorschau, Metadaten,
Schlagwörtern und Verknüpfungen. Medien sind zugleich untrusted Input, können groß sein und müssen
bei Backup und portablem Snapshot konsistent zu den SQLite-Metadaten bleiben. Gelieferte Dateinamen
dürfen weder Projektpfade bestimmen noch bestehende Dateien überschreiben.

## Entscheidung

Version 0.1 akzeptiert ausschließlich dekodierbare PNG-, JPEG- und WebP-Rasterbilder bis 25 MiB,
16.384 Pixel je Achse und 40 Millionen Pixel insgesamt. Die Rust-Grenze prüft Dateisignatur,
passende Endung, Dekodierbarkeit und Abmessungen, berechnet SHA-256 und erzeugt ein höchstens 512
Pixel großes PNG-Thumbnail. Gelieferte Dateinamen werden nur als begrenzte Metadaten gespeichert.
Projektdateien erhalten ausschließlich Namen aus Asset-UUID und Hashpräfix unter `assets/`.

SQLite-Migration 3 führt `media_asset` und `asset_link` ein. Ein Asset speichert normalisierte
relative Original- und Thumbnailpfade, MIME-Typ, Hash, Größe, Abmessungen, Herkunft und Tags. Links
adressieren Szenen, Story-Bibel-Einträge oder Ereignisse über stabile IDs. Rust prüft das Ziel vor
dem Einfügen; Löschtrigger entfernen Links zu gelöschten Zielen.

Import veröffentlicht zuerst neue, eindeutig benannte Dateien und schreibt danach die Metadaten. Bei
einem Datenbankfehler werden die neuen Dateien entfernt. Ersetzen schreibt ebenfalls neue Dateien,
schaltet dann den Datenbanksatz um und entfernt anschließend die alten Dateien. Dadurch wird kein
kanonisches Bild vor erfolgreicher Validierung überschrieben. Ein identischer Hash ist projektweit
eindeutig.

SQLite-Backups erhalten ein gleichnamiges `.assets`-Begleitverzeichnis. Die Datenbanksicherung
bestimmt die zu kopierenden Pfade; Originale werden vor der Veröffentlichung und Wiederherstellung
gegen ihren Hash geprüft. Portable Snapshots kopieren Originale als `assets/<sha256>.<endung>` und
führen Metadaten sowie Links im Manifest. Thumbnails werden nicht exportiert, weil sie
deterministisch ableitbar sind.

## Folgen

- Medienverwaltung, Vorschau und Verknüpfungen funktionieren vollständig offline.
- Fehlende oder veränderte Originale und Vorschaudateien werden in der Bibliothek sichtbar, statt
  als intakte Assets ausgegeben zu werden.
- Die UI übergibt bewusst ausgewählte Dateien als begrenzte Bytes an Rust; Rust bleibt die
  Sicherheitsgrenze. Sehr große Sammlungen benötigen später einen nativen Streamingimport.
- Gelöschte oder ersetzte Dateien werden nach erfolgreicher Datenbankänderung bestmöglich entfernt.
  Ein I/O-Fehler kann daher eine unreferenzierte Datei hinterlassen, aber keinen Datenbanksatz auf
  eine vorzeitig gelöschte Datei umbiegen.
- SVG, animierte Formate, Videos, Audio, Lizenzfelder, visuelle Profile und Bildgenerierung sind
  nicht Bestandteil dieses Inkrements.
