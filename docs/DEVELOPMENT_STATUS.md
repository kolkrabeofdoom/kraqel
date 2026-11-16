# Entwicklungsstatus

Auftrags-Fortschreibung, 16. November 2026: Forschungsadapter ergänzt dauerhaft verbrauchte IDs,
SQLite-Journal, exklusive Eigentümersperre und konservative Reconciliation vor neuen Importen.
Sieben hart beendete native Rust-Testprozesse prüfen die Veröffentlichungsgrenzen; sieben neue
aktive Integrationstests und sieben bisherige Adaptertests bestehen auf Windows. Erfolg und Abbruch
mit Replay im isolierten Desktopfenster bestätigt; Journal/Kopienzahl/Cleanup unabhängig geprüft.
[Vertrag](data-format/native-task-journal-v1.md) und
[Nachweis](testing/native-task-reconciliation.md). Kein integrierter Crash-/Neustartnachweis der
isolierten GUI, kein neuer ETW-Beleg und keine Produktfreigabe. Nächster Scope: authentisierter
isolierter Neustart mit stabiler Ablage und native Crash-/Reconciliation-Serie. Striktes Ziel,
offene Brokergrenzen und fehlende Produkt-Q-01/Q-02-Abnahme bleiben unverändert.

Assistive Fortschreibung, 16. November 2026: Forschungsseite führt den Fokus zu verfügbaren
Folgeaktionen und erhält bewusste Navigation. Native Erfolg-/Fehlerpfade einschließlich Replay mit
Sprachausgabe und Fokus bestätigt; DOM-Diagnose, SQLite/Kopienzahl und Cleanup unabhängig geprüft.
34 Browser-E2E bestanden. Frühere native Fokusabweichungen und die begrenzte lokale Diagnose sind
[gesondert dokumentiert](testing/native-live-assistive-focus.md). Keine assistive Vollabnahme und
kein neuer ETW-Beleg für diesen geänderten App-Build. Nächster Scope: dauerhafter
Auftragslebenszyklus/Crash-Reconciliation und externe Brokergrenzen. Striktes Ziel unverändert, #134
und Produkt-Q-01/Q-02 bleiben offen. Die folgenden Absätze bezeichnen frühere Buildstände.

Fensterende-Fortschreibung, 16. November 2026: Offenen nativen Dateidialog bei Appende gezielt
schließen und Auswahlthread begrenzt abwarten; final nativ mit Exitcodes 0, `CANCELLED`, beendetem
Thread und leerer Projektwurzel bestätigt. Teiltransfer nach 1.024 von 11.915 Bytes
sender-/empfängerseitig nachgewiesen und durch Fensterende verworfen. Cleanup in beiden Fällen
bestätigt. [Nachweise, Zwischenfehler und Buildgrenzen](testing/native-live-window-close.md).
Frischer ETW-Vergleich am unveränderten finalen App-/Controllerpaar: drei vollständige Bäume, 19
WebView2-Lebenszeiten, vier Messkontrollen, Nullverluste, 0 Ereignisse außerhalb Loopback und 22
getrennte Loopback-Disconnects. Referenzimport/Replay und beide Abbruchfälle unabhängig geprüft.
[Aggregate](testing/native-live-window-close-etw.json). Erfassung beendet, keine Binäränderung nach
Messung. Nächste Abnahme: assistive Fokus-/Fehlerprüfung, externe Brokergrenzen und dauerhafter
Auftrags-/Crash-Lebenszyklus. Striktes Ziel unverändert, #134 und Produkt-Q-01/Q-02 bleiben offen.

KQ-069-Fortschreibung 16. November: Replay nach Erfolg/Importfehler und Fensterende am
Publish-Haltepunkt nativ bestanden; überschriebenen Replay-Hinweis korrigiert und browserseitig
regressionsgeprüft. Drei vollständige Läufe desselben korrigierten Forschungsbuilds mit gültigem
ETW: vier Messkontrollen, Nullverluste, 0 Ereignisse außerhalb Loopback, 22 Loopback-Disconnects.
SQLite/Unicode, Kopienzahl, Replay und Cleanup unabhängig geprüft.
[Nachweis und exakte Binärgrenzen](testing/native-live-file-selection.md),
[Aggregate](testing/native-live-file-selection-etw.json). Nachträglicher Typalias für Clippy
gesondert ausgewiesen. Produkt-Q-01/Q-02 weiter nicht bestanden; #134 offen. Nächster Scope: Appende
bei offenem Dialog/Transfer, assistive Fokus-/Fehlerprüfung und externe Brokergrenzen.

Nachtrag KQ-069 Live-Auswahl, 16. November 2026: Forschungsmodus `ui-live` implementiert. Native
Auswahl aus laufender isolierter App und Dialogabbruch funktional bestätigt; Unicode/SQLite und
Cleanup unabhängig geprüft. [Nachweis](testing/native-live-file-selection.md) und
[Kanal v2](data-format/native-live-file-channel-v2.md). Kein neuer ETW-Lauf, keine Produktfreigabe.
Nächster Scope: Replay, Fehler-/Fokus-/Fensterendeprüfung und Netzabnahme. Striktes Datenschutzziel
unverändert; #134 bleibt offen.

Nachtrag 16. November 2026: KQ-069 / #134, native Vorabauswahl und Dialogabbruch auf dem
zurückgewechselten Windows-Rechner funktional bestanden. Unabhängig bestätigt: neue Projektkopie,
zwei Dokumente, Unicode/SQLite-Integrität/Fremdschlüssel, keine Stagingreste, null Jobprozesse und
entferntes Profil. [Nachweis und Grenzen](testing/native-ui-select-acceptance.md). Kein neuer
ETW-Lauf; Q-01/Q-02 für das Produkt bleiben offen. Nächster Scope: Auswahl aus einer laufenden App.

Stand: 15. November 2026

**Rechnerwechsel:** Die vollständige [Entwicklungsübergabe](WORKSTATION_HANDOFF.md) enthält den
Stand von PR #140, geprüfte native/ETW-Ergebnisse, Startanweisungen und den nächsten Scope in #134.
Code und synthetische Fixtures sind in Git; maschinenspezifische Builds/Rohdiagnosen bleiben lokal.
Fortsetzung nach geprüftem Merge auf aktualisiertem `main`, striktes Datenschutzziel unverändert.

**KQ-069 A / UI-/Tauri-Testanbindung und Einweg-Dateikanal:** PR #139 nach grüner Windows-/Linux-CI
und Dependency Review am Head `2ac11a2` zusammengeführt (`fadf0a6`). Separates Windows-Feature mit
echtem Forschungsfenster, parameterlosen Tauri-Befehlen, Origin-/Fensterprüfung und einmaliger
Vorabfreigabe per schreibgeschützter Datei/anonymer Einweg-Pipe. Keine Rendererpfade/Handles,
Hostordner- oder Internetfreigabe. Finaler Build nativ dreimal geprüft: Import/Wiederholung, Abbruch
und direktes Fensterschließen am Haltepunkt. Genau eine beziehungsweise null Kopien, keine
Stagingreste, Unicode/SQLite/Fremdschlüssel unabhängig geprüft; alle Jobs/Profile bereinigt.
Gültiges ETW desselben Builds mit drei vollständigen App-/WebView-Bäumen: vier Kontrollen,
Nullverluste, **0 Ereignisse außerhalb Loopback**, 26 Loopback-Disconnects separat. Keine
vollständige Nachfahren-/Brokerattestierung. `pnpm check`, beide Builds, Feature-Clippy, 29
Browser-E2E, 18 Helfer-/Transporttests, ein Origin-Test und 32 Python-Diagnosetests bestanden.
Normaler Debug-Build ohne Forschungsseite vorhanden.
[Protokoll](testing/native-tauri-file-channel.md),
[Kanalvertrag](data-format/native-file-channel-v1.md). Nur synthetische Fixture und manuelle
Bedienung durch den Nutzer, keine neue Auswahl-/Hörabnahme. #134 offen, Q-01/Q-02 für das Produkt
nicht bestanden, striktes Ziel unverändert. Nächster Scope: native Vorabauswahl/Abbruch prüfen,
danach Auswahl aus laufender App und dauerhafte Auftrags-/Crash-Reconciliation.

**KQ-069 A / typisierter Dateiauftrag und isolierter App-Dienst-Testadapter:** PR #138 mit grüner
Windows-/Linux-CI und Dependency Review am Head `1a233d1` zusammengeführt (`3f57291`). Separater
Rust-Testadapter mit `unsafe_code = forbid`, einmaliger gebundener ID, fester Snapshotoperation,
Ablauf-/Größenprüfung und bereinigten Antworten. Publish-Guard im bestehenden atomaren Importer:
Abbruch/Ablauf nach SQLite-Aufbau verwerfen den unveröffentlichten Zwischenstand. Sieben neue
Adaptertests und 13 Helfertests erfolgreich. Sieben native Importfälle plus Kopieregression im
AppContainer ohne Capabilities bestanden; unabhängige SQLite-/Unicode-/Hash-/Bestandsprüfung und
Bereinigung bestätigt. Gültiges ETW **desselben finalen Builds**: acht Controllerbäume, vier
Kontrollen, Nullverluste, vollständige Prozessenden, keine Netzereignisse.
[Protokoll](testing/native-isolated-app-import.md),
[Forschungsvertrag v1](data-format/native-file-task-v1.schema.json). **Headless-App-Dienste, kein
Tauri-/WebView-/GUI-Adapter integriert**; feste synthetische Fixture, keine erneute native Auswahl
oder assistive Abnahme. Normales Tauri unverändert unisoliert. Q-01/Q-02 weiter nicht bestanden,
#134 offen, striktes Datenschutzziel unverändert. Nächster Scope: UI-/Tauri-Testanbindung mit
sicherem Auftragskanal, nativer Auswahl, Abbruch-/Fehleransagen und Neustart-/Crash-Reconciliation;
danach übrige Formate, Broker-/Netz-/Packaginggrenzen. `pnpm check`, gezielter Abschlusslauf mit
sieben Adaptertests, normaler Tauri-Debug-Build, Clippy/rustfmt und 30 Python-Diagnosetests
erfolgreich; Fixture bytegleich reproduziert. Keine regulären App-Daten geöffnet.

**KQ-069 A / gezielter Datei-/Shell-Austausch:** PR #137 nach grüner Windows-/Linux-CI und
Dependency Review am Head `f5f1d35` zusammengeführt (`84b31f9`). Der getrennte Rust-Forschungshelfer
überträgt jetzt synthetische Dateien aus nativer Öffnen-/Speichern-Auswahl über genau zwei explizit
vererbte Dateihandles an einen AppContainer-Worker ohne Capabilities. Keine Paket-ACL auf dem
Hostordner, keine Parser im Host, keine neuen Netzfähigkeiten. Drei native Unicode-Kopien mit
unabhängig gleichem SHA-256, verweigertem Nachbardateizugriff und vollständiger Bereinigung;
Öffnen-Abbruch ohne Worker/Ausgabe. 13 Rust- und 30 Python-Tests prüfen unter anderem Grenzen,
Teilkopie-Bereinigung und Überschreibschutz. Gültige ETW-Messung des ersten Austauschbuilds: vier
Kontrollen, kein Verlust, vollständige Prozessenden, null Netzereignisse im Broker-/Workerbaum.
Finaler Build nach stärkerer Cleanup-/Berichtsprüfung nativ erneut geprüft, nicht erneut ETW-
gemessen. Externe COM-/Shell-Dienste und Produktintegration sind weiterhin offen.
[Protokoll und genaue Grenzen](testing/native-runtime-file-exchange.md). Das strikte Pflichtenheft
bleibt unverändert; Q-01/Q-02 für das Produkt nicht bestanden, #134 offen. Nächster Schritt:
typisierter einmaliger Dateiauftrag an einen isolierten App-Testadapter, danach konkreter Import mit
bestehender Rust-Validierung und atomarer Veröffentlichung; keine allgemeine Shell-Freigabe.

**KQ-069 A / technische Laufzeitisolation:** PR #136 ist mit vollständig grüner CI am Head `d4e2c64`
zusammengeführt (`caaa694`). Der Nutzer beauftragt Richtung A aus ADR 0034 und hält das strikte
Datenschutzziel ausdrücklich fest. Ein separater AppContainer-Forschungslauncher prüft Root-Tokens
vor Ausführung und gewährt null Capabilities. Drei gültig ETW-gemessene App-Läufe zeigen **0
Ereignisse außerhalb Loopback**, der explizit unisolierte Vergleich desselben Builds 201 öffentliche
WebView2-Ereignisse. Beide Gruppen: vier Kontrollen, kein Verlust, alle Prozessenden. 23
Tokenstichproben haben null Capabilities; keine lückenlose Nachfahrenattestierung behauptet.
Projektanlage, Unicode-Autosave, Neustart, HTML und PDF sind im synthetischen Korpus bestätigt. Der
Bild-Dateidialog öffnet im Container nicht; öffentliche UDP-/IPv6-, DNS-/Brokerpfade, Git/
Providergrenzen, Packaging und vollständige F08-/M3-Wiederholung fehlen. Socketversuch verweigert
korrekt seine Vollfreigabe. [Protokoll](testing/native-runtime-isolation.md) und
[Tool](../scripts/runtime-isolation/README.md). **Kein produktiver Launcher integriert**; normaler
Debug-Build wiederhergestellt. Q-01/Q-02 bleiben für das Produkt nicht bestanden, #134 bleibt offen.

**Historischer KQ-069-Diagnosestand vor Richtung A / Issue #134:** PR #135 ist nach grüner
Windows-/Linux-CI und Dependency Review am Head `202c5b2` zusammengeführt (`d685937`); Issue #133
geschlossen. Die lokale NetLog-Diagnose reproduziert schon ohne Projekte zwei
Konfigurations-GET-Starts und drei Komponentenupdate-POST-Starts. Korpus/HTML-Export sowie
Neustart/Archivimport/Änderung zeigen weitere Komponentenupdate-Starts. Keine Body- oder
vollständige Uploadprüfung. 27 Python-Tests prüfen ETW-/NetLog-Auswertung und isolierte
Vorbereitung. Nach erneuter UAC-Freigabe ist der ETW-Wiederholungslauf mit Dokument-CRUD,
Medienimport/-Metadaten, HTML-/Archivexport, Neuimport und drei vollständigen App-Prozessläufen
gültig: 4/4 Kontrollen, kein Verlust, alle Prozessenden; 256 öffentliche Transportereignisse
ausschließlich aus WebView2. Kein Upload-Inhaltsnachweis. `pnpm check` und Wiederherstellung des
normalen Debug-Builds erfolgreich. Unterstützte Maßnahmen sind
[bewertet](testing/native-m3-webview-network.md), keine Unterdrückung implementiert.
[ADR 0034](architecture/decisions/0034-webview-network-boundary-proposal.md) ist eine offene
Entscheidungsvorlage. Empfehlung: striktes Pflichtenheftziel beibehalten und technische Isolation
zunächst untersuchen. Q-01/Q-02 unverändert nicht bestanden; #134 bleibt offen.

**Fortschreibung KQ-068 / Issue #133:** PR #132 ist mit grüner Windows-/Linux-CI und Dependency
Review zusammengeführt (`c7629c3`); Issue #131 ist geschlossen. Die KQ-067-Aussagen bleiben
historische Belege. Neu: native Medien-/Profil-/Stil-CRUD-Teilprüfungen, zugängliche Status-/
Fehleransagen und eine unabhängige ETW-Messung. Sie findet öffentlichen Verkehr von
WebView2-Nachfahren ohne Provideraufruf: Q-01/Q-02 sind **nicht bestanden**.
[KQ-069 / #134](https://github.com/kolkrabeofdoom/kraqel/issues/134) klärt Ursache, unterstützte
Maßnahmen und die fachlich zu entscheidende Laufzeitgrenze. Keine Uploadbehauptung aus
Transportmetadaten; README/Startseite präzisieren die Telemetrieaussage.

[Protokoll](testing/native-m3-coverage.md): Nativer Medien-/Profil-/Stil-CRUD ist im dokumentierten
Korpus einschließlich der drei ausdrücklich freigegebenen GUI-Löschungen und vollständigem
Prozessneustart bestanden. B-Original/Thumbnail sind entfernt, der kanonische Verweis wurde vor
Profillöschung korrekt geleert. Bild A, seine Metadaten/Hashes/Dateien, Szenenlink, Dokumente und
Figur sind unverändert; SQLite-/FK-Prüfung fehlerfrei. Der Nutzer bestätigt den manuellen
kanonischen Bildwechsel und die Profil-/Stil-/Fehleransagen mit „alles gut“. Vier folgende
Bildexporte sind per Tastatur erzeugt und unabhängig nachgeprüft. Die letzte konkrete Hörfrage zu
Löschungen, Export-Bedienelementen und Erfolgen ist mit „Ja passt, alles gut“ bestätigt. KQ-068s
benannte native Prüfungen sind abgeschlossen; weitere allgemeine assistive F08-Randfälle bleiben
offen. PR #135 hat auf Stand `291788b` vollständig grüne CI; dieser Stand wurde erfolgreich neu
gebündelt und für Neustart/Export verwendet. 163 Rust-/53 TypeScript-Tests, 27 Browser-E2E und zwölf
ETW-Auswertungstests bestanden. Formate/Schema unverändert. M3-05/M3-10 sind **N** im Korpus;
Q-01/Q-02 und fehlende Drehbuchimplementierung verhindern weiterhin die Gesamtfreigabe.

Code-Referenz: KQ-055 (Issue #107), KQ-056 (Issue #109), KQ-058/059 (Issues #113/#115; isolierte
Prototypen), KQ-060 (Issue #117; nativer Roundtrip-Kern), KQ-061 (Issue #119; App und neue
Projektkopie), KQ-062 (Issue #121; freigegebener Medienexport-Vertrag), KQ-063 (Issue #123;
Rust-Kern und bebildertes Markdown/HTML), KQ-064 (Issue #125; gewöhnlicher DOCX-Bildwriter), KQ-065
(Issue #127; PDF-Schrift-/Unicode-Profil und Szenenbilder), KQ-066 (Issue #129; Drehbuchvertrag und
vollständige F08-/M3-Kriterienmatrix), KQ-067 (Issue #131; nativer M3-Gesamtlauf)

Planungsstand: KQ-057 konkretisiert den DOCX-Roundtrip-Vertragsentwurf. KQ-058 zeigt den Verlust
verschachtelter Marker in LibreOffice. Der beauftragte Folgeversuch KQ-059 bestätigt flache
Absatzmarker mit einem erhaltenen Inventar in Dokumenteigenschaften: Word und LibreOffice bewahren
den geprüften Korpus einschließlich echter Textänderungen. ADR 0026 nimmt diese Repräsentation als
Grundlage an. KQ-060 ergänzt das gemeinsame kanonische Manuskriptmodell und den nativen
Reader/Writer mit eigenem v1-Profil, vollständiger Paketvalidierung und semantischem Selbstimport.
KQ-061 ergänzt die getrennten App-Aktionen und den atomaren Import als neue Projektkopie. KQ-055,
KQ-056, Vertragsentwurf und beide Prototypen sind nach grüner Windows-/Linux-CI in `main`
zusammengeführt (PR #108, #110, #112, #114, #116). KQ-060 ist nach dem plattformneutralen
Fixturevergleich ebenfalls mit grüner Windows-/Linux-CI in `main` zusammengeführt (PR #118). KQ-061
ist ebenfalls mit grüner Windows-/Linux-CI und Dependency Review in `main` zusammengeführt (PR
#120). KQ-062 ist nach grüner CI in PR #122 zusammengeführt; der gesamte Vertrag ist fachlich
freigegeben. KQ-063 implementiert den Rust-Kern und den eigenen Modus „Mit Szenenbildern“ für
Markdown/HTML mit Manifest v1, metadatenfreien PNGs und atomaren neuen Paketgenerationen. PR #124
ist nach grüner Windows-/Linux-CI und Dependency Review zusammengeführt (b04cac3). KQ-064 ergänzt
den gewöhnlichen DOCX-Bildwriter; native Windows-App, Word 16.0.20527.20196 und LibreOffice 26.8.3.1
sind im dokumentierten Umfang abgenommen. PR #126 ist nach grüner CI zusammengeführt (c8708b8).
KQ-065 ergänzt vier eingebettete Noto-Sans-Schnitte, explizite Unicode-Prüfung und metrischen
Umbruch für Text-/Bild-PDF sowie weiße PNG-Nachweise mit nativer RGB-/Maßprüfung. PR #128 ist mit
grüner Windows-/Linux-CI und Dependency Review zusammengeführt (4679c89); Issue #127 ist
geschlossen. KQ-066 dokumentiert Drehbuchvertrag und F08-/M3-Abnahmematrix. PR #130 ist mit grüner
Windows-/Linux-CI und Dependency Review zusammengeführt (58dea4b); Issue #129 ist geschlossen. Der
gesamte Vertrag einschließlich Format-/Importrichtungen ist am 15. November 2026 ausdrücklich
fachlich freigegeben. KQ-067 führt den funktionalen nativen M3-Gesamtlauf durch;
Drehbuchimplementierung, unabhängige Desktop-Netzverkehrsprüfung und assistive Vollabnahme fehlen
weiterhin. Die Matrix ist vollständig erfasst, der Gesamtumfang bleibt teilweise abgenommen.

Produktstatus: frühes Entwicklungsfundament, nicht produktionsreif

Dieses Dokument trennt verifizierte Implementierungen von Planung. Maßgeblich sind Code, Tests, CI,
angenommene ADRs und das technische Pflichtenheft. Typen, Enums und leere Modulgerüste gelten nicht
als vollständige Fachfunktion.

## Status nach Pflichtenheft-Stufen

| Stufe        | Pflichtenheft-Ziel                                     | Verifizierter Stand                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           |
| ------------ | ------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **M0**       | Monorepo, Tauri, Datenbank, CI, Berechtigungsmodell    | **Technisches Fundament umgesetzt:** Monorepo, Tauri-Shell, CI, minimale Capability, Command-Verträge, SQLite/FTS5 sowie sicherer Projektlebenszyklus mit Backup und Recovery vorhanden.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      |
| **M1**       | Editor, Szenenbaum, Autosave, Backup                   | **Technischer Mindestumfang samt Browser-E2E umgesetzt:** Projekt-/Kapitel-/Szenenoberfläche, Tiptap, Undo/Redo, Wortzählung, Fokusmodus, konfliktgeprüfter Autosave sowie sicheres Umbenennen, Umordnen, Löschen und SQLite-Backup vorhanden. KQ-032 automatisiert Tastatur-, Fokus- und Axe-Prüfungen; KQ-033 bestätigt den Kern manuell mit Windows-Sprachausgabe in der Tauri-WebView.                                                                                                                                                                                                                                                                                                                                                                                    |
| **M2**       | Story-Bibel, Plotboard, Timeline                       | **Technischer Mindestumfang samt Browser-E2E umgesetzt:** Story-Bibel, persistentes Plotboard sowie parallele relative/absolute Timelines mit Szenen- und Entitätslinks sind in getrennten Chromium-Abläufen geprüft. KQ-033 bestätigt Story-Bibel-Beziehungen und Rückverweise per Windows-Sprachausgabe; Plotboard und Timelines wurden assistiv stichprobenartig geprüft.                                                                                                                                                                                                                                                                                                                                                                                                  |
| **M3**       | Git, Import/Export, Medienbibliothek, visuelle Profile | **Teilweise umgesetzt:** Snapshots, lokales Git, vier Manuskriptexporte, Medienbibliothek sowie visuelle Profile und Stilvorlagen sind im browserseitigen Kernablauf geprüft. KQ-038/KQ-050 ergänzen Snapshot-v3-Neuimport, Beispielvorlage und freie Archivauswahl. KQ-051 ergänzt den nativen `.kraqel`-Export; KQ-054 den begrenzten UTF-8-/Markdown-Import als neue Szene. KQ-055 ergänzt den DOCX-Szenenimport; KQ-060 den nativen Roundtrip-Kern. KQ-061 ergänzt App-Aktionen und atomare neue Projektkopie. Die vollständige F08-/M3-Matrix trennt belegte Teilfunktionen, fehlende Drehbuchimplementierung und gemessenen offenen WebView2-Verkehr und assistive Restprüfungen bei bestandenem funktionalen nativen Gesamtlauf und nativem Medien-/Profil-/Stil-CRUD. |
| **M4**       | Text-KI, Kontinuität, Prompt-Entwürfe, Bildverträge    | **Technischer 0.1-Umfang weitgehend umgesetzt:** Textprotokolle, browserseitig geprüfte Offline-Kontinuität und Prompt-Entwürfe sowie Bild-Capability- und Jobverträge vorhanden. Echte Providerabnahme, KI-Kontinuitätshinweise und Bildgenerierung fehlen.                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  |
| **M5**       | Windows-Release 0.1, Tests, Dokumentation              | **Alpha 4 veröffentlicht:** Debug- und Tauri-Smoke-Tests, isolierter NSIS-Testinstaller, Accessibility-Nachweise sowie ein getrennter unsignierter x64-NSIS-Releasepfad sind vorhanden. KQ-045 belegt Build, GitHub-Rückdownload und SHA-256 von `v0.1.0-alpha.4`; Release Candidate und öffentlicher Rückdownload wurden isoliert installiert und deinstalliert. MSI, Signierung, Updater und ARM64 sind ausgeschlossen; SmartScreen, Anwendungsstart und Persistenz auf Zweitrechner/Clean-VM fehlen.                                                                                                                                                                                                                                                                       |
| **M6 / 0.2** | Bildgenerierung, Referenzkonsistenz, Kartenmodell      | **Nicht begonnen und ausdrücklich außerhalb von 0.1.**                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        |

Die funktionalen Anforderungen F01 bis F09 sind damit noch nicht abgenommen. Vorhandene Verträge und
Gerüste sind technische Vorarbeit, keine fertigen Funktionen.

## Verifiziert implementiert und getestet

### Repository und Qualität

- pnpm- und Cargo-Monorepo mit gelockten Abhängigkeiten.
- Tauri-2-/React-/TypeScript-Desktop-Shell mit statischer, responsiver Startoberfläche.
- KQ-056 ergänzt die ausdrücklich ausgewählte lithografische KolkrabeofDoom-Autorenvignette als
  lokales transparentes PNG neben der Startseiten-H1. Die Figur wird responsiv verkleinert und
  dekorativ ohne zusätzliche Screenreader-Ansage eingebettet. Herkunft, Grenzen und visuelle Prüfung
  bei 1180 bis 320 Pixel Breite stehen unter
  [`design/kolkrabe-wordmark.md`](design/kolkrabe-wordmark.md).
- KQ-046 trennt Projekttitel und Rückkehraktion von einer fachlich gruppierten, responsiven
  Bereichsnavigation; der Breitbildvertrag und die Gruppenbezeichnungen sind browserseitig geprüft
  und die echte Windows-Tauri-Ansicht wurde manuell abgenommen.
- Minimale Tauri-Berechtigung `core:default`, definierte CSP und Windows-Fensterkonfiguration.
- GitHub-Actions-CI unter `windows-latest` und `ubuntu-latest`.
- KQ-047 aktiviert über den offiziellen GitHub-Endpunkt Schwachstellenwarnungen und den dafür
  erforderlichen Dependency Graph. Der bestehende Dependency-Review-Job wird in Pull Requests
  dadurch als echte Prüfung ausführbar und bestand erstmals in PR #91; automatische
  Dependabot-Sicherheitsupdates bleiben weiterhin deaktiviert.
- KQ-048 entfernt `docx-rs`, aktualisiert die einzige verbleibende ZIP-Implementierung auf den
  gelockten Stand 9.5.1 und hebt die Rust-Mindestversion aus dokumentierten Sicherheitsgründen auf
  1.90 an. `zip` selbst benötigt 1.88, die gelockte Tauri-Reihe bereits 1.90. Snapshotimport und der
  neue begrenzte OOXML-Writer sind gezielt getestet; Microsoft Word öffnete und konvertierte das
  erzeugte DOCX, dessen gerenderte Seite visuell geprüft wurde. Dependency Review sowie Ubuntu- und
  Windows-CI bestanden in PR #93. GitHubs SBOM für `main` enthält nur noch `zip` 9.5.1 und den
  sicheren Versionsbereich ab 9.0.5; die fünf alten Alert-Objekte warten noch auf die asynchrone
  Neuberechnung.
- KQ-050 ergänzt eine native Ein-Datei-Auswahl für frei gewählte `.kraqel`- und ZIP-Archive. Pfad
  und Bytes bleiben in Rust; reguläre Datei, unveränderte Größe und 128-MiB-Grenze werden vor der
  bestehenden Snapshot-v3-Prüfung kontrolliert. Abbruch bleibt neutral. Rust-Tests belegen gültige,
  beschädigte, zu große und ungeeignete Quellen; TypeScript-Vertragstest und Browser-E2E belegen
  leere IPC-Nutzlast, neue Projektkopie, Abbruch und Accessibility. Der echte native Windows-Dialog
  wurde am 12. November 2026 mit gültigem Beispielarchiv, Abbruch und ungültiger Datei manuell
  geprüft; Import, neutraler Abbruch und bereinigte Fehlermeldung verhielten sich wie vorgesehen.
- KQ-051 erzeugt vor jedem nativen `.kraqel`-Export einen aktuellen, validierten Snapshot v3, packt
  ihn deterministisch innerhalb der Importgrenzen und veröffentlicht das Archiv über eine temporäre
  Geschwisterdatei. Zielpfad und Bytes bleiben in Rust. Rust belegt Byte-Stabilität und
  erfolgreichen Neuimport; Command- und Browsertests belegen datenarme Antwort und neutralen
  Dialogabbruch. Der echte native Windows-Speicherdialog und ein erfolgreicher Export wurden am 12.
  November 2026 manuell bestätigt.
- KQ-052 ergänzt die endgültige Projektlöschung auf der Startseite. Die UI verlangt einen zweiten
  konkreten Klick, der Command ein Bestätigungsflag und der `ProjectManager` übereinstimmende UUIDs
  in Verzeichnis und Datenbank. Rust-, Command-, Browser- und Accessibility-Tests decken den Ablauf
  ab. Erster Bestätigungsschritt und endgültige Löschung einer entbehrlichen Kopie wurden am 12.
  November 2026 in der echten Windows-App manuell bestätigt; eine Wiederherstellung nach bestätigter
  Löschung ist absichtlich nicht implementiert.
- KQ-053 behebt den im echten Windows-Test gefundenen Git-Fehler `Filename too long` bei
  SHA-256-Assets unter dem langen App-Datenpfad. Jeder kontrollierte Git-Prozess erhält nun lokal
  `-c core.longpaths=true`; globale und systemweite Konfiguration bleiben unangetastet. Ein
  Windows-Regressionslauf mit einem Snapshotpfad über 260 Zeichen sowie der reale Snapshot- und
  Archivexport sind erfolgreich.
- KQ-054 ergänzt den nativen Import von `.txt`, `.md` und `.markdown` bis 2 MiB als neue Szene in
  einem gewählten Kapitel. Pfad und Bytes bleiben in Rust; reguläre Datei, unveränderte Länge, UTF-8
  und NUL-Freiheit werden geprüft. TXT-Absätze sowie Markdown-Überschriften, Zitate, Listen und
  Codeblöcke werden nach Editor-Dokument v1 übertragen; unbekannte Syntax bleibt sichtbar. Szene,
  Inhalt und Klartextindex entstehen transaktional. Rust-, Command- und Browsertests decken Import,
  editierbaren Titel, ungültige Inhalte und neutralen Abbruch ab. Die native Windows-Prüfung am 14.
  November 2026 bestätigt TXT/Markdown, Abbruch, ungültiges UTF-8, Übergröße, Umbenennung und
  Wiederöffnen. Verfahren und Grenzen stehen in
  [`testing/native-text-import-acceptance.md`](testing/native-text-import-acceptance.md).
- KQ-055 ergänzt den begrenzten DOCX-Import als neue Szene. ADR 0024 und
  [`data-format/DOCX_IMPORT.md`](data-format/DOCX_IMPORT.md) definieren unterstützte Textsemantik,
  ZIP-/XML-Grenzen und die ausdrückliche Ablehnung unbekannter Inhaltsknoten. Pfad und Archivbytes
  bleiben in Rust; Szene und Inhalt entstehen gemeinsam in der vorhandenen Storage-Transaktion. Rust
  prüft Styles, Namespaces, Ressourcenlimits und unveränderte Daten bei Ablehnung; eine unabhängig
  mit python-docx erzeugte Fixture ergänzt die Minimalpakete. IPC- und Browserprüfungen decken
  Dateiauswahlvertrag, Hinweis, Formatübernahme, Abbruch und bereinigte Fehler ab. Die native
  Windows-Prüfung bestätigt gültige Datei, Abbruch, Tabelle und beschädigte Datei; Verfahren und
  Aussagegrenzen stehen in
  [`testing/native-docx-import-acceptance.md`](testing/native-docx-import-acceptance.md). Der
  Exportwriter und der strukturerhaltende Roundtrip sind nicht Teil dieses Inkrements.
- Playwright mit Chromium und einem testexklusiven Browser-Command-Adapter für M1, Story-Bibel,
  Plotboard, Timelines, Medien, visuelle Profile, Kontinuität, Bildprompt-Builder, Versionen und
  Export sowie die eingebettete Beispielvorlage; die CI führt fachlich getrennte Abläufe unter
  Windows und Linux aus.
- Qualitätskette: Prettier, rustfmt, ESLint, Clippy mit `-D warnings`, TypeScript, Vitest,
  Rust-Tests sowie Frontend- und Workspace-Builds.
- Separater Windows-Tauri-Smoke-Test mit realem WebView, versionierten IPC-Commands, Rust und
  isolierter SQLite-Persistenz; er ist bewusst nicht Bestandteil der plattformübergreifenden
  Browser-E2E.
- Isolierter Windows-NSIS-Testinstaller mit eigenem Produktnamen, eigener Bundle-ID und temporären
  Programm-/App-Datenpfaden; lokale Installation, HKCU-Registrierung, echter Persistenzlauf und
  stille Deinstallation sind automatisiert geprüft.
- Getrennter Windows-x64-Releasepfad mit unsigniertem Current-User-NSIS-Paket, eingebettetem
  WebView2-Offlineinstaller, abgelehnten Downgrades, SHA-256 und maschinenlesbarem Buildmanifest.
  KQ-036 erzeugte aus Merge-Commit `5155ed5282cf` den 212,98 MiB großen RC, lud genau drei Assets in
  einen privaten GitHub-Entwurf und prüfte den Rückdownload auf denselben SHA-256. Das Setup wurde
  isoliert installiert, registriert und deinstalliert; bestehende reguläre App-Daten blieben
  unverändert. KQ-037 veröffentlichte exakt diese Assets als Prerelease `v0.1.0-alpha.1`. KQ-039
  veröffentlicht den geprüften KQ-038-Stand mit eingebettetem Beispielprojekt als `v0.1.0-alpha.2`.
  KQ-041 veröffentlicht den responsiven KQ-040-Stand als `v0.1.0-alpha.3`. KQ-045 veröffentlicht den
  manuell abgenommenen Dubletten-Merge als `v0.1.0-alpha.4`. MSI, Updater, ARM64 und Signierung
  werden nicht erzeugt.
- GPL-3.0-or-later, Beitrags-, Verhaltens- und Sicherheitsdokumentation vorhanden.

### Domäne und IPC-Grenze (KQ-002)

- Kanonische UUID-Typen für Projekte und Entitäten in Rust und TypeScript.
- Portable Projektformat-Version `1` in den Domänenmodulen; interner SQLite-Migrationsstand `6`.
- Versionierter lokaler Command-Vertrag v1 mit `contractVersion`, `requestId` und Nutzlast.
- Stabile Fehlercodes `INVALID_REQUEST`, `UNSUPPORTED_CONTRACT_VERSION` und `INTERNAL`.
- UI ersetzt fremde IPC-Fehlermeldungen durch bereinigte Texte; Eingaben und Pfade werden nicht
  zurückgespiegelt.
- Tauri-Commands `application_status` und `validate_project_name` samt TypeScript-Clientfunktionen.
- Tests für Serialisierungsform, Vertragsversion, UUID-/Schemaprüfung, Projektnamen und
  Fehlerbereinigung.

### Lokale Datenbank (KQ-003)

- `rusqlite` 0.39 mit gebündeltem SQLite und FTS5; das Workspace-Minimum ist Rust 1.90.
- Initialschema v1 für `project`, `document`, `entity`, `relation` und `link` mit Fremdschlüsseln,
  Indizes, JSON-Prüfungen und zentraler Schema-Version aus `kraqel-domain`.
- Vorwärtsmigration v2 für `storyline`, `plot_card`, `timeline`, `event` und deren Linktabellen.
- Atomare Vorwärtsmigration über `PRAGMA user_version`; unbekannte Zukunftsversionen werden ohne
  Schemaänderung abgelehnt.
- Verwaltete Verbindungen aktivieren Fremdschlüssel, Busy-Timeout und für Dateien WAL.
- FTS5-Index für Dokumenttitel und Klartext mit Triggern für Einfügen, Ändern und Löschen.
- Tests für leere und befüllte Datenbanken, Wiederöffnung, Fremdschlüssel, FTS-Synchronisation und
  Ablehnung eines Zukunftsschemas.

### Projektlebenszyklus, Backup und Recovery (KQ-004)

- `ProjectManager` erstellt und öffnet Projekte ausschließlich unter einer kanonischen lokalen
  Wurzel; absolute Pfade, Traversal und Symlink-Komponenten werden abgelehnt.
- Projekterstellung und Recovery erfolgen in temporären Geschwisterverzeichnissen und werden erst
  nach erfolgreicher Prüfung atomar veröffentlicht.
- Konsistente SQLite-Sicherungen verwenden die Online-Backup-API, `quick_check`, Schema-Prüfung und
  Dateisynchronisation; bestehende Sicherungen werden nicht überschrieben.
- Wiederherstellung liest die Sicherung ausschließlich lesend und erzeugt immer ein neues Ziel.
- Lokale Projekte lassen sich erst nach zweiter sichtbarer Bestätigung löschen. Vor der rekursiven
  Entfernung müssen UUID-Verzeichnis und gespeicherte Projektkennung übereinstimmen.
- Datenarme Fehler unterscheiden fehlende Projekte, Datenbanken und Sicherungen sowie beschädigte
  Datenbanken.
- Tests decken Windows-taugliche Leerzeichen-/Unicodepfade, lange Verzeichnisnamen, Traversal,
  Symlinks, fehlende und beschädigte Dateien, Backup-Konsistenz und unveränderte Recovery-Quellen
  ab.

### Editor, Szenenbaum und Autosave (KQ-005/KQ-019)

- Tiptap 3/ProseMirror mit versioniertem Editor-Dokumentvertrag v1 und grundlegender
  Rich-Text-Formatierung.
- Lokale Projektübersicht sowie Projekt-, Kapitel- und Szenenerstellung über versionierte
  Tauri-Commands; Projekte liegen vorerst im lokalen App-Datenverzeichnis.
- Stabil sortierter Kapitel-/Szenenbaum; Undo/Redo-Verlauf wird durch eine eigene Editorinstanz je
  Szene begrenzt.
- Entprelltes Autosave nach 700 Millisekunden mit sichtbaren Zuständen, serialisierten
  Schreibvorgängen und optimistischer Revision. Konflikte überschreiben keinen neueren Stand.
- Rust leitet Klartext aus Tiptap-JSON ab und aktualisiert Inhalt, Revision sowie FTS-Index
  transaktional.
- Kapitel und Szenen lassen sich inline umbenennen und per zugänglichen Hoch-/Runter-Aktionen unter
  ihren Geschwistern umordnen. Titel-Compare-and-swap und vollständige alte/neue Reihenfolgen
  verhindern stilles Überschreiben eines neueren Baums.
- Zweistufiges Löschen bindet Titel und Inhaltsrevision, bei Kapiteln zusätzlich die vollständig
  angezeigte Kindliste. SQLite kaskadiert Szenen sowie Story-, Plotboard-, Timeline-, Medien- und
  Promptreferenzen; die UI entfernt Auswahl und lokale Baumknoten konsistent.
- Automatisierte Tests für Editorvertrag, Baumordnung, Autosave, IPC-Roundtrip, Konflikte,
  Löschkaskaden sowie Backup/Recovery mit 100 Szenen. KQ-032 prüft zusätzlich Projektanlage,
  Bereichsnavigation, Editor, Formatierungszustände, Fokusmodus, Speicherstatus und
  Story-Bibel-Suche per Tastatur. KQ-033 bestätigt diesen Kern zusätzlich mit echter
  Windows-Sprachausgabe in der isolierten Tauri-WebView.
- Ein Playwright-Ablauf prüft Projektanlage, Schreiben, entprelltes Autosave, Wiederöffnen,
  Kapitel-/Szenenerstellung, Umbenennen, zugängliches Umordnen und zweistufiges Löschen in Chromium.
  Dabei wurde ein StrictMode-Lebenszyklusfehler behoben, der den Autosave-Controller im
  Entwicklungsmodus nach dem probeweisen Effekt-Cleanup dauerhaft stilllegte.
- KQ-024 korrigiert zusätzlich die geladene Szenenrevision nach erfolgreichem Autosave. Ohne diese
  Synchronisation wurde der Editor nach einem Ansichtswechsel mit einer alten Revision neu montiert
  und der nächste Schreibversuch korrekt als Konflikt abgelehnt.
- KQ-029 baut eine eigene Windows-Smoke-Konfiguration ohne Bundle, startet die echte Tauri-Anwendung
  über `tauri-driver`, legt ein künstliches Projekt an und bestätigt den per Autosave gespeicherten
  Szenentext nach einem vollständigen Neustart. Testdaten und WebView-Profile liegen ausschließlich
  unter `$TEMP/kraqel-tauri-smoke` und werden nach dem Lauf entfernt.

### Story-Bibel (KQ-006)

- Gemeinsame Rust-/TypeScript-Verträge für Figuren, Orte, Fraktionen, Gegenstände, freie
  Eigenschaften, gerichtete Relationen und Szenen-Rückverweise.
- Transaktionales `StoryBibleRepository` auf den vorhandenen Tabellen `entity`, `relation` und
  `link`; Namen und freie Eigenschaften werden begrenzt und in Rust validiert.
- Einträge lassen sich anlegen, suchen, nach Art filtern, bearbeiten und löschen. Die Suche prüft
  Namen, Eigenschaftsschlüssel und -werte Unicode-bewusst, ist derzeit aber nicht FTS-indexiert.
- Beziehungen bewahren Quelle und Ziel; die Detailansicht trennt eingehende und ausgehende Kanten.
  Selbstbeziehungen sind im Vertrag v1 nicht erlaubt. Löschkaskaden entfernen verwaiste Kanten.
- Szenenlinks werden idempotent angelegt und als Rückverweise angezeigt; aus einem Rückverweis kann
  wieder in die betreffende Szene gewechselt werden.
- KQ-043 führt ein ausdrücklich gewähltes Dublett derselben Art atomar in den geöffneten
  Zieleintrag. Ziel-UUID, -Name und vorhandene Eigenschaftswerte bleiben maßgeblich; fehlende
  Eigenschaften werden ergänzt. Die Oberfläche zeigt abweichende Werte vor der Bestätigung.
- Dokument-, Relations-, Ereignis-, POV- und Medienreferenzen wechseln vor dem Löschen der Quelle
  zum Ziel. Visuelle Profile und Promptreferenzen wandern ebenfalls mit; bei zwei Profilen gelten
  Zielvorrang für abweichende Merkmale und das kanonische Bild sowie die Vereinigung der
  Ausschlüsse. Direkte Quelle-Ziel-Relationen werden als sonst unzulässige Selbstrelationen
  entfernt.
- Zugängliche React-Oberfläche mit beschrifteten Formularen, sichtbaren Fokuszuständen,
  Tastaturbedienung sowie textlich erkennbarer Beziehungsrichtung. KQ-032 korrigiert den
  Ergebniszähler auf WCAG-AA-Kontrast.
- Automatisierte Repository-Tests decken Suche, Eigenschaften, Richtungstreue, Löschregeln,
  Rückverweise und Backup/Recovery ab. IPC- und TypeScript-Tests prüfen den vertikalen Vertrag. Ein
  Playwright-Ablauf prüft zusätzlich Erstellung, Eigenschaftssuche, gerichtete Beziehungen und
  Szenen-Rücksprung in Chromium. Dabei wurden die Listenaktualisierung nach dem Anlegen und der
  asynchrone Zugriff auf React-Ereignisse in Eigenschaftsfeldern korrigiert. KQ-033 bestätigt Suche,
  Ergebnisanzahl, gerichtete Beziehungen und Szenen-Rückverweis zusätzlich mit Windows-Sprachausgabe
  in der echten Tauri-WebView. Ein zweiter KQ-043-Ablauf prüft Auswahl, Konfliktvorschau,
  Bestätigung, Eigenschaftsübernahme sowie erhaltene Beziehung und Rückverweis.

### Plotboard und parallele Timelines (KQ-007)

- SQLite-Migration v1→v2 ergänzt Handlungsstränge, Szenenkarten, Zeitebenen, Ereignisse sowie
  Szenen-/Entitätslinks; eine befüllte v1-Datenbank bleibt erhalten.
- Technischer Datenbankstand (`PRAGMA user_version = 2`) und portables Projektformat
  (`project.schema_version = 1`) sind nun explizit getrennt.
- Jede Szene besitzt eine persistente Karte mit stabiler Reihenfolge, Status, optionalem
  Handlungsstrang und optionaler POV-Figur.
- Karten lassen sich nativ per Drag-and-drop sowie gleichwertig über beschriftete Tasten neu ordnen.
- Mehrere relative oder absolute Zeitebenen sind möglich. Relative Ereignisse verwenden eine
  Sequenz; absolute Ereignisse verlangen einen expliziten Zeitwert.
- Ereignisse lassen sich idempotent mit Szenen und Story-Bibel-Einträgen verknüpfen. Szenenlinks
  führen zurück ins Manuskript.
- Repository-Tests decken Reihenfolge, Status, POV, Zeitmodell, Links, Migration und Backup/Recovery
  ab. Alte v1-Sicherungen werden nur in der neuen Zielkopie migriert; die Quelle bleibt unverändert.
  TypeScript-Vertragstests prüfen die vollständige Kartenreihenfolge.
- Zwei getrennte Playwright-Abläufe prüfen zusätzlich Handlungsstrang, Status, POV,
  tastaturbedienbare Karten- und Ereignisreihenfolge, relative und absolute Zeitmodelle,
  Szenen-/Entitätslinks sowie den Rücksprung ins Manuskript. Der Testadapter bildet hierfür nur die
  bereits vorhandenen Command-Verträge ab; Tauri- und Screenreader-Abnahme bleiben separat offen.

### Nativer DOCX Manuskriptaustausch (KQ-060)

- Gemeinsames kanonisches Manuskriptmodell, strenge Editorattribute und inverser Szenenadapter.
- Eigenes v1-Profil nach ADR 0027 mit flachen Markern, gebundenem Strukturinventar, vollständiger
  Paket-/Beziehungsprüfung, Style-Auflösung und überprüfter Nummerierung.
- Deterministischer Writer mit nativem semantischem Selbstimport vor atomarer Dateiveröffentlichung.
  Reader und Writer laufen ohne Python, Office oder Netzwerk in Rust.
- Vier native v1-Fixtures belegen unveränderte Semantik nach echten Word-/LibreOffice-Speichergängen
  sowie eine echte Word-Textänderung. Alle vier Seiten wurden gerendert und visuell geprüft.
- Für KQ-060 bestanden `pnpm check` mit 138 Rust- und 48 TypeScript-Tests sowie 41
  Prototypprüfungen. Das Export-Crate enthält 31 reguläre Tests und drei standardmäßig ignorierte
  visuelle/Office-QA-Helfer.
- KQ-061 ergänzt getrennte App-Aktionen, native Auswahl und neue Projektkopien mit vollständig neuen
  Projekt-/Dokument-UUIDs. Ein transaktionaler Import in Staging wird vor Veröffentlichung komplett
  aus SQLite zurückgelesen und semantisch verglichen. Quelldatei und vorhandene Projekte bleiben
  bestehen.
- Migration 5→6 erhält Daten, Rowids, Referenzen und FTS-/Löschtrigger und erlaubt importierte leere
  Dokumenttitel. Die Oberfläche verwendet dafür eine reine Ersatzanzeige „Ohne Titel“.
- Lokal bestehen `pnpm check` mit 144 Rust- und 49 TypeScript-Tests sowie alle 25 Browser-E2E. Echte
  native Auswahl, Abbruch, ungültige Datei, Import einer Word-Fixture, eigener Export und dessen
  Wiederimport als zweite unabhängige Kopie sowie Umbenennen eines leeren Kapitels sind bestätigt.
  Details und Aussagegrenzen stehen im
  [Abnahmeprotokoll](testing/native-docx-roundtrip-acceptance.md).

### Szenenbilder in Markdown/HTML (KQ-063)

- Explizite Auswahl ausschließlich direkt verknüpfter Szenenbilder, Reihenfolge, Alttext und
  optionale Beschriftung in der Exportansicht; keine gespeicherten Presets und keine automatische
  Auswahl.
- Rust prüft konsistenten Projektstand, Revisionen, Links, Quellhashes, Formate und
  Ressourcenbudgets. Statische PNG/JPEG/WebP werden orientiert, ohne Hochskalierung auf 2400 Pixel
  begrenzt und als metadatenfreie RGBA-PNGs erzeugt. Animationen und ungültige Quellen brechen den
  Auftrag ab.
- Jedes Format erhält ein neues vollständiges Paket mit relativen Bildverweisen und Manifest v1
  unter `exports/mit-bildern/<format>-<uuid>/`. Alte Pakete bleiben auch bei Fehlern bestehen.
- Automatisierter Korpus: 100 Szenen/50 Originale, stabile Hashes, Reihenfolge, Maskierung,
  Metadatenentfernung, Decodergrenzen und Konfliktabbruch ohne Teilveröffentlichung.
- Native Windows- und portable Ausgabeprüfung: siehe
  [Abnahmeprotokoll](testing/native-illustrated-export-acceptance.md).
- Lokal bestehen `pnpm check` mit 154 Rust- und 50 TypeScript-Tests sowie alle 26 Browser-E2E. Vier
  zusätzliche Rust-Helfer für explizit aktivierte visuelle/native QA sind regulär ignoriert.
- Die damaligen DOCX-/PDF-Folgestufen sind inzwischen in KQ-064/KQ-065 umgesetzt; die vollständige
  F08-/M3-Abnahme bleibt offen. SQLite bleibt auf Version 6.

### Szenenbilder in gewöhnlichem DOCX (KQ-064)

- Gemeinsamer Rust-Medienexportkern, explizite App-Auswahl und Command v1 akzeptieren zusätzlich
  `docx`; keine Migration oder neue Bibliothek.
- PNGs liegen dedupliziert intern unter `word/media/<hash>.png`; Manifest v1 referenziert diese
  Parts. Nur Inline-Zeichnungen mit eindeutigen Kennungen, Alttexten und proportionalen Maßen,
  optionale Beschriftungen als Klartext mit echten Umbrüchen; keine externen Bildbeziehungen.
- Native ZIP-Selbstprüfung aller Parts vor atomarer Veröffentlichung. Alte Pakete bleiben erhalten.
  Der bisherige DOCX-Text-Roundtrip wird nicht erweitert; sein Reader weist bebildertes DOCX ab.
- `pnpm check`: 156 Rust- und 52 TypeScript-Tests; fünf explizite QA-Helfer regulär ignoriert. Alle
  26 Browser-E2E einschließlich DOCX-Aktion und Axe bestehen. Der gezielt erweiterte
  Wiederverwendungstest besteht zusätzlich mit Clippy.
- Word- und LibreOffice-Abnahme bestanden: Öffnung, Speicherung, identische Texte/Alttexte und
  Bildpixel sowie visuelle Prüfung aller drei Seiten. Details im
  [Abnahmeprotokoll](testing/native-illustrated-docx-acceptance.md).
- PDF-Schrift-/Unicode-Nachweis und bebildertes PDF folgen inzwischen mit KQ-065; F08/M3 bleiben
  teilweise abgenommen.

### PDF mit eingebetteter Schrift und Szenenbildern (KQ-065)

- Vier festgeschriebene Noto-Sans-2.008-Schnitte werden unverändert offline mit SIL-OFL-Lizenz,
  Copyright und Hashes geliefert und vollständig in Text-/Bild-PDF eingebettet.
- NFC und Glyphenprüfung vor Erzeugung; Lateinisch/Griechisch/Kyrillisch im dokumentierten Umfang.
  Fehlende Glyphen, verbleibende kombinierende Zeichen und komplexes/Bidi-Shaping werden abgewiesen.
  Keine universelle Unicode- oder PDF/UA-Zusage.
- A4, echte Glyphenvorschübe/Oberlängen und proportionaler Bildabschnitt am Szenenende mit
  sichtbarer Beschreibung und Beschriftung. Kein Beschnitt, keine Hochskalierung oder Überlagerung.
- Weiß abgeflachte PNGs begleiten das PDF; der native Rückleser prüft RGB-Hash **und Abmessungen**,
  RGB/8-Bit-Profil und Bildkennung vor atomarer Veröffentlichung einer neuen Generation.
- Automatisiert: 100 Szenen/50 Bilder, stabile Manifest-/PNG-Hashes, Unicode-/Schriftprüfung,
  metrischer Umbruch, Konfliktabbruch, Erhalt früherer Pakete und gleiche RGB-Bytes mit anderen
  Maßen.
- Der synthetische Dreiseiten-Korpus ist vollständig gerendert und visuell geprüft. Details und
  Grenzen stehen im [Abnahmeprotokoll](testing/native-illustrated-pdf-acceptance.md).
- Lokal bestehen `pnpm check` mit 161 Rust- und 53 TypeScript-Tests sowie alle 26 Browser-E2E. Sechs
  explizite QA-Helfer sind regulär ignoriert. Der native Windows-Debug-Build besteht.
- SQLite bleibt auf Version 6; Snapshot v3 und DOCX-Text-Roundtrip bleiben unverändert. F08/M3
  bleiben im übergreifenden Umfang teilweise offen.

### Drehbuchvertrag und F08-/M3-Abnahmematrix (KQ-066)

- [Fachlich freigegebener Vertrag](specifications/SCREENPLAY_FORMAT_CONTRACT_DRAFT.md): sechs
  explizite Absatzrollen, Bearbeitung, konkretes A4-Profil, Schriftprüfung, Formatgrenzen,
  Versionsfolge und acht spätere Abnahmefälle. Nach ausdrücklicher Nutzerfreigabe am 15. November
  2026 ist [ADR 0033](architecture/decisions/0033-screenplay-contract-proposal.md) fachlich
  angenommen; die Funktion ist noch nicht implementiert.
- [Vollständige Kriterienmatrix](testing/F08_M3_ACCEPTANCE_MATRIX.md): Pflichtenheft §§1/3/4/6–11
  gegen Code, Tests, native Protokolle, Einschränkungen und Restprüfungen abgeglichen. Vollständig
  bedeutet hier Erfassung aller einschlägigen Kriterien, nicht bestandene F08-/M3-Vollabnahme.
- 100-Szenen-Backup und 100-Szenen-/50-Bilder-Exporte sind Einzelbelege. Der kombinierte native
  M3-G01-Lauf mit echten Bildern, App-Neustart, Sicherung/Recovery, Snapshot-Neuimport, Backlinks,
  Umbenennung und sichtbaren Assetfehlern ist inzwischen durch KQ-067 im dokumentierten funktionalen
  Umfang bestanden; Offline-Netzverkehrs-/assistive Grenzen bleiben ausdrücklich offen.
- Reine Dokumentationsstufe: keine neue SQLite-/Editor-/Command-/Snapshot-/DOCX-Version, keine neue
  Schrift oder Laufzeitfunktion. Die 161 Rust-/53 TS-/26 E2E aus PR #128 sind bestehende Evidenz;
  diese Stufe behauptet keinen neuen manuellen Lauf.

### Funktionaler nativer M3-Gesamtlauf (KQ-067)

- Der [Abnahmenachweis](testing/native-m3-acceptance.md) verbindet 100 Szenen, 50 echte
  PNG-/JPEG-/WebP-Bilder, 102 Assetlinks, Story-Bibel, zwei Timelines, Profil, Stil und
  Promptquellen.
- Echte Windows-App: zwei Git-Commits, Szenen-/Figurenumbenennung, vollständiger Prozessneustart,
  nativer Archivexport, zwei aktuelle Neuimporte und zusätzliche Wiederherstellung des älteren
  Stands.
- Rust-QA-Helfer nutzt tatsächliche Backup-/Recovery- und Command-Implementierungen; separat
  gestartete Recovery-App zeigt erhaltene Inhalte, kanonisches Profilbild, Stil und Ereignislinks.
- Vollständiger Inventarvergleich sowie unabhängige SQLite-/FK-/Hash-/Decoderprüfung in fünf
  intakten Projektständen bestanden. Alle vier großen Bildausgaben mit 50 Bildern/100 Vorkommen
  unabhängig rückgelesen; PDF hat 34 Seiten und 100 passende RGB-Objekte, HTML lädt alle Bilder aus
  einer kopierten Ausgabe im Offline-Kontext. Große Auswahl ist über Rust geprüft, nicht als 100
  einzeln betätigte GUI-Checkboxen ausgegeben.
- Fehlende/veränderte Originale und fehlende Thumbnails sichtbar geprüft; Backup/Exportfehler
  erhalten frühere Pakete. Native Dateidialoge: Abbruch, ungültiges/übergroßes Archiv und ungültiges
  DOCX ohne zusätzliche Projekte. Die irreführende Meldung „Vorschaudatei fehlt“ ist korrigiert zu
  „Bilddatei oder Vorschau nicht verfügbar“ und im neuen nativen Build bestätigt.
- Abschluss: `pnpm check` mit 162 Rust-/53 TypeScript-Tests, sieben regulär ignorierten expliziten
  QA-Helfern, alle 26 Browser-E2E und abschließender regulärer Windows-Debug-Build bestanden.
- Ein regulärer Integrationstest und ein explizit ignorierter, auf eigene Temp-Wurzeln begrenzter
  QA-Schrittetreiber ergänzen die Reproduktion; beide fehlen im ausgelieferten App-Code.
- Keine unabhängige Desktop-Netzverkehrsmessung, neue Hörprüfung oder native Medien-CRUD-
  Vollabnahme. Die Groß-PDF-Seiten sind nicht erneut vollständig visuell geprüft; die Layoutaussage
  bleibt bei den getrennten KQ-064/065-Korpora. F08/M3 sind weiterhin teilweise abgenommen.

### Manuskriptexport (KQ-009)

- Gemeinsame, streng validierte Zwischenrepräsentation für Projekt, Kapitel, Szenen, Absätze,
  Überschriften, Fett/Kursiv, harte Umbrüche, Listen und Blockzitate.
- Unbekannte Tiptap-Knoten und -Markierungen brechen den Export mit bereinigtem Fehler ab; Inhalte
  werden nicht still verworfen.
- Markdown und eigenständiges HTML in UTF-8, DOCX im Letter-Seitenformat sowie PDF in A4 werden aus
  derselben fachlichen Repräsentation erzeugt.
- Exporte entstehen atomar und ausschließlich unter `exports/manuskript.<format>` im lokalen
  Projektordner. Symlinks und nicht reguläre Zielobjekte werden abgelehnt.
- Versionierter Tauri-Command und zugängliche React-Ansicht **Export** bieten alle vier Formate an
  und melden Dateiname sowie Größe, aber keinen absoluten lokalen Pfad. KQ-046 ergänzt einen Button,
  der den festen projektgebundenen Exportordner über den Betriebssystem-Dateimanager öffnet.
- Der neue Backendpfad akzeptiert nur die Projekt-ID, kanonisiert das verwaltete Projektverzeichnis,
  erzeugt ausschließlich dessen Kind `exports`, lehnt Symlinks und Nicht-Verzeichnisse ab und gibt
  den absoluten Pfad nicht an die WebView zurück.
- Automatisierte Tests prüfen alle Formate, Textformatierung und die Ablehnung unbekannter Knoten.
  DOCX und PDF wurden zusätzlich als Seiten gerendert und visuell geprüft; für DOCX war unter
  Windows wegen fehlendem LibreOffice eine Word-basierte Konvertierung nötig. KQ-048 verschärft den
  DOCX-Test auf die vereinbarten OOXML-Bestandteile, Inhalte und Auszeichnungen und wiederholt die
  Word-basierte Konvertierung samt visueller Seitenprüfung nach dem internen Writer-Umbau.
- KQ-025 ergänzt einen Playwright-Ablauf, der ein künstliches Manuskript speichert und anschließend
  Formatwahl, Dateiname und lokalisierte positive Bytezahl für Markdown, HTML, DOCX und PDF gegen
  den testexklusiven Command-Adapter prüft. Echte Dateien, Rendering, Tauri, Dateisystem und
  Screenreader bleiben getrennt abzunehmen.
- Nicht Teil dieses Inkrements: Import, DOCX-Roundtrip, Medien, Fußnoten, Tabellen, Links und
  beliebige Tiptap-Erweiterungen.

### Lokale Text-KI mit Ollama (KQ-010)

- Providerunabhängige asynchrone Rust-Grenze für Beschreibung, Modellabfrage und geordnete
  Streamereignisse; der erste Adapter spricht ausschließlich einen lokalen Ollama-Dienst an.
- Die feste Loopback-Adresse, deaktivierte Systemproxies und Redirects verhindern, dass die lokale
  Konfiguration versehentlich zu einem Cloudzugriff wird. Anfrage, einzelne JSONL-Zeilen und
  Gesamtausgabe sind begrenzt und Antworten werden strikt validiert.
- Vor jedem Lauf zeigt die Oberfläche den gespeicherten Szenenklartext. Die Freigabe erfolgt per
  Checkbox für genau Dokument-ID und Revision; Rust lädt den Text erneut und lehnt einen inzwischen
  geänderten Stand als Konflikt ab.
- Modellliste und Textentwurf laufen über typisierte Tauri-Commands beziehungsweise einen Kanal. Der
  Stream trägt Provenienz, bleibt verwerfbarer Entwurf und verändert oder speichert niemals
  automatisch den Szenentext.
- Automatisierte Tests prüfen Modellreihenfolge, JSON-Vertrag, geordnetes Streaming, den tatsächlich
  übertragenen Kontext sowie die Sperre ohne Freigabe und für Nicht-Loopback-Adressen.
- In der Entwicklungsumgebung ist kein laufender Ollama-Dienst vorhanden. Ein echter Modelllauf und
  die manuelle Browser-/Screenreader-Abnahme sind daher noch nicht nachgewiesen. Cloudprovider und
  Schlüsselspeicher gehören ausdrücklich nicht zu KQ-010.

### Cloud-Textprovider und Schlüsselschutz (KQ-011)

- Rust-Adapter für OpenAI Responses, Anthropic Messages, Google Gemini und Mistral Chat Completions
  bilden deren Modelllisten und gestreamte Textdeltas auf denselben providerunabhängigen Vertrag ab.
- Produktionsendpunkte sind feste offizielle HTTPS-Adressen. Redirects, Umgebungsproxies und frei
  konfigurierbare Produktionsziele sind deaktiviert; lokale HTTP-Endpunkte existieren nur in Tests.
- API-Schlüssel werden je Provider im nativen Betriebssystem-Schlüsselspeicher abgelegt. Die UI kann
  Werte setzen oder löschen und nur deren Vorhandensein prüfen; gespeicherte Werte werden niemals
  zurückgegeben oder in Projektdateien geschrieben.
- Jede Vorschau zeigt Provider, Modell, Arbeitsauftrag, Szenentext und übertragene Metadaten. Ein
  kurzlebiger Einmalbeleg bindet diese Angaben an Dokumentrevision und Projekt und wird vor dem
  Request verbraucht. Änderungen oder Wiederverwendung werden in Rust abgelehnt.
- Modelllisten werden nur nach ausdrücklicher Bedienhandlung abgerufen. Streams bleiben
  größenbegrenzt, erhalten eine Herkunftsangabe und verändern den kanonischen Text nicht.
- Protokolltests prüfen alle vier Antwortformate, Modelllisten und die Sperre ohne Freigabe mit
  lokalen Testservern. Mangels realer API-Schlüssel wurden keine kostenpflichtigen oder echten
  Cloudrequests ausgeführt; Anbieterkompatibilität und Betriebssystemdialoge benötigen manuelle
  Abnahme.

### Deterministische Kontinuitätsprüfung und Vorschlagsreview (KQ-012)

- `kraqel-continuity` wertet eine unveränderliche Faktenprojektion aus Dokumenten, Story-Bibel,
  Plotboard und Timelines vollständig offline und ohne Provider aus.
- Vier konservative Regeln erkennen doppelte normalisierte Story-Bibel-Namen, ungültige POV-Arten,
  exakte nicht verknüpfte Namensnennungen und Mehrfachzuordnungen einer Szene auf derselben
  Timeline. Nur der strukturell ungültige POV wird als Konflikt bezeichnet; mehrdeutige, aber
  möglicherweise legitime Modellierungen bleiben Prüfhinweise oder Informationen.
- Jeder Befund enthält eine stabile Regelkennung, einen Schweregrad und konkrete Quellenanker.
  Dokumentfundstellen verwenden Unicode-Zeichenpositionen; Szenenquellen lassen sich aus der
  Reviewansicht im Manuskript öffnen.
- Die React-Ansicht **Kontinuität** löst die lokale Prüfung ausdrücklich aus und gruppiert die
  reproduzierbaren Befunde. Sie verändert keine Projektdaten.
- Ein erzeugter Textentwurf kann gegen genau die zuvor freigegebene Szenenrevision als Zeilendiff
  geprüft werden. Rust lädt den Szenenstand erneut und lehnt veraltete Revisionen ab. Der Command
  besitzt keinen Schreibpfad, die Oberfläche keine automatische Übernahme.
- Unit-, Repository-, IPC- und TypeScript-Vertragstests prüfen Regelklassifikation, Quellenanker,
  Read-only-Verhalten und Revisionskonflikte. KI-basierte Kontinuitätshinweise, persistente
  Prüfläufe und ein interaktiver Teilübernahme-Editor sind nicht implementiert.
- KQ-028 ergänzt einen eigenständigen Playwright-Ablauf: Eine künstliche, nicht verknüpfte
  Namensnennung erzeugt nach ausdrücklichem Start einen Informationsbefund mit stabiler Regelkennung
  und Unicode-Zeichenanker; dessen Schaltfläche lädt die betreffende Szene im Manuskript. Der
  Browser-Adapter bildet dafür nur die bestehende Regel `story-bible.unlinked-mention` ab. Tauri,
  Rust, KI-Hinweise und schreibende Änderungen sind nicht Teil dieses Browsertests.

### Lokale Medienbibliothek (KQ-013)

- SQLite-Migration 2→3 ergänzt `media_asset`, `asset_link`, Indizes und Löschtrigger. Assets
  speichern relative Original-/Thumbnailpfade, MIME-Typ, SHA-256, Bytegröße, Abmessungen, Herkunft,
  Titel und Schlagwörter.
- Die Rust-Grenze akzeptiert ausschließlich inhaltlich dekodierbare PNG-, JPEG- und WebP-Dateien bis
  25 MiB, 16.384 Pixel je Achse und 40 Millionen Pixel. Signatur und Endung müssen übereinstimmen.
- Projektdateinamen bestehen aus Asset-UUID und Hashpräfix. Gelieferte Namen werden nicht als Pfade
  verwendet. Thumbnails entstehen lokal als begrenzte PNG-Dateien.
- Die Ansicht **Medien** unterstützt Import, Vorschau, Metadaten und Tags, inhaltsgeprüften Ersatz,
  zweistufig bestätigtes Löschen sowie Links zu Szenen, Story-Bibel-Einträgen und Ereignissen.
  Fehlende oder hash-veränderte Originale beziehungsweise fehlende Thumbnails werden sichtbar als
  nicht verfügbar gemeldet.
- Links verwenden stabile IDs und bleiben bei Umbenennungen erhalten. Rust prüft jedes Linkziel;
  Trigger entfernen Links automatisch, wenn das Zielobjekt gelöscht wird.
- Lokale SQLite-Backups erhalten ein `.assets`-Begleitverzeichnis. Die Datenbanksicherung bestimmt
  die kopierten Dateien; Originale werden vor Veröffentlichung und Wiederherstellung gegen SHA-256
  geprüft.
- Portable Snapshots kopieren Originale als `assets/<sha256>.<endung>`, führen Metadaten und Links
  im Manifest und prüfen Hash sowie exakte Dateimenge. Byte-Stabilität bleibt erhalten.
- Tests decken Inhaltsprüfung, Pfadsicherheit, Hash/Thumbnail, Migration 2→3, Linkkaskaden,
  Backup/Recovery, manipulierte Snapshotassets sowie den vertikalen Importvertrag ab.
- KQ-026 ergänzt einen Playwright-Ablauf für künstliche PNG-Auswahl, Titel und Tags, sichtbare
  Vorschau-, MIME-, Maß- und Prüfsummendaten, Verknüpfung mit der aktiven Szene und Rücksprung ins
  Manuskript. Der testexklusive Adapter dekodiert keine Bilder, berechnet keine kryptografischen
  Hashes und schreibt keine Dateien; diese Eigenschaften bleiben durch Rust-Tests beziehungsweise
  separate Tauri- und Plattformabnahme belegt.
- KQ-040 begrenzt die intrinsische Breite von Datei-Inputs und allen weiteren Karteninhalten auf die
  verfügbare Grid-Spur. Buttons dürfen bei Bedarf mehrzeilig werden; Metadaten wechseln
  platzabhängig auf weniger Spalten. Der Medien-Playwright-Ablauf prüft bei 360 Pixeln Breite, dass
  Karte, Inhalt und direkte Bedienelemente keinen horizontalen Überlauf erzeugen.

### Visuelle Profile und Stilvorlagen (KQ-014)

- SQLite-Migration 3→4 ergänzt genau ein `visual_profile` je Story-Bibel-Eintrag sowie projektweite
  `style_preset`-Datensätze.
- Profile speichern begrenzte, strukturierte Kernmerkmale, explizite Ausschlüsse und optional genau
  ein kanonisches Medienasset aus demselben Projekt. Löschen des Assets leert nur die Referenz;
  Löschen der Entität entfernt das Profil.
- Stilvorlagen besitzen stabile UUIDs, einen projektweit eindeutigen Namen, strukturierte visuelle
  Regeln und mindestens eine der vier unterstützten Entitätsarten als Geltungsbereich.
- Rust normalisiert und begrenzt Namen, Schlüssel, Werte, Listen und JSON-Gesamtgrößen. Sämtliche
  Entitäts-, Projekt- und Assetreferenzen werden an der autoritativen Grenze geprüft.
- Die Ansicht **Visuelle Profile** unterstützt Erstellen, Bearbeiten und zweistufig bestätigtes
  Löschen von Profilen und Stilvorlagen; kanonische Bilder werden aus der lokalen Medienbibliothek
  gewählt.
- Backup/Recovery übernimmt die Daten mit der SQLite-Sicherung. Deterministische Snapshots führen
  sie in `visual-profiles.json` und prüfen Struktur sowie Entitäts- und Assetreferenzen.
- Repository-, Migrations-, Backup-, Snapshot-, IPC- und TypeScript-Vertragstests decken den
  vertikalen Ablauf ab.
- KQ-027 ergänzt einen eigenständigen Playwright-Ablauf für Profilmerkmale, Ausschlüsse,
  strukturierte Stilregeln, mehrere Geltungsbereiche und das Wiederladen nach Ansichtswechsel gegen
  den vorhandenen testexklusiven Command-Adapter. Kanonische Referenzbilder, Tauri und Screenreader
  bleiben separat abzunehmen.
- Es existiert kein Netzwerkpfad für Profile oder Stilvorlagen. Bildprovider sind ausdrücklich nicht
  Bestandteil dieses Inkrements.

### Lokaler, editierbarer Bildprompt-Builder (KQ-015)

- Die Ansicht **Bildprompts** verlangt die explizite Auswahl einer Szene, mindestens eines visuellen
  Profils und optional einer für alle gewählten Entitätsarten passenden Stilvorlage.
- Rust lädt Szenentext, Story-Bibel-Eigenschaften, Kernmerkmale, Ausschlüsse, kanonische Asset-UUIDs
  und Stilregeln erneut und setzt sie deterministisch zu einem editierbaren Text zusammen.
  Szenentext ist als unvertrauenswürdiger Inhalt gekennzeichnet.
- SQLite-Migration 4→5 speichert Name, bearbeiteten Prompttext, Szenen-UUID und Quellrevision,
  Profil-UUIDs sowie die optionale Stilvorlage. Die erste Speicherung lehnt eine zwischenzeitlich
  geänderte Szene als Konflikt ab; spätere Textbearbeitung ändert Quellen nicht still.
- Erstellen, Listen, Bearbeiten, zweistufiges Löschen sowie UTF-8/LF-Export unter
  `exports/prompts/bildprompt-<uuid>.txt` sind lokal implementiert. Es gibt keinen Provider-,
  Netzwerk- oder automatischen Übernahmepfad.
- Backup/Recovery erhält Entwürfe. Snapshot v3 exportiert `prompt-drafts.json`, prüft Dokument-,
  Profil- und Stilreferenzen und liest weiterhin v1 und v2.
- Repository-, Migration-, Backup-, Snapshot-, Export-, Rust-IPC- und TypeScript-Commandtests sind
  grün. Ein Playwright-Ablauf legt künstliche Story-Bibel-, Profil- und Stilquellen über die UI an
  und prüft deterministische Zusammensetzung, vollständige Bearbeitung, lokale Speicherung,
  Wiederladen sowie die sichtbare UTF-8/LF-Exportbestätigung. Tauri-, Dateisystem-, WebView- und
  Screenreader-Abnahme bleiben separat offen.

### Begrenzte Hilfsfunktionen und Verträge

- `asset-service`: produktiver, begrenzter Rasterbildimport mit SHA-256, Thumbnail, sicherer
  Veröffentlichung und erneuter Dateiprüfung.
- `editor`: versionierter, generischer Snapshot-Vertrag.
- `ai-contracts`: Capability-, Provider-, Streaming- und explizite Kontextfreigabetypen für Text
  sowie gespiegelte Bild-Registry- und Generation-Job-Verträge.
- `visual-contracts`: Medien-, Profil-, Stilvorlagen- und Promptverträge; keine Providerlogik.
- `continuity`: ausführbare Offline-Regeln und Vorschlags-Diff; keine KI-Analyse oder
  Laufpersistenz.

### Bild-Capability-Registry und Generation-Job-Verträge (KQ-016)

- Die lokale Registry ordnet `image.generate`, `image.edit`, `image.reference` und
  `image.map_render` einzelnen Modellen zu, statt Fähigkeiten pauschal für einen Provider
  anzunehmen.
- Modelleinträge beschreiben Prompt- und Referenzlimits, unterstützte Breiten und Höhen,
  Ergebniszahl und Seed-Unterstützung. Leere Installationen und verständlich begründete inaktive
  Provider sind zulässig; doppelte oder widersprüchliche Einträge werden abgelehnt.
- Der versionierte Vertrag v1 bindet Job, Projekt, Provider, Modell, Fähigkeit, Prompt-Entwurf,
  exakt freigegebenen Prompttext, Referenzasset-UUIDs und Einstellungen.
- Die Zustandsmaschine erlaubt nur `queued -> running -> succeeded|failed|cancelled` sowie Fehler
  oder Abbruch vor dem Start. Endzustände sind endgültig; Herkunft und bereinigte Fehler dürfen nur
  zum passenden Endzustand gehören.
- Rust prüft Registry, Modellfähigkeit, Grenzen und explizite Kontextfreigabe vor einem möglichen
  Provideraufruf. Statische Fehler enthalten keine Eingaben, Pfade oder fremden Diagnosedaten.
- Ein rein lokaler, nur in Tests kompilierter Mock prüft den vollständigen Jobablauf ohne Netzwerk,
  Bilddatei oder Assetübernahme. Es gibt keinen produktiven Bildadapter, Tauri-Command, SQLite-Job
  oder Snapshotformat für Läufe.

### Deterministische Snapshots und lokales Git (KQ-008)

- Portables Snapshotformat v3 für Projektmetadaten, Dokumente, Story-Bibel, Plotboard, Timelines,
  visuelle Profile und Prompt-Entwürfe; v1/v2 bleiben lesbar und ein formales JSON-Schema wird mit
  jedem Snapshot kopiert.
- UUID-basierte Markdown-Dateinamen, stabile fachliche Sortierung, UTF-8/LF und keine flüchtigen
  Datenbankzeitstempel. Wiederholte Erzeugung unveränderter Projekte ist byte-identisch getestet.
- Freigegebene Rasterbilder werden inhaltsadressiert kopiert; Manifest, Hash und tatsächliche
  Assetdateien werden gemeinsam validiert.
- Snapshotstände entstehen in temporären Geschwisterverzeichnissen und werden als vollständiger
  Verzeichnistausch veröffentlicht.
- Deny-by-default `.gitignore` sowie feste Pfadangaben beschränken Init, Add, Status, Diff und
  Commit auf `.gitignore` und `snapshot/**`. SQLite, WAL, Sicherungen und Nebenartefakte bleiben
  ungetrackt.
- Globale Git-Konfiguration, Systemkonfiguration, Hooks, externe Diffs, Signaturzwang und
  interaktive Prompts sind für Dienstoperationen deaktiviert.
- Versionierte Tauri-Commands und die zugängliche Ansicht **Versionen** bieten Snapshoterzeugung,
  lesende Prüfung, Status, Unified Diff und expliziten Commit.
- Automatisierte Tests prüfen Byte-Stabilität, Manipulationserkennung, Ausschluss der Datenbank und
  Git-Roundtrip. Ein Playwright-Ablauf prüft zusätzlich zwei explizite Snapshotstände samt lesender
  Validierung, Änderungsliste, Unified Diff, Commitbeleg und anschließend sauberem Status gegen den
  testexklusiven In-Memory-Adapter. Echtes Git, Dateisystem, Tauri, WebView und Screenreader bleiben
  getrennt abzunehmen.
- Die Ansicht **Versionen** bleibt eine ausschließlich lesende Prüfung des aktuellen
  Projekt-Snapshotordners; sie überschreibt keinen Projektstand.

### Atomarer Snapshot-v3-Import und Beispielprojekt (KQ-038)

- Ein begrenzter `.kraqel`-ZIP-Container wird vollständig in Rust geprüft: feste Dateimenge und
  Schema-Kopie, sichere Pfade, Größen und Anzahl, kanonische UUIDs, Dokumenthierarchie, sämtliche
  fachlichen Referenzen, Assethashes sowie vollständig dekodierte Rasterbilder.
- Jeder Import erzeugt eine neue Projektkennung und befüllt SQLite, Originalassets und abgeleitete
  Vorschaubilder in einem Staging-Verzeichnis. Erst ein vollständig erfolgreicher Lauf wird atomar
  als neues Projekt veröffentlicht; ein Fehler hinterlässt kein sichtbares Teilprojekt.
- **Die Stadt, die den Dienstag verkaufte** ist aus dem festgeschriebenen Quellcommit
  `f1a6fd6b3b14852a2a44be617b366cc4c0dbfd2a` reproduzierbar in Snapshot v3 überführt und als rund
  10,6 MiB große Offlinevorlage eingebettet. Enthalten sind 20 Dokumente, 36 Story-Bibel-Einträge,
  ein Plotboard, drei Timelines mit 25 Ereignissen, 13 visuelle Profile und 21 optimierte Bilder.
- Rust-Tests belegen erfolgreichen Mehrfachimport mit unterschiedlichen Projektkennungen,
  Datenbankinhalte und Vorschaubilder sowie die rückstandsfreie Ablehnung eines manipulierten
  Archivs. Command-, UI- und Playwright-Tests prüfen den Desktoppfad und zwei unabhängige Kopien.
- Rechte, Quelle, Transformationen und Archivhash sind in `examples/NOTICE.md` dokumentiert. Die
  Vorlage benötigt zur Laufzeit weder GitHub noch einen anderen Netzwerkzugriff.
- KQ-050 ergänzt inzwischen freie native Archivauswahl; KQ-054/055/061 ergänzen Text-/DOCX-Import
  und Manuskriptaustausch. Überschreiben oder Zusammenführen vorhandener Projekte bleibt
  ausgeschlossen.

## Offene Aufgaben

### Unmittelbar: 0.1 stabilisieren

1. **F08-/M3-Kriterienmatrix abarbeiten**
   - der konkrete Drehbuchvertrag und die F08-Importrichtungen sind fachlich freigegeben
   - der funktionale native Gesamtlauf M3-G01 ist gemäß [Protokoll](testing/native-m3-acceptance.md)
     bestanden; verbleibende fachliche und assistive Restprüfungen dort einzeln nachweisen
   - nach Drehbuchfreigabe Modell-/Versionsprototyp und Schriftprüfung vor Rendererimplementierung

### Bewusst zurückgestellt

- Der isolierte Debug-NSIS-Lebenszyklus ist durch KQ-030 belegt; KQ-036 ergänzt den zurückgeladenen
  echten RC bis Installation und Deinstallation. SmartScreen, Anwendungsstart und Persistenz werden
  auf einem weiteren Windows-Rechner oder einer Clean-VM später nachgeholt. Sie bleiben offene
  Release-Evidenz, sind aber keine unmittelbar blockierende Entwicklungsaufgabe.

### Danach gemäß Pflichtenheft

- KI-basierte Kontinuitätshinweise bleiben eine spätere Erweiterung von KQ-012.
- KQ-017 und KQ-018 gehören zu Version 0.2 und sind kein Ziel für 0.1.

Für KQ-017 und KQ-018 sind noch keine GitHub-Issues angelegt; die Aufgaben stehen im Pflichtenheft.
Vor 0.2 müssen Akzeptanzkriterien, Abhängigkeiten und Rechtsfragen konkretisiert werden.

## Bekannte Fehler und Lücken

- **Löschen ist endgültig:** Dokumente und ihre abhängigen Links werden erst nach zweistufiger
  Bestätigung und Konfliktprüfung entfernt, besitzen aber noch keinen Papierkorb oder Undo über die
  laufende Sitzung hinaus. Backups und Git-Snapshots bleiben der Wiederherstellungsweg.
- **Feste Projektwurzel in der Desktopoberfläche:** Projekte liegen derzeit unter dem lokalen
  App-Datenverzeichnis. Eine frei wählbare Projektwurzel ist noch nicht umgesetzt.
- **Backup bleibt zweiteilig:** Eine Sicherung besteht aus `<name>.sqlite3` und `<name>.assets/`.
  Beide Teile werden gemeinsam erzeugt und für medienhaltige Wiederherstellungen benötigt; ein
  einzelnes gepacktes Austauscharchiv ist noch nicht definiert.
- **Snapshotimport bewusst begrenzt:** Snapshot v3 kann als neues Projekt über die eingebettete
  Vorlage oder einen nativen Dateidialog importiert werden. v1/v2-Import, Zusammenführen und
  Überschreiben sind nicht implementiert. Diese Einschränkungen sind keine zusätzlich erfundenen
  F07-Mussfunktionen; der native Nachweis der definierten lesenden Wiederherstellung ist mit KQ-067
  / M3-02/03 der Matrix im dokumentierten Umfang bestanden. Keine allgemeine Git-Checkout-Funktion.
- **Begrenzter Import ist kein Roundtrip:** Die vier Ausgaben, der UTF-8-/Markdown-Import und der
  begrenzte DOCX-Import sind vorhanden. Der DOCX-Import erzeugt eine neue Szene; er rekonstruiert
  keine Kapitel-/Szenenstruktur und übernimmt keine Medien. Ein strukturerhaltender Roundtrip zurück
  nach KRAQEL besitzt mit KQ-060 einen geprüften nativen Kern und mit KQ-061 den bedienbaren
  App-Import als atomare neue Projektkopie. Der begrenzte Manuskriptvertrag ist umgesetzt; F08
  bleibt wegen fehlender Drehbuchimplementierung und offener Restprüfungen gemäß Matrix teilweise
  erfüllt; die Format-/Importrichtungen sind fachlich freigegeben.
- **Exportformatierung bewusst begrenzt:** Links, Code, Tabellen, Fußnoten und eingebettete
  Editor-Bildknoten werden nicht still reduziert, sondern abgelehnt. Direkt verknüpfte Szenenbilder
  sind im eigenen Modus für alle vier Formate umgesetzt. PDF nutzt seit KQ-065 vier eingebettete
  Noto-Sans-Schnitte mit begrenztem, geprüftem Glyphenprofil; universelles Unicode/Shaping bleibt
  ausgeschlossen.
- **Medienumfang bewusst begrenzt:** SVG, Animationen, Audio, Video und eigene Lizenzfelder fehlen.
  Eingebettete DOCX-/PDF-Bilder existieren im expliziten Szenenbildmodus; ein Bild-Roundtrip fehlt.
  Der WebView übergibt ausgewählte Dateien als Bytearray; bei vielen großen Dateien ist später ein
  nativer Streamingimport sinnvoll. Fehlgeschlagene Dateibereinigung kann unreferenzierte Dateien
  hinterlassen, aber keine intakte Referenz vorzeitig löschen.
- **Migrationen bleiben vorwärtsgerichtet:** 0→1 sowie befüllte Migrationen 1→2, 2→3, 3→4, 4→5 und
  5→6 sind getestet; weitere Versionen benötigen jeweils eine neue unveränderliche Migration.
- **Zeitwerte ohne Kalendersemantik:** Absolute Zeitwerte sind validierte Zeichenketten. Kalender,
  Zeitzonen und Arithmetik sind bewusst noch nicht implementiert.
- **Browser-E2E bleibt vom Tauri-Smoke-Test getrennt:** Playwright prüft breite React-Abläufe gegen
  einen testexklusiven Command-Adapter. KQ-029 belegt nur einen kleinen echten Windows-Pfad durch
  WebView, Tauri-IPC, Rust und SQLite. KQ-030 ergänzt dafür den isolierten lokalen
  NSIS-Installationslebenszyklus. Native Dialoge, weitere Fachansichten, Clean-VM, Release-Packaging
  und die verbleibenden assistiven Randfälle bleiben separat abzunehmen. KQ-033 ergänzt eine
  manuelle Screenreader-Kernprüfung, aber keine vollständige fachliche Tauri-Abnahme aller
  Ansichten.
- **Prompt-Builder-Browser-E2E ist keine Dateisystemabnahme:** Der Chromium-Ablauf prüft
  Zusammensetzung, Bearbeitung, Persistenzvertrag und sichtbare Exportbestätigung gegen den
  testexklusiven Adapter. Die echte UTF-8/LF-Datei wird in Rust getestet; Tauri, WebView,
  Screenreader und Betriebssystemdateisystem bleiben separat abzunehmen. Entwürfe sind bewusst nicht
  automatisch mit Bildprovidern verbunden.
- **Export-Browser-E2E ist keine Renderer- oder Dateisystemabnahme:** Der Chromium-Ablauf prüft die
  vier Formataktionen sowie die sichtbare Dateinamen- und Größenbestätigung gegen den testexklusiven
  Adapter. Er erzeugt keine Dateien und prüft weder deren Inhalt noch Tauri,
  Betriebssystemdateisystem, Layout oder Screenreader.
- **Medien-Browser-E2E ist keine Inhalts- oder Dateisystemabnahme:** Der Chromium-Ablauf prüft
  Dateiauswahl, sichtbare Vertragsdaten und Szenenverknüpfung. Der testexklusive Adapter führt keine
  Bilddekodierung, kryptografische Hashprüfung, Thumbnailerzeugung oder Dateioperation aus; Tauri,
  Betriebssystemdateisystem und Screenreader bleiben separat abzunehmen.
- **Bildverträge ohne Laufzeitfunktion:** Registry, Jobzustände, Freigabe und Fehler sind getestet,
  aber es gibt weder produktive Provider noch UI, Persistenz, Abbruch, Bildausgabe oder
  Assetübernahme. Das ist der beabsichtigte 0.1-Umfang, keine bereits fertige Bildgenerierung.
- **Ollama nur gegen Testserver geprüft:** Kein Ollama-Dienst und kein lokales Modell waren in der
  Entwicklungsumgebung verfügbar. Abbruch, Laufpersistenz und eine manuelle Übernahme des Entwurfs
  in den Editor sind noch nicht implementiert.
- **Cloudprovider nur gegen Testserver geprüft:** Die vier Protokolle und der Schlüsselspeicher sind
  implementiert, aber ohne reale Schlüssel weder gegen Anbieterendpunkte noch hinsichtlich Kosten,
  Kontodialogen und serverseitiger Aufbewahrung abgenommen.
- **Kontinuitätsregeln bewusst konservativ:** Namensnennungen werden nur exakt erkannt; Aliasse,
  Pronomen und semantische Widersprüche bleiben unerkannt. Mehrfachpositionen auf einer Timeline
  sind Prüfhinweise, keine behaupteten Fehler. Befunde werden bei jedem Lauf neu berechnet und noch
  nicht als Prüfhistorie gespeichert.
- **Entitätssuche nicht indexiert:** Die Unicode-korrekte Suche filtert derzeit alle Einträge eines
  Projekts in Rust. Vor sehr großen Story-Bibeln sind Messung und gegebenenfalls eine echte
  FTS-Migration nötig.
- **Dependency Review benötigt GitHub-Metadaten:** Der Dependency Graph und Schwachstellenwarnungen
  sind aktiviert. Der separate Review-Job kann dadurch Abhängigkeitsänderungen in Pull Requests
  prüfen; seine Aussage bleibt auf von GitHub erkannte Manifest- und Lockdateien sowie bekannte
  Advisory-Daten begrenzt.
- **Unsigniertes Alpha-Prerelease:** Windows kann SmartScreen-Warnungen anzeigen; SHA-256 bindet
  keine Herausgeber*innenidentität und schützt nicht, wenn Setup und Sollwert über denselben
  kompromittierten Kanal ersetzt werden. GitHub-Rückdownload sowie isolierte Installation und
  Deinstallation sind geprüft; SmartScreen und gestartete Anwendung bleiben extern abzunehmen.
- Die vollständige lokale Qualitätskette ist grün: 43 TypeScript-, 97 ausgeführte Rust- und siebzehn
  Playwright-E2E-Tests, Produktions-Frontend-Build, Formatierung, ESLint, Clippy und Typprüfung.
  Zusätzlich bleiben der isolierte Tauri-Debug-Build, der echte Windows-Persistenz-Smoke-Test sowie
  Build, Installation und Deinstallation des unsignierten NSIS-Testartefakts grün. Ein visueller
  Export-QA-Test bleibt planmäßig ignoriert. Die CI-Matrix unter Windows und Linux ist verbindliche
  Merge-Voraussetzung; die nativen Windows-Smoke-Tests laufen derzeit bewusst lokal.
- KQ-033 ergänzt eine gemeinsam mit einer hörenden Testperson durchgeführte Windows-Sprachausgabe-
  Prüfung in der isolierten echten Tauri-WebView. Projektanlage, M1-Editor, Autosave-Ansagen,
  Story-Bibel-Beziehungen und Rückverweis wurden vollständig im vereinbarten Kernpfad geprüft; die
  weiteren Werkzeugansichten wurden auf Namen, Fokus, Tastaturfallen und deaktivierte Zustände
  stichprobenartig geprüft. 200 Prozent Zoom und ein schmales Fenster blieben bedienbar.
- KQ-034 ergänzt zwei Playwright-Randfälle: Ein einmaliger Fehler im E2E-Command-Mock bleibt
  ungespeichert, wird als Live-Status sichtbar und beim nächsten Bearbeiten erfolgreich nachgeholt;
  emulierte reduzierte Bewegung lässt keine wirksamen Animationen oder Übergänge übrig. Beide Fälle
  wurden zusätzlich mit Windows-Sprachausgabe in der isolierten Accessibility-Tauri-Variante
  bestätigt. Die Windows-Animationseinstellung wurde anschließend wiederhergestellt.
- KQ-035 erzeugt aus einem sauberen Git-Commit einen unsignierten Windows-x64-Release Candidate mit
  Offline-WebView2. Das 212,97-MiB-Setup aus Commit `7f229d09f9ad` wurde ohne eingebettete
  Authenticode-Zertifikatstabelle gestaged; Manifest, Prüfsummendatei und eine unabhängige
  `certutil`-Berechnung stimmen auf SHA-256
  `1c28b107f025b8e9ff3481b8d3f41eed31efff6d99b7f9966760cdad0110e0e6` überein. Der Build lief erst
  nach zwei portablen Korrekturen grün: Signatur- und Hashprüfung verwenden nun .NET/PE-Strukturen
  statt der auf diesem Rechner nicht ladbaren PowerShell-Standardmodule.
- KQ-036 baute den RC erneut aus dem sauberen Merge-Commit `5155ed5282cf`, legte den privaten
  Prerelease-Entwurf `v0.1.0-rc.1` mit genau drei Assets an und erzeugte keinen veröffentlichten
  Git-Tag. GitHub-Digest, Prüfsummendatei, Manifest und Rückdownload stimmen auf SHA-256
  `1a41a1f2d1a948cd59c4aa351993f31caa979477c399c0cfa356a0dcd4ece464` überein. Das heruntergeladene
  Setup installierte still in ein begrenztes temporäres Ziel, registrierte Version und
  Installationspfad korrekt und deinstallierte vollständig. Die Anwendung wurde wegen vorhandener
  realer App-Daten nicht gestartet; deren Fingerabdruck blieb unverändert.
- KQ-037 benannte den bestehenden Entwurf in `v0.1.0-alpha.1` um, ersetzte die Entwurfsnotizen durch
  klare Alpha-, Backup-, Unsigned- und SmartScreen-Hinweise und veröffentlichte ihn als Prerelease.
  Der Tag zeigt auf den unveränderten Build-Commit `5155ed5282cf`; Setup, Manifest und Prüfsumme
  behielten ihre GitHub-Digests. Das frühere RC-Tag wurde nicht erzeugt.
- KQ-039 baute Alpha 2 aus dem sauberen KQ-038-Merge-Commit `c35dc4766793`. Das 223,13 MiB große,
  unsignierte Setup enthält die offline eingebettete Beispielvorlage. Genau drei Assets wurden als
  Entwurf hochgeladen, erneut heruntergeladen und gegen Manifest, Commit und SHA-256
  `3bccda1f1f89383a5a1f2d3b6c006f145eeacadc57578314824b9dc1ccfae3eb` geprüft. Das zurückgeladene
  Setup installierte und deinstallierte isoliert; bestehende reguläre App-Daten blieben unverändert.
  Der Tag `v0.1.0-alpha.2` zeigt auf den Build-Commit. Die Anwendung wurde auf diesem Rechner
  weiterhin nicht gestartet.
- KQ-041 baute Alpha 3 aus dem sauberen KQ-040-Merge-Commit `e3d091771b23`. Das 223,13 MiB große,
  unsignierte Setup enthält zusätzlich die bei 360 Pixeln browserseitig geprüften responsiven
  Medienkarten. Entwurf und öffentlicher Rückdownload stimmen für alle drei Assets bytegenau mit dem
  lokalen Build überein; das Setup hat SHA-256
  `93ec0b85e454f9e0f2e776d4d052b2c8e1f7f27837045658831816ab627dff21`. Der Entwurfsdownload bestand
  isolierte Installation und Deinstallation. Beim öffentlichen Rückdownload brach dieselbe Prüfung
  erwartungsgemäß beim App-Daten-Fingerabdruck ab, weil die reguläre KRAQEL-App parallel lief; sie
  wurde nicht beendet. Der Tag `v0.1.0-alpha.3` zeigt auf den Build-Commit.
- KQ-045 baute Alpha 4 aus dem sauberen KQ-044-Merge-Commit `24ceb3c627f9`. Das 223,14 MiB große,
  unsignierte Setup enthält den auf diesem Rechner manuell abgenommenen Dubletten-Merge samt
  Vereinigung zweier visueller Profile. Release Candidate und öffentlicher GitHub-Rückdownload
  bestanden Hash-, Manifest-, Signatur-, Installations-, Registrierungs- und Deinstallationsprüfung;
  bestehende App-Daten blieben unverändert. Setup, Prüfsummendatei und GitHub-Digest stimmen auf
  SHA-256 `0e6d578d1dfb48e8ecfe6c8be04be3f593f83eff43756f829d30ebad761fb6be` überein. Der Tag
  `v0.1.0-alpha.4` zeigt auf den Build-Commit.

## Wichtige Architekturentscheidungen

- **SQLite als lokale Autorität, Git für portable Snapshots:** Die laufende Datenbank wird nicht
  versioniert. Exporte müssen deterministisch und wiederherstellbar sein (ADR 0001).
- **Reproduzierbares SQLite und atomare Vorwärtsmigrationen:** Gebündeltes SQLite stellt FTS5
  bereit; `PRAGMA user_version`, Fremdschlüssel, WAL und sofortige Transaktionen definieren den
  Verbindungs- und Migrationsvertrag (ADR 0002).
- **Kanonische Projektwurzel und nicht-destruktive Recovery:** Symlinks und unsichere relative Pfade
  werden abgelehnt; Projekte und Sicherungen werden erst nach Prüfung atomar veröffentlicht.
  Wiederherstellung schreibt ausschließlich in ein neues Ziel (ADR 0003).
- **Versioniertes Tiptap-JSON und optimistische Revisionen:** Jede Szene besitzt eine eigene
  Undo/Redo-Grenze; Autosave ist entprellt, serialisiert und konfliktbehaftete Schreibversuche
  überschreiben nichts. Metadaten- und Baumänderungen verwenden eigene Compare-and-swap-Grenzen;
  Löschen bindet den vollständig angezeigten Stand (ADR 0004).
- **Begrenzte Story-Verträge auf Schema v1:** Vier Entitätsarten und strukturierte freie
  Eigenschaften werden in Rust validiert; Relationen sind gerichtet, Szenenlinks idempotent und
  Löschkaskaden explizit. Die Unicode-Suche bleibt zunächst bewusst unindexiert (ADR 0005).
- **Getrennte Datenbankversion und explizites Zeitmodell:** SQLite-Migrationen sind vom portablen
  Projektformat getrennt; relative und absolute Zeitebenen erzwingen unterschiedliche Zeitwerte und
  Links bleiben n:m (ADR 0006).
- **Portabler Snapshot und kontrolliertes Git:** Nur `.gitignore` und `snapshot/**` sind
  freigegeben; globale Konfiguration, Hooks und interaktive Git-Abläufe bleiben deaktiviert. Die
  erste Wiederherstellungsstufe ist lesende Validierung (ADR 0007).
- **Neues Projekt statt Überschreiben:** Begrenzte Snapshot-v3-Archive werden vollständig in Rust
  geprüft, erhalten stets eine neue Projektkennung und werden erst nach erfolgreicher Befüllung
  atomar veröffentlicht. Die Beispielvorlage ist fest eingebettet und netzunabhängig (ADR 0019).
- **Ein Exportmodell für vier Renderer:** Die unterstützte Tiptap-Teilmenge wird vor jeder Ausgabe
  vollständig validiert. Renderer erhalten nur das gemeinsame Zwischenformat; Ziele werden lokal und
  atomar veröffentlicht (ADR 0008).
- **Lokales Ollama hinter einer revisionsgebundenen Freigabe:** React erhält keinen direkten
  Providerzugriff. Rust lädt den freigegebenen Szenenstand erneut, spricht ausschließlich Loopback
  an und streamt einen verwerfbaren Entwurf ohne automatische Speicherung (ADR 0009).
- **Cloudprovider mit fester Ziel- und Einmalfreigabe:** Schlüssel liegen im nativen
  Betriebssystemspeicher; feste HTTPS-Ziele und ein kurzlebiger, verbrauchbarer Freigabebeleg binden
  Provider, Modell, Arbeitsauftrag und Dokumentrevision (ADR 0010).
- **Deterministische Kontinuität und Nur-Lese-Review:** Regeln arbeiten auf einer lokalen
  Faktenprojektion und formulieren Mehrdeutigkeiten nicht als Gewissheiten. Vorschlags-Diffs sind an
  die Szenenrevision gebunden und besitzen keinen Schreibpfad (ADR 0011).
- **Inhaltsgeprüfte lokale Medien:** Rust validiert Formate und Grenzen, Projektdateinamen entstehen
  aus UUID/Hash, Ersetzen schaltet erst nach neuer Dateiveröffentlichung um, und Backup/Snapshot
  prüfen referenzierte Originale gegen SHA-256 (ADR 0012).
- **Projektgebundene visuelle Vorgaben:** Profile referenzieren Entitäten und optionale kanonische
  Assets über stabile UUIDs; Stilvorlagen bleiben strukturiert, begrenzt und vollständig lokal (ADR
  0013).
- **Explizite lokale Promptquellen:** Bildprompt-Entwürfe binden Szene, Quellrevision, Profile und
  optionale Stilvorlage; Zusammensetzung, Speicherung und Textausgabe bleiben ohne Providerpfad (ADR
  0014).
- **Modellspezifische Bildfähigkeiten:** Eine offline validierte Registry und ein geschlossener
  Jobvertrag trennen Modellfähigkeiten, Freigabe, Herkunft und bereinigte Fehler von jeder späteren
  Ausführung. KQ-016 enthält nur einen Test-Mock (ADR 0015).
- **Isolierte NSIS-Testvariante:** Produktname, Bundle-ID, App-Daten und Installationsziel sind vom
  regulären Produkt getrennt; nur der unsignierte Current-User-Debug-Installer wird lokal geprüft.
  Clean-VM, MSI und Release-Signierung bleiben eigene Nachweise (ADR 0016).
- **Isolierte Accessibility-Testvariante:** Eigener Produktname, eigene Bundle-ID, temporäre Daten,
  deaktiviertes Bundling und der E2E-Command-Mock begrenzen den kontrollierten Autosave-Fehler auf
  manuelle Entwicklungstests; Produktion, Rust und SQLite enthalten keinen Fehlerhaken (ADR 0017).
- **Bewusst unsignierter Windows-Releasepfad:** 0.1 verwendet ausschließlich NSIS für Windows x64,
  installiert pro Benutzer*innenkonto und enthält WebView2 offline. SHA-256 und Buildmanifest machen
  das konkrete Artefakt prüfbar, ersetzen aber keine Herausgeber*innenbindung; MSI, Updater, ARM64
  und Signierung bleiben ausgeschlossen (ADR 0018).
- **Privilegierte Operationen in Rust:** React kommuniziert ausschließlich über versionierte,
  typisierte Tauri-Commands mit Dateisystem, Storage, Git und späteren Providern.
- **Verträge sind explizit versioniert:** Inkompatible Änderungen erfordern eine neue
  `contractVersion`; TypeScript allein ist keine Sicherheitsgrenze.
- **Fehler sind datenarm:** IPC-Fehler enthalten keine Eingaben, lokalen Pfade, Stacktraces oder
  internen Diagnosedaten.
- **Offline und Opt-in:** Provider sind optional; Cloudzugriffe benötigen Vorschau und explizite
  Freigabe. Keine Telemetrie ab Werk.
- **Manuelle Übernahme:** KI-Vorschläge und Bildentwürfe verändern kanonische Inhalte niemals
  automatisch.

Offene Produktentscheidungen aus dem Pflichtenheft: Umfang des DOCX-Imports und -Roundtrips sowie
Medien-Einbettung je Exportformat. Rechts- und Lizenzfragen der Bildprovider sind erst vor Version
0.2 zu klären.

## Abhängigkeiten und technische Risiken

| Thema                        | Zustand / Risiko                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                   | Nächste Kontrolle                                                                                                                          |
| ---------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------ |
| SQLite und FTS5              | Dokument-FTS sowie Story-Bibel-CRUD, Relationen, Links und Recovery getestet; Entitätssuche nicht indexiert                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        | Vor großen Projekten messen; Migration nur mit Evidenz                                                                                     |
| Rust-/TS-Vertragsduplikation | Manuell gespiegelt; Drift ist möglich                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              | Gemeinsame Fixtures oder spätere Codegenerierung bewerten                                                                                  |
| Datenformat                  | Snapshot v3 samt begrenztem Neuimport, nativer und unter Windows manuell geprüfter Archivauswahl, Prompt-Entwürfen, Profilen und Hash-Assets; UTF-8-/Markdown-Import automatisiert und im dokumentierten nativen Windows-Kernumfang geprüft; Begrenzter DOCX-Import vorhanden; begrenzter strukturerhaltender Manuskriptroundtrip mit KQ-060/061 umgesetzt                                                                                                                                                                                                                                                                         | Unicode-/Shaping-Erweiterungen gesondert entscheiden; Fremdarchive fuzztesten                                                              |
| Exportrenderer               | Markdown/HTML mit Bildern nativ geprüft; DOCX-Bildwriter mit Word und LibreOffice geprüft; PDF mit vier eingebetteten Schnitten, begrenztem Unicode-Profil und pixelgeprüften Szenenbildern                                                                                                                                                                                                                                                                                                                                                                                                                                        | Assistive Vollabnahme; Unicode-/Shaping-Erweiterungen gesondert prüfen                                                                     |
| Dateisystem                  | Komponentenprüfung aktiv; TOCTOU gegen böswillige Paralleländerung bleibt                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          | Vor nicht vertrauenswürdigen Importen Handle-APIs prüfen                                                                                   |
| Editorformat                 | Tiptap-Vertrag v1; spätere inkompatible Änderungen brauchen neue Version                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           | Bei jeder Erweiterung Fixtures und Migration prüfen                                                                                        |
| Desktoptests                 | Zwölf Browser-E2E-Suites einschließlich Beispielprojekt, Accessibility-Kern und -Randfällen, echter Windows-Tauri-M1-Smoke, lokaler NSIS-Lebenszyklus sowie bestandene KQ-033/KQ-034-Hörprüfungen                                                                                                                                                                                                                                                                                                                                                                                                                                  | Vor Release mit weiterer Testperson oder NVDA verbreitern                                                                                  |
| Windows-Installer            | Debug-Lebenszyklus, unsignierter x64-Releasebau und isolierte Installation/Deinstallation grün; Alpha 4 mit Dubletten-Merge veröffentlicht, öffentlicher Rückdownload bytegleich, SmartScreen/Start/Persistenz extern offen                                                                                                                                                                                                                                                                                                                                                                                                        | Zweitrechner oder Clean-VM später manuell                                                                                                  |
| CI-Laufzeit                  | Kalte Tauri-/Rust-Builds unter Windows dauern mehrere Minuten                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      | Caching prüfen, ohne Reproduzierbarkeit zu schwächen                                                                                       |
| Abhängigkeitssicherheit      | GitHub Dependency Graph und Schwachstellenwarnungen aktiviert. KQ-048 ist in PR #93 vollständig grün; GitHubs aktuelles SBOM enthält nur noch `zip` 9.5.1. GitHub meldet am 14. November drei offene Abhängigkeitswarnungen: `lopdf` (hoch), `glib` und `minimatch` (mittel). Die frühere Triage „lopdf-Parser nicht erreichbar“ ist seit KQ-065 überholt: Der native PDF-Selbstvergleich liest die eigene begrenzte Ausgabe, keine fremden PDFs. Aktuelle Laufzeitbewertung dieses Pfads, glib-Linux-Aufrufpfad und genaue minimatch-10-Aufrufer bleiben prüfbedürftig. Keine dynamische Sicherheitsprüfung oder Alert-Schließung | Issue #94 und Upstream prüfen; lopdf/glib getrennt nach Laufzeitpfad und verfügbarer Korrektur bewerten; keine erzwungenen Major-Overrides |
| Plattformen                  | Windows ist primär; Linux baut in CI, ist fachlich aber nicht abgenommen                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                           | Plattformmatrix je Release definieren                                                                                                      |
| Lokale Text-KI               | Protokoll gegen kontrollierten Loopback-Testserver geprüft; echter Ollama-/Modelllauf fehlt                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                        | Mit freiem Testmodell manuell prüfen und Abbruch entwerfen                                                                                 |
| Cloudprovider                | Vier Protokolle lokal getestet; echte API-, Kosten-, Aufbewahrungs- und Kontodialog-Abnahme fehlt                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                  | Mit separaten Testkonten und Minimalbudget manuell prüfen                                                                                  |
| Schlüsselspeicher            | Native Backends eingebunden; reale Windows-/Linux-Dialoge nicht automatisiert getestet                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                             | Manuelle Plattformabnahme ohne produktive Schlüssel                                                                                        |
| Kontinuitätsprüfung          | Vier Offline-Regeln getestet; exakte Nennungen und grober Zeilendiff begrenzen die Abdeckung                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       | Mit größeren künstlichen Projekten messen; KI-Kanal separat entwerfen                                                                      |
| Medienimport                 | Drei Rasterformate und harte Limits getestet; Bytearray-IPC skaliert nicht für Massenuploads                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       | Nativen Streamingimport erst nach Messung großer Sammlungen bewerten                                                                       |
| Assetbackup                  | Hash-verifizierte Begleitverzeichnisse getestet; `.kraqel` ist ein portables Snapshotpaket, aber kein fortschreibbares Backup                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                      | Recovery- und Austauschzweck in der Oberfläche klar getrennt halten                                                                        |
| Visuelle Profile             | Struktur und Referenzen getestet; Stilnamen-Eindeutigkeit nutzt SQLite-NOCASE mit ASCII-Grenze                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     | Unicode-Kollation bei internationalisierten Vorlagennamen bewerten                                                                         |
| Bildprompt-Entwürfe          | Lokal, referenziell und browserseitig geprüft; kein Providerpfad und keine automatische Rekombination                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              | Tauri-/Screenreader-Abnahme; KQ-016-Grenze getrennt halten                                                                                 |
| Bild-Capability-Verträge     | Rust/TypeScript v1 und lokaler Mock getestet; keine produktive Ausführung oder Persistenz                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          | Vor KQ-017 Freigabebeleg, Ergebnisprüfung, Kosten und Lizenzen klären                                                                      |

## Nächste geplante Entwicklungsstufe

**Nächste Stufe: Den gemessenen WebView2-Netzbefund KQ-069 / #134 klären. Danach
Modell-/Versionsprototyp und Schriftprüfung für den freigegebenen Drehbuchvertrag; Grenzen gemäß
[Matrix](testing/F08_M3_ACCEPTANCE_MATRIX.md) beibehalten.**

Vertrag und Kriterienerfassung liegen vor und sind fachlich freigegeben. Der funktionale kombinierte
M3-Gesamtlauf ist bestanden. KQ-068 hat den unabhängigen Netztest durchgeführt und öffentlichen
WebView2-Verkehr gefunden; die Netzabnahme ist nicht bestanden. Drehbuchimplementierung, assistive
Vollabnahme bleiben offen; der native GUI-CRUD-Korpus einschließlich Neustart und die gezeigte
abschließende Lösch-/Export-Hörprobe sind bestanden.

Empfohlener, überprüfbarer Scope:

1. Die native UTF-8-/Markdown-Prüfung ist bestanden; Nachweis und Aussagegrenzen stehen in
   [`testing/native-text-import-acceptance.md`](testing/native-text-import-acceptance.md).
2. Der enge DOCX-Import ist mit ADR 0024 umgesetzt. Der
   [Roundtrip-Vertragsentwurf](specifications/DOCX_ROUNDTRIP_CONTRACT_DRAFT.md) legt den getrennten
   Manuskriptumfang fest. KQ-058 / ADR 0025 dokumentiert den Verlust verschachtelter Controls in
   LibreOffice. Der Folgeversuch KQ-059 besteht mit 24 flachen Absatzmarkern, 41 Struktureinträgen
   und 15 Inventareigenschaften in Word 16.0.20527.20112 und LibreOffice 26.8.3.1. Auch der Weg Word
   → LibreOffice und echte Textänderungen in beiden Anwendungen sind geprüft. Insgesamt 41
   Prototyp-Tests bestehen. Die Darstellung ist mit
   [ADR 0026](architecture/decisions/0026-flat-docx-markers.md) als Grundlage angenommen.
3. KQ-060 / [ADR 0027](architecture/decisions/0027-native-docx-roundtrip.md) setzt das gemeinsame
   kanonische Modell, das eigene Produktionsprofil v1 und den nativen Rust-Reader/Writer um.
   [Vertrag und Grenzen](data-format/DOCX_ROUNDTRIP.md) sind festgeschrieben. Vier unmittelbar vom
   Rust-Writer beziehungsweise echten Word-/LibreOffice-Speichervorgängen stammende v1-Fixtures
   bestehen einschließlich echter Word-Textänderung und visueller Prüfung. Die sechs alten
   Prototyp-Fixtures bleiben mit explizit testinterner Profilumschreibung zusätzliche Regressionen.
4. KQ-061 / [ADR 0028](architecture/decisions/0028-atomic-docx-project-copy.md) bindet den Kern in
   Exportansicht und Startseite ein. Vollständig validierte neue Projekte werden erst nach
   transaktionaler Befüllung und semantischem SQLite-Vergleich atomar veröffentlicht. Browser-E2E
   und native Windows-Dialogprüfung ergänzen den Nachweis.
5. KQ-062 ist mit grüner CI in PR #122 zusammengeführt, Issue #121 geschlossen. Der gesamte
   [Medienexport-Vertrag](specifications/MEDIA_EXPORT_CONTRACT_DRAFT.md) und ADR 0029 sind
   freigegeben. KQ-063 / ADR 0030 implementiert den gemeinsamen Rust-Kern und Markdown/HTML:
   ausdrücklich ausgewählte direkte Szenenbilder am Szenenende, Reihenfolge, Alttext, Beschriftung,
   PNG-Ableitungen, Herkunftsmanifest und unveränderliche neue Paketgenerationen. Der konkrete
   [Formatvertrag](data-format/ILLUSTRATED_MANUSCRIPT.md) legt Grammatik und Grenzen fest.
6. KQ-064 / ADR 0031 ergänzt den gewöhnlichen DOCX-Bildwriter und die App-Aktion. LibreOffice
   öffnet/speichert das Prüfmanuskript mit identischen Texten, Alttexten und Bildpixeln. Die drei
   gerenderten Seiten bleiben pixelidentisch. Word erhält nach Speicherung ebenfalls Texte, Alttexte
   und Bildpixel; die drei von Word ausgegebenen Seiten sind visuell geprüft. KQ-065 ergänzt PDF mit
   vier eingebetteten Noto-Sans-Schnitten, NFC-/Glyphenprüfung und metrischem Umbruch. Die weiße
   PNG-Ableitung wird im Manifest und am tatsächlichen PDF-Bildobjekt pixelgenau nachgewiesen.
   Drehbuchformatierung bleibt eine eigene offene F08-Entscheidung. Der bestehende
   DOCX-Text-Roundtrip wird nicht um Bilder erweitert. F08/M3 bleiben teilweise abgenommen.

Die zusätzliche Installerprüfung auf einem weiteren Windows-Rechner ist bewusst zurückgestellt und
wird später manuell nachgeholt.

## Einstieg für die nächste Codex-Session

```powershell
cd D:\kraqel
git status
git branch --show-current
git log -5 --oneline
rg "KQ-05[0-4]|Projektarchiv|Projektlöschung|Textimport" README.md docs ROADMAP.md
```

Danach lesen: `AGENTS.md`, dieses Dokument, `docs/testing/accessibility-acceptance.md`,
`docs/testing/editor-accessibility.md` und die Pflichtenheft-Abschnitte M5 und 10. KQ-032 belegt den
automatisierbaren Tastatur- und Accessibility-Kern; KQ-033 ergänzt die bestandene manuelle
Windows-Sprachausgabe-Kernprüfung und KQ-034 die isolierten Randfälle. KQ-035 ergänzt den lokal
geprüften, bewusst unsignierten Releasepfad; KQ-036 belegt Entwurf, Rückdownload sowie isolierte
Installation und Deinstallation. KQ-037 veröffentlicht dasselbe Artefakt als Alpha 1; KQ-039
veröffentlicht Alpha 2 samt Beispielprojekt nach erneutem Rückdownload und isolierter Abnahme.
KQ-041 veröffentlicht den responsiven KQ-040-Stand als Alpha 3. KQ-045 veröffentlicht den manuell
abgenommenen KQ-044-Dubletten-Merge als Alpha 4 und belegt die Bytegleichheit des öffentlichen
Rückdownloads. KQ-050 ergänzt den frei gewählten, nativen Snapshotimport ohne Pfad- oder
Archivbyte-IPC; gültiger Import, Abbruch und ungültige Datei sind im echten Windows-Dialog manuell
bestätigt. KQ-051 ergänzt den nativen, begrenzten `.kraqel`-Export; KQ-052 die zweistufig bestätigte
Projektlöschung. KQ-053 aktiviert Git-Langpfade pro internem Prozess; der reale Windows-Snapshot-
und Archivexport sowie die zweistufige Löschung einer Testkopie sind bestätigt. KQ-054 ergänzt den
automatisiert geprüften, atomaren UTF-8-/Markdown-Import als Szene; dessen nativer Windows-Dialog
ist am 14. November 2026 im dokumentierten Umfang geprüft. Der enge DOCX-Import ist mit KQ-055/ADR
0024 umgesetzt. KQ-059/ADR 0026 weist die flache Repräsentation in Word und LibreOffice nach;
KQ-060/ADR 0027 implementiert den nativen Rust-Kern; KQ-061/ADR 0028 ergänzt App-Einbindung und
atomare neue Projektkopie. KQ-062 ist vollständig freigegeben und in PR #122 zusammengeführt. KQ-063
ist mit grüner CI in PR #124 zusammengeführt. KQ-064 implementiert den gewöhnlichen DOCX-Bildexport;
Word und LibreOffice sind im dokumentierten Umfang abgenommen. KQ-065 ergänzt das abgenommene
PDF-Schrift-/Unicode-Profil und bebildertes PDF. KQ-066 erfasst den inzwischen fachlich
freigegebenen Drehbuchvertrag und die vollständige F08-/M3-Matrix. KQ-067 belegt den funktionalen
nativen M3-Gesamtlauf; nächster Implementierungsschritt sind Drehbuch-Modell-/Versions- und
Schriftprototyp. Die Matrix hält die übrigen Abnahmegrenzen offen. Die bewusst zurückgestellte
Zweitrechner- oder Clean-VM-Abnahme bleibt offen. Für einen Orts- und Rechnerwechsel gilt
`docs/WORKSTATION_HANDOFF.md`.
