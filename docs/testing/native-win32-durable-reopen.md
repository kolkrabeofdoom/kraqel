# BP4b · Szene A nach Prozessneustart aus SQLite lesen

**19. November 2026 · Bestanden.** Das erste Paar speicherte die zweite Zeile von Szene A als `43`
und quittierte Revision 1 (Dienst/UI 72/73). Beide Jobs waren leer, bevor der Starter neue Prozesse,
Profile, Pipes, Kanal-ID und Schlüssel für die Lesebindung erzeugte. Das zweite Paar las die Zeilen
`Rabe` und `43` mit Revision 1 aus derselben Datei und bestätigte den Inhalt durch Rücklesen aus dem
Win32-Feld (74/75). Der Starter endete mit Exit 0. Dieser Neustart betrifft UI und Datendienst
innerhalb einer Gastsitzung. Zusätzlich wurde die Datenbank nach dem Herunterfahren des Gasts
unabhängig geprüft.

## Ablage und Rechte

Unter der exklusiven Kontrollsperre erzeugt der Starter neben den Prüfbinaries einen frischen
Unterordner `scene-store-v1` und die leere Datei `synthetic.sqlite`. Bereits belegte Ziele sperren
die Vorbereitung. Der Starter hält Ordner und Datei während beider Durchläufe gegen Umbenennen und
Löschen offen. Der Kanal kennt nur feste Forschungszwecke für Szene A. Der Pfad kommt aus dem
Prüfprogramm.

Die geschützte DACL gibt SYSTEM und Administrierenden Vollzugriff, dem Schreibprofil Änderungsrechte
und dem gesonderten Leseprofil Lese-/Ausführungsrechte. UI und Gegenprobe erhalten keine
Speicherhandles. Unter der frischen Leserolle scheiterte der Versuch, die Datei schreibend zu
öffnen, mit Zugriff verweigert. SQLite öffnete sie anschließend mit `READ_ONLY` und `query_only`.
Nur der erste Dienst initialisiert die Tabelle und den synthetischen Ausgangsstand.

Jede Anfrage lädt den gespeicherten Dokumentbaum und die Revision. Der Datendienst prüft die
kanonische Katalogfassung und verwendet den BP4a-Fachentscheid. Bei erfolgreichem Speichern schreibt
eine SQLite-Transaktion Dokument, abgeleiteten Klartext, Revision und FTS5-Zeile. Erst nach Commit
geht die Antwort über den authentisierten Kanal. Ein Test mit fehlender Indexzeile bestätigt das
Rollback von Text und Revision. SQLite verwendet im Schreiblauf das Rollbackjournal und
`synchronous=FULL`.

## Wiederholung und Gegencheck

Die vier Forschungsimages wie beim [Edit-Gastlauf](native-win32-edit-conflict.md) mit MSVC x64 bauen
und an die erzwungene UMCI-Richtlinie eines frischen Offline-Gastoverlays binden. In der
interaktiven Gastsitzung `pipe-guest.ps1 -Trial durable-reopen` starten. Erwartet werden beide
Exitcodepaare in dieser Reihenfolge, zwei getrennte Jobenden und Starter-Exit 0. Der Speicher bleibt
auf der Gastplatte erhalten.

Nach Gastende wurde das Overlay schreibgeschützt eingebunden. Unabhängiges SQLite-Lesen mit
`mode=ro&immutable=1` bestätigte `integrity_check=ok`, genau die Zeile A, das zweizeilige Dokument,
Klartext und Revision 1. Die FTS-Suche fand `43` und keinen Treffer für `42`. Im Speicherordner lag
ausschließlich die Datenbank. Die ACLs von Verzeichnis und Datei, entfernte Startbelege, Paket-DACL
sowie Image- und Policyhashes wurden kontrolliert. Der Gast ist ausgeschaltet, die Ausgangsplatte
wieder zugeordnet, Netzadapter und aktivierte Integrationsdienste jeweils null. Crashpunkte und
ETW-Netzvergleich des neuen Binärstands stehen aus.

## Geprüfte Binärdateien

| Gastdatei        | SHA-256                                                            |
| ---------------- | ------------------------------------------------------------------ |
| native-leaf.exe  | `4058808f2f5308b1b179983bcfde66074a56cf22ec17ded24837ff91a069aaa4` |
| pipe-check.exe   | `e872c956e46bb8a916522f3f96855f6524b269cbfe3c3bd18d503d126d2bd384` |
| pipe-other.exe   | `0ce413ab5d88475380fa2dc7c540c6fa6e3e58a039fa6f46c11886a0f283f815` |
| pipe-service.exe | `7a7b5dc1ca8b2da079b63c77b76005e2fcd3917588cf1b446bf0f3a16c877853` |
