# BP4b · Schreibversuch mit Leserecht

**19. November 2026 · Bestanden.** Der Dienst lieferte 68, die UI 69 und der Starter 0. Beide Jobs
endeten leer. Der schreibgeschützte Abschlussabgleich bestätigte die entfernten Startbelege, Image-
und Policyhashes sowie die Paket-DACL. Der Gast ist aus und die Ausgangsplatte wieder zugeordnet,
bei null Netzadaptern und null aktivierten Integrationsdiensten. Der ETW-Nachweis für diesen
Binärstand steht aus.

Der Prüfzweck `A/read/denial-check` bindet ein UI-/Dienstpaar für die gesamte Probe an Szene A. Nach
HELLO/READY verarbeitet derselbe Datendienst drei authentisierte Anfragen in fester Reihenfolge. Er
hält über alle drei dieselbe `BoundScene` mit `Mode::Read` und denselben speicherresidenten Katalog.
Der zweite Auftrag läuft durch den regulären BP4a-Berechtigungsentscheid. Die Antwort wird aus
dessen Ergebnis serialisiert.

| Sequenz | Aktion                                                                        | Erfolgskriterium                                                                                    |
| ------- | ----------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------- |
| 1       | A lesen                                                                       | Revision 0 und die Zeilen `Rabe` und `42`                                                           |
| 2       | Mit erwarteter Revision 0 den gültigen Katalogtext auf `Rabe` und `43` ändern | Authentisierte Fachantwort `{"error":"DENIED"}`                                                     |
| 3       | A erneut lesen                                                                | Revision 0, derselbe Text wie bei Sequenz 1; Anzeige und Rücklesen im schreibgeschützten Win32-Feld |

Der UI-Prüfcode erzeugt den Schreibauftrag unabhängig vom schreibgeschützten Eingabefeld. Ein stumm
verworfener Frame, ein Timeout oder ein Revisionskonflikt würde die Probe scheitern lassen.
Schlüssel, Texte und Rohframes werden im Gastprotokoll nicht ausgegeben.

**Wiederholung.** Die vier Prüfprogramme wie im [Leseversuch](native-win32-scene-read.md) bauen und
an ein frisches Offline-Gastoverlay mit erzwungener UMCI binden. In der interaktiven Sitzung
`pipe-guest.ps1 -Trial deny-save` aufrufen. Der Dienst liefert Exit 68 erst nach der letzten
Antwort, die UI Exit 69 nach Kontrolle von Fehlercode, Text, Revision und Feldinhalt. Nach Gastende
beide Jobenden, Ausgangsplatte, Isolationsgrenzen, Startbelege und Hashes gegenprüfen.

## Geprüfte Binärdateien

| Gastdatei        | SHA-256                                                            |
| ---------------- | ------------------------------------------------------------------ |
| native-leaf.exe  | `4541236e778c9fc1c1a59eacffccd783cabb9f7ab58c88eb51cbd883ffc298f9` |
| pipe-check.exe   | `795d8d5cab7cd68b8d9ce95c5759aeee0d643e64d91394930cf92e154cc55a7b` |
| pipe-other.exe   | `c2c54d4a76237d14a0972e9ef8e389e5c8293d2730c93b58c0d484733114d004` |
| pipe-service.exe | `d8b542c958b49c61dfa0d40ff6b3c71e676bccc2169e64b6864f15f990b67c17` |
