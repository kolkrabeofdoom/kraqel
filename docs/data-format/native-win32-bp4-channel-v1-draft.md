# BP4b · Lokaler Kanal für synthetische Win32-Szenen

**Spezifikation v1, KQ-069.** Dieses Protokoll bindet die drei
[BP4a-Operationen](native-win32-bp4-scene-v1-draft.md) an ein einmalig gestartetes Prozesspaar im
interaktiven Offline-Gast. Der bisherige Starter prüft ein UI-Blatt und startet es ohne
Handlevererbung. Zweiter Rollenstart und Endpunktrechte der suspendierten Rollen sind im Gast
geprüft. Produktprojekte bleiben gesperrt.
[ADR 0036](../architecture/decisions/0036-native-win32-ui-role-evaluation.md) ·
[Starter](../../scripts/runtime-isolation/native-window/src/bin/native-starter.rs)

## Abschluss des synthetischen Datenkanals

**BP4b ist für die geprüfte Win32-Forschungsprobe abgeschlossen.** Die Abnahme stützt sich auf die
vorhandenen Gastläufe.

| Eigenschaft                                                                                                                                                                     | Nachweis                                                                                                                                                                                                                                                  |
| ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Nur die gebundenen Rollen erhalten die vorgesehenen Pipe-Enden. Die Gegenstelle muss sich authentisieren.                                                                       | [Handle-Rechte](../testing/native-win32-pipe-rights.md), [HELLO/READY](../testing/native-win32-hello-ready.md)                                                                                                                                            |
| Die Lesebindung erlaubt einen Szenenabruf. Schreiben erfordert eine Edit-Bindung und die passende Revision.                                                                     | [Lesen](../testing/native-win32-scene-read.md), [Schreibabweisung](../testing/native-win32-denied-save.md), [Revisionskonflikt](../testing/native-win32-edit-conflict.md)                                                                                 |
| An der Commitgrenze bleibt genau die ursprüngliche oder die bestätigte Revision erhalten. Unvollständige Übertragungen und Fensterende führen zu keiner unbestätigten Änderung. | [SQLite-Wiederöffnung](../testing/native-win32-durable-reopen.md), [Commitgrenze](../testing/native-win32-commit-crashes.md), [Teilübertragung](../testing/native-win32-channel-interruptions.md), [Fensterende](../testing/native-win32-window-close.md) |
| Ohne gültigen Endebeleg bleibt die Ablage gesperrt. Eine verbrauchte Lesefreigabe überlebt Bereinigung und Kaltstart als Sperre.                                                | [Starterabbruch](../testing/native-win32-starter-restart.md), [Einmalfreigabe und ETW-Messgrenzen](../testing/native-win32-release-proof.md)                                                                                                              |

Die Kanalprüfungen verwendeten synthetische Katalogdaten. Tastaturbedienung und softwareseitige
Sprachausgabe sind für die [Fensterprobe](../testing/native-win32-window-guest.md) separat belegt.
Der bedienbare Editor mit explizitem Speichern über den gebundenen Kanal gehört zur offenen
C4c-Integration. Für reale Projekte und eine produktive Datenfreigabe fehlen weiterhin die
Integrationsnachweise.

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

### Freigabeentscheidung nach Starterabbruch

**Entscheidung für die vorhandene synthetische Ablage:** Sie bleibt gesperrt. Im
[gemessenen Abbruchlauf](../testing/native-win32-starter-restart.md) blieb `pending` zurück. Die
ursprünglichen Jobhandles sind verloren und der Transaktionsausgang ist ungeklärt. Die drei
abgewiesenen Neustarts und der ETW-Netzvergleich bestätigen die Sperre. Sie liefern keinen
Endebeleg.

**Stand der Implementierung.** Der
[Prüfstarter](../../scripts/runtime-isolation/native-window/src/bin/pipe-check.rs) schreibt unter
exklusiver Kontrollsperre vor Arbeitsbeginn `pending` und synchronisiert den Vermerk. Im regulären
Ablauf wartet er auf die ursprünglichen Prozesshandles, prüft beide gehaltenen Jobs auf Leerstand
und verlangt die Entfernung der Startinformationen. Erst dann schreibt und synchronisiert er
`drained`. Beide Vermerke enthalten den Hash des Paketankers. Dieser identifiziert weder einen
einzelnen Auftrag noch dessen SQLite-Ergebnis. Für diese vorhandene Ablage bleibt die Sperre
endgültig bestehen, solange der Endebeleg fehlt. Die neue Probe verwendet einen separaten,
generationsgebundenen Ablauf.

**Bedingungen für einen künftigen Freigabepfad.** Alle folgenden Nachweise müssen unter derselben
exklusiven Kontrollsperre zusammenpassen, bevor der Starter irgendeine Rolle startet oder SQLite
öffnet.

1. **Herkunft und Aktualität.** Ein lokal geschützter Startvermerk bindet vor dem ersten Rollenstart
   eine eindeutige Auftragsgeneration an Paket, Richtlinie, Sitzung und die konkrete synthetische
   Ablage. Der Endebeleg muss von der dafür vorgesehenen vertrauenswürdigen lokalen Kontrollinstanz
   stammen und exakt diesem offenen Auftrag entsprechen. Ein früherer Beleg desselben Pakets darf
   weder wiederverwendet noch durch Zurückspielen eines älteren Kontrollstands gültig werden.
2. **Ende beider Rollen.** Die Kontrollinstanz hat das Ende der ursprünglichen Prozessgenerationen
   und den Leerstand beider zugehörigen Jobs anhand der seit dem Start gehaltenen Kernelobjekte
   festgestellt. Die alten Kanalenden und Startfreigaben sind unwirksam. Soll der Beleg erst nach
   dem Starterabbruch entstehen, muss eine überlebende Kontrollinstanz diese Beobachtungsmöglichkeit
   bereits vor dem Rollenstart erhalten haben. Ein späterer Prozessscan, eine freie Dateisperre oder
   eine fehlende PID ersetzt sie nicht.
3. **Abgeschlossene Transaktion.** Ein dem Auftrag zugeordneter, dauerhafter Nachweis bestätigt
   entweder den erfolgten Commit samt Zielrevision oder den abgeschlossenen Rollback samt
   Ausgangsrevision. Eine Absichtserklärung vor Commit, eine ausgebliebene UI-Antwort und der
   Zustand einer Journaldatei reichen nicht aus. Ein Abbruch zwischen SQLite-Abschluss und
   dauerhafter Bestätigung bleibt ungeklärt. Auch zum Nachsehen bleibt die Datenbank in diesem
   Zustand geschlossen.
4. **Einmaliger Übergang.** Erst wenn alle Nachweise vollständig sind, darf die Kontrollinstanz den
   alten Auftrag dauerhaft abschließen und seine Freigaben verbrauchen. Der erste neue Zugriff
   erfolgt unter frischer Lesebindung mit neuen Pipes, Schlüsseln und Kanal-ID. Er muss die belegte
   Revision bestätigen. Bei Abweichung endet die Paarung gesperrt. Ein Schreibauftrag erfordert eine
   eigene neue Edit-Bindung. Ein unterbrochener oder widersprüchlicher Kontrollübergang erlaubt
   keinen Zugriff.

Für diesen Entscheid genügen lokale Zustandskennungen und gebundene Revisionen. Szenentext,
Schlüssel, Rohframes und externe Dienste sind dafür entbehrlich. Ein manuelles Löschen oder
Umschreiben des Sperrvermerks ist kein Freigabeverfahren.

**Gezielte Abnahme vor einer Implementierungsfreigabe.** Auf einer frischen synthetischen Ablage
genau einen passenden Abschluss zur einmaligen Lesefreigabe führen. Einen alten oder fremden
Abschluss sowie den Abbruch vor dauerhafter Abschlussbestätigung vor jedem Rollenstart abweisen. Bei
wiederholtem oder konkurrierendem Aufruf darf höchstens eine neue Lesepaarung entstehen. Die
[beiden Gastläufe mit lebender Kontrollinstanz](../testing/native-win32-release-proof.md) bestätigen
genau eine Lesebindung für Revision 1, die Ablehnung einer fremden Generation, den Verbrauch bei
Konkurrenz und Replay sowie die Sperre bei Abbruch vor dem Endebeleg.

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
sind im Gast geprüft. Der
[Datendienstabbruch vor und nach Commit](../testing/native-win32-commit-crashes.md) ist geprüft. Die
[Teilübertragung und die Empfangsfrist von 30 Sekunden](../testing/native-win32-channel-interruptions.md)
sind im Offline-Gast geprüft.
[Fensterende bei offener Transaktion und frischer ETW-Netzvergleich](../testing/native-win32-window-close.md)
sind geprüft. Der Netzlauf dieses Builds umfasst Teilübertragung, Timeout, Fensterende und die
zugehörigen Wiederöffnungen. Der
[Wiederanlauf nach Starterabbruch bei fehlendem Endebeleg](../testing/native-win32-starter-restart.md)
blieb für zwei Leseversuche und einen neuen Auftrag gesperrt. Der frische ETW-Netzvergleich
desselben Binärstands bestätigte drei gesperrte Starter ohne Kindprozess und null Netzwerkereignisse
im Prüfprozessbaum.
