import { expect, test } from "@playwright/test";

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => localStorage.clear());
});

test("Bildprompt-Builder speichert und exportiert einen lokalen, editierbaren Entwurf", async ({
  page,
}) => {
  await page.goto("/");
  await page.getByLabel("Projektname").fill("Das Atelier jenseits der Wahrscheinlichkeit");
  await page.getByRole("button", { name: "Projekt anlegen" }).click();

  const sceneText = "Corax steht im Archiv und betrachtet eine sehr verdächtige Teekanne.";
  await page.getByRole("textbox", { name: "Text von Szene 1" }).fill(sceneText);
  await expect(page.getByText("Gespeichert", { exact: true })).toBeVisible({ timeout: 5_000 });

  await page.getByRole("button", { name: "Story-Bibel" }).click();
  const createForm = page.locator(".create-entity-form");
  await createForm.getByLabel("Name").fill("Corax");
  await createForm.getByRole("button", { name: "+ Eintrag" }).click();
  await page.getByRole("button", { name: "+ Eigenschaft" }).click();
  await page.getByLabel("Eigenschaftsname").fill("Rolle");
  await page.getByPlaceholder("Wert").fill("Archivar");
  await page.getByRole("button", { name: "Eintrag speichern" }).click();

  await page.getByRole("button", { name: "Visuelle Profile" }).click();
  await page.getByLabel("Kernmerkmale").fill("Gefieder: tiefschwarz\nAugen: bernsteinfarben");
  await page.getByLabel("Unerwünschte Abweichungen").fill("blaue Augen\nPlastikgefieder");
  await page.getByRole("button", { name: "Profil speichern" }).click();

  await page.getByLabel("Name", { exact: true }).fill("Ökosozialer Solarpunk");
  await page
    .getByLabel("Visuelle Regeln")
    .fill("Palette: Moosgrün und Kupfer\nLicht: warmes Gegenlicht");
  await page.getByRole("button", { name: "Stilvorlage anlegen" }).click();
  await expect(page.getByLabel("Gespeicherte Stilvorlagen")).toContainText("Ökosozialer Solarpunk");

  await page.getByRole("button", { name: "Bildprompts" }).click();
  await page.getByRole("checkbox", { name: "Figur: Corax" }).check();
  await page.getByLabel("Stilvorlage (optional)").selectOption({ label: "Ökosozialer Solarpunk" });
  await page.getByRole("button", { name: "Prompt zusammensetzen" }).click();

  const promptText = page.getByLabel("Prompttext");
  await expect(promptText).toContainText(sceneText);
  await expect(promptText).toContainText("Rolle: Archivar");
  await expect(promptText).toContainText("Gefieder: tiefschwarz");
  await expect(promptText).toContainText("Plastikgefieder");
  await expect(promptText).toContainText("Stilvorlage: Ökosozialer Solarpunk");
  await expect(promptText).toContainText("Palette: Moosgrün und Kupfer");

  const editedText = `${await promptText.inputValue()}\nKomposition: solidarisch und freundlich.`;
  await page.getByLabel("Name", { exact: true }).fill("Corax und die verdächtige Teekanne");
  await promptText.fill(editedText);
  await page.getByRole("button", { name: "Lokal speichern" }).click();
  await expect(page.getByRole("status")).toHaveText("Der Promptentwurf wurde lokal gespeichert.");

  await page.getByRole("button", { name: "Leeren" }).click();
  await expect(promptText).toHaveValue("");
  const savedDraft = page.locator(".prompt-draft-list article").filter({
    hasText: "Corax und die verdächtige Teekanne",
  });
  await savedDraft.getByRole("button", { name: "Bearbeiten" }).click();
  await expect(promptText).toHaveValue(editedText);

  await page.getByRole("button", { name: "Als UTF-8-Text exportieren" }).click();
  await expect(page.getByRole("status")).toContainText(/UTF-8-Export fertig: bildprompt-.+\.txt/);
  await expect(page.getByRole("status")).toContainText(/\([1-9][0-9]* Bytes\)/);
});
