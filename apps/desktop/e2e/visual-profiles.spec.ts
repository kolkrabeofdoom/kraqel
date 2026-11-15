import { expect, test } from "@playwright/test";

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => localStorage.clear());
});

test("Visuelle Profile und Stilvorlagen bleiben nach Ansichtswechsel erhalten", async ({
  page,
}) => {
  const traits = "Gefieder: tiefschwarz\nAugen: bernsteinfarben";
  const exclusions = "Plastikgefieder\nblaue Augen";
  const rules = "Palette: Moosgrün und Kupfer\nLicht: warmes Gegenlicht";

  await page.goto("/");
  await page.getByLabel("Projektname").fill("Das Atelier der eigensinnigen Kolkraben");
  await page.getByRole("button", { name: "Projekt anlegen" }).click();

  await page.getByRole("button", { name: "Story-Bibel" }).click();
  const createForm = page.locator(".create-entity-form");
  await createForm.getByLabel("Name").fill("Corax");
  await createForm.getByRole("button", { name: "+ Eintrag" }).click();

  await page.getByRole("button", { name: "Visuelle Profile" }).click();
  await expect(page.getByLabel("Story-Bibel-Eintrag")).toHaveValue(/.+/);
  await page.getByLabel("Kernmerkmale").fill(traits);
  await page.getByLabel("Unerwünschte Abweichungen").fill(exclusions);
  await page.getByRole("button", { name: "Profil speichern" }).click();
  await expect(page.getByRole("status")).toHaveText("Visuelles Profil gespeichert.");
  await expect(page.getByRole("button", { name: "Profil löschen" })).toBeVisible();

  await page.getByLabel("Name", { exact: true }).fill("Ökosozialer Solarpunk");
  await page.getByLabel("Visuelle Regeln").fill(rules);
  await page.getByRole("checkbox", { name: "Orte" }).check();
  await page.getByRole("button", { name: "Stilvorlage anlegen" }).click();
  await expect(page.getByRole("status")).toHaveText(
    "Stilvorlage „Ökosozialer Solarpunk“ gespeichert.",
  );

  const preset = page.locator(".preset-card").filter({ hasText: "Ökosozialer Solarpunk" });
  await expect(preset).toContainText("Figuren, Orte");
  await expect(preset).toContainText("Moosgrün und Kupfer");
  await expect(preset).toContainText("warmes Gegenlicht");

  await page.getByRole("button", { name: "Manuskript" }).click();
  await page.getByRole("button", { name: "Visuelle Profile" }).click();

  await expect(page.getByLabel("Story-Bibel-Eintrag")).toContainText("Figuren: Corax");
  await expect(page.getByLabel("Kernmerkmale")).toHaveValue(traits);
  await expect(page.getByLabel("Unerwünschte Abweichungen")).toHaveValue(exclusions);
  await expect(preset).toContainText("Figuren, Orte");
  await expect(preset).toContainText("Palette");
  await expect(preset).toContainText("Licht");

  // A validation failure must be announced and must preserve the saved profile.
  await page.getByLabel("Kernmerkmale").fill("Augen: bernstein\nAugen: blau");
  await page.getByRole("button", { name: "Profil speichern" }).click();
  await expect(page.getByRole("alert")).toContainText(
    "Das visuelle Profil konnte nicht gespeichert werden.",
  );
  await expect(page.getByRole("status")).toBeEmpty();
  await page.getByRole("button", { name: "Manuskript" }).click();
  await page.getByRole("button", { name: "Visuelle Profile" }).click();
  await expect(page.getByLabel("Kernmerkmale")).toHaveValue(traits);

  await preset.getByRole("button", { name: "Bearbeiten", exact: true }).click();
  await page.getByLabel("Name", { exact: true }).fill("Tusche geändert");
  await page.getByLabel("Visuelle Regeln").fill("Linien: klar");
  await page.getByRole("button", { name: "Änderungen speichern" }).click();
  await expect(page.getByRole("status")).toHaveText("Stilvorlage „Tusche geändert“ gespeichert.");
  const updated = page.locator(".preset-card").filter({ hasText: "Tusche geändert" });
  await updated.getByRole("button", { name: "Löschen", exact: true }).click();
  await expect(updated).toBeVisible();
  await updated.getByRole("button", { name: "Löschen bestätigen" }).click();
  await expect(page.getByRole("status")).toHaveText("Stilvorlage „Tusche geändert“ gelöscht.");
  await expect(updated).toHaveCount(0);
  await page.getByRole("button", { name: "Profil löschen" }).click();
  await expect(page.getByLabel("Kernmerkmale")).toHaveValue(traits);
  await page.getByRole("button", { name: "Löschen bestätigen" }).click();
  await expect(page.getByRole("status")).toHaveText("Visuelles Profil gelöscht.");
  await expect(page.getByLabel("Kernmerkmale")).toBeEmpty();
  await page.getByRole("button", { name: "Manuskript" }).click();
  await page.getByRole("button", { name: "Visuelle Profile" }).click();
  await expect(page.getByLabel("Kernmerkmale")).toBeEmpty();
  await expect(page.locator(".preset-card")).toHaveCount(0);
});
