# Dauerhafte native Broker-Wiederherstellung · Forschungsvertrag v1

Historischer verworfener DPAPI-Zwischenstand. Der aktuelle Broker implementiert
[HMAC-Vertrag v2](native-broker-recovery-v2.md); kein automatischer Übertrag/Migrationspfad.

KQ-069 / #134, 16. November 2026. Grundlage für einen künftigen Produktbroker, ausschließlich im
getrennten Windows-Forschungslauncher und mit synthetischen Daten implementiert. Kein installierter
Dienst, kein öffentlicher RPC-Endpunkt und keine Umstellung des normalen Produktstarts.
[Startticket v2](native-ui-restart-v2.md) und [Auftragsjournal v1](native-task-journal-v1.md)
bleiben unverändert. [Nativer Nachweis](../testing/native-controller-crash.md).

## Zuständigkeit und Ablage

```text
kraqel-isolation-ui-restart-<Session-UUID>/
  control/                    geschützte DACL: aktuelles Windows-Konto und SYSTEM
    authority.dpapi           unveränderlicher, kontogeschützter Wiederanlaufbeleg
    owner.lock                exklusiver Broker-Dateihandle, nicht vererbt
    retired                   endgültige Sperre nach expliziter Stilllegung
  bin/kraqel-desktop.exe      vorhandene, fest gebundene Forschungs-App
  data/app/                  bisherige Containerablage einschließlich Auftragsjournal
```

Nur eine frische TEMP-Wurzel darf initialisiert werden. `control` erhält eine geschützte DACL ohne
Container-/Paket-, Everyone- oder vererbte Gruppenfreigaben. Die zusätzliche
Container-Schreibfreigabe bleibt auf `data` begrenzt. Während eines Brokerlaufs sind die
Verzeichnisse gegen Umbenennen/Löschen und die Appdatei gegen Schreiben/Löschen über offene Handles
gesperrt. Reguläre lokale Dateien mit genau einem Link sind erforderlich; Reparsepunkte und
Cloudplatzhalter werden abgewiesen.

Der Beleg enthält `version: 1`, SHA-256 der kanonischen UTF-16-Wurzel, SHA-256 des Brokerprogramms
und das vollständige ursprüngliche Startticket mit Session-/Auftrags-/Launch-ID, Crashpunkt und
Apphash. Keine Manuskripttexte, Titel oder Hostdateinamen. DPAPI schützt den serialisierten Beleg
mit `CRYPTPROTECT_UI_FORBIDDEN`, ohne `CRYPTPROTECT_LOCAL_MACHINE`; keine eigene Kryptografie oder
neue Registryabhängigkeit. Ein einmaliger `create_new`-Schreibvorgang mit `sync_all` geht dem ersten
Appstart voraus. Fehlende/beschädigte Belege werden niemals automatisch ersetzt.

## Übernahme nach Controllerverlust

1. Native TEMP-/Sessionbindung und unverknüpfte Pfade prüfen, Verzeichnisse und Programmdatei
   halten, exklusive Broker-Sperre erwerben.
2. Höchstens 32 KiB DPAPI-Beleg lesen, Authentizität und strenge Form prüfen. Version, Wurzel,
   Session, ursprüngliches Ticket und beide aktuellen Programmhashes müssen passen. Eine
   `retired`-Markierung sperrt sofort.
3. Vorhandene Auftrags- und Eigentümerdatenbank verlangen. Ein gültiger Beleg allein erlaubt keine
   Journal-Neuanlage und keinen neuen Import.
4. Den aus der authentisierten Session abgeleiteten benannten Job prüfen. Solange er noch Prozesse
   enthält, höchstens zehn Sekunden warten; danach abbrechen. Nur ein fehlender oder leerer Job
   erlaubt den nächsten Versuch. Eine Kollision beim Erzeugen des nächsten Jobs wird abgewiesen.
5. Dieselbe Container-SID ableiten. Neue anonyme Pipes und neue Launch-ID verwenden; ausschließlich
   `recovery: true` für denselben verbrauchten Auftrag. Keine Quelle auswählen/lesen oder
   automatische Veröffentlichung eines Zwischenstands. Die App prüft weiterhin Ticket, SID,
   Programmhash und Journal vor WebView-Aufbau.
6. Der App-Prozess erhält den Kill-on-close-Job bereits durch `PROC_THREAD_ATTRIBUTE_JOB_LIST` bei
   `CreateProcessW`. Suspendiert Jobzugehörigkeit, exakte SID, null Capabilities und tatsächliche
   Dateizugriffsverweigerungen unter diesem App-Token prüfen. Danach erst `ResumeThread`; keine
   ersatzweise Ausführung ohne Isolation.

Der Broker besitzt den einzigen, nicht vererbbaren Jobhandle. Die Crash-Serie beendet den Controller
selbst hart mit Code 86, ohne Rust-Destruktoren oder vorherigen `TerminateJobObject`. Die
nachfolgende Übernahme muss die alte Jobbeendigung abwarten. Jeder reguläre Wiederanlauf prüft
Reconciliation sowie zwei Replay-Aufrufe; zusätzlich wird die ursprüngliche Importkennung erneut
angeboten und muss ohne Journaländerung abgewiesen werden.

Stilllegung ist eine eigene native Aktion unter derselben authentisierten Sperre: Jobleerstand
prüfen, dauerhafte `retired`-Markierung anlegen/synchronisieren, dann Profil entfernen. Weitere
Wiederanläufe bleiben gesperrt. Synthetische Daten werden zur Nachprüfung erhalten. Fehler während
der Stilllegung sind kein erfolgreicher Cleanup und führen nicht zu einer neuen Freigabe.

## Vertrauensgrenze und offene Produktarbeit

**Datenschutz-Sperre für die Produktintegration:** Kontogeschützte DPAPI ist keine garantierte
Offline-Primitive. Microsoft dokumentiert für Domänenkonten Masterkey-Backup/-Wiederherstellung über
Domänencontroller und RPC. Diese externen Systemdienstpfade sind durch einen ETW-Appbaum allein
nicht kausal abgenommen. Vor Produktintegration muss ihre Verträglichkeit mit dem strikten Ziel
belegt oder die Primitive ersetzt werden. Keine OS-/Registry-Ausnahme oder
Domänenrichtlinienänderung als Ersatz.
[Microsoft: DPAPI-Masterkey-Backup](https://learn.microsoft.com/en-us/troubleshoot/windows-server/identity/dpapi-masterkey-backup-failures).

Authentisiert wird die dauerhafte native Wiederanlaufautorität gegenüber dem Container. DPAPI ist
keine Codeattestierung beliebiger Prozesse desselben Windows-Kontos und kein Schutz gegen ein
kompromittiertes Konto, Administratorrechte oder exportierte Kontogeheimnisse. Der benannte Job ist
kein Kommunikationsendpunkt; seine Kennung ist allein keine Autorität. Untrusted Diagnosedateien,
PIDs und Rendererparameter dürfen die Prüfung nicht ersetzen.

Hashbindung verhindert stillen Programmwechsel, implementiert aber noch keinen authentisierten
Update-/Migrationspfad. Der Container kann seine eigene beschreibbare Datenablage manipulieren;
belegter Receipt/Journalzustand ersetzt keine Integritätsattestierung kompromittierter Prozesse.
Unklare Daten müssen weiterhin blockieren. Stromausfall-, Datenträger- oder vollständige
Crashgarantien in jedem Betriebssystem-Mikroschritt werden nicht behauptet.

Die CLI-Aktionen `ui-broker-seed`, `ui-broker-recover first|second|retire` sind Forschungsaktionen.
`first`/`second` wählen nur lokale Diagnoseindizes, niemals neue Rechte oder Aufträge. Ein dauerhaft
betriebsfähiger Produktbroker braucht zusätzlich geschützte Produktablage, Lebenszyklus-/Update-
Integration und authentisierte neue Dateiaufträge aus dem kombinierten Live-/Neustartablauf. Externe
COM-/Shell-/DNS-/Brokergrenzen und ein eigener ETW-Nachweis des neuen Paars bleiben offen. Keine
neue Netzfähigkeit; striktes Datenschutzziel und fehlende Produkt-Q-01/Q-02-Abnahme unverändert.

Primärquellen:
[DPAPI](https://learn.microsoft.com/en-us/windows/win32/api/dpapi/nf-dpapi-cryptprotectdata),
[Job-Lebensdauer](https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects),
[Prozesserzeugungsattribute](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-updateprocthreadattribute),
[Microsoft zur Jobzuweisung bei Prozesserzeugung](https://devblogs.microsoft.com/oldnewthing/20230209-00/?p=107812).
