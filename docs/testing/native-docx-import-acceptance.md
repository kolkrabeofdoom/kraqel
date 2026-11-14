# KQ-055: Native Windows-Prüfung des DOCX-Imports

Stand: 14. November 2026. Arbeitsbranch: `feat/kq-055-docx-import`, Issue #107, ADR 0024.

Geprüft wurde der neue native Rust-Debug-Build mit der normalen Vite-Oberfläche, ohne
E2E-Command-Adapter. Windows x64, Rust/Cargo 1.100.0, Visual Studio Build Tools 2022 17.14.45 und
WebView2 157.0.4341.60. Bedienung und Sichtprüfung erfolgten mit Computer Use über den echten
Windows-Dateidialog und den Accessibility-Tree. Das eigene Projekt `KQ-055 DOCX-Prüfung` enthält
ausschließlich künstliche Daten und bleibt lokal erhalten.

| Fall                       | Beobachtung                                                                                                                                                                | Ergebnis  |
| -------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------- |
| Umfang vor Auswahl         | DOCX-Aktion zeigt Hinweis zu unterstützten Textformatierungen, Ablehnungen, Layoutverlust und 8-MiB-Grenze.                                                                | Bestanden |
| Nativer Dialog             | Dateifilter zeigt DOCX-Dokument.                                                                                                                                           | Bestanden |
| Gültige Datei              | Mit python-docx erzeugte `Gueltig.docx` wird als zusätzliche Szene importiert; Überschrift, Unicode, Fett/Kursiv, harter Umbruch und XML-Sonderzeichen erscheinen korrekt. | Bestanden |
| Abbruch                    | Escape beendet den Dateidialog; danach ist die DOCX-Aktion wieder aktiv. Keine zusätzliche Szene, keine neue Fehlermeldung, bisheriger Text erhalten.                      | Bestanden |
| Nicht unterstützte Tabelle | `Tabelle.docx` enthält zusätzlich eine Tabelle: bereinigte Ablehnung, keine zusätzliche Szene, bisheriger Text unverändert.                                                | Bestanden |
| Beschädigte Datei          | `Kaputt.docx` enthält kein ZIP: bereinigte Ablehnung, keine zusätzliche Szene, bisheriger Text unverändert.                                                                | Bestanden |

Zusätzlich bestanden: Anwendung schließen, finalen Debug-Build starten und Prüfprojekt erneut
öffnen. Die importierte Szene, der Text und die Formatierungen bleiben erhalten.

Die Fehlermeldung nennt weder Pfad noch Bibliotheks- oder XML-Diagnostik. Eine neue
Screenreader-Hörprüfung, manuelles Speichern mit Word/LibreOffice, Releaseinstaller oder Clean-VM
sind nicht Bestandteil dieser Prüfung. Weitere Archiv-/XML-Randfälle werden getrennt in Rust
geprüft. Diese Abnahme erklärt weder den vollständigen F08-Umfang noch einen strukturerhaltenden
Manuskript-Roundtrip für implementiert.

## Automatische Ergänzungen

Auf demselben Arbeitsstand bestanden `pnpm check` (Formatierung, ESLint, Clippy, TypeScript,
Pakettests und Rust-Workspace-Tests), der Desktop-Frontend-Build und der native Debug-Build. Die
gezielten DOCX-Rust-Tests umfassen 13 Fälle. Der gemeinsame Chromium-Lauf von `docx-import.spec.ts`,
`m1-editor.spec.ts` und `accessibility.spec.ts` bestand mit acht Tests, einem Worker und 60 Sekunden
Testbudget. Ein erster paralleler Lauf während des kalten Builds überschritt Zeitlimits; der
abschließende Lauf war vollständig grün.
