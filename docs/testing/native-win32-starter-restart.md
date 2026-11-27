# BP4b · Gesperrter Wiederanlauf nach Starterabbruch

## Ergebnis vom 19. November 2026

Der Starter brach am Haltepunkt mit Exitcode 101 ab. Die Prüfrollen endeten. Alle drei neuen Starter
konnten die exklusive Kontrolldatei wieder öffnen und meldeten die Sperre (jeweils Exitcode 102).
Der Hash des Kontrollvermerks blieb bei jedem Versuch unverändert. Die unabhängige Prüfung auf der
schreibgeschützt eingebundenen Gastplatte bestätigte den zum Paketanker passenden Pending-Vermerk,
die geschützten Schreibrechte und die verbliebenen Startinformationen. Die synthetische Datenbank
wurde zur Nachprüfung nicht geöffnet. Ihr fachlicher Endstand bleibt ungeklärt.

Der Gast hatte keine Netzwerkadapter oder aktivierten Integrationsdienste. Er ist ausgeschaltet, die
ursprüngliche Gastplatte ist wieder zugeordnet.

## Ablauf

Im interaktiven Offline-Gast auf einer frischen Differenzplatte den
[Gasttreiber](../../scripts/runtime-isolation/native-window/pipe-guest.ps1) mit
`-Trial starter-restart` starten. Ein normal abgeschlossener Handshake dient als Positivkontrolle.
Der nächste Starter verwendet denselben Kontrollpfad, legt die isolierte synthetische Ablage an und
startet Oberfläche und Datendienst. Das Fenster sendet einen vollständigen, authentisierten
Schreibauftrag für Szene A. Es meldet die offene Anfrage erst nach der vorangegangenen erfolgreichen
Leseantwort.

Der Abbruchpunkt setzt ein sichtbares Fenster mit passender PID, beide lebenden Rollen und das
Rollbackjournal der offenen Transaktion voraus. Dort beendet sich der Starter hart (Exitcode 101).
Er kann weder Jobende noch Transaktionsende bestätigen. Der Gasttreiber beobachtet das Ende der
Rollen separat und ruft drei frische Starter auf.

| Neuer Start                     | Erwartung                                    |
| ------------------------------- | -------------------------------------------- |
| Lesebindung für dieselbe Ablage | Sperre vor Erzeugung des Lesepaars           |
| Neuer regulärer Auftrag         | Sperre vor der Rollen- und Kanalvorbereitung |
| Erneute Lesebindung             | Erneute Sperre, gleicher Kontrollvermerk     |

## Entscheidung unter der Kontrollsperre

Der native Starter öffnet `pipe.lock` exklusiv im geschützten Paketverzeichnis. Vor dem Anlegen
einer Ablage, einer Startfreigabe oder eines Prozesses schreibt und synchronisiert er `pending`
zusammen mit dem SHA-256 des Paketankers. Erst nach erfolgreichem Ablauf, geprüften leeren Jobs und
entfernter Startinformation schreibt und synchronisiert er `drained`. Schreibrechte im
Kontrollverzeichnis besitzen SYSTEM und die lokale Administrationsgruppe.

Ein neuer Starter akzeptiert einen vorhandenen Vermerk ausschließlich bei exakt passendem
Abschlusszustand, passendem Anker und fehlenden alten Startinformationen. Eine fehlende
Kontrolldatei erlaubt nur die Erstinitialisierung ohne bestehende Szenenablage oder
Startinformationen. Ein leerer Altvermerk, ein beschädigter Vermerk oder ein Fehler beim Lesen gibt
die Ablage nicht frei. Bei Sperre endet der Neustart vor SQLite, Profilanlage und Erzeugung der
Rollen. Es gibt in dieser Probe keinen automatischen Entsperrpfad.

Der Betriebssystem-Lock wird durch den Prozessabbruch frei. Der inhaltliche Endebeleg bleibt aus.
Die verbliebenen Startinformationen liegen zusammen mit der synthetischen Ablage im isolierten
Testabbild. Eine Zuordnung der tatsächlichen Dokumentrevision nach dem Abbruch würde eine gesonderte
Klärung erfordern.

| Programm         | SHA-256 des Gastlaufs                                            |
| ---------------- | ---------------------------------------------------------------- |
| native-leaf.exe  | E15606C1B681A3C74C5E7513AB18BE062A850530C5B791F380D273962F874885 |
| pipe-check.exe   | 4E5A160FC70348E3067FE72EFA7A4967CB5379933785643B571601D11855E1C6 |
| pipe-other.exe   | B963B4CD4A83DE06BC8B95975EC50859F535CC270A0F4A42EEDDC5FF32CE63C2 |
| pipe-service.exe | A07B0D5E3C4DFA6313B847A548029260C85ECDF1ED5E0BC032F21697D4703906 |

## Frischer ETW-Netzvergleich am 19. November 2026

Ein frisches Gastabbild verwendete die vier oben aufgeführten Programm-Hashes. ETW erfasste fünf
Startergenerationen mit den Exitcodes 0 (Positivkontrolle), 101 (Abbruch bei offener Transaktion)
und dreimal 102 (Sperre). Alle Rollen der abgebrochenen Generation endeten vor dem ersten Neustart.
Keiner der drei abgewiesenen Starter erzeugte einen Kindprozess.

| Messgröße                                              | Referenz | Starterabbruch und Neustarts |
| ------------------------------------------------------ | -------: | ---------------------------: |
| Ereignisse im Trace                                    |       66 |                           72 |
| Prozesse im gemessenen Baum einschließlich Hülle       |        3 |                           17 |
| Netzwerkereignisse im gemessenen Baum                  |        0 |                            0 |
| Gastweite Loopback-Ereignisse                          |       18 |                           18 |
| Gastweite übrige oder unbekannte Netzwerkziele         |        0 |                            0 |
| Erkannte TCP-/UDP-Positivkontrollen über IPv4 und IPv6 |      4/4 |                          4/4 |
| Verlorene Ereignisse und Puffer                        |        0 |                            0 |

Der Hash des Kontrollvermerks blieb über alle drei Neustarts gleich. Die schreibgeschützte Prüfung
des abgeschalteten Gastabbilds bestätigte den zum Paketanker passenden Pending-Vermerk, dessen
Schreibrechte und die verbliebenen Startinformationen. Die Datenbank blieb nach dem Abbruch
geschlossen. Die Rohspuren verbleiben lokal.

Für die Wiederholung im frischen Offline-Overlay den
[Gasttreiber](../../scripts/runtime-isolation/native-window/pipe-guest.ps1) als `etw-workload.ps1`
bereitstellen und den [ETW-Runner](../../scripts/runtime-isolation/bp3-umci/etw-guest.ps1) mit
`-ExerciseTrial starter-restart` ausführen. Nach Gastende beide ETLs mit `tracerpt -of XML -lr`
dekodieren und mit `python scripts/m3-network/bp3_guest.py --starter-restart <Ergebnisordner>`
auswerten.
