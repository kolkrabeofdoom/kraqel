# P2c C4b: native Kandidatenprüfung und suspendierter Erzeugungsversuch

Inzwischen ergänzt die getrennte
[authentisierte Root-/WebView-Probe](product-start-p2c-c4b-root-bootstrap.md) den tatsächlichen
leeren Runtimepfad. Die folgenden Zahlen/Angaben beschreiben den ursprünglichen Vorprüfungsstand,
ohne dessen suspendierte Geschwister zur WebView-Abnahme umzudeuten.

Stand: 17. November 2026. **Begrenzter C4b-Teilstand implementiert und geprüft; C4b insgesamt bleibt
offen. Kein ausgeführter WebView-Browser-/Rendererpfad und keine native UI-Startfreigabe.**
[C4-Vertrag](../data-format/product-start-p2c-c4-v1-draft.md),
[C4a-Modell](product-start-p2c-c4a-model.md).

## Native Vorprüfung unter der bestehenden Kontrollsperre

[Kandidatenprüfung](../../scripts/runtime-isolation/probe/src/windows/exchange/ui_restart/durable/product/workspace_persistent/c4_preflight.rs)
und
[begrenzte Inventur](../../scripts/runtime-isolation/probe/src/windows/exchange/ui_restart/durable/product/workspace_persistent/filesystem.rs)
verarbeiten ausschließlich neue private `c4-candidate/host`-/`ui`-Unterbäume in eigenen
synthetischen C2-Workspacefixtures. Die alten C2-Paket-/Datenrechte werden nicht erweitert. Kein
öffentlicher Command, Rendererzugang oder neuer nativer Storeleser ist registriert.

- Vorprüfung verlangt passende Brokeridentität, unveränderten erwarteten Kontrollhead, dieselbe
  gehaltene Lease, Schema 6 und nutzbaren Datenworkspace. Pending, aktive Jobs, fehlende Bindung
  sowie begonnene/abgeschlossene Stilllegung sperren. Ein Kandidat erzeugt kein Pending und keinen
  Startgrant; Kontrollkette/Schlüssel werden durch diese Prüfung nicht geschrieben.
- C4a-Deskriptor und kanonische Host-/UI-Inventare werden begrenzt decodiert und anhand ihrer
  Rollen/Digests/Revision abgeglichen. UI-EXE und Rollenimages müssen als Programme im geprüften
  Inventar enthalten sein; ein dynamischer Loader benötigt passenden Bibliothekseintrag/Digest.
- Native lokale Rootidentität, feste Datenträger, gehaltene Vorfahren-/Dateihandles, exakte private
  Konto-/SYSTEM-ACLs, Windows-Ordinalaliase, Reparse-/Hardlink-/ADS-Abwehr sowie exaktes Inventar
  mit tatsächlichen Längen und SHA-256. Offene Paketdateien erlauben weder Schreiben noch
  Umbenennen/Austausch. Das Ergebnis besitzt nur lokale Handles und Metadaten, keinen
  serialisierbaren Freigabebeleg.
- Eigene konservative C4-Vorprüfungsgrenzen pro Paket: 1.024 Dateien, 2.048 Baumeinträge, Tiefe 32,
  512 MiB/Datei, 2 GiB Inhalt. Die ursprünglichen C2-Grenzen bleiben 64/128/Tiefe 8, 128 MiB/Datei
  und 256 MiB Inhalt. C4a-/C1-Formatgrenzen bleiben zusätzliche Grenzen.
- Geerbte `WEBVIEW2_*`-Variablen einschließlich unbekannter/leer gesetzter Overrides werden
  abgewiesen; ebenso `COREWEBVIEW2_MAX_INSTANCES` und nichtleere `__COMPAT_LAYER`. Vorprüfung
  verändert keine Environmentwerte. Native Registryprüfung liest ausschließlich
  `Software/Policies/Microsoft/Edge/WebView2` in HKCU/HKLM und beiden WOW64-Ansichten. Nichtleere
  Schlüssel werden konservativ vollständig abgewiesen, auch bei fremder App-ID.
  Zugriff-/Abfragefehler gelten nicht als Abwesenheit. Keine Registry-/Domänenänderung.

Die
[WebView2-API](https://learn.microsoft.com/en-us/microsoft-edge/webview2/reference/win32/webview2-idl?view=webview2-1.0.4258.31)
beschreibt Environment-/Registry-Overrides. Die vollständige Abweisung dieser Eingänge ist die
konservative C4-Vorprüfungsregel, keine Zusage universeller Windows-Policyattestierung. Ein späterer
tatsächlicher Start muss die wirksame Auswahl erneut prüfen; eine lesende Registryaufnahme bindet
keine unveränderliche Systemkonfiguration.

## Offizieller lokaler Runtimekandidat

Nach ausdrücklicher Freigabe heruntergeladen: Microsoft Fixed Version **157.0.4341.60, Windows
x64**, direkt vom Microsoft-CDN über die im
[offiziellen Katalog](https://developer.microsoft.com/microsoft-edge/api/webview2) angegebene URL.
Ein fremdes Archiv wurde nur zur URL-Ermittlung verwendet; die URL anschließend am offiziellen
Katalog gegengeprüft, keine Binärdatei von einem Drittanbieter geladen.

CAB: **310.954.844 Bytes**, SHA-256
`1bad3df802a4c21f6b271cf3cd640912056291e3915b86fa7577a18899f761b9`. Mit Windows `expand` in eigene
lokale TEMP-Ablage entpackt: **257 Dateien, 711.058.264 Bytes**. Authenticode-Prüfung von
`msedgewebview2.exe`: Status `Valid`, signiert von Microsoft Corporation. Dies ist eine lokale
Herkunftsprüfung des Downloads, keine Zertifikatspflicht für KRAQEL und keine Attestierung jedes
Runtimeverhaltens. Keine Installation oder gestartete Runtime.

Ein expliziter lokaler nativer Test kopiert alle Runtime-Dateien in eine neue private Fixture,
bildet deren Inventar und prüft es unter der Kontrollsperre samt neuem synthetischen Host-/UI-Root.
Der Test besteht. Die UI-/Frontend-/Argument-/Tokenpolicies bleiben dabei **Platzhalter für die
Vorprüfung**, keine tatsächlich abgenommene WebView-/UI-Paketclosure. Selbst behauptete
Runtimeversion und Policydigests ersetzen weder native Versions-/Modulprüfung noch Durchsetzung.

Lokale Sicherung: `%TEMP%/kraqel-c4-runtime-20261117-01/`, CAB, `expanded/`, Herkunftsmetadaten,
Expandprotokoll und lokales Hash-/Signaturmanifest. Nicht nach Git übertragen; für einen anderen
Rechner erneut lokal beziehen und prüfen. DNS-Probleme am Downloadportal erforderten mehrere
Abrufversuche, aber keine System-/DNS-/Firewalländerung.

## Begrenzter nativer Erzeugungsweg

[Erzeugungsversuch](../../scripts/runtime-isolation/probe/src/windows/exchange/ui_restart/durable/product/workspace_persistent/c4_preflight/creation_probe.rs):
zwei **vom Controller erzeugte suspendierte Geschwisterprozesse**, keine echten WebView-Kinder.
Beide verwenden die gepinnte synthetische UI-EXE aus der frischen Kandidatenfixture; kein fremdes
Programm und keine Browserargumente werden ausgeführt.

`CreateProcessW` ordnet beide bereits bei Erstellung mittels Jobattribut derselben neu angelegten
Jobinstanz zu; kein nachträgliches Assign/Resume-Fenster. AppContainerprofil explizit, null
Capabilities, keine Handlevererbung, begrenzte eigene Environment. Joblimit 2, KillOnJobClose und
kein Breakaway. Vor irgendeinem Fortsetzen werden exakte Jobzugehörigkeit, AppContainer-/Profil-
Token, Capabilityfreiheit und positive native PID/Erzeugungszeit geprüft. **Der Pfad ruft
ResumeThread nicht auf und besitzt keine UI-/Bootstrap-Freigabe.**

Bei zwei aktiven Mitgliedern verweigern sowohl Job-Leerstandsprüfung als auch zweites Öffnen der
Kontrollablage. Beide bekannten eigenen Prozesse werden beendet und der Leerstand derselben
gehaltenen Jobinstanz bestätigt. Kontrollhead bleibt unverändert. Das ist kein dauerhafter
Launch-/Drained-/Finish-Record, keine neue Endautorität für frühere Jobs oder fremde Prozesse und
keine Recovery nach Wächterverlust.

## Ausgeführte Prüfung und offene Fortsetzung

```powershell
cargo test --locked --offline --manifest-path scripts/runtime-isolation/probe/Cargo.toml c4_native_candidate
cargo test --locked --offline --manifest-path scripts/runtime-isolation/probe/Cargo.toml c4_native_suspended_pair
cargo test --locked --offline --manifest-path scripts/runtime-isolation/probe/Cargo.toml c4_downloaded_fixed_runtime_native_preflight -- --ignored --nocapture
cargo test --locked --offline --manifest-path scripts/runtime-isolation/probe/Cargo.toml -q
cargo clippy --locked --offline --manifest-path scripts/runtime-isolation/probe/Cargo.toml --all-targets -- -D warnings
cargo fmt --manifest-path scripts/runtime-isolation/probe/Cargo.toml -- --check
```

Vier neue aktive Tests: Overrideabwehr, Paketpins/Manipulation/Extras/Head/Actor/Lease,
Hardlinks/ADS/Stilllegung sowie zwei nie fortgesetzte Prozesse. Gesamter Standardhelferlauf: **112
aktiv bestanden, fünf standardmäßig ignoriert** (105 Binär-/sieben Transporttests). Einer der fünf
ignorierten Fälle ist der neue Runtimeinventurtest und wurde zusätzlich ausdrücklich lokal
bestanden; die vier bisherigen ignorierten Fälle wurden nicht ausgeführt. Clippy mit Warnungen als
Fehler, rustfmt, Verweis-/Format-/Fakten-/Diffprüfung. Keine breite Crashmatrix, neuer
App-/Installerbuild oder `pnpm check`; gemeinsamer Transport und App bleiben unverändert.

**C4b bleibt offen:** authentisierter Rootbootstrap vor Framework/Runtime, dauerhaft gebundene
UI-Records, tatsächlich geladene Runtime-/Loader-/Nichtsystemmodule und effektive Konfiguration
müssen integriert werden. WebViews eigener dynamischer Browser-/Renderer-/GPU-/Utility-Kindpfad und
seine Token-/Job-/Handlebeschränkung vor Ausführung sind noch nicht belegt. Die beiden
Controller-Geschwister dürfen nicht als dieser Nachweis ausgegeben werden. Die
[Microsoft-Prozessbeschreibung](https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/process-model)
führt variable Rollen/Prozessgruppen auf; ein Joblimit 2 ist kein übertragbarer Runtimevertrag.

Der dort ursprünglich nächste Root-/Runtimeversuch ist inzwischen im oben verlinkten eigenen
Nachweis geprüft. Weiter offen: vollständige Runtime-/Nichtsystemmodul-/Rollenbindung und zwei
API-unzugeordnete Generationen vor Text-/Commandfreigabe, danach frisches ETW. Native UI-ABI bleibt
0, keine Übernahme alter C2-/C3-Grants; C4b insgesamt, C4c/d, P2/P3–P5 und Produkt-Q-01/Q-02 offen.
Striktes Datenschutzziel unverändert.
