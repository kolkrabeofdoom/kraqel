import { expect, test } from "@playwright/test";

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => localStorage.clear());
});

test("Export bietet alle lokalen Manuskriptformate mit Ergebnisbestätigung an", async ({
  page,
}) => {
  const manuscriptText = Array.from(
    { length: 24 },
    (_, index) =>
      `Abschnitt ${index + 1}: Der Kolkrabe archiviert solidarisch eine besonders höfliche Teekanne.`,
  ).join(" ");

  await page.goto("/");
  await page.getByLabel("Projektname").fill("Das Ministerium der tragbaren Manuskripte");
  await page.getByRole("button", { name: "Projekt anlegen" }).click();

  await page.getByRole("textbox", { name: "Text von Szene 1" }).fill(manuscriptText);
  await expect(page.getByText("Gespeichert", { exact: true })).toBeVisible({ timeout: 5_000 });

  await page.getByRole("button", { name: "Export", exact: true }).click();
  await expect(
    page.getByRole("heading", { level: 2, name: "Manuskript exportieren" }),
  ).toBeVisible();
  const openDirectory = page.getByRole("button", { name: "Exportordner öffnen" });
  await openDirectory.click();
  await expect(openDirectory).toBeEnabled();

  for (const [label, fileName] of [
    ["Markdown", "manuskript.md"],
    ["HTML", "manuskript.html"],
    ["DOCX", "manuskript.docx"],
    ["PDF", "manuskript.pdf"],
  ] as const) {
    await page.getByRole("button", { name: `${label} erzeugen` }).click();
    await expect(page.getByRole("status")).toContainText(`Export fertig: ${fileName}`);
    await expect(page.getByRole("status")).toContainText(/\([1-9]\d{0,2}(?:\.\d{3})+ Bytes\)/);
  }
});
