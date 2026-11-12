import { expect, test } from "@playwright/test";

test.beforeEach(async ({ page }) => {
  await page.goto("/");
  await page.evaluate(() => localStorage.clear());
  await page.reload();
});

test("Projektarchiv wird aus einem aktuellen Snapshot exportiert", async ({ page }) => {
  await page.getByRole("button", { name: "Abenteuer ausprobieren" }).click();
  await page.getByRole("button", { name: "Versionen" }).click();

  await page.getByRole("button", { name: "Projektarchiv exportieren" }).click();

  await expect(page.getByText(/Projektarchiv exportiert: kraqel-projekt\.kraqel/u)).toBeVisible();
  await expect(page.getByText(/Format 3, [1-9][0-9]* Dateien/u)).toBeVisible();
});

test("abgebrochener Projektarchivexport bleibt neutral", async ({ page }) => {
  await page.evaluate(() => localStorage.setItem("kraqel-e2e-cancel-snapshot-export", "1"));
  await page.getByRole("button", { name: "Abenteuer ausprobieren" }).click();
  await page.getByRole("button", { name: "Versionen" }).click();

  await page.getByRole("button", { name: "Projektarchiv exportieren" }).click();

  await expect(page.getByRole("alert")).toHaveCount(0);
  await expect(page.getByRole("button", { name: "Projektarchiv exportieren" })).toBeEnabled();
});

test("lokales Projekt wird erst mit dem zweiten bewussten Klick gelöscht", async ({ page }) => {
  await page.getByLabel("Projektname").fill("Die Kommune der verlorenen Regenschirme");
  await page.getByRole("button", { name: "Projekt anlegen" }).click();
  await page.getByRole("button", { name: "Projektübersicht" }).click();

  const openProject = page.getByRole("button", {
    name: /Die Kommune der verlorenen Regenschirme.*Öffnen/u,
  });
  await expect(openProject).toBeVisible();

  await page
    .getByRole("button", { name: "Die Kommune der verlorenen Regenschirme löschen" })
    .click();
  await expect(openProject).toBeVisible();
  await expect(
    page.getByRole("button", {
      name: "Die Kommune der verlorenen Regenschirme wirklich löschen",
    }),
  ).toBeVisible();

  await page
    .getByRole("button", {
      name: "Die Kommune der verlorenen Regenschirme wirklich löschen",
    })
    .click();

  await expect(openProject).toHaveCount(0);
  await expect(page.getByText("Noch kein Projekt. Die leere Liste urteilt nicht.")).toBeVisible();
});
