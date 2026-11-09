# Testplan: Plotboard und Timelines

Stand: 9. November 2026

Dieser manuelle Plan ergänzt Repository-, IPC- und Vertragstests; eine Playwright-Abnahme fehlt.

1. Plotboard ausschließlich per Tastatur öffnen. Jede Szenenkarte, Status-, Handlungsstrang- und
   POV-Auswahl muss erreichbar und beschriftet sein.
2. Karten mit „Nach links“/„Nach rechts“ neu ordnen und nach erneutem Öffnen dieselbe Reihenfolge
   prüfen. Anschließend denselben Ablauf per Drag-and-drop prüfen.
3. Relative und absolute Zeitebene anlegen. Nur die absolute Zeitebene darf einen Zeitpunkt
   verlangen; der Fehlerfall muss ohne Verlust anderer Eingaben verständlich bleiben.
4. Ereignisse mit „Nach oben“/„Nach unten“ ordnen. Reihenfolge nach Neustart prüfen.
5. Aktive Szene und Story-Bibel-Eintrag verknüpfen. Szenenlink per Tastatur zurück ins Manuskript
   verfolgen; Links müssen textlich verständlich und nicht nur farblich erkennbar sein.
6. Bei 200 Prozent Zoom und schmalem Fenster dürfen Karten, Ereignisse und Bedienaktionen nicht
   unzugänglich werden. Mit Screenreader Bereichsnavigation, Reihenfolge und Linkarten prüfen.
