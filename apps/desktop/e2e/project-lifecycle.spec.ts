import { expect, test } from "@playwright/test";

test.beforeEach(async ({ page }) => {
  await page.goto("/");
  await page.evaluate(() => localStorage.clear());
  await page.reload();
});

test("Projektordner archiviert nur ausdrücklich gewählte Zusatzbestände", async ({ page }) => {
  await page.addInitScript(() =>
    Object.defineProperty(navigator, "userAgent", { value: "Windows" }),
  );
  await page.reload();
  await page.getByRole("button", { name: "Projektordner auswählen" }).click();
  const groups = page.getByRole("group", { name: "Separat mit Passphrase archivieren" });
  await expect(groups.locator("input:checked")).toHaveCount(0);
  await groups.getByRole("checkbox", { name: /Exporte/u }).check();
  await page.getByRole("button", { name: "Arbeitskopie anlegen" }).click();
  await expect(page.getByText("Arbeitskopie angelegt und Zusatzarchiv gespeichert.")).toBeVisible();
});

test("Projektarchiv wird aus einem aktuellen Snapshot exportiert", async ({ page }) => {
  await page.getByRole("button", { name: "Abenteuer ausprobieren" }).click();
  await page.getByRole("button", { name: "Versionen" }).click();

  await page.getByRole("button", { name: "Projektarchiv exportieren" }).click();

  await expect(page.getByText(/Projektarchiv exportiert: kraqel-projekt\.kraqel/u)).toBeVisible();
  await expect(page.getByText(/Format 3, [1-9][0-9]* Dateien/u)).toBeVisible();
});

test("Arbeitskopie wird im geöffneten Projekt ausdrücklich vorbereitet", async ({ page }) => {
  await page.getByRole("button", { name: "Abenteuer ausprobieren" }).click();
  await page.getByRole("button", { name: "Versionen" }).click();
  await page.getByRole("button", { name: "Übernahme vorbereiten" }).click();
  await expect(
    page.getByText(/Übernahme vorbereitet.*uebernahme-beispiel.*SHA-256/u),
  ).toBeVisible();
  await page.getByLabel("Ordnername im Eingang").fill("uebernahme-beispiel");
  await page.getByLabel("Separat bestätigter Inventar-SHA-256").fill("a".repeat(64));
  const includeMedia = page.getByLabel("Auch referenzierte Bilder und Vorschaubilder prüfen");
  await expect(includeMedia).not.toBeChecked();
  await includeMedia.check();
  await page.getByRole("button", { name: "Arbeitsdaten vergleichen" }).click();
  await expect(
    page
      .getByRole("status")
      .filter({ hasText: "Manuskript, Medieneinträge, Bild- und Vorschaudateien" }),
  ).toBeVisible();
  await page.getByText("Einzelne Projekteinträge übernehmen", { exact: true }).click();
  const loadItems = page.getByRole("button", { name: "Abweichende Einträge laden" });
  const itemSelect = page.getByLabel("Eintrag nach Kennung auswählen");
  const reviewItem = page.getByRole("button", { name: "Fassungen im isolierten Fenster öffnen" });
  await loadItems.click();
  const id = "66666666-6666-4666-8666-666666666666";
  await itemSelect.selectOption(id);
  await page.evaluate(() => localStorage.setItem("kraqel-e2e-item-stale", "1"));
  await reviewItem.click();
  await expect(
    page.getByRole("status").filter({ hasText: "Lade die Einträge erneut" }),
  ).toBeVisible();
  await loadItems.click();
  await expect(itemSelect).toHaveValue(id);
  await reviewItem.click();
  await page.getByRole("button", { name: "Rücknahme für diesen Eintrag sichern" }).click();
  await page.getByRole("button", { name: "Eintrag aktivieren" }).click();
  await expect(
    page.getByRole("status").filter({ hasText: "Die ausgewählte Fassung ist jetzt aktiv" }),
  ).toBeVisible();
  await page.getByRole("button", { name: "Eintragsübernahme zurücknehmen" }).click();
  await expect(
    page.getByRole("status").filter({ hasText: "Der vorherige Eintragsstand ist wieder aktiv" }),
  ).toBeVisible();
  const kind = page.getByRole("combobox", { name: /^Projektbereich/u });
  await kind.selectOption("mediaAsset");
  await page.getByRole("button", { name: "Medienkennungen laden" }).click();
  await itemSelect.selectOption("99999999-9999-4999-8999-999999999999");
  await reviewItem.click();
  await page.getByRole("button", { name: "Rücknahme für diesen Eintrag sichern" }).click();
  await expect(page.getByLabel("Lokaler Beleg für Aktivierung oder Rücknahme")).toHaveValue(
    "2".repeat(64),
  );

  await kind.selectOption("event");
  await loadItems.click();
  const eventId = "77777777-7777-4777-8777-777777777777";
  await itemSelect.selectOption(eventId);
  await reviewItem.click();
  await page.getByRole("button", { name: "Fehlende Zeitachse auswählen" }).click();
  await expect(kind).toHaveValue("timeline");
  await expect(itemSelect).toHaveValue("88888888-8888-4888-8888-888888888888");
  await reviewItem.click();
  await page.getByRole("button", { name: "Rücknahme für diesen Eintrag sichern" }).click();
  await page.getByRole("button", { name: "Eintrag aktivieren" }).click();
  await expect(kind).toHaveValue("event");
  await expect(itemSelect).toHaveValue(eventId);
  await expect(page.getByLabel("Lokaler Beleg für Aktivierung oder Rücknahme")).toHaveValue("");
  await reviewItem.click();
  await expect(
    page.getByRole("button", { name: "Rücknahme für diesen Eintrag sichern" }),
  ).toBeVisible();
  await page.getByText("Szenenstände zwischen zwei Rechnern vergleichen", { exact: true }).click();
  await page.getByRole("button", { name: "Szenenbasis sichern" }).click();
  await expect(page.getByRole("status").filter({ hasText: "Basis für 2 Szenen" })).toBeVisible();
  await page.getByLabel("SHA-256 der gemeinsamen Ausgangsbasis").fill("a".repeat(64));
  await page.getByLabel("SHA-256 des Stands vom anderen Rechner").fill("b".repeat(64));
  await page.getByRole("button", { name: "Szenen lesend vergleichen" }).click();
  await expect(page.getByText(/Auswahl erforderlich.*hier nicht vorhanden/)).toBeVisible();
  await expect(page.getByText(/Löschung im Abgleich gesperrt/)).toBeVisible();
  const choice = page.getByRole("checkbox", { name: /Titel und Inhalt/ });
  await expect(choice).toHaveCount(1);
  await expect(choice).not.toBeChecked();
  await choice.check();
  await page
    .getByLabel("Quellordner unter backups")
    .fill("uebernahme-44444444-4444-4444-8444-444444444444");
  await page.getByLabel("Separat bestätigter SHA-256 des Quellinventars").fill("d".repeat(64));
  await page.getByRole("radio", { name: /im isolierten Fenster/ }).check();
  await page
    .getByRole("button", { name: "Beide Texte im isolierten Win32-Fenster anzeigen" })
    .click();
  await expect(
    page.getByRole("status").filter({ hasText: "Lokale Fassung beibehalten." }),
  ).toBeVisible();
  await choice.check();
  await page.getByRole("button", { name: /Auswahl bestätigen/ }).click();
  await expect(page.getByRole("status").filter({ hasText: "uebernahme-ergebnis" })).toBeVisible();
  await expect(choice).toHaveCount(0);
  await page.getByRole("button", { name: "Vorherige Fassungen sichern" }).click();
  await expect(page.getByLabel("SHA-256 aus der lokalen Sicherung")).toHaveValue("1".repeat(64));
  await page.getByRole("button", { name: "Auswahl jetzt im Projekt aktivieren" }).click();
  await expect(
    page.getByRole("status").filter({ hasText: "im geöffneten Projekt aktiv" }),
  ).toBeVisible();
  await page.getByRole("button", { name: "Vorherige Szenen jetzt wiederherstellen" }).click();
  await expect(
    page.getByRole("status").filter({ hasText: "vorherigen Szenenfassungen sind wieder aktiv" }),
  ).toBeVisible();
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
