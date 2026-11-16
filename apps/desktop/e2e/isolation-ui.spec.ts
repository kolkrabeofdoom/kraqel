import { test, expect } from "@playwright/test";
test("research UI completes one import and announces rejected reuse", async ({ page }) => {
  await page.goto("/isolation-test.html");
  await expect(page.getByRole("status")).toContainText("Dateiauftrag ist bereit");
  await page.getByRole("button", { name: "Testarchiv importieren" }).click();
  await expect(page.getByRole("button", { name: "Import abschließen" })).toBeEnabled();
  await page.getByRole("button", { name: "Import abschließen" }).click();
  await expect(page.getByRole("status")).toContainText("2 Dokumente");
  await page.getByRole("button", { name: "Erneuten Aufruf prüfen" }).click();
  await expect(page.getByRole("alert")).toContainText("bereits verbraucht");
});
test("research UI discards the unpublished copy on cancellation", async ({ page }) => {
  await page.goto("/isolation-test.html");
  await page.getByRole("button", { name: "Testarchiv importieren" }).click();
  await expect(page.getByRole("button", { name: "Import abschließen" })).toBeEnabled();
  await page.getByRole("button", { name: "Abbrechen", exact: true }).click();
  await expect(page.getByRole("status")).toContainText("Keine Projektkopie");
  await expect(page.getByRole("button", { name: "Import abschließen" })).toBeDisabled();
});

test("replay rejection remains visible after a failed import and status polling", async ({
  page,
}) => {
  await page.goto("/isolation-test.html?isolationFailImport=1");
  await page.getByRole("button", { name: "Testarchiv importieren" }).click();
  await expect(page.getByRole("alert")).toContainText(
    "Der Import konnte nicht abgeschlossen werden.",
  );
  await page.getByRole("button", { name: "Erneuten Aufruf prüfen" }).click();
  await expect(page.getByRole("alert")).toContainText("bereits verbraucht");
  await page.waitForTimeout(700); // Includes two 300-ms status refreshes that caused the regression.
  await expect(page.getByRole("alert")).toContainText("bereits verbraucht");
});
