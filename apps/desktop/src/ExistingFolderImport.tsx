import { useEffect, useState } from "react";
import {
  KraqelCommandError,
  cancelExistingFolder,
  previewExistingFolder,
  receiveExistingFolder,
  type ExtraGroup,
  type FolderPreview,
} from "./commands";

const labels: Record<ExtraGroup, string> = {
  backups: "Sicherungen",
  exports: "Exporte",
  git: "Git-Historie",
  other: "Weitere Dateien",
};
const groups = Object.keys(labels) as ExtraGroup[];

export function ExistingFolderImport({
  busy,
  setBusy,
  refresh,
}: {
  busy: boolean;
  setBusy: (value: boolean) => void;
  refresh: () => Promise<void>;
}) {
  const [preview, setPreview] = useState<FolderPreview | null>(null);
  const [selected, setSelected] = useState<ExtraGroup[]>([]);
  const [notice, setNotice] = useState("");
  const [error, setError] = useState("");
  useEffect(
    () => () => {
      if (preview) void cancelExistingFolder(preview.token).catch(() => undefined);
    },
    [preview],
  );
  const choose = async () => {
    setBusy(true);
    setPreview(null);
    setSelected([]);
    setNotice("");
    setError("");
    try {
      setPreview(await previewExistingFolder());
    } catch {
      setError(
        "Der lokale Projektbestand konnte nicht sicher gelesen werden. Schließe das Quellprojekt und wähle seinen Projektordner erneut.",
      );
    } finally {
      setBusy(false);
    }
  };
  const receive = async () => {
    if (!preview) return;
    setBusy(true);
    setError("");
    setNotice("");
    try {
      const result = await receiveExistingFolder(preview.token, selected);
      if (result === "cancel") setNotice("Übernahme abgebrochen.");
      else {
        await refresh();
        if (result === "archiveFailed")
          setError(
            "Die Arbeitskopie ist in der Projektliste. Das Zusatzarchiv konnte am gewählten Ziel nicht veröffentlicht werden. Die Originaldateien liegen weiter im Quellordner.",
          );
        else
          setNotice(
            selected.length
              ? "Arbeitskopie angelegt und Zusatzarchiv gespeichert."
              : "Die Arbeitskopie ist in der Projektliste verfügbar.",
          );
      }
    } catch (cause) {
      setError(
        cause instanceof KraqelCommandError && cause.code === "CONFLICT"
          ? "Ein Projekt mit dieser Kennung ist bereits vorhanden. Verwende für den Vergleich den Versionsdialog."
          : "Übernahme gesperrt. Prüfe Quelle und Ziel und lade die Vorschau erneut.",
      );
    } finally {
      setPreview(null);
      setBusy(false);
    }
  };
  return (
    <section className="project-card" aria-labelledby="existing-folder-title">
      <h2 id="existing-folder-title">Vorhandenen Windows-Projektordner übernehmen</h2>
      <p>
        Wähle den lokalen Ordner mit der KRAQEL-Projektdatenbank. Nach der Vorschau kannst du eine
        separate Arbeitskopie anlegen.
      </p>
      <button disabled={busy} onClick={() => void choose()}>
        Projektordner auswählen
      </button>
      {preview && (
        <div>
          <p>
            Arbeitskopie mit {preview.documents} Dokumenten und {preview.images} Bildern.
          </p>
          {groups.some((group) => preview.extras[group] > 0) && (
            <fieldset disabled={busy}>
              <legend>Separat mit Passphrase archivieren</legend>
              {groups
                .filter((group) => preview.extras[group] > 0)
                .map((group) => (
                  <label key={group}>
                    <input
                      type="checkbox"
                      checked={selected.includes(group)}
                      onChange={(event) =>
                        setSelected(
                          event.target.checked
                            ? [...selected, group]
                            : selected.filter((item) => item !== group),
                        )
                      }
                    />
                    {labels[group]} ({preview.extras[group]} Dateien)
                  </label>
                ))}
            </fieldset>
          )}
          <button disabled={busy} onClick={() => void receive()}>
            Arbeitskopie anlegen
          </button>
          <button
            type="button"
            className="secondary"
            disabled={busy}
            onClick={() => setPreview(null)}
          >
            Vorschau schließen
          </button>
        </div>
      )}
      {notice && <p role="status">{notice}</p>}
      {error && <p role="alert">{error}</p>}
    </section>
  );
}
