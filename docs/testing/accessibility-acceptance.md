# KQ-032 bis KQ-034: Tastatur- und Accessibility-Prüfmatrix

Stand: 11. November 2026

Diese Matrix trennt automatisiert belegte Browserprüfungen von einer echten Hörprüfung mit
assistiver Technik. Axe und der Browser-Accessibility-Tree finden viele strukturelle Fehler, hören
aber keine Ausgabe. Ein grüner Roboter ist noch keine zuhörende Testperson.

## Automatisiert verifiziert

| Bereich            | Prüfung                                                                                                                                   | Ergebnis  |
| ------------------ | ----------------------------------------------------------------------------------------------------------------------------------------- | --------- |
| Projektübersicht   | Projektname und Projektanlage ausschließlich mit `Tab`, Texteingabe und `Enter`; sichtbarer Fokusindikator                                | Bestanden |
| Bereichsnavigation | Manuskript und Story-Bibel per `Tab` und `Enter`; aktive Ansicht über `aria-current="page"`                                               | Bestanden |
| Editor             | Editor per Tastatur fokussieren, Text eingeben und markieren                                                                              | Bestanden |
| Werkzeugleiste     | Fett und Fokusmodus per Tastatur schalten; Zustand über reaktiv aktualisiertes `aria-pressed`                                             | Bestanden |
| Autosave           | Speicherzustand als gezielter höflicher Live-Status statt Live-Bereich über den gesamten Editor                                           | Bestanden |
| Story-Bibel        | Suche per Tastatur erreichen; sichtbarer Fokusindikator                                                                                   | Bestanden |
| Projektübersicht   | Axe: keine als `serious` oder `critical` bewerteten Verstöße                                                                              | Bestanden |
| Manuskript         | Axe: keine als `serious` oder `critical` bewerteten Verstöße                                                                              | Bestanden |
| Story-Bibel        | Axe: keine als `serious` oder `critical` bewerteten Verstöße                                                                              | Bestanden |
| Autosave-Fehler    | Einmaliger kontrollierter Fehler bleibt ungespeichert, wird als Live-Status gemeldet und nach weiterer Bearbeitung erfolgreich nachgeholt | Bestanden |
| Bewegung           | Emuliertes `prefers-reduced-motion: reduce`; Tastatur-Kernablauf ohne wirksame Animationen oder Übergänge                                 | Bestanden |

Der Lauf erfolgt gegen den lokalen E2E-Command-Adapter in Chromium:

```powershell
pnpm --filter @kraqel/desktop exec playwright test e2e/accessibility.spec.ts
```

KQ-032 korrigierte dabei den unzureichenden Kontrast des Story-Bibel-Ergebniszählers und die nicht
reaktiv aktualisierten `aria-pressed`-Zustände der Editorformatierung. KQ-034 ergänzt den einmaligen
testseitigen Speicherfehler und eine reduzierte Bewegungsumgebung. Der Fehlerhaken liegt nur im
E2E-Command-Mock und ist in keinem Produktionspfad erreichbar.

## Manuell verifiziert

Am 11. November 2026 wurde die isolierte Tauri-Anwendung auf dem Windows-Entwicklungsrechner mit der
Windows-Sprachausgabe und einer hörenden Testperson geprüft. NVDA war nicht installiert.

| Bereich            | Manuelle Prüfung                                                                                                                                 | Ergebnis  |
| ------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------ | --------- |
| Projektübersicht   | Projektname und Projektanlage per Tastatur; verständliche Namen und sichtbarer Fokus                                                             | Bestanden |
| Bereichsnavigation | Projektbereiche per Tastatur; verständliche Ansagen und sichtbarer Fokus                                                                         | Bestanden |
| Manuskript         | Dokumentbaum, Szenenaktionen, Werkzeugleiste und Editorbezeichnung                                                                               | Bestanden |
| Formatierung       | Fett und Fokusmodus per Tastatur; gedrückte Zustände hörbar                                                                                      | Bestanden |
| Autosave           | `Ungespeichert`, `Speichert …` und `Gespeichert` ohne störende Doppelansage                                                                      | Bestanden |
| Story-Bibel        | Suche, Artfilter, Ergebnisanzahl, Formularfelder und Eintragsanlage                                                                              | Bestanden |
| Beziehungen        | Ausgehende und eingehende Richtung sowie Szenen-Rückverweis textlich verständlich                                                                | Bestanden |
| Werkzeugansichten  | Plotboard, Timelines, Versionen, Medien, visuelle Profile, Kontinuität, Bildprompts, Export und KI-Entwurf stichprobenartig geprüft              | Bestanden |
| Vergrößerung       | Kernansichten bei 200 Prozent Zoom und schmalem Tauri-Fenster per Tastatur erreichbar                                                            | Bestanden |
| Autosave-Fehler    | Einmaliger kontrollierter Fehler in der isolierten Accessibility-Testvariante; Fehlermeldung hörbar, Text erhalten, Folgespeicherung erfolgreich | Bestanden |
| Bewegung           | Kernansichten mit deaktivierten Windows-Animationseffekten; Fokus, Tastatur und Sprachausgabe bleiben verständlich                               | Bestanden |

Die Testperson meldete für diesen Umfang keine fehlenden, doppelten oder unverständlichen Ansagen
und keine Tastaturfalle. Die Prüfung verwendete ausschließlich künstliche Testdaten.

## Wiederholbare Windows-Randfallprüfung

Die ausschließlich für manuelle Entwicklungstests bestimmte Tauri-Variante startet mit separater
Kennung, temporärem Datenpfad, dem lokalen E2E-Command-Mock und genau einem vorbereiteten
Speicherfehler:

```powershell
pnpm test:accessibility:manual:windows
```

Sie heißt **KRAQEL Accessibility Test**, erzeugt kein Bundle und darf nicht veröffentlicht werden.
Der erste Autosave schlägt kontrolliert fehl; eine weitere Texteingabe löst den erfolgreichen
Wiederholungsversuch aus. Für die Bewegungsprüfung wird die Windows-Option **Animationseffekte** nur
vorübergehend deaktiviert und anschließend auf ihren ursprünglichen Zustand zurückgestellt.

## Aussagegrenzen

Die bestandene Kern- und Randfallprüfung ist belastbare 0.1-Evidenz, aber keine vollständige
Accessibility- oder Releasezertifizierung. Insbesondere ersetzt sie keine Prüfung mit NVDA, weiteren
Windows-Versionen, mehreren Testpersonen oder einer formalen WCAG-Konformitätsbewertung.
