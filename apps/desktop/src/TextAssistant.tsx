import type { TextProviderStatus, TextStreamEvent } from "@kraqel/ai-contracts";
import type { EntityId, ProjectId } from "@kraqel/domain";
import { useEffect, useRef, useState } from "react";
import {
  listOllamaModels,
  previewAiContext,
  streamTextDraft,
  type StreamTextDraftInput,
} from "./commands";

export function TextAssistant({
  projectId,
  activeDocumentId,
}: {
  readonly projectId: ProjectId;
  readonly activeDocumentId: EntityId | null;
}) {
  const [provider, setProvider] = useState<TextProviderStatus | null>(null);
  const [model, setModel] = useState("");
  const [preview, setPreview] = useState<Awaited<ReturnType<typeof previewAiContext>> | null>(null);
  const [approved, setApproved] = useState(false);
  const [instruction, setInstruction] = useState("");
  const [draft, setDraft] = useState("");
  const [provenance, setProvenance] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const generationId = useRef(0);

  useEffect(() => {
    let active = true;
    void listOllamaModels()
      .then((status) => {
        if (!active) return;
        setProvider(status);
        setModel(status.models[0]?.id ?? "");
      })
      .catch(() => {
        if (active) setError("Der lokale Ollama-Status konnte nicht gelesen werden.");
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
  }, [activeDocumentId]);

  const loadPreview = async () => {
    if (!activeDocumentId) return;
    setError(null);
    setApproved(false);
    try {
      setPreview(await previewAiContext(projectId, activeDocumentId));
    } catch {
      setError("Der Szenenkontext konnte nicht für die Vorschau geladen werden.");
    }
  };

  const generate = async () => {
    if (!preview || !activeDocumentId || !approved || !model || !instruction.trim()) return;
    setBusy(true);
    setError(null);
    setDraft("");
    setProvenance(null);
    const currentGeneration = ++generationId.current;
    const input: StreamTextDraftInput = {
      projectId,
      documentId: activeDocumentId,
      expectedRevision: preview.revision,
      model,
      instruction: instruction.trim(),
      contextApproved: true,
    };
    try {
      await streamTextDraft(input, (event: TextStreamEvent) => {
        if (generationId.current !== currentGeneration) return;
        if (event.type === "delta") setDraft((current) => current + event.text);
        if (event.type === "completed") setProvenance(event.provenance);
      });
    } catch {
      setError(
        "Der lokale Textentwurf ist fehlgeschlagen. Prüfe Ollama, das Modell und ob die Szene seit der Vorschau geändert wurde.",
      );
    } finally {
      setApproved(false);
      setBusy(false);
    }
  };

  return (
    <section className="ai-panel" aria-labelledby="ai-heading">
      <span className="eyebrow">Lokaler Entwurfsraum</span>
      <h2 id="ai-heading">Textentwurf mit Ollama</h2>
      <p>
        Nur der unten angezeigte Szenentext wird nach ausdrücklicher Freigabe an Ollama auf diesem
        Gerät gesendet. Der erzeugte Text bleibt ein Entwurf und verändert das Manuskript nicht.
      </p>

      {!provider ? <p className="muted">Lokalen Provider prüfen …</p> : null}
      {provider && !provider.available ? (
        <p className="notice-banner" role="status">
          Ollama ist nicht erreichbar. KRAQEL bleibt vollständig ohne Text-KI nutzbar.
        </p>
      ) : null}
      {provider?.available && provider.models.length === 0 ? (
        <p className="notice-banner" role="status">
          Ollama läuft, aber es ist kein Textmodell installiert.
        </p>
      ) : null}

      <div className="ai-grid">
        <div className="ai-controls">
          <label htmlFor="ai-model">Lokales Modell</label>
          <select
            id="ai-model"
            value={model}
            disabled={busy || !provider?.available}
            onChange={(event) => setModel(event.target.value)}
          >
            {provider?.models.map(({ id }) => (
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
            disabled={!activeDocumentId || busy}
            onClick={() => void loadPreview()}
          >
            Szenenkontext anzeigen
          </button>

          {preview ? (
            <div className="context-preview">
              <h3>Freigabevorschau: {preview.documentTitle}</h3>
              <pre>{preview.text || "(Die Szene enthält noch keinen Text.)"}</pre>
              <label className="consent-row">
                <input
                  type="checkbox"
                  checked={approved}
                  disabled={busy}
                  onChange={(event) => setApproved(event.target.checked)}
                />
                Genau diesen Text einmalig an Ollama auf diesem Gerät freigeben
              </label>
            </div>
          ) : null}

          <button
            type="button"
            disabled={
              busy || !provider?.available || !model || !preview || !approved || !instruction.trim()
            }
            onClick={() => void generate()}
          >
            {busy ? "Entwurf strömt …" : "Lokalen Entwurf starten"}
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
