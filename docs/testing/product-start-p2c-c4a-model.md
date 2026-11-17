# P2c C4a: reine Runtimepaket-, Policy- und Lebenszyklusmodelle

Stand: 17. November 2026. **C4a implementiert und als Modell geprüft; keine native UI-
Startautorität, Paketattestierung oder Produktfreigabe.**
[C4-Vertrag](../data-format/product-start-p2c-c4-v1-draft.md),
[Modul](../../scripts/runtime-isolation/probe/src/product_start_p2c/c4.rs),
[Tests](../../scripts/runtime-isolation/probe/src/product_start_p2c/c4/tests.rs).

Nur der bestehende Forschungsbinärbaum deklariert das neue Untermodul. Der gemeinsame Transport,
App, native Storeleser, PE-/UI-ABI, Abhängigkeiten und Lockfiles bleiben unverändert. Das Modul
verbietet `unsafe`, führt keine Datei-/Registry-/Prozess-/Netzoperationen aus und wird von keinem
nativen Command aufgerufen. Native UI-ABI bleibt **0**; C2-`no-execution` und `.kqc3` werden nicht
erweitert. Paket-/Policydigests sind übergebene Metadaten, keine Nachweise tatsächlicher Bytes,
vollständiger Inventare oder wirksamer Token-/Job-/ACL-Beschränkungen.

## Typen und deterministische Codierung

`UiRuntimePackageV1` bindet Host-/UI-Inventardigest, relative UI-/Runtimepfade, vierteilige
Runtimeversion, statischen oder dynamischen Loader, Frontend/Konfiguration, die vollständige
`UiPolicyV1`, DB-Lese-/Schreibbereich, WebView-Datenversion und Bindungsrevision. Zweck und
Architektur sind fest auf `c4-minimal-ui`/Windows-x64 codiert. Kein frei wählbarer Ersatzpfad oder
permissiver Netzwerk-/Sandboxschalter. Das ist kein Verfügbarkeits- oder Lizenznachweis einer
tatsächlich installierten Fixed-Version-Runtime.

`RolePolicyV1` bindet je Rolle Image, Instanzgrenze, Argument- und Tokenpolicydigest. Geschlossene
Rollenwerte: Root=1, Browser=2, Renderer=3, GPU=4, Utility=5, CrashHandler=6. Root/Browser/Renderer
sind erforderlich, Root hat genau eine Instanz. Höchstens sechs Rollen, keine Rollendubletten, Summe
der Instanzgrenzen höchstens Gesamtprozessgrenze; diese liegt zwischen 3 und 64. Das sind
Modellgrenzen, kein gemessener WebView-Prozessbedarf. Runtimekandidat und native Rollenpolicy müssen
C4b noch festlegen. Runtimeimages liegen unter der angegebenen Runtimewurzel; der Root entspricht
der UI-EXE und liegt außerhalb dieser Wurzel. Relative Pfade verwenden die C1-NFC-/
Windows-Namens-/Aliasregeln; native Dateinamen-/Dateiidentität bleibt separat zu prüfen.

Codierung ohne JSON-Abhängigkeit: sämtliche Zahlen little endian, Digests 32 rohe Bytes, IDs 16 rohe
Bytes, Strings u16-UTF-8-Bytelänge plus Bytes. Pfade höchstens 1.024 Bytes/Tiefe 32 wie C1. Jeder
Eingabedecoder höchstens 16 KiB; unbekannte Versionen, Rollen, konstante Policywerte, Abbruch und
Nachlauf werden abgewiesen. Encoder sortieren Rollen nach Rollenwert; Decoder verlangen die
kanonische Reihenfolge durch vollständigen Reencodevergleich.

| Domain                              | Reihenfolge nach der Domain                                                                                                                                                                                                                                                                                                                              |
| ----------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `KRAQEL-C4-UI-POLICY\0\x01`         | Vier u16: Policy-, Bootstrap-, Command-, Modellrecordversion (je 1); u16 Gesamtprozessgrenze; fixes Policybyte 1; Job-/ACL-/Environment-/Commanddigest; u8 Rollenanzahl; pro Rolle u8 Rolle, Imagestring, u16 Instanzgrenze, Argument-/Tokendigest                                                                                                       |
| `KRAQEL-C4-RUNTIME-PACKAGE\0\x01`   | u16 Version 1; Zweck-/Architekturbyte je 1; Host-/UI-Inventardigest; UI-EXE-/Runtimewurzelstring; vier u16 Runtimeversionskomponenten; Loaderbyte 1=statisch oder 2=dynamisch mit Pfadstring/Digest; Frontend-/Konfigurationsdigest; u16 Policylänge plus vollständige Policybytes; vier u16 DB-Min/Max/Write/WebView-Datenversion; u64 Bindungsrevision |
| `KRAQEL-C4-WORKSPACE-BINDING\0\x01` | Drei u16 C1-Bindungs-/Kontrollschema-/Ziel-UI-Version; Installations-/Workspace-ID; Profilname/-SID als Strings; pro Projekte-/WebView-/Sessionroot Pfadhash, u64 Volume, Datei-ID; Host-/UI-/Konfigurations-/Command-/Job-/ACL-/C4-Paketdigest; vier u16 Workspaceformat/DB-Min/Max/Write; u32 Prozessgrenze; u64 Revision                              |
| `KRAQEL-C4-UI-MODEL\0\x01`          | u16 Version 1; u8 Schritt; Workspace-ID; u64 Bindungsrevision; Paket-/vollständiger Bindungsdigest; Launch-ID; u64 Generation; erwarteter und nächster Modellhead                                                                                                                                                                                        |

SHA-256 über Domain und vollständige Codierung ergibt Policy-/Paket-/Bindungsdigest. Die native
Root-/Inventarprüfung wird damit nicht ersetzt. Vor Modelleinrichtung müssen Paket und vollständige
C1-Workspacebindung hinsichtlich Inventaren, Konfiguration, Commands, Job-/ACL-Policy, DB-Bereich,
Revision und Prozessgrenze übereinstimmen. Vollständiger Bindungsdigest enthält zusätzlich Profil,
Installation, Datenroots und Paketidentität; gleiche Workspace-ID allein reicht nicht.
C2-`no-execution`, native UI-ABI 0, alte Kontrollschemata und abweichende Bindungen werden als
Modelleingabe abgewiesen. Ziel-UI-Version 1 ist nur ein Modellwert, kein neuer nativer Leserbereich.

Unabhängig mit Python-`struct`/`hashlib` berechneter Goldenvektor für die synthetische
Testbeschreibung: **615 Bytes**, SHA-256
`5ff2f3bc766630978c254b578561d3f168fc5e7d4a443630b4f1d0c6ca8fec39`. Er beschreibt keine reale
Runtime und keine öffentlichen Profil-/Auftragskennungen.

## Versioniertes Lebenszyklusmodell

```text
Idle -> Committed -> RootBound -> Accepted -> Running -> Ready
                       |           |           |          |
                       +-----------+-----------+----------+-> Closing -> Drained -> Idle
aktive Generation -- Wächterverlust --> Blocked (auch nach modelliertem Drained)
```

Schrittbytes 1–8: Commit, RootBound, Accepted, Running, Ready, Closing, Drained, Finish.
`ModelRecordV1` ist ein begrenzter versionsgeprüfter **Simulationsrecord**, kein HMAC-Record,
Startgrant, Bootstrappipeframe oder neuer dauerhafter Storetyp. C4b benötigt weiterhin eigene native
Persistenz-/Peer-/Root-/Rollenbelege; alte Programme erhalten keine neue Leseberechtigung. Das
Lebenszyklusobjekt kann nicht serialisiert oder aus solchen Records als authentisierte Recovery
rekonstruiert werden.

- Commit verlangt übereinstimmende Workspace-/Paket-/Bindungsidentität, Head/Revision, freie Gruppe,
  neue Launch-ID und genau die nächste Generation. Mindestens acht der höchstens 256
  Modellrecordslots müssen vorliegen. Jeder akzeptierte Schritt verbraucht einen Slot; Budgetende
  sperrt, ohne Checkpoint-/Langzeitstorefähigkeit zu behaupten.
- RootBound benötigt eine positive simulierte PID/Erzeugungszeit und genau einen Prozess im eigenen
  Job. Job-ID/Wächterepoche bleiben bis Finish gleich. Rootidentität muss bei Accepted, Running und
  Ready erhalten sein. Bootstrap hat höchstens 30.000 ms; genau an der Grenze wird abgewiesen.
  Übergebene Zeit darf innerhalb eines Launchs nicht rückwärts laufen; es gibt keine eigene Uhr oder
  einen Timer. Nach Ablauf ist Schließen einer bereits gebundenen Gruppe möglich.
- Ready ist erst nach Running und behaupteter Runtime-/Rollenprüfung mit mindestens drei Prozessen
  zulässig. Dies modelliert die spätere Command-/Textfreigabe; kein tatsächlicher Datenbankzugriff
  oder UI-Router entsteht. Zweite Generation während aktiver Gruppe verweigert.
- Rootende/Closing ersetzt keinen Gruppenabschluss. Nur derselbe behauptete gehaltene Job mit
  Leerstand erlaubt Drained und Finish; fremde Epoche, unbekannter Job oder laufende Kinder sperren.
  Profil und Workspacebindung bleiben nach Finish erhalten. Neue Generation benötigt neue Launch-ID.
- Wächterverlust setzt Blocked; spätere Leerstandsbehauptungen, Profilabwesenheit oder Replay geben
  keine neue Autorität. Committed ohne Rootbindung bleibt ohne Endautorität gesperrt.
- Sämtliche abgewiesenen Übergänge erhalten Zustand, Head, Budget und verbrauchte IDs unverändert.

`ModelObservationV1` einschließlich Lease, Paket-/Rollenprüfung, Rootdaten, Job und Zeit ist
**untrusted Simulationsinput**. Kein Feld beweist einen nativen Zustand. Insbesondere attestieren
MACs aus C3 oder diese booleschen Werte weder individuelle Renderer-PIDs noch die wirksame
Beschränkung von WebView-Kindern vor deren Ausführung. Das Stopgate C4b bleibt offen.

## Ausgeführte Prüfung und Fortsetzung

```powershell
cargo test --locked --offline --manifest-path scripts/runtime-isolation/probe/Cargo.toml c4
cargo test --locked --offline --manifest-path scripts/runtime-isolation/probe/Cargo.toml
cargo clippy --locked --offline --manifest-path scripts/runtime-isolation/probe/Cargo.toml --all-targets -- -D warnings
cargo fmt --manifest-path scripts/runtime-isolation/probe/Cargo.toml -- --check
```

**Zwölf neue C4a-Tests bestanden; gesamter Helferlauf 108 aktiv bestanden, vier bewusst ignoriert**
(101 Binär-/sieben Transporttests). Clippy ohne Warnungen, rustfmt, unabhängiger Goldenhash und
Dokumentformat-/Verweis-/Fakten-/Diffprüfung. Geprüft: Rundlauf, Digestmutationen, Sortierung,
Pfad-/Policygrenzen, fremde Versionen/Domain/Rollen, Abbruch/Nachlauf, C2- und Bindungsabweichungen,
Replay/Generation/Head/Budget, Lease, Paket-/Root-/Rollenclaims, Frist und rückwärts laufende Zeit,
laufende Kinder/fremde Jobs, Wächterverlust und atomare Ablehnung. Keine große Crashmatrix oder
zusätzliche native C4-Serie; diese reine Erweiterung bietet noch keinen ausführbaren C4-Pfad.

Keine neue Runtime ausgewählt, kein produktiver Datenbestand geöffnet, keine native UI gestartet,
keine neue Brokerfreigabe oder eigene ETW-Abnahme. Der gespeicherte C3-Build und sein Netzbeleg
bleiben separat unverändert; ein künftig gebauter C4b-Kandidat erbt diesen Beleg nicht. Keine
App-/Transport-/Packagingänderung oder vollständiger Appbuild/`pnpm check` erforderlich.

Nächster begrenzter Schritt: **C4b – native Paket-/Overrideprüfung und belegbarer eingeschränkter
Mehrprozess-Erzeugungsweg**, zunächst nur mit neuen synthetischen Ablagen. C4c Minimaloberfläche/
SQLite, C4d frisches ETW, automatische Gruppen-Recovery, P2 insgesamt/P3–P5 und Produkt-Q-01/Q-02
bleiben offen. Striktes Datenschutzziel unverändert.
