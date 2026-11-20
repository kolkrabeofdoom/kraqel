# Dateiauswahl aus laufender isolierter App · KQ-069 / #134

## Fortschreibung: Replay, Fehler, Fensterende und ETW

16. November 2026: Replay nach Erfolg und nach beschädigtem Archiv nativ geprüft. Dabei gefunden:
    Die regelmäßige Statusaktualisierung überschrieb die Replay-Ansage mit dem alten Importfehler.
    Die Oberfläche hält nun den Fehler der letzten Aktion stabil. Ein neuer Browsertest prüft das
    über zwei Aktualisierungsintervalle hinweg. Rust schreibt einen datenarmen
    Replay-Ablehnungsmarker.

Am korrigierten App-Build `51d4e39f0ac0d2d33129c2b191d70a8ff7aae7c4322c21e03b42d0b6c66ce10e` mit
Controller `c866bae71a69b23361d5c29c1855c2938c9630513bce3a51cbb35a8cee24cbb8` sind anschließend drei
vollständige Sitzungen innerhalb einer ETW-Aufzeichnung geprüft:

- Import und Replay: genau eine neue Projektkopie, zwei Dokumente, exakter Unicode-Text,
  SQLite-Integrität und Fremdschlüssel nach Prozessende unabhängig bestätigt. Replaymarker
  vorhanden.
- Beschädigtes Archiv und Replay: `IMPORT_FAILED`, keine Receipt, Projektwurzel einschließlich
  Staging leer, Replaymarker vorhanden. Controller `passed: false` und Exitcode 1 sind hier das
  erwartete Importfehlerergebnis; die App endet mit 0. Die unabhängige Fehlerprüfung besteht.
- Fenster am Publish-Haltepunkt über X schließen: `CANCELLED`, keine Kopie oder Stagingreste.

Alle drei: Root-Token vor Fortsetzung geprüft, erwartete AppContainer-Identität, null Capabilities,
beide Handle-Vererbungsflags vor WebView-Start entfernt/geprüft; null Jobprozesse, Profile entfernt.
Die testende Person bedient die nativen Fenster; die Auswertung erfolgt unabhängig nach Prozessende.

Gültiges ETW vom **16. November 2026, 06:27:23.8665633–06:30:15.7033651 UTC**: drei Controller-,
drei App-, drei Console-Host- und 18 WebView2-Prozesslebenszeiten vollständig zugeordnet. Vier/vier
positive TCP-/UDP-Kontrollen über IPv4/IPv6, null Ereignis-/Pufferverluste, keine Zuordnungslücken
oder unbekannten Ziele. **0 Ereignisse außerhalb Loopback**; 22 Loopback-Disconnects (Event 13/29
jeweils elf) getrennt ausgewiesen. [Bereinigte Messdaten](native-live-file-selection-etw.json).
ETL/XML, lokale Pfade/PIDs und fremde Prozessdaten bleiben lokal. Die eigene ETW-Sitzung ist
beendet.

Danach wurde ausschließlich der komplexe Rückgabetyp in `transport.rs` durch einen Typalias benannt,
um die Windows-Clippy-CI zu korrigieren. Das verändert keinen Laufzeitpfad, ist aber kein neuer
gemessener Binärstand. Die gemessenen EXEs sind separat gesichert; die obigen Hashes gelten für die
Messung. Keine ETW-Abnahme eines danach neu gebauten Executables behauptet.

Weiter offen: Appende bei offenem nativem Dialog oder während Transfer, echte Screenreader-/Fokus-
und Zeitgrenzenabnahme, externe COM-/Shell-/DNS-/Brokergegenproben und dauerhafter Auftrags-/Crash-
Lebenszyklus. Normale Produktstarts bleiben unisoliert. Q-01/Q-02 für das Produkt bleiben nicht
bestanden; #134 bleibt offen. Das strikte Datenschutzziel bleibt bestehen.

Die folgende Serie dokumentiert den früheren Build und wird durch diese Fortschreibung ergänzt.

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
