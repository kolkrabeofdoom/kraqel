# KQ-069: Assistive Fokus- und Fehlerprüfung

16. November 2026, Windows, ausschließlich synthetische Daten im Forschungsfeature
    `isolation-ui-test`. Keine Produktfreigabe; #134 und Produkt-Q-01/Q-02 bleiben offen.

## Änderung

Die Forschungsseite setzt den Fokus nach gestarteter Aktion auf eine verfügbare Folgeaktion:
laufende Prüfung → „Abbrechen“, vorbereitete Kopie → „Import abschließen“, Erfolg/Fehler/Abbruch →
„Erneuten Aufruf prüfen“. Bewusste Tastatur- oder Schaltflächennavigation wird bei folgenden
Statuswechseln erhalten, solange das gewählte Element bedienbar bleibt. Die 300-ms-Abfrage setzt
weder identische Ansagen erneut noch den Fokus bei unveränderter Phase.

Ein ausschließlich im Forschungsfeature verfügbarer Diagnosebefehl `isolation_test_focus_probe`
ergänzt die vier bisherigen parameterlosen Befehle. Rust prüft Fenster/Origin wie bei diesen. Er
akzeptiert ausschließlich feste Ereignis-, Phasen- und Zielelement-Enums sowie drei boolesche Werte.
Unbekannte Felder und Freitext werden abgelehnt. Höchstens 64 Einträge beziehungsweise 16 KiB werden
unter dem festen Dateinamen `ui-focus.json` im bereits freigegebenen Forschungsdatenverzeichnis
gespeichert. Keine Tastencodes, Texte, Projektinhalte, Hostpfade oder zusätzlichen Dateifreigaben.
Rendererangaben sind unzuverlässige Diagnoseinformationen, keine Autorisierung oder
Sicherheitsattestierung. Keine Netzwerkfähigkeit, keine Telemetrie und keine Änderung des normalen
App-Starts.

## Tatsächlich beobachtet

Frühere native Versuche ließen den Fokus nach vorbereiteter Kopie laut Rückmeldung auf „Abbrechen“;
diese Versuche gelten ausdrücklich nicht als bestanden. Browserprüfungen allein reichten zur Klärung
nicht aus. Nach expliziter Unterscheidung bewusster Navigation wurde deshalb ein begrenzter lokaler
Diagnosepfad ergänzt. Die Ursache der früheren nativen Abweichung ist nicht abschließend
nachgewiesen; kein behaupteter universeller Fokusfix.

Im anschließenden nativen Diagnoselauf bestätigen Rückmeldung und DOM-Protokoll den Wechsel von
„Abbrechen“ zu „Import abschließen“. Nach Enter folgen Erfolg und Fokus auf „Erneuten Aufruf
prüfen“. Die prüfende Person bestätigt ausdrücklich beide Ansagen durch die Sprachausgabe und den
passenden Fokus nach Replay; anschließend Fenster geschlossen. Unabhängig gelesen: `done`, zwei
Dokumente laut Rust-Beleg, `replayRejected: true`, genau eine Datenbank/Projektkopie, SQLite
`integrity_check = ok`, keine Fremdschlüsselverstöße oder Stagingverzeichnisse. Controllerexit 0,
App-Exit 0, Auswahlthread beendet, null Jobprozesse, Profil entfernt.

Ein zweiter Lauf nutzt im eigenen synthetischen Hostverzeichnis eine absichtlich ungültige
59-Byte-Datei `input.kraqel`. Die prüfende Person bestätigt Fehleransage, anschließende
Replay-Ansage und Fokus auf „Erneuten Aufruf prüfen“. Rust-Beleg: `failed`, `IMPORT_FAILED`, kein
Receipt, `replayRejected: true`, null Projektkopien/Datenbanken/Stagingverzeichnisse. App-Exit 0,
Thread beendet, null Jobprozesse, Profil entfernt. Controllerexit 1 und `passed: false` sind hier
erwartete Folgen des absichtlichen Importfehlers und kein bestandener erfolgreicher Import.

## Automatisierte Prüfung und Buildbindung

- Sieben gezielte Browser-E2E bestanden: verzögerte Auswahl, bewusste Navigation, Tastaturfokus,
  sichtbarer Fokus, Fehler/Abbruch, Erfolg und beständiger Replay-Hinweis. Keine schwerwiegenden
  oder kritischen Axe-Befunde in den geprüften Anfangs-/Endzuständen.
- Gesamte Browser-E2E: 34 bestanden. Browseradapter ersetzen keine nativen Hörprüfungen.
- Rust-DTO-Test weist Freitext und zusätzliche Pfadfelder zurück; Feature-Clippy bestanden.
- `pnpm check` bestanden, ebenso beide Forschungs-Origin-/DTO-Tests. Forschungsbuild geprüft und
  separat lokal gesichert; normaler Debug-Build anschließend wiederhergestellt.
- App-SHA-256 beider nativer Diagnoseläufe:
  `8ee48ec6acdb93be5b70b35bf9ac80cf43f8bdb37ad5fa82bdced690c34f0b6d`.
- Unveränderter Controller-SHA-256:
  `3c5a9e27c0e516d84a10576e109da2f773e1681bfdca9c47fec69bcafa0a86be`.

Rohdiagnosen und synthetische Arbeitsverzeichnisse bleiben lokal. Keine neuen ETW-Messungen für
diesen geänderten App-Build. Der [vorherige ETW-Nachweis](native-live-window-close-etw.json) bleibt
an dessen vorherigen App-Hash gebunden und wird ausdrücklich nicht übertragen.

## Grenzen und Fortsetzung

Die beiden konkreten Erfolg-/Fehlerpfade sind mit Sprachausgabe bestätigt. Assistive Vollabnahme,
Fehleranzeige bei jedem Transportfehler, Ablauf-/Abbruchansagen, weitere Hilfsmittel und wiederholte
native Fokusprüfungen bleiben offen. Insbesondere ist der Abbruchpfad hier nur im Browser
automatisiert geprüft; frühere funktionale Abbruchnachweise ersetzen dessen Hörprüfung nicht.
Nächster Entwicklungsscope: dauerhafter Auftragslebenszyklus und Crash-Reconciliation vor
Produktintegration; externe COM-/Shell-/DNS-/Brokergrenzen und frischer Netzbeleg des späteren
finalen Builds bleiben notwendig. Das strikte Datenschutzziel wird nicht gelockert.
