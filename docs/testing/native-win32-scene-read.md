# BP4b · Eine synthetische Szene im Win32-Fenster lesen

**19. November 2026 · Bestanden.** Dienst und UI lieferten 66/67, der Starter Exit 0. Die
angezeigten Zeilen waren `Rabe` und `42`, Revision 0. Beide Jobs endeten leer. Der schreibgeschützte
Gegencheck bestätigte entfernte Startbelege, Paket-DACL, Image- und Policyhashes, den
ausgeschalteten Gast auf der Ausgangsplatte sowie null Netzadapter und null aktivierte
Integrationsdienste. Ein eigener ETW-Vergleich dieses Binärstands steht aus.

Der Prüfauftrag bindet Szene A und den Modus `read` in beide geschützten Startbelege und in den
Kontextdigest der zugehörigen Bootstrap-Abschnitte ein. Der Starter wiederholt vor Resume die
[Endpunktprüfung einschließlich Fremdprozess](native-win32-pipe-rights.md). Nach
[HELLO/READY](native-win32-hello-ready.md) sendet das Fenster genau einen `c4_document_read_v1` mit
Sequenznummer 1. Die Anfrage enthält keine Ziel-ID.

Der Dienst begrenzt die Länge vor der Speicherreservierung und prüft Richtung, Kanal-ID,
Sequenznummer und HMAC vor der Verarbeitung. Er bindet den speicherresidenten BP4a-Katalog an A im
Lesemodus und liefert nur eine erfolgreiche Leseantwort. Die einmal verwendete Kanalbindung wird auf
beiden Seiten verbraucht. Die UI prüft die authentisierte Antwort auf genau die Felder `text` und
`revision`, die zweizeilige Katalogfassung und Revision 0. Sie setzt das native EDIT-Feld auf
schreibgeschützt, zeigt den empfangenen Text mit Windows-Zeilenumbruch an und liest ihn zum
Vergleich zurück. Fachtexte, Schlüssel und Rohframes erscheinen nicht im Prüfprotokoll.

## Wiederholen

Die vier Forschungsprogramme mit den Einstellungen der [Fensterprobe](native-win32-pipe-rights.md)
bauen. Im frischen Offline-Gastoverlay die Programmhashes an die erzwungene UMCI-Richtlinie und den
geschützten Paketanker binden. Der aktuelle Runner `pipe-guest.ps1 -Trial read-scene` in der
interaktiven Gastsitzung startet `pipe-check.exe read-scene`. Erwartet werden Dienst-Exit 66 nach
der Antwort und UI-Exit 67 nach dem Vergleich im Textfeld. Die gesamte Paarprüfung verlangt Exit 0
und leere Jobs. Nach Gastende Ausgangsplatte, Netzadapter, Integrationsdienste, Image- und
Policyhashes sowie entfernte Startbelege prüfen.

## Geprüfte Binärdateien

| Gastdatei        | SHA-256                                                            |
| ---------------- | ------------------------------------------------------------------ |
| native-leaf.exe  | `c19afc1c442835818a3d6f8ca2b58d8713032d49473caf8b645a4f5e25f18046` |
| pipe-check.exe   | `15be82c6d9999b64ed8d3f762d51db6a179fa02a401c75cb2804af514facdfb9` |
| pipe-other.exe   | `e04e748d3b72530a2b11cb11b3032b1423c7aeca0af38b1f5f5b5193190cf4f7` |
| pipe-service.exe | `dfe1c5afe00b22ba6e3f6a55850c2ec259941ee981af250dd4ad5add89b8bbed` |
