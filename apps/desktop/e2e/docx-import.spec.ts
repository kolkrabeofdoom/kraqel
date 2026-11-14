import { expect, test } from "@playwright/test";

test("DOCX-Import erklärt Grenzen, erhält Formatierungen und behandelt Abbruch und Fehler atomar", async ({
  page,
}) => {
  await page.goto("/");
  await page.getByLabel("Projektname").fill("DOCX-Importprobe");
  await page.getByRole("button", { name: "Projekt anlegen" }).click();
  const chapter = page.locator(".tree-chapter").filter({ hasText: "Kapitel 1" });
  const action = chapter.getByRole("button", { name: "Import: DOCX in Kapitel 1" });
  const select = page.getByRole("button", { name: "DOCX-Datei auswählen" });

  await action.click();
  await expect(page.getByRole("heading", { name: "DOCX als neue Szene" })).toBeFocused();
  await expect(
    page.getByText(/Schriftarten und Seitenlayout werden nicht übernommen/),
  ).toBeVisible();
  await page.getByRole("button", { name: "DOCX-Import abbrechen" }).click();
  await expect(chapter.locator(".scene-row")).toHaveCount(1);

  await page.evaluate(() => localStorage.setItem("kraqel-e2e-docx-import", "cancel"));
  await action.click();
  await select.click();
  await expect(select).toHaveCount(0);
  await expect(chapter.locator(".scene-row")).toHaveCount(1);
  await expect(page.locator(".error-banner")).toHaveCount(0);

  await page.evaluate(() => localStorage.setItem("kraqel-e2e-docx-import", "error"));
  await action.click();
  await select.click();
  await expect(page.locator(".error-banner")).toContainText("nicht unterstützte Inhalte");
  await expect(page.locator(".error-banner")).not.toContainText("untrusted parser");
  await expect(chapter.locator(".scene-row")).toHaveCount(1);

  await page.evaluate(() => localStorage.removeItem("kraqel-e2e-docx-import"));
  await action.click();
  await select.click();
  const editor = page.getByRole("textbox", { name: "Text von DOCX-Probe" });
  await expect(editor.getByRole("heading", { name: "Überschrift" })).toBeVisible();
  await expect(editor.locator("strong em, em strong")).toHaveText("Importiertes DOCX");
  await expect(chapter.locator(".scene-row")).toHaveCount(2);
  await page.reload();
  await page.getByRole("button", { name: "DOCX-Importprobe Öffnen" }).click();
  await chapter.getByRole("button", { name: "DOCX-Probe", exact: true }).click();
  await expect(editor).toContainText("Importiertes DOCX");
});
