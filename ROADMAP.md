# Roadmap

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
       - [ ] KQ-069 A: Gezielte Update-Abbruch-/Fehlerreconciliation unter derselben Sperre;
             isolierten Produktstarter/echte Projektmigration integrieren sowie weitere allgemeine
             Fensterende-/F08-Prüfung; dauerhaften authentisierten Produktbroker
             integrieren/abnehmen; weitere Formate/Git-/Providergrenzen; UDP/IPv6/DNS/Broker
             kalibrieren; Packaging und F08-/M3-Vollprüfung
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
