# P2c C4: Runtime-Paketbindung und minimale isolierte Oberfläche

Aktuell · BP2 nativer synthetischer Blattversuch samt ETW, 18. November 2026:

[BP2-Nachweis](../testing/product-start-p2c-bp2-native-leaf.md): unabhängige native Supportprüfung,
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

Status: **Vertragsentwurf v1, ergänzt am 18. November 2026; C4a-Modellkern implementiert/geprüft,
native UI-Integration und Abnahme offen.**
[C4a-Nachweis und genaue Codierung](../testing/product-start-p2c-c4a-model.md). Ergänzt den
[P2c-Vertrag](product-start-p2c-v1-draft.md), den
[Produktstartvertrag](product-start-integration-v1-draft.md) und
[ADR 0034](../architecture/decisions/0034-webview-network-boundary-proposal.md). MUSS bezeichnet
eine Implementierungsanforderung. Dieser Entwurf erteilt keine neue Start-, Datei-, Netzwerk- oder
Migrationsautorität. Striktes Datenschutzziel und Produkt-Q-01/Q-02 unverändert.

Der [präventive Modul-/Rollenvertrag](product-start-p2c-c4-preload-v1-draft.md) konkretisiert
Creation-/Readback-Anforderungen, unbewiesene Paketbindung, separate Runtime-/Datenzulassung und
zwingende Ablehnungen. BP1 implementiert dazu einen getrennten reinen Modellkern; keine Erweiterung
der aktuellen C4a-V1-Codierung, nativen Storeversion oder Startautorität.

## 1. Ausgangspunkt und erster Lieferumfang

[C1](../testing/product-start-p2c-model.md) liefert reine Typen und Inventarcodierung,
[C2](../testing/product-start-p2c-native-data.md) neue synthetische Daten-/Profilbindungen und
[C3](../testing/product-start-p2c-group-bootstrap.md) einen synthetischen Root-/Kind-Bootstrap.
[C3-ETW](../testing/product-start-p2c-group-bootstrap-etw.md) gilt nur für dessen unveränderte
Binärdateien und geprüfte Abläufe. Native UI-ABI ist weiterhin **0**. C3-Joblimit 2 und geteilter
Gruppenschlüssel sind kein WebView-Vertrag; C2-`no-execution` wird nicht stillschweigend aufgehoben.

Der erste C4-Kandidat ist ein getrenntes Windows-x64-Forschungsprogramm: ein Fenster, ein
synthetisches Projekt, ein Textdokument und eine bereits lokal bereitgestellte Fixed-Version-
Runtime. Anzeigen, revisionsgeprüft speichern, schließen und mit denselben IDs erneut öffnen. Keine
Übernahme echter Projekte, kein normaler Appstart und keine neue Installer-/Updatefunktion. Dieser
Schritt lädt oder installiert selbst keine Runtime. Unsigned bleibt zulässig.

| Baustein       | Vorhanden                              | C4-Anforderung, noch offen                                                         |
| -------------- | -------------------------------------- | ---------------------------------------------------------------------------------- |
| Paketcodierung | C1-Digests, C2-native Fixtureinventur  | Vollständiges ausführbares UI-/Runtimepaket, Versions- und Rollenpolicy            |
| Bootstrap      | C3-synthetische `.kqc3`-Frames         | Eigene UI-Version, Rootkanal ohne Renderer-Schlüsselweitergabe                     |
| Prozessgruppe  | C3-Wächter, durchgehend gehaltener Job | Tatsächliche WebView-Rollen, Token, verschachtelte Jobs und endliche Prozessgrenze |
| Daten          | C2-Textfixtures, persistentes Profil   | Getrennte synthetische SQLite-/WebView-Ablagen und definierter Writer              |
| Oberfläche     | Getrennte Import-Testoberfläche        | Eigener kleiner Commandrouter, keine normale `run()`-Registrierung                 |

Codequellen: [C1-Modell](../../scripts/runtime-isolation/probe/src/product_start_p2c.rs),
[C2](../../scripts/runtime-isolation/probe/src/windows/exchange/ui_restart/durable/product/workspace_persistent.rs),
[C3-Prozessprüfung](../../scripts/runtime-isolation/probe/src/windows/exchange/ui_restart/durable/product/workspace_persistent/group/processes.rs),
[C3-Protokoll](../../scripts/runtime-isolation/probe/src/windows/exchange/ui_restart/durable/product/workspace_persistent/group/protocol.rs),
[normale App](../../apps/desktop/src-tauri/src/lib.rs),
[Releasekonfiguration](../../apps/desktop/src-tauri/tauri.release.conf.json). Die normale App leitet
Projekte aus `app_local_data_dir()` ab und registriert breite Commands; der Release nutzt
`offlineInstaller`. Beides erfüllt diesen C4-Vertrag noch nicht.

## 2. Paketbeschreibung und wirksame Runtimebindung

Im C4a-Modell implementierter Typ `UiRuntimePackageV1`, strikt versioniert mit begrenztem Decoder.
Die Beschreibung liegt authentisiert im Hostbereich; ein Pfad oder ein selbst behaupteter
Versionsstring ist kein Paketnachweis. Sie referenziert das C1-UI-Inventar, statt eine zweite
Dateidigestcodierung einzuführen.

| Pflichtinhalt                                                          | Bedeutung                                                                                                   |
| ---------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------- |
| `version=1`, Zweck `c4-minimal-ui`, Architektur `windows-x64`          | Eigene Vertragsdomäne, keine C3-/Importkompatibilität                                                       |
| Host-/UI-Inventardigest, relative UI-EXE und Runtime-Unterwurzel       | Ausschließlich nativ unter gebundenen Roots auflösen                                                        |
| Runtime-Buildversion, Loader-Identität, Frontend-/Konfigurationsdigest | Vollständige Bytes einschließlich DLLs, Ressourcen, Locale-/Datendateien; statischer Loader über EXE-Digest |
| Bootstrap-, Command-, Prozess-/Job-, ACL- und Umgebungspolicydigest    | Wirksame Policies statt frei ergänzbarer Argumente                                                          |
| DB-Lese-/Schreibschema, WebView-Datenkompatibilität, Bindungsrevision  | Start und Paketwechsel nur bei ausdrücklicher Kompatibilität                                                |
| Endliche Gesamtprozessgrenze und geschlossene Rollenliste              | Konkrete Werte erst mit dem lokalen Runtimekandidaten und Rollenbeleg festlegen                             |

Paketgesamtidentität: SHA-256 über Domain `KRAQEL-C4-RUNTIME-PACKAGE\0\x01` und die in C4a
implementierte deterministische längenbegrenzte Codierung aller Pflichtinhalte (siehe Nachweis).
Keine beiläufige `serde_json`-Ausgabe als kanonischer Vertrag. Inventargrenzen des C1-Vertrags
bleiben obere Grenzen; die engeren C2-Fixturegrenzen werden nicht automatisch erhöht. Überschreitung
sperrt; ein echtes Runtimepaket benötigt vor Start einen eigenen begründeten nativen Grenzvertrag.

Der Wächter MUSS unter derselben Kontrollsperre exaktes Inventar, native Rootidentitäten und
ACL-Policy prüfen und Austausch/Löschen während der Laufzeit verhindern. Keine zusätzlichen ladbaren
Dateien, Hardlinks, Reparsepunkte, Netzpfade oder Laden aus Daten-/Tempbereichen.
Windows-Systembibliotheken sind eine ausdrücklich dokumentierte Plattformabhängigkeit; daraus folgt
keine Inventarisierung oder Unveränderlichkeitszusage für Windows selbst.

Die native UI MUSS die Runtime über den gebundenen expliziten Browserpfad auswählen, WebView-
Arbeitsdaten separat angeben und geerbte Runtime-/Debug-/Argument-Overrides ausschließen. Wirksame
Umgebungs- und Registry-/Policy-Overrides werden vor Erstellung ausgewertet; unbekannte oder
widersprüchliche Werte sperren. Keine Registry-/Domänenänderung. Eine Environmentvariable oder
Versionsabfrage allein genügt nicht: tatsächliches Browserimage, Loader und zugehörige geladene
Nichtsystemmodule müssen zum Paket passen. Eine fehlende Runtime führt zum Abbruch ohne
Evergreen-/Installations-/Browserfallback.

Microsoft dokumentiert explizite Runtime-/Datenpfade sowie Environment-/Registry-Overrides in der
[Win32-API](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/webview2-idl?view=webview2-1.0.4258.31).
Daraus leitet dieser Vertrag die zusätzlichen nativen Abgleichpflichten ab; die API garantiert keine
Paketattestierung.

Falls der konkrete Runtimekandidat allgemeine AppContainer-RX-Rechte braucht, sind sie
ausschließlich am nicht geheimen Runtime-Unterbaum inventarisiert zulässig. Nie an Hostpaket,
Kontrolle oder Arbeitsdaten; keine ACL-Aufweitung beim Fehler. Solche Runtimeanforderungen und die
eigene Pflege von Fixed-Version-Paketen beschreibt die
[Distributionsdokumentation](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/distribution).
Pinning ersetzt weder Sicherheitsupdates noch lokale Kompatibilitätsprüfung.

## 3. Rollen, Schlüssel und Datenrechte

| Rolle                                                                          | Erlaubte Autorität                                                                    | Ausgeschlossen                                                                           |
| ------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------- |
| Nativer Wächter außerhalb der UI-Gruppe                                        | Kontrollschlüssel/-sperre, Paketprüfung, Start-/Endrecords, gehaltenes Jobhandle      | WebView oder Parser für Manuskript-/Archivnutzlasten                                     |
| Nativer UI-Root im eingeschränkten Job                                         | Geprüfter Startgrant, kleiner Commandrouter, synthetische DB, eigene WebView-Umgebung | Kontrollschlüssel/-dateien, Updates, Importfreigaben und beliebige Hostpfade             |
| Browser/Renderer/GPU/Utility und weitere ausdrücklich gebundene Runtime-Rollen | Erforderliche Paket-RX-/WebView-Arbeitsdatenrechte nach Rollenpolicy                  | Bootstrap-/Kontrollschlüssel, geerbte Host-/Job-/Sperrhandles und privilegierte Commands |

Nur der native Root erhält die ausgewählten anonymen Bootstraphandles und einen kurzlebigen
Rootschlüssel. Vor Tauri/WebView MUSS er Vererbungsflags löschen und zurückprüfen, Bootstrap
abschließen, Schlüssel löschen und nicht mehr benötigte Handles schließen. Keine Schlüssel in
Frontend, URL, Argumenten, Environment oder Diagnosen. C3-Frames/Schlüssel werden nicht delegiert.
Ein MAC beweist den gebundenen Kanalkontext, nicht die individuelle schreibende PID; nativ gehaltene
Prozessidentität, Kanalbesitz und abgeschlossene Vererbung sind zusätzliche Pflichten.

Profil-SID und Jobzugehörigkeit allein trennen Root und Renderer nicht voneinander. Gleiche
effektive Tokenrechte können Dateirechte teilen. Der erste Kandidat darf daher keine unabhängige
Renderer-zu-DB-Isolation behaupten. Der native Root ist der einzige vorgesehene Anwendungswriter;
eine zweite UI-Generation wird vor Start unter der Kontrollsperre abgewiesen. Welche zusätzlichen
Runtime-SIDs, Restricted Tokens und Integritätslabels tatsächlich wirken, bleibt ein natives
C4b-Gate. Soll ein Renderer technisch keinen direkten DB-Zugriff besitzen, muss dies separat
durchgesetzt und geprüft werden; eine Originprüfung oder ein Rust-Mutex beweist diese Trennung
nicht.

Persistenzbereiche: `projects/` für eine synthetische SQLite-Datenbank samt WAL/SHM/Backups,
`webview/` für persistente Browserdaten und ein begrenzter sitzungseigener Tempbereich. Sie bleiben
vom unveränderlichen Paket und privaten Hostbereich getrennt. Keine Rechte auf reale Appdaten,
Kontrollablage oder alte C2/C3-Fixtures erweitern. Keine allgemeinen AppContainer-Datenrechte.
Root-/Pfadauflösung und ACLs auch bei veränderlichen Unterpfaden nativ prüfen. Persistente SQLite-
Inhalte werden nicht bei jedem Start als unveränderlicher Paketdigest behandelt.

## 4. Start, Runtimekinder und Ende

1. Exklusive Kontrollsperre übernehmen; Head, Budget, Paket-/Datenbindung, Profil und alle
   Kompatibilitätspolicies prüfen. Aktiver/unklarer alter Launch, Update oder Migration sperrt.
2. Eigene neue UI-Record-/Bootstrapversion und benötigte Recordslots vor Ressourcenanlage
   reservieren. Launch-ID/Generation dauerhaft verbrauchen. Kein Schreiben neuer UI-Records als
   `.kqc3` und kein stiller Schema-6-Umbau. C4a definiert eigene
   Policy-/Paket-/Bindungs-/Modellrecorddomains und Version 1, ohne native Storeleser zu erweitern.
   Diese Simulationsrecords sind keine nativen Lebenszyklusbelege; deren
   Persistenz-/Peer-/Root-/Rollenintegration und explizite alte Leserabwehr bleiben C4b.
3. Root suspendiert atomar im eigenen Job erzeugen. Image/Token, native PID/Erzeugungszeit,
   Handleliste, Profil, null Netzwerk-Capabilities, Jobpolicy und erforderliche wirksame frühe
   Lade-/Startbeschränkungen prüfen; RootBound vor Resume. Fehlende präventive Modul-/Rollenbindung
   sperrt gemäß ergänzendem Vertrag. Eine nachträgliche Modulaufnahme schließt diese Lücke nicht.
4. Root validiert den vollständigen Startgrant vor Framework-, DB- und WebViewinitialisierung.
   Wächter persistiert Accepted/Running vor authentisiertem Continue. Dies allein erlaubt keine
   Projektöffnung: erst nach zusätzlicher belegter Runtime-/Rollen-/Datenzulassung gemäß Schritt 6
   gebundene synthetische Projektdaten öffnen. Die normale `run()`-Funktion ist kein Ersatzpfad.
5. Runtime erstellen, ohne Netzwerk-/Loopback-Ausnahme, Breakaway oder Sandbox-Abschaltung.
   Ausschließlich belegte Rollen/Images/Argumentpolicies; verschachtelte Jobs erhalten die äußere
   Wächterbindung. Die C3-Grenze 2 darf nicht einfach entfernt werden. Fehlender Rollenbeleg oder
   Prozessbudgetüberschreitung führt zum kontrollierten Ende der eigenen gesamten Gruppe.
6. Vor Projektöffnung, Textfreigabe und Commandaktivierung tatsächliche Runtimeauswahl und alle bis
   dahin entstandenen Rollen abgleichen sowie effektive Datenrechte prüfen. Präventive Durchsetzung
   MUSS auch für zukünftige Rollen/Nachladungen bestehen; spätere Beobachtung allein reicht nicht.
   Tokens können durch die Runtime weiter eingeschränkt sein; erforderlich sind belegte effektive
   Beschränkungen, nicht ungeprüft identische Profil-SIDs aller Kinder.
7. Beim Schließen Commands sperren und Transaktionen beenden; Root-Ende gibt keinen Workspace frei.
   Wächter hält Lease/Job bis zum nativen Leerstand derselben kontinuierlich gehaltenen Jobinstanz,
   schreibt Drained/Finish und erhält Profil/Daten. Ein begrenzter Shutdown-Timeout darf nur die
   eigene gehaltene Gruppe beenden; ohne bestätigten Drain bleibt pending.

**Offenes technisches Stopgate C4b:** WebView erzeugt eigene Kinder; die C3-Strategie „jedes echte
Kind selbst suspendiert anlegen und vor Resume prüfen“ ist nicht bereits für diese API belegt. Vor
Runtimeausführung müssen Job-/Token-/Handlebeschränkungen wirksam sein. Nachträgliche
Prozessbeobachtung oder ETW ist kein Ersatz für diese Durchsetzung. C4b muss den unterstützten
Erzeugungsweg und die Beweiskette einschließlich dynamischer Rollen nachweisen. Ist dies nicht ohne
Lockerung möglich, bleibt C4 gesperrt und die Architekturfrage offen; kein unisolierter Test mit
Projektdaten als vermeintlicher Integrationsfortschritt.

Das
[WebView-Prozessmodell](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/process-model)
beschreibt Browser-, Renderer- und Hilfsrollen sowie variable Prozesszahlen und gemeinsame
Prozessgruppen bei gleichem Datenordner. Deshalb verlangt C4 einen eigenen gebundenen Datenordner
und einen belegten Rollen-/Prozessvertrag statt einer fest angenommenen Kindzahl.

Nach Wächtercrash bleibt der Launch wie in C3 gesperrt, auch bei verschwundenem Profil oder
beendeter Root-PID. Kein Neuaufbau eines gleich benannten Jobs als Endbeleg, keine Rekonstruktion
aus Diagnose-JSON und keine automatische Recovery oder Löschung. Abgelehnte Starts dürfen Daten,
Bindungsrevision und Kontrollhead nicht reparieren. Normales Wiederöffnen nach bestätigtem Finish
verwendet dieselben Workspace-/Projekt-/Dokument-IDs und dasselbe Profil, aber neue Launch-ID und
Generation. Runtime-/Paketwechsel sind erst bei Drain und eigener authentisierter Bindungsänderung
zulässig; inkompatible Browserdaten/DB sperren ohne Löschen oder ungeprüften Downgrade.

## 5. Minimale Oberfläche und Commandvertrag

Eine lokale gebündelte Seite, ein Fenster, keine Remoteinhalte, Subframes, Popups, Downloads,
externen Navigationen, Dateiauswahl, Shell-/URL-Aufrufe, Devtools oder Debugports. Ressourcen werden
aus dem gebundenen Paket bedient; kein Devserver-/Loopback-Netzdienst. CSP und Navigation-/
Permissionhandler ergänzen die native Isolation, ersetzen sie nicht. Fehlende Plattformrechte führen
nicht zum Aktivieren allgemeiner Tauri-/Plugin-Capabilities.

Geplante Commandversion 1; Aufruf nur aus dem gebundenen Hauptfenster und lokaler Top-Level-Origin
nach abgeschlossenem Bootstrap. Strikte typisierte Nutzlasten ohne unbekannte Felder, höchstens 80
KiB pro Anfrage/Antwort; ein Text höchstens 64 KiB UTF-8. Keine Dateipfade, Handles,
Paket-/Kontrollschlüssel oder freie SQL-/Commandtexte über IPC. Namen sind Zielvertrag, noch keine
registrierten Commands.

| Command                  | Eingabe und Ergebnis                                                                    |
| ------------------------ | --------------------------------------------------------------------------------------- |
| `c4_workspace_status_v1` | Keine Nutzlast; synthetische Projekt-/Dokument-ID, Revision und begrenzter Zustandswert |
| `c4_document_read_v1`    | Keine frei wählbare ID; das eine gebundene Dokument mit Text und Revision               |
| `c4_document_save_v1`    | Erwartete Revision und Text; atomare Transaktion, neue Revision erst nach Commit        |

Schließen erfolgt über reguläres Fensterende. Kein allgemeiner Brokercommand oder Start-/
Importgrant aus React. Rust leitet DB-/Dokumentziel nur aus geprüftem nativem Zustand ab.
Revisionkonflikt verändert keine Daten, Überschreitung wird vor Transaktion abgewiesen. Gleiche
Revision darf nicht zweimal überschrieben werden; unbekannter Commitausgang verlangt erneutes Lesen
statt blindem automatischem Retry. Fehler bleiben bereinigt und stabil, etwa `C4_REVISION_CONFLICT`,
`C4_LIMIT_EXCEEDED`, `C4_NOT_READY` und `C4_STORAGE_FAILED`. Vor-GUI-Startfehler erscheinen über
einen getrennten lokalen nativen Fehlerpfad, der weder WebView noch Workspaceöffnung benötigt. Keine
internen Pfade/Diagnosen in der sichtbaren Fehlermeldung.

Oberfläche: Projekt-/Dokumenttitel, beschriftetes Textfeld, Speichern und sichtbare Statusmeldung.
Klare Anzeige ungespeicherter Änderungen, sinnvolle Tabfolge und Fokus, Fehler-/Erfolgsmeldungen
über passende Live-Region. Vor Abschluss eigener nativer Tastatur-/Hörcheck mit Sprachausgabe;
historische Importfensterprüfungen werden nicht als C4-Abnahme übernommen.

## 6. Umsetzung und mittleres Prüfniveau

| Schritt              | Konkreter Lieferumfang                                                                                                                | Gezielt erforderlicher Nachweis                                                                                                                  |
| -------------------- | ------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------ |
| C4a — Modell geprüft | Reine Runtimepaket-/Policydeskriptoren, deterministische Codierung, getrennte UI-Version und Lebenszyklusmodell; keine Startautorität | Rundlauf und Mutation/Unbekanntversion, Grenzen sowie alte C2/C3-Bindung verweigert; Format-/Faktenprüfung                                       |
| C4b                  | Native Paket-/Overrideprüfung und unterstützter Mehrprozess-Erzeugungsweg mit neuer synthetischer Ablage                              | Ein regulärer Rollenlauf; manipuliertes Paket/Override, falsche Bindung und Kind-/Jobabweichung; keine echte Vorschauablage                      |
| C4c                  | Minimale Oberfläche, drei Commands und synthetische SQLite-Persistenz                                                                 | Speichern/Schließen/Wiederöffnen mit IDs, Revisionskonflikt, fremde Origin/Command und zweiter Start; ein gezielter assistiver Lauf              |
| C4d                  | Abschluss des exakten Kandidaten                                                                                                      | Frisches ETW: Referenz und gezielter Arbeits-/Fensterende-/gesperrter Wiederanlaufpfad; Kontrollen, Verluste und alle Prozessgenerationen prüfen |

Keine breite neue Crashmatrix pro Teilstück. Ein gezielter Wächterverlust reicht zunächst für die
bestehende Sperrregel. Runtime-/Rollen-/Datenfehler dürfen das Datenschutzziel nicht relativieren.
Bei ausschließlich isolierten Helferänderungen genügen die relevanten Tests/Clippy und wenige native
Fälle; bei App-, gemeinsamem Transport- oder Packagingcode gilt weiterhin `pnpm check`. ETW ist
erforderlich für geänderte tatsächlich gestartete Binaries, nicht für diesen reinen Entwurf. Nur
bereinigte Aggregate nach Git, Rohdaten/Schlüssel/IDs lokal. Außerhalb des Baums laufende
COM-/Shell-/Systemdienste bleiben eine eigene Nachweisgrenze.

Historisch wurde C4a als reiner Modellkern mit zwölf neuen Tests geprüft. Inzwischen sind native
C4b-Vorprüfung, leerer Forschungs-Root-/Runtimepfad, Modulsperre und die jeweils bytegebundenen
ETW-Nachweise ergänzt; aktuelle Belege stehen oben. Das ist keine native produktive UI-/
Projektfreigabe. BP1 implementiert inzwischen reine Typen, Codierung und nicht autorisierendes
Modell; native Umsetzung des präventiven Vertrags bleibt offen. BP2 ist nun im begrenzten nativen
Umfang geprüft; als Nächstes BP3 ohne automatische UI-/Runtimefreigabe aus
Modell-/Fähigkeitsbehauptungen. Neue BP1-Prüfzahlen im verlinkten Nachweis; keine Migration oder
ETW-Abnahme daraus. Gesamtschätzung unverändert etwa 65 %, nutzbarer 0.1-Umfang etwa 80 %; keine
Zeit- oder Produktionsreifequote. C4b insgesamt, P2, P3–P5, Langzeit-Kontrollstore und
Produkt-Q-01/Q-02 bleiben offen.
