//! Deterministische Kontinuitätsbefunde und rein lesende Änderungsreviews.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};

use kraqel_domain::{EntityId, EntityKind, ProjectId};
use serde::{Deserialize, Serialize};

/// Fachliche Dringlichkeit eines Befunds, nicht die Gewissheit einer Interpretation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FindingSeverity {
    /// Hilfreicher Strukturhinweis ohne behaupteten Widerspruch.
    Information,
    /// Mehrdeutige Datenlage, die redaktionell geprüft werden sollte.
    Warning,
    /// Strukturell nachweisbarer Konflikt.
    Conflict,
}

/// Art einer belegten Quelle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SourceKind {
    Document,
    Entity,
    PlotCard,
    TimelineEvent,
}

/// Konkreter, stabil adressierbarer Beleg für einen Befund.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceAnchor {
    pub kind: SourceKind,
    pub id: EntityId,
    pub label: String,
    pub document_id: Option<EntityId>,
    /// Zeichenposition im Klartext, nicht Byteposition.
    pub start: Option<u32>,
    /// Exklusive Zeichenposition im Klartext, nicht Byteposition.
    pub end: Option<u32>,
}

/// Ein reproduzierbarer, aus lokalen Projektdaten abgeleiteter Befund.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Finding {
    pub rule_id: String,
    pub severity: FindingSeverity,
    pub title: String,
    pub explanation: String,
    pub sources: Vec<SourceAnchor>,
}

/// Für die Regeln benötigte, speicherunabhängige Dokumentansicht.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentFact {
    pub id: EntityId,
    pub title: String,
    pub plain_text: String,
    pub linked_entity_ids: HashSet<EntityId>,
}

/// Für die Regeln benötigter Story-Bibel-Eintrag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityFact {
    pub id: EntityId,
    pub kind: EntityKind,
    pub name: String,
}

/// POV-Zuordnung einer Szenenkarte samt belegter Entitätsart.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PovFact {
    pub document_id: EntityId,
    pub document_title: String,
    pub entity_id: EntityId,
    pub entity_name: String,
    pub entity_kind: EntityKind,
}

/// Verknüpfung einer Szene mit einem Ereignis einer Zeitleiste.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimelinePlacementFact {
    pub timeline_id: EntityId,
    pub timeline_name: String,
    pub event_id: EntityId,
    pub event_title: String,
    pub document_id: EntityId,
    pub document_title: String,
}

/// Vollständiger, unveränderlicher Eingabestand eines Prüflaufs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectFacts {
    pub project_id: ProjectId,
    pub documents: Vec<DocumentFact>,
    pub entities: Vec<EntityFact>,
    pub pov_assignments: Vec<PovFact>,
    pub timeline_placements: Vec<TimelinePlacementFact>,
}

/// Führt alle deterministischen Offline-Regeln aus.
#[must_use]
pub fn analyze(facts: &ProjectFacts) -> Vec<Finding> {
    let mut findings = Vec::new();
    duplicate_names(facts, &mut findings);
    invalid_pov_assignments(facts, &mut findings);
    unlinked_mentions(facts, &mut findings);
    ambiguous_timeline_placements(facts, &mut findings);
    findings.sort_by(|left, right| {
        severity_rank(right.severity)
            .cmp(&severity_rank(left.severity))
            .then_with(|| left.rule_id.cmp(&right.rule_id))
            .then_with(|| left.title.cmp(&right.title))
    });
    findings
}

fn duplicate_names(facts: &ProjectFacts, findings: &mut Vec<Finding>) {
    let mut groups: BTreeMap<String, Vec<&EntityFact>> = BTreeMap::new();
    for entity in &facts.entities {
        groups
            .entry(entity.name.trim().to_lowercase())
            .or_default()
            .push(entity);
    }
    for entities in groups.values().filter(|group| group.len() > 1) {
        findings.push(Finding {
            rule_id: "story-bible.duplicate-name".to_owned(),
            severity: FindingSeverity::Warning,
            title: format!("Mehrdeutiger Story-Bibel-Name: {}", entities[0].name),
            explanation: "Mehrere Einträge besitzen denselben Namen. Verknüpfungen und Namensnennungen sind dadurch nicht eindeutig zuordenbar.".to_owned(),
            sources: entities
                .iter()
                .map(|entity| SourceAnchor {
                    kind: SourceKind::Entity,
                    id: entity.id,
                    label: entity.name.clone(),
                    document_id: None,
                    start: None,
                    end: None,
                })
                .collect(),
        });
    }
}

fn invalid_pov_assignments(facts: &ProjectFacts, findings: &mut Vec<Finding>) {
    for pov in facts
        .pov_assignments
        .iter()
        .filter(|pov| pov.entity_kind != EntityKind::Character)
    {
        findings.push(Finding {
            rule_id: "plot-board.pov-must-be-character".to_owned(),
            severity: FindingSeverity::Conflict,
            title: format!("POV von „{}“ ist keine Figur", pov.document_title),
            explanation: format!(
                "„{}“ ist als POV zugeordnet, in der Story-Bibel aber nicht als Figur klassifiziert.",
                pov.entity_name
            ),
            sources: vec![
                SourceAnchor {
                    kind: SourceKind::PlotCard,
                    id: pov.document_id,
                    label: format!("Szenenkarte: {}", pov.document_title),
                    document_id: Some(pov.document_id),
                    start: None,
                    end: None,
                },
                SourceAnchor {
                    kind: SourceKind::Entity,
                    id: pov.entity_id,
                    label: pov.entity_name.clone(),
                    document_id: None,
                    start: None,
                    end: None,
                },
            ],
        });
    }
}

fn unlinked_mentions(facts: &ProjectFacts, findings: &mut Vec<Finding>) {
    let ambiguous: BTreeSet<String> = facts
        .entities
        .iter()
        .fold(BTreeMap::<String, usize>::new(), |mut counts, entity| {
            *counts.entry(entity.name.trim().to_lowercase()).or_default() += 1;
            counts
        })
        .into_iter()
        .filter_map(|(name, count)| (count > 1).then_some(name))
        .collect();

    for document in &facts.documents {
        for entity in &facts.entities {
            if entity.name.chars().count() < 3
                || document.linked_entity_ids.contains(&entity.id)
                || ambiguous.contains(&entity.name.trim().to_lowercase())
            {
                continue;
            }
            if let Some((start, end)) = exact_name_mention(&document.plain_text, &entity.name) {
                findings.push(Finding {
                    rule_id: "story-bible.unlinked-mention".to_owned(),
                    severity: FindingSeverity::Information,
                    title: format!("„{}“ wird in „{}“ erwähnt", entity.name, document.title),
                    explanation: "Der Name steht im Szenentext, die Szene besitzt aber noch keinen entsprechenden Story-Bibel-Link. Das kann beabsichtigt sein.".to_owned(),
                    sources: vec![SourceAnchor {
                        kind: SourceKind::Document,
                        id: document.id,
                        label: format!("„{}“ in {}", entity.name, document.title),
                        document_id: Some(document.id),
                        start: Some(start),
                        end: Some(end),
                    }],
                });
            }
        }
    }
}

fn ambiguous_timeline_placements(facts: &ProjectFacts, findings: &mut Vec<Finding>) {
    let mut groups: HashMap<(EntityId, EntityId), Vec<&TimelinePlacementFact>> = HashMap::new();
    for placement in &facts.timeline_placements {
        groups
            .entry((placement.timeline_id, placement.document_id))
            .or_default()
            .push(placement);
    }
    for placements in groups.values().filter(|group| group.len() > 1) {
        findings.push(Finding {
            rule_id: "timeline.multiple-scene-placements".to_owned(),
            severity: FindingSeverity::Warning,
            title: format!(
                "„{}“ liegt mehrfach auf „{}“",
                placements[0].document_title, placements[0].timeline_name
            ),
            explanation: "Dieselbe Szene ist mehreren Ereignissen derselben Zeitleiste zugeordnet. Das kann mehrere Szenenereignisse ausdrücken, sollte aber auf zeitliche Eindeutigkeit geprüft werden.".to_owned(),
            sources: placements
                .iter()
                .map(|placement| SourceAnchor {
                    kind: SourceKind::TimelineEvent,
                    id: placement.event_id,
                    label: placement.event_title.clone(),
                    document_id: Some(placement.document_id),
                    start: None,
                    end: None,
                })
                .collect(),
        });
    }
}

fn exact_name_mention(text: &str, name: &str) -> Option<(u32, u32)> {
    text.match_indices(name).find_map(|(byte_start, matched)| {
        let before = text[..byte_start].chars().next_back();
        let after = text[byte_start + matched.len()..].chars().next();
        if before.is_some_and(char::is_alphanumeric) || after.is_some_and(char::is_alphanumeric) {
            return None;
        }
        let start = u32::try_from(text[..byte_start].chars().count()).ok()?;
        let length = u32::try_from(matched.chars().count()).ok()?;
        Some((start, start + length))
    })
}

const fn severity_rank(severity: FindingSeverity) -> u8 {
    match severity {
        FindingSeverity::Information => 0,
        FindingSeverity::Warning => 1,
        FindingSeverity::Conflict => 2,
    }
}

/// Zeilenart in einem rein lesenden Vorschlagsvergleich.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DiffLineKind {
    Context,
    Removed,
    Added,
}

/// Eine unveränderliche Diff-Zeile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffLine {
    pub kind: DiffLineKind,
    pub text: String,
}

/// Vergleich eines Vorschlags mit einer konkreten Szenenrevision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuggestionReview {
    pub document_id: EntityId,
    pub revision: u64,
    pub changed: bool,
    pub lines: Vec<DiffLine>,
}

/// Erzeugt einen kompakten Zeilenvergleich, ohne Daten zu speichern oder anzuwenden.
#[must_use]
pub fn review_suggestion(
    document_id: EntityId,
    revision: u64,
    original: &str,
    proposed: &str,
) -> SuggestionReview {
    // `split` bewahrt ein abschließendes Leersegment; dadurch bleibt auch eine geänderte
    // Schlusszeile im Review sichtbar.
    let original_lines: Vec<&str> = original.split('\n').collect();
    let proposed_lines: Vec<&str> = proposed.split('\n').collect();
    let prefix = original_lines
        .iter()
        .zip(&proposed_lines)
        .take_while(|(left, right)| left == right)
        .count();
    let suffix = original_lines[prefix..]
        .iter()
        .rev()
        .zip(proposed_lines[prefix..].iter().rev())
        .take_while(|(left, right)| left == right)
        .count();
    let mut lines = Vec::new();
    lines.extend(original_lines[..prefix].iter().map(|line| DiffLine {
        kind: DiffLineKind::Context,
        text: (*line).to_owned(),
    }));
    lines.extend(
        original_lines[prefix..original_lines.len() - suffix]
            .iter()
            .map(|line| DiffLine {
                kind: DiffLineKind::Removed,
                text: (*line).to_owned(),
            }),
    );
    lines.extend(
        proposed_lines[prefix..proposed_lines.len() - suffix]
            .iter()
            .map(|line| DiffLine {
                kind: DiffLineKind::Added,
                text: (*line).to_owned(),
            }),
    );
    lines.extend(
        original_lines[original_lines.len() - suffix..]
            .iter()
            .map(|line| DiffLine {
                kind: DiffLineKind::Context,
                text: (*line).to_owned(),
            }),
    );
    SuggestionReview {
        document_id,
        revision,
        changed: original != proposed,
        lines,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use kraqel_domain::{EntityId, EntityKind, ProjectId};

    use super::{
        DocumentFact, EntityFact, FindingSeverity, PovFact, ProjectFacts, SourceKind,
        TimelinePlacementFact, analyze, review_suggestion,
    };

    fn facts() -> ProjectFacts {
        let project_id = ProjectId::new();
        let document_id = EntityId::new();
        let location_id = EntityId::new();
        ProjectFacts {
            project_id,
            documents: vec![DocumentFact {
                id: document_id,
                title: "Ankunft".to_owned(),
                plain_text: "Ada wartet in Berlin.".to_owned(),
                linked_entity_ids: HashSet::new(),
            }],
            entities: vec![
                EntityFact {
                    id: EntityId::new(),
                    kind: EntityKind::Character,
                    name: "Ada".to_owned(),
                },
                EntityFact {
                    id: location_id,
                    kind: EntityKind::Location,
                    name: "Berlin".to_owned(),
                },
                EntityFact {
                    id: EntityId::new(),
                    kind: EntityKind::Item,
                    name: "berlin".to_owned(),
                },
            ],
            pov_assignments: vec![PovFact {
                document_id,
                document_title: "Ankunft".to_owned(),
                entity_id: location_id,
                entity_name: "Berlin".to_owned(),
                entity_kind: EntityKind::Location,
            }],
            timeline_placements: vec![],
        }
    }

    #[test]
    fn rules_distinguish_conflicts_warnings_and_information() {
        let findings = analyze(&facts());
        assert!(
            findings
                .iter()
                .any(|item| item.severity == FindingSeverity::Conflict)
        );
        assert!(
            findings
                .iter()
                .any(|item| item.severity == FindingSeverity::Warning)
        );
        let mention = findings
            .iter()
            .find(|item| item.rule_id == "story-bible.unlinked-mention")
            .expect("Ada mention should be reported");
        assert_eq!(mention.sources[0].kind, SourceKind::Document);
        assert_eq!(
            (mention.sources[0].start, mention.sources[0].end),
            (Some(0), Some(3))
        );
    }

    #[test]
    fn timeline_rule_is_explicitly_only_a_warning() {
        let mut input = facts();
        let timeline_id = EntityId::new();
        let document_id = input.documents[0].id;
        for title in ["Morgens", "Abends"] {
            input.timeline_placements.push(TimelinePlacementFact {
                timeline_id,
                timeline_name: "Haupthandlung".to_owned(),
                event_id: EntityId::new(),
                event_title: title.to_owned(),
                document_id,
                document_title: "Ankunft".to_owned(),
            });
        }
        let finding = analyze(&input)
            .into_iter()
            .find(|item| item.rule_id == "timeline.multiple-scene-placements")
            .expect("placement should be reported");
        assert_eq!(finding.severity, FindingSeverity::Warning);
        assert_eq!(finding.sources.len(), 2);
    }

    #[test]
    fn suggestion_review_is_a_read_only_line_diff() {
        let id = EntityId::new();
        let review = review_suggestion(id, 7, "Eins\nZwei\nDrei", "Eins\nNeu\nDrei");
        assert!(review.changed);
        assert_eq!(review.revision, 7);
        assert_eq!(review.lines.len(), 4);
    }

    #[test]
    fn suggestion_review_exposes_a_changed_trailing_newline() {
        let review = review_suggestion(EntityId::new(), 1, "Text", "Text\n");
        assert!(review.changed);
        assert_eq!(review.lines.len(), 2);
        assert_eq!(review.lines[1].kind, super::DiffLineKind::Added);
    }
}
