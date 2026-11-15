# KQ-069 A: Technischer AppContainer-Prototyp

15. November 2026. Issue [#134](https://github.com/kolkrabeofdoom/kraqel/issues/134) bleibt offen.
    Das strikte Datenschutzziel bleibt unverändert. **Machbarkeit und begrenzte Messung, keine
    produktive Isolation oder vollständige F08-/M3-Freigabe.**

## GitHub- und Anforderungsstand

PR #136 wurde nach drei erfolgreichen Checks am exakten Head
`d4e2c64cc5b72d5b6dafc0b025c72fb1715a2842` zusammengeführt. Main:
`caaa69408816960e29278c0a0f5cfe12bc347dfc`, Merge am 15. November 2026 um 13:37:14 UTC. Der Nutzer
beauftragt anschließend technische Laufzeitisolation und bestätigt ausdrücklich das strikte Ziel.
Damit ist Untersuchungsrichtung A aus ADR 0034 beauftragt; keine Runtime-Ausnahme aus B angenommen.
Pflichtenheft Revision 1 §1, §5, §10 und die bestehenden Q-01/Q-02 gelten weiter.

Der [vorherige gültige ETW-Lauf](native-m3-webview-network.md) findet 256 öffentliche
WebView2-Transportereignisse. Die normale App ist nach wie vor unverändert und startet ohne die hier
untersuchte Isolation. Ein Forschungslauncher darf diesen Produktstatus nicht überschreiben.

## Technischer Versuch

Der [separate Rust-Helfer](../../scripts/runtime-isolation/README.md) nutzt dokumentierte
Win32-AppContainer-APIs. Er erstellt eine eigene Profilidentität ohne Capabilities, startet den Root
suspendiert, prüft dessen Paketidentität und Capability-Anzahl vor der ersten Ausführung, ordnet ihn
einem Job ohne Breakaway-Freigabe zu und setzt erst dann fort. Fehler führen zum Abbruch; es gibt
keinen unisolierten Fallback. Nur der eigene TEMP-Testbereich erhält passende Dateirechte. Für die
App ist ausschließlich `data` schreib-, umbenenn- und löschbar freigegeben. Nach Ablauf/STOP oder
Schließen werden die Jobprozesse beendet und das eigene Profil entfernt.

Der erste native Vorversuch zeigte den Startscreen, scheiterte aber beim atomaren Veröffentlichen
eines Projekts: Die anfänglich unzureichenden Dateirechte wurden ausschließlich im neuen
Testdatenordner um Rename-/Delete-Rechte ergänzt. Der folgende Versuch bestätigt Anlage, Autosave
und HTML. Keine Änderung von Benutzerverzeichnis-, Workspace-, Firewall-, Registry- oder
Diagnose-Richtlinien; nur temporäre eigene AppContainer-Profile und Testdatei-ACLs.

Tauri-Konfiguration: eigene Bundle-ID und App-Datenwurzel, unveränderte CSP und unveränderte
wry-Standardargumente. Keine zusätzlichen Browser-Abschaltflags. Die isolierte Kindumgebung enthält
nur fest gebildete System-/Testpfade und keine geerbten Providergeheimnisse. Alle Daten sind
synthetisch, kein Provideraufruf und keine physische Netztrennung.

Windows 11 Pro, Build 26200; installierte WebView2-Version 157.0.4341.60. App-Quellstand `caaa694`;
gemessener nativer Executable-SHA-256:
`163dddd363bdbfe919f87f56db4d18cf0af92c7e4d1f7238ad14f84d871f5d8b`. Die
[bereinigte Versuchsevidenz](native-runtime-isolation-prototype.json) hält die Hashes des gemessenen
Forschungslaunchers und der anschließend aufbereiteten finalen CLI getrennt fest. Letztere ergänzt
gemeinsame Guard-Tests und strengere Diagnosestatus-/Neustartprüfungen; sie wurde zusätzlich live
mit Socketversuch und nativem Neustart geprüft. Der normale Debug-Build ist nach der Prüfung
wiederhergestellt. Keine dieser Binärdateien wird als Release ausgeliefert.

## Socket- und Tokenprüfung

Der Controller bestätigt lokale TCP-/UDP-Kommunikation für IPv4/IPv6 sowie einen öffentlichen
IPv4-TCP-Handshake. Isolierter Testprozess und selbst gestarteter Nachfahre haben tatsächlich
AppContainer-Tokens mit null Capabilities. Beobachtet in beiden:

| Fall                                       | Tatsächliches Ergebnis            | Aussage                                                                         |
| ------------------------------------------ | --------------------------------- | ------------------------------------------------------------------------------- |
| Öffentlicher TCP-/IPv4-Connect             | WinSock 10013, Zugriff verweigert | Synchrone Verweigerung bei erfolgreicher nicht isolierter IPv4-Positivkontrolle |
| Öffentliche IPv6-Ziele                     | 10051, Netz nicht erreichbar      | Kein Berechtigungs-/Isolationsnachweis                                          |
| Lokaler TCP-/IPv4-/IPv6-Connect            | Erfolgreich                       | Loopback ist hier nicht pauschal gesperrt                                       |
| UDP-Sendeaufruf, lokal und öffentlich IPv4 | API meldet Erfolg                 | Keine Aussage über Zustellung öffentlicher Datagramme                           |
| Feste DNS-Probe `example.com`              | 11001                             | Keine vollständige DNS-/Broker-Abnahme                                          |

Deshalb bleibt die vollständige Socket-Sperrbehauptung falsch und die CLI liefert korrekt
Exitcode 1. Die Ausgabe enthält keine Ziel-IP-Adressen, beliebigen Hosts, Pfade, SIDs oder
Header/Body-Inhalte. Öffentliche Kontrollen verwenden nur feste Diagnoseziele, UDP das Byte 42;
niemals Projektdaten. API-Ergebnisse sind keine Paket- oder Uploadbilanz.

Für die echte App wurden im gemessenen Scope 8, 8 und 7 Prozesse stichprobenartig attestiert,
jeweils AppContainer mit null Capabilities: insgesamt 23 Tokenbeobachtungen. Die dritte Stichprobe
erfasst Root und zum Testprofil gehörige WebView2-Prozesse, ohne Conhost. Die ETW-Datei enthält
insgesamt 25 Prozesslebenszeiten; zwei davon sind durch diese Momentaufnahmen nicht abgedeckt. Keine
lückenlose Vor-Ausführungsattestierung aller Nachfahren behauptet. Die drei Root-Tokens wurden
jeweils zusätzlich suspendiert und gegen die erwartete Identität geprüft.

## Native Abläufe und Vergleich

Ein gemeinsamer ETW-Trace läuft vom 15. November, 13:42:57 bis 13:53:44 UTC; Konvertierung 13:53:46
UTC. Der zuvor bereits gestartete Vorversuch gehört nicht zu den nachfolgend ausgewerteten
Root-PIDs. Vier lokale TCP-/UDP-Kontrollen (IPv4/IPv6), null verlorene Ereignisse/ Puffer und keine
Zuordnungslücken. Alle ausgewerteten Roots samt Nachfahren enden vor Trace-Ende.

| Gruppe                                         | Geprüfte Bedienung                                                                                                                 | ETW-Befund                                                     |
| ---------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------- |
| Isoliert, Root 7980                            | Neues Projekt, Unicode-Text, Autosave, HTML und PDF; erfolgloser Versuch der Bild-Dateiauswahl                                     | Zusammen mit beiden Neustarts: 0 Ereignisse außerhalb Loopback |
| Isoliert, Root 13456                           | Vollständiger Neustart mit wieder erzeugter eigener Profilidentität; Projekt öffnen, gespeicherten Text lesen                      | Im gemeinsamen isolierten Ergebnis enthalten                   |
| Explizit unisolierter Kontrolllauf, Root 22776 | Derselbe Executable-SHA und synthetische Datenbestand; Projekt öffnen; Bild-Dateidialog öffnet sich, ohne Dateizugriff abgebrochen | 201 öffentliche Ereignisse aus WebView2                        |
| Isoliert, Root 16076                           | Erneuter Neustart und Wiederholung des sichtbaren Bildauswahl-Buttons per Screenshot-Koordinate                                    | Kein Dateidialog sichtbar; im isolierten Ergebnis enthalten    |

[ETW-Auswertung beider Gruppen](native-runtime-isolation-etw.json): Die isolierte Gruppe hat 3 App-,
3 Conhost- und 19 WebView2-Lebenszeiten, 18 Loopback-Disconnect-Ereignisse und **0 außerhalb
Loopback**, Exitcode 0 des begrenzten Auswerters. Der unisolierte Vergleich hat 1 App-, 1 Conhost-
und 12 WebView2-Lebenszeiten; **201 öffentliche Ereignisse** (11 Connect, 113 Send, 67 Receive, 10
Disconnect), zusätzlich sechs Loopback-Ereignisse, Exitcode 1. Beide Messungen sind gültig;
Kontrollen werden aus demselben Trace gelesen und sind nicht acht unabhängige Kontrollen. Roh-ETL
(6.488.064 Bytes) und systemweites XML bleiben ausschließlich lokal außerhalb Git.

Der Vergleich ist ein expliziter Diagnosefall, kein Fallback. Unisolierte und isolierte Gruppe
verwenden unterschiedliche Umgebungsblöcke und Profilkontexte sowie unterschiedliche Laufzeiten.
Daher keine quantitative oder auf einen einzelnen Schalter reduzierte Kausalaussage. Die Kombination
aus tatsächlichen Tokens, TCP-/IPv4-Verweigerung und gültigem ETW ist ein Machbarkeitsbeleg für
diese Konfiguration, keine universelle Windows-Netzgarantie.

Unabhängig rückgelesen: eine SQLite-Datenbank, zwei Dokumentknoten einschließlich Kapitel, genau der
gespeicherte Unicode-Text nach Neustart, Integrität ok und keine FK-Fehler. HTML 742 Bytes enthält
den Text; PDF 2.480.595 Bytes, eine Seite, Lateinisch/Griechisch/Kyrillisch aus dem eingebetteten
Schriftprofil extrahierbar. Kein Bild importiert. Die Bildauswahl scheitert auch beim erneut
beobachteten Koordinatenklick im Container; außerhalb öffnet sie sich. Das ist eine beobachtete
Integrationsgrenze, noch keine abschließende Ursachenzuordnung zwischen Token, COM-/Shell-Broker,
Profilkontext und bereinigter Umgebung. Kein Archiv-/DOCX-Neuimport oder vollständiger
Medien-/Profil-/Stil-CRUD unter Isolation abgenommen. Keine GUI-Löschungen.

## Noch erforderlich unter dem strikten Ziel

- Datei-/Shell-Dialoge und expliziten Datei-/Archiv-/Medienaustausch gezielt integrieren, ohne
  breite Freigabe des Benutzerprofils oder Aufhebung der Netzwerkisolation.
- Loopback-, DNS-, Broker-, öffentliche UDP-/IPv6-Pfade und Prozesse außerhalb des Job-/ Elternbaums
  gesondert kalibrieren. UDP-API-Erfolg und fehlende IPv6-Route reichen nicht.
- Fail-closed-Start und sichere Profil-/Prozess-/ACL-Lebensdauer in den echten Installer und
  Launcher integrieren; Update-/Signierungs-/Crashpfade und Zweitrechner prüfen. Der eigenständige
  Forschungshelfer ist keine Produktarchitektur oder bewiesene Grenze für echte Manuskripte.
- Git, Schlüsselspeicher, lokale KI und ausdrücklich freigegebene Cloudkontexte mit getrennten
  Fähigkeiten bewerten. Den gesamten WebView2-Baum für Cloud-Opt-in zu entsperren ist kein Fix.
- Vollständige funktionale und assistive F08-/M3-Matrix unter der endgültigen Grenze wiederholen.

**Q-01/Q-02 bleiben für das Produkt nicht bestanden**, #134 offen. Das strengere Ziel wird
beibehalten; keine Telemetrie- oder Runtime-Ausnahme eingeführt.

## Prüfbarkeit und Quellen

`pnpm check` ist vollständig erfolgreich abgeschlossen, ebenso `git diff --check`. Der normale
native Debug-Build wurde erfolgreich wiederhergestellt.

Fünf Guard-Tests des Forschungshelfers und Clippy mit Warnungen als Fehler sind lokal erfolgreich.
Die CI baut/prüft das separate Workspace auf Windows/Linux, führt dort aber keine Live-Netz- oder
AppContainer-Versuche aus. Produktprüfung bleibt `pnpm check`; ETW-/NetLog-Helfer behalten ihre 27
Tests. Reproduktion und Befehle stehen im [Tool-README](../../scripts/runtime-isolation/README.md).

Microsoft dokumentiert AppContainer und Tokenvererbung auch für nicht paketierte klassische Apps.
Die gesonderten neueren Win32-Isolation-Paketwerkzeuge sind als Preview bezeichnet; dieser Versuch
installiert sie nicht und verwendet keine experimentelle CreateProcessInSandbox-API.
[Klassische AppContainer](https://learn.microsoft.com/en-us/windows/win32/secauthz/appcontainer-for-legacy-applications-),
[Start-APIs](https://learn.microsoft.com/en-us/windows/win32/secauthz/implementing-an-appcontainer),
[Preview-Abgrenzung](https://learn.microsoft.com/en-us/windows/win32/secauthz/app-isolation-overview).
