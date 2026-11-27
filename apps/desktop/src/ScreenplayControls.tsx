import { useId, useRef, useState } from "react";
import { flushSync } from "react-dom";
import { useEditorState, type Editor } from "@tiptap/react";
import type { Node } from "@tiptap/pm/model";
import { SCREENPLAY_ROLES, type ScreenplayRole } from "@kraqel/editor";
import {
  canConvert,
  convertScene,
  hasScreenplayText,
  isScreenplay,
  roleNames,
  scriptHint,
  selectedRole,
} from "./screenplay";

const options = SCREENPLAY_ROLES.map((role) => (
  <option key={role} value={role}>
    {roleNames[role]}
  </option>
));

function focusReviewHeading(heading: HTMLHeadingElement | null) {
  heading?.focus();
}

export function ScreenplayControls({ editor }: { editor: Editor }) {
  const view = useEditorState({
    editor,
    selector: () => ({
      script: isScreenplay(editor.state.doc),
      role: selectedRole(editor),
      hint: scriptHint(editor.state.doc),
      convertible: canConvert(editor),
      from: editor.state.selection.from,
      to: editor.state.selection.to,
    }),
  });
  const [review, setReview] = useState<{ doc: Node; roles: ScreenplayRole[] } | null>(null);
  const [error, setError] = useState("");
  const conversionButton = useRef<HTMLButtonElement>(null);
  const reviewId = useId();
  const dismissReview = () => {
    flushSync(() => setReview(null));
    if (conversionButton.current && !conversionButton.current.disabled) {
      conversionButton.current.focus();
    } else {
      editor.view.focus();
    }
  };
  const prepare = () => {
    const doc = editor.state.doc;
    const roles: ScreenplayRole[] = [];
    doc.forEach((node) =>
      roles.push(isScreenplay(doc) ? (node.attrs.role as ScreenplayRole) : "action"),
    );
    if (!isScreenplay(doc)) {
      let first = 0;
      while (first < doc.childCount && !hasScreenplayText(doc.child(first))) first++;
      roles[first < doc.childCount ? first : 0] = "sceneHeading";
    }
    setError("");
    setReview({ doc, roles });
  };
  return (
    <div className="screenplay-controls">
      {view.script && (
        <RoleSelection
          key={`${view.from}-${view.to}-${view.role}`}
          editor={editor}
          role={view.role}
        />
      )}
      <button
        type="button"
        className="secondary"
        disabled={!view.convertible || review !== null}
        ref={conversionButton}
        aria-expanded={review !== null}
        aria-controls={review ? reviewId : undefined}
        onClick={prepare}
      >
        {view.script ? "In Prosa umwandeln …" : "In Drehbuch umwandeln …"}
      </button>
      {!view.convertible && (
        <p>
          Für die Umwandlung sind einfache Absätze mit Text, Fett, Kursiv und Zeilenumbrüchen
          erforderlich.
        </p>
      )}
      {review && (
        <section id={reviewId} aria-label="Zuordnungsvorschau">
          <h3 ref={focusReviewHeading} tabIndex={-1}>
            Zuordnungsvorschau
          </h3>
          {view.script && (
            <p>
              Beim Umwandeln in Prosa werden die Drehbuchrollen entfernt. Der Text bleibt erhalten.
            </p>
          )}
          <ol>
            {review.roles.map((role, i) => (
              <li key={i}>
                <span>{review.doc.child(i).textContent.slice(0, 80) || "(Leerer Absatz)"}</span>
                {isScreenplay(review.doc) ? (
                  <span>{roleNames[role]} → Absatz</span>
                ) : (
                  <label>
                    Rolle für Absatz {i + 1}
                    <select
                      value={role}
                      onChange={(event) =>
                        setReview({
                          ...review,
                          roles: review.roles.map((r, j) =>
                            i === j ? (event.target.value as ScreenplayRole) : r,
                          ),
                        })
                      }
                    >
                      {options}
                    </select>
                  </label>
                )}
              </li>
            ))}
          </ol>
          <button
            type="button"
            onClick={() => {
              if (!convertScene(editor, review.doc, review.roles)) {
                setError("Die Szene hat sich geändert. Öffne die Zuordnungsvorschau erneut.");
                dismissReview();
              } else {
                setReview(null);
              }
            }}
          >
            Umwandlung bestätigen
          </button>
          <button type="button" className="secondary" onClick={dismissReview}>
            Abbrechen
          </button>
        </section>
      )}
      {view.hint && <p role="status">{view.hint}</p>}
      {error && <p role="alert">{error}</p>}
    </div>
  );
}

function RoleSelection({
  editor,
  role,
}: {
  editor: Editor;
  role: ReturnType<typeof selectedRole>;
}) {
  const [choice, setChoice] = useState<ScreenplayRole | "">(role === "mixed" || !role ? "" : role);
  return (
    <div className="screenplay-roles">
      <label>
        Absatzrolle
        <select
          value={choice}
          onChange={(event) => setChoice(event.target.value as ScreenplayRole)}
        >
          {role === "mixed" && <option value="">Verschiedene Rollen</option>}
          {options}
        </select>
      </label>
      <button
        type="button"
        disabled={!choice || choice === role}
        onClick={() => editor.chain().focus().setNode("screenplayBlock", { role: choice }).run()}
      >
        Rolle anwenden
      </button>
    </div>
  );
}
