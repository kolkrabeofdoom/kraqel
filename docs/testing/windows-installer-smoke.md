# Windows-Installer-Smoke-Test

KQ-030 erzeugt ein eindeutig getrenntes, **unsigniertes und nicht zur Veröffentlichung bestimmtes**
NSIS-Testpaket. Der Test installiert es ohne Administrator*innenrechte in ein dediziertes temporäres
Verzeichnis, startet die installierte Anwendung über den bestehenden Tauri-Smoke-Test und
deinstalliert sie wieder.

## Voraussetzungen

- Windows x64 mit Microsoft Edge WebView2
- Node.js 22+, pnpm 11+, Rust 1.90+ und die Tauri-Windows-Buildabhängigkeiten
- `tauri-driver` sowie ein zur installierten Edge-Version passender `msedgedriver`

Die Treiber können wie beim [Windows-Tauri-Smoke-Test](windows-tauri-smoke.md) über
`KRAQEL_TAURI_DRIVER` und `KRAQEL_EDGE_DRIVER` oder als Parameter angegeben werden.

## Bauen und prüfen

```powershell
pnpm build:installer:test:windows
pnpm test:installer:windows
```

Der Build verwendet `tauri.installer-test.conf.json`. Er unterscheidet sich absichtlich vom
regulären Produkt:

- Produktname `KRAQEL Installer-Test`
- Bundle-ID `org.kraqel.desktop.installer-test`
- ausschließlich NSIS im Modus `currentUser`
- App-Daten unter `%TEMP%\kraqel-tauri-smoke-installer`
- Debug-Build und keine Codesignatur

`bundle.useLocalToolsDir` legt die von Tauri geprüften NSIS-Werkzeuge unter `target/.tauri` ab.
Dadurch bleiben Extraktion und atomare Umbenennung auf dem Laufwerk des Repositorys, auch wenn der
Windows-Benutzer*innen-Cache auf einem anderen oder verschlüsselten Laufwerk liegt. Der lokale
Werkzeugcache wird wie alle `target/`-Inhalte nicht versioniert.

Das Prüfskript akzeptiert nur den erwarteten Artefaktnamen und ein Artefakt ohne eingebettete
Authenticode-Signatur. Es installiert still nach `%TEMP%\kraqel-installer-smoke`, prüft den
HKCU-Deinstallationseintrag, führt den echten WebView-/Tauri-/Rust-/SQLite-Persistenztest aus und
ruft anschließend den NSIS-Deinstaller still auf. Löschoperationen sind auf genau die beiden
genannten temporären Verzeichnisse begrenzt.

Für explizite Treiberpfade können die Umgebungsvariablen pro Sitzung gesetzt werden:

```powershell
$env:KRAQEL_TAURI_DRIVER = 'C:\tools\tauri-driver.exe'
$env:KRAQEL_EDGE_DRIVER = 'C:\tools\msedgedriver.exe'
pnpm test:installer:windows
```

## Aussagegrenzen

Der Test belegt auf dem ausgeführten Windows-System:

- Erzeugung und Metadaten des unsignierten NSIS-Testartefakts
- Installation ohne Administrator*innenrechte in ein isoliertes Ziel
- Registrierung und Entfernen des Deinstallationseintrags
- Start der installierten Anwendung sowie den KQ-029-Nachweis für IPC, Autosave, SQLite und
  Wiederöffnen im expliziten App-Datenpfad
- Entfernung der installierten Programmdatei und sichere Bereinigung der Testdaten

Er belegt ausdrücklich nicht:

- eine Abnahme auf einer frisch provisionierten VM oder einem weiteren Windows-Rechner
- MSI/WiX; dessen Build benötigt unter Windows gegebenenfalls die optionale VBScript-Komponente
- Codesignierung, SmartScreen-Reputation, Updater oder Microsoft-Store-Verteilung
- systemweite Installation, Offline-Mitlieferung von WebView2 oder ARM64
- vollständige Fach-, Screenreader- oder Release-Abnahme

Vor einem öffentlichen 0.1-Release müssen diese Punkte nach ihrem jeweiligen Risikoprofil getrennt
geprüft werden. Ein erfolgreicher lokaler Installer-Test ist ein Beleg, kein magischer Umhang der
Unfehlbarkeit.
