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

Im lokalen Windows-Lauf zeigte die geprüfte Rolle den festen Zweizeiler im schreibgeschützten Textfeld. Der Starter prüfte Sichtbarkeit, Fensterklasse und Zugehörigkeit zum primären Thread und Prozess am gemeldeten HWND. Er schloss das Fenster per WM_CLOSE und bestätigte reguläres Prozessende sowie leeren Job. Der Kontrollkopf blieb unverändert. Eine gezielte Regression am Handshake der Rolle quittierte das Ticket und wies anschließend einen fremden Öffnungswert ab. Eine neue ETW-Messung gehört nicht zu diesem Nachweis.

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
