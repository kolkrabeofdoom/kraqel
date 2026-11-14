# Flache DOCX Marker im Office Roundtrip

KQ-059 / Issue #115, Stand 14. November 2026. Der beauftragte Folgeversuch beseitigt den in KQ-058
beobachteten Strukturverlust bei LibreOffice. Isolierter Python-Prototyp, kein App-Importpfad. Die
Repräsentationsentscheidung steht in ADR 0026.

## Ergebnis

| Vorgang                                      | Absatzmarker | Inventareinträge | Vergleich                |
| -------------------------------------------- | ------------ | ---------------- | ------------------------ |
| Erzeugen und Rücklesen                       | 24           | 41               | identisch                |
| Word 16.0.20527.20112 speichert neues DOCX   | 24           | 41               | identisch                |
| LibreOffice 26.8.3.1 speichert neues DOCX    | 24           | 41               | identisch                |
| Word-Ausgabe erneut in LibreOffice speichern | 24           | 41               | identisch                |
| Word ersetzt `Original` durch `x`            | 24           | 41               | genau diese Textänderung |
| LibreOffice ersetzt `Original` durch `y`     | 24           | 41               | genau diese Textänderung |

Verglichen werden Titel, Reihenfolge, Kapitel-/Szenenzuordnung, leere Kapitel/Szenen/Absätze,
Überschriften 1–6, Unicode, Leerzeichen, Fett/Kursiv einschließlich Kombinationen, harte Umbrüche,
mehrabsätzige Listenelemente und verschachtelte Listen/Zitate.

## Experimentelle Grammatik

`flat_probe.py` verwendet den Korpus von `probe.py`. Die Absätze liegen flach unter `w:body`. Jeder
Absatz hat seine Absatz-/Nummerierungseigenschaften und genau ein Inline-`w:sdt`, das seine
sichtbaren Textläufe und Umbrüche enthält. Es gibt weder verschachtelte Container-Controls noch
einen Custom-XML-Inventarteil.

Ein Marker folgt `kqf2:<rolle>:<kennung>:<digest128>`, beispielsweise mit `chapter-title`,
`paragraph` oder `heading6` als Rolle und `n0004` als Kennung. Die Namen sind experimentell.
Äquivalente absatzweise Block-Controls kann der Verifier normalisieren; beide Formen müssen genau
einen markierten Absatz beschreiben. Unmarkierter Absatzinhalt außerhalb des Controls ist verboten.

`docProps/custom.xml` enthält Eigenschaften vom Typ `vt:lpwstr`:

- `KRAQEL.Roundtrip.Prototype2`: kanonisches JSON mit `version: 2`, `chunks` und vollständigem
  hexadezimalem `sha256`.
- `KRAQEL.Roundtrip.Prototype2.0001` bis zur angegebenen Teileanzahl: lückenlose Stücke des
  kanonischen Inventar-JSON mit höchstens 200 ASCII-Zeichen je Stück.

Das zusammengesetzte JSON enthält nur `version: 2` und `nodes`. Jeder Knoten hat genau die
Zeichenfolgenfelder `id`, `role`, `parent` und `position`. Die Wurzel ist ein `manuscript` mit
leerem Elternfeld und Position `0`; Positionen sind kanonische Dezimalzeichenfolgen. Eltern stehen
vor Kindern; das Array entspricht der Tiefensuche in Dokumentreihenfolge. Die Rolle unterscheidet
Textblätter von Containern. Leere Kapitel/Szenen behalten ihren eigenen, eventuell leeren Titel.

JSON wird mit ASCII-Escaping, ohne zusätzliche Leerzeichen und mit festgelegter Feldreihenfolge
geschrieben. Doppelte Felder oder unbekannte Schlüssel sind verboten. Experimentgrenzen: 4096
Knoten, 2048 Teile, 32 logische Ebenen. Vorhandene ZIP-/XML-Grenzen gelten zusätzlich; dies ist noch
kein vollständiger Produktparser. Der Korpus braucht 14 Nutzdatenstücke und eine Kopf-Eigenschaft.

SHA-256 bindet die Inventarstücke zusammen; sein 128-Bit-Präfix steht in jedem Absatzmarker. Ein
fremdes Inventar mit neuem Kopf-Hash genügt deshalb nicht: Es muss zu den tatsächlichen Markern
passen. Wer Inventar und alle Marker gemeinsam verändert, kann eine andere konsistente Datei
erzeugen. Es gibt keine Authentizitätssignatur. Auch konsistente Metadaten müssen alle Typ-,
Struktur-, Tiefen- und Semantikprüfungen bestehen.

Der Verifier rekonstruiert nur Container aus den validierten Beziehungen. Text, Markierungen,
Absatzrollen und Nummerierung prüft danach derselbe unabhängige Reader wie beim ersten Versuch. Es
wird keine versteckte Manuskriptkopie zurückgespielt.

## Reproduzieren

Python 3.10 oder neuer, Standardbibliothek; keine neue App-Laufzeitabhängigkeit:

```powershell
python scripts/docx-prototype/flat_probe.py generate <Testverzeichnis>/flat.docx
python scripts/docx-prototype/flat_probe.py verify <Testverzeichnis>/flat.docx
python -m unittest discover -s scripts/docx-prototype -v
```

Word: F12, unter neuem Namen als DOCX speichern und Formataktualisierung zulassen. LibreOffice: mit
eigenem Profil tatsächlich neu speichern, nicht lediglich unverändert schließen:

```powershell
soffice.com -env:UserInstallation=file:///ABSOLUTER/TESTPFAD/lo-profile --headless --convert-to "docx:Office Open XML Text" --outdir <anderer-Ordner> <flat.docx>
```

Die Textänderungen wurden in den nativen Oberflächen vorgenommen: `Original` im bestehenden Absatz
markieren, durch `x` bzw. `y` ersetzen und speichern; in LibreOffice DOCX ausdrücklich beibehalten.
Der Vergleich erhält insbesondere die zwei anschließenden Leerzeichen und sämtliche übrige Semantik.
Bearbeitung wurde nicht durch nachträgliche XML-Änderungen simuliert.

41 Tests sichern beide Darstellungen. Neue Negativfälle umfassen fehlende Teile, doppelte
Eigenschaften/JSON-Felder, geänderte Nutzdaten, fremdes Inventar mit neuem Kopf-Hash, fehlende,
doppelte und verschobene Marker, unbekannte Versionen, unmarkierten Text, ungültige Eltern, doppelte
Kennungen, falsche Positionen/Rollen und widersprüchliche Nummerierung. Mehrere Fälle aktualisieren
Hash und Marker bewusst mit: Selbstkonsistenz ersetzt keine gültige Struktur. Die bestehende CI
führt alle Tests unter Windows und Linux aus.

## Herkunft und Prüfsummen

Texte sind synthetisch und stehen unter der Projektlizenz. `flat/generated.docx` stammt vom Writer;
die fünf anderen Dateien wurden tatsächlich von den oben genannten Office-Builds geschrieben. Nur
Core-Autor-Metadaten, Core-Zeiten und ZIP-Zeiten wurden mit `retain_fixture.py` neutralisiert.
Insbesondere Dokument- und Inventarbytes bleiben bytegleich zu den Office-Ausgaben.

| Datei unter `fixtures/flat/` | SHA-256                                                            |
| ---------------------------- | ------------------------------------------------------------------ |
| `generated.docx`             | `a41d49e2d4b5b4ca41858e35e9b9569c1a015c3015de33ae1bca3a122b744553` |
| `word-resaved.docx`          | `8594a14ee7bfb3fdd1269174ce2c9e8225049840db631c7a6048fba3f639a26e` |
| `libreoffice-resaved.docx`   | `ad2414a20c3689d54e7f19e86f1fe303a31a9df0e84b20e38064e86c80920ac1` |
| `word-to-libreoffice.docx`   | `e4af105862a6182cd49ef9d6badd9488bfceb865f1a12fe791b1907afe94752d` |
| `word-edited.docx`           | `e37d92c15c59e00b58053f2fc268a90a3731de1709c01438c670c4a475a39929` |
| `libreoffice-edited.docx`    | `a8014ef626597ca44f1b1bbc8faf5a4ca8acb9a51451599709624959015128b0` |

Alle sechs Dokumente wurden gerendert und visuell kontrolliert. Lesbare Darstellung und semantischer
Vergleich sind getrennte Nachweise. Die Aussage gilt für diesen Korpus und diese Builds, nicht für
alle Office-Versionen, alle Strukturänderungen oder ODT-Umwege. Ein produktiver Rust-Reader muss
insbesondere Paketbeziehungen, Styles, unbekannte Inhalte und globale Ressourcenbudgets unabhängig
vollständig validieren.
