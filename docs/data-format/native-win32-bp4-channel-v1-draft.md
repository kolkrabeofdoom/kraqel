# BP4b · Lokaler Kanal für synthetische Win32-Szenen

**Spezifikation v1, KQ-069.** Dieses Protokoll bindet die drei
[BP4a-Operationen](native-win32-bp4-scene-v1-draft.md) an ein einmalig gestartetes Prozesspaar im
interaktiven Offline-Gast. Der bisherige Starter prüft ein UI-Blatt und startet es ohne
Handlevererbung. Zweiter Rollenstart und Endpunktrechte der suspendierten Rollen sind im Gast
geprüft. Produktprojekte bleiben gesperrt.
[ADR 0036](../architecture/decisions/0036-native-win32-ui-role-evaluation.md) ·
[Starter](../../scripts/runtime-isolation/native-window/src/bin/native-starter.rs)

## Wer verbindet sich mit wem?

1. Unter exklusiver Kontrollsperre reserviert der Starter einen einmaligen Auftrag mit
   Paket-/Policyidentität, Gast-Sitzung, Testablage, Katalogszene und Modus. Nur er wählt diese
   Werte. Er startet UI und Datendienst suspendiert in getrennten Jobs mit Prozesslimit eins und
   Beendigung beim Verlust des letzten Jobhandles. Für jede Rolle prüft er Image, Erzeugungszeit,
   Sitzung, Token, Job und Mitigations an den gehaltenen Prozesshandles. Beide Rollen müssen vor dem
   ersten Resume bestehen.
2. Der Starter erzeugt zwei frische lokale anonyme Einwegpipes mit restriktiver DACL. Per gezielter
   Duplikation in die gehaltenen Prozesse erhält die UI genau das Schreibende für Anfragen und das
   Leseende für Antworten, der Datendienst jeweils die Gegenenden. Vererbung bleibt aus. Vor Resume
   müssen Objektidentität, wirksame Zugriffsmaske und fehlendes Vererbungsflag für jedes Zielhandle
   sowie die geschlossenen Starterkopien der Pipeenden belegt sein. Die UI erhält keine DB-, Job-,
   Prozess- oder Kontrollhandles. Nur der Datendienst erhält Zugriff auf die gebundene Testablage.
3. Der Starter liefert jeder Rolle einen eigenen nur lesbaren, unbenannten
   Bootstrap-Speicherbereich. Dieser enthält Rolle, beide Prozessgenerationen (PID und
   Erzeugungszeit), Sitzung, Auftragsbindung, die rolleneigenen Handlewerte, eine frische
   kryptografisch zufällige Kanal-ID (32 Byte) und zwei unabhängige zufällige HMAC-Schlüssel (je 32
   Byte, einer pro Richtung). Die UI erhält nur eine opake Ablagekennung. Der rollenspezifische
   Startbeleg nach dem DACL-Prinzip der Fensterprobe darf den prozesslokalen Bootstrap-Handlewert,
   aber keine Schlüssel oder Szenentexte enthalten. Schreibfähige Mappings und überzählige Handles
   des Starters vor Resume schließen. Jedes Ende gleicht Rolle und eigene Generation mit dem Beleg
   ab. Schlüssel bleiben flüchtig und werden nach Ende gelöscht.
4. Die Zuordnung gilt für genau den gestarteten UI-Prozess, der das Fenster besitzt. Ein HWND,
   Prozessname oder eine vom Fenster mitgelieferte ID begründet kein Recht. Prozess-DACLs und
   effektive Tokenrechte müssen fremdes Duplizieren, Speicherlesen/-schreiben und
   Endpunktübertragung an weitere Prozesse innerhalb des Prüfmodells verhindern. Das ist im Gast
   auch mit einer fremden unprivilegierten Prozessgeneration zu prüfen. Schlüssel alleine belegen
   keine Prozessidentität. Gastadministration und Starter gehören zur Vertrauensbasis. Scheitert
   eine Voraussetzung, verwirft der Starter das Paar und beendet beide Jobs.

Die Windows-Primitiven erlauben die gezielte Übergabe an einen Zielprozess. Die effektiven Rechte
der konkreten Gastkonfiguration bleiben ein eigener Nachweis.
[DuplicateHandle](https://learn.microsoft.com/en-us/windows/win32/api/handleapi/nf-handleapi-duplicatehandle)
·
[Zugriffsrechte an anonymen Pipes](https://learn.microsoft.com/en-us/windows/win32/ipc/anonymous-pipe-security-and-access-rights)

## Frames und Zustandsübergänge

Ein Frame besteht in dieser Reihenfolge aus ASCII-Magic `KQSC`, Version (u8 = 1), Typ (u8), Kanal-ID
(32 Byte), Nummer (u64 LE), Nutzlastlänge (u32 LE), den unveränderten Nutzlastbytes und einem
32-Byte-Tag. HMAC-SHA-256 umfasst sämtliche vorangehenden Framebytes ab Magic mit dem Schlüssel der
erwarteten Richtung. Die Längengrenze von 80 KiB für den BP4a-Nachrichtenkörper vor Allokation
prüfen, exakt lesen, Tag in konstanter Zeit vergleichen. Erst nach Prüfung von Bindung, Typ, Nummer
und Tag darf BP4a JSON interpretieren. Unvollständige Frames, unbekannte Versionen und
Richtungsverwechslungen schließen den Kanal.

| Zustand            | Einziger erlaubter Übergang                                                                                                                                                                                                            |
| ------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Neuer Kanal        | UI sendet HELLO (Typ 1, Nummer 0, leer). Dienst prüft es einmal, sendet READY (Typ 2, Nummer 0, leer). Erst nach gültigem READY sendet die UI eine Anfrage.                                                                            |
| Bereit             | UI sendet REQUEST (Typ 3) mit der nächsten Nummer, beginnend bei 1. Höchstens eine Anfrage offen. Dienst akzeptiert ausschließlich die exakt nächste Nummer und verbraucht sie vor Ausführung.                                         |
| Antwort ausstehend | Dienst sendet genau eine RESPONSE (Typ 4) mit derselben Nummer. Auch ein fachlicher BP4a-Fehler verbraucht die Nummer. UI akzeptiert nur die Antwort auf ihre offene Anfrage. Danach nächste Nummer; nach u64-Maximum Kanal schließen. |
| Gesperrt           | Wiederholtes HELLO/READY, doppelte, ausgelassene oder alte Nummer, falscher Typ, falscher Tag oder fremde Kanal-ID: beide Enden stilllegen, kein Dispatch des fehlerhaften Frames, kein erneutes Senden.                               |

Der Datendienst arbeitet seriell. Er bindet alle Operationen an seine Bootstrap-Szene und den Modus;
die erwartete Dokumentrevision bleibt eine zusätzliche Schreibbedingung. Ein gültiger MAC erhöht
keine Rechte. Der Starter überwacht beide Prozessenden; EOF, Zeitüberschreitung (30 Sekunden für
Handshake oder eine offene Anfrage), Fensterende und Rollenverlust beenden den Auftrag. Im Leerlauf
darf das Fenster offen bleiben. Abbruch muss auch blockierte Pipeoperationen beenden. Keine
Schlüssel, Texte oder Rohframes in Argumenten, Umgebung, Dateien oder Diagnoseausgaben. Zulässig
sind synthetische Fallkennung, bereinigter Ausgang, Revision und Jobende.

## Ungewisser Commit und Wiederanlauf

Vor Commit abgebrochene Transaktionen rollen zurück. Nach Commit und vor zugestellter Antwort bleibt
die neue Revision bestehen. Bei ungewissem Ausgang zeigt die UI keinen Speichererfolg. Es gibt weder
Antwortcache noch automatischen Schreibretry. Erst nach belegtem Drain beider Jobs und geklärtem
Transaktionsende darf eine neue Paarung unter der Kontrollsperre entstehen. Sie erhält neue Pipes,
Schlüssel und Kanal-ID und liest zuerst den gespeicherten Stand. Der alte Startauftrag bleibt
verbraucht. Fehlt nach Starterabbruch ein verlässlicher Endebeleg, bleibt die Ablage gesperrt.

## Gezielte Abnahme

Öffentlicher Testvektor für HELLO: Kanal-ID mit aufsteigenden Bytes von Hex 00 bis 1f,
Anfrageschlüssel von Hex 20 bis 3f. Bei Nummer und Länge null umfasst der Header 50 Byte, der ganze
Frame 82 Byte. Erwarteter Tag: `fa777fcddf15349fc2f06e72cada2c0366e7588656f7b05bd167ac5ac2d4fbe9`.
Diese festen Werte dienen ausschließlich dem Codetest.

- **Im reinen Modell geprüft:** Golden-Frame für Header und MAC, HELLO/READY, Lesen und eine
  bestätigte Speicherung. Denselben REQUEST zweimal anbieten: genau ein Dispatch und ein
  Revisionswechsel. Doppelte Antwort, Lücke, Reflexion, falscher Tag, überlange Länge und Frame aus
  vorheriger Paarung jeweils ohne weitere Fachoperation abweisen.
- **Danach im Gast:** beide Rollen und Endpunktrechte vor Resume zurücklesen; vertauschte Enden,
  fremder Prozess und Direktstart müssen scheitern. Den regulären Tastaturpfad sowie Ende vor Commit
  und Ende nach Commit vor Antwort mit frischer Lesebindung vergleichen. Beide Jobs müssen leer
  sein; B bleibt bei einem Auftrag für A unverändert.
- **Netzvergleich für die neuen Binärdateien:** Referenz und Datenlauf einschließlich Starter, UI
  und Datendienst, vier lokale Positivkontrollen und Verlustprüfung. Erst dieser Lauf liefert einen
  Netzbefund zum Datenkanal.

Der [Framecodec](../../scripts/runtime-isolation/native-scene-model/src/channel.rs) verarbeitet
vollständige Bytefolgen im Arbeitsspeicher. UI und Dienst führen getrennte Sequenzzustände; ein
Protokollfehler schließt im Modell das Paar. Die fünf
[Kanaltests](../../scripts/runtime-isolation/native-scene-model/src/channel/tests.rs) prüfen auch
den Verbrauch einer Nummer bei fachlichem Fehler sowie das Ende bei u64-Maximum. Reproduktion ab
Repositorywurzel:

```sh
cargo test --locked --offline --manifest-path scripts/runtime-isolation/native-scene-model/Cargo.toml
```

Die [Gastprüfung der Pipe-Rechte](../testing/native-win32-pipe-rights.md) bestätigt die
Endpunktzuordnung vor Resume. Der [native Startnachweis](../testing/native-win32-hello-ready.md)
bestätigt Bootstrap und HELLO/READY. Der [native Leseabruf](../testing/native-win32-scene-read.md)
bestätigt genau ein REQUEST/RESPONSE-Paar für A/read. Die
[Schreibabweisung unter Lesebindung](../testing/native-win32-denied-save.md) ist über drei
Sequenznummern im Gast bestätigt.
[Edit-Bindung und Konflikt](../testing/native-win32-edit-conflict.md) sind mit speicherresidentem
Katalog bestätigt. [SQLite und regulärer Prozessneustart](../testing/native-win32-durable-reopen.md)
sind im Gast geprüft. Offen sind Crash-Reconciliation, gezielte Teilübertragung, Timeout,
Fensterende, Wiederanlauf und ETW für die neuen Bytes.
