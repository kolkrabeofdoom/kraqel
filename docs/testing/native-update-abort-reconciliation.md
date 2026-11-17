# KQ-069: Gezielte Update-Abbruch-/Fehlerreconciliation

Stand: 17. November 2026. Mittleres Prüfniveau, ausschließlich neue synthetische TEMP-Ablagen.
[Bereinigte Aggregate](native-update-abort-reconciliation.json),
[Paketvertrag](../data-format/native-product-control-v1.md),
[vorheriger eigener ETW-Beleg](native-schema4-package-update-etw.md). Striktes Datenschutzziel
unverändert; keine Produkt-Q-01/Q-02-Freigabe.

## Verhalten und überprüfte Grenze

Der vorhandene authentisierte Abbruchpfad benötigt keine Änderung am Broker-Laufzeitcode:
`Store::open` rekonstruiert ausschließlich eine vollständige MAC-Kette mit passendem Kopf und hält
den privaten exklusiven `owner.lock` über Prüfung und Aktion. Gültiges Pending nach fehlender,
unvollständiger oder veränderter Paketkopie erlaubt keine Aktivierung oder neue Importautorität. Nur
der aktuelle Actor mit aktuellem Kopf und leerem Produktjob darf `Abort` anhängen. Das entfernt
Pending und erhält den bisherigen Actor sowie App-/Sitzungs-/Aufgabenbindung.

Danach kann derselbe Broker die bestehende Sitzung authentisiert wiederaufnehmen. Erneuter Abort
ohne Pending oder Aktivierung nach Abbruch wird ohne Kontrollmutation abgewiesen. Nach Prozessende
hinter dem veröffentlichten Abort-Kopf genügt erneutes authentisiertes Öffnen. Ein gültiger
Abort-Record ohne passenden veröffentlichten Kopf bleibt dagegen gesperrt: keine automatische
Kopf-/Schlüssel-/Kettenreparatur. Geprüft ist Prozessabbruch, keine Stromausfallgarantie.

Zwei neue Helfertests prüfen fehlende, auf 1.024 Bytes gekürzte und manipulierte Pending-Kopien:
Aktivierung/falscher Actor/alter Kopf ändern nichts; die Sperre bleibt bis nach dem authentisierten
Abbruch exklusiv. Erneutes Öffnen erhält Schlüssel, Sitzung und Verbrauchsmarker. Ein vollständig
signierter Abort-Tail ohne neuen Kopf blockiert wiederholtes Öffnen ohne Mutation.

## Drei native Fälle

Unveränderte gesicherte Programme aus `ef815ea8314bb23db4b502948e5b1bc63151c6f8`:

- Broker `45542384ce42b73450453dce2f5a40b635b7b8552055de25d60c147fc21dd360`.
- Synthetischer Kandidat `6748900defbbdb8df5e8e5ec0e611091cd1e6656880381a18e43e8755510bf90`.
- Forschungs-App `6e56a51ed2e913a878a7935e039bd69628a49be25f0fbab7d02fce3a72fe8c70`.

| Fall                | Fehler-/Haltepunkt                                                 | Ergebnis                                                                                   |
| ------------------- | ------------------------------------------------------------------ | ------------------------------------------------------------------------------------------ |
| Teilkopie           | Controllerabbruch bei `package-partial`, 1.024 Bytes geschrieben   | Pending rekonstruierbar, Aktivierung gesperrt, ausdrücklicher Abbruch und Recovery möglich |
| Manipuliertes Paket | Nach Vorbereitung synthetische Bytes an verwaltete Kopie angehängt | Hashabweichung sperrt Aktivierung, Abbruch erhält bisherigen Actor und Sitzung             |
| Dauerhafter Abbruch | Controllerabbruch bei `head-published` des Abort                   | Neustart erkennt terminalen Abbruch, Recovery ohne neuen Import oder Abort                 |

Jeder Fall beginnt mit einer abgeschlossenen synthetischen Importaufgabe. Negative Aufrufe prüfen
alten Kopf, nicht autorisierten Actor, Pending-Recovery und Wiederholung nach Abbruch. Im
Teilkopiefall blockiert zusätzlich ein unabhängig gehaltener exklusiver Dateihandle Abort und
Aktivierung. Fingerabdrücke aller Kontrollrecords, des Kopfs und Schlüssels bleiben bei Abweisungen
unverändert. Der negative Actor-Aufruf startet **absichtlich ausschließlich in der Prüfung** den
bekannten synthetischen Kandidaten als fremden Aufrufer: drei erwartete Abweisungen, keine GUI oder
Mutation. Das Updateverfahren selbst lädt/startet keinen Kandidatencode vor Aktivierung. Kein
Nachweis beliebigen Kandidatenverhaltens oder einer Herausgebersignatur.

**45 Kontrollprozesse:** 24 Erfolge, 19 erwartete Abweisungen und zwei harte Prozessabbrüche mit 86.
Sechs GUI-Generationen, alle kontrolliert mit 77 beendet; 36 tatsächliche private
Dateiverweigerungen mit capabilityfreien AppContainer-Root-Tokens und verifizierter Kontrolllease.
Drei Profile nach authentisierter Stilllegung entfernt; Cleanup-Wiederholung idempotent, keine
verbleibenden Jobprozesse.

Ein unabhängiger Nur-Lese-Audit prüft pro Fall elf MAC-Records, Root-/Kopf-/Actorbindung, genau
einen Abort, erhaltene Installation/App-/Sitzungsbindung und neue Recovery-Launch-ID. Global drei
verschiedene Aufgaben und genau drei legitime Projektkopien mit jeweils zwei Dokumenten, keine
Replaykopie. SQLite-Integrität, Fremdschlüssel und Unicode-Inhalte passen. Paketartefakte
entsprechen den Fehlern; die echte Vorschauablage wurde nicht verwendet oder verändert.

## Reproduktion und Prüfstand

PowerShell 7, Repositorywurzel, explizit gesicherte Hashes:

```powershell
$savedBuild = Join-Path $env:TEMP 'kraqel-update4-build-20261117-01'
./scripts/runtime-isolation/check-update-reconciliation.ps1 `
  -Probe "$savedBuild/kraqel-isolation-probe.exe" `
  -ProbeSha256 '45542384ce42b73450453dce2f5a40b635b7b8552055de25d60c147fc21dd360' `
  -App "$savedBuild/kraqel-desktop.exe" `
  -AppSha256 '6e56a51ed2e913a878a7935e039bd69628a49be25f0fbab7d02fce3a72fe8c70'
cargo test --locked --manifest-path scripts/runtime-isolation/probe/Cargo.toml
cargo clippy --locked --manifest-path scripts/runtime-isolation/probe/Cargo.toml --all-targets -- -D warnings
```

Der Helfer erzeugt ausschließlich eine frische `kraqel-isolation-controller-*`-TEMP-Wurzel mit drei
Cases und gibt deren privaten Pfad zurück. Pfade, PIDs, Schlüssel, Journale und Rohbelege bleiben
dort und gehören nicht in Git. Keine Verwendung/Kopie/Zurücksetzung bestehender Kontrollablagen.
Programmhashes vor und nach dem Lauf unabhängig geprüft; keine neuen Broker-/Appbytes erzeugt. Nur
Test-/Reproduktionscode und Dokumentation geändert.

**53 aktive Helfer-/Transporttests und Clippy bestanden**, vier bereits bestehende Tests ignoriert;
rustfmt sowie Format-/Diff-/Faktenprüfung grün. Keine neue große Matrix, kein vollständiger Appbuild
oder Workspace-Gesamtlauf. Testprozesse beendet, private synthetische Belege erhalten.

Keine neue ETW-Erfassung: Der frühere Beleg bleibt an seine Programme, ausgeübten Pfade und
Lebenszeiten gebunden. Diese zusätzlichen Fehlerläufe erhalten keinen übertragenen Netzstatus.
Normale Produktintegration, externe Dienstkausalität und Produkt-Q-01/Q-02 bleiben offen.

## Verbleibende Paketartefakte

`Abort` löscht keine Paketdateien. Teilkopien/manipulierte oder abgebrochene vollständige Kopien
bleiben ohne Ausführungsautorität als private Diagnoseartefakte liegen. Allgemeines Paket-Cleanup
oder transparente Wiederholung sind nicht abgenommen. Die bestehende `create_new`-Vorbereitung kann
denselben Hash bei vorhandener Kopie nicht erneut installieren und dabei erneut ein authentisiertes
Pending hinterlassen, das explizit abzubrechen ist (statisch verifizierter Codepfad, kein
zusätzlicher nativer Retrylauf).

Nächster begrenzter Scope: solche Artefakte unter derselben Kontrollsperre sicher behandeln und
erneute Vorbereitung ohne unbeabsichtigtes Pending ermöglichen. Keine automatische Reparatur
beschädigter Kontrollketten, kein Löschen aktiver/früher aktivierter Programme oder Aufgabenreset.
