# BP4b · Unterbrochener Schreibframe und Empfangsfrist

## Versuchsaufbau

Im interaktiven Offline-Gast mit erzwungener UMCI erhält jeder Fall eine eigene SQLite-Testablage. Über das authentisierte Pipe-Paar liest die Oberfläche zuerst Szene A auf Revision 0. Für Anfrage 2 bildet sie einen gültigen Schreibframe mit erwarteter Revision 0 und geändertem synthetischem Text. Sie überträgt den 50-Byte-Header und das erste Nutzdatenbyte. Der Rest einschließlich Authentisierungstag bleibt aus.

| Fall | Verhalten der Oberfläche | Erwartetes Kanalende | Exitcodes Dienst/UI mit frischem Lesepaar |
| --- | --- | --- | --- |
| Teilübertragung | Schließt ihr Schreibende nach 51 Bytes und liest am Antwortende weiter. | EOF oder gebrochene Pipe beim Datendienst. | 89/90, danach 91/92 |
| Timeout | Hält ihr Schreibende offen und wartet am Antwortende. | Die monotone Empfangsfrist von 30 Sekunden läuft ab. | 93/94, danach 95/96 |

Der Empfänger der persistenten Probe startet die Frist beim ersten gelesenen Byte eines Frames. Weitere Teilstücke verlängern sie nicht. Auf der synchronen Pipe mit genau einem Leser liest er danach höchstens die bereits verfügbaren Bytes. Der äußere Prozesswächter lässt dem Timeout-Fall 45 Sekunden, damit der reguläre Timeout und das Ende beider Prozesse beobachtbar sind.

Die Prüfung verlangt genau 51 empfangene Bytes des zweiten Frames und die fallspezifische Endursache. Nur die zuvor abgeschlossene Leseanfrage erreicht die Szenenablage. Die Oberfläche akzeptiert auf den Schreibversuch ausschließlich ein Kanalende vor dem ersten Antwortbyte. Anschließend prüft ein frisches Paar mit eigenem Leseprofil die ursprünglichen Absätze „Rabe“ und „42“ sowie Revision 0.

## Reproduktion

Im vorbereiteten Offline-Gast mit frischer Differenzplatte den [Gasttreiber](../../scripts/runtime-isolation/native-window/pipe-guest.ps1) mit `-Trial channel-interruptions` ausführen. Der Starter prüft pro Paar Endpunkte, Prozessrollen, Fremdzugriff und leere Jobs. Für die unabhängige Gegenprüfung die abgeschaltete Differenzplatte schreibgeschützt einbinden. Beide Ablagen müssen jeweils genau eine intakte SQLite-Datei mit Szene A auf Revision 0 enthalten. Im Suchindex muss „42“ auffindbar sein und „43“ fehlen.

| Programm | SHA-256 des geprüften Builds |
| --- | --- |
| native-leaf.exe | F3D44592F9E8C33B34BF6AB3B86DDD7F46A249225D9DBA4DA8387C117603882C |
| pipe-check.exe | 1B76BA986BA44B309FA8F691155BB956C142CE0FAAEFD9FF21422AEF49BD2C87 |
| pipe-other.exe | 420C21B4E49FBF9CF02EB51CB0FBB01E9CEFD6C53DC0CA01C60FAFBE7492FF07 |
| pipe-service.exe | 6CED02AE25BFB0C4942064301A52FC300F4FD9B57C10605A2CDE01D153EE6494 |
