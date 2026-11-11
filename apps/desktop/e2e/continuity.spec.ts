import { expect, test } from "@playwright/test";

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => localStorage.clear());
});

test("Kontinuitätsprüfung belegt einen Offline-Befund und führt zur Szene", async ({ page }) => {
  await page.goto("/");
  await page.getByLabel("Projektname").fill("Das Paradox im Teekessel");
  await page.getByRole("button", { name: "Projekt anlegen" }).click();

  const firstChapter = page.locator(".tree-chapter").filter({ hasText: "Kapitel 1" });
  await firstChapter.getByRole("button", { name: "Neue Szene in Kapitel 1" }).click();

  const manuscriptText = "Im Nebel begrüßt Corax den Dienstag, obwohl der noch schläft.";
  const editor = page.getByRole("textbox", { name: "Text von Szene 2" });
  await editor.fill(manuscriptText);
  await expect(page.getByText("Gespeichert", { exact: true })).toBeVisible({ timeout: 5_000 });

  await page.getByRole("button", { name: "Story-Bibel" }).click();
  const createForm = page.locator(".create-entity-form");
  await createForm.getByLabel("Name").fill("Corax");
  await createForm.getByRole("button", { name: "+ Eintrag" }).click();
  await expect(page.getByRole("heading", { level: 3, name: "Corax" })).toBeVisible();

  await page.getByRole("button", { name: "Manuskript" }).click();
  await firstChapter.getByRole("button", { name: "Szene 1", exact: true }).click();
  await expect(page.getByRole("textbox", { name: "Text von Szene 1" })).toBeVisible();

  await page.getByRole("button", { name: "Kontinuität" }).click();
  await expect(
    page.getByText("Noch kein Prüflauf. Die Fakten sitzen ordentlich im Regal."),
  ).toBeVisible();
  await page.getByRole("button", { name: "Projekt prüfen" }).click();

  const finding = page.locator(".finding-card");
  await expect(finding.getByText("Information", { exact: true })).toBeVisible();
  await expect(finding.getByText("story-bible.unlinked-mention", { exact: true })).toBeVisible();
  await expect(
    finding.getByRole("heading", { level: 3, name: "„Corax“ wird in „Szene 2“ erwähnt" }),
  ).toBeVisible();

  const source = finding.getByRole("button", { name: "„Corax“ in Szene 2 (Zeichen 18–22)" });
  await expect(source).toBeVisible();
  await source.click();

  await expect(page.getByRole("button", { name: "Manuskript" })).toHaveAttribute(
    "aria-current",
    "page",
  );
  await expect(page.getByRole("textbox", { name: "Text von Szene 2" })).toContainText(
    manuscriptText,
  );
});
