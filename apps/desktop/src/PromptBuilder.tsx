import type {
  DocumentNode,
  EntityId,
  ImagePromptComposition,
  ImagePromptDraft,
  ProjectId,
  StoryEntity,
  StylePreset,
  VisualProfile,
} from "@kraqel/domain";
import { useCallback, useEffect, useMemo, useState, type FormEvent } from "react";
import {
  composeImagePrompt,
  createImagePromptDraft,
  deleteImagePromptDraft,
  exportImagePromptDraft,
  listImagePromptDrafts,
  listStoryEntities,
  listStylePresets,
  listVisualProfiles,
  updateImagePromptDraft,
} from "./commands";

interface PromptBuilderProps {
  readonly projectId: ProjectId;
  readonly documents: readonly DocumentNode[];
  readonly activeDocumentId: EntityId | null;
}

export function PromptBuilder({ projectId, documents, activeDocumentId }: PromptBuilderProps) {
  const scenes = useMemo(() => documents.filter(({ kind }) => kind === "scene"), [documents]);
  const [entities, setEntities] = useState<readonly StoryEntity[]>([]);
  const [profiles, setProfiles] = useState<readonly VisualProfile[]>([]);
  const [presets, setPresets] = useState<readonly StylePreset[]>([]);
  const [drafts, setDrafts] = useState<readonly ImagePromptDraft[]>([]);
  const [documentId, setDocumentId] = useState<EntityId | null>(
    scenes.some(({ id }) => id === activeDocumentId) ? activeDocumentId : (scenes[0]?.id ?? null),
  );
  const [entityIds, setEntityIds] = useState<readonly EntityId[]>([]);
  const [stylePresetId, setStylePresetId] = useState<EntityId | null>(null);
  const [composition, setComposition] = useState<ImagePromptComposition | null>(null);
  const [draftId, setDraftId] = useState<EntityId | null>(null);
  const [name, setName] = useState("");
  const [promptText, setPromptText] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [pendingDelete, setPendingDelete] = useState<EntityId | null>(null);

  const refresh = useCallback(async () => {
    const [nextEntities, nextProfiles, nextPresets, nextDrafts] = await Promise.all([
      listStoryEntities(projectId),
      listVisualProfiles(projectId),
      listStylePresets(projectId),
      listImagePromptDrafts(projectId),
    ]);
    setEntities(nextEntities);
    setProfiles(nextProfiles);
    setPresets(nextPresets);
    setDrafts(nextDrafts);
  }, [projectId]);

  useEffect(() => {
    void refresh().catch(() => setError("Die Bildprompt-Daten konnten nicht geladen werden."));
  }, [refresh]);

  const profiledEntities = useMemo(
    () =>
      profiles
        .map((profile) => ({
          profile,
          entity: entities.find(({ id }) => id === profile.entityId),
        }))
        .filter(
          (entry): entry is { profile: VisualProfile; entity: StoryEntity } =>
            entry.entity !== undefined,
        ),
    [entities, profiles],
  );

  const selectedKinds = useMemo(
    () =>
      new Set(
        profiledEntities
          .filter(({ entity }) => entityIds.includes(entity.id))
          .map(({ entity }) => entity.kind),
      ),
    [entityIds, profiledEntities],
  );

  const applicablePresets = useMemo(
    () =>
      presets.filter((preset) =>
        [...selectedKinds].every((kind) => preset.applicableKinds.includes(kind)),
      ),
    [presets, selectedKinds],
  );

  useEffect(() => {
    if (stylePresetId && !applicablePresets.some(({ id }) => id === stylePresetId)) {
      setStylePresetId(null);
    }
  }, [applicablePresets, stylePresetId]);

  const toggleEntity = (entityId: EntityId) => {
    setEntityIds((current) =>
      current.includes(entityId)
        ? current.filter((value) => value !== entityId)
        : [...current, entityId],
    );
  };

  const compose = async (event: FormEvent) => {
    event.preventDefault();
    if (!documentId || entityIds.length === 0) return;
    setBusy(true);
    setError(null);
    setNotice(null);
    try {
      const next = await composeImagePrompt(projectId, documentId, entityIds, stylePresetId);
      setComposition(next);
      setDraftId(null);
      setName(`Bildprompt: ${next.documentTitle}`);
      setPromptText(next.promptText);
      setPendingDelete(null);
    } catch {
      setError("Der Prompt konnte nicht zusammengesetzt werden. Prüfe Szene, Profile und Stil.");
    } finally {
      setBusy(false);
    }
  };

  const save = async () => {
    setBusy(true);
    setError(null);
    setNotice(null);
    try {
      const saved = draftId
        ? await updateImagePromptDraft(projectId, draftId, name, promptText)
        : composition
          ? await createImagePromptDraft(projectId, name, composition, promptText)
          : null;
      if (!saved) return;
      setDraftId(saved.id);
      setComposition(null);
      setName(saved.name);
      setPromptText(saved.promptText);
      await refresh();
      setNotice("Der Promptentwurf wurde lokal gespeichert.");
    } catch {
      setError(
        "Der Entwurf konnte nicht gespeichert werden. Die Szene könnte inzwischen geändert worden sein.",
      );
    } finally {
      setBusy(false);
    }
  };

  const editDraft = (draft: ImagePromptDraft) => {
    setDraftId(draft.id);
    setComposition(null);
    setName(draft.name);
    setPromptText(draft.promptText);
    setDocumentId(draft.sourceDocumentId);
    setEntityIds(draft.entityIds);
    setStylePresetId(draft.stylePresetId);
    setError(null);
    setNotice(null);
    setPendingDelete(null);
  };

  const removeDraft = async (draft: ImagePromptDraft) => {
    if (pendingDelete !== draft.id) {
      setPendingDelete(draft.id);
      return;
    }
    setBusy(true);
    try {
      await deleteImagePromptDraft(projectId, draft.id);
      if (draftId === draft.id) resetEditor();
      await refresh();
      setPendingDelete(null);
    } catch {
      setError("Der Promptentwurf konnte nicht gelöscht werden.");
    } finally {
      setBusy(false);
    }
  };

  const exportDraft = async () => {
    if (!draftId) return;
    setBusy(true);
    setError(null);
    try {
      const result = await exportImagePromptDraft(projectId, draftId);
      setNotice(
        `UTF-8-Export fertig: ${result.fileName} (${result.byteCount.toLocaleString("de-DE")} Bytes)`,
      );
    } catch {
      setError("Der Promptentwurf konnte nicht exportiert werden.");
    } finally {
      setBusy(false);
    }
  };

  const resetEditor = () => {
    setDraftId(null);
    setComposition(null);
    setName("");
    setPromptText("");
    setPendingDelete(null);
  };

  const canSave =
    !busy &&
    name.trim().length > 0 &&
    promptText.trim().length > 0 &&
    (draftId !== null || composition !== null);

  return (
    <section className="tool-page prompt-builder" aria-labelledby="prompt-builder-heading">
      <header className="tool-heading">
        <div>
          <span className="eyebrow">Lokal, editierbar, ohne Provider</span>
          <h2 id="prompt-builder-heading">Bildprompt-Builder</h2>
          <p>
            Wähle Quellen bewusst aus, stelle einen Entwurf zusammen und bearbeite ihn vollständig.
          </p>
        </div>
      </header>
      {error ? (
        <p className="error-banner" role="alert">
          {error}
        </p>
      ) : null}
      {notice ? (
        <p className="success-banner" role="status">
          {notice}
        </p>
      ) : null}

      <div className="prompt-layout">
        <form className="panel-stack" onSubmit={(event) => void compose(event)}>
          <h3>Quellen auswählen</h3>
          <label>
            Szene
            <select
              value={documentId ?? ""}
              onChange={(event) => setDocumentId(event.target.value as EntityId)}
              required
            >
              <option value="" disabled>
                Szene wählen
              </option>
              {scenes.map((scene) => (
                <option key={scene.id} value={scene.id}>
                  {scene.title}
                </option>
              ))}
            </select>
          </label>
          <fieldset>
            <legend>Visuelle Profile</legend>
            {profiledEntities.length === 0 ? (
              <p className="empty-state">Lege zuerst mindestens ein visuelles Profil an.</p>
            ) : (
              profiledEntities.map(({ entity }) => (
                <label className="checkbox-row" key={entity.id}>
                  <input
                    type="checkbox"
                    checked={entityIds.includes(entity.id)}
                    onChange={() => toggleEntity(entity.id)}
                  />
                  <span>
                    {kindLabel(entity.kind)}: {entity.name}
                  </span>
                </label>
              ))
            )}
          </fieldset>
          <label>
            Stilvorlage (optional)
            <select
              value={stylePresetId ?? ""}
              onChange={(event) =>
                setStylePresetId((event.target.value || null) as EntityId | null)
              }
            >
              <option value="">Keine Stilvorlage</option>
              {applicablePresets.map((preset) => (
                <option key={preset.id} value={preset.id}>
                  {preset.name}
                </option>
              ))}
            </select>
          </label>
          <button type="submit" disabled={busy || !documentId || entityIds.length === 0}>
            {busy ? "Kolkrabe sortiert Kontext …" : "Prompt zusammensetzen"}
          </button>
          <small>Es findet kein Netzwerkzugriff und keine Providerfreigabe statt.</small>
        </form>

        <section className="panel-stack prompt-editor" aria-labelledby="prompt-editor-heading">
          <div className="panel-title-row">
            <h3 id="prompt-editor-heading">Editierbarer Entwurf</h3>
            {draftId || composition ? (
              <button type="button" className="secondary compact" onClick={resetEditor}>
                Leeren
              </button>
            ) : null}
          </div>
          <label>
            Name
            <input maxLength={120} value={name} onChange={(event) => setName(event.target.value)} />
          </label>
          <label>
            Prompttext
            <textarea
              rows={24}
              maxLength={200000}
              value={promptText}
              onChange={(event) => setPromptText(event.target.value)}
              placeholder="Stelle links einen Prompt zusammen oder öffne einen gespeicherten Entwurf."
            />
          </label>
          <small>{promptText.length.toLocaleString("de-DE")} / 200.000 Zeichen</small>
          <div className="button-row">
            <button type="button" disabled={!canSave} onClick={() => void save()}>
              Lokal speichern
            </button>
            <button
              type="button"
              className="secondary"
              disabled={busy || !draftId}
              onClick={() => void exportDraft()}
            >
              Als UTF-8-Text exportieren
            </button>
          </div>
        </section>
      </div>

      <section className="panel-stack" aria-labelledby="saved-prompts-heading">
        <h3 id="saved-prompts-heading">Gespeicherte Entwürfe</h3>
        {drafts.length === 0 ? (
          <p className="empty-state">Noch keine lokalen Promptentwürfe.</p>
        ) : (
          <div className="prompt-draft-list">
            {drafts.map((draft) => (
              <article key={draft.id} className={draft.id === draftId ? "selected" : ""}>
                <div>
                  <strong>{draft.name}</strong>
                  <p>
                    Szene-Revision {draft.sourceDocumentRevision}; {draft.entityIds.length}{" "}
                    Profil(e)
                  </p>
                </div>
                <div className="button-row">
                  <button
                    type="button"
                    className="secondary compact"
                    onClick={() => editDraft(draft)}
                  >
                    Bearbeiten
                  </button>
                  <button
                    type="button"
                    className="danger compact"
                    disabled={busy}
                    onClick={() => void removeDraft(draft)}
                  >
                    {pendingDelete === draft.id ? "Wirklich löschen" : "Löschen"}
                  </button>
                </div>
              </article>
            ))}
          </div>
        )}
      </section>
    </section>
  );
}

function kindLabel(kind: StoryEntity["kind"]): string {
  return { character: "Figur", location: "Ort", faction: "Fraktion", item: "Gegenstand" }[kind];
}
