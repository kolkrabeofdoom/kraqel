import AxeBuilder from "@axe-core/playwright";
import { expect, test, type Locator, type Page } from "@playwright/test";

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => localStorage.clear());
});

async function tabTo(page: Page, target: Locator, maximumTabs = 40): Promise<void> {
  for (let index = 0; index < maximumTabs; index += 1) {
    await page.keyboard.press("Tab");
    if (await target.evaluate((element) => element === document.activeElement)) return;
  }

  throw new Error(`Ziel erhielt nach ${maximumTabs} Tab-Schritten keinen Fokus.`);
}

async function expectVisibleFocus(target: Locator): Promise<void> {
  await expect(target).toBeFocused();
  await expect
    .poll(() => target.evaluate((element) => getComputedStyle(element).outlineStyle))
    .not.toBe("none");
}

async function expectNoSeriousAccessibilityViolations(page: Page): Promise<void> {
  const results = await new AxeBuilder({ page }).analyze();
  const violations = results.violations.filter(
    ({ impact }) => impact === "critical" || impact === "serious",
  );

  expect(
    violations,
    violations
      .map(
        ({ id, help, nodes }) =>
          `${id}: ${help}\n${nodes.map(({ target }) => `  ${target.join(" ")}`).join("\n")}`,
      )
      .join("\n"),
  ).toEqual([]);
}

test("Projekttitel und fachlich gruppierte Navigation konkurrieren nicht um dieselbe Zeile", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1536, height: 768 });
  await page.goto("/");
  await page.getByLabel("Projektname").fill("Die Stadt, die den Dienstag verkaufte (Beispiel)");
  await page.getByRole("button", { name: "Projekt anlegen" }).click();

  const title = page.getByRole("heading", {
    level: 1,
    name: "Die Stadt, die den Dienstag verkaufte (Beispiel)",
  });
  const navigation = page.getByRole("navigation", { name: "Projektbereiche" });
  const overview = page.getByRole("button", { name: "Projektübersicht" });
  const [titleBox, navigationBox, overviewBox] = await Promise.all([
    title.boundingBox(),
    navigation.boundingBox(),
    overview.boundingBox(),
  ]);

  expect(titleBox).not.toBeNull();
  expect(navigationBox).not.toBeNull();
  expect(overviewBox).not.toBeNull();
  expect(titleBox!.height).toBeLessThan(70);
  expect(navigationBox!.y).toBeGreaterThanOrEqual(titleBox!.y + titleBox!.height);
  expect(overviewBox!.y).toBeLessThan(navigationBox!.y);
  for (const label of ["Schreiben", "Planen", "Material", "Werkzeuge"]) {
    await expect(page.getByRole("group", { name: label })).toBeVisible();
  }
});

test("Kernablauf bleibt per Tastatur erreichbar und zeigt den Fokus", async ({ page }) => {
  await page.goto("/");

  const projectName = page.getByLabel("Projektname");
  await tabTo(page, projectName);
  await expectVisibleFocus(projectName);
  await page.keyboard.type("Die barrierearme Bibliothek");

  const createProject = page.getByRole("button", { name: "Projekt anlegen" });
  await tabTo(page, createProject);
  await expectVisibleFocus(createProject);
  await page.keyboard.press("Enter");

  await expect(
    page.getByRole("heading", { level: 1, name: "Die barrierearme Bibliothek" }),
  ).toBeVisible();

  const manuscript = page.getByRole("button", { name: "Manuskript" });
  await tabTo(page, manuscript);
  await expectVisibleFocus(manuscript);
  await expect(manuscript).toHaveAttribute("aria-current", "page");

  const editor = page.getByRole("textbox", { name: "Text von Szene 1" });
  await tabTo(page, editor);
  await expectVisibleFocus(editor);
  await page.keyboard.type("Ein Kolkrabe sortiert Sterne.");
  await expect(page.getByRole("status")).toHaveText("Ungespeichert");
  await page.keyboard.press("Control+A");

  const bold = page.getByRole("button", { name: "Fett" });
  await tabTo(page, bold);
  await expectVisibleFocus(bold);
  await expect(bold).toHaveAttribute("aria-pressed", "false");
  await page.keyboard.press("Enter");
  await expect(bold).toHaveAttribute("aria-pressed", "true");

  const focusMode = page.getByRole("button", { name: "Fokusmodus" });
  await tabTo(page, focusMode);
  await page.keyboard.press("Enter");
  await expect(focusMode).toHaveAttribute("aria-pressed", "true");

  await expect(page.getByRole("status")).toHaveText("Gespeichert", { timeout: 5_000 });

  const storyBible = page.getByRole("button", { name: "Story-Bibel" });
  await tabTo(page, storyBible);
  await page.keyboard.press("Enter");
  await expect(storyBible).toHaveAttribute("aria-current", "page");

  const search = page.getByRole("searchbox", { name: "Suche" });
  await tabTo(page, search);
  await expectVisibleFocus(search);
});

test("Kernansichten haben keine schwerwiegenden automatisiert erkennbaren Verstöße", async ({
  page,
}) => {
  await page.goto("/");
  await expectNoSeriousAccessibilityViolations(page);

  await page.getByLabel("Projektname").fill("Die zugängliche Republik der Vögel");
  await page.getByRole("button", { name: "Projekt anlegen" }).click();
  await expect(page.getByRole("textbox", { name: "Text von Szene 1" })).toBeVisible();
  await expectNoSeriousAccessibilityViolations(page);

  await page.getByRole("button", { name: "Story-Bibel" }).click();
  await expect(page.getByRole("heading", { level: 2, name: "Story-Bibel" })).toBeVisible();
  await expectNoSeriousAccessibilityViolations(page);
});

test("Speicherfehler bleibt hörbar sichtbar und eine neue Bearbeitung wird erneut gespeichert", async ({
  page,
}) => {
  const failedText = "Der erste Speicherflug scheitert kontrolliert.";
  const recoveredText = " Der zweite landet sicher.";

  await page.goto("/?kraqelFailNextSave=1");
  await page.getByLabel("Projektname").fill("Fehlerlabor ohne echte Manuskripte");
  await page.getByRole("button", { name: "Projekt anlegen" }).click();

  const editor = page.getByRole("textbox", { name: "Text von Szene 1" });
  await editor.fill(failedText);
  const saveStatus = page.getByRole("status");
  await expect(saveStatus).toHaveText("Speichern fehlgeschlagen", { timeout: 5_000 });
  await expect
    .poll(() =>
      page.evaluate(
        (text) => localStorage.getItem("kraqel-e2e-command-state-v1")?.includes(text) ?? false,
        failedText,
      ),
    )
    .toBe(false);

  await editor.pressSequentially(recoveredText);
  await expect(saveStatus).toHaveText("Ungespeichert");
  await expect(saveStatus).toHaveText("Gespeichert", { timeout: 5_000 });
  await expect
    .poll(() =>
      page.evaluate(
        (text) => localStorage.getItem("kraqel-e2e-command-state-v1")?.includes(text) ?? false,
        failedText + recoveredText,
      ),
    )
    .toBe(true);
});

test("reduzierte Bewegung lässt den Tastatur-Kernablauf ohne wirksame Animationen bedienbar", async ({
  page,
}) => {
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.goto("/");
  await expect
    .poll(() => page.evaluate(() => matchMedia("(prefers-reduced-motion: reduce)").matches))
    .toBe(true);

  const projectName = page.getByLabel("Projektname");
  await tabTo(page, projectName);
  await page.keyboard.type("Die entschleunigte Republik");
  const createProject = page.getByRole("button", { name: "Projekt anlegen" });
  await tabTo(page, createProject);
  await page.keyboard.press("Enter");

  const storyBible = page.getByRole("button", { name: "Story-Bibel" });
  await tabTo(page, storyBible);
  await page.keyboard.press("Enter");
  await expect(storyBible).toHaveAttribute("aria-current", "page");

  const movingElements = await page.locator("body, body *").evaluateAll((elements) =>
    elements
      .filter((element) => {
        const style = getComputedStyle(element);
        const hasDuration = (value: string) =>
          value.split(",").some((part) => Number.parseFloat(part) > 0);
        return hasDuration(style.animationDuration) || hasDuration(style.transitionDuration);
      })
      .map((element) => `${element.tagName.toLowerCase()}.${element.className}`),
  );
  expect(movingElements).toEqual([]);
});
