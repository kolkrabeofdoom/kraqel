# Native Win32-Fensterprobe im Offline-Gast

**19. November 2026.** Der neue isolierte UI-Prozess lief in der interaktiven Hyper-V-Basissitzung
unter erzwungener UMCI. Rollenentscheidung vor Resume, Tastatur, Fokus, erzeugte Narrator-Ansagen,
Prozessende und der ETW-Netzvergleich sind für die folgenden Programmbytes belegt. Die akustische
Hörprüfung bleibt offen.
[Architekturentscheidung](../architecture/decisions/0036-native-win32-ui-role-evaluation.md).

| Startfall                              | Beobachtung                                                                                                                                                                                                          |
| -------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Direkter Aufruf des UI-Programms       | Exit 20 vor der Fensterinitialisierung.                                                                                                                                                                              |
| Anfrage der unbekannten Rolle `writer` | Der Starter beendete die gehaltene, suspendierte Generation mit Exit 75. Der Job meldete anschließend null aktive und insgesamt einen Prozess.                                                                       |
| Zulässige Rolle `synthetic-edit`       | Readback am suspendierten Prozess erfolgreich. Nach Freigabe erschien das Fenster. Exit 40 bestätigt den exakten zweizeiligen Prüftext. Auch dieser Job endete mit null aktiven Prozessen bei insgesamt einem Start. |

Der Starter hielt die Binärdatei gegen Schreiben und Löschen offen. Er prüfte Imagepfad, aktive
Konsolensitzung, Erzeugungszeit, AppContainer-SID, null Capabilities, Kindstartsperre, gesperrten
dynamischen Code und den Job mit Prozesslimit eins und Beendigung beim Schließen. Die Erstellung
erfolgte ohne Handlevererbung. Erst danach schrieb er die Freigabe für die konkrete PID,
Erzeugungszeit und Sitzung unter `Program Files`. Die Authentizität dieser kleinen Probe beruht auf
der geschützten Windows-DACL. Nur SYSTEM und die lokale Gruppe der Administrierenden haben dort
Schreibrechte. Das UI prüft die eigene Generation und das Rechteprofil vor dem ersten Fenster. Der
schreibgeschützte Gegencheck nach dem Herunterfahren bestätigte DACL, entfernte Freigabedatei, alle
drei Binärdateien und den Hash der installierten Richtlinie.

Die Gast-Orchestrierung prüft die aktive Richtlinie und die Dateihashes vor dem Aufruf des Starters.
Zur Vertrauensbasis zählen diese Orchestrierung, SYSTEM und die Gastadministration. Der
Werkzeugstand erteilt keine Produktberechtigung und belegt keine durchsetzbare Grenze gegen die
Gastadministration.

## Tastatur und Sprachausgabe

Die Eingabe lief automatisiert über die virtuelle Tastatur der Gastkonsole. Im Feld standen `Rabe`
und `42` auf getrennten Zeilen. Tab wechselte zum Schließen-Button, die Leertaste löste ihn aus. Ein
separater UI-Automation-Client bestätigte nacheinander Typ, zugänglichen Namen und Tastaturfokus von
Feld und Button (Exit 0). Er liest keinen Feldinhalt aus.

Windows-Narrator lief im Gast. In seinem eigenen **Speech Recap** waren für das fokussierte Feld
„Prüftext, editing“ und für die Schaltfläche „Schließen, button“ sichtbar. Damit sind die erzeugten
Ansagen der englischsprachigen Gastinstallation belegt. Der Gast meldete null Audiogeräte und null
aktive Wiedergabeendpunkte. Lautstärke, Aussprache und Hörbarkeit sind unverifiziert. Host-Audio und
Zwischenablageumleitung waren für die Probe entbehrlich.
[Microsofts Narrator-Handbuch](https://support.microsoft.com/en-us/accessibility/windows/narrator/complete-guide-to-narrator).

### Akustische Hörprüfung · 19. November 2026

**Offen, vor dem Start angehalten.** Der gesicherte Gastbefund weist null aktive Wiedergabeendpunkte
aus. Beim Abgleich auf dem Prüfhost war genau ein lokaler Wiedergabeendpunkt aktiv. Er gehört dem
Host. Ein separater isolierter Prüfplatz mit direkt angeschlossenem Audioausgang stand ebenfalls
nicht zur Verfügung. Ein erneuter Fensterlauf fand deshalb nicht statt. VMConnect würde für
Host-Audio eine Ressourcenumleitung über die erweiterte Sitzung verwenden.
[Microsofts Dokumentation zur VMConnect-Ressourcenumleitung](https://learn.microsoft.com/en-us/windows-server/virtualization/hyper-v/use-local-resources-virtual-machine-connection).

Für die Wiederaufnahme zuerst den isolierten Wiedergabeweg und die unten aufgeführten Programmhashes
bestätigen. Am laufenden Fenster den Fokus per Tastatur vom Textfeld zur Schaltfläche wechseln. Eine
mithörende Person bestätigt für beide Fokuswechsel die tatsächlich gehörte Bezeichnung, die
erkennbare Rolle und die Verständlichkeit. Als Beleg genügt die Rückmeldung je Bedienelement; eine
Tonaufnahme ist dafür entbehrlich.

## ETW-Ergebnis

| Messgröße                                             | Referenz | Fensterlauf |
| ----------------------------------------------------- | -------: | ----------: |
| Ereignisse im Trace insgesamt                         |       86 |          95 |
| Prozesse im gemessenen Baum mit Start und Ende        |        3 |           8 |
| TCP-/UDP-Ereignisse im gemessenen Baum                |        0 |           0 |
| TCP-/UDP-Ereignisse gastweit, beide Adressen Loopback |       18 |          18 |
| Erkannte Positivkontrollen (TCP/UDP über IPv4/IPv6)   |      4/4 |         4/4 |
| Verlorene Ereignisse oder Puffer                      |        0 |           0 |

Beide Fenster prüften die Gast- und Paketanker. Die Referenz wartete anschließend kurz, der
Fensterlauf enthielt Direktaufruf, Starter, beide Jobgenerationen und den UIA-Beobachter. Die
Auswertung ordnete Eltern, Lebenszeiten und die drei Blatt-Exitcodes im Trace zu. Beim Blatt gab es
keinen beobachteten Kindstart. Gastweite TCP-/UDP-Metadaten decken für diese Fenster auch Aktivität
außerhalb des gemessenen Baums ab. Daraus folgt keine Freigabe für den normalen Desktop oder für
echte Projektdaten. Der Gast ist aus, die ursprüngliche Gastplatte wieder ausgewählt. Hyper-V
bestätigte null virtuelle Netzadapter und null aktivierte Integrationsdienste.

## Wiederholung und Prüfarbeitsstand

1. Im Forschungsunterverzeichnis `scripts/runtime-isolation/native-window` mit `build.ps1` die
   beiden Rust-Programme und den nativen UIA-Beobachter erstellen. Die drei Dateien anhand der
   vollständigen Hashes an die jeweilige Messung binden.
2. Eine neue Differenzplatte des ausgeschalteten BP3-Gasts verwenden. Im Overlay die erzwungene
   Gastpolicy um die Hashregeln genau dieser drei Images ergänzen, Richtlinien-ID und
   Durchsetzungsoptionen prüfen. Die Dateien im frischen Verzeichnis
   `C:\Program Files\KRAQEL-Window-Probe` ablegen. Geschützte, vererbbare DACL mit Vollzugriff für
   SYSTEM und Administrierende, Lesen/Ausführen für Users und All Application Packages, Eigentümer
   Administrierende. Die lokale `anchor.json` enthält Gast-UUID, Build, Policy-ID, CIP-SHA-256
   (Felder `uuid`, `build`, `policy`, `policyHash`) sowie `files` mit `name` und `hash` je Image.
   Das Ausgangsabbild bleibt ausgeschaltet und unverändert.
3. Im Overlay den vorhandenen BP3-ETW-Runner und `guest-workload.ps1` als
   `C:\BP3-Setup\etw-workload.ps1` verwenden. Im frischen Overlay `C:\BP3-ETW` anlegen. Aus der
   angemeldeten Gastsitzung mit erhöhtem Gasttoken `etw-guest.ps1` mit den geprüften Werten für
   `GuestUuid`, `GuestBuild`, `PolicyId` und `PolicySha256` starten. Im Fenster ausschließlich den
   genannten Prüftext verwenden. Für die Narrator-Gegenprüfung den lokalen Verlauf mit
   Narrator-Taste+Alt+X anzeigen. Die Auswertung übernimmt keine Mitschrift und keine privaten
   Identifikatoren.
4. Nach Gastende beide ETLs mit `tracerpt -of XML -lr` als `reference.xml` und `exercise.xml`
   dekodieren. Im Checkout `python scripts/m3-network/bp3_guest.py --native-window <Ergebnisordner>`
   ausführen und die beiden Launcherprotokolle, den Fokusbefund und die Abschaltung gegenprüfen.

| Gemessene Datei      | SHA-256                                                            |
| -------------------- | ------------------------------------------------------------------ |
| `native-starter.exe` | `90c98b22a8a069467be147a61666b2ab4e4ca462b6287c72e7852ed325ca9e51` |
| `native-leaf.exe`    | `3d61f7d96235a77a5666a7b4d96fe638ddbff608e922f1616475d03910d5ef4a` |
| `observer.exe`       | `31780e67f710499d5bbfc63169b254d2ca7a5c4bf88f713fe73ceab24f7a0fd6` |

Die lokalen Rohbelege liegen unter `artifacts/bp3-offline-guest/result-windowd/`. Während der
Entwicklung wurde die Jobleerprüfung auf den asynchronen Abschluss der Windows-Buchhaltung
abgestimmt und Tab für die Navigation im mehrzeiligen Feld reserviert. Die Tabelle enthält nur den
abschließenden Lauf. Die Produkt-UI-ABI, C4- und BP4-Freigabe bleiben offen.
