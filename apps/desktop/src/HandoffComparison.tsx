import { useState, type FormEvent } from "react";
import type { ProjectId } from "@kraqel/domain";
import { HandoffItemPanel, type ItemPreference } from "./HandoffItemPanel";
import { compareProjectHandoff, type HandoffSource } from "./commands";

const areas: Record<string, string> = {
  project: "Projekteigenschaften",
  document: "Manuskript",
  entity: "Story-Bibel",
  relation: "Beziehungen",
  link: "Verknüpfungen",
  storyline: "Handlungsstränge",
  plot_card: "Plotkarten",
  timeline: "Zeitachsen",
  event: "Ereignisse",
  event_document_link: "Ereignis–Dokument-Verknüpfungen",
  event_entity_link: "Ereignis–Story-Bibel-Verknüpfungen",
  media_files: "Bild- und Vorschaudateien",
  media_asset: "Medieneinträge",
  asset_link: "Medienverknüpfungen",
  visual_profile: "Visuelle Profile",
  style_preset: "Stilvorlagen",
  image_prompt_draft: "Bildprompt-Entwürfe",
  image_prompt_draft_profile: "Profilzuordnungen der Bildprompt-Entwürfe",
};

export function HandoffComparison({ projectId }: { projectId: ProjectId }) {
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState("");
  const [source, setSource] = useState<HandoffSource | null>(null);
  const [preference, setPreference] = useState<ItemPreference>({ kind: "entity", selected: "" });
  async function compare(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const data = new FormData(event.currentTarget);
    const input = {
      folder: String(data.get("folder")).trim(),
      manifest: String(data.get("digest")).trim().toLowerCase(),
    };
    const includeMedia = data.has("includeMedia");
    setBusy(true);
    setSource(null);
    setNotice("");
    try {
      const changes = await compareProjectHandoff(
        projectId,
        input.folder,
        input.manifest,
        includeMedia,
      );
      setSource(input);
      setNotice(
        changes.length
          ? "Unterschiede in folgenden Bereichen: " +
              changes.map((k) => areas[k] ?? "Weiterer Projektbereich").join(", ")
          : includeMedia
            ? "Alle gespeicherten Datenbankeinträge und referenzierten Bilddateien stimmen überein."
            : "Alle gespeicherten Datenbankeinträge stimmen überein.",
      );
    } catch {
      setNotice(
        includeMedia
          ? "Vergleich fehlgeschlagen. Prüfe Ordnername, Inventarprüfsumme und Projektkennung sowie die referenzierten Bilder und Vorschaubilder auf beiden Seiten."
          : "Vergleich fehlgeschlagen. Prüfe Ordnername, Inventarprüfsumme und Projektkennung.",
      );
    } finally {
      setBusy(false);
    }
  }
  return (
    <section className="version-panel" aria-labelledby="incoming-comparison">
      <h3 id="incoming-comparison">Eingangskopie vergleichen</h3>
      <p>
        Vergleicht die gespeicherten Datenbankeinträge mit der Arbeitskopie in{" "}
        <code>projects\empfang</code>. Die Übersicht zeigt Unterschiede pro Bereich.
      </p>
      <form
        onSubmit={compare}
        onChange={() => {
          setNotice("");
          setSource(null);
        }}
      >
        <label htmlFor="comparison-folder">Ordnername im Eingang</label>
        <input
          id="comparison-folder"
          name="folder"
          required
          maxLength={47}
          disabled={busy}
          autoComplete="off"
        />
        <label htmlFor="comparison-digest">Separat bestätigter Inventar-SHA-256</label>
        <input
          id="comparison-digest"
          name="digest"
          required
          pattern="[0-9a-fA-F]{64}"
          maxLength={64}
          disabled={busy}
          autoComplete="off"
          spellCheck={false}
        />
        <label>
          <input type="checkbox" name="includeMedia" disabled={busy} /> Auch referenzierte Bilder
          und Vorschaubilder prüfen
        </label>
        <button type="submit" disabled={busy}>
          Arbeitsdaten vergleichen
        </button>
      </form>
      <p role="status">{notice}</p>
      <HandoffItemPanel
        key={source ? source.folder + source.manifest : "recovery"}
        projectId={projectId}
        source={source}
        preference={preference}
        setPreference={setPreference}
        busy={busy}
        setBusy={setBusy}
      />
    </section>
  );
}
