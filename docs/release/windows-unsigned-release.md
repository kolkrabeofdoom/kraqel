# Unsignierter Windows-Release

KRAQEL 0.1 wird als bewusst **unsigniertes NSIS-Setup für Windows x64** vorbereitet. Es installiert
für das aktuelle Benutzer*innenkonto, benötigt keine Administrator*innenrechte und enthält den
WebView2-Offlineinstaller. MSI, automatischer Updater, ARM64 und Codesignierung gehören nicht zu
diesem Releasepfad.

Diese Festlegung betrifft die Verteilung, nicht die GPL-Lizenz oder die technische Offenheit. Ein
Zertifikat ist kein moralischer Adelstitel, aber sein Fehlen hat konkrete Folgen: Windows kann eine
SmartScreen-Warnung anzeigen und die Identität der herausgebenden Stelle ist nicht kryptografisch an
die Datei gebunden.

## Voraussetzungen

- Windows x64
- sauberer, vollständig geprüfter und für den Release vorgesehener Git-Commit
- Node.js 22+, pnpm 11+ und Rust 1.85+
- Tauri-Windows-Buildwerkzeuge gemäß Repository- und Tauri-Dokumentation
- Netzwerkzugang **beim Build**, weil Tauri den WebView2-Offlineinstaller und gegebenenfalls
  NSIS-Werkzeuge bezieht

Die spätere Installation des erzeugten Setups benötigt keinen Netzwerkzugang. Der eingebettete
WebView2-Offlineinstaller vergrößert das Paket laut Tauri-Dokumentation um ungefähr 127 MB.

## Release Candidate bauen

Vom Repository-Stamm aus:

```powershell
pnpm check
pnpm build:release:windows
```

Das Skript bricht ab, wenn der Arbeitsbaum nicht sauber ist, die Versionen voneinander abweichen,
mehr als ein Setup gefunden wird oder das Ergebnis eine Authenticode-Signatur besitzt. Die Ausgabe
liegt im ignorierten Verzeichnis:

```text
artifacts/release/<Version>/windows-x64/<Commit>/
  KRAQEL_<Version>_windows-x64-setup.exe
  KRAQEL_<Version>_windows-x64-SHA256SUMS.txt
  KRAQEL_<Version>_windows-x64-build.json
```

Das JSON-Manifest hält Quellcommit, Dirty-Status, Zielplattform, Werkzeugversionen, Dateigröße,
SHA-256 und die maßgeblichen Distributionsparameter fest. Es beweist keine bit-identische
Reproduzierbarkeit: Zeitstempel, externe Werkzeuge und der jeweils bezogene
WebView2-Offlineinstaller können Bytes verändern. Es macht ein konkretes Artefakt jedoch eindeutig
prüfbar und den Build wiederholbar.

Nur für die Entwicklung kann ein nicht veröffentlichbares Paket aus lokalen Änderungen gebaut
werden:

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build-windows-release.ps1 `
  -AllowDirtyWorkingTree
```

Das Manifest kennzeichnet es als `development-only-dirty`. Eine vorhandene Ausgabe desselben Commits
wird nicht still überschrieben; nach bewusster Prüfung erlaubt `-Overwrite` den Ersatz.

## Prüfsumme verifizieren

Im Artefaktverzeichnis:

```powershell
$expected = (Get-Content .\KRAQEL_0.1.0_windows-x64-SHA256SUMS.txt).Split(' ')[0]
$sha256 = [Security.Cryptography.SHA256]::Create()
$stream = [IO.File]::OpenRead((Resolve-Path .\KRAQEL_0.1.0_windows-x64-setup.exe))
try {
  $actual = [BitConverter]::ToString($sha256.ComputeHash($stream)).Replace('-', '').ToLowerInvariant()
} finally {
  $stream.Dispose()
  $sha256.Dispose()
}
if ($actual -ne $expected) { throw 'SHA-256 stimmt nicht überein.' }
```

Bei einer Abweichung darf das Setup nicht gestartet werden. Eine passende Prüfsumme belegt nur, dass
Setup und veröffentlichter Hash zusammengehören. Werden beide über denselben kompromittierten Kanal
ersetzt, stellt SHA-256 keine Herausgeber*innenidentität her. Deshalb müssen Release-Asset, Manifest
und Prüfsumme aus dem privaten KRAQEL-GitHub-Repository stammen und der Quellcommit muss mit dem
vorgesehenen Release-Commit übereinstimmen.

## Veröffentlichtes Alpha zurückprüfen

KQ-036 prüfte zunächst den privaten Entwurf. KQ-037 veröffentlichte dieselben unveränderten Assets
als klar markiertes Prerelease `v0.1.0-alpha.1`; der gleichnamige Tag zeigt auf den geprüften
Build-Commit. Die drei Assets können in ein neues, ignoriertes Prüfverzeichnis geladen werden:

```powershell
gh release download v0.1.0-alpha.1 --dir artifacts/release-download/v0.1.0-alpha.1
pnpm test:release:windows `
  -ArtifactDirectory artifacts/release-download/v0.1.0-alpha.1 `
  -ExpectedCommit 5155ed5282cf217c9214cdd0426152ed7c5b8163
```

Das Abnahmeskript verlangt exakt Setup, Prüfsummendatei und Buildmanifest. Es prüft SHA-256,
Produkt, Version, sauberen Quellcommit, Current-User-Modus, Offline-WebView2, fehlende
Updaterartefakte und die nicht vorhandene PE-Zertifikatstabelle. Eine bereits registrierte reguläre
KRAQEL-Installation führt zum sicheren Abbruch.

Das Setup wird still ausschließlich nach `%TEMP%\kraqel-release-acceptance` installiert. Das Skript
prüft Version, Installationspfad und HKCU-Deinstallationseintrag und deinstalliert anschließend
wieder. Vorhandene reguläre App-Daten werden vor und nach dem Vorgang über Pfade, Größen,
Änderungszeiten und Inhalts-Hashes verglichen, aber weder ausgegeben noch gelöscht.

Auf dem Entwicklungsrechner wird die installierte Anwendung **nicht gestartet**, weil unter
`%LOCALAPPDATA%\org.kraqel.desktop` bereits schützenswerte App-Daten existieren und die
Releasekonfiguration absichtlich keinen Testdatenpfad besitzt. Start, SQLite-Persistenz und
SmartScreen bleiben daher Aufgaben für eine Clean-VM oder einen Zweitrechner.

## Installation und SmartScreen

Vor der Installation sind Quelle und Prüfsumme zu prüfen. Windows kann bei einem neuen, unsignierten
Setup melden, dass der Computer geschützt wurde oder die herausgebende Stelle unbekannt ist. Nur
wenn Quelle, Dateiname und SHA-256 stimmen, kann eine testende Person über `Weitere Informationen`
und `Trotzdem ausführen` fortfahren. Gibt es Zweifel, wird abgebrochen - Misstrauen ist hier keine
Charakterschwäche, sondern ein Sicherheitsfeature.

Das interaktive Setup installiert standardmäßig unter `%LOCALAPPDATA%` und legt den
Deinstallationseintrag unter `HKCU` an. Automatische Updates existieren nicht; eine neue Version
muss als neues geprüftes Setup bezogen werden. Downgrades werden abgelehnt.

## Veröffentlichung und Abnahme

Dieses Verfahren erzeugt zunächst nur einen privaten Release Candidate. Vor einer Veröffentlichung:

1. `pnpm check` und Release-Build auf dem dokumentierten Commit ausführen.
2. Manifest, Authenticode-Status `NotSigned` und SHA-256 kontrollieren.
3. Setup, Prüfsummendatei und Manifest gemeinsam als **Entwurf** hochladen und zurückprüfen.
4. Installation und Deinstallation des Rückdownloads in einem isolierten Ziel prüfen.
5. Start, Projektpersistenz, SmartScreen und Deinstallation auf einer sauberen Windows-x64-Umgebung
   prüfen; der zurückgestellte Zweitrechnertest kann diesen Nachweis liefern.
6. Erst nach dokumentierter Abnahme Tag und Release bewusst veröffentlichen.

Für Alpha 1 wurden die Schritte 1 bis 4 ausgeführt und das unveränderte Paket anschließend auf
ausdrückliche Freigabe als Prerelease veröffentlicht. Schritt 5 bleibt offen; deshalb ist
`v0.1.0-alpha.1` kein stabiles Release und darf nicht als vollständige M5-Abnahme interpretiert
werden.

Ein lokaler Build ersetzt weder die Clean-VM-Abnahme noch die Prüfung des tatsächlich
heruntergeladenen GitHub-Artefakts. Hashes sind nützlich, aber sie tragen keine kleinen Roben und
sprechen keine richterlichen Urteile.

## Maßgebliche Quellen

- [Tauri: Windows Installer](https://v2.tauri.app/distribute/windows-installer/)
- [Tauri: Bundle- und Windows-Konfiguration](https://v2.tauri.app/reference/config/#windowsconfig)
- [ADR 0018: Unsignierter Windows-Releasepfad](../architecture/decisions/0018-unsigned-windows-release-path.md)
- [Windows-Installer-Smoke-Test](../testing/windows-installer-smoke.md)
