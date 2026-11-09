# ADR 0011: Deterministische Kontinuitätsregeln und revisionsgebundener Nur-Lese-Diff

Status: angenommen am 9. November 2026

## Kontext

F06 und KQ-012 verlangen regelbasierte Kontinuitätsbefunde mit Fundstellen sowie optional
KI-gestützte Hinweise. Vorschläge dürfen den kanonischen Text nie unaufgefordert ändern. Die
vorhandenen Dokument-, Story-Bibel-, Plotboard- und Timeline-Daten erlauben bereits belastbare
strukturelle Regeln; freie Prosa erlaubt ohne Sprachmodell hingegen keine seriöse allgemeine
Widerspruchserkennung.

## Entscheidung

`kraqel-continuity` arbeitet speicherunabhängig auf einer unveränderlichen Faktenprojektion. Die
Speicherschicht liest dafür Szenenklartext, Entitäten, Links, POV-Zuordnungen und Timeline-Links aus
SQLite. Regeln erzeugen reproduzierbare Befunde mit Regelkennung, Schweregrad und mindestens einem
stabilen Quellenanker. Hinweise, die auch eine legitime Modellierung ausdrücken können, werden als
Information oder Prüfhinweis formuliert, nicht als nachgewiesener Konflikt.

Der erste Umfang erkennt doppelte normalisierte Story-Bibel-Namen, ungültige POV-Arten, exakte nicht
verknüpfte Namensnennungen und Mehrfachzuordnungen einer Szene innerhalb derselben Timeline. Die
Prüfung wird bei ausdrücklicher Anforderung neu aus den autoritativen Daten berechnet. Befunde
werden in dieser Stufe nicht persistiert, damit keine veraltete zweite Wahrheit neben SQLite
entsteht. Eine spätere Laufhistorie für KI-Hinweise benötigt einen eigenen versionierten Vertrag.

Textentwürfe können gegen genau die zuvor freigegebene Szenenrevision verglichen werden. Rust lädt
den aktuellen Stand erneut; bei abweichender Revision entsteht ein Konflikt. Das Ergebnis ist ein
reiner Zeilendiff. Der Command besitzt keinen Schreibpfad, und die UI bietet keine automatische
Übernahme an.

## Folgen

- Die Offline-Prüfung ist ohne Provider, Netzwerk oder API-Schlüssel vollständig nutzbar.
- Fundstellen sind navigierbar; Textpositionen werden als Unicode-Zeichenpositionen ausgewiesen.
- Exakte Namensnennungen sind bewusst konservativ und erkennen keine Pronomen, Aliasse oder
  semantischen Widersprüche.
- KI-basierte Kontinuitätshinweise und deren persistente Laufprovenienz bleiben getrennte, noch
  nicht implementierte Arbeit.
- Ein grober Zeilendiff ist nachvollziehbar und begrenzt, ersetzt aber noch keinen interaktiven
  Teilübernahme-Editor.
