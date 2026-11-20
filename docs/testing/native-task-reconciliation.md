# KQ-069: Dauerhafter Verbrauch und Crash-Reconciliation

Stand: 16. November 2026. [Vertrag und Grenzen](../data-format/native-task-journal-v1.md).
Implementiert im Forschungsadapter und vor Auswahl/Import der isolierten Forschungs-GUI angebunden.
Normale Produktstarts bleiben unverändert; striktes Datenschutzziel und #134 bleiben offen.

## Automatisierter Nachweis auf diesem Windows-Rechner

Sieben neue aktive Rust-Integrationstests bestanden, ein Testkind ist bewusst ignoriert und wird nur
vom Prozess-Test explizit gestartet. Die sieben bisherigen Adaptertests bestehen ebenfalls.

- Erfolg bleibt nach Schließen/Öffnen als Receipt erhalten. Dieselbe Kennung wird ohne erneutes
  Lesen abgelehnt; eine Projektkopie und kein Zwischenstand.
- Abbruch, Ablauf, ungültiges Archiv, falsche Bindung und Abbruch nach Vorbereitung bleiben nach
  Öffnen verbraucht. Keine veröffentlichte Kopie oder Zwischenreste.
- Zweite Store-Instanz scheitert während der Eigentümersperre. Andere Projektwurzel scheitert an der
  gespeicherten Bindung.
- Nach einer unterbrochenen Ausführung verhindert auch dieselbe Store-Instanz einen zweiten
  Leseversuch. Eine späte Ablehnung nach Commit-Intent beziehungsweise Rename darf den gespeicherten
  Intent nicht überschreiben; beide Zustände bleiben für die Reconciliation erhalten.
- Beschädigtes/fehlendes Journal, unbekannte Version, fremdes Zwischenverzeichnis, ungültiger
  Commit-Receipt und beschädigtes veröffentlichtes Ziel werden nicht still repariert. Das
  mehrdeutige Ziel und seine Daten bleiben erhalten.
- Sieben echte Testkind-Prozesse werden vom Elternprozess an beobachteten Haltepunkten hart beendet:
  nach Claim, beim Lesen, während SQLite-Initialisierung, nach Vorbereitung, nach gespeichertem
  Commit-Intent, nach Rename und nach gespeichertem Abschluss. Kein bloßes Simulieren eines
  Fehler-Rückgabewerts und kein Rust-Drop als Cleanup.
- Nach jedem Kill wird der Store zweimal geöffnet: vor Rename terminal `INTERRUPTED`, null neue
  Kopien; nach Rename genau eine zugeordnete Kopie mit zwei Dokumenten. Kennung stets gesperrt,
  Zwischenverzeichnisse leer. Ein zusätzliches Bestandsprojekt bleibt in jeder Variante bytegleich.
  Auch während jedes lebenden Testkindes wird eine zweite Store-Instanz abgelehnt.

Die Testkinder sind native Rust-Testprozesse außerhalb des AppContainers und enthalten keine
WebView. Ein zusätzlicher Unix-Symlinktest ist im Code für Linux vorgesehen; er ist auf diesem
Windows-Rechner nicht ausgeführt und wird nur bei tatsächlich grüner Linux-CI als dort bestanden
gewertet. Keine pauschale Plattformabnahme.

Reproduktion:

```powershell
cargo test -p kraqel-isolation-test-adapter
cargo clippy -p kraqel-desktop --features isolation-ui-test --all-targets -- -D warnings
cargo test --locked --manifest-path scripts/runtime-isolation/probe/Cargo.toml
pnpm check
```

Feature-/Helfer-Clippy und 21 aktive Helfer-/Transporttests bestanden. Der native Dialogtest bleibt
in der normalen Testsuite absichtlich ignoriert. Keine zusätzliche Registrierung oder Paketversion
im Abhängigkeitsbaum; beide Lockfiles enthalten nur die neuen direkten Verweise auf bereits
vorhandene Bibliotheken.

`pnpm check` und alle 34 Browser-E2E bestanden. Nach der nativen Serie wurden die zusätzliche Sperre
gegen wiederholtes Lesen derselben Store-Instanz und gegen spätes Überschreiben eines Commit-Intents
ergänzt und regressionsgeprüft. Zudem wurde die neue `STATE_UNAVAILABLE`-Ansage präzisiert: Ein
unklarer Abschluss darf nicht behaupten, es sei noch kein Import gestartet worden. Die folgenden
Hashes bezeichnen den tatsächlich nativ bedienten Build vor diesen Änderungen, keine pauschale
Abnahme aller späteren Binärdateien.

## Native Desktop-Rückprüfung

Zwei frische AppContainer-Sitzungen desselben Forschungsbuilds ohne Capabilities. Bedienung durch
die prüfende Person, anschließend unabhängige Kontrolle der Rust-Ergebnisse und lokalen Daten:

| Fall                                            | Rückmeldung                                           | Unabhängiger Befund                                                                      |
| ----------------------------------------------- | ----------------------------------------------------- | ---------------------------------------------------------------------------------------- |
| Erfolg, Replay, Schließen                       | Zwei Dokumente und verbrauchter Auftrag bestätigt     | `succeeded`, genau eine Projektkopie, null Zwischenverzeichnisse, Journalintegrität `ok` |
| Vorbereitete Kopie abbrechen, Replay, Schließen | Abbruch ohne Kopie und verbrauchter Auftrag bestätigt | `failed` mit `CANCELLED`, null Projekte/Zwischenverzeichnisse, Journalintegrität `ok`    |

Beide Controller/App-Exitcodes 0, Auswahlthread beendet, null Jobprozesse, Profile entfernt. Keine
neuen Hörzusagen aus dieser Rückprüfung; die separate
[assistive Serie](native-live-assistive-focus.md) hat ihre eigene Buildbindung.

App-SHA-256: `1c83de0066921a8983637bed56431a768cc1fa19494f9b44c1def13674def7a4`. Controller-SHA-256:
`85bc27711906fa344584c32b8720687c6ae7783b4f663b679e8759031a1764f2`. Rohdiagnosen, UUIDs, Pfade und
Builds bleiben lokal. Keine neue ETW-Messung für dieses Paar; frühere Netzbelege gelten
ausschließlich für deren jeweilige Hashes.

## Verbleibende Abnahme

Nach den oben ausgewiesenen Schlusskorrekturen wurden Forschungs-App und Controller erneut gebaut
und getrennt gesichert, ohne eine weitere native Bedienserie oder ETW-Messung zu behaupten: App
`e42afef6551d7ca6b0fb1c7fd2c9a97b5992fd0118a652e48be702e9832fd135`, Controller
`a47e6fa17d0dc69833f61d0c670325d3a29039e04ca7b1ac6b2a7688babb18d6`. Der normale Debug-Build wurde
anschließend wiederhergestellt.

Der vollständige harte Crash samt authentisiertem Neustart derselben isolierten GUI mit stabiler
Datenwurzel ist noch offen. Die bestehenden Controller unterstützen weiterhin nur frische
Forschungswurzeln. Die sieben Prozess-Kill-Tests ersetzen diesen integrierten Nachweis nicht. Ebenso
offen: Stromausfall-/Datenträgerfehler, assistive Wiederanlaufanzeige, übrige Formate und externe
Brokergrenzen, frische Netzabnahme des späteren finalen Builds sowie Produkt-/Packaging-
Integration. Q-01/Q-02 für das Produkt bleiben nicht bestanden.
