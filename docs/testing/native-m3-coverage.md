# Native M3-Restprüfungen: Netzverkehr, Bedienung und CRUD

15. November 2026 · KQ-068 / [Issue #133](https://github.com/kolkrabeofdoom/kraqel/issues/133).
    Basis: PR #132 mit grüner Windows-/Linux-CI und Dependency Review zusammengeführt,
    `c7629c3e19454b9633e8ff88ccd86ef454ff19be`. Abgleich: Pflichtenheft Revision 1 §§1/3/4/6 V.1/8
    M3/10 und [F08-/M3-Matrix](F08_M3_ACCEPTANCE_MATRIX.md). Der angenommene Drehbuchvertrag wird
    hier nicht implementiert.

**Ergebnis: Nativer Medien-/Profil-/Stil-CRUD einschließlich Löschung und Neustart bestanden; keine
vollständige F08-/M3-Freigabe.** Der unabhängige Netztest findet öffentlichen WebView2-Verkehr. Die
ergänzten manuellen Schritte und ihre tatsächlichen Belege stehen unten; ein angelegter Prüfplan und
ein grüner Rust-Test ersetzen keine GUI- oder Hörprüfung.

## Umgebung und reproduzierbarer Umfang

Windows 11 Pro `10.0.26200`, WebView2 `157.0.4341.60`, Rust `1.100.0`, vorhandene PowerShell 7.6.9.
Gebündelte native Tauri-Debug-App mit React, echten Dateidialogen, Rust und SQLite. Kein
Browser-Command-Mock für die nativen Abläufe. Die getrennten Browser-E2E verwenden wie bisher den
Mock und werden deshalb gesondert bewertet.

`tauri.m3-coverage-test.conf.json`: Produkt **KRAQEL M3 Coverage Test**, ID
`org.kraqel.desktop.m3-coverage-test`, Datenwurzel `$TEMP/kraqel-m3-acceptance-coverage-20261115`.
Reguläre App-Projekte bleiben unberührt. Der testexklusive vorhandene Rust-Seed erzeugt erneut 100
Szenen/50 echte Bilder. Zusätzlich wird über die native App das frische Projekt **M3 CRUD Hörprobe**
mit der Figur **CRUD Rabe** angelegt. Seine UUID ist `6b795881-e585-43dd-b1be-aacc30c9a803`.

Geprüfter Coverage-Build, vor der anschließenden Korrektur des Startseiten-Telemetriehinweises:
SHA-256 `dc4152e963c5a35e92e1a36d713eb4de70b2b4690f61c548751497ef76b835ac`. Die nachfolgende
Footeränderung ist keine Wiederholung dieses nativen Laufs. SQLite 6, Command/Editor/DOCX v1 und
Snapshot v3 bleiben unverändert. Keine neue Laufzeitabhängigkeit.

Die bestätigten GUI-Löschungen erfolgen noch im ersten Build. Danach wird dieser Prozess vollständig
beendet, der aktuelle Stand `291788b` einschließlich des korrigierten Footers neu gebündelt und als
neuer Coverage-Prozess gestartet. SHA-256:
`bcc3697af82b62dbf757732869858e6e27c60b1f4c9b76b56a4cc264f7f42ae8`. Neustartansichten und alle vier
folgenden Tastatur-Exporte verwenden diesen zweiten Build.

## Native CRUD-Bedienung und unabhängige Integritätskontrolle

| Prüffall                | Tatsächlicher Nachweis                                                                                                                                        |
| ----------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Anlage/Lesen            | PNG `crud-a.png` über den echten Dialog importiert, Titel „CRUD Variante A“, Tags „Test, Tusche“, Vorschau/Format/Maße sichtbar                               |
| Metadaten               | Titel nativ auf „CRUD Variante A geändert“ geändert und gespeichert; Statusmeldung sichtbar                                                                   |
| Szenenlink              | Nativ mit „Szene 1“ verknüpft; Chip und benannter Linkzielwähler vorhanden                                                                                    |
| Ungültiger Ersatz       | Synthetische `invalid.png` enthält Text statt Bild; Ersatz abgelehnt und verständliche Fehlermeldung angezeigt                                                |
| Gültiger Ersatz         | `crud-b.webp` über den echten Dialog ersetzt; Format jetzt WebP, Titel/Tags und Szenenlink erhalten                                                           |
| Eigenständige Variante  | `crud-c.jpg` nativ als „CRUD Variante B“ zusätzlich importiert; beide Bilder unabhängig vorhanden. Kein Variantenvergleich oder Elternmodell behauptet        |
| Profil                  | Kernmerkmale und Ausschluss nativ angelegt; anschließend „Gefieder: tiefschwarz“ gespeichert                                                                  |
| Profilfehler            | Doppelte Schlüssel „Augen“ nativ abgelehnt; gültige Korrektur danach erfolgreich gespeichert                                                                  |
| Stilvorlage             | „CRUD Tusche“ mit Regeln und Geltungsbereich Figuren/Orte nativ angelegt, anschließend auf „CRUD Tusche geändert“ bearbeitet                                  |
| SQLite-/Dateikontrolle  | Unabhängige Nur-Lese-Abfrage: zwei Assets (WebP/JPEG), ein Szenenlink, ein Profil, ein Stilpreset, eine Figur; Original-/Thumbnailhashes lokal inventarisiert |
| Kanonischer Bildwechsel | Manueller Wechsel A → B durch Nutzerrückmeldung bestätigt; B vor Löschung zusätzlich in GUI und SQLite nachgewiesen                                           |
| GUI-Löschung            | Nach konkreter Nutzerfreigabe alle drei Datensätze über die jeweilige zweite App-Bestätigung gelöscht; Dateien und Referenzen unabhängig geprüft              |
| Prozessneustart         | Coverage-Prozess beendet, aktueller Build neu gestartet; gelöschte Daten bleiben weg, Bild A, Tags, Link und Figur bleiben erhalten                           |

Der Nutzer bestätigt den angefragten manuellen Bildwechsel A → Speichern → B → Speichern zusammen
mit der Profil-/Stil-/Fehler-Hörprüfung mit „alles gut“. Bild B ist vor der Löschung zusätzlich
unmittelbar in GUI und SQLite als kanonisches Asset nachgewiesen. Anschließend bestätigt der Nutzer
ausdrücklich die drei benannten GUI-Löschungen. Jede wird tatsächlich über die zweistufige
App-Aktion ausgeführt; Erfolgsstatus und leere Profil-/Stilansicht werden beobachtet. Ein neues
Vorherinventar berücksichtigt den vom Nutzer gespeicherten Bildwechsel.

Nach der Bildlöschung sind Original und Thumbnail von B entfernt, das Profil ist noch vorhanden und
sein kanonischer Verweis ist `NULL`. Erst danach werden Profil und Stil gelöscht. Unabhängige
Nur-Lese-Abfragen und SHA-256-Vergleiche bestätigen: Bild A samt sämtlichen Metadaten und beiden
Dateien, seine Tags, der Szenenlink, beide Dokumente und die Figur bleiben unverändert.
`integrity_check=ok`, keine Fremdschlüsselverletzung. Nach vollständigem Prozessneustart sind
dieselben Ergebnisse in GUI und SQLite vorhanden. Der
[bereinigte Prüfbericht](native-m3-crud-kq068.json) hält diese Ergebnisse fest; die vollständigen
synthetischen SQLite-/Dateiinventare bleiben lokal.

Der neue reguläre Rust-Integrationstest
`native_m3_crud_keeps_files_identity_and_references_consistent` durchläuft reale Commands, SQLite
und Dateisystem, jeweils mit neuem Datenbanköffnen. Er prüft im 100-/50-Korpus: hashgleicher Import
überschreibt nichts, Metadatenänderung erhält Hash/Links, ungültiger und bereits vorhandener Ersatz
verändern das Inventar nicht, gültiger Ersatz erhält ID/Metadaten/Links und kanonische Referenz,
alte Dateien verschwinden. Ent-/Verknüpfung, tatsächliche Assetlöschung samt Dateien und Freigabe
des kanonischen Verweises, alternatives kanonisches Bild, Stiländerung mit stabiler ID,
Profil-/Stillöschung, erhaltene Figur und anderes Asset, `foreign_key_check` und anschließende
Snapshot-Erzeugung sind ebenfalls bestanden. Dies ist **native Kernabdeckung A**, kein Ersatz für
eine GUI-Abnahme. Die oben dokumentierte tatsächliche GUI-Folge ergänzt ihn separat.

## Screenreader und gefundene Bedienlücken

Windows-Sprachausgabe wurde aus der vorhandenen Installation gestartet. Nutzer-Rückmeldung:
„Sprachausgabe funktioniert bestens“. Die spätere konkrete Frage zu Profil-/Stilerfolg und den
ausgelösten Bildersatz-/Profilfehlern beantwortet der Nutzer mit „alles gut“. Damit sind diese
gezeigten Ansagen manuell bestätigt. Für die danach ausgeführten Löschmeldungen und vier
Export-Erfolge ist eine gesonderte Hörrückmeldung angefragt; ohne Antwort wird keine entsprechende
Hörabnahme behauptet. Weitere F08-Randfälle und eine allgemeine
WCAG-/Dokument-Screenreader-Konformität werden nicht daraus abgeleitet.

Tatsächlich korrigiert:

- zusätzlicher Medien-Linkwähler mit eindeutigem zugänglichem Namen je Bild;
- benannte Medienartikel für die Unterscheidung mehrerer Bilder;
- dauerhaft vorhandene `role=status`-Regionen mit Erfolgen für Import, Metadaten, Ersatz,
  Verknüpfung/Entfernung, Bildlöschung, Profil-/Stilspeicherung und -löschung;
- Fehler der Medien-/Profilansichten als `role=alert`; zu großer Ersatz wird ausdrücklich gemeldet,
  statt kommentarlos ignoriert;
- Medien-Linkanlage während einer laufenden Operation gesperrt.

Native Status-/Fehlertexte wurden sichtbar geprüft; A11y-Tree und Browserassertionen belegen deren
technische Bereitstellung, nicht ihren Klang. Browser-E2E prüfen Import-Erfolg, benannte
Mediennavigation, fehlerhaften Import mit anschließendem Erfolg sowie Profilfehler, Erhaltung
gespeicherter Werte und bestätigte Profil-/Stillöschung. Die bestehenden KQ-033-/KQ-034-Hörnachweise
bleiben historische, begrenzte Belege.

## Vier Bildexporte nach den Löschungen und dem Neustart

Im neuen Coverage-Prozess wird nur Bild A als direktes Szenenbild angeboten. Über **Tab und
Leertaste** wird es ausdrücklich ausgewählt; anschließend werden Alternativtext und Bildunterschrift
per Tastatur eingegeben. Der sichtbare Fokus erreicht nacheinander alle vier Exportknöpfe; jeder
wird mit **Eingabetaste** ausgelöst. Alle vier Erfolgsmeldungen zeigen ein Bild und ein Vorkommen.
Die zuvor gelöschte Variante B wird weder angeboten noch exportiert.

Die vier tatsächlich erzeugten Pakete werden unabhängig mit Python-Standardbibliothek, Pillow und
pypdf rückgelesen. Alle Manifeste behalten den erwarteten Quellhash, die genaue Bildauswahl,
Position, Alttext und Bildunterschrift; sämtliche Ausgabe-PNGs entsprechen dem WebP-Original
pixelgenau. DOCX enthält dieselben PNG-Bytes als internen ZIP-Part und den vereinbarten Alttext. PDF
enthält einen passenden RGB-Bildstrom, eine Seite und alle vier erwarteten eingebetteten Schriften;
die gerenderte Seite ist ohne Überlagerungen oder abgeschnittene Beschreibungen geprüft. Das ist
eine Einzelkorpusprüfung, keine neue Office-GUI- oder universelle Layoutabnahme. Hashes stehen im
[Prüfbericht](native-m3-crud-kq068.json).

Der Markdown-PNG-Pfad ist auf diesem Rechner exakt 260 Zeichen lang. Für das unabhängige
Python-Rücklesen wird deshalb das Windows-NT-Langpfadpräfix verwendet; die Datei ist tatsächlich
vorhanden und korrekt. Keine OS-Richtlinie wurde geändert. Der Punkt am Ende der
Markdown-Bildunterschrift ist vertragsgemäß maskiert. Ein einfacher unmaskierter Textvergleich wäre
hier kein sinnvoller Dateinachweis.

## Unabhängiger Windows-Netztest

Der nicht erhöhte ETW-Start wurde von Windows ausdrücklich abgewiesen. Die UAC-Bedienung wurde dem
Nutzer überlassen und von ihm bestätigt. Windows PowerShell 5 verweigerte die Skriptausführung; der
vorhandene PowerShell-7-Host mit seiner bestehenden Richtlinie führte den Helfer aus. Keine
ExecutionPolicy-, Firewall-, Datenschutz- oder Sicherheitseinstellung wurde geändert.

Aufzeichnung mit `Microsoft-Windows-Kernel-Network` (IPv4/IPv6, TCP/UDP) und
`Microsoft-Windows-Kernel-Process`, eindeutige eigene Sitzung, maximal 20 Minuten/128 MiB. Beginn
`2026-11-15T11:42:07Z`; Root-PID `8676` wurde erst danach aus dem erwarteten Executable gestartet.
Der Rust-Prozess und seine WebView2-Nachfahren werden über Prozessgenerationen und
Start-/Stop-Zeiten zugeordnet. Zeitweilige Nachfahren und PID-Wiederverwendung dürfen nicht mit
einer bloßen aktuellen Prozessliste vermischt werden.

Der abgeschlossene 20-Minuten-Messabschnitt meldet null verlorene Ereignisse und Puffer. Positive
Kontrollen des Helfers erzeugen ausschließlich Loopback-TCP/-UDP für beide Adressfamilien; die
endgültige Auswertung verlangt alle vier tatsächlich im ETW vorhandenen Kontrollen. Die Rohdateien
enthalten auch fremde Systemmetadaten und bleiben deshalb ausschließlich lokal außerhalb von Git.

**Befund:** App-eigene WebView2-Unterprozesse verbinden sich mit öffentlichen Adressen auf Port
80/443, ohne dass ein Provider aufgerufen wird. Der Rust-Hauptprozess zeigt im ausgewerteten
Ausschnitt keine eigenen Netzereignisse. Der geforderte Nachweis ohne unerwarteten Netzverkehr ist
damit **nicht bestanden**. Der [bereinigte Endbericht](native-m3-network-kq068.json) belegt vier von
vier positiven Kontrollen, null verlorene Ereignisse/Puffer und null unzuordenbare Netzereignisse.
Er enthält 262 öffentliche Transportereignisse ausschließlich von WebView2: 15 Connect-, 100 Send-,
130 Receive- und 17 Disconnect-Ereignisse; zusätzlich sechs Loopback-Ereignisse. Das sind keine 262
HTTP-Anfragen. Ein Root-Prozess, ein Conhost- und 20 WebView2-Prozesslebenszeiten wurden zugeordnet.
Die ETL-Datei hat 18.874.368 Bytes; ihr SHA-256 steht im Bericht.

Die App blieb für die angefragte manuelle Bedienung über das Messende hinaus geöffnet. Deshalb
verweigert der Auswerter zusätzlich eine grüne Gesamtmessung mit
`process-tree-not-fully-stopped-within-capture` (Exitcode 1). Der konkrete öffentliche Verkehr ist
trotzdem positiv beobachtet; eine vollständige, netzstille App-Lebenszeit ist ausdrücklich nicht
belegt. Der Zeitstempel `12:02:15Z` aus dem ursprünglichen Helfer bezeichnet das Ende inklusive
Konvertierung, nicht sekundengenau den ETW-Stop. Der versionierte Helfer trennt diese beiden
Zeitpunkte künftig.

Diese Daten enthalten keine HTTP-Inhalte und belegen weder Telemetriezweck noch einen
Manuskript-Upload. Microsoft dokumentiert eigene optionale und erforderliche
[WebView2-Diagnosedaten](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/data-privacy).
Das liefert Kontext, keine Zweckzuordnung der beobachteten Verbindungen. Browserflags sind laut
[Microsofts Dokumentation](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/webview-features-flags)
kein belastbarer Produktionsfix. Keine unsicheren Flags oder OS-Änderungen wurden eingesetzt, um den
Befund zu verdecken.

Der nächste konkrete Scope ist deshalb
[KQ-069 / Issue #134](https://github.com/kolkrabeofdoom/kraqel/issues/134): Ursache, unterstützte
App-Maßnahmen und notwendige fachliche Entscheidung zur Laufzeitgrenze. Startseite und README
präzisieren die bisher zu weite Telemetrieaussage. Keine pauschale Zusage vollständiger Netzstille;
keine Abschwächung der Pflichtenheft-Abnahme durch Umbenennung.

Die fachlichen CRUD-Aktionen funktionierten ohne konfigurierten Provider. Eine physische
Netztrennung, sämtliche Brokerwege, echte Cloudprovider und der vollständige KQ-067-Gesamtlauf
wurden in dieser Messung nicht neu abgenommen. Q-01/Q-02 bleiben offen.

Werkzeuge und Grenzen: [ETW-Prüfhelfer](../../scripts/m3-network/README.md).

## Qualität und verbleibende Arbeit

Gezielte Browserprüfungen: drei bestanden. Gesamte Browser-E2E: **27 bestanden**. Der neue native
Rust-CRUD-Test ist bestanden. Die ETW-Auswertung wird durch **12 synthetische Tests** einschließlich
PID-Wiederverwendung, verlorenen Puffern, fehlenden Kontrollen und unzuordenbaren Fehlerereignissen
abgesichert und in beiden CI-Betriebssystemen ausgeführt.

`pnpm check` ist bestanden: **163 Rust- und 53 TypeScript-Tests**, sieben regulär ignorierte
explizite QA-Helfer, rustfmt/Clippy/ESLint/TypeScript/Prettier grün. Abschließende Formatprüfung,
Python-Tests und erneute 27 Browser-E2E ebenfalls bestanden. Der isolierte gebündelte
Coverage-Debug-Build und der abschließende Frontend-Build bestanden; die übliche Vite-Warnung zum
großen JavaScript-Chunk bleibt unverändert. Der Coverage-Prozess wurde für den belegten Neustart
vollständig beendet und der aktuelle Stand erfolgreich neu gebündelt. Nach der Exportprüfung wurde
auch die zweite Coverage-Instanz geschlossen und der reguläre Debug-Build erfolgreich
wiederhergestellt. Die produktive App wurde dabei nicht gestartet.

Offen bleiben die angefragte abschließende Lösch-/Export-Hörrückmeldung, weitere allgemeine
assistive F08-Randfälle und der separate Netzbefund #134. GUI-CRUD und Prozessneustart sind im
dokumentierten Korpus bestanden. Das Pflichtenheft verlangt außerdem die weiterhin fehlende
Drehbuchimplementierung und weitere getrennte F01–F09-/M5-Abnahmen. Keine Gesamtfreigabe.
