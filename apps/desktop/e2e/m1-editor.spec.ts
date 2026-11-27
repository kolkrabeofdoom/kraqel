import { expect, test } from "@playwright/test";

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => localStorage.clear());
});

test("Drehbuchrollen bleiben nach Autosave und Wiederöffnen über die Tastatur bearbeitbar", async ({
  page,
}) => {
  await page.goto("/");
  await page.getByLabel("Projektname").fill("Drehbuchprobe");
  await page.getByRole("button", { name: "Projekt anlegen" }).click();
  const editor = page.getByRole("textbox", { name: "Text von Szene 1" });
  await editor.fill("Am frühen Abend");
  await editor.press("Control+End");
  await editor.press("Shift+Enter");
  await editor.press("Control+b");
  await editor.press("Control+i");
  await page.keyboard.insertText("Am Hafen");
  const firstBlock = editor.locator("p").first();
  await expect(firstBlock.locator("strong em, em strong")).toHaveText("Am Hafen");
  await expect(firstBlock.locator("br")).toHaveCount(1);
  const opening = await firstBlock.innerHTML();
  const toScript = page.getByRole("button", { name: "In Drehbuch umwandeln …" });
  const firstRole = page.getByLabel("Rolle für Absatz 1");
  await toScript.press("Enter");
  await expect(page.getByRole("heading", { name: "Zuordnungsvorschau" })).toBeFocused();
  await page.keyboard.press("Tab");
  await expect(firstRole).toBeFocused();
  await firstRole.selectOption("dialogue");
  await page
    .getByRole("region", { name: "Zuordnungsvorschau" })
    .getByRole("button", { name: "Abbrechen" })
    .press("Enter");
  await expect(toScript).toBeFocused();
  await expect(editor.locator("[data-kraqel-role]")).toHaveCount(0);
  await toScript.click();
  await expect(firstRole).toHaveValue("sceneHeading");
  await page.getByRole("button", { name: "Umwandlung bestätigen" }).click();
  await expect(firstBlock).toHaveAttribute("data-kraqel-role", "sceneHeading");
  expect(await firstBlock.innerHTML()).toBe(opening);
  await editor.press("Control+End");
  const roles = [
    ["action", "Es klopft."],
    ["character", "Sam"],
    ["parenthetical", "(leise)"],
    ["dialogue", "Wer ist da?"],
    ["transition", "Schwarzblende"],
  ] as const;
  for (const [role, text] of roles) {
    await page.keyboard.press("Enter");
    await page.getByRole("combobox", { name: "Absatzrolle", exact: true }).selectOption(role);
    await page.getByRole("button", { name: "Rolle anwenden" }).press("Enter");
    await expect(editor).toBeFocused();
    await page.keyboard.insertText(text);
  }
  await page.getByRole("button", { name: "Projektübersicht" }).click();
  await page.getByRole("button", { name: /Drehbuchprobe.*Öffnen/u }).click();
  for (const [role, text] of roles)
    await expect(editor.locator(`[data-kraqel-role="${role}"]`)).toHaveText(text);
  await editor.press("Control+a");
  const multi = page.getByRole("combobox", { name: "Absatzrolle", exact: true });
  await expect(multi).toHaveValue("");
  await multi.selectOption("action");
  await page.getByRole("button", { name: "Rolle anwenden" }).press("Enter");
  await expect(editor.locator("[data-kraqel-role=action]")).toHaveCount(6);
  await page.getByRole("button", { name: "Rückgängig" }).click();
  await expect(editor.locator("[data-kraqel-role=dialogue]")).toHaveText("Wer ist da?");
  await page.getByRole("button", { name: "Wiederholen" }).click();
  await expect(editor.locator("[data-kraqel-role=action]")).toHaveCount(6);
  expect(await firstBlock.innerHTML()).toBe(opening);
  await page.getByRole("button", { name: "Rückgängig" }).click();
  await page.getByRole("button", { name: "In Prosa umwandeln …" }).click();
  await page.getByRole("button", { name: "Umwandlung bestätigen" }).click();
  await expect(editor.locator("[data-kraqel-role]")).toHaveCount(0);
  expect(await firstBlock.innerHTML()).toBe(opening);
  await page.getByRole("button", { name: "Rückgängig" }).click();
  await expect(editor.locator("[data-kraqel-role]")).toHaveCount(6);
  await page.getByRole("button", { name: "Export", exact: true }).click();
  const print = page.getByRole("button", { name: "Drehbuch-PDF erzeugen" });
  await expect(print).toBeDisabled();
  await page.getByRole("group", { name: "Drehbuchszenen auswählen" }).getByRole("checkbox").check();
  await print.click();
  await expect(page.getByRole("status")).toContainText("drehbuch.pdf");

  // Both editor sessions open the same stored revision before either begins its edit.
  await page.getByRole("button", { name: "Manuskript", exact: true }).click();
  await expect(editor.locator("[data-kraqel-role=transition]")).toHaveText("Schwarzblende");
  const other = await page.context().newPage();
  await other.goto("/");
  const reopen = other.getByRole("button", { name: /Drehbuchprobe.*Öffnen/u });
  await reopen.click();
  const otherEditor = other.getByRole("textbox", { name: "Text von Szene 1" });
  const storedText = "Schwarzblende am Hafen.";
  await otherEditor.press("Control+End");
  const otherRole = other.getByRole("combobox", { name: "Absatzrolle", exact: true });
  await expect(otherRole).toHaveValue("transition");
  await otherRole.selectOption("action");
  await other.getByRole("button", { name: "Rolle anwenden" }).click();
  await otherEditor.press("Control+End");
  await other.keyboard.insertText(" am Hafen.");
  await expect(other.getByText("Gespeichert", { exact: true })).toBeVisible();

  await editor.press("Control+End");
  await page.keyboard.insertText(" im Archiv.");
  await expect(page.getByText("Speichern fehlgeschlagen", { exact: true })).toBeVisible();
  await expect(editor.locator("[data-kraqel-role=transition]")).toHaveText(
    "Schwarzblende im Archiv.",
  );
  await other.getByRole("button", { name: "Projektübersicht" }).click();
  await reopen.click();
  await expect(otherEditor.locator("[data-kraqel-role=action]").last()).toHaveText(storedText);
  await expect(otherEditor.locator("[data-kraqel-role=transition]")).toHaveCount(0);
});

test("Projekt, Autosave und Dokumentbaum bleiben über Wiederöffnen konsistent", async ({
  page,
}) => {
  const projectName = "Der höfliche Weltuntergang";
  const manuscriptText = "Der Kolkrabe prüft die Tür. Die Tür prüft höflich zurück.";

  await page.goto("/");
  await page.getByLabel("Projektname").fill(projectName);
  await page.getByRole("button", { name: "Projekt anlegen" }).click();

  await expect(page.getByRole("heading", { level: 1, name: projectName })).toBeVisible();
  const editor = page.getByRole("textbox", { name: "Text von Szene 1" });
  await editor.fill(manuscriptText);
  await expect(page.getByText("10 Wörter", { exact: true })).toBeVisible();
  await expect
    .poll(() =>
      page.evaluate(() =>
        localStorage.getItem("kraqel-e2e-command-state-v1")?.includes("Der Kolkrabe"),
      ),
    )
    .toBe(true);
  await expect(page.getByText("Gespeichert", { exact: true })).toBeVisible({ timeout: 5_000 });

  await page.getByRole("button", { name: "Projektübersicht" }).click();
  await page.getByRole("button", { name: new RegExp(`${projectName}.*Öffnen`) }).click();
  await expect(page.getByRole("textbox", { name: "Text von Szene 1" })).toContainText(
    manuscriptText,
  );

  await page.getByRole("button", { name: "+ Kapitel" }).click();
  const secondChapter = page.locator(".tree-chapter").filter({ hasText: "Kapitel 2" });
  await secondChapter.getByRole("button", { name: "Neue Szene in Kapitel 2" }).click();

  await secondChapter.getByRole("button", { name: "Umbenennen" }).last().click();
  await secondChapter.getByLabel("Neuer Titel für Szene 1").fill("Zwischenspiel");
  await secondChapter.getByRole("button", { name: "Speichern" }).click();
  await expect(
    secondChapter.getByRole("button", { name: "Zwischenspiel", exact: true }),
  ).toBeVisible();

  await secondChapter.getByRole("button", { name: "Neue Szene in Kapitel 2" }).click();
  await secondChapter.getByRole("button", { name: "Zwischenspiel nach unten" }).click();
  await expect(secondChapter.locator(".scene-button")).toHaveText(["Szene 2", "Zwischenspiel"]);

  const sceneToDelete = secondChapter.locator(".scene-row").filter({ hasText: "Szene 2" });
  await sceneToDelete.getByRole("button", { name: "Löschen", exact: true }).click();
  await expect(
    sceneToDelete.getByRole("button", { name: "Löschen bestätigen", exact: true }),
  ).toBeVisible();
  await sceneToDelete.getByRole("button", { name: "Löschen bestätigen", exact: true }).click();
  await expect(secondChapter.getByRole("button", { name: "Szene 2", exact: true })).toHaveCount(0);
  await expect(
    secondChapter.getByRole("button", { name: "Zwischenspiel", exact: true }),
  ).toBeVisible();
});

test("Textimport ergänzt eine editierbare Szene und behandelt Abbruch neutral", async ({
  page,
}) => {
  await page.goto("/");
  await page.getByLabel("Projektname").fill("Import und die Kunst des Ankommens");
  await page.getByRole("button", { name: "Projekt anlegen" }).click();
  const chapter = page.locator(".tree-chapter").filter({ hasText: "Kapitel 1" });

  await chapter.getByRole("button", { name: "Import: Textdatei in Kapitel 1" }).click();
  await expect(page.getByRole("textbox", { name: "Text von importierte-szene" })).toContainText(
    "Importierter Text",
  );
  await chapter.getByRole("button", { name: "Umbenennen" }).last().click();
  await chapter.getByLabel("Neuer Titel für importierte-szene").fill("Ankunft");
  await chapter.getByRole("button", { name: "Speichern" }).click();
  await expect(chapter.getByRole("button", { name: "Ankunft", exact: true })).toBeVisible();

  await page.evaluate(() => localStorage.setItem("kraqel-e2e-cancel-text-import", "1"));
  const scenesBeforeCancel = await chapter.locator(".scene-row").count();
  await chapter.getByRole("button", { name: "Import: Textdatei in Kapitel 1" }).click();
  await expect(chapter.locator(".scene-row")).toHaveCount(scenesBeforeCancel);
  await expect(page.getByText(/konnte nicht importiert werden/i)).toHaveCount(0);
});
