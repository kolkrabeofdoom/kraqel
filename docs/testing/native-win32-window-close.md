# BP4b · Fensterende vor Commit

## Ergebnis vom 19. November 2026

Das sichtbare Fenster wurde im Einzel- und im anschließenden ETW-Lauf per WM_CLOSE geschlossen. Der Datendienst beendete sich nach Rollback innerhalb der geprüften Fünf-Sekunden-Grenze. Beide Jobs waren leer. Das frische Lesepaar und die unabhängige Plattenprüfung bestätigten jeweils Szene A mit Revision 0 und den ursprünglichen Absätzen „Rabe“ und „42“. Auch der Suchindex enthielt „42“, ohne Treffer für „43“.

| Messgröße | Referenz | Teilübertragung, Timeout, Fensterende und drei Wiederöffnungen |
| --- | ---: | ---: |
| ETW-Ereignisse | 47 | 111 |
| Prozesse im gemessenen Baum einschließlich Hülle | 3 | 22 |
| Native Prüfprozesse | 0 | 19 |
| Netzwerkereignisse im gemessenen Baum | 0 | 0 |
| Gastweite Loopback-Ereignisse | 18 | 18 |
| Gastweite übrige oder unbekannte Netzwerkziele | 0 | 0 |
| Erkannte TCP-/UDP-Positivkontrollen über IPv4 und IPv6 | 4/4 | 4/4 |
| Verlorene Ereignisse und Puffer | 0 | 0 |

Der Prüflauf erfasste einen Starter und sechs Generationen mit je Fenster, Datendienst und Fremdprozess zur Rechteprüfung. Die Auswertung bestätigte Elternbeziehungen, Prozessenden und die erwarteten Exitcodes. Im zweiten Overlay bestätigte die unabhängige Nachprüfung alle drei SQLite-Ablagen auf Revision 0. Beide Abbilder wurden schreibgeschützt geprüft. Der Gast ist ausgeschaltet und wieder mit der ursprünglichen Platte verbunden. Es waren keine Netzwerkadapter oder aktivierten Integrationsdienste angebunden.

## Was der Gastlauf prüft

Im interaktiven Offline-Gast startet das geprüfte native Prozesspaar unter erzwungener UMCI. Die Oberfläche liest Szene A mit Revision 0 und sendet über denselben authentisierten Kanal eine vollständige, gültige Schreibanfrage für Revision 0. Der Datendienst aktualisiert Dokument und Suchindex in einer SQLite-Transaktion. Ein synthetischer Haltepunkt hält die Transaktion vor dem Commit offen.

Sobald das sichtbare Fenster die ausstehende Anfrage meldet und das Rollbackjournal der frischen Testablage existiert, prüft der Starter die Fenster-PID sowie beide laufenden Prozesse. Er sendet die reguläre Win32-Nachricht `WM_CLOSE` an dieses Fenster. Die Nachrichtenschleife verlangt die tatsächliche Zerstörung des Fensters. Mit der Rückkehr aus der Schleife schließt die Oberfläche ihre Pipe-Enden. Der Datendienst akzeptiert am Haltepunkt nur EOF bzw. Broken Pipe, rollt die Transaktion zurück und beendet sich. Der Starter wartet höchstens fünf Sekunden auf dessen reguläres Ende und prüft die Exitcodes und beide Jobs.

Ein frisches Prozesspaar prüft mit eigener authentisierter Lesebindung Text und Revision. Die unabhängige Prüfung auf dem schreibgeschützt eingebundenen Gastabbild vergleicht zusätzlich strukturiertes Dokument, Klartext, SQLite-Integrität und Suchindex. Für einen schon abgeschlossenen Commit gilt der separate [Nachweis zum Abbruch nach Commit](native-win32-commit-crashes.md).

## Wiederholen

1. Im vorbereiteten Offline-Gast auf frischer Differenzplatte den [Gasttreiber](../../scripts/runtime-isolation/native-window/pipe-guest.ps1) mit `-Trial window-close` starten. Erwartete Exitcode-Paare für Dienst/Fenster sind 97/98 und beim Wiederöffnen 99/100.
2. Für den Netzvergleich ein zweites frisches Gastabbild mit denselben vier Programm-Hashes verwenden. Neben dem Gasttreiber den [ETW-Workload](../../scripts/runtime-isolation/native-window/channel-end-workload.ps1) als `etw-workload.ps1` im Setupverzeichnis bereitstellen. Der vorhandene [ETW-Gasttreiber](../../scripts/runtime-isolation/bp3-umci/etw-guest.ps1) führt Referenz und Prüflauf aus. Der Prüflauf umfasst Teilübertragung, Timeout und Fensterende, jeweils mit frischer Lesebindung.
3. Beide ETLs nach Gastende mit `tracerpt -of XML -lr` als `reference.xml` und `exercise.xml` dekodieren. Die Laufbelege und Prozessgenerationen mit `python scripts/m3-network/bp3_guest.py --channel-ends <Ergebnisordner>` prüfen. Rohspuren verbleiben lokal.

| Programm | SHA-256 für Einzellauf und ETW-Prüfpaar |
| --- | --- |
| native-leaf.exe | E15606C1B681A3C74C5E7513AB18BE062A850530C5B791F380D273962F874885 |
| pipe-check.exe | 339E49278A57C678F3FFC1F493729B401456A61DA8A21116CC8A834A9F4E6307 |
| pipe-other.exe | B963B4CD4A83DE06BC8B95975EC50859F535CC270A0F4A42EEDDC5FF32CE63C2 |
| pipe-service.exe | A07B0D5E3C4DFA6313B847A548029260C85ECDF1ED5E0BC032F21697D4703906 |
