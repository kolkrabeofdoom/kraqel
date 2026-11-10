# ADR 0014: Lokale, editierbare Bildprompt-Entwürfe mit expliziten Quellen

Status: angenommen am 10. November 2026

## Kontext

F09, Modul V.1 und KQ-015 verlangen, Szenentext und Story-Bibel-Daten zu einem editierbaren
Bildprompt zusammenzustellen, lokal zu speichern und als Text zu exportieren. Manuskript, visuelle
Profile, Stilvorgaben und Prompts sind sensible Werkdaten. Ein Prompt-Entwurf darf daher weder eine
Providerfreigabe darstellen noch kanonische Inhalte oder Referenzbilder verändern.

## Entscheidung

Ein Prompt wird ausschließlich nach expliziter Auswahl genau einer Szene, mindestens eines visuellen
Profils und optional einer für alle gewählten Entitätsarten passenden Stilvorlage lokal
zusammengesetzt. Rust lädt die autoritativen Daten erneut, sortiert Profil-UUIDs deterministisch und
erzeugt einen vollständig editierbaren Arbeitsentwurf. Der Szenentext wird als Inhalt und nicht als
vertrauenswürdige Anweisung gekennzeichnet. Es existiert kein Provider- oder Netzwerkpfad.

SQLite-Migration 5 speichert den bearbeiteten Text, den Namen, die Szenen-UUID samt Quellrevision,
die Profil-UUIDs und die optionale Stilvorlagen-UUID. Beim erstmaligen Speichern muss die
Szenenrevision noch der angezeigten Zusammensetzung entsprechen. Spätere Text- und Namensänderungen
verändern die Quellen nicht still. Gelöschte Profile entfernen abhängige Entwürfe; das Löschen einer
Stilvorlage erhält den bereits bearbeiteten Text und leert nur deren Referenz.

Der UTF-8/LF-Export schreibt ausschließlich nach `exports/prompts/bildprompt-<uuid>.txt` im
verwalteten Projektordner. Snapshotformat v3 führt `prompt-drafts.json` ein und prüft Dokument-,
Profil- und Stilreferenzen. Die lesende Prüfung bleibt mit Snapshot v1 und v2 kompatibel.

## Folgen

- Eine sichtbare Zusammenstellung ist noch keine Zustimmung zu einer späteren Übertragung.
- Gespeicherte Entwürfe sind lokale, verwerfbare Arbeitsstände und keine kanonischen Assets.
- Szenenänderungen zwischen Zusammensetzung und erstmaligem Speichern erzeugen einen Konflikt.
- Prompttext ist auf 200.000 Zeichen, Namen auf 120 Zeichen und die Auswahl auf 32 Profile begrenzt.
- Eine künftige Capability-Registry muss Kontextvorschau und Einmalfreigabe getrennt modellieren;
  KQ-015 aktiviert keine Bildgenerierung.
