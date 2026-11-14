import { EditorContent, useEditor, useEditorState } from "@tiptap/react";
import StarterKit from "@tiptap/starter-kit";
import type { DocumentNode, EntityId } from "@kraqel/domain";
import {
  AutosaveController,
  EDITOR_DOCUMENT_VERSION,
  buildDocumentTree,
  countWords,
  type AutosaveStatus,
  type DocumentTreeNode,
  type EditorDocument,
} from "@kraqel/editor";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  createDocument,
  deleteDocument,
  importTextDocument,
  importDocxDocument,
  loadDocument,
  renameDocument,
  reorderDocuments,
  saveDocument,
  type StoredDocument,
  type Workspace,
} from "./commands";
import { StoryBible } from "./StoryBible";
import { PlotBoard } from "./PlotBoard";
import { Timelines } from "./Timelines";
import { VersionControl } from "./VersionControl";
import { Exports } from "./Exports";
import { TextAssistant } from "./TextAssistant";
import { Continuity } from "./Continuity";
import { MediaLibrary } from "./MediaLibrary";
import { VisualProfiles } from "./VisualProfiles";
import { PromptBuilder } from "./PromptBuilder";

interface EditorWorkspaceProps {
  readonly initialWorkspace: Workspace;
  readonly onClose: () => void;
}

type WorkspaceView =
  | "manuscript"
  | "storyBible"
  | "plotBoard"
  | "timelines"
  | "continuity"
  | "media"
  | "visuals"
  | "prompts"
  | "versions"
  | "exports"
  | "textAi";

const navigationGroups: readonly {
  readonly label: string;
  readonly items: readonly { readonly view: WorkspaceView; readonly label: string }[];
}[] = [
  {
    label: "Schreiben",
    items: [
      { view: "manuscript", label: "Manuskript" },
      { view: "storyBible", label: "Story-Bibel" },
    ],
  },
  {
    label: "Planen",
    items: [
      { view: "plotBoard", label: "Plotboard" },
      { view: "timelines", label: "Timelines" },
      { view: "continuity", label: "Kontinuität" },
    ],
  },
  {
    label: "Material",
    items: [
      { view: "media", label: "Medien" },
      { view: "visuals", label: "Visuelle Profile" },
      { view: "prompts", label: "Bildprompts" },
    ],
  },
  {
    label: "Werkzeuge",
    items: [
      { view: "versions", label: "Versionen" },
      { view: "exports", label: "Export" },
      { view: "textAi", label: "KI-Entwurf" },
    ],
  },
];

export function EditorWorkspace({ initialWorkspace, onClose }: EditorWorkspaceProps) {
  const [workspace, setWorkspace] = useState(initialWorkspace);
  const [activeId, setActiveId] = useState<EntityId | null>(
    firstSceneId(initialWorkspace.documents),
  );
  const [stored, setStored] = useState<StoredDocument | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [view, setView] = useState<WorkspaceView>("manuscript");
  const [docxChapter, setDocxChapter] = useState<EntityId | null>(null);
  const [importing, setImporting] = useState(false);
  const docxHeading = useRef<HTMLHeadingElement>(null);
  useEffect(() => {
    if (docxChapter) docxHeading.current?.focus();
  }, [docxChapter]);
  const tree = useMemo(() => buildDocumentTree(workspace.documents), [workspace.documents]);

  useEffect(() => {
    if (!activeId) {
      setStored(null);
      return;
    }
    let active = true;
    setLoading(true);
    setError(null);
    void loadDocument(workspace.project.id, activeId)
      .then((document) => {
        if (active) setStored(document);
      })
      .catch(() => {
        if (active) setError("Die Szene konnte nicht geladen werden.");
      })
      .finally(() => {
        if (active) setLoading(false);
      });
    return () => {
      active = false;
    };
  }, [activeId, workspace.project.id]);

  const addChapter = async () => {
    try {
      const node = await createDocument({
        projectId: workspace.project.id,
        parentId: null,
        kind: "chapter",
        title: `Kapitel ${chapterCount(workspace.documents) + 1}`,
      });
      setWorkspace((current) => ({ ...current, documents: [...current.documents, node] }));
    } catch {
      setError("Das Kapitel konnte nicht angelegt werden.");
    }
  };

  const addScene = async (chapterId: EntityId) => {
    try {
      const number =
        workspace.documents.filter(({ parentId }) => parentId === chapterId).length + 1;
      const node = await createDocument({
        projectId: workspace.project.id,
        parentId: chapterId,
        kind: "scene",
        title: `Szene ${number}`,
      });
      setWorkspace((current) => ({ ...current, documents: [...current.documents, node] }));
      setActiveId(node.id);
    } catch {
      setError("Die Szene konnte nicht angelegt werden.");
    }
  };

  const importScene = async (chapterId: EntityId) => {
    if (importing) return;
    setImporting(true);
    try {
      const node = await importTextDocument(workspace.project.id, chapterId);
      if (!node) return;
      setWorkspace((current) => ({ ...current, documents: [...current.documents, node] }));
      setActiveId(node.id);
      setError(null);
    } catch {
      setError(
        "Die Textdatei konnte nicht importiert werden. Unterstützt werden UTF-8-Dateien bis 2 MiB.",
      );
    } finally {
      setImporting(false);
    }
  };

  const importDocx = async () => {
    if (!docxChapter || importing) return;
    setImporting(true);
    try {
      const node = await importDocxDocument(workspace.project.id, docxChapter);
      if (!node) return;
      setWorkspace((current) => ({ ...current, documents: [...current.documents, node] }));
      setActiveId(node.id);
      setError(null);
    } catch {
      setError(
        "Das DOCX-Dokument konnte nicht importiert werden. Es ist beschädigt, zu groß oder enthält nicht unterstützte Inhalte (zum Beispiel Tabellen, Bilder oder Listen).",
      );
    } finally {
      setImporting(false);
      setDocxChapter(null);
    }
  };

  const renameNode = async (node: DocumentNode, title: string) => {
    try {
      const renamed = await renameDocument({
        projectId: workspace.project.id,
        documentId: node.id,
        expectedTitle: node.title,
        title,
      });
      setWorkspace((current) => ({
        ...current,
        documents: current.documents.map((item) => (item.id === renamed.id ? renamed : item)),
      }));
      setStored((current) =>
        current?.node.id === renamed.id ? { ...current, node: renamed } : current,
      );
      setError(null);
      return true;
    } catch {
      setError("Der Titel konnte nicht geändert werden. Der Dokumentbaum könnte neuer sein.");
      return false;
    }
  };

  const moveNode = async (node: DocumentNode, delta: -1 | 1) => {
    const siblings = workspace.documents
      .filter(({ parentId }) => parentId === node.parentId)
      .sort(compareDocumentNodes);
    const index = siblings.findIndex(({ id }) => id === node.id);
    const target = index + delta;
    if (index < 0 || target < 0 || target >= siblings.length) return;
    const expectedOrder = siblings.map(({ id }) => id);
    const orderedIds = [...expectedOrder];
    const currentId = orderedIds[index];
    const targetId = orderedIds[target];
    if (!currentId || !targetId) return;
    orderedIds[index] = targetId;
    orderedIds[target] = currentId;
    try {
      await reorderDocuments({
        projectId: workspace.project.id,
        parentId: node.parentId,
        expectedOrder,
        orderedIds,
      });
      const positions = new Map(orderedIds.map((id, position) => [id, position]));
      setWorkspace((current) => ({
        ...current,
        documents: current.documents.map((item) => {
          const position = positions.get(item.id);
          return position === undefined ? item : { ...item, position };
        }),
      }));
      setError(null);
    } catch {
      setError("Die Reihenfolge konnte nicht geändert werden. Der Dokumentbaum könnte neuer sein.");
    }
  };

  const prepareDelete = (ids: readonly EntityId[]) => {
    if (activeId && ids.includes(activeId)) {
      setActiveId(null);
      setStored(null);
    }
  };

  const deleteNode = async (node: DocumentNode, childIds: readonly EntityId[]) => {
    try {
      const deletedIds = await deleteDocument({
        projectId: workspace.project.id,
        documentId: node.id,
        expectedTitle: node.title,
        expectedRevision: node.revision,
        expectedChildIds: childIds,
      });
      const deleted = new Set(deletedIds);
      setWorkspace((current) => ({
        ...current,
        documents: current.documents.filter(({ id }) => !deleted.has(id)),
      }));
      if (activeId && deleted.has(activeId)) {
        setActiveId(null);
        setStored(null);
      }
      setError(null);
      return true;
    } catch {
      setError(
        "Das Dokument konnte nicht gelöscht werden. Inhalt oder Kindliste könnten neuer sein.",
      );
      return false;
    }
  };

  const updateDocumentRevision = useCallback((id: EntityId, revision: number) => {
    setWorkspace((current) => ({
      ...current,
      documents: current.documents.map((item) => (item.id === id ? { ...item, revision } : item)),
    }));
    setStored((current) =>
      current?.node.id === id ? { ...current, node: { ...current.node, revision } } : current,
    );
  }, []);

  return (
    <main className="workspace-shell">
      <header className="workspace-header">
        <div className="workspace-title-row">
          <div className="workspace-identity">
            <span className="eyebrow">Lokales Projekt</span>
            <h1>{workspace.project.name}</h1>
          </div>
          <button type="button" className="secondary project-overview-button" onClick={onClose}>
            Projektübersicht
          </button>
        </div>
        <nav aria-label="Projektbereiche" className="workspace-navigation">
          {navigationGroups.map((group) => (
            <div
              key={group.label}
              className="workspace-nav-group"
              role="group"
              aria-label={group.label}
            >
              <span className="workspace-nav-label" aria-hidden="true">
                {group.label}
              </span>
              <div className="workspace-tabs">
                {group.items.map((item) => (
                  <button
                    key={item.view}
                    type="button"
                    className={view === item.view ? "active" : "secondary"}
                    aria-current={view === item.view ? "page" : undefined}
                    onClick={() => setView(item.view)}
                  >
                    {item.label}
                  </button>
                ))}
              </div>
            </div>
          ))}
        </nav>
      </header>

      {view === "manuscript" ? (
        <div className="workspace-grid">
          <aside className="document-sidebar" aria-label="Kapitel und Szenen">
            <div className="sidebar-heading">
              <h2>Manuskript</h2>
              <button type="button" className="compact" onClick={() => void addChapter()}>
                + Kapitel
              </button>
            </div>
            <div role="tree" aria-label="Dokumentbaum" className="document-tree">
              {tree.map((chapter, index) => (
                <ChapterTree
                  key={chapter.id}
                  chapter={chapter}
                  activeId={activeId}
                  onSelect={setActiveId}
                  onAddScene={addScene}
                  onImportScene={importScene}
                  onImportDocx={setDocxChapter}
                  importing={importing || docxChapter !== null}
                  canMoveUp={index > 0}
                  canMoveDown={index < tree.length - 1}
                  onMove={moveNode}
                  onRename={renameNode}
                  onPrepareDelete={prepareDelete}
                  onDelete={deleteNode}
                />
              ))}
            </div>
          </aside>

          <section className="editor-panel">
            {docxChapter ? (
              <section aria-labelledby="docx-import-heading" className="docx-import-notice">
                <h2 id="docx-import-heading" ref={docxHeading} tabIndex={-1}>
                  DOCX als neue Szene
                </h2>
                <p>
                  Übernommen werden Absätze, Überschriften, Fett/Kursiv und Zeilenumbrüche.
                  Schriftarten und Seitenlayout werden nicht übernommen. Tabellen, Bilder, Fußnoten,
                  Links, Felder, Änderungen und Listen werden abgelehnt. Maximal 8 MiB.
                </p>
                <button type="button" disabled={importing} onClick={() => void importDocx()}>
                  DOCX-Datei auswählen
                </button>
                <button
                  type="button"
                  className="secondary"
                  disabled={importing}
                  onClick={() => setDocxChapter(null)}
                >
                  DOCX-Import abbrechen
                </button>
              </section>
            ) : null}
            {error ? <p className="error-banner">{error}</p> : null}
            {loading ? <p className="empty-editor">Szene wird geladen …</p> : null}
            {!loading && stored ? (
              <RichTextEditor
                key={stored.node.id}
                projectId={workspace.project.id}
                stored={stored}
                onRevision={updateDocumentRevision}
              />
            ) : null}
            {!loading && !stored ? (
              <p className="empty-editor">Wähle eine Szene aus oder lege eine neue an.</p>
            ) : null}
          </section>
        </div>
      ) : view === "storyBible" ? (
        <StoryBible
          projectId={workspace.project.id}
          documents={workspace.documents}
          activeDocumentId={activeId}
          onSelectDocument={(id) => {
            setActiveId(id);
            setView("manuscript");
          }}
        />
      ) : view === "plotBoard" ? (
        <PlotBoard
          projectId={workspace.project.id}
          onSelectDocument={(id) => {
            setActiveId(id);
            setView("manuscript");
          }}
        />
      ) : view === "timelines" ? (
        <Timelines
          projectId={workspace.project.id}
          documents={workspace.documents}
          activeDocumentId={activeId}
          onSelectDocument={(id) => {
            setActiveId(id);
            setView("manuscript");
          }}
        />
      ) : view === "continuity" ? (
        <Continuity
          projectId={workspace.project.id}
          onSelectDocument={(id) => {
            setActiveId(id);
            setView("manuscript");
          }}
        />
      ) : view === "media" ? (
        <MediaLibrary
          projectId={workspace.project.id}
          documents={workspace.documents}
          activeDocumentId={activeId}
          onSelectDocument={(id) => {
            setActiveId(id);
            setView("manuscript");
          }}
        />
      ) : view === "visuals" ? (
        <VisualProfiles projectId={workspace.project.id} />
      ) : view === "prompts" ? (
        <PromptBuilder
          projectId={workspace.project.id}
          documents={workspace.documents}
          activeDocumentId={activeId}
        />
      ) : view === "versions" ? (
        <VersionControl projectId={workspace.project.id} />
      ) : view === "exports" ? (
        <Exports projectId={workspace.project.id} />
      ) : (
        <TextAssistant projectId={workspace.project.id} activeDocumentId={activeId} />
      )}
    </main>
  );
}

function ChapterTree({
  chapter,
  activeId,
  onSelect,
  onAddScene,
  onImportScene,
  onImportDocx,
  importing,
  canMoveUp,
  canMoveDown,
  onMove,
  onRename,
  onPrepareDelete,
  onDelete,
}: {
  readonly chapter: DocumentTreeNode;
  readonly activeId: EntityId | null;
  readonly onSelect: (id: EntityId) => void;
  readonly onAddScene: (chapterId: EntityId) => Promise<void>;
  readonly onImportScene: (chapterId: EntityId) => Promise<void>;
  readonly onImportDocx: (chapterId: EntityId) => void;
  readonly importing: boolean;
  readonly canMoveUp: boolean;
  readonly canMoveDown: boolean;
  readonly onMove: (node: DocumentNode, delta: -1 | 1) => Promise<void>;
  readonly onRename: (node: DocumentNode, title: string) => Promise<boolean>;
  readonly onPrepareDelete: (ids: readonly EntityId[]) => void;
  readonly onDelete: (node: DocumentNode, childIds: readonly EntityId[]) => Promise<boolean>;
}) {
  const [editingId, setEditingId] = useState<EntityId | null>(null);
  const [draftTitle, setDraftTitle] = useState("");
  const [deleteArmedId, setDeleteArmedId] = useState<EntityId | null>(null);

  const startRename = (node: DocumentNode) => {
    setDeleteArmedId(null);
    setEditingId(node.id);
    setDraftTitle(node.title);
  };

  const submitRename = async (node: DocumentNode) => {
    if (await onRename(node, draftTitle)) setEditingId(null);
  };

  const requestDelete = async (node: DocumentNode, childIds: readonly EntityId[]) => {
    if (deleteArmedId !== node.id) {
      setEditingId(null);
      setDeleteArmedId(node.id);
      onPrepareDelete([node.id, ...childIds]);
      return;
    }
    if (await onDelete(node, childIds)) setDeleteArmedId(null);
  };

  return (
    <div role="treeitem" aria-expanded="true" className="tree-chapter">
      <div className="chapter-row">
        <TreeNodeLabel
          node={chapter}
          editing={editingId === chapter.id}
          draftTitle={draftTitle}
          deleteArmed={deleteArmedId === chapter.id}
          canMoveUp={canMoveUp}
          canMoveDown={canMoveDown}
          onDraftTitle={setDraftTitle}
          onCancelRename={() => setEditingId(null)}
          onSubmitRename={() => void submitRename(chapter)}
          onStartRename={() => startRename(chapter)}
          onMove={(delta) => void onMove(chapter, delta)}
          onDelete={() =>
            void requestDelete(
              chapter,
              chapter.children.map(({ id }) => id),
            )
          }
        />
        <button
          type="button"
          className="icon-button"
          aria-label={`Neue Szene in ${chapter.title}`}
          onClick={() => void onAddScene(chapter.id)}
        >
          +
        </button>
        <button
          type="button"
          className="compact secondary"
          aria-label={`Import: Textdatei in ${chapter.title}`}
          disabled={importing}
          onClick={() => void onImportScene(chapter.id)}
        >
          Import
        </button>
        <button
          type="button"
          className="compact secondary"
          aria-label={`Import: DOCX in ${chapter.title}`}
          disabled={importing}
          onClick={() => onImportDocx(chapter.id)}
        >
          DOCX
        </button>
      </div>
      <div role="group">
        {chapter.children.map((scene, index) => (
          <div
            className="scene-row"
            key={scene.id}
            role="treeitem"
            aria-selected={activeId === scene.id}
          >
            <TreeNodeLabel
              node={scene}
              editing={editingId === scene.id}
              draftTitle={draftTitle}
              deleteArmed={deleteArmedId === scene.id}
              canMoveUp={index > 0}
              canMoveDown={index < chapter.children.length - 1}
              selected={activeId === scene.id}
              onSelect={() => onSelect(scene.id)}
              onDraftTitle={setDraftTitle}
              onCancelRename={() => setEditingId(null)}
              onSubmitRename={() => void submitRename(scene)}
              onStartRename={() => startRename(scene)}
              onMove={(delta) => void onMove(scene, delta)}
              onDelete={() => void requestDelete(scene, [])}
            />
          </div>
        ))}
      </div>
    </div>
  );
}

function TreeNodeLabel({
  node,
  editing,
  draftTitle,
  deleteArmed,
  canMoveUp,
  canMoveDown,
  selected = false,
  onSelect,
  onDraftTitle,
  onCancelRename,
  onSubmitRename,
  onStartRename,
  onMove,
  onDelete,
}: {
  readonly node: DocumentNode;
  readonly editing: boolean;
  readonly draftTitle: string;
  readonly deleteArmed: boolean;
  readonly canMoveUp: boolean;
  readonly canMoveDown: boolean;
  readonly selected?: boolean;
  readonly onSelect?: () => void;
  readonly onDraftTitle: (title: string) => void;
  readonly onCancelRename: () => void;
  readonly onSubmitRename: () => void;
  readonly onStartRename: () => void;
  readonly onMove: (delta: -1 | 1) => void;
  readonly onDelete: () => void;
}) {
  if (editing) {
    return (
      <form
        className="tree-node-edit"
        onSubmit={(event) => {
          event.preventDefault();
          onSubmitRename();
        }}
      >
        <label>
          <span className="sr-only">Neuer Titel für {node.title}</span>
          <input
            autoFocus
            maxLength={240}
            value={draftTitle}
            onChange={(event) => onDraftTitle(event.target.value)}
          />
        </label>
        <button type="submit" className="compact" disabled={!draftTitle.trim()}>
          Speichern
        </button>
        <button type="button" className="compact secondary" onClick={onCancelRename}>
          Abbrechen
        </button>
      </form>
    );
  }

  return (
    <div className="tree-node-label">
      {node.kind === "scene" ? (
        <button
          type="button"
          className={selected ? "scene-button active" : "scene-button"}
          onClick={onSelect}
        >
          {node.title}
        </button>
      ) : (
        <strong>{node.title}</strong>
      )}
      <div className="tree-node-actions" aria-label={`Aktionen für ${node.title}`}>
        <button
          type="button"
          className="icon-button"
          aria-label={`${node.title} nach oben`}
          disabled={!canMoveUp}
          onClick={() => onMove(-1)}
        >
          ↑
        </button>
        <button
          type="button"
          className="icon-button"
          aria-label={`${node.title} nach unten`}
          disabled={!canMoveDown}
          onClick={() => onMove(1)}
        >
          ↓
        </button>
        <button type="button" className="compact secondary" onClick={onStartRename}>
          Umbenennen
        </button>
        <button
          type="button"
          className={deleteArmed ? "compact danger" : "compact secondary"}
          onClick={onDelete}
        >
          {deleteArmed ? "Löschen bestätigen" : "Löschen"}
        </button>
      </div>
    </div>
  );
}

function RichTextEditor({
  projectId,
  stored,
  onRevision,
}: {
  readonly projectId: Workspace["project"]["id"];
  readonly stored: StoredDocument;
  readonly onRevision: (id: EntityId, revision: number) => void;
}) {
  const [status, setStatus] = useState<AutosaveStatus>("saved");
  const [wordCount, setWordCount] = useState(countWords(stored.plainText));
  const [focusMode, setFocusMode] = useState(false);
  const persist = useCallback(
    async (content: EditorDocument, revision: number) => {
      const nextRevision = await saveDocument(projectId, stored.node.id, revision, content);
      onRevision(stored.node.id, nextRevision);
      return nextRevision;
    },
    [onRevision, projectId, stored.node.id],
  );
  const autosave = useMemo(
    () => new AutosaveController(stored.node.revision, persist, 700),
    [persist, stored.node.revision],
  );
  const autosaveLifecycle = useRef(0);
  const editor = useEditor({
    extensions: [StarterKit],
    content: stored.content.document,
    immediatelyRender: false,
    editorProps: {
      attributes: {
        "aria-label": `Text von ${stored.node.title}`,
        role: "textbox",
      },
    },
    onUpdate: ({ editor: current }) => {
      const content: EditorDocument = {
        version: EDITOR_DOCUMENT_VERSION,
        document: current.getJSON(),
      };
      setWordCount(countWords(current.getText({ blockSeparator: " " })));
      autosave.schedule(content);
    },
  });
  const formatting = useEditorState({
    editor,
    selector: ({ editor: current }) => ({
      bold: current?.isActive("bold") ?? false,
      italic: current?.isActive("italic") ?? false,
    }),
  });

  useEffect(() => {
    const lifecycle = ++autosaveLifecycle.current;
    const unsubscribe = autosave.subscribe(({ status: next }) => setStatus(next));
    return () => {
      unsubscribe();
      void autosave.flush();
      // StrictMode richtet Effekte im Devmodus sofort erneut ein; erst ein echter Unmount entsorgt.
      queueMicrotask(() => {
        if (autosaveLifecycle.current === lifecycle) autosave.dispose();
      });
    };
  }, [autosave]);

  return (
    <div className={focusMode ? "editor-document focus-mode" : "editor-document"}>
      <div className="editor-title-row">
        <div>
          <span className="eyebrow">Szene</span>
          <h2>{stored.node.title}</h2>
        </div>
        <div className="editor-meta">
          <span>{wordCount} Wörter</span>
          <span className={`save-status ${status}`} role="status" aria-live="polite">
            {statusLabel(status)}
          </span>
        </div>
      </div>
      <div className="editor-toolbar" role="toolbar" aria-label="Textformatierung">
        <button
          type="button"
          aria-label="Fett"
          aria-pressed={formatting?.bold ?? false}
          onClick={() => editor?.chain().focus().toggleBold().run()}
        >
          Fett
        </button>
        <button
          type="button"
          aria-label="Kursiv"
          aria-pressed={formatting?.italic ?? false}
          onClick={() => editor?.chain().focus().toggleItalic().run()}
        >
          Kursiv
        </button>
        <button type="button" onClick={() => editor?.chain().focus().undo().run()}>
          Rückgängig
        </button>
        <button type="button" onClick={() => editor?.chain().focus().redo().run()}>
          Wiederholen
        </button>
        <button
          type="button"
          aria-pressed={focusMode}
          onClick={() => setFocusMode((active) => !active)}
        >
          Fokusmodus
        </button>
      </div>
      <EditorContent editor={editor} className="tiptap-surface" />
    </div>
  );
}

function firstSceneId(documents: readonly DocumentNode[]): EntityId | null {
  return documents.find(({ kind }) => kind === "scene")?.id ?? null;
}

function chapterCount(documents: readonly DocumentNode[]): number {
  return documents.filter(({ kind }) => kind === "chapter").length;
}

function compareDocumentNodes(left: DocumentNode, right: DocumentNode): number {
  return left.position - right.position || left.id.localeCompare(right.id);
}

function statusLabel(status: AutosaveStatus): string {
  switch (status) {
    case "saved":
      return "Gespeichert";
    case "dirty":
      return "Ungespeichert";
    case "saving":
      return "Speichert …";
    case "error":
      return "Speichern fehlgeschlagen";
  }
}
