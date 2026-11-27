# KQ-069: Assistive Wiederanlaufprüfung

Stand: 16. November 2026. Windows-Forschungsfeature, ausschließlich synthetische Fixtures.
[Startticket v2 und manueller Ablauf](../data-format/native-ui-restart-v2.md),
[bereinigte Aggregate](native-assistive-restart.json). Keine Produktfreigabe, striktes
Datenschutzziel unverändert.

## Native und menschliche Nachweise

Die prüfende Person hat die Windows-Sprachausgabe aktiviert und folgende Beobachtungen bestätigt:

| Wiederanlauf                               | Gehörte Ausgabe/Fokus                                                                                                                                         | Nativer Gegenbeleg                                                                                             |
| ------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------- |
| Unterbrochener Import                      | Hinweis auf unterbrochenen, weiterhin verbrauchten Auftrag; Fokus auf Replay. Nach explizitem Replay-Klick Verbrauchshinweis angesagt.                        | `INTERRUPTED`, Replay abgewiesen, null Projektkopien oder Zwischenreste                                        |
| Zuvor abgeschlossener Import               | Früherer Auftrag mit zwei Dokumenten und ohne neue Kopie angesagt; Fokus auf Replay, anschließender Verbrauchshinweis angesagt.                               | `succeeded`, Replay abgewiesen, genau eine Kopie und keine Zwischenreste                                       |
| Zusätzliche Tastaturprüfung nach Abschluss | Wechsel ins Forschungsfenster, Tab-Navigation, Aktivierung per Leertaste, angesagter Verbrauchshinweis und Schließen mit Alt+F4 laut Rückmeldung erfolgreich. | Navigation, native Aktion, Replay-Abweisung und reguläres Ende belegt; zusätzlich Pointer-Ereignis im Fokuslog |

Die Tastaturnutzung ist durch die prüfende Person bestätigt. Das zusätzliche Pointer-Ereignis
erlaubt keine unabhängige Zuordnung der Eingabemethode allein aus dem Log; eine bestimmte Ursache
des Pointer-Ereignisses wird nicht behauptet. Keine Audioaufzeichnung, Tastencodes oder globalen
Eingabehooks. Gehörte Ausgabe ist menschlicher Nachweis; DOM-Fokus und native Aufrufe sind getrennte
technische Nachweise.

Der erste Unterbrechungsversuch ist **nicht** als vollständiger Lauf akzeptiert: Es gab weder einen
protokollierten nativen Replay-Aufruf noch den Replay-Diagnosebeleg. Die anfängliche Vermutung eines
verlorenen Ereignisses beim schnellen Schließen war damit nicht belegt; dafür wurde keine
vermeintliche Race-Korrektur vorgenommen. Die Wiederholung mit explizitem Button-Klick bestätigte
Ansage und nativen Replay. Die erste Initialansage/Fokusbeobachtung stammt vom selben unveränderten
App-Build; fehlendes Replay wird nicht nachträglich als bestanden gewertet. Die temporäre
Profilzuordnung dieses unvollständigen Versuchs ist nicht mehr vorhanden.

Alle drei akzeptierten manuellen Läufe: Controllerexit 0, jeweils ein harter vorbereitender
Appabbruch mit Code 77 und ein reguläres Appende mit Code 0. Job-Prozesse null, Profile entfernt.
Unabhängige Nur-Lese-Prüfung: Journalintegrität `ok`, erwarteter terminaler Zustand und Kopienzahl,
keine Zwischenreste; bei veröffentlichten Kopien SQLite-Integrität, Fremdschlüssel und exakter
synthetischer Unicode-Inhalt korrekt.

## Buildbindung und Qualität

App-SHA-256: `4db1e731e6aee50fe01e9e368d1ded61ac61f3f9dc3e2238d509cee600098e1b`. Controller-SHA-256:
`64ddbcb277c38db75b6e324687d0122a62dcc4b4690e0fefe9249a027436007e`. Die Hörläufe und die
anschließende automatische 28-Generationen-Regression verwenden dasselbe Paar. Automatik mit v2
bestanden: alle sechs Crashpunkte, zweifache Wiederherstellung, Altimportabwehr und vier abgewiesene
Starts. Keine Runtimeänderung nach den nativen Prüfungen.

`pnpm check`, 35 Browser-E2E, 24 aktive Helfer-/Transporttests, Forschungs-/Helfer-Clippy sowie
Forschungsbuild bestanden. Die zwei Forschungs-Origin-/DTO-Tests und der wiederhergestellte normale
Debug-Build gehören zum Abschluss. Keine neuen Abhängigkeiten, Netzwerkfähigkeiten, Rendererbefehle
oder Hostordnerfreigaben. Rohdaten, UUIDs, Pfade und Programme bleiben lokal.

Das neue Paar ist gesichert unter `%TEMP%\kraqel-kq069-assistive-restart-build-20261116-01`. Der
[gültige ETW-Nachweis des früheren v1-Paars](native-ui-restart-etw.md) wird ausdrücklich nicht auf
diese geänderten v2-Binaries übertragen. Ein späterer
[eigener frischer v2-ETW-Nachweis](native-assistive-restart-etw.md) ist inzwischen gültig bestanden,
einschließlich beider manueller Pfade. Dieser ursprüngliche Hörnachweis bleibt separat. Normale
Produktstarts bleiben unisoliert; #134 und Produkt-Q-01/Q-02 bleiben nicht bestanden.

## Verbleibende Abnahme

Andere Hilfsmittel/Versionen, vollständige native Tastatur- und Accessibility-Abnahme aller Pfade,
Live-Dateidialog/Transfer mit gebundenem Wiederanlauf, Controller-Crash/dauerhafter Produktbroker,
externe COM-/Shell-/DNS-/Brokergrenzen und Produkt-/Packaging-Integration. Die bestätigten
Wiederanlaufansagen ersetzen keine assistive Vollabnahme.
