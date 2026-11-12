# Rechnerwechsel und Arbeitsübergabe

Diese Anleitung trennt den öffentlichen Quellcode vom privaten Schreibprojekt. GitHub transportiert
Code und Projektdokumentation; ein `.kraqel`-Archiv transportiert Manuskript, Struktur und lokale
Medien. Wer beides verwechselt, erhält entweder fehlende Kapitel oder einen sehr literarischen
Commitverlauf.

## Diesen Rechner sauber verlassen

1. Nur auf einem geprüften, sauberen `main` enden:

   ```powershell
   cd D:\kraqel
   git switch main
   git pull --ff-only
   git status --short
   git log -1 --oneline
   ```

2. Für jedes Schreibprojekt in KRAQEL unter **Versionen** zuerst einen Snapshot erzeugen und danach
   **Projektarchiv exportieren** wählen. Das erzeugte `.kraqel`-Archiv an einem privaten,
   vertrauenswürdigen Ort ablegen. Manuskripte und API-Schlüssel gehören nicht ins öffentliche
   Repository.
3. Optional die Übertragung mit `Get-FileHash <Archiv.kraqel> -Algorithm SHA256` auf beiden Rechnern
   vergleichen.

## Auf dem anderen Rechner beginnen

Für einen neuen Checkout:

```powershell
git clone https://github.com/kolkrabeofdoom/kraqel.git D:\kraqel
cd D:\kraqel
corepack enable
pnpm install --frozen-lockfile
git status --short
pnpm --filter @kraqel/desktop typecheck
```

Für einen vorhandenen Checkout:

```powershell
cd D:\kraqel
git switch main
git pull --ff-only
pnpm install --frozen-lockfile
git status --short
```

Benötigt werden Git, Node.js 22 oder neuer, Corepack/pnpm gemäß `packageManager`, Rust 1.90 oder
neuer sowie unter Windows WebView2. Danach `AGENTS.md`, `docs/DEVELOPMENT_STATUS.md`, das nächste
GitHub-Issue und den zugehörigen ADR lesen.

Das Schreibprojekt wird auf der Startseite über **Projektarchiv importieren** als neue lokale Kopie
eingelesen. Die laufende Arbeitsdatenbank liegt unter `%LOCALAPPDATA%\org.kraqel.desktop\projects`
und wird bewusst nicht von Git synchronisiert. Cloud-API-Schlüssel liegen im
Betriebssystem-Schlüsselspeicher und müssen auf dem zweiten Rechner erneut eingerichtet werden.

## Verbindliches Arbeitsmuster

Ein Issue pro klarer Änderung, ein kurzer Branch, ein Pull Request mit grüner CI, danach Merge nach
`main`. Am Sitzungsende: Statusdokumente aktualisieren, offene manuelle Prüfungen ausdrücklich als
offen markieren, pushen und `git status --short` auf leer prüfen. So bleibt die Entwicklung
ortsunabhängig, ohne eine mystische Festplatte zum einzigen Wahrheitsministerium zu ernennen.
