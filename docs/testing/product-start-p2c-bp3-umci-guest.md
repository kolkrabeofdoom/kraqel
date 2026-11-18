# BP3 – Synthetische Gastprobe mit erzwungener UMCI

Der Referenzgast ist eine eigene Generation-2-VM mit Windows Server 2025 Standard Evaluation
(Desktop Experience). Hyper-V meldete vor dem Start null virtuelle Netzwerkadapter und null
aktivierte Integrationsdienste. Der Gast hat zwei virtuelle CPUs, 4 GiB Startspeicher und eine
eigene 64-GiB-Platte. Ein Prüfpunkt sichert den Zustand vor dem ersten Windows-Start. Die
synthetischen Dateien wurden bei ausgeschalteter VM in die Gastplatte kopiert. Beim Start waren das
Installationsmedium und die Host-Einbindung der Gastplatte ausgehängt.
[ADR 0035](../architecture/decisions/0035-bp3-independent-process-and-image-enforcement.md).

## Beobachtungen im Offline-Gast

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
   Es ergänzt Hashregeln ausschließlich für Starter, Worker und erlaubte DLL und kompiliert die
   resultierende CIP. Prüfe das XML, seine Windows-Plattformregeln und die referenzierten Testhashes
   vor der Übernahme. Die Windows-Basis enthält Plattformregeln von Microsoft. Diese Regeln bilden
   keine vollständige Einzeldateiliste der Plattform. Die beiden negativen Fixtures müssen außerhalb
   der ergänzten Allow-Regeln bleiben.
   [Policyregeln](https://learn.microsoft.com/en-us/windows/security/application-security/application-control/app-control-for-business/design/select-types-of-rules-to-create).
4. In einem frischen `C:\BP3-UMCI-Fixture` im Gast die sechs im Manifest genannten Dateien flach
   ablegen, zusammen mit `manifest.json`. Das Verzeichnis muss für die ausführende lokale
   Gastidentität beschreibbar und gegen fremde Änderungen geschützt sein. Der synthetische Marker
   bleibt für das AppContainer-Profil über den Dateipfad unzugänglich. Den Gast vor der
   Policyaktivierung auf seinen Ausgangszustand rücksetzbar machen. Die geprüfte CIP ausschließlich
   im autorisierten Gast mit CiTool aktivieren, erforderlichenfalls den Gast neu starten und
   Policyaktivierung in Code Integrity prüfen. Bei unbeaufsichtigter Aktivierung CiTool mit `-json`
   und abgeschlossenem Standardeingabestrom ausführen. Der erste Lauf hatte die Policy bereits
   geladen und wartete danach im verborgenen Setup-Fenster auf Enter. Für die Abnahme zählen der
   native Readback nach dem Neustart und die Ereignisse des eigentlichen Starterlaufs.
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
  explizite Ladevorgänge im laufenden Worker. Eine fremde statisch importierte Start-DLL ist eine
  separate Abnahme und bisher nicht enthalten. Der Handleunterschied ist durch Übergabeliste und
  Leseversuch nachgewiesen. Eine unabhängige Inspektion der Handle-Rechte im suspendierten
  Zielprozess steht aus.
- Der Starter prüft Workerexit und Jobleerstand. Ein tatsächlicher Starterabbruch, der Wiederanlauf
  bei geänderter Richtlinie und ein eigener Netzbeleg sind noch ausstehend. Ein erfolgreicher
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
