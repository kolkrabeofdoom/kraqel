# ADR 0032 PDF mit eingebetteter Schrift und Szenenbildern

Status: angenommen am 15. November 2026; KQ-065 / Issue #127. Grundlage: freigegebener
KQ-062-Vertrag, ADRs 0030/0031 und Pflichtenheft Revision 1 F08/F09, M3, Abschnitte 10/11. PR #126
ist nach grüner Windows-/Linux-CI und Dependency Review zusammengeführt (c8708b8).

## Entscheidung

Ein gemeinsamer nativer PDF-Renderer ersetzt die bisherige Helvetica-Ausgabe auch beim reinen
Textexport. Vier unveränderte statische Noto-Sans-2.008-Dateien aus einem festgeschriebenen
Upstream-Commit werden mit der App geliefert und vollständig eingebettet. Lizenz, Copyright,
Versionen und SHA-256 stehen unter `crates/export/fonts/`; Schriftinstallation oder Netzwerk sind
nicht erforderlich. Die SIL Open Font License 1.1 liegt vollständig bei.

`ttf-parser` 0.19.2 verwendet die bereits gelockte Parser-Version für Glyphenprüfung und echte
Vorschub-/Oberlängenmaße. Als einzige neue Paketversion kommt `unicode-normalization` 0.1.25 (MIT
oder Apache-2.0) für NFC hinzu; dessen MIT-Lizenz liegt bei. `image` 0.25.9 wird zusätzlich direkt
vom Export-Crate verwendet, ohne zweite Bildbibliothek oder neue Version. Seit dem 23. November 2026 erzeugt
`printpdf 0.12.14` die A4-Seiten aus den vorab berechneten Text- und Bildpositionen.
Die Ausgabeprüfung liest dieselben Bytes mit `lopdf 0.47.0`. Der Schrifttest verwendet
alle vier Schnitte, prüft eingebettete Fonts und Unicode-Zuordnungen und liest den
griechischen, kyrillischen und deutschen Testtext aus der fertigen Datei zurück.

Der vollständige Auftrag wird vor der PDF-Erzeugung geplant und auf Glyphen, Schrift-/Layoutprofil
und Budgets geprüft. Lateinisch, Griechisch und Kyrillisch sind im dokumentierten Umfang abgenommen.
Fehlende Glyphen, nicht unterstützte Schriftläufe, verbleibende kombinierende Zeichen,
Bidirektionalitäts-/Joiner-Steuerzeichen und komplexes Shaping werden abgewiesen. Kein stiller
Fontfallback oder Ersatzzeichen. NFC betrifft die Ausgabe, nicht SQLite oder das Editor-Dokument.

Der Modus „Mit Szenenbildern“ erhält additiv `pdf` im bestehenden Command v1. Bildquellen,
Revisionen, direkter Szenenbezug und atomare neue Generationen bleiben gemeinsam validiert.
PNG-Ableitungen werden mit ganzzahlig festgelegter Rundung auf Weiß abgeflacht; die PDF enthält
deren identische RGB-Pixel verlustfrei. Dieselben PNGs werden als prüfbare Begleitdateien
mitgeführt. Der Rückleser vergleicht Pixelhash **und Abmessungen**, RGB/8-Bit-Profil sowie fehlende
Transparenz. Das Manifestfeld `pdfImageId` wird mit `/KRAQELImageId` am tatsächlichen Bild-XObject
verbunden.

A4, metrischer Zeilenumbruch, sichtbare Bildbeschreibung und optionale Beschriftung ersetzen die
bisherigen groben Zeichenbreiten. Bild und beide Textarten werden als vollständiger Seitenabschnitt
geplant, proportional verkleinert, nie hochskaliert oder beschnitten. Nicht auf eine Seite passende
Textabschnitte führen zum Fehler. Zusätzlich gelten 8 MiB PDF-Text, 100000 Zeilen, 2048 Seiten und
128 MiB unkomprimierte RGB-Bytes über sämtliche Bildvorkommen; die bisherigen Paketgrenzen bleiben.

## Nachweis und Folgen

Rust prüft Schriftberechtigungen und alle vier Schnitte, Unicode/NFC, Abweisung nicht darstellbarer
Texte, metrischen Umbruch, Weißkomposition, 100 Szenen/50 Bilder, wiederholbare Manifest-/PNG-Bytes,
RGB-Selbstvergleich und Konfliktabbruch ohne Teilpakete. Ein Randfall mit gleichen RGB-Bytes bei
unterschiedlichen Maßen schützt die Bildzuordnung. Browser-E2E und native Windows-App ergänzen den
Nachweis; alle Seiten des synthetischen PDF-Korpus werden gerendert und visuell geprüft.

PDF-Container können Zeitstempel und zufällige interne Kennungen enthalten; behauptet werden
semantische Gleichheit und stabile Manifest-/Ableitungshashes, keine PDF-Bytegleichheit. Keine
PDF/UA-, PDF/A-, PDF/X- oder universelle Unicode-/Druckfarb-Zusage. Ein Bild-Roundtrip und
Drehbuchformatierung bleiben getrennte Entscheidungen. Die Implementierung erfüllt F08/M3 nur im
dokumentierten Teilumfang; assistive Vollabnahme und weitere Systemabnahmen bleiben offen.
