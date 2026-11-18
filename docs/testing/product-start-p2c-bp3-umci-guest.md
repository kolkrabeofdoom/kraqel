# BP3 – Synthetische Gastprobe mit erzwungener UMCI

Der Starter und die Testimages sind gebaut. ConfigCI hat eine Vorschau der Policy als XML und CIP
erzeugt. Der native Gastlauf steht aus. Hyper-V läuft auf dem Arbeitsrechner, aber das aktuelle
Konto erhält beim Lesen des VM-Inventars eine Zugriffsverweigerung. Ein geeigneter, entbehrlicher
Windows-Gast und ein nutzbarer Gastzugang sind damit noch nicht belegt.
[ADR 0035](../architecture/decisions/0035-bp3-independent-process-and-image-enforcement.md).

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
   Hostfreigaben prüfen. Nur synthetische Dateien über ein schreibgeschütztes Medium einbringen und
   das Medium nach der Kopie aushängen. Gast-UUID und Build festhalten. Das Gastsystem soll keine
   Domänenbindung, persönlichen Konten, Schlüssel oder Projektdaten enthalten. Die Prüfskripte
   erfassen im Gast aktive Netzadapter. Hostkanäle erfordern zusätzlich die Prüfung am Hypervisor.
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
   Policyaktivierung in Code Integrity prüfen. Der Generator enthält keinen Aktivierungsbefehl.
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

- Der Starter erwartet beim fremden EXE-Image Win32-Fehler 577. Ein unerwartet erzeugter Prozess
  bleibt suspendiert und wird beendet. Er zählt als fehlgeschlagene Prüfung.
- Derselbe Worker läuft in zwei Rollen mit AppContainer ohne Capabilities und atomarer Jobzuweisung.
  Vor Resume liest der Starter Job, AppContainer-SID, Capability-Anzahl, Child-Policy und
  Dynamic-Code-Policy zurück. Nur die Rolle `reader` erhält den lesbaren Markerhandle. Beide Rollen
  prüfen die verweigerte direkte Dateiöffnung und verweigerte Kindstarts über die dokumentierte
  Child-Policy. Der Worker probiert eine ausführbare Speicherallokation und verlangt die Ablehnung
  durch die Dynamic-Code-Policy.
- Der Worker erwartet bei explizitem Laden der fremden DLL den Fehler 577, lädt die erlaubte DLL und
  prüft ihren Export. Anschließend prüft er die fremde DLL erneut. Dies sind explizite Ladevorgänge
  im laufenden Worker. Eine fremde statisch importierte Start-DLL ist eine separate Abnahme und
  bisher nicht enthalten.
- Der Starter prüft Workerexit und Jobleerstand. Ein tatsächlicher Starterabbruch, der Wiederanlauf
  bei geänderter Richtlinie und ein eigener Netzbeleg sind noch ausstehend. Ein erfolgreicher
  Gastlauf der vorhandenen Testfälle allein genügt daher nicht für die volle Abnahme nach ADR 0035.

Die Hostvalidierung umfasst den Build mit `/W4 /WX`, die Erzeugung der unaktivierten CIP aus der
lokalen Windows-Plattformbasis, Prüfung von Artefakthashes und Policyoptionen sowie Startabweisung
außerhalb des vorbereiteten Gasts. Die lokale CIP ist eine Vorschau. Für den realen Lauf muss die
Plattformbasis aus dem vorgesehenen Gast stammen. Auf dem Arbeitsrechner wurde keine Richtlinie
aktiviert. Die C4-Startsperre gilt weiterhin.

Die geprüften lokalen Ausgaben liegen unter `artifacts/bp3-umci-fixture-03/` und
`artifacts/bp3-umci-policy-preview-03/`. Fünf unsignierte Images, sechs Fixturehashes, CIP- und
Manifesthash, die drei PowerShell-Dateien und drei abgewiesene Direkteinstiege wurden geprüft. Die
Build- und Policyverzeichnisse gehören nicht zum Git-Commit.
