# ADR 0018: Unsignierter Windows-Releasepfad

Status: angenommen

## Kontext

M5 verlangt einen reproduzierbaren Windows-Release mit dokumentierter Abnahme. KQ-030 belegt den
lokalen NSIS-Lebenszyklus ausschließlich mit einem isolierten Debug-Testprodukt; diese Variante darf
nicht veröffentlicht werden. Für 0.1 sind Installationsformat, Rechtebedarf, WebView2-Versorgung,
Signierung, Aktualisierung und Nachweisführung festzulegen.

KRAQEL ist offlinefähig und soll ohne Account- oder Netzzwang nutzbar bleiben. Das Tauri-Setup lädt
WebView2 standardmäßig bei Bedarf aus dem Netz. Tauri kann alternativ den WebView2-Offlineinstaller
einbetten, wodurch das Setup laut Projektdokumentation ungefähr 127 MB größer wird.

Eine Codesignatur ist für 0.1 ausdrücklich nicht vorgesehen. Damit fehlen Herausgeber*innenbindung
und SmartScreen-Reputation; eine SHA-256-Prüfsumme mildert nur Integritätsrisiken und ersetzt keine
Signatur.

## Entscheidung

KQ-035 führt einen getrennten Releasepfad ein:

- einziges Ziel ist ein NSIS-Setup für `x86_64-pc-windows-msvc`
- Installation im Modus `currentUser` ohne Administrator*innenrechte
- eingebetteter WebView2-Offlineinstaller für eine netzunabhängige Installation
- deutsche und englische NSIS-Texte
- keine Authenticode-Signatur, kein MSI und keine Updaterartefakte
- keine stillen Downgrades
- projektlokaler Tauri-Werkzeugcache unter `target/.tauri`
- ignoriertes Artefaktverzeichnis mit SHA-256 und maschinenlesbarem Buildmanifest

Das Buildskript akzeptiert standardmäßig nur einen sauberen Git-Arbeitsbaum und genau das erwartete
unsignierte Setup. Ein expliziter Dirty-Override erzeugt ausschließlich ein als nicht
veröffentlichbar gekennzeichnetes Entwicklungsartefakt. Es wird keine bit-identische
Reproduzierbarkeit behauptet; stattdessen werden Commit, Werkzeugversionen, Parameter und Hash des
konkreten Ergebnisses aufgezeichnet.

## Folgen

- KRAQEL 0.1 kann offline installiert und ohne erhöhte Rechte entfernt werden.
- Das Setup wird deutlich größer und der Build benötigt weiterhin Netzwerkzugang für externe
  Werkzeuge und den WebView2-Offlineinstaller.
- Windows kann SmartScreen-Warnungen anzeigen. Testende Personen benötigen eine klare Anleitung und
  müssen Quelle sowie Prüfsumme vor dem bewussten Start kontrollieren.
- SHA-256 erkennt eine veränderte Datei nur gegenüber einem vertrauenswürdig bezogenen Sollwert.
  Setup und Hash über denselben kompromittierten Kanal bleiben ein offenes Lieferkettenrisiko.
- Automatische Updates entfallen. Neue Versionen werden als vollständig neues, manuell geprüftes
  Setup verteilt.
- MSI/WiX, Signierung, ARM64, Store-Verteilung und Updater bleiben außerhalb von 0.1 und benötigen
  bei späterem Bedarf neue Entscheidungen.
- KQ-036 prüft den privaten GitHub-Entwurf, das erneut heruntergeladene Artefakt sowie dessen
  isolierte Installation und Deinstallation. Vollständig abgenommen ist der Release Candidate erst
  nach Anwendungsstart, SmartScreen- und Persistenzprüfung auf einer Clean-VM oder einem
  Zweitrechner.

## Evidenz

- `apps/desktop/src-tauri/tauri.release.conf.json`
- `scripts/build-windows-release.ps1`
- `scripts/windows-release-artifact-smoke.ps1`
- `docs/release/windows-unsigned-release.md`
- [Tauri: Windows Installer](https://v2.tauri.app/distribute/windows-installer/)
- [Tauri: Konfigurationsreferenz](https://v2.tauri.app/reference/config/#bundleconfig)
