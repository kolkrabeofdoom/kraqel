# ADR 0017: Isolierte Accessibility-Testvariante

Status: angenommen

Datum: 11. November 2026

## Kontext

Die Autosave-Fehleransage muss in einer echten Tauri-WebView mit assistiver Technik geprüft werden,
ohne absichtlich SQLite, Dateiberechtigungen oder reale Projekte zu beschädigen. Ein Fehlerhaken im
Produktionscommand oder in Rust würde die Sicherheitsgrenze unnötig erweitern.

## Entscheidung

KQ-034 führt eine ausschließlich für manuelle Entwicklungstests bestimmte Tauri-Konfiguration ein:

- `tauri.accessibility-test.conf.json` verwendet einen eigenen Produktnamen, eine eigene Bundle-ID
  und einen temporären App-Datenpfad.
- Die Variante erzeugt kein Bundle und ist kein veröffentlichbares Artefakt.
- Vite läuft im Modus `e2e`; nur dann ersetzt der vorhandene Browser-Command-Mock die
  `@tauri-apps/api/core`-Grenze.
- Ein Queryparameter bewaffnet im Mock genau den nächsten `save_document`-Aufruf. Nach dem Fehler
  entfernt sich der Schalter selbst; die folgende Bearbeitung kann normal gespeichert werden.
- Produktionsbuild, regulärer Tauri-Devmodus, Rust-Commands und SQLite kennen diesen Fehlerhaken
  nicht.

Der wiederholbare Einstieg lautet:

```powershell
pnpm test:accessibility:manual:windows
```

## Folgen

- Die sichtbare und hörbare Fehlerbehandlung kann ohne Gefährdung lokaler Daten in der echten
  Windows-WebView geprüft werden.
- Der Test belegt React-, WebView- und Screenreader-Verhalten, aber ausdrücklich keinen realen
  SQLite-Schreibfehler oder dessen Betriebssystemursache.
- Änderungen an Testname, Kennung, Datenpfad, Bundle-Schalter oder Mock-Alias müssen diese Isolation
  bewahren.
- Die Variante darf weder signiert noch verteilt noch als Releasepaket bezeichnet werden. Auch ein
  sehr überzeugend auftretender Testdoppelgänger erhält keinen Presseausweis.

## Evidenz

- Der Playwright-Fall erzwingt einen Fehler, bestätigt den ungespeicherten Zustand und prüft den
  erfolgreichen Wiederholungsversuch.
- Die manuelle KQ-034-Prüfung bestätigte dieselbe Statusfolge mit Windows-Sprachausgabe in der
  isolierten Tauri-WebView.
- Ein weiterer Playwright-Fall und die manuelle Windows-Prüfung decken reduzierte Bewegung ab.
