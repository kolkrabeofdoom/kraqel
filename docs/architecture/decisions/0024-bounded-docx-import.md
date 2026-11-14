# ADR 0024: Begrenzter DOCX-Import als neue Szene

Status: angenommen am 14. November 2026; KQ-055, Issue #107.

## Entscheidung

Der ausdrücklich freigegebene erste DOCX-Scope setzt F08/M3 des Pflichtenhefts Revision 1 teilweise
um. Ein eigener nativer Dialog importiert eine DOCX-Datei als neue Szene in ein gewähltes Kapitel.
Absätze, Unicode, Fett/Kursiv, harte Umbrüche und eindeutig auflösbare Überschriften 1 bis 6 werden
übernommen. Layout und Schriftbild werden nicht übernommen; die Oberfläche erklärt dies vor der
Auswahl. Inhaltstragende, nicht unterstützte Konstrukte werden abgelehnt, insbesondere Tabellen,
Bilder, Fußnoten, Hyperlinks, Felder, Änderungen und Listen. Der Import errät keine Kapitelgrenzen
und überschreibt keine Szene.

Archivgrenzen: 8 MiB Eingabe, 32 MiB entpackt insgesamt, 8 MiB pro Teil, 128 ZIP-Einträge, höchstens
100:1 Kompressionsverhältnis pro Teil. XML ist UTF-8, maximal 64 Ebenen und 100.000 Elemente
insgesamt; maximal 2 MiB übernommener Text. Zyklische oder mehr als 32 Stufen tiefe Style-Vererbung
wird abgelehnt. Pfadtraversal, doppelte Namen, Symlinks, Verschlüsselung, unbekannte
Paketbestandteile, externe Beziehungen, DTDs und unbekannte Entities sind unzulässig. Es werden
keine Archivdateien extrahiert und keine externen Ressourcen geladen.

Bekannte Hilfsteile wie unbenutzte Nummerierungsdefinitionen, Bibliografie-Custom-XML, Styles,
Einstellungen, Theme und Paketvorschau sind begrenzt erlaubt. Sie werden nicht als Manuskriptinhalt
übernommen. Verwendete Listen und Content Controls im Hauptdokument bleiben unzulässig. Der
[Importvertrag](../../data-format/DOCX_IMPORT.md) beschreibt diese Grenze.

`zip` 9 mit deaktivierten Default-Features und Deflate entspricht ADR 0020. `quick-xml` 0.42 liefert
namespacebewusstes XML-Lesen ohne externen Resolver; es ist bereits transitiv im gelockten Workspace
vorhanden, MIT-lizenziert und wird hier direkt ohne Zusatzfeatures genutzt. Beide Bibliotheken
ersetzen nicht die eigenen Größen-, Struktur- und Semantikprüfungen.

Der additive Command `import_docx_document` nutzt die bestehende Vertragshülle v1 und gibt
`DocumentNode | null` zurück. Pfad und Bytes bleiben vollständig in Rust. Die bestehende
Storage-Transaktion erstellt Szene, Inhalt und Klartextindex gemeinsam, erst nach vollständiger
Validierung. Abbruch bleibt neutral, Fehler enthalten weder Pfade noch Parserdiagnostik.

## Folgen

Strukturerhaltender KRAQEL-Roundtrip benötigt einen eigenen versionierten Austauschvertrag und eine
spätere Writer-Erweiterung. Der bisherige Writer bleibt unverändert. Insbesondere seine Textpräfixe
für Listen/Zitate und seine zusammengefassten Überschriften erlauben keine allgemeine verlustfreie
Rekonstruktion. Medien und vollständige Projektportabilität bleiben beim Snapshot. F08, M3 und
Releaseabnahme sind durch diesen begrenzten Import nicht vollständig abgeschlossen.
