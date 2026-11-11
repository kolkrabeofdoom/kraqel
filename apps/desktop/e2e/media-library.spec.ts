import { expect, test } from "@playwright/test";

const onePixelPng = Buffer.from(
  "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII=",
  "base64",
);

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => localStorage.clear());
});

test("Medienbibliothek importiert Metadaten und verknüpft die aktive Szene", async ({ page }) => {
  await page.goto("/");
  await page.getByLabel("Projektname").fill("Das Museum der solidarischen Kolkraben");
  await page.getByRole("button", { name: "Projekt anlegen" }).click();

  await page.getByRole("button", { name: "Medien", exact: true }).click();
  await expect(page.getByRole("heading", { level: 2, name: "Medienbibliothek" })).toBeVisible();
  await page.getByLabel("PNG, JPEG oder WebP auswählen").setInputFiles({
    name: "kolkrabe.png",
    mimeType: "image/png",
    buffer: onePixelPng,
  });
  await page.getByLabel("Titel", { exact: true }).fill("Kolkrabe im Archiv");
  await page.getByLabel("Schlagwörter, durch Komma getrennt").fill("Kolkrabe, Archiv");
  await page.getByRole("button", { name: "Bild lokal importieren" }).click();

  const card = page.locator(".media-card");
  await expect(card.getByRole("img", { name: "Kolkrabe im Archiv" })).toBeVisible();
  await expect(card.getByLabel("Titel", { exact: true })).toHaveValue("Kolkrabe im Archiv");
  await expect(card.getByLabel("Schlagwörter", { exact: true })).toHaveValue("Kolkrabe, Archiv");
  await expect(card.getByText("image/png", { exact: true })).toBeVisible();
  await expect(card.getByText("1 × 1", { exact: true })).toBeVisible();
  await expect(card.getByTitle("7a".repeat(32))).toHaveText(`${"7a".repeat(8)}…`);

  await card.getByRole("button", { name: "Mit aktiver Szene verknüpfen" }).click();
  const sceneLink = card.getByRole("button", { name: "Szene: Szene 1", exact: true });
  await expect(sceneLink).toBeVisible();
  await sceneLink.click();
  await expect(page.getByRole("textbox", { name: "Text von Szene 1" })).toBeVisible();
});
