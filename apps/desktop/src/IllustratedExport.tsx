import { useState } from "react";
import type { DocumentNode, ProjectId } from "@kraqel/domain";
import {
  exportIllustratedManuscript,
  listMediaAssets,
  openWorkspace,
  type ImagePlacement,
  type IllustratedResult,
  type MediaAssetView,
} from "./commands";

interface Candidate {
  key: string;
  scene: DocumentNode;
  media: MediaAssetView;
}
export function IllustratedExport({
  projectId,
  disabled,
}: {
  readonly projectId: ProjectId;
  readonly disabled: boolean;
}) {
  const [candidates, setCandidates] = useState<readonly Candidate[] | null>(null);
  const [selection, setSelection] = useState<readonly ImagePlacement[]>([]);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<IllustratedResult | null>(null);
  const load = async () => {
    setBusy(true);
    setError(null);
    setResult(null);
    setSelection([]);
    try {
      const [workspace, media] = await Promise.all([
        openWorkspace(projectId),
        listMediaAssets(projectId),
      ]);
      const chapters = workspace.documents
        .filter((n) => n.kind === "chapter")
        .sort((a, b) => a.position - b.position || a.id.localeCompare(b.id));
      const scenes = chapters.flatMap((c) =>
        workspace.documents
          .filter((n) => n.kind === "scene" && n.parentId === c.id)
          .sort((a, b) => a.position - b.position || a.id.localeCompare(b.id)),
      );
      setCandidates(
        scenes.flatMap((scene) =>
          media
            .filter((m) =>
              m.asset.links.some((l) => l.targetKind === "document" && l.targetId === scene.id),
            )
            .map((item) => ({ key: `${scene.id}:${item.asset.id}`, scene, media: item })),
        ),
      );
    } catch {
      setError("Die Bildauswahl konnte nicht geladen werden.");
    } finally {
      setBusy(false);
    }
  };
  const toggle = (candidate: Candidate, checked: boolean) => {
    setResult(null);
    setSelection((current) =>
      checked
        ? [
            ...current,
            {
              sceneId: candidate.scene.id,
              assetId: candidate.media.asset.id,
              expectedChecksum: candidate.media.asset.checksum,
              expectedSceneRevision: candidate.scene.revision,
              altText: candidate.media.asset.title,
              caption: null,
            },
          ]
        : current.filter(
            (p) => p.sceneId !== candidate.scene.id || p.assetId !== candidate.media.asset.id,
          ),
    );
  };
  const update = (item: ImagePlacement, changes: Partial<ImagePlacement>) => {
    setResult(null);
    setSelection((current) => current.map((p) => (p === item ? { ...p, ...changes } : p)));
  };
  const move = (item: ImagePlacement, delta: number) => {
    const same = selection.filter((p) => p.sceneId === item.sceneId);
    const other = same[same.indexOf(item) + delta];
    if (!other) return;
    const next = [...selection];
    const i = next.indexOf(item);
    const j = next.indexOf(other);
    [next[i], next[j]] = [next[j]!, next[i]!];
    setSelection(next);
    setResult(null);
  };
  const run = async (format: "markdown" | "html") => {
    setBusy(true);
    setError(null);
    setResult(null);
    try {
      setResult(await exportIllustratedManuscript(projectId, format, selection));
    } catch {
      setError(
        "Das Bildpaket konnte nicht erzeugt werden. Auswahl und Quelldateien müssen gültig und unverändert sein. Prüfe die Grenzen und lade bei Änderungen die Bildauswahl neu.",
      );
    } finally {
      setBusy(false);
    }
  };
  const invalid =
    selection.length === 0 ||
    selection.length > 200 ||
    new Set(selection.map((p) => p.assetId)).size > 50 ||
    selection.some(
      (p) =>
        !p.altText.trim() ||
        Array.from(p.altText).length > 500 ||
        Array.from(p.caption ?? "").length > 1200,
    );
  return (
    <section aria-labelledby="illustrated-export-heading">
      <h3 id="illustrated-export-heading">Mit Szenenbildern</h3>
      <p>
        Wähle ausdrücklich Bilder für das Ende ihrer verknüpften Szene. Reihenfolge, Alternativtext
        und optionale Bildunterschrift gelten nur für diesen Export. Ohne Auswahl verwende die
        Textausgabe.
      </p>
      <p className="muted">
        Markdown und HTML als vollständiger Ordner mit Bildern und Herkunftsnachweis. Höchstens 50
        Originale, 200 Bildvorkommen und 128 MiB je Paket. DOCX/PDF mit Bildern folgen separat.
        Frühere Pakete bleiben erhalten.
      </p>
      <button type="button" disabled={busy || disabled} onClick={() => void load()}>
        {candidates ? "Bildauswahl neu laden" : "Bildauswahl laden"}
      </button>
      {candidates?.length === 0 ? (
        <p>
          Keine direkt mit Szenen verknüpften Bilder. Verknüpfe sie zuerst in der Medienbibliothek.
        </p>
      ) : null}
      {candidates
        ? Array.from(new Set(candidates.map((c) => c.scene.id))).map((sceneId) => {
            const group = candidates.filter((c) => c.scene.id === sceneId);
            const scene = group[0]!.scene;
            const selected = selection.filter((p) => p.sceneId === sceneId);
            const ordered = [...group].sort((a, b) => {
              const ai = selected.findIndex((p) => p.assetId === a.media.asset.id);
              const bi = selected.findIndex((p) => p.assetId === b.media.asset.id);
              return (ai < 0 ? 1000 : ai) - (bi < 0 ? 1000 : bi);
            });
            return (
              <fieldset key={sceneId} disabled={busy || disabled} className="illustrated-scene">
                <legend>{scene.title || "Ohne Titel"}</legend>
                {ordered.map((candidate) => {
                  const asset = candidate.media.asset;
                  const item = selection.find(
                    (p) => p.sceneId === sceneId && p.assetId === asset.id,
                  );
                  const position = item ? selected.indexOf(item) : -1;
                  return (
                    <div key={candidate.key} className="illustrated-image">
                      {candidate.media.thumbnailDataUrl ? (
                        <img
                          src={candidate.media.thumbnailDataUrl}
                          width={96}
                          height={96}
                          style={{ objectFit: "contain" }}
                          alt=""
                        />
                      ) : null}
                      <label>
                        <input
                          type="checkbox"
                          checked={Boolean(item)}
                          disabled={!candidate.media.fileAvailable}
                          onChange={(e) => toggle(candidate, e.target.checked)}
                        />
                        {asset.title}
                      </label>
                      {!candidate.media.fileAvailable ? (
                        <p>Original fehlt oder ist verändert.</p>
                      ) : null}
                      {item ? (
                        <>
                          <label>
                            Alternativtext für {asset.title}
                            <input
                              value={item.altText}
                              maxLength={1000}
                              onChange={(e) => update(item, { altText: e.target.value })}
                            />
                          </label>
                          <label>
                            Bildunterschrift für {asset.title}
                            <textarea
                              value={item.caption ?? ""}
                              maxLength={2400}
                              onChange={(e) => update(item, { caption: e.target.value || null })}
                            />
                          </label>
                          <p>Bild {position + 1} am Szenenende</p>
                          <button
                            type="button"
                            disabled={position === 0}
                            aria-label={`${asset.title} in ${scene.title || "Ohne Titel"} nach oben`}
                            onClick={() => move(item, -1)}
                          >
                            Nach oben
                          </button>
                          <button
                            type="button"
                            disabled={position === selected.length - 1}
                            aria-label={`${asset.title} in ${scene.title || "Ohne Titel"} nach unten`}
                            onClick={() => move(item, 1)}
                          >
                            Nach unten
                          </button>
                        </>
                      ) : null}
                    </div>
                  );
                })}
              </fieldset>
            );
          })
        : null}
      <div className="illustrated-actions">
        {(["markdown", "html"] as const).map((format) => (
          <button
            key={format}
            type="button"
            disabled={busy || disabled || invalid}
            onClick={() => void run(format)}
          >
            {format === "html" ? "HTML mit Bildern erzeugen" : "Markdown mit Bildern erzeugen"}
          </button>
        ))}
      </div>
      {busy ? <p role="status">Bildpaket wird vorbereitet …</p> : null}
      {result ? (
        <p role="status" className="success-banner">
          Bildpaket fertig: <strong>{result.packageName}</strong> ({result.imageCount} Bilder,{" "}
          {result.placementCount} Vorkommen, {result.byteCount.toLocaleString("de-DE")} Bytes). Gib
          den vollständigen Ordner weiter.
        </p>
      ) : null}
      {error ? (
        <p role="alert" className="error-banner">
          {error}
        </p>
      ) : null}
    </section>
  );
}
