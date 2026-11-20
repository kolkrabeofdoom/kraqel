# KQ-069: Integrierter isolierter GUI-Neustart

Historischer v1-Funktionsnachweis. Die spätere
[assistive Wiederanlaufprüfung](native-assistive-restart.md) nutzt ein anderes Hashpaar und
Startticket v2. Ein Neubuild aktuellen Quellcodes erhält die hier dokumentierten v1-Nachweise nicht
automatisch.

Finale Serie bestanden, Exitcode des Controllers 0. [Bereinigte Aggregate](native-ui-restart.json).
App-SHA-256: `ca4e70a39a96ec6c642863494be616c47a7e9243afc4ba20726606ddf7c54324`. Controller-SHA-256:
`12cee20170c7672f959703d592d370f57b7cb90311fed524330f2dd05208508f`. Nach dieser Serie keine weiteren
Runtimeänderungen. Das geprüfte Paar ist getrennt lokal gesichert; der normale Debug-Build
anschließend wiederhergestellt. Ein späterer
[frischer ETW-Lauf desselben unveränderten Paars](native-ui-restart-etw.md) ist inzwischen gültig
abgenommen; der hier dokumentierte ursprüngliche Funktionslauf bleibt ein eigener Nachweis. Frühere
Hashnachweise werden nicht übertragen.

Stand: 16. November 2026.
[Startticket und Bedrohungsgrenze](../data-format/native-ui-restart-v1.md). Striktes Datenschutzziel
unverändert; #134 und Produkt-Q-01/Q-02 bleiben offen.

## Finaler nativer Ablauf

Die echte gebündelte Tauri-Forschungsoberfläche durchläuft sechs Crashpunkte, jeweils mit zwei
Wiederanläufen derselben Kennung und einem zusätzlich abgewehrten Erstversuch mit derselben Kennung.
Gleicher lebender Controller, gleiche AppContainer-Identität, gleicher Programmstand und dieselbe
private Datenwurzel. Frische einmalige Pipes/Launch-Bindungen je Start.

| Crashpunkt                                  | Erwarteter Wiederanlauf                                 | Kumulierte neue Projekte |
| ------------------------------------------- | ------------------------------------------------------- | ------------------------ |
| Dauerhafter Claim                           | `INTERRUPTED`, kein Lesen erneut freigegeben            | 0                        |
| Erste 512 Fixturebytes gelesen              | `INTERRUPTED`                                           | 0                        |
| Vollständig vorbereitete Kopie              | `INTERRUPTED`, Zwischenstand entfernt                   | 0                        |
| Commit-Intent gespeichert, noch kein Rename | `INTERRUPTED`, keine Veröffentlichung beim Wiederanlauf | 0                        |
| Rename abgeschlossen                        | Vorhandene Kopie geprüft zugeordnet                     | 1                        |
| Terminaler Abschluss gespeichert            | Vorhandener Abschluss erhalten                          | 2                        |

Der Crash erfolgt jeweils durch `TerminateJobObject` an einem zuvor nativ gemeldeten Haltepunkt.
Weitere Generationen beginnen erst nach Job-Leerstand. Auch die wiederhergestellten Testfenster
werden nach den erwarteten Replay-Ereignissen kontrolliert hart beendet; Code 77 ist daher in 24
Generationen erwartet. Vier negative Starts enden mit Code 2 vor Rendererstatus und lassen das
Journal unverändert. Das Profil wird abschließend entfernt.

Die feste Automatisierung bedient die realen Schaltflächen; Wiederherstellung/Status, native IPC,
Rust-Importer und SQLite sind beteiligt. Der erste Replay-Hinweis muss sichtbar im DOM stehen und
die Schaltfläche DOM-Fokus haben, bevor das zweite Replay ausgelöst wird. Keine gehörte Ansage oder
manuell bestätigte native Fokusabnahme behauptet. Der Browser-Mock wird nur für getrennte E2E
verwendet. Dateidialog-/Live-Pipe-Crash ist nicht Bestandteil dieser Fixture-Serie.

Nach Prozessende unabhängig geprüft und im bereinigten Aggregat festgehalten: sechs verbrauchte
Aufträge, davon vier `interrupted` und zwei `succeeded`; genau zwei verschiedene neue Projekt-UUIDs,
je zwei Dokumente, exakter synthetischer Unicode-Inhalt, Receipt-/Archivhash, SQLite-Integrität und
Fremdschlüssel. Keine Zwischenreste, alle 28 suspendierten Root-Tokenprüfungen korrekt, alle Jobs
leer. Rohdaten einschließlich PIDs, UUIDs und lokaler Pfade bleiben lokal.

## Reproduktion und Qualität

Für die nächste ETW-Serie liegt genau das geprüfte Paar lokal unter
`%TEMP%\kraqel-kq069-restart-final-build-20261116-01` (`kraqel-desktop.exe` und
`kraqel-isolation-probe.exe`). Vor Verwendung beide Hashes mit dem Aggregat abgleichen. Der
wiederhergestellte normale `target/debug`-Build ist nicht dieses Forschungsartefakt. Auf einem
anderen Rechner neu bauen und die native Serie neu abnehmen; alte lokale Hashbelege nicht
übertragen.

```powershell
$env:KRAQEL_ISOLATION_UI_TEST = '1'
pnpm --filter @kraqel/desktop tauri build --debug --no-bundle --features isolation-ui-test
Remove-Item Env:KRAQEL_ISOLATION_UI_TEST
cargo build --locked --manifest-path scripts/runtime-isolation/probe/Cargo.toml
$restartRoot = Join-Path $env:TEMP ('kraqel-isolation-ui-restart-' + [guid]::NewGuid().ToString('N'))
& scripts/runtime-isolation/probe/target/debug/kraqel-isolation-probe.exe ui-crash-series $restartRoot target/debug/kraqel-desktop.exe
```

Nur Forschungsfenster entstehen; während der autonomen Serie nicht bedienen. Ein bestehendes
Verzeichnis wird vor Profil-/ACL-Anlage abgelehnt. Direkter unisolierter Forschungsstart ohne
gültigen Kanal wird mit Exitcode 2 abgelehnt. Bei Fehler oder Timeout nicht Ergebnisse alter
Generationen übernehmen; zuerst lokale Diagnose und Cleanup prüfen.

Gezielte Qualität: acht Browserprüfungen der Forschungsseite einschließlich beider
Wiederherstellungszustände, insgesamt 35 Browser-E2E; zwei Forschungs-Origin-/DTO-Tests; 23 aktive
Helfer-/Transporttests einschließlich Ticket-/Nachweisprüfung. Native Dialogprüfung bleibt in der
regulären Suite bewusst ignoriert. Feature- und Helfer-Clippy mit Warnungen als Fehler sowie
`pnpm check` gehören zur Übergabe. Keine neuen Abhängigkeiten oder Registryversionen.

Alle genannten lokalen Qualitätsprüfungen bestanden, ebenso Forschungs- und normaler Debug-Build.
Die autonome native Serie bleibt ein lokaler Windows-Nachweis; die CI führt die Compiler-, Unit- und
Browserprüfungen aus. Die Crash-Serie mit geöffneten nativen Fenstern läuft ausschließlich lokal.

## Noch offen

Assistive Wiederanlauf-Hörprüfung, Controllerabsturz und dauerhafter Produktbroker, weitere Formate,
externe COM-/Shell-/DNS-/Broker- Grenzen, Produkt-/Packaging-Integration und Stromausfallabnahme.
Keine universelle Netzstille oder Produktfreigabe aus dem funktionalen Neustarttest ableiten.
