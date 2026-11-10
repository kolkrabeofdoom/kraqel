import { expect, test } from "@playwright/test";

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => localStorage.clear());
});

test("Plotboard verbindet Metadaten, Reihenfolge und Manuskript-Rücksprung", async ({ page }) => {
  await page.goto("/");
  await page.getByLabel("Projektname").fill("Der geordnete Flügelschlag");
  await page.getByRole("button", { name: "Projekt anlegen" }).click();

  const firstChapter = page.locator(".tree-chapter").filter({ hasText: "Kapitel 1" });
  await firstChapter.getByRole("button", { name: "Neue Szene in Kapitel 1" }).click();

  await page.getByRole("button", { name: "Story-Bibel" }).click();
  const createForm = page.locator(".create-entity-form");
  await createForm.getByLabel("Name").fill("Corax");
  await createForm.getByRole("button", { name: "+ Eintrag" }).click();

  await page.getByRole("button", { name: "Plotboard" }).click();
  await page.getByLabel("Handlungsstrang", { exact: true }).fill("Die verschwundene Teekanne");
  await page.getByRole("button", { name: "+ Strang" }).click();
  await expect(page.getByLabel("Handlungsstränge")).toContainText("Die verschwundene Teekanne");

  const secondCard = page.locator(".plot-card").filter({ hasText: "Szene 2" });
  await secondCard.getByLabel("Status").selectOption("draft");
  await secondCard
    .getByLabel("Handlungsstrang")
    .selectOption({ label: "Die verschwundene Teekanne" });
  await secondCard.getByLabel("POV").selectOption({ label: "Corax" });
  await expect(secondCard.getByLabel("Status")).toHaveValue("draft");
  await expect(secondCard.getByLabel("Handlungsstrang")).not.toHaveValue("");
  await expect(secondCard.getByLabel("POV")).not.toHaveValue("");

  await secondCard.getByRole("button", { name: "Nach links" }).focus();
  await page.keyboard.press("Enter");
  await expect(page.locator(".plot-card h3 button")).toHaveText(["Szene 2", "Szene 1"]);

  await page.locator(".plot-card").first().getByRole("button", { name: "Szene 2" }).click();
  await expect(page.getByRole("button", { name: "Manuskript" })).toHaveAttribute(
    "aria-current",
    "page",
  );
  await expect(page.getByRole("textbox", { name: "Text von Szene 2" })).toBeVisible();
});
