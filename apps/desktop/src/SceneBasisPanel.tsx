import type { DocumentNode, ProjectId } from "@kraqel/domain";
import { useMemo, useState } from "react";
import { compareSceneBases, saveSceneBasis, type SceneBasisComparison } from "./commands";

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
}: {
  projectId: ProjectId;
  documents: readonly DocumentNode[];
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

  async function save() {
    setBusy(true);
    setNotice("");
    setResult(null);
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
    setResult(null);
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
                setResult(null);
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
                setResult(null);
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
              {differences[change.difference]}
              {!change.baseExists && " · seit der Ausgangsbasis neu"}
              {!change.localExists && " · hier nicht vorhanden"}
              {!change.incomingExists && " · auf dem anderen Rechner nicht vorhanden"}
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
    </details>
  );
}
