//! Transaction-bound preview of the rows removed by the document cascade.
use super::{
    DocumentError, DocumentKind, DocumentNode, DocumentRepository, delete_node, load_node,
    sibling_ids,
};
use rusqlite::{Transaction, params, types::ValueRef};
use sha2::{Digest, Sha256};

const SCOPE: &str =
    "WITH scope AS (SELECT id FROM document WHERE project_id=?1 AND (id=?2 OR parent_id=?2)) ";

#[derive(PartialEq, Eq)]
pub struct DeletePreview {
    selected: DocumentNode,
    parent: Option<DocumentNode>,
    children: Vec<DocumentNode>,
    related_counts: [usize; 6],
    contents: [u8; 32],
}

impl DeletePreview {
    #[must_use]
    pub fn selected(&self) -> &DocumentNode {
        &self.selected
    }
    #[must_use]
    pub fn parent(&self) -> Option<&DocumentNode> {
        self.parent.as_ref()
    }
    #[must_use]
    pub fn children(&self) -> &[DocumentNode] {
        &self.children
    }
    /// Story links, plot cards, timeline links, asset links, prompt drafts, draft profile links.
    #[must_use]
    pub fn related_counts(&self) -> [usize; 6] {
        self.related_counts
    }
}

impl DocumentRepository<'_> {
    /// Captures one consistent snapshot of the selected document and its deletion scope.
    ///
    /// # Errors
    /// Rejects stale navigation metadata and malformed document hierarchies.
    pub fn preview_delete(
        &mut self,
        selected: &DocumentNode,
    ) -> Result<DeletePreview, DocumentError> {
        let transaction = self.database.transaction()?;
        let result = snapshot(&transaction, selected)?;
        transaction.commit()?;
        Ok(result)
    }

    /// # Errors
    /// Rejects any changed snapshot before performing the deletion in the same transaction.
    pub fn confirm_delete(&mut self, preview: &DeletePreview) -> Result<(), DocumentError> {
        let transaction = self.database.transaction()?;
        if &snapshot(&transaction, &preview.selected)? != preview {
            return Err(DocumentError::RevisionConflict);
        }
        delete_node(&transaction, &preview.selected)?;
        transaction.commit()?;
        Ok(())
    }
}

fn snapshot(
    transaction: &Transaction<'_>,
    expected: &DocumentNode,
) -> Result<DeletePreview, DocumentError> {
    let project = expected.project_id;
    let selected = load_node(transaction, project, expected.id)?;
    if &selected != expected {
        return Err(DocumentError::RevisionConflict);
    }
    let parent = selected
        .parent_id
        .map(|id| load_node(transaction, project, id))
        .transpose()?;
    match (selected.kind, parent.as_ref()) {
        (DocumentKind::Chapter, None) => {}
        (DocumentKind::Scene, Some(p))
            if p.kind == DocumentKind::Chapter && p.parent_id.is_none() => {}
        _ => return Err(DocumentError::InvalidStoredData),
    }
    let children = sibling_ids(transaction, project, Some(selected.id))?
        .into_iter()
        .map(|id| load_node(transaction, project, id))
        .collect::<Result<Vec<_>, _>>()?;
    if (selected.kind == DocumentKind::Scene && !children.is_empty())
        || children.iter().any(|n| n.kind != DocumentKind::Scene)
    {
        return Err(DocumentError::InvalidStoredData);
    }
    let bindings = params![project.to_string(), selected.id.to_string()];
    let grandchildren: bool = transaction.query_row(
        "SELECT EXISTS (SELECT 1 FROM document WHERE project_id=?1 AND parent_id IN
          (SELECT id FROM document WHERE project_id=?1 AND parent_id=?2))",
        bindings,
        |r| r.get(0),
    )?;
    if grandchildren {
        return Err(DocumentError::InvalidStoredData);
    }

    // Bind complete row values. Edited prompt text or a replaced link must invalidate an
    // outstanding confirmation even if the displayed counts and document revisions match.
    let queries = [
        "SELECT * FROM document WHERE project_id=?1 AND id IN scope ORDER BY id",
        "SELECT * FROM link WHERE project_id=?1 AND document_id IN scope ORDER BY id",
        "SELECT * FROM plot_card WHERE project_id=?1 AND document_id IN scope ORDER BY document_id",
        "SELECT * FROM event_document_link WHERE project_id=?1 AND document_id IN scope ORDER BY event_id,document_id",
        "SELECT * FROM asset_link WHERE project_id=?1 AND target_type='document' AND target_id IN scope ORDER BY asset_id,target_id,role",
        "SELECT * FROM image_prompt_draft WHERE project_id=?1 AND source_document_id IN scope ORDER BY id",
        "SELECT * FROM image_prompt_draft_profile WHERE project_id=?1 AND draft_id IN
          (SELECT id FROM image_prompt_draft WHERE project_id=?1 AND source_document_id IN scope) ORDER BY draft_id,entity_id",
    ];
    let mut counts = [0usize; 7];
    let mut digest = Sha256::new();
    for (count, query) in counts.iter_mut().zip(queries) {
        let mut statement = transaction.prepare(&format!("{SCOPE}{query}"))?;
        let width = statement.column_count();
        let mut rows = statement.query(bindings)?;
        while let Some(row) = rows.next()? {
            for col in 0..width {
                match row.get_ref(col)? {
                    ValueRef::Null => digest.update([0]),
                    ValueRef::Integer(i) => {
                        digest.update([1]);
                        digest.update(i.to_le_bytes());
                    }
                    ValueRef::Text(text) => {
                        digest.update([2]);
                        digest.update(text.len().to_le_bytes());
                        digest.update(text);
                    }
                    _ => return Err(DocumentError::InvalidStoredData),
                }
            }
            *count += 1;
        }
        digest.update([3]); // Table boundary, outside the value tag range.
    }
    let [_, links, cards, events, assets, drafts, profiles] = counts;
    Ok(DeletePreview {
        selected,
        parent,
        children,
        related_counts: [links, cards, events, assets, drafts, profiles],
        contents: digest.finalize().into(),
    })
}
