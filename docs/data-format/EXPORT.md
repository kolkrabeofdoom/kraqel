# Manuskriptexport v1

KQ-009 exportiert den aktuellen SQLite-Stand eines Projekts lokal nach
`exports/manuskript.<format>`. Unterstützte Formate sind Markdown (`md`), eigenständiges HTML, DOCX
und PDF. Vorhandene Dateien desselben Formats werden erst nach vollständig erfolgreichem Rendern
ersetzt.

## Unterstützte Semantik

- Projekt-, Kapitel- und Szenentitel in stabiler Baumreihenfolge
- Absätze und Überschriften der Ebenen 1 bis 6
- fett und kursiv gesetzter Text, auch kombiniert
- harte Zeilenumbrüche
- geordnete und ungeordnete, auch verschachtelte Listen
- Blockzitate

Andere Tiptap-Knoten oder -Markierungen werden nicht still entfernt: Der Export wird abgelehnt.
Insbesondere Medien, Links, Tabellen, Fußnoten und Codeblöcke gehören noch nicht zu v1.

## Sicherheits- und Datenschutzgrenze

Der Export greift nicht auf das Netzwerk zu. Ziele sind feste Dateinamen unter dem kanonischen
Projektordner; Symlink-Ziele und nicht reguläre Dateien werden abgelehnt. IPC-Antworten nennen nur
Format, relativen Dateinamen und Bytezahl, niemals den absoluten lokalen Pfad.

Import und Roundtrip sind keine Eigenschaft dieses Formats und werden getrennt spezifiziert.
