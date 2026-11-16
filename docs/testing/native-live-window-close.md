# Fensterende bei offenem Dialog und Teiltransfer · KQ-069 / #134

## ETW-Fortschreibung am unveränderten Fensterende-Build

Neue Serie vom 16. November 2026: drei vollständige Sitzungen mit exakt den unten genannten finalen
App-/Controllerhashes, Code-Stand `eb52ee1c58bf72286350cae36b36d92d7763cced`. Während der Messung
keine Code- oder Binäränderung. Regulärer Import mit Replay als Referenz, Fensterende bei offenem
Dateidialog und Fensterende nach angehaltenem Teiltransfer wurden nativ bedient und nach Ende
unabhängig geprüft. Damit sind nun beide Abbruchfälle am selben finalen Controller gemessen; die
unten beschriebene frühere unterschiedliche Controllerzuordnung bleibt historische Evidenz.

ETW: **16. November 2026, 06:54:42.8694694–07:02:26.5745349 UTC**. Drei Controller-, drei App-, drei
Console-Host- und 19 WebView2-Prozesslebenszeiten vollständig zugeordnet. Vier/vier positive
TCP-/UDP-Kontrollen über IPv4/IPv6, keine Ereignis-/Pufferverluste, keine Zuordnungslücken oder
unbekannten Ziele. **0 Ereignisse außerhalb Loopback**; 22 Loopback-Disconnects getrennt.
[Bereinigte Messdaten](native-live-window-close-etw.json). Die 3.866.624 Byte ETL-Datei und das
systemweite XML bleiben lokal, die eigene Erfassung ist beendet.

| Nachweis                     | Vollständige Controller-/App-Bäume | WebView2-Lebenszeiten | Kontrollen | Nicht-Loopback | Loopback | Verluste |
| ---------------------------- | ---------------------------------- | --------------------- | ---------- | -------------- | -------- | -------- |
| Vorheriger Live-Kanal        | 3                                  | 18                    | 4/4        | 0              | 22       | 0        |
| Geänderter Fensterende-Stand | 3                                  | 19                    | 4/4        | 0              | 22       | 0        |

Dies ist ein Vergleich kalibrierter Beobachtungen unterschiedlicher Builds und Bedienpfade, kein
kausaler oder laufzeitnormalisierter Ratenvergleich. Keine HTTP-/Upload- oder Body-Aussage.

Der Referenzlauf erzeugt genau eine neue UUID-Kopie, zwei Dokumente und exakten Unicode-Text;
SQLite-Integrität und Fremdschlüssel sind unabhängig bestätigt, Replay ist protokolliert abgewiesen.
Beide Abbruchläufe liefern `CANCELLED` und leere Projektwurzeln einschließlich Staging. Der
Dialoglauf protokolliert sechs gezielte Schließnachrichten; im Transferlauf stimmen Sender und
Empfänger mit 1.024 von 11.915 Archivbytes überein. Alle drei: App-/Controller-Exitcode 0,
Auswahlthread beendet, null Jobprozesse und Profil entfernt. Erwartete AppContainer-Identität mit
null Capabilities, geprüfte Handle-Vererbungsflags und nativer Tauri-Aufruf sind pro Sitzung
bestätigt.

Die Forschungsergebnisse ändern nicht den fehlenden Produkt-Q-01-/Q-02-Pass. Externe COM-/Shell-/
DNS-/Brokerpfade und kompromittierte Prozesse sind nicht abgenommen. Striktes Datenschutzziel
unverändert, #134 bleibt offen. Die folgende funktionale Serie dokumentiert die vorausgehenden
Versuche; ihr damaliger fehlender ETW-Bezug wird durch diesen separat gebundenen Nachweis ergänzt.

16. November 2026, Windows, isoliertes Forschungsfeature, ausschließlich synthetische Daten. Keine
    Produktfreigabe; striktes Datenschutzziel unverändert.

## Offener Dateidialog

Die testende Person öffnet aus KRAQEL den nativen Dateidialog, wechselt zurück zum Hauptfenster und
schließt es über X. Die ersten drei Versuche schlossen beide Fenster und räumten App, Job und Profil
auf, endeten jedoch mit `live-selection-unfinished`. Zwei Timerexperimente lieferten keinen
kontrollierten Abschluss und wurden aus dem Code entfernt. Sie zählen nicht als bestandene Serie.

Der finale Controller ermittelt die ID seines eigenen Auswahlthreads. Beim Appende sendet er
`WM_CLOSE` ausschließlich an Fenster dieses Threads mit Dialogklasse `#32770`, nachdem zusätzlich
die eigene Prozess-ID geprüft wurde. Keine fremden Fenster und keine Rendererparameter. Nach Ende
des App-Jobs wird Abbruch gesetzt, die controllerseitigen Pipe-Duplikate werden geschlossen, und
höchstens fünf Sekunden auf den Auswahlthread gewartet. Timeout bleibt ein Fehler. Ein überflüssiger
Schreibversuch einer Abbruchantwort an die schon beendete App wurde entfernt.

Finale native Sitzung: beide Fenster schließen laut testender Person unmittelbar; unabhängig
bestätigt sind Controller- und App-Exitcode 0, `CANCELLED`, `passed: true`,
`selectionThreadJoined: true`, vier gezielte Schließnachrichten, leere Projektwurzel einschließlich
Staging, null Jobprozesse und entferntes AppContainer-Profil. Root vor Fortsetzung als erwarteter
AppContainer mit null Capabilities geprüft.

- App-SHA-256: `3b99bae5e3ff9b1cf0f47b1f377dceea4f558cf3b2805ca2a2ea8ea2d0112385`
- Finaler Controller-SHA-256: `3c5a9e27c0e516d84a10576e109da2f773e1681bfdca9c47fec69bcafa0a86be`

## Angehaltener Teiltransfer

Der explizite Controller-Testmodus `ui-live-transfer-hold` überträgt nach nativer Auswahl den
regulären Auftrag und genau 1.024 von 11.915 Archivbytes. Er hält höchstens 60 Sekunden; es gibt
keinen solchen Rendererparameter und keine zusätzliche Dateifreigabe. Sender und Empfänger schreiben
ausschließlich lokale Bytezähler, keine Nutzdaten. Der übliche Forschungsmodus hat diesen Haltepunkt
nicht.

Native Prüfung: Nach Dateiauswahl zeigt die Oberfläche weiter „Testarchiv wird geprüft …“; die
testende Person schließt das Hauptfenster. Unabhängig nach Ende: `bytesSent: 1024`,
`payloadBytesReceived: 1024`, `expectedBytes: 11915`, `complete: false`. Controller- und
App-Exitcode 0, `CANCELLED`, `passed: true`, Auswahlthread beendet, keine Receipt, vollständig leere
Projektwurzel, null Jobprozesse und Profil entfernt. Es war ein echter unvollständiger Empfang, kein
lediglich langsamer oder bereits veröffentlichter Import.

Die gestagte App besitzt denselben obigen SHA. Dieser Fall lief vor der letzten ausschließlich den
Dialog-Abbruchantwortpfad betreffenden Controllerkorrektur. Der vorherige Controllerhash wurde nicht
separat aufgezeichnet; deshalb kein identischer finaler Controller-Binärstand für beide Fälle
behauptet. Die vorherigen Reports und fehlgeschlagenen Bewertungen bleiben lokal erhalten.

## Prüfungen, Reproduktion und Grenzen

`pnpm check`, 21 reguläre Helfer-/Transporttests und direkte Clippy-Prüfung mit Warnungen als Fehler
bestanden. Normaler Debug-Build ohne Forschungsseite erfolgreich wiederhergestellt. Zusätzlich
besteht der gezielt lokal ausgeführte, in CI ignorierte native Test
`native_owned_dialog_can_be_closed_and_joins`: eigener Shell-Dateidialog, eigene Schließnachricht,
erfolgreiche Rückkehr und beendeter Hilfsthread. Broken-Pipe-Regressionsprüfung akzeptiert nur den
konkreten Transportfehler bei `CANCELLED` ohne Receipt, keine anderen Datei-/Berechtigungsfehler.

```powershell
cargo test --locked --manifest-path scripts/runtime-isolation/probe/Cargo.toml native_owned_dialog_can_be_closed_and_joins -- --ignored --nocapture
```

Normale Reproduktion: neue direkte TEMP-Wurzel mit Präfix `kraqel-isolation-ui-`, Forschungs-App und
gelockter Controller. Für Dialogende `ui-live`, für Teiltransfer `ui-live-transfer-hold` verwenden.
Im Transfermodus nach Auswahl das Hauptfenster schließen, bevor 60 Sekunden vergehen;
Sender-/Empfängerzähler und leere Projektwurzel unabhängig prüfen.

[`WM_CLOSE`](https://learn.microsoft.com/en-us/windows/win32/winmsg/wm-close) ist eine
Schließaufforderung; der Erfolg wird hier anhand tatsächlicher Thread-/Prozessenden geprüft, nicht
aus erfolgreichem `PostMessageW` abgeleitet.

Zum Zeitpunkt der vorausgehenden funktionalen Serie lag noch kein neuer ETW-Lauf vor. Die oben
ergänzte Messung bindet nun Referenz und beide Abbruchfälle an das identische finale
App-/Controllerpaar. Der [ältere Netzbeleg](native-live-file-selection-etw.json) bleibt seinen
eigenen Hashes zugeordnet. Keine Screenreader-/Fokusabnahme, kein Nachweis gegen kompromittierte
Runtimeprozesse oder externe COM-/Shell-/DNS-/Brokerpfade. Dauerhafte Auftrags-/Crash-Reconciliation
und Produkt-/Installerintegration bleiben offen. Q-01/Q-02 für das Produkt weiterhin nicht
bestanden, #134 bleibt offen. Lokale Wurzeln, PIDs und Diagnosen bleiben außerhalb von Git.
