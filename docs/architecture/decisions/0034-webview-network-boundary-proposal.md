# ADR 0034: Windows-Laufzeitgrenze nach WebView2-Netzbefund

Dauerhafte Sitzungsbindung/Recovery, 17. November:
[Nachweis](../../testing/native-product-session-recovery.md) bindet Apppaket und persistierte
synthetische Sitzung in Kontrollschema 3. Authentisierte neue Startrecords vor Recovery unter
derselben Kontrollsperre und demselben Produktjob; vorhandenes Journal und signierte Appbytes
zwingend. Zwei echte Controllerabbrüche, vier Wiederanläufe, keine Replay-Kopie. Keine echte
Datenmigration, Schema-3-Updatepolitik oder normale Produktintegration. Neuer Broker benötigt
frisches ETW; keine Änderung am strikten Datenschutzziel.

Kontrollgebundener synthetischer Start, 17. November:
[Implementierung und Beleg](../../testing/native-product-starter.md) verbinden erstmals
Startlebenszeit, private Kontrollsperre und den von Updates geprüften Produktjob. Zwei isolierte
Starts mit zwölf tatsächlichen Dateiverweigerungen und unabhängigem MAC-/Datenaudit bestanden.
Appdaten bleiben frische getrennte TEMP-Sitzungen; keine normale Produktintegration, dauerhafte
Apppaket-/Sitzungsbindung oder echte Migration. Kein frischer ETW-Beleg des neuen Brokerhashs;
striktes Datenschutzziel unverändert.

Kontroll-Crash-/Netzbeleg, 16. November:
[Eigener frischer Nachweis](../../testing/native-product-control-crash-etw.md) für denselben neuen
gesicherten Prüfstand. 28 Prozessabbrüche, 17 gesperrte/elf nutzbare Zustände, acht explizite
Weiterführungen; keine automatische Reparatur oder Schlüsselneuanlage. Die echte Vorschauablage
wurde nicht verwendet. Zwei gültige ETW-Phasen, acht Kontrollen, Nullverluste, 1.136 explizite
Enden, 0 Nicht-Loopback-/444 getrennte Loopback-Ereignisse. Normale Produktintegration, echte
Datenmigration, externe Dienstgrenzen und vollständiges Checkpoint-/Stromausfallverhalten bleiben
offen; striktes Datenschutzziel unverändert.

Kontrollablage-Vorschau, 16. November:
[Implementierung/Beleg](../../testing/native-product-control.md) und
[Vertrag](../../data-format/native-product-control-v1.md) ergänzen dauerhafte private Ablage, lokal
authentisierte Programmwechsel und explizite Kontrollschema-Migration. Root außerhalb des
Coding-Werkzeug-Paketcache, ACL-Prüfung vor Schlüsselerzeugung, MAC-Kette/Kopf, exklusive Sperre und
Jobleerstand. Drei native Programmwechsel und 18 echte AppContainer-Dateiverweigerungen bestanden.
Keine automatische Übernahme alter Wurzeln/Journale, keine normale Produktintegration oder
Herausgeber-/Netzabnahme; striktes Ziel unverändert. Nächster Scope: Kontroll-Crash-Serie und
eigenes frisches ETW, danach isolierter Produktstarter/echte Datenmigration.

Fensterende-ETW, 16. November:
[Eigener frischer Beleg](../../testing/native-combined-window-close-etw.md) des unveränderten
gesicherten Paars mit Automatik und beiden nativen Fensterendepfaden inklusive Recovery. Acht
Kontrollen, Nullverluste, alle 942 Enden explizit, 0 beobachtete Nicht-Loopback-/536 getrennte
Loopback-Ereignisse. Unabhängiger MAC-/Daten-/Lebenszeitaudit bestanden. Keine neue
Runtime-/Domänenausnahme, kein Produkt- oder externer Dienstnachweis. Nächster Scope: geschützte
Produktablage und authentisierte Updates/Migration; weitere F08-/Dienstgrenzen offen.

Kombinierter Hauptfensterschluss, 16. November:
[Eigener Funktionsbeleg](../../testing/native-combined-window-close.md) für offenen Dialog und
Teiltransfer, danach authentisierte Recovery und neue Aufträge bestanden. App unverändert; neue
native Helfermodi mit konsistenter manueller Modusprüfung, regulärem Ende, Threadjoin und
Jobleerstand. Verworfener Erstlauf bleibt unbestanden. Eigener frischer ETW-Beleg inzwischen oben
verlinkt; keine Übertragung früherer Nachweise oder Produkt-/Dienstfreigabe. Striktes
Datenschutzziel unverändert, Produktablage/Updates/Migration und weitere F08-Randfälle offen.

Tastatur und ETW des manuellen Brokerpaars, 16. November: Eigener frischer
[Nachweis](../../testing/native-combined-assistive-restart-etw.md) mit automatischer Serie und vier
manuellen Wiederanläufen. Acht Kontrollen, Nullverluste, 906 explizite Enden, 0 Nicht-Loopback-/514
getrennte Loopback-Ereignisse. Menschliche Tastatur-/Hörbestätigung und native Replay-/Fokusbelege
getrennt von physischer Eingabezuordnung behandeln. Kein Runtimeumbau, keine neue Produktautorität
oder externe Dienstabnahme. Weitere Fensterendeprüfung/Produktablage/Updates/Migration offen;
striktes Ziel unverändert.

Assistiver kombinierter Wiederanlauf, 16. November: Vier Zustände mit Windows-Sprachausgabe, Fokus
und nativem Replay-Gegenbeleg bestätigt. Zwei Erstversuche ohne nativen Aufruf wurden korrekt
abgelehnt; reine Tastaturprüfung war damals noch offen. Neue manuelle Helferaktion, App unverändert;
[Nachweis](../../testing/native-combined-assistive-restart.md). Neuer Helferhash benötigt einen
eigenen frischen ETW-Beleg (inzwischen oben verlinkt); keine Übertragung des folgenden älteren
Nachweises. Striktes Ziel unverändert, Produktintegration und weitere Fensterendeprüfung offen.

Kombiniertes ETW, 16. November: Eigener gültiger Nachweis des gesicherten v3-Paars mit automatischer
Serie und echtem nativem Dateidialog in getrennten kalibrierten Phasen. 80 Apps, 480 WebViews und
alle 868 Prozessenden explizit; acht Kontrollen, Nullverluste, 0 Nicht-Loopback- und 480 getrennte
Loopback-Ereignisse. Native Abläufe/Daten/MACs unabhängig geprüft.
[Nachweis](../../testing/native-combined-live-restart-etw.md). Kein Runtimeumbau, keine Übertragung
auf geänderte Binaries, externe Dienste oder das Produkt. Striktes Ziel ohne
Runtime-/Domänenausnahme unverändert; assistive Prüfung damals offen, weitere Fensterendeprüfung
weiterhin offen.

Fortschreibung kombiniert, 16. November: Startticket/Brokervertrag v3 binden Live-Pipes an den
initialen verbrauchten Auftrag. Recovery hat keine Auswahlhandles. Erst privater signierter
Abschluss nach Reconciliation/Replay und ausdrückliche neue native Aktion erlauben den nächsten
unveränderlichen Grant mit neuer ID/Vorgängerhash. Keine Rendererparameter, Netzfähigkeit,
Richtlinien-/DPAPI-Ausnahme oder Produktintegration.
[Vertrag](../../data-format/native-broker-recovery-v3.md),
[native Serie und manuelle Auswahl](../../testing/native-combined-live-restart.md) bestanden.
Shell-Aufruf nutzt nach nativ reproduziertem Fehler eine gleichwertige lokale Pfaddarstellung mit
identischem kanonischem Ziel; Scopeprüfung bleibt. Eigener ETW-Beleg inzwischen oben vorhanden,
neuer Hörnachweis noch offen; striktes Ziel unverändert.

HMAC-ETW 16. November: Neues gesichertes Paar mit eigenem gültigem Netzbeleg des gesamten
Serienbaums, einschließlich 78 negativer Brokerstarts. 103 Broker/30 Apps/180 WebViews, alle 446
Prozessenden explizit, vier Kontrollen, Nullverluste, 0 Nicht-Loopback- und 180 getrennte
Loopback-Ereignisse. Native Abläufe und Daten/MACs unabhängig geprüft.
[Nachweis](../../testing/native-hmac-broker-etw.md). Keine Übertragung auf externe Dienste oder
geänderte Programme, keine Produktfreigabe. DPAPI-Primitive entfernt statt Domänenausnahme; striktes
Ziel unverändert.

Fortschreibung HMAC, 16. November: Die DPAPI-Primitive ist im aktuellen Forschungsbroker entfernt.
Lokaler Schlüssel aus dem fest betrachteten `ProcessPrng`-Backend und RustCrypto-HMAC-SHA-256
ersetzen die kontogeschützte Authority;
[Vertrag v2](../../data-format/native-broker-recovery-v2.md). Die private DACL übernimmt die
Schlüsselverwahrung; keine Verschlüsselung bei ruhendem Datenträger oder zusätzliche
Kontogeheimnis-/Hardwarebindung behaupten. Fehlende/falsche Schlüssel sperren; keine Migration,
Domänen-/OS-Ausnahme oder Produktfreigabe. Neue native Serie mit 78 Ablehnungen bestanden;
[Nachweis](../../testing/native-hmac-broker.md). Eigener ETW-Beleg oben separat dokumentiert. Die
folgenden DPAPI-Absätze bezeichnen den historischen verworfenen Zwischenstand.

Produktintegration des DPAPI-basierten Forschungsbrokers bleibt zusätzlich gesperrt, solange
mögliche Domänencontroller-/RPC-Pfade nicht unter dem strikten Ziel belegt oder die Primitive
ersetzt sind. Keine OS-/Domänenrichtlinienänderung und keine stillschweigende Metadatenausnahme;
[Vertrag und Primärquelle](../../data-format/native-broker-recovery-v1.md).

Controller-Fortschreibung 16. November: Expliziter Forschungsbroker darf nach Controllerverlust
bestehende synthetische Ablage ausschließlich nach DPAPI-/Programm-/Wurzelprüfung, exklusiver Sperre
und Jobleerstand übernehmen. Kein neuer Importgrant. Geschützte native Kontrollablage und
Jobzuweisung bei Prozesserzeugung; Containerzugriffe unter dem tatsächlichen Token verweigert.
[Vertrag](../../data-format/native-broker-recovery-v1.md),
[native Serie](../../testing/native-controller-crash.md): sechs Controller-Kills, zwölf neue
Wiederanlaufprozesse, 30 App-Generationen, 60 Ablehnungen und unabhängige Datenprüfung bestanden.
Kein Produktbroker, kein neuer ETW-Beleg; Produktintegration und externe Grenzen weiter offen.
Striktes Ziel unverändert.

ETW v2 am 16. November: Neuer gültiger Nachweis exakt des unveränderten gesicherten v2-Paars,
einschließlich beider manueller Wiederanlaufpfade und automatischer Serie. 32 App-Generationen, 168
WebViews, vier Kontrollen, Nullverluste, 0 Nicht-Loopback-Ereignisse; 176 Loopback-Ereignisse
separat. Native Abläufe, Daten und Cleanup unabhängig gegengeprüft.
[Nachweis und Grenzen](../../testing/native-assistive-restart-etw.md). Keine Runtimeänderung oder
Erweiterung auf externe Dienste; Controller-Crash/dauerhafter Broker und Produktabnahme offen.

Assistive Fortschreibung 16. November: Startticket v2 ergänzt das verpflichtende native Flag
`automatic`. Nur für Wiederherstellung darf es false sein; dann keine Skriptbedienung und höchstens
600 Sekunden manuelle Betrachtung ohne Importgrant. Gleiche SID-/Hash-/Launch-/Pipe- Bindungen, null
Capabilities. [Vertrag](../../data-format/native-ui-restart-v2.md),
[Hörprüfung und Grenzen](../../testing/native-assistive-restart.md). v1-Tickets werden abgewiesen;
der alte ETW-Beleg wird nicht auf neue Binaries übertragen. Striktes Ziel bleibt unverändert.

ETW-Fortschreibung 16. November: Frischer gültiger Nachweis exakt des unveränderten gesicherten
Neustart-Paars. 28 App-Generationen mit vollständiger Endezuordnung, 144 WebViews, vier Kontrollen,
Nullverluste und 0 Ereignisse außerhalb Loopback; 144 Loopback-Ereignisse separat. Native Serie und
Daten/Cleanup im Messlauf unabhängig geprüft. [Nachweis](../../testing/native-ui-restart-etw.md),
[Hashbindung und Aggregate](../../testing/native-ui-restart-etw.json). Keine Erweiterung der
Vertrauensgrenze auf externe COM-/Shell-/DNS-/Broker oder kompromittierte Prozesse. Striktes Ziel,
fehlende Produktabnahme und offene Controller-Crash-/Dauerbrokerintegration bleiben unverändert.

Neustart-Fortschreibung 16. November: Gebundene Starttickets über frische anonyme Pipes ermöglichen
den Neustart derselben Forschungs-GUI mit unveränderter Identität, Ablage und Programmhash innerhalb
eines lebenden Controllers. Suspendierte Root-Tokenprüfung und empfangsseitige SID-/Hash-/Launch-
Prüfung vor WebView-Aufbau; keine neuen Capabilities oder Rendererbefehle. Finale Serie mit 28
Generationen, sechs Crashpunkten, zweifachem Wiederanlauf, Altimportabwehr und vier negativen Starts
bestanden. [Vertrag](../../data-format/native-ui-restart-v1.md),
[Nachweis und Buildbindung](../../testing/native-ui-restart.md). Controllerabsturz/dauerhafter
Produktbroker und neuer ETW-Beleg dieses Paars bleiben offen; striktes Ziel unverändert.

Auftrags-Fortschreibung 16. November: Das Forschungsfeature bindet einen privaten, an die lokale
Projektwurzel gebundenen SQLite-Auftragsstore vor Auswahl/Lesen ein. Dauerhafter Verbrauch,
exklusive Sperre, eigene Zwischenverzeichnisse und persistierter Commit-Receipt erlauben
konservative Wiederanlaufzuordnung ohne erneutes Lesen oder automatische Veröffentlichung. Keine
neuen Capabilities, Hostordnerfreigaben, Rendererparameter oder Produktabhängigkeiten.
[Vertrag und Bedrohungsgrenzen](../../data-format/native-task-journal-v1.md),
[Windows-Prozess-Kills und native Rückprüfung](../../testing/native-task-reconciliation.md).
Integrierter isolierter GUI-Neustart und neuer ETW-Nachweis dieses Builds fehlen; striktes Ziel und
offene Produktabnahme unverändert.

Assistive Fortschreibung 16. November: Erfolg-/Fehlerpfade der Forschungsseite mit Replay und Fokus
nativ bestätigt. Eine auf feste Enum-/Bool-Werte begrenzte lokale Diagnose ergänzt die
Forschungsbefehle ohne neue Dateifreigaben, Netzwerkrechte oder Produktintegration.
[Nachweis, Grenzen und Buildbindung](../../testing/native-live-assistive-focus.md). Frühere
Fokusabweichungen gelten nicht als bestanden. Kein neuer ETW-Beleg dieses geänderten App-Builds;
striktes Datenschutzziel, offene Brokergrenzen und fehlende Produktabnahme bleiben unverändert.

Fortschreibung 16. November: Der getrennte Live-Auswahlprototyp beendet bei Fensterende auch offenen
Dateidialog und nachgewiesenen Teiltransfer. Schließnachrichten sind auf eigenen Prozess, eigenen
Auswahlthread und Dialogklasse beschränkt; Pipe-Duplikate werden geschlossen und der Threadabschluss
wird begrenzt abgewartet. [Nachweise und Buildgrenzen](../../testing/native-live-window-close.md).
Keine Produktintegration oder neue Netzwerkfähigkeit. Statt Übertragung des vorherigen Netzbelegs
besteht nun ein
[frischer ETW-Vergleich dieses unveränderten Builds](../../testing/native-live-window-close-etw.json):
drei vollständige Bäume, vier Kontrollen, Nullverluste, 0 Ereignisse außerhalb Loopback. Striktes
Ziel unverändert; Produkt-Q-01/Q-02 und externe Brokergrenzen bleiben offen.

Status: **Untersuchungsrichtung A ausdrücklich beauftragt am 15. November 2026.** Nutzerauftrag:
technische Laufzeitisolation, „Das strikte Datenschutzziel bleibt bestehen.“ KQ-069 / Issue #134.
Keine Produktarchitektur- oder Netzfreigabe und keine Änderung des Pflichtenhefts. B gehört nicht
zum freigegebenen Scope.

Der [Machbarkeitsversuch](../../testing/native-runtime-isolation.md) ergänzt einen getrennten
AppContainer-Forschungslauncher und gültige ETW-Vergleiche. Lokal schreiben, neu starten und
HTML/PDF exportieren funktioniert; der Dateidialog scheitert. Die 0 beobachteten Ereignisse
außerhalb Loopback im Prototyp ändern die fehlende Isolation des normalen Starts nicht.

Der anschließende [Datei-/Shell-Transportversuch](../../testing/native-runtime-file-exchange.md)
belegt native Auswahl im separaten Hostcontroller und begrenzten Bytetransport über zwei explizite
Handles in einen Worker ohne Capabilities. Der Hostordner erhält keine Paket-ACL; nicht ausgewählte
Dateien bleiben über ihren Pfad unlesbar. Kein beliebiger Shell-Aufruf oder Produktbroker
freigegeben. Der gültige ETW-Ausschnitt erfasst keine Netzereignisse im Controller-/Workerbaum;
außerhalb laufende COM-/Shell-Dienste sind damit nicht abgenommen. Der spätere finale Cleanup-Stand
ist separat nativ geprüft. Nächster Scope ist ein typisierter einmaliger Dateiauftrag an einen
isolierten App-Testadapter mit konkretem atomarem Import; die strikte Zielsetzung bleibt bestehen.

## Belegter Konflikt

Die [separate Tauri-Testanbindung](../../testing/native-tauri-file-channel.md) ergänzt ein echtes
Forschungsfenster mit parameterlosen Commands, strikter Origin-/Fensterprüfung und vorab delegiertem
Einwegkanal. Kernelhandles statt Rendererpfaden oder öffentlichem RPC-Endpunkt; Flags vor
WebView-Start gelöscht/geprüft. Keine neue Hostordner- oder Netzfähigkeit. Drei finale native
Sitzungen prüfen Import/Wiederholung, Abbruch und Fensterende am Haltepunkt; SQLite und Cleanup
unabhängig bestätigt. Gültiges ETW desselben Builds: vollständige Bäume, vier Kontrollen,
Nullverluste, 0 Ereignisse außerhalb Loopback. Keine universelle Netzstille oder vollständige
Nachfahren-/Brokerattestierung. Vorabauswahl, Auswahl aus laufender App, dauerhafte Aufträge/
Crash-Reconciliation und Produktintegration offen. Striktes Ziel unverändert, keine Ausnahme.

Der [native App-Dienst-Testadapter](../../testing/native-isolated-app-import.md) bindet einen
einmaligen Snapshotauftrag an den isolierten Worker. Parser und atomare Veröffentlichung nutzen die
vorhandenen Rust-Dienste; Ablauf/Abbruch zusätzlich nach vollständigem Aufbau geprüft. ETW des
finalen Builds: gültige Kontrollen, keine Netzereignisse in acht Controller-/Workerbäumen. **Keine
Tauri-/WebView-/GUI-Anbindung und keine reale Nutzdateiauswahl.** UUID/Ablaufzeit ersetzen keinen
produktiven authentisierten Auftragskanal. Nach Commitbeginn keine Rücknahmegarantie;
Crash-Reconciliation, externe COM-/Shell-Dienste und Produktintegration offen. Keine neue
Netzfähigkeit oder fachliche Ausnahme.

Pflichtenheft Revision 1 verlangt Offlinefähigkeit, keine Telemetrie ab Werk und keine Übertragung
von Manuskripttexten, Bildern oder Metadaten bei Cloud-Opt-out (§1, §5, §10). KQ-068 misst
öffentliche WebView2-Verbindungen. KQ-069 reproduziert Konfigurations- und Komponentenupdate-Starts
bereits ohne Projekte. Die unterstützten betrachteten APIs ergeben keinen belegten universellen
Ausschalter. Siehe [Versuch, Quellen und Grenzen](../../testing/native-m3-webview-network.md).

Eine funktionierende Offline-App und fehlender eigener Telemetriecode erfüllen noch keine
vollständige Windows-Laufzeit-Netzstille. Wir nehmen weder die Unbedenklichkeit aller Bodies noch
eine garantierte Übertragung von Projektdaten an. Q-01/Q-02 bleiben nicht bestanden.

## Beauftragte Richtung und nicht freigegebene Alternative

**A – Striktes Ziel beibehalten; technische Isolation untersuchen (beauftragt).** Nächster
begrenzter Scope: unterstützte, durch die Anwendung durchsetzbare Windows-Isolation für Rust und
WebView2 einschließlich Prozessnachfahren prüfen. Zunächst Machbarkeit und reproduzierbarer lokaler
Prototyp, kein ungeprüfter Umbau des produktiven Launchers. Abnahme mit unabhängiger
ETW-Prozesszuordnung und positiven Kontrollen, vollständigen Prozessenden, CRUD, Export/Neuimport
und Neustart. Lokale Ressourcen, Tauri-IPC, Barrierearmut, Installation und sicherer
Runtime-Updatepfad dürfen nicht stillschweigend ausfallen. Falls diese Plattform das Ziel nicht
trägt, bleibt auch ein Plattformwechsel als gesonderter größerer Entscheid offen.

**B – Fachliche Laufzeitgrenze ausdrücklich ändern.** Eine mögliche Ausnahme würde ausschließlich
konkret belegte Runtime-Konfigurations- und Komponentenmetadaten betreffen. Vor Annahme müssen
erlaubte Daten, Zielklassen, Nutzerinformation und überprüfbare Grenzen präzise definiert und
Pflichtenheft, Abnahmematrix und Datenschutzaussagen gemeinsam geändert werden. Die aktuelle Messung
beweist keine zulässigen Body-Inhalte und reicht daher allein nicht zur Freigabe dieser Ausnahme.
Keine pauschale Ausnahme für Manuskripte, Bilder, Prompt-Kontext oder beliebige Telemetrie. Das wäre
eine Produktentscheidung, kein technischer Bugfix; sie ist bisher nicht autorisiert.

## Leitplanken der Untersuchung

Keine neuen Abschaltflags, Firewall-/OS-Datenschutzänderungen, TLS-/Sandbox-Abschaltung oder
automatische Providerfreigabe. Unterstütztes Custom-Crash-Reporting ist ein separater, begrenzter
Mechanismus und löst die beobachteten Routen nicht. Diagnoselogs gehören nicht in Releases. Issue
#134 bleibt offen. A ist ein Forschungs- und Integrationsauftrag, keine Aussage, dass bereits eine
vollständige tragfähige Produktisolation existiert. Datei-/Shell-Broker, kontrollierter Austausch,
Git-/Credential-/Providerpfade, Prozess-/Profil-Lebensdauer und Installer-/Updatepfade müssen die
strikte Grenze erhalten; zusätzliche Internetfähigkeiten für den WebView2-Baum sind kein Fix.
