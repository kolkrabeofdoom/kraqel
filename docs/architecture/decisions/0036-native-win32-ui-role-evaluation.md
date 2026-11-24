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

### Datendienstabbruch an der dauerhaften Speichergrenze

Der verwaltete Dienst synchronisiert beim Anlegen zuerst den Anfangszustand von Szene A. Das SQLite-Commit im Arbeitsspeicher allein macht eine Änderung noch nicht dauerhaft. Erst nach dem Schreiben und Synchronisieren des neuen Abbilds darf der Dienst die Speicherantwort senden.

Für den Windows-Lauf vom 20. November 2026 erhielt jeder der beiden frischen privaten Teststores sein eigenes gebundenes Rollenpaket. Nur die Dienst-Binärdatei wurde mit dem jeweiligen Cargo-Prüffeature gebaut. Beide Läufe verwendeten den regulären Auftrag `save-win32-scene`. Nach dem Dienstabbruch prüfte der Prüfaufbau die geschlossene Datei unabhängig mit SQLite im unveränderlichen Lesemodus.

| Abbruchpunkt | Szene A auf der Platte | Neuer verwalteter Leseauftrag |
| --- | --- | --- |
| `probe-stop-before-snapshot`, unmittelbar vor dem Schreiben der Änderung | „Rabe“ / „42“, Revision 0 | gesperrt |
| `probe-stop-after-snapshot`, nach erfolgreicher Synchronisierung und vor der Antwort | „Rabe“ / „43“, Revision 1 | gesperrt |

Beide Dateien bestanden die SQLite-Integritätsprüfung. Beide Speicheraufträge endeten mit `scene-service-denied`. Der Sitzungsbeleg blieb offen. Der erneute Brokeraufruf verweigerte das Lesen mit `win32-window-session-open`; der authentisierte Kontrollkopf blieb dabei unverändert. Die beiden Befunde betreffen Prozessabbrüche an den angegebenen Grenzen. Ein Abbruch mitten im Überschreiben des Abbilds oder ein Stromausfall ist damit nicht geprüft.

### Unterbrechung während des Schreibens

Der verwaltete Speicherauftrag verwendet seit der zweiten Version des Edit-Bootstraps zwei exklusiv neu angelegte Dateien im privaten Teststore. Der Datendienst synchronisiert Revision 0 in `scene-a-base-v2.sqlite` und schließt sein Handle dafür, bevor er die Speicheranfrage verarbeitet. Revision 1 schreibt er in `scene-a-v1.sqlite`. Die Fensterrolle erhält ausschließlich die Kanalenden. Der Starter bindet die neue Fassung erst nach regulärem Sitzungsende über Dateiidentität und Hash an den Kontrollverlauf. Bestehende regulär abgeschlossene Abbilder behalten das bisherige Leseformat.

Im gezielten Windows-Lauf vom 20. November 2026 beendete das Prüffeature `probe-stop-mid-snapshot` den Dienst nach 16.384 geschriebenen und synchronisierten Bytes des 32.768 Byte großen Abbilds. Eine unabhängige schreibgeschützte SQLite-Abfrage bestätigte in der Basisdatei Szene A mit „Rabe“ / „42“ auf Revision 0 und intakter Datenbank. Der offene Sitzungsbeleg sperrte den neuen Leseauftrag bei unverändertem Kontrollkopf. Ein regulärer Lauf mit den beiden neu gebauten Rollen ohne Prüffeature bestätigte anschließend das Speichern und Wiederöffnen von Revision 1 unter frischer Lesebindung.

### ETW beim dauerhaften Speichern und Wiederöffnen

Am 20. November 2026 wurden die regulären Programmdateien zum Quellstand `40ab6b4` mit zwei lokalen ETW-Erfassungen verglichen. Die Referenz öffnete und schloss das synthetische Fenster. Die zweite Erfassung umfasste in einer frischen privaten Testablage das Speichern von Szene A und den separaten Starteraufruf zum erneuten Lesen. Der erste Auftrag meldete Revision 1, der zweite dieselbe Revision unter frischer Lesebindung. Die Jobs waren nach jedem Auftrag leer.

| Auftrag | Erfasste Rollen einschließlich Starter | Zugeordnete Netzwerkereignisse |
| --- | --- | ---: |
| `check-win32-surface` | Starter und Fenster | 0 |
| `save-win32-scene` | Starter, Fenster und Datendienst | 0 |
| `read-saved-win32-scene` | neuer Starter, neues Fenster und neuer Datendienst | 0 |

Beide Erfassungen enthielten alle vier lokalen TCP-/UDP-Positivkontrollen über IPv4 und IPv6. Die Auswertung meldete jeweils null verlorene Ereignisse, verlorene Puffer und nicht zuordenbare Netzwerkereignisse. Der Befund gilt für die erfassten Prozessbäume während der regulären Aufträge. Die Abbruchvarianten waren Gegenstand der funktionalen Prüfung. Rohspuren verbleiben lokal.

SHA-256 der erfassten Dateien:

- Verwalteter Starter: `53fca2b28c0361c16f9ffcdbb7b48ac49b7616fb5f966e1e149b602636d53add`
- Win32-Fensterrolle: `8d77b0a1fd5e040e6ec1228a9642b43ffbf85a5917c845aadceff0a0fcaec61b`
- Datendienst: `164ac72f511350fd9321e3bd69aebf227466cf43b0143ffe0d787b45729c36cb`

### Textänderung in einem markierten Absatz

Der verwaltete synthetische Speicherlauf legt Szene A mit zwei Absätzen an. Im zweiten Absatz ist der Textknoten „42“ fett markiert. Die feste Edit-Anfrage ändert die Zahl zu „43“. Der Datendienst prüft zuerst die Edit-Bindung, die erwartete Revision und den vollständigen zulässigen Dokumentbaum. Dann ersetzt er ausschließlich den Textwert des zweiten Knotens. Eine frische Lesebindung akzeptiert sowohl die bisherige unformatierte Szene A als auch diese genau definierte markierte Variante. Die Fensterrolle empfängt weiterhin die Klartextansicht.

Im Windows-Lauf vom 20. November 2026 bestätigte der neu gestartete Leseprozess Revision 1. Eine unabhängige SQLite-Abfrage verglich die vollständigen versionierten Dokumente in Basisdatei und gespeicherter Fassung mit den erwarteten Bäumen. Beide enthielten genau zwei Absätze und dieselbe Fettmarkierung an der Zahl. Klartext und Revision stimmten jeweils mit dem Dokument überein. Der vorhandene Modelltest für unzulässige Dokumentbäume bestätigte weiterhin die Ablehnung einer unerwarteten Markierung am ersten Absatz.

### Formatierte Vorschau

Der Datendienst überträgt bei einer validierten Fettmarkierung ein zusätzliches Darstellungsmerkmal in der authentisierten Leseantwort. Ohne dieses Merkmal zeigt das Fenster weiterhin Klartext. Für den markierten zweiten Absatz verwendet es ein eigenes natives Anzeigeelement mit fetter Schrift. Schriftdateien, RTF und externe Inhalte werden für die Darstellung nicht benötigt.

Im verwalteten Windows-Lauf endeten Speichern und erneutes Lesen von Revision 1 regulär. Beim Schließen prüft die Fensterrolle die beiden angezeigten Absatztexte und das am zweiten Steuerelement abgerufene Schriftgewicht. Der vorhandene Modelltest umfasst die Leseantworten für die markierte und die unmarkierte Szene.

### Tastatureingabe in Szene A

`broker-store edit-win32-scene <Kontrollkopf>` startet in einer frischen synthetischen Ablage die eigene Edit-Bindung. Nach authentisiertem Bootstrap und Erhalt der markierten Fassung auf Revision 0 öffnet das Fenster zwei einzeilige Eingabefelder. Tab und Umschalt+Tab wechseln zwischen beiden Absätzen und „Speichern“. Strg+S oder Enter auf „Speichern“ sendet den eingegebenen Text. Jeder Absatz enthält 1 bis 48 UTF-16-Codeeinheiten. Der Dienst prüft den vollständigen Dokumentbaum, die Edit-Bindung und die erwartete Revision. Er ersetzt die beiden Textwerte und bewahrt die Markierung des zweiten Absatzes. Vor dem dauerhaften Abschluss prüft die Fensterrolle den authentisierten Rücklesewert.

Der lokale Windows-Lauf verwendete native Zeichen- und Tastennachrichten für „Kolkrabe“ und „Stern 47“, Tab und Enter. Ein neuer `read-saved-win32-scene`-Auftrag öffnete Revision 1 schreibgeschützt. Die unabhängige SQLite-Abfrage bestätigte beide Textwerte und die Fettmarkierung ausschließlich am zweiten Absatz. Die Edit-Bindung gilt für einen einzelnen Speichervorgang an synthetischen Testdaten.

„Änderungen verwerfen“ setzt beide Eingabefelder auf die beim Öffnen authentisiert gelesenen Absätze zurück. Die Aktion ist per Tab und Enter erreichbar, setzt den Fokus in den ersten Absatz und löst keinen Speicherauftrag aus. Im gezielten Windows-Lauf wurden beide bearbeiteten Felder auf „Rabe“ und „42“ zurückgesetzt. Anschließend wurden im selben Fenster andere Texte eingegeben, gespeichert und unter frischer Lesebindung auf Revision 1 mit erhaltener Fettmarkierung geöffnet.

Der Fenstertitel zeigt „Ungespeicherte Änderungen“, sobald einer der beiden Absätze vom geladenen Text abweicht. Bei identischem Inhalt sind Speichern und Verwerfen deaktiviert und werden bei der Tab-Navigation übersprungen. Ein Speicherbefehl vergleicht den Text vor dem Absenden erneut. Im Windows-Lauf wechselten Fenstertitel und Schaltflächen beim Tippen und manuellen Zurückändern. Ein expliziter Speicherbefehl bei unverändertem Text ließ das Fenster offen. Die anschließende tatsächliche Änderung wurde als Revision 1 gespeichert und schreibgeschützt mit erhaltener Fettmarkierung geöffnet.

Beim Schließen eines veränderten Entwurfs fragt der Editor, ob beide Änderungen verworfen werden sollen. „Nein“ ist vorausgewählt. Im Windows-Lauf blieben nach „Nein“ die Eingaben erhalten und ließen sich per Tastatur speichern und erneut öffnen. Nach „Ja“ schloss das Fenster ohne Veröffentlichung einer Revision.

Der Tastatureditor hält bis zu 64 Änderungen der beiden Absätze in der Reihenfolge ihrer Eingabe im Fensterspeicher. Strg+Z nimmt den letzten Schritt zurück. Strg+Y und Strg+Umschalt+Z wiederholen ihn. Eine neue Eingabe nach dem Zurückgehen verwirft den Wiederholungszweig. „Änderungen verwerfen“ leert den Verlauf, ebenso endet er mit dem Fenster. Die Datenbankrevision ändert sich erst beim Speichern.

Der gezielte Windows-Lauf verwendete native Zeichennachrichten für Änderungen in beiden Absätzen sowie für Rückgängig und Wiederholen. Das Zurückgehen bis zum geladenen Stand deaktivierte Speichern. Neue Eingabe und explizites Verwerfen begrenzten den Verlauf wie vorgesehen. Anschließendes Speichern und Wiederöffnen bestätigten Revision 1 mit Fettmarkierung am zweiten Absatz.

Unter den Schaltflächen zählt der Tastatureditor die Wörter beider Absätze direkt im Fensterprozess. Er verwendet dieselbe Leerraumregel wie `countWords` im Manuskripteditor. Im Windows-Lauf zeigte das Steuerelement „0 Wörter“, „1 Wort“ und die korrekten Mehrwortwerte auch bei aufeinanderfolgenden und geschützten Leerzeichen. Rückgängig und Verwerfen aktualisierten die Anzeige. Die Wortzahl wird beim Speichern nicht mitgeschrieben.

Editor und Vorschau berechnen die Feldgrößen aus der aktuellen Clientfläche. Im Windows-Lauf wuchsen beide Absatzfelder beim Vergrößern des Editors von 460 × 250 auf 980 × 420 Pixel von 414 auf 934 Pixel Breite. Bei beiden Größen lagen Beschriftung, Absätze, Aktionen und Wortzahl innerhalb der Clientfläche und überdeckten einander nicht. Zwei Absätze mit jeweils 48 Zeichen ließen sich nach dem Größenwechsel speichern und mit erhaltener Fettmarkierung wieder öffnen.

### Anzeigeskalierung der Szenenoberfläche

Die Fensterrolle verwendet Per-Monitor-DPI-Awareness V2. Startgröße, Mindestgröße und
Abstände richten sich nach der DPI des Fensters. Bei einem DPI-Wechsel übernimmt das Fenster
das von Windows vorgeschlagene Rechteck und erneuert die Schriften anhand der Systemschrift
für die Ziel-DPI. Der zweite Absatz behält sein gespeichertes Schriftgewicht.

Im verwalteten Prüflauf bei 96 DPI passten Szenenbeschriftung, beide Schaltflächentexte und
Wortzähler bei Mindestgröße in ihre Steuerelemente. Nach Tastatureingabe und Wiederöffnen
enthielt Revision 1 beide Absätze und die Fettmarkierung. Der Wechsel zwischen Monitoren
mit unterschiedlicher Skalierung bleibt praktisch ungeprüft.

### Gespeicherte markierte Szene weiterbearbeiten

Ein erneuter Aufruf von `edit-win32-scene <aktueller Kontrollkopf>` öffnet die zuletzt bestätigte markierte Fassung mit einer frischen Edit-Bindung. Der Starter prüft unter der Kontrollsperre Eigentümergeneration, Dateiidentität und Hash. Er übergibt dem Datendienst ein Lesehandle für die bestätigte Fassung und ein Schreibhandle für die exklusiv neu angelegte Folgedatei. Die Fensterrolle erhält ausschließlich den Szenenkanal. Die erwartete Revision gehört zum Bootstrap des neuen Auftrags. Dienst und Fenster prüfen die Revision beim Speichern und Rücklesen. Nach regulärem Rollenende bindet der Abschlussrecord die neue Generation. Bei unklarem Sitzungsende bleibt die Sperre wirksam.

Im Windows-Lauf wurde Revision 1 gespeichert und im Folgeauftrag per Tastaturnachrichten zu „Kolkrabe“ und „Stern 47“ geändert. Unter frischer Lesebindung öffnete sich Revision 2. Die unabhängige SQLite-Abfrage bestätigte die beiden Absätze mit Fettmarkierung am zweiten Textknoten; der SHA-256-Wert der vorherigen Datei blieb unverändert. Der Nachweis gilt für die isolierte synthetische Ablage.

Ein zweiter Aufruf von `edit-win32-scene` mit dem Kontrollkopf vor der Bearbeitung von Revision 1 wurde nach Abschluss von Revision 2 mit `product-stale-head-or-budget` abgewiesen. Vor und nach dem abgewiesenen Aufruf waren Sequenz, Kontrollkopf, geschlossener Sitzungszustand sowie Dateinamen und SHA-256-Werte der Szenenablage identisch. Revision 3 wurde nicht angelegt. Die Kopfprüfung liegt im Starter vor Sitzungsbeginn und Rollenstart. Mit dem aktuellen Kontrollkopf ließ sich Revision 2 anschließend schreibgeschützt öffnen; der SQLite-Gegencheck bestätigte beide Texte und die Fettmarkierung.

### Authentisierter Abschluss beim Verwerfen

Beim regulären Schließen ohne Speichern sendet die Fensterrolle an Stelle der Speicheranfrage die authentisierte Verwerfanforderung auf Sequenz 2. Der Datendienst quittiert sie vor jedem Schreibzug. Beide geprüften Rollen melden ihren getrennten, an das frische Kanalmaterial gebundenen Endebeleg über ihre jeweilige Bootstrap-Pipe. Der Starter verlangt beide Belege, die vorgesehenen Exitcodes, beendete Rollen und leere Jobs. Unter der ursprünglichen Kontrollsperre entfernt er den nachweislich leeren Kandidaten über das gehaltene Handle. Beim ersten Entwurf entfernt er auch die neu erzeugte synthetische Basisdatei. Erst danach schreibt er den regulären authentisierten Sitzungsabschluss. Ein vorzeitiges Ende ohne vollständige Nachweise lässt die Sitzung offen.

Der Windows-Lauf prüfte drei Fälle. Nach bestätigtem Verwerfen einer Änderung an Revision 1 blieb deren Datei bytegleich und ließ sich schreibgeschützt neu öffnen. Nach Verwerfen des ersten Entwurfs blieben keine SQLite-Dateien zurück und ein neuer Auftrag konnte Revision 1 anlegen. Nach erzwungenem Abbruch der Fensterrolle blieb der nächste Edit-Auftrag mit `win32-window-session-open` gesperrt.

### ETW beim Ersetzen, Formatieren und Wiederöffnen

Am 23. November 2026 wurden für den Editorstand `8df0aad` zwei lokale ETW-Spuren
auf dem Windows-Arbeitsplatz erfasst. Im Referenzfenster lief `check-win32-surface`
mit verwaltetem Starter und Fensterrolle. Im zweiten Messfenster ersetzte der Editor
„47“ durch „48“, entfernte die Fettmarkierung des zweiten Absatzes und speicherte.
Ein frischer Leseauftrag öffnete Revision 1. Die SQLite-Gegenprüfung bestätigte
„Kolkrabe“ und „Stern 48“ sowie den unmarkierten zweiten Textknoten.

Für Edit- und Leseauftrag wurden jeweils die Lebenszeiten von Starter, Fensterrolle
und Datendienst zugeordnet. Für jeden der drei Aufträge ergaben sich null Netzwerkereignisse
im zugeordneten Prozessbaum und ein leerer Job am Ende. Beide Spuren enthielten alle vier
lokalen TCP-/UDP-Positivkontrollen über IPv4 und IPv6. Die Auswertung meldete jeweils null
nicht zuordenbare Netzwerkereignisse sowie null verlorene Ereignisse und Puffer. Der Nachweis
gilt für die gemessenen Programmbytes und Abläufe. Die Rohspuren verbleiben lokal.

| Binärdatei | SHA-256 |
| --- | --- |
| Verwalteter Starter | `13d448540a2a5c5d46a8be38777fd7603ddb6703c742a0cee3e4d5815e7a9da6` |
| Fensterrolle | `cfbdd3386001ce40378799b13734c8ca7c0020487a82633b51672b859fc2574d` |
| Datendienst | `5b0ff5f267098bbd908c65c8a0c58d23835219b1905334b8fe11d1ea95569428` |

### Paketkompatibilität und ETW für einzelne Fettmarkierungen

Seit dem 24. November 2026 deklarieren Fensterrolle und Datendienst den gemeinsamen Vertrag für Lesen, Speichern und Entfernen einer einzelnen Fettstelle in der schreibgeschützten PE-Sektion `.kqspan`. Der Starter liest die Kennung über die gehaltenen Dateihandles unter der Kontrollsperre. Bei der Aufnahme prüft er die Quelldateien vor der Veröffentlichung und anschließend das verwaltete Paket. Bei späteren Aufrufen prüft er das gebundene Paket vor dem Sitzungsbeginn. Eine einseitige Kennung oder eine ungültige Sektion sperrt den Aufruf. Die bestehenden Paketdigests bestimmen weiterhin die Identität der Dateien. Für zwei unmarkierte Rollen gilt der bisherige Pfad. Deren Unterstützung für einzelne Fettstellen lässt sich aus den fehlenden Kennungen nicht ableiten.

In einer frischen synthetischen Kontrollablage wies der Starter die Kombination aus neuem Fenster und bisherigem Datendienst vor dem Kopieren zurück. Der Status vor und nach der Ablehnung stimmte überein. Anschließend ließ sich das passende Paar unter demselben Kontrollkopf aufnehmen.

Zwei kurze lokale ETW-Spuren erfassten das Referenzfenster sowie drei aufeinanderfolgende Aufträge für Szene A. Das Fenster markierte „ab“ in „Rabe“ fett, entfernte diese Markierung unter frischer Edit-Bindung und öffnete Revision 2 lesend. Beide Revisionen enthielten denselben Text und die Fettmarkierung von „42“. Der Referenzlauf enthielt Starter und Fensterrolle, jeder Szenenauftrag zusätzlich den Datendienst. Alle vier Prozessbäume endeten mit leerem Job und jeweils null zugeordneten Netzwerkereignissen. Beide Spuren erfassten vier von vier lokalen TCP-/UDP-Kontrollen über IPv4 und IPv6. Keine verlorenen Ereignisse oder Puffer und keine nicht zuordenbaren Netzwerkereignisse in der Auswertung. Gemessen wurde der synthetische Win32-Pfad auf dem Windows-Arbeitsplatz. Die Rohspuren bleiben lokal.

| Gemessene Binärdatei | SHA-256 |
| --- | --- |
| Verwalteter Starter | `c950c38c4e668f548111eceac56fc607f859be4a844e1dc4ffa3b4be54410b09` |
| Fensterrolle | `8aed700f2682db1907290f1e790159f37a2e823445ffd2700366e6b9a12ead13` |
| Datendienst | `10ba323592c03ef3bb94af10a4dff0dc51f2a2f662d0226f39feb73fe456a122` |

### ETW beim bestätigten Verwerfen und Wiederöffnen

Für den regulären Binärstand zu `f0cbb5d` enthielt die erste Messung den Referenzaufruf `check-win32-surface` (Starter und Fenster). In der zweiten Messung wurde die vorbereitete Revision 1 bearbeitet und das Verwerfen beim Schließen bestätigt. Der anschließende Leseauftrag unter frischer Bindung bestätigte Revision 1. Für beide Aufträge wurden Starter, Fenster und Datendienst jeweils anhand der erfassten Prozesslebenszeiten ausgewertet. Der Dateihash von Revision 1 blieb unverändert.

Für jeden der drei Aufrufe ergab die Zuordnung null Netzwerkereignisse im KRAQEL-Prozessbaum. Beide Erfassungen enthielten vier von vier lokalen TCP-/UDP-Kontrollen über IPv4 und IPv6, null nicht zuordenbare Netzwerkereignisse und null verlorene Ereignisse oder Puffer. Der Nachweis gilt für diese Prozessbäume und Abläufe. Die Rohspuren bleiben lokal.

| Binärdatei | SHA-256 |
| --- | --- |
| Verwalteter Starter | `13d448540a2a5c5d46a8be38777fd7603ddb6703c742a0cee3e4d5815e7a9da6` |
| Fensterrolle | `82189d56b0be13befc03e53977f09f2332ec54b970a949ca3f761db6ebcb59e6` |
| Datendienst | `51a080c6a929f5110ca6f16ddb03ea0d17fd5b44e6040df8c85370e91b7ea4b5` |

### Weiterbearbeiten unformatierter Altbestände

Eine gespeicherte synthetische Szene A der ersten Generation kann zwei unmarkierte Absätze enthalten. Der Editor übernimmt deren Darstellung aus der authentisierten Leseantwort. Der bisherige Speicherbefehl v1 erhält die vorhandene Markierung. Der verwaltete Editor verwendet
`c4_document_save_v2` mit Pflichtfeld `boldSecond` und erwarteter Revision. Nur eine Edit-Bindung
an die verwaltete Szene A darf diesen Befehl ausführen. Die Fensterrolle vergleicht beim Rücklesen
Text, Revision und Markierung mit dem beauftragten Zustand. Für unmarkierte Folgerevisionen prüft der Dienst den vollständigen Dokumentbaum einschließlich Version und Absatzstruktur.

Im Windows-Kompatibilitätslauf erzeugte ein isolierter Prüfdienst den früheren unmarkierten Ausgangszustand. Revision 1 wurde über die normale Edit-Bindung geändert und als Revision 2 schreibgeschützt wieder geöffnet. Die SQLite-Prüfung bestätigte zwei unmarkierte Absätze und unveränderte Bytes der Quelldatei. Ein separater Lauf mit dem regulären Dienst bestätigte die weiterhin fett markierte zweite Zeile bei einer neu angelegten Szene. Die Prüfung bezieht sich auf das Datenformat; eine Paketmigration einer bestehenden Installation war kein Teil des Laufs.

### Zwischenablage in nativen Textfeldern

Die Fensterrolle installiert den lokalen Nachrichtenfilter am ersten EDIT-Steuerelement auch in der schreibgeschützten Vorschau. Im Bearbeitungsfenster gilt er zusätzlich für das zweite Eingabefeld. Schlägt die Installation fehl, endet der Fensteraufbau vor der Anzeige. Der zweite Absatz der Vorschau ist ein statisches Anzeigeelement.

Der Filter unterdrückt die Win32-Befehle für Kopieren, Ausschneiden, Einfügen und das Kontextmenü sowie die entsprechenden Strg-, Einfüge- und Umschalt-Tastenkombinationen. Der gezielte Filtertest deckt direkte Nachrichten, übersetzte Steuerzeichen und reguläre Eingabe ab. Der verwaltete Windows-Lauf bestätigte Bearbeiten, Speichern und erneutes Öffnen der fett markierten Szene. Für diese Prüfungen wurde der Inhalt der Systemzwischenablage weder gelesen noch ersetzt.

### Revisionsanzeige und nächster Autosave-Schritt

**Implementiert.** Beim Bearbeiten benennt die Titelleiste die Revision der authentisierten,
gegen die Auftragsbindung geprüften Leseantwort. Änderungen an Text oder Formatierung sowie
Rückgängig und Verwerfen aktualisieren den Änderungszustand. Die angezeigte Basisrevision
bleibt während der Bearbeitung stabil. In der schreibgeschützten Ansicht stammt die angezeigte
Revision ebenfalls aus dem geprüften Rücklesen. Der Titel enthält nur Szenenkennung, Zahl und
Zustand. Eine Leseantwort des Datendienstes geht dem abschließenden Sitzungsbeleg des Starters
voraus. Beide Bestätigungen haben unterschiedliche Bedeutung.

**Entwurf für wiederholtes Autosave.** Der bestehende Schreibauftrag bindet genau eine
Folgedatei; auf Speichern und Rücklesen folgt der Rollenabschluss. Für fortlaufendes Schreiben
gemäß [ADR 0004](0004-editor-document-and-autosave.md) wird ein eigener Sitzungsvertrag benötigt:

1. Der Editor hält den aktuellen Entwurf und genau einen laufenden Speicherstand im
   Fensterspeicher. Nach 700 ms Schreibpause wird ein gültiger geänderter Stand fällig.
   Weitere Eingaben während eines Schreibzugs bleiben im neueren Entwurf erhalten.
2. Der Starter muss jeden Folgeschreibzug unter seiner Kontrollsperre an Szene, Prozessgeneration,
   aktuellen Kontrollkopf und erwartete Revision binden und die nächste Zieldatei selbst anlegen.
   Das Fenster erhält weiterhin ausschließlich den Szenenkanal. Der bisherige Einmalauftrag und
   sein Replay-Schutz bleiben für bestehende Aufrufe maßgeblich.
3. Erst die bestätigte Veröffentlichung der Folgerevision durch den Starter verschiebt die
   Speicherbasis. Der Editor vergleicht danach den *aktuellen* Entwurf mit dem bestätigten Stand.
   Bei Konflikt bleibt der Entwurf zur Entscheidung offen. Bei unklarem Commit oder fehlendem
   Endebeleg darf weder automatisch erneut geschrieben noch eine Sperre gelöst werden.
4. Für die nächste Implementierung genügen zwei integrierte Fälle: Schreiben während eines
   laufenden Schreibzugs mit anschließendem Wiederöffnen der letzten bestätigten Revision sowie
   Verlust der Bestätigung mit gesperrtem Folgeschreibzug. Ausstehende Änderungen beim Schließen
   müssen explizit gespeichert oder verworfen werden.

### Veröffentlichung bei offenem Editorfenster

Der Forschungsaufruf `broker-store checkpoint-win32-scene <Kontrollkopf>` bestätigt einen
Schreibzug für die synthetische Szene A. Beim Speichern hält die Fensterrolle das ursprüngliche
Editorfenster offen und sperrt dessen Eingabe. Nach dem authentisierten Rücklesen vergleicht sie
Text und Markierung mit dem Entwurf und meldet das Fensterhandle auf der Bootstrap-Pipe.

Unter der ursprünglichen Kontrollsperre prüft der Starter das Ende des Datendienstes, dessen
leeren Job und Profilbereinigung sowie PID, Thread, Sichtbarkeit und Klasse des Fensters.
Er bindet Dateiidentität, Hash und Folgerevision im authentisierten Journalrecord
`PublishWin32Scene`. Dieser Übergang verlangt den unveränderten Beginn dieser Sitzung als
Kontrollkopf und lässt die Sitzung offen. Der Starter übermittelt Revision und neuen
Record-Digest über die bestehende rollenprivate Pipe. Die Fensterrolle prüft die Revision und
quittiert den Empfang durch einen domänengetrennten Digest des Frames. Erst Quittung, erwarteter
Prozessabschluss und Bereinigung erlauben den separaten Sitzungsabschluss.

Im Windows-Lauf bestätigten zwei explizite Aufträge Revision 1 und 2 einschließlich frischer
Leseprüfung. Eine separate synthetische Fensterrolle endete nach Empfang des Frames vor ihrer
Quittung. Der Aufruf schlug fehl; die Folgedatei war vorhanden, das Journal zeigte eine offene
Sitzung. Der nächste Schreibauftrag wurde mit `win32-window-session-open` abgewiesen und der
Kontrollkopf blieb gleich. Der nachfolgende Fortsetzungspfad erweitert diesen Einmalauftrag um einen zweiten Schreibzug.

### Zweiter Schreibzug im selben Fenster

`broker-store continue-win32-scene <Kontrollkopf>` gestattet höchstens zwei
Schreibzüge für Szene A in einer Fenstersitzung. Während der Veröffentlichung ist
die Eingabe gesperrt. Die Fensterrolle quittiert entweder Abschluss oder Fortsetzung, gebunden
an die vom Starter bestätigte Revision und den Digest des Veröffentlichungsrecords. Bei
Fortsetzung verlangt der Journalübergang `RebindWin32Scene` unter derselben Kontrollsperre
den bisherigen Sitzungstoken, den aktuellen Veröffentlichungskopf und die zum Akteur gehörende
Snapshotrevision. Der Digest des neuen Records wird zum Sitzungstoken für den Folgeschreibzug.

Der Starter behält die bestätigte Basisdatei am geprüften Handle, legt eine neue Zieldatei
exklusiv an und startet einen frischen Datendienst. Dieser erhält die Basis lesend und das
Ziel schreibend. Die Fensterrolle erhält frische Pipe-Enden und Schlüssel. Vor der Freigabe
der Eingabe vergleicht sie das authentisierte Rücklesen mit der bestätigten Revision und dem
noch sichtbaren Text samt Formatierung. Das ursprüngliche HWND bleibt bestehen, ebenso der
lokale Rückgängig-Verlauf. Beim zweiten Speichern prüft der Starter erneut die Identität des
Fensters. Nach der zweiten Bestätigung endet der Auftrag.

Im verwalteten Windows-Lauf änderte sich der erste Absatz von „Kolkrabe 7“ zu „Kolkrabe 8“.
HWND und PID waren nach dem ersten Speichern identisch. Revision 1 enthielt den ersten Stand
mit fett markiertem zweiten Absatz, Revision 2 den geänderten Text ohne Markierung. Revision 2
öffnete sich auch unter frischer Lesebindung. Beim regulären Schließen direkt nach der ersten
Bestätigung blieb Revision 1 lesbar; der leere zweite Kandidat wurde entfernt und die Sitzung
geschlossen. Ein erzwungenes Prozessende während der erneut gebundenen Bearbeitung ließ die
Sitzung offen. Der nächste Schreibauftrag wurde bei unverändertem Kontrollkopf abgewiesen.

### Entprelltes Autosave in der Zwei-Schreibzug-Probe

Im Auftrag `continue-win32-scene` startet jede tatsächliche Änderung von Text oder Fettmarkierung
eines gültigen Entwurfs eine lokale Frist von 700 ms. Die Fensterrolle prüft bei Zustellung des
Timers die monotone Frist, den aktuellen Entwurf und seine Basis erneut. Ein zurückgenommener
oder ungültiger Entwurf verwirft den ausstehenden Timer. Suchtext und Ansichtswechsel sind keine
Dokumentänderungen. Jeder Schreibzug hat seinen eigenen Timer. Beim Verlassen der Bearbeitung
wird er aufgehoben. Während der Schließabfrage ruht er; beim Weiterarbeiten wird neu geplant.

Autosave verwendet den regulären Speicherpfad. Während der Veröffentlichung ist die Eingabe
gesperrt. Nach Bestätigung und frischer Bindung kann im selben Fenster weitergeschrieben werden.
Der Titel kennzeichnet den ersten und zweiten Autosave-Schreibzug. Nach der zweiten Veröffentlichung
schließt der Forschungsauftrag. Ein Timerfehler lässt das manuelle Speichern zu. Suchbegriffe,
Timerzustand und der vorgemerkte Entwurf verlassen über den Timer keine Prozessgrenze.

Im zusammenhängenden Windows-Lauf verbrauchten ein zurückgenommener und ein leerer Entwurf
über die Schreibpause hinaus keinen Schreibzug. Eine Folgeeingabe setzte die Frist zurück:
Der erste Schreibzug begann im beobachteten Lauf rund 718 ms nach der letzten Eingabe.
Eine reine Formatänderung löste den zweiten Schreibzug aus. Die frische Leseprüfung und beide
SQLite-Stände bestätigten denselben Text mit der jeweils erwarteten Markierung.

### Fortlaufendes Autosave unter Journalbudget

Vor dem Sitzungsbeginn reserviert der Starter unter der Kontrollsperre Platz für einen
Beginn, eine Veröffentlichung pro Schreibzug, je eine neue Edit-Bindung zwischen den Zügen
und ein reguläres Ende. Für N Speicherungen müssen also 2N + 1 weitere Records Platz haben.
Der Starter erkennt die Budget-Unterstützung an der gebundenen Fensterdatei und übergibt
das Limit nach dem Bootstrap über den privaten Kontrollkanal. Ältere Fensterpakete
verwenden weiterhin das Protokoll für zwei Schreibzüge. Jede neue Bindung nimmt die
zuletzt bestätigte Revision als Basis. Nach dem letzten bestätigten Schreibzug schließt das
Fenster. Der Ablauf verwendet ausschließlich die synthetische Szene A.

Auf Windows wurden drei aufeinanderfolgende Speicherungen durch Schreibpause im selben
Fenster ausgelöst. Nach regulärem Schließen ließ sich Revision 3 mit dem erwarteten Text
öffnen. Ein gesonderter Grenzlauf nutzte den manuellen Speicherbefehl im selben Fenster,
um das Journal auszuschöpfen. Bei 124 verfügbaren Schreibzügen war Revision 124 mit dem
erwarteten Text dauerhaft gespeichert und die Sitzung geschlossen. Ein weiterer Start
wurde bei unverändertem Journalstand abgewiesen.

### Eingaben während der Autosave-Bestätigung

Bei einem verfügbaren Folgeschreibzug bleiben Fenster und Eingabefokus während der
Veröffentlichung erhalten. Die Fensterrolle hält den eingereichten Text samt Fettmarkierung
fest. Sie vergleicht sowohl das Rücklesen als auch die frisch gebundene Basis mit diesem
Stand. Reguläre Tastatureingaben warten in der lokalen Win32-Nachrichtenwarteschlange.
Sie werden nach der frischen Bindung sichtbar und durchlaufen wieder die normale
Entwurfsprüfung und Entprellung. Während eines langsamen Schreibzugs kann sich die Anzeige
also verzögern. Beim letzten Schreibzug und beim ausdrücklich gewählten Speichern und
Schließen wird die Eingabe für den Abschluss gesperrt.

Zwei Windows-Läufe verwendeten ausschließlich die synthetische Szene A. Bei angehaltenem
Starter wurde während der ausstehenden Bestätigung ein Zeichen in die Fensterwarteschlange
gestellt. Nach Fortsetzung enthielt Revision 1 den eingereichten Text, Revision 2 auch das
Folgezeichen. Die frische Lesebindung öffnete Revision 2. Im zweiten Lauf endete der
Starter bei wartender Eingabe. Das Fenster endete, die Sitzung blieb gesperrt und es
entstand keine zweite Revisionsdatei.

### Schließen mit wartender Eingabe

Ein Schließwunsch im fortlaufenden Autosave bleibt lokal vorgemerkt. Mehrere
Schließnachrichten ergeben eine Entscheidung. Nach der bestätigten Bindung arbeitet die
Fensterschleife die vorhandenen Nachrichten einschließlich der aus Tastendrücken erzeugten
Zeichen ab. Erst dann vergleicht sie den Entwurf mit der bestätigten Basis. Während der
ausstehenden Entscheidung kann der Timer keinen Folgeschreibzug auslösen.

Für jede Auswahl wurde der Starter beim ersten Autosave angehalten. Während der Bestätigung
wurden ein Tastendruck und zwei Schließnachrichten eingereiht. Die Abfrage blieb länger als
die Autosave-Frist offen. **Ja** speicherte das Folgezeichen in Revision 2 und schloss.
**Nein** verwarf die Folgeänderung und ließ Revision 1 erhalten. **Abbrechen** hielt
dasselbe Fenster offen. Nach einer weiteren Eingabe enthielt Revision 2 beide Folgezeichen.
Jede Variante endete mit regulärem Sitzungsabschluss und frischem Lesen der erwarteten
Revision der synthetischen Szene A.

### Speicheranzeige im fortlaufenden Editor

Ein ungültiger Entwurf bleibt im Fenster. Der Titel nennt den ersten zu korrigierenden
Absatz mit derselben Meldung wie beim manuellen Speichern. Bei einem gültigen geänderten
Entwurf zeigt der Editor die Schreibpause an. Während der Veröffentlichung steht dort
„Speichert“. Nach der Starterbestätigung und der frischen Bindung zeigt ein unveränderter
Entwurf ab Revision 1 „Gespeichert“. Die Anzeige enthält ausschließlich feste Meldungen
und Revisions- oder Budgetzahlen.

Der lokale Windows-Lauf ließ den zweiten Absatz länger als die Schreibpause leer.
Nach der Korrektur blieb „Speichert“ bei angehaltenem Starter sichtbar. Nach dessen
Fortsetzung erschien „Gespeichert“. Der frische Leseauftrag und die SQLite-Prüfung
bestätigten Revision 1 mit dem korrigierten Text.

### Revisionskonflikt bei einem Schreibzug

Nur ein authentisiertes `CONFLICT` als Antwort auf den konkreten Schreibversuch öffnet
die Konfliktansicht. Bei Protokollfehlern oder unklarem Commit gilt weiter der Fehlerpfad.
Das ursprüngliche Fenster hält Text, Markierung und Verlauf lokal. Weitere Änderungen
bleiben im Entwurf. Autosave und manuelles Speichern sind gesperrt. Beim Schließen oder
Verwerfen ist das Behalten des Entwurfs vorausgewählt. Erst nach ausdrücklichem Verwerfen
sendet das Fenster den authentisierten Verwerfbefehl mit Sequenz 3. Der Datendienst
akzeptiert an dieser Stelle ausschließlich diesen Befehl. Der Starter schließt den Auftrag
über die bestehenden Verwerfquittungen beider Rollen ab.

Im Windows-Lauf wurde nach einer bestätigten Revision 1 die Arbeitskopie eines synthetischen
Prüfdatendiensts vor der nächsten Speicherung intern um eine Revision weitergesetzt.
Die reguläre Revisionsprüfung meldete den Konflikt. Das Fenster behielt den eingereichten
Text, ließ nach abgelehntem Verwerfen lokale Änderungen zu und veröffentlichte auch nach
Speicherbefehl und Schreibpause keine neue Revision. Nach bestätigtem Verwerfen öffnete
der frische Leseauftrag den ursprünglichen Stand der Revision 1. Eine gezielte Kanalprüfung
bestätigte zusätzlich die Zurückweisung eines authentisierten Folgeschreibversuchs vor
dem Aufruf der Speicherlogik.

### Lesender Revisionsvergleich im Konfliktfenster

Wenn die beiden gepinnten Rollen die Vergleichsversion ausweisen, aktiviert der Starter
den Vergleich. Nach einem authentisierten Konflikt leiten die Rollen aus dem einmaligen
Schreibzug eine separate Lesebindung für Szene A ab. Kennung und Richtungsschlüssel
sind an den Zweck, den ursprünglichen Kanal und dessen Basisrevision gebunden. Auf den
bestehenden privaten Pipes nimmt diese Bindung genau einen Lesebefehl an. Der Datendienst
liest Szene A dabei unter Leserechten aus seinem aktuellen Szenenstand. Das Fenster prüft
die Antwort und die höhere Revision. Die ursprüngliche Schreibbindung kann weiterhin nur
noch den expliziten Abschluss durch Verwerfen quittieren.

Links bleibt der lokale Entwurf bearbeitbar. Rechts steht die einmal geladene Dienstrevision
einschließlich beider Absätze und der Angabe zur Fettmarkierung. F6 wechselt den Fokus
zwischen Vergleich und Entwurf. Das schreibgeschützte Feld nutzt den vorhandenen lokalen
Zwischenablagefilter. Der Vergleich veröffentlicht keine Revision. Die gezielte Übernahme im fortlaufenden Auftrag ist im folgenden Abschnitt beschrieben.

Im verwalteten Windows-Lauf lieferte der synthetisch fortgeschriebene Datendienst Revision 2,
während im ursprünglichen HWND der abweichende Entwurf erhalten blieb. F6 wechselte in
beide Richtungen. Eingabe im Vergleich änderte dessen Text nicht. Weitere lokale Änderungen
ließen den Vergleich unverändert. Nach explizitem Verwerfen öffnete der Starter die zuletzt
veröffentlichte Revision 1. Die gezielte Kanalprüfung wies einen Schreibversuch und eine
wiederholte Leseanfrage unter der Vergleichsbindung vor der Speicherausführung ab.

### Gezielte Übernahme unter frischer Schreibbindung

Im fortlaufenden Editor bietet das Menü **Übernahme** drei einzeln wählbare lokale Werte an:
erster Absatz, zweiter Absatz und Fettmarkierung des zweiten Absatzes. Anfangs ist nichts
ausgewählt. Nicht gewählte Werte stammen aus dem angezeigten Dienststand. Die Auswahl
ändert den Entwurf nicht. **Auswahl speichern und schließen** verlangt ein gültiges Ergebnis
mit mindestens einer tatsächlichen Änderung.

Schon bei der ersten wählbaren Übernahme zeigt das schreibgeschützte Vergleichsfeld
unter dem Dienststand die Vorschau. Häkchen, weitere Texteingaben und die Fettmarkierung
aktualisieren die Mischung lokal. Entspricht sie dem Dienststand oder enthält sie einen
ungültigen ausgewählten Absatz, nennt die Vorschau den Zustand und beide Speicherbefehle
sind deaktiviert. Die Sitzungs- und Revisionsgrenzen gelten auch bei gültiger Auswahl.

Ein zusammenhängender Windows-Lauf prüfte die erste Vorschau, die beiden Menübefehle bei
leerer Auswahl, einen ungültigen gewählten Absatz und das manuelle Zurückändern auf den
Dienststand. Nach der Korrektur enthielt die wieder geöffnete Revision 3 den gewählten
ersten Absatz und den zweiten Absatz der Dienstrevision 2.

**Gesamten lokalen Entwurf auswählen** setzt alle drei Häkchen. **Auswahl auf Dienststand
zurücksetzen** entfernt sie. Beide Befehle aktualisieren die Vorschau. Der lokale Entwurf
bleibt bearbeitbar. Im Windows-Lauf mit Szene A sperrte die Rückkehr zum Dienststand beide
Speicherbefehle. Erst die anschließende Einzelwahl mit ausdrücklichem Speichern erzeugte
Revision 3. Das Wiederöffnen bestätigte die gewählte Mischung.

Fehlt die Freigabe für revisionsübergreifendes Schreiben und liegt bei der ersten Konfliktantwort eine Revisionslücke vor, zeigt **Übernahme**
die Auswahl als lokale Vorschau an. Im verwalteten Windows-Lauf lagen zwischen der
Bearbeitungsbasis 1 und dem frisch gelesenen Dienststand 3 zwei synthetische Änderungen.
Einblenden der Basis, Einzelwahl und Rücknahme funktionierten. Beide Speicherbefehle waren
gesperrt und ließen auch bei direkter Befehlsauslösung den Entwurf offen. Nach ausdrücklich
bestätigtem Verwerfen meldete der Starter die zuletzt veröffentlichte Revision 1 und null
verbleibende Jobprozesse.

Das Vergleichsfeld folgt in der Tab-Reihenfolge auf den zweiten Absatz. Die Navigation
überspringt ausgeblendete und gesperrte Bedienelemente. Im verwalteten Windows-Lauf
erreichte Tab das Vergleichsfeld vom zweiten Absatz aus. F6 führte mit erhaltener
Textmarkierung in diesen Absatz zurück. Ein weiterer Wechsel ins Vergleichsfeld und Tab
führten zur nächsten verfügbaren Schaltfläche. Die Navigation löste keinen Schreibzug aus.

Im normalen Vergleichsfenster öffnet Esc auch bei Fokus im schreibgeschützten Feld
die Verwerfungsabfrage. Beim Windows-Lauf lag der Dialogfokus auf **Nein**. Nach
dieser Entscheidung kehrte der Fokus zur markierten Textstelle im Vergleich zurück.
Entwurf und Übernahmehäkchen blieben erhalten. Erst der anschließende Speicherbefehl
veröffentlichte die geprüfte Mischung.

Beim Aktualisieren desselben Vergleichsfelds behält die Vorschau die erste sichtbare
Zeile bei, soweit der neue Inhalt das zulässt. Im Windows-Lauf mit verkleinertem Feld
blieben Scrollposition und Textmarkierung beim Weiterschreiben und beim Ändern der
Übernahmeauswahl erhalten. Nach Ausblenden der markierten Bearbeitungsbasis erschien
die alte Markierung nicht auf dem nachrückenden Diensttext.

**Letzte Auswahl rückgängig machen** stellt die Häkchen vor der letzten Einzel- oder
Sammelauswahl wieder her. Das Fenster merkt sich dafür drei Auswahlwerte bis zum nächsten
Revisionsvergleich. Im Windows-Lauf wurde eine gemischte Auswahl auf den Dienststand
zurückgesetzt und nach Einblenden der Bearbeitungsbasis wiederhergestellt. Entwurf und
Basisansicht blieben erhalten. Erst der Speicherbefehl veröffentlichte die wiederhergestellte
Mischung als Revision 3.

**Ursprüngliche Bearbeitungsbasis anzeigen** blendet im geschützten Vergleichsfeld die
Revision ein, auf der der lokale Entwurf beruht, samt beiden Absätzen und Fettmarkierung.
Das Fenster verwendet dafür die bereits gebundene Fassung im Arbeitsspeicher. Im
Windows-Lauf ließ sich Revision 1 neben Dienstrevision 2 ein- und ausblenden. Entwurf,
Auswahl und Vorschau blieben erhalten. Nach ausdrücklichem Speichern enthielt die frisch
gelesene Revision 3 den gewählten lokalen ersten und den zweiten Absatz des Datendienstes.

**Eindeutig lokale Änderungen zusätzlich auswählen** vergleicht jeden Absatz und die
Fettmarkierung mit der Bearbeitungsbasis des lokalen Entwurfs und dem angezeigten
Dienststand. Der Befehl ergänzt Häkchen für gültige lokale Änderungen an Feldern, die der
Dienst gegenüber der Basis beibehalten hat. Bereits manuell gesetzte Häkchen bleiben erhalten.
Die Vorschau nennt Absätze, die beide Seiten unterschiedlich geändert haben. Bei weiterer
Eingabe wird dieser Hinweis neu berechnet. Erst der separate Speicherbefehl veröffentlicht
die gewählte Mischung.

Im Windows-Lauf blieben zwei unterschiedlich geänderte Fassungen des ersten Absatzes zur
Entscheidung offen. Die Vorauswahl setzte die Häkchen für den nur lokal geänderten zweiten
Absatz und seine Fettmarkierung. Erneutes Anwenden erhielt ein manuell gesetztes Häkchen.
Nach Rückkehr des ersten Absatzes zur Bearbeitungsbasis verschwand sein Konflikthinweis.
Die frisch geöffnete Revision 3 enthielt den ersten Dienstabsatz und den lokalen zweiten
Absatz mit der gewählten Markierung.

Der Starter aktiviert die Funktion für ein passendes gepinntes Rollenpaar bei verfügbarem
Folgeschreibzug. Der Dienststand muss die unmittelbare Folgerevision der bisherigen Basis sein.
Ein eigener authentisierter Vorbereitungsauftrag prüft Revision, Text und Markierung nochmals
gegen die angebotene Vergleichsansicht. Ändert sie sich vor dieser Prüfung, entfällt das
Angebot; der lokale Entwurf bleibt offen. Der Datendienst kann unter dieser Bindung nur den
geprüften Dienststand als Snapshot bereitstellen. Die Lese- und bisherigen Schreibrechte
bleiben verbraucht.

Der Starter veröffentlicht zunächst die Vergleichsbasis im Journal und bindet einen neuen
Datendienst an diese Revision. Das Fenster prüft den zurückgelesenen Inhalt gegen die
bestätigte Basis, bevor es die Auswahl einmalig mit dieser erwarteten Revision speichert.
Die Auswahl schließt die Sitzung nach bestätigter Veröffentlichung. Zwischen beiden
Veröffentlichungen kann die Vergleichsbasis bereits dauerhaft sein. Bei unklarem Abschluss
gelten die bestehenden Sitzungs- und Neustartsperren.

Zwei verwaltete Windows-Läufe mit synthetischer Szene A deckten die Auswahl und den erneuten
Konflikt ab. Der erste behielt den ersten Absatz und die Markierung aus dem Entwurf und den
zweiten Absatz aus dem Dienststand. Revision 2 enthielt den geprüften Dienststand. Revision 3
enthielt die Mischung und ließ sich unter neuer Lesebindung öffnen. Im zweiten Lauf änderte
der Prüfdatendienst den Vergleichsstand vor der Vorbereitung erneut. Die Auswahl veröffentlichte
keinen Snapshot, der Entwurf blieb im Fenster. Nach ausdrücklichem Verwerfen war die zuvor
bestätigte Revision 1 wieder lesbar.

### Erneuter Vergleich vor dem Speichern

Meldet die Vorbereitung der Auswahl einen zweiten Konflikt, lädt das dafür freigegebene
Rollenpaar den aktuellen Dienststand einmal über eine neue Lesebindung. Deren Schlüssel
sind an die bisherige Vergleichsrevision gebunden. Nur eine authentisierte, gültige Szene A
mit höherer Revision erscheint als neuer Vergleich. Nach dem Lesen ist auf dem Kanal nur
noch der reguläre Verwerfabschluss möglich.

Der lokale Entwurf und die Häkchen im Menü **Übernahme** bleiben erhalten. Das rechte,
schreibgeschützte Feld zeigt die neue Dienstrevision und darunter die Vorschau der Auswahl.
Änderungen an den Häkchen oder am lokalen Entwurf aktualisieren die Vorschau im Fenster.
Die vorausgegangene Speicherbestätigung ist verbraucht. Für Rollenpaare mit reiner
Vergleichsunterstützung bleibt die Vorschau lesend.

Ein verwalteter Windows-Lauf prüfte Revision 2 → 3 vor der Speicherung: Der lokale erste
Absatz blieb ausgewählt. Die Vorschau enthielt den neueren zweiten Dienstabsatz und folgte
der geänderten Auswahl. Auch ein direkter Aufruf des gesperrten Speicherbefehls veröffentlichte
keinen Snapshot. Nach explizitem Verwerfen öffnete die zuvor bestätigte Revision 1.
Eine Kanalprüfung wies die Wiederholung der ersten Leseanfrage beim erneuten Vergleich ab.

### Bestätigte Übernahme über eine Revisionslücke

Unter **Übernahme → Auswahl speichern und schließen** nennt eine zweite Abfrage die
erneut geprüfte Dienstrevision. **Nein** ist vorausgewählt und führt mit allen Häkchen und
dem lokalen Entwurf zur Vorschau zurück. **Ja** gilt genau für die angezeigte Basis und
die gewählten Absätze samt Markierung. Der Starter aktiviert diesen Pfad nur bei einem
gepinnten Rollenpaar mit passender Protokollversion, Budget für den Folgeschreibzug und
Platz für dessen Folgerevision.

Vor der Vorbereitung liest der Datendienst die Szene erneut. Bei einer weiteren Änderung
bleibt der Entwurf lokal offen. Bei Übereinstimmung bestätigen Fenster und Datendienst auf
ihren privaten Kanälen unabhängig voneinander Revision und Digest von Text und Markierung.
Nach Ende des Datendienstes prüft der Starter das weiterhin offene Fenster. Er legt den
geprüften Basisstand exklusiv unter seiner tatsächlichen Revisionsnummer an, prüft den
Dateihash und entfernt die vorläufige Datei. Der eigene Journalübergang
`PublishReviewedWin32Scene` bindet diesen Sprung an Akteur, offenen Sitzungstoken, aktuellen
Journalstand und bisherige Szenenrevision. Für gewöhnliche Speicherungen gilt weiter die
unmittelbare Folgerevision. Überbrückte Nummern erhalten keine erfundenen Snapshots.

Erst nach Veröffentlichung der geprüften Basis erhält ein neuer Datendienst die frische
Edit-Bindung. Das Fenster prüft das Rücklesen und sendet dann die bereits bestätigte Auswahl.
Die beiden Veröffentlichungen sind getrennt. Nach einer Unterbrechung kann die geprüfte
Basis bereits dauerhaft sein; bei unbestätigtem Ende bleibt die Sitzung gesperrt.

Im verwalteten Windows-Lauf blieb nach **Nein** die Vorschau auf Dienstrevision 3 offen.
**Ja** veröffentlichte die Basis als Revision 3 und die Auswahl als Revision 4. Unter
frischer Lesebindung stimmten Text und Revisionsnummer; Revision 2 hatte keine Datei.
In einem zweiten Lauf änderte der Prüfdatendienst die Basis unmittelbar vor dem Speichern
nochmals. Der lokale Entwurf blieb offen. Nach explizitem Verwerfen war weiterhin nur
Revision 1 veröffentlicht.

### Weiterschreiben nach bestätigter Übernahme

**Übernahme → Auswahl speichern und weiterschreiben** hält das Fenster offen. Der Befehl
setzt zusätzlich zur Vorbereitung der Basis und zum Speichern der Auswahl noch Budget für
einen weiteren Schreibzug voraus. Auch die zulässige Revisionsnummer muss diesen zulassen.
Über eine Revisionslücke hinweg gilt die ausdrückliche Bestätigung mit **Nein** als Vorauswahl.

Das Fenster übernimmt die Mischung erst nach Veröffentlichung und authentisiertem Rücklesen
unter der nächsten Edit-Bindung in die Eingabefelder. Die Vergleichsansicht schließt, der
Fokus kehrt in den Editor zurück und der Rückgängig-Verlauf beginnt bei der bestätigten
Mischung. Folgeeingaben verwenden das reguläre Autosave. Das Fenster kennzeichnet den letzten
zulässigen Schreibzug auch dann, wenn die Revisionsgrenze vor dem Journalbudget erreicht ist.

Im verwalteten Windows-Lauf blieb dasselbe Fenster nach der bestätigten Revision 4 offen.
Rückgängig holte den alten Konfliktentwurf nicht zurück. Nach einer weiteren Eingabe speicherte
Autosave Revision 5; die frische Leseprüfung und die SQLite-Daten bestätigten den Text. In
einem Grenzlauf wurde Dienstrevision 254 als Basis bestätigt. Nach Revision 255 zeigte das
Fenster den letzten Autosave an. Revision 256 war nach regulärem Sitzungsende wieder lesbar.

### Weitere Konflikte während der erneuten Bestätigung

Für ein passendes gepinntes Rollenpaar aktiviert der Starter den wiederholten Abgleich
auf beiden Seiten. Wird eine schon aktualisierte Basis vor dem Speichern erneut überholt,
liest das Fenster den Dienststand über eine neue Einmalbindung an die zuletzt geprüfte
Revision. Nur eine höhere Revision mit gültigem Szeneninhalt erscheint als neuer Vergleich.
Die lokale Auswahl bleibt als Vorschlag erhalten. Jede Übernahme erfordert eine neue
ausdrückliche Bestätigung. Verwerfen beendet den Auftrag weiterhin über den ursprünglichen
Verwerfkanal. Für ältere Rollenpaare gilt der von beiden unterstützte Vergleichsumfang.

Im verwalteten Windows-Lauf wechselte die bereits geprüfte Basis zweimal hintereinander
auf Revision 4 und 5. Der lokale Text und die Häkchen blieben erhalten. Bis zur jeweiligen
Bestätigung blieben die entsprechenden Revisionsdateien aus. Erst die stabile Basis 5
und die ausgewählte Mischung 6 wurden veröffentlicht. Revision 6 ließ sich unter einer
neuen Lesebindung öffnen. Die bestehende Kanalprüfung für eine wiederholte alte
Leseanfrage bestand ebenfalls.

## Schreibgeschützte Textvorschau für formatierte Szenen

Der verwaltete Datendienst prüft beim Wiederöffnen zuerst den bestehenden Editiervertrag.
Für gespeicherte Revisionen ab 1 erkennt die Lesevorschau einen oder mehrere Absätze,
einschließlich Leerabsätzen, mit Klartext und den attributlosen Markierungen `bold`,
`italic` und `strike`. Insgesamt sind 4.096 UTF-16-Codeeinheiten einschließlich eines LF
pro Absatzgrenze zugelassen. Die gespeicherte JSON-Hülle darf 64 KiB umfassen.
Überlange Inhalte werden vollständig abgewiesen. Der formatierbare Editor unterstützt zwei nichtleere Absätze mit jeweils höchstens 48 UTF-16-Codeeinheiten. Für unformatierten Text gilt zusätzlich der folgende Klartextvertrag. Unbekannte Versionen, Knoten, Markierungen und
zusätzliche Felder sperren die Vorschau. Das strukturierte Original bleibt in der Ablage.

Die authentisierte Erstantwort für die Vorschau enthält ausschließlich Klartext und Revision
und ist von einem bearbeitbaren Szenenstand unterscheidbar. Das Win32-Fenster kennzeichnet
ausgeblendete Formatierungen. Bei einem Bearbeitungsauftrag erlaubt der Kanal auf diese
Antwort nur noch den regulären Abschluss ohne Veröffentlichung. Für einen Konfliktvergleich
oder eine Schreibbestätigung ist die Vorschauantwort unzulässig.

Der Modelltest prüfte gemischte Textknoten, beide Speicherbefehle, den Erhalt des Dokuments
und unbekannte Markierungen bzw. Attribute. Für den verwalteten Windows-Lauf erzeugte ein
temporärer Prüfdatendienst ein synthetisches SQLite-Abbild mit kursivem Text auf Revision 1.
Lesender Wiederaufruf und Bearbeitungsauftrag öffneten die Szene. Im Bearbeitungsauftrag
waren Vorschauhinweis, schreibgeschütztes Eingabefeld und beide Texte sichtbar. Nach dem
Schließen waren der Dateihash identisch, die Revision weiter 1 und der Job leer. Die
Präparation gehört ausschließlich zum Prüflauf.

Im Mehrabsatzlauf wurde eine synthetische Revision mit 37 Absätzen einschließlich
Leerabsatz über beide Aufträge geöffnet. Das scrollbare Win32-Lesefeld enthielt den
gesamten Text. Nach Scrollen bis zum Ende lag das letzte Zeichen im sichtbaren Bereich.
Schließen erhielt Revision und Dateihash. Der Modelltest deckt exakt 4.096 Einheiten mit
Supplementärzeichen und Leerabsatz sowie die Abweisung einer weiteren Einheit ab.

Die Vorschau durchsucht den angezeigten Text lokal im Fenster. Strg+F fokussiert das
Suchfeld, Enter sucht, F3 springt vorwärts und Umschalt+F3 rückwärts. Nach dem letzten
Treffer geht es beim ersten weiter. Die Suche unterscheidet Groß- und Kleinschreibung.
Ein Windows-Lauf mit 34 Absätzen prüfte Treffer am Anfang und Ende, den Umlauf und
die Meldung bei ausbleibendem Treffer. Die Markierung nach einem Supplementärzeichen
saß auf dem vollständigen Suchwort. Nach dem Schließen blieben Dateihash und Revision erhalten.

## Längere Klartextszenen bearbeiten

Im verwalteten Bearbeitungsauftrag öffnet eine bereits gespeicherte längere Szene A ohne
Markierungen in einem mehrzeiligen Klartextfeld. Bei einer kurzen gespeicherten Klartextfassung
öffnet **Bearbeiten → Als längeren Klartext bearbeiten** dasselbe Feld. Der Wechsel setzt
eine bestätigte, unformatierte Basis voraus. Bekannte Formatierungen außerhalb des
Zwei-Absatz-Vertrags bleiben auf die Lesevorschau beschränkt.

**Speichern und schließen** (Strg+S) veröffentlicht die Änderung auf der gebundenen Revision.
Bei verfügbarem Folgezug speichert **Speichern und weiterschreiben** (Strg+Enter)
im selben Fenster. Der Starter bestätigt die Veröffentlichung. Die Fensterrolle prüft
unter der nächsten Bindung Revision, Text und Markierung gegen die eingereichte Fassung,
bevor sie weitere Eingaben verarbeitet. Beim letzten Schreibzug ist nur Speichern und
Schließen verfügbar. Bei verfügbarem Folgezug nutzt der lange Klartexteditor denselben
700-ms-Timer wie der kurze Editor. Der Timer ist an den gültigen geänderten Text gebunden.
Eine Rückkehr zum bestätigten Stand, ungültiger Text oder das Öffnen der Schließabfrage
verwirft den Timer. Das maximale Textbudget beträgt 4.096 UTF-16-Einheiten
einschließlich LF. Das bestehende Speicherlimit von 64 KiB für das strukturierte Dokument
gilt auch für absatzreiche Entwürfe. Leere und abschließende Absätze bleiben beim Speichern
und Wiederlesen erhalten. Ein Revisionskonflikt hält den Entwurf im Fenster und sperrt die
Veröffentlichung. Das Verwerfen verlangt eine Bestätigung.

Der verwaltete Windows-Lauf prüfte den Wechsel aus der bestätigten kurzen Fassung, die
Sperre bei 4.097 Einheiten, Revision 2 nach ausdrücklicher Speicherung und Revision 3
nach Wiederöffnen und Weiterbearbeiten. Struktur, Klartext und Suchindex wurden in beiden
Abbildern geprüft. Im Konfliktlauf blieb der Entwurf sichtbar. Nach bestätigtem Verwerfen
blieb Revision 2 die zuletzt veröffentlichte Fassung. Der Modelltest prüft zusätzlich die
Budgetgrenze mit Supplementärzeichen, die Lese- und Formatsperren und den veralteten Schreibzug.

Im Windows-Lauf mit verzögerter Starterbestätigung wurden während der Bestätigung
eingegebene Zeichen erst Teil von Revision 3. Revision 2 enthielt den zuvor eingereichten
Text. Fensteridentität, Klartext, Absatzstruktur, Suchindex und frisches Lesen wurden
geprüft. Beim letzten verfügbaren Schreibzug blieb Weiterschreiben auch bei direktem
Aufruf gesperrt. Speichern und Schließen beendete die Sitzung regulär.

Der Windows-Lauf zum Klartext-Autosave prüfte das erneute Starten der Schreibpause,
das Zurückgehen zum bestätigten Text und den Wechsel zu einem ungültigen Entwurf.
Während der verzögerten Bestätigung eingereihte Eingaben erschienen erst in Revision 3.
Die Schließabfrage blieb über die Schreibpause hinaus offen. Verwerfen und anschließendes
Wiederöffnen bestätigten Revision 3 samt Absatzstruktur und Suchindex. Beim letzten
Journalzug ließ eine Schreibpause das Fenster offen, bis ausdrücklich gespeichert wurde.

### Lesender Konfliktvergleich im langen Klartext

Nach einem fehlgeschlagenen Schreibzug liest die Fensterrolle die neuere Revision unter
der einmaligen Vergleichsbindung. Links bleibt der lokale Entwurf bearbeitbar, rechts
steht der vollständig geprüfte Text samt Revisionsnummer in einem geschützten Lesefeld.
F6 und Tab wechseln zwischen den Fassungen. Bei einer formatierten Dienstfassung umfasst
der Vergleich die zugelassene Klartextprojektion. Eine Übernahme mit der bisherigen
Zwei-Absatz-Auswahl setzt passende kurze Fassungen und einen passenden kurzen Entwurf voraus.

Im verwalteten Windows-Lauf löste der synthetische Datendienst vor dem Autosave einen
Revisionskonflikt aus. Der Vergleich zeigte die abweichende Revision 2 samt Leerabsatz.
Der lokale Entwurf blieb auch nach weiterer Eingabe erhalten. Beide Speicherbefehle
blieben gesperrt. Nach bestätigtem Verwerfen meldete der Starter die zuletzt veröffentlichte
Revision 1 und einen leeren Job. Der Kanaltest prüft zusätzlich, dass ein direkt versuchtes
Prepare bei langem Ausgangstext, langem Entwurf, langem Dienststand oder reiner
Textprojektion vor der Vorbereitung abgewiesen wird.

### Absatznavigation im langen Konfliktentwurf

F7 und Umschalt+F7 durchlaufen die abweichenden Absatzblöcke zyklisch. Die Suche setzt
am nächstgelegenen gemeinsamen Absatz wieder auf. Fehlt ein Block auf einer Seite,
markiert die Einfügeposition dort sein Gegenstück. Die Revisionsüberschrift gehört
nicht zum Vergleich. Jede lokale Änderung setzt die Navigation zurück.

Der gezielte Modelltest deckt Vorwärts- und Rückwärtssprung sowie UTF-16-Positionen ab.
Im verwalteten Win32-Fenster wurden ein nur lokal vorhandener Absatz und ein zusätzlicher
Dienstabsatz getrennt gefunden. Nach lokalem Textabgleich meldete das Fenster Gleichstand.
Eine weitere Änderung setzte den Sprung zurück. Die Speicheraktionen blieben gesperrt,
und Verwerfen beendete die Sitzung auf ihrer Ausgangsrevision.

### Bestätigter Klartextentwurf

F9 bietet bei einem langen, unformatierten Konfliktentwurf eine ausdrückliche Bestätigung
für Speichern und Schließen. Ein Nein lässt den Entwurf offen. Der separate Prepare-Befehl
prüft unformatierte Quelle, Schreibversuch und Dienststand sowie die Ausgangsrevision.
Der Dienst liest vor der Vorbereitung erneut und verlangt dieselbe vollständige Antwort
auf der angezeigten Revision. Liegt diese schon beim ersten Vergleich mehr als eine Revision voraus,
verlangt der Dienst die Freigabe für den revisionsübergreifenden Schreibzug. Die erste
Bestätigung ist an die angebotene Revision und einen eigenen Protokollzweck gebunden.
Fenster und Dienst liefern dem Starter den Beleg über Revision und Inhalt. Erst der nächste gebundene Auftrag speichert den
bestätigten Entwurf. Eine Textprojektion bleibt auf den lesenden Vergleich beschränkt.

Ändert sich die Dienstrevision zwischen Anzeige und Bestätigung, verfällt die Auswahl
für diesen Schreibzug. Der freigegebene Wiedervergleich liest unter frischer Bindung.
Das Fenster behält den lokalen Text und zeigt die neue Revision. Der Klartextpfad
prüft dabei auch im Dienst erneut die Bearbeitbarkeit und das Fehlen der Fettmarkierung.
Eine weitere F9-Bestätigung bindet den gesamten Entwurf an genau diesen Stand. Beide
Rollen bestätigen denselben revisions- und inhaltsgebundenen Beleg vor dem Folgeauftrag.

### Erste Revisionslücke im strukturierten Editor

Bei freigegebenem revisionsübergreifendem Schreibzug bindet der Datendienst die
strukturierte Auswahl an einen eigenen Protokollzweck und die angebotene Revision.
Er liest den Dienststand vor der Freigabe erneut. Der Beleg umfasst Text und Fettmarkierung.
Das Bestätigungsfenster nennt die Basisrevision und die gewählte Abschlussaktion.
Im synthetischen Windows-Lauf blieben nach „Nein“ Entwurf und Auswahl offen. Nach „Ja“
enthielt Revision 4 den lokalen ersten Absatz, den zweiten Dienstabsatz und die lokale
Fettmarkierung. Beim Weiterschreiben im selben Fenster bestätigte Autosave Revision 5.

Ein weiterer lokaler Windows-Lauf prüfte den Wechsel während der Bestätigung. Zunächst
war Revision 3 sichtbar. Beim erneuten Lesen vor der Freigabe lag Revision 4 vor.
Im selben Fenster blieben der lokale erste Absatz und die lokale Fettmarkierung
ausgewählt. Der zweite Absatz kam aus der neuen Dienstfassung. Das Bestätigungsfenster
nannte Revision 4. Nach „Nein“ blieben die drei Häkchen unverändert und Revision 5
war noch nicht angelegt. Nach erneuter Bestätigung ergab das Wiederöffnen Revision 5
mit der geprüften Mischung aus lokalem Text, neuem Dienstabsatz und lokaler Fettmarkierung.

Für „Auswahl speichern und weiterschreiben“ wurde derselbe Revisionswechsel mit
synthetischen Daten geprüft. Nach der Bestätigung für Revision 4 zeigte dasselbe
Win32-Fenster die gespeicherte Mischung als Revision 5. Die nächste Eingabe im ersten
Absatz löste nach der Schreibpause Autosave aus. Beim Wiederöffnen enthielt Revision 6
die neue Eingabe, den zweiten Absatz der Dienstfassung und die gewählte Fettmarkierung.
Die gespeicherte Mischung war weiterhin als Revision 5 lesbar.

Im Grenzlauf wechselte die Dienstbasis während der Bestätigung auf Revision 254.
Der Bestätigungsdialog kündigte den letzten Autosave an. Nach der Übernahme als Revision
255 zeigte dasselbe Fenster den Hinweis zum automatischen Schließen. Nach einer weiteren
Eingabe speicherte der letzte Autosave Revision 256 und beendete die Sitzung. Der frische
Leseauftrag bestätigte die Eingabe, den zweiten Dienstabsatz und die gewählte Fettmarkierung.

Im letzten Schreibzug hat ein ungültiger Absatz Vorrang im Fenstertitel. Im lokalen
Windows-Lauf blieb das Fenster nach dem Leeren des ersten Absatzes über die Schreibpause
hinweg offen. Für die korrigierte Eingabe wurde ein Fehler beim Einrichten des Timers
synthetisch ausgelöst. Der Titel verwies auf manuelles Speichern und das anschließende
Schließen. Erst der manuelle Speicherbefehl veröffentlichte die letzte Revision. Die
SQLite-Nachprüfung bestätigte den korrigierten Text und den unveränderten zweiten Absatz.

Ein lokaler Lauf mit synthetischer Szene prüfte die Erholung beim Weitertippen.
Bei simuliertem Timerfehler blieb Revision 0 über die Schreibpause hinaus offen.
Die nächste gültige Eingabe startete die Entprellung erneut. Nach einer weiteren
Schreibpause und erneutem Öffnen lag genau Revision 1 vor. Sie enthielt den zuletzt
eingegebenen Text sowie den unveränderten zweiten Absatz mit seiner Fettmarkierung.

### Lokales Suchen und Ersetzen im langen Klartext

Suche, Ersetzung und Änderungsverlauf arbeiten im Fenster auf dem aktuellen Entwurf.
Auch bei geöffneter Vergleichsfassung bleibt der Entwurf das Suchziel. Die Felder für
Such- und Ersatztext nehmen je bis zu 48 UTF-16-Einheiten auf. Die Navigation berücksichtigt
CRLF und Zeichen mit zwei UTF-16-Einheiten. Die vollständige Ersatzfassung muss die
bestehenden Szenengrenzen erfüllen. Ein leerer Ersatztext löscht passende Zeichenfolgen.
Eine Sammelersetzung bildet einen Schritt im fensterlokalen Verlauf. Bei bestätigtem
Autosave bleibt der Verlauf für den gebundenen Folgeauftrag im selben Fenster verfügbar.

Lokale Windows-Läufe mit synthetischem Text prüften die Markierung nach einem Emoji
und nach einer Leerzeile, die Rückwärtssuche, den Fall ohne Treffer, Einzel- und
Sammelersetzung sowie die Abweisung einer zu langen Ersatzfassung. Nach Rückgängig und
Wiederholen über bestätigte Autosaves enthielten die Revisionen 2, 3 und 4 jeweils den
erwarteten Text. Auch der frische Leseauftrag und der Klartextindex wurden geprüft.

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
