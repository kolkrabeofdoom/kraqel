# P2c C3: synthetischer Mehrprozess-Bootstrap mit gehaltenem Wächter

Der C4-Vertrag ist durch den [reinen C4a-Modellkern](product-start-p2c-c4a-model.md) ergänzt:
Paket-/Policytypen, deterministische Codierung und versionierter Lebenszyklus, zwölf neue Tests.
Keine native UI-/Startautorität; native UI-ABI bleibt 0. Nächster Schritt ist C4b mit nativer
Paket-/Overrideprüfung und belegbarem eingeschränktem Mehrprozess-Erzeugungsweg. C3-Bytes und deren
eigener ETW-Beleg bleiben separat unverändert.

Stand: 17. November 2026. **Synthetischer C3-Kern implementiert und gezielt nativ geprüft; keine
Oberfläche oder Produktfreigabe.** [Bereinigte Aggregate](product-start-p2c-group-bootstrap.json),
[C2-Datenbindung](product-start-p2c-native-data.md),
[P2c-Zielvertrag](../data-format/product-start-p2c-v1-draft.md).

## Implementierter Umfang

Eigener
[C3-Kern](../../scripts/runtime-isolation/probe/src/windows/exchange/ui_restart/durable/product/workspace_persistent/group.rs)
mit getrennten Modulen für
[Frames](../../scripts/runtime-isolation/probe/src/windows/exchange/ui_restart/durable/product/workspace_persistent/group/protocol.rs),
[native Prozess-/Jobwächter](../../scripts/runtime-isolation/probe/src/windows/exchange/ui_restart/durable/product/workspace_persistent/group/processes.rs),
[Root-/Kindrollen](../../scripts/runtime-isolation/probe/src/windows/exchange/ui_restart/durable/product/workspace_persistent/group/worker.rs)
und
[Tests](../../scripts/runtime-isolation/probe/src/windows/exchange/ui_restart/durable/product/workspace_persistent/group/tests.rs).

Der normale native Controller bleibt als Wächter bis zum tatsächlichen Gruppenende aktiv. Er hält
denselben privaten `Store`, `owner.lock`, MAC-Schlüssel und dieselbe neu erzeugte Jobinstanz über
den gesamten Ablauf. Kein Dienst installiert, kein benannter IPC-Endpunkt, kein UI-Start oder
App-Command hinzugefügt.

- Frische C2-DATA-Stores, keine Übernahme alter Ablagen/Actors oder Migration. Eigene readonly
  `.kqc3`-Deklaration: synthetischer Gruppenvertrag 1 für Schema 6, **UI-ABI 0**. Bestehende
  PE-Deklarationen werden nicht zum UI-Startvertrag umetikettiert.
- Neue Codefixture `c3-code/worker.exe` statt ACL-Promotion der C2-Ressourcenfixture. Vor Ausführung
  tatsächliche Paketbytes, C3-Deklaration, Inventar, Rootidentität und Profilrechte prüfen und
  Paketdateien/Vorfahren gepinnt halten. Root und Kind verwenden dieses unveränderliche Programm als
  Arbeitsverzeichnis; Umgebungswerte stammen aus der begrenzten Forschungsumgebung.
- Ein Job mit **ActiveProcessLimit 2**, KillOnJobClose und ohne Breakaway. Root wird bereits bei
  `CreateProcessW` dem Job zugeordnet und bleibt zunächst suspendiert. Native Prüfung von PID/
  Erzeugungszeit, tatsächlichem Image/Hash, Profil-SID, null Capabilities und privater Dateisperre
  vor Resume. Unzugeordneter Root wird nie fortgesetzt; eigenes Ende muss bestätigt werden.
  Unbestätigtes Ende eines außerhalb des Jobs erzeugten Roots lässt den Auftrag pending.
- Root erzeugt nach Continue ein **echtes direktes Kind** suspendiert. Wächter prüft dessen
  tatsächliche Identität, Eltern-PID, Threadzugehörigkeit, Code, Token, private Dateisperren und
  Zugehörigkeit zur gehaltenen Jobinstanz vor dauerhafter ChildBound-Bindung und Resume.
  Jobmitglieder werden gegen die erlaubten Root-/Kind-PIDs abgeglichen.

## Authentisierung und dauerhafte Reihenfolge

Eigener Probevertrag, kein implementierter C1-UI-ABI-1-Startgrant. MAC-Domain
`KRAQEL-C3-SYNTHETIC-GROUP\0\x01`; höchstens 8.192 JSON-Bytes pro Frame mit fester Länge/MAC,
strengem Feldschema und höchstens 30 Sekunden Wächterfrist. Frames werden einzeln begrenzt gelesen;
ein geerbter Writer kann das Ende nicht durch ausbleibendes EOF blockieren.

Grant bindet Zweck, Root-/Kindrolle, Launch/Generation, Wächter-PID/Erzeugungszeit/Epoche, Nonce,
Workspace/Profil, Codeidentität, Digest der C2-Bindung und synthetische Ausgangsdaten. Root prüft
die nativen Pipe-Server-PIDs von Bootstrap/Schlüssel/Continue, Profil, Jobzugehörigkeit und eigene
Paketbytes. Vier explizite Root-Handles: Bootstrap-Lesen, Einmalschlüssel-Lesen, Ereignis-Schreiben,
Continue-Lesen. Kein Job-/Kontroll-/dauerhafter Schlüsselhandle wird vererbt.

Root quittiert Accepted; erst nach dessen dauerhafter Speicherung und Running-Record sendet der
Wächter das authentisierte Continue. **Vor Continue wird keine Arbeitsdatei geöffnet.** Danach liest
Root die synthetische C2-Datei und schreibt nur in eine neue auftragsgebundene Datenfixture. Nur
dieses neue Verzeichnis erhält geprüfte Low-Integrity-Kennzeichnung und profilgebundene
Modify-Rechte. Bestehende C2-/private Ablagen werden dafür weder neu gelabelt noch freigegeben.

Das Kind erhält drei ausdrücklich ausgewählte, vom Root delegierte Probehandles und denselben
kurzlebigen **Gruppenschlüssel**. Es prüft den Root als Pipe-Erzeuger, seine Rolle, Profil und Code;
es startet erst nach separatem nativem ChildBound-/Resume-Gate des Wächters. Root gibt den Writer
für die Kindquittung weiter und beendet sich. Die MAC bindet den Gruppenkontext, **beweist allein
keine unabhängige Schreiber-PID**. Deshalb sind echte Prozess-/Thread-/Token-/Jobprüfungen
unverzichtbar. Das ist keine Freigabe dieses Delegationsmodells für untrusted Renderer oder
beliebige WebView-Kindrollen; deren getrennte Autoritäten bleiben Integrationsarbeit.

| Record             | Native Grenze                                                                       |
| ------------------ | ----------------------------------------------------------------------------------- |
| CodeBound          | Neue gepinnte Codefixture, noch kein Start                                          |
| Commit             | Einmaliger Launch, nächste Generation, frische Epoche/Nonce, echte Wächteridentität |
| Staged             | Neue eigene Datenfixture, tatsächliche Rootidentität und Low-Labelprüfung           |
| RootBound          | Exakter suspendierter Root vor Resume                                               |
| Accepted / Running | Root-only-Job, erneute native Identitäts-/Token-/Paketprüfung; danach Continue      |
| ChildBound         | Exaktes suspendiertes Kind, bevor der Wächter dessen Thread fortsetzt               |
| Drained            | Dieselbe kontinuierlich gehaltene Jobinstanz tatsächlich ActiveProcesses=0          |
| Finish             | Erst nach Drained; Erfolg oder verbrauchter fehlgeschlagener Bootstrap              |

Acht Launchrecords plus zwei Slots für spätere explizite Profilstilllegung werden vorab
vorausgesetzt. Allgemeine Storemutationen, C2-Zugriffe/Stilllegung und neue Starts sind bei pending
C3 unabhängig von einem inzwischen verschwundenen Jobnamen gesperrt. Der aktive Recordpfad prüft
echte Kontrollsperre, Wächteridentität/Epoche, Jobpolicy/Mitglieder und bei Prozessrecords erneut
Identität/Token/Paket. Serialisierbare Events allein erteilen keine Autorität.

**Root-Ende ist kein Gruppenende:** Vor Kind-Resume wird Root-Ende bestätigt, während das Kind noch
suspendiert lebt und ActiveProcesses=1 gilt. Zweiter Storezugriff und Profilstilllegung bleiben
gesperrt. Erst nach Kindquittung/-ende beziehungsweise kontrollierter Terminierung der gesamten
eigenen Gruppe und nativem Leerstand folgen Drained/Finish. Profil/Daten überleben normale
Gruppenabschlüsse; Stilllegung bleibt eine eigene Operation.

## Gezielte Prüfung und gesicherter Build

Controller/Codefixture: `b24b3aac3d80bc59b63c74e2cbad03bf4980d8496b88434f3afc035fdbbe48af`. Lokale
Sicherung `%TEMP%/kraqel-product-p2c-group-build-20261117-01`. Keine neue EXE an echter
Vorschauablage autorisiert.

```powershell
cargo test --locked --offline --manifest-path scripts/runtime-isolation/probe/Cargo.toml --quiet
cargo clippy --locked --offline --manifest-path scripts/runtime-isolation/probe/Cargo.toml --all-targets -- -D warnings
cargo fmt --manifest-path scripts/runtime-isolation/probe/Cargo.toml -- --check
./scripts/runtime-isolation/check-workspace-group.ps1 -Probe <gesicherter-Controller> -ProbeSha256 <obiger-Hash>
```

Vier neue Tests für strikte MAC-/Framerollen/Nonce, Fristen ohne EOF-Abhängigkeit,
Reihenfolge/Replay/Pending und eigene Paketdeklaration/abweichenden Wächter. Insgesamt **96 aktive
Helfer-/Transporttests bestanden, vier bewusst ignoriert**, Clippy grün.

Gesicherter nativer Ablauf: zwei erfolgreiche Root-/Kindgruppen, falsche Root-MAC, unzugeordneter
nie fortgesetzter Root und falsche Kind-MAC; danach erneut erfolgreiche Generation und explizite
Stilllegung. Beide Erfolgsfälle bestätigen je zwölf tatsächliche private Dateisperren, null
Capabilities und Datenlesen/-schreiben im Lowbox-Kontext. Drei abgewickelte Bootstrapfehler werden
als `passed=false` gemeldet; der Controlleraufruf selbst bleibt dabei erfolgreich. Alter Head wird
als echter Controllerfehler abgewiesen.

Zusätzlich **ein gezielter harter Wächterabbruch mit Exit 86**: nach Root-Ende und ChildBound, Kind
noch suspendiert. Neustart zeigt pending und verweigert Start/Profilstilllegung. Ein ausschließlich
eigener Fixturecleanup im Prüfablauf prüft die beiden bekannten nativen Prozessgenerationen auf Ende
und entfernt sein eigenes Testprofil. **Das ist keine Broker- Reconciliation oder allgemeine
Mehrprozess-Endautorität.** Brokerhistorie bleibt unverändert pending und auch nach
Profilabwesenheit gesperrt; keine Rekonstruktion aus PID-Abwesenheit. Keine breite Crashmatrix.

Zusammen 22 Controlleraufrufe: 16 reguläre Rückgaben, fünf erwartete Abweisungen und ein harter
Abbruch. Unabhängiger lesender Audit bestätigt **51 MAC-Records, sechs verbrauchte Launches, fünf
Root-/vier Kindbindungen, fünf Drained-/Finish-Paare**, Code-/Inventarbytes und erhaltene
C2-Ausgangsdaten. Zwei Profile entfernt: eines durch authentisierte Stilllegung, eines ausdrücklich
als eigener Testfixturecleanup. Schlüssel bleiben erhalten; Ablehnungen und Fixturecleanup verändern
keine Kontrollhistorie, echte Vorschau-Kontrollhashes bleiben unverändert. Ein negativer
Kontrollstore bleibt absichtlich gesperrt. Prozesse beendet, Rohdaten/IDs/Schlüssel nur lokal;
bereinigte Aggregate in Git.

## Verbleibende Gates

[Frisches C3-ETW](product-start-p2c-group-bootstrap-etw.md) ist inzwischen für denselben
unveränderten Build ergänzt; der C2-Beleg für `73a7aad…` wird nicht übertragen. Die oben genannten
Funktionsprüfungen bleiben der ursprüngliche Implementierungsnachweis und wurden dafür nicht
wiederholt. Keine tatsächliche UI, WebView, SQLite-Schreiboberfläche, vollständige Runtime-
Paketclosure, assistive GUI-Abnahme oder automatische Recovery nach Wächterverlust. Der Wächter ist
heute ein residenter Forschungsaufruf, kein normaler Appstarter/Produktdienst. Nicht als neue
Produktfreigabe oder universelle Netz-/Dienst-/Inhaltsattestierung verwenden.

Der C4-Vertrag ist oben ergänzt. Nächster begrenzter Schritt nach dem C4a-Modellkern: native
Paket-/Overrideprüfung und belegbarer eingeschränkter Mehrprozess-Erzeugungsweg in C4b. P2
insgesamt, vollständige C3/C4-Produktintegration, P3–P5 und Produkt-Q-01/Q-02 bleiben offen. Das
strikte Datenschutzziel bleibt unverändert.
