# Dauerhafte native Broker-Wiederherstellung · lokaler HMAC-Vertrag v2

Historische Stufe. Aktueller [Brokervertrag v3](native-broker-recovery-v3.md) ergänzt Live-Aufträge
und eine unveränderliche signierte Grantfolge. Keine automatische Übernahme von v2-Sitzungen oder
Übertragung ihrer Netzbelege.

KQ-069 / #134, 16. November 2026. Ersetzt im aktuellen Forschungsbroker die
[DPAPI-Primitive aus v1](native-broker-recovery-v1.md). Kein Produktlauncher, kein neuer
Rendererbefehl, keine Änderung an Startticket v2 oder Auftragsjournal v1.

## Auflösung der Domänengrenze

Die Wiederanlaufautorität verwendet weder `CryptProtectData`/`CryptUnprotectData` noch
Kontogeheimnisse, Credential-/Schlüsselanbieter oder Domänenbackup. Ein frischer 256-Bit-Schlüssel
wird genau einmal über `getrandom` 0.4.3 erzeugt; dessen fest betrachteter Windows-Backend ruft
`ProcessPrng` aus `bcryptprimitives.dll` auf. HMAC-SHA-256 wird anschließend durch RustCrypto `hmac`
0.12.1 mit dem bestehenden `sha2` 0.10.9 berechnet und über `verify_slice` geprüft. Keine selbst
implementierte MAC-Konstruktion oder konfigurierbare Cryptoanbieter. Fehlender Schlüssel/RNG-Fehler
hat keinen DPAPI- oder sonstigen Ersatzpfad.

`hmac` 0.12.1 (MIT/Apache-2.0) ist passend zur vorhandenen digest-/sha2-Reihe festgeschrieben;
transitiv hinzu kommt `subtle` 2.6.1 (BSD-3-Clause) für den MAC-Vergleich. `getrandom` 0.4.3
(MIT/Apache-2.0) war bereits im Helfer-Lockfile vorhanden und wird nun direkt gebunden. Alle Pakete
waren lokal verfügbar; Auflösung/Build offline möglich. Keine neue Abhängigkeit des normalen
Produktstarts, keine Änderung von Windows-/Firewall-/Domänenrichtlinien. Das frühere
Win32-Cryptography-Feature wird für diesen Helfer nicht mehr angefordert.

## Geschützter Zustand und Binärformat

```text
control/authority.key    exakt 32 zufällige geheime Bytes, lokaler Klartext
control/authority.mac    Magic/Version || 32-Byte-MAC || ursprüngliche JSON-Authority
control/owner.lock       vorhandene exklusive Broker-Sperre
control/retired          vorhandene endgültige Stilllegungsmarkierung
```

Die vorhandene geschützte DACL erlaubt ausschließlich dem aktuellen Windows-Konto und SYSTEM
Zugriff. Keine Paket-SID oder geerbte Gruppenfreigabe. Der AppContainer erhält weder den Schlüssel
noch den Beleg; tatsächliche Dateiöffnungen unter seinem suspendierten Root-Token müssen
Lesen/Schreiben beider Dateien verweigern. Je App-Generation jetzt sechs Prüfungen: Beleg
lesen/schreiben, Schlüssel lesen/schreiben, Eigentümersperre lesen, Appdatei schreiben.

Magic sind die 23 Bytes `KRAQEL-BROKER-AUTHORITY` gefolgt von `00 02` (zusammen 25 Bytes). Der HMAC
authentisiert exakt `Magic || JSON-Bytes`. Der MAC steht unmittelbar nach Magic; danach folgen
unveränderte JSON-Bytes. Insgesamt höchstens 32.768 Bytes, nichtleere JSON-Nutzlast, genau 32
MAC-Bytes. Version/Magic, MAC und Framegrenzen werden vor JSON-Deserialisierung geprüft. Unbekannte
JSON-Felder werden abgewiesen; Authority-`version` muss 2 sein. Ihre übrigen Bindungen bleiben
unverändert: kanonische Wurzel, Session, ursprünglicher Auftrag/Launch und beide Programmhashes.
HMAC ist **keine Verschlüsselung**. Die Metadaten sind über die DACL geschützt; der Schlüssel bleibt
ein lokales Geheimnis und gehört weder in Git, Projektarchive noch Logs.

Beide Dateien werden ausschließlich im frischen Seed mit `create_new` geschrieben und jeweils
synchronisiert: erst Schlüssel, dann Beleg, anschließend erst Appstart. Wiederherstellung öffnet
beide nur lesend, verlangt reguläre lokale Dateien mit genau einem Link und akzeptiert den Schlüssel
nur bei exakt 32 Bytes. Keine Neuanlage/Rotation/Reparatur in einer benutzten Sitzung. Fehlende, zu
kurze/lange, ausgetauschte oder beschädigte Schlüssel sperren die Übernahme.

v1-`authority.dpapi` wird nicht gelesen, migriert oder als Fallback verwendet. Ein nicht vollständig
angelegter Seed bleibt gesperrt. Frühere gesicherte Binaries und Belege bleiben historische
Forschungsartefakte mit eigener Buildbindung; ihre Sitzungen werden nicht still übernommen.

## Unveränderte Lebenszyklusgrenze

Exklusive Brokersperre und gepinnte Verzeichnisse/Appdatei gehen der Authentisierung voraus.
Vorhandenes Auftragsjournal, leerer alter Job, identische SID, null Capabilities, neue anonyme Pipes
und Launch-ID sind verpflichtend. Jobzuweisung bereits bei Prozesserzeugung, danach
Job-/Token-/Dateiverweigerungsprüfung vor Resume. Wiederherstellung erteilt keinen neuen
Dateiauftrag und liest keine Quelldatei erneut. Explizite Stilllegung sperrt weitere Übernahmen. Die
CLI bleibt ausschließlich für synthetische Forschung vorgesehen.

Die native Serie verlangt je Crashpunkt jetzt 13 Ablehnungen, insgesamt 78: die bisherigen zehn
Fälle mit HMAC statt DPAPI sowie fehlender, verkürzter und falscher Schlüssel. Unit-Tests prüfen
zusätzlich byteweise Veränderungen, sämtliche Trunkierungen, angehängte Bytes und Altdaten. Ein
eigener ETW-Modus `broker-ui` erfasst den gesamten Serienbaum einschließlich aller negativen
Brokerstarts; er verlangt Broker→App→WebView und behält Kontrollen/Verlust-/Lebenszeitprüfungen.

## Aussagegrenzen

Die DPAPI-Domänenroute ist durch **Entfernung der Primitive** aus diesem Brokerpfad aufgelöst, nicht
durch eine erlaubte Netz-/Metadatenausnahme. Ein frischer Netzbeleg bleibt trotzdem an das genaue
neue Binärpaar und die ausgeführten Pfade gebunden. Externe Shell-/COM-/DNS-Dienste sind dadurch
nicht allgemein attestiert; ein fehlender Netzbefund ist keine universelle Garantie.

Die Schlüsselverwahrung hängt ausdrücklich an Windows-DACL und unverknüpften lokalen Pfaden. Keine
Verschlüsselung bei ruhendem Datenträger, Hardwarebindung, Schutz vor Administrationsrechten,
Kompromittierung desselben Kontos, Schlüsselkopien oder Prozessspeicherauslese behaupten. Ein mit
Schlüssel und Beleg kopiertes/verändertes Konto kann Autorität erzeugen; das wird nicht als
zusätzlich gelöste Bedrohung ausgegeben. Ein Container kann weiterhin seine beschreibbare
Datenablage manipulieren. Geschützte Produktablage, authentisierter Update-/Migrationspfad und
kombinierte neue Live-Aufträge bleiben Produktarbeit; Q-01/Q-02 sind nicht automatisch bestanden.

Primärquellen: [ProcessPrng](https://learn.microsoft.com/en-us/windows/win32/seccng/processprng),
[getrandom-Windows-Backend](https://docs.rs/crate/getrandom/0.4.3/source/src/backends/windows.rs),
[RustCrypto HMAC](https://docs.rs/hmac/0.12.1/hmac/),
[HMAC-Verifikation](https://docs.rs/digest/0.10.7/digest/trait.Mac.html#method.verify_slice).
