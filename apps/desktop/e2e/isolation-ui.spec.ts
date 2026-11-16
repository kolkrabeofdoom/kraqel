import { test, expect } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

test("restored outcomes focus replay and never offer a fresh import", async ({ page }) => {
  for (const result of ["done", "interrupted"]) {
    await page.goto(`/isolation-test.html?isolationRecovered=${result}`);
    const replay = page.getByRole("button", { name: "Erneuten Aufruf prüfen" });
    await expect(replay).toBeFocused();
    await expect(page.getByRole("button", { name: "Testarchiv importieren" })).toBeDisabled();
    if (result === "done")
      await expect(page.getByRole("status")).toContainText("Keine neue Projektkopie");
    else await expect(page.getByRole("alert")).toContainText("unterbrochen");
    await page.keyboard.press("Enter");
    await expect(page.getByRole("alert")).toContainText("bereits verbraucht");
    await expect(replay).toBeFocused();
  }
});

test("delayed selection hands focus to completion after a transient background pointer event", async ({
  page,
}) => {
  await page.goto("/isolation-test.html?isolationDelayImport=1");
  await page.keyboard.press("Tab");
  await page.keyboard.press("Enter");
  await expect(page.getByRole("button", { name: "Abbrechen", exact: true })).toBeFocused();
  await page.locator("body").dispatchEvent("pointerdown");
  await page.evaluate(() => window.dispatchEvent(new Event("kraqel-e2e-selection-complete")));
  await expect(page.getByRole("button", { name: "Import abschließen" })).toBeFocused();
});

test("delayed selection preserves a deliberate keyboard focus choice", async ({ page }) => {
  await page.goto("/isolation-test.html?isolationDelayImport=1");
  await page.keyboard.press("Tab");
  await page.keyboard.press("Enter");
  const cancel = page.getByRole("button", { name: "Abbrechen", exact: true });
  await expect(cancel).toBeFocused();
  await page.keyboard.press("Shift+Tab");
  await page.keyboard.press("Tab");
  await page.evaluate(() => window.dispatchEvent(new Event("kraqel-e2e-selection-complete")));
  await expect(page.getByRole("status")).toContainText("Projektkopie ist vorbereitet");
  await expect(cancel).toBeFocused();
});

test("keyboard focus follows enabled research actions and remains visible", async ({ page }) => {
  await page.goto("/isolation-test.html");
  const start = page.getByRole("button", { name: "Testarchiv importieren" });
  const proceed = page.getByRole("button", { name: "Import abschließen" });
  const replay = page.getByRole("button", { name: "Erneuten Aufruf prüfen" });
  await page.keyboard.press("Tab");
  await expect(start).toBeFocused();
  await expect
    .poll(() => start.evaluate((node) => getComputedStyle(node).outlineStyle))
    .not.toBe("none");
  await page.keyboard.press("Enter");
  await expect(proceed).toBeFocused();
  await page.keyboard.press("Tab");
  await expect(page.getByRole("button", { name: "Abbrechen", exact: true })).toBeFocused();
  await page.waitForTimeout(700); // Polling must not steal intentionally moved focus.
  await expect(page.getByRole("button", { name: "Abbrechen", exact: true })).toBeFocused();
  await page.keyboard.press("Shift+Tab");
  await expect(proceed).toBeFocused();
  await page.keyboard.press("Enter");
  await expect(page.getByRole("status")).toContainText("2 Dokumente");
  await expect(replay).toBeFocused();
  await page.keyboard.press("Enter");
  await expect(page.getByRole("alert")).toContainText("bereits verbraucht");
  await expect(replay).toBeFocused();
});

test("failure and cancellation leave a reachable focused action without serious axe findings", async ({
  page,
}) => {
  for (const fail of [false, true]) {
    await page.goto(`/isolation-test.html${fail ? "?isolationFailImport=1" : ""}`);
    const ready = await new AxeBuilder({ page }).analyze();
    expect(
      ready.violations.filter(({ impact }) => impact === "serious" || impact === "critical"),
    ).toEqual([]);
    await page.keyboard.press("Tab");
    await page.keyboard.press("Enter");
    if (!fail) {
      await expect(page.getByRole("button", { name: "Import abschließen" })).toBeFocused();
      await page.keyboard.press("Tab");
      await expect(page.getByRole("button", { name: "Abbrechen", exact: true })).toBeFocused();
      await page.keyboard.press("Enter");
    }
    await expect(page.getByRole("button", { name: "Erneuten Aufruf prüfen" })).toBeFocused();
    const result = await new AxeBuilder({ page }).analyze();
    expect(
      result.violations.filter(({ impact }) => impact === "serious" || impact === "critical"),
    ).toEqual([]);
  }
});
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
