import type { AssetTargetKind, DocumentNode, EntityId, ProjectId } from "@kraqel/domain";
import { useCallback, useEffect, useMemo, useState } from "react";
import {
  changeMediaAssetLink,
  deleteMediaAsset,
  importMediaAsset,
  listMediaAssets,
  listStoryEntities,
  listTimelines,
  loadTimeline,
  replaceMediaAsset,
  updateMediaAsset,
  type MediaAssetView,
} from "./commands";

const MAX_FILE_BYTES = 25 * 1024 * 1024;

interface LinkTarget {
  readonly kind: AssetTargetKind;
  readonly id: EntityId;
  readonly label: string;
}

export function MediaLibrary({
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
  const [assets, setAssets] = useState<readonly MediaAssetView[]>([]);
  const [targets, setTargets] = useState<readonly LinkTarget[]>([]);
  const [file, setFile] = useState<File | null>(null);
  const [title, setTitle] = useState("");
  const [tags, setTags] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    const [nextAssets, entities, timelines] = await Promise.all([
      listMediaAssets(projectId),
      listStoryEntities(projectId),
      listTimelines(projectId),
    ]);
    const timelineDetails = await Promise.all(
      timelines.map((timeline) => loadTimeline(projectId, timeline.id)),
    );
    setAssets(nextAssets);
    setTargets([
      ...documents
        .filter(({ kind }) => kind === "scene")
        .map(({ id, title: documentTitle }) => ({
          kind: "document" as const,
          id,
          label: `Szene: ${documentTitle}`,
        })),
      ...entities.map(({ id, name }) => ({
        kind: "entity" as const,
        id,
        label: `Story-Bibel: ${name}`,
      })),
      ...timelineDetails.flatMap(({ timeline, events }) =>
        events.map(({ id, title: eventTitle }) => ({
          kind: "event" as const,
          id,
          label: `Ereignis: ${timeline.name} / ${eventTitle}`,
        })),
      ),
    ]);
  }, [documents, projectId]);

  useEffect(() => {
    let active = true;
    void refresh().catch(() => {
      if (active) setError("Die Medienbibliothek konnte nicht geladen werden.");
    });
    return () => {
      active = false;
    };
  }, [refresh]);

  const importFile = async () => {
    if (!file || !title.trim() || file.size > MAX_FILE_BYTES) return;
    setBusy(true);
    setError(null);
    try {
      await importMediaAsset(
        projectId,
        file.name,
        title.trim(),
        parseTags(tags),
        Array.from(new Uint8Array(await file.arrayBuffer())),
      );
      setFile(null);
      setTitle("");
      setTags("");
      await refresh();
    } catch {
      setError(
        "Das Bild konnte nicht importiert werden. Unterstützt werden gültige PNG-, JPEG- und WebP-Dateien bis 25 MiB.",
      );
    } finally {
      setBusy(false);
    }
  };

  const activeTarget = useMemo(
    () => targets.find(({ kind, id }) => kind === "document" && id === activeDocumentId) ?? null,
    [activeDocumentId, targets],
  );

  return (
    <section className="media-library" aria-labelledby="media-heading">
      <span className="eyebrow">Lokale, hash-geprüfte Projektmedien</span>
      <div className="panel-heading">
        <div>
          <h2 id="media-heading">Medienbibliothek</h2>
          <p>
            Bilder bleiben im Projektordner. KRAQEL prüft Inhalt und Abmessungen, erzeugt lokale
            Vorschaubilder und übernimmt keine Datei ungefragt in Text oder Cloud.
          </p>
        </div>
      </div>

      <form
        className="media-import"
        onSubmit={(event) => {
          event.preventDefault();
          void importFile();
        }}
      >
        <label htmlFor="media-file">PNG, JPEG oder WebP auswählen</label>
        <input
          id="media-file"
          type="file"
          accept="image/png,image/jpeg,image/webp"
          disabled={busy}
          onChange={(event) => {
            const next = event.target.files?.[0] ?? null;
            setFile(next);
            if (next && !title) setTitle(next.name.replace(/\.[^.]+$/, ""));
            if (next && next.size > MAX_FILE_BYTES) {
              setError("Die ausgewählte Datei ist größer als 25 MiB.");
            }
          }}
        />
        <label htmlFor="media-title">Titel</label>
        <input
          id="media-title"
          maxLength={240}
          value={title}
          disabled={busy}
          onChange={(event) => setTitle(event.target.value)}
        />
        <label htmlFor="media-tags">Schlagwörter, durch Komma getrennt</label>
        <input
          id="media-tags"
          value={tags}
          disabled={busy}
          onChange={(event) => setTags(event.target.value)}
        />
        <button
          type="submit"
          disabled={busy || !file || !title.trim() || file.size > MAX_FILE_BYTES}
        >
          {busy ? "Bild wird geprüft …" : "Bild lokal importieren"}
        </button>
      </form>

      {error ? <p className="error-banner">{error}</p> : null}
      {assets.length === 0 ? (
        <p className="empty-editor">Noch keine Bilder. Der Bilderschrank hallt dezent.</p>
      ) : (
        <div className="media-grid">
          {assets.map((view) => (
            <MediaCard
              key={view.asset.id}
              projectId={projectId}
              view={view}
              targets={targets}
              activeTarget={activeTarget}
              onChanged={refresh}
              onSelectDocument={onSelectDocument}
              onError={setError}
            />
          ))}
        </div>
      )}
    </section>
  );
}

function MediaCard({
  projectId,
  view,
  targets,
  activeTarget,
  onChanged,
  onSelectDocument,
  onError,
}: {
  readonly projectId: ProjectId;
  readonly view: MediaAssetView;
  readonly targets: readonly LinkTarget[];
  readonly activeTarget: LinkTarget | null;
  readonly onChanged: () => Promise<void>;
  readonly onSelectDocument: (id: EntityId) => void;
  readonly onError: (message: string | null) => void;
}) {
  const [title, setTitle] = useState(view.asset.title);
  const [tags, setTags] = useState(view.asset.tags.join(", "));
  const [targetValue, setTargetValue] = useState("");
  const [busy, setBusy] = useState(false);
  const [deleteArmed, setDeleteArmed] = useState(false);
  const targetMap = useMemo(
    () => new Map(targets.map((target) => [`${target.kind}:${target.id}`, target])),
    [targets],
  );

  const run = async (operation: () => Promise<unknown>, message: string) => {
    setBusy(true);
    onError(null);
    try {
      await operation();
      await onChanged();
    } catch {
      onError(message);
    } finally {
      setBusy(false);
    }
  };

  const linkTarget = async (target: LinkTarget) =>
    run(
      () =>
        changeMediaAssetLink(true, projectId, view.asset.id, target.kind, target.id, "reference"),
      "Die Bildverknüpfung konnte nicht gespeichert werden.",
    );

  return (
    <article className="media-card">
      {view.thumbnailDataUrl ? (
        <img src={view.thumbnailDataUrl} alt={view.asset.title} />
      ) : (
        <div className="missing-media" role="status">
          Vorschaudatei fehlt
        </div>
      )}
      <div className="media-card-body">
        <label>
          Titel
          <input
            maxLength={240}
            value={title}
            disabled={busy}
            onChange={(event) => setTitle(event.target.value)}
          />
        </label>
        <label>
          Schlagwörter
          <input value={tags} disabled={busy} onChange={(event) => setTags(event.target.value)} />
        </label>
        <button
          type="button"
          className="secondary"
          disabled={busy || !title.trim()}
          onClick={() =>
            void run(
              () => updateMediaAsset(projectId, view.asset.id, title.trim(), parseTags(tags)),
              "Die Medienmetadaten konnten nicht gespeichert werden.",
            )
          }
        >
          Metadaten speichern
        </button>

        <dl className="media-metadata">
          <div>
            <dt>Format</dt>
            <dd>{view.asset.mimeType}</dd>
          </div>
          <div>
            <dt>Maße</dt>
            <dd>
              {view.asset.width} × {view.asset.height}
            </dd>
          </div>
          <div>
            <dt>SHA-256</dt>
            <dd title={view.asset.checksum}>{view.asset.checksum.slice(0, 16)}…</dd>
          </div>
        </dl>

        <label>
          Bildinhalt ersetzen
          <input
            type="file"
            accept="image/png,image/jpeg,image/webp"
            disabled={busy}
            onChange={(event) => {
              const replacement = event.target.files?.[0];
              if (!replacement || replacement.size > MAX_FILE_BYTES) return;
              void run(
                async () =>
                  replaceMediaAsset(
                    projectId,
                    view.asset.id,
                    replacement.name,
                    Array.from(new Uint8Array(await replacement.arrayBuffer())),
                  ),
                "Das Ersatzbild konnte nicht validiert oder gespeichert werden.",
              );
            }}
          />
        </label>

        {activeTarget &&
        !view.asset.links.some(
          ({ targetKind, targetId }) =>
            targetKind === activeTarget.kind && targetId === activeTarget.id,
        ) ? (
          <button type="button" className="secondary" onClick={() => void linkTarget(activeTarget)}>
            Mit aktiver Szene verknüpfen
          </button>
        ) : null}
        <div className="asset-link-form">
          <select value={targetValue} onChange={(event) => setTargetValue(event.target.value)}>
            <option value="">Weiteres Linkziel wählen</option>
            {targets.map((target) => (
              <option key={`${target.kind}:${target.id}`} value={`${target.kind}:${target.id}`}>
                {target.label}
              </option>
            ))}
          </select>
          <button
            type="button"
            className="secondary"
            disabled={!targetValue || busy}
            onClick={() => {
              const target = targetMap.get(targetValue);
              if (target) void linkTarget(target);
            }}
          >
            Verknüpfen
          </button>
        </div>

        <div className="asset-links" aria-label="Bildverknüpfungen">
          {view.asset.links.map((link) => {
            const target = targetMap.get(`${link.targetKind}:${link.targetId}`);
            return (
              <span className="link-chip" key={`${link.targetKind}:${link.targetId}:${link.role}`}>
                <button
                  type="button"
                  className="text-link"
                  disabled={link.targetKind !== "document"}
                  onClick={() => {
                    if (link.targetKind === "document") onSelectDocument(link.targetId);
                  }}
                >
                  {target?.label ?? `${link.targetKind}: ${link.targetId}`}
                </button>
                <button
                  type="button"
                  className="icon-button"
                  aria-label={`Verknüpfung zu ${target?.label ?? link.targetId} entfernen`}
                  onClick={() =>
                    void run(
                      () =>
                        changeMediaAssetLink(
                          false,
                          projectId,
                          view.asset.id,
                          link.targetKind,
                          link.targetId,
                          link.role,
                        ),
                      "Die Bildverknüpfung konnte nicht entfernt werden.",
                    )
                  }
                >
                  ×
                </button>
              </span>
            );
          })}
        </div>

        <button
          type="button"
          className="secondary danger"
          disabled={busy}
          onClick={() => {
            if (!deleteArmed) {
              setDeleteArmed(true);
              return;
            }
            void run(
              () => deleteMediaAsset(projectId, view.asset.id),
              "Das Bild konnte nicht gelöscht werden.",
            );
          }}
        >
          {deleteArmed ? "Löschen endgültig bestätigen" : "Bild löschen"}
        </button>
      </div>
    </article>
  );
}

function parseTags(value: string): string[] {
  return value
    .split(",")
    .map((item) => item.trim())
    .filter(Boolean);
}
