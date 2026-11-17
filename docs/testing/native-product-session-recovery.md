# KQ-069: Dauerhafte Apppaket-/Sitzungsbindung und authentisierter Wiederanlauf

Stand: 17. November 2026. Native Windows-Vorschau mit synthetischen Daten. Striktes Datenschutzziel
unverändert; keine normale Produktintegration oder Produkt-Q-01/Q-02-Freigabe.
[Bereinigte Aggregate](native-product-session-recovery.json),
[Kontrollvertrag](../data-format/native-product-control-v1.md).

Ausgangspunkt `b0e6ca618c073febbcbaae086c57721f47a99450`. Gesicherter Prüfbuild:
`%TEMP%/kraqel-product-session-build-20261117-02/`.

- Broker-SHA-256: `703c50c1cf207b5a6a6e9281111298d768fb6a73618de810e85a4657d67cdd22`.
- Unveränderte Forschungs-App: `6e56a51ed2e913a878a7935e039bd69628a49be25f0fbab7d02fce3a72fe8c70`.

## Implementierte Verträge

Die explizite authentisierte Kontrollmigration **2 → 3** erlaubt genau eine dauerhafte synthetische
Sitzungsbindung je Vorschauinstallation. Sie migriert keine vorhandenen Appdaten. Ein neuer
`bindSession`-Record bindet Sitzung, Auftrag, erste Startkennung, App-SHA, kanonischen
UTF-16LE-Sitzungswurzelhash und typisierten synthetischen Haltepunkt. Alle Records bleiben in
derselben privaten HMAC-Kette mit authentisiertem Kopf. Kein neuer Kontrollschlüssel oder separater
aus Diagnosedateien abgeleiteter Grant.

Die Sitzung liegt als Geschwisterablage zur privaten Kontrollwurzel:

```text
<Kontrollwurzel-Elternordner>/.kraqel-app-preview/<Installation>/
  kraqel-isolation-ui-restart-<Sitzung>/
    bin/kraqel-desktop.exe  # an signierte SHA gebundene, bei Starts gepinnte Appkopie
    data/                  # dieselben App-/Journal-/Projektdaten bei Wiederanlauf
```

Basis und Installationsordner bleiben privat für Konto/SYSTEM. Nur Sitzungswurzel/Appkopie erhalten
die nötigen Container-Leserechte und `data/` Schreibrechte. Die Kontrollwurzel samt Schlüssel,
Records, verwalteten Brokerpaketen und reserviertem privatem `data/` bleibt unverändert privat.
Lokale feste Laufwerke, gepinnte Vorfahren, keine Remote-/Reparse-/Cloud-/Hardlinkübernahme. Die
normale Profilablage ist außerhalb TEMP; die native Testserie verwendet ausdrücklich neue
synthetische Kontrollwurzeln und dadurch private Test-Geschwisterablagen unter TEMP.

`recover-session <aktueller Kopf>` liest Sitzung, Auftrag, Apphash, Wurzel und Haltepunkt
ausschließlich aus der authentisierten Kontrollkette. Keine Pfad-/App-/Taskauswahl beim Recovery.
Paket und vorhandene Journale müssen vorliegen; fehlende Dateien werden nicht erzeugt oder ersetzt.
Vor Start wird eine frische `recoverSession`-Startkennung synchron in Kontrollrecord/Kopf
festgeschrieben. Jede neue GUI erhält frische begrenzte Pipes und ein Recovery-Ticket für denselben
verbrauchten Auftrag. Kein initialer Importgrant. Bereits verwendete Startkennungen, alte Köpfe,
nil-Kennungen und mehr als 16 Starts insgesamt werden abgewiesen.

Die exklusive Kontrollsperre bleibt über Authentisierung, Recordabschluss, kompletten GUI-/Joblauf
und Jobleerstand gehalten. Der gleiche benannte Produktjob wie bei Updates wird verwendet. Nach
Controllerabbruch wartet Recovery unter dieser Sperre höchstens zehn Sekunden auf das natürliche
Verschwinden des alten kill-on-close-Jobs. Keine Übernahme oder manuelle Beendigung eines fremden
Jobs. Suspendierte Root-/Token-/Jobprüfung, echte Sperrprüfung und sechs private
Kontrolldateiverweigerungen erfolgen vor jedem Resume.

Das bestehende AppContainer-Profil bleibt über Generationen erhalten. `retire-session` schreibt die
Stilllegung vor Profilentfernung authentisiert fest; danach bleibt Recovery gesperrt. Keine
Neubindung derselben Vorschauinstallation nach Stilllegung. Allgemeine Mehrsitzungsverwaltung,
Cleanup nach fehlgeschlagener Profilentfernung und echte Projektmigration bleiben Folgearbeit.
Schema 3 erlaubt derzeit keine Broker-Updatevorbereitung/-aktivierung; die bekannte Updatepolitik
bleibt auf Schema 2 begrenzt. Keine automatische Schema-Rückkehr oder
Paketkompatibilitätsbehauptung.

## Native Prüfung

`product-session-series <frische TEMP-Controllerwurzel> <Forschungs-App> <App-SHA>` startet
getrennte native Prozesse für zwei Fälle: **Prepared** (vor Veröffentlichung) und **Completed**
(veröffentlicht). Nach authentisierter Bindung wird der Controller am wirklichen GUI-Haltepunkt mit
`TerminateProcess`, Exitcode 86, beendet. Keine Destruktoren oder absichtliches Job-Cleanup vorher.
Windows beendet den gehaltenen kill-on-close-Job. Der Wiederanlauf kommt aus einem neuen
Controllerprozess mit neu geöffnetem Kontrollstore und derselben App-/Sitzungs-/Auftragsbindung.

Je Fall zwei authentisierte Wiederanläufe mit je zwei echten Replay-Ereignissen. Danach negative
Prüfung: veralteter Kopf; fehlendes Journal im Prepared-Fall beziehungsweise nachträglich
manipulierte Appkopie im Completed-Fall; Recovery nach authentisierter Stilllegung. Keine dieser
Abweisungen schreibt einen Kontrollrecord oder erzeugt eine neue Projektkopie. Das fehlende
synthetische Journal bleibt unter `tasks.missing` zur Nachprüfung erhalten; die Originaldatei wird
nicht neu angelegt. Die manipulierte Test-App wird nicht repariert oder erneut ausgeführt.

Ergebnis: **18 Child-Controller**, zehn reguläre Enden 0, sechs erwartete Abweisungen 1 und zwei
harte Abbrüche 86. **Sechs GUI-Starts, vier authentisierte Wiederanläufe, acht Replay-Ereignisse, 36
tatsächliche private Dateiverweigerungen.** Zwei verschiedene Aufgaben und sechs verschiedene
Startkennungen, insgesamt genau **eine** legitime Projektkopie: Prepared bleibt unterbrochen ohne
Kopie, Completed behält seine Kopie mit zwei Dokumenten. Keine Replay-Kopie. Alle vier Recovery-GUIs
kontrolliert mit 77 beendet; zwei Profile nach authentisierter Stilllegung entfernt, Jobprozesse 0.

Unabhängiger Nur-Lese-Audit bestätigt je sieben Kontrollrecords, HMAC/Installation/Vorgängerkette,
kanonische Store-/Sitzungswurzelbindung und Kopf, unveränderte Schlüssel bei den Negativfällen,
Apphash vor Manipulation, alle sechs nativen Token-/Sperr-/Jobbelege, persistierte Taskzustände,
Archivhash/-länge, SQLite-Integrität/Fremdschlüssel und exakten Unicode-Inhalt. Die identischen
Sitzungswurzeln werden in beiden Wiederanläufen gegengeprüft. Rohpfade, PIDs, UUIDs, Schlüssel und
private Datenbanken bleiben lokal; nur Aggregate veröffentlicht.

45 aktive Helfer-/Transporttests, Helfer-Clippy mit Warnungen als Fehler, rustfmt und `pnpm check`
bestanden. Zwei neue aktive Tests prüfen Schema-/Bindungs-/Recoverypersistenz sowie
Stilllegung/Replaybudget/alte Köpfe. Der bisher unbekannte Übergangstest verwendet nun 2 → 4; 2 → 3
ist jetzt der ausdrücklich implementierte Übergang. Keine neue Abhängigkeit oder kompletter
Tauri-Neubuild; Forschungs-App unverändert. Ein Vorlauf mit regulär endenden Controllern bestand
ebenfalls; maßgeblich ist die danach frisch gebaute Serie mit echten Controllerabbrüchen oben.

Zusätzlich besteht die bisherige 28-Fälle-Kontroll-Crash-Serie funktional mit demselben neuen
Brokerhash: 17 gesperrte/elf nutzbare Zustände, acht explizite Weiterführungen, 162 Child-Controller
mit 76 erfolgreichen, 58 abgewiesenen und 28 harten Enden. Unabhängiger MAC-/Kopf-/Paket-/Datenaudit
grün; zwei zusätzliche Container-GUIs mit zwölf weiteren Dateiverweigerungen und entfernten
Profilen. Das ist eine Funktionsregression, kein neuer Netzbeleg.

## Grenzen und nächster Schritt

Die echte `.kraqel-broker-preview` und ihre bisher autorisierte EXE bleiben unverändert; der neue
Prüfbuild wurde dort nicht aktiviert. Kein Kopieren oder Zurücksetzen von Kontrollablagen. Native
Serien verwenden synthetische Daten, keine realen Manuskripte oder übernommenen Forschungsjournale.
Profile und Testprozesse beendet; synthetische Fixtures einschließlich negativer Fälle lokal
erhalten.

**Kein frischer ETW-Netzbeleg dieses neuen Brokerhashs.** Frühere Messungen gelten nur für ihre
eigenen Hashes/Lebenszeiten. Keine Inhalts-/Uploadbilanz oder vollständige externe COM-/Shell-/DNS-
Brokerattestierung. Allgemeine Stromausfälle, vollständiger privater Checkpoint-Rollback,
Mehrsitzungsbetrieb, Update-/Apppaketmigration in Schema 3 und normale Produktintegration sind nicht
abgenommen. Unvollständige Kontrollabschlüsse bleiben gesperrt, keine automatische Reparatur.
Nächster Schritt: weitere Crashpunkte der Sitzungsbindung/Startrecord-/Stilllegungsübergänge und
frischer kalibrierter ETW-Vergleich des gesicherten Paars. Striktes Datenschutzziel unverändert.
