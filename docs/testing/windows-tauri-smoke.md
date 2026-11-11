# Windows-Tauri-Smoke-Test

Dieser Test prüft einen kleinen Offline-Kernablauf gegen die tatsächlich gebaute
Windows-Tauri-Anwendung. Er startet das eingebettete WebView über `tauri-driver`, legt ein Projekt
über die React-Oberfläche und versionierte IPC-Commands an, speichert Szenentext in SQLite, beendet
die Anwendung und bestätigt den Text nach einem zweiten Start.

Der eigene Smoke-Build verwendet Tauri `appDirectoriesOverride` und bindet seine Daten an
`$TEMP/kraqel-tauri-smoke`. Das Testskript prüft und entfernt ausschließlich dieses dedizierte
Verzeichnis. Reale KRAQEL-Projekte und die regulären Anwendungsdaten bleiben unberührt.

## Voraussetzungen

- Windows mit installierter Microsoft-Edge-WebView2-Laufzeit
- Node.js, pnpm und Rust gemäß `AGENTS.md`
- aktueller `tauri-driver`
- Microsoft Edge Driver in exakt der Version des installierten Microsoft Edge

Die Tauri-Dokumentation empfiehlt für den nativen Windows-WebDriver:

```powershell
cargo install tauri-driver --locked
cargo install --git https://github.com/chippers/msedgedriver-tool
& "$HOME/.cargo/bin/msedgedriver-tool.exe"
```

## Build und Ausführung

Der Build erzeugt bewusst noch kein Installationspaket:

```powershell
pnpm build:tauri:smoke:windows
```

Wenn beide Treiber in `PATH` liegen, genügt:

```powershell
pnpm test:tauri:windows
```

Explizite Pfade verhindern Mehrdeutigkeit auf Rechnern mit mehreren Treiberversionen:

```powershell
pnpm test:tauri:windows -- `
  --app D:\kraqel\target\debug\kraqel-desktop.exe `
  --tauri-driver C:\Werkzeuge\tauri-driver.exe `
  --edge-driver C:\Werkzeuge\msedgedriver.exe `
  --data-root "$env:TEMP\kraqel-tauri-smoke"
```

Der Lauf ist erfolgreich, wenn Start, Projektanlage, Autosave, eine isolierte `kraqel.sqlite3` und
das Wiederöffnen des gespeicherten Textes bestätigt wurden.

## Aussagegrenzen

Der Smoke-Test belegt einen realen Windows-Pfad durch WebView, Tauri-IPC, Rust und SQLite. Er ist
keine vollständige Funktions-, Barrierefreiheits- oder Releaseabnahme. Insbesondere bleiben offen:

- Installer und reproduzierbare Installation auf einem sauberen Zielsystem
- Codesignierung, Updatepfad und Deinstallation
- Screenreader- und vollständige Tastaturabnahme
- Pflichtenheft-Gesamttest mit 100 Szenen und 50 verlinkten Bildern
- echte Cloudprovider und Betriebssystem-Schlüsselspeicherdialoge
