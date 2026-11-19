# BP4b · Generationsgebundener Endebeleg und einmalige Lesefreigabe

## Ergebnis vom 19. November 2026

| Sicherheitsfall | Beobachtung im Offline-Gast |
| --- | --- |
| Starterabbruch nach dauerhaftem Endebeleg | Schreibpaar 72/73, Jobs leer. Die fremde Generation wurde abgewiesen. Von zwei konkurrierenden Aufrufen startete genau einer das Lesepaar 74/75. Dieses bestätigte Text und Revision 1. Erneuter Aufruf abgewiesen. Dauerhafter Schlusszustand `read-consumed`. |
| Starterabbruch vor dauerhaftem Endebeleg | Schreibpaar 72/73, Jobs leer. Die Kontrollinstanz verweigerte die Freigabe. Kein Lesepaar, dauerhafter Schlusszustand `running`. |

Beide Läufe endeten mit Prüfcode 0. Die schreibgeschützte Plattenprüfung bestätigte die erwarteten Kontrollzustände, entfernte Rollenfreigaben, Rollenrechte, Programm-Hashes und Richtlinie. Der Gast war ohne Netzwerkadapter und aktivierte Integrationsdienste eingerichtet. Er ist ausgeschaltet und wieder mit seiner ursprünglichen Platte verbunden.

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

## ETW-Netzvergleich am 19. November 2026

Beide Messpaare liefen mit den oben genannten Binär-Hashes auf frischen Offline-Gastplatten. Die ursprüngliche gesperrte Ablage blieb unberührt. Der Abgleich von Elternprozessen, Lebenszeiten und Exitcodes erfasste im positiven Fall genau ein Lesepaar nach dem Starterabbruch. Im negativen Fall endete der Lauf nach der Schreibpaarung ohne Lesepaar.

| Messgröße | Referenz positiv | Freigabe | Referenz negativ | Verweigerung |
| --- | ---: | ---: | ---: | ---: |
| ETW-Ereignisse | 54 | 60 | 52 | 55 |
| Prozesse im Prüfbaum einschließlich Hülle | 3 | 11 | 3 | 8 |
| Netzwerkereignisse im Prüfbaum | 0 | 0 | 0 | 0 |
| Gastweite Loopback-Ereignisse | 18 | 18 | 18 | 18 |
| Gastweite übrige oder unbekannte Ziele | 0 | 0 | 0 | 0 |
| Erkannte TCP-/UDP-Kontrollen über IPv4 und IPv6 | 4/4 | 4/4 | 4/4 | 4/4 |
| Verlorene Ereignisse und Puffer | 0 | 0 | 0 | 0 |

Die Plattengegenprüfung bestätigte den Verbrauch im positiven und die fortbestehende Sperre im negativen Fall. Der Gast ist ausgeschaltet und wieder mit seiner ursprünglichen Platte verbunden. Rohspuren bleiben lokal.

Für beide Läufe den Gasttreiber als `etw-workload.ps1` bereitstellen. Pro frischer Platte den [ETW-Gasttreiber](../../scripts/runtime-isolation/bp3-umci/etw-guest.ps1) mit `-ExerciseTrial proof-run` bzw. `-ExerciseTrial proof-incomplete` ausführen. Nach dem Dekodieren der ETLs mit `tracerpt -of XML -lr` prüft `python scripts/m3-network/bp3_guest.py --proof <Fall> <Ergebnisordner>` die erwarteten Prozessgenerationen sowie Netz-, Kontroll- und Verlustereignisse.

## Ausfall der Kontrollinstanz vor dem Verbrauch

Der Gastlauf `-Trial proof-witness-loss` vom 19. November 2026 verwendete eine weitere frische Differenzplatte. Nach dem bestätigten Commit auf Revision 1 und dem Ende des Schreibpaars (72/73, beide Jobs leer) prüfte die ursprüngliche Kontrollinstanz den synchronisierten Endebeleg gegen ihre eigene Generation und die Entfernung der alten Startfreigaben. Unter der exklusiven Sperre endete sie hart mit Exitcode 103, bevor sie die Freigabe verbrauchte.

Eine neue Kontrollinstanz nahm den regulären Einstieg `proof-run`. Sie meldete die Sperre mit Exitcode 102 vor Ablagevorbereitung und Rollenstart. Kein Lesepaar entstand. Der SHA-256 des Kontrollvermerks war vor und nach dem Neustart identisch. Der schreibgeschützte Plattengegencheck bestätigte weiterhin `committed-revision-1/jobs-drained` und entfernte Startfreigaben. Die ursprüngliche gesperrte Ablage blieb unberührt. Der Gast ist ausgeschaltet und wieder mit der ursprünglichen Gastplatte verbunden.

Dieser gezielte Lauf prüfte ausschließlich den Ausfall der Kontrollinstanz. SHA-256 des zugehörigen Prüfstarters: `E4A47166DB5DEFE6EE5F9D1FE3F463B52321A85DC1DD97592AD05220840035F2`.

### ETW-Netzvergleich bei Ausfall der Kontrollinstanz

Am 19. November 2026 liefen Referenz und Ausfallprobe auf einer frischen Offline-Differenzplatte mit denselben vier Programm-Hashes wie im vorstehenden Gastlauf. ETW bestätigte das Ende des Ausstellers (101), danach das Ende der Kontrollinstanz (103) und anschließend einen gesperrten Neustart (102) ohne Kindprozess. Nur das ursprüngliche Schreibpaar lief. Der Endebeleg blieb beim Neustart bytegleich und unverbraucht. Die schreibgeschützte Plattenprüfung bestätigte seinen Zustand und die entfernten Startfreigaben, ohne SQLite zu öffnen.

| Messgröße | Referenz | Ausfall und Neustart |
| --- | ---: | ---: |
| ETW-Ereignisse / Prozesse im gemessenen Baum | 56 / 3 | 57 / 9 |
| Netzwerkereignisse im gemessenen Baum | 0 | 0 |
| Gastweite Loopback-Ereignisse / übrige oder unbekannte Ziele | 18 / 0 | 18 / 0 |
| Erkannte TCP-/UDP-Kontrollen über IPv4 und IPv6 | 4/4 | 4/4 |
| Verlorene Ereignisse und Puffer | 0 | 0 |

Für die Wiederholung gilt der oben beschriebene ETW-Ablauf mit `-ExerciseTrial proof-witness-loss` und der Auswertung `--proof proof-witness-loss`. Die ursprüngliche gesperrte Ablage blieb unberührt, die Rohspuren lokal. Der Gast ist ausgeschaltet und wieder mit der ursprünglichen Gastplatte verbunden.

## Ausfall nach Verbrauch und vor dem Rollenstart

Der gezielte Gastlauf `-Trial proof-consumed-loss` vom 19. November 2026 nutzte eine neue synthetische Ablage. Die Kontrollinstanz verbrauchte die Freigabe unter exklusiver Sperre und synchronisierte `read-consumed`. Unmittelbar danach endete sie hart mit Exitcode 104, noch vor dem Aufruf zur Erzeugung des Lesepaars. Der reguläre Neustart endete vor der Ablagevorbereitung mit Exitcode 102. Es entstand kein Lesepaar. Der Hash des Kontrollvermerks blieb über den Neustart gleich.

Die schreibgeschützte Nachprüfung bestätigte `read-consumed` und die Entfernung der alten Startfreigaben, ohne SQLite zu öffnen. Die ursprüngliche gesperrte Ablage blieb unberührt. Der Gast ist ausgeschaltet und wieder mit der ursprünglichen Platte verbunden. Der erste Versuch endete vor dem Neustart an einer durch die Gast-Sprachrichtlinie gesperrten Diagnosemethode. Der vollständige Nachweis stammt vom Wiederholungslauf mit unveränderter Richtlinie und identischem Prüfprogramm.

### ETW-Netzvergleich nach Verbrauch der Lesefreigabe

Am 19. November 2026 liefen Referenz und Prüffall auf einer frischen Offline-Gastplatte. Alle vier Programm-Hashes stimmten mit dem vorstehenden Funktionslauf überein. ETW erfasste das ursprüngliche Schreibpaar, das Ende des Ausstellers (101), den Ausfall der Kontrollinstanz nach synchronisiertem Verbrauch (104) und den gesperrten Neustart (102). Es entstand kein Lesepaar. Der neu gestartete Prozess hatte keine Kindprozesse.

| Messgröße | Referenz | Ausfall und Neustart |
| --- | ---: | ---: |
| ETW-Ereignisse / Prozesse im gemessenen Baum | 70 / 3 | 57 / 9 |
| Netzwerkereignisse im gemessenen Baum | 0 | 0 |
| Gastweite Loopback-Ereignisse / übrige oder unbekannte Ziele | 18 / 0 | 18 / 0 |
| Erkannte TCP-/UDP-Kontrollen über IPv4 und IPv6 | 4/4 | 4/4 |
| Verlorene Ereignisse und Puffer | 0 | 0 |

Der Hash des Verbrauchsvermerks blieb über den Neustart gleich. Die schreibgeschützte Plattenprüfung bestätigte `read-consumed` und entfernte Startfreigaben, ohne die Datenbank zu öffnen. Die ursprüngliche gesperrte Ablage blieb unberührt. Der Gast ist ausgeschaltet und wieder mit der ursprünglichen Gastplatte verbunden. Zur Wiederholung den bestehenden ETW-Ablauf mit `-ExerciseTrial proof-consumed-loss` und der Auswertung `--proof proof-consumed-loss` verwenden. Rohspuren bleiben lokal.

## Ausfall bei laufendem Lesepaar

Auf einer frischen synthetischen Differenzplatte prüft `-Trial proof-live-loss` den Zeitraum nach dem Lesen von Szene A auf Revision 1. Das Fenster setzt seinen Bereitschaftstitel erst nach dem Rücklesen des angezeigten Texts. Der Datendienst hält die schreibgeschützt geöffnete Ablage. Ein separater Beobachtungsprozess gleicht beide Prozessgenerationen mit den geschützten Startbelegen ab und hält ausschließlich Prozesshandles für Abfrage und Synchronisation. Er prüft, dass beide Rollen leben, beendet die Kontrollinstanz mit Code 105 und wartet höchstens fünf Sekunden auf beide Prozessenden. Der anschließende reguläre Neustart muss vor der Ablagevorbereitung mit Code 102 enden. Der Hash von `pipe.lock` muss über diesen Neustart gleich bleiben.

Der Gastlauf am 19. November 2026 bestand mit Prüfcode 0. Beide ursprünglichen Prozesshandles signalisierten innerhalb des Fünfsekundenfensters das Ende. Der Neustart endete mit Code 102, der Verbrauchsvermerk blieb unverändert. Die schreibgeschützte Nachprüfung bestätigte `read-consumed` und die zurückgebliebenen Startbelege der beendeten Generationen. Der Gast war ohne Netzwerkadapter und aktivierte Integrationsdienste eingerichtet. Er ist ausgeschaltet und wieder mit der ursprünglichen Platte verbunden.


### ETW-Netzvergleich bei laufendem Lesepaar

Am 19. November 2026 liefen Referenz und Ausfallprüfung auf einer frischen synthetischen Differenzplatte. Die Hashes aller vier Programme stimmten mit dem Funktionslauf überein. Der Beobachtungsprozess bestätigte beide lebenden Lesegenerationen unmittelbar vor dem Abbruch der Kontrollinstanz (105). ETW erfasste das Ende beider Rollen vor dem Ende des Beobachtungsprozesses und den anschließenden gesperrten Neustart (102) ohne Kindprozess. Der Verbrauchsvermerk blieb über den Neustart unverändert.

| Messgröße | Referenz | Ausfall und Neustart |
| --- | ---: | ---: |
| ETW-Ereignisse / Prozesse im gemessenen Baum | 56 / 3 | 64 / 13 |
| Netzwerkereignisse im gemessenen Baum | 0 | 0 |
| Gastweite Loopback-Ereignisse / übrige oder unbekannte Ziele | 18 / 0 | 18 / 0 |
| Erkannte TCP-/UDP-Kontrollen über IPv4 und IPv6 | 4/4 | 4/4 |
| Verlorene Ereignisse und Puffer | 0 | 0 |

Der schreibgeschützte Gegencheck bestätigte den Verbrauchszustand, die verbliebenen Startbelege, die Rollenrechte und die erzwungene Richtlinie. Der Gast ist ausgeschaltet und wieder mit der ursprünglichen Platte verbunden. Zur Wiederholung den ETW-Gasttreiber mit `-ExerciseTrial proof-live-loss` und die Auswertung mit `--proof proof-live-loss` verwenden. Die Rohspuren bleiben lokal.


## Ausfall nach Ende des Lesepaars vor der Bereinigung

Der gezielte Gastlauf `-Trial proof-pre-cleanup-loss` vom 19. November 2026 verwendete eine frische synthetische Ablage. Nach dem Lesen von Szene A auf Revision 1 endeten Datendienst (74) und Fenster (75) regulär. Erst nachdem die Kontrollinstanz beide Exitcodes und beide leeren Prozessjobs geprüft hatte, beendete sie sich hart mit Code 106. Zu diesem Zeitpunkt lagen die zwei Startbelege und die Prozessliste noch vor.

Der reguläre Neustart endete vor der Ablagevorbereitung mit Code 102. Die Hashes des Verbrauchsvermerks und aller drei zurückgebliebenen Dateien blieben über den Neustart gleich. Die schreibgeschützte Nachprüfung bestätigte den Zustand `read-consumed`, die Rollenrechte und die erzwungene Richtlinie. Der Gast ist ausgeschaltet und wieder mit der ursprünglichen Platte verbunden.


### ETW-Netzvergleich vor der Bereinigung

Am 19. November 2026 liefen Referenz und Prüffall auf einer frischen synthetischen Differenzplatte. Alle vier Programm-Hashes entsprachen dem vorstehenden Funktionslauf. ETW bestätigte das reguläre Ende von Datendienst (74) und Fenster (75) vor dem Ausfall der Kontrollinstanz (106) und den darauffolgenden Neustart (102) ohne Kindprozess. Der Gasttreiber bestätigte die leeren Jobs am Ausfallpunkt und identische Hashes für den Verbrauchsvermerk, beide Startbelege und die Prozessliste vor und nach dem Neustart.

| Messgröße | Referenz | Ausfall und Neustart |
| --- | ---: | ---: |
| ETW-Ereignisse / Prozesse im gemessenen Baum | 57 / 3 | 64 / 12 |
| Netzwerkereignisse im gemessenen Baum | 0 | 0 |
| Gastweite Loopback-Ereignisse / übrige oder unbekannte Ziele | 18 / 0 | 18 / 0 |
| Erkannte TCP-/UDP-Kontrollen über IPv4 und IPv6 | 4/4 | 4/4 |
| Verlorene Ereignisse und Puffer | 0 | 0 |

Die schreibgeschützte Nachprüfung bestätigte den Verbrauchszustand, die verbliebenen Startbelege, die Rollenrechte und die erzwungene Richtlinie. Der Gast ist ausgeschaltet und wieder mit der ursprünglichen Platte verbunden. Der ETW-Gasttreiber und die Auswertung verwenden für diesen Fall `-ExerciseTrial proof-pre-cleanup-loss` bzw. `--proof proof-pre-cleanup-loss`. Rohspuren bleiben lokal.


## Ausfall nach Entfernung des ersten Startbelegs

Der Gastlauf `-Trial proof-partial-cleanup-loss` vom 19. November 2026 verwendete eine frische synthetische Ablage. Nach regulärem Ende beider Leserollen und Prüfung der leeren Jobs entfernte die Kontrollinstanz die Prozessliste und den Fensterbeleg. Unmittelbar nach der Prüfung, dass `window.start` fehlte und `data.start` noch vorlag, endete sie hart mit Code 107.

Der reguläre Neustart blieb vor der Ablagevorbereitung mit Code 102 gesperrt. Die Hashes von `pipe.lock` und `data.start` blieben gleich. Fensterbeleg und Prozessliste fehlten auch nach dem Neustart. Die schreibgeschützte Nachprüfung bestätigte dieses Inventar und den Zustand `read-consumed`. Beim ersten Versuch verglich das Prüfskript nur das erste Zeichen des einzelnen Beleg-Hashes. Nach Korrektur der Auswertung bestand der Wiederholungslauf mit identischen Binärdateien auf einer frischen Differenzplatte. Der Gast ist ausgeschaltet und wieder mit der ursprünglichen Platte verbunden.


### ETW-Netzvergleich nach Entfernung des Fensterbelegs

Der frische Vergleich vom 19. November 2026 verwendete die vier hashgleichen Programme des bestandenen Funktionslaufs auf einer neuen synthetischen Differenzplatte. ETW erfasste das reguläre Ende von Datendienst (74) und Fenster (75), den Ausfall der Kontrollinstanz (107) und den gesperrten Neustart (102) ohne Kindprozess. Der bestehende Gasttreiber bestätigte den unveränderten Verbrauchsvermerk und Datendienstbeleg über den Neustart. Fensterbeleg und Prozessliste fehlten weiterhin.

| Messgröße | Referenz | Ausfall und Neustart |
| --- | ---: | ---: |
| ETW-Ereignisse / Prozesse im gemessenen Baum | 70 / 3 | 62 / 12 |
| Netzwerkereignisse im gemessenen Baum | 0 | 0 |
| Gastweite Loopback-Ereignisse / übrige oder unbekannte Ziele | 18 / 0 | 18 / 0 |
| Erkannte TCP-/UDP-Kontrollen über IPv4 und IPv6 | 4/4 | 4/4 |
| Verlorene Ereignisse und Puffer | 0 | 0 |

Die schreibgeschützte Nachprüfung bestätigte den Verbrauchszustand und den verbliebenen Datendienstbeleg. Der Gast ist ausgeschaltet und wieder mit der ursprünglichen Platte verbunden. Für diesen Fall verwenden der bestehende ETW-Gasttreiber und die Auswertung `-ExerciseTrial proof-partial-cleanup-loss` bzw. `--proof proof-partial-cleanup-loss`. Rohspuren bleiben lokal.


## Ausfall nach Entfernung des letzten Startbelegs

Der gezielte Gastlauf `-Trial proof-post-cleanup-loss` vom 19. November 2026 bestand auf einer frischen synthetischen Ablage. Beide Leserollen endeten regulär und ihre Prozessjobs waren leer. Nach Entfernung des letzten Startbelegs prüfte die Kontrollinstanz, dass `window.start`, `data.start` und `pipe-targets.txt` fehlten. Noch unter derselben Kontrollsperre endete sie mit Code 108.

Der reguläre Neustart blieb vor der Ablagevorbereitung mit Code 102 gesperrt. Der Hash von `pipe.lock` blieb gleich. Beide Startbelege und die Prozessliste fehlten vor und nach dem Neustart. Die schreibgeschützte Nachprüfung bestätigte dieses Inventar und den dauerhaften Zustand `read-consumed`. Der Gast ist ausgeschaltet und wieder mit der ursprünglichen Platte verbunden.


### ETW-Netzvergleich nach Entfernung des letzten Startbelegs

Am 19. November 2026 liefen Referenz und Ausfallprüfung auf einer frischen synthetischen Differenzplatte mit denselben vier Programm-Hashes wie im Funktionslauf. ETW erfasste das reguläre Ende beider Leserollen (74 und 75), den Ausfall der Kontrollinstanz (108) und den gesperrten Neustart (102) ohne Kindprozess. Der Gasttreiber bestätigte den unveränderten Hash des Verbrauchsvermerks. Beide Startbelege und die Prozessliste fehlten vor und nach dem Neustart.

| Messgröße | Referenz | Ausfall und Neustart |
| --- | ---: | ---: |
| ETW-Ereignisse / Prozesse im gemessenen Baum | 54 / 3 | 63 / 12 |
| Netzwerkereignisse im gemessenen Baum | 0 | 0 |
| Gastweite Loopback-Ereignisse / übrige oder unbekannte Ziele | 18 / 0 | 18 / 0 |
| Erkannte TCP-/UDP-Kontrollen über IPv4 und IPv6 | 4/4 | 4/4 |
| Verlorene Ereignisse und Puffer | 0 | 0 |

Die schreibgeschützte Nachprüfung bestätigte den Verbrauchszustand und das Fehlen der Startartefakte. Der Gast ist ausgeschaltet und wieder mit der ursprünglichen Platte verbunden. Für die Wiederholung verwenden der bestehende ETW-Gasttreiber und die Auswertung `-ExerciseTrial proof-post-cleanup-loss` bzw. `--proof proof-post-cleanup-loss`. Rohspuren bleiben lokal.
