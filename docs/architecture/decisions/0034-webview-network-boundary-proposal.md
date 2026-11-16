# ADR 0034: Windows-Laufzeitgrenze nach WebView2-Netzbefund

Fortschreibung 16. November: Der getrennte Live-Auswahlprototyp beendet bei Fensterende auch offenen
Dateidialog und nachgewiesenen Teiltransfer. Schließnachrichten sind auf eigenen Prozess, eigenen
Auswahlthread und Dialogklasse beschränkt; Pipe-Duplikate werden geschlossen und der Threadabschluss
wird begrenzt abgewartet. [Nachweise und Buildgrenzen](../../testing/native-live-window-close.md).
Keine Produktintegration oder neue Netzwerkfähigkeit. Der vorherige ETW-Nachweis wird nicht auf
diesen geänderten Binärstand übertragen. Striktes Ziel unverändert; Produkt-Q-01/Q-02 bleiben offen.

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
