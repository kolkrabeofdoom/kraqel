import type {
  EntityId,
  EntityKind,
  ProjectId,
  StoryEntity,
  StylePreset,
  VisualProfile,
} from "@kraqel/domain";
import { useCallback, useEffect, useMemo, useState, type FormEvent } from "react";
import {
  deleteStylePreset,
  deleteVisualProfile,
  listMediaAssets,
  listStoryEntities,
  listStylePresets,
  listVisualProfiles,
  saveStylePreset,
  saveVisualProfile,
  type MediaAssetView,
} from "./commands";

const entityKinds: readonly EntityKind[] = ["character", "location", "faction", "item"];

export function VisualProfiles({ projectId }: { readonly projectId: ProjectId }) {
  const [entities, setEntities] = useState<readonly StoryEntity[]>([]);
  const [assets, setAssets] = useState<readonly MediaAssetView[]>([]);
  const [profiles, setProfiles] = useState<readonly VisualProfile[]>([]);
  const [presets, setPresets] = useState<readonly StylePreset[]>([]);
  const [selectedEntityId, setSelectedEntityId] = useState<EntityId | null>(null);
  const [traits, setTraits] = useState("");
  const [exclusions, setExclusions] = useState("");
  const [canonicalAssetId, setCanonicalAssetId] = useState<EntityId | null>(null);
  const [presetId, setPresetId] = useState<EntityId | null>(null);
  const [presetName, setPresetName] = useState("");
  const [visualRules, setVisualRules] = useState("");
  const [applicableKinds, setApplicableKinds] = useState<readonly EntityKind[]>(["character"]);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [pendingDelete, setPendingDelete] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    const [nextEntities, nextAssets, nextProfiles, nextPresets] = await Promise.all([
      listStoryEntities(projectId),
      listMediaAssets(projectId),
      listVisualProfiles(projectId),
      listStylePresets(projectId),
    ]);
    setEntities(nextEntities);
    setAssets(nextAssets);
    setProfiles(nextProfiles);
    setPresets(nextPresets);
    setSelectedEntityId((current) =>
      current && nextEntities.some(({ id }) => id === current)
        ? current
        : (nextEntities[0]?.id ?? null),
    );
  }, [projectId]);

  useEffect(() => {
    void refresh().catch(() => setError("Die visuellen Profile konnten nicht geladen werden."));
  }, [refresh]);

  const selectedProfile = useMemo(
    () => profiles.find(({ entityId }) => entityId === selectedEntityId) ?? null,
    [profiles, selectedEntityId],
  );

  useEffect(() => {
    setTraits(formatMap(selectedProfile?.traits ?? {}));
    setExclusions((selectedProfile?.exclusions ?? []).join("\n"));
    setCanonicalAssetId(selectedProfile?.canonicalAssetId ?? null);
    setPendingDelete(null);
  }, [selectedProfile, selectedEntityId]);

  const submitProfile = async (event: FormEvent) => {
    event.preventDefault();
    if (!selectedEntityId) return;
    setBusy(true);
    setError(null);
    try {
      await saveVisualProfile(
        projectId,
        selectedEntityId,
        parseMap(traits),
        canonicalAssetId,
        parseLines(exclusions),
      );
      await refresh();
    } catch {
      setError(
        "Das visuelle Profil konnte nicht gespeichert werden. Prüfe doppelte oder leere Werte.",
      );
    } finally {
      setBusy(false);
    }
  };

  const removeProfile = async () => {
    if (!selectedEntityId) return;
    const key = `profile:${selectedEntityId}`;
    if (pendingDelete !== key) {
      setPendingDelete(key);
      return;
    }
    setBusy(true);
    try {
      await deleteVisualProfile(projectId, selectedEntityId);
      await refresh();
      setPendingDelete(null);
    } catch {
      setError("Das visuelle Profil konnte nicht gelöscht werden.");
    } finally {
      setBusy(false);
    }
  };

  const submitPreset = async (event: FormEvent) => {
    event.preventDefault();
    setBusy(true);
    setError(null);
    try {
      await saveStylePreset(
        projectId,
        presetId,
        presetName,
        parseMap(visualRules),
        applicableKinds,
      );
      resetPreset();
      await refresh();
    } catch {
      setError(
        "Die Stilvorlage konnte nicht gespeichert werden. Name, Regeln und Geltungsbereich müssen eindeutig sein.",
      );
    } finally {
      setBusy(false);
    }
  };

  const editPreset = (preset: StylePreset) => {
    setPresetId(preset.id);
    setPresetName(preset.name);
    setVisualRules(formatMap(preset.visualRules));
    setApplicableKinds(preset.applicableKinds);
    setPendingDelete(null);
  };

  const removePreset = async (preset: StylePreset) => {
    const key = `preset:${preset.id}`;
    if (pendingDelete !== key) {
      setPendingDelete(key);
      return;
    }
    setBusy(true);
    try {
      await deleteStylePreset(projectId, preset.id);
      if (presetId === preset.id) resetPreset();
      await refresh();
      setPendingDelete(null);
    } catch {
      setError("Die Stilvorlage konnte nicht gelöscht werden.");
    } finally {
      setBusy(false);
    }
  };

  const resetPreset = () => {
    setPresetId(null);
    setPresetName("");
    setVisualRules("");
    setApplicableKinds(["character"]);
  };

  return (
    <section className="tool-page visual-profiles" aria-labelledby="visual-profiles-heading">
      <header className="tool-heading">
        <div>
          <span className="eyebrow">Offline und projektbezogen</span>
          <h2 id="visual-profiles-heading">Visuelle Profile</h2>
          <p>Kernmerkmale, unerwünschte Abweichungen und wiederverwendbare Stilregeln.</p>
        </div>
      </header>
      {error ? <p className="error-banner">{error}</p> : null}

      <div className="visual-layout">
        <form className="panel-stack" onSubmit={(event) => void submitProfile(event)}>
          <h3>Profil eines Story-Bibel-Eintrags</h3>
          {entities.length === 0 ? (
            <p className="empty-state">Lege zuerst einen Eintrag in der Story-Bibel an.</p>
          ) : (
            <>
              <label>
                Story-Bibel-Eintrag
                <select
                  value={selectedEntityId ?? ""}
                  onChange={(event) => setSelectedEntityId(event.target.value as EntityId)}
                >
                  {entities.map((entity) => (
                    <option key={entity.id} value={entity.id}>
                      {kindLabel(entity.kind)}: {entity.name}
                    </option>
                  ))}
                </select>
              </label>
              <label>
                Kernmerkmale
                <textarea
                  rows={7}
                  value={traits}
                  onChange={(event) => setTraits(event.target.value)}
                  placeholder={"Augen: bernsteinfarben\nKleidung: dunkelgrüner Mantel"}
                />
                <small>Eine Zeile je Merkmal im Format „Name: Wert“.</small>
              </label>
              <label>
                Unerwünschte Abweichungen
                <textarea
                  rows={5}
                  value={exclusions}
                  onChange={(event) => setExclusions(event.target.value)}
                  placeholder={"blaue Augen\nmoderne Kleidung"}
                />
                <small>Eine explizite Ausschlussregel je Zeile.</small>
              </label>
              <label>
                Kanonisches Referenzbild
                <select
                  value={canonicalAssetId ?? ""}
                  onChange={(event) =>
                    setCanonicalAssetId(
                      event.target.value ? (event.target.value as EntityId) : null,
                    )
                  }
                >
                  <option value="">Kein kanonisches Bild</option>
                  {assets.map(({ asset, fileAvailable }) => (
                    <option key={asset.id} value={asset.id}>
                      {asset.title}
                      {fileAvailable ? "" : " (Datei fehlt)"}
                    </option>
                  ))}
                </select>
              </label>
              <div className="button-row">
                <button type="submit" disabled={busy}>
                  Profil speichern
                </button>
                {selectedProfile ? (
                  <button
                    type="button"
                    className="danger"
                    disabled={busy}
                    onClick={() => void removeProfile()}
                  >
                    {pendingDelete === `profile:${selectedEntityId}`
                      ? "Löschen bestätigen"
                      : "Profil löschen"}
                  </button>
                ) : null}
              </div>
            </>
          )}
        </form>

        <div className="panel-stack">
          <form onSubmit={(event) => void submitPreset(event)}>
            <h3>{presetId ? "Stilvorlage bearbeiten" : "Neue Stilvorlage"}</h3>
            <label>
              Name
              <input
                value={presetName}
                onChange={(event) => setPresetName(event.target.value)}
                maxLength={120}
                required
              />
            </label>
            <label>
              Visuelle Regeln
              <textarea
                rows={6}
                value={visualRules}
                onChange={(event) => setVisualRules(event.target.value)}
                placeholder={"Palette: Ocker und Nachtblau\nLicht: weiches Gegenlicht"}
              />
              <small>Eine Zeile je Regel im Format „Name: Wert“.</small>
            </label>
            <fieldset>
              <legend>Gilt für</legend>
              <div className="checkbox-grid">
                {entityKinds.map((kind) => (
                  <label key={kind}>
                    <input
                      type="checkbox"
                      checked={applicableKinds.includes(kind)}
                      onChange={() =>
                        setApplicableKinds((current) =>
                          current.includes(kind)
                            ? current.filter((value) => value !== kind)
                            : [...current, kind],
                        )
                      }
                    />
                    {kindLabel(kind)}
                  </label>
                ))}
              </div>
            </fieldset>
            <div className="button-row">
              <button
                type="submit"
                disabled={
                  busy || !presetName.trim() || !visualRules.trim() || applicableKinds.length === 0
                }
              >
                {presetId ? "Änderungen speichern" : "Stilvorlage anlegen"}
              </button>
              {presetId ? (
                <button type="button" className="secondary" onClick={resetPreset}>
                  Abbrechen
                </button>
              ) : null}
            </div>
          </form>

          <div className="preset-list" aria-label="Gespeicherte Stilvorlagen">
            {presets.length === 0 ? <p className="empty-state">Noch keine Stilvorlagen.</p> : null}
            {presets.map((preset) => (
              <article key={preset.id} className="preset-card">
                <h4>{preset.name}</h4>
                <p>{preset.applicableKinds.map(kindLabel).join(", ")}</p>
                <dl>
                  {Object.entries(preset.visualRules).map(([key, value]) => (
                    <div key={key}>
                      <dt>{key}</dt>
                      <dd>{value}</dd>
                    </div>
                  ))}
                </dl>
                <div className="button-row">
                  <button type="button" className="secondary" onClick={() => editPreset(preset)}>
                    Bearbeiten
                  </button>
                  <button
                    type="button"
                    className="danger"
                    disabled={busy}
                    onClick={() => void removePreset(preset)}
                  >
                    {pendingDelete === `preset:${preset.id}` ? "Löschen bestätigen" : "Löschen"}
                  </button>
                </div>
              </article>
            ))}
          </div>
        </div>
      </div>
    </section>
  );
}

function parseMap(value: string): Record<string, string> {
  const result: Record<string, string> = {};
  for (const line of value
    .split(/\r?\n/u)
    .map((candidate) => candidate.trim())
    .filter(Boolean)) {
    const separator = line.indexOf(":");
    if (separator <= 0 || separator === line.length - 1) throw new Error("invalid map line");
    const key = line.slice(0, separator).trim();
    const text = line.slice(separator + 1).trim();
    if (Object.keys(result).some((candidate) => candidate.trim() === key)) {
      throw new Error("duplicate map key");
    }
    result[key] = text;
  }
  return result;
}

function parseLines(value: string): string[] {
  return value
    .split(/\r?\n/u)
    .map((line) => line.trim())
    .filter(Boolean);
}

function formatMap(value: Readonly<Record<string, string>>): string {
  return Object.entries(value)
    .map(([key, text]) => `${key}: ${text}`)
    .join("\n");
}

function kindLabel(kind: EntityKind): string {
  return { character: "Figuren", location: "Orte", faction: "Fraktionen", item: "Gegenstände" }[
    kind
  ];
}
