# Rechnerwechsel und Arbeitsübergabe

Diese Anleitung trennt den öffentlichen Quellcode vom privaten Schreibprojekt. GitHub transportiert
Code und Projektdokumentation; ein `.kraqel`-Archiv transportiert Manuskript, Struktur und lokale
Medien. Wer beides verwechselt, erhält entweder fehlende Kapitel oder einen sehr literarischen
Commitverlauf.

## Aktuelle Entwicklungsübergabe · 15. November 2026

Neuester Stand 16. November: Kombinierter Forschungsablauf v3 mit echter Live-Auswahl, Wiederanlauf
und ausdrücklich neuem Folgeauftrag bestanden. Signierte Grantkette und private Abschlussbelege
ersetzen keine neue menschliche/native Beauftragung.
[Verträge und finale Belege](testing/native-combined-live-restart.md). Neun automatische Fälle, 72
App-Starts, 54 verweigerte Folgeaktionen; manueller Dialoglauf bestätigt und vollständig
weitergeprüft. Beide Vorläufersitzungen und fehlgeschlagener Dialoglauf stillgelegt; finale Profile
entfernt, Prozesse beendet. Normale Debug-App ohne Forschungsseite wiederhergestellt. Gesichertes
neues Paar: `%TEMP%\\kraqel-kq069-combined-build-20261116-04`, vollständige Hashes im Aggregat. Alte
Binaries/Nachweise nicht übertragen. Nächster Scope: frisches ETW genau dieses v3-Paars, assistive
und weitere Fensterendeprüfung des kombinierten Ablaufs. Produktablage, authentisierte
Updates/Migration und externe Dienste bleiben offen; striktes Ziel unverändert.

Neuester Nachweis 16. November: Frisches ETW des gesicherten HMAC-Paars bestanden, einschließlich
aller negativen Brokerstarts. 103 Broker/30 Apps/180 WebViews/133 Console-Hosts, 446 explizite
Enden, vier Kontrollen, Nullverluste, 0 Nicht-Loopback- und 180 getrennte Loopback-Ereignisse.
Native Serie, unabhängige MACs/Daten und PID-Wiederverwendung geprüft.
[Nachweis/Buildgrenzen](testing/native-hmac-broker-etw.md). Erfassung/Prozesse beendet. Nächster
Scope: kombinierter Live-/Neustartpfad, geschützte Produktablage und authentisierter Update-/
Migrationspfad; externe Dienste/Produktintegration weiter offen. Striktes Ziel unverändert.

Aktuell 16. November: Broker-Vertrag [v2](data-format/native-broker-recovery-v2.md) entfernt DPAPI
und authentisiert den dauerhaften Beleg lokal mit HMAC-SHA-256. Schlüssel nur unter privater DACL,
keine Verschlüsselung/Hardwarebindung behaupten; keine automatische Schlüsselneuanlage oder
Altdatenmigration. [Funktionsnachweis](testing/native-hmac-broker.md): sechs harte Controllerenden,
zwölf Wiederanlaufprozesse, 30 App-Generationen, 78 Ablehnungen und unabhängige Daten-/MAC-Prüfung
bestanden. Neues Paar separat unter `%TEMP%\kraqel-kq069-hmac-broker-build-20261116-01` gesichert;
eigener ETW-Beleg ist oben verlinkt. Alte Paare und Belege nicht übertragen. Produktintegration und
externe Grenzen bleiben offen, striktes Ziel unverändert. Historische Übergaben folgen.

Historischer DPAPI-Produktblocker: DPAPI-Domänencontroller-/RPC-Grenze gesondert prüfen oder
Primitive ersetzen. Keine Produktintegration allein aufgrund eines unauffälligen ETW-Appbaums;
[Vertrag](data-format/native-broker-recovery-v1.md). Striktes Ziel unverändert.

Neuester Stand 16. November: Controller-Crash-Serie und dauerhafte native Wiederanlaufautorität im
Forschungslauncher bestanden: sechs harte Controllerenden, zwölf neue Broker-Wiederanläufe, 30
App-Generationen, 60 Ablehnungsfälle. [Vertrag](data-format/native-broker-recovery-v1.md),
[Nachweis und gesicherte Hashes](testing/native-controller-crash.md). Daten unabhängig geprüft,
Profile entfernt, Prozesse beendet. Gesichertes Paar lokal unter
`%TEMP%\\kraqel-kq069-controller-broker-build-20261116-03`; alte Paare unverändert erhalten.
Nächster Scope: eigener ETW-Netzvergleich dieses neuen Paars, danach kombinierter Live-/Neustartpfad
und Produktbrokerintegration. Alter ETW-Beleg ist keine Freigabe des neuen Brokers. Produktstart
weiter unisoliert; striktes Datenschutzziel und offene Produkt-Q-01/Q-02 unverändert. Frühere
Übergabeschritte folgen.

Neuester Nachweis 16. November: Frisches ETW exakt des gesicherten v2-Paars bestanden,
einschließlich automatischer 28-Start-Serie und beider manueller Wiederanlaufpfade. Drei
Controllerbäume mit 32 Apps/168 WebViews, alle 238 Lebenszeiten explizit beendet; vier Kontrollen,
Nullverluste, 0 Nicht-Loopback- und 176 getrennte Loopback-Ereignisse. Daten/Cleanup unabhängig
geprüft. [Nachweis](testing/native-assistive-restart-etw.md),
[Aggregate/Hashes](testing/native-assistive-restart-etw.json). Programme unverändert,
Erfassung/Prozesse beendet, Rohdaten lokal. Nächster Scope: Controller-Crash und dauerhafter
authentisierter Broker, danach kombinierter Live-/Neustartpfad und externe Grenzen. Keine
Produktfreigabe; striktes Ziel unverändert.

Neuester Stand 16. November: Assistive Wiederanlaufprüfung mit Windows-Sprachausgabe für
Unterbrechung/Abschluss/Replay bestanden; zusätzliche Tastaturnutzung menschlich bestätigt, mit
begrenzter technischer Eingabequellen-Zuordnung.
[Startticket v2](data-format/native-ui-restart-v2.md) mit `automatic: false` lässt nur
Wiederherstellung manuell offen, ohne neue Rechte/Importquelle.
[Nachweis, Wiederholung und Hashes](testing/native-assistive-restart.md). 28 automatische v2-Starts
und lokale Qualitätsprüfungen bestanden. Neues Paar unter
`%TEMP%\kraqel-kq069-assistive-restart-build-20261116-01` gesichert; Hashes vor Verwendung prüfen.
Nächster Scope: frisches ETW genau dieses Paars. Der alte v1-Netzbeleg gilt separat; Controller-
Crash, dauerhafter Broker, externe Grenzen und Produktabnahme bleiben offen. Striktes Ziel gilt.

Neuester Nachweis 16. November: Frischer gültiger ETW-Netzvergleich exakt des gesicherten
Neustart-Paars bestanden. Ein Controllerbaum mit 28 Apps, 144 WebViews und insgesamt 202 explizit
beendeten Lebenszeiten; vier Kontrollen, Nullverluste, 0 Ereignisse außerhalb Loopback und 144
getrennte Loopback-Ereignisse. Auch unter ETW sind Crash/Reconciliation/Altimportabwehr sowie
SQLite/Unicode/Kopienzahl/Cleanup unabhängig bestätigt.
[Nachweis](testing/native-ui-restart-etw.md),
[Aggregate und Hashes](testing/native-ui-restart-etw.json). Rohdaten lokal, Erfassung beendet,
Programme unverändert. Nächster Scope: assistive Wiederanlaufprüfung, Controller-Crash/dauerhafter
Broker und externe Grenzen. Keine Produktfreigabe; striktes Ziel unverändert.

Neuester Stand 16. November: Integrierte isolierte GUI-Crash-/Neustartserie bestanden, 28
Generationen desselben App-/Controllerpaars. Gleicher lebender Controller, OS-Identität und
Datenwurzel; frische Pipes/Launch-Bindungen je Start. Sechs Crashpunkte, je zwei Reconciliations und
Altimportabwehr; vier ungültige Starts verworfen. Genau zwei Kopien, vier unterbrochene Aufträge,
keine Zwischenreste, alle Jobs leer und Profil entfernt.
[Vertrag](data-format/native-ui-restart-v1.md), [Nachweis und Hashes](testing/native-ui-restart.md),
[Aggregate](testing/native-ui-restart.json). Nächster Scope: neuer ETW-Netzvergleich mit exakt dem
gesicherten Paar, danach assistive Wiederanlaufprüfung. Controllerabsturz/dauerhafter Broker und
Produktintegration weiterhin offen; striktes Datenschutzziel unverändert. Frühere Stände folgen zur
historischen Einordnung.

Neueste Fortschreibung 16. November: Dauerhafter Auftragsverbrauch und konservative
Crash-Reconciliation im Forschungsadapter, vor der nativen Auswahl im Tauri-Forschungsfeature
angebunden. Sieben neue Windows-Integrationstests mit sieben echten Prozess-Kills bestehen;
Bestandsdaten bleiben bytegleich, Replay bleibt gesperrt. Native Erfolg-/Abbruchrückprüfung mit
Replay, Journal und Cleanup bestätigt. [Vertrag](data-format/native-task-journal-v1.md),
[Buildbindung und Nachweis](testing/native-task-reconciliation.md). Nächster Scope: authentisierter
Neustart derselben isolierten GUI mit stabiler Ablage und integrierte Crash-Serie; Controller
verlangen bisher frische Forschungswurzeln. Kein neuer ETW-Beleg, keine Produktfreigabe.

Neueste Fortschreibung 16. November: Assistive Erfolg-/Fehlerpfade mit Replay, passenden Ansagen und
Fokus nativ bestätigt. 34 Browser-E2E bestanden; zusätzliche begrenzte Forschungsdiagnose enthält
ausschließlich feste Zustandswerte.
[Nachweis und Buildhash](testing/native-live-assistive-focus.md). Frühere Fokusabweichungen bleiben
dokumentiert; keine assistive Vollabnahme. Der geänderte App-Build hat keinen neuen ETW-Nachweis.
Nächster Scope: dauerhafter Auftragslebenszyklus und Crash-Reconciliation, anschließend weitere
Broker-/Netzabnahme. Striktes Ziel unverändert.

Neuester funktionaler Nachweis 16. November: Fensterende bei offenem Dateidialog und nach echtem
Teiltransfer bestanden. Eigener Prozess/Thread/Dialogklasse vor Schließnachricht geprüft,
Auswahlthread begrenzt beendet; keine Projektkopien oder Stagingreste. Bytezähler belegen 1.024
empfangene von 11.915 erwarteten Bytes.
[Buildgrenzen und Reproduktion](testing/native-live-window-close.md). Frischer ETW-Vergleich am
unveränderten finalen Build bestanden: Referenzimport/Replay, Dialogende und Teiltransfer in drei
vollständigen Bäumen mit 19 WebView2-Lebenszeiten; vier Kontrollen, Nullverluste, 0 Ereignisse
außerhalb Loopback. [Aggregate](testing/native-live-window-close-etw.json). Forschungsprozesse und
Erfassung beendet; Rohdiagnosen bleiben lokal. Nächste Abnahme: Screenreader/Fokus, externe
Brokergrenzen und dauerhafter Auftragslebenszyklus. Striktes Ziel unverändert, keine
Produktfreigabe. Die folgenden Notizen beschreiben frühere Stufen.

Aktualisierung 16. November: Replay-/Fehlerprüfung und Fensterende am Publish-Haltepunkt bestanden.
Neuer gültiger ETW-Nachweis für drei vollständige Forschungs-App-Bäume: 0 Ereignisse außerhalb
Loopback, vier Kontrollen, Nullverluste. Replay-Anzeigefehler korrigiert.
[Exakter Buildbezug und offene Aufgaben](testing/native-live-file-selection.md). Rohdiagnosen
bleiben lokal, ETW-Sitzung beendet. Nach Messung Typalias zur Clippy-Korrektur ohne neuen
Binärnachweis. Striktes Ziel unverändert; nächste Prüfung: Appende bei offenem Dialog/Transfer,
Screenreader/Fokus und externe Brokergrenzen. Keine Produktfreigabe, #134 bleibt offen.

Fortschreibung 16. November: `ui-live` ergänzt die Auswahl aus dem laufenden isolierten
Forschungsfenster. Native Erfolg-/Dialogabbruchserie und unabhängige SQLite-/Cleanup-Prüfung
bestanden. [Nachweis und verbleibende Schritte](testing/native-live-file-selection.md). Replay,
assistive Fokus-/Fehlerprüfung, Appende bei Auswahl/Transfer und neuer ETW-Lauf bleiben offen. Keine
Produktfreigabe; striktes Datenschutzziel unverändert.

Entwicklungsstand: [PR #140](https://github.com/kolkrabeofdoom/kraqel/pull/140), KQ-069 A /
[Issue #134](https://github.com/kolkrabeofdoom/kraqel/issues/134). PR #139 ist nach grüner CI
zusammengeführt (`fadf0a6`). PR #140 enthält die UI-/Tauri- Testanbindung und den einmaligen
Einweg-Dateiauftragskanal einschließlich finaler Nachweise. Die abschließende Übergabe wird mit
diesem PR nach vollständig grüner CI nach `main` übernommen. Beim Wiederbeginn den tatsächlichen
GitHub-Merge-Status prüfen; danach `main` aktualisieren. Falls der PR noch offen ist, liegt die
gesamte Arbeit auf `test/kq-069-isolated-tauri-channel`.

Bestanden: `pnpm check`, normaler und Forschungs-Tauri-Build, Feature-Clippy, 29 Browser-E2E, 18
Helfer-/Transporttests, ein Tauri-Origin-Test und 32 Python-Diagnosetests. Drei finale native
Sitzungen prüfen Import/Wiederholungsablehnung, Abbruch und direktes Fensterschließen am Haltepunkt.
SQLite/Unicode, neue Projekt-ID, Kopienzahl und Cleanup sind unabhängig bestätigt. Gültiges ETW
vom 15. November, 16:43:07–16:46:11 UTC: drei vollständige App-/WebView-Bäume, vier Kontrollen,
Nullverluste, 0 Ereignisse außerhalb Loopback; Loopback-Disconnects getrennt ausgewiesen.

**Das strikte Datenschutzziel bleibt bestehen.** Normale App-Starts sind weiterhin unisoliert;
Q-01/Q-02 für das Produkt sind nicht bestanden. Nur feste synthetische Vorabfreigaben sind im
Forschungsfeature geprüft. Keine allgemeine Freigabe realer Manuskripte, externer Broker oder
kompromittierter Runtime-Prozesse; #134 bleibt offen.

Nachtrag 16. November: Native Vorab-Dateiauswahl (`ui-select`) und Dialogabbruch sind auf dem
zurückgewechselten Rechner funktional geprüft; der neue Build, Controllerausgabe und unabhängige
SQLite-/Cleanup-Prüfung sind im [Nachweis](testing/native-ui-select-acceptance.md) dokumentiert.
Kein neuer ETW-Nachweis und keine Produktfreigabe. Nächster Scope: Auswahl aus einer laufenden App.

Historischer Ausgangspunkt der Übergabe: native Vorab-Dateiauswahl und deren Abbruch prüfen. Die
erfolgreiche finale UI-Serie verwendete `ui-fixture`; der vorherige unvollständige Auswahldialoglauf
ist kein Abnahmenachweis. Danach Auswahl aus einer laufenden App, assistive Abnahme und dauerhafte
Auftrags-/Replay-/Crash-Reconciliation. Weitere Formate, Git-/Credential-/Providergrenzen,
öffentliche UDP-/IPv6-/DNS-/Broker-Gegenproben und Produkt-/Installerintegration folgen separat.

GitHub zeigt außerdem drei bestehende, noch offene Dependabot-Meldungen:
[`lopdf` · hoch · #8](https://github.com/kolkrabeofdoom/kraqel/security/dependabot/8),
[`glib` · mittel · #7](https://github.com/kolkrabeofdoom/kraqel/security/dependabot/7) und
[`minimatch` · mittel · #6](https://github.com/kolkrabeofdoom/kraqel/security/dependabot/6). Sie
wurden durch die UI-/Isolationsarbeit nicht behoben; Erreichbarkeit und Updates separat prüfen.

Für die Fortsetzung zuerst lesen:

1. `AGENTS.md`, [Entwicklungsstatus](DEVELOPMENT_STATUS.md), [Roadmap](../ROADMAP.md) und #134.
2. [ADR 0034](architecture/decisions/0034-webview-network-boundary-proposal.md) und
   [Bedrohungsmodell](architecture/threat-model.md).
3. [Kanalvertrag](data-format/native-file-channel-v1.md),
   [native UI-/ETW-Nachweise und Buildanleitung](testing/native-tauri-file-channel.md) sowie
   [Toolanleitung](../scripts/runtime-isolation/README.md).

Der Quellcode einschließlich synthetischer `.kraqel`-Fixture ist vollständig versioniert. Builds,
temporäre Testprojekte, WebView-Profile, lokale PID-Journale und rohe ETW/XML-Dateien bleiben auf
diesem Rechner. Auf dem anderen Rechner neue TEMP-Wurzeln, neue Builds/Hashes, neue Journale und bei
Bedarf einen neuen ETW-Lauf erzeugen. Historische JSON-Aggregate sind Nachweise dieses Rechners,
keine lokale Abnahme des anderen Rechners. Alte absolute TEMP-/Workspacepfade nicht übernehmen. Es
gibt keine erforderlichen, nur in einem lokalen `work`-Ordner liegenden Quelldateien.

Hier geprüft: Node.js 26.13.1, pnpm 11.25.0, Rust/Cargo 1.100.0, Windows/WebView2. Maßgeblich für
die Installation bleiben `packageManager`, `engines` und `rust-version` im Repository. Das
Isolationsfeature benötigt Windows; die übrige CI prüft auch Linux. Computer Use war hier mit
`Transport closed` ausgefallen, daher erfolgte die native Fensterbedienung durch den Nutzer.
Verfügbarkeit auf dem anderen Rechner neu prüfen, keine alternative Windows-Automation annehmen.

## Diesen Rechner sauber verlassen

1. Nur auf einem geprüften, sauberen `main` enden:

   ```powershell
   cd D:\kraqel
   git switch main
   git pull --ff-only
   git status --short
   git log -1 --oneline
   ```

2. Wenn zusätzlich Schreibprojekte wechseln sollen, in KRAQEL unter **Versionen** zuerst einen
   Snapshot erzeugen und danach **Projektarchiv exportieren** wählen. Das erzeugte `.kraqel`-Archiv
   an einem privaten, vertrauenswürdigen Ort ablegen. Manuskripte und API-Schlüssel gehören nicht
   ins öffentliche Repository.
3. Optional die Übertragung mit `Get-FileHash <Archiv.kraqel> -Algorithm SHA256` auf beiden Rechnern
   vergleichen.

## Auf dem anderen Rechner beginnen

Für einen neuen Checkout:

```powershell
git clone https://github.com/kolkrabeofdoom/kraqel.git D:\kraqel
cd D:\kraqel
corepack enable
pnpm install --frozen-lockfile
git status --short
pnpm --filter @kraqel/desktop typecheck
```

Für einen vorhandenen Checkout:

```powershell
cd D:\kraqel
git switch main
git pull --ff-only
pnpm install --frozen-lockfile
git status --short
```

Benötigt werden Git, Node.js 22 oder neuer, Corepack/pnpm gemäß `packageManager`, Rust 1.90 oder
neuer sowie unter Windows WebView2. Danach `AGENTS.md`, `docs/DEVELOPMENT_STATUS.md`, das nächste
GitHub-Issue und den zugehörigen ADR lesen.

Das Schreibprojekt wird auf der Startseite über **Projektarchiv importieren** als neue lokale Kopie
eingelesen. Die laufende Arbeitsdatenbank liegt unter `%LOCALAPPDATA%\org.kraqel.desktop\projects`
und wird bewusst nicht von Git synchronisiert. Cloud-API-Schlüssel liegen im
Betriebssystem-Schlüsselspeicher und müssen auf dem zweiten Rechner erneut eingerichtet werden.

## Verbindliches Arbeitsmuster

Ein Issue pro klarer Änderung, ein kurzer Branch, ein Pull Request mit grüner CI, danach Merge nach
`main`. Am Sitzungsende: Statusdokumente aktualisieren, offene manuelle Prüfungen ausdrücklich als
offen markieren, pushen und `git status --short` auf leer prüfen. So bleibt die Entwicklung
ortsunabhängig, ohne eine mystische Festplatte zum einzigen Wahrheitsministerium zu ernennen.
