import { IllustratedExport } from "./IllustratedExport";
import type { DocumentNode, EntityId, ProjectId } from "@kraqel/domain";
import { buildDocumentTree } from "@kraqel/editor";
import { useState } from "react";
import {
  exportManuscript,
  exportRoundtripManuscript,
  exportScreenplayPdf,
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

export function Exports({
  projectId,
  documents,
}: {
  readonly projectId: ProjectId;
  readonly documents: readonly DocumentNode[];
}) {
  const [busy, setBusy] = useState<ExportFormat | "roundtrip" | "screenplay" | null>(null);
  const [sceneIds, setSceneIds] = useState<EntityId[]>([]);
  const [openingDirectory, setOpeningDirectory] = useState(false);
  const [result, setResult] = useState<ExportResult | null>(null);
  const [error, setError] = useState<string | null>(null);

  const run = async (format: ExportFormat | "roundtrip" | "screenplay") => {
    setBusy(format);
    setError(null);
    setResult(null);
    try {
      setResult(
        await (format === "screenplay"
          ? exportScreenplayPdf(projectId, sceneIds)
          : format === "roundtrip"
            ? exportRoundtripManuscript(projectId)
            : exportManuscript(projectId, format)),
      );
    } catch {
      setError(
        format === "screenplay"
          ? "Drehbuch-PDF konnte nicht erzeugt werden. Wähle ausschließlich Drehbuchszenen. Prüfe Szenenkopf, Dialogfolge, Zeichen und Seitenlayout."
          : "Der Export ist gescheitert. Prüfe Formatierungen und bei PDF auch den unterstützten Zeichenumfang und die Layoutgrenzen.",
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
        werden übernommen. Markdown, HTML und das gewöhnliche DOCX geben auch Drehbuchrollen aus.
        Für den Druck von Drehbuchszenen wähle unten „Drehbuch als A4-PDF“. Die Dateien liegen im
        Projektordner <code>exports</code>.
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
        <h3>Drehbuch als A4-PDF</h3>
        <fieldset className="checkbox-grid" disabled={busy !== null}>
          <legend>Drehbuchszenen auswählen</legend>
          {buildDocumentTree(documents).flatMap((chapter) =>
            chapter.children.map((scene) => (
              <label key={scene.id}>
                <input
                  type="checkbox"
                  checked={sceneIds.includes(scene.id)}
                  onChange={(event) =>
                    setSceneIds(
                      event.target.checked
                        ? [...sceneIds, scene.id]
                        : sceneIds.filter((id) => id !== scene.id),
                    )
                  }
                />
                {chapter.title || "Ohne Kapiteltitel"} · {scene.title || "Ohne Szenentitel"}
              </label>
            )),
          )}
        </fieldset>
        <p>
          {sceneIds.length} Szenen ausgewählt. Gedruckt werden ihre Rollenabsätze in
          Baumreihenfolge. Projekt-, Kapitel- und Szenentitel, Bilder und alle übrigen Szenen
          bleiben außerhalb der Ausgabe.
        </p>
        <button
          type="button"
          disabled={busy !== null || sceneIds.length === 0}
          onClick={() => void run("screenplay")}
        >
          Drehbuch-PDF erzeugen
        </button>
      </article>
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
