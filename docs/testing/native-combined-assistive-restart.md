# KQ-069: Assistiver kombinierter Wiederanlauf

Stand: 16. November 2026. Ausschließlich Windows-Forschung und synthetische Testarchive; keine
Produktfreigabe. [Aggregate und Hashes](native-combined-assistive-restart.json),
[Brokervertrag v3](../data-format/native-broker-recovery-v3.md).

## Gesichertes Paar und Verfahren

Die App bleibt binär unverändert gegenüber dem
[kombinierten ETW-Paar](native-combined-live-restart-etw.md). Der Helfer ergänzt
`ui-broker-recover <Wurzel> manual`; sein neuer Hash besitzt noch keinen eigenen ETW-Beleg. Beide
Hashes stehen in den Aggregaten. Lokal gesichert unter
`%TEMP%/kraqel-kq069-combined-assistive-build-20261116-01/` mit Buildmanifest. Frische Testwurzeln
sind erforderlich; alte MAC-Wurzeln sind an ihren ursprünglichen Helfer gebunden.

Manuelle Recovery authentisiert die bestehende Kette, erwirbt die exklusive Sperre und wartet auf
Jobleerstand. Das Ticket enthält keine Auswahlhandles, die Bedienungsautomatik ist aus. Ein nativer
Replay-Aufruf und reguläres Appende mit Code 0 sind zwingend. Danach prüft eine automatische
Generation die ursprüngliche, verbrauchte Importanfrage, ohne erneut auszuwählen oder das Journal zu
verändern. Erst anschließend entsteht der MAC-Abschlussbeleg. Die Diagnoseindizes sind
`sequence * 8 + 5` und `+ 6`; bestehende automatische Modi bleiben erhalten.

## Hörprüfung und technische Gegenbelege

Die prüfende Person verwendete die Windows-Sprachausgabe. Bei den folgenden vier akzeptierten
Wiederanläufen wurden die Zustandsmeldung, Fokus auf „Erneuten Aufruf prüfen“ und nach einem
ausdrücklichen Mausklick der angesagte Verbrauchshinweis bestätigt. Vor dem Schließen wurden jeweils
der native Replay-Beleg und die Fokus-/Aktionsdiagnose gegengeprüft.

| Fall                                           | Gehörter Zustand                                        | Nativer Gegenbeleg         |
| ---------------------------------------------- | ------------------------------------------------------- | -------------------------- |
| Unterbrechung nach echter nativer Dateiauswahl | Früherer Import unterbrochen, Auftrag bleibt verbraucht | `INTERRUPTED`, 0 Kopien    |
| Abgeschlossener frischer Fixture-Auftrag       | 2 Dokumente wiederhergestellt, keine neue Projektkopie  | `succeeded`, genau 1 Kopie |
| Synthetisch ausgelöster Auswahlabbruch         | Import abgebrochen, keine Projektkopie                  | `CANCELLED`, 0 Kopien      |
| Ungültiges synthetisches Archiv                | Import konnte nicht abgeschlossen werden                | `IMPORT_FAILED`, 0 Kopien  |

Alle vier akzeptierten manuellen Generationen: reguläres Ende 0, mindestens drei gebundene
Ereignisse, Replay gesperrt, Job leer, Staging leer. Alle vier anschließenden Altanfragen wurden
abgewiesen; deren Testende 77 ist eine kontrollierte Beendigung und kein reguläres Appende. Vier
Abschlussbelege wurden unabhängig kryptografisch geprüft. Vier Profile nach authentisierter
Stilllegung entfernt; keine KRAQEL-Forschungsprozesse am Ende vorhanden.

Die unabhängige lokale Prüfung bestätigte MACs, Vorgängerketten, kanonische Wurzel-/Programmhash-
Bindungen, unterschiedliche Auftragskennungen, exakte AppContainer-Identität ohne Capabilities und
verweigerte private Brokerdateizugriffe. SQLite: Integrität/Fremdschlüssel, beide erfolgreichen
Kopien mit jeweils zwei Dokumenten, Archivhash/Bytezahl und Unicode-Inhalt korrekt. Insgesamt fünf
Aufträge: einer unterbrochen, zwei erfolgreich, zwei fehlgeschlagen; genau zwei legitime Kopien,
keine durch Replay entstandene Kopie.

## Nicht bestandene Versuche und Grenzen

Der erste Unterbrechungsversuch wurde trotz menschlicher Hör-/Tastaturbestätigung technisch
abgelehnt: kein nativer Replay-Aufruf, `manual-recovery-closed-before-replay`, kein Abschlussbeleg.
Ein versuchter Folgeauftrag wurde ebenfalls abgewiesen. Wiederholung desselben verbrauchten Auftrags
mit ausdrücklichem Klick und Gegencheck vor dem Schließen bestand.

Der danach ausdrücklich erzeugte neue Fixture-Auftrag wurde erfolgreich importiert, sein erster
manueller Wiederanlauf aber ebenfalls ohne Replay-Gegenbeleg geschlossen und korrekt abgelehnt.
Diese Sitzung wurde stillgelegt; für den akzeptierten Abschlussfall wurde ein frischer Fixture-
Auftrag angelegt. Kein Abschlussbeleg für den abgelehnten Wiederanlauf, kein Journalreset und keine
Neuvergabe einer verbrauchten Kennung. Vorläufige Bestätigungen allein zählen nicht als Abnahme. Die
Ursache der fehlenden Aufrufe ist ungeklärt; eine Race-Korrektur oder ein Fehler der Sprachausgabe
wird nicht behauptet. Reine Tastatur-Replay-Abnahme bleibt offen.

Die Abbruchprüfung verwendet einen synthetischen Senderabbruch, keinen manuell abgebrochenen
Dateidialog. Nur die initiale Unterbrechungsprüfung enthält eine echte menschliche Dateiauswahl.
Keine Audioaufzeichnung, Tastencodes oder globalen Hooks; Hörbestätigung und technische Diagnose
bleiben getrennt. Bestehende Helfertests einschließlich Ereignisreihenfolge/Replay-Grenzen, 28
aktive Tests insgesamt, Helfer-Clippy, Formatierung und `pnpm check` bestanden. Zwei interaktive
Helfertests wurden diesmal nicht erneut ausgeführt. Kein neuer vollständiger Appbuild erforderlich.

Nächster Schritt: frischer ETW-Netzvergleich dieses neuen Hashpaares mit automatischer Regression
und manuellen Wiederanläufen; der frühere Netznachweis wird nicht übertragen. Weitere Fensterende-
und reine Tastaturprüfungen, Produktablage/Updates/Migration sowie externe Dienstgrenzen bleiben
offen. Das strikte Datenschutzziel bleibt unverändert; Produkt-Q-01/Q-02 sind nicht bestanden.
