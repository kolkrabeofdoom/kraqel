# KQ-069: Lokale Brokerpaket-Updates für Kontrollschema 4

Stand: 17. November 2026. Native Vorschau mit synthetischen Daten, mittleres Prüfniveau.
[Bereinigter Nachweis](native-schema4-package-update.json),
[Kontroll-/Paketvertrag](../data-format/native-product-control-v1.md). Striktes Datenschutzziel
unverändert, keine Produktfreigabe.

Ausgangspunkt `3e576a5f7d7e9e0dece31b2bd5da973f22639fca`, gesicherter Stand unter
`%TEMP%/kraqel-update4-build-20261117-01/`:

- Broker-SHA-256: `45542384ce42b73450453dce2f5a40b635b7b8552055de25d60c147fc21dd360`.
- Kandidat-SHA-256: `6748900defbbdb8df5e8e5ec0e611091cd1e6656880381a18e43e8755510bf90`.
- Gebundene Forschungs-App unverändert:
  `6e56a51ed2e913a878a7935e039bd69628a49be25f0fbab7d02fce3a72fe8c70`.

Schema 4 erlaubt jetzt lokale `prepare`-/`activate`-/`abort`-Übergänge unter derselben exklusiven
Kontrollsperre und dem bisherigen Jobleerstandsguard. Der erwartete vollständige Datei-SHA bleibt
die ausdrückliche lokale Freigabe. Alte Köpfe, aktive Jobs und früher aktivierte Programmhashes
bleiben gesperrt. Pending blockiert Sitzung/Recovery/Cleanup, bis Aktivierung oder ausdrücklicher
Abbruch abgeschlossen ist. Die bekannte Schema-2-Politik bleibt unverändert; Schema 3 erlaubt
weiterhin keine Updates.

Für Schema 4 muss der Kandidat außerdem genau eine gültige `.kqmeta`-PE-Section besitzen:
initialisierte, lesbare, nicht schreibbare/ausführbare/verwerfbare Metadaten, exakt 32 Bytes
Version-1-Vertrag. Dieser deklariert Lesen der gesamten Kontrollhistorie 1–4, Startticket 3 und
Profil-Cleanup-Vertrag 1; reservierte Bytes null. Der begrenzte Parser liest ausschließlich Header,
höchstens 96 Sectionheader und die Metadaten aus der gepinnten lokalen Datei. Keine Ausführung,
Bibliotheksladung oder Netzprüfung des Kandidaten vor Aktivierung. Prüfung vor Pending sowie erneut
an der geschützten Paketkopie vor Aktivierung. Unbekannte/fehlende Deklaration, falsche Architektur,
unzulässige Bereiche/Flags oder mehrdeutige Sections werden abgewiesen.

Die Metadaten sind eine **Kompatibilitätsdeklaration**, keine Herausgeberauthentisierung oder
Attestierung beliebigen Programmverhaltens. Hashfreigabe und Kontroll-MAC binden die konkrete lokale
Entscheidung und die Paketbytes. Keine Downloads, Zertifikate oder automatischen Updates. Diese
Stufe wechselt das **Brokerpaket**; die gebundene App, Sitzung, Aufgabe und Datenwurzel werden nicht
migriert oder ausgetauscht. Alte Vorschauablagen werden nicht übernommen/zurückgesetzt.

Gezielte Prüfung: drei neue aktive Tests für den begrenzten PE-Vertrag und Schema-4-Zustandsfolge,
insgesamt 51 schnelle Helfer-/Transporttests. Abbruch nach Manipulation, alte Köpfe, Pending und
Erhalt von Bindung/Schlüssel/Daten geprüft. Helfer-Clippy mit Warnungen als Fehler und rustfmt grün.

Ein nativer Ablauf mit 15 Child-Controllern besteht: zehn Erfolge und fünf erwartete Abweisungen.
Paket ohne Metadaten vor Pending abgewiesen, Recovery bei Pending und Aktivierung mit altem Kopf
gesperrt. Nach lokaler Aktivierung nur der verwaltete neue Broker autorisiert; alter Broker und
Rollback-Vorbereitung abgewiesen. Neuer Broker übernimmt denselben Task/Sitzungs-/Apphash und führt
Recovery/Replay aus. Zwei GUI-Starts, zwölf tatsächliche private Dateiverweigerungen, genau eine
erhaltene Projektkopie mit zwei Dokumenten; danach authentisierter Cleanup samt idempotenter
Wiederholung. Profil entfernt, Jobprozesse null.

Ein kleiner unabhängiger Nur-Lese-Audit bestätigt beide Paketmetadaten, SHA-256, elf MAC-Records,
Actorwechsel, unveränderte Installation/Schlüssel/Bindung, Abschlusskopf sowie SQLite-Integrität,
Fremdschlüssel und Unicode-Inhalt. Der synthetische Kandidat ist dieselbe EXE mit statischem
Overlay: anderer Hash, gleicher Code. Das prüft den realen lokalen Actorwechsel, nicht beliebige
Versions-/Appdatenmigration. Abort/Manipulation sind gezielt im Helfertest geprüft, nicht als
zusätzliche native GUI-Matrix.

Keine Workspace-Gesamtprüfung, kein vollständiger Appbuild oder neue große Crash-/ETW-Matrix. Keine
neuen Abhängigkeiten. Echte Vorschauablage/verwaltete EXE unverändert; synthetische Daten und
private Belege lokal erhalten, keine Schlüssel/rohen Kennungen veröffentlicht. Inzwischen liegt der
[eigene frische ETW-Beleg beider Brokerhashes](native-schema4-package-update-etw.md) einschließlich
hashbenannter verwalteter EXE vor; der vorherige Cleanup-Beleg bleibt an sein anderes Paar gebunden.
Der ursprüngliche funktionale JSON-Beleg bleibt getrennt ohne Netzmessung. Nächster begrenzter
Scope: Update-Abbruch-/Fehlerreconciliation unter derselben Kontrollsperre. Normale
Produktintegration, Apppaketmigration, weitere Crashpunkte und echte Datenmigration bleiben offen;
striktes Ziel unverändert.
