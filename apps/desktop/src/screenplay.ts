import { Node, type Editor } from "@tiptap/react";
import { splitBlockAs } from "@tiptap/pm/commands";
import { closeHistory, isHistoryTransaction } from "@tiptap/pm/history";
import { Fragment, Slice, type Node as DocumentNode } from "@tiptap/pm/model";
import { Plugin } from "@tiptap/pm/state";
import { SCREENPLAY_ROLES, type ScreenplayRole } from "@kraqel/editor";

export const roleNames: Record<ScreenplayRole, string> = {
  sceneHeading: "Szenenkopf",
  action: "Handlung",
  character: "Figur",
  parenthetical: "Klammeranweisung",
  dialogue: "Dialog",
  transition: "Übergang",
};

const conversion = "kraqelScreenplayConversion";
const isScript = (node: DocumentNode | null) => node?.type.name === "screenplayBlock";
export const isScreenplay = (doc: DocumentNode) => isScript(doc.firstChild);

export const ScreenplayBlock = Node.create({
  name: "screenplayBlock",
  group: "block",
  // The prose paragraph must remain the schema's default block.
  content: "(text | hardBreak)*",
  marks: "bold italic",
  addAttributes() {
    return { role: { default: "action", rendered: false, keepOnSplit: true } };
  },
  parseHTML() {
    return [
      {
        tag: "p[data-kraqel-role]",
        getAttrs: (element) => {
          const role = element.getAttribute("data-kraqel-role") as ScreenplayRole;
          return SCREENPLAY_ROLES.includes(role) ? { role } : false;
        },
      },
    ];
  },
  renderHTML({ node }) {
    return [
      "p",
      {
        "data-kraqel-role": node.attrs.role,
        "aria-label": roleNames[node.attrs.role as ScreenplayRole],
      },
      0,
    ];
  },
  addKeyboardShortcuts() {
    return {
      Enter: () => {
        const { state, view } = this.editor;
        const parent = state.selection.$from.parent;
        return (
          isScript(parent) &&
          splitBlockAs(() => ({ type: parent.type, attrs: parent.attrs }))(state, view.dispatch)
        );
      },
    };
  },
  addProseMirrorPlugins() {
    return [
      new Plugin({
        filterTransaction(tr, state) {
          if (!tr.docChanged) return true;
          const script = isScreenplay(tr.doc);
          for (let index = 0; index < tr.doc.childCount; index++) {
            if (isScript(tr.doc.child(index)) !== script) return false;
          }
          return (
            script === isScreenplay(state.doc) ||
            tr.getMeta(conversion) === true ||
            isHistoryTransaction(tr)
          );
        },
        props: {
          transformPasted(slice, view) {
            const block = view.state.selection.$from.parent;
            if (!isScript(block)) return slice;
            const blocks: DocumentNode[] = [];
            slice.content.forEach((node) => {
              if (node.type.name === "paragraph" && block.type.validContent(node.content)) {
                blocks.push(block.type.create(block.attrs, node.content));
              } else blocks.push(node);
            });
            return new Slice(Fragment.fromArray(blocks), slice.openStart, slice.openEnd);
          },
        },
      }),
    ];
  },
});

export function canConvert(editor: Editor): boolean {
  const { doc, schema } = editor.state;
  for (let i = 0; i < doc.childCount; i++) {
    const node = doc.child(i);
    if (
      !isScript(node) &&
      (node.type.name !== "paragraph" || !schema.nodes.screenplayBlock!.validContent(node.content))
    )
      return false;
  }
  return true;
}

export function convertScene(
  editor: Editor,
  reviewed: DocumentNode,
  roles: readonly ScreenplayRole[],
): boolean {
  const { doc, schema, tr } = editor.state;
  if (doc !== reviewed || !canConvert(editor) || roles.length !== doc.childCount) return false;
  const script = isScreenplay(doc);
  doc.forEach((_node, pos, index) =>
    tr.setNodeMarkup(
      pos,
      script ? schema.nodes.paragraph : schema.nodes.screenplayBlock,
      script ? null : { role: roles[index] },
    ),
  );
  editor.view.dispatch(closeHistory(tr.setMeta(conversion, true)));
  editor.view.dispatch(closeHistory(editor.state.tr));
  editor.commands.focus();
  return true;
}

export function selectedRole(editor: Editor): ScreenplayRole | "mixed" | null {
  const { doc, selection } = editor.state;
  if (!isScreenplay(doc)) return null;
  if (selection.empty) return selection.$from.parent.attrs.role as ScreenplayRole;
  const roles = new Set<ScreenplayRole>();
  doc.nodesBetween(selection.from, selection.to, (node) => {
    if (isScript(node)) roles.add(node.attrs.role as ScreenplayRole);
    return false;
  });
  return roles.size === 1 ? [...roles][0]! : "mixed";
}

export function scriptHint(doc: DocumentNode): string | null {
  if (!isScreenplay(doc)) return null;
  const blocks: { role: ScreenplayRole; filled: boolean }[] = [];
  doc.forEach((node) =>
    blocks.push({
      role: node.attrs.role as ScreenplayRole,
      filled: node.textContent.trim().length > 0,
    }),
  );
  const first = blocks.find((block) => block.filled);
  if (!first && blocks.every((block) => block.role === "action" || block.role === "sceneHeading"))
    return null;
  if (
    first?.role !== "sceneHeading" ||
    blocks.filter((b) => b.filled && b.role === "sceneHeading").length !== 1
  )
    return "Für den PDF-Druck braucht die Szene genau einen Szenenkopf als ersten nichtleeren Absatz.";
  let speaker = false;
  let needsDialogue = false;
  let afterInstruction = false;
  for (const { role, filled } of blocks) {
    if (role === "dialogue" && speaker && filled) {
      needsDialogue = false;
      afterInstruction = false;
      continue;
    }
    if (role === "parenthetical" && speaker && filled && !afterInstruction) {
      needsDialogue = true;
      afterInstruction = true;
      continue;
    }
    if (needsDialogue || role === "dialogue" || role === "parenthetical")
      return "Prüfe die Dialogfolge für den PDF-Druck: Figur, optional Klammeranweisung, dann Dialog.";
    if (!filled && role !== "action")
      return "Für den PDF-Druck benötigen die gewählten Rollen Text. Leere Handlungsabsätze sind möglich.";
    speaker = role === "character";
    needsDialogue = speaker;
    afterInstruction = false;
  }
  return needsDialogue
    ? "Ergänze den Dialog zur Figur. Der Entwurf kann bereits gespeichert werden."
    : null;
}
