# ADR 0028 DOCX Manuskript als atomare neue Projektkopie

Status: angenommen am 14. November 2026; KQ-061 / Issue #119, aufbauend auf ADR 0027.

## Entscheidung

Die Ansicht Export bietet eine eigene Aktion **DOCX zum Wiederimport erzeugen**. Der gewöhnliche
DOCX-Export bleibt getrennt. Die Startseite erklärt vor **Manuskript-DOCX auswählen** den Umfang:
Manuskriptstruktur und unterstützte Textsemantik, ohne Story-Bibel, Medien oder weitere
Projektdaten. Textbearbeitung innerhalb bestehender Absatzmarker ist unterstützt; freie
Strukturänderungen in Office sind nicht freigegeben. Snapshot v3 bleibt das vollständige
Projektformat.

Zwei zusätzliche Command-v1-Aktionen verwenden bestehende Antworttypen:
`export_roundtrip_manuscript` erhält nur die Projekt-ID und gibt Dateiname, Format und Größe zurück.
`import_docx_project` erhält eine leere Nutzlast, öffnet den nativen Dateidialog und liefert eine
neue Workspace-Antwort oder `null` bei Abbruch. Quellpfad und DOCX-Bytes bleiben ausschließlich in
Rust. Der vollständige native Reader aus ADR 0027 prüft die Datei vor jedem Schreibvorgang.

Der Import vergibt eine neue Projekt-UUID. Ein neuer transaktionaler Storage-Adapter befüllt nur ein
leeres Projekt und vergibt neue Kapitel-/Szenen-UUIDs. Titel, Reihenfolge und Editorinhalt werden
ohne Trimmen oder Platzhalter gespeichert; Klartext und FTS werden lokal abgeleitet. Das geschieht
im unveröffentlichten Staging-Verzeichnis des vorhandenen `ProjectManager`. Danach werden sämtliche
Dokumente aus SQLite erneut geladen, in das kanonische Modell übersetzt und vollständig mit dem
eingelesenen Manuskript verglichen. Erst bei Gleichheit veröffentlicht der Manager das Verzeichnis.
Abbruch schreibt nichts; Fehler vor Veröffentlichung räumen den Zwischenstand auf. Vorhandene
Projekte und die Quelle werden nicht überschrieben.

Der App-Import muss zusätzlich die Speichergrenzen erfüllen: gültiger Projekttitel mit 1–120
Zeichen, Kapitel-/Szenentitel mit höchstens 240 Zeichen und Editor-JSON mit höchstens 5 MiB je
Dokument. Nicht speicherbare gültige Profile werden vollständig abgelehnt. Das Austauschprofil
bleibt v1.

## SQLite Migration 6

Der bisherige Tabellen-Check verbietet leere Dokumenttitel. Migration 5→6 baut `document` in einer
IMMEDIATE-Transaktion neu auf und erlaubt unveränderte leere Titel. Rowids, Kennungen und Inhalte
werden kopiert, Index und FTS-/Asset-Löschtrigger wiederhergestellt und FTS neu aufgebaut. Für
diesen SQLite-Tabellenumbau werden Fremdschlüssel ausschließlich außerhalb der Transaktion kurz
deaktiviert; vor Commit muss `foreign_key_check` leer sein. Danach werden Fremdschlüssel auch im
Fehlerfall wieder aktiviert. Bekannte Backup-Versionen umfassen jetzt 1–6. Bestehende Migrationen
bleiben unverändert.

Normale Erstellen-/Umbenennen-Aktionen verlangen weiterhin einen nichtleeren neuen Titel. Der
Compare-and-swap-Wert beim Umbenennen/Löschen entspricht jetzt dem exakten gespeicherten Titel,
damit importierte leere Titel und Rand-Leerzeichen bedienbar sind. Die Oberfläche zeigt bei leeren
Titeln **Ohne Titel**, ohne diesen Ersatztext zu speichern.

Der bereits offene Snapshot-v3-Dokumentvertrag erlaubt Zeichenketten ohne Mindestlänge. Die zu enge
Rust-Importprüfung wird an gespeicherte Dokumenttitel angepasst; Schema-Datei und Formatversion
ändern sich nicht. Ein Regressionstest belegt Snapshotexport und Neuimport eines DOCX-Manuskripts
mit leeren Titeln. Ältere App-Builds mit der engeren Implementierung können solche Archive ablehnen.

## Nachweise und Grenzen

Rust prüft echte Office-Fixtures, unabhängige Mehrfachimporte, Wiederöffnen, FTS,
Snapshotportabilität, bereinigte Command-Fehler und spätes Scheitern ohne sichtbares Teilprojekt.
Befüllte Migration, Referenzen, Löschverhalten sowie Backup-/Recovery-Abläufe sind
regressionsgeprüft. Browser-E2E prüft die getrennten Aktionen, Umfangshinweise, Abbruch, Fehler,
neue Kopie, Bearbeitbarkeit und Axe. Die echte Windows-Dialogabnahme wird getrennt dokumentiert; ein
Mock prüft weder Dateidialog noch Rust-Paketparser. F08/M3 bleiben wegen weiterer Format- und
Medienfragen nur teilweise abgenommen.
