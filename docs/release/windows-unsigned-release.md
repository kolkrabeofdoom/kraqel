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

Dieses Verfahren erzeugt zunächst nur einen lokalen Release Candidate. Vor einem GitHub-Release:

1. `pnpm check` und Release-Build auf dem dokumentierten Commit ausführen.
2. Manifest, Authenticode-Status `NotSigned` und SHA-256 kontrollieren.
3. Setup, Prüfsummendatei und Manifest gemeinsam als **Entwurf** hochladen.
4. Installation, Start, Projektpersistenz und Deinstallation auf einer sauberen Windows-x64-
   Umgebung prüfen; der zurückgestellte Zweitrechnertest kann diesen Nachweis liefern.
5. Erst nach dokumentierter Abnahme Tag und Release bewusst veröffentlichen.

Ein lokaler Build ersetzt weder die Clean-VM-Abnahme noch die Prüfung des tatsächlich
heruntergeladenen GitHub-Artefakts. Hashes sind nützlich, aber sie tragen keine kleinen Roben und
sprechen keine richterlichen Urteile.

## Maßgebliche Quellen

- [Tauri: Windows Installer](https://v2.tauri.app/distribute/windows-installer/)
- [Tauri: Bundle- und Windows-Konfiguration](https://v2.tauri.app/reference/config/#windowsconfig)
- [ADR 0018: Unsignierter Windows-Releasepfad](../architecture/decisions/0018-unsigned-windows-release-path.md)
- [Windows-Installer-Smoke-Test](../testing/windows-installer-smoke.md)
