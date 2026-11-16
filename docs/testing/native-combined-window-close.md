# KQ-069: Fensterende im kombinierten Live-/Neustartpfad

Stand: 16. November 2026. Ausschließlich Windows-Forschung und synthetische Testarchive.
[Bereinigte Aggregate](native-combined-window-close.json),
[Brokervertrag v3](../data-format/native-broker-recovery-v3.md). Striktes Datenschutzziel
unverändert; keine Produktfreigabe, Produkt-Q-01/Q-02 nicht bestanden.

## Gesichertes Paar und Änderungen

Die App bleibt unverändert. Der Helfer ergänzt die nativen Testmodi `closeDialog` und
`closePartial`; Ausgangspunkt ist Commit `e6d00086487f854095afaf41f73943a4bf05884f`. Die
Implementierung gehört zum Commit dieses Belegs. Finales Paar lokal unter
`%TEMP%/kraqel-kq069-combined-window-build-20261116-02/` mit Manifest gesichert:

- App-SHA-256: `6e56a51ed2e913a878a7935e039bd69628a49be25f0fbab7d02fce3a72fe8c70`.
- Helfer-SHA-256: `1b5c0efa771f58a7e1274369d391b92a2c788aa1d64756e6ef4530b2f2555b4f`.

Die neuen Modi erlauben menschlichen Hauptfensterschluss ohne erzwungenen Brokerabsturz. Native
Auswahl und Ticket-Bedienungsflag werden bei Anlage und Authentisierung konsistent geprüft. Nur
Crashpunkt `completed` ist erlaubt: Er bleibt beim Dialog oder Teiltransfer unerreicht, damit kein
Crash-Haltepunkt die Abbruchbehandlung blockiert. Keine neue Rendererfunktion oder Dateifreigabe.
Der Teiltransfer hält höchstens 60 Sekunden; der Dateiauftrag erhält keine längere Gültigkeit.

Erfolg verlangt Renderer-/Terminalereignis, reguläres Appende 0, begrenztes Beenden/Join des
Auswahlthreads und Jobleerstand. Nach beobachtetem Appende wird die Ereignispipe nochmals geleert,
damit ein unmittelbar vor Ende geschriebenes Terminalereignis nicht übergangen wird. Nur die
konkrete Broken-Pipe-Abbruchantwort des geschlossenen Dialogs (109/232) darf als erwarteter
Transportabschluss gelten; fremde Stufen/Berechtigungsfehler bleiben Fehler.

## Native Prüfung und unabhängige Gegenbelege

| Fall                   | Menschliche Bestätigung                                                                                             | Nativer Gegenbeleg vor Recovery                                                                                                |
| ---------------------- | ------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------ |
| Offener Dateidialog    | Import gestartet, nichts ausgewählt, ausschließlich Hauptfenster per X geschlossen; Dialog schließt automatisch mit | App/Broker 0; `CANCELLED`, keine Receipt; Auswahlthread gejoint, acht gezielte Dialog-Schließnachrichten, Job leer             |
| Laufender Teiltransfer | `input.kraqel` nativ ausgewählt und Hauptfenster während „Testarchiv wird geprüft …“ per X geschlossen              | Sender und Empfänger jeweils 1.024 von 11.915 Bytes; App/Broker 0; `CANCELLED`, keine Receipt; Auswahlthread gejoint, Job leer |

Die acht Schließnachrichten sind Polling-Wiederholungen an den eigenen Auswahlthread, keine acht
Dialoge. Keine gemessene Millisekundenlatenz behauptet. Der Empfängerzähler wurde vor dem neuen
Folgeimport gesichert, damit dessen vollständiger Empfang den Teiltransferbeleg nicht ersetzt.

Je Fall anschließend separat und automatisch geprüft:

1. Neuer Folgeauftrag vor authentisierter Wiederherstellung abgewiesen, Controllerende 1.
2. Authentisierte Recovery des verbrauchten Auftrags: keine Kopie, Replay gesperrt, Staging leer.
   Ursprüngliche Importanfrage ebenfalls gesperrt, ohne neue Auswahl oder Journaländerung.
3. Erst nach MAC-Abschlussbeleg ausdrücklich neuer Fixture-Auftrag mit anderer Kennung;
   erfolgreicher Import, absichtlicher Brokerverlust mit Exitcode 86.
4. Zweifache Recovery des neuen Auftrags und Altanfrage-Abwehr: genau eine Kopie mit zwei
   Dokumenten.
5. Authentisierte Stilllegung und Profilentfernung.

Beide vollständigen Fälle bestanden: 16 App-Starts, sechs Recovery-Controller, zwei abgewiesene
vorzeitige Folgeaktionen und zwei absichtliche Brokerabstürze erst beim jeweils neuen Auftrag. Zwei
reguläre initiale Appenden 0 und zwölf kontrollierte Recovery-/Altanfrage-Testenden 77 direkt
belegt; die beiden Crash-Appenden werden hier nicht als reguläre Enden ausgegeben. Kein
ETW-Lebenszeitenbeleg für diese neue Population behauptet.

Unabhängiger Nur-Lese-Audit bestätigt HMACs und Abschlussbelege, Vorgängerhashes, kanonische
Wurzel-/Programmhashbindung, vier unterschiedliche verbrauchte Aufträge und zwei unterschiedliche
legitime Kopien. Zwei alte Aufträge dauerhaft `failed:CANCELLED`, zwei neue `succeeded`.
SQLite-Integrität/Fremdschlüssel, Archivhash/Bytezahl, je zwei Dokumente und Unicode-Inhalt stimmen;
kein Replay-Duplikat oder Stagingrest. Alle 16 nativen Starts mit exakter AppContainer-Identität,
null Capabilities und verweigertem Zugriff auf private Brokerdateien. Beide finalen Profile
entfernt, keine Forschungsprozesse aktiv. Rohdaten, Pfade, IDs und Schlüssel bleiben lokal.

## Verworfener Erstlauf, Prüfungen und Grenzen

Beim ersten Helferstand funktionierte der Dialogschluss, die anschließende Recovery scheiterte
jedoch mit `broker-authority-binding`: Die neue manuelle Modusausnahme war bei der Anlage, noch
nicht konsistent bei der Authentisierung berücksichtigt. Kein Abschlussbeleg oder Folgegrant. Dieser
Lauf zählt nicht als bestandene kombinierte Prüfung. Sein bereits prozessfreies Testprofil wurde
ausschließlich über die gezielte Windows-Profillöschung entfernt; die alte Autorität wurde nicht
repariert, migriert oder erneut verwendet. Diagnosebelege bleiben lokal erhalten. Korrektur mit
zusätzlichem Modus-Vertragstest, neuem Helferhash und frischen Testwurzeln für beide finalen Fälle.

31 aktive Helfer-/Transporttests, Helfer-Clippy mit Warnungen als Fehler, rustfmt, `pnpm check`
sowie Dokumentformatierung/Diff-/Hash-/Faktenabgleich bestanden. Neue Tests sichern insbesondere
Ereignisreihenfolge, Ausschluss von Replay/Recovery im initialen Fensterendepfad, begrenzte manuelle
Modi und enge Transportfehlerausnahme. Zwei interaktive Shell-Helfertests nicht erneut separat
ausgeführt; beide oben beschriebenen echten nativen Auswahlabläufe wurden ausgeführt. Kein neuer
vollständiger Appbuild, normale Debug-App unverändert.

Reproduktion: frische `kraqel-isolation-ui-restart-<UUID ohne Bindestriche>`-Wurzel verwenden,
`ui-broker-live-seed <Wurzel> <gesicherte App> completed closeDialog|closePartial` starten und den
jeweiligen Hauptfensterschluss manuell ausführen. Danach `ui-broker-live-next` vor Recovery abweisen
lassen, `ui-broker-recover <Wurzel> first`, explizites `ui-broker-live-next`, Recovery
`first`/`second` und `retire` prüfen. MACs/Journal und Zwischenzähler unabhängig abgleichen. Keine
Testwurzeln/Schlüssel auf andere Rechner übertragen oder Diagnose-JSON als Autorität verwenden.

Nächster Schritt: eigener frischer ETW-Netzvergleich dieses neuen Hashpaars einschließlich beider
Fensterendepfade und Wiederherstellung. Der
[vorige Netzbeleg](native-combined-assistive-restart-etw.md) gilt ausschließlich für seinen
damaligen Helferhash. Weitere allgemeine F08-/Fensterende-Randfälle, Produktablage,
Updates/Migration und externe Dienstgrenzen bleiben offen. Keine Netzabnahme des neuen Paars aus
funktionalen Tests ableiten und keine Runtime-/Domänenausnahme einführen.
