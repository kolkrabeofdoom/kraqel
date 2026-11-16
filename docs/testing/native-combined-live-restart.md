# KQ-069: Kombinierte Live-Auswahl, Wiederanlauf und neuer Dateiauftrag

Stand: 16. November 2026. Im getrennten Windows-Forschungsmodus implementiert und nativ geprüft.
[Startticket v3](../data-format/native-ui-restart-v3.md),
[Brokerkette v3](../data-format/native-broker-recovery-v3.md),
[bereinigte Aggregate](native-combined-live-restart.json). Kein produktiver Launcher oder neuer
Hörnachweis. Ein späterer [eigener frischer ETW-Nachweis](native-combined-live-restart-etw.md)
dieses unveränderten Paars ist inzwischen gültig bestanden; der ursprüngliche Funktionslauf bleibt
separat. Frühere Paarabnahmen werden nicht übertragen.

## Implementiert

Der initiale Live-Auftrag besitzt zusätzliche einmalige Request-/Response-Pipes, wird vor Auswahl
dauerhaft im isolierten Journal verbraucht und erhält begrenzte Archivbytes statt eines Hostpfads.
Nach Controllerverlust authentisiert ein neuer Broker den privaten HMAC-Beleg; die App erhält beim
Wiederanlauf keine Auswahlhandles. Der alte Erstversuch muss vor einem weiteren Auswahlwunsch und
ohne Journaländerung scheitern. Kein Renderer kann Pfade, Schlüssel oder Grantparameter liefern.

Nach erfolgreicher Reconciliation und Replay-Abwehr schreibt der Host einen eigenen signierten
Abschlussbeleg. Erst eine ausdrücklich neue native Aktion darf den nächsten unveränderlichen Grant
mit neuer Kennung und Vorgängerhash anhängen. Jeder Wiederanlauf prüft die vollständige Kette,
einschließlich Abschlussbelegen/Projektanzahlen. Kein Schlüssel-/Journalreset, keine
Altdatenmigration oder automatische Wiederholungsfreigabe.

## Finales gesichertes Paar

Quellstand `9f5a7ee8c56220bb90f10c4f304c9b0f01cd43ee`. App-SHA-256
`6e56a51ed2e913a878a7935e039bd69628a49be25f0fbab7d02fce3a72fe8c70`; Broker-SHA-256
`49a2a0c0f0a8428c9419f8a70ce809e89a9f66171053e0a00032fcb3f6a48852`. Separat gesichert unter
`%TEMP%\kraqel-kq069-combined-build-20261116-04`. Keine Runtimeänderung nach den finalen Läufen. Der
normale Debug-Build wurde getrennt ohne Forschungsseite wiederhergestellt; er ist kein Ersatz für
dieses Paar.

Reproduktion des Forschungsbuilds verlangt **beide** Schalter und eine tatsächlich vorhandene
Frontendseite vor dem Start:

```powershell
$env:KRAQEL_ISOLATION_UI_TEST = '1'
try {
  pnpm --filter @kraqel/desktop tauri build --debug --no-bundle --features isolation-ui-test
} finally {
  Remove-Item Env:KRAQEL_ISOLATION_UI_TEST
}
Test-Path apps/desktop/dist/isolation-test.html
cargo build --locked --manifest-path scripts/runtime-isolation/probe/Cargo.toml
```

## Automatische native Serie

Neun frische Sitzungen, jeweils zwei verschiedene Live-Aufträge, zwei Controllerenden mit Code 86,
drei Wiederherstellungen und Stilllegung. Je Fall acht App-Starts: erster Versuch, Recovery und
abgewiesener Altimport; neuer Auftrag, Recovery/Altimport und nochmalige Recovery/Altimport.
Zusammen 18 Controller-Kills, 27 Wiederanlaufprozesse, 72 App-Generationen und 18 neue Auftrags-IDs.

| Erster Versuch          | Erstes Ergebnis nach Wiederanlauf | Neuer Folgeauftrag | Kopien am Ende |
| ----------------------- | --------------------------------- | ------------------ | -------------- |
| Fixture `claimed`       | `interrupted`                     | `succeeded`        | 1              |
| Fixture `reading`       | `interrupted`                     | `succeeded`        | 1              |
| Fixture `prepared`      | `interrupted`                     | `succeeded`        | 1              |
| Fixture `commitIntent`  | `interrupted`                     | `succeeded`        | 1              |
| Fixture `published`     | `succeeded`                       | `succeeded`        | 2              |
| Fixture `completed`     | `succeeded`                       | `succeeded`        | 2              |
| Auswahlabbruch          | `failed`, `CANCELLED`             | `succeeded`        | 1              |
| Ungültiges Archiv       | `failed`, `IMPORT_FAILED`         | `succeeded`        | 1              |
| 1.024-Byte-Teiltransfer | `interrupted`                     | `succeeded`        | 1              |

Alle sechs unzulässigen Folgeaktionen je Sitzung abgewiesen, insgesamt 54: kein Abschlussbeleg,
beschädigter Abschluss-MAC, Lücke der Grantfolge, falscher Vorgängerhash trotz gültigem Test-MAC,
noch nicht abgeglichener Folgeauftrag und stillgelegte Sitzung. Native Prüfung bestätigt dabei
unverändertes Journal und keine weitere App-Startdiagnose. Die negativen Dateien werden
ausschließlich vom nativen Testhost präpariert und danach kontrolliert zurückgenommen, niemals vom
Renderer.

Im Teiltransferfall sind exakt 1.024 von 11.915 Bytes gesendet; der Host verlangt vor Selbstabbruch
zusätzlich den passenden tatsächlich geschriebenen Empfangszähler. Spätere Folgeaufträge
überschreiben die Empfangsdiagnose, weshalb ihr finaler Wert nicht als historischer 1.024-Byte-Beleg
ausgegeben wird. Die private Senderdiagnose bleibt erhalten, und die Runtime-Gegenprüfung ist
Voraussetzung für den akzeptierten Controller-Kill.

## Tatsächliche manuelle Auswahl

Mit demselben finalen Paar `source: live`, `automatic: false`, Crashpunkt `prepared`: Die prüfende
Person bestätigt, dass der Importknopf den nativen Dialog öffnet, die vorbereitete `input.kraqel`
gewählt werden kann und das Fenster anschließend am vorgesehenen Haltepunkt selbst schließt. Native
Senderdiagnose bestätigt tatsächlichen Dialog und vollständige 11.915 Bytes.

Danach automatisch geprüft: authentisierte Wiederherstellung desselben verbrauchten Auftrags ohne
Kopie, abgewehrter Altimport, ausdrücklicher neuer Fixture-Auftrag in derselben Sitzung, Controller-
Kill, zweimaliger Wiederanlauf/Replay und Stilllegung. Zwei Journal-IDs (`interrupted`,
`succeeded`), genau eine Projektkopie und zwei unveränderte MAC-Grants. Die anfängliche manuelle
Controller- Exitnummer wurde nicht gesondert aufbewahrt; daraus keine unabhängig protokollierte 86
behaupten. Nachfolgender neuer Controller-Kill und Rückgaben sind in der auswertenden Skriptfolge
geprüft. Dies ist keine neue Sprachausgabe-/Tastaturabnahme.

## Unabhängige Prüfung, Fehler und Grenzen

Nur-Lese-Audit mit Pythons unabhängiger HMAC-Implementierung: alle Grant-/Abschluss-MACs,
Vorgängerhashes, verschiedene IDs, kanonische Rootbindung über Windows-Dateihandle und beide
Programmhashes gültig. Journalzustände, Kopienzahlen, SQLite-Integrität/Fremdschlüssel,
Receipt-/Archivhash, je zwei Dokumente und exakter synthetischer Unicode-Inhalt geprüft. Leeres
Staging, Profile nach Stilllegung entfernt. Insgesamt 80 App-Startbelege mit korrekter suspendierter
SID/null Capabilities und 480 tatsächlichen Dateizugriffsverweigerungen. 60 reguläre native
Generationsberichte mit Code 77/leerem Job; keine erfundenen Abschlussberichte für Controller-Kills.
Rohbelege, PIDs, UUIDs, Pfade und Schlüssel bleiben lokal.

Ein früher Build ohne Frontend-Buildflag enthielt die Forschungsseite nicht und wurde nicht
akzeptiert. Danach korrigierter Testhost-Faktencheck mit nichtkanonischem TEMP-Alias: kein
Implementierungserfolg daraus abgeleitet. Beide Sitzungen explizit stillgelegt. Das erste manuelle
Klicken scheiterte tatsächlich vor Dialogöffnung: kanonischer `\\?\`-Pfad wurde vom Shell-
Ordneraufruf mit `E_INVALIDARG` abgewiesen, Kanal schloss, der Host brach den unerwarteten Ablauf
ab. Der Fehler ist getrennt nativ reproduziert. Jetzt wird ausschließlich für den Shell-Aufruf eine
gleichwertige lokale Diskdarstellung mit identischem kanonischem Ziel verwendet; Scope-/Link-/
Dateiprüfungen bleiben erhalten. Canonical- und normaler Dialog-/Schließtest bestehen, der manuelle
Wiederholungslauf ist menschlich bestätigt. Die fehlgeschlagene manuelle Sitzung wurde stillgelegt.

Qualität: `pnpm check`, 28 aktive Helfer-/Transporttests, beide standardmäßig ignorierten nativen
Dialogtests ausdrücklich lokal bestanden, zwei Feature-Origin-/DTO-Tests, acht gezielte Browser-
Forschungstests, Feature-/Helfer-Clippy mit Warnungen als Fehler und Builds bestanden. Die alten
Ticket-/Broker-v2-Binaries behalten ausschließlich ihre alten Funktions-/ETW-Nachweise.

Zusätzlich besteht die bisherige synthetische 28-Generationen-Neustartserie mit dem finalen neuen
Paar und Startticket v3, einschließlich vier abgewiesener Fehlstarts. Journalintegrität, vier
unterbrochene/zwei erfolgreiche Aufträge, zwei Kopien, leeres Staging und Cleanup unabhängig
geprüft. Diese Regression ist getrennt von den 80 kombinierten App-Startbelegen gezählt.

Nächster Scope nach dem eigenen ETW-Beleg: kombinierte assistive und weitere Fensterendeprüfung.
Geschützte Produktablage, authentisierter Update-/Migrationspfad, externe
COM-/Shell-/DNS-/Brokergrenzen und Produkt-/Packaging-Integration bleiben offen. Ein Crash nach
Grant-Synchronisierung, aber vor Journal-Claim kann konservativ blockieren; keine automatische
Neuvergabe. Schlüssel bleibt unter privater DACL unverschlüsselt; kein Schutz gegen kompromittiertes
Konto/Administrationsrechte/Schlüsselkopie. Striktes Datenschutzziel unverändert; normale Starts
bleiben unisoliert, #134 und Produkt-Q-01/Q-02 nicht bestanden.
