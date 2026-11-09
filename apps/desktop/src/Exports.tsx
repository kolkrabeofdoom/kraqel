import type { ProjectId } from "@kraqel/domain";
import { useState } from "react";
import { exportManuscript, type ExportFormat, type ExportResult } from "./commands";

const formats: readonly { format: ExportFormat; label: string; description: string }[] = [
  { format: "markdown", label: "Markdown", description: "Portabel und gut versionierbar" },
  { format: "html", label: "HTML", description: "Eigenständig im Browser lesbar" },
  { format: "docx", label: "DOCX", description: "Für Office-Programme und Lektor*innen" },
  { format: "pdf", label: "PDF", description: "Für Weitergabe und Ausdruck" },
];

export function Exports({ projectId }: { readonly projectId: ProjectId }) {
  const [busy, setBusy] = useState<ExportFormat | null>(null);
  const [result, setResult] = useState<ExportResult | null>(null);
  const [error, setError] = useState<string | null>(null);

  const run = async (format: ExportFormat) => {
    setBusy(format);
    setError(null);
    setResult(null);
    try {
      setResult(await exportManuscript(projectId, format));
    } catch {
      setError(
        "Der Export ist gescheitert. Prüfe, ob das Manuskript nur unterstützte Formatierungen enthält.",
      );
    } finally {
      setBusy(null);
    }
  };

  return (
    <section className="export-panel" aria-labelledby="export-heading">
      <span className="eyebrow">Lokale Ausgabe</span>
      <h2 id="export-heading">Manuskript exportieren</h2>
      <p>
        Kapitel, Szenen, Überschriften, Absätze, Listen, Zitate sowie fett und kursiv gesetzter Text
        werden übernommen. Dateien entstehen ausschließlich im Projektordner <code>exports</code>.
      </p>
      <div className="export-grid">
        {formats.map(({ format, label, description }) => (
          <article key={format}>
            <h3>{label}</h3>
            <p>{description}</p>
            <button type="button" disabled={busy !== null} onClick={() => void run(format)}>
              {busy === format ? "Kolkrabe arbeitet …" : `${label} erzeugen`}
            </button>
          </article>
        ))}
      </div>
      {result ? (
        <p className="success-banner" role="status">
          Export fertig: <strong>{result.fileName}</strong> (
          {result.byteCount.toLocaleString("de-DE")} Bytes)
        </p>
      ) : null}
      {error ? (
        <p className="error-banner" role="alert">
          {error}
        </p>
      ) : null}
      <p className="muted">
        Medien und Import folgen in einer späteren Stufe. Der Kolkrabe führt Buch.
      </p>
    </section>
  );
}
