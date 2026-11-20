# P2c C2: persistente synthetische Arbeitsablage und nativer Paketabgleich

Stand: 17. November 2026. **Datenvorbereitung implementiert und im beschriebenen Forschungsumfang
geprüft. Keine UI-/Workerstartautorität und keine Produktabnahme.**
[Vertragsentwurf](../data-format/product-start-p2c-v1-draft.md),
[reines C1-Modell](product-start-p2c-model.md),
[maschinelle Zusammenfassung](product-start-p2c-native-data.json).

## Implementierung und Versionsgrenze

[Nativer C2-Pfad](../../scripts/runtime-isolation/probe/src/windows/exchange/ui_restart/durable/product/workspace_persistent.rs)
mit eigenem
[Dateisystem-/ACL-Abgleich](../../scripts/runtime-isolation/probe/src/windows/exchange/ui_restart/durable/product/workspace_persistent/filesystem.rs)
und
[vier gezielten Tests](../../scripts/runtime-isolation/probe/src/windows/exchange/ui_restart/durable/product/workspace_persistent/tests.rs).

- Ausschließlich ein neuer synthetischer Store unter
  `%TEMP%/kraqel-isolation-controller-<id>/case-NN/store`. `InitializeC2` legt direkt einen
  Schema-6-DATA-Store an. **Keine Migration 5→6**, kein Öffnen/Autorisieren neuer Programme an
  echter Vorschauablage und kein Einbau in den normalen Appstart.
- Eigene schreibgeschützte PE-Deklaration `.kqc2`: DATA-Recordvertrag 1 für Schema 6, native
  UI-Unterstützung **0**. `.kqmeta` (1–4), `.kqprod` (Schema 5/UI 0) und `.kqboot` bleiben bei ihren
  bisherigen Bedeutungen. C2 wird nicht über deren bisherige Updatepfade als kompatibles UI-Paket
  angeboten.
- Der gebundene `WorkspaceBindingV2` enthält UI-ABI 1 als **zukünftigen Zielvertrag**. C2 speichert
  ihn in einer ausdrücklich datenbezogenen Action. Weder dieser Wert noch `maxProcesses=2` oder
  DB-Zielbereich 6 erlauben einen Start: effektive C2-Policies lauten `no-execution` und
  `no-ui-or-import-commands`. Es gibt keine UI-EXE im Ressourcenpaket, keine WebView-Runtime und
  noch keine SQLite-Datenbank. Diese Integration gehört zu C3/C4.

## Kontrollsperre und dauerhafte Records

Vorbereitung, Prüfung, Bearbeitung und Stilllegung halten jeweils denselben privaten `Store` samt
`owner.lock`, MAC-Schlüssel, authentisierter Head-/Recordkette und gepinnten Vorfahren. Actor,
erwarteter Head, Budget und der bestehende native Jobwächter werden vor Änderungen geprüft. Schema 6
besitzt **keinen** ausführbaren Startübergang; der bestehende Jobwächter wird dadurch ausdrücklich
nicht zum Mehrprozess-Endbeleg für C3 umgedeutet.

| Action               | Wirkung und Abbruchgrenze                                                                                |
| -------------------- | -------------------------------------------------------------------------------------------------------- |
| `InitializeC2`       | Nur frische synthetische Genesis, keine Übernahme eines vorhandenen Verzeichnisses                       |
| `PrepareC2Workspace` | Neue Workspace-ID dauerhaft verbrauchen, bevor externe Ressourcen entstehen                              |
| `C2ProfileReady`     | Erst nach erfolgreicher nativer Neuanlage und SID-/Mapping-/Namensprüfung                                |
| `BindC2Workspace`    | V2-Bindung plus native Identitäten der Ausführungs-/Hostpaket-/UI-Paketroots nach vollständigem Abgleich |
| `RetireC2Workspace`  | Eigener authentisierter Stilllegungsauftrag für ein bestätigtes Profil                                   |
| `C2ProfileRetired`   | Profilabwesenheit bestätigt; Daten/Bindung bleiben erhalten, weitere Nutzung gesperrt                    |

Vor Vorbereitung werden fünf weitere Recordslots reserviert, einschließlich expliziter Stilllegung.
Unbekannte oder unvollständige Vorbereitung wird nicht fortgesetzt, neu gebunden oder automatisch
repariert. Insbesondere berechtigt Prepare ohne ProfileReady nicht zur Profillöschung, auch wenn
nach einem Crash ein Profil existieren könnte. Nach ProfileReady darf die eigene bestätigte
Profilressource ausdrücklich stillgelegt werden, auch wenn die abschließende Bindung fehlt. Eine
unterbrochene Stilllegung kann unter derselben Sperre fortgesetzt werden; kein fremder Profilname
und keine automatische Wiederanlage. Keine neue allgemeine Crashserie oder Stromausfallgarantie.

## Ablagen und Paketprüfung

Frische Ausführungsablage `%TEMP%/kraqel-isolation-p2c-<workspace-id>` mit getrennten `projects`,
`webview`, `sessions` und `package`. Das dauerhafte Profil heißt `kraqel-p2c-<workspace-id>`;
normales Controllerende löscht es nicht. Profil-Drop gibt nur den lokalen SID-Speicher frei. Private
P2a-Fixtures werden nicht promoviert.

- Kontrollstore und Hostkopie `store/bin/c2-host/controller.exe` bleiben ausschließlich Konto/SYSTEM
  zugänglich. Die Ausführungsroot und das Ressourcenpaket erhalten nur RX für die exakt gebundene
  Profil-SID; die drei Datenroots erhalten Modify. Geschützte DACLs müssen exakt den erwarteten drei
  ACEs einschließlich Rechten, Flags und SIDs entsprechen. Keine allgemeinen AppContainer-Grants
  oder Firewall-/Loopbackänderungen.
- Drei Datenroots sowie Ausführungs- und Paketroots werden mit kanonischem Pfadhash, Volume-ID und
  128-Bit-Datei-ID gebunden. Verzeichnisvorfahren bleiben während des Abgleichs gegen
  Umbenennen/Löschen gepinnt. Physisch ausgetauschte Roots werden beim Wiederöffnen abgewiesen.
- Beide vollständigen **synthetischen** Paketbäume werden gegen private kanonische C1-Inventare
  geprüft. Hostpaket: nur die tatsächliche Controllerkopie; UI-Ressourcenfixture: `policy.json` und
  Unicode-Text `ä.txt`. Keine Behauptung einer vollständigen normalen App-/Runtime-Paketclosure.
- Native C2-Grenzen sind enger als der allgemeine Inventarvertrag: 64 Dateien, 128 Baumeinträge,
  Tiefe 8, 128 MiB/Datei und 256 MiB/Paket. Fehlende/zusätzliche Dateien oder leere Zusatzordner,
  falsche Längen/Hashes/ACLs, Reparse-/Offlineflags und Hardlinks werden abgewiesen. Gehaltene
  Dateihandles ohne Write-/Delete-Sharing schützen die tatsächlich gehashten Paketdateien bis zum
  Bindungsrecord bzw. Ende der Verifikation.
- Zusätzlich zum portablen C1-Kollisionsschlüssel verwendet C2 den nativen
  [Windows-Ordinalvergleich](https://learn.microsoft.com/en-us/windows/win32/api/stringapiset/nf-stringapiset-comparestringordinal).
  [Datenstromenumeration](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-findfirststreamw)
  weist benannte ADS an Paketdateien und -ordnern ab; unvollständige/fehlgeschlagene Erfassung
  bleibt gesperrt. Pfade werden vor Zugriff auf lokale feste Datenträger begrenzt.
- Arbeitsdaten bleiben veränderlich: Der synthetische Text ist kein dauerhaft festgeschriebener
  Dateihash. Der Forschungsbefehl `exercise` bearbeitet ausschließlich diese feste Datei nach
  Link-/Reparse-/ACL-Prüfung. Kein Projektinventar, keine Bestandsdatenbanköffnung oder Migration.

Explizite Profilstilllegung löscht keine Arbeitsdaten. Verbliebene Dateien sind lokale synthetische
Fixtures; Stilllegung ist kein allgemein abgenommener Datenlösch-/ACL-Retirementvertrag für spätere
laufende UI-Prozesse. Tatsächliche Lowbox-Dateizugriffe und Token/Kindrollen bleiben C3-Prüfungen.

## Ausgeführte Prüfung

Gesicherter Controller: `73a7aad5e61a0ef7ebbc8cf8ccc75611c50ebd6bad3c52ad5f9fcc21ddc957b6`. Lokale
Sicherung `%TEMP%/kraqel-product-p2c-data-build-20261117-01`; kein Installer/Release.

```powershell
cargo test --locked --offline --manifest-path scripts/runtime-isolation/probe/Cargo.toml --quiet
cargo clippy --locked --offline --manifest-path scripts/runtime-isolation/probe/Cargo.toml --all-targets -- -D warnings
cargo fmt --manifest-path scripts/runtime-isolation/probe/Cargo.toml -- --check
cargo build --locked --offline --manifest-path scripts/runtime-isolation/probe/Cargo.toml
./scripts/runtime-isolation/check-workspace-persistent.ps1 -Probe <gesicherter-Controller> -ProbeSha256 <obiger-Hash>
```

Vier neue native C2-Tests, gesamter Helferlauf **92 bestanden/vier bewusst ignoriert** (85
Binärtests plus sieben Transporttests), Clippy grün. Gezielte Fälle: Profil-/Datenbestand nach
Schließen/Wiederöffnen, stabile Bindung/Schlüssel, gehaltene Paketdateien nicht schreib- oder
umbenennbar, manipulierte Inhalte/Zusatzdateien/-ordner/ADS/Hardlinks, Unicode-Ordinalvergleich,
falsche Rootidentität/ACL, private Kontrollrechte, fremder Actor, alter Head, Budget und
unbestätigte Vorbereitung ohne Adoption. Keine breite Crashmatrix oder assistive UI-Prüfung.

Zusätzlicher gesicherter Prozesslauf: **zehn getrennte Controlleraufrufe**, davon drei erwartete
Abweisungen. Vorbereitung → Verifikation → synthetische Änderung → neue Verifikation; Profil, IDs
und geänderte Daten bleiben erhalten. Alte Heads und erneute Vorbereitung mutieren die
Kontrollhistorie nicht. Explizite Stilllegung entfernt das eine Profil, behält die Daten und
unveränderten Schlüssel. Ein unabhängiger lesender Python-Audit bestätigt alle sechs MAC-Records,
Head/Kette/Actor-/Rootbindung, beide Inventare samt tatsächlichem Inhalt, Profilabwesenheit,
Controllerergebnisse und unveränderte Hashes der echten Vorschau-Kontrollfiles.

Prozesse beendet; Profile der gezielten Tests und des gesicherten Prozesslaufs entfernt. Rohpfade,
PIDs, IDs, Schlüssel und Records bleiben lokal; nur bereinigte Zusammenfassung in Git. Kein
gemeinsamer Transport-/App-/Installerumbau, daher kein vollständiger Tauri-Build oder `pnpm check`.
Der eigene [frische ETW-Netzvergleich](product-start-p2c-native-data-etw.md) des unveränderten
Controllers ist inzwischen ergänzt. Die obige Funktionsprüfung und ihr JSON bleiben als eigener
Stand ohne Netzwerkmessung erhalten; der ältere `37d7bec…`-Beleg wird nicht übertragen.

Der [C3-Synthesekern](product-start-p2c-group-bootstrap.md) und sein
[eigener frischer Netzbeleg](product-start-p2c-group-bootstrap-etw.md) sind inzwischen ergänzt.
Nächster begrenzter Schritt ist der C4-Vertrag für Runtime-Paket und minimale isolierte Oberfläche.
P2 insgesamt, vollständige C3/C4-Produktintegration, P3–P5 und Produkt-Q-01/Q-02 bleiben offen. Das
strikte Datenschutzziel bleibt unverändert.
