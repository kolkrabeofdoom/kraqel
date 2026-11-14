# Medienexport v1 Vertragsentwurf für Szenenbilder

Status: vollständig fachlich freigegeben am 14. November 2026; KQ-062 / Issue #121, PR #122 nach
grüner Windows-/Linux-CI und Dependency Review zusammengeführt. Bilder erscheinen am Szenenende.
KQ-063 implementiert Rust-Kern und Markdown/HTML nach diesem Vertrag; der verbindliche Teilvertrag
steht unter [`ILLUSTRATED_MANUSCRIPT.md`](../data-format/ILLUSTRATED_MANUSCRIPT.md). Gewöhnliches
DOCX/PDF mit Bildern bleiben eigene Stufen. Der DOCX-Text-Roundtrip bleibt unverändert. Der
historische Dateiname bleibt zur Referenzierung erhalten.

## Anforderungen und belegter Ausgangsstand

Pflichtenheft Revision 1 vom 6. November 2026 verlangt mit F08 definierte Markdown-/HTML-/DOCX-/PDF-
Formatierungen und mit F09 lokale Medien und Verknüpfungen. M3 sowie Abschnitt 10 verlangen
nachvollziehbare Mediendateien im dokumentierten Austauschformat; Abschnitt 11 lässt die Einbettung
je Format ausdrücklich offen. Die hier vorgeschlagene Auswahl schließt einen Teil dieser Lücke.
Drehbuchformatierung bleibt eine getrennte F08-Entscheidung; Bildgenerierung bleibt Version 0.2.

GitHub-Basis ist `main` mit PR #120 / Merge `f7c34980c02bbb2525d381efdf8abe8cfcca9522`.
Windows-/Linux-CI und Dependency Review sind grün, Issue #119 ist geschlossen. KQ-060/061 erhält
Manuskripttext und Struktur, schließt Medien im DOCX-Wiederimport jedoch ausdrücklich aus.

Im Code vorhanden sind PNG-/JPEG-/WebP-Originale, SHA-256, Abmessungen, Herkunft `imported`, Titel,
Tags und Verknüpfungen zu Dokumenten, Entitäten und Ereignissen. Importgrenzen sind 25 MiB, 16384
Pixel je Kante und 40 Millionen Pixel je Bild. Die Medienbibliothek verwendet eine
Titel-/Kennungssortierung. Ihre Verknüpfungen besitzen keine Bildposition im Manuskript.
Alternativtext, Bildunterschrift, Lizenz- und Urheberfelder sowie Exportauswahl sind nicht
gespeichert. Die vier gewöhnlichen Manuskriptrenderer verarbeiten nur Textblöcke. Snapshot v3
enthält bereits geeignete Projektassets samt Metadaten, ist aber ein anderes Format und kein
bebildertes Manuskript.

Belegstellen: `crates/asset-service/src/lib.rs`, `crates/domain/src/lib.rs`,
`crates/storage/src/media_repository.rs`, `crates/export/src/lib.rs`, `crates/export/src/render.rs`,
ADR 0012 und `docs/data-format/snapshot-v3.schema.json`.

## Bedienung und Platzierung

Die Exportansicht erhält einen eigenen Modus **Mit Szenenbildern**. Die vorhandenen Textausgaben und
**DOCX zum Wiederimport** behalten ihre Bedeutung. Der neue Modus beginnt ohne ausgewählte Bilder;
eine Verknüpfung ist keine automatische Exportfreigabe. Ohne Auswahl wird auf die Textausgabe
verwiesen.

Zur Auswahl stehen ausschließlich Bilder mit mindestens einer direkten Verknüpfung zur jeweiligen
Szene desselben Projekts. Kapitel-, Entitäts-, Ereignis- und kanonische Profilbilder werden nicht
indirekt eingesammelt. Wer ein solches Bild ausgeben möchte, verknüpft es zunächst ausdrücklich mit
einer Szene. Mehrere Linkrollen erzeugen keine mehrfachen Bildvorkommen.

Die Auswahl zeigt Szene, Bildtitel und lokale Vorschau. Pro ausgewähltem Bildvorkommen werden
Reihenfolge, Alternativtext und optionale Bildunterschrift festgelegt. Alle Bildtexte sind Klartext,
kein ausführbares HTML oder Markdown. Steuerzeichen werden abgelehnt; nur Beschriftungen dürfen
explizite Zeilenumbrüche enthalten. Alternativtext wird zunächst aus dem Bildtitel vorgeschlagen,
sichtbar bearbeitbar angeboten und muss nach äußerem Whitespace-Trim 1–500 Zeichen enthalten. Eine
Bildunterschrift hat höchstens 1200 Zeichen. Keine automatische KI-Beschreibung. Diese Angaben
gelten für den Exportauftrag; sie verändern weder Medienmetadaten noch Editorinhalt und werden in v1
nicht als Exportvorlage gespeichert.

Nach den vollständigen Textblöcken jeder Szene folgen ausschließlich deren ausgewählte Bilder. Die
Szenenreihenfolge entspricht dem Manuskript, innerhalb einer Szene gilt die ausdrücklich angezeigte
Reihenfolge. Ein Bild erscheint je Szene höchstens einmal; derselbe Inhalt darf in mehreren Szenen
erscheinen. Unbekannte, doppelte oder widersprüchliche Zuordnungen sind Fehler. Keine freien Anker
zwischen Textabsätzen, Titelbilder oder gemeinsamen Bildanhänge in dieser Stufe.

Die vorhandenen Textinhalte bleiben unverändert. Auch leere Szenen können ausgewählte Bilder haben.
Für eine leere Szenenüberschrift wird nur in der Exportauswahl eine Ersatzanzeige verwendet. Die
bebilderte Ausgabe ist eine Weitergabeversion; es gibt dafür keine Wiederimportgarantie.

## Ausgabe je Format

Jeder Auftrag erzeugt genau ein vollständiges Paket in
`exports/mit-bildern/<markdown|html|docx|pdf>-<neue-uuid>/`. Alle Pakete enthalten eine
UTF-8/LF-Datei `medien.json` mit dem nachfolgenden Herkunfts- und Zuordnungsnachweis. Der Ordner
wird als neue Einheit veröffentlicht; keine losen neuen Begleitdateien neben einem alten Manuskript.
Ältere Pakete bleiben erhalten; jeder Auftrag benötigt erneut Speicherplatz. Automatische
Bereinigung ist nicht Teil von v1.

| Format            | Manuskript und Bilder                                                                                                                                            | Zugänglichkeit und Weitergabe                                                                                                                         |
| ----------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------- |
| Markdown          | `manuskript.md`, relative Bilder unter `assets/<export-sha256>.png`; Markdown-Bildsyntax, optionale Beschriftung als eigener Absatz                              | Alternativtext in der Bildsyntax; Ordner vollständig weitergeben, kein `file://` und keine absoluten Pfade                                            |
| HTML              | `manuskript.html` mit lokalen `assets/<export-sha256>.png`; semantische `figure`, `img`, optionale `figcaption`                                                  | `alt`, natürliche Lesereihenfolge, responsive Breite; kein Script, keine entfernten Fonts/Bilder, kein notwendiges Netzwerk                           |
| Gewöhnliches DOCX | `manuskript.docx`; Bilder ausschließlich intern in `word/media/<export-sha256>.png`, bekannte interne Bildbeziehungen und Inline-Zeichnungen in eigenen Absätzen | Alternativtext in Bildbeschreibung, optionale sichtbare Beschriftung; Word-/LibreOffice-Prüfung erforderlich                                          |
| PDF               | `manuskript.pdf`; Bilder intern, feste A4-Seiten und eigener Bildabschnitt am Szenenende                                                                         | Bild und Beschriftung zusammenhalten, ausreichend Platz oder neue Seite; Alternativtext zusätzlich sichtbar als Bildbeschreibung und in `medien.json` |

DOCX/PDF benötigen zur Bildanzeige keinen zusätzlichen Assetordner. Für PDF wird die verwendete
weiße PNG-Ableitung zusätzlich unter `assets/<export-sha256>.png` im Paket mitgeführt, damit ihr
Bytehash unabhängig prüfbar bleibt. Der PDF-Writer bettet dieselben RGB-Pixel verlustfrei ein;
Abnahme und Paketprüfung müssen diese Zuordnung kontrollieren. Dieser zusätzliche Nachweisordner ist
für die Anzeige der PDF-Datei nicht nötig. `medien.json` begleitet sie für den maschinenlesbaren
Nachweis; die Bilddarstellung funktioniert auch bei Weitergabe nur der Datei. HTML ist hier bewusst
ein Ordnerpaket; das bisherige eigenständige reine Text-HTML bleibt getrennt. Markdown-/HTML-Pfade
und Textwerte werden formatgerecht maskiert; Bildtitel sind keine Dateinamen.

DOCX-Bilder werden inline innerhalb der verfügbaren Satzbreite proportional und ohne Beschnitt oder
Hochskalierung gesetzt, keine schwebenden Objekte. Für PDF wird ebenfalls nie verzerrt oder
beschnitten. Die erste Bildhöhe ist auf eine vollständig verfügbare Inhaltsseite begrenzt;
Bildbeschreibung und Beschriftung dürfen weder abgeschnitten noch überlagert werden.

**PDF-Voraussetzung:** Der bisherige Renderer verwendet eingebaute Helvetica-Schriften und hat eine
offene Unicode-Grenze. Die bebilderte PDF-Stufe benötigt vor Freigabe eine überprüfte freie
einbettbare Schrift, dokumentierte Lizenz und nachgewiesene Glyphenabdeckung für den gesamten
Auftrag einschließlich Bildtexte. Nicht abbildbare Zeichen führen zum Fehler. PDF/UA-Konformität
wird mit sichtbarer Bildbeschreibung allein ausdrücklich nicht behauptet. Eine passende Schrift und
die erforderlichen Renderer-Erweiterungen sind noch nicht ausgewählt oder implementiert.

## Bildableitungen und Metadaten

Die erste Stufe gibt deterministische **PNG-Ableitungen** aus PNG, JPEG und statischem WebP aus. Der
Originalinhalt wird nicht verändert und nicht ungeprüft in Office-Dateien kopiert. Native
Dekodierung, EXIF-Orientierung, proportionale Verkleinerung mit festgelegtem Filter und Re-Encoding
erzeugen die veröffentlichte Ableitung. Lange Kante höchstens 2400 Pixel, keine Hochskalierung. Der
Implementierungsschritt muss Filter, Encoderparameter und Versionsstand reproduzierbar fixieren.
Thumbnails sind keine Exportquelle.

RGB8/RGBA8 ist das Ausgabeprofil; Transparenz bleibt für Markdown/HTML/DOCX erhalten. Die
PDF-Ableitung wird auf Weiß abgeflacht. Bei identischen Pixeln und gleicher Ableitungsart wird eine
Bilddatei innerhalb eines Pakets nur einmal gespeichert; Verwendungen bleiben getrennt
nachvollziehbar. PDF und andere Formate können deshalb unterschiedliche Ableitungshashes besitzen.

Original-EXIF/XMP/IPTC, GPS, eingebettete Vorschaubilder, ursprüngliche Dateinamen und zusätzliche
PNG-Textfelder werden nicht übernommen. ICC-Profile werden nicht als private Metadaten kopiert;
farbmetrisch identische Druckausgabe ist kein Versprechen dieses Profils. Eine reproduzierbare
Orientierungs-/Farbdarstellung des Prüfkorpus ist vor Freigabe zu belegen. Animationen und
mehrteilige Bilder müssen als solche erkannt und abgelehnt werden; keine stille Auswahl des ersten
Frames. SVG, Audio, Video, entfernte URLs und neue Bildgenerierung sind ausgeschlossen.

`medien.json` erhält die vorgeschlagene Kennung `kraqel-illustrated-manuscript`, `version: 1`. Die
endgültige Grammatik wird vor der ersten Implementierung eingefroren. Es ist kein Snapshot und kein
Wiederimportformat. Erforderliche Felder des Vorschlags:

- Paket: `format`, `version`, `manuscriptFormat`, `manuscriptFile`, `assets`, `placements`.
- Asset: lokale Exportkennung `m0001` usw., `sourceSha256`, `sourceMimeType`, `sourceByteCount`,
  `sourceWidth`, `sourceHeight`, `provenance: "imported"`, `exportSha256`, `exportMimeType`,
  `exportByteCount`, `exportWidth`, `exportHeight`, `location`.
- Bildvorkommen: nullbasierte Kapitel-/Szenenposition, nullbasierte Bildposition innerhalb der
  Szene, Exportkennung, ausdrücklich ausgegebener Bildtitel, `altText` und optionale `caption`.
- `location` ist bei Markdown/HTML ein relativer Paketpfad, bei DOCX der interne ZIP-Part und bei
  PDF ein relativer PNG-Nachweispfad plus lokale Bildkennung zur Zuordnung im Dokument; Seitenzahlen
  sind kein stabiler Anker.

Assetkennungen werden nach sortiertem Originalhash vergeben, Bildvorkommen in Manuskriptreihenfolge
ausgegeben. Objektschlüssel sind kanonisch sortiert; keine Zeitstempel oder Zufallswerte. Derselbe
Auftrag mit demselben Modell, denselben Quellbytes und derselben Toolchain ergibt dieselben
Ableitungen und Manifestbytes. Formatcontainer dürfen nur dann als byte-identisch bezeichnet werden,
wenn Zufallskennungen/Zeitstempel und Betriebssystem-Metadaten tatsächlich normalisiert und geprüft
sind; andernfalls sind semantische Gleichheit und identische Bildhashes der Nachweis.

Nicht enthalten sind Datenbank-/Projekt-/Dokument-UUIDs, Datenbankpfade, Tags, vollständige
Verknüpfungsgraphen, visuelle Profile, Prompttexte oder Schlüssel. Bildtitel, Alt- und
Beschriftungstext sind ausdrücklich sichtbare Exportdaten. Es werden keine unbekannten
Lizenz-/Urheberangaben erfunden oder aus Dateinamen abgeleitet. Echte Herkunfts-/Lizenzfelder
erfordern später einen eigenen Datenvertrag; `imported` ist keine Nutzungsrechtsbehauptung.

## Native Validierung und Veröffentlichung

Vorgeschlagen ist eine zusätzliche Command-v1-Aktion `export_illustrated_manuscript`; bestehende
Commands ändern ihre Nutzlast nicht. Die Anfrage enthält nur Projekt-ID, Format und geordnete
Bildvorkommen mit Szenen-ID, Asset-ID, erwartetem Originalhash, erwarteter Szenenrevision,
Alternativtext und optionaler Bildunterschrift. Keine Pfade, Bildbytes oder vom Browser gerenderte
HTML-Fragmente. Ein eigener Ergebnistyp liefert relativen Paketnamen, Format, Dateianzahl,
Bytegröße, Bildanzahl und Vorkommensanzahl; keine absoluten Pfade.

Rust löst sämtliche Identitäten projektgebunden auf, prüft Szenenart, direkte Verknüpfungen, Format,
Grenzen und fremde/doppelte Einträge erneut. Ein konsistenter SQLite-Lesestand umfasst das
vollständige Manuskript sowie ausgewählte Asset-/Verknüpfungsdaten. Rohdateien werden nur über
verwaltete reguläre unverknüpfte Pfade gelesen, begrenzt, mit dem erwarteten SHA-256 verglichen und
vollständig dekodiert. MIME, Abmessungen und Bytegröße müssen zu den Metadaten passen. Nach Prüfung
werden nur diese eingefrorenen Bytes verarbeitet, kein späteres erneutes Öffnen desselben Pfads.

Vor Veröffentlichung wird kontrolliert, dass der erfasste Projekt-/Dokument-/Auswahlstand nicht
zwischenzeitlich umbenannt, gespeichert, umgeordnet, verknüpft, ersetzt oder gelöscht wurde. Eine
Abweichung ist ein Konflikt und verlangt einen neuen Auftrag. Der Auftrag überschreibt weder
Editorinhalt noch Medienmetadaten. Eine unbeteiligte, nicht ausgewählte defekte Bilddatei ist kein
Fehler des reinen Texts oder einer anderen gültigen Auswahl.

Das Paket wird vollständig in einem neuen Geschwister-Staging-Verzeichnis erstellt, auf Dateiliste,
Pfade, Bildreferenzen, Manifesthashes und Zielgröße geprüft und erst dann durch Umbenennen als neue
Generation veröffentlicht. Der UUID-Zielname darf zuvor nicht existieren; kein vorhandener Ordner
wird ersetzt. So benötigt der Vertrag keinen vermeintlich überall atomaren Austausch zweier belegter
Verzeichnisse. Bei Fehlern wird Staging entfernt, ältere vollständige Pakete bleiben intakt.
Symlinks, ungeeignete Elternverzeichnisse und bereits belegte Ziele werden abgelehnt. Ein Auftrag
wird nicht gleichzeitig doppelt ausgeführt. Plattformübergreifende Dateisystemrennen bleiben wie
beim vorhandenen Projektmanager zu bewerten. Die zufällige äußere Generationskennung ist kein
Bestandteil von Manifest, Dokumenten oder Bildreferenzen; deren Reproduzierbarkeit bleibt getrennt
von Ordnernamen prüfbar.

## Vorgeschlagene harte Grenzen

| Größe                                                                  | Grenze für die erste Stufe                               |
| ---------------------------------------------------------------------- | -------------------------------------------------------- |
| Ausgewählte unterschiedliche Originale                                 | 50                                                       |
| Bildvorkommen insgesamt                                                | 200; doppelte Asset-/Szenenpaare unzulässig              |
| Original pro Bild                                                      | Bestehende 25 MiB, 16384 Pixel/Kante, 40 Millionen Pixel |
| Summe Originalbytes / Quellpixel nach Hash-Deduplizierung              | 128 MiB / 200 Millionen Pixel                            |
| Ableitung                                                              | Lange Kante 2400 Pixel, höchstens 8 MiB je PNG           |
| Gesamtes veröffentlichtes Paket einschließlich Manuskript und Manifest | 128 MiB                                                  |
| Manifest / Dateien insgesamt                                           | 1 MiB / 64 reguläre Dateien                              |
| Bildtitel / Alternativtext / optionale Beschriftung                    | 240 / 500 / 1200 Unicode-Zeichen                         |

Die Grenzen gelten gleichzeitig; die niedrigere effektive Grenze entscheidet. Dekodierung und
Ableitungen erfolgen nacheinander unter zusätzlichem Speicherbudget, nicht als 50 parallele
Vollbilder. Eine sehr schlecht komprimierbare gültige Quelle kann die Ableitungsgrenze verletzen;
dann scheitert der Auftrag vollständig. Keine automatische Qualitätsabsenkung, Verkleinerung unter
die festgelegte Ableitung oder stille Reduktion der Auswahl zur Einhaltung des Limits.

## Fehler und bestehende Formatgrenzen

Fehlende, geänderte, nicht dekodierbare, animierte, zu große oder fremde ausgewählte Bilder brechen
den gesamten bebilderten Auftrag ab. Rust ordnet dies den bestehenden bereinigten Fehlerklassen zu;
Konflikte bleiben von ungültigen Eingaben und internen Schreibfehlern unterscheidbar. Keine Pfade,
Decoderdiagnosen oder Bildbytes in IPC-Fehlern. Die Oberfläche zeigt keinen erfolgreichen Teilexport
und ersetzt die Auswahl nicht automatisch durch Platzhalterbilder.

Die Auswahl, Reihenfolge und Bildtexte sind tastaturbedienbar; Fehler und Abschluss werden als
Status/Alert angesagt. Der Erfolg benennt Paket und Bildumfang und erklärt bei Markdown/HTML, dass
der ganze Ordner weitergegeben werden muss. Ein Aufruf zum Öffnen des Exportordners leitet das Ziel
weiterhin ausschließlich in Rust aus der Projektkennung ab.

**DOCX zum Wiederimport** bleibt auf dem geprüften `kqm1`-/`KRAQEL.Manuscript.v1`-Textprofil. Kein
Medienanhang, keine Lockerung seiner Paket-Allowlist und kein stilles Wegwerfen von Bildern beim
Import. Ein bebildertes gewöhnliches DOCX ist kein Roundtrip-v1-Dokument und wird vom
Roundtrip-Reader abgewiesen. Snapshot v3 und bestehende Backups bleiben für vollständige Projekte.
Ein späterer Bild-Roundtrip benötigt eine neue ausdrücklich entschiedene Formatversion.

## Abnahme und Umsetzung in getrennten Stufen

1. **Vertrag und gemeinsamer Rust-Kern:** fachliche Regeln/Parameter freigeben, genaue
   Manifestgrammatik und DTOs festlegen, Auswahlmodell, sichere Quellauflösung, Ableitung, Budgets
   und atomaren Paketwriter implementieren. Keine Änderung am Editor-Dokument v1 oder an Snapshot v3
   vorgesehen.
2. **Markdown und HTML:** erste bedienbare Ausgabe mit Auswahl am Szenenende, relativen Bildern und
   Manifest. Andere bebilderte Formate bleiben bis zur jeweiligen Abnahme nicht verfügbar.
3. **Gewöhnliches DOCX:** interner Bildwriter, reine interne Beziehungen, Alttexte/Beschriftungen,
   echte Word-/LibreOffice-Öffnung und visuelle Prüfung. Roundtrip-v1-Negativtest bleibt bestehen.
4. **PDF:** freie Schrift und Unicode-Abdeckung zuerst nachweisen, anschließend Bildlayout,
   Seitenumbrüche und sichtbare Bildbeschreibungen implementieren und prüfen.

Jede Renderer-Stufe braucht Rust-Tests mit synthetischen beziehungsweise klar lizenzierten Bildern,
Commandtests, Browser-E2E, Tastatur/Axe sowie echte native Ausgabe- und visuelle Abnahme.
Pflichtfälle:

- 100 Szenen und 50 ausgewählte kleine Originale, wiederholte Verwendung, keine Verknüpfung, mehrere
  Linkrollen, gleiche Titel, leere Szenentitel und explizite Reihenfolge.
- Nur die gewählten Szenenbilder erscheinen; reine Entitäts-/Profil-/Ereignisbilder und private
  Tags, Originalnamen, GPS und Prompts fehlen. Verschachtelte Textstruktur bleibt semantisch
  unverändert.
- PNG/JPEG/statisches WebP, Transparenz, Orientierung, Hoch-/Querformat, Randgrößen, sehr lange
  Beschriftungen und unterstütztes Unicode; keine Verzerrung, Überlagerung oder abgeschnittenen
  Bilder.
- Quelldatei fehlt, ist verändert, verknüpft, übergroß oder animiert; falscher MIME/Hash, fremde
  UUID, doppelte Vorkommen und Grenzüberschreitungen führen zum vollständigen Fehler.
- Autosave, Umbenennen, Umordnen, Entkoppeln und Bildersatz während des Auftrags erzeugen Konflikte;
  alte vollständige Pakete bleiben bei Decoder-/Schreib-/Veröffentlichungsfehlern erhalten;
  misslungene Aufträge veröffentlichen keine neue Generation.
- Wiederholung liefert gleiche Manifest-/Ableitungshashes; alle Verweise sind auflösbar, Ordnerkopie
  auf einen anderen lokalen Pfad funktioniert offline, keine absolute Pfadabhängigkeit.
- Reine Exporte, Medienverwaltung, Snapshot/Backup und DOCX-Text-Roundtrip bleiben
  regressionsgeprüft.

Die Markdown-/HTML-Stufe wird mit KQ-063 und separatem Abnahmeprotokoll nachgewiesen. DOCX/PDF und
vollständige F08-/M3-Abnahme bleiben offen; jede weitere Stufe benötigt eigene Nachweise.
