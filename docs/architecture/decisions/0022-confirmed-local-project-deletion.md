# ADR 0022: Zweistufig bestätigte lokale Projektlöschung

Status: angenommen

## Kontext

Die lokale Projektübersicht konnte Projekte bislang nur öffnen. Eine Löschung ist notwendig, aber im
Gegensatz zu Dokumentkaskaden entfernt sie Datenbank, Medien, Backups, Snapshots und lokale
Git-Geschichte gemeinsam und ist nicht innerhalb der Anwendung rückgängig zu machen.

## Entscheidung

- Öffnen und Löschen sind getrennte, zugänglich benannte Aktionen je Projekt.
- Der erste Löschklick verändert nur die konkrete Schaltfläche zu **Wirklich löschen?**. Erst der
  zweite Klick sendet den Command mit explizitem Bestätigungsflag.
- Rust akzeptiert ausschließlich eine Projekt-UUID, löst deren Verzeichnis unter der kanonischen
  Projektwurzel auf und vergleicht vor dem Löschen die gespeicherte Datenbankkennung.
- Auswahl eines anderen Projekts oder Öffnen hebt die vorgemerkte Bestätigung auf. Fehler bleiben
  als zugängliche Meldung sichtbar.

## Folgen

- Eine versehentliche Ein-Klick-Löschung ist ausgeschlossen; eine bewusst bestätigte Löschung ist
  endgültig.
- Backup oder `.kraqel`-Export bleiben eine eigenständige Entscheidung der Nutzer*innen und werden
  nicht automatisch erzwungen.

## Evidenz

- `crates/storage/src/project_manager.rs`
- `apps/desktop/src-tauri/src/lib.rs`
- `apps/desktop/src/App.tsx`
- `apps/desktop/e2e/project-lifecycle.spec.ts`
