# DOCX-Import v1

KQ-055, Issue #107; ADR 0024. Bezug: Pflichtenheft Revision 1, F08, M3 und Abschnitt 11.

## Vertrag und Semantik

`import_docx_document` verwendet die Command-Hülle v1 und erhält nur `{ projectId, parentId }`. Die
Antwort ist `DocumentNode | null`; `null` bedeutet Dialogabbruch. Pfad und Archivbytes bleiben in
Rust. Unterstützt wird DOCX, nicht DOCM oder das alte binäre DOC-Format.

Eine Datei erzeugt eine neue Szene im gewählten Kapitel. Der auf 240 Zeichen begrenzte Dateistamm
wird zum editierbaren Titel. Die bestehende Storage-Transaktion schreibt Szene, Editor-Dokument v1,
Position und Klartextindex erst nach vollständiger Validierung gemeinsam. Fehler und Abbruch
schreiben nichts; bestehende Szenen werden nicht überschrieben.

Übernommen werden Unicode-Text, Absätze einschließlich leerer Absätze, harte Textumbrüche,
Fett/Kursiv auch kombiniert sowie eindeutig auflösbare Überschriften 1 bis 6. Absatz- und
Zeichenstyles werden über Standardstyles, `basedOn` und direkte Formatierung aufgelöst. Bekannte
Heading-IDs und explizite Outline-Ebenen werden unterstützt. Unbekannte, zyklische oder nicht
unterstützte angewendete Styles werden abgelehnt. Unterschiedliche Formatierung komplexer und
normaler Schriften wird nur bei einheitlicher Abbildbarkeit übernommen.

Vor der Auswahl erläutert die Oberfläche, dass Schriftarten und Seitenlayout nicht übernommen
werden. Tabellen, Bilder, Fußnoten, Links, Felder, Änderungsmarkierungen, Listen und unbekannte
Inhaltsknoten werden abgelehnt statt still entfernt. Kapitel-/Szenengrenzen werden nicht geraten.
Der bestehende Exportwriter bleibt unverändert; dies ist kein strukturerhaltender Roundtrip.

## Ressourcen- und Paketgrenzen

| Grenze                                             | Maximum   |
| -------------------------------------------------- | --------- |
| Lokale reguläre, nicht symbolisch verknüpfte Datei | 8 MiB     |
| ZIP-Einträge einschließlich Verzeichnissen         | 128       |
| Entpackte Daten insgesamt                          | 32 MiB    |
| Einzelner entpackter Teil                          | 8 MiB     |
| Kompressionsverhältnis je Teil                     | 100:1     |
| XML-Verschachtelung                                | 64 Ebenen |
| XML-Elemente insgesamt über alle Teile             | 100.000   |
| Style-Vererbung                                    | 32 Ebenen |
| Übernommener UTF-8-Text                            | 2 MiB     |

Es wird nichts extrahiert. Doppelte ZIP-Namen, unsichere Paketpfade, Verschlüsselung, Makroteile,
DTD, Processing Instructions, unbekannte Entities und externe Beziehungen sind unzulässig. XML muss
UTF-8 und wohlgeformt sein; Namen werden namespacebewusst verglichen. Interne Relationship-Ziele
werden ausschließlich innerhalb des Pakets aufgelöst, ohne Netzwerk- oder Betriebssystemzugriff.

Bekannte Hilfsteile (Styles, Einstellungen, Theme, Fonts, unbenutzte Nummerierungsdefinitionen,
Bibliografie-Custom-XML, Dokumentmetadaten und Paketvorschauen) dürfen vorhanden sein.
XML-Hilfsteile unterliegen denselben Parsinggrenzen. Vorschaubilder werden nur begrenzt gelesen und
weder gerendert noch übernommen. Verwendete Listen und Content Controls bleiben abgelehnt.
Unbekannte Paketbestandteile sind unzulässig.

Die Dateilänge wird vor und nach dem Öffnen sowie gegen die begrenzt gelesenen Bytes geprüft. Fehler
enthalten keine Pfade, Roh-XML oder Bibliotheksdiagnostik. Die bestehende Einschränkung gegenüber
böswilligen parallelen Dateisystemänderungen bleibt bestehen.

## Nachweise und Aussagegrenzen

Rust-Tests prüfen Semantik, Styles, Namespaces, fehlerhafte Pakete, Ressourcenlimits und
unveränderte Projekte bei Ablehnung. Die unabhängige python-docx-Fixture ist keine manuelle
Word-/LibreOffice- Abnahme. Browser-E2E nutzen den testexklusiven Command-Adapter; native
Windows-Dialoge und echte Dateien benötigen einen getrennten Nachweis.
