import { EditorContent, useEditor } from "@tiptap/react";
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
import { useCallback, useEffect, useMemo, useState } from "react";
import {
  createDocument,
  loadDocument,
  saveDocument,
  type StoredDocument,
  type Workspace,
} from "./commands";

interface EditorWorkspaceProps {
  readonly initialWorkspace: Workspace;
  readonly onClose: () => void;
}

export function EditorWorkspace({ initialWorkspace, onClose }: EditorWorkspaceProps) {
  const [workspace, setWorkspace] = useState(initialWorkspace);
  const [activeId, setActiveId] = useState<EntityId | null>(
    firstSceneId(initialWorkspace.documents),
  );
  const [stored, setStored] = useState<StoredDocument | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
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

  return (
    <main className="workspace-shell">
      <header className="workspace-header">
        <div>
          <span className="eyebrow">Lokales Projekt</span>
          <h1>{workspace.project.name}</h1>
        </div>
        <button type="button" className="secondary" onClick={onClose}>
          Projektübersicht
        </button>
      </header>

      <div className="workspace-grid">
        <aside className="document-sidebar" aria-label="Kapitel und Szenen">
          <div className="sidebar-heading">
            <h2>Manuskript</h2>
            <button type="button" className="compact" onClick={() => void addChapter()}>
              + Kapitel
            </button>
          </div>
          <div role="tree" aria-label="Dokumentbaum" className="document-tree">
            {tree.map((chapter) => (
              <ChapterTree
                key={chapter.id}
                chapter={chapter}
                activeId={activeId}
                onSelect={setActiveId}
                onAddScene={addScene}
              />
            ))}
          </div>
        </aside>

        <section className="editor-panel" aria-live="polite">
          {error ? <p className="error-banner">{error}</p> : null}
          {loading ? <p className="empty-editor">Szene wird geladen …</p> : null}
          {!loading && stored ? (
            <RichTextEditor key={stored.node.id} projectId={workspace.project.id} stored={stored} />
          ) : null}
          {!loading && !stored ? (
            <p className="empty-editor">Wähle eine Szene aus oder lege eine neue an.</p>
          ) : null}
        </section>
      </div>
    </main>
  );
}

function ChapterTree({
  chapter,
  activeId,
  onSelect,
  onAddScene,
}: {
  readonly chapter: DocumentTreeNode;
  readonly activeId: EntityId | null;
  readonly onSelect: (id: EntityId) => void;
  readonly onAddScene: (chapterId: EntityId) => Promise<void>;
}) {
  return (
    <div role="treeitem" aria-expanded="true" className="tree-chapter">
      <div className="chapter-row">
        <strong>{chapter.title}</strong>
        <button
          type="button"
          className="icon-button"
          aria-label={`Neue Szene in ${chapter.title}`}
          onClick={() => void onAddScene(chapter.id)}
        >
          +
        </button>
      </div>
      <div role="group">
        {chapter.children.map((scene) => (
          <button
            type="button"
            role="treeitem"
            aria-selected={activeId === scene.id}
            className={activeId === scene.id ? "scene-button active" : "scene-button"}
            key={scene.id}
            onClick={() => onSelect(scene.id)}
          >
            {scene.title}
          </button>
        ))}
      </div>
    </div>
  );
}

function RichTextEditor({
  projectId,
  stored,
}: {
  readonly projectId: Workspace["project"]["id"];
  readonly stored: StoredDocument;
}) {
  const [status, setStatus] = useState<AutosaveStatus>("saved");
  const [wordCount, setWordCount] = useState(countWords(stored.plainText));
  const [focusMode, setFocusMode] = useState(false);
  const persist = useCallback(
    (content: EditorDocument, revision: number) =>
      saveDocument(projectId, stored.node.id, revision, content),
    [projectId, stored.node.id],
  );
  const autosave = useMemo(
    () => new AutosaveController(stored.node.revision, persist, 700),
    [persist, stored.node.revision],
  );
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

  useEffect(() => autosave.subscribe(({ status: next }) => setStatus(next)), [autosave]);
  useEffect(
    () => () => {
      void autosave.flush();
      autosave.dispose();
    },
    [autosave],
  );

  return (
    <div className={focusMode ? "editor-document focus-mode" : "editor-document"}>
      <div className="editor-title-row">
        <div>
          <span className="eyebrow">Szene</span>
          <h2>{stored.node.title}</h2>
        </div>
        <div className="editor-meta">
          <span>{wordCount} Wörter</span>
          <span className={`save-status ${status}`}>{statusLabel(status)}</span>
        </div>
      </div>
      <div className="editor-toolbar" role="toolbar" aria-label="Textformatierung">
        <button
          type="button"
          aria-label="Fett"
          aria-pressed={editor?.isActive("bold") ?? false}
          onClick={() => editor?.chain().focus().toggleBold().run()}
        >
          Fett
        </button>
        <button
          type="button"
          aria-label="Kursiv"
          aria-pressed={editor?.isActive("italic") ?? false}
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
