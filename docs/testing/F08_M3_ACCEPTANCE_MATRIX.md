# F08-/M3-Abnahmematrix

KQ-069 A untersucht die ausdrücklich beauftragte strikte Isolation: Der
[AppContainer-Prototyp](native-runtime-isolation.md) zeigt bei gültiger ETW-Messung drei
vollständige App-Läufe mit 0 Ereignissen außerhalb Loopback; derselbe unisolierte Build zeigt 201
öffentliche WebView2-Ereignisse. Projekt/Autosave/Neustart/HTML/PDF im synthetischen Korpus geprüft,
Dateidialoge/Austausch, weitere Broker-/UDP-/IPv6-Pfade und vollständige assistive/funktionale
Wiederholung offen. Kein produktiver Launcher eingebunden: **Q-01/Q-02 bleiben nicht bestanden**.

**Fortschreibung KQ-068 / Issue #133:** PR #132 ist grün zusammengeführt (`c7629c3`).
[Restprüfungsprotokoll](native-m3-coverage.md): M3-05 und M3-10 sind im dokumentierten CRUD-Korpus
jetzt **N**. Der Nutzer bestätigt den manuellen kanonischen Bildwechsel und die
Profil-/Stil-/Fehleransagen. Die drei ausdrücklich freigegebenen GUI-Löschungen,
Datei-/Referenzprüfung und vollständiger Prozessneustart sind bestanden. Bild A, Tags, Szenenlink
und Figur bleiben unverändert. Vier anschließende bebilderte Exporte sind per Tastatur erzeugt und
unabhängig rückgelesen. Der Nutzer bestätigt auch Löschmeldungen, Export-Bedienelemente und Erfolge
mit „Ja passt, alles gut“. Die gezeigte Hörprobe ist bestanden; keine pauschale assistive F08- oder
WCAG-Freigabe.

KQ-069 ergänzt die [NetLog-Diagnose](native-m3-webview-network.md): Konfigurations- und
Komponentenupdate-Starts schon ohne Projekte; keine Inhalte-/Netzfreigabe. Unterstützte Maßnahmen
sind bewertet, keine Unterdrückung implementiert. Der erneute ETW-Lauf ist mit allen Prozessenden,
Dokument-CRUD, Medienimport/-Metadaten, Export/Neuimport und drei App-Starts gültig, bestätigt aber
256 öffentliche WebView2-Transportereignisse. ADR 0034 bleibt Entscheidungsvorlage; #134 offen.

**Q-01/Q-02: gemessen, nicht bestanden.** Windows-ETW findet öffentlichen Verkehr von
WebView2-Nachfahren ohne Provideraufruf. Der bisher fehlende Gesamtnachweis ist durch diesen
konkreten Befund ersetzt; [KQ-069 / #134](https://github.com/kolkrabeofdoom/kraqel/issues/134) hält
ihn offen. Keine Zweck-/Uploadbehauptung aus Transportmetadaten, keine physische Netztrennung und
kein vollständig wiederholter KQ-067-Lauf in dieser Messung. Die folgenden historischen Belegzeilen
überschreiben diesen neueren Befund nicht. F08/M3 bleiben teilweise abgenommen.

Stand: 15. November 2026 · KQ-066 / Issue #129, fortgeschrieben durch KQ-067 / Issue #131.
Ursprüngliche Codebasis PR #128 (`4679c89`); der funktionale native Gesamtlauf basiert auf dem mit
grüner CI zusammengeführten PR #130 (`58dea4b`) plus den KQ-067-Testhelfern und der korrigierten
Medienstatusmeldung. Issues #127/#129 sind geschlossen; Issue #94 bleibt separat als
Abhängigkeitsaufgabe offen. [KQ-067-Abnahmeprotokoll](native-m3-acceptance.md) dokumentiert die
neuen Laufzeit-/GUI-Prüfungen und ihre Grenzen.

**Gesamturteil: F08 und M3 bleiben teilweise erfüllt, nicht vollständig abgenommen.** Die Matrix
erfasst vollständig die einschlägigen Anforderungen und Restprüfungen. Sie erklärt nicht alle
aufgeführten Kriterien für bestanden und ersetzt keine allgemeine F01–F09-/0.1-Abnahme.

## Quellen und Bewertungsregeln

Primärquelle ist das
[Pflichtenheft v0.1 Revision 1](../specifications/KRAQEL_Technisches_Pflichtenheft_v0.1_Revision_1.pdf):
§1 Produktziele; §3 F07/F08/F09; §4 Referenzintegrität; §6 V.1; §7 Snapshotregeln; §8 M3; §10
Offline-, Speicherintegritäts-, Referenz-, Datenschutz-, Git- und Exportkriterien; §11 offene
Formatentscheidungen. Bestehende ADRs und Datenverträge konkretisieren den Umfang; die Drehbuch-ADR
0033 ist nach ausdrücklicher Vertragsfreigabe am 15. November 2026 fachlich angenommen, aber nicht
implementiert. Die IDs unten sind lokale Prüfkennungen, keine neuen
Pflichtenheft-Anforderungsnummern.

| Status | Aussage                                                                                                          |
| ------ | ---------------------------------------------------------------------------------------------------------------- |
| **A**  | Automatisiert im dokumentierten Umfang belegt; keine native Vollabnahme daraus ableiten                          |
| **N**  | Zusätzlich nativ/manuell im verlinkten synthetischen Korpus und den dort genannten Builds geprüft                |
| **T**  | Teilnachweise vorhanden; explizite Restprüfung oder Entscheidung fehlt                                           |
| **O**  | Offen, kein ausreichender Nachweis bzw. keine Implementierung                                                    |
| **V**  | Konkreter Vorschlag zur fachlichen Entscheidung; keine Produktfunktion                                           |
| **X**  | Bewusst außerhalb dieses Umfangs; nur mit verlinkter Begründung, kein stiller Verzicht auf eine Muss-Anforderung |

Native Nachweise sind historische, datierte Prüfungen. Grüne CI bestätigt ihre Fixtures und
Regressionsprüfungen, wiederholt aber weder Office-GUI-Prüfungen noch Windows-Dialog- oder
Screenreader-Abnahmen. Die zuletzt geprüfte Implementierungsstufe KQ-065 hat 161 Rust- und 53
TypeScript-Tests, sechs regulär ignorierte explizite QA-Helfer und 26 Browser-E2E. Diese Zahlen und
die folgenden Belege stammen aus PR #128; sie sind kein neuer Prüflauf von KQ-066.

KQ-067 besteht zusätzlich `pnpm check` mit 162 Rust- und 53 TypeScript-Tests (sieben explizite
QA-Helfer regulär ignoriert), alle 26 Browser-E2E sowie die nativen Quell-/Recovery-Builds und den
abschließenden regulären Windows-Debug-Build. Die neue manuelle Evidenz steht im
[Abnahmeprotokoll](native-m3-acceptance.md).

## F08: unterstützte Formate und Austauschgrenzen

| ID / Quelle                     | Kriterium und aktueller Umfang                                                                                                      | Status / konkreter Beleg                                                                                                                                                               | Restprüfung bzw. Grenze                                                                                                                                                                                                                                              |
| ------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| F08-01 · §3 F08, §10 Export     | Gemeinsame Semantik: Baumreihenfolge, Titel, Absätze, Heading 1–6, Fett/Kursiv, Umbrüche, Listen und Zitate                         | **A** · [Exportvertrag](../data-format/EXPORT.md), [Modell](../../crates/export/src/manuscript.rs), [Tests](../../crates/export/src/lib.rs)                                            | Editor kann mehr Knoten erzeugen als der Export unterstützt; unbekannte Formatierung wird abgewiesen, nicht als unterstützt gezählt                                                                                                                                  |
| F08-02 · §3 F08                 | Markdown und eigenständiges HTML aus validiertem Modell                                                                             | **A** · [Renderer](../../crates/export/src/render.rs), [Export-E2E](../../apps/desktop/e2e/exports.spec.ts)                                                                            | Kein beliebiger HTML-/Markdown-Roundtrip; keine identischen Seitenumbrüche versprochen                                                                                                                                                                               |
| F08-03 · §3 F08                 | Gewöhnliches DOCX, getrennt vom Austauschprofil                                                                                     | **N** für den Bild-/Textkorpus · [DOCX-Abnahme](native-illustrated-docx-acceptance.md), [Renderer](../../crates/export/src/render.rs)                                                  | Keine universelle Office-Kompatibilität oder freie Struktur-Rückimportgarantie                                                                                                                                                                                       |
| F08-04 · §3 F08, §10 Export     | A4-PDF mit eingebetteten vier Schriften, NFC/Glyphenprüfung und metrischem Umbruch                                                  | **N** · [PDF-Abnahme](native-illustrated-pdf-acceptance.md), [PDF-Code](../../crates/export/src/pdf.rs)                                                                                | Begrenztes Glyphenprofil; keine PDF/UA-, PDF/A/X- oder universelle Shaping-Abnahme                                                                                                                                                                                   |
| F08-05 · §3 F08                 | TXT-/Markdown-Datei wird atomar neue Szene, begrenztes UTF-8-Profil                                                                 | **N** · [Textimport-Abnahme](native-text-import-acceptance.md), [Formatbeschreibung](../data-format/README.md#utf-8-markdown-import)                                                   | Inline-Markdown bleibt teilweise Klartext; Codeblöcke können importiert werden, sind im gewöhnlichen Export nicht unterstützt                                                                                                                                        |
| F08-06 · §3 F08                 | Fremdes DOCX wird begrenzt als neue Szene importiert                                                                                | **N** · [DOCX-Importabnahme](native-docx-import-acceptance.md), [Vertrag](../data-format/DOCX_IMPORT.md)                                                                               | Keine automatische Kapitel-/Szenenerkennung; Bilder, Listen und weitere fremde Inhalte werden abgewiesen                                                                                                                                                             |
| F08-07 · §11 Roundtrip          | Eigenes DOCX-v1-Profil, Textbearbeitung in bestehenden Absatzmarkern, atomarer Neuimport als neue Projektkopie                      | **N** · [Roundtrip-Abnahme](native-docx-roundtrip-acceptance.md), [Vertrag](../data-format/DOCX_ROUNDTRIP.md), [Rust-Tests](../../crates/export/src/roundtrip/tests.rs)                | Keine Bilder/Story-Bibel/Links im Manuskriptaustausch, kein freier Office-Strukturumbau; Originalprojekt wird nicht überschrieben                                                                                                                                    |
| F08-08 · §10 Export, §11 Medien | Explizite direkt verknüpfte Szenenbilder in Markdown/HTML                                                                           | **N** · [Abnahme](native-illustrated-export-acceptance.md), [Bildvertrag](../data-format/ILLUSTRATED_MANUSCRIPT.md)                                                                    | Szeneende, keine automatische Auswahl aus Profilen/Entitätslinks; vollständiges Paket weitergeben                                                                                                                                                                    |
| F08-09 · §10 Export, §11 Medien | Gewöhnliches DOCX mit internen PNG-Parts und Alttext; Bild-PDF mit weißen PNG-Nachweisen                                            | **N** · [DOCX](native-illustrated-docx-acceptance.md), [PDF](native-illustrated-pdf-acceptance.md)                                                                                     | Kein Bild-Roundtrip; PDF-Bildbeschreibung sichtbar, keine Zusage getaggter Alternativtexte                                                                                                                                                                           |
| F08-10 · §10 Export             | Unbekannte Inhalte, beschädigte Pakete, Limits und Glyphenfehler verhindern Teilveröffentlichung; frühere Ausgaben bleiben bestehen | **A/N** · [Roundtrip-Tests](../../crates/export/src/roundtrip/tests.rs), [Bildtests](../../crates/export/src/illustrated/tests.rs), native Abnahmen oben                               | Dateisystem-Austauschrennen bleibt laut Threat Model begrenzt; keine harte RSS-Garantie                                                                                                                                                                              |
| F08-11 · §1, §11 Drehbuch       | Eigene Absatzrollen, Darstellung und Exportsemantik                                                                                 | **O** für Implementierung · [Freigegebener Vertrag](../specifications/SCREENPLAY_FORMAT_CONTRACT_DRAFT.md), [ADR 0033](../architecture/decisions/0033-screenplay-contract-proposal.md) | Fachliche Freigabe liegt vor; Schriftprüfung, Modell/Versionen, Implementierung und DB-01–DB-08 fehlen                                                                                                                                                               |
| F08-12 · §3 F08, §11            | Richtungen und Grenzen der Formatliste ausdrücklich festlegen                                                                       | **T** · ADR 0008 legt vier **Ausgaben** fest; Importverträge definieren nur TXT/Markdown, begrenztes DOCX, eigenes DOCX-Profil und Snapshot v3                                         | Allgemeiner HTML-/PDF-Import und FDX/Fountain sind nicht implementiert. Die bloße F08-Formatliste wird nicht als Garantie beider Richtungen gelesen; diese Auslegung und der Drehbuchumfang sind durch ausdrückliche Vertragsfreigabe am 15. November 2026 bestätigt |

## M3: Git, Wiederherstellung, Medien und visuelle Profile

| ID / Quelle                               | Kriterium und aktueller Umfang                                                                               | Status / konkreter Beleg                                                                                                                                                                                                                                                       | Restprüfung bzw. Grenze                                                                                                                              |
| ----------------------------------------- | ------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------- |
| M3-01 · §3 F07, §7, §10 Git               | Deterministische Snapshots, Hash-Assets, keine laufende SQLite/Schlüssel/temporären Daten in Git             | **A** · `snapshot_is_byte_stable_and_git_never_tracks_database`, `snapshot_copies_assets_by_hash_and_rejects_tampering` in [Git-Service](../../crates/git-service/src/lib.rs); [Snapshotvertrag](../data-format/README.md)                                                     | PDF-Containerstabilität ist davon unabhängig; Snapshotstabilität bleibt geprüft                                                                      |
| M3-02 · §3 F07                            | Lokales Init, Status, Diff, expliziter Commit                                                                | **N** im dokumentierten Korpus · Zwei echte native Snapshot-/Commitstände, Status und lesbarer Diff; Git-Verlauf unabhängig geprüft. Siehe [M3-Abnahme](native-m3-acceptance.md).                                                                                              | Keine assistive Git-Vollabnahme oder zusätzliche Git-Checkout-/Merge-Funktion.                                                                       |
| M3-03 · §3 F07, §8 M3                     | Lesende Wiederherstellung aus Export-Snapshot: v3-Archiv als neue unabhängige Projektkopie                   | **N** im dokumentierten Korpus · Nativer Archivexport, zwei aktuelle Neuimporte und zusätzlicher Import des früher gesicherten Stands; vollständiger Inventarvergleich und unabhängige Projektidentitäten. [Nachweis](native-m3-acceptance.md).                                | Keine Überschreibung/kein Git-Checkout; Wiederherstellung aus ausdrücklich gesichertem Snapshot, keine automatische Auswahl historischer Git-Stände. |
| M3-04 · §8 M3, §11 Paket                  | Deterministisches `.kraqel`-Archiv, eigener nativer Datei-/Speicherdialog, keine absolute Pfad-/Byte-IPC     | **N** im dokumentierten Dialogumfang · [ADR 0019](../architecture/decisions/0019-atomic-snapshot-import-and-bundled-example.md), [ADR 0021](../architecture/decisions/0021-native-snapshot-archive-export.md), [Status](../DEVELOPMENT_STATUS.md)                              | Archiv ist Austauschpaket, kein fortschreibbares Backup; v1/v2 können geprüft, aber nicht als neues Projekt importiert werden                        |
| M3-05 · §3 F09, §6 V.1                    | PNG/JPEG/WebP importieren, anzeigen, Tags/Metadaten, ersetzen/löschen, mehrere eigenständige Varianten       | **N** im dokumentierten CRUD-Korpus · [KQ-068](native-m3-coverage.md): native PNG-Anlage, Metadaten/Tags, WebP-Ersatz, separate JPEG-Variante, bestätigte GUI-Löschung, Datei-/Referenzprüfung und Neustart bestanden.                                                         | Mehrere eigenständige Varianten; kein Elternmodell oder Variantenvergleich. Allgemeine assistive Abnahme bleibt getrennt.                            |
| M3-06 · §4, §6 V.1, §8 M3, §10 Referenzen | Ein Asset an mehrere Szenen und Story-Bibel-/Ereignisziele; stabile IDs und intakte Backlinks                | **N** im dokumentierten Korpus · Nativ umbenannte Figur/Szene; Charakterbild an zwei Szenen, Figur und Ereignis. Zwei Story-Bibel-Rückverweise und sämtliche 102 Assetlinks nach Neustart, Recovery und Neuimporten verglichen. [Nachweis](native-m3-acceptance.md).           | Allgemeine Medien-CRUD-/assistive Vollabnahme bleibt M3-05 bzw. Querschnitt.                                                                         |
| M3-07 · §10 Speicherintegrität            | 100 Szenen und 50 verlinkte Bilder nach Neustart und Sicherung konsistent                                    | **N** funktionaler Gesamtlauf · [M3-G01 bestanden](native-m3-acceptance.md): 100 Szenen/50 dekodierbare Originale, Neustart, tatsächliche Backup-/Recovery-Implementierung und native Recovery-App; unabhängige SQLite-/FK-/Hash-Prüfung.                                      | Unabhängiger Desktop-Netzverkehrsnachweis und assistive Vollabnahme bleiben offen; Export-Großauswahl wird über Rust aufgebaut.                      |
| M3-08 · §10 Speicherintegrität            | Fehlende und hashveränderte Originale/fehlende Thumbnails sichtbar melden                                    | **N** im dokumentierten Korpus · Drei getrennte Dateimanipulationen, native Medienansicht und korrigierte Meldung „Bilddatei oder Vorschau nicht verfügbar“; Rust erkennt alle drei. [Nachweis](native-m3-acceptance.md).                                                      | Die Meldung unterscheidet die drei Ursachen nicht einzeln; keine Behauptung einer solchen Diagnose.                                                  |
| M3-09 · §8 M3, §10 Speicherintegrität     | Backup/Recovery für Daten und Hash-Assets, ohne Quelle zu verändern                                          | **N** im dokumentierten Korpus · Tatsächliche SQLite-/Asset-Sicherung und Recovery via testexklusivem Rust-Helfer, Recovery-App separat geöffnet; Inventar, 50 echte Originale/Thumbnails, Quelle und bytegleiche Sicherung geprüft. [Nachweis](native-m3-acceptance.md).      | Kein vorhandener Backup-UI-Button behauptet; breitere Plattform-/Recovery-Varianten gesondert prüfen.                                                |
| M3-10 · §3 F09, §6 V.1, §8 M3             | Visuelle Profile mit Kernmerkmalen/Ausschlüssen/kanonischem Asset, projektweite Stilpresets                  | **N** im dokumentierten CRUD-Korpus · [KQ-068](native-m3-coverage.md): Profil-/Stil-Anlage und Bearbeitung, manuell bestätigter kanonischer Bildwechsel, GUI-Löschung und Neustart; Bildverweis wird beim Assetlöschen korrekt geleert.                                        | KQ-067 ergänzt Recovery; KQ-068 belegt Erhaltung von Figur und anderem Asset. Keine allgemeine assistive Gesamtabnahme.                              |
| M3-11 · §6 V.1, Übergang M4               | Editierbarer Bildprompt aus expliziten Quellen, Speichern und UTF-8-Export; Offline-Capabilities/Jobverträge | **A** · [Promptrepository](../../crates/storage/src/prompt_repository.rs), [Prompt-E2E](../../apps/desktop/e2e/prompt-builder.spec.ts), [ADR 0015](../architecture/decisions/0015-model-specific-image-capabilities-and-job-contracts.md)                                      | Gehört überwiegend M4; Export-/Backupreferenzen sind M3-relevant. Keine aktive Bildgenerierung oder native Vollabnahme daraus ableiten               |
| M3-12 · §4, §7, §8 M3                     | Snapshot-Neuimport erhält referenzielle Integrität, Profile und Promptquellen                                | **N** im dokumentierten Korpus · Zwei aktuelle native Neuimporte plus historischer Neuimport erhalten alle fachlichen IDs/Quellen/Referenzen bei neuen Projekt-UUIDs; vollständiger Snapshotinventarvergleich und unabhängige FK-Prüfung. [Nachweis](native-m3-acceptance.md). | Kein beliebiger Fremdarchiv-/Versionsimport und keine v1/v2-Projektneuanlage.                                                                        |

## Querschnitt und offene Entscheidungen

| ID / Quelle                   | Bewertung                       | Beleg und verbleibende Grenze                                                                                                                                                                                                                                                                                                                                                                    |
| ----------------------------- | ------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Q-01 · §1, §10 Offline        | **T** für den Gesamtumfang      | Native Bildexporte und eingebettete Vorlage benötigen kein Netz; Rust-Tests und lokale Quellen belegen Einzelpfade. KQ-067 benötigt keinen Provider; kopiertes HTML lädt 100 Bildvorkommen in unabhängigem Offline-Kontext ohne externe Requests. KQ-068: ETW findet öffentlichen WebView2-Verkehr; Q-01 nicht bestanden, KQ-069 / #134 offen                                                    |
| Q-02 · §5, §10 KI-Datenschutz | **T** für den Gesamtumfang      | Lokale Pfade/Schlüssel bleiben Rust; [Threat Model](../architecture/threat-model.md), [Cloudgrenze](../architecture/decisions/0010-cloud-provider-boundary-and-os-credentials.md), Provider-Tests. KQ-068 findet öffentlichen WebView2-Verkehr ohne Provideraufruf; Q-02 nicht bestanden, #134 offen. Keine Uploadzuordnung aus Transportmetadaten; echte Cloud-Liveabnahme bleibt separat offen |
| Q-03 · §10 Export             | **N/A** für definierte Profile  | Rollen/Formatierungen, PNG-/RGB-Nachweise, SHA-256 und Maskierung gemäß F08-Zeilen. Vollständige Drehbuch-/assistive Abnahme fehlt; Unicode-Grenzen sichtbar beibehalten                                                                                                                                                                                                                         |
| Q-04 · §10 Timeline           | **T**, angrenzender M2-Nachweis | [Timeline-E2E](../../apps/desktop/e2e/timelines.spec.ts) und [Strukturrepository](../../crates/storage/src/structure_repository.rs) belegen zwei Zeitebenen; KQ-067 bestätigt native Ereignis-/Asset-/Szenenlinks und beide Zeitebenen nach Recovery. Die vollständige M2-Timeline-Abnahme bleibt separat. Kein Kalender-/Zeitzonenversprechen                                                   |
| D-01 · §11 Projektpaket       | **N/A**, Umfang entschieden     | SQLite-Projektordner plus `.kraqel`-Snapshotarchiv existieren; Backup bleibt separat zweiteilig. Kein offener Bedarf, nochmals ein Archivformat zu erfinden                                                                                                                                                                                                                                      |
| D-02 · §11 DOCX-Roundtrip     | **N**, begrenzt entschieden     | [DOCX v1](../data-format/DOCX_ROUNDTRIP.md) und native Abnahme; keine allgemeine Office-Bearbeitung oder Bilder zugesagt                                                                                                                                                                                                                                                                         |
| D-03 · §11 Medieneinbettung   | **N**, begrenzt entschieden     | [Bildvertrag](../data-format/ILLUSTRATED_MANUSCRIPT.md), ADR 0029–0032 und drei native Abnahmen; vier Ausgaben am Szenenende                                                                                                                                                                                                                                                                     |
| D-04 · §11 Drehbuch           | **O** für Implementierung       | [Fachlich freigegebener Vertrag](../specifications/SCREENPLAY_FORMAT_CONTRACT_DRAFT.md) liegt vor; Produktentscheid am 15. November 2026 erfolgt; Implementierung und Funktionsabnahme fehlen                                                                                                                                                                                                    |
| X-01 · §1, §6 V.2, M6         | **X** für 0.1                   | Bildgenerierung, Kartenmodell, FDX/Fountain-Integration und Storyboard sind keine hier nachgewiesenen Funktionen. FDX/Fountain sind Projektvorschlagsausschlüsse, keine wörtliche Pflichtenheftregel                                                                                                                                                                                             |
| X-02 · §8 M5                  | **O**, bewusst später           | Installerstart/Persistenz auf Zweitrechner/Clean-VM bleiben Releaseabnahme. Das verhindert keine Dokumentationsarbeit, darf aber nicht als bestanden gelten                                                                                                                                                                                                                                      |

## M3-G01: Prüfplan und durchgeführter funktionaler Gesamtlauf

Der folgende Ablauf war der KQ-066-Abnahmeplan. KQ-067 ist im
[Abnahmeprotokoll](native-m3-acceptance.md) als funktionaler Gesamtlauf bestanden: Datenlebenszyklus
nativ, große Bildauswahl über die Rust-Implementierungen, separate begrenzte
Layout-/Offline-/assistive Aussagen. Die 100 Bildcheckboxen wurden nicht einzeln nativ bedient. Der
Plan allein bleibt kein Ergebnis. Eigene Test-Bundle-ID und temporäre Daten, keine privaten
Manuskripte. Quellcommit, Windows-/App-Build, Prüflaufdatum und verwendete Werkzeuge im späteren
Protokoll festhalten. Rust-Test-Seed oder überprüfbares synthetisches Archiv verwenden;
Manipulationen nur in Kopien nach dokumentiertem Fingerabdruck.

1. **Inventar:** 100 Szenen, 50 unterschiedliche tatsächlich dekodierbare PNG/JPEG/WebP-Originale
   unter allen bestehenden Paket-/Dateigrenzen, mehrere Kapitel, mindestens zwei Timelines,
   Story-Bibel, visuelle Profile und Stilpresets. Ein Charakterbild an mindestens zwei Szenen und
   denselben Charakter binden; mindestens ein Asset an ein Ereignis. IDs, Text-/Dateihashes,
   Dimensionen, Linkmengen, Profil-/Promptquellen und Dokumentarten als Sollinventar erfassen.
   Vorgeschlagene Drehbuchrollen sind keine Voraussetzung dieses bestehenden M3-Korpus.
2. **Bearbeitung und Neustart:** Charakter und Szenen umbenennen, speichern, Prozess beenden, neu
   starten. Texte/IDs/Hashes sowie Links in beide Richtungen prüfen; Umbenennung darf nur die
   beabsichtigten Titel verändern. Keine neue Assetidentität oder verlorene Profilreferenz.
3. **Sicherung/Recovery:** Echte SQLite-/Asset-Sicherung über den vorhandenen Rust-Mechanismus
   erzeugen, Quellprojekt unverändert lassen und in isoliertes Ziel wiederherstellen. Der
   Mechanismus kann einen expliziten QA-Helfer brauchen; es wird kein derzeit nicht vorhandener
   UI-Button behauptet. Neues Ziel in der nativen App öffnen und Sollinventar vollständig
   vergleichen.
4. **Git und Archiv:** Zwei ausdrücklich erzeugte Snapshotstände mit nachvollziehbarer Änderung
   prüfen/committen, aktuellen gesicherten Stand über den nativen Archivdialog ausgeben, zweimal als
   neue Kopie importieren. Projektidentitäten müssen verschieden sein; innerhalb jeder Kopie müssen
   sämtliche fachlichen Referenzen gültig und Inhalte/Assethashes erwartungsgemäß erhalten sein.
   Wiederherstellung eines älteren Git-Stands nur über einen ausdrücklich gesicherten Snapshot
   nachweisen; kein nicht vorhandenes Checkout-UI voraussetzen.
5. **Exporte:** Definierte Formatierung und explizite Bildauswahl für alle vier Formate prüfen; bei
   50 Bildern genug Vorkommen für Wiederverwendung vorsehen. Manifest, relative Pfade,
   DOCX-Parts/PDF-RGB, alte Generationen und Öffnung ohne Quelldatenpfad vergleichen. Layoutseiten
   vollständig prüfen oder klar auf getrennte bereits abgenommene synthetische Korpora begrenzen;
   keine universelle Layoutabnahme behaupten.
6. **Fehler und Grenzen:** In separaten Kopien Original entfernen, Thumbnail entfernen und
   Originalbytes verändern. Nach erneutem Öffnen sichtbare Unverfügbarkeit prüfen; Backup/Export
   dürfen fehlerhafte Originale nicht erfolgreich übernehmen. Ungültiges Archiv/DOCX, Übergröße und
   Dialogabbruch müssen ohne neue Teilprojekte/-pakete enden. Vorherige Pakete/Quelle per Hash
   unverändert nachweisen; Integritätsfehler nicht bloß per Screenshot vermuten.
7. **Offline/Bedienung:** Gesamten Ablauf ohne eingerichteten Provider durchführen; keine
   unerwarteten Netzaktivitäten akzeptieren. Fachansichten mit Tastatur, sichtbarem Fokus und
   Screenreader stichprobenartig einschließlich Fehleransagen prüfen. Ohne unabhängigen
   Netz-/assistiven Nachweis den jeweiligen Status weiterhin als teilweise/offen kennzeichnen.

Ein bestandenes M3-G01 allein nimmt noch nicht den vorgeschlagenen Drehbuchumfang ab. Für eine
vollständige Freigabe sind Scopeentscheidungen, alle einschlägigen Zeilen und ihre Restprüfungen
einzeln zu bestätigen. Fehlende Evidenz bleibt mit Datum, Verantwortlichkeit und konkretem nächsten
Nachweis offen; Teilnachweise werden nicht hochgestuft.

## Reihenfolge nach KQ-066

1. Fachentscheid erfolgt: gesamter Drehbuchvertrag einschließlich F08-Importrichtungen am 15.
   November 2026 ausdrücklich freigegeben; ADR 0033 angenommen.
2. Funktionaler M3-G01-Lauf in KQ-067 bestanden; irreführende Medienstatusmeldung korrigiert und
   nativ erneut geprüft. KQ-068 schließt die nativen CRUD-Restprüfungen einschließlich bestätigter
   Löschung und Neustart; Q-01/Q-02 bleiben wegen des gemessenen WebView2-Verkehrs in #134 offen.
   Die abschließende Lösch-/Export-Hörprobe ist ebenfalls bestätigt; weitere allgemeine assistive
   F08-Randfälle bleiben getrennt.
3. Nach Drehbuchfreigabe Modell-/Versions- und Schriftprototyp, dann Renderer und DB-01–DB-08;
   Status zeilenweise mit tatsächlichen Nachweisen aktualisieren.
4. Vor stabilem 0.1 die übrigen F01–F09-/M5- und Abhängigkeitsrisiken separat schließen.
