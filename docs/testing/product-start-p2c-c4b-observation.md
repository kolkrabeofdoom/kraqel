# P2c C4b: native Generationen, Rollen- und Modulbeobachtung

Stand: 18. November 2026. **Begrenzter Forschungsnachweis bestanden; keine produktive
Paket-/Rollenfreigabe.** [Bereinigte Aggregate](product-start-p2c-c4b-observation.json),
[vorheriger Bootstrapnachweis](product-start-p2c-c4b-root-bootstrap.md),
[C4-Vertrag](../data-format/product-start-p2c-c4-v1-draft.md).

## Änderung und Grenzen

Der
[leere Runtimepfad](../../scripts/runtime-isolation/probe/src/windows/exchange/ui_restart/durable/product/workspace_persistent/c4_preflight/root_bootstrap/runtime.rs)
fragt `GetProcessInfos` während derselben zweisekündigen `about:blank`-Phase wiederholt ab, ungefähr
alle 100 ms und einmal abschließend. Rollen werden über native PID/Erzeugungszeit zusammengeführt,
maximal 32 Generationen; unbekannte API-Kinds, widersprüchliche Zuordnungen und nicht mehr
abfragbare/beendete Generationen brechen diesen Forschungsversuch ab. Eine bloße wiederverwendete
PID ersetzt keine Generation. Die API-Zuordnung bleibt eine Laufzeitbehauptung, keine vor Ausführung
wirksame Rollenautorität; auch der Zeitraum zwischen API-Abfrage und Prozessöffnung ist kein
atomarer Beweis.

Der
[Controller](../../scripts/runtime-isolation/probe/src/windows/exchange/ui_restart/durable/product/workspace_persistent/c4_preflight/root_bootstrap/controller.rs)
erfasst zusätzlich native
[ToolHelp-Modulaufnahmen](../../scripts/runtime-isolation/probe/src/windows/exchange/ui_restart/durable/product/workspace_persistent/c4_preflight/root_bootstrap/controller/modules.rs)
an den bereits gehaltenen Jobgenerationen, ungefähr alle 100 ms während der Ergebniserwartung.
Maximal 256 Versuche und 512 Modulpfade je Generation, drei Snapshotversuche bei `ERROR_BAD_LENGTH`.
Lebender gehaltener Prozess vor/nach dem PID-adressierten Snapshot; Fehler bleiben ausdrücklich im
lokalen Ergebnis. Unvollständige/fehlgeschlagene Aufnahmen gelten nicht als leeres sicheres
Inventar. Die Aufruf-/Mengengrenzen sind keine garantierte maximale Laufzeit eines
Windows-API-Aufrufs.

Diese Beobachtung gewährt keine neue Autorität und prüft **keine vollständige Bindung geladener
Modulbytes**. Pfadlisten erfassen weder sämtliche kurzlebigen DLLs noch alle Ladearten; Microsoft
nennt insbesondere Loaderänderungen und `LOAD_LIBRARY_AS_DATAFILE` als Grenzen der
[Snapshot-API](https://learn.microsoft.com/en-us/windows/win32/api/tlhelp32/nf-tlhelp32-createtoolhelp32snapshot).
Keine Präfix-/Hersteller-/Signaturfreigabe aus Diagnosepfaden ableiten.

## Frische native Ergebnisse

Roothelfer SHA-256 `0ce6b48ff22af39624db14449d943562cb229835500b9ed1bab9214feeb48a42`,
Testcontroller `39015362731ed5579de304fea8fdd7cce1a1281bf4e6a717dd51f2d1cc080767`. Unveränderte
Fixed-Version-Runtime 157.0.4341.60/x64; ausschließlich neue synthetische Kandidaten-, Kontroll- und
Datenfixtures. Normaler Produktstart, echte Vorschauablage und gemeinsamer Transport unverändert.
Keine realen Projekte, Provider oder Tauri-Commands.

Ein funktionaler Lauf und eine gezielte Wiederholung unter ETW liefern jeweils:

- Falsche Grant-MAC vor Accepted/Continue abgewiesen, Root-Ende 2; gültiger Root authentisiert.
- Sieben native Jobgenerationen: ein Root und sechs Runtimeprozesse, alle beobachtet und im
  AppContainer ohne Capabilities. Gepinnte Images, sechs Elternbezüge und alle gehaltenen
  Prozesshandles vor Drained beendet; kontrollierter Finish, keine verbleibende Probe.
- 21 API-Aufnahmen; fünf exakt mit nativer Erzeugungszeit zugeordnete Runtimegenerationen: Browser,
  Renderer, GPU und zwei Utilityprozesse. Eine Runtimegeneration ohne API-Kind.
- 126 erfolgreiche Modulaufnahmen, 294 verschiedene Prozess-/Modulpfadpaare, keine Diagnosefehler.
  Dies ist weder die Zahl verschiedener DLLs noch eine vollständige Ladehistorie.
- Unabhängig je 14 MAC-Records mit zehn C4-Proberecords, zwei verbrauchten Launches, zwei
  Rootbindungen und den zulässigen Recordfolgen geprüft. Wiederöffnen bestätigt keinen Pending-
  Auftrag. Eigene Daten-/Kontrollfixtures entfernt; beide Testprofile unabhängig als abwesend
  geprüft.

Die
[WebView-API](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/icorewebview2environment8)
schließt Crashpad ausdrücklich aus. Die verbliebene native Generation wird deshalb **nicht** bloß
aufgrund fehlender API-Zuordnung als Crashpad freigegeben. Die zwei unzugeordneten Generationen des
vorherigen Builds sind damit nicht rückwirkend aufgeklärt; Prozesszahlen variieren zwischen Läufen.

Zwei beobachtete Modulpfade liegen außerhalb von Windows-Verzeichnis und Kandidaten-UI:
Verhaltensüberwachung im Root sowie ein HEVC-Decoder in einem Runtimeprozess. Die nach dem Lauf an
diesen lokalen Pfaden geprüften Dateien sind gültig signiert. Diese Herkunftsdiagnose belegt keine
gehaltene Identität der tatsächlich geladenen Bytes. Exakte Pfade, Herstellerdetails und Dateihashes
bleiben im geschützten lokalen Befund. Daraus entsteht keine zusätzliche
Modul-/Medien-/Sicherheitssoftware-Ausnahme; Schutzsoftware und Systemeinstellungen bleiben
unverändert.

## Frischer gezielter ETW-Vergleich

Unveränderte [Erfassung](../../scripts/m3-network/capture.ps1) und
[Auswertung](../../scripts/m3-network/analyze.py), Modus `native-control`, explizite zulässige
hashbenannte EXE-Kopien beider Programme. Hashvergleich, Start-PID/Erzeugungszeit und gehaltene
Hostprozesshandles bis nach Konvertierung; keine automatische Rootsuche oder gelockerten Regeln. Nur
die Erfassung mit erhöhtem Token, keine Policy-/Firewall-/Loopbackänderung.

| Phase                                               | Hostaufrufe | Roothelfer | Runtime | Konsolenhosts | Explizite Enden | Kontrollen | Netzereignisse |
| --------------------------------------------------- | ----------: | ---------: | ------: | ------------: | --------------: | ---------: | -------------: |
| Referenz: fehlender Bootstrap und reiner Rollentest |           2 |          0 |       0 |             2 |               4 |        4/4 |              0 |
| Falsche MAC und gültiger leerer Runtimepfad         |           1 |          2 |       6 |             1 |              10 |        4/4 |              0 |

Der erste Hostaufruf der Referenz ist der direkt gestartete Roothelfer ohne Bootstrap; er endet
mit 2. Die Runtimephase prüft denselben negativen MAC-Pfad und anschließend einen gültigen Root,
keine Crash-/Recoverymatrix. Beide Erfassungen gültig: null verlorene Events/Buffers, keine
Zuordnungslücken oder unbekannten Ziele. Alle 14 Enden explizit; alle drei Hoststartzeiten und acht
native Root-/Runtimegenerationen mit Kernelzeiten auf 100 ns sowie Elternbezügen abgeglichen. **Kein
beobachteter Netzwerkverkehr einschließlich Loopback in diesen Prozessbäumen.** Keine universelle
Netzfreiheit oder Zuordnung fremder COM-/Shell-/Sicherheitsdienstprozesse daraus. Alte C3-/C4b-Bytes
und Nachweise bleiben getrennt.

## Prüfungen, Sicherung und nächster Scope

117 aktive Helfer-/Transporttests bestanden (110 Binär-/sieben Transporttests), sechs standardmäßig
ignoriert. Zwei neue aktive Tests für Generations-/Rollenkonflikte und native Moduldiagnosen;
lokaler ignorierter Rootfall funktional und einmal gemessen ausgeführt. Clippy
`--all-targets -- -D warnings`, rustfmt und Dokumentprüfung. Mittleres Prüfniveau; kein
App-/Transport-/Installerumbau und kein vollständiger Appbuild.

Lokale geschützte Sicherung `%TEMP%/kraqel-c4b-observation-build-20261118-01/`, ergänzend
`artifacts/c4b-observation-20261118-01/` außerhalb von TEMP. `run-functional.json`, `run.json`,
private MAC-Audits und Modulherkunft, exaktes Programmpaar und Quellmanifest. ETL/XML nur lokal
unter `%TEMP%/kraqel-c4b-observation-etw-20261118-01-{reference,runtime}/`; keine Schlüssel, PIDs,
absoluten Prozesspfade oder nutzbaren Grants in Git. Alte Sicherung unverändert.

**Nächster begrenzter C4b-Scope:** native Identität der API-unzugeordneten Rolle sowie vollständigen
Modul-/Rollenvertrag und sichere Sperrentscheidung vor jeder späteren Text-/Commandfreigabe
umsetzen. Mit den beiden zusätzlichen Modulen konkret prüfen; keine pauschale Ausnahme oder
Abschaltung von Schutzsoftware. Effektive Daten-ACLs nach Runtimebetrieb weiterhin unabhängig
abnehmen. Danach benötigen neue Programmbytes erneut ihren gezielten Netznachweis. C4b insgesamt,
C4c/d, automatische Gruppen-Recovery, P2/P3–P5 und Produkt-Q-01/Q-02 bleiben offen; native UI-ABI 0.
Gesamtschätzung unverändert grob 65 %, nutzbarer 0.1-Umfang grob 80 %.
