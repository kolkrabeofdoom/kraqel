# ADR 0033 Drehbuchrollen und überprüfbare F08-/M3-Abnahme

Status: **fachlich angenommen am 15. November 2026**, nach ausdrücklicher Freigabe des gesamten
Vertrags durch den Projektinitiator („Der Vertrag passt so“). KQ-066 / Issue #129; Freigabenachtrag
KQ-067 / Issue #131. Keine Implementierungs- oder Funktionsabnahme.

## Kontext

PR #128 ist mit grüner Windows-/Linux-CI und Dependency Review in `main` zusammengeführt
(`4679c89`). DOCX-Manuskriptaustausch v1 und Szenenbilder in vier Ausgabeformaten sind im
dokumentierten Umfang geprüft. Das Pflichtenheft Revision 1 lässt Drehbuchformatierung in §11 offen.
Einzelne native Abnahmen und automatisierte Export-Großkorpora ergeben noch keine vollständige
M3-Abnahme.

## Vorschlag

Der [Drehbuchvertrag](../../specifications/SCREENPLAY_FORMAT_CONTRACT_DRAFT.md) definiert sechs
explizite Rollen statt visueller Heuristiken, ein festes vorgeschlagenes A4-Profil und eigene
Kompatibilitätsgrenzen. Ein kanonisches Modell muss Rollen erhalten; ein bloß eingerückter
Prosaabsatz reicht nicht. Neue Semantik verlangt versionierte Editor-/Command-/Snapshot- und
gegebenenfalls DOCX-Verträge; bestehende v1-/v3-Profile werden nicht erweitert.

Schriftfamilie und Monospace-Profil benötigen eine eigene Herkunfts-, Lizenz-, Einbettungs- und
Glyphenprüfung. Der Drehbuch-Druckmodus enthält zunächst keine Bilder. Es gibt weder eine
automatische Konvertierung von Prosa noch eine unbemerkte Auslassung nicht gewählter Inhalte.

Die [F08-/M3-Abnahmematrix](../../testing/F08_M3_ACCEPTANCE_MATRIX.md) bildet alle einschlägigen
Pflichtenheft-Kriterien auf Code, automatisierte Tests, native Nachweise und verbleibende Arbeiten
ab. „Vollständig“ bezeichnet die Kriterienerfassung, nicht den bestandenen Gesamtmeilenstein.

## Folgen und Entscheidungstor

Die Vertragsstufe KQ-066 änderte ausschließlich Dokumentation. SQLite 6, Editor-Dokument v1, Command
v1, Snapshot v3 und DOCX-Manuskriptaustausch v1 bleiben unverändert. Die sechs Rollen, A4-Maße,
Schriftauswahlprüfung, Druckauswahl und Ausschlüsse sind durch die explizite Nutzerfreigabe am 15.
November 2026 fachlich angenommen. Ein Dokumentationsmerge allein hätte keine Annahme ersetzt.
Technische Versionsgrammatik und Schriftprüfung bleiben vor Implementierung auszuarbeiten. Die
Freigabe ist keine Funktionsabnahme.

Unabhängig davon kann der native M3-Gesamtkorpus samt Neustart, Backup/Recovery, Snapshot-Neuimport,
Backlinks und sichtbaren Assetfehlern gemäß Matrix geplant und geprüft werden. Ein dort fehlender
Nachweis wird nicht durch einen neuen Formatvertrag ersetzt. Zweitrechner-/Clean-VM-Abnahme und
Bildgenerierung bleiben in ihren bestehenden späteren Stufen.
