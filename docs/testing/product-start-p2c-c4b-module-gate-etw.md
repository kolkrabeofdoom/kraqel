# P2c C4b: frischer ETW-Netzvergleich des Sperrpaars

Stand: 18. November 2026. **Gezielter Netzvergleich bestanden; keine Produktfreigabe.**
[Bereinigte Aggregate](product-start-p2c-c4b-module-gate-etw.json),
[Modulsperre und funktionaler Nachweis](product-start-p2c-c4b-module-gate.md). Mittleres Prüfniveau
und striktes Datenschutzziel unverändert.

## Exakte Programme und Messgrenze

Unverändertes gesichertes Paar aus `59d059e`:

- Roothelfer: `daf59d085588061b1daaabffe1ac24e76fb9c0303fd54e29b4f7231c5a72f5d5`.
- Tatsächlicher Host-Testcontroller:
  `83679e5311988c78b5f37feaf651bf9cda8638a58e554d95f536052cc45caf0c`.

Keine neuen Programme gebaut. Dateien vor und nach der Messung unabhängig gegen diese Hashes
geprüft; hashbenannte identische Kopien für die bestehende explizite Rootzuordnung. Keine
automatische Suche, keine erweiterten zulässigen EXE-Namen oder abgeschwächten Auswertungsregeln.
Der Roothelfer wird im nativen Test als `ui.exe` in die frische Kandidatenfixture kopiert.

Unveränderte [Erfassung](../../scripts/m3-network/capture.ps1) und
[Auswertung](../../scripts/m3-network/analyze.py), Modus `native-control`. Jede Host-PID mit
EXE-Namen und nativer Erzeugungszeit beim Start getrennt festgehalten, Hosthandles bis nach
Konvertierung gehalten. Nur die Erfassung mit erhöhtem Windows-Token; Prüfsteuerung, Host und Root
bleiben ohne zusätzlich erhöhtes Token. Keine Policy-/Firewall-/Loopback-/Registryänderung oder
Änderung an Schutzsoftware.

Je vier lokale positive TCP-/UDP-Kontrollen für IPv4/IPv6 aus dem Capturehelfer. Höchstens drei
Minuten je eigene Sitzung; nach tatsächlichem Ende per eigener STOP-Datei beendet. Messdauer ab
Ready bis Stop rund **0,5 und 35,5 Sekunden**. Keine fremden Systemerfassungen angefasst.

## Gezielte Abläufe und Ergebnis

Referenz: direkter Rootaufruf ohne Bootstrap, erwartetes Ende 2; danach genau ein reiner Test zur
vollständigen API-Generationsabdeckung im gesicherten Testcontroller. Der Testfilter wurde gegen die
tatsächliche Ausgabe „1 passed“ geprüft, kein erfolgreicher Nulltestlauf.

Sperrlauf: genau ein Aufruf des bereits gesicherten nativen Tests mit neuer synthetischer
Kontroll-/Daten-/Kandidatenfixture. Zunächst falsche Grant-MAC ohne Accepted/Continue, danach gültig
authentisierter Root mit fremdem Modul. Dieser endet vor Running/Continue mit
`c4-module-outside-policy`. **Kein WebView-Start und keine Runtimekindprozesse.** Beide zusätzlichen
Modulpfade durch dieselbe Policy abgewiesen; der HEVC-Pfad bleibt separat geprüft und wird wegen der
früheren Rootsperre nicht tatsächlich in einem neuen Runtimepfad geladen.

| Phase                                        | Hostaufrufe | Rootkinder | Konsolenhosts | Explizite Enden | Kontrollen | Nicht-Loopback | Loopback |
| -------------------------------------------- | ----------: | ---------: | ------------: | --------------: | ---------: | -------------: | -------: |
| Bootstrapabweisung und reiner Rollentest     |           2 |          0 |             2 |               4 |        4/4 |              0 |        0 |
| MAC-Abweisung und authentisierte Modulsperre |           1 |          2 |             1 |               4 |        4/4 |              0 |        0 |
| Zusammen                                     |       **3** |      **2** |         **3** |           **8** |    **8/8** |          **0** |    **0** |

Der direkte Rootaufruf der Referenz zählt als Hostaufruf. Alle drei Konsolenhosts gehören direkt zu
diesen expliziten Hosts; beide `ui.exe`-Kinder zum tatsächlichen Testcontroller. Keine weiteren
Worker oder Runtimeprozesse im ausgewählten Baum.

Beide Messungen gültig: null verlorene Events/Buffers, keine Zuordnungslücken und keine unbekannten
Ziele. **Null beobachtete Netzereignisse einschließlich Loopback in diesen Prozessbäumen.** Dies ist
keine universelle Netzfreiheit, Inhalts-/Uploadanalyse oder Zuordnung fremder COM-/Shell-/
Sicherheitsdienstprozesse. Der alte Beobachtungs-/ETW-Stand bleibt seinen eigenen Bytes zugeordnet.

## Unabhängiger Gegencheck und Bereinigung

Chronologischer Kernel-Audit bestätigt alle acht expliziten Enden. Drei Hoststartzeiten und beide
native Rootgenerationen exakt über PID/Erzeugungszeit auf 100 ns abgeglichen, jeweils mit den
richtigen Eltern und tatsächlichen Exitcodes. Beide Rootenden liegen vor dem Hostende. Alle durablen
Root-/Wächterbindungen passen zu diesen Kernelgenerationen; keine reine PID-Zuordnung.

Lesender unabhängiger HMAC-/Ketten-/Head-Audit: **13 MAC-Records, neun C4-Proberecords und zwei
verbrauchte Launches**. Zweimal Commit/RootBound/Drained/Finish; nur der gültige Root erhält
Accepted, keiner Running. MAC-Fehler endet ohne Accepted, Modulsperre nach Accepted. Native
Prozessenden und kontrolliertes Finish bestätigt; Auftragssperre wird nicht als Runtimefreigabe
umgedeutet. Tatsächliches Store-Wiederöffnen ohne Pending durch den gemessenen Test geprüft.

Eigenes Testprofil unabhängig als abwesend, eigene Kontroll-/Datenfixtures als entfernt geprüft.
Beide Erfassungen gestoppt und konvertiert, eigener Supervisor beendet. 27 unveränderte
Auswertungs-Tests erneut bestanden; frühere 119 Helfer-/23 C4-Tests sind keine neuen Testläufe.
Keine breite Crashmatrix, kein Helfer-/Appbuild und kein vollständiger Workspacecheck.

Private Messdaten unter `%TEMP%/kraqel-c4b-gate-etw-20261118-01-{reference,gate}/`, exaktes Paar und
neuer lesender Audit unter `%TEMP%/kraqel-c4b-module-gate-20261118-03/`. Ursprünglicher funktionaler
Lauf als `run-functional.json` erhalten, gemessener Lauf als `run.json`; getrennte MAC-Audits.
Geschützte zusätzliche Sicherung außerhalb von TEMP unter
`artifacts/c4b-module-gate-etw-20261118-01/`. Keys, PIDs, Kommandozeilen, absolute Prozesspfade und
ETL/XML bleiben lokal und von Git ausgeschlossen. Kein Audit wird als aktiver Store oder neuer
Startgrant übernommen; reale Projekte und echte Vorschauablage unverändert.

## Nächster begrenzter Scope

C4b: überprüfbaren Vertrag für eine **vor dem Laden wirksame Modul-/Rollenbindung** und sicheren
Abbruch bei nicht erzwingbarer Policy ausarbeiten. Die aktuelle Stichprobenprüfung findet bereits
geladene Module; sie verhindert deren frühere Ausführung nicht. Kein Kompatibilitätsfallback durch
Hersteller-/Signaturausnahmen, keine Abschaltung von Schutzsoftware. Effektive Datenrechte nach
Runtimebetrieb zusätzlich unabhängig abnehmen.

Der gemessene Abbruch ersetzt keinen kompatiblen Produktstart und keinen Nachweis des noch
gesperrten Runtimepfads. C4b insgesamt, C4c/d, automatische Gruppen-Recovery, P2/P3–P5 und
Produkt-Q-01/Q-02 offen; native UI-ABI bleibt 0. Keine neue Text-/Command-/Projektfreigabe.
