# KRAQEL

**KRAQEL's Recursive Authoring, Queries, Editions & Lore** ist ein quelloffenes, offlinefähiges
Autorenstudio für Romane, Drehbücher und Rollenspielmodule. Schreiben steht zuerst; KI bleibt
optional, transparent und immer unter menschlicher Kontrolle.

> Status: **0.1.0 Alpha 2** ist als Windows-x64-Prerelease verfügbar. Noch kein produktiver Einsatz
> ohne Backups - der Drache kennt inzwischen Undo, aber die Brandschutzordnung nur vom Hörensagen.

## Alpha herunterladen

[KRAQEL 0.1.0 Alpha 2](https://github.com/kolkrabeofdoom/kraqel/releases/tag/v0.1.0-alpha.2) enthält
ein bewusst unsigniertes NSIS-Setup für Windows x64 mit eingebettetem WebView2-Offlineinstaller.
Windows kann deshalb eine SmartScreen-Warnung anzeigen. Vor dem Start ist die beigefügte
SHA-256-Datei zu prüfen; Alpha-Versionen gehören nicht ohne aktuelle Backups an das einzige Exemplar
eines Manuskripts.

## Leitlinien

- **Offline first:** Schreiben, strukturieren, suchen und exportieren ohne Netz.
- **Datensouveränität:** Projekte liegen lokal in dokumentierten, portablen Formaten.
- **Explizite Freigabe:** Kein Text, Bild oder Metadatum verlässt ohne Zustimmung den Rechner.
- **Offene Architektur:** Austauschbare Provider, keine Konto- oder Cloudpflicht.
- **Nachvollziehbarkeit:** SQLite ist die lokale Autorität; Git erfasst deterministische Exporte.

## Mitgeliefertes Beispielprojekt

Über **Abenteuer ausprobieren** erzeugt KRAQEL vollständig offline eine frei bearbeitbare Kopie von
**Die Stadt, die den Dienstag verkaufte**. Das Rollenspielabenteuer demonstriert Szenenbaum,
Story-Bibel, Plotboard, drei Timelines und eine Medienbibliothek mit 21 Illustrationen. Jede
Betätigung legt ein neues Projekt mit eigener Kennung an; die eingebettete Vorlage bleibt
unverändert und es findet kein Download statt.

Die Originalinhalte stehen unter CC BY-SA 4.0, enthaltenes Daggerheart-SRD-Material unter der
Darrington Press Community Gaming License 2.0. Herkunft, Quellcommit und Aufbereitung sind in
[`examples/NOTICE.md`](examples/NOTICE.md) dokumentiert. Seit Alpha 2 ist die Vorlage im
Windows-Setup enthalten.

## Schnellstart

Vorausgesetzt werden Node.js 22+, pnpm 11+, Rust 1.85+ sowie die
[Tauri-Systemabhängigkeiten](https://v2.tauri.app/start/prerequisites/) für das jeweilige
Betriebssystem.

```powershell
pnpm install
pnpm dev       # Browser-UI
pnpm desktop   # Tauri-Desktop-App
pnpm check     # Format, Lint, Typen und Tests
```

## Aufbau

| Bereich               | Verantwortung                                              |
| --------------------- | ---------------------------------------------------------- |
| `apps/desktop`        | Tauri-Desktop-App mit React-Oberfläche                     |
| `packages/*`          | TypeScript-Domäne, Editor-, UI-, KI- und visuelle Verträge |
| `crates/*`            | Storage, Git, Export, Kontinuität und Asset-Verarbeitung   |
| `docs/architecture`   | Architektur, Entscheidungen und Bedrohungsmodell           |
| `docs/specifications` | Pflichtenheft und Abnahmekriterien                         |
| `docs/data-format`    | Offenes Austauschformat                                    |
| `examples`            | Eingebettetes, frei lizenziertes Beispielprojekt           |

Die aktuelle technische Quelle ist das Pflichtenheft unter `docs/specifications/`. Die nächste
umsetzbare Arbeit ist in `ROADMAP.md` und den GitHub-Issues beschrieben.

## Mitwirken

Siehe [CONTRIBUTING.md](CONTRIBUTING.md) und [AGENTS.md](AGENTS.md). Der Umgang soll solidarisch,
inklusiv und sachlich bleiben; Software ist kompliziert genug, wir müssen einander nicht auch noch
kompliziert machen.

## Lizenz

KRAQEL steht unter der [GNU General Public License v3.0 oder später](LICENSE).
