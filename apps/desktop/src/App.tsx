import { useEffect, useState, type FormEvent } from "react";
import {
  createExampleProject,
  createProject,
  deleteProject,
  importSnapshotProject,
  listProjects,
  openWorkspace,
  type ProjectOverview,
  type Workspace,
} from "./commands";
import { EditorWorkspace } from "./EditorWorkspace";

export function App() {
  const [projects, setProjects] = useState<readonly ProjectOverview[]>([]);
  const [workspace, setWorkspace] = useState<Workspace | null>(null);
  const [name, setName] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [pendingDeleteId, setPendingDeleteId] = useState<ProjectOverview["id"] | null>(null);

  const refreshProjects = async () => {
    try {
      setProjects(await listProjects());
    } catch {
      setError("Die lokalen Projekte konnten nicht gelesen werden.");
    }
  };

  useEffect(() => {
    void refreshProjects();
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
        "Das Projektarchiv konnte nicht importiert werden. Unterstützt werden gültige Snapshot-v3-Archive bis 128 MiB.",
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
            Eine Snapshot-v3-Datei sicher als neue lokale Projektkopie öffnen. Das Quellarchiv
            bleibt unverändert.
          </p>
          <button type="button" disabled={busy} onClick={() => void importSnapshot()}>
            Projektarchiv auswählen
          </button>
          <p className="muted">
            KRAQEL-Archive (.kraqel oder .zip), höchstens 128 MiB. Der Quellpfad bleibt außerhalb
            der Oberfläche.
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
        <span>Keine Cloudpflicht. Keine Telemetrie ab Werk.</span>
        <span>GPL-3.0-or-later</span>
      </footer>
    </main>
  );
}
