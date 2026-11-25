use super::{
    DocumentError, DocumentKind, DocumentNode, DocumentRepository, EditorDocument, EntityId,
    MAX_EDITOR_DOCUMENT_BYTES, OptionalExtension, Transaction, current_timestamp, kind_name,
    load_node, params, plain_text, sibling_ids, validate_content, validate_reorder_parent,
    validate_title,
};

impl DocumentRepository<'_> {
    /// Copies manuscript documents into new rows. Metadata links stay with the original IDs.
    ///
    /// # Errors
    /// The displayed source, ordered children and destination must still match in one transaction.
    /// An invalid title, unsupported source or any insert failure leaves the outline intact.
    pub fn duplicate_manuscript(
        &mut self,
        source: &DocumentNode,
        listed_children: &[&DocumentNode],
        destination: Option<&DocumentNode>,
        title: &str,
    ) -> Result<DocumentNode, DocumentError> {
        validate_title(title)?;
        let transaction = self.database.transaction()?;
        let project = source.project_id;
        let schema =
            transaction.pragma_query_value(None, "user_version", |r| r.get::<_, u32>(0))?;
        if !crate::supported_current_schema(schema) {
            return Err(DocumentError::InvalidStoredData);
        }
        if load_node(&transaction, project, source.id)? != *source {
            return Err(DocumentError::RevisionConflict);
        }
        let parent = match (source.kind, source.parent_id, destination) {
            (DocumentKind::Chapter, None, None) => None,
            (DocumentKind::Scene, Some(_), Some(target))
                if target.kind == DocumentKind::Chapter
                    && target.parent_id.is_none()
                    && target.project_id == project
                    && listed_children.is_empty() =>
            {
                validate_reorder_parent(&transaction, project, source.parent_id)?;
                if load_node(&transaction, project, target.id)? != *target {
                    return Err(DocumentError::RevisionConflict);
                }
                Some(target.id)
            }
            _ => return Err(DocumentError::InvalidParent),
        };
        let actual_children = sibling_ids(&transaction, project, Some(source.id))?;
        if !actual_children
            .iter()
            .copied()
            .eq(listed_children.iter().map(|child| child.id))
        {
            return Err(DocumentError::RevisionConflict);
        }

        let position: u32 = transaction.query_row(
            "SELECT COALESCE(MAX(position) + 1, 0) FROM document
             WHERE project_id = ?1 AND parent_id IS ?2",
            params![project.to_string(), parent.map(|id| id.to_string())],
            |r| r.get(0),
        )?;
        if source.kind == DocumentKind::Chapter
            && transaction.query_row(
                "SELECT EXISTS(SELECT 1 FROM document child JOIN document grandchild
             ON grandchild.project_id = child.project_id AND grandchild.parent_id = child.id
             WHERE child.project_id = ?1 AND child.parent_id = ?2)",
                params![project.to_string(), source.id.to_string()],
                |r| r.get::<_, bool>(0),
            )?
        {
            return Err(DocumentError::InvalidStoredData);
        }
        let timestamp = current_timestamp()?;
        let copy = copy_document(
            &transaction,
            source,
            parent,
            title.trim(),
            position,
            &timestamp,
            schema,
        )?;
        for (position, &child) in listed_children.iter().enumerate() {
            if child.kind != DocumentKind::Scene
                || child.parent_id != Some(source.id)
                || load_node(&transaction, project, child.id)? != *child
            {
                return Err(DocumentError::RevisionConflict);
            }
            copy_document(
                &transaction,
                child,
                Some(copy.id),
                &child.title,
                u32::try_from(position).map_err(|_| DocumentError::InvalidStoredData)?,
                &timestamp,
                schema,
            )?;
        }
        transaction.commit()?;
        Ok(copy)
    }
}

fn copy_document(
    transaction: &Transaction<'_>,
    source: &DocumentNode,
    parent_id: Option<EntityId>,
    title: &str,
    position: u32,
    timestamp: &str,
    schema: u32,
) -> Result<DocumentNode, DocumentError> {
    let json: String = transaction
        .query_row(
            "SELECT content_json FROM document WHERE project_id = ?1 AND id = ?2
         AND length(CAST(content_json AS BLOB)) <= ?3",
            params![
                source.project_id.to_string(),
                source.id.to_string(),
                i64::try_from(MAX_EDITOR_DOCUMENT_BYTES)
                    .map_err(|_| DocumentError::InvalidInput)?
            ],
            |r| r.get(0),
        )
        .optional()?
        .ok_or(DocumentError::InvalidStoredData)?;
    let content: EditorDocument =
        serde_json::from_str(&json).map_err(|_| DocumentError::InvalidStoredData)?;
    validate_content(&content).map_err(|_| DocumentError::InvalidStoredData)?;
    if content.version == kraqel_domain::SCREENPLAY_DOCUMENT_VERSION
        && (schema != crate::SCREENPLAY_DATABASE_VERSION.0 || source.kind != DocumentKind::Scene)
    {
        return Err(DocumentError::InvalidStoredData);
    }
    let id = EntityId::new();
    transaction.execute(
        "INSERT INTO document (id, project_id, parent_id, kind, title, position, revision,
          content_json, plain_text, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, ?7, ?8, ?9, ?9)",
        params![
            id.to_string(),
            source.project_id.to_string(),
            parent_id.map(|p| p.to_string()),
            kind_name(source.kind),
            title,
            position,
            json,
            plain_text(&content),
            timestamp
        ],
    )?;
    Ok(DocumentNode {
        id,
        project_id: source.project_id,
        parent_id,
        kind: source.kind,
        title: title.into(),
        position,
        revision: 0,
    })
}
