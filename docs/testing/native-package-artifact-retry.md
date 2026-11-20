# KQ-069: Sichere Paketartefakte und erneute Vorbereitung

Stand: 17. November 2026. Mittleres Prüfniveau, synthetische TEMP-Ablagen.
[Bereinigte Aggregate](native-package-artifact-retry.json),
[Kontrollvertrag](../data-format/native-product-control-v1.md),
[vorherige Abbruchprüfung](native-update-abort-reconciliation.md). Striktes Datenschutzziel
unverändert; normale Produktintegration und Produkt-Q-01/Q-02 bleiben offen.

## Änderung

Bisher konnte eine abgebrochene Paketkopie beim nächsten `create_new` scheitern, nachdem bereits ein
neuer Pending-Record geschrieben war. Jetzt rekonstruiert der Broker aus der authentisierten
Kontrollhistorie die abgebrochenen Paket-SHAs. Kein neues Wireformat, keine Kennung aus Ordnernamen
oder Diagnosen als Autorität. Unter Kontrollschema 4 kann erneute explizite Vorbereitung desselben
Hashs dessen alte Kopie ersetzen, sofern dieser Hash **niemals aktiviert** war.

Dieselbe private Storelease und die gepinnten Vorfahren bleiben durchgehend gehalten:

1. Gepinnte Quelle, vollständigen erwarteten SHA und Paketvertrag prüfen.
2. Aktuellen Actor/Kopf, Jobleerstand, Recordbudget und erlaubten Zustandsübergang ohne Mutation
   prüfen.
3. Nur für einen authentisiert abgebrochenen Schema-4-Hash die exakt benannte Paketdatei öffnen:
   exklusiver Handle mit Löschrecht, `OPEN_REPARSE_POINT`, keine Pfadverfolgung. Attribute, private
   ACL und genau einen Hardlink prüfen. Verzeichnisse, Reparse-/Offline-/Recall-Attribute,
   Hardlinks, fremde ACLs und gesperrte Dateien bleiben abgewiesen.
4. Die geprüfte Datei über `SetFileInformationByHandle(FileDispositionInfo)` entfernen. Keine
   Pfadsuche, Ordnersäuberung, Wildcards oder Löschung aktiver/früher aktivierter Programme.
5. Neue Zieldatei exklusiv mit `create_new` anlegen, ACL und Quellenrewind prüfen; erst dann neuen
   Prepare-Record anhängen und die gepinnten Bytes kopieren/synchronisieren/verifizieren.

Kollisionen, fehlende Schreibmöglichkeit oder gesperrte/unsichere Altdateien scheitern dadurch vor
neuem Pending. Nach Beginn der signierten Kopierabsicht kann ein I/O-Fehler weiterhin ein
**beabsichtigtes, authentisiertes Pending** hinterlassen, das explizit abzubrechen ist. Fehler in
Record-/Kopfabschluss bleiben gesperrt; keine automatische Ketten-/Schlüsselreparatur.

Die Vorab-Erzeugung des Ziels gilt auch für Schema 2; seine alten Kopien werden nicht automatisch
entfernt. Schema 3 bleibt für Updates gesperrt. Ein unbekanntes Artefakt ohne authentisierten Abort
wird nicht gelöscht oder übernommen. Insbesondere kann ein Prozessende zwischen Zieldateianlage und
erstem Prepare-Record ein unautorisiertes Artefakt hinterlassen: ein weiterer Aufruf scheitert dann
ohne neues Pending. Allgemeines Orphan-Cleanup bleibt offen. Es gibt keinen Verzeichnis-Sweep und
keinen automatischen Neuimport.

## Prüfung

Drei neue Helfertests, insgesamt **56 aktive Tests**, vier bestehende Tests ignoriert; Clippy mit
Warnungen als Fehler und rustfmt bestanden:

- Vollständige, fehlende, gekürzte und manipulierte abgebrochene Kopien nach authentisiertem
  Wiederöffnen erneut vorbereiten; alter Kopf/falscher Actor/gesperrte Datei ändern nichts. Danach
  ist Aktivierung geprüft; erneute Vorbereitung eines aktivierten Hashs verändert dessen Datei
  nicht.
- Unbekannte vorhandene Paketkopie bleibt unangetastet; weder sofort noch nach Wiederöffnen Pending.
- Hardlink auf abgebrochenes Paket: keine Löschung, keine neue Autorität; beide Links bleiben
  erhalten.

Ein neuer reiner Helferbuild, keine App-/Transport-/Installeränderung. Gesicherte Programme unter
`%TEMP%/kraqel-package-retry-build-20261117-01/`:

- Broker `c10b1c3fe22c1cd33c6ee33f0a1b72a639a00610ed5a1b86f560bcd19532d0bd`.
- Synthetischer Kandidat `285850152b96812cdda8c22ea64164d1018725f28d952aa58042b7d16aa474a9`.
- Unveränderte Forschungs-App `6e56a51ed2e913a878a7935e039bd69628a49be25f0fbab7d02fce3a72fe8c70`.

Der Kandidat hat denselben Code plus statisches Overlay, keine beliebige Release-/Datenmigration.
Quellbasis vor Änderung: `d99d9486ac30665c8f7de0774bb9613f4d36da4e`; genaue Buildbindung im privaten
Manifest. Die echte Vorschauablage und ihre dort autorisierte EXE bleiben unverändert.

**Drei native Fälle** erweitern die gezielte Abbruchprüfung: Teilkopie, manipulierte Kopie und
Prozessende nach veröffentlichtem Abort. Nach Abbruch jeweils erneute Vorbereitung mit altem Kopf
und gesperrter Datei abgewiesen, Kontrollkette und Paketbytes unverändert. Danach erfolgreiche
Ersetzung/Vorbereitung desselben erwarteten Hashs, erneuter ausdrücklicher Abort und Recovery
derselben Sitzung. Keine native Aktivierung in dieser Serie; Aktivierung nach Retry im Helfertest
geprüft. Drei synthetische fremde Actor-Aufrufe werden absichtlich ausschließlich als negative
Prüfung gestartet und abgewiesen; keine GUI/Mutation. Der Updater startet keinen Kandidaten vor
Aktivierung.

57 Kontrollprozesse: 30 Erfolge, 25 erwartete Abweisungen, zwei harte Prozessabbrüche. Sechs GUIs,
36 tatsächliche private Dateiverweigerungen, drei authentisiert entfernte Profile. Unabhängiger
Nur-Lese-Audit bestätigt pro Fall 13 MAC-Records, zwei Abbrüche/eine erneute Vorbereitung,
Kopf-/Root-/Actorbindung und erhaltene Schlüssel/App-/Sitzungsbindung. Global drei verschiedene
Aufgaben und drei legitime Projektkopien mit je zwei Dokumenten, keine Replaykopie; SQLite,
Fremdschlüssel und Unicode-Inhalte passen. Cleanup-Wiederholung idempotent, Jobs am Ende leer.

## Reproduktion und Fortsetzung

```powershell
$savedBuild = Join-Path $env:TEMP 'kraqel-package-retry-build-20261117-01'
./scripts/runtime-isolation/check-update-reconciliation.ps1 `
  -Probe "$savedBuild/kraqel-isolation-probe.exe" `
  -ProbeSha256 'c10b1c3fe22c1cd33c6ee33f0a1b72a639a00610ed5a1b86f560bcd19532d0bd' `
  -App "$savedBuild/kraqel-desktop.exe" `
  -AppSha256 '6e56a51ed2e913a878a7935e039bd69628a49be25f0fbab7d02fce3a72fe8c70' -RetryAborted
cargo test --locked --manifest-path scripts/runtime-isolation/probe/Cargo.toml
cargo clippy --locked --manifest-path scripts/runtime-isolation/probe/Cargo.toml --all-targets -- -D warnings
```

Der Helfer erzeugt ausschließlich drei frische synthetische TEMP-Cases. Prozesse beendet, private
Rohbelege/Schlüssel/PIDs/Pfade/Journale lokal erhalten; nur bereinigte Aggregate in Git.
Format-/Diff-/ Hash-/Faktenprüfung grün. Kein vollständiger Appbuild, Workspace-Gesamtlauf oder neue
große Matrix.

Inzwischen liegt der [eigene frische ETW-Beleg](native-package-artifact-retry-etw.md) der oben
genannten unveränderten Programme vor, einschließlich Teilkopie/Abbruch/Retry, Aktivierung und
Wiederanlauf des verwalteten Kandidaten. Alte Belege bleiben an ihre Programme/Pfade/Lebenszeiten
gebunden; dieser funktionale JSON-Beleg bleibt getrennt ohne Netzmessung. Nächster begrenzter Scope:
Prozessabbrüche zwischen Artefaktentfernung, Zielanlage und Prepare-Record. Keine HTTP-/Inhalts-/
Upload- oder vollständige externe Dienstattestierung, keine Produktfreigabe.
