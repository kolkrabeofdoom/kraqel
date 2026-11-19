# ADR 0036 – Native Windows-Oberfläche mit eigener Prozessrolle

**Ergebnis der anschließenden Gastprobe:** Das eigene Win32-Fenster mit gebundener Rolle lief unter
erzwungener UMCI. Tastatur, die im Narrator-Sprachverlauf sichtbaren Ansagen, Jobende und
ETW-Vergleich sind geprüft. Die akustische Abnahme und die Produktintegration bleiben offen.
[Messwerte und Beleggrenzen](../../testing/native-win32-window-guest.md).

Für die Fensterprobe ist der Softwarebeleg der Sprachausgabe als bestanden angenommen. Die
akustische Prüfung ist zurückgestellt. Der synthetische BP4b-Kanal zwischen UI-Blatt und Rust-Datendienst ist abgeschlossen. Die weitere Integration betrifft den nativen C4b-Kandidaten.
[BP4-Datenvertrag](../../data-format/native-win32-bp4-scene-v1-draft.md).

**19. November 2026 · Als Kandidat für eine Machbarkeitsprobe ausgewählt.** Eine eigene, native
Windows-EXE mit einem Unicode-Textfeld der Fensterklasse `EDIT` ist für KRAQEL prüfenswert. Das
Fenster und seine Steuerelemente können im selben UI-Prozess liegen. Damit kann der
vertrauenswürdige Starter jede von KRAQEL benötigte Anwendungsprozessrolle selbst vergeben. Ein
späterer Szenenwechsel benötigt architektonisch keinen neuen Prozess. Die Entscheidung betrifft eine
Forschungsprobe. Ein Wechsel der Produktoberfläche bedarf einer eigenen Entscheidung.
[Win32-Edit-Steuerelemente](https://learn.microsoft.com/en-us/windows/win32/controls/about-edit-controls),
[Ausgangspunkt mit WebView2](0035-bp3-independent-process-and-image-enforcement.md).

## Wer darf was starten?

- **Starter.** Er gehört zur vertrauenswürdigen Basis. Für die Probe kennt er genau eine zulässige
  UI-Rolle mit festem Image und Rechteprofil. Er prüft Paket und erzwungene Gast-Imagepolicy und
  erzeugt das Blatt suspendiert mit AppContainer ohne Capabilities, Ein-Prozess-Job,
  Kindstartsperre, gesperrtem dynamischem Code und ohne geerbte Projekt-, DB- oder Schlüsselhandles.
  Vor Resume prüft er Imageidentität, Token, Job, Mitigations und die für diese Rolle erlaubten
  Handles an der gehaltenen Prozessgeneration. Bei einer Abweichung beendet er sie suspendiert. Ein
  Rollenname in einer Nachricht verleiht keine Rechte.
- **UI-Blatt.** Es verarbeitet Fenster, Tastatur und synthetischen Text im eigenen Adressraum. Vor
  dem ersten Fenster muss es den eigenen Start an einen einmaligen, authentisierten Auftrag für
  genau diese Prozessgeneration binden. Ein Direktstart oder wiederholter Auftrag endet vor der
  UI-Initialisierung. Die Betriebssystemrechte sind bereits vor Resume festgelegt. Es darf weder
  Hilfsprozesse erzeugen noch einen allgemeinen Startauftrag an den Starter delegieren. Jeder
  künftig benötigte KRAQEL-Hilfsprozess braucht einen eigenen geprüften Startvertrag. Das
  Datenmodell sieht später einen getrennten Rust-Dienst vor. Für die erste Probe werden weder
  Dienstzugang noch Schreibbefugnisse benötigt.
- **Windows-Dienste.** Desktop, Eingabemethoden, Zwischenablage, UI Automation und
  Absturzberichterstattung liegen teilweise außerhalb des KRAQEL-Prozessbaums. KRAQEL vergibt ihnen
  keine Rollen. Ihre Daten- und Netzwege müssen für die unterstützte Produktkonfiguration gesondert
  bewertet werden. Die Offline-Gastgrenze schützt die Probe, ersetzt jedoch keine durchsetzbare
  Auslieferungsgrenze auf dem üblichen Desktop.

Die vorhandene BP3-Gastprobe belegt suspendierte eigene Rollenstarts, die effektiven Rechte am
synthetischen Marker, die Abweisung einer falschen Zugriffsmaske sowie das Ende des Blatts nach
Starterabbruch. Der Nachweis betrifft die damaligen Binärdateien. Beim neuen UI-Blatt sind
insbesondere interaktive Sitzung, Fensterstation, Desktopzugriff und die effektive Modulmenge unter
erzwungener UMCI offen. Die bisherige Gast-Startaufgabe unter SYSTEM eignet sich allein nicht zur
Abnahme eines bedienbaren Fensters.
[Gastnachweis](../../testing/product-start-p2c-bp3-umci-guest.md),
[Windows-Erzeugungsattribute](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-updateprocthreadattribute).

## Was für diesen Kandidaten geprüft ist

Ein lokaler C-Linkversuch mit MSVC x64, statischer C-Laufzeit und `/W4 /WX` war erfolgreich. Der
Einstieg verwendet `CreateWindowExW` für die Klasse `EDIT` mit `ES_MULTILINE` und `DestroyWindow`.
`dumpbin /headers /imports` meldete das Windows-GUI-Subsystem, genau `USER32.dll` und `KERNEL32.dll`
als direkte DLL-Importe und ein leeres Delay-Importverzeichnis. Der Versuch liegt lokal unter
`artifacts/native-ui-alternative-20261119/`. Die EXE wurde ausschließlich gebaut und statisch
geprüft. Spätere DLL-Ladevorgänge, tatsächliche Kindstarts, Fensterbedienung, Barrierefreiheit und
Netzverhalten erfordern einen Lauf im Gast.

## C4b · Paketbindung im nativen Prüfstart

Der [Prüfstarter](../../../scripts/runtime-isolation/native-window/src/bin/pipe-check.rs) hält das geschützte Paketverzeichnis, den Paketanker und die vier fest vorgegebenen EXE-Dateien über die Lebenszeit des Auftrags offen. Der [native Paketabgleich](../../../scripts/runtime-isolation/native-window/src/package.rs) prüft jeden SHA-256-Wert am gehaltenen Handle, bevor der Starter Kontrollablage oder Rollen öffnet. Schreib- und Löschfreigaben fehlen. Verzeichniseinträge mit Reparse-Attribut, ein unvollständiges Inventar und abweichende Programmbytes sperren den Start. Vertrauensbasis sind das vom Gastaufbau geschützte Paketverzeichnis, sein Anker und die erzwungene UMCI-Richtlinie.

Ein gezielter Windows-Test bestätigte die Sperre für Schreiben und Umbenennen während der Bindung sowie die Abweisung nach einer Änderung der Datei. Ein regulärer Szenenabruf im Offline-Gast bestätigte den neuen Einstieg mit erzwungener Richtlinie (Datendienst 66, UI 67, Starter 0, beide Jobs leer). Die schreibgeschützte Nachprüfung bestätigte Programm- und Richtlinienhashes sowie entfernte Rollenbelege. Die Anbindung an die aktive verwaltete Broker-Generation ist im folgenden Abschnitt beschrieben. Die Produkt-UI-ABI bleibt bei 0.

### Aktive Generation im Produktstarter

Der [Vorschau-Produktstarter](../../../scripts/runtime-isolation/probe/src/windows/exchange/ui_restart/durable/product/win32_binding.rs) bindet das Paket unter der exklusiven Sperre `control/owner.lock`. In einem Kontrollstore mit Schema 4 muss der laufende Broker dieselbe Datei wie die aktivierte, hashbenannte EXE in `bin` sein. Ein vorbereiteter Updatekandidat, ein überholter Kontrollkopf oder eine zweite Bindung derselben Generation sperren die Aufnahme. Ein frisch initialisierter Store benötigt zuerst eine regulär aktivierte Broker-Generation.

| Aufruf über die aktive verwaltete EXE | Wirkung |
| --- | --- |
| `broker-store bind-win32-package <lokales-Paketverzeichnis> <erwarteter-Anker-SHA256> <aktueller-Kopf-SHA256>` | Prüft den unabhängig bereitgestellten Hash von `anchor.json` und die vier festen Rollenimages an gehaltenen Handles. Veröffentlicht die geprüfte Kopie per Verzeichnisumbenennung unter `bin/win32-<aktiver-Broker-SHA256>` und bindet ihren Anker einmalig durch einen authentisierten Kontrollrecord. |
| `broker-store verify-win32-package <aktueller-Kopf-SHA256>` | Öffnet unter derselben Sperre den authentisierten Verlauf, die aktive Brokerdatei und das private Rollenpaket erneut. Erwarteter Anker und Zielverzeichnis folgen aus dem aktiven Zustand. |

Ein Abbruch beim Kopieren kann ein privates temporäres Verzeichnis hinterlassen. Veröffentlichen allein erteilt keine Bindung. Eine Wiederholung prüft eine bereits veröffentlichte Kopie erneut. Bei Aktivierung eines anderen Brokers verliert die frühere Bindung ihre Gültigkeit. Ein Folgeupdate muss den Recordtyp unterstützen (PE-Abschnitt `.kqwin`), sobald er im Verlauf vorkommt.

**Nachweis unter Windows:** Der gezielte Kontrollstore-Test prüfte falschen Anker, überholten Kopf, geänderten verwalteten Anker, Update-Pending, inkompatiblen Updatekandidaten und Generationswechsel. Ein separater Aufruf über die verwaltete Vorschau-EXE bestätigte Aufnahme und Wiederöffnung mit den vier Binärdateien der Win32-Gastprobe und einem synthetischen lokalen Inventaranker. Die bytegleiche externe Brokerkopie wurde abgewiesen. Der reine Verifikationsaufruf gibt seine Handles beim Rücksprung frei. Der suspendierte Rollenstart führt die Prüfung im eigenen Aufruf erneut aus. Produktdaten bleiben gesperrt.

### Suspendierte Fensterrolle im verwalteten Starter

`broker-store check-win32-suspended <aktueller-Kopf-SHA256>` ist der Windows-Vorschaupfad für genau die gebundene Rolle `native-leaf.exe`. Er hält `owner.lock`, den verwalteten Broker, die Verzeichnisse, den Paketanker und die vier geprüften Images über Prozesserzeugung, Identitätsprüfung und Beendigung. Der Kernel erzeugt den primären Thread suspendiert und ordnet den Prozess sofort dem frischen Einprozess-Job mit Kill-on-close zu. Der Prozess erhält ein eigenes AppContainer-Profil ohne Capabilities, eine begrenzte Umgebung und �ber die explizite Handleliste genau das Leseende einer anfangs leeren anonymen Pipe als Standardeingabe.

Vor einem möglichen Resume prüft der Starter die Zugehörigkeit von Prozess und primärem Thread, das erwartete Token, das Verbot von Kindstarts und dynamischem Code, die Jobmitgliedschaft und die wirksamen Joblimits. Eine zweite Öffnung der Kontrollsperre muss an der exklusiven Belegung scheitern. Der über das Prozesshandle ermittelte Imagepfad und die Volume- und 128-Bit-Datei-ID müssen zur gebundenen Fensterdatei passen. Der Prüfaufruf beendet den Job bei Erfolg und Abweisung ohne Resume. Ein erfolgreicher Abschluss setzt ein signalisiertes Prozessende, einen leeren Job und die Entfernung des Prüfprofils voraus.

**Sicherheitsnachweis unter Windows.** Über die aktivierte verwaltete EXE lief der Einstieg mit den vier Programmbytes der Gastprobe in einer separaten synthetischen Vorschauablage. Die Fensteridentität war bestätigt, der Kontrollkopf unverändert und der Job am Ende leer. Eine gezielte Gegenprobe mit einem gehaltenen Handle auf eine bytegleiche zweite EXE scheiterte beim Datei-ID-Abgleich. Ein erster Entwicklungsversuch mit leerer Umgebung scheiterte bereits bei der Prozesserzeugung (Windows-Fehler 203). Der bestätigte Lauf verwendete den bestehenden begrenzten Umgebungsblock. Produktdaten bleiben gesperrt.

### Bootstrap vor Resume

Erst nach der Identitätsprüfung schreibt der Starter ein 32-Byte-Ticket in die Pipe. Der HMAC-SHA256 bindet die versionierte Fensterrolle, Installation, Kontrollkopf, aktiven Broker, Paketanker, Volume- und Datei-ID sowie Prozess-ID, primäre Thread-ID und Prozess-Erzeugungszeit. Der Kontrollschlüssel verbleibt beim Starter. Unter derselben Sperre dupliziert der Starter das Lesehandle aus dem suspendierten Zielprozess zurück und prüft Länge und MAC der dort bereitliegenden Bytes, ohne sie zu verbrauchen. Das Ticket erteilt keine Datenrechte.

Der verwaltete Windows-Vorschaustart mit synthetischer Ablage bestätigte den Rücklesepfad, unveränderten Kontrollkopf und leeren Job nach der Beendigung. Bei unveränderten numerischen Prozess- und Thread-IDs genügte eine abweichende Erzeugungszeit, um das frühere Ticket abzuweisen.

### Kontrollierter Resume und Empfangsbestätigung

Der getrennte Einstieg `broker-store check-win32-receipt <aktueller-Kopf-SHA256>` gibt der Fensterrolle zusätzlich nur das Schreibende einer zweiten anonymen Pipe als Standardausgabe. Nach erfolgreicher Ticketprüfung schließt der Starter sein Eingabe-Schreibende und setzt den primären Thread genau einmal fort. Im Empfangsmodus liest die Rolle das Ticket, sendet dessen domänengetrennten SHA-256-Empfangsbeleg und endet vor dem UI- oder Datenzugriff. Der Starter hält die Kontrollsperre und Pakethandles bis zur Beendigung und Bereinigung. Er verlangt binnen fünf Sekunden das Prozessende, Exitcode null und genau die erwarteten 32 Antwortbytes.

Der lokale Windows-Lauf über die aktivierte verwaltete EXE bestätigte den Empfang, unveränderten Kontrollkopf und leeren Job. Eine separat gebundene synthetische Gegenrolle endete erfolgreich mit einer gleich langen falschen Antwort. Der Starter wies sie beim Belegvergleich ab. Der vorhandene Datei-ID-Test prüft jetzt den Pfad mit angefordertem Resume.

### Synthetisches Fenster nach bestätigtem Empfang

`broker-store check-win32-surface <aktueller-Kopf-SHA256>` hält die Kontrollsperre und die geprüften Dateien bis zum Ende des Fensterprozesses. Das UI-Ticket enthält den Hash eines generationsgebundenen Öffnungswerts. Erst nach erfolgreicher Prüfung der Antwort auf das Ticket sendet der Starter diesen Wert über die bestehende Pipe. Die Rolle prüft zusätzlich ihr AppContainer-Profil, die fehlenden Capabilities, die Prozessbeschränkungen und die Jobzugehörigkeit. Erst wenn der empfangene Öffnungswert zum Ticket passt, ruft sie den Fensteraufbau auf.

Im lokalen Windows-Lauf zeigte die geprüfte Rolle den festen Zweizeiler im schreibgeschützten Textfeld. Der Starter prüfte Sichtbarkeit, Fensterklasse und Zugehörigkeit zum primären Thread und Prozess am gemeldeten HWND. Er schloss das Fenster per WM_CLOSE und bestätigte reguläres Prozessende sowie leeren Job. Der Kontrollkopf blieb unverändert. Eine gezielte Regression am Handshake der Rolle quittierte das Ticket und wies anschließend einen fremden Öffnungswert ab.

### ETW-Netzvergleich der verwalteten Fensterrolle

Am 19.11.2026 liefen auf dem lokalen Windows-Prüfplatz zwei getrennte ETW-Erfassungen für Code-Stand `2d767af`. Beide verwendeten dieselbe synthetische Ablage und dieselben gebundenen Dateien. SHA-256 des tatsächlich gestarteten hashbenannten Starters war `c9116cd4abec02b2c4836793b4fa95c3e4c9436f3c2297290fb200236f2a98e6`, SHA-256 von `native-leaf.exe` war `7cdb0553a8816637ffca4f6bc3ff698ff3adc3dc9ea51aac9980e12767ed1724`.

| Messpfad | Vollständige Prozesslebenszeiten | Loopback-Positivkontrollen | Verlorene Ereignisse / Puffer | Netzwerkereignisse im Prozessbaum |
| --- | ---: | ---: | ---: | ---: |
| `check-win32-receipt`, Bootstrap ohne Fenster | 2 | 4/4 | 0 / 0 | 0 |
| `check-win32-surface`, Freigabe, sichtbares Fenster und reguläres Schließen | 2 | 4/4 | 0 / 0 | 0 |

Der vorhandene Auswerter ordnete den Starter über Start-PID und vollständigen EXE-Basisnamen zu. Die Prozessereignisse belegten für beide Läufe genau eine direkte Fensterrolle und beide Prozessenden innerhalb der Erfassung. Die Positivkontrollen umfassten je TCP und UDP über IPv4 und IPv6. Es gab keine nicht zuordenbaren Netzwerkereignisse. Beide Aufrufe bestätigten denselben Kontrollkopf, gesperrte Produktdaten und einen leeren Job am Ende. Die Aussage gilt für Kernel-Netzwerkereignisse dieser Prozesslebenszeiten. Eigenständige Windows-Dienste außerhalb des Prozessbaums liegen außerhalb dieser Zuordnung. Die Rohspuren verbleiben lokal.

### C4c – Tastaturbedienung im verwalteten Fenster

`broker-store open-win32-surface <aktueller-Kopf-SHA256>` öffnet das synthetische Fenster über dieselbe Bootstrap-Freigabe wie die C4b-Fensterprobe. Der Starter hält Kontrollsperre, Paketdateien und Job bis zum Schließen. Das schreibgeschützte Feld mit dem festen Zweizeiler erhält zuerst den Fokus. Tab und Umschalt+Tab wechseln zwischen Feld und Schaltfläche. Enter auf „Schließen“ oder Escape im Fenster beendet die Sitzung.

Im Windows-Lauf wurden Tastenmeldungen gezielt an die Steuerelemente der geprüften Prozessgeneration gesendet. Der Fokus wechselte vom schreibgeschützten Feld zur Schaltfläche und zurück. Enter auf der Schaltfläche beendete den Prozess regulär. Ein zweiter Starter erhielt beim Kontrollzugriff während der Sitzung einen Freigabefehler (Windows-Code 32). Nach dem Schließen waren der Job leer, der Kontrollkopf unverändert und die Ablage für den Starter wieder zugänglich. Produktdaten blieben gesperrt.

### Softwareprüfung von Fokus, Beschriftung und Sprachausgabe

Am 19.11.2026 lief `open-win32-surface` mit Stand `89e73de` und einer frischen synthetischen Ablage. Der lokale Beobachter ordnete das Fenster über seinen direkten Starterprozess und die verwaltete EXE-Datei zu. Er las ausschließlich die beiden Steuerelemente dieser Fensterinstanz. Im Fensterthread lag der Fokus zuerst auf dem schreibgeschützten Feld und nach einer gezielten Tab-Tastenmeldung auf der Schaltfläche. Native Windows UI Automation lieferte den Namen „Prüftext“, den Typ Edit, den Nur-Lese-Wert und den erwarteten Zweizeiler. Für die Schaltfläche lieferte sie den Namen „Schließen“ und den Typ Button.

Die lokal installierte deutsche SAPI-Stimme Microsoft Hedda Desktop synthetisierte die geprüften Namen und lokalisierten Rollen sowie beim Feld den synthetischen Inhalt. Für beide Ausgaben entstanden im Arbeitsspeicher mindestens 100 ms PCM mit Ausschlägen über der Prüfschwelle (16 kHz, 16 Bit, mono). Der Beobachter verwendete keinen Wiedergabeausgang und speicherte keine Audiodatei. Nach dem Schließen bestätigte der Starter den unveränderten Kontrollkopf, gesperrte Produktdaten und den leeren Job.

**Beleggrenze.** Das automatisiert geöffnete Fenster war im erfolgreichen Lauf im Hintergrund. `GetGUIThreadInfo` belegte den Fokus innerhalb seines Threads. UI Automation meldete für beide Elemente keinen desktopweiten Tastaturfokus. Der Softwarebeleg umfasst die zugänglichen Inhalte und deren lokale Synthese. Der anschließende Vordergrundlauf prüfte die automatisch erzeugten Ansagen direkt in Narrators Sprachverlauf.

### Narrator beim Tab-Wechsel im Vordergrund

In einem frischen verwalteten Lauf mit synthetischer Ablage wurde das Prüffenster nach bestätigtem Bootstrap in den Vordergrund geholt. Nach dem Fokuswechsel per Tab bestätigte die testende Person in Narrators eigenem Sprachverlauf die Reihenfolge „Prüftext“, dann „Schließen“ als Schaltfläche. Damit ist die automatische Ansage für diesen Bedienablauf softwareseitig bestätigt. Die unabhängige Fokusabfrage lieferte in diesem Lauf keine Stichprobe, die zugleich Vordergrundfenster, Steuerelement und laufenden Narrator erfasste. Der Beleg für die Ansage ist die Sichtprüfung des Sprachverlaufs.

Der verwaltete Starter bestätigte beim Abschluss den unveränderten Kontrollkopf, gesperrte Produktdaten und den leeren Job. Ins Repository gelangte ausschließlich das Ergebnis. Für diese Prüfung wurden weder Sprachverlauf noch Screenshot oder Audio übernommen.

### C4c-ETW-Netzvergleich beim Fokuswechsel

Am 19.11.2026 lief `open-win32-surface` mit identischen Binärdateien in frischen synthetischen Ablagen. Der Messhelfer adressierte Tab an das fokussierte Feld der gebundenen Fensterrolle. In der Referenz lief Narrator nicht. Im Vergleich wurde der laufende lokale Windows-Narrator geprüft. Unmittelbar vor und nach Tab gehörte das Vordergrundfenster der Fensterrolle, und der Fokus ihres Threads wechselte von „Prüftext“ zu „Schließen“. Ein erster Narrator-Versuch mit Vordergrundwechsel während der Nachbeobachtung wurde verworfen. Nur diese Phase wurde wiederholt.

| Gültige Phase | Prozesslebenszeiten mit Start und Ende | TCP-/UDP-Kontrollen über IPv4 und IPv6 | Verlorene Ereignisse / Puffer | Netzwerkereignisse von Starter und Fensterrolle |
| --- | ---: | ---: | ---: | ---: |
| Tab ohne Narrator | 2 | 4/4 | 0 / 0 | 0 |
| Tab mit aktivem Narrator im KRAQEL-Vordergrundfenster | 2 | 4/4 | 0 / 0 | 0 |

SHA-256 des verwalteten Starters war `b838511b155ca62ac18855957515b6506eaef42299cf704c2d3f5b96b124e8f1`, die der Fensterrolle `01e907b71328219456a23dac4cb39d7b88e3e69ee28d1a2614ec22a64b22212d`. Die ETW-Auswertung band die Starter-PID an den vollständigen EXE-Namen und bestätigte die direkte Kindrolle. Beide Aufrufe meldeten bestätigten Bootstrap, gesperrte Produktdaten, unveränderten Kontrollkopf und leeren Job nach dem Schließen. Die Aussage gilt für den KRAQEL-Prozessbaum. Narrators eigener Netzverkehr und der anderer Windows-Dienste liegen außerhalb dieser Zuordnung. Der Sprachverlauf wurde für den Netzvergleich nicht ausgelesen, Rohspuren blieben lokal.

### C4d · Abnahmeentscheidung für das verwaltete Win32-Paket

**C4c ist für die schreibgeschützte synthetische Win32-Oberfläche abgeschlossen. C4d hat diesen Paketkandidaten zur weiteren Freigabe abgelehnt.** Der Umfang von C4c umfasst den authentisierten Fensterstart, die Tastaturbedienung, die bestätigten Narrator-Ansagen und das reguläre Ende. Der [ursprüngliche C4-Vertrag](../../data-format/product-start-p2c-c4-v1-draft.md#6-umsetzung-und-mittleres-prüfniveau) enthält zusätzlich Dokumentbefehle und synthetische SQLite-Persistenz. Deren Integration in diesen verwalteten Fensterpfad bleibt offen. Die separaten BP4b-Gastbefunde gelten für die dort geprüften Rollen und Binärstände.

**Identität am 19.11.2026.** Der aktivierte, hashbenannte Starter hat das neu bereitgestellte Paket in einer frischen synthetischen Kontrollablage über `verify-win32-package` verifiziert. Kontrollkopf und Paketbindung stimmten überein. Der Aufruf startete keine Rolle und erteilte keine Datenfreigabe. Die SHA-256-Werte von Starter und Fensterrolle stimmen mit dem [C4c-Netzvergleich](#c4c-etw-netzvergleich-beim-fokuswechsel) überein. Für die übrigen Paketdateien belegt der Inventarabgleich die Identität im aktuellen Abnahmelauf.

| Bestandteil | SHA-256 |
| --- | --- |
| Verwalteter Starter | `b838511b155ca62ac18855957515b6506eaef42299cf704c2d3f5b96b124e8f1` |
| anchor.json | `52b4dc1f1a1dd28293e8c10e0613e034646907d1731db3f6e6c92d1c8fcc340f` |
| native-leaf.exe | `01e907b71328219456a23dac4cb39d7b88e3e69ee28d1a2614ec22a64b22212d` |
| pipe-check.exe | `5b2f019d340cab2fc7426137ea1a81b10df12defdb59a8bee6f499406b835f35` |
| pipe-other.exe | `3c13c886b02af2541ab6e2918c408e121e0b3a2f6c8f806f8795fd28ba36974f` |
| pipe-service.exe | `3dd1518dcfbd5c89dd7a31dbac8b812b3f25a4f28823813fb468dffabef9b70f` |

**Offenes Abnahmekriterium.** Nach einem Wächterverlust verlangt der Vertrag eine dauerhafte Sperre, bis ein vertrauenswürdiger Endebeleg vorliegt. Die [Paketbindung](../../../scripts/runtime-isolation/probe/src/windows/exchange/ui_restart/durable/product/win32_binding.rs) protokolliert die Paketaufnahme. Der [verwaltete Fensterstart](../../../scripts/runtime-isolation/probe/src/windows/exchange/ui_restart/durable/product/win32_binding/suspended.rs) führt die Sitzung unter exklusiver Sperre und im Kill-on-close-Job, schreibt aber keinen authentisierten Start-/Endzustand für diese Fensterprozessgeneration. Nach Wegfall des Jobs liefert die Leerlaufprüfung keinen dauerhaften Beleg, ob die frühere Sitzung regulär abgeschlossen wurde. Die belegte Sperre eines zweiten Starters bei lebender Sitzung deckt diesen Fall nicht ab. Dieser Befund stammt aus dem Startpfad im Quellcode. Ein Wiederanlauf nach Crash wurde in dieser Abnahme nicht ausgeführt.

Für den nächsten Kandidaten muss der Starter die ausstehende Sitzung unter derselben Kontrollsperre vor der Prozesserzeugung dauerhaft vermerken und erst nach bestätigtem Ende abschließen. Ein unaufgelöster Eintrag muss die nächste Fenstererzeugung sperren. Dann sind der gezielte Abbruch-/Neustartnachweis und der ETW-Vergleich für die tatsächlich geänderten Bytes fällig. Produktdaten und WebView2 bleiben gesperrt.

### C4d · Dauerhafte Fenstersitzung im Folgekandidaten

Unter derselben Kontrollsperre schreibt der verwaltete Starter nach der Paketprüfung und vor jeder Prozesserzeugung `BeginWin32Window` mit dem gebundenen Paketanker. Der bestehende HMAC-Verlauf bindet Installation, Starter, Vorgänger und eindeutige Operation. Record und Kontrollkopf müssen dauerhaft geschrieben sein, bevor ein Job oder ein Fensterprozess angelegt wird. Der Digest dieses Begin-Records bezeichnet die Sitzung und geht über den Kontrollkopf in die Bootstrap-Bindung ein.

Erst nach erfolgreichem Rollenpfad, signalisiertem Prozessende, leerem Job und bestätigter Profilbereinigung schreibt der Starter `EndWin32Window` für genau diesen Digest. Der Aufruf liefert dann den neuen Kontrollkopf. Ein offener Beleg erlaubt die lesende Statusabfrage (`windowSessionOpen`), sperrt aber den nächsten Fensterstart und Änderungen am Kontrollzustand. Es gibt keinen externen Abschlussbefehl. Auch ein Fehler nach dauerhafter Vorbereitung und vor Prozesserzeugung lässt die Sitzung offen. Updates müssen nach dem ersten Sitzungseintrag die Unterstützung für dessen Verlauf im PE-Abschnitt `.kqwses` deklarieren.

**Gezielter Windows-Nachweis.** Ein regulärer Fensterlauf erzeugte zwei Kontrollrecords und erlaubte die nächste Sitzung. Beim anschließenden Abbruch des verwalteten Starters mit sichtbarem, geprüftem Fenster endete die Rolle. Der neu gestartete Broker las die offene Sitzung aus dem Verlauf und wies den Fensterstart mit aktuellem Kopf als `win32-window-session-open` ab. Die Nachprüfung bestätigte unveränderten Kopf und offene Sitzung. Eine Regression prüfte zusätzlich, dass der Endebeleg einer früheren Sitzung die nachfolgende Sitzung nicht freigeben kann. SHA-256 des verwalteten Prüfkandidaten: `fda5ccc1dbe9fe88fad12fb497c2449faf9435b96be66a3885a353a097e833d4`.

### C4d · ETW-Netzvergleich des Folgekandidaten

Am 19.11.2026 wurden der verwaltete Starter aus dem vorigen Abschnitt und die Fensterrolle `01e907b71328219456a23dac4cb39d7b88e3e69ee28d1a2614ec22a64b22212d` in derselben synthetischen Kontrollablage gemessen (Code-Stand `430f1d3`). Jede Phase hatte eine eigene Kernel-Netzwerk- und Prozesserfassung. Die Auswertung band den Starter über PID und vollständigen hashbasierten EXE-Namen. Sie prüfte die Start- und Endereignisse jeder erwarteten Prozessgeneration sowie die direkte Elternbeziehung der Fensterrolle.

| Messpfad | Starter / Fensterrollen | Netzwerkereignisse im KRAQEL-Baum | Lokale TCP-/UDP-Kontrollen (IPv4/IPv6) | Verlorene Ereignisse / Puffer |
| --- | ---: | ---: | ---: | ---: |
| Reguläres Schließen nach sichtbarem Fenster | 1 / 1 | 0 | 4/4 | 0 / 0 |
| Starterabbruch bei sichtbarem Fenster | 1 / 1 | 0 | 4/4 | 0 / 0 |
| Neustart mit offenem Sitzungsbeleg und aktuellem Kontrollkopf | 1 / 0 | 0 | 4/4 | 0 / 0 |

Beim regulären Ende wurden Start und Abschluss dauerhaft vermerkt, die Sitzung war geschlossen und der Job leer. Nach dem Starterabbruch endete auch die Fensterrolle. Anschließend blieb der Startbeleg offen. Beim Neustart lieferte der Broker `win32-window-session-open`; ETW erfasste ausschließlich den Starter. Die anschließende lesende Statusabfrage bestätigte identischen Kontrollkopf und Sitzungszustand. In allen drei Erfassungen gab es null nicht zuordenbare Netzwerkereignisse. Die Aussage gilt für diese Binärdateien und Prozesslebenszeiten im synthetischen Vorschaupfad. Die Rohspuren blieben lokal; Netzwerkaktivität eigenständiger Windows-Dienste liegt außerhalb der Prozessbaumzuordnung.

### C4d-Abschluss für den synthetischen Vorschaupfad

**Entscheidung vom 19.11.2026.** Der Folgekandidat ist für den verwalteten Start des schreibgeschützten synthetischen Win32-Fensters abgenommen. Die Abnahme bindet den Starter `fda5ccc1dbe9fe88fad12fb497c2449faf9435b96be66a3885a353a097e833d4` an das vierteilige Rollenpaket mit Anker `52b4dc1f1a1dd28293e8c10e0613e034646907d1731db3f6e6c92d1c8fcc340f`. Der abschließende lesende Abgleich der vorhandenen Binärdateien und Messaggregate bestätigte die vier inventarisierten Imagehashes, den Anker und die Identität beider gemessenen Programme.

Der authentisierte Sitzungsbeginn vor Prozesserzeugung und der generationsgebundene Endebeleg schließen die beim Vorgänger festgestellte Lücke. Nach regulärem Ende konnte eine neue Sitzung beginnen. Nach Starterabbruch endete die Rolle und die offene Sitzung sperrte den Neustart mit aktuellem Kontrollkopf. ETW belegte in diesem Fall genau einen Starter und null Fensterrollen. Alle drei Messpfade waren vollständig kalibriert, verlustfrei und ohne Netzwerkereignisse im gemessenen KRAQEL-Prozessbaum.

Die Freigabe umfasst ausschließlich diesen lokalen Vorschaupfad mit festem Prüftext. Produktdaten und WebView2 bleiben gesperrt. Für den verwalteten Zugriff auf Szenen und SQLite ist eine eigene Integration erforderlich. Die Netzbeobachtung ist auf die gemessenen Prozesslebenszeiten begrenzt.

### Verwalteter Einmalabruf der synthetischen Szene A

`broker-store open-win32-scene <aktueller-Kopf-SHA256>` startet Fensterrolle und `pipe-service.exe` aus dem gebundenen Paket unter derselben Kontrollsperre und demselben dauerhaften Sitzungsbeleg. Jede Rolle hat einen eigenen AppContainer ohne Capabilities und einen eigenen Einprozess-Job. Der Starter prüft beide suspendierten Prozessgenerationen, Token, Mitigations und Imageidentitäten. Er dupliziert die anonymen Pipe-Enden mit einseitigen Rechten in die jeweilige Generation und vergleicht die zurückgelesenen Objektidentitäten. Erst nach beiden Bootstrap-Quittungen übermittelt er die Kanaldaten. Die Kanal-ID bindet beide Rollenbelege und den Zweck A/read; die Richtungsschlüssel sind frisch.

Nach HELLO/READY verarbeitet der Datendienst über das vorhandene BP4b-Protokoll genau einen Leseaufruf. Er bindet den eingebauten synthetischen Katalog an Szene A im Lesemodus. Die Fensterrolle prüft die authentisierte Antwort für Revision 0 und setzt den empfangenen Text vor dem Anzeigen in das schreibgeschützte Win32-Feld. Der Datendienst endet nach dem Abruf. Das Fenster bleibt bis zum Schließen bedienbar. Die Produktablage bleibt gesperrt. Dieser Schritt arbeitet im Speicher. Der Zugriff auf eine persistente Szenenablage ist offen.

Der lokale Windows-Lauf prüfte Inhalt (`Rabe` und `42`) und Schreibschutz am sichtbaren Feld. Beide Jobs endeten und der Sitzungsbeleg wurde geschlossen. Eine Gegenprobe mit einem gebundenen Datendienst ohne den neuen Bootstrap brach vor der Szenenfreigabe ab; die Sitzung blieb gesperrt. Die frühere C4d-Abnahme gilt für die dort bezeichneten Bytes.

| Im Szenenlauf gestartetes Programm | SHA-256 |
| --- | --- |
| Verwalteter Starter | `8ab95846ef5888c3d8f06316f80164062bb5b407939203bee9062c49e5351c7a` |
| native-leaf.exe | `f49f0c5d1e733d60caf978d2131c41f70876d65c70270ce458deff6c281ccec9` |
| pipe-service.exe | `f7c47c896444928bf150dbf70a6be51950423eb359cf33e4c3faa04b9f48ef9f` |

### ETW-Netzvergleich für den einmaligen Szenenabruf

Am 19.11.2026 wurden die drei oben angegebenen SHA-256-Werte vor der Messung an den verwalteten Dateien bestätigt (Code-Stand `0dd8c05`). Zwei getrennte Erfassungen verwendeten dasselbe Paket und dieselbe synthetische Kontrollablage. In beiden Läufen prüfte der Messhelfer den festen Zweizeiler und den Schreibschutz am sichtbaren Fenster und schloss es regulär.

| Messpfad | Vollständige Prozesslebenszeiten | Netzwerkereignisse im KRAQEL-Baum | Lokale TCP-/UDP-Kontrollen über IPv4/IPv6 | Verlorene Ereignisse / Puffer |
| --- | --- | ---: | ---: | ---: |
| Referenz · `open-win32-surface` | 1 Starter, 1 Fensterrolle | 0 | 4/4 | 0 / 0 |
| Szene A · `open-win32-scene` | 1 Starter, 1 Fensterrolle, 1 Datendienst | 0 | 4/4 | 0 / 0 |

Die Auswertung bestätigte den Starter anhand seiner Start-PID und seines hashbasierten EXE-Namens sowie die direkten Kindrollen und deren Prozessenden innerhalb der Starterlebenszeit. Im Szenenlauf bestätigte der Starter den Einmalabruf von Szene A auf Revision 0. In beiden Fällen blieb der Produktdatenzugriff gesperrt, der Job war am Ende leer und die authentisierte Sitzung regulär geschlossen. Es gab keine nicht zuordenbaren Netzwerkereignisse. Der Nachweis gilt für diesen lokalen synthetischen Ablauf und die erfassten Prozesslebenszeiten. Eigenständige Windows-Dienste liegen außerhalb der Prozessbaumzuordnung. Die Rohspuren bleiben lokal.

### Schreibversuch unter der verwalteten Lesebindung

`broker-store check-win32-scene-denied-save <aktueller-Kontrollkopf>` verwendet denselben isolierten Rollenstart wie der Einmalabruf. Der Starter bindet den Prüfzweck an den authentisierten Kanal. Fensterrolle und Datendienst verwenden die vorhandene BP4b-Gegenprobe. Der Dienst hält über alle drei Nachrichten denselben synthetischen Katalog und dieselbe Bindung an Szene A im Modus `Read`.

Im gezielten Windows-Lauf las die Fensterrolle „Rabe“ / „42“ auf Revision 0 und versuchte, die gültige zweite Fassung „Rabe“ / „43“ mit erwarteter Revision 0 zu speichern. Der Dienst antwortete mit `DENIED`. Der erneute Abruf auf derselben Bindung bestätigte den ursprünglichen Text und Revision 0, bevor das schreibgeschützte Fenster angezeigt und regulär geschlossen wurde. Beide Jobs endeten, die Sitzung wurde abgeschlossen, Produktdaten blieben gesperrt. Der interaktive Einmalabruf behält seinen bisherigen Ablauf. Der ETW-Nachweis im vorigen Abschnitt bezieht sich auf den dort bezeichneten Binärstand.

### Revisionskonflikt unter eigener Edit-Bindung

`broker-store check-win32-scene-edit-conflict <aktueller-Kontrollkopf>` startet einen eigenen Auftrag für A/edit. Der Starter bindet den Edit-Prüfzweck in die authentisierte Kanal-ID ein. Nach dem Bootstrap bindet der Datendienst genau eine synthetische Szene im Arbeitsspeicher für die gesamte Folge. Beide Rollen nutzen den vorhandenen BP4b-Konfliktpfad.

Im verwalteten Windows-Lauf las die Fensterrolle Szene A auf Revision 0 und speicherte „Rabe“ / „43“ mit erwarteter Revision 0. Der Dienst bestätigte Revision 1. Ein anschließender Versuch, den früheren Text „Rabe“ / „42“ erneut mit erwarteter Revision 0 zu speichern, erhielt die authentisierte Antwort `CONFLICT`. Der abschließende Abruf bestätigte „Rabe“ / „43“ auf Revision 1 unter derselben Bindung. Das Fenster zeigte das Ergebnis schreibgeschützt an und schloss regulär. Beide Jobs endeten und die Sitzung erhielt ihren Abschlussbeleg. Der Nachweis gilt für die laufende synthetische Sitzung; eine persistente Ablage war nicht beteiligt.

### Dauerhafte Szene A im verwalteten Teststore

`broker-store save-win32-scene <Kontrollkopf>` legt unter der Kontrollsperre einmalig `data/scene-a-v1.sqlite` im privaten synthetischen Vorschau-Store an. Nur der isolierte Datendienst erhält nach dem Bootstrap das begrenzte Datei-Handle für seinen Auftrag. Er übernimmt die Änderung von „Rabe“ / „42“ zu „Rabe“ / „43“ in einer SQLite-Transaktion und schreibt das begrenzte SQLite-Abbild über dieses Handle. Die Speicherantwort folgt erst nach erfolgreicher Synchronisierung der Datei. Die Fensterrolle erhält den Szenentext über den authentisierten Kanal. Nach bestätigtem Rollenende bindet der authentisierte Abschlussrecord Volume, Datei-ID und SHA-256 an die Sitzung und die verwaltete Startergeneration. Bei unklarem Sitzungsende greift die bestehende Neustartsperre.

`broker-store read-saved-win32-scene <neuer-Kontrollkopf>` verlangt diesen Abschlussrecord, die passende Paketgeneration und die unveränderte, exklusiv geöffnete Datei. Die Prüfung erfolgt vor dem neuen Rollenstart. Der frische Datendienst erhält nur Leserechte am Abbild und öffnet es in SQLite schreibgeschützt. Pfad und Abfrage sind fest vorgegeben. Die Fenstersitzung erhält eine neue A/read-Bindung.

**Windows-Nachweis.** Nach vollständigem Ende des Speicherlaufs wurden Starter, Datendienst und Fensterrolle neu erzeugt. Die neue Lesesitzung bestätigte den gespeicherten Zweizeiler und Revision 1. Der Prüfaufbau verglich die Datenbankbytes vor und nach dem Lesen; sie waren identisch. Beide Sitzungen wurden regulär abgeschlossen. Eine nachträgliche Byteänderung an der isolierten Testdatei wurde vor einem weiteren Rollenstart mit `scene-snapshot-mismatch` abgewiesen; Kontrollkopf und Sitzungszustand blieben dabei unverändert. Der Nachweis betrifft einen Prozessneustart. Produktdaten bleiben gesperrt.

## Kosten für den Editor

Die Rust-Domäne, das versionierte Dokumentformat und die SQLite-Transaktionen sind mögliche
Wiederverwendungspunkte hinter einer getrennt zu prüfenden Datenschnittstelle. React, die
Tauri-Anbindung und die Tiptap-Ansicht lassen sich in einem Win32-Textfeld nicht ausführen. Das Feld
genügt für den Versuch mit Klartext. Beim Produkt muss ein strukturierter Editoradapter unter
anderem Absätze und Markierungen verlustfrei erhalten, Undo/Redo pro Szene trennen und Autosave samt
Revisionskonflikten abbilden. Das Zurückschreiben eines abgeflachten Manuskripts wäre Datenverlust.
Auch Medienansichten, Plotboard und die übrigen Arbeitsbereiche benötigen eigene UI-Arbeit.
[Editorvertrag](0004-editor-document-and-autosave.md),
[heutige Editoransicht](../../../apps/desktop/src/EditorWorkspace.tsx).

## Nächster Versuch und Abbruchkriterien

1. Im interaktiven Offline-Gast mit erzwungener, an die neuen Bytes gebundener UMCI genau ein
   UI-Blatt über den Starter erzeugen. Ein absichtlich abweichendes Rollenprofil muss vor Resume
   scheitern. Direktstart ohne Auftrag ebenfalls prüfen. Im Positivfall müssen die native Zuordnung,
   genau ein Prozess im Blattjob und dessen vollständiges Ende nach Schließen belegbar sein.
2. Mit festem synthetischem Text Mehrzeileneingabe, Fokus, Beschriftung und Vorlesen prüfen.
   Ausschließlich die Gasttastatur und lokale Sprachausgabe verwenden. Die Probe benötigt keine
   Hostfreigaben, Cloudstimmen, Projektablage, Zwischenablageübertragung oder Dateiimporte.
   [Bestehende Tastatur- und Hörprüfung](../../testing/editor-accessibility.md).
3. Für die zugrunde gelegten Programmbytes einen kurzen ETW-Vergleich mit Referenz, vier lokalen
   TCP-/UDP-Positivkontrollen, Prozessenden und Verlustprüfung durchführen. Gastweite Aktivität
   außerhalb des Blattjobs separat zuordnen. Ein zusätzlicher ungebundener Start, nötige
   Rechteausweitung oder eine für die Datenschutzaussage ungeklärte Übertragung sperrt die weitere
   Zulassung. Ein Bestehen trägt zunächst nur das native Fenster mit synthetischem Text.
