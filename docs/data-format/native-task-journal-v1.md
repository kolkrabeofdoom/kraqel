# Forschungsjournal für native Dateiaufträge · v1

KQ-069 / #134, 16. November 2026. Nur `kraqel-isolation-test-adapter` und das explizite
Windows-Feature `isolation-ui-test`; kein neuer Produktbroker und keine Änderung des
[Dateiauftrags v1](native-file-task-v1.schema.json) oder des
[Live-Kanals v2](native-live-file-channel-v2.md).

Die Herkunft der Auftragskennung bleibt der geprüfte native Grant beziehungsweise die gebundene
Live-Sitzung. Ein Journal ist keine zusätzliche Freigabe und authentisiert keine fremden Aufträge.
Der Renderer kann weder Kennungen noch Pfade, Wiederherstellungsaktionen oder Zustände nachliefern.

## Lokaler Aufbau

```text
data/app/projects/<neue Projekt-UUID>/   sichtbare Projektkopien
data/app/tasks/
  owner.sqlite3                         exklusive SQLite-Schreibsperre für die Sitzung
  tasks.sqlite3                         Verbrauch, Zustand, begrenztes Ergebnis
  staging/<Auftrags-UUID>/               privater Importzwischenstand
```

Die Journalwurzel wird ausschließlich nativ aus der bestehenden Forschungsdatenfreigabe abgeleitet
und darf die Projektwurzel weder enthalten noch darin liegen. Keine zusätzliche ACL, Capability,
Shell-Operation, Netzwerkverbindung oder Renderer-Berechtigung. Die bereits vorhandenen
Abhängigkeiten `rusqlite` 0.39 (MIT, gebündeltes SQLite) und `serde_json` 1 (MIT/Apache-2.0) werden
direkt im Forschungsadapter verwendet; keine neue Registryversion oder Produkt-Laufzeitabhängigkeit.

Das Journal nutzt SQLite `user_version = 1`, `journal_mode = DELETE`, `synchronous = FULL`.
`binding.root_hash` bindet es an den SHA-256 der kanonischen lokalen Projektwurzel. Ein Verschieben
ist kein erlaubtes automatisches Neubinden. `task` speichert Kennung, Zustand, optional den strengen
`ImportReceipt` oder einen serialisierten `TaskError`. Keine Titel, Manuskripttexte, Hostdateinamen
oder absoluten Pfade in den Journalzeilen. Die Zwischenverzeichnisse enthalten dagegen ausdrücklich
die lokal importierten Projektinhalte; Journal und Zwischenstände bleiben privat und gehören weder
in Git noch in Projektarchive.

Eine zweite Verbindung hält `BEGIN IMMEDIATE` auf `owner.sqlite3` bis zum Ende des Store-Objekts.
Ein zweites Store-Objekt scheitert sofort. SQLite gibt diese Sperre nach Prozessende frei; eine
PID-Datei oder ein Timeout wird nicht als Beweis für einen toten Prozess verwendet. Höchstens 4.096
verbrauchte Kennungen pro Journal; danach keine neuen Aufträge. Es gibt kein automatisches Löschen
alter Kennungen, Zurücksetzen des Journals oder Rotieren mit erneuter Freigabe.

## Zustandsübergänge

| Zustand       | Bedeutung                                              | Verhalten bei erneutem Öffnen                                                                                    |
| ------------- | ------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------- |
| `started`     | Kennung vor Dialog/erstem Lesen dauerhaft verbraucht   | Eigenen Zwischenstand entfernen, terminal `interrupted`                                                          |
| `committing`  | Vollständiger Receipt vor finalem Rename gespeichert   | Vorhandenes Ziel prüfen und als `succeeded` zuordnen; ohne Ziel Zwischenstand verwerfen und `interrupted` setzen |
| `succeeded`   | Rename und Abschluss gespeichert                       | Receipt lesbar, keine neue Ausführung                                                                            |
| `failed`      | Fehler, Abbruch oder Ablauf vor Veröffentlichung       | Fehler lesbar, keine neue Ausführung                                                                             |
| `interrupted` | Früherer Versuch ohne belegte Veröffentlichung beendet | `INTERRUPTED` lesbar, keine neue Ausführung                                                                      |

Der gewöhnliche validierende Importer baut zunächst eine neue Kopie vollständig innerhalb des
auftragseigenen Zwischenverzeichnisses auf. Vor der endgültigen Veröffentlichung werden
Abbruch/Ablauf erneut geprüft. Nach gespeichertem `committing` beginnt die Commit-Grenze: ein
späterer Abbruch kann die Veröffentlichung nicht garantiert zurücknehmen. Nur ein Rename des
vorbereiteten Projektverzeichnisses macht die Kopie in der eigentlichen Projektwurzel sichtbar.
Journal und Ziel müssen dafür auf einem geeigneten gemeinsamen Dateisystem liegen; ein gescheiterter
Rename lässt den Auftrag gesperrt und wird beim Wiederanlauf konservativ aufgelöst.

Reconciliation veröffentlicht niemals einen Zwischenstand und liest niemals die Quelldatei erneut.
Bei vorhandenem Commit-Ziel prüft sie reguläre unverknüpfte Pfade, die gespeicherte Projekt-ID,
SQLite-Integrität/Fremdschlüssel und Dokument-/Entitäts-/Assetanzahlen. Das ist kein neuer
vollständiger Hashnachweis sämtlicher Assetdateien. Unklare oder beschädigte Ziele werden erhalten
und blockieren die weitere Bearbeitung. Bestandsprojekte werden nicht gelöscht oder überschrieben.
Die Wiederholung einer Kennung liefert vor jedem neuen Lesen `ALREADY_CONSUMED`.

Fehlendes Journal in einer bereits benutzten Wurzel, unbekannte Version, falsche Bindung,
beschädigte Datenbank, unzuordenbare Zwischenverzeichnisse, Links/Reparsepunkte, Sperrkonflikte und
mehrdeutige Ergebnisse führen zu `STATE_UNAVAILABLE`. Es gibt keine automatische Reparatur durch
Neuanlage. Die UI erklärt die neuen datenarmen Codes `INTERRUPTED` und `STATE_UNAVAILABLE`; ein
Fehler bereits bei der Start-Reconciliation beendet den Forschungsstart vor WebView-Aufbau.

## Grenzen

Fortschreibung 16. November: Der neue explizite
[Broker-Wiederanlaufvertrag](native-broker-recovery-v3.md) ergänzt authentisierte Übernahme nach
Controllerverlust für synthetische Forschungsablagen. Der Journalvertrag selbst wird dadurch nicht
zu einer Autorität; alte Launcherpfade behalten ihre bisherigen Grenzen. Produktintegration und
neuer ETW-Nachweis des Brokers bleiben offen.

Geprüft sind Prozessabbrüche und wiederholtes Öffnen des Stores, keine Stromausfälle,
Datenträgerdefekte, verlorenen Verzeichniseinträge oder beliebige Manipulationen durch Prozesse mit
denselben Schreibrechten. SQLite-FULL allein beweist keine plattformübergreifende
Stromausfall-Atomarität des zusätzlichen Verzeichnis-Renames. Die Pfadprüfungen ersetzen keine
vollständige Abwehr konkurrierender privilegierter Dateisystemmanipulation. Ein kompromittierter
Containerprozess kann seine eigene beschreibbare Forschungsablage manipulieren; das Journal ersetzt
weder die OS-Grenze noch einen authentisierten künftigen Produktbroker.

Ein Controlleraufruf verlangt weiterhin eine frische Forschungswurzel. Der ergänzte
[Neustartkanal v1](native-ui-restart-v1.md) behält innerhalb desselben lebenden Controllers
Identität/Ablage über mehrere GUI-Generationen; die integrierte Crash-/Reconciliation-Serie ist
[nativ geprüft](../testing/native-ui-restart.md). Übernahme einer bestehenden Wurzel durch einen
anderen Controller oder Wiederanlauf nach Controllerabsturz bleiben ausgeschlossen und offen. Die
frühere reine Adapterstufe hat eine eigene [Buildbindung](../testing/native-task-reconciliation.md).
