# KQ-069: Dauerhafte Kontrollablage, autorisierte Updates und Kontrollmigration

Fortschreibung:
[28 Kontroll-Crashfälle und eigener frischer ETW-Beleg](native-product-control-crash-etw.md) eines
neuen separat gesicherten Prüfbuilds inzwischen bestanden. Die folgende ursprüngliche dauerhafte
Vorschauablage und ihre autorisierte EXE blieben dabei unverändert. Keine automatische Übertragung
von Programmfreigaben oder Aufweichung gesperrter Kopfzustände.

Stand: 16. November 2026. Erste native Windows-Vorschau, keine normale Produktintegration.
[Vertrag](../data-format/native-product-control-v1.md),
[bereinigte Aggregate](native-product-control.json). Striktes Datenschutzziel unverändert;
Produkt-Q-01/Q-02 nicht bestanden.

## Implementiert und nativ verifiziert

Private dauerhafte Ablage unter `%USERPROFILE%\.kraqel-broker-preview`, unabhängig von TEMP und dem
Paketcache des Coding-Werkzeugs. Atomar private Verzeichniserzeugung, ACL-Prüfung vor
Schlüsselerzeugung, exklusive Sperre, gepinnte lokale Vorfahren, eigener HMAC-Bereich,
unveränderliche Übergangsrecords und authentisierter Abschlusskopf. Schlüssel werden ausschließlich
bei frischer Anlage erzeugt; kein Reset bei Fehlern. Die Vorschauablage bleibt bewusst erhalten; ihr
`data/` ist leer, vorhandene Schreibprojekte und Forschungsjournale wurden nicht übernommen.

Native Prozessfolge mit **30 eigenständigen Kontrollaufrufen**, davon zehn erwartete Abweisungen:
Neuanlage, explizite Kontrollschema-Migration 1 → 2, falscher Dateihash abgewiesen, Update
vorbereitet, Pending nach Prozessneustart erhalten, veraltete Migration/Aktivierung abgewiesen,
Aktivierung, alte EXE abgewiesen, neue verwaltete EXE akzeptiert, wiederholte Migration und Rückkehr
zur alten SHA abgewiesen. Danach zwei weitere ausdrücklich autorisierte Programmwechsel; jeweils der
Vorgänger abgewiesen. Kein Schlüssel-, Wurzel- oder Datenreset zwischen den Wechseln.

Die erste neue EXE war eine Kopie mit synthetischem PE-Overlay, also anderer Dateihash bei gleichem
Programmcode. Der zweite Wechsel verwendet tatsächlich geänderten Code mit dem finalen Schema-2-Gate
für Updateaktionen. Der dritte Wechsel enthält die modulare Trennung von Dateisystemgrenze/Tests und
die begrenzte Verzeichniseinlesung. Die explizite Migration wurde vor allen Wechseln ausgeführt; die
gesamte Kette wird vom finalen Code erneut authentisiert. Aktueller Zustand: Kontrollschema 2,
Sequenz 7, acht Records, vier nacheinander autorisierte Programmhashes. Finale EXE-SHA-256:
`cfe51760acc79112a5bff612326fa5cf1f6ef94d97cd897f7b09cc14850ae51c`. Native Rohbelege und gesicherte
Programme ausschließlich lokal unter `%TEMP%/kraqel-product-control-build-20261116-02/`; aktive
verwaltete EXE zusätzlich in der privaten Ablage. Quelländerungen gehören zum Commit dieses Belegs,
Ausgangspunkt `90c28f6d98aeeed09e31d0057e3dd497dbed161b`.

## Tatsächlicher Containerzugriff und unabhängige Prüfung

Nach jedem der drei Programmwechsel je eine echte isolierte synthetische GUI mit unveränderter
Forschungs-App (`6e56a51ed2e913a878a7935e039bd69628a49be25f0fbab7d02fce3a72fe8c70`): exakte
AppContainer-Identität, null Capabilities und **18 echte Dateiverweigerungen** insgesamt. Unter dem
suspendierten Root-Token wurden Schlüssel und Kopf jeweils lesend/schreibend sowie Sperrdatei und
Genesis lesend geöffnet; alle Versuche endeten mit `ERROR_ACCESS_DENIED`. Anschließend je
synthetischer Import mit zwei Dokumenten, regulär kontrolliertes Testende 77, Job leer und
Testprofil entfernt. Keine Ableitung allein aus einem ACL-String.

Zusätzliche native Regression des bestehenden dauerhaften Forschungsablaufs: fünf App-Starts, Import
am Crashpunkt, zwei authentisierte Wiederanläufe, Replay-/Altanfrage-Abwehr und Stilllegung
bestanden. Alle vier Testprofile entfernt, keine Forschungsprozesse aktiv. Die dauerhafte
Kontrollablage wird nicht als bereinigtes TEMP-Profil ausgegeben.

Unabhängiger Nur-Lese-Audit mit separater HMAC-Implementierung bestätigt alle acht Records,
Vorgängerhashes, Operationskennungen, unveränderte Installation und kanonische Wurzelbindung,
Abschlusskopf sowie Paket-/Programmhashes. Der erste signierte Record stimmt bytehashgenau mit dem
vor Updates protokollierten Ursprung überein und authentisiert weiterhin mit dem vorhandenen
Schlüssel. Native Prozesscodes, Token-/Verweigerungsbelege sowie SQLite-Integrität, Fremdschlüssel,
Unicode und Dokumentzahlen der synthetischen GUI-Importe gegengeprüft. Keine Schlüssel, IDs,
Absolutpfade oder Rohdaten veröffentlicht.

## Automatisierte Prüfungen und verworfene Ansätze

39 aktive Helfer-/Transporttests einschließlich acht neuer Kontrollablage-Tests bestanden;
Helfer-Clippy mit Warnungen als Fehler, rustfmt und `pnpm check` grün. Geprüft sind unter anderem:

- Exklusivität, unveränderter Daten-Sentinel, fehlende Schlüssel ohne Neuerzeugung.
- Pending über Wiederöffnen, manipulierte Kandidaten, expliziter Abbruch, alte Köpfe und
  Programmhashes.
- Schemafreischaltung, unzulässige Schemaübergänge und Downgrade-Abweisung.
- Fremde/alte MAC-Bereiche, verschobene Wurzeln, Hardlinks, Remote-/Namespacepfade, breitere ACLs.
- Manipulierte Records, Lücken, fehlender letzter Record, unpassender Kopf und `head.next` nach
  unterbrochenem Abschluss; unveränderte konservative Sperre statt Reparatur.
- Tatsächlich aktiver nativer Job blockiert Änderungen, ohne den Job zu beenden; nach geprüftem
  Jobende ist die Migration möglich. Der Test verwendet einen begrenzten netzfreien Unterprozess.

Zwei interaktive Shell-Einzeltests nicht erneut separat ausgeführt; der im Standardlauf ignorierte
Job-Unterprozesshelfer wird vom aktiven Jobtest ausdrücklich gestartet. Eine beobachtete
Zeitstempel-Kollision vorhandener paralleler Dateitest-Fixtures wurde auf UUID-Namen umgestellt.
Kein vollständiger Tauri-Build erforderlich, normale Debug-App unverändert.

Ein erster Initialisierungslauf unter nominalem LocalAppData scheiterte beim Kopfabschluss mit
Fehler 17: Der paketierte Hostkontext leitete Dateizugriffe in seinen Cache auf einem anderen
Laufwerk um. Kein bestandener Store, keine Aktivierung oder Übernahme dieses Zustands. Die neue
Ablage verwendet den separat geprüften lokalen Profilpfad und normalisierte native Pfade; der
unvollständige Cacheversuch wird nicht repariert oder wiederverwendet. Zwischenzeitliche Fehler beim
Pinnen der Verbatim-Laufwerkspräfixe wurden vor dem erfolgreichen Lauf korrigiert.

## Aussagegrenzen und nächste Arbeit

Dies ist eine funktionale Kontrollablage-Vorschau. Keine Übernahme existierender Manuskripte, keine
Projekt-/Auftragsdatenbankmigration, kein normaler isolierter Produktstarter und kein
Installer-/Release-Updater. Lokale explizite Hashfreigabe authentisiert die Entscheidung, nicht die
Herkunft eines Release-Herausgebenden. Keine Code-Signing-Zertifikate erforderlich, aber
Herausgebervertrauen/Kompatibilitätsprüfung sind damit nicht gelöst.

Keine vollständige private Checkpoint-Rollbackerkennung, Absicherung gegen Kontokompromittierung,
beliebige Stromausfälle oder integrierte Produkt-Crash-Serie behauptet. Das unvollständige
Record-/Kopf-Paar bleibt absichtlich gesperrt. Der künftige Starter muss dieselbe Sperre und den
gebundenen Produktjob verwenden; erst diese Integration schützt den gesamten Produktlebenszyklus.
Schreibprojekttransfer auf andere Rechner erfolgt weiterhin über ausdrücklich exportierte Archive,
nicht durch Kopieren dieser Schlüsselablage.

**Noch kein frischer ETW-Beleg dieses neuen Codes oder der Update-Prozessfolge.** Der vorige
[Fensterende-Netznachweis](native-combined-window-close-etw.md) gilt nur für sein gesichertes Paar.
Nächste Arbeit: Crash-Reconciliation der Kontrollübergänge und eigener kalibrierter ETW-Lauf für
Initialisierung/Neustart/Programmwechsel/Schemafreischaltung; anschließend Einbindung in den
isolierten Produktstarter und ausdrücklich autorisierte echte Datenmigration. Striktes Ziel ohne
Runtime-/Domänenausnahme unverändert.
