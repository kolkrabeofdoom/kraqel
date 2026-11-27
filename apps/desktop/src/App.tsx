import { useEffect, useState, type FormEvent } from "react";
import {
  createExampleProject,
  createProject,
  deleteProject,
  importSnapshotProject,
  importDocxProject,
  receiveProjectHandoff,
  KraqelCommandError,
  listProjects,
  openWorkspace,
  startupProject,
  type ProjectOverview,
  type Workspace,
} from "./commands";
import { EditorWorkspace } from "./EditorWorkspace";
import { ExistingFolderImport } from "./ExistingFolderImport";

export function App() {
  const [projects, setProjects] = useState<readonly ProjectOverview[]>([]);
  const [workspace, setWorkspace] = useState<Workspace | null>(null);
  const [name, setName] = useState("");
  const [busy, setBusy] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [pendingDeleteId, setPendingDeleteId] = useState<ProjectOverview["id"] | null>(null);

  const [handoffNotice, setHandoffNotice] = useState("");
  const receive = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const form = event.currentTarget;
    const data = new FormData(form);
    setBusy(true);
    setError(null);
    setHandoffNotice("");
    try {
      await receiveProjectHandoff(
        String(data.get("folder")).trim(),
        String(data.get("digest")).trim().toLowerCase(),
      );
      form.reset();
      setHandoffNotice("Die Arbeitskopie ist jetzt in der Projektliste verfügbar.");
      await refreshProjects();
    } catch (cause) {
      setError(
        cause instanceof KraqelCommandError && cause.code === "CONFLICT"
          ? "Die Projektkennung ist hier bereits belegt. Zum Szenenvergleich öffne das vorhandene Projekt und wähle Werkzeuge → Versionen."
          : "Empfang fehlgeschlagen. Prüfe den Eingangsordner, die separat bestätigte Prüfsumme und die Vollständigkeit der Arbeitskopie.",
      );
    } finally {
      setBusy(false);
    }
  };

  const refreshProjects = async () => {
    try {
      setProjects(await listProjects());
    } catch {
      setError("Die lokalen Projekte konnten nicht gelesen werden.");
    }
  };

  useEffect(() => {
    let active = true;
    async function initialize() {
      try {
        const [items, selected] = await Promise.all([listProjects(), startupProject()]);
        if (!active) return;
        setProjects(items);
        if (selected) {
          try {
            const current = await openWorkspace(selected);
            if (active) setWorkspace(current);
          } catch {
            if (active) setError("Das ausgewählte Projekt konnte nicht geöffnet werden.");
          }
        }
      } catch {
        if (active) setError("Die lokalen Projekte konnten nicht gelesen werden.");
      } finally {
        if (active) setBusy(false);
      }
    }
    void initialize();
    return () => {
      active = false;
    };
  }, []);

  const submitProject = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    if (!name.trim()) return;
    setBusy(true);
    setError(null);
    try {
      const created = await createProject(name);
      setWorkspace(created);
      setName("");
    } catch {
      setError("Das Projekt konnte nicht angelegt werden.");
    } finally {
      setBusy(false);
    }
  };

  const open = async (project: ProjectOverview) => {
    setPendingDeleteId(null);
    setBusy(true);
    setError(null);
    try {
      setWorkspace(await openWorkspace(project.id));
    } catch {
      setError("Das Projekt konnte nicht geöffnet werden.");
    } finally {
      setBusy(false);
    }
  };

  const removeProject = async (project: ProjectOverview) => {
    if (pendingDeleteId !== project.id) {
      setPendingDeleteId(project.id);
      setError(null);
      return;
    }
    setBusy(true);
    setError(null);
    try {
      await deleteProject(project.id);
      setPendingDeleteId(null);
      await refreshProjects();
    } catch {
      setError("Das Projekt konnte nicht gelöscht werden.");
    } finally {
      setBusy(false);
    }
  };

  const createExample = async () => {
    setBusy(true);
    setError(null);
    try {
      setWorkspace(await createExampleProject());
    } catch {
      setError("Das Beispielprojekt konnte nicht angelegt werden.");
    } finally {
      setBusy(false);
    }
  };

  const importSnapshot = async () => {
    setBusy(true);
    setError(null);
    try {
      const imported = await importSnapshotProject();
      if (imported) setWorkspace(imported);
    } catch {
      setError(
        "Das Projektarchiv konnte nicht importiert werden. Unterstützt werden gültige Snapshot-Archive der Versionen 3 und 4 bis 128 MiB.",
      );
    } finally {
      setBusy(false);
    }
  };

  const importManuscript = async () => {
    setBusy(true);
    setError(null);
    try {
      const imported = await importDocxProject();
      if (imported) setWorkspace(imported);
    } catch {
      setError(
        "Das DOCX-Manuskript konnte nicht importiert werden. Prüfe, ob die Datei aus „DOCX zum Wiederimport erzeugen“ stammt und höchstens 8 MiB groß ist. Falls du in Office Absätze oder deren Markierungen entfernt hast, übertrage deine Textänderungen in die vorhandenen Absätze einer frischen Austauschkopie.",
      );
    } finally {
      setBusy(false);
    }
  };

  if (workspace) {
    return (
      <EditorWorkspace
        initialWorkspace={workspace}
        onClose={() => {
          setWorkspace(null);
          void refreshProjects();
        }}
      />
    );
  }

  return (
    <main className="project-shell">
      <header className="project-hero">
        <div className="project-brand">
          <div className="project-wordmark">
            <span className="eyebrow">Offline-first Autorenstudio</span>
            <h1>KRAQEL</h1>
          </div>
          <img
            className="project-author"
            src="/kolkrabe-author.png"
            width={1281}
            height={1227}
            alt=""
            aria-hidden="true"
          />
        </div>
        <p>
          Geschichten ordnen, ohne sie zu domestizieren. Projekte und Manuskripte bleiben lokal auf
          diesem Gerät.
        </p>
      </header>

      <div className="project-grid">
        <section className="project-card" aria-labelledby="new-project-heading">
          <span className="eyebrow">Anfangen</span>
          <h2 id="new-project-heading">Neues Projekt</h2>
          <form onSubmit={(event) => void submitProject(event)}>
            <label htmlFor="project-name">Projektname</label>
            <input
              id="project-name"
              value={name}
              maxLength={120}
              onChange={(event) => setName(event.target.value)}
              placeholder="Arbeitstitel"
              autoComplete="off"
            />
            <button type="submit" disabled={busy || !name.trim()}>
              Projekt anlegen
            </button>
          </form>
        </section>

        <section className="project-card" aria-labelledby="example-project-heading">
          <span className="eyebrow">Mit Inhalt ausprobieren</span>
          <h2 id="example-project-heading">Die Stadt, die den Dienstag verkaufte</h2>
          <p>
            Szenen, Figuren, Timelines, Plotboard und Bilder als vollständig lokale, frei
            bearbeitbare Beispielkopie.
          </p>
          <button type="button" disabled={busy} onClick={() => void createExample()}>
            Abenteuer ausprobieren
          </button>
          <p className="muted">
            Eigene Inhalte: CC BY-SA 4.0. Daggerheart-SRD-Anteile: DPCGL 2.0. Kein Download beim
            Anlegen.
          </p>
        </section>

        <section className="project-card" aria-labelledby="import-project-heading">
          <span className="eyebrow">Vorhandenes Projekt</span>
          <h2 id="import-project-heading">Projektarchiv importieren</h2>
          <p>
            Ein Snapshot-Archiv der Version 3 oder 4 sicher als neue lokale Projektkopie öffnen. Das
            Quellarchiv bleibt unverändert.
          </p>
          <button type="button" disabled={busy} onClick={() => void importSnapshot()}>
            Projektarchiv auswählen
          </button>
          <p className="muted">
            KRAQEL-Archive (.kraqel oder .zip), höchstens 128 MiB. Der Quellpfad bleibt außerhalb
            der Oberfläche.
          </p>
        </section>

        {navigator.userAgent.includes("Windows") && (
          <ExistingFolderImport busy={busy} setBusy={setBusy} refresh={refreshProjects} />
        )}
        <section className="project-card" aria-labelledby="receive-heading">
          <h2 id="receive-heading">Windows-Arbeitskopie empfangen</h2>
          <p>
            Lege den vorbereiteten Ordner in{" "}
            <code>%LOCALAPPDATA%\org.kraqel.desktop\projects\empfang</code> ab. Erstelle den
            Eingangsordner bei Bedarf.
          </p>
          <form onSubmit={(event) => void receive(event)}>
            <label htmlFor="receive-folder">Ordnername aus der Vorbereitung</label>
            <input
              id="receive-folder"
              name="folder"
              placeholder="uebernahme-…"
              required
              maxLength={47}
              autoComplete="off"
            />
            <label htmlFor="receive-digest">Separat bestätigter SHA-256 des Inventars</label>
            <input
              id="receive-digest"
              name="digest"
              required
              pattern="[A-Fa-f0-9]{64}"
              maxLength={64}
              autoComplete="off"
              spellCheck={false}
            />
            <button type="submit" disabled={busy}>
              Als lokales Projekt übernehmen
            </button>
          </form>
          {handoffNotice && <p role="status">{handoffNotice}</p>}
        </section>

        <section className="project-card" aria-labelledby="import-manuscript-heading">
          <span className="eyebrow">Manuskript aus Office</span>
          <h2 id="import-manuscript-heading">KRAQEL-Manuskript aus DOCX</h2>
          <p>
            Importiert einen KRAQEL-Export zum Wiederimport als neues lokales Projekt. Kapitel,
            Szenen, Drehbuchrollen und unterstützte Textformatierungen bleiben erhalten. Vorhandene
            Projekte und die Quelldatei bleiben unverändert.
          </p>
          <p className="muted">
            Ohne Story-Bibel, Plotboard, Timelines, Medien, Profile, Prompts und Versionsgeschichte.
            Für vollständige Projektkopien verwende das Projektarchiv. Gewöhnliche DOCX-Dateien
            kannst du im Editor als einzelne Szene importieren.
          </p>
          <button type="button" disabled={busy} onClick={() => void importManuscript()}>
            Manuskript-DOCX auswählen
          </button>
          <p className="muted">
            KRAQEL-Manuskript v1 oder v2, höchstens 8 MiB. Projekttitel 1–120 Zeichen, Kapitel- und
            Szenentitel bis 240 Zeichen. Textänderungen innerhalb bestehender Absätze sind
            unterstützt; Strukturänderungen in Office können den Wiederimport verhindern.
          </p>
        </section>

        <section className="project-card" aria-labelledby="projects-heading">
          <span className="eyebrow">Auf diesem Gerät</span>
          <h2 id="projects-heading">Projekte</h2>
          {projects.length === 0 ? (
            <p className="muted">Noch kein Projekt. Die leere Liste urteilt nicht.</p>
          ) : (
            <ul className="project-list">
              {projects.map((project) => (
                <li key={project.id}>
                  <div className="project-list-item">
                    <button
                      type="button"
                      className="project-open"
                      disabled={busy}
                      onClick={() => void open(project)}
                    >
                      <strong>{project.name}</strong>
                      <span>Öffnen</span>
                    </button>
                    <button
                      type="button"
                      className={`project-delete secondary ${pendingDeleteId === project.id ? "danger" : ""}`}
                      disabled={busy}
                      aria-label={
                        pendingDeleteId === project.id
                          ? `${project.name} wirklich löschen`
                          : `${project.name} löschen`
                      }
                      onClick={() => void removeProject(project)}
                    >
                      {pendingDeleteId === project.id ? "Wirklich löschen?" : "Löschen"}
                    </button>
                  </div>
                </li>
              ))}
            </ul>
          )}
        </section>
      </div>

      {error ? (
        <p className="error-banner" role="alert">
          {error}
        </p>
      ) : null}
      <footer>
        <span>
          Keine Cloudpflicht. Keine KRAQEL-Telemetrie. Unter Windows kann die WebView2-Laufzeit
          eigene Netzverbindungen aufbauen.
        </span>
        <span>GPL-3.0-or-later</span>
      </footer>
    </main>
  );
}
