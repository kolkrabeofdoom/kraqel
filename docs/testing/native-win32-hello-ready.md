# BP4b · HELLO/READY im Offline-Gast

## Ablauf

**19. November 2026 · Bestanden im interaktiven Offline-Gast unter erzwungener UMCI.** Bei
abweichendem Antwortschlüssel bestätigte der Dienst HELLO (Exit 60) und die UI verwarf READY (Exit
63). Im frischen regulären Paar bestätigten Dienst und UI beide Nachrichten (60/62). Beide Jobs
waren nach jedem Lauf leer. Der Gast ist ausgeschaltet und die Ausgangsplatte wieder ausgewählt. Der
schreibgeschützte Gegencheck bestätigte null Netzadapter und aktive Integrationsdienste, die
Entfernung beider Startbelege und der Fremdprüfkennung sowie die Paket-DACL, die vier Imagehashes
und den Richtlinienhash.

Der Starter hält die lokale Prüfsperre über beide Durchläufe. Pro Durchlauf erzeugt er ein frisches
Rollenpaar, zwei anonyme Einwegpipes, eine zufällige Kanal-ID und zwei zufällige
256-Bit-Richtungsschlüssel. Er wiederholt die
[Prüfung der Endpunktrechte und des Fremdprozesses](native-win32-pipe-rights.md). Zusätzlich erhält
jede suspendierte Rolle genau ein nicht vererbbares Handle mit `SECTION_MAP_READ` auf ihren eigenen
unbenannten Bootstrap-Speicherbereich. Die schreibbaren Ansichten und die lokalen Section- und
Pipehandles des Starters sind vor Resume geschlossen.

Die Startbelege unter der geschützten Paket-DACL enthalten Rollenkennung, beide Prozessgenerationen
samt Sitzung, den festen synthetischen Szenenbezug A/edit und den prozesslokalen
Bootstrap-Handlewert. Der gelesene Abschnitt bindet diese Angaben und den Paket-/Policyanker per
SHA-256. Er enthält die rolleneigenen Pipehandles, die Kanal-ID und die Richtungsschlüssel. Jede
Rolle prüft das eigene Image am erwarteten Gastpfad, den AppContainer, die Mitigations, die eigene
Prozessgeneration und die Kontextbindung. Der Starter hat beide Images zuvor bei gehaltenen Handles
geprüft.

Nach Aufbau des Win32-Fensters sendet die UI genau ein HELLO. Der Datendienst sendet READY erst nach
erfolgreicher Prüfung von Frameform, Kanal-ID, Nummer null und HMAC. Die UI prüft READY mit dem
Antwortschlüssel. Beide Frames haben 82 Byte und einen leeren Nachrichtenkörper. Der Starter wartet
höchstens 30 Sekunden auf das Prozesspaar. Die Prüfausgabe enthält nur feste Ergebnisse und
Exitcodes. Es gibt keinen Szenenaufruf.

Für die Wiederholung den [Gastaufbau](native-win32-pipe-rights.md) mit den vier Images dieses Stands
verwenden und in der interaktiven Sitzung den aktuellen Runner `pipe-guest.ps1 -Trial handshake`
ausführen. Der Runner startet `pipe-check.exe handshake`. Der erste Durchlauf ändert ein Bit im
Antwortschlüssel des Diensts. Der zweite verwendet übereinstimmendes Schlüsselmaterial für das neue
Paar. [Bootstrap-Implementierung](../../scripts/runtime-isolation/native-window/src/bootstrap.rs) ·
[Kanalvertrag](../data-format/native-win32-bp4-channel-v1-draft.md)

## Gemessene Binärdateien

| Gastdatei        | SHA-256                                                            |
| ---------------- | ------------------------------------------------------------------ |
| native-leaf.exe  | `3a95546bee978e2aede83f900f8c3a15e324a821771dde6d4e4af897b88e278e` |
| pipe-check.exe   | `e9ba4541794ee982233b7088aba03e089ac2245cd3bbf72f34aad4a1f0950480` |
| pipe-other.exe   | `04784020c602ec8ee6dad58e99a0ae31c644ecdce2e5afee820d65858de9785a` |
| pipe-service.exe | `da78af6a4cc1b515a0d3e04fc72fbad2c577276675abd8ca8cee8ccda6b07f5f` |
