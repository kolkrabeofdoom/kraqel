# BP4b · Datendienstabbruch an der SQLite-Commitgrenze

Am 19. November 2026 lief die Probe im interaktiven Offline-Gast mit erzwungener UMCI. Jeder Fall begann in einer eigenen isolierten Testablage mit Szene A auf Revision 0. Die Oberfläche las den Ausgangstext und sendete genau einen authentisierten Schreibauftrag mit erwarteter Revision 0.

| Zeitpunkt des erzwungenen Prozessendes | Antwortbytes auf den Schreibauftrag | Szene A unter frischer Lesebindung | Suchindex |
| --- | --- | --- | --- |
| Nach Dokument- und Indexänderung und Cache-Flush, unmittelbar vor Commit | 0 | Revision 0, zwei Absätze „Rabe“ und „42“ | Treffer für 42, keiner für 43 |
| Nach erfolgreichem Commit, vor der Antwort | 0 | Revision 1, zwei Absätze „Rabe“ und „43“ | Treffer für 43, keiner für 42 |

Beim ersten Wiederöffnen durfte SQLite im Datendienstprofil das Rollbackjournal zurückspielen. Die Anwendung führte dabei nur eine Leseabfrage aus. Danach prüfte ein frischer Prozess mit eigenem Leseprofil den alten Stand. Im zweiten Fall öffnete das Leseprofil direkt den bestätigten Plattenstand. In beiden Fällen verweigerte Windows dem Leseprofil den Schreibzugriff auf die Datenbank.

Alle fünf Prozesspaare endeten mit leeren Jobs. Eine separate Prüfung auf der schreibgeschützt eingebundenen Gastplatte bestätigte für beide Ablagen genau eine Szene, Dokumentinhalt, Klartext, Revision, Suchindex, Dateirechte und SQLite-Integrität. Der Gast ist ausgeschaltet, die ursprüngliche Gastplatte wieder zugeordnet. Im Gast waren keine Netzwerkadapter oder Integrationsdienste angebunden.

## Wiederholen

Im vorbereiteten Offline-Gast auf frischer Testplatte den Gasttreiber `scripts/runtime-isolation/native-window/pipe-guest.ps1 -Trial commit-crashes` verwenden. Erwartete Exitcode-Paare (Datendienst/Oberfläche) in Reihenfolge sind 80/82, 83/84, 85/86, 81/82 und 87/88. Der Nachweis gilt für den abrupten Datendienstabbruch bei laufendem Gast. Der frische ETW-Netzvergleich für diese Binärdateien steht aus.

| Programm | SHA-256 |
| --- | --- |
| native-leaf.exe | 168D19A9065AD55079AF316B8AE5C1EA2BBCBC7B56594380F0BB8AF99E87FAF2 |
| pipe-check.exe | EDD04B3B829751CB5B7ED5E6E73EAAE00FAC90C08DA2A928B7ECFC9206D3489B |
| pipe-other.exe | 262B8ED470B4253656539507CBD91D5CB64F27A50B117A55529666F5DB4A23F9 |
| pipe-service.exe | 3C2BC329EB128D127062A32C34D4F95E00ACE19785AAFA2BD14E500A1E50F2D1 |
