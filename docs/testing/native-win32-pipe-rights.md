# BP4b · Pipe-Rechte vor Resume im Offline-Gast

## Prüfaufbau

**19. November 2026 · Bestanden.** Der Gastlauf unter erzwungener UMCI endete mit Exit 0. Vier
Objektidentitäten, Rechte und Attribute stimmten. Alle zwölf DACL-Gegenprüfungen und alle sechs
Prozesszugriffsversuche ergaben die erwartete Abweisung. UI und Datendienst blieben bis zur
Beendigung suspendiert. Beide Jobs waren danach leer. Die Ausgangsplatte ist wieder zugeordnet, der
Gast ist aus. Der schreibgeschützte Abgleich bestätigte Paket-DACL, entfernte Zielkennung, Image-
und Policyhashes. Bootstrap, Datentransfer und ETW des neuen Paars stehen aus.

Der Starter hält drei selbst erzeugte Prozessgenerationen an ihren Handles: das Win32-UI-Image, ein
synthetisches Blatt für die Datendienstrolle und eine unabhängige Gegenprobe. Jede Rolle hat eine
eigene AppContainer-SID, null Capabilities und einen eigenen Ein-Prozess-Job. Der gemeinsame
Startpfad prüft Image, Token, Sitzung, Job und Mitigations vor Resume. Die Datendienst-Fixture
enthält keinen Szenenspeicher. In diesem Versuch fließen keine Nutzdaten.
[Startpfad](../../scripts/runtime-isolation/native-window/src/suspended.rs) ·
[Prüfprogramm](../../scripts/runtime-isolation/native-window/src/bin/pipe-check.rs)

| Rolle        | Anfrage-Pipe | Antwort-Pipe |
| ------------ | ------------ | ------------ |
| Win32-UI     | schreiben    | lesen        |
| Datendienst  | lesen        | schreiben    |
| Fremdprozess | kein Ende    | kein Ende    |

Die beiden Pipes entstehen erst nach der ersten Handleinventur, alle drei Primärthreads sind zu
diesem Zeitpunkt suspendiert. Der Starter dupliziert genau die vier vorgesehenen Enden ohne
Vererbung. Für jedes Ende vergleicht er eine zurückgelesene Referenz mit dem ursprünglichen
Kernelobjekt. Nach dem Schließen seiner lokalen Pipehandles muss der zweite vollständige
Inventarabgleich exakt die vorherigen Handles plus die zwei vorgesehenen Einträge pro Ziel ergeben.
Leserechte sind exakt `FILE_READ_DATA | SYNCHRONIZE`, Schreibrechte exakt
`FILE_WRITE_DATA | SYNCHRONIZE`, jeweils mit Handleattributen null.

Die Gegenprobe verwendet eine dritte, unprivilegierte AppContainer-Rolle. Nur ihr Primärthread wird
fortgesetzt. Sie prüft ihren Token und einen erfolgreichen Kontrollaufruf am eigenen Prozess. An den
beiden weiter gehaltenen Zielgenerationen muss jeder Öffnungsversuch für `PROCESS_DUP_HANDLE`,
`PROCESS_VM_READ` und `PROCESS_VM_WRITE` einzeln mit `ERROR_ACCESS_DENIED` enden. Der Starter prüft
zusätzlich mit `AccessCheck` am ausgelesenen Sicherheitsdeskriptor jedes Pipe-Endes die Rechte des
tatsächlichen Fremdtokens für Lesen, Schreiben und Erzeugen einer Pipe-Instanz. Die Ergebnisse
gelten für diese Rolle und Gastkonfiguration.
[Fremdprozess](../../scripts/runtime-isolation/native-window/src/bin/pipe-other.rs)

## Wiederholung

Im Verzeichnis `scripts/runtime-isolation/native-window` mit MSVC x64,
`RUSTFLAGS=-C target-feature=+crt-static` und `cargo build --release --offline --locked` bauen. Wie
bei der Fensterprobe ein frisches Gastoverlay verwenden. Für diesen Lauf genau `native-leaf.exe`,
`pipe-service.exe`, `pipe-other.exe` und `pipe-check.exe` als Hashregeln in die erzwungene
Gastpolicy und in den geschützten Paketanker aufnehmen. In der interaktiven Gastsitzung
`pipe-guest.ps1` ausführen. Der Runner prüft Gastidentität, Build, Offlinezustand, durchgesetzte
Policy und Imagehashes. Exitcode, Ergebniszeilen, Jobende und Rückkehr zur ausgeschalteten
Ausgangsplatte prüfen. [Gastaufbau](native-win32-window-guest.md) ·
[Gast-Runner](../../scripts/runtime-isolation/native-window/pipe-guest.ps1) ·
[Kanalvertrag](../data-format/native-win32-bp4-channel-v1-draft.md)

## Geprüfte Binärdateien

| Gastdatei        | SHA-256                                                            |
| ---------------- | ------------------------------------------------------------------ |
| native-leaf.exe  | `b26143db751ab7914781baab0f5d74e8d78bbd97c392f47e7e01f3470ebd515a` |
| pipe-check.exe   | `72f9b8964540e37eb5ba1619dfa1408742ac37d740b0abf676ea180e47210c9b` |
| pipe-other.exe   | `9cad9dccd92286efd6a11486837a973698e9d2742572a5f4165875c3af35d3cc` |
| pipe-service.exe | `fac8ccfdad0939944de29b8a1a5ce655e8f1fa99f2e5086b6dffff34478e5a34` |
