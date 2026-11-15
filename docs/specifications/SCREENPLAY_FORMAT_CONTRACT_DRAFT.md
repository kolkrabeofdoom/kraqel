# Drehbuchformatierung: fachlich freigegebener Vertrag v1

Stand: 15. November 2026 · KQ-066 / Issue #129 · **Am 15. November 2026 vom Projektinitiator
fachlich freigegeben; noch nicht implementiert.** Codebasis:
`4679c893e021c44ff9b471cac0783458e7640f2b` (PR #128 grün zusammengeführt).

## 1. Quelle und fachliche Freigabe

Das [Pflichtenheft Revision 1](KRAQEL_Technisches_Pflichtenheft_v0.1_Revision_1.pdf) nennt
Drehbücher als Produktziel (§1), definierte Formatierungen für F08 (§3), Datenportabilität in M3
(§8) und nachvollziehbare Exporte (§10). §11 lässt den Umfang der Drehbuchformatierung ausdrücklich
offen. Es schreibt **keine** Absatzrollen, Schrift, Papiergröße oder Branchenkonformität vor.

Die nachfolgenden Maße, Rollen, Bedienabläufe und Versionsgrenzen sind fachlich freigegebene
KRAQEL-Festlegungen. Sie sind kein behaupteter Film-/Fernsehstandard und keine Zusage bestimmter
Seitenlaufzeiten. Die ausdrückliche Freigabe „Der Vertrag passt so“ am 15. November 2026 entscheidet
diesen Scope einschließlich der Format-/Importrichtungen. Erst spätere Implementierung und
dokumentierte Abnahme machen daraus unterstütztes Produktverhalten. Siehe
[ADR 0033](../architecture/decisions/0033-screenplay-contract-proposal.md) und
[F08-/M3-Matrix](../testing/F08_M3_ACCEPTANCE_MATRIX.md).

## 2. Ausgangslage und Ziel

Editor-Dokument v1 und das kanonische Manuskriptmodell kennen keine Drehbuchrollen. Die heutigen
Renderer lehnen unbekannte Knoten/Attribute ab. Ein eingerückter Absatz ist deshalb noch kein
Dialog, und eine Überschrift ist noch kein Szenenkopf. DOCX-Manuskriptaustausch v1 erhält
ausschließlich seine dokumentierte Textsemantik, kein Seitenlayout und keine Drehbuchtypen.

Fachlich vorgesehen ist eine explizite Rolle je Drehbuchabsatz innerhalb einer bestehenden
KRAQEL-Szene. Kapitel-/Szenenbaum, IDs, Story-Bibel und Medienlinks bleiben fachlich maßgeblich.
Prosa- und Drehbuchszenen dürfen im selben Projekt vorkommen; es wird kein globaler
Projektgenrewechsel verlangt. Es gibt keine automatische Erkennung aus Großbuchstaben, Einzügen,
Figurennamen oder `INT./EXT.`.

## 3. Rollen und kanonisches Modell

Fachlich festgelegter neuer Block: `screenplayBlock` mit genau `attrs.role` aus der folgenden Menge
und Inline-Inhalt aus Text, Fett/Kursiv und harten Umbrüchen. Keine Listen, Zitate oder
verschachtelten Drehbuchblöcke innerhalb dieses Blocks. Reihenfolge, Rolle, Unicode, Leerzeichen,
leere Blöcke und Inline-Markierungen sind kanonische Semantik; Einzüge und Schrift sind
Ausgabeprofil.

| Maschinenrolle  | Sichtbare Bezeichnung | Bedeutung und Ausgabe                                               |
| --------------- | --------------------- | ------------------------------------------------------------------- |
| `sceneHeading`  | Szenenkopf            | Ort/Zeit als frei eingegebener Text; eigene Zeile, ganze Satzbreite |
| `action`        | Handlung              | Beschreibender Text, ganze Satzbreite                               |
| `character`     | Figur                 | Sprecher*innenzeile; keine automatische Verknüpfung zur Story-Bibel |
| `parenthetical` | Klammeranweisung      | Sprech-/Spielanweisung innerhalb eines Dialogblocks                 |
| `dialogue`      | Dialog                | Gesprochener Text                                                   |
| `transition`    | Übergang              | Expliziter Übergangstext, rechtsbündig innerhalb der Satzbreite     |

Alle Rollen enthalten Autor*innentext: Keine automatisch eingefügten Klammern, Großschreibung,
Doppelpunkte, Szenennummern oder Namensersetzung. Groß-/Kleinschreibung und Kennzeichnungen wie
`(V.O.)` werden bewusst eingegeben und unverändert erhalten. NFC-Normalisierung gilt ausschließlich
für die begrenzte PDF-Ausgabe wie beim bestehenden PDF-Profil, nicht als Änderung in SQLite.

Ein Kapitel besitzt weiterhin keinen eigenen Manuskripttext. In einer nichtleeren Drehbuchszene ist
genau ein nichtleerer `sceneHeading` der erste nichtleere Block; seine Position leitet sich nicht
aus dem bearbeitbaren Szenentitel ab. Leere Szenen bleiben speicher- und exportierbar. Innerhalb
einer Szene werden Prosa-Blöcke und Drehbuchblöcke in Stufe 1 nicht gemischt. Eine Szene wird erst
durch ausdrückliche Rollenvergabe zur Drehbuchszene, nicht durch ihren Namen.

Dialogfolge: `character`, optional `parenthetical`, dann mindestens ein `dialogue`; anschließend
dürfen weitere Paare aus optionalem `parenthetical` und `dialogue` folgen. `action`, `transition`,
neue `character`-Zeile oder Szenenende beenden den Dialogblock. Verwaiste Anweisungen/Dialoge, leere
Sprecher*innenzeilen und unvollständige Dialogblöcke dürfen beim Schreiben vorübergehend existieren,
verhindern aber den gesamten Drehbuchexport mit sichtbarer, bereinigter Erklärung. Leere
`action`-Blöcke sind zulässig; sie bilden eine Leerzeile. Leere `transition`-Blöcke sind beim Export
ungültig. Weitere leere Rollen erfüllen keine obligatorische Rolle oder Dialogfolge.

## 4. Bearbeitung und Datenintegrität

Eine beschriftete, tastaturbedienbare Rollenauswahl verändert nur ausgewählte Absätze nach einer
expliziten Aktion. Mehrfachauswahl zeigt gemischte Rollen; Rolle und Inhalt sind mit Undo/Redo
gemeinsam rückgängig zu machen. Keine automatische Rollenfolge beim Tippen oder bei Enter in
Stufe 1. Ein neuer Absatz übernimmt die bisherige Rolle; die UI zeigt unvollständige Folgen als
bearbeitbare Hinweise. Leere Blöcke entstehen nicht durch versteckte Konvertierung.

Prosa → Drehbuch und zurück verlangen eine sichtbare Zuordnungsvorschau: Es wird kein Text gelöscht,
keine Liste implizit abgeflacht und kein Szenenkopf aus dem Titel erfunden. Nicht abbildbare
Strukturen verhindern die Konvertierung. Eine Rückwandlung nach Prosa weist darauf hin, dass die
Rollen verloren gehen; sie ist eine eigene bestätigte Aktion mit Undo, keine Exportnebenwirkung. Die
Umsetzung muss die bestehende Revisionsprüfung, Autosave-Serialisierung, FTS-Klartexte und stabilen
Dokument-/Medien-IDs erhalten. Kein schreibender Provider- oder Netzwerkpfad.

## 5. Erstes Layoutprofil: `kraqel-screenplay-a4-v1`

Dieses festgeschriebene Profil ist für die erste Stufe fachlich freigegeben. US Letter und
individuell konfigurierbare Profile folgen nur nach eigener Entscheidung. Es gibt kein automatisches
Papierformat je Betriebssystem und keine Zusage „eine Seite = eine Minute“.

| Eigenschaft              | Vorschlag                                                                          |
| ------------------------ | ---------------------------------------------------------------------------------- |
| Papier                   | A4, 210 × 297 mm                                                                   |
| Ränder                   | links 30 mm, rechts 25 mm, oben 25 mm, unten 20 mm                                 |
| Satzbreite / Inhaltshöhe | 155 mm / 252 mm                                                                    |
| Schrift                  | offline gelieferte, frei einbettbare Monospace-Familie, vier echte Schnitte, 12 pt |
| Zeilenhöhe               | 14 pt; keine automatische Silbentrennung                                           |
| Absatzabstand            | davor 0 pt, danach 12 pt; innerhalb eines Dialogblocks danach 0 pt                 |
| Figur                    | linker Einzug 55 mm, rechter Einzug 0 mm relativ zur Satzbreite                    |
| Klammeranweisung         | links 40 mm, rechts 25 mm                                                          |
| Dialog                   | links 25 mm, rechts 25 mm                                                          |
| Szenenkopf / Handlung    | keine zusätzlichen Einzüge; linksbündig                                            |
| Übergang                 | keine zusätzlichen Einzüge; rechtsbündig                                           |

Die eingebetteten Noto-Sans-Schriften aus KQ-065 sind **keine** Monospace-Freigabe für dieses
Profil. Eine passende Familie (Kandidat: Noto Sans Mono) muss vor Umsetzung mit unveränderlichem
Quellstand, Hashes, Lizenz/Copyright, Einbettbarkeit, vier Schnitten und Glyphenkorpus separat
geprüft werden. Es findet durch diesen Vertrag weder Download noch Installation statt. Ohne
bestandene Schriftprüfung darf kein Drehbuch-PDF als profilkonform angeboten werden.

Szenenkopf und mindestens zwei folgende Textzeilen bleiben zusammen, soweit vorhanden. Figurenzeile,
zugehörige Klammeranweisung und die ersten zwei Dialogzeilen bleiben zusammen. Lange
Dialoge/Handlungen dürfen an Zeilengrenzen umbrechen; verbleibende einzeilige Absätze werden nach
Möglichkeit auf die nächste Seite genommen. Passt eine untrennbare Gruppe selbst auf eine leere
Seite nicht, scheitert die Ausgabe ausdrücklich. Kein Abschneiden, Verkleinern der Schrift oder
automatisches Ergänzen von `MORE`/`CONT'D`. Keine erzwungene neue Seite je Szene.

Projekt-/Kapitel-/Szenentitel bleiben vollständig im Modell und in Austauschformaten. Ein eigener
Drehbuch-Druckmodus gibt nur die Rollenblöcke der ausdrücklich gewählten Drehbuchszenen in
Baumreihenfolge aus; die Vorschau nennt den Ausschluss struktureller Titel und nicht gewählter
Szenen. Bei gemischten Projekten werden Prosa-Szenen nicht automatisch ausgelassen: Die Auswahl muss
ausdrücklich auf reine Drehbuchszenen eingeschränkt sein, sonst Fehler vor Veröffentlichung. Ein
PDF-Titelblatt, Kopf-/Fußzeilen, Seitenzahlen und Revisionsfarben sind nicht Teil von Stufe 1.

## 6. Format- und Importmatrix des freigegebenen Umfangs

| Weg                               | Vorgeschlagene Garantie                                                            | Begrenzung                                                                     |
| --------------------------------- | ---------------------------------------------------------------------------------- | ------------------------------------------------------------------------------ |
| Editor / SQLite / Backup          | Exakte Rollen und Textsemantik; Bearbeitung und Neustart                           | Neue Version muss zuerst implementiert und geprüft werden                      |
| Snapshot                          | Rollen, IDs, Links und Text verlustfrei in neuer Version                           | Bestehender v3-Importer wird nicht still erweitert                             |
| Markdown                          | Je Absatz ein fetter Rollenname, Leerzeile, maskierter Inline-Inhalt und Leerzeile | Lesbare Ausgabe, kein neuer Markdown-Roundtrip                                 |
| HTML                              | Eigene semantisch benannte Klassen/Attribute mit lokalen Druckstyles               | Keine Zusage gleicher Seitenumbrüche in verschiedenen Browsern                 |
| Gewöhnliches DOCX                 | Benannte Absatzstyles mit profilgemäßen Einzügen                                   | Layout kann in Office variieren; kein v1-Wiederimport                          |
| PDF-Druckmodus                    | Festes A4-Profil, eingebettete Monospace-Schrift, vorab geprüfte Glyphen           | Eigene Abnahme aller Seiten nötig; keine PDF/UA- oder Universal-Unicode-Zusage |
| DOCX-Manuskriptaustausch          | Späteres v2-Profil mit expliziten Rollen im Inventar                               | v1 weist Rollen ab; keine bloße Style-Heuristik                                |
| Fremdes DOCX / TXT / Markdown     | Bestehende begrenzte Importwege bleiben erhalten                                   | Keine Erkennung von Drehbuchrollen aus Schrift/Einzug/Textmustern              |
| FDX / Fountain / HTML-/PDF-Import | Nicht Teil dieser Stufe                                                            | Keine behauptete neue Formatzusage aus der allgemeinen F08-Formatliste         |

Der bisherige Prosa-PDF-Pfad und DOCX-Manuskriptaustausch v1 dürfen neue Rollen nicht still
abflachen; solche Aufträge scheitern vor Veröffentlichung. Markdown/HTML und gewöhnliches DOCX
dürfen nach ihrer v2-Erweiterung Prosa- und Drehbuchszenen gemeinsam mit erhaltener Baumstruktur
ausgeben. Der vorgeschlagene A4-Drehbuch-Druckmodus bleibt dagegen auf explizit gewählte reine
Drehbuchszenen beschränkt. Eine gemischte PDF-Gesamtausgabe ist nicht Teil von Stufe 1.

Markdown verwendet exakt die sichtbaren deutschen Rollennamen aus §3 als `**Rollenname**`; Text und
Fett/Kursiv werden wie im bestehenden Renderer maskiert, harte Umbrüche mit zwei Leerzeichen vor LF
dargestellt. Projekt-/Kapitel-/Szenentitel bleiben gewöhnliche Überschriften. HTML verwendet
`<p class="screenplay screenplay-<role>" data-kraqel-role="<role>">` mit der festen Maschinenrolle;
Inhalt wird maskiert, harte Umbrüche als `br` ausgegeben. Druck-CSS bildet die Maße aus §5 ab;
`font-family: monospace` ist nur ein lokaler Fallback, keine reproduzierbare Schrift- oder
Seitenbruchgarantie. Kein Remote-CSS und kein Webfontabruf.

Gewöhnliches DOCX verwendet eigene Absatzstyle-IDs `KQScreenplaySceneHeading`, `KQScreenplayAction`,
`KQScreenplayCharacter`, `KQScreenplayParenthetical`, `KQScreenplayDialogue` und
`KQScreenplayTransition`. Maße und Schriftname folgen §5. Ohne auf dem Zielsystem verfügbare Schrift
kann Office ersetzen; diese Ausgabe garantiert weder identische Seiten noch rollenbewahrenden
Rückimport. Nur ein späteres explizites DOCX-v2-Austauschprofil kann Rollen kanonisch binden; ein
Fremd-DOCX-Import kann lediglich zulässigen Prosa-Text übernehmen.

Der bereits freigegebene Szenenbildexport bleibt für Prosa unverändert. Kombination aus
Drehbuch-Druckmodus und Bildern wird in Stufe 1 ausdrücklich abgelehnt, nicht still ausgelassen. Ein
eigener Storyboard-/Bild-Drehbuch-Vertrag kann später das Layout festlegen.

## 7. Versionierung, Ressourcen und Veröffentlichung

Neue Rollen sind inkompatibel zum heutigen kanonischen Modell. Vorgeschlagene Implementierungsfolge:
Editor-Dokument v2, passender versionierter Load-/Save-Command-Vertrag v2, Snapshot v4 und bei
Erweiterung des Manuskriptaustauschs DOCX-Profil v2. Exakte Felder/Markergrammatik und Migration
werden vor dem jeweiligen Code in Datenformat und ADR fixiert. Versionen werden **jetzt nicht**
angehoben. Neue Leser müssen v1-Prosa explizit weiterhin unterstützen; versionierte eigene
Austauschprofile dürfen neue Rollen nicht als gewöhnliche Absätze importieren. Gewöhnliches DOCX
trägt dagegen keine Garantie gegen verlustbehafteten Fremdimport als Prosa. Kein pauschales
Umschreiben vorhandener Projekte.

Ein zukünftiger Datenbankwechsel benötigt eine neue unveränderliche Migration und befüllte
Vorwärtsmigrationstests; `user_version = 6` bleibt in dieser Dokumentationsstufe unverändert.
Snapshot v3 und DOCX v1 bleiben eingefroren. Alle Importwege validieren Version, Rolle, Inhalt,
Hierarchie und Budgets erneut in Rust, bevor neue Projekte atomar veröffentlicht werden.

Die neue Stufe darf bestehende Grenzen nicht still erhöhen: Editor-/Titel-Speicherlimits, beim
DOCX-Austausch 8 MiB Datei, 32 MiB entpackt, 2 MiB Text, 4096 Inventarknoten und Tiefe 32; beim PDF
8 MiB Text, 100000 Zeilen, 2048 Seiten und 128 MiB Ausgabe. Die tatsächlichen kombinierten Grenzen
müssen vor Umsetzung im neuen Profil stehen. Ohne Bildfreigabe gelten keine neuen Bildbudgets.
Unbekannte Rolle, Version, Glyphen oder übergroße Gruppen verursachen einen Fehler vor Dateiersatz;
alte Ausgaben und Quelldaten bleiben unverändert. IPC übermittelt keine Pfade, Quelldaten, Fontbytes
oder Bibliotheksdiagnosen in Fehlermeldungen.

## 8. Verbindliche Abnahmefälle für eine spätere Umsetzung

| ID    | Nachweis vor einer Funktionsfreigabe                                                                                                                              |
| ----- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| DB-01 | Alle sechs Rollen, leere Szenen, Rollenwechsel, Undo/Redo, Autosave-Konflikt und echter Neustart erhalten Text/Rollen/IDs; Tastatur und Screenreader prüfen       |
| DB-02 | Positive und negative Dialogfolgen, zwei Szenenköpfe, Prosa-Mischung, unbekannte Rollen/Attribute und beschädigte Versionshüllen werden deterministisch behandelt |
| DB-03 | Bestehender v1-Prosa-Korpus bleibt unverändert; v2 kann nicht still als v1/v3 gelesen werden; Migration/Backup/Snapshot und FTS geprüft                           |
| DB-04 | Lateinisch/Griechisch/Kyrillisch, NFD/NFC, fehlende Glyphen, Emoji und überlange Wörter; vier echte Schriften ohne Systemfont-Abhängigkeit                        |
| DB-05 | Alle Absatzmaße, Seitenränder, gemischte Schnitte und Rechtsbündigkeit unabhängig messen; PDF vollständig rendern und visuell prüfen                              |
| DB-06 | Seitenende unmittelbar vor/nach Szenenkopf, Figurenzeile und Anweisung; mehrseitiger Dialog; untrennbare Übergröße scheitert ohne Dateiersatz                     |
| DB-07 | Markdown-/HTML-Maskierung sowie Öffnung/Speicherung von DOCX in Word und LibreOffice; spätere v2-Rollenmarker separat rücklesen                                   |
| DB-08 | Native App mit synthetischen Drehbuchszenen, Abbruch/Fehler, Ausgabealterhalt, expliziter Szenenauswahl und abgewiesenem Bildmodus                                |

Fachlich freigegebener nächster Implementierungsschritt: Modell-/Versionsprototyp mit DB-01 bis
DB-03 und Schriftprüfung, danach Renderer und Office-Abnahme. Die fachliche Freigabe ist keine
Funktionsabnahme; der anschließende M3-Gesamttest implementiert keine Drehbuchfunktion.
