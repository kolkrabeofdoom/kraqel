# BP4b · Generationsgebundener Endebeleg und einmalige Lesefreigabe

## Ergebnis vom 19. November 2026

| Sicherheitsfall | Beobachtung im Offline-Gast |
| --- | --- |
| Starterabbruch nach dauerhaftem Endebeleg | Schreibpaar 72/73, Jobs leer. Die fremde Generation wurde abgewiesen. Von zwei konkurrierenden Aufrufen startete genau einer das Lesepaar 74/75. Dieses bestätigte Text und Revision 1. Erneuter Aufruf abgewiesen. Dauerhafter Schlusszustand `read-consumed`. |
| Starterabbruch vor dauerhaftem Endebeleg | Schreibpaar 72/73, Jobs leer. Die Kontrollinstanz verweigerte die Freigabe. Kein Lesepaar, dauerhafter Schlusszustand `running`. |

Beide Läufe endeten mit Prüfcode 0. Die schreibgeschützte Plattenprüfung bestätigte die erwarteten Kontrollzustände, entfernte Rollenfreigaben, Rollenrechte, Programm-Hashes und Richtlinie. Der Gast war ohne Netzwerkadapter und aktivierte Integrationsdienste eingerichtet. Er ist ausgeschaltet und wieder mit seiner ursprünglichen Platte verbunden. Der ETW-Netzvergleich für die neuen Programm-Hashes steht aus.

## Umfang der Probe

Die Gastläufe verwenden zwei frische Differenzplatten und je eine neue synthetische SQLite-Ablage. Die zuvor unter `pending` gesperrte Ablage gehört zu einem anderen Gastabbild. Die Probe kennt keinen Entsperraufruf für dieses Abbild.

Ein lokaler Kontrollprozess bleibt vom Anlegen der Ablage bis zur Entscheidung am Leben. Er hält Verzeichnis und Datenbankdatei gegen Austausch fest und erzeugt eine zufällige Auftragsgeneration. Ein eigener Starter erhält deren Bootstrapwert über eine anonyme Eingabepipe. Der geschützte Kontrollvermerk bindet die Generation an den Paket-/Richtlinienanker, die Windows-Sitzung, die Volumen- und Dateiidentitäten beider gehaltenen Objekte sowie den festgelegten Ausgang „Commit von Szene A auf Revision 1“.

Der Starter verarbeitet die Speicherung über das authentisierte Rollenpaar. Bevor er den Endebeleg synchronisiert, müssen die gespeicherte Revision über den Kanal bestätigt, die SQLite-Verbindung geschlossen, beide Rollen beendet, beide gehaltenen Jobs leer und die Startfreigaben entfernt sein. Die Probe beendet den Starter hart mit Exitcode 101. Der Abbruchpunkt liegt wahlweise unmittelbar vor oder nach dem dauerhaften Endebeleg.

Vor der Lesefreigabe vergleicht die Kontrollinstanz den Endebeleg unter exklusiver Kontrollsperre mit ihrer eigenen Generation. Vor dem Start des neuen Paars verbraucht sie die Freigabe im Arbeitsspeicher und synchronisiert den Verbrauch auf Platte. Die neue Lesebindung verwendet eigene Prozesse, Kanalenden und Schlüssel. Das separate Leseprofil prüft auch, dass Windows einen Schreibhandle auf die Datenbank verweigert.

Der positive Pfad setzt voraus, dass die ursprüngliche Kontrollinstanz überlebt. Ein neuer Kontrollprozess erzeugt eine neue Generation und darf den gespeicherten Endebeleg nicht als eigenständiges Leseticket importieren. Vollständiger Maschinenneustart, Wiederherstellung eines laufenden VM-Speicherabbilds und eine Freigabe zum Schreiben liegen außerhalb dieses Nachweises.

## Wiederholen

Im vorbereiteten Offline-Gast den [Gasttreiber](../../scripts/runtime-isolation/native-window/pipe-guest.ps1) mit `-Trial proof-run` ausführen. Auf einer zweiten frischen Differenzplatte mit denselben Programm-Hashes `-Trial proof-incomplete` ausführen. Beide Läufe verwenden die [native Kontrollinstanz](../../scripts/runtime-isolation/native-window/src/release_proof.rs). Der Plattengegencheck prüft den dauerhaften Kontrollzustand, die Rechte und die Programm-Hashes auf dem schreibgeschützt eingebundenen Abbild. Er benötigt keine SQLite-Verbindung.

| Programm | SHA-256 in beiden Gastläufen |
| --- | --- |
| native-leaf.exe | F89BD417E42A7B79CDD96B313C5034CAC49356833C3314A317B36277C2A3ECC7 |
| pipe-check.exe | 55CE3130C51CA12C5A03500A1AF13AC6133A8FFA7ACD04F5AE40F709DA32AA05 |
| pipe-other.exe | 9B1C8561C4FACB3CE6CFA79AF7E5E727CEDAC8FBA431A7CDE4B96FAA77DF8CB3 |
| pipe-service.exe | 8299E653C28CA93835905DB834317C87BEAD46D5B8D12B314ECE05DDA81240A6 |
