import { expect, test } from "@playwright/test";

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => localStorage.clear());
});

test("Versionen prüfen Snapshot, Diff und explizite lokale Commits", async ({ page }) => {
  await page.goto("/");
  await page.getByLabel("Projektname").fill("Das Archiv der vernünftigen Teekannen");
  await page.getByRole("button", { name: "Projekt anlegen" }).click();

  const editor = page.getByRole("textbox", { name: "Text von Szene 1" });
  await editor.fill("Corax katalogisiert eine Teekanne.");
  await expect(page.getByText("Gespeichert", { exact: true })).toBeVisible({ timeout: 5_000 });

  await page.getByRole("button", { name: "Versionen" }).click();
  await expect(page.getByText("Noch nicht initialisiert", { exact: true })).toBeVisible();
  await page.getByRole("button", { name: "Snapshot erzeugen" }).click();
  await expect(page.getByText(/Snapshot mit [1-9][0-9]* Dateien/)).toBeVisible();
  await expect(page.getByText(/Änderung\(en\) bereit/)).toBeVisible();
  await page.getByRole("button", { name: "Lesend prüfen" }).click();
  await expect(
    page.getByText("Der Snapshot ist lesbar und entspricht dem Formatvertrag."),
  ).toBeVisible();

  await page.getByLabel("Commitnachricht").fill("Portablen Grundstand sichern");
  await page.getByRole("button", { name: "Commit erstellen" }).click();
  await expect(page.getByText(/Snapshot wurde als [0-9a-f]{12} versioniert/)).toBeVisible();
  await expect(page.getByText("Snapshot ist unverändert", { exact: true })).toBeVisible();

  await page.getByRole("button", { name: "Manuskript" }).click();
  const changedText = "Corax katalogisiert eine Teekanne und entdeckt solidarische Fußnoten.";
  await editor.fill(changedText);
  await expect(page.getByText("Gespeichert", { exact: true })).toBeVisible({ timeout: 5_000 });

  await page.getByRole("button", { name: "Versionen" }).click();
  await expect(page.getByText("Snapshot ist unverändert", { exact: true })).toBeVisible();
  await page.getByRole("button", { name: "Snapshot erzeugen" }).click();
  await expect(page.getByText("2 Änderung(en) bereit", { exact: true })).toBeVisible();
  await expect(
    page
      .getByRole("region", { name: "Geänderte Snapshotdateien" })
      .getByText(/snapshot\/manuscript\/.+\.md/),
  ).toBeVisible();
  const diff = page.getByRole("region", { name: "Unified Diff" }).locator("pre");
  await expect(diff).toContainText("diff --git");
  await expect(diff).toContainText(`+${changedText}`);

  await page.getByLabel("Commitnachricht").fill("Solidarische Fußnoten ergänzen");
  await page.getByRole("button", { name: "Commit erstellen" }).click();
  await expect(page.getByText("Snapshot ist unverändert", { exact: true })).toBeVisible();
  await expect(page.getByText("Keine freigegebenen Änderungen.", { exact: true })).toBeVisible();
  await expect(diff).toHaveText("Noch kein Diff vorhanden.");
});
