# P2c C2: frischer ETW-Netzvergleich mit Wiederöffnen und Stilllegung

Stand: 17. November 2026. Gezieltes mittleres Prüfniveau, ausschließlich neue synthetische Ablagen.
[Bereinigte Aggregate](product-start-p2c-native-data-etw.json),
[funktionaler C2-Nachweis](product-start-p2c-native-data.md). **Keine Produktfreigabe**; striktes
Datenschutzziel und Produkt-Q-01/Q-02 bleiben unverändert.

## Programme und Messung

Unveränderter gesicherter C2-Controller aus Quellstand `e10429a`:
`73a7aad5e61a0ef7ebbc8cf8ccc75611c50ebd6bad3c52ad5f9fcc21ddc957b6`. Sicherung
`%TEMP%/kraqel-product-p2c-data-build-20261117-01`. Für die bestehende Auswertung byteidentische
Kopie mit Basisnamen `kraqel-isolation-probe.exe`; Hash vor und nach Messung geprüft. Keine
Runtime-/App-/Transport-/Paket-/Capabilityänderung und kein neuer Build.

Unveränderte [Erfassung](../../scripts/m3-network/capture.ps1) und
[Auswertung](../../scripts/m3-network/analyze.py), Modus `native-control`. Alle tatsächlichen
Root-PIDs und erwarteten EXE-Namen explizit erfasst; native Prozesshandles bis nach Konvertierung
gehalten. Keine automatische Rootsuche oder Lockerung der Regeln. Private lokale Prüfsteuerung
basiert auf dem [C2-Prüfablauf](../../scripts/runtime-isolation/check-workspace-persistent.ps1),
ergänzt um den kurzen Referenzpfad und Handlehaltung.

Nur die Erfassung läuft mit erhöhtem PowerShell-7-Token; Controller und Prüfsteuerung laufen normal.
Je vier eigene lokale TCP-/UDP-IPv4/IPv6-Kontrollen aus dem Capturehelfer, Frist höchstens fünf
Minuten je Sitzung. Nach fachlichem Abschluss per eigener STOP-Datei beendet: ungefähr 6,1 bzw. 4,6
Sekunden ab Ready. Beide eigenen Sitzungen gestoppt/konvertiert und Supervisor beendet. Keine
Firewall-, Loopback-, ExecutionPolicy- oder Domänenänderung.

## Geprüfte Abläufe und Ergebnis

Referenz: neuer Schema-6-DATA-Store → Vorbereitung/Profilbestätigung/Bindung → Verifikation →
explizite Stilllegung → bestätigter Stilllegungsstatus. Fünf Controller, sechs MAC-Records, ein
Profil angelegt und entfernt, unveränderte synthetische Ausgangsdaten erhalten.

Datenpfad: neuer Store → Vorbereitung → Verifikation → feste synthetische Bearbeitung → neues
Wiederöffnen/Verifikation. Geänderte Daten, Profil und Bindung bleiben erhalten. Alter Head und
erneute Vorbereitung werden in drei Aufrufen ohne Kontrollmutation abgewiesen. Anschließend
explizite Stilllegung und bestätigter Status; geänderte Daten und Schlüssel bleiben erhalten. Zehn
Controller, sechs MAC-Records, ein Profil angelegt und entfernt.

| Phase                              | Controller | Konsolenhosts | Explizite Enden | Kontrollen | Nicht-Loopback | Loopback |
| ---------------------------------- | ---------: | ------------: | --------------: | ---------: | -------------: | -------: |
| Referenz                           |          5 |             5 |              10 |        4/4 |              0 |        0 |
| Bearbeiten/Wiederöffnen/Stilllegen |         10 |            10 |              20 |        4/4 |              0 |        0 |
| Zusammen                           |     **15** |        **15** |          **30** |    **8/8** |          **0** |    **0** |

Beide Messungen gültig: null verlorene Events/Buffers, keine Zuordnungslücken oder unbekannten
Ziele. **Null beobachtete Netzereignisse in den ausgewählten Bäumen**, einschließlich Loopback. Alle
Konsolenhosts sind direkten Controllern zugeordnet. Keine Worker-, UI- oder WebView-Prozesse.
Private Controllerkopien in den Paketfixtures wurden als Dateien abgeglichen, nicht gestartet.

## Unabhängiger Gegencheck

Separater chronologischer Kernel-Audit: alle 15 erwarteten Aufrufe anhand Root-PID, Image,
Elternbezug und Exitcode zugeordnet; zwölf Erfolge und drei erwartete Abweisungen mit Exit 1. Alle
30 Prozessenden explizit belegt, Erzeugungszeit in Start/Ende exakt auf 100 ns abgeglichen. Kein
Ende allein aus später fehlender PID und kein fachlicher Erfolg allein aus Exit 0.

Unabhängiger lesender MAC-/Daten-/Inventaraudit bestätigt beide sechs Records langen Ketten,
Head-/Actor-/Rootbindung sowie vier Inventare samt tatsächlichem Paketinhalt. Beide eigenen
Profilmappings sind nach authentisierter Stilllegung abwesend. Referenztext beziehungsweise
geänderter Text und Schlüssel bleiben erhalten; echte Vorschau-Kontrollhashes unverändert. Keine
neue Brokerfreigabe an echten Daten und keine Bestandsmigration.

42 unveränderte Diagnose-Tests erneut bestanden; Hash-/Quellen-/Verweis-/Format-/Diffprüfung
ergänzt. Kein zusätzlicher Runtimebuild, keine erneute große Helfer-/Crashmatrix und kein
vollständiger Appcheck für diesen Nachweis. Die zuvor belegten 92 Helfertests bleiben der
funktionale C2-Stand und wurden hier nicht als neue Messung ausgegeben.

## Grenzen und Fortsetzung

Dieser Beleg gilt nur den exakten C2-Bytes und den genannten Datenpfaden. Native UI-ABI bleibt 0;
Schema-6-DATA-Bindung mit Ziel-ABI 1 ist kein ausführbarer Startgrant. Keine tatsächlichen
Lowbox-Dateizugriffe, Token-/Kindrollenprüfung, mehrprozessige GUI-Lebenszeit oder WebView-
Runtimepaketbindung mit diesem Lauf abgenommen. Normale App, P2 insgesamt, C3/C4, P3–P5 und
Produkt-Q-01/Q-02 bleiben offen.

ETW erfasst Transportmetadaten, keine HTTP-/Upload-/Inhaltsfreigaben. Externe COM-/Shell-/
Systemdienste und die Prüfsteuerung außerhalb der ausgewählten Bäume werden nicht pauschal
mitattestiert; keine physische Netztrennung oder universelle Netzstille zugesagt. Alte Netzbelege
behalten ihre eigenen Build-/Pfadgrenzen.

Rohdaten nur lokal unter `%TEMP%/kraqel-c2-etw-20261117-01-reference` und `…-data`, ergänzter Audit
beim gesicherten Build. Keine ETL/XML, PIDs, Rootpfade, Profil-IDs, Schlüssel oder Records auf
GitHub. Eigene Prozesse beendet und beide Testprofile stillgelegt; synthetische Daten/
Kontrollketten als lokale Nachweise erhalten. Nächster begrenzter Schritt: **C3-Wächter und
authentisierter Mehrprozess-Bootstrap mit synthetischen Kindprozessen**, ohne normale UI-Freigabe.
