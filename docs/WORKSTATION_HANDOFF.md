# Rechnerwechsel und Arbeitsübergabe

Stand vom 25. November 2026 auf dem Entwicklungszweig. Der Desktop unterstützt Drehbuchszenen mit sechs Absatzrollen und einen eigenen A4-PDF-Export für ausgewählte Szenen. Die erste gespeicherte Drehbuchszene hebt das Projektschema auf 7 an. Reine Prosaprojekte behalten Schema 6. Alpha 7 enthält diese Funktion noch nicht. [Datenformat und Kompatibilität](data-format/screenplay-v2.md)

## Windows-Entwicklungsstand vom 25. November

Der reguläre Windows-Start zeigt die lokale Projektliste. **Projekt anlegen** erstellt ein neues Projekt und führt direkt zur Kapitelauswahl. **Manuskript öffnen** führt für ein ausgewähltes Projekt dorthin. **Im Studio öffnen** und **Projektverwaltung öffnen** starten den bisherigen Desktop mit WebView2, etwa für den Import oder das Löschen eines Projekts. Die veröffentlichte Alpha 7 hat noch den bisherigen Einstieg.

Der installierte Drehbuchkandidat `aca30d0ae0d9` bestand den Bedienlauf bis zum Wiederöffnen und A4-Export. Im neueren Stand kann das gesamte Manuskript einschließlich Drehbuchrollen auch als Markdown, HTML und gewöhnliches DOCX ausgegeben werden. Die DOCX-Austauschfunktion v1 bleibt auf Prosa beschränkt. [Abnahme mit Word und LibreOffice](testing/F08_M3_ACCEPTANCE_MATRIX.md)

## Ein Manuskript nativ lesen und bearbeiten

1. **Projekt anlegen** wählen, den Projektnamen eingeben und bestätigen. Das neue Projekt enthält **Kapitel 1** und **Szene 1**. Für ein bestehendes Projekt dessen Namen auswählen und **Manuskript öffnen** drücken.
2. Das Kapitel öffnen und in der Szenenauswahl **Lesen** oder **Bearbeiten** wählen. Lesen öffnet ein schreibgeschütztes Fenster. Bearbeiten öffnet für einfache Prosa direkt den isolierten Editor. **Speichern** übergibt den aktuellen Entwurf zur Speicherung unter der beim Öffnen gebundenen Revision.
3. Zum Wiederöffnen dieselbe Szene auswählen. Mehrere unformatierte Absätze und Leerzeilen werden unterstützt. Die Textgrenze beträgt 64 KiB UTF-8. Markierungen, Listen, Überschriften, Drehbuchrollen und sonstige nicht exakt rückwandelbare Dokumentstrukturen erhalten eine schreibgeschützte Textansicht. Für ihre Bearbeitung ist weiterhin das Studio vorgesehen.

**Menübedienung.** Mit **Alt** wechselst du in die native Menüleiste. Unter **Datei** kannst du das Szenenfenster schließen und im Editor speichern. Unter **Bearbeiten** findest du die Szenensuche und die Treffernavigation. Im Editor sind außerdem Ersetzen und der Fokusmodus verfügbar. Der Fassungsvergleich setzt einen gemeldeten Revisionskonflikt voraus.

**Beim Schreiben.** Die normale Bearbeitungsansicht zeigt die Wortzahl und die belegten UTF-8-Bytes. Für die Grenze von 65.536 Bytes zählt ein Absatzwechsel als ein Byte. Umlaute und Emoji können mehrere Bytes belegen. Einen zu langen Entwurf kannst du im Fenster kürzen und anschließend speichern. **F11** schaltet den Fokusmodus ein und aus. **Esc** führt aus dem Fokusmodus zur normalen Bearbeitungsansicht zurück. **Strg+A** markiert im Szenenfeld den gesamten Text. **Strg+S** speichert die bearbeitete Szene auch im Fokusmodus. **Strg+F** öffnet beim Lesen und Bearbeiten die Suche innerhalb der geöffneten Szene. **F3** springt zum nächsten Treffer, **Umschalt+F3** zum vorherigen. Am Ende geht es beim ersten Treffer weiter, in Gegenrichtung beim letzten. **Esc** schließt die Suchzeile und führt zum Szenentext zurück. Zum Ersetzen im Editor **Strg+H** drücken, einen Treffer auswählen und den Ersatz ausdrücklich anwenden. Mit **Alle ersetzen** zeigt der Editor zunächst die Zahl der betroffenen Treffer in dieser Szene. Bestätige sie, um alle Treffer im Entwurf zu ändern. Beim Öffnen mit **Strg+F** oder **Strg+H** wird eine gültige einzeilige Auswahl aus dem Szenentext als Suchbegriff übernommen. Für diese Vorbelegung gilt eine Grenze von 256 UTF-16-Einheiten. Bei längerer, leerer oder mehrzeiliger Auswahl bleibt der bisherige Suchbegriff erhalten. Im Ersatzfeld ändert **Enter** den aktuellen Treffer. **Strg+Enter** zeigt die Trefferzahl für den Gesamtersatz und fragt nach Bestätigung. Ein leeres Ersatzfeld löscht die betroffenen Fundstellen. **Strg+Z** im Szenenfeld nimmt die Änderung zurück. Erst **Speichern** oder **Strg+S** schreibt den Entwurf.

**Editor schließen.** Bei einem geänderten Entwurf öffnen X und Alt+F4 die Schließabfrage. **Weiterschreiben** ist vorausgewählt. **Verwerfen** schließt das Fenster und lässt die gespeicherte Szene bestehen. **Speichern** schließt nach erfolgreicher Bestätigung durch den Desktop. Entspricht der Text wieder vollständig dem ursprünglichen Stand, entfällt die Abfrage.

Bei einem Speicherfehler bleibt der Text im selben Editfenster und lässt sich nach Behebung erneut speichern. Hat sich die Szene zwischenzeitlich geändert, schützt die Revisionsprüfung die neuere Fassung. Der lokale Entwurf bleibt im Arbeitsspeicher des offenen Fensters. Ein Programmabsturz oder ein Neustart beendet diesen Schutz.

**Konflikt auflösen.** Nach einem Revisionskonflikt **Fassungen vergleichen** wählen. Erst dieser Aufruf lädt den aktuellen Text derselben Szene. Im isolierten Vergleich stehen die Absätze an derselben Position nebeneinander. Für jede Position den lokalen oder den gespeicherten Absatz wählen und das vollständige Ergebnis prüfen. Bei eingefügten oder entfernten Absätzen besonders auf die Zuordnung achten. Das Schließen des Vergleichs führt zum ursprünglichen Entwurf zurück.

Die ausdrückliche Speicherbestätigung gilt für die angezeigte Vergleichsbasis. Hat sich diese erneut geändert, zeigt der Vergleich den neueren Stand mit den bisherigen Entscheidungen je Absatzposition an. Das Ergebnis erneut prüfen und bestätigen. Kann der gespeicherte Stand nicht verlustfrei als einfache Prosa bearbeitet werden, bleibt der lokale Entwurf im Editor.

**Weitere Kapitel und Szenen.** In der Kapitelauswahl **Kapitel anlegen** wählen und einen Titel eingeben. Das neue Kapitel ist anschließend ausgewählt. Das Kapitel öffnen und über **Szene anlegen** die erste oder weitere Szenen hinzufügen. Zum Ändern eines Titels den Eintrag markieren und **Umbenennen** wählen. Bei **Abbrechen** bleibt die bisherige Auswahl erhalten. Gleichnamige Szenen in verschiedenen Kapiteln behalten ihre eigenen Kennungen. Das Umbenennen lässt Manuskripttext und Textrevision unberührt.

**Reihenfolge ändern.** Das Kapitel oder die Szene markieren, mit Tab zu **Nach oben** oder **Nach unten** wechseln und die Schaltfläche auslösen. Der verschobene Eintrag bleibt ausgewählt. Für einen Kapitelwechsel in der Szenenliste **In Kapitel verschieben** wählen und das Ziel bestätigen. Die Nummern in der Zielauswahl entsprechen der Kapitelreihenfolge. Nach dem Verschieben ist die Szene am Ende des Zielkapitels markiert. Bei einer zwischenzeitlich geänderten Gliederung die aktualisierte Liste prüfen und den Vorgang bei Bedarf erneut auslösen.

**Kapitel oder Szene entfernen.** Den Eintrag markieren und in der Liste **Entf** drücken oder **Entfernen …** wählen. Bei einer Szene zeigt die Vorschau auch ihr Kapitel, bei einem Kapitel die zugehörigen Szenen. Betroffene Plotkarten und Bildprompt-Entwürfe werden mit entfernt. Verknüpfungen zu Story-Bibel-Einträgen, Ereignissen, Medien und Profilen werden gelöst. Die verknüpften Einträge, Ereignisse, Medien und Profile selbst bleiben erhalten. **Abbrechen** ist die Vorgabe. Erst die ausdrückliche Bestätigung löscht den geprüften Stand. Hat sich dieser inzwischen geändert, die aktualisierte Auswahl prüfen und bei Bedarf erneut bestätigen.

Die lokale Datenbank bleibt im vertrauenswürdigen Desktop-Speicherdienst. Die beiden isolierten Rollen erhalten nur die ausgewählte Szene über private Pipes. Beim Speichern prüft eine SQLite-Transaktion die gebundene Revision und das ursprüngliche Dokument. Ein konkurrierender Stand bleibt erhalten. Der Editierauftrag `--managed-project-scene-edit-v3` hält den bereits authentisierten privaten Kanal bis zur Speicherentscheidung offen. Der Desktop beantwortet den Speicherversuch mit einer Bestätigung nach dem SQLite-Commit, einem Revisionskonflikt oder einem Speicherfehler. Nach einem Konflikt bleibt die ursprüngliche Schreibfreigabe gesperrt. Für die bestätigte Übernahme bildet der Desktop den Text aus dem gebundenen Entwurf, der geprüften gespeicherten Fassung und der Absatzwahl. Eine frische Schreibfreigabe bindet die Übernahme an die geprüfte Szene und Revision.

**Bedienabnahme vom 25. November.** Im echten Windows-Debug-Build öffnete sich die ausgewählte Szene eines eigens angelegten Projekts mit Leseschutz und dem erwarteten Text auf Revision 0. Nach ausdrücklicher Bearbeitungsfreigabe, Speichern zweier Absätze, regulärem Beenden und einem neuen Prozess zeigte eine frische Lesesitzung beide Absätze exakt wieder an. SQLite bestätigte Editordokument v1 und Revision 1. Der Speichertest verwarf außerdem eine veraltete Revision sowie ein bei gleicher Revision zwischenzeitlich geändertes Originaldokument.

Der optimierte Windows-Desktop aus `e24c9c959069` bestand denselben Bedienlauf. Nach einem neuen Desktopprozess zeigte das schreibgeschützte Fenster beide bearbeiteten Absätze wieder an. Die direkte SQLite-Prüfung bestätigte beide Absatzknoten und Revision 1. Im Vorläufer scheiterte der Start des isolierten Datendienstes vor dessen Bootstrap-Bestätigung. Der ausgelieferte Manuskriptdatendienst ist jetzt als konsolenloses Windows-Programm gebaut und verwendet die ausdrücklich vererbten Pipes. Die reguläre Startfrist beträgt weiterhin fünf Sekunden.

Beim ersten abgebrochenen UI-Prüflauf wurde der Starter bei offenem Lesefenster beendet. Im Windows-Paketverzeichnis blieb ein temporäres UI-AppContainer-Profil. Seine Kontrollablage war bereits entfernt; ohne generationsgebundenen Beleg bleibt das Profil unangetastet. Der reguläre Folgelauf hinterließ kein weiteres Profil. Die Prüfhilfen behalten künftig bei unbestätigtem Sitzungsende die Kontrollablage zurück.

## Wiedereinstieg nach dem 24. November 2026

Auf dem Entwicklungszweig `feat/kq-069-live-file-selection` mit `git pull --ff-only` beginnen.
Die vorherige [Alpha 6](https://github.com/kolkrabeofdoom/kraqel/releases/tag/v0.1.0-alpha.6)
trägt den Tag auf `a202aca3cd43`. Die öffentliche [Alpha 7](https://github.com/kolkrabeofdoom/kraqel/releases/tag/v0.1.0-alpha.7) mit Ordnerimport stammt aus
`a9a3a673ff14`. Auf dem Buildrechner liegen Installer und Prüfsummendatei unter
`artifacts/release/0.1.0/windows-x64/a9a3a673ff14/`, Archivhelfer und Anleitung unter
`Rechnerwechsel/` daneben. Die lokalen Installer-Prüfsummen beider Stände, die drei öffentlichen
Release-Dateien und der portable Archivhelfer wurden zum Tagesabschluss abgeglichen.

Der gebaute Desktop übernahm das geschlossene Demoprojekt mit 22 inhaltlich identischen
SQLite-Tabellen und 42 bytegleichen Bild- und Vorschaudateien. Nach einem Neustart öffnete sich
die Kopie. Die separate Archivwahl ließ sich mit der Passphrase wiederherstellen und enthielt
genau die ausgewählte Zusatzdatei.

**Upgrade-Abnahme am 25. November 2026 bestanden.** Im getrennten Windows-Testprofil öffnete
dieselbe Startmenüverknüpfung nach dem Update von Alpha 6 auf `a9a3a673ff14` das vorhandene
Demoprojekt mit derselben UUID bis zur Projektansicht. Auf der Startseite war **Projektordner
auswählen** sichtbar, fokussierbar und aktiviert. Beide Desktopprogramme verwendeten das
Windows-GUI-Subsystem. Testinstallation, Registrierung, Verknüpfungen und Testprofil wurden
bereinigt. Der geprüfte Installer hat SHA-256
`85c4f09e8e24ae4f82946a69d8d67f4f540f7af087ea2fe2ef90c2e07c905bdd`.
Der unveränderte Installer wurde als Alpha 7 veröffentlicht. Das separate Rechnerwechsel-Paket enthält den portablen Archivhelfer für die Wiederherstellung.

## Einen vorhandenen Windows-Projektordner übernehmen

Ab Alpha 7 auf der Startseite **Projektordner auswählen** öffnen. Den
geschlossenen lokalen Projektordner mit seiner ursprünglichen UUID als Namen auswählen. Die Vorschau
zeigt die Anzahl der Dokumente und Bilder sowie die Dateianzahl jeder Zusatzgruppe. Projektpfade,
Dateinamen und Passphrase bleiben im nativen Teil der Anwendung.

**Arbeitskopie anlegen** und die native Rückfrage bestätigen. KRAQEL legt die Kopie unter der
ursprünglichen, am Ziel freien Projektkennung an. Die Auswahl läuft nach zehn Minuten ab. Bei
geändertem Dateibestand die Vorschau neu laden. Ist die Kennung im lokalen Katalog belegt, steht der
Vergleich vorbereiteter Arbeitskopien im Versionsdialog zur Verfügung.

Wer Sicherungen, Exporte, Git-Historie oder weitere Dateien mitnehmen möchte, markiert die
gewünschten Gruppen einzeln. Ein neues Archivziel außerhalb von Quelle und Katalog wählen und die
eigene Passphrase zweimal im Windows-Dialog eingeben. Das portable Zusatzarchiv enthält ein ZIP mit
höchstens 32 MiB vor der Verschlüsselung. Es lässt sich auch auf einem anderen Rechner mit der
Passphrase und `kraqel-archiv restore QUELLE.zip.age NEUES_ZIEL.zip` wiederherstellen. Kann das
Archiv nach erfolgreicher Projektanlage nicht am Ziel veröffentlicht werden, meldet die App beide
Ergebnisse getrennt. Die Zusatzdateien bleiben in der Quelle.

## Getrennt auf zwei Rechnern schreiben

Vor dem Abzweigen im Versionsdialog die gemeinsame Szenenbasis sichern. Nach der Rückkehr lassen sich einseitige Änderungen und Konflikte anhand der Basisdateien lesend vergleichen. Für die Auswahl einseitiger Änderungen den vorbereiteten Quellordner und seinen getrennt bestätigten Inventarhash mitnehmen. Die Bestätigung erzeugt einen weiteren Vorbereitungsordner. Die vollständigen Arbeitskopien für die weitere Bearbeitung separat aufbewahren. Aktivierung und Rücknahme geschehen im geöffneten Zielprojekt. Seine lokale Freigabe und die unverschlüsselte Rücknahmedatei gehören zu diesem Arbeitsplatz. Für den Wiederanlauf die separat gesicherte Prüfsumme bereithalten. [Ablauf im Desktop-Entwicklungsstand](../README.md#szenenstände-auf-zwei-rechnern-vergleichen)

Bei einem Textkonflikt im Windows-Entwicklungsstand genau die gewünschte Szene im Versionsdialog auswählen und die isolierte Volltextansicht ausdrücklich öffnen. Die bestätigte externe Fassung ergibt eine neue Arbeitskopie. Zur Aktivierung im geöffneten Zielprojekt zuerst die früheren Fassungen lokal sichern. Die eingebetteten Rollenprogramme werden beim Desktop-Build aus dem Repository erzeugt. [Bedienweg](../README.md#szenenstände-auf-zwei-rechnern-vergleichen)

## Desktop-Vorbereitung und isolierte Vorschau

Der lokale Empfang ist vom Desktop-Speichermodul bis zum schreibgeschützten Win32-Fenster mit synthetischen Arbeitsdaten geprüft. Die Anleitung enthält den Build und den automatischen VHDX-Lauf. Der Windows-Entwicklungsstand unterstützt jetzt den erstmaligen Empfang als lokales Projekt. Die physische Zweitrechnerprobe steht aus. [Prüflauf und Grenzen](data-format/product-start-integration-v1-draft.md#vorbereitete-desktop-arbeitskopie-im-isolierten-vorschaufenster)

## Vorbereitete Arbeitskopie erstmals empfangen

1. Auf dem Quellrechner im Versionsdialog **Übernahme vorbereiten** wählen. Den angezeigten Inventar-SHA-256 separat sichern.
2. KRAQEL auf dem Windows-Zielrechner einmal öffnen. Unter `%LOCALAPPDATA%\org.kraqel.desktop\projects` den Ordner `empfang` anlegen und den vollständigen Ordner `uebernahme-<UUID>` hineinkopieren. Die Arbeitskopie enthält unverschlüsselte Manuskript- und Mediendaten. Für den Transport einen entsprechend geschützten Datenträger verwenden.
3. Auf der Startseite unter **Windows-Arbeitskopie empfangen** den Ordnernamen und die separat bestätigte Inventarprüfsumme eingeben. **Als lokales Projekt übernehmen** wählen. Erst nach erfolgreicher Prüfung erscheint das Projekt in der Liste. Der Eingangsordner bleibt erhalten.
4. Ist die Projektkennung am Ziel belegt, im vorhandenen Projekt **Werkzeuge → Versionen → Eingangskopie vergleichen** öffnen. Mit Ordnername und separat bestätigtem Inventar-SHA-256 zeigt **Arbeitsdaten vergleichen** die Bereiche mit abweichenden gespeicherten Einträgen an. Dazu zählen auch Revisionen und Zeitstempel. Bilder und Vorschaubilder werden nur bei angekreuzter Zusatzprüfung gelesen. Dann vergleicht KRAQEL die referenzierten Dateien lokal anhand ihrer Prüfsummen. Fehlende referenzierte Dateien auf beiden Seiten oder ein abweichender Inhaltsbeleg sperren die Zusatzprüfung. Das Ergebnis beschreibt die einzeln gelesenen Dateien. Eine spätere Übernahme braucht eine frische Prüfung. Für die anschließende Entscheidung über einzelne Szenen den Szenenvergleich mit der gemeinsamen Ausgangsbasis verwenden.

## Einzelne Projekteinträge übernehmen

Im geöffneten Zielprojekt unter **Werkzeuge → Versionen** die Eingangskopie mit ihrem separat bestätigten Inventar-SHA-256 vergleichen.

1. **Einzelne Projekteinträge übernehmen** aufklappen, den Projektbereich wählen und **Abweichende Einträge laden**. Die Liste verwendet die stabilen Kennungen. Bei Ereigniszuordnungen gehört das Kennungspaar zusammen. Gleiche Namen sind kein Identitätsbeleg.
2. Einen geänderten oder nur im Eingang vorhandenen Eintrag auswählen. **Fassungen im isolierten Fenster öffnen** und den nativen Dialog bestätigen. Die vollständigen Felder sind im Win32-Fenster lesbar. Nur lokal vorhandene Einträge bleiben erhalten.
3. Die externe Fassung im Fenster ausdrücklich wählen. KRAQEL bereitet eine separate Arbeitskopie vor. **Rücknahme für diesen Eintrag sichern** und den angezeigten lokalen Beleg aufbewahren. Erst **Eintrag aktivieren** verändert das geöffnete Projekt.
4. Mit **Eintragsübernahme zurücknehmen** den gesicherten Ausgangsstand wiederherstellen. Nach einem Programmneustart lässt sich der Beleg im selben Projekt erneut eingeben. Bei einem zwischenzeitlich geänderten Stand die Einträge neu laden und die Fassungswahl wiederholen. Die Auswahl bleibt beim erneuten Laden anhand ihrer Kennung erhalten, sofern der Eintrag weiterhin abweicht.

Für ein einzelnes Bild **Bilder und Vorschaubilder → Medienkennungen laden** wählen. Auch **Gleicher Katalogstand** lässt sich öffnen, weil die Vorschaudatei bei identischen Metadaten abweichen kann. Nach ausdrücklicher Bestätigung prüft KRAQEL Original und Vorschau. Im isolierten Fenster stehen die Metadaten, Dateiprüfsummen und Verknüpfungen zur Wahl. Die verknüpften Ziele müssen im Zielprojekt vorhanden sein. Die lokale Rücknahmesicherung enthält die früheren Bilddaten. Frühere Dateigenerationen können für noch offene Lesezugriffe im Projektordner verbleiben. Projekt und Sicherung deshalb gemeinsam schützen.

Eine Beziehung setzt die beteiligten Einträge im Ziel voraus. Das gilt entsprechend für Dokumentverknüpfungen, Plotkarten, Ereignisse und ihre Zuordnungen. Fehlt einer gewählten Plotkarte der Handlungsstrang oder einem Ereignis die Zeitachse, bietet der Dialog die passende Kennung aus derselben Eingangskopie an. **Fehlenden Handlungsstrang auswählen** bzw. **Fehlende Zeitachse auswählen** führt zur separaten Fassungswahl. Die Voraussetzung erhält einen eigenen Rücknahmebeleg. Nach ihrer Aktivierung lädt der Dialog die ursprüngliche Karte bzw. das Ereignis erneut. Die Fassung dieses Eintrags ausdrücklich prüfen und übernehmen. Für die Rücknahme zuerst den abhängigen Eintrag und anschließend die Voraussetzung zurücknehmen. Handlungsstränge und Zeitachsen lassen sich auch direkt im Projektbereich wählen. Ein Typwechsel für dieselbe Story-Bibel-Kennung ist gesperrt. Hat eine neu übernommene Figur inzwischen eine Beziehung erhalten, sperrt KRAQEL die Rücknahme. Bei unklarem Transaktionsende den aktuellen Stand erneut vergleichen. Lokale Sicherungen können vollständige frühere Inhalte enthalten und gehören auf einen geschützten Datenträger.

## Zusatzarchiv auf einem anderen Windows-PC öffnen

Für die synthetische P3-Übernahme lassen sich ausgewählte Zusatzdateien lokal mit einer Passphrase verschlüsseln. Auf dem Entwicklungsrechner im Repository ausführen (der native P3-Prüfstarter muss wie in der P3-Anleitung gebaut sein):

```powershell
cargo build --locked --release -p kraqel-archiv
python -I scripts/runtime-isolation/p3-volume-fixture.py export-supplement .\zusatz.zip.age
```

Im Fenster die gewünschten Gruppen markieren und das Quellprojekt bestätigen. Im Terminal eine lange, einzigartige Passphrase zweimal verdeckt eingeben. Das Ziel muss neu sein. Die Beispieldateien stammen aus dem synthetischen Projektkatalog.

Für den Rechnerwechsel die Datei `zusatz.zip.age` und die portable Windows-x64-EXE mitnehmen. Der normale Build für den Prüfpfad lädt `VCRUNTIME140.dll`. Die portable Variante bindet die C-Laufzeit statisch. Der folgende Build setzt die Option ausschließlich im gestarteten Prozess:

```powershell
cmd /d /c "set CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS=-C target-feature=+crt-static&&cargo build --locked --release --target x86_64-pc-windows-msvc -p kraqel-archiv"
```

Die portable EXE liegt in `target/x86_64-pc-windows-msvc/release/kraqel-archiv.exe`. Die Passphrase getrennt aufbewahren. Auf dem zweiten Windows-x64-PC im Ordner mit beiden Dateien ausführen:

```powershell
.\kraqel-archiv.exe restore .\zusatz.zip.age .\wiederhergestellt.zip
```

Nach Eingabe der Passphrase entsteht ein **unverschlüsseltes ZIP** am gewählten neuen Ziel. Ein vorhandenes Ziel bleibt erhalten. Zur Wiederherstellung genügen Archiv, Programm und Passphrase. Windows-Konto, Laufwerksbuchstabe und ursprüngliches Projektjournal spielen dafür keine Rolle. Eine verlorene Passphrase lässt sich über KRAQEL nicht zurücksetzen.

## Wiedereinstieg in den verwalteten Szeneneditor

Arbeitsbranch ist `feat/kq-069-live-file-selection`. Der Fokusmodus ist im verwalteten Szeneneditor verfügbar; Bedienung und Prüfumfang stehen im Entwicklungsstatus.
Im vorhandenen sauberen Checkout auf einem beliebigen Laufwerk:

```powershell
git switch feat/kq-069-live-file-selection
git pull --ff-only
git status -sb
```

Für den aktuellen P3-Vorschaupfad nach dem Checkout die Auswahl und die schreibgeschützte Anzeige als einen Vorgang starten. Die vorhandenen VHDX-/Wiederanlaufprüfungen können separat aufgerufen werden. [Befehl, Voraussetzungen und Netzbeleg](data-format/product-start-integration-v1-draft.md#bedienbare-p3-übernahme-mit-separatem-zusatzarchiv)

Der Wiederanlauf der synthetischen Szenenübernahme ist im geprüften Umfang abgeschlossen. Im langen Klartexteditor sind Wortzahl, UTF-16-Budget und Fokusmodus verfügbar. F11 wechselt die Ansicht, Esc verlässt den Fokusmodus. Beim Revisionsvergleich zeigt F7 die Wirkung von F8. Nach der Übernahme bleibt die bearbeitete Stelle im Entwurf sichtbar. Die Befehle für Suche, Fokusmodus und Konfliktentscheidung sind auch per Menü erreichbar. Rückgängig und Wiederholen stellen die Auswahl im langen Entwurf wieder her. [Aktueller Stand und Prüfumfang](DEVELOPMENT_STATUS.md#verwalteter-win32-szeneneditor)

Der aktualisierte Starter lässt unmarkierte Altpaare nur lesen. Ein gebundenes Altpaar kann unter demselben Starter mit `upgrade-win32-package` auf ein geprüftes Schreibpaar wechseln. Der lokale Windows-Lauf bestätigte Sperre, Lesen, Rollenwechsel, Schreiben und Wiederöffnen. Nach einem Starterwechsel gibt erst `adopt-win32-scene <Revision> <Snapshot-SHA-256> <Kontrollkopf>` die gespeicherte synthetische Szene für den neuen Starter frei. Revision und Hash stehen in `verify-win32-package`. Eine offene Sitzung sperrt die Übernahme. [Bedienung, Wiederanlauf und Windows-Nachweis](architecture/decisions/0036-native-win32-ui-role-evaluation.md#ausdrückliche-übernahme-der-synthetischen-szene-nach-einem-starterupdate). [Aufruf, Grenzen und ETW-Nachweis](architecture/decisions/0036-native-win32-ui-role-evaluation.md#lesezugriff-für-altpaare-und-authentisierter-rollenwechsel).

Bei einem Starterabbruch während der Übernahme entscheidet der veröffentlichte Kontrollkopf über die Zuordnung. Vor dem Eintrag bleibt die alte Zuordnung erhalten, nach Veröffentlichung gilt die neue. Zwischen synchronisiertem Eintrag und Kopf sperrt der reguläre Start die Ablage. Der dafür ausgerüstete aktive Starter kann mit `recover-win32-scene-adoption <Revision> <Snapshot-SHA-256> <Kontrollkopf-vor-Übernahme>` genau einen eindeutig belegten Übernahmeeintrag abschließen. Der Windows-Prozesslauf bestätigte die einmalige Übernahme nach echtem Starterabbruch. Nach einem zweiten Abbruch bei vorbereitetem Kopf kann derselbe ausdrückliche Aufruf den bytegenau passenden authentisierten Kopf veröffentlichen. Ein abweichender Kopf sperrt die Übernahme. [Geprüfte Abbruchpunkte und Vorgehen](architecture/decisions/0036-native-win32-ui-role-evaluation.md#wiederanlauf-nach-abbruch-der-szenenübernahme).

**Stand zum Tagesabschluss am 23. November 2026.** F4 oder **Bearbeiten → Gespeicherte Fettstelle auswählen** springt im Bearbeitungsauftrag zur eindeutig zugeordneten Fettstelle und markiert sie im Absatz. Im lokalen Windows-Lauf markierte F4 nach dem Wechsel von „Rabe“ zu „Kolkrabe“ genau „ab“. Bei „Kolkrabe ab“ blieb die vorhandene Auswahl erhalten.

**Stand am 24. November 2026.** Zum Entfernen einer gespeicherten einzelnen Fettmarkierung die Szene im Bearbeitungsauftrag öffnen und **Bearbeiten → Fettmarkierung entfernen, speichern und schließen** wählen. Der Befehl ist bei ungespeicherten Änderungen gesperrt. Der lokale Windows-Lauf bestätigte nach dem Wiederöffnen den vollständigen Text und die weiterhin fett gesetzte „42“.

Der verwaltete Starter erkennt gemischte Generationen mit einseitiger Inline-Kennung vor dem Sitzungsstart. Für das passende Paar sind Markieren, Entfernen und Wiederöffnen im aktuellen Windows-Netzvergleich geprüft. [Paketprüfung und ETW-Nachweis](architecture/decisions/0036-native-win32-ui-role-evaluation.md#paketkompatibilität-und-etw-für-einzelne-fettmarkierungen).

Zum Weiterbearbeiten einer gespeicherten Szene mit einer einzelnen Fettstelle den Auftrag `edit-win32-scene` öffnen. Strg+S erhält die Fettstelle auch nach einer eindeutigen Änderung davor oder dahinter. Fordert das Fenster zur erneuten Auswahl auf, die gewünschte Stelle markieren und mit Strg+Umschalt+B speichern. Zum gezielten Ersetzen der gespeicherten Fettstelle Strg+H drücken, den Ersatztext eingeben und im Ersatzfeld mit Strg+Umschalt+Enter speichern und schließen.

Der verwaltete Win32-Editor bearbeitet ausschließlich die synthetische Szene A. Text,
Fettmarkierung des zweiten Absatzes, Rückgängig/Wiederholen und lokales Suchen/Ersetzen sind
bis zum Speichern und Wiederöffnen geprüft. Der [Entwicklungsstatus](DEVELOPMENT_STATUS.md#verwalteter-win32-szeneneditor)
beschreibt die Bedienung und die geprüften Grenzen. Beim Öffnen einer bereits gespeicherten Szene A mit zusätzlichen Absätzen oder bekannten Inline-Markierungen ist die [schreibgeschützte Textvorschau](architecture/decisions/0036-native-win32-ui-role-evaluation.md#schreibgeschützte-textvorschau-für-formatierte-szenen) geprüft. In der Vorschau öffnet Strg+F die lokale Suche. Enter, F3 und Umschalt+F3 navigieren zu Treffern. Zusätzliche Felder in der Dokumenthülle sperren das Wiederöffnen im synthetischen Datendienst. Im langen Klartextentwurf sind [lokales Suchen, Ersetzen und Rückgängig/Wiederholen](architecture/decisions/0036-native-win32-ui-role-evaluation.md#lokales-suchen-und-ersetzen-im-langen-klartext) verfügbar. Die Arbeitsweise für fortlaufende Roadmap-Schritte steht in [AGENTS.md](../AGENTS.md).

Für das fortlaufende Autosave den Forschungsauftrag `continue-win32-scene` verwenden. Die Zahl im Fenstertitel zeigt den aktuellen Schreibzug und das Sitzungsbudget. Nach dem letzten bestätigten Schreibzug schließt das Fenster regulär. Während der Bestätigung eingegebene Zeichen werden nach der frischen Revisionsbindung verarbeitet. Der [Windows-Nachweis](architecture/decisions/0036-native-win32-ui-role-evaluation.md#eingaben-während-der-autosave-bestätigung) umfasst auch den Starterabbruch bei wartender Eingabe. [Vertrag und Windows-Nachweise](architecture/decisions/0036-native-win32-ui-role-evaluation.md#fortlaufendes-autosave-unter-journalbudget). Beim [Schließen mit wartender Eingabe](architecture/decisions/0036-native-win32-ui-role-evaluation.md#schließen-mit-wartender-eingabe) sind alle drei Entscheidungen geprüft. Die [Speicheranzeige](architecture/decisions/0036-native-win32-ui-role-evaluation.md#speicheranzeige-im-fortlaufenden-editor) ist geprüft. Bei einem [Revisionskonflikt](architecture/decisions/0036-native-win32-ui-role-evaluation.md#revisionskonflikt-bei-einem-schreibzug) bleibt der Entwurf bis zur Entscheidung offen. Der [lesende Revisionsvergleich](architecture/decisions/0036-native-win32-ui-role-evaluation.md#lesender-revisionsvergleich-im-konfliktfenster) ist umgesetzt. Im fortlaufenden Auftrag ist die [gezielte Übernahme](architecture/decisions/0036-native-win32-ui-role-evaluation.md#gezielte-übernahme-unter-frischer-schreibbindung) für zwei Absätze und die Fettmarkierung geprüft. Im Bearbeitungsauftrag öffnet längerer unformatierter Text im Klartexteditor. Bei einer bereits gespeicherten kurzen Klartextszene führt **Bearbeiten → Als längeren Klartext bearbeiten** dorthin. Bei verfügbarem Folgezug speichert auch der lange Klartexteditor nach 700 ms Schreibpause. Der letzte Schreibzug verlangt ausdrücklich Speichern und Schließen. Strg+Enter speichert und lässt das Fenster bei verfügbarem Folgezug offen. Strg+S speichert und schließt. Beim Konflikt im langen Klartext steht rechts die aktuelle Dienstrevision als schreibgeschützte Textvorschau. Mit F6 wechselt man zurück zum lokalen Entwurf. F7 bzw. Umschalt+F7 springt zum nächsten bzw. vorherigen Unterschied. Nach einer lokalen Änderung beginnt die Navigation wieder beim ersten bzw. letzten Unterschied. F8 übernimmt den ausgewählten Dienstblock in den lokalen Entwurf. Fehlt er im Dienststand, entfernt F8 die entsprechenden lokalen Absätze. Autosave bleibt im Konflikt gesperrt. Bei unformatiertem Klartext und freiem Schreibzug bietet F9 an, den gesamten Entwurf ausdrücklich zu bestätigen und unter frischer Bindung zu speichern. Liegt die Dienstfassung bereits beim ersten Vergleich mehrere Revisionen voraus, erfordert F9 zusätzlich die Freigabe für einen revisionsübergreifenden Schreibzug. Ändert sich die Vergleichsbasis während der Bestätigung, bleibt der Entwurf offen. Bei freigegebenem Wiedervergleich zeigt das Fenster die neue Dienstrevision. F9 verlangt für einen zulässigen Klartextstand eine neue Bestätigung. Bei einer Revisionslücke im ersten Zwei-Absatz-Vergleich können beide Absätze und die Fettmarkierung einzeln aus Entwurf oder Dienstfassung übernommen werden. Zum Speichern muss der revisionsübergreifende Schreibzug freigegeben und die Auswahl für die angezeigte Revision ausdrücklich bestätigt sein. Ohne Freigabe bleibt die Auswahl eine lokale Vorschau. Beim Überarbeiten im selben Vergleich bleibt die gelesene Zeile nach Möglichkeit sichtbar. Mit Tab geht es vom zweiten Absatz ins Vergleichsfeld, mit F6 zurück zum zuletzt bearbeiteten Absatz. Außerhalb des Fokusmodus öffnet Esc auch aus dem Vergleichsfeld die Schließabfrage. Mit **Nein** liest man dort weiter. Die lokale Vorschau ist bereits bei der ersten Auswahl verfügbar. Über **Übernahme → Ursprüngliche Bearbeitungsbasis anzeigen** lässt sich beim Entscheiden die Ausgangsfassung einblenden. Im Menü **Übernahme** kann man vom gesamten lokalen Entwurf oder vom Dienststand ausgehen und die Auswahl pro Feld anpassen. Eindeutig lokale Änderungen lassen sich zusätzlich markieren. Nach einer versehentlichen Auswahl stellt **Übernahme → Letzte Auswahl rückgängig machen** die vorherigen Häkchen wieder her. Beim [erneuten Vergleich](architecture/decisions/0036-native-win32-ui-role-evaluation.md#erneuter-vergleich-vor-dem-speichern) kann die Auswahl auf der aktualisierten Vorschau geprüft und geändert werden. Die [ausdrücklich bestätigte Übernahme über die Revisionslücke](architecture/decisions/0036-native-win32-ui-role-evaluation.md#bestätigte-übernahme-über-eine-revisionslücke) ist unter frischer Schreibbindung geprüft. Bei ausreichendem Budget ist auch [Weiterschreiben nach bestätigter Übernahme](architecture/decisions/0036-native-win32-ui-role-evaluation.md#weiterschreiben-nach-bestätigter-übernahme) im selben Fenster möglich. [Weitere Konflikte während der Bestätigung](architecture/decisions/0036-native-win32-ui-role-evaluation.md#weitere-konflikte-während-der-erneuten-bestätigung) lassen den Entwurf samt Auswahl erneut prüfen.

Der geplante [ETW-Netzvergleich des Editorstands](architecture/decisions/0036-native-win32-ui-role-evaluation.md#etw-beim-ersetzen-formatieren-und-wiederöffnen) ist seit dem 23. November abgeschlossen. Referenz, Bearbeitung und frisches Lesen waren in den gemessenen Prozessbäumen ohne Netzwerkereignisse.

Der Win32-Editor übernimmt beim Weiterbearbeiten auch die [unformatierte Fassung einer älteren synthetischen Szene](architecture/decisions/0036-native-win32-ui-role-evaluation.md#weiterbearbeiten-unformatierter-altbestände).


Im synthetischen Win32-Editor sind Rückgängig und Wiederholen über beide Absätze verfügbar. Der Verlauf bleibt auf das offene Fenster beschränkt.


Der [authentisierte Abschluss beim Verwerfen](architecture/decisions/0036-native-win32-ui-role-evaluation.md#authentisierter-abschluss-beim-verwerfen) gibt die synthetische Ablage nach regulärem Schließen wieder frei. Ein erzwungenes Prozessende hält sie gesperrt. Der [ETW-Netzvergleich](architecture/decisions/0036-native-win32-ui-role-evaluation.md#etw-beim-bestätigten-verwerfen-und-wiederöffnen) bestätigte null Netzwerkereignisse in den gemessenen Prozessbäumen beim Verwerfen und erneuten Lesen.


Der verwaltete Auftrag `edit-win32-scene` kann die [zuletzt gespeicherte markierte Szene weiterbearbeiten](architecture/decisions/0036-native-win32-ui-role-evaluation.md#gespeicherte-markierte-szene-weiterbearbeiten). Der Windows-Lauf bestätigte Revision 2 mit erhaltener Fettmarkierung und unveränderter Quelldatei auf Revision 1.


Beim verwalteten Szenenzugriff ist der [Erhalt einer Fettmarkierung beim Ändern des markierten Textes](architecture/decisions/0036-native-win32-ui-role-evaluation.md#textänderung-in-einem-markierten-absatz) bis zum Wiederöffnen verifiziert. Die [native Vorschau](architecture/decisions/0036-native-win32-ui-role-evaluation.md#formatierte-vorschau) zeigt die gespeicherte Fettmarkierung am zweiten Absatz an.


Der [Teilabbruch beim Speichern](architecture/decisions/0036-native-win32-ui-role-evaluation.md#unterbrechung-während-des-schreibens) ist geprüft. Die getrennte Basisdatei bewahrte Revision 0, während der neue Leseauftrag gesperrt blieb. Reguläres Speichern und Wiederöffnen von Revision 1 ist mit dem aktualisierten Rollenpaket bestätigt.

Der [ETW-Vergleich des regulären Speicher- und Lesepfads](architecture/decisions/0036-native-win32-ui-role-evaluation.md#etw-beim-dauerhaften-speichern-und-wiederöffnen) ergab für beide Aufträge null Netzwerkereignisse im jeweiligen Prozessbaum.

Beim verwalteten Speichern von Szene A sind die beiden [Datendienstabbrüche an der dauerhaften Speichergrenze](architecture/decisions/0036-native-win32-ui-role-evaluation.md#datendienstabbruch-an-der-dauerhaften-speichergrenze) geprüft. Auf der Platte blieben Revision 0 bzw. 1 erhalten; der offene Sitzungsbeleg sperrte beide Neustarts.

Diese Anleitung trennt den öffentlichen Quellcode vom privaten Schreibprojekt. GitHub transportiert
Code und Projektdokumentation; ein `.kraqel`-Archiv transportiert Manuskript, Struktur und lokale
Medien. Wer beides verwechselt, erhält entweder fehlende Kapitel oder einen sehr literarischen
Commitverlauf.

## Entwicklungsübergabe vom 19. November 2026

### Aktueller Einstieg und gesicherter Stand

Arbeitsbranch ist `feat/kq-069-live-file-selection`. Für die Fortsetzung im vorhandenen Checkout
diesen Branch auswählen und mit `git pull --ff-only` aktualisieren. Die lokalen Belege zur
Handleprüfung liegen unter `artifacts/bp3-offline-guest/result-series12/`, die frischen ETW-Spuren
und das Protokoll des gemessenen Handlelaufs unter `artifacts/bp3-offline-guest/result-etw12a/`. Die
früheren Gastbelege liegen unter `artifacts/bp3-offline-guest/result-restart09/` (Wiederanlauf) und
`result-series07/` im selben Verzeichnisbaum (Starterabbruch). Der Gast ist nach dem Lauf
heruntergefahren. [Draft-PR #142](https://github.com/kolkrabeofdoom/kraqel/pull/142).

Der Offline-Gastlauf vom 19. November ist bestanden. Bei erzwungener UMCI erreichte das
Kontrollprogramm mit statisch importierter erlaubter DLL den erwarteten Exit 50. Beim statischen
Import der fremden DLL beendete der Windows-Loader den Prozess mit `0xC0E90002`. Ereignis 3077 ist
dem Importprogramm, der Richtlinie und dem vollständigen DLL-Hash zugeordnet. Die beiden bisherigen
Blattrollen bestanden ebenfalls. Der Gast ist ausgeschaltet.
[Prüfbericht und Wiederholung](testing/product-start-p2c-bp3-umci-guest.md).

Der gezielte Abbruch des Starters bei laufendem Blattprozess ist im Offline-Gast bestätigt. Vor dem
Abbruch war genau ein Blattprozess im Zähljob aktiv. Am gehaltenen Prozesshandle wurde sein Ende
beobachtet. Danach meldete der Zähljob null aktive Prozesse bei einer Gesamtzahl von eins. Beide
Wiederanläufe des synthetischen Starts sind geprüft. Ohne Prüfrichtlinie (UMCI 0) und mit aktiver
Richtlinie derselben ID, aber abweichendem CIP-Hash (UMCI 2), sperrte der reguläre Einstieg vor dem
Starter. Zwei Prüffenster nach Gastneustarts mit je zwei Kontrollprozessen belegen null
Starterstarts. Vor Resume ist für die synthetischen Blattrollen die Bindung an das offene
Markerobjekt geprüft: Leserolle exakt `FILE_GENERIC_READ`, leere Rolle kein Handle auf dieses
Objekt, reine Attributrechte als Leserolle vor Resume abgewiesen. Der frische ETW-Vergleich mit
demselben Prüfpaket ergab null TCP-/UDP-Ereignisse in beiden Prozessbäumen. Alle vier
Loopback-Kontrollen waren in jedem Messfenster sichtbar. Die Rollenprüfung des echten
WebView2-Kandidaten ergab sechs Runtimegenerationen mit einem WebView2-Prozess als Parent, darunter
ein verschachtelter Crashpad-Start. Im eingebundenen Win32-SDK fehlt ein belegter Freigabepunkt vor
jedem Kindstart. Auch Microsofts öffentliches stabiles SDK und Vorschau-SDK dokumentieren kein
Startveto für verschachtelte Crashpad-Prozesse oder spätere Renderer. C4-Start und BP4-Datenzugriff
bleiben bis zum Nachweis gemäß ADR 0035 gesperrt.
[ADR 0035](architecture/decisions/0035-bp3-independent-process-and-image-enforcement.md).

Das isolierte native Win32-Fenster ist im interaktiven Offline-Gast mit synthetischem Text geprüft.
Direktstart und falsche Rolle wurden gesperrt. Tastatureingabe, Fokuswechsel, Narrator-Ansagen im
lokalen Speech Recap und reguläres Prozessende sind bestätigt. ETW ergab null TCP-/UDP-Ereignisse in
beiden Messbäumen bei je vier Positivkontrollen. Für diese Fensterprobe gilt die softwareseitige
Sprachausgabe als bestanden. Die akustische Hörprüfung ist zurückgestellt.
[Prüfbericht und Wiederholung](testing/native-win32-window-guest.md).

Für BP4 liegt der
[Vertrag für synthetische Win32-Szenen](data-format/native-win32-bp4-scene-v1-draft.md) vor. BP4a
ist als eigenständiges Rust-Forschungsmodell geprüft. Eine exklusive Bindung begrenzt den Zugriff
auf eine Katalogszene. Leseaufträge, Revisionskonflikte und ungültige Dokumente sind gegen
unbeabsichtigtes Überschreiben getestet. Der
[BP4b-Kanalvertrag](data-format/native-win32-bp4-channel-v1-draft.md) legt Rollenbindung und
Replayabwehr fest. Framecodec und Replayabwehr sind im reinen Modell geprüft. Die
[Pipe-Prüfung im Offline-Gast](testing/native-win32-pipe-rights.md) bestätigt vor Resume die
Endpunktrechte und die Zugriffssperre für den geprüften Fremdprozess. Auch der
[native HELLO/READY-Lauf](testing/native-win32-hello-ready.md) ist bestanden, einschließlich der
Abweisung eines falsch authentisierten READY. Der
[native Leseabruf](testing/native-win32-scene-read.md) von A im Lesemodus ist bis zum
schreibgeschützten Win32-Feld bestätigt. Der
[Schreibversuch unter derselben Lesebindung](testing/native-win32-denied-save.md) ergab DENIED. Die
nächste Leseantwort bestätigte Text und Revision 0. Der
[Edit-Gastlauf](testing/native-win32-edit-conflict.md) bestätigte Revision 1, CONFLICT beim zweiten
Schreibversuch mit Revision 0 und den unveränderten Endtext. Der
[SQLite-Gastlauf](testing/native-win32-durable-reopen.md) bestätigt Text und Revision 1 nach
Prozessneustart unter frischer Lesebindung sowie unabhängig auf der ausgeschalteten Gastplatte.
Der [Datendienstabbruch an der Commitgrenze](testing/native-win32-commit-crashes.md) ist im Gast geprüft.
Vor Commit bleibt Revision 0 erhalten, nach Commit vor Antwort Revision 1. Der frische ETW-Netzvergleich beider Pfade ergab null Netzwerkereignisse im Prüfprozessbaum.
[Teilübertragung und 30-Sekunden-Timeout](testing/native-win32-channel-interruptions.md) sind im Gast bestätigt.
Beide endeten ohne Schreibantwort und mit Revision 0. [Fensterende vor Commit und ETW](testing/native-win32-window-close.md) sind für den neuen Build geprüft.
Dienstende und Revision 0 sind bestätigt, im Prüfprozessbaum wurden null Netzwerkereignisse gemessen. Der [Wiederanlauf nach Starterabbruch](testing/native-win32-starter-restart.md) blieb dreimal gesperrt.
Der frische ETW-Netzvergleich bestätigte null Netzwerkereignisse im Prüfprozessbaum. Alle drei gesperrten Starter endeten ohne Kindprozess.
**BP4b ist für den synthetischen Win32-Kanal abgeschlossen.** Der [Abschluss mit Nachweisen](data-format/native-win32-bp4-channel-v1-draft.md#abschluss-des-synthetischen-datenkanals) umfasst Rechte, Revisionen, Transaktionsende und die Sperre der verbrauchten Einmalfreigabe bis zum Kaltstart. C4b bis C4d sind für das schreibgeschützte synthetische Win32-Fenster abgeschlossen. Der verwaltete Starter bindet Paket und Prozessgeneration unter Kontrollsperre und gibt das Fenster nach bestätigtem Bootstrap frei. Nach Starterabbruch blockiert der offene Sitzungsbeleg die nächste Fenstererzeugung. Die [bytegebundene C4d-Abnahme](architecture/decisions/0036-native-win32-ui-role-evaluation.md#c4d-abschluss-für-den-synthetischen-vorschaupfad) enthält Paketidentität, Sicherheitsnachweise und Freigabeumfang. Der neue Einstieg `broker-store open-win32-scene <Kontrollkopf>` zeigt nach beiden Bootstrap-Quittungen genau eine vom isolierten Datendienst gelesene Szene A schreibgeschützt an. Der ETW-Netzvergleich erfasste im Referenzlauf und beim einmaligen Szenenabruf je null Netzwerkereignisse im KRAQEL-Prozessbaum. Nur im Szenenlauf startete der Datendienst. Der anschließende verwaltete Prüflauf `check-win32-scene-denied-save` bestätigte `DENIED` für einen gültigen Speicherversuch und unveränderten Text auf Revision 0 beim erneuten Lesen unter derselben Bindung. Unter eigener Edit-Bindung bestätigte `check-win32-scene-edit-conflict` die erste Speicherung, `CONFLICT` für den veralteten Schreibversuch und den gespeicherten Text auf Revision 1 im Arbeitsspeicher. Der dauerhafte Prüfpfad `save-win32-scene` und `read-saved-win32-scene` bestätigte nach Prozessneustart Text und Revision 1 im isolierten SQLite-Abbild; veränderte Dateibytes sperren das Wiederöffnen vor dem Rollenstart. Produktdaten und WebView2 bleiben gesperrt.

Der Host-Abgleich für die Hörprüfung ergab einen aktiven hostseitigen Wiedergabeendpunkt. Für den
isolierten Gast ist kein eigener Ausgang belegt, ein separater Prüfplatz war nicht verfügbar. Die
akustische Prüfung erst bei verfügbarem isoliertem Wiedergabeweg und anwesender Hörperson wieder
aufnehmen.

Vorheriger Stand · BP3 synthetische Debug-Wächter-Gegenprobe, 18. November 2026:

[Debug-Wächter-Nachweis](testing/product-start-p2c-bp3-debug-guardian.md): Root beim Create-Ereignis
vor User-Mode-Ausführung abgewiesen, normale Kindvererbung bestätigt. Ein Kind mit eigenem
DEBUG_PROCESS läuft dagegen außerhalb der äußeren Debugkette; dieselbe Kindgeneration vorher vom
Host an gehaltenem Handle, Token, Profil und Job geprüft. Kein AppContainer-/Jobausbruch. Drei
native Fälle, neun Jobprozesse (fünf Worker/vier Konsolenhosts), sieben äußere Create-/Exit-Paare
und zwei innere Create-Berichte; alle eigenen Jobs leer, Profile/Fixtures entfernt. **Debugvererbung
allein ungeeignet für BP3.** Drei neue aktive Tests/141 Helfer-/Transporttests insgesamt und Clippy
grün, sieben standardmäßig ignoriert. Kein frisches ETW oder tatsächlicher Wächter-Crash. C4-Sperre
und native UI-ABI 0 unverändert; keine Runtime-/Datenautorität. BP3/BP4/C4c/d weiter offen. Nächster
begrenzter Scope: Architekturentscheid für unabhängig durchgesetzte Erzeugung samt früher
Wächter-/Ladebindung, keine weiteren Runtime-Versuche auf Basis bloßer Debugvererbung. Mittleres
Prüfniveau und striktes Datenschutzziel unverändert; Gesamt grob 65 %, nutzbarer 0.1-Umfang grob 80
%.

Arbeitsbranch `feat/kq-069-live-file-selection`, Draft-PR #142 unverändert. Ausgangshead `fa894f7`
bei Beginn sauber, CI Windows/Linux/Dependency Review vollständig grün. Neue Head-CI separat prüfen;
kein Merge/Release.

Exaktes neues Paar unter `artifacts/bp3-debug-20261118-01/` geschützt außerhalb von TEMP gesichert:
normal `56a1b43…`, Testcontroller `e154373…`, privater `run.json`, unabhängiger Audit und Manifest
mit Quellhashes. Originalausgabe `%TEMP%/kraqel-bp3-debug-build-20261118-01/`. Roh-PIDs, Zeiten,
Nonces und Pfade bleiben lokal; keine MAC-/Kontrollschlüssel oder echte Projekte in der Probe. Keine
fremden System-/Profilartefakte entfernt. Fünf eigene Entwicklungsreste aus fehlgeschlagenen
Vorversuchen nach Ende aller Testprozesse im exakt geprüften TEMP-Bereich entfernt; keine
Produktbereinigung. Keine eigenen Prozesse/Profile/Fixtures mehr aktiv, kein neues ETW oder UAC
nötig.

Eine neue Sitzung darf aus der gelungenen normalen Debugvererbung keine universelle Startsperre
ableiten: Die Gegenprobe widerlegt das gerade. Der Kindmarker wurde mit eigener Debugkette im selben
AppContainer/Job ausgeführt. Kein DLL-/TLS-/JIT- oder Wächter-Crashnachweis daraus. Zuerst
unabhängigen Erzeugungs-/Lade-/Wächteranker bewerten, ohne Hersteller-/Signaturausnahme,
Schutzsoftwareänderung oder automatische UI-/VM-/Treiberumstellung. Normale App und C4-Sperre
unverändert. Geänderte Bytes haben weiterhin keinen frischen Netzbeleg.

### Vorherige Übergabe: BP3-Startsperre (historisch)

Vorheriger Stand · BP3-Machbarkeitsgrenze und native Startsperre, 18. November 2026:

[BP3-Nachweis](testing/product-start-p2c-bp3-admission.md): Für den aktuellen C4-Kandidaten ist noch
kein vollständiger präventiver Modul-/WebView-Rollenmechanismus nachgewiesen. Der native Controller
sperrt jetzt vor ACL-/Ressourcenanlage, Commit und CreateProcess; Worker und Runtime-Einstieg
sperren zusätzlich vor Handleübernahme bzw. COM/WebView. Keine konfigurierbare Ausnahme oder
Umdeutung von Paketpins, BP1/BP2 oder Diagnosen. Zwei aktive Regressionen, 138 aktive
Helfer-/Transporttests und Clippy grün; sechs standardmäßig ignoriert. Drei direkte Workeraufrufe
enden mit 2 ohne Ausgabe/Dateianlage. Kein neues ETW; frühere Messungen bleiben bytegebunden. **BP3
insgesamt offen**, keine Runtime-/Datenfreigabe, BP4/C4c/d weiter gesperrt. Nächster begrenzter
Scope: separate synthetische Debug-Wächter-Probe mit ausdrücklicher Debugketten-Gegenprobe; noch
nicht implementiert. Keine System-/Schutzsoftware-/Signaturausnahme. Mittleres Prüfniveau und
striktes Datenschutzziel unverändert; Gesamt grob 65 %, nutzbarer 0.1-Umfang grob 80 %.

Arbeitsbranch unverändert `feat/kq-069-live-file-selection`, Draft-PR #142. Ausgangscommit dieser
Sitzung `3772c0c`; dessen Windows-/Linux-CI beim letzten Abgleich noch laufend, Dependency Review
grün. Neue Head-CI gesondert prüfen, kein Merge/Release. Checkout zu Beginn sauber.

Neuer gesicherter, noch nicht ETW-gemessener Helfer unter `artifacts/bp3-admission-20261118-01/`:
normal `04e38c2…`, Testcontroller `503fdbc…`, Manifest mit Quellhashes und drei direkten
Abweisungen. Keine echte Kontrollablage oder alte Auditkopie übernehmen. Bei Rechnerwechsel
Programme lokal neu bauen/prüfen; historische Netzbelege nicht auf neue Bytes übertragen. Die
C4-Beobachtungsserie ist im neuen Build ausdrücklich nicht mehr aufrufbar, auch nicht mit einer
alten Bootstrapnachricht. Der BP2-Blattzweck bleibt separat.

Die Machbarkeitsprobe des Debug-Wächters muss zuerst die dokumentierte Unterbrechung der
Debugvererbung durch einen eigenen DEBUG_PROCESS-Nachfahren untersuchen. Bei fehlendem Nachweis
weiter sperren; keinen Runtime-/Datenstart, keine Signatur-/Herstellerausnahme oder AV-Änderung.

### Vorherige BP2-Übergabe (historisch)

Arbeitsbranch **`feat/kq-069-live-file-selection`**,
[Draft-PR #142](https://github.com/kolkrabeofdoom/kraqel/pull/142). `git pull --ff-only` zu Beginn
dieser Sitzung: aktuell/sauber. CI des vorherigen Übergabeheads `b881d59` vollständig grün
(Windows/Linux/Dependency Review); neue Head-CI gesondert prüfen. Kein Merge, Release oder
Produktstart daraus.

Vorheriger Stand · BP2 nativer synthetischer Blattversuch samt ETW, 18. November 2026:

[BP2-Nachweis](testing/product-start-p2c-bp2-native-leaf.md): unabhängige native Supportprüfung,
NoRemote-/NoLowLabel-/Child-Policy atomar bei suspendierter Erstellung; Readback vor Resume und im
Blattprozess. Kindstart mit Windows-Fehler 367 abgewiesen, Jobbudget zwei zur Abgrenzung von
Quotenfehlern, tatsächlich nur eine Generation je Job. Unsupported vor Create/Headänderung,
abweichende Sollerwartung vor Resume und falsche MAC vor Accepted geprüft. Eigener BP2-Zweck,
MAC-Frames/Records unter derselben Kontrollsperre, Ergebnis-Hashes in Endrecords gebunden. Je
funktionaler/gemessener Serie 18 MAC-Records/14 BP2-Records/drei verbrauchte Launches geprüft,
Profile/Fixtures entfernt. Frisches ETW des Paars `5205d72…`/`62d7a73…`: neun explizite Enden, acht
Kontrollen, keine Verluste/Zuordnungslücken oder beobachtetes Netz in diesen Bäumen. Drei neue
aktive Tests/136 insgesamt bestanden, sieben bewusst ignoriert, Clippy/rustfmt und 27
Auswertungstests grün. Kein App-/Transport-/Installerumbau, native UI-ABI bleibt 0. **Keine
Runtime-/Datenautorität:** BP1-Modell ist kein nativer Grant, BP2 keine vollständige
Modul-/Rollenattestierung. **Nächster Scope BP3:** präventiven Wächter-/Lade-/WebView-Erzeugungsweg
und Machbarkeit klären; Kindstartsperre nicht für Kompatibilität abschalten, keine Hersteller-/
Signaturausnahme oder Schutzsoftwareänderung. BP4 Datenrechte/Commands und C4b insgesamt/C4c/d,
P2/P3–P5 und Produkt-Q-01/Q-02 bleiben offen. Mittleres Prüfniveau und striktes Datenschutzziel
unverändert; Gesamt grob 65 %, nutzbarer 0.1-Umfang grob 80 %.

Nächster begrenzter Scope **BP3**: unterstützten präventiven Wächter-/Lade-/WebView-
Mehrprozess-Erzeugungsweg und Machbarkeit konkretisieren. NoRemote/NoLowLabel sind keine exakte
Modulpositivliste; ChildPolicy Restricted ist kein Rollenfilter und darf nicht einfach abgeschaltet
werden. Vollständige Wächter-Startkette vor produktiven Schlüsseln, dynamische Rollen und spätere
Nachladungen bleiben offen. Keine Hersteller-/Signaturausnahme, Schutzsoftwareänderung oder neue
Text-/Commandfreigabe. BP4 und C4c/d erst nach ihren jeweiligen nativen Gates.

Aktuelles exaktes Paar `%TEMP%/kraqel-bp2-build-20261118-01/`: normaler Helfer/Blattcode `5205d72…`,
tatsächlicher Host-Testcontroller `62d7a73…`. `run-functional.json` und gemessener `run.json`,
getrennte private MAC-Audits, unabhängiger Audit und Manifest. Rohdaten unter
`%TEMP%/kraqel-bp2-etw-20261118-01-{reference,gate}/`, geschützte zusätzliche Sicherung
`artifacts/bp2-native-20261118-01/` außerhalb von TEMP. Keys/PIDs/SIDs/Pfade bleiben lokal; keine
Auditkopie als aktiven Store oder wiederverwendbaren Grant verwenden.

BP2 ist ein eigener Forschungszweck, neue native `bp2LeafProbe`-Action/Eventversion 1 im
Schema-6-Forschungsreader. Keine bestehende echte Kontrollablage adoptieren/migrieren; alte Reader
weisen unbekannte Action ab. UI-ABI weiter 0. Eigener kurzlebiger Pipe-Schlüssel; keine permanente
Kontrollautorität im Blatt. Modi zum Unsupported-/Sollwertvergleich nur intern beim ignorierten
Testcontroller, kein öffentlicher Host-CLI-Aufruf. Normaler Workerzweig validiert Bootstrap vor
seinem begrenzten Versuch und fällt niemals in generische Netzwerkdiagnostik.

Der Diagnosejob erlaubt zwei Plätze, um Fehler 367 von Quotenablehnung abzugrenzen; beobachtet je
Job genau ein Blatt und kein Kind. Das ist kein BP1-zu-Native-Grant oder Produktbudget. Ergebnis-
Hashes sind MAC-Record-gebunden. Drei gestartete Aufträge verbraucht, echter Store-Wiederanlauf ohne
Pending bestätigt, Profile/Fixtures entfernt; beide Erfassungen gestoppt/konvertiert und Supervisor
beendet. Neue Programmbytes brauchen wieder eigenen Netzbeleg. Mittleres Prüfniveau.

Vorheriger unveränderter Sperrstand:

Exaktes Paar weiter `%TEMP%/kraqel-c4b-module-gate-20261118-03/`: Root `daf59d0…`,
Host-Testcontroller `83679e5…`, funktionaler Bericht `run-functional.json`, neuer gemessener Bericht
`run.json`, getrennte private MAC-Audits, `etw-independent-audit.json`. Frische Rohmessungen unter
`%TEMP%/kraqel-c4b-gate-etw-20261118-01-{reference,gate}/`, geschützte zusätzliche Sicherung
`artifacts/c4b-module-gate-etw-20261118-01/` außerhalb von TEMP einschließlich exaktem Paar,
ETL/XML, Audit und Hashmanifest. Keys/PIDs/Pfade/Kommandozeilen bleiben lokal; keine Auditkopie als
aktiven Store oder wiederverwendbaren Grant übernehmen. Alte Nachweise/Bytepaare unverändert.

Runtimequelle weiter `%TEMP%/kraqel-c4-runtime-20261117-01/expanded/`; CAB-Sicherung unverändert
unter `artifacts/session-close-20261117-01/`. Bei TEMP-Verlust eigenes gesichertes CAB entpacken und
Manifesthashes prüfen. Keine echten Projekte/echte Vorschauablage starten und Pending nicht
reparieren. BP1 hat Modell-/Helfertests und Clippy ausgeführt; keine neuen GUI-/Runtimeversuche oder
ETW-Läufe.

Gemessene Prozesse beendet, Testprofil/Daten-/Kontrollfixtures entfernt; beide eigenen Erfassungen
gestoppt/konvertiert und eigener Supervisor beendet. Bestehende Systemerfassungen nicht angefasst.
Mittleres Prüfniveau und striktes Datenschutzziel beibehalten. Neue Programmbytes benötigen einen
eigenen frischen Netzbeleg; diese Messung erteilt keine Runtime-/Produktfreigabe.

Vorheriger nativer Netzbeleg · C3, 17. November 2026:

[Gezieltes C3-ETW](testing/product-start-p2c-group-bootstrap-etw.md) des unveränderten Builds
`b24b3aa…` ist ergänzt: normaler Abschluss und Gruppenfehler/gesperrter Wiederanlauf. 28 Controller,
sieben Root-/fünf Kindrollen, 28 Konsolenhosts; alle 68 Enden explizit. Acht lokale Kontrollen,
Nullverluste/ Zuordnungslücken und null beobachtete Netzereignisse in diesen Bäumen einschließlich
Loopback. Alle 28 nativen Controller-Erzeugungszeiten sowie sechs Root-/fünf Kindbindungen exakt mit
Kernel abgeglichen; Root-Ende vor Kindende in allen fünf gebundenen Fällen. 66 MAC-Records/sieben
verbrauchte Launches/sechs Inventare unabhängig geprüft; drei Profile entfernt (zwei Broker-/ein
eigener Fixturecleanup), negativer Store weiter pending, echte Kontrollhashes unverändert. 42
Diagnose-Tests grün, kein neuer Build/Runtimeumbau oder Produktfreigabe. Native UI-ABI bleibt 0;
Gruppen-MAC ist keine unabhängige Renderer-/Schreiber-PID-Autorität. Der inzwischen ergänzte
C4-Vertrag steht oben; keine automatische Übertragung auf WebView. P2 insgesamt/P3–P5 und
Produkt-Q-01/Q-02 bleiben offen. Gesamt grob 65 % (60–70 %), nutzbarer 0.1-Funktionsumfang grob 80
%; keine Zeit-/Produktionsreifequote. Striktes Datenschutzziel unverändert.

Gesicherter Controller: `%TEMP%/kraqel-product-p2c-group-build-20261117-01/controller.exe`, SHA-256
`b24b3aac3d80bc59b63c74e2cbad03bf4980d8496b88434f3afc035fdbbe48af`, Runtimequellstand `6e2c089`.
Private Captures `%TEMP%/kraqel-c3-etw-20261117-01-reference` und `…-group` konvertiert;
Supervisor/Testprozesse beendet, drei Profile entfernt. Audit/Manifest beim Build ergänzt,
synthetische Daten/Kontrollketten lokal erhalten. Ein negativer Store bleibt pending: nicht
reparieren oder fehlendes Profil als Startfreigabe auslegen. Eigener Fixturecleanup war keine
Broker-Reconciliation. Rohdaten/Keys/PIDs nicht nach GitHub oder neue Programme an echter
Vorschauablage autorisieren. Für C4 Paketclosure/Runtime-/Command-/Kindrollenvertrag lesen;
C3-Gruppenschlüssel nicht zur Rendererautorität machen. Ältere Netzbelege bleiben bytegebunden.

Vorheriger Implementierungs-/Netznachweis:

[Frischer Workerbindungs-/Wiederanlauf-Netzbeleg](testing/product-start-worker-lifetime-etw.md).
Frisches Workerbindungs-v3-ETW, 17. November: unveränderter Build `37d7bec…`, normale Referenz und
vier Wiederanlaufpfade. 52 Controller/acht Worker/52 Controller-Konsolenhosts, alle 112 Enden
explizit; acht Kontrollen, Nullverluste/Zuordnungslücken, null beobachtete Netzereignisse in den
ausgewählten Bäumen. Alle acht nativen Erzeugungszeiten zum Kernel abgeglichen, sieben dauerhafte
Bindungen und drei Endrecords nach bestätigtem Kernelende zugeordnet. Exit 0 ersetzt keinen
fachlichen Abschluss: vier Finish-Records, drei reconciliierte Starts, ein ungebundener Start weiter
pending/gesperrt. Daten/MAC-/Paket-/Root-/Profilaudit und 42 Diagnose-Tests grün, echte Daten und
Schlüssel unverändert. Kein neuer Build/App-/Transport-/Installerumbau oder pnpm check; Rohdaten nur
lokal. Der damals nächste Vertragsumfang ist inzwischen als P2c-Entwurf dokumentiert; die Umsetzung
bleibt offen. Ein-Prozess-Beleg nicht auf WebView2 übertragen, private Fixtures nicht promovieren;
ungebundene Starts/unbestätigte Profile und P2 insgesamt/P3–P5/Produkt-Q-01/Q-02 offen, striktes
Ziel unverändert.

Gesicherter Runtimebuild bleibt `%TEMP%/kraqel-product-worker-lifetime-build-20261117-01`,
Quellstand `03e4ed1`. Lokale Captures `%TEMP%/kraqel-worker-v3-etw-20261117-01-reference` und
`…-recovery`, zusammengefasster Audit und ergänztes Manifest bei der Buildsicherung. Traces und
Testprozesse beendet, Profile aufgeräumt; ein negativer Kontrollstore bleibt absichtlich gesperrt.
Keine Rohdaten nach GitHub oder neue Programme an echter Vorschauablage autorisieren. Für die
nächste Umsetzung den oben verlinkten P2c-Vertrag lesen; keine UI-ABI-0-Kopie oder
Ein-Prozess-Lebenszeitregel stillschweigend als Produktfreigabe nutzen.

Vorheriger Stand:
[Dauerhafte Workerbindung und Wiederanlauf](testing/product-start-worker-lifetime.md). Dauerhafte
P2b-Workerbindung, 17. November: Execution v3/Bereinigungs-ABI 3, sechs Recordslots reserviert.
WorkerBound bindet Launch, native PID/Erzeugungszeit und geprüfte Ein-Prozess-Jobpolicy vor Resume;
enger nativer Lease-/Job-/Token-Wächter erlaubt nur diesen Record bei aktivem Worker, andere
Storemutationen behalten Jobleerstand. Wiederanlauf vor Drained benötigt diesen MAC-Beleg und
tatsächliches Ende derselben Prozessgeneration: signalisiertes Handle, andere Erzeugungszeit oder
PID-Abwesenheit mit vollständigem Snapshot; keine fremde Terminierung, Fehler bleiben gesperrt.
RecoveredDrain wird vor eigenem Profilcleanup gespeichert. Vier native Crashfälle: drei gebundene
Starts sicher reconciliert und neue Generation erfolgreich; Start vor WorkerBound bleibt auch nach
separatem eigenem Fixturecleanup pending/gesperrt. 47 Controller/sieben Worker, 63 Records, sieben
verbrauchte Launches, 42 private Dateisperren; sieben Profile am Ende abwesend. 75 aktive
Helfer-/Transporttests/Clippy und unabhängiger MAC-/PE-/Root-/Inventar-/Profilaudit grün, echte
Daten und Schlüssel unverändert. Neuer gesicherter Runtimehash `37d7bec…`, kein
App-/Transport-/Installer- umbau oder pnpm check. Nächster Scope: gezieltes frisches ETW dieses
Controllers samt Worker-/ Lebenszeit-/Recoverypfaden. Voriger Netzbeleg bleibt an `9021872…`
gebunden. Ein-Prozess-Prototyp ist keine mehrprozessige GUI; ungebundene Starts/unbestätigte
Profile, persistente echte Arbeitsablagen-/UI-Bindung, P2 insgesamt/P3–P5 und Produkt-Q-01/Q-02
offen; striktes Ziel unverändert.

Gesicherter neuer Build `%TEMP%/kraqel-product-worker-lifetime-build-20261117-01`,
`functional-root.txt`, privater Audit/Manifest. Reproduktion mit
`check-workspace-worker-crashes.ps1` und expliziten Hashes aus dem Nachweis. Ein negativer
synthetischer Store bleibt absichtlich pending: kein nachträgliches Binden, kein Schlüssel-/Kopf-
Reset nach Fixturecleanup. Echte Vorschauablage nicht öffnen, übernehmen oder für neue EXE
autorisieren. V2-Build/ETW getrennt unverändert behalten. Kein Übertrag der Ein-Prozess-Regel auf
WebView2/mehrprozessige GUI und kein Importticket als Startautorität.

Vorheriger Stand:
[Frischer Ressourcen-Reconciliation-Netzbeleg](testing/product-start-resource-reconciliation-etw.md).
Frisches Ressourcen-ABI-2-ETW, 17. November: unveränderter Runtimebuild `9021872…`, normaler
Workerabschluss als Referenz gegenüber vier gezielten Ressourcenabbrüchen/Reconciliation. 54 direkte
Controller, sechs Worker, 54 Controller-Konsolenhosts, alle 114 Enden/Eltern/Exitcodes explizit
abgeglichen. Acht lokale TCP-/UDP-IPv4/IPv6-Kontrollen, Nullverluste/Zuordnungslücken, null
beobachtete Netzereignisse in den ausgewählten Bäumen (auch null Loopback). Daten/MACs/
Paket-/Rootbindung und Profile unabhängig geprüft: 67 Records, neun verbrauchte Launches, 36 private
Dateisperren, acht Profile am Ende abwesend. Sieben Broker-Cleanups; die eine eigene mehrdeutige
Fixture bleibt ausdrücklich separate Prüfablaufbereinigung, keine Produktfreigabe. 42 Diagnose-Tests
grün, kein neuer Build/Runtime-/App-/Transport-/Installerumbau oder pnpm check. Rohdaten nur lokal,
echte Vorschauablage unverändert. Nächster Scope: dauerhafter Workerstartbeleg und gesicherter
Wiederanlauf bei Controller-Crash zwischen RunIntent und Drained; bis belegtem Lebenszeitende weiter
gesperrt. P2 insgesamt/P3–P5 und Produkt-Q-01/Q-02 offen, striktes Ziel unverändert; Beleg gilt nur
für diese Bytes/Lebenszeiten/Pfade.

Build weiterhin `%TEMP%/kraqel-product-resource-crashes-build-20261117-01`, Runtimequellstand
`1d24762`. Frische lokale Captures `%TEMP%/kraqel-resource-v2-etw-20261117-01-reference` und
`…-reconciliation`; zusammengefasster Audit und ergänztes Manifest bei der Buildsicherung. Alle
eigenen Traces und Testprozesse beendet, keine Rohdaten nach GitHub. Keine Profilreste adoptieren
oder neue Brokerprogramme an echter Vorschauablage autorisieren. Vor Live-Crash-Recovery
Authentizität und dauerhafte Lebenszeitbindung planen/prüfen; verschwundener Jobname und unsignierte
Logs ersetzen keinen Beleg. Vorherige Implementierungsabschnitte sind historisch.

Vorheriger Stand:
[Begrenzte Ressourcen-Reconciliation](testing/product-start-resource-reconciliation.md).
P2b-Ressourcen-Reconciliation, 17. November: Execution v2/Profilbereinigungs-ABI 2 mit neuen
RunIntent-/Drained-Records, fünf Slots vor Ressourcenanlage reserviert. Unter derselben Lease werden
fehlende unbestätigte Profile ohne Löschung abgeschlossen und bestätigte Profile nur ohne RunIntent
oder nach dauerhaft belegtem Drain bereinigt. Exakte begrenzte HKCU-Mapping-/Moniker- Abfrage, keine
Registryänderung; Fehler bleiben gesperrt. Bestehendes Profil ohne ProfileReady oder RunIntent ohne
Drained wird nicht adoptiert/gelöscht. Vier native Abbruchfälle, 49 Controller, fünf erfolgreiche
Worker, 30 private Dateisperren; sieben Profile erzeugt, sechs Broker-Cleanups, eine nachweislich
eigene mehrdeutige Fixture ausschließlich durch den Prüfablauf separat entfernt. 73 aktive
Helfer-/Transporttests/Clippy und unabhängiger MAC-/PE-/Root-/Inventar-/Profilaudit grün; acht
verbrauchte Launches/56 Records, Daten/Schlüssel/echte Vorschauablage unverändert. Neuer Runtimehash
`9021872…`, ein Helferbuild, keine App-/Transport-/Installeränderung oder pnpm check. Nächster
Scope: gezielter frischer ETW-Vergleich dieses Builds mit Worker/Reconciliation. Voriger Netzbeleg
bleibt ausschließlich an `590c3d4…` gebunden. Vollständige Live-Worker-Crashbereinigung, persistente
echte Arbeitsablagen-/UI-Bindung, P2 insgesamt/P3–P5 und Produkt-Q-01/Q-02 offen; striktes
Datenschutzziel unverändert.

Gesicherter neuer Build: `%TEMP%/kraqel-product-resource-crashes-build-20261117-01` mit
`functional-root.txt`, privatem Audit und Buildmanifest. Reproduktion ausschließlich mit
`check-workspace-resource-crashes.ps1` und expliziten Hashes aus dem Nachweis. Keine alten
Execution-v1-Records migrieren oder neue Programme an echter Vorschauablage autorisieren. Vorige
Builds und ETW-Rohdaten getrennt unverändert behalten. RunIntent ohne Drained bleibt gesperrt; keine
Schlussfolgerung aus verschwundenem Jobnamen oder unsignierten Diagnosen.

Vorheriger Stand: [Frischer P2b-Netzbeleg](testing/product-start-p2b-bootstrap-etw.md). Frisches
P2b-ETW, 17. November: unveränderter gesicherter Runtimebuild `590c3d4…`, zwei kurze gültige
Messungen – Kontrollablagenreferenz und drei gezielte Workerfälle. 31 Controller, sechs Worker, 31
direkt zugeordnete Controller-Konsolenhosts, alle 68 Enden explizit. Acht lokale
TCP-/UDP-IPv4/IPv6-Kontrollen, Nullverluste/Zuordnungslücken, null beobachtete Netzereignisse in den
gemessenen Bäumen (auch null Loopback). Worker normal/MAC-Ablehnung/Sperre vor Resume und neue
Generationen mit Replay/Kopfabwehr; native Daten/MACs, Token/Job/Lease und sechs entfernte Profile
unabhängig bestätigt. Rohdaten nur lokal, echte Vorschauablage unverändert. 42 Diagnose-Tests;
zusätzlich reinen Modelltest für Linux-Clippy korrigiert (Accessor statt privater Set-Zugriff), acht
gezielte Kernprüfungen/Clippy grün. Keine Runtimeänderung/neuer Build,
App-/Transport-/Installerumbau oder Workspace-Gesamtlauf. Nächster Scope: gesicherte
Ressourcen-Crash-Reconciliation der P2b-Probeprofile, insbesondere
Profilerzeugung/ProfileReady/Entfernung/Abschlussrecord. Keine Adoption unklarer Profile oder
Bestandsdaten; P2 insgesamt, P3–P5 und Produkt-Q-01/Q-02 offen. Striktes Datenschutzziel
unverändert, keine Übertragung auf normale App oder ungemessene Pfade.

Gesicherter Runtimebuild bleibt `kraqel-product-start-p2b-build-20261117-05`, Runtimequellstand
`4c26ba4`. Hashidentische Messkopie `etw-package/kraqel-isolation-probe.exe`; sechs separat kopierte
`worker.exe` durch private Records/Hash/Kernelstarts abgeglichen. Lokale Capturepfade
`%TEMP%/kraqel-p2b-etw-20261117-01-reference` und `…-worker`, keine Rohdaten nach GitHub. Kein neuer
Runtimebuild für die reine Test-/Dokumentationskorrektur. Nächste Arbeit gezielt an
P2b-Ressourcencrashes; echte `.kraqel-broker-preview` weiterhin nicht öffnen, zurücksetzen oder neue
Brokerhashes dort autorisieren. Frühere Abschnitte beschreiben historische Gates.

Vorheriger Stand: [P2b · synthetischer Bootstrap](testing/product-start-p2b-bootstrap.md).
Produktstart P2b, 17. November: authentisierter synthetischer Workerbootstrap unter derselben
Kontrollsperre, ohne GUI oder Importtask. Separate frische Probeprofile/-daten, keine ACL-Promotion
der privaten P2a-Fixture. Commit/ProfileReady/Receiptabschluss bzw. kontrollierte Reconciliation
persistiert; exakte Token-/Job-/Leaseprüfung vor Resume, eigener Einmalschlüssel statt
Kontrollschlüssel, begrenzte Pipes mit monotoner Receiptfrist. Jobleerstand am gehaltenen Objekt vor
Profilentfernung, unklare Crashressourcen bleiben gesperrt; keine automatische Crashbereinigung.
`.kqboot` Probe-ABI 1/UI-ABI 0, Legacy-/P2a-Marker unverändert. Vier neue Tests, 71 aktive
Helfer-/Transporttests, gezielte Nachprüfung/Clippy grün. Drei finale native Fälle, 27
Kontrollprozesse, sechs Worker (vier Erfolge, eine MAC-Ablehnung, eine Sperre vor Resume), 30
private Zugriffssperren, sechs Probeprofile entfernt; unabhängiger
MAC-/PE-/Root-/Prozess-/Profilaudit grün. Echte Daten und Vorschauablage unverändert. Finaler
Broker/Worker `590c3d4…`, Sicherung `kraqel-product-start-p2b-build-20261117-05`, mehrere vorherige
Entwicklungsbuilds nicht angenommen. Mittleres Prüfniveau ohne App-/Transport-/Installerumbau oder
`pnpm check`. Nächster Scope: gezielter frischer ETW-Netzvergleich dieses Controllers samt separatem
Workerpfad und beiden Ablehnungen. Alter ETW-Beleg nicht übertragbar. Danach
Ressourcen-Crash-Reconciliation und echte persistente Arbeitsablagen-/UI-Bindung; P2 insgesamt/P3–P5
und Produkt-Q-01/Q-02 offen, striktes Datenschutzziel unverändert.

Reproduktion mit `scripts/runtime-isolation/check-workspace-bootstrap.ps1` und expliziten Hashes aus
dem Nachweis. Keine GUI starten, keine echte Vorschauablage öffnen/zurücksetzen oder neue EXE dort
autorisieren. Für ETW getrennte Controller-/Worker-PIDs samt Profil-/Jobzuständen erfassen; keine
alten Netzbelege auf neue Bytes übertragen. Vorherige P2a-/P1-Abschnitte sind historisch.

Vorheriger Stand: [Produktstart P2a · Record-only](testing/product-start-p2a-records.md).
Produktstart P2a, 17. November: authentisierte persistente Schema-5-Bindungs-/Launch-Records im
pfadbegrenzten Forschungschild implementiert, nur synthetische private Ablagen ohne GUI. P1-Kern
wird nach nativen Actor-/Kopf-/Job-/MAC-Prüfungen genutzt, Zustand aus Recorddigests rekonstruiert;
erst passender Abschlusskopf gibt Store frei. `.kqprod` deklariert Record-ABI 1 und UI-Bootstrap-ABI
0; Legacy-`.kqmeta` 1–4 unverändert. Keine Startcapability, kein Profil-/Importtask-/UI-Start. Drei
native Abbruchfälle mit 31 Prozessen: vor Record, nach Kopf, unveröffentlichter Tail. Replay bleibt
gesperrt, Tail wird nicht repariert; Schlüssel/Policy/Unicode-Daten erhalten. Vier neue Tests, 67
aktive Helfer-/Transporttests/Clippy und unabhängiger MAC-/PE-/Root-/Inventaraudit grün. Ein
Helferbuild, kein App-/Transport-/Installerumbau oder Workspace-Gesamtlauf. Echte
Vorschau-/Projektdaten unverändert, Prozesse beendet. Brokerprivate Fixture ist kein
Container-Produktvault; nicht per ACL promovieren. Neuer Broker ohne eigenen frischen ETW-Beleg.
Nächster Scope P2b: authentisierter synthetischer Bootstrap mit Token/Job vor Ausführung, ohne
Importtask; vor weiterer Freigabe eigener Netznachweis. Produktintegration/Bestandsübernahme offen,
striktes Datenschutzziel unverändert.

Reproduktion `scripts/runtime-isolation/check-workspace-records.ps1` mit gesicherten Hashes aus dem
Nachweis; aktuelle Prüfung verwendet Build `kraqel-product-start-p2a-build-20261117-02`. UI-Fixture
nur als Datei kopiert, noch nicht startfähig für den Produktvertrag. P2b benötigt explizit getrennte
Appdaten/Paket-/Konfigurationsbindung, passenden Bootstrap und native Vor-Ausführungschecks. Keine
Autorisierung des neuen Brokers an der echten Vorschauablage und keine alten Records löschen.

Neuester funktionaler Prüfstand: [Produktstart P1 · Testmodell](testing/product-start-p1-model.md).
Produktstart P1, 17. November: reine versionierte Bindungs-/Startvorschlagstypen und Zustandsregeln
implementiert, ausschließlich `#[cfg(test)]` in `probe/src/main.rs`; keine produktive Autorität.
`product_start.rs`/`product_start/tests.rs` ohne I/O/unsafe/Schlüssel/Transport/UI. Parser liefert
`UntrustedStartProposal`, lehnt Import-/Fremdfelder und ungültige Versionen/Bindungen ab. Modell
verbraucht Launch vor Running, erhält Binding/Replayhistorie bei Ende/Unterbrechung, reserviert
Abschlussbudget; Ablehnungen ohne Mutation. Sieben neue Modelltests, insgesamt 63 aktive Helfer-/
Transporttests und Clippy grün. Vorgeschlagenes Schema 5 bleibt nativ unimplementiert; Modellköpfe/
Jobbedingungen sind keine MAC-/OS-Nachweise, keine Persistenz/Bootstrapfrist/Frameintegration.
Mittleres Prüfniveau, kein App-/Helfer-EXE-Build, `pnpm check` oder neuer ETW-Lauf. Echte Daten/
ACLs/Schlüssel unverändert, gesicherte Forschungsprogramme separat erhalten. Nächster begrenzter
Scope P2a: authentisierte persistente Arbeitsablagenbindung/Launch-Records im Forschungsbroker, nur
frische synthetische Wurzeln und zunächst ohne GUI; dann P2b Bootstrap/Token/Job ohne Importtask.
Normale Produktintegration und Produkt-Q-01/Q-02 offen; striktes Datenschutzziel unverändert.

P2a darf weder Modellbedingungen als echte Token-/Jobattestierung noch Parseerfolg als
Authentisierung verwenden. `StartModel` ist nicht serialisierbar; kein heimlicher Persistenzweg.
Bisheriges RestartTicket/Importtask und Forschungs-CLI bleiben unverändert; Schema-5-/Paket- und
Kontroll-MAC-Integration separat implementieren. Keine Freigabe/Kopie echter Vorschau-/Projektdaten.

Neuester Planungsstand:
[Produktstartvertrag v1 · Entwurf](data-format/product-start-integration-v1-draft.md).
Produktstart-/Bestandsdatenvertrag v1, 17. November: neuer ausdrücklicher Entwurf, keine
Implementierung/Produktfreigabe. Statisch bestätigt: normaler Tauri-Start ohne vorgeschalteten
Produktguard, getrennte Forschungs-UI, taskgebundenes RestartTicket v3 und migrationsfähige
SQLite-Öffnung. Vorgesehen: eigene Workspace-Startautorität ohne Importtask, kein unisolierter
Fallback, getrennte Hostkontrolle/Arbeitsablage, zerstörungsfreie explizite Bestandskopie mit
erhaltenen IDs. Kontrollschema 5/Paketdeklaration geplant, nicht aktuell unterstützt; Forschung mit
einer Sitzung/256 Records ist keine Produkt-Langzeitlösung. Stufen P1–P5 mit Prüfgates festgelegt.
Nächster begrenzter Scope P1: reine versionierte Start-/Bindungstypen samt Zustandsregeln und
gezielten Negativfällen, ohne UI-/Datei-/Produktfreigabe. Nur Dokumentation geändert, Quellen-/
Verweis-/Format-/Diffprüfung; kein Build/neue Testmatrix. Keine echten Daten geöffnet/übernommen
oder ACLs/Schlüssel geändert. Normaler Start weiter unisoliert, Produkt-Q-01/Q-02 offen; striktes
Datenschutzziel unverändert. Letzter eigener Forschungs-Netzbeleg bleibt am unveränderten Build.

Nicht das normale `run()` mit dem Forschungs-RestartTicket verdrahten. P1 erzeugt zunächst nur einen
getrennten Typ-/Zustandsvertrag; keine neue Startautorität allein durch gültiges JSON.
Kontroll-MAC-Schlüssel bleiben im Broker. Bestandsinventarisierung nicht über den migrationsfähigen
`Database::open`-Weg; Snapshotimport erzeugt neue IDs und ist keine identitätserhaltende Übernahme.

Neuester Netzbeleg:
[Prepare-Übergänge/verwalteter Kandidat](testing/native-prepare-transition-etw.md). Frisches ETW der
Prepare-Übergänge und des verwalteten Kandidaten, 17. November: unveränderte gesicherte Programme,
zwei kurze gültige Phasen. Regulärer Schema-4-Update-/Recoverypfad als Referenz gegenüber den drei
gezielten Übergangsabbrüchen; keine zusätzliche allgemeine Matrix. Alle 59 direkten Aufrufe
einschließlich fünf hashbenannter verwalteter EXEs erfasst, acht Kontrollen,
Nullverluste/Zuordnungslücken, 182 explizite Enden, 0 beobachtete Nicht-Loopback-/48 getrennte
Loopback-Ereignisse. Unabhängige MAC-/Paket-/SQLite-/Kernel-Audits grün: vier Aufgaben/vier legitime
Kopien, keine Replaykopie, Schlüssel/App-/Sitzungsbindung erhalten. 42 Diagnose-Tests, mittleres
Prüfniveau ohne neuen Build oder Workspace-Gesamtlauf. Echte Vorschauablage unverändert,
Erfassungen/Prozesse beendet, Rohbelege lokal. Nächster begrenzter Scope: Start-/Integrationsvertrag
für normalen Produktstart mit bestehenden Projekten konkretisieren; Forschungsautorität nicht
automatisch auf echte Daten übertragen. Striktes Ziel und offene Produkt-Q-01/Q-02 unverändert.

Neuester funktionaler Stand:
[Prepare-Übergangsabbrüche](testing/native-prepare-transition-crashes.md). Gezielte
Prepare-Übergangsabbrüche, 17. November: drei standardmäßig inaktive Punkte im pfadbegrenzten
Forschungschild: `artifact-removed`, `package-created`, `prepare-recorded`. Nach den ersten beiden
bleibt der Kopf unverändert/kein Pending; danach gültiges Pending mit leerer Kopie, vor Retry
ausdrücklich abzubrechen. Drei native Fälle mit 44 Kontrollprozessen/sechs GUIs bestanden; 56 aktive
Helfertests/Clippy grün, zwei vorhandene Tests um leere Kopien erweitert. Unabhängiger
MAC-/Root-/Actor-/SQLite-Audit bestätigt 13/13/15 Records, erhaltene Schlüssel und
App-/Sitzungsbindung sowie drei Aufgaben/Kopien ohne Replayduplikat. Derselbe Storelease bis
Prozessende, keine neue Autorität oder Reparatur. Echte Vorschauablage unverändert, Testprozesse
beendet. Mittleres Prüfniveau, kein Appbuild/Workspace-Gesamtlauf. Neue Brokerbytes ohne eigenen
frischen ETW-Beleg; nächster begrenzter Scope: gezieltes ETW der neuen Programme einschließlich
dieser drei Übergänge und verwaltetem Kandidaten. Unbekannte Artefakte/defekte Ketten bleiben
gesperrt; striktes Datenschutzziel und offene Produktabnahme unverändert.

Reproduktion: `scripts/runtime-isolation/check-update-reconciliation.ps1 -PrepareTransitions` mit
den neuen gesicherten Hashes aus dem Nachweis; alternativ zur bisherigen `-RetryAborted`-Serie.
Keine Prüfprogramme an vorhandener Vorschauablage autorisieren oder deren Autorität kopieren.

Neuester Netzbeleg: [Retry und verwalteter Kandidat](testing/native-package-artifact-retry-etw.md).
Frisches ETW des gesicherten Retry-Brokers, 17. November: unveränderte Programme, zwei kurze gültige
Phasen. Referenz und kombinierter Teilkopie-/Abort-/Retry-/Aktivierungs-/Recoverypfad bestanden.
Alle 21 direkten Aufrufe einschließlich fünf hashbenannter verwalteter EXEs erfasst, unveränderte
Root-/Kalibrierungs-/Verlust-/Lebenszeitguards. Acht Kontrollen, Nullverluste/Zuordnungslücken, 144
explizite Enden, 0 beobachtete Nicht-Loopback-/48 getrennte Loopback-Ereignisse. Unabhängige
MAC-/Paket-/SQLite-/Kernel-Audits grün: drei Aufgaben/zwei legitime Kopien, keine Replaykopie,
Schlüssel/App-/Sitzungsbindung erhalten. 42 Diagnose-Tests, mittleres Prüfniveau ohne neuen Build
oder Workspace-Gesamtlauf. Echte Vorschauablage unverändert; Prozesse/Erfassungen beendet, Rohbelege
lokal. Nächster begrenzter Scope: Prozessabbrüche zwischen Artefaktentfernung, Zielanlage und
Prepare-Record gezielt prüfen. Unbekannte Artefakte/defekte Ketten bleiben gesperrt; striktes
Datenschutzziel, normale Produktintegration und offene Produkt-Q-01/Q-02 unverändert.

Neuester funktionaler Stand: [Paketartefakte/Retry](testing/native-package-artifact-retry.md).
Sichere Paketartefakte/erneute Vorbereitung, 17. November: Schema 4 leitet ersetzbare Hashs nur aus
authentisierten Abort-Records ab. Aktueller Actor/Kopf, Jobleerstand und erlaubter Übergang vor
jeder Dateimutation; dieselbe Kontrollsperre durchgehend gehalten. Geprüfte Einlink-Datei über
exklusiven Handle entfernen; aktive/früher aktivierte oder unbekannte Pakete, unsichere Links/ACLs
und gesperrte Dateien nicht bereinigen. Zieldatei vor neuem Pending exklusiv erzeugen. Drei neue
Tests, 56 aktive Helfertests/Clippy und drei native Retryfälle/Daten-MAC-Audit grün. Schlüssel,
App-/Sitzungsbindung und drei legitime Projektkopien erhalten, kein Replayduplikat. Echte
Vorschauablage unverändert, Testprozesse beendet. Neuer Broker ohne eigenen frischen ETW-Beleg;
mittleres Prüfniveau, kein App-/Workspace-Gesamtlauf. Nächster begrenzter Scope: gezieltes frisches
ETW des neuen Brokers samt Retry und verwaltetem Kandidaten. Unbekannte Artefakte ohne Abort und
unvollständige Kontrollketten bleiben gesperrt; striktes Ziel und offene Produktabnahme unverändert.

Reproduktion: `scripts/runtime-isolation/check-update-reconciliation.ps1 -RetryAborted` mit
expliziten neuen Hashes aus dem Nachweis, ausschließlich frische TEMP-Cases. Keine neue Prüf-EXE an
vorhandener Vorschauablage autorisieren oder diese kopieren/zurücksetzen.

Neuester funktionaler Prüfstand:
[Update-Abbruch-/Fehlerreconciliation](testing/native-update-abort-reconciliation.md). Drei native
Fälle bestanden, vorhandener Laufzeitcode unverändert: Teilkopie, Manipulation und Crash nach
veröffentlichtem Abort. Dieselbe Kontrollsperre, alter Actor und bestehende Aufgabe bleiben
erhalten; keine defekte Kette reparieren. Zwei neue Tests, insgesamt 53 aktive Helfertests/Clippy
sowie Daten-/MAC-/SQLite-Audit grün. Neuer
`scripts/runtime-isolation/check-update-reconciliation.ps1` erzeugt nur drei frische TEMP-Cases mit
explizit gesicherten Hashes; Reproduktion im Nachweis. Drei legitime Projektkopien, keine
Replaykopie; Profile/Prozesse beendet, Rohbelege lokal. Echte Vorschauablage/verwaltete EXE
unverändert, dort keine Prüfprogramme neu autorisieren. Keine neue ETW-Erfassung dieser Fehlerpfade.
Nächster begrenzter Scope: verbleibende abgebrochene Paketdateien unter derselben Sperre sicher
behandeln und erneute Vorbereitung ohne unbeabsichtigtes Pending ermöglichen. Striktes
Datenschutzziel und offene Produktabnahme unverändert.

Neuester Netzbeleg:
[Frisches ETW beider Updateprogramme](testing/native-schema4-package-update-etw.md). Unveränderte
gesicherte Programme, zwei gültige kurze Phasen, acht Kontrollen, Nullverluste/Zuordnungslücken, 132
explizite Enden, 0 Nicht-Loopback-/48 getrennte Loopback-Ereignisse. Alle 15 nativen Updateaufrufe
samt fünf hashbenannten verwalteten EXEs abgeglichen; private exakte PID-/Image-Map nur in
`native-control`. Daten-/MAC-/Lebenszeitaudits und 42 Diagnose-Tests grün. Drei Aufgaben/zwei
legitime Kopien, keine Replaykopie. Echte Vorschauablage/verwaltete EXE unverändert; Erfassungen und
Prozesse beendet, Rohbelege lokal. Mittleres Prüfniveau ohne neuen Build/Workspace-Gesamtprüfung.
Nächster begrenzter Scope: Update-Abbruch-/Fehlerreconciliation unter derselben Sperre vor normaler
Produktintegration. Striktes Datenschutzziel und offene Produkt-Q-01/Q-02 unverändert.

Vorheriger funktionaler Stand:
[Lokale Schema-4-Brokerpaket-Updates](testing/native-schema4-package-update.md). Read-only
Paketvertrag vor Pending/Aktivierung, kein Ausführen des Kandidaten vor Freigabe. Dieselbe
Kontrollsperre/Jobprüfung. 51 aktive Helfertests/Clippy und ein kurzer nativer Wechsel auf
verwalteten Kandidaten samt Recovery/Cleanup/Daten-MAC-Audit grün. Ein synthetischer Kandidat,
gleicher Code/anderer Hash; genau eine Projektkopie erhalten, Apphash unverändert. Echte
Vorschauablage/verwaltete EXE unverändert, neue Prüfprogramme dort nicht autorisieren oder alte
Kontrollablagen übernehmen. Eigener frischer ETW-Beleg beider Actorprogramme einschließlich
hashbenannter verwalteter EXE inzwischen oben verlinkt. Mittleres Prüfniveau, striktes
Datenschutzziel und offene Produktabnahme unverändert. Testprofil/Prozesse beendet, private
synthetische Belege erhalten.

Vorheriger Prüfstand: Neuester Netzbeleg:
[Gezieltes ETW des unveränderten Cleanup-Paars](testing/native-profile-cleanup-etw.md). Zwei kurze
gültige Phasen, acht Kontrollen, Nullverluste, 140 explizite Prozessenden, 0 Nicht-Loopback-/48
getrennte Loopback-Ereignisse. Alle 19 direkten Kontrollaufrufe einschließlich fensterloser
Cleanup-/Statusprozesse erfasst (`native-control`), erwartete Aufrufe/GUIs/Daten unabhängig
gegengeprüft. 39 Diagnose-Tests grün; keine neuen Builds oder große Matrix. Erfassungen/
Supervisor/Testprozesse beendet; rohe Diagnosen und synthetische Fixtures lokal erhalten. Echte
Vorschauablage/verwaltete EXE unverändert. Nächster Scope: lokale Update-/Paketkompatibilität für
Kontrollschema 4 unter derselben Sperre, weiterhin mittleres Prüfniveau. Striktes Datenschutzziel
unverändert, keine Produktfreigabe.

Vorheriger Prüfstand: Neuester funktionaler Stand:
[Authentisierte Profilbereinigung](testing/native-profile-cleanup.md). Kontrollschema 4 signiert
Vorbereitung vor Profilerzeugung und Cleanupabschluss; Wiederholung mit aktuellem Kopf idempotent,
gleiche Kontrollsperre, keine neue Importautorität. Zwei native Fälle samt kurzem Start/Recovery und
48 Helfertests/Clippy bestanden; Daten-/MAC-Audit grün. Auf Wunsch mittleres Prüfniveau statt großer
Matrix/Workspace-Gesamtprüfung. Kein frisches ETW des neuen Brokerhashs; nächster Scope: gezielter
Netzvergleich des Cleanup-Stands. Echte Vorschauablage und unten genannte autorisierte EXE
unverändert; neuer Prüfbroker nicht automatisch aktiviert. Testprofil/Prozesse beendet, private
synthetische Belege erhalten. Ungebundene Alt-Orphans und unvollständige Kontrollketten nicht
übernehmen oder reparieren. Striktes Ziel unverändert, Produktintegration weiter offen.

Vorheriger Prüfstand: Neuester Prüfstand:
[Übergangs-Crashpunkte und frisches ETW](testing/native-session-transition-crash-etw.md). 20 neue
Übergangsabbrüche, neun gesperrte/elf authentisierbare Kontrollzustände, vier explizite Recoveries;
Status ist kein Ausführungsgrant. Testprofil-Cleanup repariert keinen Kontrollzustand. Beide
frischen ETW-Phasen gültig: acht Kontrollen, Nullverluste, 458 explizite Enden, 0
Nicht-Loopback-/138 getrennte Loopback-Ereignisse. 46 aktive Helfertests, `pnpm check` und 36
Diagnose-Tests grün; unabhängige Daten-/MAC-/Kernel-Lebenszeitaudits bestanden. Profile/Prozesse/
Erfassungen beendet; gesperrte synthetische Fixtures privat erhalten. Echte Vorschauablage und
nachfolgend genannte autorisierte EXE unverändert, neuer Prüfbroker nicht automatisch aktiviert.
Nächster Scope: authentisierte Bereinigung verwaister/stillgelegter Profile unter derselben
Kontrollsperre, keine neue Importautorität oder Schlüssel-/Kopfreparatur. Striktes Ziel unverändert;
Schema-3-Updates/echte Datenmigration/Produktintegration bleiben offen.

Vorheriger Prüfstand: Neuester funktionaler Prüfstand:
[Dauerhafte Sitzungsbindung/Recovery](testing/native-product-session-recovery.md). Neue native
Vorschau mit explizitem Kontrollschema 3, persistiertem Apphash/Sitzungswurzel/Task und verbrauchten
Startkennungen. Zwei echte Controllerabbrüche, vier Recoveries/acht Replay-Ereignisse, 36
Dateiverweigerungen, genau eine legitime Projektkopie; unabhängiger MAC-/Datenaudit, 45 aktive
Helfertests und `pnpm check` grün. Profile entfernt/Prozesse beendet; synthetische negative Fixtures
privat erhalten. Echte Vorschauablage und nachfolgend genannte autorisierte EXE unverändert. Kein
automatisches Aktivieren dieses neuen Prüfbrokers oder Migrieren echter Daten. Schema 3 erlaubt
derzeit keine Brokerupdates, eine Sitzung je Installation. Kein frischer ETW-Beleg dieses
Brokerhashs. Nächster Scope: zusätzliche Bindungs-/Startrecord-/Stilllegungs-Crashpunkte, danach
frischer ETW-Vergleich. Striktes Datenschutzziel unverändert; keine Produktfreigabe.

Vorheriger funktionaler Prüfstand: Neuester funktionaler Prüfstand:
[Kontrollgebundener isolierter Startpfad](testing/native-product-starter.md) mit neuer gesicherter
Broker-EXE. Zwei neue synthetische Sitzungen verwenden dieselbe private Kontrollsperre und denselben
Produktjob; Quelle/Kopie gepinnt, zwölf private Dateiverweigerungen und unabhängiger MAC-/Datenaudit
bestanden. 43 aktive Helfertests, Clippy/`pnpm check` grün. Echte Vorschauablage und nachfolgend
genannte autorisierte EXE unverändert, kein automatisches Aktivieren des neuen Prüfbuilds. Neue
Appdaten bleiben getrennte TEMP-Fixtures, keine vorhandenen Projekte/Journale übernommen. Kein
frischer ETW-Beleg dieses Brokerhashs. Nächster Scope: authentisierte dauerhafte
Apppaket-/Sitzungsbindung und Wiederanlauf unter derselben Sperre, danach eigener ETW-Vergleich.
Striktes Ziel unverändert, keine Produktfreigabe.

Vorheriger Prüfstand mit eigenem Netzbeleg:
[Kontroll-Crash-Serie und frisches ETW](testing/native-product-control-crash-etw.md) mit separat
gesichertem Prüfbroker/Updatekandidat bestanden. 28 Abbrüche, 17 gesperrte/elf nutzbare Zustände,
acht explizite Weiterführungen ohne Schlüsselreset; zwei gültige ETW-Phasen mit acht Kontrollen,
Nullverlusten, 1.136 expliziten Enden, 0 Nicht-Loopback-/444 getrennten Loopback- Ereignissen. 40
Helfertests, Clippy/`pnpm check` und 36 Diagnose-Tests bestanden. Alle GUI-Testprofile entfernt,
Prozesse/Erfassungen beendet; private synthetische Crash-Fixtures bleiben lokal. Wichtig: Echte
`.kraqel-broker-preview` und ihre nachfolgend genannte autorisierte EXE wurden nicht verändert. Der
neue Prüfbroker ist dort nicht automatisch autorisiert. Keine Kopf-/Recordreparatur, keinen
Schlüsselreset und keine Kopie der Kontrollablage auf andere Rechner. Nächster Scope: isolierter
Produktstarter mit gleicher Sperre und getrennten Datenrollen, echte Datenmigration und gesperrte
Abschlüsse separat prüfen. Striktes Ziel unverändert, keine Produktfreigabe.

### Frühere Prüfstände (absteigend, damalige nächste Schritte)

Der aktuelle nächste Schritt steht oben. Frühere Netzbelege gelten jeweils nur für ihre eigenen
Programmhashes und Lebenszeiten.

Vorheriger Stand:
[Native Kontrollablage/Updates/Kontrollmigration](testing/native-product-control.md) als Vorschau
implementiert und nativ geprüft. Lokale dauerhafte `.kraqel-broker-preview` im Benutzerprofil bleibt
erhalten; keine privaten Schreibprojekte übernommen. Kontrollschema 2, drei explizite
Programmwechsel mit durchgehender MAC-Kette; finale verwaltete EXE unter
`%USERPROFILE%/.kraqel-broker-preview/bin/cfe51760acc79112a5bff612326fa5cf1f6ef94d97cd897f7b09cc14850ae51c.exe`.
Weitere Änderungen nur über aktuell autorisierte EXE und aktuellen nativen Kopfhash, keine
Schlüssel-/Kopf-/Journal-Reparatur. Auf anderem Rechner neue Vorschauablage anlegen; niemals diese
Schlüssel oder Forschungswurzeln kopieren. 39 Helfertests, Clippy/`pnpm check`, 30 native Kontroll-
aufrufe, zehn erwartete Abweisungen, 18 echte Container-Dateiverweigerungen und GUI-Regression
bestanden. Alle Testprofile entfernt/Prozesse beendet. Die damals folgende Kontroll-Crash-Serie und
deren eigener frischer ETW-Lauf sind inzwischen oben belegt. Normale Produktintegration, echte
Datenmigration und Herausgebervertrauen bleiben offen; striktes Ziel unverändert.

Neuester Netzbeleg:
[Frisches ETW des Fensterende-Paars](testing/native-combined-window-close-etw.md) einschließlich
beider manuell bedienten Hauptfensterschlüsse und automatischer Wiederherstellung bestanden. Paar
unverändert, Hashes im Nachweis. Zwei getrennte gültige Phasen, acht Kontrollen, Nullverluste, alle
942 Enden explizit; 0 Nicht-Loopback-/536 getrennte Loopback-Ereignisse. Unabhängige
MAC-/Daten-/Lebenszeitaudits grün, 22 Aufträge/13 legitime Kopien. Alle elf Profile entfernt,
Erfassungen/Prozesse beendet; 36 Diagnose-Tests bestanden, kein Runtimeumbau/neuer Build. Nächster
Scope: geschützte dauerhafte Produktablage und authentisierte Updates/Migration. Weitere allgemeine
F08-/Fensterende- und externe Dienstgrenzen offen. Rohdaten/Schlüssel bleiben lokal; auf anderem
Rechner frische Testwurzeln und eigener Nachweis. Striktes Ziel unverändert, keine Produktfreigabe.

Vorheriger Funktionsstand: [Kombinierte Fensterendeprüfung](testing/native-combined-window-close.md)
für offenen nativen Dateidialog und Teiltransfer bestanden. Gesichertes finales Paar mit Helferhash
`1b5c0efa771f58a7e1274369d391b92a2c788aa1d64756e6ef4530b2f2555b4f`, App unverändert. Nach Schließen
reguläres Ende/Threadjoin/Jobleerstand, Abbruch ohne Kopie; danach je Recovery, gesperrte
Altanfragen, neuer Auftrag und zweifache Wiederherstellung geprüft. 16 Apps, vier Aufträge, zwei
legitime Kopien; unabhängige MAC-/Datenprüfung, 31 Helfertests/Clippy und `pnpm check` bestanden.
Alle Profile einschließlich des verworfenen Erstlaufs bereinigt, Prozesse beendet. Damals nächster
Scope: frischer ETW-Netzvergleich des neuen Paars mit beiden Fensterendepfaden, keine Übertragung
älterer Netzbelege. Rohdaten/Schlüssel lokal; auf anderem Rechner frische Testwurzeln verwenden.
Striktes Datenschutzziel unverändert, keine Produktfreigabe.

Vorheriger Nachweis:
[Tastatur und frisches ETW des manuellen Brokerpaars](testing/native-combined-assistive-restart-etw.md).
Eigener Tastaturvorlauf plus vier gemessene Wiederanlaufzustände menschlich bestätigt und nativ
gegengeprüft; Pointer-Ereignisse nicht als physischer Eingabenachweis auslegen. Zwei gültige Phasen,
acht Kontrollen, Nullverluste, alle 906 Enden explizit; 0 Nicht-Loopback-/514 getrennte Loopback-
Ereignisse. Unabhängige MAC-/Daten-/Lebenszeitaudits bestanden. Gesichertes App-/Helferpaar
unverändert, Hashes im Nachweis; kein Runtimeumbau. Alle Profile entfernt, Messungen/Prozesse
beendet. Nächster Scope: weitere kombinierte Fensterendeprüfung, danach
Produktablage/Updates/Migration. Rohdaten/Schlüssel bleiben lokal; auf anderem Rechner neue
Testwurzeln und eigener Nachweis. Keine Produktfreigabe oder Datenschutzausnahme.

Vorheriger Stand: [Kombinierter assistiver Nachweis](testing/native-combined-assistive-restart.md)
für Unterbrechung, Abschluss, synthetischen Abbruch und ungültiges Archiv mit Hör-/Fokusbestätigung
und nativem Replay-Gegenbeleg. Zwei Erstversuche korrekt abgelehnt, reine Tastaturabnahme damals
offen. Neue Helferaktion `ui-broker-recover <Wurzel> manual`, App unverändert; neue Hashes im
Nachweis. 28 aktive Helfertests, Clippy und `pnpm check` bestanden. Alle vier Profile entfernt,
keine Forschungsprozesse mehr. Damals nächster Scope: frischer ETW-Netzvergleich des neuen Paars mit
automatischer Regression und manuellen Wiederanläufen. Alte ETW-Belege nicht übertragen.
Rohdaten/Schlüssel nur lokal, niemals über Git transportieren; auf anderem Rechner frisch bauen und
neue Testwurzeln verwenden. Striktes Datenschutzziel unverändert, keine Produktfreigabe.

Älterer Nachweis 16. November: Frisches ETW exakt des gesicherten kombinierten v3-Paars bestanden.
Zwei getrennt kalibrierte Phasen: Automatik einschließlich 54 abgewiesener Folgeaktionen, danach
tatsächliche manuelle Dateiauswahl mit Wiederanlauf/neuem Auftrag/Replay. 114 Broker/80 Apps/480
WebViews/194 Console-Hosts, 868 explizite Enden, acht Kontrollen, Nullverluste, 0 Nicht-Loopback-
und 480 getrennte Loopback-Ereignisse. Daten/MACs und Generationszuordnung unabhängig geprüft.
[Nachweis und Grenzen](testing/native-combined-live-restart-etw.md). Alle Profile entfernt,
Erfassungen/Prozesse beendet, Paar unverändert. Damals nächster Scope: kombinierte assistive
Wiederanlauf-/Fehlerprüfung und weitere Fensterendeprüfung mit genau diesem Paar. Produktintegration
und externe Dienste weiter offen; keine Übertragung auf Folgeänderungen, striktes Datenschutzziel
unverändert.

Vorheriger Funktionsstand 16. November: Kombinierter Forschungsablauf v3 mit echter Live-Auswahl,
Wiederanlauf und ausdrücklich neuem Folgeauftrag bestanden. Signierte Grantkette und private
Abschlussbelege ersetzen keine neue menschliche/native Beauftragung.
[Verträge und finale Belege](testing/native-combined-live-restart.md). Neun automatische Fälle, 72
App-Starts, 54 verweigerte Folgeaktionen; manueller Dialoglauf bestätigt und vollständig
weitergeprüft. Beide Vorläufersitzungen und fehlgeschlagener Dialoglauf stillgelegt; finale Profile
entfernt, Prozesse beendet. Normale Debug-App ohne Forschungsseite wiederhergestellt. Gesichertes
neues Paar: `%TEMP%\\kraqel-kq069-combined-build-20261116-04`, vollständige Hashes im Aggregat. Alte
Binaries/Nachweise nicht übertragen. Nach dem oben ergänzten ETW-Beleg folgen assistive und weitere
Fensterendeprüfung des kombinierten Ablaufs. Produktablage, authentisierte Updates/Migration und
externe Dienste bleiben offen; striktes Ziel unverändert.

Neuester Nachweis 16. November: Frisches ETW des gesicherten HMAC-Paars bestanden, einschließlich
aller negativen Brokerstarts. 103 Broker/30 Apps/180 WebViews/133 Console-Hosts, 446 explizite
Enden, vier Kontrollen, Nullverluste, 0 Nicht-Loopback- und 180 getrennte Loopback-Ereignisse.
Native Serie, unabhängige MACs/Daten und PID-Wiederverwendung geprüft.
[Nachweis/Buildgrenzen](testing/native-hmac-broker-etw.md). Erfassung/Prozesse beendet. Nächster
Scope: kombinierter Live-/Neustartpfad, geschützte Produktablage und authentisierter Update-/
Migrationspfad; externe Dienste/Produktintegration weiter offen. Striktes Ziel unverändert.

Aktuell 16. November: Broker-Vertrag [v2](data-format/native-broker-recovery-v2.md) entfernt DPAPI
und authentisiert den dauerhaften Beleg lokal mit HMAC-SHA-256. Schlüssel nur unter privater DACL,
keine Verschlüsselung/Hardwarebindung behaupten; keine automatische Schlüsselneuanlage oder
Altdatenmigration. [Funktionsnachweis](testing/native-hmac-broker.md): sechs harte Controllerenden,
zwölf Wiederanlaufprozesse, 30 App-Generationen, 78 Ablehnungen und unabhängige Daten-/MAC-Prüfung
bestanden. Neues Paar separat unter `%TEMP%\kraqel-kq069-hmac-broker-build-20261116-01` gesichert;
eigener ETW-Beleg ist oben verlinkt. Alte Paare und Belege nicht übertragen. Produktintegration und
externe Grenzen bleiben offen, striktes Ziel unverändert. Historische Übergaben folgen.

Historischer DPAPI-Produktblocker: DPAPI-Domänencontroller-/RPC-Grenze gesondert prüfen oder
Primitive ersetzen. Keine Produktintegration allein aufgrund eines unauffälligen ETW-Appbaums;
[Vertrag](data-format/native-broker-recovery-v1.md). Striktes Ziel unverändert.

Neuester Stand 16. November: Controller-Crash-Serie und dauerhafte native Wiederanlaufautorität im
Forschungslauncher bestanden: sechs harte Controllerenden, zwölf neue Broker-Wiederanläufe, 30
App-Generationen, 60 Ablehnungsfälle. [Vertrag](data-format/native-broker-recovery-v1.md),
[Nachweis und gesicherte Hashes](testing/native-controller-crash.md). Daten unabhängig geprüft,
Profile entfernt, Prozesse beendet. Gesichertes Paar lokal unter
`%TEMP%\\kraqel-kq069-controller-broker-build-20261116-03`; alte Paare unverändert erhalten.
Nächster Scope: eigener ETW-Netzvergleich dieses neuen Paars, danach kombinierter Live-/Neustartpfad
und Produktbrokerintegration. Alter ETW-Beleg ist keine Freigabe des neuen Brokers. Produktstart
weiter unisoliert; striktes Datenschutzziel und offene Produkt-Q-01/Q-02 unverändert. Frühere
Übergabeschritte folgen.

Neuester Nachweis 16. November: Frisches ETW exakt des gesicherten v2-Paars bestanden,
einschließlich automatischer 28-Start-Serie und beider manueller Wiederanlaufpfade. Drei
Controllerbäume mit 32 Apps/168 WebViews, alle 238 Lebenszeiten explizit beendet; vier Kontrollen,
Nullverluste, 0 Nicht-Loopback- und 176 getrennte Loopback-Ereignisse. Daten/Cleanup unabhängig
geprüft. [Nachweis](testing/native-assistive-restart-etw.md),
[Aggregate/Hashes](testing/native-assistive-restart-etw.json). Programme unverändert,
Erfassung/Prozesse beendet, Rohdaten lokal. Nächster Scope: Controller-Crash und dauerhafter
authentisierter Broker, danach kombinierter Live-/Neustartpfad und externe Grenzen. Keine
Produktfreigabe; striktes Ziel unverändert.

Neuester Stand 16. November: Assistive Wiederanlaufprüfung mit Windows-Sprachausgabe für
Unterbrechung/Abschluss/Replay bestanden; zusätzliche Tastaturnutzung menschlich bestätigt, mit
begrenzter technischer Eingabequellen-Zuordnung.
[Startticket v2](data-format/native-ui-restart-v2.md) mit `automatic: false` lässt nur
Wiederherstellung manuell offen, ohne neue Rechte/Importquelle.
[Nachweis, Wiederholung und Hashes](testing/native-assistive-restart.md). 28 automatische v2-Starts
und lokale Qualitätsprüfungen bestanden. Neues Paar unter
`%TEMP%\kraqel-kq069-assistive-restart-build-20261116-01` gesichert; Hashes vor Verwendung prüfen.
Nächster Scope: frisches ETW genau dieses Paars. Der alte v1-Netzbeleg gilt separat; Controller-
Crash, dauerhafter Broker, externe Grenzen und Produktabnahme bleiben offen. Striktes Ziel gilt.

Neuester Nachweis 16. November: Frischer gültiger ETW-Netzvergleich exakt des gesicherten
Neustart-Paars bestanden. Ein Controllerbaum mit 28 Apps, 144 WebViews und insgesamt 202 explizit
beendeten Lebenszeiten; vier Kontrollen, Nullverluste, 0 Ereignisse außerhalb Loopback und 144
getrennte Loopback-Ereignisse. Auch unter ETW sind Crash/Reconciliation/Altimportabwehr sowie
SQLite/Unicode/Kopienzahl/Cleanup unabhängig bestätigt.
[Nachweis](testing/native-ui-restart-etw.md),
[Aggregate und Hashes](testing/native-ui-restart-etw.json). Rohdaten lokal, Erfassung beendet,
Programme unverändert. Nächster Scope: assistive Wiederanlaufprüfung, Controller-Crash/dauerhafter
Broker und externe Grenzen. Keine Produktfreigabe; striktes Ziel unverändert.

Neuester Stand 16. November: Integrierte isolierte GUI-Crash-/Neustartserie bestanden, 28
Generationen desselben App-/Controllerpaars. Gleicher lebender Controller, OS-Identität und
Datenwurzel; frische Pipes/Launch-Bindungen je Start. Sechs Crashpunkte, je zwei Reconciliations und
Altimportabwehr; vier ungültige Starts verworfen. Genau zwei Kopien, vier unterbrochene Aufträge,
keine Zwischenreste, alle Jobs leer und Profil entfernt.
[Vertrag](data-format/native-ui-restart-v1.md), [Nachweis und Hashes](testing/native-ui-restart.md),
[Aggregate](testing/native-ui-restart.json). Nächster Scope: neuer ETW-Netzvergleich mit exakt dem
gesicherten Paar, danach assistive Wiederanlaufprüfung. Controllerabsturz/dauerhafter Broker und
Produktintegration weiterhin offen; striktes Datenschutzziel unverändert. Frühere Stände folgen zur
historischen Einordnung.

Neueste Fortschreibung 16. November: Dauerhafter Auftragsverbrauch und konservative
Crash-Reconciliation im Forschungsadapter, vor der nativen Auswahl im Tauri-Forschungsfeature
angebunden. Sieben neue Windows-Integrationstests mit sieben echten Prozess-Kills bestehen;
Bestandsdaten bleiben bytegleich, Replay bleibt gesperrt. Native Erfolg-/Abbruchrückprüfung mit
Replay, Journal und Cleanup bestätigt. [Vertrag](data-format/native-task-journal-v1.md),
[Buildbindung und Nachweis](testing/native-task-reconciliation.md). Nächster Scope: authentisierter
Neustart derselben isolierten GUI mit stabiler Ablage und integrierte Crash-Serie; Controller
verlangen bisher frische Forschungswurzeln. Kein neuer ETW-Beleg, keine Produktfreigabe.

Neueste Fortschreibung 16. November: Assistive Erfolg-/Fehlerpfade mit Replay, passenden Ansagen und
Fokus nativ bestätigt. 34 Browser-E2E bestanden; zusätzliche begrenzte Forschungsdiagnose enthält
ausschließlich feste Zustandswerte.
[Nachweis und Buildhash](testing/native-live-assistive-focus.md). Frühere Fokusabweichungen bleiben
dokumentiert; keine assistive Vollabnahme. Der geänderte App-Build hat keinen neuen ETW-Nachweis.
Nächster Scope: dauerhafter Auftragslebenszyklus und Crash-Reconciliation, anschließend weitere
Broker-/Netzabnahme. Striktes Ziel unverändert.

Neuester funktionaler Nachweis 16. November: Fensterende bei offenem Dateidialog und nach echtem
Teiltransfer bestanden. Eigener Prozess/Thread/Dialogklasse vor Schließnachricht geprüft,
Auswahlthread begrenzt beendet; keine Projektkopien oder Stagingreste. Bytezähler belegen 1.024
empfangene von 11.915 erwarteten Bytes.
[Buildgrenzen und Reproduktion](testing/native-live-window-close.md). Frischer ETW-Vergleich am
unveränderten finalen Build bestanden: Referenzimport/Replay, Dialogende und Teiltransfer in drei
vollständigen Bäumen mit 19 WebView2-Lebenszeiten; vier Kontrollen, Nullverluste, 0 Ereignisse
außerhalb Loopback. [Aggregate](testing/native-live-window-close-etw.json). Forschungsprozesse und
Erfassung beendet; Rohdiagnosen bleiben lokal. Nächste Abnahme: Screenreader/Fokus, externe
Brokergrenzen und dauerhafter Auftragslebenszyklus. Striktes Ziel unverändert, keine
Produktfreigabe. Die folgenden Notizen beschreiben frühere Stufen.

Aktualisierung 16. November: Replay-/Fehlerprüfung und Fensterende am Publish-Haltepunkt bestanden.
Neuer gültiger ETW-Nachweis für drei vollständige Forschungs-App-Bäume: 0 Ereignisse außerhalb
Loopback, vier Kontrollen, Nullverluste. Replay-Anzeigefehler korrigiert.
[Exakter Buildbezug und offene Aufgaben](testing/native-live-file-selection.md). Rohdiagnosen
bleiben lokal, ETW-Sitzung beendet. Nach Messung Typalias zur Clippy-Korrektur ohne neuen
Binärnachweis. Striktes Ziel unverändert; nächste Prüfung: Appende bei offenem Dialog/Transfer,
Screenreader/Fokus und externe Brokergrenzen. Keine Produktfreigabe, #134 bleibt offen.

Fortschreibung 16. November: `ui-live` ergänzt die Auswahl aus dem laufenden isolierten
Forschungsfenster. Native Erfolg-/Dialogabbruchserie und unabhängige SQLite-/Cleanup-Prüfung
bestanden. [Nachweis und verbleibende Schritte](testing/native-live-file-selection.md). Replay,
assistive Fokus-/Fehlerprüfung, Appende bei Auswahl/Transfer und neuer ETW-Lauf bleiben offen. Keine
Produktfreigabe; striktes Datenschutzziel unverändert.

Entwicklungsstand: [PR #140](https://github.com/kolkrabeofdoom/kraqel/pull/140), KQ-069 A /
[Issue #134](https://github.com/kolkrabeofdoom/kraqel/issues/134). PR #139 ist nach grüner CI
zusammengeführt (`fadf0a6`). PR #140 enthält die UI-/Tauri- Testanbindung und den einmaligen
Einweg-Dateiauftragskanal einschließlich finaler Nachweise. Die abschließende Übergabe wird mit
diesem PR nach vollständig grüner CI nach `main` übernommen. Beim Wiederbeginn den tatsächlichen
GitHub-Merge-Status prüfen; danach `main` aktualisieren. Falls der PR noch offen ist, liegt die
gesamte Arbeit auf `test/kq-069-isolated-tauri-channel`.

Bestanden: `pnpm check`, normaler und Forschungs-Tauri-Build, Feature-Clippy, 29 Browser-E2E, 18
Helfer-/Transporttests, ein Tauri-Origin-Test und 32 Python-Diagnosetests. Drei finale native
Sitzungen prüfen Import/Wiederholungsablehnung, Abbruch und direktes Fensterschließen am Haltepunkt.
SQLite/Unicode, neue Projekt-ID, Kopienzahl und Cleanup sind unabhängig bestätigt. Gültiges ETW
vom 15. November, 16:43:07–16:46:11 UTC: drei vollständige App-/WebView-Bäume, vier Kontrollen,
Nullverluste, 0 Ereignisse außerhalb Loopback; Loopback-Disconnects getrennt ausgewiesen.

**Das strikte Datenschutzziel bleibt bestehen.** Normale App-Starts sind weiterhin unisoliert;
Q-01/Q-02 für das Produkt sind nicht bestanden. Nur feste synthetische Vorabfreigaben sind im
Forschungsfeature geprüft. Keine allgemeine Freigabe realer Manuskripte, externer Broker oder
kompromittierter Runtime-Prozesse; #134 bleibt offen.

Nachtrag 16. November: Native Vorab-Dateiauswahl (`ui-select`) und Dialogabbruch sind auf dem
zurückgewechselten Rechner funktional geprüft; der neue Build, Controllerausgabe und unabhängige
SQLite-/Cleanup-Prüfung sind im [Nachweis](testing/native-ui-select-acceptance.md) dokumentiert.
Kein neuer ETW-Nachweis und keine Produktfreigabe. Nächster Scope: Auswahl aus einer laufenden App.

Historischer Ausgangspunkt der Übergabe: native Vorab-Dateiauswahl und deren Abbruch prüfen. Die
erfolgreiche finale UI-Serie verwendete `ui-fixture`; der vorherige unvollständige Auswahldialoglauf
ist kein Abnahmenachweis. Danach Auswahl aus einer laufenden App, assistive Abnahme und dauerhafte
Auftrags-/Replay-/Crash-Reconciliation. Weitere Formate, Git-/Credential-/Providergrenzen,
öffentliche UDP-/IPv6-/DNS-/Broker-Gegenproben und Produkt-/Installerintegration folgen separat.

**Abhängigkeiten des Windows-Kandidaten, geprüft am 24. November 2026.** GitHub meldete drei offene
Dependabot-Warnungen für `main` (`9650ee7d6730`). Die eingecheckten Lockfiles auf dem
Entwicklungsbranch (`8c4132a4a176`) ergeben für den lokalen Kandidaten folgenden Stand.

| Warnung                                                                               | Auflösung auf `main`                                     | Auflösung auf dem Entwicklungsbranch                                                                                                                 |
| ------------------------------------------------------------------------------------- | -------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- |
| [#8 · PDF](https://github.com/kolkrabeofdoom/kraqel/security/dependabot/8)            | `printpdf 0.7.0` zieht `lopdf 0.31.0` ein.               | PDF-Schreiber und Rückleseprüfung verwenden dieselbe korrigierte Version `lopdf 0.47.0`.                                                             |
| [#6 · Lint-Werkzeuge](https://github.com/kolkrabeofdoom/kraqel/security/dependabot/6) | `typescript-estree 8.72.1` zieht `minimatch 10.2.6` ein. | `typescript-estree 8.73.0` verwendet `11.0.9`. Die separate ESLint-Abhängigkeit `3.1.5` liegt auf beiden Branches außerhalb des gemeldeten Bereichs. |
| [#7 · GLib](https://github.com/kolkrabeofdoom/kraqel/security/dependabot/7)           | `glib 0.18.5` im plattformübergreifenden Cargo-Lockfile. | `cargo tree --locked --offline` bestätigt `glib 0.18.5` über GTK/WebKitGTK für Linux und keine Einbindung in das Windows-x64-MSVC-Ziel.              |

Für Linux bleibt #7 offen. Der korrigierte GLib-Zweig beginnt mit 0.20.0, während Tauri 2.12.7 GTK
^0.18 verwendet. Erneut prüfen, sobald der Desktop-Stack die korrigierte Reihe unterstützt oder ein
bestätigter Backport erscheint. Die Korrekturen des Entwicklungsbranches erreichen `main` und ältere
Installationen jeweils erst mit Integration und Auslieferung.
[GLib-Advisory](https://github.com/advisories/GHSA-wrw7-89jp-8q8g) ·
[Abhängigkeiten von Tauri 2.12.7](https://docs.rs/crate/tauri/2.12.7)

Für die Fortsetzung zuerst lesen:

1. `AGENTS.md`, [Entwicklungsstatus](DEVELOPMENT_STATUS.md), [Roadmap](../ROADMAP.md) und #134.
2. [ADR 0034](architecture/decisions/0034-webview-network-boundary-proposal.md) und
   [Bedrohungsmodell](architecture/threat-model.md).
3. [Kanalvertrag](data-format/native-file-channel-v1.md),
   [native UI-/ETW-Nachweise und Buildanleitung](testing/native-tauri-file-channel.md) sowie
   [Toolanleitung](../scripts/runtime-isolation/README.md).

Der Quellcode einschließlich synthetischer `.kraqel`-Fixture ist vollständig versioniert. Builds,
temporäre Testprojekte, WebView-Profile, lokale PID-Journale und rohe ETW/XML-Dateien bleiben auf
diesem Rechner. Auf dem anderen Rechner neue TEMP-Wurzeln, neue Builds/Hashes, neue Journale und bei
Bedarf einen neuen ETW-Lauf erzeugen. Historische JSON-Aggregate sind Nachweise dieses Rechners,
keine lokale Abnahme des anderen Rechners. Alte absolute TEMP-/Workspacepfade nicht übernehmen. Es
gibt keine erforderlichen, nur in einem lokalen `work`-Ordner liegenden Quelldateien.

Hier geprüft: Node.js 26.13.1, pnpm 11.25.0, Rust/Cargo 1.100.0, Windows/WebView2. Maßgeblich für
die Installation bleiben `packageManager`, `engines` und `rust-version` im Repository. Das
Isolationsfeature benötigt Windows; die übrige CI prüft auch Linux. Computer Use war hier mit
`Transport closed` ausgefallen, daher erfolgte die native Fensterbedienung durch den Nutzer.
Verfügbarkeit auf dem anderen Rechner neu prüfen, keine alternative Windows-Automation annehmen.

## Diesen Rechner sauber verlassen

1. Bei einem offenen Arbeitsstand den oben genannten Arbeitsbranch pushen und mit
   `git status --short --branch` den sauberen Checkout und den Abgleich mit GitHub prüfen. Nach
   einem Merge den integrierten Stand auf `main` prüfen:

   ```powershell
   cd D:\kraqel
   git switch main
   git pull --ff-only
   git status --short
   git log -1 --oneline
   ```

2. Wenn zusätzlich Schreibprojekte wechseln sollen, in KRAQEL unter **Versionen** zuerst einen
   Snapshot erzeugen und danach **Projektarchiv exportieren** wählen. Das erzeugte `.kraqel`-Archiv
   an einem privaten, vertrauenswürdigen Ort ablegen. Manuskripte und API-Schlüssel gehören nicht
   ins öffentliche Repository.
3. Optional die Übertragung mit `Get-FileHash <Archiv.kraqel> -Algorithm SHA256` auf beiden Rechnern
   vergleichen.

## Auf dem anderen Rechner beginnen

Für einen neuen Checkout:

```powershell
git clone https://github.com/kolkrabeofdoom/kraqel.git D:\kraqel
cd D:\kraqel
corepack enable
pnpm install --frozen-lockfile
git status --short
pnpm --filter @kraqel/desktop typecheck
```

Für einen vorhandenen Checkout nach einem Merge (bei einem offenen Draft-PR den Arbeitsbranch aus
der aktuellen Übergabe auswählen):

```powershell
cd D:\kraqel
git switch main
git pull --ff-only
pnpm install --frozen-lockfile
git status --short
```

Benötigt werden Git, Node.js 22 oder neuer, Corepack/pnpm gemäß `packageManager`, Rust 1.90 oder
neuer sowie unter Windows WebView2. Danach `AGENTS.md`, `docs/DEVELOPMENT_STATUS.md`, das nächste
GitHub-Issue und den zugehörigen ADR lesen.

Das Schreibprojekt wird auf der Startseite über **Projektarchiv importieren** als neue lokale Kopie
eingelesen. Die laufende Arbeitsdatenbank liegt unter `%LOCALAPPDATA%\org.kraqel.desktop\projects`
und wird bewusst nicht von Git synchronisiert. Cloud-API-Schlüssel liegen im
Betriebssystem-Schlüsselspeicher und müssen auf dem zweiten Rechner erneut eingerichtet werden.

## Verbindliches Arbeitsmuster

Ein Issue pro klarer Änderung, ein kurzer Branch, ein Pull Request mit grüner CI, danach Merge nach
`main`. Am Sitzungsende: Statusdokumente aktualisieren, offene manuelle Prüfungen ausdrücklich als
offen markieren, pushen und `git status --short` auf leer prüfen. So bleibt die Entwicklung
ortsunabhängig, ohne eine mystische Festplatte zum einzigen Wahrheitsministerium zu ernennen.
