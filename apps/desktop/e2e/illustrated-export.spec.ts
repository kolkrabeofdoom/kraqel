import { expect, test } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";
test("explizite Szenenbilder ergeben getrennte Pakete mit Alttext und neutralem Fehler", async ({
  page,
}) => {
  await page.goto("/");
  await page.getByLabel("Projektname").fill("Szenenbildprobe");
  await page.getByRole("button", { name: "Projekt anlegen" }).click();
  await page.getByRole("button", { name: "Medien", exact: true }).click();
  await page.getByLabel("PNG, JPEG oder WebP auswählen").setInputFiles({
    name: "private.png",
    mimeType: "image/png",
    buffer: Buffer.from(
      "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII=",
      "base64",
    ),
  });
  await page.getByLabel("Titel", { exact: true }).fill("Szenenbild");
  await page.getByRole("button", { name: "Bild lokal importieren" }).click();
  await page
    .locator(".media-card")
    .getByRole("button", { name: "Mit aktiver Szene verknüpfen" })
    .click();
  await page.getByRole("button", { name: "Export", exact: true }).click();
  const action = page.getByRole("button", { name: "HTML mit Bildern erzeugen" });
  await expect(action).toBeDisabled();
  await page.getByRole("button", { name: "Bildauswahl laden", exact: true }).click();
  await page.getByRole("checkbox", { name: "Szenenbild", exact: true }).check();
  await page.getByLabel("Alternativtext für Szenenbild").fill("Ein Rabe am Schreibtisch");
  await page.getByLabel("Bildunterschrift für Szenenbild").fill("Nur dieses ausgewählte Bild");
  await action.click();
  await expect(page.getByRole("status")).toContainText("Bildpaket fertig: mit-bildern/html-test-1");
  await expect(page.getByRole("status")).toContainText("vollständigen Ordner");
  const request = await page.evaluate(() =>
    JSON.parse(localStorage.getItem("kraqel-e2e-illustrated-request")!),
  );
  expect(request.placements).toHaveLength(1);
  expect(request.placements[0].altText).toBe("Ein Rabe am Schreibtisch");
  expect(JSON.stringify(request)).not.toContain("private.png");
  await page.setViewportSize({ width: 360, height: 900 });
  const results = await new AxeBuilder({ page }).analyze();
  expect(
    results.violations.filter((v) => v.impact === "serious" || v.impact === "critical"),
  ).toEqual([]);
  await page.getByRole("button", { name: "Markdown mit Bildern erzeugen" }).click();
  await expect(page.getByRole("status")).toContainText("markdown-test-2");
  await page.evaluate(() => localStorage.setItem("kraqel-e2e-illustrated-failure", "1"));
  await action.click();
  await expect(page.getByRole("alert")).toContainText("nicht erzeugt");
  await expect(page.getByRole("alert")).not.toContainText("private path");
  await page.getByRole("button", { name: "Bildauswahl neu laden" }).click();
  await expect(action).toBeDisabled();
  await expect(page.getByRole("checkbox", { name: "Szenenbild", exact: true })).not.toBeChecked();
});
