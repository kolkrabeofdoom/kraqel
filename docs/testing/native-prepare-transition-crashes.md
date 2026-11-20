# KQ-069: Gezielte Prozessabbrüche im Paketvorbereitungsübergang

Stand: 17. November 2026. Mittleres Prüfniveau mit drei frischen synthetischen TEMP-Cases.
[Bereinigte Aggregate](native-prepare-transition-crashes.json),
[Paketartefakt-Vertrag](../data-format/native-product-control-v1.md),
[vorheriger Retry-Netzbeleg](native-package-artifact-retry-etw.md). Striktes Datenschutzziel
unverändert; keine Produkt-Q-01/Q-02-Freigabe.

## Abbruchpunkte und Ergebnis

Der Forschungsbroker ergänzt drei standardmäßig inaktive Punkte. Nur der explizite pfadbegrenzte
`product-control-child` kann sie im synthetischen TEMP-Controller-/Case-/Store-Layout aktivieren;
keine Umgebungsvariable oder Rendererfunktion. Harter Prozessabbruch mit 86 ohne Rust-Destruktoren
und ohne vorheriges Aufräumen. Storelease und Vorfahrenpins bleiben bis zum Prozessende gehalten.
Keine Änderung an Autoritätsformat, Netzfähigkeiten oder normalem Produktstart.

| Punkt              | Exakte Grenze                                                                                 | Zustand nach erneutem authentisiertem Öffnen          |
| ------------------ | --------------------------------------------------------------------------------------------- | ----------------------------------------------------- |
| `artifact-removed` | Nach handlegebundener Entfernung des authentisiert abgebrochenen Pakets, vor neuer Zielanlage | Alter Kopf unverändert, kein Pending, Paket fehlt     |
| `package-created`  | Nach exklusiver Zielanlage und ACL-Prüfung, vor Prepare-Record                                | Alter Kopf unverändert, kein Pending, Paketdatei leer |
| `prepare-recorded` | Nach vollständigem Prepare-Record/Kopfabschluss, vor erstem kopierten Byte                    | Neuer authentisierter Kopf, Pending, Paketdatei leer  |

Jeder Fall beginnt mit genau einer abgeschlossenen synthetischen Importaufgabe und einem
ausdrücklich abgebrochenen Kandidaten. Der frühere Abort ist die notwendige Berechtigung für die
spätere Ersetzung. Nach frühem Abbruch darf der aktuelle Actor mit aktuellem Kopf die fehlende/leere
Kopie erneut vorbereiten. Nach gespeichertem Prepare muss zuerst ein weiterer authentisierter Abort
erfolgen. Aktivierung der fehlenden/leeren Kopie sowie Retry mit altem Kopf werden ohne Kontroll-
oder Artefaktmutation abgewiesen; Pending-Recovery bleibt gesperrt.

Danach in allen drei Fällen denselben vollständigen Kandidatenhash erfolgreich erneut vorbereitet,
ausdrücklich abgebrochen und **dieselbe** gebundene Sitzung authentisiert wiederaufgenommen. Keine
Kandidatenausführung oder Aktivierung in dieser Serie. Bestehende Aufgabe bleibt verbraucht, Replay
gesperrt. Stilllegung und Profil-Cleanup einschließlich idempotenter Wiederholung bestanden.

44 Kontrollprozesse: 34 Erfolge, sieben erwartete Abweisungen und drei harte Prozessabbrüche. Sechs
GUI-Generationen, je Fall genau eine legitime Projektkopie mit zwei Dokumenten; keine Replaykopie.
Capabilityfreie AppContainer-Root-Tokens, verifizierte Kontrolllease und 36 tatsächliche private
Dateiverweigerungen. Alle GUIs enden kontrolliert mit 77, Jobs am Ende leer, drei Profile entfernt.

## Unabhängiger Nachweis und Prüfstand

Nur-Lese-Audit von SHA-256, Kontroll-MACs, kanonischer Rootbindung, Actor/Kopf und Sitzungsbindung.
Erste zwei Fälle je 13 Records, dritter Fall 15 durch zusätzliches Pending/Abort. Gespeicherte
Abbruchköpfe passen genau zur Kette. Installation/Schlüssel/App-/Sitzungs-/Aufgabenbindung erhalten,
neue Recovery-Launch-ID ohne neue Importautorität. Global drei verschiedene Aufgaben und drei
legitime Kopien. SQLite-Integrität, Fremdschlüssel und Unicode-Inhalte passen; Paketkopien nach
Retry entsprechen dem erwarteten vollständigen Hash. Private Abbruch-Snapshots unterscheiden
fehlende von leeren Dateien; bereinigte Zustandsflags stehen im JSON.

56 aktive Helfer-/Transporttests und Clippy mit Warnungen als Fehler bestanden, vier bestehende
Tests ignoriert. Zwei vorhandene Tests wurden um leere Paketdateien erweitert: expliziter Abort
eines leeren Pending-Pakets sowie erneute Vorbereitung einer authentisiert abgebrochenen leeren
Kopie. rustfmt, PowerShell-Syntax und Format-/Diff-/Hash-/Faktenprüfung grün. Ein neuer reiner
Helferbuild; kein vollständiger Appbuild/Workspace-Gesamtlauf oder neue große Matrix.

Gesicherte Programme unter `%TEMP%/kraqel-prepare-transition-build-20261117-01/`:

- Broker `88f861ab454f598a85c5eabe3b6fdb192c1af1e58ccfedef36db72bae1f4828a`.
- Synthetischer Kandidat `e4791d2954603bcec39030db5682331bbda6b11abed230e9639cd886f8fe92d7`.
- Unveränderte Forschungs-App `6e56a51ed2e913a878a7935e039bd69628a49be25f0fbab7d02fce3a72fe8c70`.

Quellbasis vor Änderung: `a106bcd313eb072ce22fce7f5148b46feed18ca0`; genaue Buildbindung im privaten
Manifest. Kandidat mit identischem Code plus Overlay, keine beliebige Release-/Datenmigration.

## Reproduktion und Grenzen

```powershell
$savedBuild = Join-Path $env:TEMP 'kraqel-prepare-transition-build-20261117-01'
./scripts/runtime-isolation/check-update-reconciliation.ps1 `
  -Probe "$savedBuild/kraqel-isolation-probe.exe" `
  -ProbeSha256 '88f861ab454f598a85c5eabe3b6fdb192c1af1e58ccfedef36db72bae1f4828a' `
  -App "$savedBuild/kraqel-desktop.exe" `
  -AppSha256 '6e56a51ed2e913a878a7935e039bd69628a49be25f0fbab7d02fce3a72fe8c70' -PrepareTransitions
cargo test --locked --manifest-path scripts/runtime-isolation/probe/Cargo.toml
cargo clippy --locked --manifest-path scripts/runtime-isolation/probe/Cargo.toml --all-targets -- -D warnings
```

`-PrepareTransitions` und `-RetryAborted` sind alternative begrenzte Serien. Ausschließlich neue
TEMP-Wurzeln, explizite Programmhashes, keine Übernahme/Kopie/Zurücksetzung vorhandener
Kontrollablagen. Die echte Vorschauablage und dort autorisierte EXE wurden nicht verwendet oder
verändert. Prozesse beendet, private Rohbelege/PIDs/UUIDs/Pfade/Schlüssel/Journale lokal erhalten,
nur Aggregate in Git.

Geprüft ist authentisierter Retry mit vorherigem Abort, keine Stromausfallgarantie und kein
allgemeines Orphan-Cleanup beim ersten Prepare ohne Abbruchhistorie. Unbekannte Artefakte und
unvollständige Kontrollketten bleiben gesperrt; keine Kopf-/Schlüsselreparatur oder neue
Importberechtigung. Inzwischen liegt der
[eigene frische ETW-Beleg](native-prepare-transition-etw.md) der unveränderten Programme vor, mit
regulärem verwaltetem Update-/Wiederanlaufpfad und den drei Übergängen. Der funktionale JSON-Beleg
bleibt separat ohne Netzmessung; frühere Netzbelege gelten weiter nur für ihre
Programme/Pfade/Lebenszeiten. Nächster begrenzter Scope: Start-/Integrationsvertrag für normalen
Produktstart mit bestehenden Projekten. Normale Produktintegration und weitere Produktabnahme
bleiben offen.
