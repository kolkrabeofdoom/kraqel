# ADR 0037 – Versionierter DOCX-Manuskriptaustausch für Drehbuchszenen

**27. November 2026 · Angenommen und auf dem Entwicklungszweig umgesetzt.**

ADR 0033 sieht für den kanonischen Austausch von Drehbuchrollen ein eigenes versioniertes Profil
vor. Das bestehende v1-Inventar erfasst ausschließlich Prosa. Leser*innen können am Einzug oder an
einem Absatzstil allein nicht erkennen, welche Szenenart und Rolle dauerhaft gemeint sind.
[Drehbuchvertrag](0033-screenplay-contract-proposal.md)

## Entscheidung

Sobald das Manuskript eine Drehbuchszene enthält, verwendet der getrennte Austauschmodus v2 für das
gesamte Manuskript. Eine eigene Inventarrolle unterscheidet die Drehbuchszene auch bei leerem Text.
Sechs Blattrollen binden die sichtbaren Absätze an Rolle und Reihenfolge. Die vorhandenen
Ressourcengrenzen gelten über das gesamte Dokument. Reine Prosamanuskripte verwenden weiter v1.
[Technischer Profilvertrag](../../data-format/DOCX_ROUNDTRIP_V2.md)

## Folgen

Die Umsetzung betrifft Writer, Reader und den transaktionalen Neuimport mit semantischem Rücklesen.
Bei gemischten, unbekannten oder widersprüchlichen Versionen endet der Austausch vor der
Veröffentlichung. Ein bestandener Office-Rundlauf für gewöhnliches DOCX belegt nur die Darstellung
und das Speichern der Rollenstile. Die eigenständigen Nachweise für den kanonischen Austausch stehen
unter [DB-07](../../testing/F08_M3_ACCEPTANCE_MATRIX.md#stand-der-drehbuchkriterien).
