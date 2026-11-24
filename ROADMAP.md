# Roadmap

## Verwalteter Win32-Szeneneditor · 24. November 2026

Der Wiederanlauf der synthetischen Szenenübernahme ist für die geprüften Prozessabbrüche abgeschlossen. Der ausdrückliche Wiederanlauf darf genau einen authentisierten Übernahmeeintrag samt gegebenenfalls vorbereitetem Kontrollkopf bestätigen. Die Bindung an Starter, Szene, Revision und Snapshot gilt auch nach dem zweiten Abbruch. [Abnahme und Grenzen](docs/architecture/decisions/0036-native-win32-ui-role-evaluation.md#wiederanlauf-nach-abbruch-der-szenenübernahme)

Der M1-Schreibkern umfasst im langen synthetischen Klartexteditor jetzt die lokale Wortzählung, eine Anzeige der belegten UTF-16-Einheiten und den Fokusmodus. [Bedienung und Windows-Nachweis](docs/DEVELOPMENT_STATUS.md#verwalteter-win32-szeneneditor)

Für P3 wurde an einer synthetischen SQLite-Quelle geprüft, wie ein im WAL bestätigter Stand ohne Änderungen an den Quelldateien in einem getrennten Staging wiederhergestellt werden kann. [Befund und offene Voraussetzungen der Bestandskopie](docs/data-format/product-start-integration-v1-draft.md#vorprüfung-zur-sqlite-erfassung-bei-p3)

Der [synthetische P3-Quelldatenträger](docs/data-format/product-start-integration-v1-draft.md#stabiler-dateibestand-auf-einem-synthetischen-quelldatenträger) sichert den gesamten Projektbaum während der Kopie. Die Anlage neuer Assets wurde auch im Unterverzeichnis verhindert. Für gewöhnliche Bestandsordner steht die sichere Erfassungsgrenze weiter aus.

Die [zielgebundene synthetische Veröffentlichung](docs/data-format/product-start-integration-v1-draft.md#einmalige-veröffentlichung-am-gebundenen-synthetischen-ziel) erkennt eine bereits abgeschlossene Kopie nach Ausfall des Kopierprozesses wieder. Die synthetische Teilkopie kann auf ausdrücklichen Wiederanlauf mit derselben Autorität fortgesetzt werden. Ein bestätigtes Paket kann nach Prozessabbruch vor der Umbenennung veröffentlicht werden. Unklare Bestände bleiben gesperrt. Der synthetische Starter nutzt beim Wiederanlauf ein [installationsgebundenes Kontrolljournal](docs/data-format/product-start-integration-v1-draft.md#wiederanlauf-mit-installationsgebundenem-kontrolljournal). Die Bereinigungsentscheidung ist im synthetischen Journal umgesetzt. Entfernung, Wiederanlauf nach Unterbrechung und Abschluss sind für das kontrollierte synthetische Ziel umgesetzt. Offen bleibt die Übergabe einer veröffentlichten Kopie an den verwalteten Datenzugriff. Die Integration in eine reguläre Installation ist offen.

## Prüfstand für KQ-069 · 19. November 2026

Der [synthetische BP4b-Datenkanal](docs/data-format/native-win32-bp4-channel-v1-draft.md#abschluss-des-synthetischen-datenkanals) ist abgeschlossen. C4b bis C4d sind für das schreibgeschützte synthetische Win32-Fenster abgeschlossen. Der verwaltete Starter bindet Paket und Prozessgeneration unter Kontrollsperre und gibt das Fenster nach bestätigtem Bootstrap frei. Nach Starterabbruch blockiert der offene Sitzungsbeleg die nächste Fenstererzeugung. Die [bytegebundene C4d-Abnahme](docs/architecture/decisions/0036-native-win32-ui-role-evaluation.md#c4d-abschluss-für-den-synthetischen-vorschaupfad) enthält Paketidentität, Sicherheitsnachweise und Freigabeumfang. Der neue Einstieg `broker-store open-win32-scene <Kontrollkopf>` zeigt nach beiden Bootstrap-Quittungen genau eine vom isolierten Datendienst gelesene Szene A schreibgeschützt an. Der ETW-Netzvergleich erfasste im Referenzlauf und beim einmaligen Szenenabruf je null Netzwerkereignisse im KRAQEL-Prozessbaum. Nur im Szenenlauf startete der Datendienst. Der anschließende verwaltete Prüflauf `check-win32-scene-denied-save` bestätigte `DENIED` für einen gültigen Speicherversuch und unveränderten Text auf Revision 0 beim erneuten Lesen unter derselben Bindung. Unter eigener Edit-Bindung bestätigte `check-win32-scene-edit-conflict` die erste Speicherung, `CONFLICT` für den veralteten Schreibversuch und den gespeicherten Text auf Revision 1 im Arbeitsspeicher. Der dauerhafte Prüfpfad `save-win32-scene` und `read-saved-win32-scene` bestätigte nach Prozessneustart Text und Revision 1 im isolierten SQLite-Abbild; veränderte Dateibytes sperren das Wiederöffnen vor dem Rollenstart. Produktdaten und WebView2 bleiben gesperrt.

## Grobe Fortschrittseinschätzung · 17. November 2026

**Gesamtprojekt einschließlich M6/Version 0.2: ungefähr 65 %, plausible Spanne 60–70 %.** Der
bereits nutzbare Funktionsumfang für 0.1 liegt grob bei **80 %**. Das sind fachlich gewichtete
Schätzungen anhand der großen Arbeitsblöcke und ihrer offenen Integrations-/Abnahmerisiken, keine
gemessene Kennzahl, Aufgabenquote oder Zusage zum verbleibenden Zeitaufwand. Änderungen am Umfang
können die Schätzung verschieben. Insbesondere sind die vielen Forschungsunterpunkte innerhalb
KQ-069 Teil einer noch offenen Produktintegration.

Auch nach C3 samt frischem ETW bleibt diese Schätzung unverändert: Die synthetischen
Sicherheitsnachweise schließen wichtige Forschungsfragen, ersetzen aber weder die echte isolierte
Oberfläche noch Bestandsmigration, Launcher-/Installerintegration und Produktabnahme.

| Stufe    | Stand und wesentliche Restarbeit                                                                                                                                    |
| -------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| M0       | Fundament abgeschlossen.                                                                                                                                            |
| M1/M2    | Schreibkern, Story-Bibel, Plotboard und Timelines implementiert und im dokumentierten Kernumfang geprüft.                                                           |
| M3       | Import/Export, Git, Medien und visuelle Profile weitgehend nutzbar; Drehbuchmodell/-renderer, vollständige F08-/M3-Abnahme und normale Produktisolation offen.      |
| M4       | Offline-Kontinuität, Prompt-Entwürfe und Textproviderprotokolle vorhanden; reale Providerabnahme und optionale KI-Kontinuitätshinweise offen.                       |
| M5       | Alpha 4 veröffentlicht, Windows-Paket-/Smoke-/Assistenznachweise im dokumentierten Umfang; vollständige Produkt-, Zweitrechner-/Clean-VM- und Offlineabnahme offen. |
| M6 / 0.2 | Bildgenerierung, Referenzkonsistenz und Kartenmodell noch nicht begonnen.                                                                                           |

Aktuell innerhalb KQ-069: P1-Modell und P2a-Records sowie P2b-Probeworker samt eigenem
[ETW-Beleg](docs/testing/product-start-p2b-bootstrap-etw.md) vorhanden. **P2 insgesamt ist offen**:
Begrenzte [Ressourcen-Reconciliation](docs/testing/product-start-resource-reconciliation.md) ist
umgesetzt. Der
[frische Ressourcen-Netzbeleg](docs/testing/product-start-resource-reconciliation-etw.md) ist
ergänzt.
[Dauerhafte Workerbindung/Wiederanlauf vor Drained](docs/testing/product-start-worker-lifetime.md)
ist für den synthetischen Ein-Prozess-Worker umgesetzt. Der
[eigene frische Netzbeleg](docs/testing/product-start-worker-lifetime-etw.md) ist ergänzt. Der
[P2c-Vertrag](docs/data-format/product-start-p2c-v1-draft.md) ist als Entwurf ausgearbeitet, nicht
vollständig produktintegriert. [C1-Modell](docs/testing/product-start-p2c-model.md) mit Typen,
Inventarcodierung und Lebenszyklus ist geprüft; 13 neue Tests, insgesamt 88 aktive Helfertests.
[C2-Datenpfad](docs/testing/product-start-p2c-native-data.md) ist in frischen Schema-6-Stores
umgesetzt: persistentes Profil, native Root-/Paketbindung und eigene explizite Stilllegung ohne UI.
92 aktive Helfertests, zehn Controlleraufrufe bestanden.
[Frisches C2-ETW](docs/testing/product-start-p2c-native-data-etw.md) ist ergänzt: 15 Controller/15
Konsolenhosts, 30 Enden, acht Kontrollen und null beobachtete Netzereignisse.
[C3-Synthesekern](docs/testing/product-start-p2c-group-bootstrap.md) ist umgesetzt: 96 aktive
Helfertests, 22 Controlleraufrufe und ein gesperrter Wächterabbruch.
[Frisches C3-ETW](docs/testing/product-start-p2c-group-bootstrap-etw.md) bestätigt 28 Controller,
zwölf Root-/Kindrollen, 68 explizite Enden und acht Kontrollen ohne beobachtetes Netz oder
Erfassungsverluste. Der [C4-Vertrag](docs/data-format/product-start-p2c-c4-v1-draft.md) für
Runtime-Paket und minimale isolierte Oberfläche ist als Entwurf ergänzt.
[C4a-Modellkern](docs/testing/product-start-p2c-c4a-model.md) implementiert reine
Paket-/Policytypen, deterministische Codierung und getrennten versionierten Lebenszyklus; zwölf neue
Tests, 108 aktive Helfertests/Clippy bestanden.
[C4b-Vorprüfung](docs/testing/product-start-p2c-c4b-preflight.md),
[Root-/WebView-Probe](docs/testing/product-start-p2c-c4b-root-bootstrap.md),
[Beobachtung mit eigenem ETW](docs/testing/product-start-p2c-c4b-observation.md) und
[verbindliche Modulsperre/Crashpad-Diagnose](docs/testing/product-start-p2c-c4b-module-gate.md) sind
ergänzt. 119 aktive Tests im Vollsatz, danach 23 gezielte C4-Tests; neue native Sperre stoppt den
authentisierten Root vor Continue/WebView. Beide zusätzlichen Modulpfade abgewiesen, 13 MAC-Records
unabhängig geprüft.
[Frisches ETW des Sperrpaars](docs/testing/product-start-p2c-c4b-module-gate-etw.md) bestanden: acht
explizite Enden/acht Kontrollen, keine Verluste/Zuordnungslücken oder beobachteten Netzereignisse in
diesen Bäumen; 27 Auswertungstests erneut grün, kein neuer Build.
[Präventiver Modul-/Rollenvertrag](docs/data-format/product-start-p2c-c4-preload-v1-draft.md) jetzt
als nativer Zielvertrag ergänzt. [BP1](docs/testing/product-start-p2c-bp1-model.md) implementiert
inzwischen reine Typen, vier deterministische Modellformate und nicht autorisierendes Modell: 14
neue Tests/133 aktive Helfertests und Clippy bestanden.
[BP2-Blattversuch samt eigenem ETW](docs/testing/product-start-p2c-bp2-native-leaf.md) ist
inzwischen geprüft. Die
[BP3-Machbarkeitsprüfung/Startsperre](docs/testing/product-start-p2c-bp3-admission.md) ist ergänzt:
138 aktive Tests, frühe Abweisung auch gültiger C4-Kandidaten ohne belegten Mechanismus. BP3
insgesamt offen. Die
[synthetische Debug-Gegenprobe](docs/testing/product-start-p2c-bp3-debug-guardian.md) ist inzwischen
durchgeführt: neun Jobprozesse in drei Fällen, selbst eröffnete Debugkette entzieht dem äußeren
Wächter das Kindereignis. Debugvererbung allein ungeeignet, 141 aktive Tests bestanden. Nächster
Scope ist ein Architekturentscheid für unabhängig durchgesetzte Erzeugungs-/Ladebindung; keine
Runtime-/Datenfreigabe. C4b bleibt offen: vollständige Runtime-/Nichtsystemmodul-/effektive
Rollenbindung vor Datenfreigabe und effektive Datenrechte; ungebundene Starts, mehrprozessige
GUI-Lebenszeiten und persistente echte Arbeitsablagen-/UI-Bindung bleiben offen. P3 Bestandskopie
sowie P4/P5 normale App-/Launcher-/Installerintegration und Produktabnahme folgen. Die normalen
Produkt-Q-01/Q-02 bleiben offen; der neue Forschungsnetzbeleg ersetzt dieses Gate nicht. Das strikte
Datenschutzziel bleibt unverändert.

## M0 - Fundament (abgeschlossen)

- [x] Monorepo, Tauri-2-Shell, React, TypeScript und Rust-Workspace
- [x] Grundlegende CI, Sicherheits- und Beitragsdokumentation
- [x] Domänenmodell und typisierte Tauri-Commands
- [x] SQLite-Schema, Migrationen und FTS5-Migrationstests
- [x] Lokaler Projektmanager, Backup, Path-Safety und Recovery
- [x] Lokale Projekte auf der Startseite zweistufig und UUID-geprüft löschen
- [x] Ausgewählten KolkrabeofDoom als lokale, responsive Autorenvignette am Startseiten-Wortzeichen
      einbetten

## M1 bis M5 - Version 0.1

1. [x] Editor-Grundfunktionen, verwalteter Szenenbaum, Autosave und Backup
   - [x] Kapitel und Szenen erstellen, umbenennen, umordnen und explizit löschen
2. M2-Strukturwerkzeuge
   - [x] Story-Bibel mit Entitäten, Beziehungen, Suche und Szenen-Rückverweisen
     - [x] Browser-E2E für den fachlichen Story-Bibel-Kernablauf
     - [x] Doppelte Einträge derselben Art atomar samt Referenzen zusammenführen
   - [x] Plotboard und Timeline
     - [x] Getrennte Browser-E2E für Plotboard- und Timeline-Kernabläufe
3. M3-Datenportabilität und Medien
   - [x] Deterministische Projektsnapshots und lokales Git für Status, Diff und Commit
     - [x] Browser-E2E für Snapshot, Status, Diff und expliziten Commit
     - [x] Aktuellen Snapshot über nativen Dialog als begrenztes `.kraqel`-Archiv exportieren
   - [x] Export nach Markdown, HTML, DOCX und PDF für die dokumentierte Formatierungsmenge
     - [x] Browser-E2E für Formatwahl, Dateiname und sichtbare Größenbestätigung
     - [x] Projektgebundenen Exportordner direkt und pfadsicher aus der Oberfläche öffnen
   - [x] Ausgewählte Szenenbilder in gewöhnlichen Manuskriptexporten ausgeben
     - [x] Medienexport-Vertragsentwurf mit ausdrücklich gewählter Platzierung am Szenenende,
           Formatmatrix, Herkunft, Grenzen und Abnahme ausarbeiten (KQ-062 / angenommene ADR 0029)
     - [x] Vertragsregeln/Parameter fachlich freigeben und Markdown-/HTML-Manifestgrammatik fixieren
     - [x] Gemeinsamen nativen Medienexport-Kern und Markdown-/HTML-Pakete implementieren und
           abnehmen (KQ-063 / ADR 0030)
     - [x] Gewöhnliches DOCX mit eingebetteten Szenenbildern und Office-Nachweis ergänzen
       - [x] Nativen Bildwriter, App-Aktion und Manifestbezug auf interne Parts ergänzen (KQ-064)
       - [x] Word-/LibreOffice-Abnahme abschließen (beide bestanden; KQ-064)
     - [x] PDF-Schrift-/Unicode-Voraussetzung und bebilderten PDF-Export abnehmen (KQ-065 /
           ADR 0032)
     - Vertrag: `docs/data-format/ILLUSTRATED_MANUSCRIPT.md`; alle vier Formate im dokumentierten
       Teilumfang
   - [x] Begrenzter Import und DOCX-Manuskriptaustausch im dokumentierten Umfang
     - [x] Snapshot-v3-Archiv sicher als neues Projekt importieren und Beispielprojekt einbetten
     - [x] Frei wählbare Snapshotarchive über einen nativen Dialog als neue Projektkopie importieren
     - [x] Begrenzte UTF-8-/Markdown-Dateien über einen nativen Dialog als neue Szene importieren
     - [x] Nativen Textdialog mit gültigem TXT/Markdown, Abbruch, ungültigem UTF-8 und Übergröße
           prüfen
     - [x] DOCX-Import und begrenzte Roundtrip-Garantien definieren und implementieren
       - [x] Begrenzten DOCX-Import als neue Szene mit expliziter Ablehnung nicht unterstützter
             Inhalte umsetzen
       - [x] Konkreten Roundtrip-Vertragsentwurf mit Strukturmarkern, Office-Grenzen und Abnahme
             ausarbeiten (KQ-057; Repräsentation inzwischen mit ADR 0026 festgelegt)
       - [x] Verschachtelten Strukturprototyp mit echten Word-/LibreOffice-Speichervorgängen und
             Word-Textänderung prüfen (KQ-058; Word positiv, LibreOffice-Strukturverlust)
       - [x] Flache Repräsentation und erhaltenes Inventar durch Speichern und Textbearbeitung in
             Word/LibreOffice nachweisen (KQ-059 / ADR 0026)
       - [x] Gemeinsames kanonisches Manuskriptmodell, produktive Versionsgrammatik und nativen
             Rust-Reader/Writer mit vollständiger Paketvalidierung implementieren (KQ-060 /
             ADR 0027)
       - [x] Getrennten Austauschmodus in die App einbinden und neue Projektkopie atomar importieren
             (KQ-061 / ADR 0028)
       - Importvertrag: `docs/data-format/DOCX_IMPORT.md`; nativer Austauschvertrag:
         `docs/data-format/DOCX_ROUNDTRIP.md`; ursprünglicher Roundtrip-Vorschlag:
         `docs/specifications/DOCX_ROUNDTRIP_CONTRACT_DRAFT.md`
   - [ ] F08-/M3-Gesamtumfang fachlich entscheiden und vollständig abnehmen
     - [x] Konkreten Drehbuchvertragsvorschlag und vollständige Kriterienmatrix ausarbeiten (KQ-066
           / fachlich angenommene ADR 0033; keine Funktionsabnahme)
     - [x] Drehbuchrollen, A4-Profil, Schriftauswahlprüfung und Format-/Importrichtungen fachlich
           entscheiden
     - [ ] Nach Freigabe Modell-/Versionsprototyp und Schriftprüfung, danach Renderer und
           DB-01–DB-08
     - [x] Funktionalen nativen Gesamtlauf M3-G01 mit 100 Szenen/50 Bildern, Neustart,
           Backup/Recovery, Snapshot-Neuimport, Umbenennung, Backlinks und sichtbaren Assetfehlern
           durchführen (KQ-067)
     - [ ] Unabhängigen Desktop-Netzverkehrs-/Offline-Gesamtnachweis, assistive Vollabnahme und
           verbleibende native Medien-CRUD-Prüfungen gemäß Matrix ergänzen
       - [x] ETW-Netzmessung mit App-/WebView2-Zuordnung und Kontrollen durchführen; öffentlicher
             WebView2-Verkehr gefunden, keine bestandene Netzfreigabe (KQ-068)
       - [x] Native CRUD-Teilprüfung und Rust-Datei-/Referenztest ergänzen; Status-/Fehleransagen
             und Linkwählerbeschriftung korrigieren (KQ-068)
       - [x] Manuellen kanonischen Bildwechsel, bestätigte GUI-Löschfolge und Neustart abnehmen
             (KQ-068 / #133)
       - [x] Vier bebilderte Exporte nach Löschung/Neustart per Tastatur erzeugen und rücklesen
       - [x] Abschließende Lösch-/Export-Hörprobe mit Nutzerbestätigung abnehmen (KQ-068)
       - [ ] Weitere allgemeine assistive F08-Randfälle ergänzen
       - [ ] WebView2-Netzbefund klären und Laufzeitgrenze ausdrücklich entscheiden; Q-01/Q-02 erst
             anhand neuer Evidenz freigeben (KQ-069 / #134)
       - [x] KQ-069: NetLog-Ursachendiagnose mit leerem Projektbestand und synthetischem
             Export-/Importlauf; unterstützte Maßnahmen und Grenzen dokumentieren
       - [x] KQ-069: ETW mit drei vollständigen App-Läufen, Dokument-CRUD, Medienimport/-Metadaten
             und Export/Neuimport wiederholen; gültige Messung bestätigt öffentlichen Verkehr
       - [x] KQ-069 A: AppContainer-Machbarkeitsprototyp mit Tokenprüfungen, lokalem Kernablauf und
             ETW-Vergleich (0 außerhalb Loopback gegenüber 201 öffentlichen Kontrollereignissen)
       - [x] KQ-069 A: Synthetischen Datei-/Shell-Austausch mit zwei Handlefreigaben statt
             Hostordner-ACL, nativer Kopie, Fehlerpfadtests und begrenzter ETW-Prüfung belegen
       - [x] KQ-069 A: Typisierten einmaligen Dateiauftrag an isolierten App-Dienst-Testadapter
             anbinden; Snapshotimport mit Abbruch/Ablauf vor Veröffentlichung atomar prüfen
       - [x] KQ-069 A: Separates UI-/Tauri-Testfeature mit vorab delegiertem Einweg-Dateikanal,
             Origin-/Fensterprüfung und einmaligem Rust-Grant implementieren
       - [x] KQ-069 A: Finalen UI-Build mit Import/Wiederholung, Abbruch und Fensterende am
             Haltepunkt nativ/ETW prüfen; drei vollständige Bäume ohne Ereignisse außerhalb Loopback
       - [x] KQ-069 A: Native Vorab-Dateiauswahl und Dialogabbruch funktional prüfen; neuer
             Windows-Build, Unicode/SQLite und Cleanup unabhängig bestätigt, kein neuer ETW-Lauf
       - [x] KQ-069 A: Synthetische Auswahl aus laufender Forschungs-App und Dialogabbruch
             implementieren und funktional nativ prüfen; SQLite/Unicode und Cleanup bestätigt
       - [x] KQ-069 A: Replay nach Erfolg/Fehler, Fensterende am Publish-Haltepunkt und ETW des
             korrigierten Forschungsbuilds prüfen; drei vollständige Bäume ohne Nicht-Loopback
       - [x] KQ-069 A: Appende bei offenem Dateidialog und nach nachgewiesenem Teiltransfer prüfen;
             Auswahlthread und Cleanup bestätigt, Controller-Buildgrenzen dokumentiert
       - [x] KQ-069 A: Frischen ETW-Vergleich des finalen Fensterende-Builds durchführen; drei
             vollständige Bäume, vier Kontrollen, Nullverluste, 0 Nicht-Loopback-Ereignisse
       - [x] KQ-069 A: Assistive Erfolg-/Fehlerpfade mit Replay und Fokus nativ prüfen; begrenzte
             lokale Diagnose und 34 Browser-E2E, keine Vollabnahme oder neuer ETW-Beleg
       - [x] KQ-069 A: Dauerhaften Verbrauch und konservative Reconciliation im Forschungsadapter
             mit sieben Prozess-Kills prüfen; isolierte GUI mit Erfolg/Abbruch/Replay rückprüfen
       - [x] KQ-069 A: Authentisierten isolierten GUI-Neustart innerhalb desselben Controllers mit
             stabiler Ablage prüfen; 28 Generationen mit Crash/zweifachem
             Wiederanlauf/Altimportabwehr
       - [x] KQ-069 A: Gesichertes Neustart-Paar frisch per ETW messen; 28 App-Generationen, 144
             WebViews, vier Kontrollen, Nullverluste und 0 Nicht-Loopback-Ereignisse
       - [x] KQ-069 A: Startticket v2 und manuelle Wiederanlaufprüfung mit Windows-Sprachausgabe
             ergänzen; Ansagen/Fokus/Replay und zusätzliche Tastaturnutzung bestätigen
       - [x] KQ-069 A: Gesichertes v2-Paar mit Automatik und manuellen Pfaden per ETW messen; 32
             App-Generationen, vier Kontrollen, Nullverluste, 0 Nicht-Loopback-Ereignisse
       - [x] KQ-069 A: Dauerhafte native Wiederanlaufautorität im Forschungsbroker und echte
             Controller-Crash-Serie prüfen; sechs Kills, zwölf Wiederanläufe, 60 Ablehnungen
       - [x] KQ-069 A: DPAPI-Domänengrenze durch lokalen HMAC-Beleg ersetzen; neue Serie mit 78
             Ablehnungen, privater Schlüsselverwahrung und unabhängiger MAC-/Datenprüfung
       - [x] KQ-069 A: Neues HMAC-Brokerpaar frisch per ETW messen; 446 explizite Enden, vier
             Kontrollen, Nullverluste, 0 Nicht-Loopback-Ereignisse, native Daten/MACs bestätigt
       - [x] KQ-069 A: Kombinierten Forschungs-Live-/Neustartpfad mit signierter Grantfolge prüfen;
             neun Fälle/72 App-Starts, 54 abgewiesene Folgeaktionen, manuelle Dateiauswahl bestätigt
       - [x] KQ-069 A: Gesichertes kombiniertes v3-Paar einschließlich nativer Dateiauswahl frisch
             per ETW messen; 868 explizite Enden, acht Kontrollen, Nullverluste, 0 Nicht-Loopback
       - [x] KQ-069 A: Kombinierte assistive Wiederanlauf-/Fehlerzustände mit Hörbestätigung und
             nativem Replay-Gegenbeleg prüfen; zwei Erstversuche abgelehnt, Tastaturabnahme separat
       - [x] KQ-069 A: Tastaturprüfung und eigenes frisches ETW des manuellen Brokerpaars; vier
             Zustände, acht Kontrollen, Nullverluste, alle 906 Enden, 0 Nicht-Loopback
       - [x] KQ-069 A: Kombinierter Hauptfensterschluss bei offenem Dialog und Teiltransfer;
             reguläre Enden, Recovery/Replay und neue Aufträge geprüft, 16 Apps/zwei legitime Kopien
       - [x] KQ-069 A: Eigener frischer ETW-Netzvergleich des Fensterende-Paars einschließlich
             beider Pfade; acht Kontrollen, Nullverluste, alle 942 Enden, 0 Nicht-Loopback
       - [x] KQ-069 A: Native private Kontrollablage-Vorschau mit lokalen autorisierten EXE-Wechseln
             und Kontrollschema 1 → 2; 30 native Aufrufe/18 Container-Dateiverweigerungen
       - [x] KQ-069 A: Kontroll-Crash-Serie und eigener frischer ETW-Vergleich; 28 Abbrüche, 17
             gesperrte/elf nutzbare Zustände, acht Kontrollen, 1.136 Enden, 0 Nicht-Loopback
       - [x] KQ-069 A: Synthetischen isolierten Startpfad an Kontrollsperre/Produktjob binden; zwei
             native Starts/zwölf private Dateiverweigerungen, Daten-/MAC-Audit
       - [x] KQ-069 A: Dauerhafte synthetische Apppaket-/Sitzungsbindung und authentisierter
             Wiederanlauf unter Kontrollsperre; zwei Controllerabbrüche/vier Recoveries, kein
             Duplikat
       - [x] KQ-069 A: 20 zusätzliche Sitzungsübergangs-Crashpunkte und frisches ETW; neun
             gesperrte/ elf authentisierbare Zustände, acht Kontrollen, 458 explizite Enden, 0
             Nicht-Loopback
       - [x] KQ-069 A: Authentisierte Profilbereinigung unter Kontrollsperre; Kontrollschema 4, zwei
             gezielte native Fälle, idempotenter Abschluss ohne neue Importautorität
       - [x] KQ-069 A: Gezieltes Cleanup-ETW des unveränderten Paars; acht Kontrollen, 140 explizite
             Enden, 0 Nicht-Loopback, alle 19 direkten Kontrollaufrufe erfasst
       - [x] KQ-069 A: Lokale Schema-4-Brokerupdates mit read-only Paketvertrag; gezielter nativer
             Actorwechsel/Recovery/Cleanup ohne Schlüssel-/Bindungs-/Datenreset
       - [x] KQ-069 A: Frisches ETW beider Updateprogramme einschließlich fünf hashbenannter
             verwalteter Starts; acht Kontrollen, 132 explizite Enden, 0 Nicht-Loopback
       - [x] KQ-069 A: Gezielte Update-Abbruch-/Fehlerreconciliation unter derselben Sperre;
             Teilkopie/Manipulation/dauerhafter Abort, drei native Fälle, keine Replaykopie
       - [x] KQ-069 A: Authentisiert abgebrochene Paketartefakte unter derselben Sperre sicher
             ersetzen/erneut vorbereiten; Kollisionen ohne neues Pending, drei native Retryfälle
       - [x] KQ-069 A: Frisches ETW des Retry-Brokers samt verwaltetem Kandidaten; 21 direkte
             Aufrufe, acht Kontrollen, 144 explizite Enden, 0 Nicht-Loopback
       - [x] KQ-069 A: Drei gezielte Prozessabbrüche zwischen Artefaktentfernung, Zielanlage und
             Prepare; authentisierter Retry/Recovery ohne zusätzliche Projektkopie
       - [x] KQ-069 A: Frisches ETW der Prepare-Übergänge und des verwalteten Kandidaten; 59 direkte
             Aufrufe, acht Kontrollen, 182 explizite Enden, 0 Nicht-Loopback
       - [x] KQ-069 A: Start-/Integrationsvertrag v1 als Entwurf mit belegten Lücken und Stufen
             P1–P5; keine Runtime-/Migrationsfreigabe
       - [x] KQ-069 A: P1 reine versionierte Produktstart-/Bindungstypen und Zustandsregeln als
             Testmodell; sieben neue Negativ-/Lebenszyklustests, keine Start-/Datenautorität
       - [x] KQ-069 A: P2a authentisierte Schema-5-Workspace-/Launch-Records in synthetischen
             privaten Wurzeln ohne GUI; drei Abbruchfälle/Replayabwehr, keine Startcapability
       - [x] KQ-069 A: P2b-Probeworker ohne GUI/Importtask, authentisierter Bootstrap, suspendierte
             Token-/Jobprüfung und kontrollierter Abschluss; drei gezielte native Fälle
       - [x] KQ-069 A: P2b gezielter frischer ETW-Beleg für Controller/Worker, 68 explizite
             Prozessenden und acht Kontrollen, null beobachtete Netzereignisse in den Messbäumen
       - [x] KQ-069 A: Begrenzte Probeprofil-Reconciliation bei belegter Eigentums-/Lebenszeitlage;
             vier gezielte Crashfälle, unbekannte Profile und unbewiesene Workerenden gesperrt
       - [x] KQ-069 A: Frisches ETW des Ressourcen-ABI-2-Builds; 54 Controller/sechs Worker, acht
             Kontrollen/114 explizite Enden, null beobachtete Netzereignisse in den Messbäumen
       - [x] KQ-069 A: Dauerhafter Workerstartbeleg/Wiederanlauf vor Drained im
             Ein-Prozess-Prototyp; drei gebundene Starts reconciliert, ungebundener Start
             unverändert gesperrt
       - [x] KQ-069 A: Frisches ETW des Workerbindungs-v3-Builds mit
             Kernel-Erzeugungszeit-/Endabgleich; 52 Controller/acht Worker, 112 Enden/acht
             Kontrollen, null beobachtete Netzereignisse
       - [x] KQ-069 A: [P2c-Vertragsentwurf](docs/data-format/product-start-p2c-v1-draft.md) für
             persistente Arbeitsdaten/Pakete/Mehrprozessstart ausgearbeitet; keine Implementierung
       - [x] KQ-069 A:
             [C1 reine P2c-Typen/Inventarcodierung/Lebenszyklusmodell](docs/testing/product-start-p2c-model.md)
             implementiert; 13 neue Tests, 88 aktive Helfertests/Clippy grün; keine native Autorität
       - [x] KQ-069 A:
             [C2 persistente synthetische Ablage/Profil/Paketbindung](docs/testing/product-start-p2c-native-data.md)
             mit frischen Schema-6-DATA-Records; 92 aktive Helfertests und zehn Controlleraufrufe
       - [x] KQ-069 A: [frisches C2-ETW](docs/testing/product-start-p2c-native-data-etw.md) des
             Builds `73a7aad…`; 15 Controller/15 Konsolenhosts, 30 Enden/acht Kontrollen, null
             beobachtetes Netz
       - [x] KQ-069 A:
             [C3 synthetischer Wächter/Mehrprozess-Bootstrap](docs/testing/product-start-p2c-group-bootstrap.md)
             mit echtem Kind/Root-Ende, 96 aktiven Helfertests, 22 Controlleraufrufen und gesperrtem
             Crash
       - [x] KQ-069 A: [frisches C3-ETW](docs/testing/product-start-p2c-group-bootstrap-etw.md) des
             unveränderten Builds `b24b3aa…`, einschließlich Root-/Kindrollen und gesperrtem
             Wiederanlauf: 68 Enden, acht Kontrollen, null beobachtete Netzereignisse
       - [x] KQ-069 A: [C4-Vertragsentwurf](docs/data-format/product-start-p2c-c4-v1-draft.md) für
             explizites Runtime-Paket und minimale isolierte Oberfläche; keine native Umsetzung
       - [x] KQ-069 A: [C4a-Modellkern](docs/testing/product-start-p2c-c4a-model.md): reine
             Paket-/Policytypen, deterministische Codierung und versionierter Lebenszyklus; zwölf
             neue Tests, 108 aktiv bestanden, keine native UI-/Startautorität
       - [x] KQ-069 A: [C4b-Teilstand](docs/testing/product-start-p2c-c4b-preflight.md), native
             Vorprüfung und nie fortgesetzte Controller-Geschwister; Runtimekandidat vorbereitet,
             kein WebView-Start-/Kindnachweis
       - [x] KQ-069 A:
             [C4b-Root-/WebView-Probe](docs/testing/product-start-p2c-c4b-root-bootstrap.md),
             authentisierter Bootstrap und tatsächlicher leerer Kindpfad; 115 aktive Tests, keine
             Produkt-/UI-Freigabe
       - [x] KQ-069 A:
             [C4b-Rollen-/Moduldiagnose und gezieltes ETW](docs/testing/product-start-p2c-c4b-observation.md),
             117 aktive Tests; zusätzliche Module ausdrücklich keine automatische Policyausnahme
       - [x] KQ-069 A:
             [C4b-Modulsperre/Crashpad-Diagnose](docs/testing/product-start-p2c-c4b-module-gate.md),
             keine neue Runtime-/Textfreigabe; beide zusätzlichen Modulpfade abgewiesen
       - [x] KQ-069 A:
             [gezieltes frisches ETW des C4b-Sperrpaars](docs/testing/product-start-p2c-c4b-module-gate-etw.md),
             acht Enden/acht Kontrollen ohne beobachtetes Netz, kein neuer Build
       - [x] KQ-069 A:
             [Vertragsentwurf präventive Modul-/Rollenbindung](docs/data-format/product-start-p2c-c4-preload-v1-draft.md),
             keine Implementierung oder native Freigabe
       - [x] KQ-069 A: [BP1 reiner Präload-Modellkern](docs/testing/product-start-p2c-bp1-model.md),
             Typen/Codierung/Entscheidungen ohne native Autorität, 14 neue Tests/133 insgesamt
       - [x] KQ-069 A:
             [BP2 nativer Blattversuch und eigenes ETW](docs/testing/product-start-p2c-bp2-native-leaf.md),
             Readback/Kindablehnung 367, drei neue Tests/136 insgesamt, kein Runtime-/Datengrant
       - [x] KQ-069 A:
             [BP3-Vorprüfung und native Startsperre](docs/testing/product-start-p2c-bp3-admission.md),
             zwei aktive Regressionen/138 Tests, keine positive präventive Bindung oder neues ETW
       - [x] KQ-069 A:
             [BP3 synthetische Debug-Wächter-/Vererbungsgegenprobe](docs/testing/product-start-p2c-bp3-debug-guardian.md),
             drei native Fälle/141 aktive Tests, Debugvererbung allein als Durchsetzung abgelehnt
       - [x] KQ-069 A:
             [BP3 unabhängige Prozess- und Imagefreigabe bewertet](docs/architecture/decisions/0035-bp3-independent-process-and-image-enforcement.md).
             Erzwungene UMCI und eigener Erzeuger als synthetische Referenz im Offline-Gast
             ausgewählt.
       - [x] KQ-069 A:
             [Starter, Testimages und Policyvorschau für die synthetische Gastprobe](docs/testing/product-start-p2c-bp3-umci-guest.md)
             vorbereitet und auf dem Host geprüft.
       - [x] KQ-069 A: Separaten Hyper-V-Gast ohne virtuelle Netzadapter und mit Rücksetzpunkt für
             die synthetische Probe bereitgestellt.
       - [x] KQ-069 A: Vorhandene synthetische Gastprobe mit erzwungener UMCI und eigenem Starter
             bestanden. Zwei Rollen, ein EXE- und vier DLL-Sperrereignisse, Job und Testprofil
             bereinigt.
       - [x] KQ-069 A: Statischer DLL-Import im isolierten Blattprozess mit Positivkontrolle,
             Loaderstatus und CI-Ereignis geprüft.
       - [x] KQ-069 A: Starterabbruch bei laufendem Blattprozess im Gast geprüft, Prozessende und
             leerer Zähljob bestätigt.
       - [x] KQ-069 A: Wiederanlauf bei fehlender Richtlinie und geänderter Richtlinienversion mit
             identischer ID jeweils vor dem Starter gesperrt.
       - [x] KQ-069 A: Effektive Markerrechte vor Resume für Leserolle und leere Rolle geprüft,
             Negativkontrolle mit Attributrechten vor Ausführung abgewiesen.
       - [x] KQ-069 A: Frischer ETW-Netzvergleich des Prüfpakets mit Handleprüfung im Offline-Gast.
             Beide Prozessbäume ohne beobachtete TCP-/UDP-Ereignisse, je vier Positivkontrollen.
       - [x] KQ-069 A: Tatsächliche WebView2-Elternkette und Win32-Schnittstellen des Kandidaten
             geprüft. Browser- und verschachtelte Crashpad-Kindstarts liegen im Runtimebaum.
       - [x] KQ-069 A: Öffentliche Microsoft-SDKs (stabil und Vorschau) und Herstellerdokumentation
             auf ein Startveto für verschachtelte Crashpad-Prozesse und spätere Renderer geprüft.
             Für beide Fälle ist kein unterstützter Freigabepunkt dokumentiert (ADR 0035).
       - [x] KQ-069 A: Native Win32-Oberfläche mit eigener UI-Rolle als Alternative bewertet.
             Statischer Linkversuch für ein Unicode-Textfeld bestanden (ADR 0036).
       - [x] KQ-069 A: Natives UI-Blatt im interaktiven Offline-Gast mit Rollenbindung vor Resume,
             Startablehnung, Tastatur, Narrator-Sprachverlauf, Prozessende und ETW geprüft.
       - [x] KQ-069 A: Softwareseitige Sprachausgabe der Win32-Probe anhand von Fokus und
             Narrator-Sprachverlauf abgenommen.
       - [ ] KQ-069 A: Akustische Hörprüfung am isolierten Audioausgang zurückgestellt.
       - [x] KQ-069 A:
             [BP4-Vertrag für synthetische Win32-Szenen](docs/data-format/native-win32-bp4-scene-v1-draft.md)
             mit getrennten UI-/Datenrechten und revisionsgebundenem Speichern festgelegt.
       - [x] KQ-069 A: BP4a als reines Rust-Modell geprüft: exklusive Szenenbindung,
             Lese-/Editrolle, revisionsgebundenes Speichern und Abweisung fremder oder beschädigter
             Inhalte.
       - [x] KQ-069 A: BP4b-Kanalbindung und Replayabwehr für das synthetische Rollenpaar
             spezifiziert.
       - [x] KQ-069 A: BP4b-Framecodec und Replaymodell mit BP4a-Szenenzugriff offline geprüft.
       - [x] KQ-069 A: Effektive Pipe-Endpunktrechte und Fremdprozessabweisung vor Resume im
             Offline-Gast geprüft.
       - [x] KQ-069 A: Gebundenen Kanalbootstrap und HELLO/READY mit echter Win32-UI, Datendienst
             und abweichendem Antwortschlüssel im Gast geprüft.
       - [x] KQ-069 A: Eine gebundene synthetische Szene nach HELLO/READY bis zum schreibgeschützten
             Win32-Feld gelesen.
       - [x] KQ-069 A: Schreibversuch unter derselben nativen Lesebindung mit DENIED abgewiesen,
             Text und Revision 0 erneut gelesen.
       - [x] KQ-069 A: Speichern im synthetischen In-Memory-Katalog unter Edit-Bindung,
             Revisionskonflikt und Endstand auf Revision 1 im Gast bestätigt.
       - [x] KQ-069 A: Szene A in isolierter SQLite-Testablage dauerhaft gespeichert, nach Neustart
             beider Rollen und auf der ausgeschalteten Gastplatte bestätigt.
       - [x] KQ-069 A: Datendienstabbruch vor und nach Commit im Gast geprüft.
             [Wiederöffnen und unabhängige Plattenprüfung](docs/testing/native-win32-commit-crashes.md) bestätigen Revision 0 bzw. 1.
       - [x] KQ-069 A: Beide Abbruch- und Wiederöffnungspfade mit frischem ETW-Netzvergleich geprüft.
             Je Lauf vier Positivkontrollen, null Verluste und null Netzwerkereignisse im Prüfprozessbaum.
       - [x] KQ-069 A: [Partieller Schreibframe und Kanal-Timeout](docs/testing/native-win32-channel-interruptions.md)
             im Offline-Gast geprüft. Beide frischen Lesepaare bestätigten Revision 0.
       - [x] KQ-069 A: [Fensterende bei laufender Anfrage und frischer ETW-Netzvergleich](docs/testing/native-win32-window-close.md).
             Revision 0, leere Jobs und null Netzwerkereignisse im Prüfprozessbaum bestätigt.
       - [x] KQ-069 A: [Starterabbruch ohne Endebeleg](docs/testing/native-win32-starter-restart.md) geprüft.
             Die synthetische Ablage blieb bei drei Neustartversuchen gesperrt.
       - [x] KQ-069 A: Frischer ETW-Netzvergleich für Starterabbruch und gesperrten Wiederanlauf.
             Alle drei Neustarts ohne Kindprozess, null Netzwerkereignisse im Prüfprozessbaum.
       - [x] KQ-069 A: [Freigabebedingungen nach Starterabbruch](docs/data-format/native-win32-bp4-channel-v1-draft.md#freigabeentscheidung-nach-starterabbruch) spezifiziert.
       - [x] KQ-069 A: [Generationsgebundener Endebeleg und einmalige Lesefreigabe](docs/testing/native-win32-release-proof.md) auf neuer synthetischer Ablage bei überlebender Kontrollinstanz geprüft.
       - [x] KQ-069 A: Frischer ETW-Netzvergleich für Belegausstellung, einmalige Freigabe und verweigerten Lesezugriff. Beide Prüfprozessbäume ohne Netzwerkereignisse.
       - [ ] KQ-069 A: Durchsetzbaren Erzeugungsweg für jede Runtimegeneration belegen. Der geprüfte
             WebView2-Win32-Einstieg erfüllt BP3 nicht. Produktive BP4-Datenrechte bleiben offen.
       - [ ] KQ-069 A: C4b vollständige Runtime-/Nichtsystemmodul-/Rollenbindung, wirksame native
             Rollenbindung und effektive Datenrechte; neue Bytes erneut messen; anschließend C4c/d
             WebView/SQLite unter belegtem Rollen-/Token-/Jobvertrag getrennt integrieren;
             ungebundene Starts und vollständige Live-Worker-Crashbereinigung und persistente echte
             Arbeitsablagen-/UI-Bindung; danach P3–P5 isolierten Produktstarter/echte
             Projektmigration integrieren sowie weitere allgemeine Fensterende-/F08-Prüfung;
             dauerhaften authentisierten Produktbroker integrieren/abnehmen; weitere
             Formate/Git-/Providergrenzen; UDP/IPv6/DNS/Broker kalibrieren; Packaging und
             F08-/M3-Vollprüfung
       - [ ] KQ-069: Strikte Isolation in normalen Start/Installer integrieren und vollständig mit
             ETW abnehmen; keine fachliche Runtime-Ausnahme (ADR 0034)
     - Vertrag: `docs/specifications/SCREENPLAY_FORMAT_CONTRACT_DRAFT.md`; Nachweise und offene
       Zeilen: `docs/testing/F08_M3_ACCEPTANCE_MATRIX.md`
   - [x] Lokale Medienbibliothek mit Metadaten, Links, Backup und Snapshot-Assets
     - [x] Browser-E2E für PNG-Import, Metadaten und Szenenverknüpfung
     - [x] Medienkarten bei schmalem Fenster und hohem Zoom ohne abgeschnittene Inhalte darstellen
   - [x] Visuelle Profile und projektweite Stilvorlagen
     - [x] Eigenständiger Browser-E2E für Profil-, Stil- und Geltungsbereichspersistenz
4. Optionale Text-KI, Kontinuitätsprüfung, Prompt-Entwürfe und Bildverträge
   - [x] Providerunabhängige Textgrenze und lokales Ollama-Streaming mit Kontextfreigabe
   - [x] Cloud-Textprovider und Betriebssystem-Schlüsselspeicher (Protokolltests; Live-Abnahme
         offen)
   - [x] Deterministische Kontinuitätsprüfung mit Quellenankern und Nur-Lese-Diff
     - [x] Browser-E2E für Offline-Befund, Regelkennung, Zeichenanker und Manuskript-Rücksprung
   - [x] Lokale, editierbare Bildprompt-Entwürfe mit UTF-8-Export und Snapshot
     - [x] Browser-E2E für den lokalen Bildprompt-Kernablauf
   - [x] Provider- und modellspezifische Bild-Capability-Registry sowie Generation-Job-Verträge
   - [ ] Optionale KI-Kontinuitätshinweise
5. Reproduzierbarer Windows-Release mit Dokumentation und Abnahme
   - [x] Playwright-Browser-E2E für den M1-Kernablauf unter Windows und Linux
   - [x] Echter Windows-Tauri-Debug-Smoke-Test für Start, Autosave, SQLite und Wiederöffnen
   - [x] Isolierter, unsignierter NSIS-Testinstaller mit lokalem Installationslebenszyklus
   - [x] Automatisierte Tastatur-, Fokus- und Accessibility-Prüfung der Kernansichten
   - [x] Manuelle Kern-Hörprüfung mit Windows-Sprachausgabe in der Tauri-WebView
   - [x] Erzwungene Speicherfehleransage und „Bewegung reduzieren“ abnehmen
   - [x] Bewusst unsignierten NSIS-Releasepfad mit Offline-WebView2, Prüfsumme und Buildmanifest
         lokal abnehmen
   - [ ] Installer später auf einem weiteren Windows-Rechner gegenprüfen
   - [x] Privaten GitHub-Releaseentwurf, Rückdownload sowie isolierte Installation und
         Deinstallation abnehmen
   - [x] Geprüftes Artefakt als `v0.1.0-alpha.1`-Prerelease veröffentlichen
   - [x] Alpha 2 mit eingebettetem Beispielprojekt bauen, zurückprüfen und veröffentlichen
   - [x] Alpha 3 mit responsiven Medienkarten bauen, zurückprüfen und veröffentlichen
   - [x] Alpha 4 mit manuell geprüftem Dubletten-Merge bauen, zurückprüfen und veröffentlichen

## M6 - Version 0.2

Bildgenerierung, Variantenvergleich und Kartenmodell folgen erst nach einem stabilen 0.1-Fundament.
Bestehende Assets werden niemals ohne explizite Freigabe ersetzt. Auch Bilder haben ein Recht auf
ein ordentliches Änderungsverfahren.
