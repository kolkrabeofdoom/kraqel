# KQ-067: nativer M3-Gesamtlauf M3-G01

Stand: 15. November 2026 · Issue #131. Ausgangsbasis `58dea4be4e730b208d9002d6b6d30631d8d1cf15` (PR
#130 nach grüner Windows-/Linux-CI und Dependency Review zusammengeführt), ergänzt um den
KQ-067-QA-Helfer und die korrigierte Medienstatusmeldung. Bezug: Pflichtenheft Revision 1 §§3/4/6–11
und [F08-/M3-Matrix](F08_M3_ACCEPTANCE_MATRIX.md).

**Ergebnis:** Der funktionale native Datenlebenszyklus mit 100 Szenen und 50 echten Bildern ist im
folgenden Umfang bestanden. Das ist keine pauschale F08-/M3-, Offline-Netzverkehrs-, assistive oder
Release-Vollabnahme. Insbesondere wird keine Drehbuchfunktion implementiert oder abgenommen. Der
gesamte Drehbuchvertrag und ADR 0033 sind durch ausdrückliche Nutzerfreigabe am 15. November 2026
fachlich angenommen; die technischen Implementierungsstufen fehlen weiterhin.

## Umgebung und Isolation

Windows 11 Pro `10.0.26200`, Rust `1.100.0`. Echte gebündelte Tauri-Debug-App mit React-WebView,
nativen Datei-/Speicherdialogen, Rust und SQLite; kein Browser-Command-Mock. Zwei getrennte
Testkonfigurationen:

- `tauri.m3-test.conf.json`: Produkt **KRAQEL M3 Gesamttest**, ID `org.kraqel.desktop.m3-test`,
  Datenwurzel `$TEMP/kraqel-m3-acceptance-20261115`.
- `tauri.m3-recovery-test.conf.json`: Produkt **KRAQEL M3 Recovery Test**, ID
  `org.kraqel.desktop.m3-recovery-test`, Datenwurzel darunter in `recovery/`.

Reguläre App-Projekte werden nicht geöffnet. Alle Inhalte und Bildpixel sind synthetisch. Der
QA-Helfer wird ausschließlich mit `cfg(test)` kompiliert; die gebündelte Anwendung enthält weder
Seed-Funktion noch zusätzliche Commands. `image` wird im Desktop-Crate nur als Dev-Abhängigkeit in
der bereits gelockten Version 0.25.9 verwendet; keine neue Bibliotheksversion, Migration oder
Produktformatversion. SQLite bleibt 6, Editor/Command/DOCX-Austausch v1 und Snapshot v3 unverändert.

## Korpus und Inventar

| Gegenstand     | Geprüfter Umfang                                                                                                                   |
| -------------- | ---------------------------------------------------------------------------------------------------------------------------------- |
| Dokumente      | 3 Kapitel und 100 Szenen, zusammen 103; Überschrift, Unicode, Fett/Kursiv und harter Umbruch je Szene                              |
| Bilder         | 50 unterschiedliche dekodierbare Originale: 17 PNG, 17 JPEG, 16 WebP; jeweils Originalhash und Thumbnail                           |
| Assetlinks     | 100 Szenenlinks, ein Charakterlink und ein Ereignislink; insgesamt 102                                                             |
| Story-Bibel    | Figur Rhea, Ort Bibliothek, gerichtete Relation und zwei Szenen-Rückverweise                                                       |
| Struktur       | Plotkarten mit POV/Handlungsstrang, relative Chronologie und absolute Erzählzeitebene; Ereignis mit Szenen-/Entitätslink           |
| Visuelle Daten | Ein Profil mit Kernmerkmalen, Ausschluss und kanonischem Bild, ein Stilpreset und ein editierter Promptentwurf mit Quellreferenzen |

Das Sollinventar erfasst die sechs Snapshot-JSON-Datensätze plus vollständige Assetmetadaten und
Links. Der Vergleich normalisiert ausschließlich projektgebundene `projectId`-Felder; Dokument-,
Entitäts-, Asset-, Ereignis- und sonstige fachliche IDs werden nicht weg-normalisiert. Nach
Umbenennung sind nur `Szene 1` → `Szene 1 geändert` und `Rhea` → `Rhea geändert` als beabsichtigte
Titel-/Namensänderungen erlaubt. Text, Reihenfolge, Revisionen, IDs und Referenzen bleiben gleich.
Die zweite Sicherungskopie behält die Projektidentität; Snapshot-Neuimporte erhalten neue
Projektidentitäten. Das sind bewusst verschiedene Wiederherstellungswege.

## Beobachtete native Abläufe

| Prüfung                         | Ergebnis                                                                                                                                                                                                                        |
| ------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Ausgangsstand versionieren      | In der App Snapshot erzeugt und explizit committed: `c63c61f` („M3 Ausgangsstand: 100 Szenen und 50 Bilder“)                                                                                                                    |
| Umbenennen                      | Szene und Figur über die nativen App-Oberflächen geändert; beide Story-Bibel-Rückverweise und Relation weiterhin sichtbar                                                                                                       |
| Zweiten Stand versionieren      | Neuer Snapshot und Commit `c7e8c5f` („M3 Szene und Figur umbenannt; Referenzen erhalten“); echter Git-Verlauf unabhängig gelesen                                                                                                |
| Echter Neustart                 | Prozess vollständig geschlossen und neu gestartet; umbenannte Szene, Figur und Rückverweise erhalten; vollständiger Rust-Inventarvergleich bestanden                                                                            |
| Nativer Archivexport            | Aktueller Snapshot über den nativen Speicherdialog ausgegeben; 162 Archivdateien, fertiges Archiv 78989 Bytes                                                                                                                   |
| Zwei native Neuimporte          | Dieselbe Datei zweimal über native Öffnendialoge importiert; Original und beide neuen Kopien haben verschiedene Projekt-UUIDs und identisches fachliches Inventar                                                               |
| Früheren Stand wiederherstellen | Vor der Umbenennung gesichertes `before.kraqel` zusätzlich über den nativen Dialog importiert; ursprünglicher Szenentitel sichtbar und vollständiger Vergleich mit Ausgangsinventar bestanden; neuere Projekte bleiben bestehen |
| SQLite-/Asset-Recovery          | Tatsächlicher `ProjectManager::backup_project`-/`restore_backup`-Pfad über ausdrücklich aktivierten QA-Helfer, kein erfundener Backup-UI-Button; wiederhergestellte Daten separat in der Recovery-App geöffnet                  |
| Fachliche Recovery-Sicht        | Umbenannte Szene, Profil der umbenannten Figur, bernsteinfarbene Augen, Ausschluss, kanonisches Bild, Stil Tusche und beide Zeitebenen/Ereignislinks sichtbar erhalten                                                          |
| Medien der Importkopie          | Bildvorschauen und direkte Links zu zwei Szenen, umbenannter Figur und Ereignis in der nativen Medienansicht vorhanden                                                                                                          |

Die tatsächliche Speicherdialog-Ausgabe hieß `kraqel-projekt.kraqel` im Dokumente-Verzeichnis. Nach
unabhängiger Prüfung von Projekt-ID, 50 Assets und SHA-256 wurde sie als `native.kraqel` in die
isolierte QA-Wurzel kopiert. Nur die anschließend hashgleich verifizierte synthetische Dialogausgabe
wurde aus Dokumente entfernt. Die App hat keinen lokalen Zielpfad über IPC ausgegeben.
Archiv-SHA-256 dieses Laufs: `4369dc7dfa1cb7867e47615d2eea23addeab4beae3d50b00ebf2abda6da4031f`.

## Sicherung, Exporte und unabhängige Rückprüfung

Der Rust-QA-Helfer vergleicht Quelle und Recovery vollständig und prüft die bytegleiche
Sicherungsdatei nach Recovery. Der reguläre Integrationstest führt zusätzlich zwei Archivimporte,
Wiederherstellung des früheren Inventars und alle vier Bildausgaben durch. Er ersetzt nicht die
separat oben dokumentierten WebView-/Dialogaktionen.

Die vollständigen Bildausgaben werden aus dem realen Korpus über dieselben nativen
Rust-Command-Implementierungen erzeugt: **50 Originale / 100 ausdrücklich aufgebaute
Bildplatzierungen pro Format**. Die 100 Checkboxen wurden nicht einzeln in der UI betätigt; die
Auswahl-/IPC-Bedienung ist ergänzend in KQ-063/064/065 nativ geprüft. Diese Trennung ist Bestandteil
des Nachweises, keine Behauptung eines zusätzlichen großen GUI-Exports.

| Format   | Paketbytes | Unabhängiger Nachweis                                                                                                                                                                       |
| -------- | ---------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Markdown | 234570     | 50 PNG-SHA-256 und 100 Bildverweise                                                                                                                                                         |
| HTML     | 249016     | 50 PNG-SHA-256, 100 Bildvorkommen; vollständiges Paket an anderen Pfad kopiert und in Chromium mit Offline-Kontext geöffnet, alle 100 Bilder geladen, keine externen Requests               |
| DOCX     | 220896     | ZIP enthält 50 hashgeprüfte interne PNGs und 100 Inline-Bildvorkommen                                                                                                                       |
| PDF      | 8367119    | 34 Seiten; 50 weiße PNG-Nachweise, 100 tatsächliche RGB-XObjects einschließlich Maße/8-Bit-RGB/Manifestkennung geprüft; vier eingebettete Schriften mit ToUnicode, Unicode-Text rückgelesen |

Python `sqlite3`, Pillow und pypdf dienten ausschließlich der unabhängigen Entwicklungsprüfung. Fünf
intakte Projektstände (Original, zwei aktuelle Neuimporte, historischer Neuimport und Recovery)
bestehen jeweils SQLite-Integritäts-/Fremdschlüsselprüfung und enthalten 103 Dokumente, 50
vollständig dekodierte Originale mit passenden SHA-256, 50 dekodierte Thumbnails und 102 Links. Die
bewusst beschädigten Kopien sind von dieser positiven Aussage ausgeschlossen.

Die 34 Seiten des Großkorpus sind **nicht erneut vollständig visuell abgenommen**. Die visuelle
Layoutaussage bleibt auf die getrennten, vollständig gerenderten KQ-064-/KQ-065-Korpora in
[DOCX-Abnahme](native-illustrated-docx-acceptance.md) und
[PDF-Abnahme](native-illustrated-pdf-acceptance.md) begrenzt. RGB-/Paket-/Textintegrität des
Großkorpus wird dadurch nicht mit einer universellen Layout- oder Office-Zusage verwechselt.

## Negativfälle und gefundene Korrektur

Eine eigene Fehlerkopie enthält drei verschiedene Manipulationen: Original 1 entfernt, Thumbnail 2
entfernt, Original 3 byteweise verändert. Quelle, Archive und vorherige Exportpakete werden nicht
manipuliert. Die Recovery-App erhält zusätzlich eine separate Kopie dieses Fehlerprojekts, um den
korrigierten Build zu prüfen.

| Fall                  | Ergebnis                                                                                                                                                                                   |
| --------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Verfügbarkeit         | Native Rust-Abfrage weist genau die drei Assets als nicht verfügbar aus; übrige Quellen bleiben intakt                                                                                     |
| UI-Meldung            | Ursprünglich irreführend „Vorschaudatei fehlt“, auch bei Originalfehlern; korrigiert zu „Bilddatei oder Vorschau nicht verfügbar“ und für alle drei Fälle im neuen nativen Build bestätigt |
| Backup/Export         | Fehlerhafte Originale verhindern Backup und ausgewählten Bildexport; ein vorheriges gültiges HTML-Paket bleibt vollständig bytegleich, keine Teilgeneration                                |
| Nativer Dialogabbruch | Escape im Archivdialog: neutral, Aktionen erneut bedienbar, weiterhin genau zwei Recovery-Testprojekte                                                                                     |
| Ungültiges Archiv     | Native Auswahl von `invalid.kraqel`: bereinigte sichtbare Ablehnung ohne Projektanlage                                                                                                     |
| Übergröße             | Native Auswahl einer synthetischen Datei mit 128 MiB + 1 Byte: Ablehnung mit dokumentiertem Limit, keine Projektanlage                                                                     |
| Ungültiges DOCX       | Nativer Startseitenimport von `invalid.docx`: bereinigter v1-/8-MiB-Hinweis, keine neue Projektkopie                                                                                       |

Die drei letzten Dateifehler sind zusätzlich über native Rust-Dateiimplementierungen geprüft. Nach
allen negativen GUI-Aktionen enthält die Recovery-Wurzel unverändert zwei Projekte; die
Haupt-Testwurzel enthält vier intakte Projekte und genau eine absichtlich beschädigte Kopie.

## Abschlussprüfungen

`pnpm check` bestanden: 162 Rust- und 53 TypeScript-Tests, sieben explizite QA-Helfer regulär
ignoriert. Alle 26 Browser-E2E bestanden. Beide isolierten nativen Debug-Builds und der
abschließende reguläre Windows-Debug-Build ohne Testkonfiguration bestehen. Formatierung, relative
Dateiverweise, alle 34 Matrixkennungen und `git diff --check` sind geprüft.

## Reproduktion und Aussagegrenzen

Regulärer, plattformübergreifender Integrationstest:

```powershell
cargo test -p kraqel-desktop integrated_m3_corpus_preserves_backlinks_through_backup_and_two_imports
```

Native QA benötigt eine frische Wurzel. Ein erneutes `seed` auf vorhandenem Inventar wird
abgewiesen. Die testexklusiven Schritte sind keine Produkt-Commands:

```powershell
$env:KRAQEL_M3_QA_ROOT = Join-Path $env:TEMP 'kraqel-m3-acceptance-20261115'
$env:KRAQEL_M3_QA_STEP = 'seed'
cargo test -p kraqel-desktop run_native_m3_qa_step -- --ignored --nocapture
pnpm --filter @kraqel/desktop tauri build --debug --no-bundle --config src-tauri/tauri.m3-test.conf.json
```

Nach den passenden manuellen Aktionen jeweils einzeln dieselbe Testfunktion mit `verify-renamed`,
`backup`, `verify-imports`, `verify-history`, `exports` oder `negative` aufrufen. `verify-imports`
verlangt genau Original plus zwei tatsächlich nativ angelegte Kopien und das Archiv `native.kraqel`;
`verify-history` anschließend genau eine zusätzliche historische Kopie. `negative` erzeugt erst
danach die Fehlerkopie. Eine andere frische Wurzel verlangt entsprechende Anpassung beider
isolierter Tauri-Konfigurationen. Der Helfer akzeptiert ausschließlich direkte Unterverzeichnisse
von `%TEMP%` mit Präfix `kraqel-m3-acceptance-`.

Der Lauf benötigt keinen eingerichteten Provider. Der kopierte HTML-Export ist unabhängig mit einem
Offline-Browserkontext geprüft. Ein unabhängiger vollständiger Netzverkehrsmitschnitt der
Desktop-App, eine erneute Screenreader-Hörprüfung sowie eine vollständige Tastatur-/Medien-CRUD-
Abnahme fanden **nicht** statt. Native Dialogtastatur, sichtbare Meldungen und Accessibility-Texte
sind Teil dieses Laufs, aber kein Ersatz für diese offenen Prüfungen. Die Matrix behält die
entsprechenden Zeilen ausdrücklich teilweise offen. Zweitrechner-/Clean-VM-Releaseabnahme und
Abhängigkeitssicherheit bleiben separat; keine neue Sicherheitsprüfung oder Alert-Schließung.
