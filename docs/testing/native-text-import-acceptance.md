# KQ-054: Native Windows-Prüfung des Textimports

Stand: 14. November 2026

Geprüfter Code: `1ad9e380bbe7ccde7c5494e28c2b12ad1b430915`, Issue #105. Fachlicher Bezug:
Pflichtenheft Revision 1, F08, M3 und Abschnitt 10; ADR 0023.

## Umgebung und Methode

Windows x64, nativer Rust-Debug-Build mit Rust/Cargo 1.100.0, Visual Studio Build Tools 2022
17.14.45 und WebView2 157.0.4341.60. Die Tauri-WebView lud die lokale Vite-Entwicklungsoberfläche
ohne E2E-Command-Adapter. Bedienung und Sichtprüfung erfolgten mit Computer Use über den echten
Windows-Dateidialog und den Accessibility-Tree.

Ein eigenes Projekt `KQ-054 Native Prüfung 2026-11-14` enthielt ausschließlich künstliche Testdaten.
Die Dateien lagen außerhalb des Repositorys im Arbeitsverzeichnis der Session. Vorhandene
Schreibprojekte wurden nicht verändert. Das Prüfprojekt bleibt lokal erhalten.

## Ergebnisse

| Fall             | Eingabe und Beobachtung                                                                                                                                                          | Ergebnis  |
| ---------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------- |
| TXT              | `Prüfung.txt`, UTF-8 ohne BOM, 89 Bytes: Umlaute, zwei Absätze und ein harter Zeilenumbruch erscheinen korrekt; eine zusätzliche Szene mit Dateistamm als Titel entsteht.        | Bestanden |
| Markdown         | `Struktur.md`, 102 Bytes: Überschrift, Zitat, ungeordnete und geordnete Liste erscheinen strukturiert; `**sichtbare Syntax**` bleibt entsprechend ADR 0023 als Text erhalten.    | Bestanden |
| Abbruch          | Nativen Dialog abbrechen: keine zusätzliche Szene, unveränderter Text und keine neue Fehlermeldung; auch Escape beendet einen Dialog.                                            | Bestanden |
| Ungültiges UTF-8 | `ungueltig.txt`, zwei Bytes `C3 28`: bereinigte Fehlermeldung, keine neue Szene, bisheriger Inhalt bleibt erhalten.                                                              | Bestanden |
| Übergröße        | `zu-gross.txt`, 2.097.153 ASCII-Bytes, also 2 MiB plus ein Byte: dieselbe bereinigte Fehlermeldung, keine neue Szene.                                                            | Bestanden |
| Titel            | Importierte Szene `Prüfung` lässt sich über den bestehenden Ablauf in `Prüfung umbenannt` ändern.                                                                                | Bestanden |
| Wiederöffnen     | Zur Projektübersicht zurückkehren und das Prüfprojekt erneut öffnen: beide importierten Szenen und die Umbenennung bleiben vorhanden; der TXT-Inhalt wird unverändert angezeigt. | Bestanden |

Die Fehlermeldung lautet: „Die Textdatei konnte nicht importiert werden. Unterstützt werden
UTF-8-Dateien bis 2 MiB.“ Sie enthält weder lokalen Pfad noch Rohfehler.

## Aussagegrenzen

Dies ist eine UI-geführte native Funktionsprüfung, keine neue Screenreader-Hörprüfung und kein
automatisierter nativer Regressionstest. Codeblöcke, NUL-Bytes, Symlinks, zeitgleiche
Dateiänderungen und die exakt zulässige 2-MiB-Grenze wurden hier nicht zusätzlich manuell geprüft.
Das Wiederöffnen erfolgte innerhalb derselben App-Sitzung, nicht nach einem Prozessneustart. Ein
Releaseinstaller und eine Clean-VM wurden nicht geprüft. DOCX-Import und Roundtrip bleiben offen;
diese Prüfung erklärt F08 oder M3 nicht insgesamt für abgenommen.
