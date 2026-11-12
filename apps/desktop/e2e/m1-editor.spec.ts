import { expect, test } from "@playwright/test";

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => localStorage.clear());
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
