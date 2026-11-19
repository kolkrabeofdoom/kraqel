# BP3 – Synthetische Gastprobe mit erzwungener UMCI

Der Referenzgast ist eine eigene Generation-2-VM mit Windows Server 2025 Standard Evaluation
(Desktop Experience). Hyper-V meldete vor dem Start null virtuelle Netzwerkadapter und null
aktivierte Integrationsdienste. Der Gast hat zwei virtuelle CPUs, 4 GiB Startspeicher und eine
eigene 64-GiB-Platte. Ein Prüfpunkt sichert den Zustand vor dem ersten Windows-Start. Die
synthetischen Dateien wurden bei ausgeschalteter VM in die Gastplatte kopiert. Beim Start waren das
Installationsmedium und die Host-Einbindung der Gastplatte ausgehängt.
[ADR 0035](../architecture/decisions/0035-bp3-independent-process-and-image-enforcement.md).

## Wiederanlauf mit fehlender oder abweichender Prüfrichtlinie

**Bestanden am 19. November 2026.** Im ersten Prüffenster meldete Windows UMCI-Status 0 und keine
KRAQEL-Prüfrichtlinie. Im zweiten meldete Windows nach einem weiteren Neustart Status 2 und die
erzwungene ursprüngliche ID. Nur die Richtlinienversion war geändert. Der Einstieg sperrte im ersten
Fall wegen fehlender Durchsetzung, im zweiten wegen abweichender installierter CIP-Bytes.

Der schreibgeschützte Gegencheck der Gastplatte bestätigt in beiden Zeitfenstern je zwei
Kontrollprozessstarts und null Starterstarts anhand der Security-Ereignisse 4688. Ereignis 4608
belegt den Neustart zwischen den Fenstern. Der extern gesicherte Startanker blieb identisch. Der
Gast ist aus und seine zuvor verwendete virtuelle Platte wieder ausgewählt. Die bereinigte
Auswertung liegt lokal unter `artifacts/bp3-offline-guest/result-restart09/verified.json`.

Als Referenz dienen der separat gesicherte Startanker, die unveränderten acht Fixturedateien und der
reguläre Einstieg `run-guest.ps1` aus dem bestandenen Starterabbruch-Lauf. Beide Versuche verwenden
dieselbe Gastidentität und denselben erwarteten CIP-Hash. Der Prüfbereich ist dieser synthetische
Wiederanlaufpfad.

1. Im entbehrlichen Offline-Gast einen frischen Abzweig des Prüfpunkts vor der Installation der
   KRAQEL-Richtlinie verwenden. Fixturedateien, Startanker und Prüfskripte bereitstellen. Nur im
   Gast erfolgreiche Prozesserzeugungen protokollieren (Security-Ereignis 4688). Nach dem
   Gastneustart das Fehlen der erwarteten ID im CiTool-Inventar und der zugehörigen CIP auf der
   Gastplatte prüfen.
2. Aus der originalen Gast-XML zunächst den erwarteten CIP-Hash reproduzieren. Ausschließlich die
   Richtlinienversion erhöhen, die neue CIP im Testgast installieren und den Gast neu starten. UMCI
   muss erzwungen werden, die erwartete ID muss als aktiv erscheinen und die installierten Bytes
   müssen der neuen CIP entsprechen.
3. Pro Startversuch das folgende Skript im Gast mit `-Case Missing` bzw. `-Case Changed`,
   `-AnchorPath` und dem separat gesicherten `-AnchorSha256` ausführen. Es prüft den exakten
   Ablehnungsgrund und das Ereignisfenster zwischen zwei beobachtbaren Prozessstarts desselben
   Prüfprozesses. Ein Starterereignis, fehlende Kontrollen oder ein anderer Fehler lassen die
   Prüfung scheitern. Die Aufzeichnungen bleiben lokal.
   [Wiederanlaufprüfung](../../scripts/runtime-isolation/bp3-umci/test-restart-policy.ps1).

## Starterabbruch bei laufendem Blattprozess

**Bestanden am 19. November 2026.** Vor dem Abbruch war genau ein Blattprozess aktiv und hatte aus
seinem Programmeinstieg Bereitschaft gemeldet. Der Starter endete mit dem vorgegebenen Exit 73, der
Blattprozess mit Exit 0. Sein gehaltener Prozesshandle wurde innerhalb der Wartefrist von fünf
Sekunden signalisiert. Der äußere Job meldete anschließend `ActiveProcesses=0` und
`TotalProcesses=1`. Diese Messung fand vor der Bereinigung statt. Die bisherigen Rollen- und
Importkontrollen bestanden ebenfalls. Das Testprofil wurde entfernt und der Gast ausgeschaltet. Die
lokalen Belege liegen unter `artifacts/bp3-offline-guest/result-series07/`, der Manifest-SHA-256
lautet `a53099c2d9d3a40b89822bbfb98754e9509a0bf1388367fedca974154a2b9cbd`.

Im eigenen Abbruchfall startet der Beobachter eine weitere Instanz des geprüften Starters. Nur diese
Instanz hält den Handle des inneren Jobs mit `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`. Der Blattprozess
wird suspendiert und atomar dem inneren sowie einem äußeren Zähljob zugewiesen. Nach der Prüfung von
Job, Token und Mitigations wird er fortgesetzt. Im AppContainer signalisiert der Worker seine
Bereitschaft über einen einzeln vererbten Signalhandle und wartet bis zu 60 Sekunden.

Der Beobachter identifiziert den einzigen Prozess im Zähljob, öffnet und hält seinen Prozesshandle,
prüft Image, Jobmitgliedschaft und Lebendzustand und beendet den Starter gezielt mit Exit 73. Er
wartet auf beide Prozesshandles und liest die Buchhaltung des äußeren Jobs bei weiterhin offenem
Handle aus. Der reguläre Worker-Timeout hat einen eigenen Exitcode. Eine Bereinigung nach Fehler
kann den Test nicht bestehen lassen. Die Jobverschachtelung erlaubt die Schlussfolgerung vom leeren
äußeren Job auf den inneren Job, dessen letzter Handle beim Abbruch geschlossen wird.
[Windows-Jobhierarchie](https://learn.microsoft.com/en-us/windows/win32/procthread/nested-jobs).

## Statische Start-DLL · 19. November 2026

**Bestanden im Offline-Gast.** Der Positivimport erreichte den vorgesehenen Exit 50. Beim
Negativimport endete der Prozess nach Resume mit `0xC0E90002` (Systemintegritätsrichtlinie
verletzt). Das zugehörige Ereignis 3077 nennt `bp3-static-foreign.exe` und `foreign.dll`. Beim
schreibgeschützten Gegencheck der Gastplatte stimmten für alle sechs Sperrereignisse
Richtlinien-GUID, Prozesspfad, Zielpfad und vollständiger Zielhash. Davon entfiel eines auf die
statische DLL, vier auf explizite DLL-Ladevorgänge und eines auf die fremde EXE. Starterexit 0, Job
leer, Profil entfernt. Hyper-V bestätigte nach dem Lauf den ausgeschalteten Gast, null Netzadapter
und null aktivierte Integrationsdienste. Die lokalen Belege liegen unter
`artifacts/bp3-offline-guest/result-series06/`. SHA-256 des Fixturemanifests ist
`f1d26bda1b720288444d8cb6a0036a46b06de318c90aa00ab0f02367bc1fa518`.

Das Fixturemanifest v2 enthält acht Dateien. Zwei zugelassene fensterlose Programme werden aus
derselben Quelle gebaut und gegen unterschiedliche Importbibliotheken gelinkt. Ihre regulären
PE-Importtabellen enthalten `Bp3Marker` aus `allowed.dll` beziehungsweise `foreign.dll`. Der Starter
verwendet für beide denselben vorbereiteten Blattprozess ohne delegierten Datenhandle und prüft
Token, Job und Mitigations vor dem Resume. Der positive Kontrollstart muss den Export ausführen und
50 zurückgeben. Für das gesperrte Programm zählen ausschließlich ein nativer Integritätsfehler und
ein neues CI-Ereignis 3077 für seine statische Abhängigkeit. Der Runner gleicht dabei Zielpfad,
Prozesspfad, Richtlinien-GUID und vollständigen SHA-256 der DLL ab.
[Prüfprogramm](../../scripts/runtime-isolation/bp3-umci/static-import.c).

## Beobachtungen im Offline-Gast am 18. November

**Endstand vom 18. November 2026.** Die vorhandene synthetische Probe ist bestanden. Der
abschließende Lauf startete von einem frischen Abzweig des Prüfpunkts. Das Fixturemanifest hat
SHA-256 `56aa438921ed13d158f998de84bd05d9353ca1b78475eb275841f9ca1c0122e1`.

| Prüfung               | Ergebnis im abschließenden Gastlauf                                                                                                                                                                                                                                                   |
| --------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Startanker            | UMCI-Status 2, aktiv erzwungene Policy-ID, installierte CIP-Bytes und Fixturemanifest stimmen.                                                                                                                                                                                        |
| Rollen                | Derselbe Worker beendet sich als `reader` und `empty` mit dem vorgesehenen Erfolgscode 40. Nur `reader` liest das synthetische Byte über den übergebenen Handle. Beide Rollen prüfen die Sperre der direkten Dateiöffnung, der Kindstarts und des ausführbaren dynamischen Speichers. |
| Imagefreigabe         | Fremde EXE beim Start abgewiesen (Win32 4551). In beiden Rollen funktioniert die erlaubte DLL. Je ein Versuch vor und nach ihrem Laden sperrt die fremde DLL.                                                                                                                         |
| Unabhängige Kontrolle | Ein EXE- und vier DLL-Ereignisse 3077 im Messfenster. Auf der schreibgeschützt eingebundenen Gastplatte sind je Ereignis Richtlinien-GUID, vollständiger Zielpfad, Pfad der aufrufenden EXE und vollständiger SHA-256 der abgewiesenen Datei abgeglichen.                             |
| Ende                  | Starterexit 0, Job leer, Testprofil entfernt. Hyper-V bestätigt ausgeschalteten Gast, null virtuelle Netzadapter und null aktivierte Integrationsdienste.                                                                                                                             |

Die VM `KRAQEL-BP3-Offline` und die Rohbelege bleiben lokal. Der verifizierte Ergebnisstand liegt
unter `artifacts/bp3-offline-guest/result-series05/`, das zugehörige Prüfpaket unter
`artifacts/bp3-umci-fixture-05/`. Die Hostkonfiguration und die Ereignisse zu Ladesperren sind
geprüft. Ein eigener ETW-Netzvergleich gehört weiterhin zur offenen Abnahme.

### Befunde aus den Vorläufen

Auf Windows-Build 26100 lief der eigene, unsignierte Starter mit der im Gast erzeugten Richtlinie.
Vor dem Start bestätigte der Runner UMCI-Status 2, die erzwungene Policy-ID, den SHA-256 der
installierten CIP und die Fixturehashes. Die fremde EXE wurde mit Win32-Code 4551 abgewiesen. Im
interaktiven Gegenlauf stimmten Richtlinien-GUID, aufrufender Starter, Zielpfad und vollständiger
SHA-256 im Code-Integrity-Ereignis 3077 mit der Probe überein.

Der ursprüngliche Konsolenworker beendete sich als SYSTEM und in einer interaktiven Gastsitzung nach
dem Resume mit `0xC0000142` (DLL-Initialisierung fehlgeschlagen). Im KernelMode-Protokoll von
Security-Mitigations zeigt Ereignis 4 den vom Worker beantragten und abgewiesenen Start von
`conhost.exe`. Jobleerstand und Profilentfernung waren in beiden Läufen bestätigt. Die neue
Worker-Fixture verwendet deshalb das Windows-Subsystem ohne Konsole. Die Child-Policy gilt weiterhin
vor dem Resume.

## Vorbereitung und Gastablauf

1. Auf dem Entwicklungsrechner mit PowerShell und MSVC x64 die synthetischen Fixtures bauen. Die
   Ausgabe muss neu sein. Das Manifest enthält Größen und vollständige SHA-256-Werte.
   [Buildskript](../../scripts/runtime-isolation/bp3-umci/build.ps1).

   ```powershell
   pwsh -NoProfile -File scripts/runtime-isolation/bp3-umci/build.ps1 -OutputDirectory artifacts/bp3-umci-fixture
   ```

2. Für die Gastprobe eine entbehrliche Hyper-V-VM mit Windows x64 und CiTool verwenden. Vor dem
   Start von außen VM-Identität, fehlende virtuelle Netzwerkadapter, grundlegenden Konsolenmodus
   ohne Zwischenablage und Geräteumleitungen sowie fehlenden Gastdateikopierdienst und fehlende
   Hostfreigaben prüfen. Nur synthetische Prüfdateien über ein schreibgeschütztes Medium oder bei
   ausgeschaltetem Gast in die Gastplatte einbringen. Das Medium bzw. die Host-Einbindung vor dem
   Start aushängen. Gast-UUID und Build festhalten. Das Gastsystem soll keine Domänenbindung,
   persönlichen Konten, Schlüssel oder Projektdaten enthalten. Die Prüfskripte erfassen im Gast
   aktive Netzadapter. Hostkanäle erfordern zusätzlich die Prüfung am Hypervisor.
   [Hyper-V-Integrationsdienste](https://learn.microsoft.com/en-us/windows-server/virtualization/hyper-v/manage/Manage-Hyper-V-integration-services).
3. Im **Gast** mit Windows PowerShell
   `prepare-policy.ps1 -FixtureDirectory <Paket> -OutputDirectory <neuer Ordner>` ausführen. Das
   Skript verwendet das mit dem Gast gelieferte DefaultWindows-Enforced-Template als Plattformbasis.
   Es ergänzt Hashregeln für Starter, Worker, erlaubte DLL und in Manifest v2 beide statischen
   Importprogramme. Es kompiliert die resultierende CIP. Prüfe das XML, seine
   Windows-Plattformregeln und die referenzierten Testhashes vor der Übernahme. Die Windows-Basis
   enthält Plattformregeln von Microsoft. Diese Regeln bilden keine vollständige Einzeldateiliste
   der Plattform. Die beiden negativen Fixtures müssen außerhalb der ergänzten Allow-Regeln bleiben.
   [Policyregeln](https://learn.microsoft.com/en-us/windows/security/application-security/application-control/app-control-for-business/design/select-types-of-rules-to-create).
4. In einem frischen `C:\BP3-UMCI-Fixture` im Gast alle im Manifest genannten Dateien flach ablegen,
   zusammen mit `manifest.json`. Das Verzeichnis muss für die ausführende lokale Gastidentität
   beschreibbar und gegen fremde Änderungen geschützt sein. Der synthetische Marker bleibt für das
   AppContainer-Profil über den Dateipfad unzugänglich. Den Gast vor der Policyaktivierung auf
   seinen Ausgangszustand rücksetzbar machen. Die geprüfte CIP ausschließlich im autorisierten Gast
   mit CiTool aktivieren, erforderlichenfalls den Gast neu starten und Policyaktivierung in Code
   Integrity prüfen. Bei unbeaufsichtigter Aktivierung CiTool mit `-json` und abgeschlossenem
   Standardeingabestrom ausführen. Der erste Lauf hatte die Policy bereits geladen und wartete
   danach im verborgenen Setup-Fenster auf Enter. Für die Abnahme zählen der native Readback nach
   dem Neustart und die Ereignisse des eigentlichen Starterlaufs.
   [CiTool](https://learn.microsoft.com/en-us/windows/security/application-security/application-control/app-control-for-business/operations/citool-commands).
5. Im Gast `run-guest.ps1` mit der geprüften Gast-UUID, OS-Version, Policy-ID, CIP-SHA-256 und
   Manifest-SHA-256 starten. Der Einstieg vergleicht Gastidentität und Build, Arbeitsgruppe,
   UMCI-Modus, erzwungene Policy und installierte CIP-Bytes sowie Fixturehashes. Ankerwerte aus der
   vorherigen Prüfung übergeben. Die Prüfung kann bei abweichendem Windows-Build oder Zugriff auf
   Code Integrity geschlossen abbrechen. Der Runner erwartet lokale 3077-Ereignisse für beide
   negativen Images im Laufzeitfenster. Für den endgültigen Nachweis zusätzlich die Ereignisdetails
   mit der erwarteten Policy, dem exakten Image und der ausführenden Generation abgleichen.
   [App-Control-Ereignisse](https://learn.microsoft.com/en-us/windows/security/application-security/application-control/app-control-for-business/operations/event-id-explanations).

Alle freigegebenen und abgewiesenen Images sind synthetisch. Policy- und Ereignisdetails mit
Gastidentifikatoren bleiben lokal. Aus dem Gast nur bereinigte Statuscodes, Zählwerte und
Artefakthashes übernehmen.

## Eingebaute Prüfungen und offene Abnahme

- Der Starter prüft beim fremden EXE-Image die Win32-Fehler 577 (ungültiger Imagehash) oder 4551
  (Verletzung der Systemintegritätsrichtlinie). Ein unerwartet erzeugter Prozess bleibt suspendiert
  und wird beendet. Er zählt als fehlgeschlagene Prüfung.
- Derselbe Worker läuft in zwei Rollen mit AppContainer ohne Capabilities und atomarer Jobzuweisung.
  Vor Resume liest der Starter Job, AppContainer-SID, Capability-Anzahl, Child-Policy und
  Dynamic-Code-Policy zurück. Nur die Rolle `reader` erhält den lesbaren Markerhandle. Beide Rollen
  prüfen die verweigerte direkte Dateiöffnung und verweigerte Kindstarts über die dokumentierte
  Child-Policy. Der Worker probiert eine ausführbare Speicherallokation und verlangt die Ablehnung
  durch die Dynamic-Code-Policy.
- Der Worker erwartet beim expliziten Laden der fremden DLL ebenfalls 577 oder 4551, lädt die
  erlaubte DLL und prüft ihren Export. Anschließend prüft er die fremde DLL erneut. Dies sind
  explizite Ladevorgänge im laufenden Worker. Den statischen Startimport prüfen die beiden gesondert
  gelinkten Programme im v2-Gastlauf. Der Handleunterschied ist durch Übergabeliste und Leseversuch
  nachgewiesen. Eine unabhängige Inspektion der Handle-Rechte im suspendierten Zielprozess steht
  aus.
- Der Starter prüft Workerexit und Jobleerstand. Der Abbruchfall ist oben dokumentiert. Die beiden
  Wiederanlaufsperren sind oben dokumentiert. Ein eigener Netzbeleg steht aus. Ein erfolgreicher
  Gastlauf der vorhandenen Testfälle allein genügt daher nicht für die volle Abnahme nach ADR 0035.

Die Hostvalidierung umfasst den Build mit `/W4 /WX`, die Erzeugung der unaktivierten CIP aus der
lokalen Windows-Plattformbasis, Prüfung von Artefakthashes und Policyoptionen sowie Startabweisung
außerhalb des vorbereiteten Gasts. Die lokale CIP ist eine Vorschau. Für den realen Lauf muss die
Plattformbasis aus dem vorgesehenen Gast stammen. Auf dem Arbeitsrechner wurde keine Richtlinie
aktiviert. Die C4-Startsperre gilt weiterhin.

Die Ausgaben der ursprünglichen Hostvorbereitung liegen unter `artifacts/bp3-umci-fixture-03/` und
`artifacts/bp3-umci-policy-preview-03/`. Fünf unsignierte Images, sechs Fixturehashes, CIP- und
Manifesthash, die drei PowerShell-Dateien und drei abgewiesene Direkteinstiege wurden geprüft. Die
Build- und Policyverzeichnisse gehören nicht zum Git-Commit.
