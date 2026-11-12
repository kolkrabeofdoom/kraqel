# ADR 0021: Nativer, begrenzter Snapshotarchiv-Export

Status: angenommen

## Kontext

Der sichere Snapshot-v3-Import benötigt ein symmetrisches Austauschformat. Der vorhandene
Snapshotordner ist für lokales Git geeignet, aber außerhalb des verwalteten Projektordners weder
einfach weiterzugeben noch über die Oberfläche speicherbar.

## Entscheidung

- **Versionen → Projektarchiv exportieren** erzeugt immer zuerst einen aktuellen Snapshot v3.
- Rust packt ausschließlich reguläre Snapshotdateien in stabiler Pfadreihenfolge und ohne
  Verzeichniseinträge. Dateianzahl, unkomprimierte Summe und fertiges Archiv bleiben innerhalb der
  Importgrenzen.
- Der native Speicherdialog und die Veröffentlichung bleiben in Rust. React erhält nur Dateiname,
  Bytegröße und Snapshotzusammenfassung.
- Das Ziel wird über eine synchronisierte temporäre Geschwisterdatei veröffentlicht. Symlinks und
  Nicht-Dateien werden abgelehnt; ein Dialogabbruch bleibt neutral.

## Folgen

- Ein exportiertes `.kraqel`-Archiv kann unmittelbar wieder als neue Projektkopie importiert werden.
- Der Export aktualisiert bewusst den lokalen Snapshot- und Git-Arbeitsstand.
- Native Dialogdarstellung und tatsächliches Speichern sind unter Windows manuell bestätigt;
  Browser-E2E verwendet weiterhin nur den testexklusiven Command-Adapter.
- Git erhält `core.longpaths=true` ausschließlich pro internem Prozess, damit lange App-Datenpfade
  mit SHA-256-Assetnamen den vorgelagerten Snapshot nicht blockieren.

## Evidenz

- `crates/git-service/src/lib.rs`
- `apps/desktop/src-tauri/src/lib.rs`
- `apps/desktop/src/VersionControl.tsx`
- `apps/desktop/e2e/project-lifecycle.spec.ts`
