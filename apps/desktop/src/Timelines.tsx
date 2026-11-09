import type {
  DocumentNode,
  EntityId,
  ProjectId,
  StoryEntity,
  Timeline,
  TimelineDetails,
  TimelineMode,
} from "@kraqel/domain";
import { useEffect, useState, type FormEvent } from "react";
import {
  createTimeline,
  createTimelineEvent,
  deleteTimeline,
  deleteTimelineEvent,
  linkTimelineEvent,
  listStoryEntities,
  listTimelines,
  loadTimeline,
  reorderTimelineEvents,
} from "./commands";

export function Timelines({
  projectId,
  documents,
  activeDocumentId,
  onSelectDocument,
}: {
  readonly projectId: ProjectId;
  readonly documents: readonly DocumentNode[];
  readonly activeDocumentId: EntityId | null;
  readonly onSelectDocument: (id: EntityId) => void;
}) {
  const [timelines, setTimelines] = useState<readonly Timeline[]>([]);
  const [selectedId, setSelectedId] = useState<EntityId | null>(null);
  const [details, setDetails] = useState<TimelineDetails | null>(null);
  const [entities, setEntities] = useState<readonly StoryEntity[]>([]);
  const [name, setName] = useState("");
  const [mode, setMode] = useState<TimelineMode>("relative");
  const [eventTitle, setEventTitle] = useState("");
  const [timeValue, setTimeValue] = useState("");
  const [linkEntityId, setLinkEntityId] = useState<EntityId | "">("");
  const [error, setError] = useState<string | null>(null);
  const refreshTimelines = async (preferred: EntityId | null = selectedId) => {
    const result = await listTimelines(projectId);
    setTimelines(result);
    setSelectedId(
      preferred && result.some(({ id }) => id === preferred) ? preferred : (result[0]?.id ?? null),
    );
  };
  const refreshDetails = async () => {
    if (selectedId) setDetails(await loadTimeline(projectId, selectedId));
  };

  useEffect(() => {
    let active = true;
    void Promise.all([listTimelines(projectId), listStoryEntities(projectId)])
      .then(([lines, lore]) => {
        if (active) {
          setTimelines(lines);
          setEntities(lore);
          setSelectedId(lines[0]?.id ?? null);
          setLinkEntityId(lore[0]?.id ?? "");
        }
      })
      .catch(() => active && setError("Die Zeitebenen konnten nicht geladen werden."));
    return () => {
      active = false;
    };
  }, [projectId]);
  useEffect(() => {
    if (!selectedId) {
      setDetails(null);
      return;
    }
    let active = true;
    void loadTimeline(projectId, selectedId)
      .then((value) => active && setDetails(value))
      .catch(() => active && setError("Die Zeitebene konnte nicht geladen werden."));
    return () => {
      active = false;
    };
  }, [projectId, selectedId]);

  const addTimeline = async (event: FormEvent) => {
    event.preventDefault();
    try {
      const created = await createTimeline(projectId, name, mode);
      setName("");
      await refreshTimelines(created.id);
    } catch {
      setError("Die Zeitebene konnte nicht angelegt werden.");
    }
  };
  const addEvent = async (event: FormEvent) => {
    event.preventDefault();
    if (!details) return;
    try {
      await createTimelineEvent(
        projectId,
        details.timeline.id,
        eventTitle,
        "",
        details.timeline.mode === "absolute" ? timeValue : null,
      );
      setEventTitle("");
      setTimeValue("");
      await refreshDetails();
    } catch {
      setError("Das Ereignis konnte nicht angelegt werden.");
    }
  };
  const move = async (id: EntityId, delta: number) => {
    if (!details) return;
    const ids = details.events.map((item) => item.id);
    const index = ids.indexOf(id);
    const target = index + delta;
    if (target < 0 || target >= ids.length) return;
    [ids[index], ids[target]] = [ids[target]!, ids[index]!];
    await reorderTimelineEvents(projectId, details.timeline.id, ids);
    await refreshDetails();
  };
  const sceneTitle = documents.find(({ id }) => id === activeDocumentId)?.title;
  const entityName = (id: EntityId) =>
    entities.find((item) => item.id === id)?.name ?? "Unbekannter Eintrag";
  const documentTitle = (id: EntityId) =>
    documents.find((item) => item.id === id)?.title ?? "Unbekannte Szene";

  return (
    <section className="structure-view" aria-labelledby="timeline-title">
      <header className="structure-header">
        <div>
          <span className="eyebrow">Zeitmodell</span>
          <h2 id="timeline-title">Parallele Timelines</h2>
        </div>
        <form className="inline-form" onSubmit={(event) => void addTimeline(event)}>
          <label htmlFor="timeline-name">Zeitebene</label>
          <input
            id="timeline-name"
            value={name}
            required
            maxLength={120}
            onChange={(event) => setName(event.currentTarget.value)}
          />
          <label htmlFor="timeline-mode">Ordnung</label>
          <select
            id="timeline-mode"
            value={mode}
            onChange={(event) => setMode(event.currentTarget.value as TimelineMode)}
          >
            <option value="relative">Relativ</option>
            <option value="absolute">Absolut</option>
          </select>
          <button type="submit" className="compact">
            + Zeitebene
          </button>
        </form>
      </header>
      {error ? <p className="error-banner">{error}</p> : null}
      <div className="timeline-layout">
        <aside className="timeline-list" aria-label="Zeitebenen">
          <ul>
            {timelines.map((timeline) => (
              <li key={timeline.id}>
                <button
                  type="button"
                  className={selectedId === timeline.id ? "active" : "secondary"}
                  onClick={() => setSelectedId(timeline.id)}
                >
                  {timeline.name}
                  <small>{timeline.mode === "absolute" ? "Absolut" : "Relativ"}</small>
                </button>
              </li>
            ))}
          </ul>
        </aside>
        <div className="timeline-detail">
          {details ? (
            <>
              <div className="section-heading">
                <div>
                  <span className="eyebrow">
                    {details.timeline.mode === "absolute" ? "Absolute Ordnung" : "Relative Ordnung"}
                  </span>
                  <h3>{details.timeline.name}</h3>
                </div>
                <button
                  type="button"
                  className="danger secondary"
                  onClick={() =>
                    void deleteTimeline(projectId, details.timeline.id).then(() => {
                      setDetails(null);
                      return refreshTimelines(null);
                    })
                  }
                >
                  Zeitebene löschen
                </button>
              </div>
              <form className="inline-form event-form" onSubmit={(event) => void addEvent(event)}>
                <label htmlFor="event-title">Ereignis</label>
                <input
                  id="event-title"
                  value={eventTitle}
                  required
                  maxLength={240}
                  onChange={(event) => setEventTitle(event.currentTarget.value)}
                />
                {details.timeline.mode === "absolute" ? (
                  <>
                    <label htmlFor="event-time">Zeitpunkt</label>
                    <input
                      id="event-time"
                      value={timeValue}
                      required
                      maxLength={80}
                      placeholder="z. B. 2042-05-17"
                      onChange={(event) => setTimeValue(event.currentTarget.value)}
                    />
                  </>
                ) : null}
                <button type="submit" className="compact">
                  + Ereignis
                </button>
              </form>
              <ol className="event-list">
                {details.events.map((item, index) => (
                  <li key={item.id}>
                    <div className="event-heading">
                      <div>
                        {item.timeValue ? (
                          <time>{item.timeValue}</time>
                        ) : (
                          <span className="eyebrow">Position {index + 1}</span>
                        )}
                        <h4>{item.title}</h4>
                      </div>
                      <button
                        type="button"
                        className="secondary compact"
                        onClick={() =>
                          void deleteTimelineEvent(projectId, item.id).then(refreshDetails)
                        }
                      >
                        Löschen
                      </button>
                    </div>
                    <div className="event-links">
                      {item.documentIds.map((id) => (
                        <button
                          type="button"
                          className="secondary compact"
                          key={id}
                          onClick={() => onSelectDocument(id)}
                        >
                          Szene: {documentTitle(id)}
                        </button>
                      ))}
                      {item.entityIds.map((id) => (
                        <span className="link-chip" key={id}>
                          Entität: {entityName(id)}
                        </span>
                      ))}
                    </div>
                    <div className="event-actions">
                      <button
                        type="button"
                        className="secondary compact"
                        disabled={index === 0}
                        onClick={() => void move(item.id, -1)}
                      >
                        Nach oben
                      </button>
                      <button
                        type="button"
                        className="secondary compact"
                        disabled={index === details.events.length - 1}
                        onClick={() => void move(item.id, 1)}
                      >
                        Nach unten
                      </button>
                      {activeDocumentId && sceneTitle ? (
                        <button
                          type="button"
                          className="compact"
                          onClick={() =>
                            void linkTimelineEvent(
                              projectId,
                              details.timeline.id,
                              item.id,
                              activeDocumentId,
                              null,
                            ).then(setDetails)
                          }
                        >
                          Mit „{sceneTitle}“ verknüpfen
                        </button>
                      ) : null}
                      <select
                        aria-label={`Entität für ${item.title}`}
                        value={linkEntityId}
                        onChange={(event) => setLinkEntityId(event.currentTarget.value as EntityId)}
                      >
                        <option value="">Entität wählen</option>
                        {entities.map((entity) => (
                          <option key={entity.id} value={entity.id}>
                            {entity.name}
                          </option>
                        ))}
                      </select>
                      <button
                        type="button"
                        className="compact"
                        disabled={!linkEntityId}
                        onClick={() =>
                          void linkTimelineEvent(
                            projectId,
                            details.timeline.id,
                            item.id,
                            null,
                            linkEntityId || null,
                          ).then(setDetails)
                        }
                      >
                        Entität verknüpfen
                      </button>
                    </div>
                  </li>
                ))}
              </ol>
            </>
          ) : (
            <p className="empty-editor">Lege eine Zeitebene an oder wähle eine aus.</p>
          )}
        </div>
      </div>
    </section>
  );
}
