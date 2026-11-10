import { expect, test } from "@playwright/test";

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => localStorage.clear());
});

test("Story-Bibel verbindet Suche, gerichtete Beziehung und Szenen-Rückverweis", async ({
  page,
}) => {
  await page.goto("/");
  await page.getByLabel("Projektname").fill("Das Archiv am Ende der Welt");
  await page.getByRole("button", { name: "Projekt anlegen" }).click();
  await page.getByRole("button", { name: "Story-Bibel" }).click();

  const createForm = page.locator(".create-entity-form");
  await createForm.getByLabel("Name").fill("Mara");
  await createForm.getByRole("button", { name: "+ Eintrag" }).click();
  await expect(page.getByRole("heading", { level: 3, name: "Mara" })).toBeVisible();

  await page.getByRole("button", { name: "+ Eigenschaft" }).click();
  await page.getByLabel("Eigenschaftsname").fill("Rolle");
  await page.getByPlaceholder("Wert").fill("Navigatorin");
  await page.getByRole("button", { name: "Eintrag speichern" }).click();

  await createForm.getByLabel("Art").selectOption("location");
  await createForm.getByLabel("Name").fill("Archiv der Möwen");
  await createForm.getByRole("button", { name: "+ Eintrag" }).click();
  await expect(page.getByRole("heading", { level: 3, name: "Archiv der Möwen" })).toBeVisible();

  const entityList = page.locator(".entity-list");
  const search = page.getByRole("searchbox", { name: "Suche" });
  await search.fill("Navigatorin");
  await expect(page.getByText("1 Einträge", { exact: true })).toBeVisible();
  await expect(entityList.getByRole("button", { name: /Mara/ })).toBeVisible();
  await expect(entityList.getByRole("button", { name: /Archiv der Möwen/ })).toHaveCount(0);
  await search.fill("");

  await entityList.getByRole("button", { name: /Mara/ }).click();
  await page.getByLabel("Beziehungsart").fill("bewacht");
  await page.getByLabel("Ziel").selectOption({ label: "Archiv der Möwen" });
  await page.getByRole("button", { name: "Beziehung anlegen" }).click();
  await expect(page.locator(".relation-list")).toContainText(
    "Ausgehend: Mara → bewacht → Archiv der Möwen",
  );

  await entityList.getByRole("button", { name: /Archiv der Möwen/ }).click();
  await expect(page.locator(".relation-list")).toContainText(
    "Eingehend: Mara → bewacht → Archiv der Möwen",
  );

  await entityList.getByRole("button", { name: /Mara/ }).click();
  await page.getByRole("button", { name: "Mit „Szene 1“ verknüpfen" }).click();
  const backlink = page.locator(".backlink-list").getByRole("button", { name: "Szene 1" });
  await expect(backlink).toBeVisible();
  await backlink.click();

  await expect(page.getByRole("button", { name: "Manuskript" })).toHaveAttribute(
    "aria-current",
    "page",
  );
  await expect(page.getByRole("textbox", { name: "Text von Szene 1" })).toBeVisible();
});
