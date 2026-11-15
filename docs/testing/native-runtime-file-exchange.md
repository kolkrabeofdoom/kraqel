# KQ-069 A: Gezielter Datei-/Shell-Austausch

Stand: 15. November 2026. PR #137 wurde am exakten Head `f5f1d35` nach grüner Windows-/Linux-CI und
Dependency Review zusammengeführt (`84b31f9`). Issue #134 bleibt offen. Pflichtenheft Revision 1
§1/§5/§10 und ADR 0034 Richtung A gelten unverändert: keine fachliche Runtime-Ausnahme.

## Implementierter Forschungsumfang

Der separate [Rust-Helfer](../../scripts/runtime-isolation/README.md) hat einen neuen einmaligen
`exchange`-Modus. Der normale KRAQEL-Start und seine Commands bleiben unverändert. Ausschließlich
selbst erzeugte synthetische UTF-8-Dateien in einem neuen TEMP-Ordner sind zugelassen. Dies ist ein
Transportprototyp, keine fertige Integration von Archiv-, DOCX- oder Medienimport.

Der nicht isolierte Controller erzeugt zwei **getrennte Geschwisterordner**:

- `kraqel-exchange-host-…`: `input.txt` und die nicht ausgewählte `withheld.txt`, ohne zusätzliche
  Paket-ACL. Hier wählt die Person Eingabe und ein neues Ausgabeziel im nativen Shell-Dialog.
- `kraqel-isolation-…`: gestagtes Programm und eigener Daten-/Berichtsbereich für den Worker. Nur
  diese Forschungswurzel erhält die bereits dokumentierten Paket-Rechte.

`IFileDialog2::SetNavigationRoot` begrenzt die angezeigte Navigation. Unabhängig davon akzeptiert
Rust nur einen direkten, flachen ASCII-`.txt`-Namen innerhalb des frischen Hostordners. Die Eingabe
ist in dieser Stufe ausschließlich die bekannte `input.txt`. Pfade, UNC, alternative Datenströme,
Gerätenamen und Shellverknüpfungen sind keine erlaubten Transportaufträge. Die Shell-Beschränkung
ersetzt keine Rust-Prüfung. Es gibt keinen generischen Shell-Execute- oder URL-Aufruf.

Der Controller öffnet die Eingabe nur lesend, ohne Schreib-/Löschfreigabe für andere Handles, und
eine neue temporäre Ausgabe nur schreibend/exklusiv. Er prüft **Metadaten**, liest aber keine
Nutzbytes und ruft weder Formatparser noch Netzdiagnosekontrollen auf. Reparse-, Verzeichnis-,
Offline-/Recall-Dateien und Hardlinks werden anhand des geöffneten Handles abgelehnt. Der
experimentelle Grenzwert ist **1 Byte bis 8 MiB**; das ersetzt keine Produkt-Formatbudgets.

Genau zwei Dateihandles stehen in `PROC_THREAD_ATTRIBUTE_HANDLE_LIST`. Erst der suspendierte, auf
erwartete eigene SID und null Capabilities geprüfte Worker wird im Kill-on-close-Job fortgesetzt.
Seine Argumente enthalten Handlewerte und Bytezahl, keine ausgewählten Hostpfade. Die fest bekannte
synthetische Negativdatei wird aus dem Testwurzel-Namensraum abgeleitet. Die Vererbungsflags werden
im Controller nach `CreateProcessW` zurückgesetzt.

Der Worker prüft seinen Token erneut, versucht die nicht ausgewählte Datei über ihren Pfad zu öffnen
und verlangt Windows-Fehler 5. Erst dann kopiert er mit festem Puffer und SHA-256, verlangt die
exakte deklarierte Länge und synchronisiert die Ausgabe. Der Controller prüft den auf 4 KiB
begrenzten Bericht, Tokenfelder, Bytezahl, Hashsyntax, tatsächliche Ausgabelänge, Prozessende und
Profilbereinigung. Anschließend veröffentlicht `MoveFileExW` ohne `REPLACE_EXISTING` die neue Datei.
Abbruch startet keinen Worker; Fehler veröffentlichen keine fertige Ausgabe. Eine RAII-Verwaltung
schließt und entfernt ausschließlich die selbst angelegte temporäre Datei. Testfixtures und
Diagnoseberichte bleiben zur lokalen Prüfung liegen.

## Native Nachweise und Fehlerpfade

Drei erfolgreiche native Läufe mit synthetischem Unicode-Text:

- 70 Byte mit Umlauten, Griechisch und Kyrillisch, SHA-256
  `5e406f423178df6e2ada87509d8c4f17b96a83c2179cb8f4d4b2db6036a42215`.
- SHA-256 von Eingabe und veröffentlichter Ausgabe zusätzlich unabhängig verglichen; Hash und Bytes
  stimmen überein. Dieses separate Prüfen liest die Testbytes, der Broker selbst nicht.
- Worker ist AppContainer mit null Capabilities; erwartete Identität vor Ausführung bestätigt.
  `withheld.txt` ist über den Pfad nicht lesbar. Null aktive Jobprozesse, Profil entfernt.
- Nativer Abbruch im Öffnendialog: kein Worker-/Isolationsordner und keine Ausgabe; nur die beiden
  vorher erzeugten synthetischen Hostfixtures bleiben.
- Der letzte Speichern-Klick des finalen Builds erfolgte nach Ausfall des Bedienhelfers durch den
  Nutzer; Ergebnis, Hashes und Bereinigung anschließend unabhängig geprüft.

13 Rust-Tests unter Windows, davon neun reine Guards/Transporttests auch unter Linux vorgesehen:
Pfad-/Gerätenamen, Größenbudget, verkürzte/verlängerte Eingabe, Schreibfehler, unvollständige
Berichte sowie bestehende Root-/Messguards. Vier Windows-Dateitests prüfen Hardlink-Ablehnung,
Schutz einer vorhandenen temporären Datei, konkurrierend entstandenes Ausgabeziel und
Teilkopie-Bereinigung. Kein Test ändert Firewall- oder Systemeinstellungen. Die drei neuen
ETW-Auswertertests ergänzen die bisherigen 27 Python-Tests auf 30; der Desktopmodus verlangt
weiterhin WebView2, der neue explizite Austauschmodus einen exakt benannten Broker samt Worker.

## Begrenzter ETW-Nachweis

Nach neuer manueller UAC-Bestätigung lief der vorhandene Helfer vom **14:32:52 bis 14:34:11 UTC**.
ETL: 720.896 Byte; Rohdaten bleiben lokal. Auswertung mit `--mode exchange`, separat beim Start
notierter Broker-PID, vier beobachteten TCP-/UDP-Kontrollen über IPv4/IPv6, null Verlusten und
vollständigen Prozessenden: **keine Netzereignisse im erfassten Baum** (Broker, Worker, zwei
Console-Hosts). Keine unbekannten Zuordnungen oder Ziele.

Der ETW-Build hat SHA-256 `a1141ae3c13de414ae23574691c1cfee7c3ae959d2d37775524d4d99097e4dc6`. Danach
wurden Besitz/Bereinigung der temporären Datei und die begrenzte Berichtsvalidierung verstärkt. Der
final gebaute und nativ erneut geprüfte Helfer hat SHA-256
`dc69059160785db0ff73fab2a51fb5c22edf702389d5bab42b93bf1918fd5ad0`. **Der ETW-Lauf wird nicht als
Messung dieses nachträglich geänderten Binärstands ausgegeben.** Bereinigte Messdaten:
[JSON](native-runtime-file-exchange.json).

## Offene Grenzen und nächster Integrationsschritt

Keine vollständige Netzstille-Garantie: COM-/Shell-Dienste außerhalb des Prozessbaums werden nicht
kausal zugeordnet; Shell-Erweiterungen, Metadaten-/Vorschauhandler und mögliche Broker-Routen
bleiben gesondert zu untersuchen. Die GUI ist keine Sicherheitsgrenze. Der Prototyp ist kein Broker
für reale Manuskripte und kein Schutz vor beliebigen Angreifern im gleichen Benutzerkonto. Es gibt
keine allgemeine Benutzerprofil-, Bibliotheks- oder Internetfreigabe.

Nächster kleiner Scope: einen typisierten, einmaligen Dateiauftrag mit Lebensdauer/Abbruch und
Handlebesitz an einen isolierten App-Testadapter anbinden, dann zunächst einen konkreten
Produktimport samt zugehöriger Rust-Validierung und atomarer Veröffentlichung prüfen. Keine frei
wählbaren Shell-Verben oder rendererbestimmten Hostpfade. Reale Formatbudgets, Neuimport,
Speichern-Abbruch, Screenreader und Fehleransagen müssen separat abgenommen werden. Erst danach
weitere Formate, Git-/Credential-/Providergrenzen, öffentliche UDP-/IPv6-/DNS-/Broker-Gegenproben,
Packaging/Updates und vollständige F08-/M3-Wiederholung. Q-01/Q-02 bleiben für das Produkt **nicht
bestanden**, #134 offen.

Primärquellen: Microsoft
[Handleliste und Capabilities](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-updateprocthreadattribute),
[Navigation Root](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl/nf-shobjidl-ifiledialog2-setnavigationroot),
[Dialogoptionen](https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/ne-shobjidl_core-_fileopendialogoptions).
