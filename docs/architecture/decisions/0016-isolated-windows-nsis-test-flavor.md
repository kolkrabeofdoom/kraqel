# ADR 0016: Isolierte Windows-NSIS-Testvariante

Status: angenommen

## Kontext

M5 verlangt einen reproduzierbaren Windows-Release. Vor Codesignierung und öffentlicher Verteilung
müssen Installerzeugung, Installation, Anwendungspersistenz und Deinstallation prüfbar sein, ohne
reguläre KRAQEL-Installationen oder reale Projektdaten zu berühren. Tauri kann Windows-Pakete als
NSIS-Setup oder MSI/WiX erzeugen; beide Pfade besitzen unterschiedliche Voraussetzungen und sind
daher getrennt abzunehmen.

## Entscheidung

KQ-030 führt eine ausschließlich für Tests bestimmte Tauri-Konfigurationsvariante ein:

- eigener Produktname `KRAQEL Installer-Test` und eigene Bundle-ID
  `org.kraqel.desktop.installer-test`
- ausschließlich NSIS als Debug-Bundle im Modus `currentUser`
- ein festes, isoliertes App-Datenverzeichnis unter `%TEMP%`
- Tauri-Werkzeugcache unter `target/.tauri` durch `bundle.useLocalToolsDir`
- kein Zertifikat, kein Updater und keine Veröffentlichung des Artefakts

Ein Windows-Skript akzeptiert ausschließlich das erwartete unsignierte Artefakt. Es installiert in
ein fest benanntes temporäres Ziel, prüft den HKCU-Deinstallationseintrag, verwendet den echten
Tauri-Persistenztest und deinstalliert anschließend still. Jede rekursive Bereinigung ist auf zwei
explizite Verzeichnisse unter dem realen Windows-Temp-Pfad begrenzt.

## Folgen

- Der NSIS-Lebenszyklus ist lokal reproduzierbar, ohne Administrator*innenrechte oder produktive
  Datenpfade zu verwenden.
- Die Testvariante ist kein veröffentlichbares Releasepaket. Ihr Name, ihre Bundle-ID, ihr Datenpfad
  und ihr Debug-Profil unterscheiden sich absichtlich vom späteren Release.
- `useLocalToolsDir` vermeidet Laufwerks- und Verschlüsselungsgrenzen des globalen
  Benutzer*innen-Caches; `target/` bleibt unveröffentlicht und unversioniert.
- MSI/WiX, Codesignierung, SmartScreen, Updater, WebView2-Offlineverteilung, ARM64 und systemweite
  Installation benötigen eigene Entscheidungen und Nachweise.
- Der erfolgreiche Lauf auf dem Entwicklungsrechner ersetzt keine Abnahme auf einer frisch
  provisionierten Windows-VM.

Maßgebliche Tauri-Referenzen sind die Dokumentation zum
[Windows-Installer](https://v2.tauri.app/distribute/windows-installer/) und zur
[Bundle-Konfiguration](https://v2.tauri.app/reference/config/#nsisconfig).
