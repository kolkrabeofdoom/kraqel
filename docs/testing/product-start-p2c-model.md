# P2c C1: reiner Vertrags- und Lebenszykluskern

Stand: 17. November 2026. **C1 implementiert und als Modell geprüft; keine native P2c-Ausführung,
Authentisierung, Persistenz oder Produktfreigabe.** Grundlage ist der
[P2c-Vertragsentwurf](../data-format/product-start-p2c-v1-draft.md).

Dieser Nachweis dokumentiert den C1-Stand. Inzwischen ergänzt der getrennte
[C2-Datenpfad](product-start-p2c-native-data.md) native Daten-/Profil-/Paketbindung ausschließlich
in frischen Schema-6-DATA-Stores. Er erteilt keine native UI-Startberechtigung; die folgenden
Angaben zur reinen Modellgrenze bleiben gültig.

## Implementierter Umfang

Eigenes [Modul](../../scripts/runtime-isolation/probe/src/product_start_p2c.rs) neben P1/P2a, mit
[gezielten Tests](../../scripts/runtime-isolation/probe/src/product_start_p2c/tests.rs). Nur der
Forschungsbinärbaum deklariert das Modul; kein Command, nativer Storeleser oder normaler Appstart
ruft es auf. Kein Einbau in den gemeinsam genutzten Transport.

- Strikte Serde-Typen für `WorkspaceBindingV2`, Rootmetadaten, `UiStartProposalV1`, `UiStartGrantV1`
  und Bootstrapframes; Modelltypen für Rootbindung und Gruppenende. Kontrollschema 6/UI-ABI 1 sind
  ausschließlich zukünftige Vertragswerte. Der native Schema-5-/UI-ABI-0-Forschungspfad wird dadurch
  nicht erweitert.
- Begrenzte Inventarcodierung und kanonischer Decoder gemäß Paketdomain, Rollenbytes, little-endian
  Längen, sortierten UTF-8-Pfaden, Dateilängen und SHA-256. Domain und Host-/UI-Rolle gehen in den
  Paketdigest ein. Die Funktionen verarbeiten übergebene Metadaten, lesen keine Dateien und belegen
  weder vollständige Paketbestände noch Hardlinkfreiheit.
- NFC wird geprüft, nicht durch stilles Umbenennen hergestellt. `unicode-normalization` ist auf
  0.1.25 festgeschrieben, wie bereits im Export-Crate; der Helferlock enthält zusätzlich `tinyvec`
  1.13.3. Keine Änderung des Workspace-/App-Locks oder der Transportimplementierung.
- Konservativer portabler Kollisionsschlüssel ist Unicode-Großschreibung. Er erkennt auch
  Dateiname-/Vorfahrenkonflikte, ist aber **kein Beleg für Windows-Ordinalvergleich**. C2 muss
  tatsächliche Namen und Dateiidentitäten unter gehaltenen nativen Handles zusätzlich prüfen.
  Grenzen: 32.768 Dateien, 1.024 Pfadbytes/Tiefe 32, 512 MiB/Datei, 2 GiB Inhalt, 4 MiB Codierung;
  leeres Inventar und Nullhashes abgewiesen. Decoder lehnt unbekannte Rollen, nichtkanonische
  Reihenfolge, Abbruch und Nachlauf ab.
- JSON-Payloads sind höchstens 8.192 Bytes groß; unbekannte/mehrfach vorhandene Structfelder und
  fremde Zwecke werden abgewiesen. Bootstrap als UTF-8-JSON mit Domain `KRAQEL-P2C-UI-START\0\x01`,
  Version 1, Rollen `grant`/`accepted`/`continue` und Sequenzen 0/1/2. Kein Frame erzeugt selbst
  eine MAC oder ein authentisiertes Handle.
- Vollständiges Bindungstupel, Job-ID, Wächterepoche und Nonce müssen zwischen Grant, Accepted und
  Continue gleich bleiben. Die übergebene verstrichene monotone Zeit darf nicht rückwärts laufen;
  spätestens bei der Startfrist, maximal 30.000 ms, ist der Bootstrap gesperrt. Es gibt weder eine
  eigene Uhr noch einen ausführbaren Timer in C1.

## Modell und Grenzen der Nachweise

```text
Idle -> Committed -> RootBound -> Accepted -> Running
                       |           |           |
                       +-----------+-----------+-> Closing
                       |           |           |      |
                       +-----------+-----------+------+-> Drained -> Idle
aktive Generation -- Wächterverlust --> Blocked
```

Root-/Clientende wird lediglich als Schließanforderung behandelt. Erst ein passender simulierter
Jobleerstand unter derselben Lease/Wächterepoche erlaubt Drained; anschließend folgt Finish.
Committed ohne Rootbindung bleibt gesperrt. Ein neuer Writer ist während aktiver Generation
abgewiesen. Sechs Recordslots werden vor Commit vorausgesetzt; alle fehlgeschlagenen Record-
Übergänge lassen Head, Budget und Zustand unverändert. Launch-ID bleibt nach Finish verbraucht, die
nächste Generation muss exakt folgen. Workspace-/Profilbindung bleibt im Modell erhalten.

`ModelObservation` und alle geparsten Grants sind **untrusted Simulationsdaten**. C1 prüft keine
ACLs, Tokens, MACs, echten Jobs/PIDs/Erzeugungszeiten oder gehaltenen Sperren. Root- und Lease-
Felder können keine native Berechtigung erzeugen. Kein Serialisieren/Rekonstruieren des
Lebenszyklusmodells als Kontrollstore; keine Adoption alter Journale. Nach modelliertem
Wächterverlust bleibt Blocked auch bei nachträglich behauptetem Leerstand bestehen. Selbst ein
vorher modelliertes Drained eröffnet in C1 keinen Recoverypfad; dessen dauerhafte Recordauswertung
ist noch nicht implementiert.

## Ausgeführte Prüfung

```powershell
cargo test --offline --manifest-path scripts/runtime-isolation/probe/Cargo.toml
cargo clippy --locked --offline --manifest-path scripts/runtime-isolation/probe/Cargo.toml --all-targets -- -D warnings
cargo fmt --manifest-path scripts/runtime-isolation/probe/Cargo.toml -- --check
```

13 neue C1-Tests, gesamter Helferlauf: **88 bestanden, vier bewusst ignoriert** (81 Binärtests plus
sieben Transporttests). Geprüft sind Golden-Bytes und ein unabhängig mit Python/`struct`/`hashlib`
berechneter SHA-256, Rollen-/Inhaltsbindung, Reihenfolge/NFC/Aliasse, Pfad-/Größengrenzen,
kanonischer Decoder, strikte Parser, falsche Domain/Sequenz,
Head/Revision/Generation/Replay/Budget/Lease, Nonce/Frist, vorzeitiges Schließen und laufende
Kinder, unbekannter/fremder Job sowie Wächterverlust. Keine zusätzliche Crashmatrix.

Goldenvektor: UI-Rolle, ein Ressourceneintrag `a.txt`, Länge 0, Dateihash 32-mal Byte `01`;
Codierung 78 Bytes, SHA-256 `78ea47a3fe725f1ad898cb2f5a94da69f5f6c100dfe12f998b9e55669b4e4e67`.

Rustfmt/Clippy sowie Dokumentformat-, Verweis- und Diffprüfung gehören zur Übergabe. Es wird nur der
Helfer getestet/geprüft; keine neue Broker-/UI-EXE als Forschungsablauf gestartet, keine
App-/Installer-/Transportintegration oder vollständiger Tauri-Build. Kein neues ETW erforderlich:
der vorhandene [Worker-v3-Beleg](product-start-worker-lifetime-etw.md) bleibt ausschließlich an
seinen gesicherten Build `37d7bec…` gebunden und gilt nicht automatisch für neu kompilierte Bytes.
Keine neue Freigabe an echten Vorschau-/Projektablagen, keine Schlüssel-/ACLänderungen dort.

## Damals nächster Schritt und heutige Fortsetzung

**C2** ist inzwischen im begrenzten [nativen Datenumfang](product-start-p2c-native-data.md)
umgesetzt. Dessen [frisches ETW](product-start-p2c-native-data-etw.md) ist inzwischen ergänzt;
nächster Schritt ist C3. Reale Datenmigration, automatische Wächtercrash-Recovery,
C3-Mehrprozessstart, C4-WebView und Produkt-Q-01/Q-02 bleiben offen. Das strikte Datenschutzziel
bleibt unverändert.
