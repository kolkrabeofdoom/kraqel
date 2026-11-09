import type {
  AiContextPreview,
  CloudProviderStatus,
  TextProviderStatus,
  TextStreamEvent,
} from "@kraqel/ai-contracts";
import type { EntityId, ProjectId } from "@kraqel/domain";
import { useEffect, useMemo, useRef, useState } from "react";
import {
  deleteCloudCredential,
  listCloudModels,
  listCloudProviderStatuses,
  listOllamaModels,
  previewAiContext,
  saveCloudCredential,
  streamTextDraft,
  type StreamTextDraftInput,
} from "./commands";

const OLLAMA_ID = "ollama";

export function TextAssistant({
  projectId,
  activeDocumentId,
}: {
  readonly projectId: ProjectId;
  readonly activeDocumentId: EntityId | null;
}) {
  const [ollama, setOllama] = useState<TextProviderStatus | null>(null);
  const [cloudProviders, setCloudProviders] = useState<readonly CloudProviderStatus[]>([]);
  const [providerId, setProviderId] = useState(OLLAMA_ID);
  const [models, setModels] = useState<readonly string[]>([]);
  const [model, setModel] = useState("");
  const [apiKey, setApiKey] = useState("");
  const [instruction, setInstruction] = useState("");
  const [preview, setPreview] = useState<AiContextPreview | null>(null);
  const [approved, setApproved] = useState(false);
  const [draft, setDraft] = useState("");
  const [provenance, setProvenance] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [providerBusy, setProviderBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const generationId = useRef(0);

  const selectedCloud = useMemo(
    () => cloudProviders.find(({ provider }) => provider.id === providerId) ?? null,
    [cloudProviders, providerId],
  );
  const selectedProviderName =
    providerId === OLLAMA_ID
      ? (ollama?.provider.displayName ?? "Ollama (lokal)")
      : selectedCloud?.provider.displayName;

  useEffect(() => {
    let active = true;
    void listOllamaModels()
      .then((localStatus) => {
        if (!active) return;
        setOllama(localStatus);
        setModels(localStatus.models.map(({ id }) => id));
        setModel(localStatus.models[0]?.id ?? "");
      })
      .catch(() => {
        if (active) setError("Der lokale Ollama-Status konnte nicht gelesen werden.");
      });
    void listCloudProviderStatuses()
      .then((cloudStatuses) => {
        if (active) setCloudProviders(cloudStatuses);
      })
      .catch(() => {
        if (active) {
          setError(
            "Der Betriebssystem-Schlüsselspeicher ist nicht verfügbar; lokale Funktionen bleiben nutzbar.",
          );
        }
      });
    return () => {
      active = false;
    };
  }, []);

  useEffect(() => {
    generationId.current += 1;
    setPreview(null);
    setApproved(false);
    setDraft("");
    setProvenance(null);
  }, [activeDocumentId, providerId, model, instruction]);

  const chooseProvider = (nextProviderId: string) => {
    setProviderId(nextProviderId);
    setApiKey("");
    if (nextProviderId === OLLAMA_ID) {
      const localModels = ollama?.models.map(({ id }) => id) ?? [];
      setModels(localModels);
      setModel(localModels[0] ?? "");
    } else {
      setModels([]);
      setModel("");
    }
  };

  const refreshCloudStatuses = async () => {
    setCloudProviders(await listCloudProviderStatuses());
  };

  const saveCredential = async () => {
    if (!selectedCloud || !apiKey) return;
    setProviderBusy(true);
    setError(null);
    try {
      await saveCloudCredential(selectedCloud.provider.id, apiKey);
      setApiKey("");
      await refreshCloudStatuses();
    } catch {
      setError("Der Schlüssel konnte nicht im Betriebssystem-Schlüsselspeicher gesichert werden.");
    } finally {
      setProviderBusy(false);
    }
  };

  const removeCredential = async () => {
    if (!selectedCloud) return;
    setProviderBusy(true);
    setError(null);
    try {
      await deleteCloudCredential(selectedCloud.provider.id);
      setModels([]);
      setModel("");
      await refreshCloudStatuses();
    } catch {
      setError("Der gespeicherte Schlüssel konnte nicht gelöscht werden.");
    } finally {
      setProviderBusy(false);
    }
  };

  const loadCloudModels = async () => {
    if (!selectedCloud?.credentialConfigured) return;
    setProviderBusy(true);
    setError(null);
    try {
      const availableModels = await listCloudModels(selectedCloud.provider.id);
      const ids = availableModels.map(({ id }) => id);
      setModels(ids);
      setModel(ids[0] ?? "");
      if (ids.length === 0) setError("Der Provider hat keine verwendbaren Textmodelle gemeldet.");
    } catch {
      setError("Die Cloudmodelle konnten nicht abgerufen werden. Prüfe Schlüssel und Verbindung.");
    } finally {
      setProviderBusy(false);
    }
  };

  const loadPreview = async () => {
    if (!activeDocumentId || !model || !instruction.trim()) return;
    setError(null);
    setApproved(false);
    try {
      setPreview(
        await previewAiContext(projectId, activeDocumentId, providerId, model, instruction.trim()),
      );
    } catch {
      setError("Die laufbezogene Freigabevorschau konnte nicht erzeugt werden.");
    }
  };

  const generate = async () => {
    if (!preview || !activeDocumentId || !approved) return;
    setBusy(true);
    setError(null);
    setDraft("");
    setProvenance(null);
    const currentGeneration = ++generationId.current;
    const input: StreamTextDraftInput = {
      projectId,
      documentId: activeDocumentId,
      expectedRevision: preview.revision,
      model: preview.model,
      instruction: preview.instruction,
      contextApproved: true,
      providerId: preview.providerId,
      releaseId: preview.releaseId,
    };
    try {
      await streamTextDraft(input, (event: TextStreamEvent) => {
        if (generationId.current !== currentGeneration) return;
        if (event.type === "delta") setDraft((current) => current + event.text);
        if (event.type === "completed") setProvenance(event.provenance);
      });
    } catch {
      setError(
        "Der Textentwurf ist fehlgeschlagen. Prüfe Provider, Modell, Schlüssel und den freigegebenen Szenenstand.",
      );
    } finally {
      setApproved(false);
      setPreview(null);
      setBusy(false);
    }
  };

  const providerReady =
    providerId === OLLAMA_ID
      ? Boolean(ollama?.available && model)
      : Boolean(selectedCloud?.credentialConfigured && model);

  return (
    <section className="ai-panel" aria-labelledby="ai-heading">
      <span className="eyebrow">Kontrollierter Entwurfsraum</span>
      <h2 id="ai-heading">Textentwurf mit optionalen Providern</h2>
      <p>
        KRAQEL funktioniert ohne Provider. Cloudzugriffe erfolgen nur nach einer vollständigen,
        einmaligen Vorschau; Ergebnisse bleiben verwerfbare Entwürfe.
      </p>

      <div className="ai-grid">
        <div className="ai-controls">
          <label htmlFor="ai-provider">Textprovider</label>
          <select
            id="ai-provider"
            value={providerId}
            disabled={busy || providerBusy}
            onChange={(event) => chooseProvider(event.target.value)}
          >
            <option value={OLLAMA_ID}>Ollama (lokal)</option>
            {cloudProviders.map(({ provider }) => (
              <option key={provider.id} value={provider.id}>
                {provider.displayName} (Cloud)
              </option>
            ))}
          </select>

          {providerId === OLLAMA_ID && ollama && !ollama.available ? (
            <p className="notice-banner" role="status">
              Ollama ist nicht erreichbar. Alle übrigen KRAQEL-Funktionen bleiben verfügbar.
            </p>
          ) : null}

          {selectedCloud ? (
            <fieldset className="credential-panel">
              <legend>API-Schlüssel für {selectedCloud.provider.displayName}</legend>
              <p className="muted">
                Status:{" "}
                {selectedCloud.credentialConfigured ? "sicher gespeichert" : "nicht eingerichtet"}.
                Der gespeicherte Wert wird nie wieder angezeigt.
              </p>
              <label htmlFor="cloud-api-key">Neuen Schlüssel speichern oder ersetzen</label>
              <input
                id="cloud-api-key"
                type="password"
                autoComplete="off"
                maxLength={4096}
                value={apiKey}
                disabled={busy || providerBusy}
                onChange={(event) => setApiKey(event.target.value)}
              />
              <div className="provider-actions">
                <button
                  type="button"
                  className="secondary"
                  disabled={providerBusy || apiKey.length < 8}
                  onClick={() => void saveCredential()}
                >
                  Im Betriebssystem speichern
                </button>
                <button
                  type="button"
                  className="secondary"
                  disabled={providerBusy || !selectedCloud.credentialConfigured}
                  onClick={() => void removeCredential()}
                >
                  Gespeicherten Schlüssel löschen
                </button>
                <button
                  type="button"
                  disabled={providerBusy || !selectedCloud.credentialConfigured}
                  onClick={() => void loadCloudModels()}
                >
                  Verbindung freigeben und Modelle abrufen
                </button>
              </div>
            </fieldset>
          ) : null}

          <label htmlFor="ai-model">Modell</label>
          <select
            id="ai-model"
            value={model}
            disabled={busy || providerBusy || models.length === 0}
            onChange={(event) => setModel(event.target.value)}
          >
            {models.map((id) => (
              <option key={id}>{id}</option>
            ))}
          </select>

          <label htmlFor="ai-instruction">Arbeitsauftrag</label>
          <textarea
            id="ai-instruction"
            maxLength={4000}
            rows={5}
            value={instruction}
            disabled={busy}
            placeholder="Zum Beispiel: Formuliere drei mögliche nächste Absätze."
            onChange={(event) => setInstruction(event.target.value)}
          />

          <button
            type="button"
            className="secondary"
            disabled={!activeDocumentId || !providerReady || !instruction.trim() || busy}
            onClick={() => void loadPreview()}
          >
            Vollständige Freigabevorschau anzeigen
          </button>

          {preview ? (
            <ReleasePreview preview={preview} approved={approved} onApprove={setApproved} />
          ) : null}

          <button
            type="button"
            disabled={busy || !preview || !approved}
            onClick={() => void generate()}
          >
            {busy
              ? "Entwurf strömt …"
              : `Entwurf mit ${selectedProviderName ?? "Provider"} starten`}
          </button>
        </div>

        <div className="draft-panel" aria-live="polite">
          <div className="draft-heading">
            <h3>Verwerfbarer Entwurf</h3>
            <button
              type="button"
              className="secondary compact"
              disabled={busy || !draft}
              onClick={() => {
                setDraft("");
                setProvenance(null);
              }}
            >
              Verwerfen
            </button>
          </div>
          <pre>{draft || "Noch kein Entwurf. Das Manuskript wartet gelassen."}</pre>
          {provenance ? <p className="muted">Herkunft: {provenance}</p> : null}
        </div>
      </div>

      {error ? (
        <p className="error-banner" role="alert">
          {error}
        </p>
      ) : null}
    </section>
  );
}

function ReleasePreview({
  preview,
  approved,
  onApprove,
}: {
  readonly preview: AiContextPreview;
  readonly approved: boolean;
  readonly onApprove: (approved: boolean) => void;
}) {
  const destination = preview.providerLocation === "cloud" ? "Cloudprovider" : "lokalen Dienst";
  return (
    <div className="context-preview">
      <h3>Einmalige Freigabevorschau</h3>
      <dl className="release-details">
        <div>
          <dt>Ziel</dt>
          <dd>{preview.providerDisplayName}</dd>
        </div>
        <div>
          <dt>Modell</dt>
          <dd>{preview.model}</dd>
        </div>
        {Object.entries(preview.metadata).map(([key, value]) => (
          <div key={key}>
            <dt>{key}</dt>
            <dd>{value}</dd>
          </div>
        ))}
      </dl>
      <h4>Arbeitsauftrag</h4>
      <pre>{preview.instruction}</pre>
      <h4>Freigegebener Szenentext</h4>
      <pre>{preview.text || "(Die Szene enthält noch keinen Text.)"}</pre>
      <p className="muted">Medien: keine. Weitere Metadaten: keine.</p>
      <label className="consent-row">
        <input
          type="checkbox"
          checked={approved}
          onChange={(event) => onApprove(event.target.checked)}
        />
        Genau diese Texte und Metadaten einmalig an den {destination} freigeben
      </label>
    </div>
  );
}
