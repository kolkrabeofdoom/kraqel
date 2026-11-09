import type {
  DocumentNode,
  EntityId,
  EntityKind,
  ProjectId,
  StoryEntity,
  StoryEntityDetails,
} from "@kraqel/domain";
import { useEffect, useMemo, useState, type FormEvent } from "react";
import {
  createStoryEntity,
  createStoryRelation,
  deleteStoryEntity,
  deleteStoryRelation,
  linkStoryEntity,
  listStoryEntities,
  loadStoryEntity,
  updateStoryEntity,
} from "./commands";

interface StoryBibleProps {
  readonly projectId: ProjectId;
  readonly documents: readonly DocumentNode[];
  readonly activeDocumentId: EntityId | null;
  readonly onSelectDocument: (id: EntityId) => void;
}

interface PropertyDraft {
  readonly id: number;
  readonly key: string;
  readonly value: string;
}

const entityKinds: readonly EntityKind[] = ["character", "location", "faction", "item"];

export function StoryBible({
  projectId,
  documents,
  activeDocumentId,
  onSelectDocument,
}: StoryBibleProps) {
  const [entities, setEntities] = useState<readonly StoryEntity[]>([]);
  const [allEntities, setAllEntities] = useState<readonly StoryEntity[]>([]);
  const [query, setQuery] = useState("");
  const [kindFilter, setKindFilter] = useState<EntityKind | "">("");
  const [selectedId, setSelectedId] = useState<EntityId | null>(null);
  const [details, setDetails] = useState<StoryEntityDetails | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const refreshList = async (preferredId: EntityId | null = selectedId) => {
    const [result, complete] = await Promise.all([
      listStoryEntities(projectId, query, kindFilter || null),
      listStoryEntities(projectId),
    ]);
    setEntities(result);
    setAllEntities(complete);
    const nextId =
      preferredId && result.some(({ id }) => id === preferredId)
        ? preferredId
        : (result[0]?.id ?? null);
    setSelectedId(nextId);
  };

  useEffect(() => {
    let active = true;
    setLoading(true);
    void Promise.all([
      listStoryEntities(projectId, query, kindFilter || null),
      listStoryEntities(projectId),
    ])
      .then(([result, complete]) => {
        if (!active) return;
        setEntities(result);
        setAllEntities(complete);
        setSelectedId((current) =>
          current && result.some(({ id }) => id === current) ? current : (result[0]?.id ?? null),
        );
      })
      .catch(() => active && setError("Die Story-Bibel konnte nicht durchsucht werden."))
      .finally(() => active && setLoading(false));
    return () => {
      active = false;
    };
  }, [kindFilter, projectId, query]);

  useEffect(() => {
    if (!selectedId) {
      setDetails(null);
      return;
    }
    let active = true;
    setDetails(null);
    void loadStoryEntity(projectId, selectedId)
      .then((result) => active && setDetails(result))
      .catch(() => active && setError("Der Story-Bibel-Eintrag konnte nicht geladen werden."));
    return () => {
      active = false;
    };
  }, [projectId, selectedId]);

  const handleCreated = async (entity: StoryEntity) => {
    setQuery("");
    setKindFilter("");
    setSelectedId(entity.id);
    setDetails(await loadStoryEntity(projectId, entity.id));
  };

  return (
    <section className="story-bible" aria-labelledby="story-bible-title">
      <header className="story-bible-header">
        <div>
          <span className="eyebrow">Weltdaten</span>
          <h2 id="story-bible-title">Story-Bibel</h2>
        </div>
        <CreateEntityForm projectId={projectId} onCreated={handleCreated} onError={setError} />
      </header>

      {error ? <p className="error-banner">{error}</p> : null}

      <div className="story-bible-grid">
        <aside className="entity-browser" aria-label="Story-Bibel durchsuchen">
          <label htmlFor="entity-search">Suche</label>
          <input
            id="entity-search"
            type="search"
            value={query}
            onChange={(event) => setQuery(event.currentTarget.value)}
            placeholder="Name oder Eigenschaft"
          />
          <label htmlFor="entity-kind-filter">Art</label>
          <select
            id="entity-kind-filter"
            value={kindFilter}
            onChange={(event) => setKindFilter(event.currentTarget.value as EntityKind | "")}
          >
            <option value="">Alle Arten</option>
            {entityKinds.map((kind) => (
              <option key={kind} value={kind}>
                {kindLabel(kind)}
              </option>
            ))}
          </select>
          <p className="result-count" aria-live="polite">
            {loading ? "Suche …" : `${entities.length} Einträge`}
          </p>
          <ul className="entity-list">
            {entities.map((entity) => (
              <li key={entity.id}>
                <button
                  type="button"
                  className={selectedId === entity.id ? "entity-button active" : "entity-button"}
                  aria-pressed={selectedId === entity.id}
                  onClick={() => setSelectedId(entity.id)}
                >
                  <span>{entity.name}</span>
                  <small>{kindLabel(entity.kind)}</small>
                </button>
              </li>
            ))}
          </ul>
        </aside>

        <div className="entity-details">
          {details ? (
            <EntityEditor
              key={details.entity.id}
              details={details}
              entities={allEntities}
              documents={documents}
              activeDocumentId={activeDocumentId}
              onSelectDocument={onSelectDocument}
              onChanged={async (entityId) => {
                await refreshList(entityId);
                setDetails(await loadStoryEntity(projectId, entityId));
              }}
              onDeleted={async () => {
                setDetails(null);
                await refreshList(null);
              }}
              onError={setError}
            />
          ) : (
            <p className="empty-editor">Lege einen Eintrag an oder wähle einen aus.</p>
          )}
        </div>
      </div>
    </section>
  );
}

function CreateEntityForm({
  projectId,
  onCreated,
  onError,
}: {
  readonly projectId: ProjectId;
  readonly onCreated: (entity: StoryEntity) => Promise<void>;
  readonly onError: (message: string) => void;
}) {
  const [name, setName] = useState("");
  const [kind, setKind] = useState<EntityKind>("character");

  const submit = async (event: FormEvent) => {
    event.preventDefault();
    try {
      const entity = await createStoryEntity({ projectId, kind, name, properties: {} });
      setName("");
      await onCreated(entity);
    } catch {
      onError("Der Story-Bibel-Eintrag konnte nicht angelegt werden.");
    }
  };

  return (
    <form className="create-entity-form" onSubmit={(event) => void submit(event)}>
      <label htmlFor="new-entity-kind">Art</label>
      <select
        id="new-entity-kind"
        value={kind}
        onChange={(event) => setKind(event.currentTarget.value as EntityKind)}
      >
        {entityKinds.map((value) => (
          <option key={value} value={value}>
            {kindLabel(value)}
          </option>
        ))}
      </select>
      <label htmlFor="new-entity-name">Name</label>
      <input
        id="new-entity-name"
        value={name}
        maxLength={240}
        required
        onChange={(event) => setName(event.currentTarget.value)}
      />
      <button type="submit" className="compact">
        + Eintrag
      </button>
    </form>
  );
}

function EntityEditor({
  details,
  entities,
  documents,
  activeDocumentId,
  onSelectDocument,
  onChanged,
  onDeleted,
  onError,
}: {
  readonly details: StoryEntityDetails;
  readonly entities: readonly StoryEntity[];
  readonly documents: readonly DocumentNode[];
  readonly activeDocumentId: EntityId | null;
  readonly onSelectDocument: (id: EntityId) => void;
  readonly onChanged: (entityId: EntityId) => Promise<void>;
  readonly onDeleted: () => Promise<void>;
  readonly onError: (message: string) => void;
}) {
  const [name, setName] = useState(details.entity.name);
  const [kind, setKind] = useState(details.entity.kind);
  const [properties, setProperties] = useState<readonly PropertyDraft[]>(() =>
    Object.entries(details.entity.properties).map(([key, value], id) => ({ id, key, value })),
  );
  const [nextPropertyId, setNextPropertyId] = useState(properties.length);
  const [relationKind, setRelationKind] = useState("");
  const relationTargets = useMemo(
    () => entities.filter(({ id }) => id !== details.entity.id),
    [details.entity.id, entities],
  );
  const [targetId, setTargetId] = useState<EntityId | "">(relationTargets[0]?.id ?? "");
  const sceneTitle = documents.find(({ id }) => id === activeDocumentId)?.title;

  const save = async (event: FormEvent) => {
    event.preventDefault();
    const propertyMap: Record<string, string> = {};
    for (const property of properties) {
      const key = property.key.trim();
      if (!key || key in propertyMap) {
        onError("Eigenschaftsnamen müssen eindeutig und nicht leer sein.");
        return;
      }
      propertyMap[key] = property.value;
    }
    try {
      await updateStoryEntity({
        projectId: details.entity.projectId,
        entityId: details.entity.id,
        kind,
        name,
        properties: propertyMap,
      });
      await onChanged(details.entity.id);
    } catch {
      onError("Der Story-Bibel-Eintrag konnte nicht gespeichert werden.");
    }
  };

  const addRelation = async (event: FormEvent) => {
    event.preventDefault();
    if (!targetId) return;
    try {
      await createStoryRelation(
        details.entity.projectId,
        details.entity.id,
        targetId,
        relationKind,
      );
      setRelationKind("");
      await onChanged(details.entity.id);
    } catch {
      onError("Die Beziehung konnte nicht angelegt werden.");
    }
  };

  return (
    <div>
      <form className="entity-edit-form" onSubmit={(event) => void save(event)}>
        <div className="entity-title-row">
          <div>
            <span className="eyebrow">{kindLabel(details.entity.kind)}</span>
            <h3>{details.entity.name}</h3>
          </div>
          <button
            type="button"
            className="danger secondary"
            onClick={() => {
              if (!window.confirm(`„${details.entity.name}“ samt Beziehungen löschen?`)) return;
              void deleteStoryEntity(details.entity.projectId, details.entity.id)
                .then(onDeleted)
                .catch(() => onError("Der Story-Bibel-Eintrag konnte nicht gelöscht werden."));
            }}
          >
            Löschen
          </button>
        </div>
        <div className="form-row">
          <label htmlFor="edit-entity-kind">Art</label>
          <select
            id="edit-entity-kind"
            value={kind}
            onChange={(event) => setKind(event.currentTarget.value as EntityKind)}
          >
            {entityKinds.map((value) => (
              <option key={value} value={value}>
                {kindLabel(value)}
              </option>
            ))}
          </select>
          <label htmlFor="edit-entity-name">Name</label>
          <input
            id="edit-entity-name"
            value={name}
            maxLength={240}
            required
            onChange={(event) => setName(event.currentTarget.value)}
          />
        </div>
        <fieldset>
          <legend>Freie Eigenschaften</legend>
          {properties.map((property) => (
            <div className="property-row" key={property.id}>
              <label className="visually-hidden" htmlFor={`property-key-${property.id}`}>
                Eigenschaftsname
              </label>
              <input
                id={`property-key-${property.id}`}
                placeholder="Eigenschaft"
                value={property.key}
                maxLength={80}
                onChange={(event) =>
                  setProperties((current) =>
                    current.map((item) =>
                      item.id === property.id ? { ...item, key: event.currentTarget.value } : item,
                    ),
                  )
                }
              />
              <label className="visually-hidden" htmlFor={`property-value-${property.id}`}>
                Eigenschaftswert
              </label>
              <input
                id={`property-value-${property.id}`}
                placeholder="Wert"
                value={property.value}
                maxLength={2000}
                onChange={(event) =>
                  setProperties((current) =>
                    current.map((item) =>
                      item.id === property.id
                        ? { ...item, value: event.currentTarget.value }
                        : item,
                    ),
                  )
                }
              />
              <button
                type="button"
                className="secondary compact"
                aria-label={`Eigenschaft ${property.key || "ohne Namen"} entfernen`}
                onClick={() =>
                  setProperties((current) => current.filter(({ id }) => id !== property.id))
                }
              >
                −
              </button>
            </div>
          ))}
          <button
            type="button"
            className="secondary compact"
            onClick={() => {
              setProperties((current) => [...current, { id: nextPropertyId, key: "", value: "" }]);
              setNextPropertyId((current) => current + 1);
            }}
          >
            + Eigenschaft
          </button>
        </fieldset>
        <button type="submit">Eintrag speichern</button>
      </form>

      <section className="detail-section" aria-labelledby="relations-title">
        <h4 id="relations-title">Beziehungen</h4>
        <ul className="relation-list">
          {details.outgoingRelations.map((relation) => (
            <li key={relation.id}>
              <span>
                <small>Ausgehend:</small> {details.entity.name} → <strong>{relation.kind}</strong> →{" "}
                {entityName(entities, relation.targetEntityId)}
              </span>
              <button
                type="button"
                className="secondary compact"
                onClick={() =>
                  void deleteStoryRelation(details.entity.projectId, relation.id)
                    .then(() => onChanged(details.entity.id))
                    .catch(() => onError("Die Beziehung konnte nicht gelöscht werden."))
                }
              >
                Entfernen
              </button>
            </li>
          ))}
          {details.incomingRelations.map((relation) => (
            <li key={relation.id}>
              <span>
                <small>Eingehend:</small> {entityName(entities, relation.sourceEntityId)} →{" "}
                <strong>{relation.kind}</strong> → {details.entity.name}
              </span>
            </li>
          ))}
        </ul>
        {relationTargets.length ? (
          <form className="relation-form" onSubmit={(event) => void addRelation(event)}>
            <label htmlFor="relation-kind">Beziehungsart</label>
            <input
              id="relation-kind"
              value={relationKind}
              maxLength={80}
              required
              onChange={(event) => setRelationKind(event.currentTarget.value)}
              placeholder="z. B. kennt"
            />
            <label htmlFor="relation-target">Ziel</label>
            <select
              id="relation-target"
              value={targetId}
              required
              onChange={(event) => setTargetId(event.currentTarget.value as EntityId)}
            >
              {relationTargets.map((entity) => (
                <option key={entity.id} value={entity.id}>
                  {entity.name}
                </option>
              ))}
            </select>
            <button type="submit" className="compact">
              Beziehung anlegen
            </button>
          </form>
        ) : (
          <p className="muted">Für eine Beziehung braucht es noch einen zweiten Eintrag.</p>
        )}
      </section>

      <section className="detail-section" aria-labelledby="backlinks-title">
        <div className="section-heading">
          <h4 id="backlinks-title">Szenen-Rückverweise</h4>
          {activeDocumentId && sceneTitle ? (
            <button
              type="button"
              className="compact"
              onClick={() =>
                void linkStoryEntity(details.entity.projectId, activeDocumentId, details.entity.id)
                  .then(() => onChanged(details.entity.id))
                  .catch(() => onError("Die Szene konnte nicht verknüpft werden."))
              }
            >
              Mit „{sceneTitle}“ verknüpfen
            </button>
          ) : null}
        </div>
        <ul className="backlink-list">
          {details.backlinks.map((backlink) => (
            <li key={`${backlink.documentId}-${backlink.role}`}>
              <button
                type="button"
                className="secondary"
                onClick={() => onSelectDocument(backlink.documentId)}
              >
                {backlink.documentTitle}
              </button>
            </li>
          ))}
        </ul>
        {!details.backlinks.length ? (
          <p className="muted">Noch keine Szene verweist auf diesen Eintrag.</p>
        ) : null}
      </section>
    </div>
  );
}

function entityName(entities: readonly StoryEntity[], id: EntityId): string {
  return entities.find((entity) => entity.id === id)?.name ?? "Unbekannter Eintrag";
}

function kindLabel(kind: EntityKind): string {
  switch (kind) {
    case "character":
      return "Figur";
    case "location":
      return "Ort";
    case "faction":
      return "Fraktion";
    case "item":
      return "Gegenstand";
  }
}
