# ADR 0020: Sichere ZIP-Basis und begrenzter OOXML-Writer

Status: angenommen am 12. November 2026

## Kontext

KRAQEL verwendete `zip` 8.6.0 direkt zum Lesen fremder Snapshotarchive und indirekt über `docx-rs`
0.4.22 zur DOCX-Ausgabe. Nach Aktivierung des GitHub Dependency Graph meldeten fünf Advisories für
diese Laufzeitabhängigkeit eine kritische, drei hohe und eine mittlere Schwachstelle. Die reparierte
Reihe beginnt bei `zip` 9.0.5 und verlangt mindestens Rust 1.88.

Die veröffentlichte aktuelle `docx-rs`-Version bindet weiterhin `zip` 8.6. Ein Pin auf den
unveröffentlichten Upstream-Stand würde zusätzliche Umbauten und dieselbe höhere Rust-Mindestversion
übernehmen. KRAQEL nutzt aus der Bibliothek jedoch nur Absätze, Überschriften, Fett, Kursiv, harte
Umbrüche sowie textuelle Listen- und Zitatpräfixe.

## Entscheidung

Das Workspace-Minimum steigt bewusst von Rust 1.85 auf 1.90. Die reparierte ZIP-Reihe selbst
benötigt mindestens Rust 1.88; die bereits gelockte Tauri-Reihe setzt für das gesamte Workspace Rust
1.90 voraus. Snapshotimport und DOCX-Ausgabe verwenden dieselbe direkte Abhängigkeit `zip` ab 9.0.5;
der gelockte und geprüfte Stand ist 9.5.1, Default-Features bleiben deaktiviert und nur Deflate ist
freigegeben.

`docx-rs` wird entfernt. `kraqel-export` erzeugt ein begrenztes OOXML-Paket selbst: Content Types,
Paket- und Dokumentbeziehungen, Absatzstyles sowie `word/document.xml`. Der Writer erhält
ausschließlich das bereits validierte gemeinsame Exportmodell und erweitert dessen fachlichen Umfang
nicht.

Der DOCX-Test öffnet das ZIP-Paket, verlangt alle vereinbarten Bestandteile und prüft Text,
XML-Escaping, Auszeichnungen, Listen und Zitate im Dokumentteil. Zusätzlich wird ein QA-Artefakt mit
Microsoft Word geöffnet, nach PDF konvertiert und als gerenderte Seite visuell kontrolliert. Die
bestehenden adversarialen Snapshot-Importtests bleiben Teil der Abnahme.

## Folgen

- Die fünf `zip`-Advisories verschwinden aus dem normalen Cargo-Abhängigkeitsbaum.
- Die Mindest-Toolchain ist Rust 1.90; Build- und Releaseanleitungen nennen diese Grenze explizit.
- Eine große transitive Exportabhängigkeit entfällt, während KRAQEL Verantwortung für die kleine
  erzeugte OOXML-Teilmenge übernimmt.
- DOCX-Import, Roundtrip, Medien, Tabellen, Fußnoten, echte Word-Listen und beliebige OOXML-Teile
  bleiben außerhalb des aktuellen Vertrags.
- Erweiterungen des DOCX-Formats benötigen Strukturtests und eine erneute reale Renderingprüfung.
