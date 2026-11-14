# ADR 0026 Flache DOCX Marker mit erhaltenem Strukturinventar

Status: angenommen als Repräsentationsentscheidung am 14. November 2026; KQ-059 / Issue #115. Der
Nachweis gilt für den isolierten Prototyp und die dokumentierten Office-Builds. Produktiver
Rust-Import, Austauschformatfreigabe und vollständige F08-/M3-Abnahme stehen weiterhin aus.

## Kontext

Der beauftragte Folgeversuch zu ADR 0025 soll das Manuskript auch nach LibreOffice-Bearbeitung
erhalten. Verschachtelte Container-Controls und Custom XML überleben dort den DOCX-Speichervorgang
nicht. Der fachliche Umfang aus Pflichtenheft Revision 1, F08/M3 und Abschnitt 11 sowie dem
Roundtrip-Vertragsentwurf bleibt unverändert: neue Projektkopie, explizite Struktur, begrenzte
Textsemantik, keine Medien oder vollständige Projektwiederherstellung.

## Entscheidung

Der weitere native Austausch wird auf **flache, absatzweite Inline-Content-Controls** und ein
**versioniertes Strukturinventar in benutzerdefinierten Dokumenteigenschaften** ausgerichtet. Jeder
Titel und Absatz erhält genau einen Marker. Kapitel, Szenen, Listen, Listenelemente und Zitate
erhalten ihre Hierarchie aus dem Inventar. Der Korpus hat 41 Struktureinträge und 24 Marker. Auch
leere Kapitel und Szenen haben obligatorische Titelmarker; ihre Entfernung bleibt erkennbar.

Im Unterschied zum ersten Versuch ist das Inventar nun die ausdrückliche Quelle für die
Containerbeziehungen, nicht nur ein Vergleich gegen verschachtelte Controls. Das ist eine bewusst
versionierte Repräsentationsänderung. Text, Fett/Kursiv und harte Umbrüche werden ausschließlich aus
dem sichtbaren Dokument gelesen. Das Inventar enthält keine Textkopien, Pfade oder Datenbank-UUIDs.
Überschriften 1–6 bleiben von Kapitel-/Szenentiteln unterscheidbar.

Das begrenzte kanonische JSON-Inventar wird in ASCII-Stücke von höchstens 200 Zeichen zerlegt. Eine
Kopf-Eigenschaft hält Version, Teileanzahl und SHA-256. Jeder Marker enthält Rolle, lokale Kennung
und die ersten 128 Bit dieses Digest. So werden versehentlicher Verlust, Austausch oder Änderung
einzelner Inventarteile erkannt. Dies ist **keine Signatur** und behauptet keine Authentizität gegen
eine Person, die Inventar und sämtliche Marker gemeinsam gezielt verändert.

Der Verifier prüft vollständige Teilemenge, eindeutige Eigenschaften/JSON-Felder/Kennungen, Version,
Digest, erlaubte Rollen, Eltern, azyklische Zuordnung, Tiefe, Geschwisterposition und kanonische
Reihenfolge. Tatsächliche Absatzmarker müssen exakt in der erwarteten Reihenfolge übereinstimmen.
Unmarkierter Text wird nicht verworfen. Absatzrollen und Nummerierung müssen zur Struktur passen.
Fehlende Metadaten lösen keine heuristische Rekonstruktion aus Titeln aus.

Die experimentellen Namen `KRAQEL.Roundtrip.Prototype2` und `kqf2` sind **keine Produktversion**.
Grammatik, Grenzen und Ablehnungen beschreibt der
[Prototypnachweis](../../../scripts/docx-prototype/FLAT_MARKERS.md). Für Rust werden die endgültige
Produktversion und der vollständige Paketvertrag getrennt festgeschrieben.

## Nachweise

Word 16.0.20527.20112 und LibreOffice 26.8.3.1 erhalten nach tatsächlichem Speichern alle 24 Marker,
15 Inventareigenschaften und die vollständige geprüfte Manuskriptsemantik. Ein zusätzlicher Gang
Word → LibreOffice besteht ebenfalls. Textänderungen direkt in beiden Anwendungen werden mit
unveränderter übriger Semantik übernommen. Sechs synthetische Fixtures mit neutralisierten
Core-Metadaten und Prüfsummen liegen im Repository.

41 Tests umfassen 23 neue Prüfungen für die flache Darstellung und die 18 bisherigen Prüfungen. Die
Ablehnung der beschädigten verschachtelten LibreOffice-Ausgabe bleibt bestehen. Die erzeugte Datei
und alle fünf Office-Ergebnisse wurden in Seitenbilder gerendert und visuell geprüft. Dies
verspricht keine Kompatibilität mit ungeprüften Office-Versionen, ODT-Zwischenspeichern, anderen
DOCX-Filtern oder beliebigen Strukturänderungen.

## Folgen für die Umsetzung

Der Szenenimport nach ADR 0024 und der gewöhnliche Export bleiben unverändert. Die Python-Werkzeuge
sind nur Prüfwerkzeuge und werden nicht aus der App aufgerufen. Die native Stufe muss vollständige
Paket-/Beziehungs-Allowlist, Style-Auflösung, globale Budgets, unbekannte Inhalte, Modellvalidierung
und atomare Veröffentlichung unabhängig in Rust durchsetzen. Keine Netzwerkauflösung oder
Office-Automation in der App.

KQ-060 setzt das gemeinsame kanonische Manuskriptmodell und den nativen Reader/Writer mit eigenem
Produktionsprofil v1 um; [ADR 0027](0027-native-docx-roundtrip.md) ergänzt diese Entscheidung. Neue,
unmittelbar vom Rust-Writer stammende Word-/LibreOffice-Fixtures prüfen den Kern zusätzlich ohne
Profilumschreibung. UI und neue Projektveröffentlichung folgen als eigene Stufe.
