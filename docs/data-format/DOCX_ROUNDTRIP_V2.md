# DOCX-Manuskriptaustausch v2 für Prosa und Drehbuch

**Stand vom 27. November 2026 auf dem Entwicklungszweig.** Der folgende Vertrag erweitert den
ausdrücklich gewählten Manuskriptaustausch. Die Regeln für Paket, Inventar, Prosa,
Inline-Formatierung, Ressourcen und Veröffentlichung gelten aus [Profil v1](DOCX_ROUNDTRIP.md),
soweit hier keine Erweiterung beschrieben ist.
[ADR 0037](../architecture/decisions/0037-docx-screenplay-roundtrip.md)

## Profilwahl und Identität

Der Writer wählt anhand der gespeicherten Szenen. Enthält das Manuskript mindestens eine
Editor-v2-Drehbuchszene, schreibt er das gesamte Manuskript als v2, einschließlich aller
Prosaszenen. Auch eine Drehbuchszene mit ausschließlich leeren Rollenabsätzen zählt. Enthält es
ausschließlich Prosa oder keine Szenen, schreibt er v1. Die Datenbankschemaversion hat auf diese
Wahl keinen Einfluss. Bestehende Dateien und Projekte benötigen keine Konvertierung.

Der Reader verlangt genau eine vollständige, unterstützte Profilfamilie. Unbekannte, doppelte oder
gemischte KRAQEL-Profildeklarationen, fehlende Pflichtteile und Marker einer anderen Version führen
zur Ablehnung. Ein v2-Paket muss mindestens eine Drehbuchszene enthalten. Der gewöhnliche
DOCX-Export und der Fremd-DOCX-Szenenimport behalten ihre getrennten Einstiegspunkte.

## Inventar und sichtbare Absätze

Der Kopf heißt `KRAQEL.Manuscript.v2`, die fortlaufenden Teile heißen `KRAQEL.Manuscript.v2.0001`
usw. Kopf und Inventar verwenden die Zahl 2. Feldreihenfolge, lokale Kennungen, Chunkbildung,
SHA-256, Baumreihenfolge und Vollständigkeitsprüfung entsprechen v1. Das Inventar enthält nur
Struktur. Sämtlicher Text stammt aus den sichtbaren, markierten Absätzen.

Alle v1-Knoten behalten im v2-Inventar ihre Bedeutung. Ein Kapitel darf in derselben geordneten
Kindfolge `scene` (Prosa) und `screenplay` (Drehbuch) enthalten. Beide haben genau einen
`scene-title` als erstes Kind an Position 0, auch bei leerem Titel. Unter `screenplay` folgen an den
Positionen 1 bis n ausschließlich Drehbuchblätter, mindestens eines. Auch ein leerer Rollenabsatz
besitzt ein eigenes Blatt und einen eigenen sichtbaren Absatzmarker.

| Inventarrolle      | Editorrolle     | DOCX-Absatzstil beim Schreiben |
| ------------------ | --------------- | ------------------------------ |
| `sc-heading`       | `sceneHeading`  | `KQScreenplaySceneHeading`     |
| `sc-action`        | `action`        | `KQScreenplayAction`           |
| `sc-character`     | `character`     | `KQScreenplayCharacter`        |
| `sc-parenthetical` | `parenthetical` | `KQScreenplayParenthetical`    |
| `sc-dialogue`      | `dialogue`      | `KQScreenplayDialogue`         |
| `sc-transition`    | `transition`    | `KQScreenplayTransition`       |

Das Tag eines sichtbaren Blatts lautet `kqm2:<rolle>:<id>:<digest128>`. Die Kennung umfasst `n` und
4 bis 8 Dezimalziffern. Der Digest umfasst die ersten 32 kleinen Hexzeichen des Inventar-SHA-256.
Das längste erlaubte Tag mit `sc-parenthetical` und achtstelliger Nummer benötigt genau 64
ASCII-Zeichen (4 Präfixzeichen, 3 Doppelpunkte, 16 Rollen-, 9 Kennungs- und 32 Digestzeichen). Jeder
sichtbare Marker muss genau seinem Inventarblatt entsprechen.

Eine Drehbuchrolle verlangt den zugehörigen Absatzstil aus der Tabelle. Der Reader prüft den
effektiven Stil einschließlich Vererbung auf eindeutige Übereinstimmung. Die sechs Basisstile dürfen
sich nicht gegenseitig erben. Ein fehlender oder widersprüchlicher Stil, ein Drehbuchstil an einem
Prosablatt und jede unerwartete Nummerierung an einem Drehbuchblatt führen zur Ablehnung. Ein Stil
allein begründet keine Rolle. Struktur- oder Textfehler lösen keinen Ersatzimport aus.

Der v2-Writer setzt den Namen jeder Rollenbasis gleich ihrer Stil-ID. Word verwendet beim Speichern
den Namen, um die Kennung neu zu bilden. Ein lokalisierter Anzeigename würde damit die Bindung
auflösen. Für gewöhnliche DOCX-Ausgaben bleiben die deutschen Stilnamen verfügbar.

Ein Drehbuchblatt hält Text, Fett, Kursiv und harte Umbrüche gemäß der Editor-v2-Speichergrammatik
fest. Auch ein als Entwurf speicherbarer unvollständiger Dialog darf ausgetauscht werden. Für den
Selbstimport und das Wiederöffnen der Projektkopie gilt dieselbe Speichergrammatik.
[Editor-v2-Vertrag](screenplay-v2.md)

## Grenzen und Veröffentlichung

Die v1-Budgets gelten gemeinsam für das gesamte gemischte Dokument: 8 MiB Datei, 128 ZIP-Einträge, 8
MiB je Part, 32 MiB entpackt, Kompressionsverhältnis höchstens 100:1 je Part, XML-Tiefe 64 und
100.000 XML-Knoten, Manuskripttiefe 32 und 4.096 Inventarknoten, 2 MiB UTF-8-Text einschließlich
Titeln und 100.000 Inlines. Für das Inventar gelten höchstens 2.048 ASCII-Chunks à 200 Zeichen. Für
jede rekonstruierte Szene gelten zusätzlich ihre eigenen Editorgrenzen. Medien, externe Beziehungen,
Projektkennungen und lokale Pfade sind im Austauschprofil ausgeschlossen.

Vor Dateiersatz prüft der Writer seinen Export durch nativen Selbstimport. Der Vergleich umfasst
Szenenart, Rollen, Reihenfolge, Titel und kanonische Inline-Inhalte. Der Import veröffentlicht eine
neue Projektkopie mit eigener Kennung erst nach erfolgreichem SQLite-Rücklesen derselben Semantik.
Prosaszenen erhalten Editor v1, Drehbuchszenen Editor v2. Eine v2-Kopie mit Drehbuchdaten benötigt
Schema 7. Ablehnung oder Abbruch bewahren die Quelldatei und veröffentlichen keine Teilkopie.

Word 16.0.17932 und LibreOffice 26.8.3 konnten Änderungen innerhalb markierter Textabsätze
speichern. Beim Rücklesen blieben auch leere Rollenabsätze sowie Fett/Kursiv und harte Umbrüche
erhalten. Die LibreOffice-Probe umfasste alle sechs Drehbuchrollen. Für v1 gelten die vorhandenen
Kompatibilitätsnachweise.
[Abnahme und Prüfumgebungen](../testing/F08_M3_ACCEPTANCE_MATRIX.md#stand-der-drehbuchkriterien)
