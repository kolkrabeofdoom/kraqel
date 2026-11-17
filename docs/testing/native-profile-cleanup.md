# KQ-069: Authentisierte Profilbereinigung

Stand: 17. November 2026. Native Windows-Vorschau, ausschließlich synthetische Daten.
[Bereinigter Nachweis](native-profile-cleanup.json),
[Kontrollvertrag](../data-format/native-product-control-v1.md). Striktes Datenschutzziel
unverändert.

Ausgangspunkt `de26f50ac2b48affd2f23993bfbdb9132ae9e50b`; gesicherter Prüfbuild unter
`%TEMP%/kraqel-cleanup-build-20261117-02/`:

- Broker-SHA-256: `b2c4f21f080337c231fcb54a71dae73777b71923f73c7961c37678da13cd7dbd`.
- Forschungs-App unverändert: `6e56a51ed2e913a878a7935e039bd69628a49be25f0fbab7d02fce3a72fe8c70`.

Die explizite Kontrollmigration **3 → 4** ermöglicht einen signierten `prepareSession`-Record, bevor
ein AppContainer-Profil erzeugt wird. Er bindet die konkrete Sitzung samt Apphash, Task,
Startkennung, Haltepunkt und kanonischer Wurzel. `bindSession` muss exakt dieser Vorbereitung
entsprechen. Eine Vorbereitung allein erteilt weder Import- noch Recoveryautorität.

`retire-session <aktueller Kopf>` schreibt nötigenfalls die Stilllegung und führt anschließend
Cleanup aus. `cleanup-session <aktueller Kopf>` verlangt eine bereits authentisiert stillgelegte
Sitzung. Profilidentität ausschließlich aus der Kontrollkette, keine Verzeichnissuche oder
caller-seitige Profil-/Pfadauswahl. Actor, Kopf und Jobleerstand werden geprüft; dieselbe exklusive
Kontrollsperre bleibt über Windows-Profillöschung und Abschlussrecord gehalten. Kein fremder Job
wird beendet oder übernommen.

Nach erfolgreichem Entfernen beziehungsweise bestätigtem Nichtvorhandensein folgt in Schema 4 der
signierte `profileCleaned`-Record. Ein weiterer Aufruf mit dem aktuellen Kopf ist idempotent und
schreibt keinen zusätzlichen Record. Andere Windows-Fehler bleiben Fehler; die signierte Stilllegung
bleibt für einen ausdrücklichen erneuten Versuch erhalten. Keine Schlüssel-/Kopfreparatur und keine
neue Importfreigabe. Appkopien, Projekte, Journale und Kontrollschlüssel werden nicht gelöscht.

Authentisiert gebundene Alt-Sitzungen aus Schema 3 können ebenfalls nach Stilllegung bereinigt
werden; dort bleibt es ohne neuen Schema-4-Abschlussrecord. Alte ungebundene Orphans ohne
authentisierten Beleg werden nicht übernommen. Beschädigte/unvollständige Kontrollketten bleiben
gesperrt. Eine Sitzung je Installation und die bisherige Updategrenze bleiben bestehen; kein
Schema-4-Brokerupdate oder echte Datenmigration implementiert. Der Schema-3-Kompatibilitätspfad
wurde diesmal nicht separat nativ ausgeführt.

Mittleres Prüfniveau auf ausdrücklichen Wunsch: zwei neue aktive Tests für Vorbereitungsbindung,
Stilllegung, Cleanupzustand und Abweisungen vor OS-Aufrufen. Insgesamt 48 schnelle aktive
Helfer-/Transporttests, Helfer-Clippy mit Warnungen als Fehler und rustfmt bestanden. Zusätzlich
**zwei native Fälle**: Abbruch nach signierter Vorbereitung vor Profilerzeugung sowie Cleanup nach
Controllerabbruch zwischen Stilllegung und Profillöschung. Der zweite Fall enthält einen kurzen
Schema-4-GUI-Start und einen authentisierten Wiederanlauf; zwei Replay-Ereignisse, zwölf echte
private Dateiverweigerungen, genau eine erhaltene Projektkopie mit zwei Dokumenten.

19 Child-Controller: 13 erfolgreiche Enden, vier erwartete Abweisungen und zwei gezielte
Abbrüche 86. Wiederholter Cleanup erhält denselben Kopf; alte Köpfe und Recovery nach Stilllegung
werden abgewiesen. Ein kleiner unabhängiger Nur-Lese-Audit prüft je sieben/neun MAC-Records samt
Wurzel, Kopf und Vorbereitung/Bindung sowie erhaltene SQLite-/Unicode-Daten. Keine große neue
Crashmatrix, kein kompletter Workspace-Testlauf oder Tauri-Neubuild.
Format-/Diff-/Hash-/Faktenabgleich bestanden; GitHub-CI wird separat ausgewiesen. Ein vorläufiger
headless Lauf bestand ebenfalls; maßgeblich ist der gesicherte Lauf mit kurzem Start/Recovery oben.

Die echte Vorschauablage und ihre autorisierte EXE blieben unverändert. Testprofil entfernt,
Testprozesse beendet; synthetische Daten und private Belege bleiben lokal. Keine Rohpfade,
Kennungen, Schlüssel oder privaten Datenbanken veröffentlicht. **Kein neuer ETW-Netzbeleg dieses
Brokerhashs**; der vorherige Nachweis bleibt auf seinen anderen Hash begrenzt. Nächster Schritt:
gezielter frischer ETW-Vergleich des geänderten Cleanup-Stands. Produkt-Q-01/Q-02, normale
Produktintegration, weitere Crashpunkte und allgemeine Stromausfall-/Rollbackgarantien bleiben
offen.
