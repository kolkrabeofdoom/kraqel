# ADR 0025 DOCX Strukturprototyp und Office Grenzen

Status: vorgeschlagen, Experiment ausgewertet am 14. November 2026; KQ-058 / Issue #113. Keine
Freigabe einer produktiven Austauschversion oder eines Projektimports.

## Kontext

Pflichtenheft Revision 1, F08/M3 und Abschnitt 11 verlangen die Festlegung des DOCX-Roundtrips. ADR
0024 implementiert den engen Import als einzelne Szene. Der von KQ-057 konkretisierte
Vertragsentwurf schlägt einen getrennten Manuskript-Austausch mit expliziten Strukturmarkern vor.
Die Zustimmung zum Strukturprototyp erlaubt dessen Prüfung, begründet aber keine vorweggenommene
Office-Kompatibilitätszusage.

## Untersuchte Repräsentation

Das isolierte Python-Werkzeug unter `scripts/docx-prototype/` erzeugt verschachtelte Rich-Text-
Content-Controls für Manuskript, Titel, Kapitel, Szenen, Absätze, Überschriften, Listen, Elemente
und Zitate. Rollen und lokale Kennungen stehen in `w:tag`; ein separates XML-Inventar prüft Rollen,
Eltern und Reihenfolge. Der Prototyp hält das Inventar im über eine OPC-Beziehung referenzierten
Custom-XML-Teil, nicht in einem unbegrenzt wachsenden einzelnen Tag-Attribut. Manuskripttext wird
nur aus den sichtbaren Absätzen gelesen. Nummerierte Listen sind echte OOXML-Nummerierungen.

Der Korpus enthält auch leere Kapitel/Szenen/Absätze, doppelte Titel, alle Überschriftenebenen,
Unicode, Leerzeichen, kombinierte Fett-/Kursivmarkierungen, harte Umbrüche, mehrabsätzige
Listenelemente und ineinander verschachtelte Listen und Zitate. 18 Tests sichern Vergleich und
Ablehnungen ab. Die App, der bestehende Writer und der Fremdimport bleiben unverändert.

## Beobachtungen

- Word 16.0.20527.20112 erhält nach tatsächlichem Speichern alle 41 Marker und das Inventar. Der
  kanonische Vergleich ist identisch. Nach einer echten Textänderung wird ausschließlich der
  geänderte Text übernommen; keine Rücksetzung auf eine versteckte Textkopie.
- LibreOffice 26.8.3.1 erhält nach einem tatsächlichen DOCX-Speichern nur die 24 Blattmarker. Die 17
  Container-Marker und das Custom-XML-Inventar gehen verloren. Der Rückimport wird entsprechend dem
  Vertrag abgelehnt, obwohl das Dokument weiterhin lesbar aussieht.
- Echte Office-Fixtures mit neutralisierten Core-Metadaten, genaue Abläufe und Prüfsummen stehen im
  [Versuchsnachweis](../../../scripts/docx-prototype/README.md).
- Die erzeugte Datei und beide Word-Ergebnisse wurden über LibreOffice in Seitenbilder gerendert und
  visuell kontrolliert. Dieser Rendernachweis ist kein LibreOffice-Speicher-Roundtrip.

## Vorgeschlagene Folgerung

Die verschachtelte Repräsentation erfüllt den untersuchten Word-Fall, aber nicht die angestrebte
gemeinsame Word-/LibreOffice-Grenze. Deshalb noch keine Annahme des Austauschformats und keine
Übernahme des Verifiers in den privilegierten Rust-Import. Die experimentelle `kq1`-Kennung ist
keine produktive Version.

Als nächster begrenzter Versuch sollen flache Blattmarker mit explizitem Strukturinventar in einem
von beiden Anwendungen erhaltenen Paketbestandteil geprüft werden. Ein solcher Wechsel muss
Zuordnung, Reihenfolge, leere Container und Verlustfälle ebenso streng nachweisen. Eine
Word-spezifische Garantie wäre eine gesonderte Produktentscheidung und wird hier nicht impliziert.

Der [Vertragsentwurf](../../specifications/DOCX_ROUNDTRIP_CONTRACT_DRAFT.md) bleibt fachlich der
Maßstab. Allgemeine Office-Strukturänderungen, Medien, Datenbank-UUIDs und vollständige
Projektwiederherstellung sind nicht Teil dieses Experiments. Ein zukünftiger Produktparser muss alle
Paket-, Ressourcen- und Inhaltsgrenzen unabhängig in Rust durchsetzen.
