import AxeBuilder from "@axe-core/playwright";
import { expect, test } from "@playwright/test";

test("DOCX-Austausch erklärt Umfang, importiert neue Kopien und behandelt Abbruch und Fehler", async ({
  page,
}) => {
  await page.goto("/");
  const select = page.getByRole("button", { name: "Manuskript-DOCX auswählen" });
  await expect(page.getByText(/Ohne Story-Bibel, Plotboard/)).toBeVisible();
  await select.focus();
  await expect(select).toBeFocused();
  await page.evaluate(() => localStorage.setItem("kraqel-e2e-roundtrip-import", "cancel"));
  await page.keyboard.press("Enter");
  await expect(select).toBeEnabled();
  await expect(page.locator(".project-list li")).toHaveCount(0);
  await expect(page.getByRole("alert")).toHaveCount(0);
  await page.evaluate(() => localStorage.setItem("kraqel-e2e-roundtrip-import", "error"));
  await select.click();
  await expect(page.getByRole("alert")).toContainText("nicht importiert");
  await expect(page.getByRole("alert")).not.toContainText("private DOCX");
  await expect(page.locator(".project-list li")).toHaveCount(0);

  await page.getByLabel("Projektname").fill("Manuskriptkopie");
  await page.getByRole("button", { name: "Projekt anlegen" }).click();
  const editor = page.getByRole("textbox", { name: "Text von Szene 1" });
  await editor.fill("Text bleibt in beiden Projekten.");
  await expect(page.getByText("Gespeichert", { exact: true })).toBeVisible();
  await page.getByRole("button", { name: "Export", exact: true }).click();
  await expect(page.getByRole("button", { name: "DOCX erzeugen", exact: true })).toBeVisible();
  await page.getByRole("button", { name: "DOCX zum Wiederimport erzeugen" }).click();
  await expect(page.getByRole("status")).toContainText("manuskript-roundtrip.docx");
  const axe = await new AxeBuilder({ page }).analyze();
  expect(
    axe.violations.filter(({ impact }) => impact === "serious" || impact === "critical"),
  ).toEqual([]);
  await page.getByRole("button", { name: "Projektübersicht" }).click();
  await page.evaluate(() => localStorage.removeItem("kraqel-e2e-roundtrip-import"));
  await select.click();
  await expect(editor).toContainText("Text bleibt in beiden Projekten.");
  await editor.fill("Nur die neue Kopie wird geändert.");
  await expect(page.getByText("Gespeichert", { exact: true })).toBeVisible();
  await page.getByRole("button", { name: "Projektübersicht" }).click();
  await expect(page.locator(".project-list li")).toHaveCount(2);
  await page.getByRole("button", { name: "Manuskriptkopie Öffnen" }).first().click();
  await expect(editor).toHaveText("Text bleibt in beiden Projekten.");
  await page.reload();
  await page.getByRole("button", { name: "Manuskriptkopie Öffnen" }).nth(1).click();
  await expect(editor).toHaveText("Nur die neue Kopie wird geändert.");
});
