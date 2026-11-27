import { getSchema } from "@tiptap/react";
import StarterKit from "@tiptap/starter-kit";
import { expect, it } from "vitest";
import { ScreenplayBlock, scriptHint } from "./screenplay";

it("checks dialogue text using the same Unicode whitespace as the print grammar", () => {
  const schema = getSchema([StarterKit, ScreenplayBlock]);
  function hint(dialogue: string) {
    const blocks = [
      ["sceneHeading", "INT. ARCHIV"],
      ["character", "Mira"],
      ["dialogue", dialogue],
    ] as const;
    return scriptHint(
      schema.node(
        "doc",
        null,
        blocks.map(([role, text]) => schema.node("screenplayBlock", { role }, schema.text(text))),
      ),
    );
  }
  expect(hint("\u0085")).not.toBeNull();
  expect(hint("\u0085Ja.")).toBeNull();
});
