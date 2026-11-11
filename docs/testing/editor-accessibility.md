# Testplan: Editor und Tastaturbedienung

Stand: 11. November 2026

Dieser Plan ergänzt die automatisierten Vertrags-, Repository-, Autosave- und Accessibility-Tests.
Der automatisierte KQ-032-Stand und die manuelle KQ-033-Prüfmatrix stehen in
[`accessibility-acceptance.md`](accessibility-acceptance.md).

## Tastatur

1. Anwendung ohne Maus starten und mit `Tab` Projektname, Projektanlage und vorhandene Projekte
   erreichen; der Fokusindikator bleibt stets sichtbar.
2. Projekt mit `Enter` öffnen. Kapitel- und Szenenaktionen müssen über `Tab` sowie `Enter` oder
   `Leertaste` bedienbar sein.
3. Eine Szene wählen und Text ausschließlich per Tastatur eingeben. `Strg+Z` und `Strg+Umschalt+Z`
   müssen nur den Verlauf dieser Szene verändern.
4. Fett, Kursiv, Rückgängig, Wiederholen und Fokusmodus über die Werkzeugleiste bedienen. Aktive
   Umschalter müssen ihren Zustand mit `aria-pressed` ausweisen.
5. Zwischen zwei Szenen wechseln und prüfen, dass deren Undo-Verläufe getrennt bleiben.
6. Über die Bereichsnavigation die Story-Bibel öffnen. Suche, Artfilter, Eintragsliste, Anlegen,
   Bearbeiten sowie Löschen müssen vollständig über Tastatur erreichbar sein.
7. Eine gerichtete Beziehung anlegen und entfernen; Quelle, Richtung und Ziel müssen im Text
   verständlich bleiben und dürfen nicht nur durch Pfeilposition oder Farbe vermittelt werden.
8. Den gewählten Story-Eintrag mit der aktiven Szene verknüpfen und den Rückverweis aktivieren. Der
   Fokus muss in den Manuskriptbereich zurückkehren können.

## Wahrnehmbarkeit und Status

1. Bei einer Änderung die Folge `Ungespeichert` -> `Speichert …` -> `Gespeichert` prüfen.
2. Einen simulierten Speicherfehler auslösen; `Speichern fehlgeschlagen` muss sichtbar bleiben und
   der Inhalt darf nicht als gespeichert erscheinen.
3. Mit Screenreader Projektübersicht, Dokumentbaum, Werkzeugleiste, Editorbezeichnung und
   Speicherstatus prüfen. Kein Bedienelement darf ausschließlich durch Farbe erklärt werden.
4. Bei 200 Prozent Zoom und 760 Pixel Fensterbreite dürfen Editor und Dokumentbaum keine Funktionen
   verdecken.
5. Betriebssystemoption „Bewegung reduzieren“ aktivieren; die Anwendung darf keine notwendige
   Information durch Animation allein vermitteln.
6. Mit Screenreader die Beschriftungen freier Eigenschaften, der Beziehungsfelder, Ergebnisanzahl
   und Rückverweise prüfen. Eingehende und ausgehende Beziehungen müssen unterscheidbar vorgelesen
   werden.

## Automatisierungsstand

- Fokusreihenfolge, Tastaturaktivierung, sichtbarer Fokus sowie zentrale Namen und Zustände werden
  in `e2e/accessibility.spec.ts` geprüft.
- Axe prüft Projektübersicht, Manuskript und Story-Bibel auf schwerwiegende oder kritische
  Accessibility-Verstöße.
- Der fachliche Story-Bibel-CRUD-, Relations- und Rückverweisablauf besitzt einen eigenen
  browsergestützten E2E-Test.
- Die KQ-033-Kernprüfung mit Windows-Sprachausgabe in der echten Tauri-WebView ist bestanden; ein
  erzwungener Speicherfehler und „Bewegung reduzieren“ bleiben offen.
- Szenenwechsel während eines laufenden Autosaves
- Wiederherstellung eines gespeicherten Szenenstands bis in die sichtbare Oberfläche
