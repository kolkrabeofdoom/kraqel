# Privater Brokervertrag v3 · fortlaufende Live-Aufträge

16. November 2026, KQ-069 / #134. Baut auf der lokalen HMAC-/DACL-Grenze aus
    [v2](native-broker-recovery-v2.md) auf und verbindet sie mit
    [Startticket v3](native-ui-restart-v3.md). Ausschließlich synthetischer Forschungslauncher,
    keine Produktintegration oder Migration alter v1-/v2-Sitzungen.

## Autorität und Kette

Die bestehende geschützte native Kontrollablage enthält weiterhin genau einen einmalig erzeugten
32-Byte-Schlüssel. Magic des HMAC-Frames endet jetzt mit `00 03`; das begrenzte Binärformat und
RustCrypto-HMAC-SHA-256 bleiben sonst unverändert. Authority-`version` muss 3 sein.

```text
control/authority.key           privater Klartextschlüssel unter geschützter DACL
control/authority.mac           unveränderlicher Grant, sequence = 0
control/authority-0001.mac      ausdrücklich neuer Grant, sequence = 1
control/settled-0000.mac        geprüfte Wiederherstellung des ersten Grants
control/settled-0001.mac        geprüfte Wiederherstellung des zweiten Grants
control/owner.lock              weiterhin exklusive Broker-Eigentümersperre
control/retired                 weiterhin endgültige Stilllegungssperre
```

Zusätzlich zu Wurzel-/Brokerhash und ursprünglichem Ticket signiert jeder Grant `sequence`,
`previousSha256`, `previousProjects` und den optionalen nativen `liveMode`. Der erste Vorgängerhash
und die erste Projektanzahl sind null. Jeder weitere Grant referenziert SHA-256 der gesamten
vorherigen MAC-Datei, verwendet neue Auftrags-/Launch-IDs und übernimmt die zuletzt bestätigte
Projektanzahl. Auftrags-IDs dürfen innerhalb der Kette nicht doppelt vorkommen.

Bei jeder Übernahme die komplette Folge ab null lesen und jeden MAC, jedes Bindungsfeld sowie die
Vorgängerverkettung prüfen. Vorherige Projektanzahl muss zum zugehörigen privaten Abschlussbeleg
passen. Fehlende Sequenzen, unerwartete Authoritynamen, falsche Vorgänger, beschädigte Daten,
Versions-/Programmwechsel und wiederholte Kennungen sperren. Höchstens 64 Grants; keine Rotation,
Neuanlage oder Reparatur durch Löschen/Überschreiben alter Dateien. Kompromittierte Prozesse mit
denselben Kontorechten bleiben außerhalb der bisherigen Schutzbehauptung; kein Rollbackschutz gegen
einen privilegierten Host, der private Schlüssel und Ablage verändern kann.

## Voraussetzungen für einen neuen Auftrag

Ein erfolgreicher nativer Wiederanlauf bestätigt Reconciliation, passende Kopienzahl/leeres Staging
und Replay-Abwehr. Zusätzlich wird der alte Erstversuch mit seinem Live-Kanal angeboten; er muss vor
Auswahl/Lesen und ohne Journaländerung scheitern. Erst danach schreibt der Host den privaten
`settled`-Beleg mit eigenem MAC. Sein Payload beginnt mit `KRAQEL-SETTLED\0` und bindet die genaue
Grant-Datei sowie Projektanzahl; Authority- und Abschlussbelege sind nicht austauschbar. Wiederholte
Wiederherstellung prüft vorhandene Abschlussbelege, statt sie zu überschreiben.

Nur eine **ausdrückliche neue native Aktion** darf danach den nächsten Grant anhängen. Sie erwirbt
dieselbe Sperre, authentisiert die ganze Kette und den aktuellen Abschlussbeleg, prüft Jobleerstand
und synchronisiert die neue MAC-Datei mit `create_new` vor Appstart. Kein neuer Schlüssel, kein
Journalreset und keine Wiederverwendung alter IDs. Ein bloßer Neustart, eine Diagnose-JSON, ein
Prozessende, ein Rendereraufruf oder ein Timeout erzeugt keine neue Autorität.

Die App erhält nur das neue gebundene Ticket und die anonymen Live-Pipes. HMAC-Schlüssel, Kette,
Abschlussbelege und Hostpfade bleiben außerhalb ihrer Freigabe. Exakte SID/null Capabilities,
Programm-/Jobprüfung vor Resume und Jobzuweisung bei Prozesserzeugung bleiben verpflichtend.
Wiederanläufe erhalten keine Auswahlhandles. Die Stilllegung ist nach authentisiertem Jobleerstand
auch ohne importierbaren Journalzustand zulässig: Sie erzeugt lediglich die endgültige Sperre und
entfernt das Profil, eröffnet keine GUI und löscht weder Projekte noch Journal.

## Forschungsmodi und Grenzen

Zusätzliche begrenzte Fensterendemodi des Forschungshelfers:
`ui-broker-live-seed <frische Wurzel> <App> completed closeDialog|closePartial`. Beide verwenden
native Auswahl und deaktivierte Bedienungsautomatik. `completed` darf vor Fensterende nicht erreicht
werden; andere Crashpunkte werden vor Sitzungsanlage abgelehnt, damit kein Crash-Haltepunkt die
Abbruchbehandlung blockiert. `closeDialog` verlangt die gezielte native Schließnachricht an den
eigenen Auswahlthread; `closePartial` hält genau 1.024 Bytes maximal 60 Sekunden. Die Zeitgrenze des
Dateiauftrags wird dadurch nicht verlängert. Kein zusätzlicher Rendererparameter oder Grant.

Erfolg verlangt Renderer-/Terminalereignis, reguläres Appende 0, beendeten Auswahlthread innerhalb
der begrenzten Bereinigung und leeren Job. Ausschließlich die konkrete Broken-Pipe-Abbruchantwort
des Dialogs darf als erwarteter Transportabschluss gelten; sonstige Fehler bleiben Fehler. Der
Appzustand muss `CANCELLED` ohne Receipt sein, beim Teiltransfer müssen Sender und Empfänger
übereinstimmend 1.024 Bytes belegen. Der Fensterendebeleg erzeugt keinen Abschluss-MAC. Erst eine
separate authentisierte Recovery mit Replay- und Altanfrageprüfung darf einen Folgeauftrag
freigeben. Die nativen Funktionsbelege stehen im
[kombinierten Fensterendenachweis](../testing/native-combined-window-close.md). Der
[eigene frische ETW-Beleg](../testing/native-combined-window-close-etw.md) dieses gesicherten
Helferpaars liegt inzwischen vor; ältere Hashpaare nicht übertragen.

`ui-combined-series <frische Controllerwurzel> <Forschungs-App>` verbindet sechs Crashpunkte mit
gebundenem Live-Transport sowie Abbruch-, ungültige-Datei- und 1.024-Byte-Teiltransferfällen. Danach
jeweils Wiederanlauf, ausdrücklich neuer vollständiger Live-Auftrag, erneuter Controllerverlust,
zweifache Wiederherstellung und Stilllegung. Fixture-Auswahl ist keine manuelle Dateidialogabnahme.

`ui-broker-live-seed <frische Neustartwurzel> <App> <Crashpunkt> native` wartet auf menschliche
Bedienung der gebündelten Forschungsoberfläche und die vorhandene native Auswahl ausschließlich der
vorbereiteten `input.kraqel`. Kein Hostordner erhält Paket-ACLs. Der native Sender öffnet nur
reguläre begrenzte Dateien. Auswahlthread/Dialog/Transfer werden bei regulärem Hostende begrenzt
geschlossen/abgewartet; bei hartem Brokerende beendet Windows dessen Threads/Fenster. Es wird keine
erfolgreiche Bereinigung mit einem losgelösten Auswahlthread gemeldet.

`ui-broker-recover <Wurzel> first|second` stellt nur den aktuellen verbrauchten Auftrag wieder her.
Die Aktion `manual` verwendet dieselbe Authentisierung und denselben verbrauchten Auftrag,
deaktiviert aber die Bedienungsautomatik ausschließlich für das Wiederanlauffenster. Ein echter
Replay-Aufruf und reguläres Fensterende mit Exitcode 0 sind erforderlich; vorzeitiges Schließen gilt
als Fehler. Anschließend läuft die gesperrte ursprüngliche Importanfrage automatisch und ohne
Journaländerung. Erst danach darf der signierte Abschlussbeleg entstehen. Diagnoseindizes 5/6 je
Achterblock trennen diesen Ablauf von `first` und `second`. Es entstehen keine Auswahlhandles und
keine neue Importautorität. Hörbestätigung bleibt eine separate menschliche Prüfung. Der geänderte
Helfer übernimmt den früheren ETW-Beleg nicht; er besitzt inzwischen einen
[eigenen frischen Nachweis](../testing/native-combined-assistive-restart-etw.md) mit
Tastaturprüfung.

`ui-broker-live-next <Wurzel>` erzeugt ausdrücklich einen neuen synthetischen Fixture-Auftrag; keine
allgemeine Produkt-Import-API. `retire` bleibt die Stilllegungsaktion. Native Journale speichern
Fehlversuche dauerhaft; auch nach Cancel/ungültiger Datei entsteht keine Wiederholungsfreigabe.

Die initiale Dateioperation behält ihre Zeit-/Bytegrenzen; manuelle Betrachtungszeit verlängert
keinen erteilten Dateiauftrag. Stromausfälle, Datenträgerfehler und jeder mögliche OS-Mikroschritt
sind nicht abgenommen. Insbesondere ein Crash nach Grant-Synchronisierung, aber vor dem ersten
Journal-Claim kann die Sitzung konservativ blockieren; keine automatische Wiederaufnahme oder
Kennungsneuvergabe daraus ableiten. Produktablage, Update-/Migrationspfad, weitere Fensterende-/
assistive Prüfungen und externe Dienstgrenzen bleiben offen. Das unveränderte gesicherte v3-Paar hat
inzwischen einen [eigenen ETW-Nachweis](../testing/native-combined-live-restart-etw.md); keine
automatische Übertragung auf Änderungen oder das Produkt. Keine neue Netzwerkfähigkeit oder
DPAPI-/Domänendienstausnahme; striktes Datenschutzziel unverändert.
