import { expect, test } from "@playwright/test";

test.beforeEach(async ({ page }) => {
  await page.goto("/");
  await page.evaluate(() => localStorage.clear());
  await page.reload();
});

test("gebündeltes Beispiel wird als bearbeitbare Offline-Kopie angelegt", async ({ page }) => {
  await page.getByRole("button", { name: "Abenteuer ausprobieren" }).click();

  await expect(
    page.getByRole("heading", {
      level: 1,
      name: "Die Stadt, die den Dienstag verkaufte (Beispiel)",
    }),
  ).toBeVisible();
  await expect(page.getByRole("tree", { name: "Dokumentbaum" })).toContainText("Das Abenteuer");
  await expect(page.getByRole("tree", { name: "Dokumentbaum" })).toContainText(
    "1 · Der fehlende Morgen",
  );

  await page.getByRole("button", { name: "Story-Bibel" }).click();
  await expect(page.getByText("2 Einträge")).toBeVisible();
  await expect(page.getByRole("button", { name: /Mio Kest/u })).toBeVisible();

  await page.getByRole("button", { name: "Timelines" }).click();
  await expect(page.getByText("Zwei Abende", { exact: true })).toBeVisible();
  await expect(page.getByText("Abend 1 · 0:00–0:20", { exact: true })).toBeVisible();

  await page.getByRole("button", { name: "Medien" }).click();
  await expect(page.getByRole("heading", { name: "Medienbibliothek" })).toBeVisible();
  await expect(page.getByRole("img", { name: "Mio Kest" })).toBeVisible();

  await page.getByRole("button", { name: "Projektübersicht" }).click();
  await page.getByRole("button", { name: "Abenteuer ausprobieren" }).click();
  await page.getByRole("button", { name: "Projektübersicht" }).click();
  await expect(
    page.getByRole("button", { name: /Die Stadt, die den Dienstag verkaufte \(Beispiel\)/u }),
  ).toHaveCount(2);
});

test("frei gewähltes Projektarchiv wird als neue lokale Kopie geöffnet", async ({ page }) => {
  await page.getByRole("button", { name: "Projektarchiv auswählen" }).click();

  await expect(
    page.getByRole("heading", {
      level: 1,
      name: "Die Stadt, die den Dienstag verkaufte (Beispiel)",
    }),
  ).toBeVisible();
  await expect(page.getByRole("tree", { name: "Dokumentbaum" })).toContainText(
    "1 · Der fehlende Morgen",
  );
});

test("abgebrochene Archivauswahl bleibt neutral", async ({ page }) => {
  await page.evaluate(() => localStorage.setItem("kraqel-e2e-cancel-snapshot-import", "1"));

  await page.getByRole("button", { name: "Projektarchiv auswählen" }).click();

  await expect(page.getByRole("heading", { level: 1, name: "KRAQEL" })).toBeVisible();
  await expect(page.getByRole("alert")).toHaveCount(0);
  await expect(page.getByRole("button", { name: "Projektarchiv auswählen" })).toBeEnabled();
});
