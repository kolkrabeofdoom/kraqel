import type { DocumentNode, ProjectId } from "@kraqel/domain";
import { useMemo, useState } from "react";
import {
  secureSceneActivation,
  switchSceneActivation,
  type Workspace,
  type SceneSelection,
  type PreparedHandoff,
  compareSceneBases,
  prepareSceneMerge,
  saveSceneBasis,
  type SceneBasisComparison,
} from "./commands";

const differences = {
  local: "Nur hier geändert",
  incoming: "Nur auf dem anderen Rechner geändert",
  bothEqual: "Auf beiden Seiten gleich geändert",
  conflict: "Unterschiedliche Änderungen · Auswahl erforderlich",
};

const effectLabels = {
  plotCards: "Plotkarten",
  storyLinks: "Story-Bibel-Verknüpfungen",
  timelineLinks: "Timeline-Verknüpfungen",
  mediaLinks: "Medienverknüpfungen",
  promptDrafts: "Bildprompt-Entwürfe",
};

export function SceneBasisPanel({
  projectId,
  documents,
  onActivated,
}: {
  projectId: ProjectId;
  documents: readonly DocumentNode[];
  onActivated: (workspace: Workspace) => void;
}) {
  const titles = useMemo(
    () => new Map<string, string>(documents.map((d) => [d.id, d.title])),
    [documents],
  );
  const [base, setBase] = useState("");
  const [incoming, setIncoming] = useState("");
  const [busy, setBusy] = useState(false);
  const [notice, setNotice] = useState("");
  const [result, setResult] = useState<SceneBasisComparison | null>(null);

  const [selected, setSelected] = useState<string[]>([]);
  const [folder, setFolder] = useState("");
  const [manifest, setManifest] = useState("");
  const [prepared, setPrepared] = useState<{
    selection: SceneSelection;
    candidate: PreparedHandoff;
  } | null>(null);
  const [ticket, setTicket] = useState("");
  async function secure() {
    if (!prepared || busy) return;
    setBusy(true);
    try {
      const proof = await secureSceneActivation(projectId, prepared.selection, prepared.candidate);
      setTicket(proof);
      setPrepared(null);
      setNotice(
        "Die bisherigen Szenen sind lokal gesichert. Bewahre den angezeigten SHA-256 für den Wiederanlauf oder die Rücknahme auf.",
      );
    } catch {
      setNotice("Der Stand konnte nicht gesichert werden. Bitte erneut vergleichen.");
    } finally {
      setBusy(false);
    }
  }
  async function switchState(undo: boolean) {
    if (busy || !/^[0-9a-f]{64}$/.test(ticket)) return;
    setBusy(true);
    try {
      onActivated(await switchSceneActivation(projectId, ticket, undo));
      resetReview(true);
      setNotice(
        undo
          ? "Die vorherigen Szenenfassungen sind wieder aktiv."
          : "Die ausgewählten Szenen sind jetzt im geöffneten Projekt aktiv.",
      );
    } catch {
      setNotice(
        "Der Wechsel wurde nicht bestätigt. Prüfe den lokalen Beleg und den aktuellen Projektstand. Nach einer unterbrochenen Antwort kannst du denselben Vorgang wiederholen.",
      );
    } finally {
      setBusy(false);
    }
  }
  function resetReview(keepTicket = false) {
    if (!keepTicket) setTicket("");
    setPrepared(null);
    setResult(null);
    setSelected([]);
    setFolder("");
    setManifest("");
  }

  async function prepareSelection() {
    if (!result || !selected.length || busy) return;
    setBusy(true);
    setNotice("");
    try {
      const selection = {
        base,
        incoming,
        local: result.localSha256,
        folder,
        manifest,
        scenes: selected,
      };
      const receipt = await prepareSceneMerge(projectId, selection);
      resetReview();
      setPrepared({ selection, candidate: receipt });
      setNotice(
        `Neue Arbeitskopie mit der Auswahl: backups/${receipt.folder}. SHA-256 des Inventars: ${receipt.manifestSha256}. SHA-256 der Szenenbasis: ${receipt.sceneBasisSha256}.`,
      );
    } catch {
      resetReview();
      setNotice(
        "Die Vorbereitung wurde nicht bestätigt. Prüfe den Sicherungsordner und den Inventarhash und vergleiche die Stände erneut. Verschobene Szenen benötigen einen Strukturabgleich.",
      );
    } finally {
      setBusy(false);
    }
  }

  async function save() {
    setBusy(true);
    setNotice("");
    resetReview();
    try {
      const receipt = await saveSceneBasis(projectId);
      setNotice(
        `Basis für ${receipt.sceneCount} Szenen gesichert. Datei im Projektordner: backups/szenenbasis-${receipt.sha256}.json. SHA-256 zum getrennten Abgleich: ${receipt.sha256}`,
      );
    } catch {
      setNotice("Die Szenenbasis konnte nicht gesichert werden.");
    } finally {
      setBusy(false);
    }
  }

  async function compare() {
    setBusy(true);
    setNotice("");
    resetReview();
    try {
      setResult(await compareSceneBases(projectId, base, incoming));
    } catch {
      setNotice(
        "Vergleich gesperrt. Prüfe, ob beide Basisdateien im Sicherungsordner liegen und ihre Prüfsummen zu diesem Projekt gehören.",
      );
    } finally {
      setBusy(false);
    }
  }

  return (
    <details className="version-panel">
      <summary>Szenenstände zwischen zwei Rechnern vergleichen</summary>
      <p>
        Sichere vor dem getrennten Weiterschreiben die gemeinsame Basis. Die kleine Datei enthält
        Szenenkennungen und Prüfsummen. Sichere auf dem anderen Rechner später dessen neuen Stand
        auf dieselbe Weise.
      </p>
      <button type="button" disabled={busy} onClick={() => void save()}>
        Szenenbasis sichern
      </button>
      <p>
        Kopiere beide Basisdateien in den Ordner <code>backups</code> dieses Projekts. Gib die beim
        Sichern separat angezeigten Prüfsummen ein. Verglichen wird mit dem hier gespeicherten
        Stand.
      </p>
      <form
        onSubmit={(event) => {
          event.preventDefault();
          void compare();
        }}
      >
        <fieldset disabled={busy}>
          <legend>Vergleichsstände</legend>
          <label>
            SHA-256 der gemeinsamen Ausgangsbasis
            <input
              required
              pattern="[0-9a-fA-F]{64}"
              maxLength={64}
              value={base}
              onChange={(e) => {
                setBase(e.target.value.trim().toLowerCase());
                resetReview();
                setNotice("");
              }}
            />
          </label>
          <label>
            SHA-256 des Stands vom anderen Rechner
            <input
              required
              pattern="[0-9a-fA-F]{64}"
              maxLength={64}
              value={incoming}
              onChange={(e) => {
                setIncoming(e.target.value.trim().toLowerCase());
                resetReview();
                setNotice("");
              }}
            />
          </label>
          <button type="submit">Szenen lesend vergleichen</button>
        </fieldset>
      </form>
      <p role="status" style={{ overflowWrap: "anywhere" }}>
        {notice ||
          (result && `${result.unchanged} unverändert · ${result.changes.length} mit Änderungen`)}
      </p>
      {result && result.changes.length > 0 && (
        <ul>
          {result.changes.map((change) => (
            <li key={change.sceneId}>
              <strong>{titles.get(change.sceneId) || change.sceneId}</strong> ·{" "}
              {change.baseExists && change.localExists && !change.incomingExists
                ? "Löschung im Abgleich gesperrt · lokal erhalten"
                : differences[change.difference]}
              {!change.baseExists && " · seit der Ausgangsbasis neu"}
              {!change.localExists && " · hier nicht vorhanden"}
              {!change.incomingExists && " · auf dem anderen Rechner nicht vorhanden"}
              {change.difference === "incoming" &&
                change.baseExists &&
                change.localExists &&
                change.incomingExists && (
                  <label>
                    <input
                      type="checkbox"
                      disabled={busy}
                      checked={selected.includes(change.sceneId)}
                      onChange={(e) =>
                        setSelected(
                          e.target.checked
                            ? [...selected, change.sceneId]
                            : selected.filter((id) => id !== change.sceneId),
                        )
                      }
                    />
                    Titel und Inhalt dieser Szene für die neue Kopie auswählen
                  </label>
                )}
              {change.deletionEffects && (
                <div>
                  Bei einer Löschung hier ebenfalls betroffen:{" "}
                  {Object.entries(effectLabels)
                    .flatMap(([key, label]) => {
                      const count = change.deletionEffects?.[key as keyof typeof effectLabels];
                      return count ? [`${label}: ${count}`] : [];
                    })
                    .join(", ")}
                </div>
              )}
            </li>
          ))}
        </ul>
      )}
      {result && selected.length > 0 && (
        <form
          onSubmit={(e) => {
            e.preventDefault();
            void prepareSelection();
          }}
        >
          <fieldset disabled={busy}>
            <legend>Auswahl als eigene Arbeitskopie vorbereiten</legend>
            <p>
              Kopiere den vorbereiteten Ordner des anderen Rechners nach <code>backups</code> dieses
              Projekts. Die neue Kopie enthält die ausgewählten Texte und Titel sowie den übrigen
              lokalen Projektstand. Beide Ausgangsfassungen bleiben erhalten.
            </p>
            <label>
              Quellordner unter backups
              <input
                required
                value={folder}
                pattern="uebernahme-[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}"
                placeholder="uebernahme-…"
                onChange={(e) => setFolder(e.target.value.trim())}
              />
            </label>
            <label>
              Separat bestätigter SHA-256 des Quellinventars
              <input
                required
                value={manifest}
                pattern="[0-9a-f]{64}"
                maxLength={64}
                onChange={(e) => setManifest(e.target.value.trim().toLowerCase())}
              />
            </label>
            <button type="submit">
              Auswahl bestätigen und neue Kopie vorbereiten ({selected.length})
            </button>
          </fieldset>
        </form>
      )}
      {prepared && (
        <fieldset disabled={busy}>
          <legend>Aktivierung vorbereiten</legend>
          <p>
            Für die Rücknahme werden die bisherigen Titel und Inhalte der ausgewählten Szenen
            unverschlüsselt im lokalen Sicherungsordner gespeichert.
          </p>
          <button type="button" onClick={() => void secure()}>
            Vorherige Fassungen sichern
          </button>
        </fieldset>
      )}
      <details open={ticket ? true : undefined}>
        <summary>Aktivieren oder vorherige Szenen wiederherstellen</summary>
        <fieldset disabled={busy}>
          <legend>Lokaler Aktivierungsbeleg</legend>
          <label>
            SHA-256 aus der lokalen Sicherung
            <input
              value={ticket}
              maxLength={64}
              onChange={(e) => setTicket(e.target.value.trim().toLowerCase())}
            />
          </label>
          <p>
            Beide Aktionen betreffen die ausgewählten Szenen im geöffneten Projekt. Nach einem
            Neustart denselben vor der Aktivierung gesicherten Beleg verwenden.
          </p>
          <button
            type="button"
            disabled={!/^[0-9a-f]{64}$/.test(ticket)}
            onClick={() => void switchState(false)}
          >
            Auswahl jetzt im Projekt aktivieren
          </button>
          <button
            type="button"
            disabled={!/^[0-9a-f]{64}$/.test(ticket)}
            onClick={() => void switchState(true)}
          >
            Vorherige Szenen jetzt wiederherstellen
          </button>
        </fieldset>
      </details>
    </details>
  );
}
