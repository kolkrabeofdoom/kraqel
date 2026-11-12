import type { ProjectId } from "@kraqel/domain";
import { FormEvent, useCallback, useEffect, useState } from "react";
import {
  commitProjectSnapshot,
  createProjectSnapshot,
  exportProjectArchive,
  getVersionControlDiff,
  getVersionControlStatus,
  validateProjectSnapshot,
  type SnapshotSummary,
  type VersionControlStatus,
} from "./commands";

interface VersionControlProps {
  readonly projectId: ProjectId;
}

export function VersionControl({ projectId }: VersionControlProps) {
  const [status, setStatus] = useState<VersionControlStatus | null>(null);
  const [summary, setSummary] = useState<SnapshotSummary | null>(null);
  const [diff, setDiff] = useState("");
  const [message, setMessage] = useState("");
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    const nextStatus = await getVersionControlStatus(projectId);
    setStatus(nextStatus);
    setDiff(nextStatus.initialized ? await getVersionControlDiff(projectId) : "");
  }, [projectId]);

  useEffect(() => {
    let active = true;
    void refresh().catch(() => {
      if (active) setError("Der lokale Versionsstand konnte nicht geladen werden.");
    });
    return () => {
      active = false;
    };
  }, [refresh]);

  const createSnapshot = async () => {
    setBusy(true);
    setError(null);
    setNotice(null);
    try {
      const created = await createProjectSnapshot(projectId);
      setSummary(created);
      await refresh();
      setNotice(`Snapshot mit ${created.fileCount} Dateien wurde deterministisch erzeugt.`);
    } catch {
      setError("Der Projektsnapshot konnte nicht erzeugt werden.");
    } finally {
      setBusy(false);
    }
  };

  const validateSnapshot = async () => {
    setBusy(true);
    setError(null);
    setNotice(null);
    try {
      const validated = await validateProjectSnapshot(projectId);
      setSummary(validated);
      setNotice("Der Snapshot ist lesbar und entspricht dem Formatvertrag.");
    } catch {
      setError("Der Projektsnapshot fehlt oder ist ungültig.");
    } finally {
      setBusy(false);
    }
  };

  const exportArchive = async () => {
    setBusy(true);
    setError(null);
    setNotice(null);
    try {
      const exported = await exportProjectArchive(projectId);
      if (exported) {
        setSummary(exported.snapshot);
        await refresh();
        setNotice(
          `Projektarchiv exportiert: ${exported.fileName} (${exported.byteCount.toLocaleString("de-DE")} Bytes).`,
        );
      }
    } catch {
      setError("Das Projektarchiv konnte nicht exportiert werden.");
    } finally {
      setBusy(false);
    }
  };

  const commit = async (event: FormEvent) => {
    event.preventDefault();
    setBusy(true);
    setError(null);
    setNotice(null);
    try {
      const result = await commitProjectSnapshot(projectId, message);
      setMessage("");
      await refresh();
      setNotice(`Snapshot wurde als ${result.commitId.slice(0, 12)} versioniert.`);
    } catch {
      setError("Der Snapshot konnte nicht committed werden. Prüfe Nachricht und Änderungen.");
    } finally {
      setBusy(false);
    }
  };

  return (
    <section className="version-control" aria-labelledby="version-control-title">
      <header className="section-heading">
        <div>
          <span className="eyebrow">Portable Projektgeschichte</span>
          <h2 id="version-control-title">Versionen</h2>
          <p>
            Git erfasst ausschließlich freigegebene Snapshots. Datenbank, Sicherungen und private
            Nebenartefakte bleiben lokal ausgeschlossen.
          </p>
        </div>
        <div className="version-actions">
          <button type="button" disabled={busy} onClick={() => void createSnapshot()}>
            Snapshot erzeugen
          </button>
          <button
            type="button"
            className="secondary"
            disabled={busy}
            onClick={() => void exportArchive()}
          >
            Projektarchiv exportieren
          </button>
          <button
            type="button"
            className="secondary"
            disabled={busy || !status?.initialized}
            onClick={() => void validateSnapshot()}
          >
            Lesend prüfen
          </button>
        </div>
      </header>

      <div className="version-status" aria-live="polite">
        <strong>
          {!status?.initialized
            ? "Noch nicht initialisiert"
            : status.clean
              ? "Snapshot ist unverändert"
              : `${status.changedFiles.length} Änderung(en) bereit`}
        </strong>
        {status?.head ? <span>Letzter Commit: {status.head.slice(0, 12)}</span> : null}
        {summary ? (
          <span>
            Format {summary.formatVersion}, {summary.fileCount} Dateien, {summary.byteCount} Bytes
          </span>
        ) : null}
      </div>

      {notice ? <p className="success-banner">{notice}</p> : null}
      {error ? <p className="error-banner">{error}</p> : null}

      <div className="version-grid">
        <section className="version-panel" aria-labelledby="changes-title">
          <h3 id="changes-title">Geänderte Snapshotdateien</h3>
          {status?.changedFiles.length ? (
            <ul>
              {status.changedFiles.map((file) => (
                <li key={file}>
                  <code>{file}</code>
                </li>
              ))}
            </ul>
          ) : (
            <p>Keine freigegebenen Änderungen.</p>
          )}
        </section>

        <form className="version-panel commit-form" onSubmit={(event) => void commit(event)}>
          <h3>Snapshot committen</h3>
          <label htmlFor="commit-message">Commitnachricht</label>
          <input
            id="commit-message"
            value={message}
            maxLength={200}
            required
            onChange={(event) => setMessage(event.currentTarget.value)}
            placeholder="Was hat sich fachlich geändert?"
          />
          <button type="submit" disabled={busy || !status?.initialized || status.clean}>
            Commit erstellen
          </button>
        </form>
      </div>

      <section className="diff-panel" aria-labelledby="diff-title">
        <h3 id="diff-title">Unified Diff</h3>
        <pre tabIndex={0}>{diff || "Noch kein Diff vorhanden."}</pre>
      </section>
    </section>
  );
}
