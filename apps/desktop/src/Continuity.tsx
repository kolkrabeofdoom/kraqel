import type { EntityId, ProjectId } from "@kraqel/domain";
import { useState } from "react";
import {
  analyzeContinuity,
  type ContinuityFinding,
  type FindingSeverity,
  type SourceAnchor,
} from "./commands";

export function Continuity({
  projectId,
  onSelectDocument,
}: {
  readonly projectId: ProjectId;
  readonly onSelectDocument: (id: EntityId) => void;
}) {
  const [findings, setFindings] = useState<readonly ContinuityFinding[] | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const run = async () => {
    setBusy(true);
    setError(null);
    try {
      setFindings(await analyzeContinuity(projectId));
    } catch {
      setError("Die lokale Kontinuitätsprüfung konnte nicht ausgeführt werden.");
    } finally {
      setBusy(false);
    }
  };

  return (
    <section className="continuity-panel" aria-labelledby="continuity-heading">
      <span className="eyebrow">Reproduzierbare Offline-Prüfung</span>
      <div className="panel-heading">
        <div>
          <h2 id="continuity-heading">Kontinuität</h2>
          <p>
            Die Regeln werten ausschließlich lokale, strukturierte Projektdaten aus. Hinweise sind
            Prüfangebote, keine literarischen Naturgesetze.
          </p>
        </div>
        <button type="button" disabled={busy} onClick={() => void run()}>
          {busy ? "Prüfung läuft …" : "Projekt prüfen"}
        </button>
      </div>

      {error ? <p className="error-banner">{error}</p> : null}
      {findings?.length === 0 ? (
        <p className="notice-banner">
          Keine regelbasierten Befunde. Der Kolkrabe wirkt unauffällig.
        </p>
      ) : null}
      {findings ? (
        <div className="finding-list" aria-live="polite">
          {findings.map((finding, index) => (
            <article
              className={`finding-card severity-${finding.severity}`}
              key={`${finding.ruleId}-${index}`}
            >
              <div className="finding-heading">
                <span className="finding-severity">{severityLabel(finding.severity)}</span>
                <code>{finding.ruleId}</code>
              </div>
              <h3>{finding.title}</h3>
              <p>{finding.explanation}</p>
              <div className="source-list" aria-label="Fundstellen">
                {finding.sources.map((source, sourceIndex) => (
                  <SourceButton
                    key={`${source.id}-${sourceIndex}`}
                    source={source}
                    onSelectDocument={onSelectDocument}
                  />
                ))}
              </div>
            </article>
          ))}
        </div>
      ) : (
        <p className="empty-editor">Noch kein Prüflauf. Die Fakten sitzen ordentlich im Regal.</p>
      )}
    </section>
  );
}

function SourceButton({
  source,
  onSelectDocument,
}: {
  readonly source: SourceAnchor;
  readonly onSelectDocument: (id: EntityId) => void;
}) {
  const position = source.start === null ? "" : ` (Zeichen ${source.start + 1}–${source.end})`;
  return source.documentId ? (
    <button
      type="button"
      className="secondary compact"
      onClick={() => onSelectDocument(source.documentId as EntityId)}
    >
      {source.label}
      {position}
    </button>
  ) : (
    <span className="source-label">
      {source.label}
      {position}
    </span>
  );
}

function severityLabel(severity: FindingSeverity): string {
  if (severity === "conflict") return "Konflikt";
  if (severity === "warning") return "Prüfhinweis";
  return "Information";
}
