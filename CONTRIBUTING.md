# Mitwirken

Danke für das Interesse an KRAQEL. Beiträge sind willkommen, wenn sie Datensouveränität,
Offlinefähigkeit und die Würde aller Beteiligten respektieren.

## Entwicklung

1. Issue wählen oder für größere Änderungen zuerst einen Vorschlag eröffnen.
2. Branch von `main` erstellen.
3. Abhängigkeiten mit `pnpm install --frozen-lockfile` installieren. Dabei wird der lokale Push-Hook
   automatisch eingerichtet.
4. Kleine Commits mit verständlicher Begründung schreiben.
5. `pnpm check` ausführen und relevante manuelle Prüfungen dokumentieren.
6. Pull Request mit Problem, Lösung, Risiken und Testnachweis eröffnen.

Neue Netzwerkzugriffe, Datenformate und Vertrauensgrenzen benötigen ein ADR unter
`docs/architecture/decisions/`. Sicherheitsprobleme bitte nicht öffentlich melden; siehe
`SECURITY.md`.

## Automatische Prüfung vor dem Push

Der versionierte Hook `.githooks/pre-push` führt vor jedem normalen `git push` die vollständige
Prüfung `pnpm check` aus: Formatierung, ESLint, Clippy, TypeScript, Hook-Tests, Vitest und
Rust-Tests. Ein Fehler oder fehlendes pnpm stoppt den Push. Die Prüfung formatiert keine Dateien
automatisch. Bei Formatfehlern zuerst `pnpm format` ausführen und die Korrekturen mit committen.

`pnpm install` aktiviert den Hook über `prepare` für diesen Checkout. Für bestehende Installationen
oder nach einer Installation ohne Lifecycle-Skripte genügt einmalig:

```powershell
pnpm hooks:install
```

Bei einer unveränderten bestehenden Installation kann pnpm den Installationslauf samt `prepare`
überspringen. Dann ebenfalls `pnpm hooks:install` verwenden.

Die Einrichtung setzt ausschließlich die lokale Git-Einstellung `core.hooksPath` auf `.githooks`.
Sie ist wiederholbar und benötigt keine zusätzlichen Pakete. Bereits konfigurierte fremde Hook-Pfade
oder eigene Hooks im bisherigen Standardordner bleiben erhalten. In diesem Fall weist die
automatische Einrichtung darauf hin und überspringt die Aktivierung; die manuelle Einrichtung meldet
einen Fehler. Bestehende Hooks müssen dann ausdrücklich mit der Projektprüfung verbunden werden. In
CI und bei Quellarchiven ohne Git-Metadaten wird die automatische Einrichtung übersprungen. Zum
Zurücknehmen der Einrichtung lässt sich die lokale Einstellung mit
`git config --local --unset core.hooksPath` entfernen.

Der Hook prüft den aktuellen Arbeitsbaum auf dem lokalen Betriebssystem. Änderungen vor dem Push
committen und den aktuell ausgecheckten Zweig pushen, damit Prüfung und übertragener Stand
zusammenpassen. Betriebssystemspezifische Unterschiede und die zusätzlichen Build- und
Browserprüfungen bleiben Aufgabe der GitHub-CI unter Windows und Linux. Lokale Hooks sind eine frühe
Rückmeldung und ersetzen diese CI nicht.

`pnpm test:hooks` prüft mit temporären lokalen Git-Repositories, dass ein fehlgeschlagener Check
einen echten Push verhindert, ein erfolgreicher Check ihn erlaubt und bestehende Hooks sowie
CI-Konfiguration erhalten bleiben. Es werden keine Test-Pushes an GitHub gesendet.
