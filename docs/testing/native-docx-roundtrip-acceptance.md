# Native Windows Abnahme des DOCX Manuskriptaustauschs

Stand: 14. November 2026, KQ-061 / Issue #119. Pflichtenheft Revision 1, F08/M3 und Abschnitt 11;
ADRs 0026–0028. Der geprüfte Umfang ist ein begrenztes Manuskript, keine vollständige Projektkopie.

## Umgebung und Isolation

Windows-Entwicklungsrechner, echter Tauri-Debug-Build mit React-WebView, nativen Dialogen, Rust und
SQLite. Eigener Produktname `KRAQEL Roundtrip Test`, eigene Bundle-ID und App-Datenwurzel
`$TEMP/kraqel-roundtrip-test-20261114`. Reguläre App-Daten wurden nicht geöffnet oder verändert.
Keine Übertragung und keine Cloud-/Office-Automation durch die App.

Die Quelle ist die synthetische `word-edited.docx` aus `crates/export/tests/fixtures/roundtrip-v1/`:
41 Inventarknoten, 24 Marker, 3 Kapitel und 3 Szenen, doppelte sowie zwei leere Dokumenttitel,
Überschriften 1–6, Unicode/Leerzeichen, Fett/Kursiv, harte Umbrüche, verschachtelte Listen und
Zitate. Die vorangegangenen echten Word-/LibreOffice- Speichervorgänge sind bei diesen Fixtures und
im Datenvertrag dokumentiert.

## Beobachtete Ergebnisse

| Prüfung                  | Ergebnis                                                                                                                                                      |
| ------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Umfang vor Auswahl       | Startkarte nennt neue Projektkopie, erhaltene Inhalte, ausgeschlossene Projektdaten und begrenzte Office-Bearbeitung.                                         |
| Nativer Dialog           | Eigener Fenstertitel und DOCX-v1-Dateifilter vorhanden.                                                                                                       |
| Abbruch                  | Neutral, Aktionen wieder bedienbar, keine Fehlermeldung, Projektwurzel enthält weiterhin null Projekte.                                                       |
| Ungültige Datei          | Synthetische Datei ohne ZIP/DOCX wird bereinigt abgewiesen; keine Pfade oder Quelldaten in der Meldung, weiterhin null Projekte.                              |
| Word-Datei importieren   | Neues Projekt mit 3 Kapiteln/3 Szenen. Doppelte Titel bleiben getrennt; leere Titel erscheinen als „Ohne Titel“.                                              |
| Text und Struktur        | Echte Word-Änderung `Original` → `x`, Überschriften, Fett/Kursiv, Umbruch, Listen und Zitate im Editor sichtbar.                                              |
| Eigener Export           | Getrennte Aktion neben gewöhnlichem DOCX; Erfolg mit `manuskript-roundtrip.docx` und 22649 Bytes.                                                             |
| Native Dateiprüfung      | Öffentlicher Rust-Dateireader liest die tatsächlich durch die App erzeugte Datei; vollständige kanonische Gleichheit mit der Word-Fixture einschließlich `x`. |
| Wiederimport             | Genau diese App-Ausgabe wird über den nativen Dialog als zweites Projekt geöffnet.                                                                            |
| Neue Identitäten         | Zwei verschiedene Projekt-UUIDs; je 6 Dokumente, insgesamt 12 verschiedene Dokument-UUIDs. Quelle und erste Kopie bleiben bestehen.                           |
| Leerer Titel bearbeitbar | Leeres Kapitel in der zweiten Kopie zu „Leeres Kapitel“ umbenannt; erster Import bleibt unverändert.                                                          |
| Echter Neustart          | App geschlossen, abschließenden Debug-Build neu gestartet; beide Projekte gelistet, neue Kopie geöffnet, umbenanntes Kapitel und Text erhalten.               |

Die Quelldatei bleibt mit SHA-256 `3d75b2d11b7e45d2f43ba8b58b38dc7c32c45b7eaa6aee3b8bc62724817f3cf2`
unverändert. Nur synthetische Testprojekte wurden angelegt. Die zusätzliche Metadatenprüfung der
Testdatenbank war lesend; reguläre lokale Projekte wurden nicht verwendet.

## Automatisierte Ergänzung und Aussagegrenzen

`pnpm check` besteht mit 144 Rust- und 49 TypeScript-Tests. Alle 25 Browser-E2E bestehen
einschließlich der getrennten Aktionen, Abbruch/Fehler, unabhängigen bearbeitbaren Kopie,
Wiederöffnen, Tastatur und Axe-Prüfung. Rust belegt semantischen Vergleich nach SQLite-Befüllung,
spätes Scheitern ohne veröffentlichtes Projekt, FTS, unabhängige Mehrfachimporte und
Snapshot-v3-Export/Neuimport mit leeren Titeln. Migration 5→6 wird mit vorhandenen Referenzen und
Löschtriggern geprüft; Backup-/Recovery- Regressionen bestehen ebenfalls.

Browser-Mocks ersetzen keinen Paketparser; die native Prüfung ersetzt keine Clean-VM, assistive
Vollabnahme oder Kompatibilitätszusage für ungeprüfte Office-Versionen. Freie Strukturänderungen in
Office, Medien und vollständige Projektwiederherstellung sind ausgeschlossen. Die App verlangt
zusätzlich die bestehenden Projekt-/Dokumenttitel- und Editor-JSON-Speichergrenzen; nicht
speicherbare Profile werden vollständig abgelehnt. F08/M3 bleiben insgesamt teilweise offen.
