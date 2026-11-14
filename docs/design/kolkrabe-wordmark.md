# KQ-056: KolkrabeofDoom am Wortzeichen

Stand: 14. November 2026. Issue #109.

Die ausdrücklich ausgewählte dritte Entwurfsrichtung, eine alte Schwarzweiß-Lithografie, ist als
freigestellte Autorenvignette in der Startseite eingebettet. Erkennungsmerkmale sind Baskenmütze,
tief sitzende Sonnenbrille, skeptischer Blick, Rollkragen, Füllfederhalter und Tintenfass. Feine
Texturen sind für die kleine Darstellung vereinfacht.

Das Rasterasset `apps/desktop/public/kolkrabe-author.png` wurde mit Imagegen anhand der
bereitgestellten Figurenreferenz und des ausgewählten Entwurfs erstellt. Es ist ein lokales PNG mit
echtem Alpha-Kanal, 1281 × 1227 Pixel. Referenzbilder und temporäre Generierungsdateien gehören
nicht zum App-Bundle. Zur Laufzeit findet kein Bildabruf oder Generierungszugriff auf das Netz
statt.

Das Wortzeichen bleibt eine echte H1. Die begleitende Grafik ist dekorativ und wird für Screenreader
ausgeblendet. Intrinsische Bildabmessungen vermeiden nachträgliche Layoutsprünge.
Desktopdarstellung: maximal 208 CSS-Pixel Breite; bei schmalen Ansichten wird die Figur verkleinert.
Die vorhandenen Projektaktionen bleiben separat bedienbar.

Visuell geprüft bei 1180, 760, 380 und 320 Pixel Ansichtsbreite: keine horizontale Überbreite, keine
Überschneidung von Wortzeichen und Figur. Die transparente Vignette sitzt ohne weißen
Hintergrundkasten auf dem bestehenden Startseitenverlauf.

Zusätzlich bestanden: Frontend-Build und vollständiges `pnpm check` einschließlich
Rust-Workspace-Tests. Die Einbindung ist auch in der laufenden nativen Windows-App visuell geprüft.
