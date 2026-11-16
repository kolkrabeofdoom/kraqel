# Dateiauswahl aus laufender isolierter App · KQ-069 / #134

16. November 2026, Windows, synthetische Fixture, native Bedienung durch die testende Person.
    Forschungsmodus `ui-live`, kein produktiver Launcher.
    [Kanalvertrag und Grenzen](../data-format/native-live-file-channel-v2.md).

## Finale funktionale Serie

- App-SHA-256: `1ed58dd288177bcdc8ca77ad63ce6c527541848f3aa8689c0615e49bfbc03ae6`
- Controller-SHA-256: `c866bae71a69b23361d5c29c1855c2938c9630513bce3a51cbb35a8cee24cbb8`
- Zwei frische Sitzungen: Importknopf im laufenden Forschungsfenster öffnet den nativen Dialog.
  Erwartete AppContainer-Identität mit null Capabilities vor Prozessfortsetzung geprüft.
  Vererbungsflags beider Pipes vor WebView-Start entfernt und zurückgelesen.
- Erfolg: synthetische `input.kraqel` auswählen, Import vorbereiten und abschließen, Fenster
  schließen. App und Controller enden mit Exitcode 0. Bericht `passed: true`, `phase: done`, zwei
  Dokumente, null Assets. Nach Prozessende unabhängig per SQLite-Nur-Lesezugriff geprüft: genau eine
  neue Projektkopie, exakter Unicode-Text, Integrität und Fremdschlüssel fehlerfrei, Projekt-ID
  entspricht dem Ergebnis, kein Stagingrest.
- Dialogabbruch: Importknopf, native Auswahl abbrechen, Fenster schließen. App und Controller enden
  mit Exitcode 0, `passed: true`, `CANCELLED`, keine Receipt, Projektwurzel vollständig leer.
- Beide finalen Läufe: null aktive Jobprozesse, AppContainer-Profil entfernt, keine zusätzliche
  Paket-ACL auf Hostordner. Lokale Wurzeln, PIDs und Rohdiagnosen bleiben außerhalb von Git.

## Gefundene und korrigierte Fehler

1. Der erste Abbruch räumte auf, aber fehlende Erfolgsfelder führten zu Controller-Exitcode 1. Der
   neue Modus validiert nun den begrenzten typisierten UI-Abschluss und bilanziert Cleanup.
2. Erste Dateiauswahl scheiterte am lexikalischen Vergleich von Windows-8.3-TEMP-Pfad und langem
   Shell-Pfad. Controller beendete die Sitzung, App meldete `READ_FAILED`, keine Projektkopie.
   Dieser Lauf ist kein bestandener Cleanup-Nachweis. Aufgelöste Verzeichnisgleichheit und
   Rekonstruktion unter dem Controllerordner korrigieren den Aliasfehler; Datei-Handleprüfungen
   bleiben erhalten. Regression weist Nachbarverzeichnis und ADS ab. Auswahlthread-Fehler werden
   jetzt nach Appende bilanziert. Dieser separate Fehlerpfad ist noch nicht erneut nativ geprüft.

## Prüfungen und offene Abnahme

`pnpm check`, 20 Helfer-/Transporttests, Feature-Clippy und Forschungs-Tauri-Build bestanden.
Decoderregression: jede Verkürzung eines gültigen Pakets, falsche UUID, Ablauf, Zusatzbytes,
neutrale Abbruchantwort und übergroße Längenfelder. Kein neuer ETW-Trace dieses Builds; historische
Messungen aus PR #140 sind getrennte Nachweise.

Offen: Replay-Bedienprüfung, Screenreader/Fokus und Fehleransagen, Appende bei offenem Dialog oder
Transfer, zeitliche Grenzfälle, unabhängiger ETW-Lauf einschließlich externer Brokergrenzen sowie
dauerhafter Auftrags-/Crash-Lebenszyklus. Weitere Dateien/Formate, Git-/Credential-/Provider- und
Produkt-/Installerintegration fehlen. Q-01/Q-02 für das Produkt bleiben nicht bestanden; #134 bleibt
offen. Striktes Datenschutzziel unverändert.

Reproduktion: Forschungsbuild gemäß `native-tauri-file-channel.md`, Controller gelockt bauen,
`ui-live <neue TEMP-Wurzel kraqel-isolation-ui-…> <Forschungs-EXE>`. Erst im laufenden Fenster
importieren. Nur synthetische `input.kraqel` auswählen. 60 Sekunden ab Auswahl; neue Sitzung pro
Versuch. Anschließend normalen Debug-Build wiederherstellen.
