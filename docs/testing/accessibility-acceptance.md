# KQ-032: Tastatur- und Accessibility-Prüfmatrix

Stand: 11. November 2026

Diese Matrix trennt automatisiert belegte Browserprüfungen von einer echten Hörprüfung mit
assistiver Technik. Axe und der Browser-Accessibility-Tree finden viele strukturelle Fehler, hören
aber keine Ausgabe. Ein grüner Roboter ist noch keine zuhörende Testperson.

## Automatisiert verifiziert

| Bereich            | Prüfung                                                                                                    | Ergebnis  |
| ------------------ | ---------------------------------------------------------------------------------------------------------- | --------- |
| Projektübersicht   | Projektname und Projektanlage ausschließlich mit `Tab`, Texteingabe und `Enter`; sichtbarer Fokusindikator | Bestanden |
| Bereichsnavigation | Manuskript und Story-Bibel per `Tab` und `Enter`; aktive Ansicht über `aria-current="page"`                | Bestanden |
| Editor             | Editor per Tastatur fokussieren, Text eingeben und markieren                                               | Bestanden |
| Werkzeugleiste     | Fett und Fokusmodus per Tastatur schalten; Zustand über reaktiv aktualisiertes `aria-pressed`              | Bestanden |
| Autosave           | Speicherzustand als gezielter höflicher Live-Status statt Live-Bereich über den gesamten Editor            | Bestanden |
| Story-Bibel        | Suche per Tastatur erreichen; sichtbarer Fokusindikator                                                    | Bestanden |
| Projektübersicht   | Axe: keine als `serious` oder `critical` bewerteten Verstöße                                               | Bestanden |
| Manuskript         | Axe: keine als `serious` oder `critical` bewerteten Verstöße                                               | Bestanden |
| Story-Bibel        | Axe: keine als `serious` oder `critical` bewerteten Verstöße                                               | Bestanden |

Der Lauf erfolgt gegen den lokalen E2E-Command-Adapter in Chromium:

```powershell
pnpm --filter @kraqel/desktop exec playwright test e2e/accessibility.spec.ts
```

KQ-032 korrigierte dabei den unzureichenden Kontrast des Story-Bibel-Ergebniszählers und die nicht
reaktiv aktualisierten `aria-pressed`-Zustände der Editorformatierung.

## Manuell verifiziert

Am 11. November 2026 wurde die isolierte Tauri-Anwendung auf dem Windows-Entwicklungsrechner mit der
Windows-Sprachausgabe und einer hörenden Testperson geprüft. NVDA war nicht installiert.

| Bereich            | Manuelle Prüfung                                                                                                                    | Ergebnis  |
| ------------------ | ----------------------------------------------------------------------------------------------------------------------------------- | --------- |
| Projektübersicht   | Projektname und Projektanlage per Tastatur; verständliche Namen und sichtbarer Fokus                                                | Bestanden |
| Bereichsnavigation | Projektbereiche per Tastatur; verständliche Ansagen und sichtbarer Fokus                                                            | Bestanden |
| Manuskript         | Dokumentbaum, Szenenaktionen, Werkzeugleiste und Editorbezeichnung                                                                  | Bestanden |
| Formatierung       | Fett und Fokusmodus per Tastatur; gedrückte Zustände hörbar                                                                         | Bestanden |
| Autosave           | `Ungespeichert`, `Speichert …` und `Gespeichert` ohne störende Doppelansage                                                         | Bestanden |
| Story-Bibel        | Suche, Artfilter, Ergebnisanzahl, Formularfelder und Eintragsanlage                                                                 | Bestanden |
| Beziehungen        | Ausgehende und eingehende Richtung sowie Szenen-Rückverweis textlich verständlich                                                   | Bestanden |
| Werkzeugansichten  | Plotboard, Timelines, Versionen, Medien, visuelle Profile, Kontinuität, Bildprompts, Export und KI-Entwurf stichprobenartig geprüft | Bestanden |
| Vergrößerung       | Kernansichten bei 200 Prozent Zoom und schmalem Tauri-Fenster per Tastatur erreichbar                                               | Bestanden |

Die Testperson meldete für diesen Umfang keine fehlenden, doppelten oder unverständlichen Ansagen
und keine Tastaturfalle. Die Prüfung verwendete ausschließlich künstliche Testdaten.

## Verbleibende Restfälle

Folgende Fälle wurden nicht ausgeführt und gelten ausdrücklich nicht als bestanden:

1. Ein erzwungener Speicherfehler samt verständlicher Screenreader-Ansage.
2. Die echte Tauri-WebView mit aktivierter Windows-Option „Bewegung reduzieren“.

Die bestandene Kern-Hörprüfung ist damit belastbare 0.1-Evidenz, aber noch keine vollständige
Accessibility- oder Releasezertifizierung. Insbesondere ersetzt sie keine Prüfung mit NVDA, weiteren
Windows-Versionen oder mehreren Testpersonen.
