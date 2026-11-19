# ADR 0036 – Native Windows-Oberfläche mit eigener Prozessrolle

**Ergebnis der anschließenden Gastprobe:** Das eigene Win32-Fenster mit gebundener Rolle lief unter
erzwungener UMCI. Tastatur, die im Narrator-Sprachverlauf sichtbaren Ansagen, Jobende und
ETW-Vergleich sind geprüft. Die akustische Abnahme und die Produktintegration bleiben offen.
[Messwerte und Beleggrenzen](../../testing/native-win32-window-guest.md).

Für die Fensterprobe ist der Softwarebeleg der Sprachausgabe als bestanden angenommen. Die
akustische Prüfung ist zurückgestellt. Der nächste Versuch bindet genau eine synthetische Szene an
ein UI-Blatt und einen getrennten Rust-Datendienst. Die native Durchsetzung steht aus.
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
