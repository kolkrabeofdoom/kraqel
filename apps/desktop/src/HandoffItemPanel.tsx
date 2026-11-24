import { useState } from "react";
import type { ProjectId } from "@kraqel/domain";
import {
  listHandoffItems,
  reviewHandoffItem,
  secureHandoffItem,
  switchHandoffItem,
  type HandoffItemChange,
  type HandoffItemKind,
  type HandoffItemSelection,
  type HandoffSource,
  type PreparedHandoff,
} from "./commands";

const kinds: Record<HandoffItemKind, string> = {
  entity: "Story-Bibel-Einträge",
  relation: "Beziehungen",
  link: "Verknüpfungen",
  plotCard: "Plotkarten",
  event: "Timeline-Ereignisse",
  eventDocumentLink: "Ereignis–Dokument-Zuordnungen",
  eventEntityLink: "Ereignis–Story-Bibel-Zuordnungen",
};
const keyOf = (change: HandoffItemChange) => change.key.ids.join(" · ");

export interface ItemPreference {
  kind: HandoffItemKind;
  selected: string;
}

export function HandoffItemPanel({
  projectId,
  source,
  preference,
  setPreference,
  busy,
  setBusy,
}: {
  projectId: ProjectId;
  source: HandoffSource | null;
  preference: ItemPreference;
  setPreference: (choice: ItemPreference) => void;
  busy: boolean;
  setBusy: (value: boolean) => void;
}) {
  const { kind, selected } = preference;
  const [rows, setRows] = useState<HandoffItemChange[] | null>(null);
  const [prepared, setPrepared] = useState<{
    selection: HandoffItemSelection;
    candidate: PreparedHandoff;
  } | null>(null);
  const [ticket, setTicket] = useState("");
  const [notice, setNotice] = useState("");
  const change = rows?.find((row) => keyOf(row) === selected);
  async function load() {
    if (!source) return;
    setBusy(true);
    setPrepared(null);
    setNotice("");
    try {
      const next = await listHandoffItems(projectId, source, kind);
      setRows(next);
      if (!next.some((row) => keyOf(row) === selected)) setPreference({ kind, selected: "" });
      if (!next.length) setNotice("In diesem Bereich stimmen die Einträge überein.");
    } catch {
      setRows(null);
      setNotice(
        "Die Auswahl konnte nicht geprüft werden. Prüfe Eingangskopie und Inventarprüfsumme.",
      );
    } finally {
      setBusy(false);
    }
  }

  async function review() {
    if (!change?.incoming || !source) return;
    setTicket("");
    setBusy(true);
    setPrepared(null);
    try {
      const selection = { ...source, change };
      const result = await reviewHandoffItem(projectId, selection);
      if (result.choice === "incoming") {
        setPrepared({ selection, candidate: result.candidate });
        setNotice("Die bestätigte Fassung liegt als separate Arbeitskopie vor.");
      } else
        setNotice(
          result.choice === "local"
            ? "Die lokale Fassung bleibt erhalten."
            : "Fassungswahl geschlossen.",
        );
    } catch {
      setRows(null);
      setNotice(
        "Fassungswahl fehlgeschlagen. Lade die Einträge erneut und prüfe deine Auswahl. Kontrolliere auch, ob die benötigten Bezugseinträge im Ziel vorhanden sind.",
      );
    } finally {
      setBusy(false);
    }
  }

  async function secure() {
    if (!prepared) return;
    setBusy(true);
    try {
      setTicket(await secureHandoffItem(projectId, prepared.selection, prepared.candidate));
      setPrepared(null);
      setNotice("Rücknahme vorbereitet. Bewahre den Beleg für den Wiederanlauf auf.");
    } catch {
      setPrepared(null);
      setRows(null);
      setNotice("Sicherung fehlgeschlagen. Lade die Einträge erneut und prüfe deine Auswahl.");
    } finally {
      setBusy(false);
    }
  }

  async function apply(undo: boolean) {
    setBusy(true);
    try {
      await switchHandoffItem(projectId, ticket, undo);
      setRows(null);
      setPrepared(null);
      setNotice(
        undo
          ? "Der vorherige Eintragsstand ist wieder aktiv."
          : "Die ausgewählte Fassung ist jetzt aktiv.",
      );
    } catch {
      setRows(null);
      setPrepared(null);
      setNotice(
        "Der Wechsel wurde nicht bestätigt. Beleg aufbewahren und Einträge für einen aktuellen Vergleich erneut laden.",
      );
    } finally {
      setBusy(false);
    }
  }

  return (
    <details className="version-panel">
      <summary>Einzelne Projekteinträge übernehmen</summary>
      {source && (
        <p>
          Wähle unter Windows einen Eintrag anhand seiner Kennung. Die vollständigen Felder siehst
          du nach Bestätigung im isolierten Fenster.
        </p>
      )}
      {source ? (
        <>
          <label>
            Projektbereich
            <select
              value={kind}
              disabled={busy}
              onChange={(e) => {
                setTicket("");
                setPreference({ kind: e.target.value as HandoffItemKind, selected: "" });
                setPrepared(null);
                setRows(null);
                setNotice("");
              }}
            >
              {Object.entries(kinds).map(([value, label]) => (
                <option key={value} value={value}>
                  {label}
                </option>
              ))}
            </select>
          </label>
          <button type="button" disabled={busy} onClick={() => void load()}>
            Abweichende Einträge laden
          </button>
          {rows && rows.length > 0 && (
            <label>
              Eintrag nach Kennung auswählen
              <select
                value={selected}
                disabled={busy}
                onChange={(e) => {
                  setTicket("");
                  setPreference({ kind, selected: e.target.value });
                  setPrepared(null);
                  setNotice("");
                }}
              >
                <option value="">Eintrag wählen</option>
                {rows.map((row) => (
                  <option key={keyOf(row)} value={keyOf(row)}>
                    {row.incoming ? (row.local ? "Geändert" : "Nur im Eingang") : "Nur lokal"} ·{" "}
                    {keyOf(row)}
                  </option>
                ))}
              </select>
            </label>
          )}
          {change &&
            (change.incoming ? (
              <button type="button" disabled={busy} onClick={() => void review()}>
                Fassungen im isolierten Fenster öffnen
              </button>
            ) : (
              <p>Dieser Eintrag bleibt lokal erhalten.</p>
            ))}
          {prepared && (
            <>
              <p>
                Vorbereitete Kopie: <code>{prepared.candidate.folder}</code>
              </p>
              <button type="button" disabled={busy} onClick={() => void secure()}>
                Rücknahme für diesen Eintrag sichern
              </button>
            </>
          )}
        </>
      ) : (
        <p>
          Vergleiche unter Windows zuerst die Eingangskopie. Für einen früheren Vorgang kannst du
          direkt den lokalen Beleg eingeben.
        </p>
      )}
      <label>
        Lokaler Beleg für Aktivierung oder Rücknahme
        <input
          value={ticket}
          disabled={busy}
          maxLength={64}
          autoComplete="off"
          spellCheck={false}
          onChange={(e) => setTicket(e.target.value.trim())}
        />
      </label>
      <button
        type="button"
        disabled={busy || !/^[a-f0-9]{64}$/.test(ticket)}
        onClick={() => void apply(false)}
      >
        Eintrag aktivieren
      </button>
      <button
        type="button"
        disabled={busy || !/^[a-f0-9]{64}$/.test(ticket)}
        onClick={() => void apply(true)}
      >
        Eintragsübernahme zurücknehmen
      </button>
      <p role="status">{notice}</p>
    </details>
  );
}
