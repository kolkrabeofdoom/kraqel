# BP4 · Szenenzugriff des nativen Win32-Fensters

**Vertragsentwurf v1 · 19. November 2026 · KQ-069.** Die erste Datenprobe soll genau eine
synthetische Szene lesen, ändern und nach einem Neustart wieder öffnen können. Der bisherige
Gastlauf belegt das isolierte Fenster, die Rollenprüfung und die softwareseitige Sprachausgabe.
Datenkanal, Datenrolle und native Zugriffskontrollen dieses Entwurfs stehen zur Umsetzung und
Abnahme aus. Produktprojekte bleiben gesperrt.
[Fensterprobe](../testing/native-win32-window-guest.md) ·
[ADR 0036](../architecture/decisions/0036-native-win32-ui-role-evaluation.md) ·
[bisheriger C4-Commandvertrag](product-start-p2c-c4-v1-draft.md)

## Wer darf die Szene öffnen?

1. Der Prüfaufbau erzeugt im Offline-Gast eine eigene, zugriffsgeschützte Testablage aus einem
   versionierten, fest eingebauten Szenenkatalog. Die zweite Szene dient als Gegenprobe. Eine
   Kennzeichnung im Dokument, eine übergebene UUID oder ein vom Fenster gewählter Ordner gilt
   niemals als Herkunftsnachweis. Produktverzeichnisse, Importe, Hostfreigaben, Hostzwischenablage
   und Netzadapter bleiben außerhalb des Aufbaus.
2. Unter derselben exklusiven Kontrollsperre bindet der Starter Paket und erzwungene
   UMCI-Richtlinie, Sitzung, Laufgeneration, Testablage, genau eine Szene, Zugriffsmodus sowie beide
   Prozessrollen. Er startet UI-Blatt und getrennten Rust-Datendienst selbst. Vor dem jeweiligen
   Resume prüft er an gehaltenen Handles Image, Prozessgeneration, Token, Job und effektive Rechte.
   Jede Rolle benötigt einen eigenen begrenzten Job und eine Startbindung. Die UI erhält weder
   Datei-/DB-Zugriff noch Kontrollschlüssel. Nur der Datendienst darf die Testdatenbank samt
   erforderlichen SQLite-Nebendateien öffnen. Zugriff auf fremde Ablagen, ausgetauschte
   Verzeichnisobjekte oder umgeleitete Pfade muss scheitern.
3. Vor der ersten Datennachricht müssen beide Enden des lokalen IPC-Kanals an genau diese lebenden
   Prozessgenerationen, Rollen, Sitzung und den einmaligen Auftrag gebunden sein. Der Starter muss
   die effektiven Endpunktrechte und die Nichtübertragbarkeit an Dritte für den konkreten Aufbau
   belegen. Ein öffentlicher Endpunktname, PID, eine selbst gemeldete Rolle oder die bisherige
   Fenster-Startquittung reichen dafür nicht. Bis zur nativen Gegenprüfung bleibt der Kanal
   gesperrt. Es gibt keinen Loopback-Dienst und keinen Rückgriff auf den Desktop-Commandrouter.
4. Der Datendienst bestimmt Ziel und Rechte aus der geprüften Bindung. Der Modus `read` erlaubt
   Lesen, `edit` zusätzlich Speichern. Ein Szenen- oder Moduswechsel verlangt das Ende der
   bisherigen Bindung und einen frischen Auftrag. Verliert der Starter Sperre, Rolle oder
   Gegenstelle, endet die Datenfreigabe. Eine neue Sitzung erhält erst nach belegtem Ende der alten
   Jobs und abgeschlossener Transaktion Zugriff. Ungeklärtes Ende hält die Ablage gesperrt.

## Szenen und Nachrichten

Die Tabelle definiert den vollständigen Testkatalog. Jede Zeile ist ein Absatz im gespeicherten
Dokument. Der Zeilentrenner im Commandtext ist LF. Szene B bleibt bei jedem Auftrag für A
unverändert.

| Szene | Revision 0     | Zulässige zweite Fassung |
| ----- | -------------- | ------------------------ |
| A     | `Rabe` LF `42` | `Rabe` LF `43`           |
| B     | `Mond` LF `7`  | `Mond` LF `8`            |

Gespeichert wird gemäß
[Editorvertrag](../architecture/decisions/0004-editor-document-and-autosave.md) ein Dokument der
Version 1 mit einem `doc`, pro Zeile einem `paragraph` und genau einem unmarkierten `text` darin.
Zusätzliche Knoten, Attribute, Marks oder vom Katalog abweichende Texte sperren Lesen und Schreiben
vor der Textausgabe bzw. Mutation. Auch beschädigte gespeicherte Inhalte dürfen beim Speichern nicht
durch eine Katalogfassung ersetzt werden. Das Win32-Feld wandelt ausschließlich zwischen LF und CRLF
an der Anzeigegrenze. Die UI sendet nur vollständige Katalogfassungen der gebundenen Szene. Der
Datendienst validiert jede Fassung unabhängig.

Der lokale Kanal transportiert logisch je eine Anfrage und eine Antwort in Reihenfolge. Für v1 gilt
UTF-8-JSON, maximal 80 KiB pro Nachricht und 64 KiB UTF-8 pro Text. Rahmengröße vor Allokation
prüfen. Nur die folgenden exakten Schlüssel, Typen und Werte sind zulässig. Doppelte und unbekannte
Schlüssel, Restbytes, ungültiges UTF-8 und unbekannte Commands sperren den Aufruf. Die
authentisierte Transportbindung und ihre Replayabwehr sind im
[BP4b-Kanalvertrag](native-win32-bp4-channel-v1-draft.md) spezifiziert und im reinen Modell geprüft.
Der Gastnachweis steht aus.

| `command`                | Weitere Anfrageschlüssel      | Erfolgsantwort                                                                                      |
| ------------------------ | ----------------------------- | --------------------------------------------------------------------------------------------------- |
| `c4_workspace_status_v1` | keine                         | `{"scene":"A","mode":"edit","revision":0}` (gebundener Katalogeintrag, Modus und aktuelle Revision) |
| `c4_document_read_v1`    | keine                         | `{"text":"Rabe\n42","revision":0}`                                                                  |
| `c4_document_save_v1`    | `expectedRevision` und `text` | `{"revision":1}` nach Commit                                                                        |

Die Namen greifen die drei Operationen des C4-Entwurfs auf. Dieser Forschungsendpunkt verwendet
einen eigenen Protokollbezeichner `kraqel.native.synthetic-scene.v1`; er ist mit der
WebView-Schnittstelle nicht austauschbar. Der Auftrag legt das Ziel fest, auch beim Lesen. Die
Statusantwort gibt nur das synthetische Szenenkürzel aus. Revisionen sind nichtnegative
JSON-Ganzzahlen bis 2³¹−1; beim Erreichen der Grenze bleibt nur Lesen zulässig.

Beispiel für den ersten Schreibzug im Modus `edit` für A:

```json
{ "command": "c4_document_save_v1", "expectedRevision": 0, "text": "Rabe\n43" }
```

Der Dienst prüft Bindung, Modus, zulässigen Ist- und Sollinhalt sowie erwartete Revision innerhalb
derselben serialisierten Speicheroperation. Bei Erfolg werden Dokument, abgeleiteter Klartext,
Suchindex und Revision atomar aktualisiert. Dieselbe Revision kann nur einmal erfolgreich
beschrieben werden. Während eines Schreibzugs wartet die UI auf dessen Abschluss. „Gespeichert“
erscheint erst nach bestätigtem Commit. Bei Revisionskonflikt bleiben die Eingabe im Fenster und der
gespeicherte Stand erhalten; erneutes Lesen und bewusste Auswahl sind erforderlich. Ein
Reply-Verlust oder Absturz kann vor oder nach Commit liegen. Nach neuem Bootstrap zuerst den
gebundenen Stand lesen. Kein automatisches Wiederholen und kein stilles Speichern beim Schließen.

Fehlerantworten haben genau die Form `{"error":"CONFLICT"}`. Weitere zulässige Codes sind `DENIED`
(Bindung/Recht), `INVALID` (Format/Katalog), `LIMIT` und `UNAVAILABLE` (Ablage). Vor erfolgreicher
Kanalbindung wird die Verbindung ohne Nutzdaten beendet. UI und Diagnose dürfen keine fremden
Inhalte, Projektpfade, Schlüssel, SQL-Fehler oder Nachrichtendumps ausgeben. Für die Abnahme genügen
Fallkennung, Ergebniscode, Revision und Prozessende. Narrator und UI Automation bleiben im
Offline-Gast.

## Abnahme in zwei Schritten

| Stufe                      | Vergleich und erwartetes Ergebnis                                                                                                                                                                                                                                                                                                                                                                                                                                                        |
| -------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **BP4a · reines Modell**   | Beide Katalogszenen verlustfrei hin und zurück abbilden. A bei Revision 0 lesen, ändern und als Revision 1 lesen. B bleibt bei 0. Nur-Lesen, veraltete Revision, fremde Ziel-ID, unbekanntes Feld, falscher Katalogtext und gespeichertes Rich Text jeweils ohne Mutation abweisen.                                                                                                                                                                                                      |
| **BP4b · isolierter Gast** | Effektive Rollen-, Datei- und IPC-Rechte mit erlaubter und fremder Prozessgeneration gegenprüfen. Erst dann synthetisches Lesen und Speichern per Tastatur. Ende vor Commit sowie nach Commit vor Antwort prüfen: nach frischer Bindung genau alter oder neuer vollständiger Stand gemäß Haltepunkt. Alte Gegenstelle gesperrt, Jobs leer. Geänderte Binärdateien mit kurzem ETW-Referenz-/Nutzlauf für Starter, Datendienst und UI, vier Loopback-Kontrollen und Verlustprüfung messen. |

**Abgleich mit dem Bestand.** Der bestehende SQLite-Test
`saves_with_optimistic_revision_and_server_derived_plain_text` bestand offline in einer temporären
Quellkopie. Er prüft Speichern, Wiederlesen und die Abweisung einer überholten Revision im
vorhandenen Repository. Cargo verlangte für den direkten Lauf mit `--locked` eine Aktualisierung der
Workspace-Lockdatei. Die temporäre Kopie durfte sie lokal auflösen.

### BP4a · umgesetztes Modell

Das [Rust-Modell](../../scripts/runtime-isolation/native-scene-model/src/lib.rs) hält beide Szenen
ausschließlich im Arbeitsspeicher. Eine exklusive Rust-Ausleihe bindet den Auftrag an einen
Szeneneintrag und Modus. Nur der Prüfcode wählt die Bindung. Das Modell verarbeitet die drei
Commandformen, prüft den vollständigen gespeicherten Dokumentbaum gegen den Szenenkatalog und
ersetzt Dokument und Revision gemeinsam. Nur Katalogtexte können als Textantwort zurückkommen. Die
[Tests](../../scripts/runtime-isolation/native-scene-model/src/tests.rs) prüfen beide Szenen, die
Nur-Lese-Rolle, Konflikte, fremde Ziele, beschädigte Inhalte sowie Nachrichten- und
Revisionsgrenzen. Eine Compiler-Gegenprobe prüft, dass eine alte Bindung nach Neubindung nicht
weiterverwendet werden kann.

Reproduktion ab Repositorywurzel:

```sh
cargo test --locked --offline --manifest-path scripts/runtime-isolation/native-scene-model/Cargo.toml
```

BP4b-Framecodec und Zustandsmodell sind offline geprüft. Persistenz, native Zugriffskontrollen,
Fensteranbindung und ETW-Abnahme des Datenpfads stehen aus. Reale Manuskripte, allgemeines Rich
Text, die Produkt-UI-ABI und eine produktive `DataAllowed`-Freigabe erfordern eine eigene
Entscheidung und Nachweise.
