import { IllustratedExport } from "./IllustratedExport";
import type { ProjectId } from "@kraqel/domain";
import { useState } from "react";
import {
  exportManuscript,
  exportRoundtripManuscript,
  openExportDirectory,
  type ExportFormat,
  type ExportResult,
} from "./commands";

const formats: readonly { format: ExportFormat; label: string; description: string }[] = [
  { format: "markdown", label: "Markdown", description: "Portabel und gut versionierbar" },
  { format: "html", label: "HTML", description: "Eigenständig im Browser lesbar" },
  { format: "docx", label: "DOCX", description: "Für Office-Programme und Lektor*innen" },
  { format: "pdf", label: "PDF", description: "Für Weitergabe und Ausdruck" },
];

export function Exports({ projectId }: { readonly projectId: ProjectId }) {
  const [busy, setBusy] = useState<ExportFormat | "roundtrip" | null>(null);
  const [openingDirectory, setOpeningDirectory] = useState(false);
  const [result, setResult] = useState<ExportResult | null>(null);
  const [error, setError] = useState<string | null>(null);

  const run = async (format: ExportFormat | "roundtrip") => {
    setBusy(format);
    setError(null);
    setResult(null);
    try {
      setResult(
        await (format === "roundtrip"
          ? exportRoundtripManuscript(projectId)
          : exportManuscript(projectId, format)),
      );
    } catch {
      setError(
        "Der Export ist gescheitert. Prüfe, ob das Manuskript nur unterstützte Formatierungen enthält.",
      );
    } finally {
      setBusy(null);
    }
  };

  const openDirectory = async () => {
    setOpeningDirectory(true);
    setError(null);
    try {
      await openExportDirectory(projectId);
    } catch {
      setError("Der Exportordner konnte nicht geöffnet werden.");
    } finally {
      setOpeningDirectory(false);
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
      <button
        type="button"
        className="secondary export-folder-button"
        disabled={openingDirectory}
        onClick={() => void openDirectory()}
      >
        {openingDirectory ? "Exportordner wird geöffnet …" : "Exportordner öffnen"}
      </button>
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
      <article>
        <h3>DOCX zum Wiederimport</h3>
        <p>
          Für die Rückkehr aus Word oder LibreOffice als neue Projektkopie: Kapitel, Szenen und
          Textformatierungen bleiben erhalten. Ohne Medien, Story-Bibel und weitere Projektdaten.
          Bearbeite Text innerhalb bestehender Absätze; Strukturänderungen in Office können den
          Wiederimport verhindern.
        </p>
        <button type="button" disabled={busy !== null} onClick={() => void run("roundtrip")}>
          {busy === "roundtrip" ? "Kolkrabe arbeitet …" : "DOCX zum Wiederimport erzeugen"}
        </button>
      </article>
      <IllustratedExport projectId={projectId} disabled={busy !== null} />
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
        Für vollständige Projektkopien einschließlich Medien verwende das Projektarchiv unter
        Versionen.
      </p>
    </section>
  );
}
