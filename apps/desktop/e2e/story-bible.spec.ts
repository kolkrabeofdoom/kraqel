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

test("Story-Bibel führt ein Dublett samt Referenzen bewusst in den geöffneten Eintrag", async ({
  page,
}) => {
  await page.goto("/");
  await page.getByLabel("Projektname").fill("Doppelte Raben");
  await page.getByRole("button", { name: "Projekt anlegen" }).click();
  await page.getByRole("button", { name: "Story-Bibel" }).click();

  const createForm = page.locator(".create-entity-form");
  const entityList = page.locator(".entity-list");
  await createForm.getByLabel("Name").fill("Mara");
  await createForm.getByRole("button", { name: "+ Eintrag" }).click();
  await page.getByRole("button", { name: "+ Eigenschaft" }).click();
  await page.getByLabel("Eigenschaftsname").fill("Rolle");
  await page.getByPlaceholder("Wert").fill("Navigatorin");
  await page.getByRole("button", { name: "Eintrag speichern" }).click();

  await createForm.getByLabel("Name").fill("Mara doppelt");
  await createForm.getByRole("button", { name: "+ Eintrag" }).click();
  await page.getByRole("button", { name: "+ Eigenschaft" }).click();
  await page.getByLabel("Eigenschaftsname").fill("Rolle");
  await page.getByPlaceholder("Wert").fill("Späherin");
  await page.getByRole("button", { name: "+ Eigenschaft" }).click();
  await page.getByLabel("Eigenschaftsname").nth(1).fill("Herkunft");
  await page.getByPlaceholder("Wert").nth(1).fill("Veyra");
  await page.getByRole("button", { name: "Eintrag speichern" }).click();

  await createForm.getByLabel("Art").selectOption("location");
  await createForm.getByLabel("Name").fill("Archiv");
  await createForm.getByRole("button", { name: "+ Eintrag" }).click();

  await entityList.getByRole("button", { name: /Mara doppelt/ }).click();
  await page.getByLabel("Beziehungsart").fill("bewacht");
  await page.getByLabel("Ziel").selectOption({ label: "Archiv" });
  await page.getByRole("button", { name: "Beziehung anlegen" }).click();
  await page.getByRole("button", { name: "Mit „Szene 1“ verknüpfen" }).click();

  await page.getByRole("button", { name: "Visuelle Profile" }).click();
  await page.getByLabel("Story-Bibel-Eintrag").selectOption({ label: "Figuren: Mara doppelt" });
  await page.getByLabel("Kernmerkmale").fill("Augen: grau\nHaltung: wach");
  await page.getByLabel("Unerwünschte Abweichungen").fill("Maske\nRegen");
  await page.getByRole("button", { name: "Profil speichern" }).click();
  await page.getByLabel("Story-Bibel-Eintrag").selectOption({ label: "Figuren: Mara" });
  await expect(page.getByLabel("Kernmerkmale")).toHaveValue("");
  await page.getByLabel("Kernmerkmale").fill("Augen: bernsteinfarben\nKleidung: Mantel");
  await page.getByLabel("Unerwünschte Abweichungen").fill("Regen\nHut");
  await page.getByRole("button", { name: "Profil speichern" }).click();

  await page.getByRole("button", { name: "Story-Bibel" }).click();

  await entityList.getByRole("button", { name: /^Mara\s+Figur$/ }).click();
  await page.getByLabel("Doppelten Eintrag wählen").selectOption({ label: "Mara doppelt" });
  await expect(page.locator(".merge-preview")).toContainText(
    "1 fehlende Eigenschaften werden ergänzt; 1 unterschiedliche Werte",
  );
  await expect(page.locator(".merge-preview")).toContainText(
    "Rolle: „Navigatorin“ bleibt statt „Späherin“",
  );

  page.once("dialog", (dialog) => dialog.accept());
  await page.getByRole("button", { name: "Dublette in „Mara“ zusammenführen" }).click();

  await expect(page.getByRole("status")).toContainText(
    "„Mara doppelt“ wurde in „Mara“ zusammengeführt.",
  );
  await expect(page.getByText("2 Einträge", { exact: true })).toBeVisible();
  await expect(entityList.getByRole("button", { name: /Mara doppelt/ })).toHaveCount(0);
  await expect(page.getByLabel("Eigenschaftsname").nth(1)).toHaveValue("Herkunft");
  await expect(page.getByPlaceholder("Wert").first()).toHaveValue("Navigatorin");
  await expect(page.locator(".relation-list")).toContainText("Mara → bewacht → Archiv");
  await expect(page.locator(".backlink-list")).toContainText("Szene 1");

  await page.getByRole("button", { name: "Visuelle Profile" }).click();
  await page.getByLabel("Story-Bibel-Eintrag").selectOption({ label: "Figuren: Mara" });
  await expect(page.getByLabel("Kernmerkmale")).toHaveValue(/Augen: bernsteinfarben/);
  await expect(page.getByLabel("Kernmerkmale")).toHaveValue(/Haltung: wach/);
  await expect(page.getByLabel("Kernmerkmale")).toHaveValue(/Kleidung: Mantel/);
  await expect(page.getByLabel("Unerwünschte Abweichungen")).toHaveValue(/Regen/);
  await expect(page.getByLabel("Unerwünschte Abweichungen")).toHaveValue(/Hut/);
  await expect(page.getByLabel("Unerwünschte Abweichungen")).toHaveValue(/Maske/);
});
