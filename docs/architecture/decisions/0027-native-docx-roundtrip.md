# ADR 0027 Nativer DOCX Manuskriptaustausch v1

Status: angenommen am 14. November 2026; KQ-060 / Issue #117. Der native Kern ist implementiert; UI,
Commands und neue Projektveröffentlichung bleiben eine eigene Folgestufe.

## Kontext

ADR 0026 weist flache Absatzmarker und ein Strukturinventar in Dokumenteigenschaften für Word und
LibreOffice nach. Das Pflichtenheft Revision 1, F08/M3 und Abschnitt 11 verlangt eine ausdrücklich
begrenzte Roundtrip-Entscheidung. Ein Python-Prototyp reicht als App-Vertrauensgrenze nicht aus.

## Entscheidung

`kraqel-export` besitzt ein gemeinsames kanonisches Manuskriptmodell mit expliziter Kapitel-,
Szenen-, Block- und Inline-Semantik. Editor-JSON wird vor der Konvertierung auf unterstützte
Attribute geprüft; unbekannte Formatierung und nicht exportierbarer Kapitelinhalt dürfen nicht still
verschwinden. Der inverse Szenenadapter erzeugt wieder Editor-Dokument v1. Gewöhnliche Exporte
verwenden dieselbe Validierung, ohne die neuen Roundtrip-Größenlimits zu übernehmen.

Das Produktionsprofil heißt `KRAQEL.Manuscript.v1` mit Markerpräfix `kqm1`, nicht wie der Prototyp.
Der vollständige [Datenvertrag](../../data-format/DOCX_ROUNDTRIP.md) definiert Struktur, Paket und
Grenzen. Inventar und sichtbare Marker werden unabhängig validiert; der sichtbare Text ist die
einzige Textquelle. Die Hashbindung ist keine Authentizitätssignatur.

Reader und Writer laufen vollständig offline in Rust. Der vorhandene begrenzte DOCX-Szenenparser
wird aus dem Tauri-Modul ins Export-Crate verlegt und für ZIP/XML und Textsemantik wiederverwendet.
Der neue Roundtrip ergänzt strikte Paket-/Beziehungs-/MIME-Prüfung, Inventargrammatik, verwendete
Style-Auflösung, Nummerierung und globale Budgets. Bestehender Szenenimport bleibt über eine dünne
Facade erreichbar. Unbekannte sichtbare Inhalte werden abgelehnt.

Jede Writer-Ausgabe wird nativ zurückgelesen und vollständig mit dem kanonischen Eingangsmodell
verglichen. Die Bibliotheksfunktion für den Projektexport veröffentlicht erst anschließend atomar
unter einem festen eigenen Namen. Es gibt in diesem Schritt keine neuen IPC- oder UI-Aktionen und
keinen Datenbankschreibpfad für den Roundtrip.

`quick-xml` 0.42 und `sha2` 0.10 werden aus bereits gelockten Workspace-Abhängigkeiten
wiederverwendet (MIT beziehungsweise MIT/Apache-2.0; gepflegte Upstream-Projekte). `zip` bleibt auf
dem bestehenden Stand 9.5.1. Es werden keine zusätzlichen Netzwerkzugriffe, Laufzeitprogramme oder
Office-Bibliotheken eingeführt. Python/Office sind reine Testwerkzeuge und werden nicht mit der App
ausgeführt.

## Nachweise und Folgen

Die Rust-Tests prüfen vollständige Modellgleichheit, deterministische Bytes,
Editor-Rückkonvertierung, Größen-/Tiefengrenzen, Verlustfälle und beschädigte Pakete. Vier neue
v1-Fixtures stammen unmittelbar vom nativen Writer beziehungsweise echten
Word-/LibreOffice-Speichervorgängen, einschließlich einer Word-Textänderung. Die früheren sechs
Prototyp-Fixtures bleiben mit einer transparenten testinternen Profilumschreibung zusätzliche
Regressionseingaben; der Produktreader lehnt deren Originalprofil ab.

Die neue Grammatik ist eine begrenzte Austauschversion, keine allgemeine DOCX-Unterstützung und
keine vollständige F08-Abnahme. Strukturänderungen in Office, Medieneinbettung, zusätzliche
Textformate, ungeprüfte Office-Builds und vollständige Projektwiederherstellung werden dadurch nicht
freigegeben. Als Nächstes folgen UI/Commands und atomare Veröffentlichung einer neuen Projektkopie
mit neuer Identität. Bestehende Projekte dürfen beim Import nicht überschrieben werden.
