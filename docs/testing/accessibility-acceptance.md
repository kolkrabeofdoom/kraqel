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

## Manuell offen

Auf dem Entwicklungsrechner ist die Windows-Sprachausgabe vorhanden; NVDA ist nicht installiert. Die
bloße Existenz einer ausführbaren Datei ist keine Abnahme. Folgende Fälle benötigen eine hörende
Testperson und die echte Tauri-WebView:

1. Überschriften, Landmarken, Dokumentbaum, Werkzeugleiste und Editorbezeichnung in sinnvoller
   Reihenfolge vorlesen lassen.
2. `Ungespeichert`, `Speichert …`, `Gespeichert` sowie einen simulierten Speicherfehler auf
   verständliche und nicht störend doppelte Ansagen prüfen.
3. Story-Bibel-Ergebnisanzahl, freie Eigenschaften, eingehende und ausgehende Beziehungen sowie
   Szenen-Rückverweise anhören.
4. Plotboard, Timelines, Versionen, Medien, visuelle Profile, Kontinuität, Bildprompts, Export und
   KI-Entwürfe stichprobenartig mit Tastatur und Screenreader prüfen.
5. Oberfläche bei 200 Prozent Zoom und schmalem Fenster sowie mit aktivierter Betriebssystemoption
   „Bewegung reduzieren“ prüfen.

Erst nach diesem Protokoll darf die Screenreader-Abnahme als bestanden gelten. Die automatisierten
Prüfungen ersetzen weder die Tauri-WebView noch reale Ansagen oder menschliches Verstehen.
