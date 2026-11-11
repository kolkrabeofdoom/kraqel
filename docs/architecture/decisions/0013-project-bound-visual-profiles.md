# ADR 0013: Projektgebundene visuelle Profile und Stilvorlagen

Status: angenommen am 10. November 2026

## Kontext

F09, Modul V.1 und KQ-014 verlangen konsistente visuelle Merkmale je Story-Bibel-Eintrag sowie
projektweit wiederverwendbare Stilregeln. Diese Daten können später in Bildprompts einfließen, sind
aber bereits ohne Provider nützlich und potenziell sensible Werkdaten. Referenzen dürfen weder
projektübergreifend zeigen noch beim Umbenennen von Entitäten oder Assets brechen.

## Entscheidung

SQLite-Migration 4 führt `visual_profile` und `style_preset` ein. Genau ein Profil ist über die
stabile Entitäts-UUID an einen Story-Bibel-Eintrag gebunden. Kernmerkmale sind ein begrenztes,
deterministisch sortiertes Objekt aus Namen und Werten; unerwünschte Abweichungen sind eine
begrenzte, eindeutige Liste. Ein Profil kann genau ein kanonisches Medienasset desselben Projekts
referenzieren. Wird dieses Asset gelöscht, setzt SQLite nur die kanonische Referenz auf `NULL`. Beim
Löschen der Entität wird das Profil kaskadiert entfernt.

Stilvorlagen besitzen eine stabile UUID, einen projektweit ohne Beachtung der ASCII-Großschreibung
eindeutigen Namen, strukturierte visuelle Regeln und mindestens eine unterstützte Entitätsart als
Geltungsbereich. Rust normalisiert und begrenzt sämtliche Zeichenketten, Listen und JSON-Größen und
prüft Entitäts-, Projekt- und Assetreferenzen innerhalb einer Transaktion.

Profile und Stilvorlagen bleiben ausschließlich lokal. Sie werden durch die bestehende
SQLite-Sicherung wiederhergestellt und deterministisch in `visual-profiles.json` des portablen
Snapshotformats v2 exportiert. Die lesende Prüfung behält v1-Kompatibilität. Die Snapshotprüfung
validiert Struktur sowie Entitäts- und Assetreferenzen. Kein Command sendet diese Daten an einen
Provider oder erzeugt daraus automatisch einen Prompt.

## Folgen

- Umbenennungen verändern Referenzen nicht, weil ausschließlich UUIDs gespeichert werden.
- Ein Asset kann Variantenlinks behalten, während genau eines davon kanonisch sein kann.
- Leere Profile sind zulässig, um nur ein kanonisches Asset festzulegen; Stilvorlagen benötigen
  dagegen mindestens einen Geltungsbereich.
- Die derzeitige Namenseindeutigkeit verwendet SQLite `NOCASE` und ist damit ASCII-orientiert.
  Vollständige Unicode-Kollation bleibt eine spätere Produktentscheidung.
- Prompt-Zusammenstellung, Capability-Registry und jede Netzwerkübertragung bleiben KQ-015 und
  KQ-016 vorbehalten.
- Ein eigenständiger Browser-E2E prüft Profilmerkmale, Ausschlüsse, Stilregeln, Geltungsbereiche und
  Wiederladen nach Ansichtswechsel gegen den testexklusiven Command-Adapter. Kanonische
  Referenzbilder, Tauri und Screenreader bleiben separat abzunehmen.
