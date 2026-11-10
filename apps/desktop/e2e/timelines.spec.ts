import { expect, test } from "@playwright/test";

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => localStorage.clear());
});

test("Timelines verbinden relative und absolute Zeit mit Szene und Entität", async ({ page }) => {
  await page.goto("/");
  await page.getByLabel("Projektname").fill("Die Uhr des Kolkraben");
  await page.getByRole("button", { name: "Projekt anlegen" }).click();

  await page.getByRole("button", { name: "Story-Bibel" }).click();
  const createForm = page.locator(".create-entity-form");
  await createForm.getByLabel("Name").fill("Corax");
  await createForm.getByRole("button", { name: "+ Eintrag" }).click();

  await page.getByRole("button", { name: "Timelines" }).click();
  await page.getByRole("textbox", { name: "Zeitebene" }).fill("Erzählzeit");
  await page.getByLabel("Ordnung").selectOption("relative");
  await page.getByRole("button", { name: "+ Zeitebene" }).click();

  await page.getByLabel("Ereignis").fill("Der Brief trifft ein");
  await page.getByRole("button", { name: "+ Ereignis" }).click();
  await page.getByLabel("Ereignis").fill("Die Antwort fliegt los");
  await page.getByRole("button", { name: "+ Ereignis" }).click();

  const answerEvent = page
    .locator(".event-list > li")
    .filter({ hasText: "Die Antwort fliegt los" });
  await answerEvent.getByRole("button", { name: "Nach oben" }).focus();
  await page.keyboard.press("Enter");
  await expect(page.locator(".event-list h4")).toHaveText([
    "Die Antwort fliegt los",
    "Der Brief trifft ein",
  ]);

  const firstEvent = page.locator(".event-list > li").first();
  await firstEvent.getByRole("button", { name: "Mit „Szene 1“ verknüpfen" }).click();
  await firstEvent
    .getByLabel("Entität für Die Antwort fliegt los")
    .selectOption({ label: "Corax" });
  await firstEvent.getByRole("button", { name: "Entität verknüpfen" }).click();
  await expect(firstEvent).toContainText("Entität: Corax");

  await page.getByRole("textbox", { name: "Zeitebene" }).fill("Kalenderzeit");
  await page.getByLabel("Ordnung").selectOption("absolute");
  await page.getByRole("button", { name: "+ Zeitebene" }).click();
  await expect(page.locator(".timeline-detail")).toContainText("Absolute Ordnung");
  await page.getByLabel("Ereignis").fill("Der große Dienstag");
  await page.getByLabel("Zeitpunkt").fill("2042-05-17");
  await page.getByRole("button", { name: "+ Ereignis" }).click();
  await expect(page.locator(".event-list")).toContainText("2042-05-17");

  await page.getByRole("button", { name: /Erzählzeit/ }).click();
  const sceneLink = page
    .locator(".event-list > li")
    .filter({ hasText: "Die Antwort fliegt los" })
    .getByRole("button", { name: "Szene: Szene 1" });
  await expect(sceneLink).toBeVisible();
  await sceneLink.click();
  await expect(page.getByRole("button", { name: "Manuskript" })).toHaveAttribute(
    "aria-current",
    "page",
  );
  await expect(page.getByRole("textbox", { name: "Text von Szene 1" })).toBeVisible();
});
