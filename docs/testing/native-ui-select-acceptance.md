# Native Vorabauswahl und Dialogabbruch · KQ-069 / #134

Am 16. November 2026 auf dem zurückgewechselten Windows-Rechner aus `bf8891b` neu gebaut. Nur
synthetische `input.kraqel`, Forschungsfeature `isolation-ui-test`, Bedienung durch die testende
Person. Kein Produktfreigabenachweis.

## Binärstand

- Forschungs-App SHA-256: `38874434ad365f908e650f4e6b55fe3d95cd7169aef33b1906a5b972944fbad6`
- Controller SHA-256: `bd886367022f8d343d293314b31f27865b1c30043144a66ce70b5b046baca790`
- Installation aus unverändertem Lockfile, Forschungs-Tauri-Build und separat gelockter
  Controllerbuild erfolgreich. Gesicherte Forschungs-EXE liegt ausschließlich lokal in TEMP.

## Ergebnisse

| Fall                                                                        | Nachweis                                                                                                                                                                                                                                                                                                                 |
| --------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Nativen Öffnendialog abbrechen                                              | Zweiter Lauf direkt über erfasste Standardausgabe: Exitcode 0, `cancelled: true`, `passed: true`, `productionAcceptance: false`. Keine Isolations-, Projekt- oder Stagingwurzel, keine verbleibenden App-/Controllerprozesse. Nur die beiden vorher erzeugten synthetischen Hostdateien bleiben zur Inspektion erhalten. |
| `input.kraqel` nativ auswählen, importieren, abschließen, Fenster schließen | Controller endet mit Exitcode 0, `phase: done`, zwei Dokumenten und null Assets. Vor Start erwartete AppContainer-Identität mit null Capabilities. Zwei vererbte Handles, einseitiger Kanal, keine Paket-ACL auf Hostordner.                                                                                             |
| Unabhängige Prüfung nach Prozessende                                        | Bestehendes `verify_tauri_ui.verify_run` besteht: exakt eine neue UUID-Projektkopie, zwei Dokumente, exakter synthetischer Unicode-Text, SQLite-Integrität und Fremdschlüssel fehlerfrei, keine Stagingreste. Null aktive Jobprozesse, Profil entfernt. Gesicherter und gestarteter App-SHA stimmen überein.             |

Der erste Dialogabbruch wurde durch Prozessende und Artefaktinspektion bestätigt, seine
Standardausgabe war nicht aufgezeichnet. Deshalb wurde er mit erfasster Ausgabe wiederholt. Die
60-Sekunden-Freigabe blieb unverändert. Die Oberfläche meldete beim erfolgreichen Lauf „Projektkopie
angelegt: 2 Dokumente.“; der unabhängige Verifier bestätigt die Persistenz.

## Aussagegrenzen und Fortsetzung

Kein neuer ETW-Trace dieses Rechners; keine Aussage über die Netzwerkbilanz dieser Läufe.
Historische Messungen aus PR #140 bleiben getrennte Nachweise. Keine neue assistive Hörabnahme,
keine Auswahl aus einer bereits laufenden App und keine echten Manuskripte geprüft. Q-01/Q-02 für
das Produkt bleiben nicht bestanden; #134 bleibt offen und das strikte Datenschutzziel unverändert.
Nächster Scope ist die Auswahl aus einer laufenden App mit sicherem Auftragskanal; assistive Abnahme
und dauerhafter Auftrags-/Replay-/Crash-Lebenszyklus folgen. Lokale Pfade, PIDs, Profile und
Rohdiagnosen werden nicht versioniert.
