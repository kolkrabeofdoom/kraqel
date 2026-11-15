# KQ-069: WebView2-Netzverkehr unterscheiden

Stand: 15. November 2026. Issue [#134](https://github.com/kolkrabeofdoom/kraqel/issues/134) bleibt
offen. **Diagnosefortschritt, keine Netz- oder Datenschutzfreigabe.**

## Ausgangspunkt und Aufbau

PR #135 wurde nach drei erfolgreichen Checks am exakten Head
`202c5b27333f8aaa19f357f8b02132a67ae01bf6` zusammengeführt. Main:
`d6859378f86146980aaaddfc32afdfe3eb27feeb`, Issue #133 geschlossen. Der
[ETW-Befund KQ-068](native-m3-coverage.md) bleibt gültig.

Neue lokale NetLog-Diagnose des nativen Debug-Builds auf dieser Windows-Maschine: Tauri 2.12.5, wry
0.57.1, WebView2 157.0.4341.60 laut NetLog-Clientinfo, isolierte Bundle-ID und App-Datenwurzel
außerhalb des Repositorys. Executable-SHA-256:
`ab026592fcfc3281844d62fe9386f7edc44d156b227007c4ae82a6c7920b3e57`. Keine Providerbedienung,
Schlüssel oder privaten Projekte. Die Messung erfolgte bei verfügbarer Internetverbindung, nicht bei
physisch getrenntem Netz.

`additionalBrowserArgs` ersetzt bei dieser wry-Version die Standardargumente. Der Diagnosebuild
übernimmt deshalb ausdrücklich die vorhandenen Defaults
`--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection` und
`--autoplay-policy=no-user-gesture-required`; hinzu kommen ausschließlich `--log-net-log` und
`--net-log-capture-mode=Default`. Insbesondere wurde kein zusätzlicher Sicherheitsdienst
abgeschaltet. Keine Änderung von Firewall, Registry, Windows-Datenschutz, globalen
WebView2-Umgebungsvariablen oder produktiver Tauri-Konfiguration. Die drei einschlägigen
WebView2-Umgebungsvariablen waren vor dem Build nicht gesetzt. `dataDirectory` bleibt ungesetzt,
damit das Browserprofil zur isolierten App-Datenwurzel gehört.

## Beobachtung

Die [bereinigte maschinenlesbare Auswertung](native-m3-network-kq069.json) enthält ausschließlich
feste öffentliche Host-/Routenklassen und Zählwerte. Diese ersten drei App-Läufe wurden sauber
beendet und die jeweils abgeschlossenen JSON-Dateien vor einem Neustart unter eigenem Namen
gesichert. Der erste Lauf begann ohne Projektverzeichnis und zeigte die leere Projektliste. Er lief
von 12:53:43 bis 12:55:29 UTC (rund 106 Sekunden; 90 Sekunden waren nur das Ziel). Danach wurde der
vorhandene synthetische 100-Szenen-/50-Bilder-Korpus in dieselbe isolierte Wurzel eingespielt. Die
späteren Läufe verwenden somit dasselbe, bereits aufgewärmte Browserprofil.

| Lauf                     | Bedienung                                                                                  | URL-Start-Ereignisse                          | Antwortstatus-Ereignisse |
| ------------------------ | ------------------------------------------------------------------------------------------ | --------------------------------------------- | ------------------------ |
| Leerer Erststart         | Keine Projekte, kein Provider                                                              | 2 GET Konfiguration, 3 POST Komponentenupdate | 5 × 200                  |
| Projekt/Export           | Korpus öffnen, natives HTML erzeugen, App schließen                                        | 2 POST Komponentenupdate                      | 2 × 200                  |
| Neustart/Import/Änderung | App neu starten, synthetisches `.kraqel` importieren, Szene in Kopie umbenennen, schließen | 3 POST Komponentenupdate                      | 3 × 200                  |

Das HTML wurde mit Erfolgsmeldung `manuskript.html` (17.945 Bytes) erzeugt. Der Archivimport legte
eine neue Projekt-UUID an. Die Kopie enthält nach Prozessende `KQ-069 Netzprobe`, das
Ausgangsprojekt diesen Prüftitel nicht. Beide SQLite-Datenbanken: `integrity_check = ok`, keine
Fremdschlüsselfehler. Das ist ein zusätzlicher Import-/Änderungsnachweis, kein vollständiger neuer
CRUD-Löschlauf. Keine GUI-Löschung und kein Provideraufruf in KQ-069.

Die GETs gehen an `config.edge.skype.com` unter der festen Klasse `/config/…`. Microsoft bezeichnet
diesen Host als Experimentation and Configuration service. Die POSTs gehen an
`edge.microsoft.com/componentupdater/api/v1/update`. Die Einordnung als Komponentenupdate-Protokoll
ist eine **Schlussfolgerung aus der beobachteten Route**, gestützt durch Microsofts Beschreibung der
Browser-Komponentendienste. Der Host allein wäre keine ausreichende Zuordnung, da er zahlreiche
unterschiedliche Dienste bedient.
[Microsofts Endpunktdokumentation](https://learn.microsoft.com/en-us/deployedge/microsoft-edge-security-endpoints).

Im leeren Lauf existieren bereits Upload-Stream-Initialisierungen mit deklarierten Größen 4.902,
1.314 und 1.306 Bytes. Das sind Stream-Metadaten, keine gemessenen Nutzdaten auf der Leitung. Der
Default-NetLog enthält hier keinen prüfbaren Body-Inhalt. Der Initiator ist `other-or-opaque`; das
ist kein Kausalitätsbeweis. Insbesondere gilt:

- Diese Starts lassen sich ohne vorhandene Manuskripte reproduzieren. Das belegt keine vollständige
  Unbedenklichkeit des Verkehrs mit geöffneten Projekten.
- Start-Ereignisse, HTTP-Antworten und ETW-Transportereignisse sind verschiedene Größen. Die zehn
  Starts sind weder zehn TCP-Verbindungen noch eine vollständige Uploadbilanz.
- Unterschiedliche Anzahlen zwischen kaltem und warmem Profil erlauben keine Aussage darüber, ob
  eine bestimmte Fachaktion den Hintergrundverkehr ausgelöst hat.
- NetLog ersetzt die unabhängige ETW-Prozesszuordnung nicht und deckt nicht sämtliche
  Windows-Brokerpfade ab. Auch ein leerer NetLog würde keine Netzstille beweisen.

## Unterstützte Maßnahmen und Grenzen

Microsoft dokumentiert erforderliche WebView2-Diagnosedaten auch unabhängig vom optionalen
Windows-Diagnoseschalter. Die allgemeine Erfassung ist nicht vollständig durch die App steuerbar.
[WebView2-Datenschutz](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/data-privacy).

| Maßnahme                                                                       | Bewertung für diesen Befund                                                                                                                                           |
| ------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Bestehende Seiten-CSP/Cloudfreigabe                                            | Begrenzt App-Inhalte und Providerpfade; ergibt keinen belegten Stopp der hier beobachteten Runtime-Dienste.                                                           |
| Unterstütztes `IsCustomCrashReportingEnabled = true` vor Environment-Erzeugung | Verhindert laut API Microsoft-Crashdump-Upload; steuert nicht die beobachtete Konfigurations-/Komponentenroute. Kein Fix hierfür, nicht implementiert oder getestet.  |
| Tauri/wry mit eigenem WebView2-Environment                                     | Technisch zugänglich; ein Integrationspunkt ist keine vollständige Netzwerkgrenze.                                                                                    |
| Weitere Browserflags zum Unterdrücken von Hintergrundverkehr                   | Microsoft rät von produktiver Abhängigkeit ab; kein dauerhaft tragfähiger Freigabenachweis. Kein solches Experiment als Produktfix übernommen.                        |
| Firewall-/OS-Einstellungen auf diesem Rechner                                  | Keine App-Eigenschaft und keine portable Produktabnahme; unverändert.                                                                                                 |
| Eigenständig durchsetzbare Laufzeitisolation                                   | Möglicher nächster Prototyp; Unterstützung, Updatepfad, lokale IPC-/Assetpfade und assistive Bedienung müssen erst untersucht werden. Keine fertige Lösung behauptet. |

Quellen:
[Crash-API, TRUE-Semantik](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/icorewebview2environmentoptions3?view=webview2-1.0.4022.49),
[Browserflags](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/webview-features-flags).
Es wurde **keine** Maßnahme zur Unterdrückung des Verkehrs implementiert; folglich gibt es auch
keinen behaupteten Vorher-/Nachher-Abnahmenachweis.

## Vollständige ETW-Prozessmessung nach erneuter Nutzerfreigabe

Der erste UAC-Start war bereits als abgebrochen zurückgemeldet worden. Nach der Nutzerantwort „UAC
bestätigt“ wurde der Helfer erneut gestartet; diesmal lag `ready.json` vor. Aufzeichnung: 13:14:35
bis 13:21:29 UTC (rund 6 Minuten 54 Sekunden), Konvertierung abgeschlossen um 13:21:31 UTC. Der
bekannte Helfer und die Auswertung aus KQ-068 bleiben unverändert. Keine Paketnutzdaten, keine
Sicherheits-/Firewalländerung und kein Provideraufruf.

Erneuter Diagnosebuild aus demselben Quellstand und derselben isolierten Loggingkonfiguration,
Executable-SHA-256 `2a62f2be296e5816c8d0536fe358a3370b41bd6ca8d72e55022220591219a2b5`. Die drei
Root-PIDs wurden unmittelbar beim Start notiert: 10016, 21048, 11832. Alle drei Apps und ihre
Nachfahren sind **vor** Messende beendet; nach dem letzten Ende wurde der eigene Helfer über seine
STOP-Datei gestoppt. Der Zeitnachweis unterscheidet App-Ende, Trace-Ende und Konvertierung.

| Lauf | Im Trace tatsächlich bedient                                                                                                                                                                                                                                                                                           |
| ---- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1    | Neues synthetisches Projekt `KQ-069 ETW Probe`, Text in Szene 1 schreiben und Autosave abwarten; zweite leere Szene anlegen und in `ETW Löschprobe` umbenennen; HTML exportieren (743 Bytes); natives `.kraqel` exportieren (4.133 Bytes); synthetisches PNG importieren und Tags `etw, lokal` speichern; App beenden. |
| 2    | Vollständiger Prozessneustart, das gerade erzeugte Archiv als neue Projektkopie importieren, gespeicherten Text anzeigen; App beenden.                                                                                                                                                                                 |
| 3    | Erneuter Prozessneustart, Original anhand des nur dort vorhandenen Testbildes identifizieren, persistierte Tags prüfen; ausschließlich die vom Nutzer unmittelbar freigegebene leere Szene über Löschen und Löschen bestätigen entfernen; Szene 1 erhalten; App beenden.                                               |

[ETW-Ergebnis](native-m3-network-kq069-etw.json): `measurementValid = true`, keine Messprobleme,
vier/vier positive TCP-/UDP-Kontrollen für IPv4/IPv6, null verlorene Ereignisse/Puffer und null
unzuordenbare Netz- oder unbekannte Zielereignisse. Vollständig zugeordnet und beendet: 3 App-, 3
Conhost-, 24 WebView2- und 10 Git-Prozesslebenszeiten. **256 öffentliche Transportereignisse**,
ausschließlich WebView2: 12 Connect, 143 Send, 95 Receive, 6 Disconnect. Weitere 18
Loopback-Disconnect-Ereignisse; keine eigenen Netzereignisse von Rust oder Git im ausgewerteten
Scope. Exitcode **1** ist trotz gültiger Messung richtig: Der positive öffentliche Verkehr
verhindert die Freigabe. Das sind nicht 256 HTTP-Anfragen oder belegte Manuskript-Uploads. Die
8.781.824 Bytes große Roh-ETL und das systemweite XML bleiben lokal außerhalb Git.

Die [begleitenden abgeschlossenen NetLogs](native-m3-network-kq069-retest-netlog.json) enthalten 5,
1 und 1 POST-Starts an der Komponentenupdate-Route. Das ist zusätzliche Dienstevidenz, keine
vollständige Eins-zu-eins-Zuordnung aller ETW-Ereignisse oder ein Inhaltsnachweis.

[Unabhängiger Datenvergleich](native-m3-network-kq069-workflow.json): Szene 1 stimmt zwischen
Original und Archivkopie im Titel, semantisch geparsten Editor-JSON und Klartext überein; die
JSON-Serialisierung ist nicht bytegleich. Das vor der PNG-Einfuhr und Löschung erzeugte Archiv
enthält die leere Probe weiterhin; nur im Original ist sie gelöscht. Zwei Dokumente im Original,
drei in der Kopie, einschließlich Kapitel. Testbild und Thumbnail bleiben im Original, Original-SHA
entspricht SQLite, Tags persistiert. Beide Datenbanken: Integrität ok, keine FK-Fehler. Dies ist
Dokument-CRUD sowie Medienimport/-Metadaten unter ETW, keine erneute Vollabnahme sämtlicher
Medien-/Profil-/Stil-CRUD-Varianten aus KQ-068.

## Offene Abnahme und nächste Entscheidung

Der frühere Mangel fehlender Prozessenden ist für diesen Wiederholungslauf behoben. Eine Maßnahme
zur Unterdrückung des Verkehrs ist weiterhin nicht implementiert oder vergleichend abgenommen.
**Q-01/Q-02 bleiben nicht bestanden.**

Pflichtenheft §1, §5 und §10 sowie Q-01/Q-02 gelten unverändert. Die
[Entscheidungsvorlage ADR 0034](../architecture/decisions/0034-webview-network-boundary-proposal.md)
trennt den empfohlenen weiteren Isolationsversuch von einer ausschließlich ausdrücklich
freizugebenden Änderung der fachlichen Laufzeitgrenze. #134 wird nicht geschlossen.

## Reproduktion und Prüfung

[`scripts/m3-network/README.md`](../../scripts/m3-network/README.md) beschreibt Vorbereitung,
isolierten Build und bereinigte Auswertung. Die Rohlogs bleiben ausschließlich lokal außerhalb Git;
auch Default-NetLogs können sensible URLs, Kennungen und Header enthalten. Kein externer
NetLog-Viewer und kein Upload der Rohdaten. Nach Ende der Diagnose wird der normale native
Debug-Build ohne Zusatzkonfiguration wiederhergestellt.

`pnpm check` sowie Diagnosebuild und Wiederherstellung des normalen Debug-Builds sind erfolgreich.
27 Python-Tests prüfen ETW-Zuordnung, Diagnosevorbereitung und die beschränkte NetLog-Ausgabe. Die
zwölf neuen NetLog-Fälle umfassen URL-/Header-Geheimnisse, unbekannte Ziele, Loopback, mehrfache
Starts einer Quelle, unvollständige Logs und die verweigerte Netzstillebehauptung.
