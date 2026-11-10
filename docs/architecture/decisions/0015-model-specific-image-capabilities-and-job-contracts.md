# ADR 0015: Modellspezifische Bildfähigkeiten und geschlossene Jobverträge

Status: angenommen am 10. November 2026

## Kontext

F09, Modul V.1 und KQ-016 verlangen eine Bild-Capability-Registry und einen Generation-Job-Vertrag,
ohne bereits Bildgenerierung oder Netzpflicht einzuführen. Anbieter*innen und einzelne Modelle
unterstützen unterschiedliche Funktionen, Größen und Referenzmengen. Ein gespeicherter
Prompt-Entwurf ist außerdem noch keine Zustimmung, ihn oder Referenzbilder zu übertragen.

## Entscheidung

`kraqel-ai-service` enthält eine offline validierbare Registry. Fähigkeiten werden am konkreten
Modell ausgewiesen, nicht pauschal am Provider. Jeder Modelleintrag nennt Prompt- und
Referenzgrenzen, Bildgrößen, maximale Ergebniszahl und Seed-Unterstützung. Die Registry akzeptiert
auch eine leere Installation sowie verständlich begründete inaktive Provider, lehnt aber doppelte
Kennungen, widersprüchliche Verfügbarkeit und ungültige Grenzen ab.

Der versionierte Vertrag v1 bindet einen Auftrag an Job, Projekt, Provider, Modell, Fähigkeit,
Prompt-Entwurf, den exakt freigegebenen Prompttext, Referenzasset-UUIDs und Einstellungen. Vor einem
möglichen Provideraufruf prüft Rust Registry, Modellfähigkeit, Grenzen und die explizite Freigabe.
Die Jobzustände bilden ausschließlich `queued -> running -> succeeded|failed|cancelled` sowie den
direkten Abbruch oder Fehler aus `queued` ab. Endzustände bleiben endgültig; Herkunft darf nur zum
erfolgreichen Job und ein bereinigter Fehler nur zum fehlgeschlagenen Job gehören.

KQ-016 liefert keine produktive Implementierung des Providertraits. Ein lokaler Mock existiert nur
im Rust-Testmodul und gibt technische Herkunft statt einer Bilddatei zurück. Es entstehen weder
Netzwerkzugriffe noch Tauri-Commands, SQLite-Tabellen, Snapshots oder Assets.

## Folgen

- KRAQEL kann verfügbare und inaktive Bildfunktionen darstellen, ohne Provider zu kontaktieren.
- Referenzbilder setzen zusätzlich zur Primärfähigkeit `image.reference` am gewählten Modell voraus.
- Statische Fehlercodes und Meldungen verhindern, dass fremde Antworten oder sensible Eingaben in UI
  oder persistierte Jobdaten durchsickern.
- TypeScript und Rust verwenden dieselben Wire-Namen; inkompatible Änderungen benötigen eine neue
  Vertragsversion.
- KQ-017 muss Freigabebelege, echte Abbruchsemantik, sichere Ergebnisvalidierung, Persistenz,
  Lizenzinformationen und explizite Assetübernahme gesondert implementieren und prüfen.
