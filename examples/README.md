# Beispiele

`die-stadt-die-den-dienstag-verkaufte.kraqel` ist die in KRAQEL eingebettete, vollständig offline
verfügbare Beispielvorlage. Sie enthält 20 Dokumente, 36 Story-Bibel-Einträge, ein Plotboard, drei
Timelines mit 25 Ereignissen, 13 visuelle Profile und 21 optimierte Illustrationen. Beim Anlegen
entsteht stets eine neue, frei bearbeitbare Projektkopie mit eigener UUID.

Herkunft, Lizenzaufteilung, Quellcommit, Transformationen und SHA-256 stehen in
[`NOTICE.md`](NOTICE.md). Das Archiv wird reproduzierbar aus einem sauberen Checkout des dort
genannten Commits erzeugt:

```powershell
pnpm build:example
# oder bei abweichendem Nachbarpfad:
node scripts/build-example-project.mjs D:\Pfad\zum\Abenteuer
```

Der Build benötigt Git LFS und Rust. Er verweigert abweichende Quellcommits sowie lokale Änderungen.
Neue Beispiele müssen künstlich oder nachvollziehbar frei lizenziert sein; private Manuskripte und
lizenzrechtliches Schrödinger-Material bleiben draußen.
