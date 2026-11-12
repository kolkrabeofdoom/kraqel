# ADR 0008: Gemeinsames, validiertes Exportmodell

Status: angenommen am 9. November 2026

## Kontext

F08 verlangt Markdown-, HTML-, DOCX- und PDF-Ausgabe mit definierten Formatierungen. Vier direkte
Tiptap-Renderer würden Validierung und Semantik vervierfachen; unbekannte Knoten könnten je nach
Format unbemerkt verschwinden.

## Entscheidung

KRAQEL übersetzt versioniertes Tiptap-JSON zuerst in eine interne, formatunabhängige
Manuskriptrepräsentation. Unterstützt sind Projekt-, Kapitel- und Szenentitel, Absätze,
Überschriften, Fett/Kursiv, harte Umbrüche, geordnete und ungeordnete Listen sowie Blockzitate.
Unbekannte Knoten oder Markierungen beenden den gesamten Export mit einem bereinigten Fehler.

Alle Renderer erhalten ausschließlich dieses Modell. Dateien werden unter dem festen lokalen
Projektunterordner `exports/` über eine temporäre Geschwisterdatei veröffentlicht; Symlinks und
nicht reguläre Ziele sind unzulässig. Die IPC-Antwort enthält keinen absoluten Pfad.

KQ-046 ergänzt eine explizite Oberfläche zum Öffnen dieses Ordners. Die WebView übergibt nur die
Projekt-ID. Rust öffnet ausschließlich den kanonischen, verwalteten Kindpfad `exports/`, nachdem
Symlinks und Nicht-Verzeichnisse abgelehnt wurden; der absolute Pfad wird weiterhin nicht über IPC
offengelegt.

## Folgen

- Die unterstützte Semantik ist über alle vier Formate konsistent prüfbar.
- Neue Editorfunktionen benötigen eine bewusste Erweiterung des Exportvertrags.
- Medien, Links, Tabellen, Fußnoten, Import und DOCX-Roundtrip bleiben getrennte Aufgaben.
- PDF benötigt vor breiter Unicode-Abnahme eine eingebettete freie Schrift.
- Ein Browser-E2E prüft Formatwahl, Dateiname und Größenbestätigung gegen einen testexklusiven
  Command-Adapter. Tatsächliche Dateien, Renderer, Tauri, Dateisystem und Screenreader werden damit
  ausdrücklich nicht abgenommen.
- KQ-046 prüft den Requestvertrag, die kanonische Pfadauflösung, den Browserbutton und die echte
  Windows-Tauri-Bedienung; andere Betriebssystem-Dateimanager bleiben separat abzunehmen.
