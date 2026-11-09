import type {
  EntityId,
  PlotBoard as PlotBoardData,
  PlotStatus,
  ProjectId,
  StoryEntity,
} from "@kraqel/domain";
import { useEffect, useState, type FormEvent } from "react";
import {
  createStoryline,
  deleteStoryline,
  getPlotBoard,
  listStoryEntities,
  reorderPlotCards,
  updatePlotCard,
} from "./commands";

export function PlotBoard({
  projectId,
  onSelectDocument,
}: {
  readonly projectId: ProjectId;
  readonly onSelectDocument: (id: EntityId) => void;
}) {
  const [board, setBoard] = useState<PlotBoardData | null>(null);
  const [characters, setCharacters] = useState<readonly StoryEntity[]>([]);
  const [name, setName] = useState("");
  const [color, setColor] = useState("#0c7772");
  const [dragging, setDragging] = useState<EntityId | null>(null);
  const [error, setError] = useState<string | null>(null);
  const refresh = async () => setBoard(await getPlotBoard(projectId));

  useEffect(() => {
    let active = true;
    void Promise.all([getPlotBoard(projectId), listStoryEntities(projectId, "", "character")])
      .then(([nextBoard, nextCharacters]) => {
        if (active) {
          setBoard(nextBoard);
          setCharacters(nextCharacters);
        }
      })
      .catch(() => active && setError("Das Plotboard konnte nicht geladen werden."));
    return () => {
      active = false;
    };
  }, [projectId]);

  const addStoryline = async (event: FormEvent) => {
    event.preventDefault();
    try {
      await createStoryline(projectId, name, color);
      setName("");
      await refresh();
    } catch {
      setError("Der Handlungsstrang konnte nicht angelegt werden.");
    }
  };

  const reorder = async (source: EntityId, target: EntityId) => {
    if (!board || source === target) return;
    const ids = board.cards.map(({ documentId }) => documentId);
    const from = ids.indexOf(source);
    const to = ids.indexOf(target);
    if (from < 0 || to < 0) return;
    ids.splice(to, 0, ids.splice(from, 1)[0]!);
    try {
      await reorderPlotCards(projectId, ids);
      await refresh();
    } catch {
      setError("Die Kartenreihenfolge konnte nicht gespeichert werden.");
    }
  };

  const move = async (id: EntityId, delta: number) => {
    if (!board) return;
    const index = board.cards.findIndex(({ documentId }) => documentId === id);
    const target = board.cards[index + delta]?.documentId;
    if (target) await reorder(id, target);
  };

  return (
    <section className="structure-view" aria-labelledby="plotboard-title">
      <header className="structure-header">
        <div>
          <span className="eyebrow">Szenenstruktur</span>
          <h2 id="plotboard-title">Plotboard</h2>
        </div>
        <form className="inline-form" onSubmit={(event) => void addStoryline(event)}>
          <label htmlFor="storyline-name">Handlungsstrang</label>
          <input
            id="storyline-name"
            value={name}
            maxLength={120}
            required
            onChange={(event) => setName(event.currentTarget.value)}
          />
          <label htmlFor="storyline-color">Farbe</label>
          <input
            id="storyline-color"
            type="color"
            value={color}
            onChange={(event) => setColor(event.currentTarget.value)}
          />
          <button type="submit" className="compact">
            + Strang
          </button>
        </form>
      </header>
      {error ? <p className="error-banner">{error}</p> : null}
      {board ? (
        <>
          <ul className="storyline-legend" aria-label="Handlungsstränge">
            {board.storylines.map((line) => (
              <li key={line.id}>
                <span className="color-dot" style={{ background: line.color }} />
                {line.name}
                <button
                  type="button"
                  className="secondary compact"
                  aria-label={`${line.name} löschen`}
                  onClick={() =>
                    void deleteStoryline(projectId, line.id)
                      .then(refresh)
                      .catch(() => setError("Der Handlungsstrang konnte nicht gelöscht werden."))
                  }
                >
                  ×
                </button>
              </li>
            ))}
          </ul>
          <div className="plot-card-grid" aria-label="Szenenkarten">
            {board.cards.map((card, index) => (
              <article
                key={card.documentId}
                className="plot-card"
                draggable
                onDragStart={() => setDragging(card.documentId)}
                onDragOver={(event) => event.preventDefault()}
                onDrop={() => {
                  if (dragging) void reorder(dragging, card.documentId);
                  setDragging(null);
                }}
              >
                <div className="plot-card-heading">
                  <span className="eyebrow">Szene {index + 1}</span>
                  <h3>
                    <button
                      type="button"
                      className="text-button"
                      onClick={() => onSelectDocument(card.documentId)}
                    >
                      {card.title}
                    </button>
                  </h3>
                </div>
                <label>
                  Status
                  <select
                    value={card.status}
                    onChange={(event) =>
                      void updatePlotCard(
                        projectId,
                        card.documentId,
                        event.currentTarget.value as PlotStatus,
                        card.storylineId,
                        card.povEntityId,
                      ).then(refresh)
                    }
                  >
                    <option value="planned">Geplant</option>
                    <option value="draft">Entwurf</option>
                    <option value="revised">Überarbeitet</option>
                    <option value="done">Fertig</option>
                  </select>
                </label>
                <label>
                  Handlungsstrang
                  <select
                    value={card.storylineId ?? ""}
                    onChange={(event) =>
                      void updatePlotCard(
                        projectId,
                        card.documentId,
                        card.status,
                        (event.currentTarget.value || null) as EntityId | null,
                        card.povEntityId,
                      ).then(refresh)
                    }
                  >
                    <option value="">Ohne</option>
                    {board.storylines.map((line) => (
                      <option key={line.id} value={line.id}>
                        {line.name}
                      </option>
                    ))}
                  </select>
                </label>
                <label>
                  POV
                  <select
                    value={card.povEntityId ?? ""}
                    onChange={(event) =>
                      void updatePlotCard(
                        projectId,
                        card.documentId,
                        card.status,
                        card.storylineId,
                        (event.currentTarget.value || null) as EntityId | null,
                      ).then(refresh)
                    }
                  >
                    <option value="">Ohne</option>
                    {characters.map((character) => (
                      <option key={character.id} value={character.id}>
                        {character.name}
                      </option>
                    ))}
                  </select>
                </label>
                <div className="order-buttons" aria-label={`Reihenfolge von ${card.title}`}>
                  <button
                    type="button"
                    className="secondary compact"
                    disabled={index === 0}
                    onClick={() => void move(card.documentId, -1)}
                  >
                    Nach links
                  </button>
                  <button
                    type="button"
                    className="secondary compact"
                    disabled={index === board.cards.length - 1}
                    onClick={() => void move(card.documentId, 1)}
                  >
                    Nach rechts
                  </button>
                </div>
              </article>
            ))}
          </div>
        </>
      ) : (
        <p className="empty-editor">Plotboard wird geladen …</p>
      )}
    </section>
  );
}
