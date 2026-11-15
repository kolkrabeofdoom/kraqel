# Native M3-Restprüfungen: Netzverkehr, Bedienung und CRUD

15. November 2026 · KQ-068 / [Issue #133](https://github.com/kolkrabeofdoom/kraqel/issues/133).
    Basis: PR #132 mit grüner Windows-/Linux-CI und Dependency Review zusammengeführt,
    `c7629c3e19454b9633e8ff88ccd86ef454ff19be`. Abgleich: Pflichtenheft Revision 1 §§1/3/4/6 V.1/8
    M3/10 und [F08-/M3-Matrix](F08_M3_ACCEPTANCE_MATRIX.md). Der angenommene Drehbuchvertrag wird
    hier nicht implementiert.

**Ergebnis: Teilprüfungen bestanden; keine vollständige F08-/M3-Freigabe.** Der unabhängige Netztest
findet öffentlichen WebView2-Verkehr. Die noch ausstehenden manuellen Schritte werden unten
ausdrücklich getrennt gehalten. Ein angelegter Prüfplan und ein grüner Rust-Test ersetzen keine
tatsächlich ausgeführte GUI- oder Hörprüfung.

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

## Native CRUD-Bedienung und unabhängige Integritätskontrolle

| Prüffall                | Tatsächlicher Nachweis                                                                                                                                                                                |
| ----------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Anlage/Lesen            | PNG `crud-a.png` über den echten Dialog importiert, Titel „CRUD Variante A“, Tags „Test, Tusche“, Vorschau/Format/Maße sichtbar                                                                       |
| Metadaten               | Titel nativ auf „CRUD Variante A geändert“ geändert und gespeichert; Statusmeldung sichtbar                                                                                                           |
| Szenenlink              | Nativ mit „Szene 1“ verknüpft; Chip und benannter Linkzielwähler vorhanden                                                                                                                            |
| Ungültiger Ersatz       | Synthetische `invalid.png` enthält Text statt Bild; Ersatz abgelehnt und verständliche Fehlermeldung angezeigt                                                                                        |
| Gültiger Ersatz         | `crud-b.webp` über den echten Dialog ersetzt; Format jetzt WebP, Titel/Tags und Szenenlink erhalten                                                                                                   |
| Eigenständige Variante  | `crud-c.jpg` nativ als „CRUD Variante B“ zusätzlich importiert; beide Bilder unabhängig vorhanden. Kein Variantenvergleich oder Elternmodell behauptet                                                |
| Profil                  | Kernmerkmale und Ausschluss nativ angelegt; anschließend „Gefieder: tiefschwarz“ gespeichert                                                                                                          |
| Profilfehler            | Doppelte Schlüssel „Augen“ nativ abgelehnt; gültige Korrektur danach erfolgreich gespeichert                                                                                                          |
| Stilvorlage             | „CRUD Tusche“ mit Regeln und Geltungsbereich Figuren/Orte nativ angelegt, anschließend auf „CRUD Tusche geändert“ bearbeitet                                                                          |
| SQLite-/Dateikontrolle  | Unabhängige Nur-Lese-Abfrage: zwei Assets (WebP/JPEG), ein Szenenlink, ein Profil, ein Stilpreset, eine Figur; Original-/Thumbnailhashes lokal inventarisiert                                         |
| Kanonischer Bildwechsel | Noch offen: Der Windows-Bedienhelfer verweigert Klicks auf das WebView2-Auswahl-Unterfenster. Konkreter manueller Wechsel A → Speichern → B → Speichern angefragt; ohne Rückmeldung kein GUI-Nachweis |
| GUI-Löschung            | Noch offen: Aktionsbestätigung für genau Bild B, Profil von CRUD Rabe und Stil „CRUD Tusche geändert“ angefragt. Keine GUI-Löschung ohne diese Bestätigung                                            |
| Prozessneustart         | In KQ-067 für den großen Korpus belegt; für die neuen CRUD-Änderungen nach Abschluss der manuellen Schritte erneut prüfen                                                                             |

Der neue reguläre Rust-Integrationstest
`native_m3_crud_keeps_files_identity_and_references_consistent` durchläuft reale Commands, SQLite
und Dateisystem, jeweils mit neuem Datenbanköffnen. Er prüft im 100-/50-Korpus: hashgleicher Import
überschreibt nichts, Metadatenänderung erhält Hash/Links, ungültiger und bereits vorhandener Ersatz
verändern das Inventar nicht, gültiger Ersatz erhält ID/Metadaten/Links und kanonische Referenz,
alte Dateien verschwinden. Ent-/Verknüpfung, tatsächliche Assetlöschung samt Dateien und Freigabe
des kanonischen Verweises, alternatives kanonisches Bild, Stiländerung mit stabiler ID,
Profil-/Stillöschung, erhaltene Figur und anderes Asset, `foreign_key_check` und anschließende
Snapshot-Erzeugung sind ebenfalls bestanden. Dies ist **native Kernabdeckung A**, keine
nachträgliche GUI-Abnahme der offenen Zeilen.

## Screenreader und gefundene Bedienlücken

Windows-Sprachausgabe wurde aus der vorhandenen Installation gestartet. Nutzer-Rückmeldung:
„Sprachausgabe funktioniert bestens“. Das ist eine allgemeine Funktionsbestätigung während der
anfänglichen Medienprüfung, keine einzeln zugeordnete Hörliste sämtlicher späterer Fehler- und
Exportpfade. Für Profil-/Stilspeicherung und die später ausgelösten Fehler ist eine konkrete
Hörrückmeldung angefragt; die umfassende F08-/M3-Hörabnahme bleibt offen.

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
großen JavaScript-Chunk bleibt unverändert. Der Coverage-Prozess bleibt wegen der angefragten
manuellen Schritte geöffnet; sein Executable wird nicht im laufenden Betrieb durch einen regulären
Build ersetzt.

Offen bleiben die oben benannten manuellen GUI-/Hörschritte, erneuter CRUD-Prozessneustart und der
separate Netzbefund #134. Das Pflichtenheft verlangt außerdem die weiterhin fehlende
Drehbuchimplementierung und weitere getrennte F01–F09-/M5-Abnahmen. Keine Gesamtfreigabe.
