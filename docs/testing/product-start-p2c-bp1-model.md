# P2c BP1: reine Präload-Typen, Codierung und Entscheidungsmodell

Fortsetzung: [getrennter nativer BP2-Blattversuch](product-start-p2c-bp2-native-leaf.md). BP1 bleibt
ein reines Modell; der folgende Modellnachweis erteilt keine native Autorität.

Stand: 18. November 2026. **BP1 im reinen Modellumfang implementiert und geprüft; keine native
Autorität.** [Bereinigte Prüfdaten](product-start-p2c-bp1-model.json),
[präventiver Vertrag](../data-format/product-start-p2c-c4-preload-v1-draft.md).

## Tatsächlicher Umfang

[Typen/Modell](../../scripts/runtime-isolation/probe/src/product_start_p2c/preload.rs),
[Codec](../../scripts/runtime-isolation/probe/src/product_start_p2c/preload/codec.rs),
[Tests](../../scripts/runtime-isolation/probe/src/product_start_p2c/preload/tests.rs). Der
vorhandene P2c-Modulkopf bindet ausschließlich dieses zusätzliche reine Untermodul ein.
`forbid(unsafe_code)`; keine I/O-, Netzwerk-, Zeit-, Windows-API-, Schlüssel-, MAC- oder
Prozessstartfunktion und kein neuer CLI-/Commandpfad. Keine Abhängigkeit ergänzt.

Geschlossene Policy-/Rollen-/Support-/Witness-/Entscheidungstypen sowie Kontext-, Request- und
Endtypen. C4-Rollen bleiben getrennt von WebView-API-Kinds. Native Windows-Bitmasken werden weder
angenommen noch erzeugt; sieben semantische Schutzregeln sind typisiert. Support, effektive Regeln
und Wächter-/Lade-/Rollenbehauptungen bleiben ausschließlich eingegebene Simulationsdaten.

Policies: höchstens 16 KiB, sechs verschiedene C4-Rollen, Root genau einmal mit Instanzgrenze 1;
positive Instanzsummen bis Gesamtgrenze, Gesamtgrenze höchstens 16. Portable NFC-Pfade bis 1.024
UTF-8-Bytes, nichtleere Digests, geschlossene Architektur/Zweck/Version. Kanonische Rollen-/
Regelreihenfolge, kein stilles Entfernen unbekannter Werte. Beide Signaturklassen gleichzeitig
werden im Modell bewusst als widersprüchliche Anforderung abgewiesen; dies ist keine Behauptung über
die native Windows-Bitinterpretation.

## Entscheidungen und Lebenszyklus

- Vorprüfung validiert Policy-/Support-/Requestkontext und den Mindest-OS-Build. Abfrage unklar,
  Pflichtregel nicht unterstützt oder Kindpolicy unbekannt: Ablehnung ohne Reservierungsänderung.
  Eine Versionszahl allein belegt keine Fähigkeit.
- Nur das synthetische Blattprofil kann `SyntheticLeafEligible` erreichen: eine Rootrolle, Grenze
  eins, Kindstartregel Denied und erforderliche NoRemote-/NoLowLabel-Regeln. Dies ist eine reine
  Modellentscheidung, kein Startticket oder nativer Witness.
- Nach Reservierung bleiben Launch-ID und Nonce verbraucht. Nur eine Bewertung je passendem Auftrag,
  auch bei fehlendem Readback, Audit statt Durchsetzung oder falschem Image-/Support-/
  Token-/Handlebeleg. Fremder Request- oder Rootgenerationskontext verändert die aktive Reservierung
  nicht. Auch eine frühere Rootgeneration ist unter einem neuen Launch nicht wieder bindbar;
  dieselbe PID mit neuer Erzeugungszeit ist dagegen eine neue Modellgeneration.
- Eine positive Blattbewertung beendet die Gruppe nicht. Modellende benötigt passende Generation und
  bestätigte Behauptungen zu Rootende/derselben leeren Gruppe. Unklarer Drain bleibt pending;
  Wächterverlust bleibt auch bei später behauptetem Ende gesperrt. Vor einer Rootbindung ist ein
  eigener Abbruch nur mit passendem Request und modelliertem Jobleerstand möglich, ebenfalls ohne
  Wiederfreigabe des verbrauchten Auftrags.
- Höchstens 64 Reservierungen je Modellinstanz. Der Zustand ist flüchtig und nicht serialisiert; ein
  neues Modell ist **kein** nativer Wiederanlauf- oder dauerhafter Replaybeleg.

Runtime-/Datenziele werden immer abgewiesen: fehlender Wächteranker, unbewiesene Ladebindung,
unbewiesene Rollenbindung oder abschließend `NativeAuthorityUnavailable`. Selbst alle positiv
gesetzten Modellbehauptungen ergeben weder `RuntimeStartAllowed` noch `DataAllowed`. Unbekannte
Lade-/Rollenmechanismen werden nicht aus Signaturen, Suchreihenfolge oder Crashpad-Argumenten
abgeleitet.

## Deterministische Formate und Prüfdaten

Getrennte Policy-/Modellpräfixe aus dem Vertrag; vier bounded Decoder für Policy, Support, Witness
und End. Little Endian, feste Arraylängen, begrenzte UTF-8-Strings, geschlossene Tags, boolesch nur
0/1. Nichtkanonische Reihenfolge, doppelte Regeln, fremde Frameart, unbekannte Versionen,
abgeschnittene oder zusätzliche Bytes werden abgewiesen. Vollständige Feldfolge steht im
[Vertrag](../data-format/product-start-p2c-c4-preload-v1-draft.md#8-bp1-codierung-und-modellgrenze).

Feste vollständig synthetische Testdaten, unabhängig mit Python `struct`/SHA-256 gegengeprüft:

| Format  | Bytes | SHA-256                                                            |
| ------- | ----: | ------------------------------------------------------------------ |
| Policy  |   309 | `17e1b86f3e2e9d36638a5f9c2a5effa98121d901e576eb65ac4907eef71bb552` |
| Support |    71 | `ba476c1999688de5d270dc7c0007b73035e167b02376e7eda15d708afbb0e3d4` |
| Witness |   328 | `21117d814cfaca4535132018b6d6793309b54481d9c2dc76ce891c80ca69cc1f` |
| End     |   160 | `bb44dd92ec4fc3cc30f453f5e1277b96a7f83d8b1814dce886e6f652350af2a8` |

Die vollständigen Policy-Golden-Bytes sind im Test festgeschrieben; sämtliche Formatsummen sowie
Rundläufe werden geprüft. Keine tatsächlichen Prozessgenerationen, Keys oder Systemdaten in diesen
Testvektoren. Der Policy-SHA-256 bindet die gesamte kanonische Policy; der Supportdigest den
gesamten kanonischen Supportframe, einschließlich seiner Modellbehauptungen.

14 neue Tests: feste Formate/Rundlauf, alle Trunkierungsstellen, fremde Frames/Versionen/Tags,
boolesche Werte, kanonische Rollen-/Regelordnung, Pfad-/Digest-/Instanzgrenzen, widersprüchliche
Regeln, fehlender Support, atomare Vorprüfung, Kontextwechsel, Audit/Opt-out, erneute Bewertung,
Launch-/Nonce-/Generationsreplay, fehlgeschlagene Anlage, Drain/Wächterverlust, Budget und
grundsätzliche Runtime-/Datenabweisung.

`cargo test --locked --offline --manifest-path scripts/runtime-isolation/probe/Cargo.toml -q`: **133
aktive Helfer-/Transporttests bestanden** (126 Binär-/sieben Transporttests), sechs bewusst
ignoriert. Clippy `--all-targets -- -D warnings`, rustfmt sowie Format-/Fakten-/Verweisprüfung.
Mittleres Prüfniveau; kein neuer Runtime-/GUI-/Produktstarterbuild und kein zusätzlicher ignorierter
nativer Runtimefall oder ETW-Lauf. Der Helfer-Testbuild ist von einem Produktbuild zu unterscheiden.
Vorhandene aktive Helfertests behalten ihren synthetischen Fixtureumfang.

## Nächster Schritt und offene Grenzen

**BP2:** nativen Capability-/Creation-/Readback-Vertrag in einem gesonderten neuen synthetischen
Blattprozess umsetzen. Frühe Regeln tatsächlich setzen und zurücklesen, einen verbotenen Kindstart
prüfen, falschen Support/Policykontext abweisen, Handles/Ende bestätigen; danach gezieltes ETW des
exakten gestarteten Paars. Kein Vertrauen auf `SyntheticLeafEligible` als native Startautorität,
keine existente echte Kontrollablage oder Projektdaten.

Präventiver Wächter-/Lade-/WebView-Mehrprozessbeleg (BP3), effektive Datenrechte/Commandfreigabe
(BP4), C4b insgesamt/C4c/d, P2/P3–P5 und Produkt-Q-01/Q-02 bleiben offen. ABI 0 und vorhandene
native Schemas/Grants unverändert. Gesicherte Root-/Testcontrollerbytes und deren ETW bleiben
unverändert separat; kein Netzbeleg für neu kompilierte Programme daraus. Das strikte
Datenschutzziel bleibt bestehen, unsignierte Pakete bleiben zulässig.
