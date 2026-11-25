# Abhängigkeitswarnungen vom 25. November 2026

GitHub meldete beim lesenden Abruf drei offene Dependabot-Warnungen für main, Revision `7adb2b8b952e83450d0d5bfe63ac3e0193aad85e`. Verglichen wurde der lokale Entwicklungsstand `a4b5a2cf4842b67d25108bf5b2e5322339889f01`. Die geprüften Lockfiles entsprachen diesem Commit. Die Bewertung ergänzt die [vorherige Plattformprüfung](../WORKSTATION_HANDOFF.md) durch erneuten Abgleich mit den gemeldeten Versionsbereichen.

| Meldung | main | Entwicklungsstand und Maßnahme |
| --- | --- | --- |
| [#8 · hoch](https://github.com/kolkrabeofdoom/kraqel/security/dependabot/8) | Der PDF-Export bindet über printpdf 0.7.0 den PDF-Parser lopdf 0.31.0 ein. Die gemeldete Stapelerschöpfung betrifft das Parsen tief verschachtelter PDF-Objekte. Ein Import beliebiger Fremd-PDFs ist für diesen Pfad nicht belegt. | printpdf 0.12.14 und die KRAQEL-Rückleseprüfung verwenden lopdf 0.47.0. Die Prüfung verarbeitet die eigene Ausgabe. Der Fix liegt ab 0.42.0 vor. Den geprüften Elternwechsel bei der Integration nach main übernehmen. [RustSec](https://rustsec.org/advisories/RUSTSEC-2026-0187.html) |
| [#7 · mittel](https://github.com/kolkrabeofdoom/kraqel/security/dependabot/7) | glib 0.18.5 im GTK-/WebKitGTK-Abhängigkeitsbaum. Betroffen sind unsichere Iteratorimplementierungen von VariantStrIter. | Derselbe Stand. Die bestehende Plattformprüfung weist diesen Pfad Linux zu. Für Windows x64 ist er nicht Teil des Desktop-Baums. Die Linux-Bewertung bleibt offen, ein Aufruf des betroffenen Iterators aus Produktdaten ist unbestätigt. Eine mit GTK/WebKitGTK kompatible Aktualisierung über die Elternpakete auf glib ab 0.20 oder einen belegten Backport verfolgen. [RustSec](https://rustsec.org/advisories/RUSTSEC-2024-0429.html) |
| [#6 · mittel](https://github.com/kolkrabeofdoom/kraqel/security/dependabot/6) | typescript-estree 8.72.1 verwendet minimatch 10.2.6 für Entwicklungswerkzeuge. Betroffen ist die Verarbeitung bestimmter Glob-Muster im Bereich 7.0.0 bis vor 11.0.1. | typescript-estree 8.73.0 verwendet 11.0.9. Die getrennten ESLint-Kopien 3.1.5 liegen auf beiden Zweigen außerhalb des gemeldeten Bereichs. Den vorhandenen Elternwechsel nach main übernehmen. [Herstellerhinweis](https://github.com/isaacs/minimatch/security/advisories/GHSA-g3hv-75wg-m249) |

SHA-256 der geprüften Root-Lockfiles (main, Entwicklung):

- `Cargo.lock`: `ae83b4a75d02a1a35f910955423ff374ebf549e5df51ce32fed024c846b82fdd`, `3b565437acf37c9a7f869b27471e9f4708539fdddfc60ba3a55d8f24ef9b4a70`
- `pnpm-lock.yaml`: `f7412c5642731ffa9339e973276a10a475ba23412c90fb0daec904f5b6f44afa`, `ec3323771d2a9a6359dd086afc18f6ec1be5f7f03fa6d8067ee77b9f22838e00`

Die getrennten Cargo-Lockfiles der Forschungsprogramme im Entwicklungsstand enthalten keine der drei Abhängigkeiten. Alle Befunde beziehen sich auf die angegebenen Quellenstände. Der Status der veröffentlichten Installer und das Schließen der GitHub-Meldungen sind eigenständige Freigaben.
