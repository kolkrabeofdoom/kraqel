# DOCX Strukturprototyp

KQ-058 / Issue #113. Isoliertes Experiment zum Vertragsentwurf aus KQ-057, kein App-Import und keine
freigegebene Austauschversion. Python 3.10 oder neuer, ausschließlich Standardbibliothek. Python ist
nur für diesen Versuch erforderlich; die App bleibt unverändert.

Der inzwischen beauftragte und geprüfte [Folgeversuch mit flachen Markern](FLAT_MARKERS.md) (KQ-059)
erhält die Struktur auch in LibreOffice. Dieser Bericht und seine Fixtures bleiben als historischer
Vergleich und Regressionstest für die verschachtelte Darstellung erhalten.

## Reproduzieren

Aus dem Repositoryverzeichnis, mit einem verfügbaren Python-Interpreter:

```powershell
python scripts/docx-prototype/probe.py generate <Testverzeichnis>/generated.docx
python scripts/docx-prototype/probe.py verify <Testverzeichnis>/generated.docx
python -m unittest discover -s scripts/docx-prototype -v
```

`generate` schreibt ausschließlich die angegebene Testdatei und prüft den semantischen Selbstimport.
`verify` vergleicht standardmäßig gegen das synthetische Testmanuskript; `--expected <JSON-Datei>`
erlaubt ein anderes erwartetes kanonisches Modell. Ein Fehler liefert einen von null verschiedenen
Exitcode. Keine SQLite-Verbindung, Projektveröffentlichung, Tauri-Anbindung oder Netzwerkaufrufe.

41 verschachtelte Rich-Text-Content-Controls kennzeichnen Rollen und austauschlokale Kennungen
(`kq1:<rolle>:n0001`). Das separate `customXml/item1.xml` enthält das versionierte Strukturinventar
mit Kennung, Rolle, Elternkennung und Geschwisterposition. Es enthält keinen Manuskripttext.
Sichtbarer Text kommt ausschließlich aus den markierten Absätzen in `word/document.xml`. Die
`kq1`-Grammatik ist experimentell und reserviert keine produktive Formatversion.

Die 18 Tests vergleichen kanonische Semantik und echte Office-Fixtures und prüfen u. a. das
Entfernen einer ganzen Szene, doppelte Kennungen, geänderte Geschwisterreihenfolge, fehlende Marker,
unbekannte Versionen, beschädigtes Inventar, unmarkierte Absätze, widersprüchliche Nummerierung,
falsche Überschriftenebenen, einen unzulässigen Listenstart, XML-Grenzen und doppelte ZIP-Namen. Die
CI führt diese Tests zusätzlich zu den bestehenden Prüfungen unter Windows und Linux aus.

## Ergebnis des Versuchs am 14 November 2026

| Speichervorgang                                         | Strukturmarker | Inventar | Semantischer Vergleich            |
| ------------------------------------------------------- | -------------- | -------- | --------------------------------- |
| Prototyp schreibt und liest                             | 41             | erhalten | identisch                         |
| Word 16.0.20527.20112 speichert als aktuelles DOCX      | 41             | erhalten | identisch                         |
| Word ändert `Original` in `x` und speichert             | 41             | erhalten | ausschließlich diese Textänderung |
| LibreOffice 26.8.3.1 speichert als Office Open XML Text | 24             | entfernt | abgelehnt                         |

Word wurde über die native Oberfläche geprüft: lokale Kopie öffnen, F12, unter neuem Namen als
Word-Dokument speichern, die angebotene Aktualisierung des Dateiformats bestätigen. Erst dieses neue
Paket ist die Speichern-Fixture; ein unverändertes Strg+S allein war kein Nachweis, weil Word dabei
nicht neu schreiben musste. Danach wurde das Wort `Original` im vorhandenen Absatz durch `x` ersetzt
und Strg+S ausgeführt. Der strikte Vergleich erhält auch die zwei anschließenden Leerzeichen,
Unicode, Fett/Kursiv und harte Umbrüche.

LibreOffice wurde mit eigenem temporärem Benutzerprofil ohne Oberfläche ausgeführt:

```powershell
soffice.com -env:UserInstallation=file:///ABSOLUTER/TESTPFAD/lo-profile --headless --convert-to "docx:Office Open XML Text" --outdir <anderer-Ordner> <generated.docx>
```

LibreOffice behält die 24 Blatt-Controls, entfernt aber die 17 äußeren Struktur-Controls sowie das
Inventar. Ein optisch ähnliches Dokument ist deshalb kein strukturerhaltender Roundtrip. Der
Verifier rekonstruiert nichts aus Titeltexten und weist die Datei zurück.

Die OPC-Beziehungen werden mit Default-Namespace geschrieben; mit präfixiertem Relationships-Root
scheiterte der lokale LibreOffice-Ladeversuch. Die Nummerierungsdefinitionen stehen entsprechend der
OOXML-Reihenfolge vor den Nummerierungsinstanzen. Das erzeugte Manuskript und beide Word-Ergebnisse
wurden zusätzlich in Seitenbilder gerendert und visuell kontrolliert.

## Grenzen und nächste Entscheidung

Der Verifier prüft gezielt Strukturinventar, sichtbare Textläufe, Absatzrollen und verwendete
Nummerierungsart/Startwert. Begrenztes ZIP-/XML-Lesen dient dem Experiment; dies ist **kein
vollständiger Sicherheitsparser**. Insbesondere eine vollständige Paket-/Beziehungs-Allowlist,
Style-Vererbung, globale Parserbudgets und die Ablehnung aller unbekannten Formatierungen gehören
vor eine produktive Rust-Implementierung. Nicht an Dateien beliebiger Herkunft als Importfunktion
bereitstellen. Die bestehenden App-Grenzen aus ADR 0024 wurden nicht gelockert.

Die verschachtelte Repräsentation ist noch nicht angenommen. Vor einer gemeinsamen Office-Garantie
ist eine alternative Darstellung zu prüfen, etwa flache Blattmarker mit expliziten, begrenzt
validierten Strukturbeziehungen in einem von beiden Anwendungen erhaltenen Inventar. Dabei müssen
auch gelöschte leere Kapitel und Szenen erkennbar bleiben. Metadaten bleiben untrusted und ersetzen
niemals den sichtbaren Text. Die Aufnahme als produktiver Vertrag erfordert einen neuen
Repräsentationsnachweis und eine angenommene ADR; siehe ADR 0025.

## Herkunft der Fixtures

Alle Texte sind synthetisch und gehören zur Projektlizenz. Keine privaten Manuskripte oder
Referenzdokumente. `generated.docx` wird deterministisch durch `probe.py` erzeugt. Die anderen
Dateien sind tatsächlich von den oben genannten Office-Versionen geschrieben; nur Core-Metadaten mit
Autor*in und Zeitstempeln sowie ZIP-Zeitstempel wurden mit `retain_fixture.py` neutralisiert.
Dokument-, Style-, Nummerierungs- und Inventarbytes bleiben dabei unverändert. Die Fixtures werden
auch auf CI ohne Office-Installation geprüft; dies ersetzt keinen neuen Test anderer Office-Builds.

| Datei                      | SHA-256                                                            |
| -------------------------- | ------------------------------------------------------------------ |
| `generated.docx`           | `15bd07bbf0641e977aa211cd3c05876807bb8526010e41ba7ae72d998acca89a` |
| `word-resaved.docx`        | `c683bcfdd05f522f259c8c64c8002cf8fa26c098dd965e1e73e8bf0c2bf93491` |
| `word-edited.docx`         | `4c1b72944522c11d99c0111da8c58c6752fddf7d9a4670dcd5b0e98329a667cf` |
| `libreoffice-resaved.docx` | `d5aaca1ec20e02f2444ea8c90317eed26f3bc17369f55a7e1e3165d31b741196` |
