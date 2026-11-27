# F08-/M3-Abnahmematrix

Stand vom 27. November 2026 auf dem Entwicklungszweig. Der Desktop unterstützt Drehbuchszenen mit
sechs Absatzrollen und einen eigenen A4-PDF-Export für ausgewählte Szenen. Die erste gespeicherte
Drehbuchszene hebt das Projektschema auf 7 an. Reine Prosaprojekte behalten Schema 6. Alpha 7
enthält diese Funktion noch nicht. [Datenformat und Kompatibilität](../data-format/screenplay-v2.md)

## Stand der Drehbuchkriterien

- **DB-01 · Bearbeiten.** Der Studio-Bedienlauf deckt sechs Rollen und konkurrierendes Autosave ab.
  Abbrechen verwirft die Rollenauswahl. Umwandlung, Rollenwechsel mit Undo/Redo und Rückwandlung
  erhalten Fettdruck, Kursivschrift und harten Umbruch im synthetischen Manuskript. Der installierte
  Desktop belegt den Neustart mit Szenenkopf und Handlung. In der isolierten Installation aus
  `df2ee4ca165f` wurde zusätzlich eine leere Prosaszene im Studio umgewandelt. Rückgängig zeigte
  wieder Prosa, Wiederholen wieder Drehbuch. Nach Speichern und regulärem Ende enthielt SQLite genau
  einen leeren Szenenkopf in v2 unter Schema 7. Nach dem Prozessneustart öffnete dieselbe Szene im
  Drehbuchmodus. Projekt-, Kapitel- und Szenenkennung stimmten vor der Umwandlung, nach dem
  Speichern und nach dem zweiten Programmende überein. Der Lauf nutzte lokale UI-Automation und
  synthetische Daten. Eine drehbuchspezifische Sprachausgabeprüfung ist offen und auf Wunsch
  zurückgestellt.
- **DB-02 · Validieren.** Ein Dialog ohne Figur und ein zweiter Szenenkopf bleiben als Entwurf
  speicherbar. Der Druck weist beide Folgen vor der Dateianlage ab. Der Studio-Druckhinweis und der
  Vorschlag für den ersten Szenenkopf behandeln U+0085 wie die native Druckprüfung als
  Unicode-Leerraum. Damit zählt ein alleinstehendes U+0085 bei der Rollenprüfung als leer.
- **DB-03 · Wiederherstellen.** SQLite-Backup und v4-Archiv erhalten Rollen, Umbruch, Formatierung,
  Revision und FTS-Suchbarkeit. Der Archivimport erzeugt eine eigene Projektkennung.
- **DB-04 · Schrift.** Vier gebündelte Schriftschnitte, lateinische, griechische und kyrillische
  Stichproben sowie kombinierende Zeichen über Textknotengrenzen sind geprüft. Ein fehlendes Emoji
  führt zur Ablehnung und die vorhandene PDF bleibt bytegleich. Für zwei Wörter mit 2.623 und 2.806
  Zeichen bestätigte das PDF-Rücklesen genau 43 bzw. 46 Zeilen mit je 61 Zeichen in ursprünglicher
  Reihenfolge.
- **DB-05 · Satzmaße.** Für die vollständige synthetische A4-Szene sind Schriftgröße, Einzüge aller
  Rollen, rechter Übergangsrand und Grundlinienabstände unabhängig vermessen. Bei den beiden
  Wortumbruchproben lagen die sichtbaren Zeichen auf allen sechs gerasterten Seiten horizontal
  zwischen 29,99 und 184,86 mm. Die unterste Pixelkante der kritischen Seite lag bei 288 dpi bei
  277,019 mm ab Blattoberkante, innerhalb eines Rasterpixels der 20-mm-Randvorgabe. Für die rechten
  Umbruchgrenzen von Dialog und Klammeranweisung fehlt hier eine unabhängige Einzelmessung.
- **DB-06 · Seitenwechsel.** Figurenzeile und die ersten zwei Dialogzeilen bleiben auch bei
  getrennten Dialogabsätzen zusammen. Eine unteilbare Gruppe von 52 Figurenzeilen scheitert vor der
  Veröffentlichung. Zwei rückgelesene PDFs belegen den Seitenwechsel am Szenenkopf. Die beiden
  folgenden Handlungszeilen stehen jeweils auf derselben Seite wie der Kopf. Alle 65 Dialogzeilen
  folgen über zwei Seiten genau einmal in der richtigen Reihenfolge.
- **DB-07 · Austausch.** Word und LibreOffice erhielten beim Speichern alle sechs Rollen und den
  Dialogumbruch. Im [kanonischen DOCX-v2-Austausch](../data-format/DOCX_ROUNDTRIP_V2.md) prüft der
  native Reader Absatzmarker, Inventarrolle und effektiven Absatzstil. Eine korrekt abgeleitete
  Dialogvorlage wurde akzeptiert, widersprüchliche Rollenbindungen wurden abgewiesen. Der
  Desktop-Command exportierte ein gemischtes Manuskript und veröffentlichte die zurückgelesene Kopie
  mit neuen Projekt- und Dokumentkennungen. Nach Wiederöffnen enthielt sie Prosa v1 und den
  ursprünglichen Drehbuchentwurf v2 mit leerer Handlung, Fett/Kursiv und hartem Umbruch. SQLite
  meldete Schema 7 und die Volltextsuche fand die Szene. Word 16.0.17932 öffnete das separate
  v2-Testmanuskript. Nach einer Textänderung im markierten Prosaabsatz speicherte Word die Datei mit
  allen 27 sichtbaren Inhaltssteuerelementen. Der native Reader las den geänderten Text und
  sämtliche erwarteten Rollen und Formatierungen zurück. Nach Import derselben Word-Datei und
  erneutem Öffnen entsprach der SQLite-Inhalt dem zurückgelesenen Manuskript. Die FTS-Suche nach dem
  geänderten Wort fand einen Treffer. Die Word-Datei blieb bytegleich.

  LibreOffice 26.8.3 öffnete anschließend ein erweitertes synthetisches v2-Manuskript mit allen
  sechs Rollen und einem leeren Handlungsabsatz. Über die lokale UNO-Schnittstelle wurde „Original“
  im bestehenden Prosaabsatz durch „LibreFassung“ ersetzt. LibreOffice speicherte die bearbeitete
  DOCX als neue Datei. Der native Reader las den geänderten Text und das vollständige erwartete
  Manuskript einschließlich Fett/Kursiv und hartem Umbruch zurück. Für die Probe wurde das signierte
  Windows-x64-Paket der Document Foundation separat entpackt. LibreOffice verwendete ein frisches
  Profil mit deaktivierten Makros. Nach Import derselben Datei und erneutem Öffnen der Projektkopie
  stimmten das gesamte Manuskript und beide Editorversionen. Die Datenbank verwendete Schema 7, die
  Volltextsuche fand „LibreFassung“ genau einmal. Die Quelle blieb bytegleich.

- **DB-08 · Desktop-Export.** Szenen- und Kapitelauswahl sind im gebauten Desktop belegt. Ein Fehler
  im produktiven Studio-Export lässt die vorhandene PDF bytegleich. Beim Gesamt-PDF kommen Rollen-
  und Seitenhöhenfehler als feste Hinweise an. Der Studio-Adapter verwirft abweichende Meldungen
  einschließlich angehängter Fremdtexte. Im nativen Command-Test lehnte der bebilderte PDF-Export
  eine gültige Bildauswahl nach Umwandlung der Szene in ein Drehbuch ab. Alle vier vorhandenen
  Paketverzeichnisse und die Bytes der vorhandenen PDF blieben erhalten. Der Studio-Bedienlauf in
  Edge mit lokal simuliertem IPC bestätigte den sichtbaren Hinweis auf die Textausgabe.

## Windows-Kandidat und Drehbuch-Rundlauf vom 27. November

### DOCX-v2-Import nach frischer Offlineinstallation

Der unsignierte x64-Installer aus `df2ee4ca165f` (SHA-256
`5b37094fd784a86adceb91eec86dce7613305e68c7d84987e96c2c45c73ff3ae`) installierte den Desktop auf
einem frisch eingerichteten Windows Server 2025 Standard mit Desktop Experience. Vor dem Gaststart
meldete Hyper-V null virtuelle Netzadapter und null aktive Integrationsdienste. Die Installation
endete mit Exitcode 0. Der Projektkatalog war beim ersten Programmstart leer.

**Projektverwaltung öffnen → Manuskript-DOCX auswählen** führte von der nativen Projektübersicht
über die Studio-Startseite zum Windows-Dateidialog. Der Import der synthetischen v2-Datei zeigte den
erwarteten Projekttitel. Nach Schließen und erneutem Start enthielt die native Projektliste genau
eine Kopie. Diese ließ sich erneut öffnen. Beide Programmstarts endeten regulär.

Nach dem Gastlauf wurde die geschlossene SQLite-Datenbank unabhängig mit der DOCX-Quelle verglichen.
Die Integritätsprüfung bestand. Schema 7 enthielt drei Kapitel und fünf Szenen sowie neun eindeutige
UUIDs für Projekt, Kapitel und Szenen. Prosa lag in Editorformat v1, Drehbuch in v2 vor. Alle sechs
Rollen und sämtliche Drehbuch-Textläufe stimmten mit der Quelle überein, einschließlich Leerabsatz,
Fett/Kursiv und hartem Umbruch. Die Volltextsuche nach „STELLWERK“ fand genau die Szene „Abfahrt“.
Die Prüfsumme der DOCX blieb beim Import unverändert. Der Netzbeleg dieses Laufs ist die fehlende
Netzanbindung des Gasts. Das Verhalten an einem angeschlossenen Netzwerk wird gesondert bewertet.
Nach der Auswertung wurden Prüf-VM und Testplatte entfernt.

### Druckproben

In einem separaten Windows-Bedienlauf mit der lokal gebauten Debug-EXE entstand in einem
synthetischen Projekt zunächst eine gültige gemischte Kapitel-PDF mit 704.878 Byte. Nach dem
Neustart enthielt die Drehbuchszene einen Dialog ohne Figurenzeile. Nach **Nein** bei der
Exportbestätigung und erneut nach **Ja** blieb die einzige vorhandene Datei unter demselben Namen
bytegleich (SHA-256 `1d477f6665896a2cc0841e681634de6d1659874c04fd1464fbac25ab0ef85d0b`). Im zweiten
Fall zeigte das native Fenster „Dialog benötigt eine Figur.“. Beide Desktopprozesse endeten regulär.
Das isolierte Testprofil wurde entfernt.

Der unsignierte Installer aus dem grünen PR-Stand `8b6bb61348e0` umfasst 320.147.064 Byte (SHA-256
`161021972105fee6c149baefe8ab6d0261addc0b6f12da9a0a6777e5ecfce1fd`). Die aus diesem Setup
extrahierte Desktop-EXE öffnete ein synthetisches Projekt in einem frischen Profil. Nach der
Bestätigung im nativen Exportdialog entstand genau eine Kapitel-PDF. Die App endete regulär. Die
Installationsprobe im frischen Windows-Gast weiter unten gilt für den Vorgänger.

In der Drehbuchszene waren „Zu“ und das kombinierende Trema mit „rich“ getrennte, fett-kursive
Textknoten. Unabhängiges Rücklesen ergab das zusammengesetzte Wort „Zürich“ in der richtigen
Textreihenfolge. Die eingebettete Schrift der sechs Zeichen hatte Gewicht 700 und das Kursivmerkmal.
Auf der gerasterten A4-Seite war das Wort vollständig lesbar. Alle drei Marker des Nachbarkapitels
fehlten. Die PDF umfasste 842.242 Byte.

Für DB-02 prüft der vorhandene Auszugstest zwei speicherbare Entwürfe mit ungültiger Druckfolge:
einen Dialog ohne Figur am Szenenanfang und einen zweiten Szenenkopf. Beide Aufträge endeten vor der
Dateianlage. Für DB-03 wurde eine synthetische Drehbuchszene aus dem SQLite-Backup und aus einem
v4-Archiv wiederhergestellt. In beiden Fällen stimmten Szenen-ID, Elternkapitel, Revision, Rollen,
Formatierung und harter Umbruch. Eine FTS5-Suche im Szenentext fand die erwartete Szene. Beim
Archivimport erhielt das Projekt eine neue Kennung. Für DB-01 arbeiteten im vorhandenen
Browser-Bedientest zwei Editorsitzungen auf derselben Szenenrevision. Die zweite speicherte eine
geänderte Rolle und einen neuen Text. Beim anschließenden Autosave der ersten Sitzung meldete der
Editor einen Speicherfehler und behielt den lokalen Text samt Übergangsrolle im offenen Fenster.
Beim Wiederöffnen in der zweiten Sitzung war ihre gespeicherte Handlung samt Text erhalten. Dieser
Konfliktnachweis verwendet die Studio-Oberfläche mit synthetischem IPC-Dienst.

## Installationsprobe im netzlosen Windows-Gast am 27. November 2026

Geprüft wurde der unsignierte x64-Kandidat `5eaadf32b236` mit SHA-256
`c282818e445dc69fdc702922716015142e4c3558575dd028fc79485645dd5cf9`. Der Prüfplatz erhielt eine
frische Windows-Installation vom lokalen ISO (Windows Server 2025 Standard Evaluation mit Desktop
Experience). Hyper-V meldete beim Start null virtuelle Netzadapter und null aktivierte
Integrationsdienste. Die stille Installation endete mit Exitcode 0. Der Prüfhelfer bestätigte das
Installationsziel anhand der Registrierung und startete die dort installierte Desktop-EXE.

Im installierten Desktop öffnete der Prüflauf das synthetische Projekt über die native
Projektübersicht, wählte ein Kapitel mit Prosa und Drehbuchszene und bestätigte dessen PDF-Export.
KRAQEL erzeugte genau eine Kapiteldatei und endete beim Schließen über die Oberfläche mit
Exitcode 0. Das unabhängige Rücklesen auf dem Host bestätigte eine A4-Seite mit allen neun
erwarteten Titeln und Textstellen in Reihenfolge. Titel, Szenenname und Kontrolltext des zweiten
Kapitels fehlten. Die Ausgabe umfasste 704.878 Byte; SHA-256 dieser Probe:
`1d477f6665896a2cc0841e681634de6d1659874c04fd1464fbac25ab0ef85d0b`.

Die ersten beiden Anläufe zeigten Fehler im temporären Bedienhelfer. Für den bestandenen Lauf wurde
derselbe Installer erneut in einem frischen Gast installiert. Nach dem regulären Gastende wurden
Prüf-VM und Testplatte entfernt. Der Nachweis gilt für Windows Server 2025 mit Desktop Experience
und die native Kapitelansicht. Der physische Zweitrechnertest und eine Netzwerkverkehrsmessung sind
davon getrennt.

## Kapitel- und Manuskriptdruck mit Prosa und Drehbuch

Beim Export eines synthetischen Manuskripts entstanden zwei A4-Seiten. Beide wurden mit Poppler
vollständig gerastert und visuell geprüft. Auf Seite 1 folgen auf Projekt-, Kapitel- und Szenentitel
formatierte Prosa, Liste, Zitat und die erste Drehbuchszene mit ihrem Gliederungstitel. Die Handlung
läuft über den Seitenwechsel. Seite 2 enthält die restlichen Handlungszeilen, Figur,
Klammeranweisung, typografisch ausgezeichneten Dialog und rechtsbündigen Übergang. Anschließend
folgen das nächste Kapitel, der nächste Szenentitel und der abschließende Prosatext. Das PDF wurde
zusätzlich maschinell rückgelesen.

Der native Exportpfad wurde mit Kapitel-, Manuskript- und Einzelszenenauswahl geprüft. Die
Einzelszene enthielt nur die Rollenabsätze. Im Desktop-Integrationstest erzeugte der gewöhnliche
Studio-Export die gemischte Gesamt-PDF. Nach Beschädigung eines benötigten Kapitelinhalts scheiterte
der Gesamtdruck. Die vorhandene Ausgabe blieb bytegleich und der gezielte Rollenexport der intakten
Szene gelang. Die Belege gelten für die Textausgabe.

## Unicode und untrennbare Gruppen im Drehbuchdruck (DB-04 und DB-06)

Der PDF-Rücklesetest vom 27. November prüft einen fett-kursiven Dialog, in dem der Grundbuchstabe
und sein kombinierendes Trema in zwei benachbarten Textknoten liegen. Die ausgegebene PDF enthält
„Zürich“. Schriftschnittwechsel und harte Umbrüche begrenzen die Normalisierung. Im selben
Exportpfad wurde eine druckgrammatisch gültige Gruppe aus 52 Figurenzeilen und anschließendem Dialog
vor der Veröffentlichung als zu hoch zurückgewiesen. Sowohl nach diesem Fehler als auch nach einem
nicht druckbaren Zeichen blieb die zuvor erzeugte PDF bytegleich.

## Ausgewählte Drehbuchszenen drucken (DB-05, DB-06 und DB-08)

Der bestehende QA-Generator schreibt mit dem produktiven Renderer eine A4-Probe mit allen sechs
Rollen. Poppler rasterte die ganze Seite. Bei der Sichtprüfung waren Szenenkopf, Figur,
Klammeranweisung, beide Dialogzeilen und Übergang vollständig lesbar. Die unabhängige Messung mit
pdfplumber ergab 595,276 × 841,890 pt und durchgängig 12 pt Schriftgröße. Szenenkopf und Handlung
beginnen bei 30 mm, Figur bei 85 mm, Klammeranweisung bei 70 mm und Dialog bei 55 mm vom linken
Papierrand. Der Übergang endet bei 185 mm. Die Grundlinien liegen im Dialogblock 14 pt auseinander,
an den gemessenen Absatzgrenzen 26 pt. Die Messung gilt für diese synthetische Ausgabe.

Eine zweiteilige Dialogeröffnung an der Seitengrenze wird jetzt mit der Figurenzeile zusammen
umgebrochen, auch wenn die beiden Dialogzeilen getrennte Absätze sind. Beim Rücklesen lagen alle
drei auf Seite 2. Der bestehende Desktop-Integrationstest verwendete den produktiven Aufruf des
Studio-Exports mit synthetischen Daten. Eine leere Auswahl erzeugte keine Datei. Ein unlesbarer
Kapitelinhalt hinderte den Druck der gewählten Szene nicht. Nach einer ungültigen Kapitelwahl und
nach einer druckungültigen Figurenzeile ohne Dialog blieb die zuvor erzeugte PDF jeweils bytegleich.
Der SQLite-Snapshottest prüfte das Lesen während eines parallelen Schreibvorgangs. Dies sind
Teilnachweise für DB-05, DB-06 und DB-08.

## Native Manuskriptauszüge und Exportverlauf

Die aus dem unsignierten Installer `0af9bd27cf55` extrahierte Desktop-EXE lief zweimal mit einem
isolierten synthetischen Projekt. Der Verlauf war zunächst leer. Nach Ablehnung der Bestätigung
entstand keine Datei. Zwei Markdown-Exporte derselben Szene hatten verschiedene Namen und
identischen Inhalt. Die Nachbarszenen fehlten wie vorgesehen. Kapitel-HTML und Manuskript-DOCX
wurden inhaltlich geprüft, die PDF-Datei strukturell. Nach dem Neustart stimmten Zielordner, alle
fünf Dateinamen und Größen. Die fünf Prüfsummen blieben gleich. Das Prüfprofil und die extrahierte
EXE wurden entfernt.

Der SQLite-Test prüfte die Revision bei gleichzeitigem Schreiben und die Begrenzung auf ausgewählte
Inhalte. Ein Windows-Verzeichnistest bestätigte, dass ein umgeleiteter Exportordner (Junction) weder
zur Ausgabe noch zur Auflistung zugelassen wurde. Der gezielte PDF-Test belegte die A4-Ausgabe für
eine ausdrücklich ausgewählte Drehbuchszene. Der damalige Kandidat sperrte Kapitel- und
Manuskript-PDFs mit Drehbuchanteilen. Den später freigegebenen Textdruck für gemischte Kapitel
beschreibt der [Windows-Rundlauf](#windows-kandidat-und-drehbuch-rundlauf-vom-27-november).

SHA-256 des Installers: `608086783077e9cf980a207a90b4f18b4f046d037a19194080f99b8b0178a869`.

## Kapitel und Szenen direkt einordnen

Die aus dem finalen Windows-Installer `3801e1e089b8` extrahierte Desktop-EXE öffnete ein
synthetisches Projekt. Bei nur einem Kapitel war die Platzwahl deaktiviert. Die Liste zeigte den
aktuellen Platz als Vorauswahl. Abbrechen behielt den bisherigen Stand bei. Ein Kapitel rückte von
Platz 3 auf 1 und eine Szene von Platz 1 auf 3. In beiden Fällen blieb der verschobene Eintrag
markiert. Nach einem vollständigen Desktopneustart stimmten beide dreiteiligen Reihenfolgen. Der
erweiterte SQLite-Ordnungstest bestätigte auch die Sperre bei veralteter Gliederung, die
Bereichsprüfung und den unveränderten Zustand bei Wahl des bisherigen Platzes. Das Prüfprofil und
die extrahierte EXE wurden entfernt.

Der unsignierte Installer enthält auch die unten geprüften Kopierabläufe und die kombinierbaren
Suchoptionen. SHA-256: `a6f96bab046bdf2fb10305635132bea46e5fb53edc20e7d31ab07546f63376f5`.

## Unabhängige Szenen- und Kapitelfassungen

Die native Bedienprobe lief mit dem Windows-Releaseprogramm zum Commit `c8279c7c2753` und einem
eigenen synthetischen Projekt. Eine Szene wurde zunächst im selben Kapitel kopiert. Die Kapitelkopie
mit zwei Szenen wurde einmal abgebrochen und anschließend bestätigt. Danach wurde eine Szene ins
andere Kapitel kopiert, nur diese Variante bearbeitet und gespeichert. Nach vollständigem
Prozessneustart zeigten drei getrennte Leseöffnungen die erwarteten Texte. SQLite enthielt sieben
Gliederungseinträge. Die ursprüngliche Szene hatte Revision 1, die entsprechende Szene der
Kapitelkopie Revision 0 und die eigenständig bearbeitete Variante Revision 1. Das Prüfprofil wurde
entfernt.

Zwei gezielte SQLite-Tests belegen den Erhalt kanonischer formatierter Prosa und von Drehbuchrollen,
neue Kennungen, Szenenreihenfolge und Volltextindex. Ein inzwischen geändertes Ziel oder eine
veraltete Kindrevision blockierte die Kopie. Beim Fehler nach Beginn einer Kapitelkopie enthielt die
Gliederung anschließend keinen Teileintrag.

## Schreibweisen bei Szenensuche und Ersatz

Leser und Editor wurden mit „Raben Rabe rabe RABE Rabe_2.“ geöffnet. Die native Bedienprobe fand für
„Rabe“ zunächst drei Treffer, nach Abschalten von **Groß-/Kleinschreibung beachten** fünf, mit
zusätzlicher Ganzwortsuche drei. Nach dem Umschalten war die vorherige Markierung für Einzelersatz
gesperrt. Der Editor ersetzte das ausgewählte kleingeschriebene Wort einzeln und auf ausdrückliche
Bestätigung alle drei selbständigen Schreibweisen. „Raben“ und „Rabe_2“ blieben erhalten. Rückgängig
stellte jeweils den Ausgangstext her. Beide Fenster schlossen regulär. Die Probe lief am
kompilierten nativen Testziel. Der Unicode-Regressionstest bestätigt außerdem, dass als Ersatz
eingegebenes „k“ die Originaltreffer „K“, „K“ und „k“ einheitlich wiedergibt.

## Ganze Wörter in der nativen Szenensuche

Die native Fensterprobe zum Quellstand `aa8b8ac684e2` öffnete Leser und Editor mit „Raben Rabe
Rabe_2 Rabe.“. Beide starteten mit Teilwortsuche und fanden „Rabe“ viermal. Mit **Nur ganze Wörter**
gab es zwei Treffer, der erste lag im UTF-16-Bereich 6 bis 10. Nach dem Moduswechsel wies der Editor
den zuvor markierten Teiltreffer beim Ersetzen zurück. Einzelersatz änderte ein ganzes Wort, der
bestätigte Gesamtersatz beide. Rückgängig stellte jeweils den ursprünglichen Text wieder her. Beide
Fenster schlossen regulär. Ein gezielter Regressionstest deckt zusätzlich kombinierende Zeichen und
einen überlappenden gültigen Treffer nach einem verworfenen Teiltreffer ab.

## Direkter Absatzsprung im Fassungsvergleich

Im kompilierten Win32-Dialog wurde bei drei synthetischen Absätzen die Quelle für Absatz 1
gewechselt. Strg+G fokussierte das Nummernfeld. Die Eingabe 4 mit Enter zeigte den gültigen Bereich
an und erhielt Position und Feldfokus. Die Eingabe 3 und die Sprungschaltfläche zeigten beide
Fassungen von Absatz 3 mit Fokus links. Die Gesamtvorschau blieb gleich. Rückgängig nahm
anschließend die vorherige Quellwahl zurück. Die Probe bediente die nativen Steuerelemente über
Win32-Nachrichten.

## Speichergrenze der Absatzmischung

Die lokale Win32-Dialogprobe zum Quellstand 3a80dc7 verwendete synthetische Absätze mit Umlauten und
Absatzwechseln. Bei 65.536 UTF-8-Bytes war die Auswahl speicherbar. Nach dem Quellenwechsel auf
65.537 Bytes zeigte die Vorschau die Überschreitung um ein Byte. Auch ein direkt ausgelöster
Speicherbefehl ließ den Dialog offen. Rückgängig stellte die ursprüngliche Mischung wieder her und
die ausdrückliche Bestätigung schloss den Dialog erfolgreich. Geprüft wurde der native Dialog im
Testziel. Die letzte Installer- und ETW-Abnahme gehört zum unten genannten Vorgänger.

## Gesamtfassung wählen und Absatzentscheidungen zurücknehmen

Im installierten Kandidaten `1e89f10bf482` wurden beide Gesamtfassungen als Ausgangswahl geprüft.
Die Vorschau folgte auch den anschließenden Einzelentscheidungen, Strg+Z und Strg+Y. Eine neue
Quellwahl nach Rücknahme verwarf den alten Wiederholen-Zweig. Während der Auswahl schrieb eine
zweite Verbindung Revision 2 der synthetischen Szene. Die erste Bestätigung öffnete daraufhin den
Vergleich mit der neuen Basis. Die bisherigen Positionsentscheidungen und der lokale Entwurf blieben
erhalten. Die Vorschau zeigte den aktualisierten gespeicherten Absatz. Die alten Auswahlschritte
ließen sich auf der neuen Basis weder zurücknehmen noch wiederholen.

SQLite blieb bis zur erneuten ausdrücklichen Bestätigung auf Revision 2. Nach dem Speichern und
Neustart stimmten der gemischte Text mit sechs Absätzen, der Suchindex und Revision 3. Der
kombinierte Bedien- und ETW-Lauf erfasste bei beiden Desktopstarts je eine Fensterrolle und einen
Datendienst. Beide Prozessbäume zeigten null Netzwerkereignisse einschließlich Loopback. Alle vier
Positivkontrollen wurden erkannt, ohne Verluste oder Zuordnungslücken. Testinstallation,
synthetisches Profil und Rohspur wurden bereinigt. SHA-256 des Installers:
`771fe5feef27bcdb45c5389336d9c8ef7db3f33371e0fcd928825335da548c04`.

## Absatzunterschiede im nativen Konfliktvergleich

Die synthetische Szene im installierten Kandidaten `c30187a585b4` umfasste sechs Absätze mit
Unterschieden an den Positionen 2, 4 und 6. Der Vergleich öffnete bei Absatz 2 und zeigte
Unterschied 1 von 3 an. F7 führte über 4 und 6 zurück zu 2, Umschalt+F7 von 2 zurück zu 6. F6
wechselte zwischen den beiden Fassungen. Beim Navigieren blieben Entwurf und Gesamtvorschau
unverändert. Auch nach der Quellwahl lag die Datenbank noch auf Revision 1. Erst die ausdrückliche
Bestätigung speicherte Absatz 2 aus der gespeicherten Fassung sowie die Absätze 4 und 6 aus dem
Entwurf als Revision 2. Nach dem Desktopneustart stimmten die sechs Absätze im Leser, der kanonische
Inhalt und der Suchindex.

Die kalibrierte ETW-Messung erfasste beim Vergleichen und Wiederlesen je einen Desktop, eine
Fensterrolle und einen Datendienst. In beiden Prozessbäumen wurden null Netzwerkereignisse
einschließlich Loopback beobachtet. Alle vier Positivkontrollen wurden erkannt, ohne Verluste oder
Zuordnungslücken. Testinstallation, synthetisches Profil und Rohspur wurden bereinigt. SHA-256 des
Installers: `1c63b29fbd1b8760e1ba8d375db036bd57d4e45e27a1473200aa4c9b3dff8497`.

## Speicherstatus, ausdrücklicher Wiederholungsversuch und Revisionskonflikt

Im installierten Kandidaten `64c1fe30857d` zeigte der Fenstertitel nach Tippen, Rückgängig und
Wiederholen an, ob der sichtbare Entwurf dem bestätigten Stand entsprach. Der Änderungsstatus war
auch im Fokusmodus sichtbar. Eine SQLite-Schreibsperre bis zum Timeout löste einen Speicherfehler
aus. Der Entwurf blieb offen. Autosave war ausgeschaltet und bis zum manuellen Erfolg gesperrt. Nach
Freigabe der Sperre und einer weiteren Schreibpause lag die Szene weiter auf Revision 0. Erst
Strg+Enter speicherte Revision 1.

Ein konkurrierender Schreibzug legte eine abweichende Fassung auf Revision 2 ab. Der nächste
Autosave führte zum Revisionskonflikt, erhielt den lokalen Entwurf, sperrte normales Speichern und
gab den Fassungsvergleich frei. Nach ausdrücklichem Verwerfen des lokalen Entwurfs und einem
Desktopneustart stimmten der schreibgeschützt angezeigte Text, die gespeicherten Absätze und der
Suchindex der Fremdfassung auf Revision 2. Der kombinierte Lauf nutzte ein synthetisches Manuskript.
Bei beiden Desktopstarts waren je eine Fensterrolle und ein Datendienst zugeordnet. Beide
Prozessbäume zeigten null Netzwerkereignisse einschließlich Loopback. Vier von vier
Positivkontrollen wurden erkannt, ohne Verluste oder unklare Zuordnung. Installation, synthetisches
Profil und Rohspur wurden bereinigt. SHA-256 des Installers:
`e8e3fcb584916b565ebc9f6f571737e1dcd84814aadf61d8c16ce940a2b58f35`.

## Lokales Autosave mit Folgeeingabe und Schließanforderung

Im installierten Kandidaten `65e46b7e9237` war Autosave zunächst aus. Nach 900 ms Schreibpause lag
die synthetische Szene weiterhin auf Revision 0. Nach dem Einschalten wurde der automatische
Schreibzug 728 ms nach der Eingabe beobachtet. Eine kurzzeitige SQLite-Schreibsperre verzögerte die
Bestätigung. Währenddessen nahm das Szenenfeld ein weiteres Zeichen an. Der Schließwunsch wartete
auf die Bestätigung. Als die Schließabfrage erschien, enthielt Revision 1 ausschließlich den zuerst
gesendeten Stand. Nach Weiterschreiben wurde im selben Fenster die Folgeeingabe als Revision 2
gespeichert. Eine weitere Pause ohne Änderung erzeugte keine Revision.

Nach dem Ausschalten blieb eine neue Eingabe bis Strg+S lokal. Nach dem Neustart stimmten der
schreibgeschützte Text, der Suchindex und Revision 3. Der Leser bot keinen Autosave-Befehl an. Ein
gezielter Zeitgebertest prüfte veraltete und wiederholte Zeitgeberereignisse. Der gemeinsame Bedien-
und ETW-Lauf erfasste bei beiden Desktopstarts je eine Fensterrolle und einen Datendienst. Beide
KRAQEL-Prozessbäume zeigten null Netzwerkereignisse einschließlich Loopback. Alle vier
Positivkontrollen wurden erkannt, ohne Verluste oder Zuordnungslücken. Testinstallation,
synthetisches Profil und Rohspur wurden bereinigt. SHA-256 des Installers:
`1cbf93196bf042e6e196d66c01d392c8bf294b54705f763d3c76f7d65e6b5d0c`.

## Speichern und im selben Szenenfenster weiterschreiben

Im installierten Kandidaten `62ac9104ce0d` speicherte Strg+Enter im Szenenfeld die erste Änderung
als Revision 1. Prozess, Fenster, Fokus, Textauswahl und der Rückgängig-Verlauf blieben erhalten.
Der erneute Menübefehl ohne Textänderung meldete „Keine Änderungen zu speichern“ und erzeugte keine
Revision. Nach Rückgängig und einer weiteren Eingabe blieb SQLite auf dem bestätigten Zwischenstand,
bis Strg+S den neuen Entwurf als Revision 2 speicherte. Nach dem Desktopneustart stimmten der
schreibgeschützte Text, der kanonische Inhalt und der Suchindex. Im Leser fehlte der Menübefehl zum
Zwischenspeichern.

Der gezielte Speichertest prüfte zwei aufeinanderfolgende Schreibzüge und einen konkurrierenden
Commit. Der danach versuchte Schreibzug unter der alten Bindung wurde abgewiesen, die Fremdfassung
blieb erhalten. Der kombinierte Bedien- und ETW-Lauf mit synthetischem Text erfasste beim Bearbeiten
und Wiederlesen je einen Desktop, eine Fensterrolle und einen Datendienst. Beide Prozessbäume
zeigten null Netzwerkereignisse einschließlich Loopback. Vier von vier Positivkontrollen wurden
erkannt, ohne Verluste oder Zuordnungslücken. Testinstallation, synthetisches Profil und Rohspur
wurden bereinigt. SHA-256 des Installers:
`a8e82a9a5b8a0df11d06993bf79ee71b08ac6dc6e31d1408290f5e1865dade2a`.

## Mehrstufiger Bearbeitungsverlauf im nativen Szenenfenster

Im installierten Kandidaten `b7dd0426e81a` wurden eine markierte Emoji-Stelle überschrieben und ein
weiterer Unicode-Treffer ersetzt. Zweimal Rückgängig über Strg+Z und das Menü stellte die jeweiligen
Texte und die ursprünglichen Auswahlen wieder her. Zweimal Wiederholen über Strg+Y und das Menü
stellte beide Änderungen samt Einfügemarken wieder her. Eine neue Eingabe nach der Rücknahme verwarf
den Wiederholen-Zweig. Der Menüpunkt war deaktiviert und Strg+Y ließ den neuen Entwurf unverändert.
Bis Strg+S blieb die Datenbank auf Revision 0. Nach dem Neustart stimmten Text, Suchindex und
Revision 1. Der Leser bot keine Menübefehle für den Schreibverlauf.

Eine gezielte Regression prüfte unvollständige UTF-16-Zeichen und den Neubeginn des Verlaufs nach
Überschreiten seines Textpufferbudgets. Der gemeinsame Bedien- und ETW-Lauf mit synthetischem Text
erfasste bei beiden Desktopstarts je eine Fensterrolle und einen Datendienst. Beide
KRAQEL-Prozessbäume hatten null Netzwerkereignisse einschließlich Loopback. Alle vier
Positivkontrollen wurden erkannt, ohne Verluste oder Zuordnungslücken. Testinstallation,
synthetisches Profil und Rohspur wurden bereinigt. SHA-256 des Installers:
`24133df438446139ead9d1c2cf72029e67dd388eb197f2ea42d9c683bebf7874`.

## Native Szenenmenüs im Editor und Leser

Beim installierten Kandidaten `e7a44897c58f` wurden die Menüeinträge samt Befehlskennungen und
Beschriftungen direkt an den nativen Fenstern geprüft. Der Editor bot Datei, Bearbeiten und Ansicht,
der Leser Datei und Bearbeiten mit Lesefunktionen. Der Fassungsvergleich war vor einem Konflikt
gesperrt. Der Menühaken für den Fokusmodus folgte dem Ein- und Ausschalten der Schreibansicht. Eine
an das Fenster adressierte Alt+B-Tastenmeldung öffnete das native Bearbeiten-Menü. Die Zugriffstaste
S öffnete dort die Suche.

Der Prüfläufer löste Speichern im Editor und Schließen im Leser anhand der geprüften Menükennungen
über WM_COMMAND aus. Nach dem Neustart stimmten der schreibgeschützte Szenentext, der Suchindex und
Revision 1. Der gemeinsame Bedien- und ETW-Lauf mit einem synthetischen Manuskript erfasste in
beiden Prozessbäumen je einen Desktop, eine Fensterrolle und einen Datendienst. Es wurden jeweils
null Netzwerkereignisse einschließlich Loopback beobachtet. Alle vier Positivkontrollen wurden
erkannt, ohne Verluste oder Zuordnungslücken. Testinstallation, synthetisches Profil und Rohspur
wurden bereinigt. SHA-256 des Installers:
`8a95a3aecfb1969e53efc6aff1bfcc2e0023fcc8bb9431b7f4ccf62d553f7530`.

## Markierte Stellen suchen und per Tastatur ersetzen

Im installierten Kandidaten `445d32fa7a9d` übernahm Strg+F eine manuell markierte einzeilige
Unicode-Stelle im Editor und im schreibgeschützten Leser als Suchbegriff. Bei einer anschließenden
mehrzeiligen Auswahl blieb in beiden Fenstern der bisherige Begriff erhalten. Strg+H übernahm im
Editor die markierte Stelle anstelle des bisherigen Suchbegriffs und fokussierte das Ersatzfeld.
Enter ersetzte ohne vorherige Treffernavigation genau diese Stelle und ließ den Fokus im Ersatzfeld.
Strg+Enter öffnete für die zwei verbliebenen Treffer den Bestätigungsdialog mit Nein als Vorgabe.

Die SQLite-Szene blieb bis Strg+S auf Revision 0. Nach dem Neustart stimmten der vollständige Text
im Leser, Revision 1 und der normalisierte Suchindex. Der gemeinsame Bedien- und ETW-Lauf nutzte ein
synthetisches Manuskript. Beide Desktopstarts waren mit je einer Fensterrolle und einem Datendienst
vollständig zugeordnet. In beiden Prozessbäumen wurden null Netzwerkereignisse einschließlich
Loopback beobachtet. Vier von vier Positivkontrollen wurden erkannt, ohne Verluste oder
Zuordnungslücken. Testinstallation, synthetisches Profil und Rohspur wurden bereinigt. SHA-256 des
Installers: `1ec1b54e2e9f97ff8ed96b5bbaef9bf669eabbad2dec84963e248608bf0064ae`.

## Alle Treffer in einer Prosaszene ersetzen

Im installierten Kandidaten `eff5ce03a2c7` meldete der Bestätigungsdialog drei Treffer im
synthetischen Entwurf. Nein war vorausgewählt und erhielt den Text. Nach Ja waren alle drei
Unicode-Treffer in zwei Absätzen ersetzt. Ein einziges Rückgängig stellte den vollständigen
Ausgangstext wieder her. Nach erneutem Ersetzen blieb die SQLite-Szene bis zum ausdrücklichen
Speichern auf Revision 0. Nach Strg+S enthielt Revision 1 die erwarteten Absätze und den passenden
Suchindex. Der schreibgeschützte Leser zeigte den gespeicherten Text nach dem Neustart an und bot
keine Ersetzfunktion an.

Der gemeinsame Bedien- und ETW-Lauf erfasste beim Bearbeiten und Wiederlesen jeweils einen Desktop,
eine Fensterrolle und einen Datendienst. Beide zugeordneten Prozessbäume zeigten null
Netzwerkereignisse einschließlich Loopback. Alle vier Positivkontrollen wurden erkannt, ohne
Verluste oder Zuordnungslücken. Testinstallation, synthetisches Profil und Rohspur wurden bereinigt.
Ein gezielter Modelltest prüfte die zulässige UTF-16-Ergebnislänge am Eingabelimit vor der
Speicherreservierung. SHA-256 des Installers:
`bb943fb69f5c26faf39e9521aa8414f7be36a02bd1adcc90ada113eacabe6135`.

## Einzelnen Treffer im Szenenentwurf ersetzen

Im installierten Kandidaten `cc27c0c1fc37` öffnete Strg+H die Ersetzzeile des Editors. Bei
abweichender Auswahl blieb der Entwurf erhalten. Bei gültiger Auswahl änderte „Treffer ersetzen“ nur
den ersten von drei Unicode-Treffern. Rückgängig stellte den vollständigen Ausgangstext wieder her.
Nach erneutem Ersetzen blieb die SQLite-Szene bis zum ausdrücklichen Strg+S auf Revision 0.
Anschließend stimmten die Absätze auf Revision 1 und der für die Suche normalisierte Klartext. Nach
dem Desktopneustart zeigte der Leser den vollständigen Text schreibgeschützt an. Die
Ersetzbedienelemente waren dort nicht vorhanden.

Der kombinierte Bedien- und ETW-Lauf verwendete ausschließlich eine synthetische Szene. Bearbeiten
und Wiederlesen wurden in getrennten Desktopstarts erfasst. In beiden Prozessbäumen waren je ein
Desktop, eine Fensterrolle und ein Datendienst zugeordnet. Je Baum wurden null Netzwerkereignisse
einschließlich Loopback beobachtet. Alle vier Positivkontrollen wurden erkannt, ohne verlorene
Ereignisse, Puffer oder Zuordnungslücken. Testinstallation, synthetisches Profil und Rohspur wurden
bereinigt. SHA-256 des Installers:
`c289c906559b105ee50fa8790c44f4509781f515aa1cdcd27b0cc2a452037da9`.

## Lokale Szenensuche im installierten Editor und Leser

Im Kandidaten `63d3aca5cebe` öffnete Strg+F die Suche im Fokusmodus des Editors und im
schreibgeschützten Leser. Der synthetische Text enthielt drei gleiche Unicode-Treffer in zwei
Absätzen. F3, Umschalt+F3 und erneut F3 markierten den ersten, den letzten und wieder den ersten
Treffer. Die ausgewählten UTF-16-Bereiche stimmten einschließlich der Windows-Zeilenumbrüche. Bei
einem fehlenden Begriff erschien „Kein Treffer“. Esc schloss die Suche, gab den Fokus an die Szene
zurück und erhielt im Editor den zuvor gewählten Fokusmodus.

Beide Fenster schlossen ohne Speicherabfrage. Klartext, Prüfsumme des gespeicherten JSON-Inhalts und
Revision 0 stimmten vor und nach der Suche überein. Die Bedienprüfung verwendete zielgebundene
Windows-Tastenmeldungen. Testinstallation und synthetisches Profil wurden bereinigt. Die frische
kalibrierte ETW-Messung dieses Installers erfasste die Suche in zwei getrennten Desktopstarts. Zu
jedem Start gehörten genau eine Fensterrolle und ein Datendienst. Beide Prozessbäume hatten null
Netzwerkereignisse einschließlich Loopback. Es startete kein WebView2-Prozess. Vier von vier
Positivkontrollen wurden erkannt, ohne verlorene Ereignisse, verlorene Puffer oder unklare Ziel-
oder Prozesszuordnungen. Die Rohspur wurde bereinigt. SHA-256 des Installers:
`1a147901b4c797c0e952439b7af0c8aecefc926d75eb805401861f842040b88b`.

## Tastenkürzel im installierten Prosaeditor

Im Kandidaten `155509ff5e06` markierte Strg+A den gesamten mehrabsätzigen Unicode-Text im
fokussierten Szenenfeld. Im Fokusmodus löste Strg+S bei 65.537 UTF-8-Bytes die Größenwarnung aus und
erhielt den Entwurf. Nach dem Kürzen speicherte dasselbe Kürzel genau 65.536 Bytes als Revision 1.
Nach dem Desktopneustart stimmten der schreibgeschützt angezeigte Text und der bytegenau geprüfte
SQLite-Inhalt. Testprofil und Installation wurden bereinigt.

Der Prüfläufer adressierte Windows-Tastennachrichten mit synchronisiertem Strg-Zustand direkt an das
gebundene Szenenfeld. Die erste ETW-Messung endete vor der Tastensequenz, weil der Prüfläufer den
Vordergrund des Fensters nicht bestätigen konnte. Der vollständige Bedienlauf wurde anschließend
ohne ETW ausgeführt.

Eine frische kalibrierte ETW-Messung desselben Installers umfasste anschließend die vollständige
Tastensequenz, die Größenwarnung, das Speichern und das Wiederlesen nach dem Neustart. Beide
Desktop-Prozessbäume enthielten je eine Fensterrolle und einen Datendienst. Es wurde kein
WebView2-Prozess gestartet. Vier von vier Positivkontrollen wurden erkannt, ohne verlorene
Ereignisse oder Zuordnungslücken. In beiden Prozessbäumen wurden null Netzwerkereignisse außerhalb
von Loopback beobachtet. Beim Wiederlesen war die Ereignisliste vollständig leer. Für den
Bearbeitungslauf ist die Loopback-Einzelangabe im erhaltenen Ergebnis gekürzt. Installation,
synthetisches Profil und Rohspur wurden bereinigt. Installer-SHA-256:
`0e50ccd9194f06067e62d78b90dfbe361b611d50823c1b1134708875b32dc2cf`.

## Wortzahl, Speichergrenze und Fokusmodus im installierten Prosaeditor

Im Kandidaten `eb7eec27a369` stimmten Wortzahl und UTF-8-Budget bei leerem Text, Umlauten, Emoji und
Windows-Zeilenumbrüchen. F11 und Escape erhielten Text, Auswahlposition und Eingabefokus. Bei 65.537
Bytes warnte der Editor beim Speichern und hielt den Entwurf vollständig offen. Nach dem Kürzen auf
genau 65.536 Bytes wurde Revision 1 gespeichert. Nach einem vollständigen Desktopneustart zeigte das
schreibgeschützte Szenenfenster den gesamten Text. Ein separater Vergleich bestätigte den Inhalt der
synthetischen SQLite-Szene bytegenau.

Die Wiederöffnung am Grenzwert deckte im Vorgänger einen Fehler auf. Das native Textfeld berechnete
den Zeilenumbruch vor der ersten Größenanpassung und überschritt die Startfrist. Der geprüfte
Kandidat setzt die Feldgröße vor dem Einfügen des Textes. Zwei synthetische Prüfprofile des
Vorgängers blieben nach dem fehlenden Endebeleg gesperrt.

In der kalibrierten ETW-Spur waren beiden Desktopstarts jeweils eine Fensterrolle und ein
Datendienst zugeordnet. Beide Prozessbäume hatten einschließlich Loopback null Netzereignisse. Alle
vier Positivkontrollen wurden erkannt. Es gab keine verlorenen Ereignisse oder Puffer.
Testinstallation, Erfolgsprofil und Rohaufzeichnung wurden entfernt. SHA-256 des geprüften
Installers ist `314be1ba82c943913d0ca5fed61b48154fd3d918d807796846e0428f49845e6a`.

## Revisionskonflikt mit absatzweiser Auswahl im installierten Desktop

Im installierten Kandidaten `41bc9f03f02d` erschien die gespeicherte Gegenfassung erst auf
ausdrücklichen Aufruf. Das Abbrechen des Vergleichs erhielt beide lokalen Absätze im selben
bearbeitbaren Fenster und ließ die gespeicherte Revision 1 bestehen. Beim erneuten Vergleich wurde
für Position 1 der lokale Absatz und für Position 2 der gespeicherte Absatz gewählt.

Ein zweiter kontrollierter Schreibzug änderte die Vergleichsbasis vor dem Speichern. Die erste
Bestätigung führte zur neuen Vorschau auf Revision 2. Sie behielt beide Quellenentscheidungen und
zeigte den inzwischen geänderten zweiten Absatz. SQLite blieb bis zur erneuten ausdrücklichen
Bestätigung auf Revision 2. Nach dem Speichern und einem vollständigen Desktopneustart stimmten die
Szenenkennung, beide Absätze und die Inhaltsprüfsumme auf Revision 3. Separate gezielte Prüfungen
decken unterschiedliche Absatzanzahlen, tatsächlich leere Absätze, ungültige Auswahlen und einen
Formatwechsel bei unveränderter Revisionsnummer ab.

In der gemeinsamen kalibrierten ETW-Spur wurden je Desktopstart genau eine Fensterrolle und ein
Datendienst erfasst. Beide Prozessbäume zeigten null Netzereignisse einschließlich Loopback und
keine WebView2-Prozesse. Vier von vier Positivkontrollen wurden erkannt, ohne verlorene Ereignisse
oder Puffer. Testinstallation, synthetisches Profil und Rohaufzeichnung wurden entfernt.
Installer-SHA-256: `ff4129953733a9d96457fb89d17321551e9f484344a5f48a25e9daed81a8f200`.

## Ungespeicherter Entwurf und Speicherbestätigung im installierten Desktop

Im installierten Kandidaten `db3184d7b200` schlossen unveränderte und vollständig zurückgenommene
Texte direkt. Bei einem geänderten Entwurf öffneten X und Alt+F4 die Schließabfrage.
**Weiterschreiben** hatte Fokus und Vorgabe. Enter kehrte in den Entwurf zurück. **Verwerfen** ließ
Inhalt und Revision 0 der gespeicherten Szene unverändert.

Eine zweite Schreibverbindung blockierte die synthetische SQLite-Datenbank. Der fehlgeschlagene
Speicherversuch erhielt den vollständigen Text im selben Editfenster. Nach dem Lösen der Sperre
bestätigte **Speichern** dort beide Absätze auf Revision 1. Ein weiterer kontrollierter Schreibzug
änderte die Szene während einer neuen Editiersitzung auf Revision 2. Der Konflikt hielt den lokalen
Entwurf im selben Fenster bearbeitbar. Beide Speicheraktionen der veralteten Sitzung waren gesperrt.
Nach ausdrücklichem Verwerfen und einem Desktopneustart stimmten Text und Revision 2 der
zwischenzeitlich gespeicherten Fassung.

Eine gemeinsame kalibrierte ETW-Aufzeichnung erfasste beim ersten Desktopstart fünf Paare aus
Fensterrolle und Datendienst, beim zweiten ein Paar. Beide Prozessbäume zeigten null Netzereignisse
einschließlich Loopback und keine WebView2-Prozesse. Alle vier Positivkontrollen wurden erkannt,
ohne verlorene Ereignisse oder Puffer. Der erste Messversuch war wegen uneindeutiger Dialogzuordnung
im Prüfläufer verworfen worden. Die gültige Wiederholung prüfte zusätzlich Fensterthread, Prozess,
Eigentümerfenster und Fokus des aktiven Dialogs. Testinstallation, synthetisches Profil und
Rohaufzeichnung wurden entfernt. Installer-SHA-256:
`96c9c4aba9a6c51b9e97f971d79c003b448f1bc1eeed2e3f180b696ef7df75b3`.

## Geschütztes Entfernen im installierten Win32-Desktop

Im Kandidaten `30329182c440` öffnete Entf in der fokussierten Liste die schreibgeschützte Vorschau
zur ausgewählten Szene und ihrem Kapitel. **Abbrechen** hatte den Fokus und war die
Standardschaltfläche. Enter ließ alle vier Dokumente bestehen. Die ausdrückliche Bestätigung
entfernte genau die gewählte Szene. Eine weitere Vorschau führte das gewählte Kapitel und dessen
eine verbliebene Kindszene auf. Die Bestätigung entfernte beide. Nach vollständigem Desktopneustart
enthielt die gleichnamige Szene im anderen Kapitel ihre beiden Absätze auf Revision 1. Ihre
Dokumentkennung, Kapitelzuordnung und Inhaltsprüfsumme waren unverändert.

Der SQLite-Test mit sechs betroffenen Zusatzgruppen erfasste die tatsächlich mitgelöschten Daten.
Nach einer Änderung des Bildprompt-Texts bei gleicher Kennung und Anzahl wurde die Bestätigung
abgewiesen. Eine neue Textrevision einer Kindszene sperrte das Entfernen des Kapitels. Eine frische
Vorschau erlaubte anschließend das gezielte Entfernen der Szene.

Die gemeinsame ETW-Aufzeichnung erfasste bei beiden Desktopstarts je eine Fensterrolle und einen
Datendienst. Beide Prozessbäume zeigten null Netzereignisse einschließlich Loopback und keine
WebView2-Prozesse. Alle vier TCP-/UDP-Kontrollen über IPv4/IPv6 wurden erkannt, ohne verlorene
Ereignisse oder Puffer. Testinstallation, synthetisches Profil und Rohaufzeichnung wurden entfernt.
Installer-SHA-256: `4b3a1f35085af3ed0cfa8ee43a6802da6f0822a3210b35ddec775dd2f67b053f`.

## Reihenfolge und Kapitelwechsel im installierten Win32-Desktop

Im installierten Kandidaten `daec7364a865` wurde ein Kapitel über die fokussierte Schaltfläche per
Leertaste nach oben verschoben. Zwei Kapitel enthielten zunächst je eine gleichnamige Szene. Die
bearbeitete Szene wurde an das Ende des anderen Kapitels verschoben, blieb dort ausgewählt und ließ
sich anschließend nach oben und wieder nach unten bewegen.

Nach vollständigem Desktopneustart stimmten Kapitelreihenfolge, Dokumentkennungen, Kapitelzuordnung,
Positionen und beide schreibgeschützten Textansichten. Die unberührte Szene war leer auf Revision 0.
Die verschobene Szene enthielt beide gespeicherten Absätze auf Revision 1. Die Inhaltsprüfsummen
beider Dokumente waren vor und nach den Verschiebungen identisch.

Eine gemeinsame ETW-Spur erfasste im ersten Desktopprozessbaum eine Fensterrolle und einen
Datendienst, beim Wiederöffnen zwei Paare. In beiden Prozessbäumen wurden null Netzereignisse
einschließlich Loopback und keine WebView2-Prozesse beobachtet. Vier von vier Positivkontrollen
wurden erkannt, ohne verlorene Ereignisse oder Puffer. Testinstallation, synthetisches Profil und
Rohaufzeichnung wurden entfernt. Installer-SHA-256:
`ecac6e742eae2c70d50f27c53bb72089ebd57d149b778eee455d821797adecbb`.

## Kapitel und Szenen im installierten Win32-Desktop

Im installierten Kandidaten `1f3806824b7a` wurden ein weiteres Kapitel und eine zweite Szene
angelegt. Beide Kapitel enthielten anschließend je eine „Szene 1“. Beim Umbenennen blieb die Kennung
des Kapitels erhalten. Nach Abbruch einer weiteren Kapitelanlage blieb das gewählte Kapitel
markiert. Der Katalog enthielt weiterhin zwei Kapitel und zwei Szenen.

Nur die Szene im zweiten Kapitel wurde bearbeitet und anschließend umbenannt. Ihre Inhaltsprüfsumme
und Textrevision blieben beim Umbenennen erhalten. Nach vollständigem Desktopneustart zeigte das
erste Lesefenster die leere Szene auf Revision 0, das zweite die exakten zwei gespeicherten Absätze
auf Revision 1. Die Prüfung der lokalen Datenbank bestätigte beide ursprünglichen Szenenkennungen
und Kapitelzuordnungen.

Eine kalibrierte ETW-Aufzeichnung erfasste beim ersten Desktopstart ein Paar aus Fensterrolle und
Datendienst und beim zweiten Start zwei Paare. In beiden Prozessbäumen wurden null Netzereignisse
einschließlich Loopback und keine WebView2-Prozesse beobachtet. Die Messung erfasste alle vier
TCP-/UDP-Kontrollen über IPv4/IPv6, ohne verlorene Ereignisse oder Puffer. Testinstallation,
synthetisches Profil und Rohaufzeichnung wurden entfernt. Installer-SHA-256:
`2b237fca58e02df3af7f43687a86dcfaab5fa38244695442bec89f7fb14778e5`.

## Native Projektanlage und direkte Bearbeitung im Installer

Der installierte Windows-Kandidat `abaca29bec34` startete mit einem leeren synthetischen Profil.
Über **Projekt anlegen** entstand ein Projekt mit **Kapitel 1** und **Szene 1**. **Bearbeiten**
öffnete die leere Szene direkt. Nach dem Speichern zweier Absätze und einem vollständigen
Desktopneustart zeigte **Lesen** den exakten Text schreibgeschützt an. Die separate SQLite-Abfrage
bestätigte beide Absatzknoten auf Revision 1.

Die gemeinsame ETW-Spur erfasste je Desktopstart genau eine Fensterrolle und einen Datendienst. In
beiden Prozessbäumen wurden null Netzereignisse einschließlich Loopback und keine WebView2-Prozesse
beobachtet. Alle vier positiven TCP-/UDP-Kontrollen über IPv4/IPv6 wurden erkannt, ohne Ereignis-
oder Pufferverlust. Die Messung erfasst die ausgeübten nativen Wege. Installer-SHA-256:
`fcf46560be8d7e9684df5d110baa33db9a8960dd696edcb27c4a3900a14acee3`.

## Installiertes Schreiben mit Wiederöffnen · 25. November 2026

Im installierten Kandidaten `e24c9c959069` wurde eine synthetische Prosaszene gelesen, ausdrücklich
zum Bearbeiten geöffnet und mit zwei Absätzen gespeichert. Nach dem Beenden und einem neuen
Desktopstart zeigte das schreibgeschützte Fenster beide Absätze exakt wieder an. SQLite bestätigte
beide Absatzknoten auf Revision 1.

Eine gemeinsame lokale ETW-Aufzeichnung wurde für beide Desktop-Prozessbäume getrennt ausgewertet.

| Ausgeübter Weg                    | Desktop | Fensterrolle | Datendienst | Netzereignisse einschließlich Loopback |
| --------------------------------- | ------: | -----------: | ----------: | -------------------------------------: |
| Lesen und ausdrücklich bearbeiten |       1 |            2 |           2 |                                      0 |
| Nach Neustart wieder lesen        |       1 |            1 |           1 |                                      0 |

Vier von vier IPv4-/IPv6-Kontrollen über TCP und UDP wurden erfasst. Es gab keine verlorenen
Ereignisse oder Puffer, keine unzuordenbaren Netzereignisse und keine WebView2-Prozesse in den
beiden Bäumen. Testinstallation, synthetisches Projekt und Rohaufzeichnung wurden entfernt. Der
Nachweis umfasst die oben genannten Bedienwege dieses Binärstands.

## Installierter nativer Szenenlesepfad · 25. November 2026

Im frisch installierten Kandidaten `e24c9c959069` wurde ein synthetisches Projekt über Projekt,
Kapitel und Szene geöffnet. Der sichtbare Text und der Leseschutz des Win32-Steuerelements wurden
geprüft. Anschließend wurden Lesefenster und Desktop regulär geschlossen. Die installierte
Programmdatei stimmte mit dem Installerinhalt überein. Installer-SHA-256:
`096ed45d9fe870d928ca9bffc53eda9de8e87383de0fa767c23a5182e3e817af`.

Die ETW-Auswertung im Modus `native-scene` erfasste genau einen Desktop, eine Fensterrolle und einen
Datendienst. Für diesen Prozessbaum wurden keine Transportereignisse erfasst, auch keine über
Loopback. Alle vier positiven Kontrollen wurden erkannt. Verlorene Ereignisse, verlorene Puffer und
nicht zuordenbare Netzereignisse: jeweils null. Dieser Nachweis gilt für den ausgeübten Leseweg. Der
separate Schreib- und Wiederöffnungslauf ist in der
[Arbeitsübergabe](../WORKSTATION_HANDOFF.md#ein-manuskript-nativ-lesen-und-bearbeiten) dokumentiert.
Das WebView2-Studio und externe Systemdienste liegen außerhalb dieses Messumfangs.

## Native Windows-Projektübersicht · 25. November 2026

Der installierte Kandidat `66b0ff0e103f` zeigte beim regulären Start die native Projektübersicht.
Fenster und Verwaltungsschaltfläche wurden anhand von Prozess-ID, Titel und Steuerelement
identifiziert. Je ein Start mit frischem leerem Profil, 30 Sekunden sichtbares Fenster und reguläres
Schließen. Der separate Bedienlauf im Debug-Build bestätigte zusätzlich Projektanlage, Neustart,
Auswahl des vorhandenen Projekts per Enter und Rückkehr aus dem Editor in die Verwaltung.

| Installierter Stand                    | Desktop / WebView2-Prozesse |        Netzereignisse außerhalb von Loopback | Loopback |
| -------------------------------------- | --------------------------: | -------------------------------------------: | -------: |
| Direkter Vorgänger `aca30d0ae0d9`      |                       1 / 6 | 52 (48 öffentlich, 4 privat oder link-lokal) |        6 |
| Native Projektübersicht `66b0ff0e103f` |                       1 / 0 |                                            0 |        0 |

Beide EXE-Dateien wurden mit ihrem Installerinhalt abgeglichen. SHA-256 des neuen Installers:
`fd85f92fa93bfb5597cc1caab7723bb1ecb9536732db3ed4bca16beebbdfc22b`. Die gemeinsame ETW-Aufzeichnung
wurde je Prozessbaum getrennt ausgewertet (Modi `desktop` und `native-home`). Vier von vier lokalen
IPv4-/IPv6-TCP-/UDP-Kontrollen, null verlorene Ereignisse oder Puffer und vollständige Prozessenden.
Der erste Versuch wurde wegen uneindeutiger Fensterzuordnung vor Abschluss verworfen.
Testinstallationen, Profile und Rohdaten der Messung wurden entfernt.

**Abgenommen ist der Start bis zum Schließen der nativen Projektübersicht mit leerem Katalog.**
Projektöffnung und Verwaltung starten weiterhin WebView2. Für diese Wege, externe Systemdienste und
den gesamten Editor lässt sich aus dem Lauf keine Netzwerkfreiheit ableiten. Q-01/Q-02 bleiben für
das Gesamtprodukt offen.

## Regulärer Windows-Start am 25. November 2026

**Q-01/Q-02 für den Desktop weiterhin nicht bestanden.** Beim Start ohne geöffnetes Manuskript
wurden in beiden Prozessbäumen WebView2-Transportereignisse zu öffentlichen Zielen erfasst.

| Installierter Stand              | Öffentliche Ziele | Privat oder link-lokal | Loopback | Außerhalb Loopback gesamt |
| -------------------------------- | ----------------: | ---------------------: | -------: | ------------------------: |
| Alpha 7, `a9a3a673ff14`          |                83 |                      4 |        6 |                        87 |
| Drehbuchkandidat, `aca30d0ae0d9` |                97 |                      4 |        6 |                       101 |

Je ein regulärer Start, eigenes leeres Projekt- und WebView-Profil, 30 Sekunden bei geöffnetem
Fenster, danach reguläres Schließen. Beide EXE-Dateien stimmten mit dem Inhalt ihres geprüften
Installers überein. SHA-256 des Kandidaten:
`30173318109346a9748bd658e2493f39cf50688d7f16fa86669cd843e94a9860`. Gemeinsame ETW-Erfassung mit
getrennter Auswertung je Start-PID über `scripts/m3-network/analyze.py`. Je ein Desktopprozess und
sechs WebView2-Prozesse mit vollständigem Ende. Vier von vier lokalen IPv4-/IPv6-TCP-/UDP-Kontrollen
erkannt, keine Ereignis- oder Pufferverluste, keine ungeklärten Zuordnungen. Alle gezählten
Netzereignisse im jeweiligen Baum stammen von WebView2.

Die Zähler beschreiben Transportereignisse. Übertragene Inhalte und Vorgänge außerhalb des
Prozessbaums lassen sich daraus nicht bestimmen. Aus einem Start pro Stand lässt sich keine Ursache
für den Mengenunterschied ableiten. Beim ersten Versuch stoppte die Binärprüfung vor dem zweiten
Programmstart. Für die Tabelle wurde ausschließlich die vollständige Wiederholung verwendet.
Testinstallationen, Profile und Rohaufzeichnung wurden nach der Auswertung entfernt.

**Historische Drehbuchprüfung am 25. November 2026.** Spätere Abnahmen einschließlich
DOCX-v2-Rückimport stehen im [aktuellen Kriterienstand](#stand-der-drehbuchkriterien).

| Bereich  | Nachweis und verbleibende Grenze                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| -------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| DB-01–03 | Edge-Bedienlauf mit simuliertem IPC für sechs Rollen, Speichern, Wiederöffnen und Undo. Separater nativer Test mit SQLite, revisionsgebundenem Speichern und PDF-Ausgabe.                                                                                                                                                                                                                                                                                                                                                                                                                                |
| DB-04–06 | Zwei Seiten aus dem echten Renderer vollständig mit Poppler gerastert und visuell geprüft. Format 595,276 × 841,89 pt. Auf Seite 2 folgen Figur, Klammeranweisung und die ersten beiden Dialogzeilen unmittelbar aufeinander. Der Übergang steht am rechten Satzrand. Die Stichprobe deckt keine vollständige Vermessung sämtlicher Grenzfälle ab. Alle vier Schriften sind lokal gebündelt.                                                                                                                                                                                                             |
| DB-07    | Markdown enthält maskierte Textinhalte mit Rollenbezeichnung, HTML lokale Druckstile und DOCX sechs eigene Absatzvorlagen. Word 16.0.17932 und LibreOffice 26.8.3 öffneten und speicherten dieselbe synthetische DOCX mit 13 Absätzen. Nach dem Wiederöffnen blieben sämtliche Absatztexte, Reihenfolge, Dialogumbruch und alle sechs angewandten Rollen erhalten. Beide Programme renderten eine vollständige Seite mit den vorgesehenen Einzügen und rechtsbündigem Übergang. Schrift und Abstände unterscheiden sich je nach lokal verfügbarem Font. Ein kanonischer DOCX-v2-Rückimport bleibt offen. |
| DB-08    | Im installierten Windows-Kandidaten `aca30d0ae0d9` wurde eine synthetische Szene mit Szenenkopf und Handlung per Tastatur geschrieben. Nach regulärem Prozessende enthielt SQLite den exakten Text mit beiden Rollen in einem v2-Dokument (Schema 7). Nach neuem Programmstart öffnete sich die Szene im Drehbuchmodus. Die ausdrückliche Auswahl ergab eine einseitige A4-PDF mit 131.732 Byte. Fehler und Abbruch waren nicht Teil dieses Bedienlaufs.                                                                                                                                                 |

Kombiniertes ETW, 16. November: Zwei frische gültige Messphasen exakt des gesicherten v3-Paars,
einschließlich manueller Dateiauswahl, Wiederanlauf und neuem Folgeauftrag. 80 Apps/480 WebViews,
alle 868 Prozessenden explizit; acht Kontrollen, Nullverluste, 0 Nicht-Loopback- und 480 getrennte
Loopback-Ereignisse. Native Daten/MACs unabhängig bestätigt.
[Nachweis und Scope](native-combined-live-restart-etw.md). Kein neuer Hör-/vollständiger
Fensterende- oder Produktbeleg; Q-01/Q-02 bleiben für das Produkt nicht bestanden.

Kombiniert v3, 16. November: Neun native Forschungsfälle mit Live-Kanal, Wiederanlauf und neuem
signiertem Folgeauftrag bestanden: 72 App-Starts, 18 Controllerenden, 54 abgewiesene Folgeaktionen.
Zusätzlicher tatsächlicher manueller Dateidialog nach Shell-Pfadkorrektur bestätigt; MACs,
Root-/Programmhashbindung, Journal/SQLite/Unicode/Kopien unabhängig geprüft.
[Nachweis](native-combined-live-restart.md). Eigener ETW-Beleg oben ergänzt; neuer
Hör-/vollständiger Fensterendebeleg fehlt, Q-01/Q-02 für das Produkt weiterhin nicht bestanden.

HMAC 16. November: DPAPI im Forschungsbroker durch lokale HMAC-Authority ersetzt. Neue native Serie
mit sechs Controllerenden, zwölf Wiederanlaufprozessen, 30 App-Generationen und 78 Ablehnungen
bestanden; Daten/MACs unabhängig geprüft. [Nachweis](native-hmac-broker.md). Eigener gültiger
[ETW-Beleg](native-hmac-broker-etw.md): 103 Broker/30 Apps/180 WebViews, 446 explizite Enden, vier
Kontrollen, Nullverluste, 0 Nicht-Loopback- und 180 getrennte Loopback-Ereignisse. Keine
Produkt-/Hörabnahme, Q-01/Q-02 bleiben nicht bestanden.

Controller-Crash 16. November: Getrennter Forschungsbroker mit dauerhafter nativer Autorität,
exklusiver Sperre und Jobzuweisung bei Prozesserzeugung geprüft. Sechs harte Controllerenden, zwölf
neue Wiederanlaufprozesse, 30 App-Generationen, 60 Ablehnungen; Journal/Kopien/SQLite/Unicode
unabhängig bestätigt. [Nachweis und Grenzen](native-controller-crash.md). Kein produktiver Broker,
kein eigener neuer ETW-/Hörbeleg; Produkt-Q-01/Q-02 weiterhin nicht bestanden.

ETW v2 am 16. November: Eigener gültiger Netzbeleg des gesicherten v2-Paars mit automatischer Serie
und beiden manuellen Pfaden. 32 Apps/168 WebViews vollständig erfasst, vier Kontrollen,
Nullverluste, 0 Nicht-Loopback- und 176 getrennte Loopback-Ereignisse; native Daten/Cleanup geprüft.
[Nachweis](native-assistive-restart-etw.md), [Aggregate](native-assistive-restart-etw.json). Keine
externe Broker-, Inhalts- oder Produktabnahme; Q-01/Q-02 unverändert nicht bestanden.

Assistiver Wiederanlauf 16. November: Unterbrechungs-/Abschluss-/Replay-Ansagen mit
Windows-Sprachausgabe bestätigt; native Aufrufe, Daten und Cleanup gegengeprüft. Tastaturnutzung
menschlich bestätigt, aus dem Log allein nicht beweisbar. Erster Versuch ohne Replay nicht als
bestanden gewertet. [Nachweis](native-assistive-restart.md). Neues v2-Paar ohne eigenen ETW-Beleg;
keine assistive Voll- oder Produktabnahme, Q-01/Q-02 bleiben nicht bestanden.

ETW-Fortschreibung 16. November: Das gesicherte Neustart-Paar ist mit neuer gültiger Messung belegt:
28 vollständige App-Generationen, 144 WebViews, vier Kontrollen, Nullverluste, 0 Ereignisse
außerhalb Loopback und 144 getrennte Loopback-Ereignisse. Native Crash-Serie und Daten/Cleanup im
Messlauf unabhängig bestätigt. [Nachweis](native-ui-restart-etw.md),
[Aggregate](native-ui-restart-etw.json). Keine externe Broker-, assistive Voll- oder Produktabnahme;
Q-01/Q-02 unverändert nicht bestanden.

Neustart-Fortschreibung 16. November: Integrierter isolierter GUI-Neustart innerhalb desselben
Controllers nativ bestanden. 28 Generationen mit sechs Crashpunkten, zweifachem Wiederanlauf,
Altimportabwehr und vier negativen Starts; unabhängige Journal-/SQLite-/Unicode-/Cleanup-Prüfung
bestanden. [Nachweis](native-ui-restart.md), [Aggregate](native-ui-restart.json). Kein neuer
ETW-Beleg, Controller-Crash-, assistiver Voll- oder Produktnachweis. Q-01/Q-02 unverändert offen.

Auftrags-Fortschreibung 16. November: Dauerhafter Verbrauch und konservative Reconciliation im
Forschungsadapter implementiert. Sieben echte Windows-Testprozess-Kills, erneutes Öffnen,
Replay-Sperre und bytegleicher Bestand geprüft. Native Erfolg-/Abbruchrückprüfung bestätigt Journal,
Kopienzahl und Cleanup. [Nachweis und Buildgrenzen](native-task-reconciliation.md). Integrierter
isolierter GUI-Neustart und neuer ETW-Beleg fehlen; keine Produkt- oder assistive Vollabnahme.
Q-01/Q-02 für das Produkt unverändert nicht bestanden.

Neueste Fortschreibung 16. November: Live-Auswahl mit Replay sowie Fensterende bei offenem Dialog
und nach echtem Teiltransfer funktional belegt. Frischer ETW-Vergleich am unveränderten finalen
App-/Controllerpaar: drei vollständige Bäume, 19 WebView2-Lebenszeiten, vier Kontrollen,
Nullverluste, 0 Ereignisse außerhalb Loopback. SQLite/Unicode, Kopienzahl, Teil-Empfang und Cleanup
unabhängig bestätigt. [Nachweis](native-live-window-close.md),
[Aggregate](native-live-window-close-etw.json). Keine externe Brokerattestierung, assistive
Vollabnahme oder Produktintegration. Q-01/Q-02 für das Produkt weiter **nicht bestanden**, #134
offen.

Fortschreibung [UI-/Tauri-Testanbindung](native-tauri-file-channel.md): separates echtes
Forschungsfenster, einmaliger vorab delegierter Einweg-Dateikanal, keine Rendererpfade/Handles,
Origin-/Fensterprüfung. Finaler Build nativ dreimal mit Import/Wiederholung, Abbruch und Fensterende
am Haltepunkt bestanden; SQLite/Unicode und Cleanup unabhängig bestätigt. Gültiges ETW desselben
Builds: drei vollständige Bäume, vier Kontrollen, Nullverluste, 0 Ereignisse außerhalb Loopback. Nur
synthetischer Fixturemodus; native Vorabauswahl, Assistenz und Produkt-Vollabnahme offen. Keine
universelle Netzstille oder externe Brokerattestierung. Q-01/Q-02 **nicht bestanden**, #134 offen.

Fortschreibung [typisierter Dateiauftrag und atomarer Import](native-isolated-app-import.md):
Separater nativer App-Dienst-Testadapter ohne Tauri-/WebView-/GUI-Anbindung; Snapshot-v3-Import im
AppContainer mit einmaliger ID, Ablauf/Budget und Publish-Guard. Sieben native Importfälle plus
Kopieregression und unabhängige SQLite-/Hash-/Unicodeprüfung bestanden. Gültiges ETW des finalen
Builds ohne Netzereignisse im Controller-/Workerbaum. Keine reale Nutzdatei, native Auswahl,
assistive oder Produkt-Vollabnahme; externe Broker und Crash-Reconciliation offen. Q-01/Q-02
unverändert **nicht bestanden**, #134 offen.

Fortschreibung Datei-/Shell-Austausch:
[separater Transportprototyp](native-runtime-file-exchange.md) mit nativer Auswahl, zwei
Handlefreigaben, Unicode-Kopie, gesperrter Nachbardatei, Abbruch und Fehlerpfadtests geprüft.
Begrenzte gültige ETW-Messung des Controller-/Workerbaums ohne Netzereignisse; finale
Cleanup-/Berichtsverstärkung separat nativ geprüft. Keine Wiederholung von
M3-Import/Export/Screenreader unter Produktisolation, kein Nachweis externer COM-/Shell-Dienste.
Dateiauftrag/App-Testadapter und konkrete Formatimporte bleiben offen. Q-01/Q-02 unverändert **nicht
bestanden**, #134 offen.

KQ-069 A untersucht die ausdrücklich beauftragte strikte Isolation: Der
[AppContainer-Prototyp](native-runtime-isolation.md) zeigt bei gültiger ETW-Messung drei
vollständige App-Läufe mit 0 Ereignissen außerhalb Loopback; derselbe unisolierte Build zeigt 201
öffentliche WebView2-Ereignisse. Projekt/Autosave/Neustart/HTML/PDF im synthetischen Korpus geprüft,
Dateidialoge/Austausch, weitere Broker-/UDP-/IPv6-Pfade und vollständige assistive/funktionale
Wiederholung offen. Kein produktiver Launcher eingebunden: **Q-01/Q-02 bleiben nicht bestanden**.

**Fortschreibung KQ-068 / Issue #133:** PR #132 ist grün zusammengeführt (`c7629c3`).
[Restprüfungsprotokoll](native-m3-coverage.md): M3-05 und M3-10 sind im dokumentierten CRUD-Korpus
jetzt **N**. Der Nutzer bestätigt den manuellen kanonischen Bildwechsel und die
Profil-/Stil-/Fehleransagen. Die drei ausdrücklich freigegebenen GUI-Löschungen,
Datei-/Referenzprüfung und vollständiger Prozessneustart sind bestanden. Bild A, Tags, Szenenlink
und Figur bleiben unverändert. Vier anschließende bebilderte Exporte sind per Tastatur erzeugt und
unabhängig rückgelesen. Der Nutzer bestätigt auch Löschmeldungen, Export-Bedienelemente und Erfolge
mit „Ja passt, alles gut“. Die gezeigte Hörprobe ist bestanden; keine pauschale assistive F08- oder
WCAG-Freigabe.

KQ-069 ergänzt die [NetLog-Diagnose](native-m3-webview-network.md): Konfigurations- und
Komponentenupdate-Starts schon ohne Projekte; keine Inhalte-/Netzfreigabe. Unterstützte Maßnahmen
sind bewertet, keine Unterdrückung implementiert. Der erneute ETW-Lauf ist mit allen Prozessenden,
Dokument-CRUD, Medienimport/-Metadaten, Export/Neuimport und drei App-Starts gültig, bestätigt aber
256 öffentliche WebView2-Transportereignisse. ADR 0034 bleibt Entscheidungsvorlage; #134 offen.

**Q-01/Q-02: gemessen, nicht bestanden.** Windows-ETW findet öffentlichen Verkehr von
WebView2-Nachfahren ohne Provideraufruf. Der bisher fehlende Gesamtnachweis ist durch diesen
konkreten Befund ersetzt; [KQ-069 / #134](https://github.com/kolkrabeofdoom/kraqel/issues/134) hält
ihn offen. Keine Zweck-/Uploadbehauptung aus Transportmetadaten, keine physische Netztrennung und
kein vollständig wiederholter KQ-067-Lauf in dieser Messung. Die folgenden historischen Belegzeilen
überschreiben diesen neueren Befund nicht. F08/M3 bleiben teilweise abgenommen.

Stand: 15. November 2026 · KQ-066 / Issue #129, fortgeschrieben durch KQ-067 / Issue #131.
Ursprüngliche Codebasis PR #128 (`4679c89`); der funktionale native Gesamtlauf basiert auf dem mit
grüner CI zusammengeführten PR #130 (`58dea4b`) plus den KQ-067-Testhelfern und der korrigierten
Medienstatusmeldung. Issues #127/#129 sind geschlossen; Issue #94 bleibt separat als
Abhängigkeitsaufgabe offen. [KQ-067-Abnahmeprotokoll](native-m3-acceptance.md) dokumentiert die
neuen Laufzeit-/GUI-Prüfungen und ihre Grenzen.

**Gesamturteil: F08 und M3 bleiben teilweise erfüllt, nicht vollständig abgenommen.** Die Matrix
erfasst vollständig die einschlägigen Anforderungen und Restprüfungen. Sie erklärt nicht alle
aufgeführten Kriterien für bestanden und ersetzt keine allgemeine F01–F09-/0.1-Abnahme.

## Quellen und Bewertungsregeln

Primärquelle ist das
[Pflichtenheft v0.1 Revision 1](../specifications/KRAQEL_Technisches_Pflichtenheft_v0.1_Revision_1.pdf):
§1 Produktziele; §3 F07/F08/F09; §4 Referenzintegrität; §6 V.1; §7 Snapshotregeln; §8 M3; §10
Offline-, Speicherintegritäts-, Referenz-, Datenschutz-, Git- und Exportkriterien; §11 offene
Formatentscheidungen. Bestehende ADRs und Datenverträge konkretisieren den Umfang; die Drehbuch-ADR
0033 ist nach ausdrücklicher Vertragsfreigabe am 15. November 2026 fachlich angenommen, aber nicht
implementiert. Die IDs unten sind lokale Prüfkennungen, keine neuen
Pflichtenheft-Anforderungsnummern.

| Status | Aussage                                                                                                          |
| ------ | ---------------------------------------------------------------------------------------------------------------- |
| **A**  | Automatisiert im dokumentierten Umfang belegt; keine native Vollabnahme daraus ableiten                          |
| **N**  | Zusätzlich nativ/manuell im verlinkten synthetischen Korpus und den dort genannten Builds geprüft                |
| **T**  | Teilnachweise vorhanden; explizite Restprüfung oder Entscheidung fehlt                                           |
| **O**  | Offen, kein ausreichender Nachweis bzw. keine Implementierung                                                    |
| **V**  | Konkreter Vorschlag zur fachlichen Entscheidung; keine Produktfunktion                                           |
| **X**  | Bewusst außerhalb dieses Umfangs; nur mit verlinkter Begründung, kein stiller Verzicht auf eine Muss-Anforderung |

Native Nachweise sind historische, datierte Prüfungen. Grüne CI bestätigt ihre Fixtures und
Regressionsprüfungen, wiederholt aber weder Office-GUI-Prüfungen noch Windows-Dialog- oder
Screenreader-Abnahmen. Die zuletzt geprüfte Implementierungsstufe KQ-065 hat 161 Rust- und 53
TypeScript-Tests, sechs regulär ignorierte explizite QA-Helfer und 26 Browser-E2E. Diese Zahlen und
die folgenden Belege stammen aus PR #128; sie sind kein neuer Prüflauf von KQ-066.

KQ-067 besteht zusätzlich `pnpm check` mit 162 Rust- und 53 TypeScript-Tests (sieben explizite
QA-Helfer regulär ignoriert), alle 26 Browser-E2E sowie die nativen Quell-/Recovery-Builds und den
abschließenden regulären Windows-Debug-Build. Die neue manuelle Evidenz steht im
[Abnahmeprotokoll](native-m3-acceptance.md).

## F08: unterstützte Formate und Austauschgrenzen

| ID / Quelle                     | Kriterium und aktueller Umfang                                                                                                      | Status / konkreter Beleg                                                                                                                                                | Restprüfung bzw. Grenze                                                                                                                                                                                                                                              |
| ------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| F08-01 · §3 F08, §10 Export     | Gemeinsame Semantik: Baumreihenfolge, Titel, Absätze, Heading 1–6, Fett/Kursiv, Umbrüche, Listen und Zitate                         | **A** · [Exportvertrag](../data-format/EXPORT.md), [Modell](../../crates/export/src/manuscript.rs), [Tests](../../crates/export/src/lib.rs)                             | Editor kann mehr Knoten erzeugen als der Export unterstützt; unbekannte Formatierung wird abgewiesen, nicht als unterstützt gezählt                                                                                                                                  |
| F08-02 · §3 F08                 | Markdown und eigenständiges HTML aus validiertem Modell                                                                             | **A** · [Renderer](../../crates/export/src/render.rs), [Export-E2E](../../apps/desktop/e2e/exports.spec.ts)                                                             | Kein beliebiger HTML-/Markdown-Roundtrip; keine identischen Seitenumbrüche versprochen                                                                                                                                                                               |
| F08-03 · §3 F08                 | Gewöhnliches DOCX, getrennt vom Austauschprofil                                                                                     | **N** für den Bild-/Textkorpus · [DOCX-Abnahme](native-illustrated-docx-acceptance.md), [Renderer](../../crates/export/src/render.rs)                                   | Keine universelle Office-Kompatibilität oder freie Struktur-Rückimportgarantie                                                                                                                                                                                       |
| F08-04 · §3 F08, §10 Export     | A4-PDF mit eingebetteten vier Schriften, NFC/Glyphenprüfung und metrischem Umbruch                                                  | **N** · [PDF-Abnahme](native-illustrated-pdf-acceptance.md), [PDF-Code](../../crates/export/src/pdf.rs)                                                                 | Begrenztes Glyphenprofil; keine PDF/UA-, PDF/A/X- oder universelle Shaping-Abnahme                                                                                                                                                                                   |
| F08-05 · §3 F08                 | TXT-/Markdown-Datei wird atomar neue Szene, begrenztes UTF-8-Profil                                                                 | **N** · [Textimport-Abnahme](native-text-import-acceptance.md), [Formatbeschreibung](../data-format/README.md#utf-8-markdown-import)                                    | Inline-Markdown bleibt teilweise Klartext; Codeblöcke können importiert werden, sind im gewöhnlichen Export nicht unterstützt                                                                                                                                        |
| F08-06 · §3 F08                 | Fremdes DOCX wird begrenzt als neue Szene importiert                                                                                | **N** · [DOCX-Importabnahme](native-docx-import-acceptance.md), [Vertrag](../data-format/DOCX_IMPORT.md)                                                                | Keine automatische Kapitel-/Szenenerkennung; Bilder, Listen und weitere fremde Inhalte werden abgewiesen                                                                                                                                                             |
| F08-07 · §11 Roundtrip          | Eigenes DOCX-v1-Profil, Textbearbeitung in bestehenden Absatzmarkern, atomarer Neuimport als neue Projektkopie                      | **N** · [Roundtrip-Abnahme](native-docx-roundtrip-acceptance.md), [Vertrag](../data-format/DOCX_ROUNDTRIP.md), [Rust-Tests](../../crates/export/src/roundtrip/tests.rs) | Keine Bilder/Story-Bibel/Links im Manuskriptaustausch, kein freier Office-Strukturumbau; Originalprojekt wird nicht überschrieben                                                                                                                                    |
| F08-08 · §10 Export, §11 Medien | Explizite direkt verknüpfte Szenenbilder in Markdown/HTML                                                                           | **N** · [Abnahme](native-illustrated-export-acceptance.md), [Bildvertrag](../data-format/ILLUSTRATED_MANUSCRIPT.md)                                                     | Szeneende, keine automatische Auswahl aus Profilen/Entitätslinks; vollständiges Paket weitergeben                                                                                                                                                                    |
| F08-09 · §10 Export, §11 Medien | Gewöhnliches DOCX mit internen PNG-Parts und Alttext; Bild-PDF mit weißen PNG-Nachweisen                                            | **N** · [DOCX](native-illustrated-docx-acceptance.md), [PDF](native-illustrated-pdf-acceptance.md)                                                                      | Kein Bild-Roundtrip; PDF-Bildbeschreibung sichtbar, keine Zusage getaggter Alternativtexte                                                                                                                                                                           |
| F08-10 · §10 Export             | Unbekannte Inhalte, beschädigte Pakete, Limits und Glyphenfehler verhindern Teilveröffentlichung; frühere Ausgaben bleiben bestehen | **A/N** · [Roundtrip-Tests](../../crates/export/src/roundtrip/tests.rs), [Bildtests](../../crates/export/src/illustrated/tests.rs), native Abnahmen oben                | Dateisystem-Austauschrennen bleibt laut Threat Model begrenzt; keine harte RSS-Garantie                                                                                                                                                                              |
| F08-11 · §1, §11 Drehbuch       | Absatzrollen, Darstellung und Export                                                                                                | **T**                                                                                                                                                                   | Modell v2 und A4-Szenenauswahl im Entwicklungszweig umgesetzt. Die vollständige DB-01–DB-08-Abnahme bleibt offen.                                                                                                                                                    |
| F08-12 · §3 F08, §11            | Richtungen und Grenzen der Formatliste ausdrücklich festlegen                                                                       | **T** · ADR 0008 legt vier **Ausgaben** fest; Importverträge definieren nur TXT/Markdown, begrenztes DOCX, eigenes DOCX-Profil und Snapshot v3                          | Allgemeiner HTML-/PDF-Import und FDX/Fountain sind nicht implementiert. Die bloße F08-Formatliste wird nicht als Garantie beider Richtungen gelesen; diese Auslegung und der Drehbuchumfang sind durch ausdrückliche Vertragsfreigabe am 15. November 2026 bestätigt |

## M3: Git, Wiederherstellung, Medien und visuelle Profile

| ID / Quelle                               | Kriterium und aktueller Umfang                                                                               | Status / konkreter Beleg                                                                                                                                                                                                                                                       | Restprüfung bzw. Grenze                                                                                                                              |
| ----------------------------------------- | ------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------- |
| M3-01 · §3 F07, §7, §10 Git               | Deterministische Snapshots, Hash-Assets, keine laufende SQLite/Schlüssel/temporären Daten in Git             | **A** · `snapshot_is_byte_stable_and_git_never_tracks_database`, `snapshot_copies_assets_by_hash_and_rejects_tampering` in [Git-Service](../../crates/git-service/src/lib.rs); [Snapshotvertrag](../data-format/README.md)                                                     | PDF-Containerstabilität ist davon unabhängig; Snapshotstabilität bleibt geprüft                                                                      |
| M3-02 · §3 F07                            | Lokales Init, Status, Diff, expliziter Commit                                                                | **N** im dokumentierten Korpus · Zwei echte native Snapshot-/Commitstände, Status und lesbarer Diff; Git-Verlauf unabhängig geprüft. Siehe [M3-Abnahme](native-m3-acceptance.md).                                                                                              | Keine assistive Git-Vollabnahme oder zusätzliche Git-Checkout-/Merge-Funktion.                                                                       |
| M3-03 · §3 F07, §8 M3                     | Lesende Wiederherstellung aus Export-Snapshot: v3-Archiv als neue unabhängige Projektkopie                   | **N** im dokumentierten Korpus · Nativer Archivexport, zwei aktuelle Neuimporte und zusätzlicher Import des früher gesicherten Stands; vollständiger Inventarvergleich und unabhängige Projektidentitäten. [Nachweis](native-m3-acceptance.md).                                | Keine Überschreibung/kein Git-Checkout; Wiederherstellung aus ausdrücklich gesichertem Snapshot, keine automatische Auswahl historischer Git-Stände. |
| M3-04 · §8 M3, §11 Paket                  | Deterministisches `.kraqel`-Archiv, eigener nativer Datei-/Speicherdialog, keine absolute Pfad-/Byte-IPC     | **N** im dokumentierten Dialogumfang · [ADR 0019](../architecture/decisions/0019-atomic-snapshot-import-and-bundled-example.md), [ADR 0021](../architecture/decisions/0021-native-snapshot-archive-export.md), [Status](../DEVELOPMENT_STATUS.md)                              | Archiv ist Austauschpaket, kein fortschreibbares Backup; v1/v2 können geprüft, aber nicht als neues Projekt importiert werden                        |
| M3-05 · §3 F09, §6 V.1                    | PNG/JPEG/WebP importieren, anzeigen, Tags/Metadaten, ersetzen/löschen, mehrere eigenständige Varianten       | **N** im dokumentierten CRUD-Korpus · [KQ-068](native-m3-coverage.md): native PNG-Anlage, Metadaten/Tags, WebP-Ersatz, separate JPEG-Variante, bestätigte GUI-Löschung, Datei-/Referenzprüfung und Neustart bestanden.                                                         | Mehrere eigenständige Varianten; kein Elternmodell oder Variantenvergleich. Allgemeine assistive Abnahme bleibt getrennt.                            |
| M3-06 · §4, §6 V.1, §8 M3, §10 Referenzen | Ein Asset an mehrere Szenen und Story-Bibel-/Ereignisziele; stabile IDs und intakte Backlinks                | **N** im dokumentierten Korpus · Nativ umbenannte Figur/Szene; Charakterbild an zwei Szenen, Figur und Ereignis. Zwei Story-Bibel-Rückverweise und sämtliche 102 Assetlinks nach Neustart, Recovery und Neuimporten verglichen. [Nachweis](native-m3-acceptance.md).           | Allgemeine Medien-CRUD-/assistive Vollabnahme bleibt M3-05 bzw. Querschnitt.                                                                         |
| M3-07 · §10 Speicherintegrität            | 100 Szenen und 50 verlinkte Bilder nach Neustart und Sicherung konsistent                                    | **N** funktionaler Gesamtlauf · [M3-G01 bestanden](native-m3-acceptance.md): 100 Szenen/50 dekodierbare Originale, Neustart, tatsächliche Backup-/Recovery-Implementierung und native Recovery-App; unabhängige SQLite-/FK-/Hash-Prüfung.                                      | Unabhängiger Desktop-Netzverkehrsnachweis und assistive Vollabnahme bleiben offen; Export-Großauswahl wird über Rust aufgebaut.                      |
| M3-08 · §10 Speicherintegrität            | Fehlende und hashveränderte Originale/fehlende Thumbnails sichtbar melden                                    | **N** im dokumentierten Korpus · Drei getrennte Dateimanipulationen, native Medienansicht und korrigierte Meldung „Bilddatei oder Vorschau nicht verfügbar“; Rust erkennt alle drei. [Nachweis](native-m3-acceptance.md).                                                      | Die Meldung unterscheidet die drei Ursachen nicht einzeln; keine Behauptung einer solchen Diagnose.                                                  |
| M3-09 · §8 M3, §10 Speicherintegrität     | Backup/Recovery für Daten und Hash-Assets, ohne Quelle zu verändern                                          | **N** im dokumentierten Korpus · Tatsächliche SQLite-/Asset-Sicherung und Recovery via testexklusivem Rust-Helfer, Recovery-App separat geöffnet; Inventar, 50 echte Originale/Thumbnails, Quelle und bytegleiche Sicherung geprüft. [Nachweis](native-m3-acceptance.md).      | Kein vorhandener Backup-UI-Button behauptet; breitere Plattform-/Recovery-Varianten gesondert prüfen.                                                |
| M3-10 · §3 F09, §6 V.1, §8 M3             | Visuelle Profile mit Kernmerkmalen/Ausschlüssen/kanonischem Asset, projektweite Stilpresets                  | **N** im dokumentierten CRUD-Korpus · [KQ-068](native-m3-coverage.md): Profil-/Stil-Anlage und Bearbeitung, manuell bestätigter kanonischer Bildwechsel, GUI-Löschung und Neustart; Bildverweis wird beim Assetlöschen korrekt geleert.                                        | KQ-067 ergänzt Recovery; KQ-068 belegt Erhaltung von Figur und anderem Asset. Keine allgemeine assistive Gesamtabnahme.                              |
| M3-11 · §6 V.1, Übergang M4               | Editierbarer Bildprompt aus expliziten Quellen, Speichern und UTF-8-Export; Offline-Capabilities/Jobverträge | **A** · [Promptrepository](../../crates/storage/src/prompt_repository.rs), [Prompt-E2E](../../apps/desktop/e2e/prompt-builder.spec.ts), [ADR 0015](../architecture/decisions/0015-model-specific-image-capabilities-and-job-contracts.md)                                      | Gehört überwiegend M4; Export-/Backupreferenzen sind M3-relevant. Keine aktive Bildgenerierung oder native Vollabnahme daraus ableiten               |
| M3-12 · §4, §7, §8 M3                     | Snapshot-Neuimport erhält referenzielle Integrität, Profile und Promptquellen                                | **N** im dokumentierten Korpus · Zwei aktuelle native Neuimporte plus historischer Neuimport erhalten alle fachlichen IDs/Quellen/Referenzen bei neuen Projekt-UUIDs; vollständiger Snapshotinventarvergleich und unabhängige FK-Prüfung. [Nachweis](native-m3-acceptance.md). | Kein beliebiger Fremdarchiv-/Versionsimport und keine v1/v2-Projektneuanlage.                                                                        |

## Querschnitt und offene Entscheidungen

| ID / Quelle                   | Bewertung                       | Beleg und verbleibende Grenze                                                                                                                                                                                                                                                                                                                                                                    |
| ----------------------------- | ------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Q-01 · §1, §10 Offline        | **T** für den Gesamtumfang      | Native Bildexporte und eingebettete Vorlage benötigen kein Netz; Rust-Tests und lokale Quellen belegen Einzelpfade. KQ-067 benötigt keinen Provider; kopiertes HTML lädt 100 Bildvorkommen in unabhängigem Offline-Kontext ohne externe Requests. KQ-068: ETW findet öffentlichen WebView2-Verkehr; Q-01 nicht bestanden, KQ-069 / #134 offen                                                    |
| Q-02 · §5, §10 KI-Datenschutz | **T** für den Gesamtumfang      | Lokale Pfade/Schlüssel bleiben Rust; [Threat Model](../architecture/threat-model.md), [Cloudgrenze](../architecture/decisions/0010-cloud-provider-boundary-and-os-credentials.md), Provider-Tests. KQ-068 findet öffentlichen WebView2-Verkehr ohne Provideraufruf; Q-02 nicht bestanden, #134 offen. Keine Uploadzuordnung aus Transportmetadaten; echte Cloud-Liveabnahme bleibt separat offen |
| Q-03 · §10 Export             | **N/A** für definierte Profile  | Rollen/Formatierungen, PNG-/RGB-Nachweise, SHA-256 und Maskierung gemäß F08-Zeilen. Vollständige Drehbuch-/assistive Abnahme fehlt; Unicode-Grenzen sichtbar beibehalten                                                                                                                                                                                                                         |
| Q-04 · §10 Timeline           | **T**, angrenzender M2-Nachweis | [Timeline-E2E](../../apps/desktop/e2e/timelines.spec.ts) und [Strukturrepository](../../crates/storage/src/structure_repository.rs) belegen zwei Zeitebenen; KQ-067 bestätigt native Ereignis-/Asset-/Szenenlinks und beide Zeitebenen nach Recovery. Die vollständige M2-Timeline-Abnahme bleibt separat. Kein Kalender-/Zeitzonenversprechen                                                   |
| D-01 · §11 Projektpaket       | **N/A**, Umfang entschieden     | SQLite-Projektordner plus `.kraqel`-Snapshotarchiv existieren; Backup bleibt separat zweiteilig. Kein offener Bedarf, nochmals ein Archivformat zu erfinden                                                                                                                                                                                                                                      |
| D-02 · §11 DOCX-Roundtrip     | **N**, begrenzt entschieden     | [DOCX v1](../data-format/DOCX_ROUNDTRIP.md) und native Abnahme; keine allgemeine Office-Bearbeitung oder Bilder zugesagt                                                                                                                                                                                                                                                                         |
| D-03 · §11 Medieneinbettung   | **N**, begrenzt entschieden     | [Bildvertrag](../data-format/ILLUSTRATED_MANUSCRIPT.md), ADR 0029–0032 und drei native Abnahmen; vier Ausgaben am Szenenende                                                                                                                                                                                                                                                                     |
| D-04 · §11 Drehbuch           | **T**                           | Rollenmodell und A4-Renderer umgesetzt. Für die vollständige Abnahme fehlen insbesondere die unabhängigen Office-Prüfungen.                                                                                                                                                                                                                                                                      |
| X-01 · §1, §6 V.2, M6         | **X** für 0.1                   | Bildgenerierung, Kartenmodell, FDX/Fountain-Integration und Storyboard sind keine hier nachgewiesenen Funktionen. FDX/Fountain sind Projektvorschlagsausschlüsse, keine wörtliche Pflichtenheftregel                                                                                                                                                                                             |
| X-02 · §8 M5                  | **O**, bewusst später           | Installerstart/Persistenz auf Zweitrechner/Clean-VM bleiben Releaseabnahme. Das verhindert keine Dokumentationsarbeit, darf aber nicht als bestanden gelten                                                                                                                                                                                                                                      |

## M3-G01: Prüfplan und durchgeführter funktionaler Gesamtlauf

Der folgende Ablauf war der KQ-066-Abnahmeplan. KQ-067 ist im
[Abnahmeprotokoll](native-m3-acceptance.md) als funktionaler Gesamtlauf bestanden: Datenlebenszyklus
nativ, große Bildauswahl über die Rust-Implementierungen, separate begrenzte
Layout-/Offline-/assistive Aussagen. Die 100 Bildcheckboxen wurden nicht einzeln nativ bedient. Der
Plan allein bleibt kein Ergebnis. Eigene Test-Bundle-ID und temporäre Daten, keine privaten
Manuskripte. Quellcommit, Windows-/App-Build, Prüflaufdatum und verwendete Werkzeuge im späteren
Protokoll festhalten. Rust-Test-Seed oder überprüfbares synthetisches Archiv verwenden;
Manipulationen nur in Kopien nach dokumentiertem Fingerabdruck.

1. **Inventar:** 100 Szenen, 50 unterschiedliche tatsächlich dekodierbare PNG/JPEG/WebP-Originale
   unter allen bestehenden Paket-/Dateigrenzen, mehrere Kapitel, mindestens zwei Timelines,
   Story-Bibel, visuelle Profile und Stilpresets. Ein Charakterbild an mindestens zwei Szenen und
   denselben Charakter binden; mindestens ein Asset an ein Ereignis. IDs, Text-/Dateihashes,
   Dimensionen, Linkmengen, Profil-/Promptquellen und Dokumentarten als Sollinventar erfassen.
   Vorgeschlagene Drehbuchrollen sind keine Voraussetzung dieses bestehenden M3-Korpus.
2. **Bearbeitung und Neustart:** Charakter und Szenen umbenennen, speichern, Prozess beenden, neu
   starten. Texte/IDs/Hashes sowie Links in beide Richtungen prüfen; Umbenennung darf nur die
   beabsichtigten Titel verändern. Keine neue Assetidentität oder verlorene Profilreferenz.
3. **Sicherung/Recovery:** Echte SQLite-/Asset-Sicherung über den vorhandenen Rust-Mechanismus
   erzeugen, Quellprojekt unverändert lassen und in isoliertes Ziel wiederherstellen. Der
   Mechanismus kann einen expliziten QA-Helfer brauchen; es wird kein derzeit nicht vorhandener
   UI-Button behauptet. Neues Ziel in der nativen App öffnen und Sollinventar vollständig
   vergleichen.
4. **Git und Archiv:** Zwei ausdrücklich erzeugte Snapshotstände mit nachvollziehbarer Änderung
   prüfen/committen, aktuellen gesicherten Stand über den nativen Archivdialog ausgeben, zweimal als
   neue Kopie importieren. Projektidentitäten müssen verschieden sein; innerhalb jeder Kopie müssen
   sämtliche fachlichen Referenzen gültig und Inhalte/Assethashes erwartungsgemäß erhalten sein.
   Wiederherstellung eines älteren Git-Stands nur über einen ausdrücklich gesicherten Snapshot
   nachweisen; kein nicht vorhandenes Checkout-UI voraussetzen.
5. **Exporte:** Definierte Formatierung und explizite Bildauswahl für alle vier Formate prüfen; bei
   50 Bildern genug Vorkommen für Wiederverwendung vorsehen. Manifest, relative Pfade,
   DOCX-Parts/PDF-RGB, alte Generationen und Öffnung ohne Quelldatenpfad vergleichen. Layoutseiten
   vollständig prüfen oder klar auf getrennte bereits abgenommene synthetische Korpora begrenzen;
   keine universelle Layoutabnahme behaupten.
6. **Fehler und Grenzen:** In separaten Kopien Original entfernen, Thumbnail entfernen und
   Originalbytes verändern. Nach erneutem Öffnen sichtbare Unverfügbarkeit prüfen; Backup/Export
   dürfen fehlerhafte Originale nicht erfolgreich übernehmen. Ungültiges Archiv/DOCX, Übergröße und
   Dialogabbruch müssen ohne neue Teilprojekte/-pakete enden. Vorherige Pakete/Quelle per Hash
   unverändert nachweisen; Integritätsfehler nicht bloß per Screenshot vermuten.
7. **Offline/Bedienung:** Gesamten Ablauf ohne eingerichteten Provider durchführen; keine
   unerwarteten Netzaktivitäten akzeptieren. Fachansichten mit Tastatur, sichtbarem Fokus und
   Screenreader stichprobenartig einschließlich Fehleransagen prüfen. Ohne unabhängigen
   Netz-/assistiven Nachweis den jeweiligen Status weiterhin als teilweise/offen kennzeichnen.

Ein bestandenes M3-G01 allein nimmt noch nicht den vorgeschlagenen Drehbuchumfang ab. Für eine
vollständige Freigabe sind Scopeentscheidungen, alle einschlägigen Zeilen und ihre Restprüfungen
einzeln zu bestätigen. Fehlende Evidenz bleibt mit Datum, Verantwortlichkeit und konkretem nächsten
Nachweis offen; Teilnachweise werden nicht hochgestuft.

## Reihenfolge nach KQ-066

1. Fachentscheid erfolgt: gesamter Drehbuchvertrag einschließlich F08-Importrichtungen am 15.
   November 2026 ausdrücklich freigegeben; ADR 0033 angenommen.
2. Funktionaler M3-G01-Lauf in KQ-067 bestanden; irreführende Medienstatusmeldung korrigiert und
   nativ erneut geprüft. KQ-068 schließt die nativen CRUD-Restprüfungen einschließlich bestätigter
   Löschung und Neustart; Q-01/Q-02 bleiben wegen des gemessenen WebView2-Verkehrs in #134 offen.
   Die abschließende Lösch-/Export-Hörprobe ist ebenfalls bestätigt; weitere allgemeine assistive
   F08-Randfälle bleiben getrennt.
3. Modell v2, lokale Schriftschnitte und A4-Renderer sind umgesetzt. Bei der weiteren Abnahme gelten
   die [Einzelnachweise für DB-01 bis DB-08](#stand-der-drehbuchkriterien) einschließlich ihrer
   jeweiligen Format- und Laufzeitgrenzen.
4. Vor stabilem 0.1 die übrigen F01–F09-/M5- und Abhängigkeitsrisiken separat schließen.
